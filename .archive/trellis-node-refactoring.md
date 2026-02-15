# Plan: Grid-Aligned Node System Refactoring

## Context

The current Trellis renderer places nodes at arbitrary float coordinates (Sugiyama layout) and overlays a grid afterwards. The new spec (docs/trellis-grid-system.md) requires nodes to be first-class grid citizens: top-left corners on grid points, dimensions expressed as grid-point counts, and connectors at exact grid points on node edges.

This is a fundamental shift in the pipeline: cell_size must be known before node sizing, not after.

## Key Decisions

"width = N * cell_size" means N grid points along the edge, so pixel width = (N-1) * cell_size. Minimum 5-wide node = 4*cs pixels, with 3 connectors per horizontal edge.
Node.x/y switches to top-left coordinates (currently center-based).
cell_size comes from config (not computed from node dims). Keep calculate_cell_size as suggest_cell_size for backward compat.

## Implementation Phases

### Phase 1: Pipeline Reorder -- cell_size from config

Move cell_size to the front of the pipeline so it's available for node sizing.

**Files:**

* `crates/trellis-core/src/config.rs` -- Change cell_size: f64 to cell_size: i32, default = 10
* `crates/trellis-core/src/grid/params.rs` -- Rename calculate_cell_size to suggest_cell_size (kept for reference but not used in pipeline)
* `crates/trellis-core/src/pipeline.rs` -- Use config.cell_size instead of calling calculate_cell_size; pass cell_size to placement

**New pipeline order:**

```
parse -> cell_size from config -> snap_node_dimensions(cell_size) -> place_nodes(cell_size) -> grid_extent -> build_grid -> assign_connectors -> routing -> SVG
```

### Phase 2: Grid-Aligned Node Sizing

Add snap_node_dimensions_to_grid(graph, cell_size) that rounds node dimensions up to grid-point counts and enforces minimums.

**Files:**

* `crates/trellis-core/src/placement/mod.rs` -- Replace expand_nodes_for_ports with:

```rust
fn snap_node_dimensions_to_grid(graph: &mut Graph, cell_size: i32) {
    let cs = cell_size as f64;
    for node in &mut graph.nodes {
        // Round up to next grid-point count (N grid points = (N-1)*cs pixels)
        let w_points = ((node.width / cs).ceil() as i32 + 1).max(5);  // min 5 grid points
        let h_points = ((node.height / cs).ceil() as i32 + 1).max(3); // min 3 grid points
        node.width = (w_points - 1) as f64 * cs;
        node.height = (h_points - 1) as f64 * cs;
    }
}
```

**Tests:** verify all node dimensions produce correct grid-point counts, meet minimums

### Phase 3: Switch to Top-Left Coordinates

Change Node.x/y from center to top-left throughout. This is mechanical but touches many files.

**Files to update (all center→top-left conversions):**

* `crates/trellis-parser/src/ast.rs` -- Update doc comments on Node.x/y
* `crates/trellis-core/src/placement/sugiyama.rs` -- assign_coordinates():
    * Change node.x = offset + node.width / 2.0 → node.x = offset
    * Change node.y = layer * LAYER_SPACING (already top-left friendly for TB)
    * Normalization: n.x - n.width / 2.0 → n.x
* `crates/trellis-core/src/placement/snap.rs` -- Snap functions work with top-left
* `crates/trellis-core/src/grid/params.rs` -- calculate_grid_extent(): n.x - n.width/2.0 → n.x, n.x + n.width/2.0 → n.x + n.width
* `crates/trellis-core/src/grid/builder.rs` -- build_grid(): node.x - node.width/2.0 → node.x, etc.
* `crates/trellis-core/src/ports/assignment.rs` -- Port position calculations (center refs → top-left)
* `crates/trellis-core/src/render/nodes.rs` -- Remove left = cx - w/2, use node.x directly; compute center as node.x + w/2 for text
* `crates/trellis-core/src/render/svg.rs` -- calculate_viewbox(): same pattern
* `crates/trellis-core/src/render/edges.rs` -- Fallback edge rendering: use node.x + node.width/2 for center
* `crates/trellis-core/src/pipeline.rs` -- count_layers() uses n.x/n.y as-is
* Also snap top-left to grid points after Sugiyama placement:

```rust
fn snap_node_positions_to_grid(graph: &mut Graph, cell_size: i32) {
    let cs = cell_size as f64;
    for node in &mut graph.nodes {
        node.x = (node.x / cs).round() * cs;
        node.y = (node.y / cs).round() * cs;
    }
}
```

### Phase 4: Grid-Point Connector System (Replace Fractional Ports)

Replace the fractional port assignment with discrete grid-point connectors.

**Files:**

* `crates/trellis-core/src/ports/assignment.rs` -- Major rewrite:
    * Enumerate connectors: For node at grid position (gc, gr) with w_points wide, h_points tall:
    * Top: (gr, gc+1)...(gr, gc+w_points-2) → w_points-2 connectors
    * Bottom: (gr+h_points-1, gc+1)...(gr+h_points-1, gc+w_points-2)
    * Left: (gr+1, gc)...(gr+h_points-2, gc) → h_points-2 connectors
    * Right: (gr+1, gc+w_points-1)...(gr+h_points-2, gc+w_points-1)
    * For each edge: determine side via angle heuristic (keep angle_to_side), pick best connector on that side (closest to ideal position), mark as used
    * Keep Port struct but ensure grid_row/grid_col are exact integer grid coords
    * Keep EdgePorts struct interface so downstream routing is minimally affected

### Phase 5: Grid Building -- Boundary vs Interior Blocking

Refine build_grid to distinguish interior cells (blocked) from boundary cells (connectors, free for routing access) and corners (blocked).

**Files:**

* `crates/trellis-core/src/grid/builder.rs` -- New blocking logic:
    * Interior cells (strictly inside rectangle): CellState::Blocked
    * Corner cells (4 vertices): CellState::Blocked
    * Boundary cells (non-corner edge points): CellState::Free (connectors)

### Phase 6: Fix Remaining Files

* crates/trellis-core/src/deadlock/expand.rs -- Grid expansion must preserve grid-aligned node sizes
* crates/trellis-core/src/render/crossing.rs -- No fundamental change needed (uses grid_to_world)
* crates/trellis-core/src/lib.rs -- Update integration tests for new cell_size source and coordinate system
* All test files with hardcoded node positions/dimensions need updating

## Verification

**After each phase:**

```bash
cargo build -- must compile
cargo test --workspace -- fix failing tests
cargo clippy -- no errors
```

**End-to-end after all phases:**

```bash
cargo run -p trellis-cli -- render tests/benchmarks/fixtures/b01_linear_chain.mmd -o /tmp/b01.svg
```

**Visual inspection:** nodes should be grid-aligned rectangles, edges on grid lines, connectors at grid points
