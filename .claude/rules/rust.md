---
paths: ["**/*.rs", "**/Cargo.toml", "**/Cargo.lock"]
---

# Rust Best Practices — AI Agent Instruction File

> **Target:** Expert-level Rust projects  
> **Priority:** Separation of concerns · Small focused files · Correctness over convenience  
> **References:** [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) · [Rustonomicon](https://doc.rust-lang.org/nomicon/) · [Clippy lints](https://rust-lang.github.io/rust-clippy/)

---

## 1. Workspace & Project Structure

### Rule: Use Cargo workspaces for any non-trivial project
Split functionality into focused crates. Each crate has a single responsibility.

```
my-project/
├── Cargo.toml               # [workspace] manifest
├── crates/
│   ├── core/                # Pure domain logic — no I/O, no async
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── models/
│   │       └── error.rs
│   ├── infra/               # I/O, DB, HTTP — depends on core
│   │   └── src/
│   │       ├── lib.rs
│   │       └── db/
│   └── cli/                 # Binary entry point — depends on core + infra
│       └── src/
│           └── main.rs
```

### Rule: `core` crate must never depend on I/O crates
`core` depends only on `std`, `serde`, `thiserror`. No `tokio`, `sqlx`, `reqwest`. This makes it trivially testable.

### Rule: `main.rs` contains only wiring
```rust
// cli/src/main.rs
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cfg = infra::config::load()?;
    let db = infra::db::connect(&cfg.database_url).await?;
    cli::run(db, cfg).await
}
```

### Rule: One file per type, one module per concern
Keep files under 200 lines. If a module's `mod.rs` contains logic, extract it.

### Rule: Feature-flag optional dependencies
```toml
# Cargo.toml
[features]
default = []
metrics = ["dep:metrics", "dep:metrics-exporter-prometheus"]

[dependencies]
metrics = { version = "0.21", optional = true }
```

### Rule: Use `build.rs` only for code generation or compile-time checks
Never use `build.rs` for runtime logic. Document its purpose at the top of the file.

---

## 2. Error Handling

### Rule: Define a typed error enum per crate in `error.rs`
```rust
// core/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("entity not found: {entity} with id {id}")]
    NotFound { entity: &'static str, id: u64 },

    #[error("validation failed: {0}")]
    Validation(String),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}
```

### Rule: Never expose third-party error types in your public API
Wrap them. Leaking `sqlx::Error` or `reqwest::Error` couples consumers to your implementation.

```rust
// ✅ Good — implementation detail hidden
#[error("database unavailable")]
Database(#[from] sqlx::Error),

// ❌ Bad — consumer now depends on sqlx
pub fn get_user(id: u64) -> Result<User, sqlx::Error>
```

### Rule: Use `anyhow::Context` to attach call-site meaning
```rust
use anyhow::Context;

let user = db.find_user(id)
    .await
    .with_context(|| format!("failed to fetch user {id}"))?;
```

### Rule: Design error hierarchies across workspace crates
Each crate owns its error type. Higher-level crates wrap lower-level errors via `#[from]` or explicit `From` impls.

```
CoreError  ←  InfraError  ←  AppError
```

### Rule: Avoid `.unwrap()` entirely outside of tests and `const` contexts
Use `.expect("invariant: reason this cannot fail")` with a justification when a panic is truly unreachable.

---

## 3. Testing

### Rule: Unit tests live in the same file, always in a `tests` submodule
```rust
// core/src/models/user.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_username() {
        assert!(User::new("", 25).is_err());
    }
}
```

### Rule: Integration tests go in `tests/`, one file per feature boundary
```
crates/core/tests/
├── user_lifecycle.rs
└── billing_rules.rs
```

### Rule: Use `proptest` for invariant testing on domain logic
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn total_never_negative(amount in 0u64..1_000_000) {
        let order = Order::new(amount);
        prop_assert!(order.total() >= 0);
    }
}
```

### Rule: Use `criterion` for any performance-critical path
```rust
// benches/parsing.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_parse(c: &mut Criterion) {
    c.bench_function("parse_event", |b| {
        b.iter(|| parse_event(criterion::black_box(INPUT)))
    });
}

criterion_group!(benches, bench_parse);
criterion_main!(benches);
```

### Rule: Use `mockall` for dependency injection boundaries
Design traits first, mock them in tests. Never mock concrete types.

```rust
#[cfg_attr(test, mockall::automock)]
pub trait UserRepository {
    async fn find(&self, id: u64) -> Result<User, CoreError>;
}
```

### Rule: Name tests by observable behaviour
```rust
// ✅ Describes what the system does
fn returns_validation_error_when_email_is_missing()
fn charges_customer_only_once_on_retry()

// ❌ Describes implementation
fn test_create_user_calls_repo()
```

---

## 4. API Design

### Rule: Apply the newtype pattern to enforce domain invariants
```rust
pub struct EmailAddress(String);  // cannot be constructed with arbitrary strings

impl EmailAddress {
    pub fn parse(s: &str) -> Result<Self, CoreError> {
        // validation logic
    }
}
```

### Rule: Use the typestate pattern for compile-time state enforcement
```rust
pub struct Connection<State> { inner: tcp::Stream, _state: PhantomData<State> }
pub struct Unauthenticated;
pub struct Authenticated;

impl Connection<Unauthenticated> {
    pub fn authenticate(self, token: &str) -> Result<Connection<Authenticated>, AuthError> { ... }
}

impl Connection<Authenticated> {
    pub fn send(&mut self, msg: Message) -> Result<(), IoError> { ... }
}
```

### Rule: Seal traits that are not meant for external implementation
```rust
mod private { pub trait Sealed {} }

pub trait MyTrait: private::Sealed {
    fn method(&self);
}
```

### Rule: Mark public enums `#[non_exhaustive]`
```rust
#[non_exhaustive]
pub enum Status { Active, Inactive, Suspended }
```

### Rule: Prefer `impl Trait` in argument position; use `-> impl Trait` only when hiding complexity is justified
```rust
// ✅ Flexible input
pub fn process(items: impl Iterator<Item = Event>) { ... }

// ✅ Justified return-position opaque type
pub fn make_handler() -> impl Fn(Request) -> Response { ... }
```

---

## 5. Memory, Ownership & Unsafe

### Rule: Prefer borrowing over cloning in hot paths
```rust
// ✅ Zero-copy
fn log_name(user: &User) { tracing::info!(name = %user.name); }

// ❌ Unnecessary heap allocation
fn log_name(name: String) { tracing::info!(%name); }
```

### Rule: Choose the right shared-ownership primitive
| Scenario | Use |
|---|---|
| Single-threaded shared state | `Rc<RefCell<T>>` |
| Multi-threaded shared state | `Arc<Mutex<T>>` or `Arc<RwLock<T>>` |
| Message passing between tasks | `tokio::sync::mpsc` |
| Single writer, many readers | `Arc<RwLock<T>>` |

### Rule: Isolate all `unsafe` in dedicated modules with a safety contract
```rust
// safety/ffi.rs
/// # Safety
/// `ptr` must be non-null and point to a valid `T` aligned for its type.
/// Caller must ensure the pointed-to memory lives at least as long as `'a`.
pub unsafe fn deref_raw<'a, T>(ptr: *const T) -> &'a T {
    &*ptr
}
```

Never spread `unsafe` blocks across business logic files.

### Rule: Run Miri in CI for any crate containing `unsafe`
```yaml
# .github/workflows/ci.yml
- name: Miri
  run: cargo +nightly miri test
```

---

## 6. Async

### Rule: Keep async out of `core` — push it to the boundary layer
Pure domain logic should be synchronous. Async belongs in `infra` and `cli`.

### Rule: Never hold a `Mutex` lock across an `.await`
```rust
// ❌ Deadlock risk — Mutex held across await point
async fn bad(state: Arc<Mutex<State>>) {
    let lock = state.lock().unwrap();
    some_async_call().await;  // lock still held here
}

// ✅ Drop lock before awaiting
async fn good(state: Arc<Mutex<State>>) {
    let value = { state.lock().unwrap().get_value() };
    some_async_call(value).await;
}
```

### Rule: Prefer `tokio::sync` primitives over `std::sync` in async code
Use `tokio::sync::Mutex`, `tokio::sync::RwLock`, `tokio::sync::mpsc`.

### Rule: Annotate futures that must be cancellation-safe
```rust
/// This future is cancellation-safe: no state is partially written
/// unless the inner transaction commits.
async fn transfer_funds(...) -> Result<(), AppError> { ... }
```

---

## 7. Tooling & Enforcement

### Rule: Enforce these Clippy lint groups in `Cargo.toml`
```toml
[lints.clippy]
pedantic = "warn"
nursery = "warn"
unwrap_used = "deny"
expect_used = "warn"
panic = "warn"
```

### Rule: Mandatory CI checks
```yaml
steps:
  - run: cargo fmt --check
  - run: cargo clippy -- -D warnings
  - run: cargo test --all-features
  - run: cargo doc --no-deps --all-features
  - run: cargo audit                       # requires cargo-audit
  - run: cargo +nightly miri test          # for crates with unsafe
```

### Rule: Use `cargo doc` comments on every public item
```rust
/// Parses a raw string into a validated [`EmailAddress`].
///
/// # Errors
/// Returns [`CoreError::Validation`] if the input is not a valid RFC 5322 address.
pub fn parse(s: &str) -> Result<EmailAddress, CoreError> { ... }
```

---

## 8. Recommended Expert Crates

| Purpose | Crate |
|---|---|
| Typed errors (lib) | `thiserror` |
| Contextual errors (bin) | `anyhow` |
| Serialization | `serde`, `serde_json` |
| Async runtime | `tokio` |
| Tracing / structured logs | `tracing`, `tracing-subscriber` |
| Property-based testing | `proptest` |
| Benchmarking | `criterion` |
| Mocking | `mockall` |
| Fuzzing | `cargo-fuzz` |
| Security audits | `cargo-audit` |
| UB detection | `miri` |
