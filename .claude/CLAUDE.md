# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Trellis is a Mermaid-compatible diagram renderer in Rust that uses VLSI maze routing (A* pathfinding on a grid) to produce overlap-free orthogonal edge layouts. It supports flowchart, class, ER, and C4 diagram types.

## Specification Documents

Before implementing any feature, read the relevant specs:
- `./docs/trellis-spec.md` — full algorithm spec (Hungarian)
- `./docs/trellis-project-structure.md` — crate and directory layout
- `./docs/trellis-grid-system.md` — grid coordinate system and node/edge snapping rules

## Common Commands

```bash
# Build
cargo build --workspace

# Run all tests
cargo test --workspace

# Run a single test by name
cargo test --workspace -- test_name

# Run tests in a specific crate
cargo test -p trellis-parser
cargo test -p trellis-core

# Static analysis (fix errors; ask user before fixing warnings)
cargo clippy --workspace

# Render a diagram (smoke test)
cargo run -p trellis-cli -- render tests/benchmarks/fixtures/b01.mmd -o test.svg

# Validate a diagram
cargo run -p trellis-cli -- validate input.mmd

# Batch render
cargo run -p trellis-cli -- render-batch ./input-dir -o ./output-dir

# Run benchmarks
cargo bench -p benchmarks

# Build WASM (outputs to editors/vscode/wasm/ and editors/intellij/.../wasm/)
./scripts/build-wasm.sh [--release]
```

## Architecture

### Workspace Crates

- **trellis-parser** — Mermaid syntax parser (nom-based). Tokenizer detects diagram type, dispatches to `flowchart.rs`, `class_diagram.rs`, `er_diagram.rs`, or `c4_diagram.rs`. Produces a `Graph` AST (`ast.rs`).
- **trellis-core** — Rendering pipeline: placement → grid → ports → routing → deadlock → labels → SVG. Has optional `png` feature (default=on, disabled for WASM via `default-features = false`).
- **trellis-cli** — CLI binary (`trellis`). Commands: `render`, `render-batch`, `validate`, `license`. Supports `--config` TOML, stdin input, Pandoc Lua filter.
- **trellis-wasm** — wasm-bindgen API for editor plugins.

### Rendering Pipeline (`trellis-core/src/pipeline.rs`)

`render(graph, config, format)` runs these phases sequentially:
1. **Placement** — Sugiyama (flowchart), class-specific, force-directed (ER), row-flow (C4). Dispatched in `placement/mod.rs` by `DiagramType`.
2. **Grid** — Builds a cell grid around placed nodes; cell size is uniform.
3. **Ports** — Assigns connector points on node edges (grid-aligned, excluding corners).
4. **Routing** — Priority scoring → A* pathfinding → path commitment. Occupied cells block subsequent edges (overlap-free by construction).
5. **Deadlock** — Rip-up-and-reroute / grid expansion / crossing fallback.
6. **Labels** — Edge label placement with collision detection.
7. **Render** — SVG generation, dispatched by diagram type for node shapes and arrow markers.

### Diagram-Type Dispatch Pattern

Most modules dispatch on `DiagramType` (Flowchart, ClassDiagram, ErDiagram, C4Diagram). When adding a new diagram type, update: parser `lib.rs`, `placement/mod.rs`, `render/svg.rs`, and add shape/marker modules.

### AST Conventions (`trellis-parser/src/ast.rs`)

- `Node` and `Edge` derive `Default` — always use `..Default::default()` when constructing.
- Diagram-specific fields are `Option` types on the shared `Node`/`Edge` structs (e.g., `c4_type`, `er_attributes`, `class_methods`).

### WASM Considerations

- `std::time::Instant` is unavailable on `wasm32`; guarded with `#[cfg(not(target_arch = "wasm32"))]` in `pipeline.rs`.
- `trellis-core` uses `default-features = false` in the WASM crate to exclude `resvg`/PNG.

## Quality Checks (Required Before Finishing Work)

1. **Compile and test**: `cargo test --workspace` — fix all compile errors (stop after 3 failed attempts and ask for help).
2. **Clippy**: `cargo clippy --workspace` — fix all **errors**. Ask the user before fixing **warnings**.
3. **Mark done**: If implementing from the implementation plan, mark completed items `[x]`.

## Test Structure

- **Unit tests**: Inline `#[cfg(test)]` modules within each crate.
- **Integration tests**: `tests/integration/` — pipeline end-to-end tests, routing invariant tests.
- **Benchmarks**: `tests/benchmarks/` — criterion benchmarks using fixtures in `tests/benchmarks/fixtures/` (b01–b12.mmd).
