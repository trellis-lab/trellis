# Pluggable Port Assignment Strategies

## Goal

Introduce a trait-based abstraction so multiple port assignment algorithms can coexist,
selected via the existing TOML/JSON configuration system. The current angle-based algorithm
becomes the default. No existing behaviour changes.

---

## Current State

### Entry point

```
crates/trellis-core/src/ports/assignment.rs
```

Free function:

```rust
pub fn assign_ports(
    graph: &Graph,
    cell_size: i32,
    offset_x: i32,
    offset_y: i32,
) -> HashMap<usize, EdgePorts>
```

### Public types (from `ports/mod.rs`)

```rust
pub use assignment::{assign_ports, EdgePorts, Port, Side};
```

### Pipeline call site (`pipeline.rs:57`)

```rust
let port_assignments = assign_ports(&graph, cell_size, extent.offset_x, extent.offset_y);
```

### Config (`config.rs`)

- `TrellisConfig` — serde-based, all fields have `#[serde(default)]`
- Enum pattern already used: `DecompositionMode { None, Single, Multi }` with `#[default]`
- Factory: `configuration_factory(ConfigurationType) -> TrellisConfig`

---

## Implementation Steps

### Step 1: Add `PortAssignmentStrategy` enum to `config.rs`

Add after the `DecompositionMode` enum (around line 127):

```rust
/// Port assignment algorithm selection
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum PortAssignmentStrategy {
    #[default]
    Default,
    // Future variants:
    // Centered,
    // Minimal,
}
```

Add default function (follow existing pattern):

```rust
fn default_port_assignment() -> PortAssignmentStrategy {
    PortAssignmentStrategy::Default
}
```

Add field to `TrellisConfig` struct:

```rust
/// Port assignment algorithm
#[serde(default = "default_port_assignment")]
pub port_assignment: PortAssignmentStrategy,
```

Update `Default` impl for `TrellisConfig`:

```rust
port_assignment: default_port_assignment(),
```

Update `configuration_factory` `Benchmark` variant to include the new field:

```rust
port_assignment: PortAssignmentStrategy::Default,
```

### Step 2: Define the trait and context in `ports/mod.rs`

Replace the current content of `crates/trellis-core/src/ports/mod.rs`:

```rust
pub mod assignment;

use std::collections::HashMap;
use trellis_parser::Graph;

pub use assignment::{assign_ports, DefaultPortAssigner, EdgePorts, Port, Side};

use crate::config::PortAssignmentStrategy;

/// Context provided to port assignment strategies.
///
/// Bundled as a struct so future strategies can receive additional data
/// (e.g. Grid, diagram direction) without breaking the trait signature.
pub struct PortAssignmentContext<'a> {
    pub graph: &'a Graph,
    pub cell_size: i32,
    pub offset_x: i32,
    pub offset_y: i32,
}

/// Trait for port assignment strategies.
///
/// Each implementation assigns source and target connection points (ports)
/// to all edges in a graph, returning a map from edge index to port pair.
pub trait PortAssigner {
    fn assign_ports(&self, ctx: &PortAssignmentContext) -> HashMap<usize, EdgePorts>;
}

/// Resolve a config enum value to a concrete port assigner.
pub fn create_port_assigner(strategy: PortAssignmentStrategy) -> Box<dyn PortAssigner> {
    match strategy {
        PortAssignmentStrategy::Default => Box::new(DefaultPortAssigner),
    }
}
```

### Step 3: Wrap the existing algorithm in `ports/assignment.rs`

Keep all existing private functions (`calculate_angle`, `handle_overflow`, `enumerate_connectors`,
`sort_edges_on_side`, `angle_to_side`, etc.) unchanged.

Add a struct that implements the trait:

```rust
use super::{PortAssigner, PortAssignmentContext};

/// The default (angle-based) port assignment algorithm.
///
/// For each node, edges are grouped by side (based on the angle to the connected node),
/// overflow is handled by spilling to clockwise neighbours, edges are sorted within
/// each side by perpendicular axis, and connector positions are evenly distributed.
pub struct DefaultPortAssigner;

impl PortAssigner for DefaultPortAssigner {
    fn assign_ports(&self, ctx: &PortAssignmentContext) -> HashMap<usize, EdgePorts> {
        assign_ports(ctx.graph, ctx.cell_size, ctx.offset_x, ctx.offset_y)
    }
}
```

The existing free function `pub fn assign_ports(...)` stays as-is. This preserves
backward compatibility for all existing call sites and unit tests.

### Step 4: Update the pipeline (`pipeline.rs`)

Change the import (line 5):

```rust
// Before
use crate::ports::assign_ports;

// After
use crate::ports::{create_port_assigner, PortAssignmentContext};
```

Change the call site (line 57):

```rust
// Before
let port_assignments = assign_ports(&graph, cell_size, extent.offset_x, extent.offset_y);

// After
let assigner = create_port_assigner(config.port_assignment);
let port_ctx = PortAssignmentContext {
    graph: &graph,
    cell_size,
    offset_x: extent.offset_x,
    offset_y: extent.offset_y,
};
let port_assignments = assigner.assign_ports(&port_ctx);
```

### Step 5: Tests

#### Existing tests — no changes

All unit tests in `assignment.rs` call the free function directly and remain unaffected.

#### New tests to add

**In `ports/mod.rs` (or a new `ports/tests.rs`):**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use trellis_parser::{Graph, Node, Edge, NodeShape, ArrowHead, EdgeStyle};

    #[test]
    fn default_assigner_matches_free_function() {
        let mut graph = Graph::new();
        graph.nodes = vec![
            Node { id: "A".into(), label: "A".into(), shape: NodeShape::Rectangle,
                   width: 40.0, height: 20.0, x: 60.0, y: 30.0, ..Default::default() },
            Node { id: "B".into(), label: "B".into(), shape: NodeShape::Rectangle,
                   width: 40.0, height: 20.0, x: 60.0, y: 130.0, ..Default::default() },
        ];
        graph.edges = vec![Edge {
            from: "A".into(), to: "B".into(), label: None,
            style: EdgeStyle::Solid, arrow_head: ArrowHead::Arrow,
            ..Default::default()
        }];

        let free_fn_result = assign_ports(&graph, 10, 0, 0);

        let assigner = DefaultPortAssigner;
        let ctx = PortAssignmentContext {
            graph: &graph, cell_size: 10, offset_x: 0, offset_y: 0,
        };
        let trait_result = assigner.assign_ports(&ctx);

        assert_eq!(free_fn_result.len(), trait_result.len());
        for (idx, expected) in &free_fn_result {
            let actual = &trait_result[idx];
            assert_eq!(expected.source_port.grid_row, actual.source_port.grid_row);
            assert_eq!(expected.source_port.grid_col, actual.source_port.grid_col);
            assert_eq!(expected.target_port.grid_row, actual.target_port.grid_row);
            assert_eq!(expected.target_port.grid_col, actual.target_port.grid_col);
        }
    }
}
```

**In `config.rs`:**

```rust
#[test]
fn port_assignment_defaults_when_absent() {
    let config: TrellisConfig = toml::from_str("").unwrap();
    assert!(matches!(config.port_assignment, PortAssignmentStrategy::Default));
}

#[test]
fn port_assignment_parses_from_toml() {
    let config: TrellisConfig = toml::from_str(r#"port_assignment = "Default""#).unwrap();
    assert!(matches!(config.port_assignment, PortAssignmentStrategy::Default));
}
```

---

## File Change Summary

| File | Action |
|---|---|
| `crates/trellis-core/src/config.rs` | Add `PortAssignmentStrategy` enum, default fn, field in `TrellisConfig`, update `Default` impl and `configuration_factory` |
| `crates/trellis-core/src/ports/mod.rs` | Add `PortAssignmentContext`, `PortAssigner` trait, `create_port_assigner` factory, update exports |
| `crates/trellis-core/src/ports/assignment.rs` | Add `DefaultPortAssigner` struct + `impl PortAssigner`. No logic changes |
| `crates/trellis-core/src/pipeline.rs` | Change import and call site (lines 5, 57) |

---

## Adding a New Strategy (Future Pattern)

When someone wants to add a new algorithm (e.g. `Centered`):

1. **`config.rs`** — Add `Centered` variant to `PortAssignmentStrategy`
2. **`ports/centered.rs`** — Create file with `pub struct CenteredPortAssigner;` implementing `PortAssigner`
3. **`ports/mod.rs`** — Add `pub mod centered;` and a match arm in `create_port_assigner`

No pipeline, CLI, or WASM changes needed.

TOML usage:

```toml
port_assignment = "Centered"
```

JSON (WASM bridge):

```json
{ "port_assignment": "Centered" }
```

---

## Backward Compatibility Checklist

| Concern | Status |
|---|---|
| Free function `assign_ports()` | Kept as convenience wrapper |
| Existing unit tests | Unchanged — call free function directly |
| Integration tests | Unchanged — use `TrellisConfig::default()` which selects `Default` |
| Benchmarks | Unchanged — `configuration_factory(Benchmark)` includes new field |
| TOML configs without `port_assignment` | `#[serde(default)]` resolves to `Default` |
| WASM API | Unchanged — config JSON without the field works |
| Public types (`Port`, `EdgePorts`, `Side`) | Unchanged |

---

## Design Notes

### Why `Box<dyn PortAssigner>` over enum dispatch?

- Simpler, idiomatic Rust for strategy pattern
- One vtable call per `render()` invocation — negligible cost
- Object-safe: trait has no generics, no `Self` in return types
- If profiling ever shows this matters, switch to enum dispatch internally without changing the public API

### Why `PortAssignmentContext` struct instead of bare parameters?

- Future strategies may need additional data (e.g. `Grid`, `DiagramType`, layout direction)
- Adding fields to the context is non-breaking — it's constructed inside the pipeline
- Avoids growing the trait method signature over time

### Thread safety

- The pipeline does not send the assigner across threads currently
- If `render_batch` ever parallelizes at the port-assignment level, add `Send + Sync` bounds:
  `pub trait PortAssigner: Send + Sync { ... }`
- `DefaultPortAssigner` is a unit struct — it's `Send + Sync` by default
