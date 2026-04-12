# Strategy: Dual-Grid Rendering (Design Grid + Render Grid)

**Status:** Draft  
**Date:** 2026-04-12  
**Scope:** SVG edge rendering, crossing hops, bend smoothing

---

## Problem Statement

Current crossing hops are rendered as full circles at grid intersection points (see `render/crossing.rs`). This produces visually poor results:

1. **Circle-based hops** — a white circle + arc overlay at the crossing cell center. This doesn't look like a proper "hop" (semicircular arc that one edge jumps over another). It's just a circle sitting on top of both edges.

2. **No arc integration into edge path** — the hop arc is a separate SVG element, not part of the edge's `<path>` data. The edge line passes straight through the crossing point underneath the white circle. The illusion breaks at zoom or with transparency.

3. **Grid-locked coordinates** — all edge vertices snap to integer grid points. There's no sub-grid precision for arc entry/exit points, so hops can't start/end at half-cell offsets.

4. **Overlay-only approach** — bends use quadratic Bezier approximations at grid-snapped corners. There's no concept of mid-points between grid nodes for smoother geometry.

5. **Stale crossings after reroute (BUG)** — The reroute phases (quality_reroute, crossing_reroute, rip_up_and_reroute) do not update crossing metadata on the grid. When an edge is uncommitted and rerouted to a new path, the old crossing cells retain `crossing = true` (phantom hops) and new shared cells never get `crossing = true` (missing hops). This produces incorrect hop rendering regardless of the rendering approach.

---

## Bug Fix: Crossing Metadata Staleness

### Root Cause

Three reroute modules share the same flaw. The cycle is:

1. `uncommit_path()` clears `cell.owner` and resets state to `Free` — but **only for cells where `cell.owner == edge_id`**.
2. At crossing cells (where two edges share a cell), the uncommitted edge is either the `owner` or the `crossed_by`. Two sub-cases:
   - **Edge is `owner`**: cell gets freed, `owner` cleared. But `crossing` stays `true` and `crossed_by` still points to the other edge. The other edge still occupies this cell, yet it's now marked Free — corrupt state.
   - **Edge is `crossed_by`**: `uncommit_path` doesn't match `cell.owner`, so the cell is **not freed at all**. The `crossing` flag and `crossed_by` field remain stale.
3. `commit_path()` on the new path sets `crossing = true` only when a cell is already `Occupied`. If the new path avoids the old crossing point, no cleanup happens. If it crosses a different edge at a new point, that's handled correctly — but only for that one cell.

Net result: after any reroute, `cell.crossing` flags on the grid are unreliable. The renderer trusts these flags, producing phantom arcs and missing arcs.

### Affected Modules

| Module | File | How it triggers the bug |
|--------|------|------------------------|
| Quality reroute | `routing/quality_reroute.rs` | `uncommit_path()` → try 16 combos → `commit_path()` best. Crossing flags from old path linger. |
| Crossing reroute | `routing/crossing_reroute.rs` | Same pattern. Ironically, this module exists to *reduce* crossings but leaves stale crossing metadata. |
| Rip-up-and-reroute | `deadlock/rip_up.rs` | Uncommits multiple blocking edges, reroutes failed edge, re-commits blocking edges. Crossing flags from all uncommitted paths go stale. |

### Fix Strategy: Crossing-Aware Uncommit + Post-Reroute Reconciliation

Two-pronged approach — fix `uncommit_path()` itself, and add a safety-net reconciliation pass.

#### 1. Fix `uncommit_path()` to handle crossing cells

Current code (commit.rs:60-65):
```rust
if cell.state == CellState::Occupied && cell.owner.as_deref() == Some(edge_id) {
    cell.state = CellState::Free;
    cell.owner = None;
    cell.cost = costs.base_cost;
}
```

New logic must handle both roles at a crossing:

```rust
if cell.owner.as_deref() == Some(edge_id) {
    if cell.crossing {
        // This edge was the first occupant. The crossed_by edge still occupies.
        // Promote crossed_by to owner, clear crossing state.
        cell.owner = cell.crossed_by.take();
        cell.crossing = false;
        // Cell stays Occupied — the other edge still uses it.
    } else {
        // Normal non-crossing cell: free it.
        cell.state = CellState::Free;
        cell.owner = None;
        cell.cost = costs.base_cost;
    }
} else if cell.crossed_by.as_deref() == Some(edge_id) {
    // This edge was the second occupant. Owner stays, just clear crossing.
    cell.crossed_by = None;
    cell.crossing = false;
    // Cell stays Occupied under the original owner.
}
```

This ensures:
- Removing the first edge at a crossing promotes the second edge to sole owner.
- Removing the second edge at a crossing clears the crossing flag, owner keeps the cell.
- No cell is left with `crossing = true` when only one edge occupies it.

#### 2. Post-reroute crossing reconciliation

After any reroute pass completes (quality_reroute, crossing_reroute, rip_up_and_reroute), run a lightweight reconciliation that re-derives crossings from the authoritative source: the `paths` HashMap.

```rust
pub fn reconcile_crossings(
    grid: &mut Grid,
    paths: &HashMap<usize, RoutedPath>,
) {
    // Step 1: Clear all crossing metadata on the grid.
    for row in 0..grid.rows {
        for col in 0..grid.cols {
            if let Some(cell) = grid.get_mut(row, col) {
                cell.crossing = false;
                cell.crossed_by = None;
            }
        }
    }

    // Step 2: Rebuild from paths.
    // For each cell, track which edges pass through it.
    let mut cell_edges: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (&edge_idx, path) in paths {
        for pt in &path.points {
            cell_edges
                .entry((pt.row as usize, pt.col as usize))
                .or_default()
                .push(edge_idx);
        }
    }

    // Step 3: Where 2+ edges share a cell, set crossing flags.
    for ((row, col), edges) in &cell_edges {
        if edges.len() >= 2 {
            if let Some(cell) = grid.get_mut(*row, *col) {
                cell.crossing = true;
                // Owner is the first edge (by index — matches commit order).
                // crossed_by is the second.
                let owner_idx = edges.iter().min().unwrap();
                let crosser_idx = edges.iter().filter(|&&i| i != *owner_idx).min().unwrap();
                cell.owner = Some(format!("edge_{}", owner_idx));
                cell.crossed_by = Some(format!("edge_{}", crosser_idx));
            }
        }
    }
}
```

#### 3. Where to call reconciliation

Insert `reconcile_crossings()` at two points in the pipeline:

1. **After Phase 5a-5d** (after all reroute passes complete, before deadlock resolver) — ensures deadlock resolver sees accurate crossing state.
2. **After deadlock resolver** (Phase 6) — deadlock's rip-up-and-reroute may introduce new crossings.

Alternatively, a single call after the final routing mutation (after Phase 6, before Phase 7 labels) is sufficient if crossing metadata is only consumed by the renderer.

#### 4. Impact on crossing data extraction (Phase 2 of render-grid strategy)

The `crossing_points` HashMap planned in Phase 2 should be **derived from the reconciled grid state** or directly from the `paths` HashMap (comparing point sets), not from `commit_path()` return values. This makes it immune to any future reroute bugs.

Recommended: compute `crossing_points` as a post-routing step, not during commit:

```rust
pub fn compute_crossing_points(
    paths: &HashMap<usize, RoutedPath>,
) -> HashMap<usize, Vec<(i64, i64)>> {
    // Build cell → edge set map
    let mut cell_edges: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (&idx, path) in paths {
        for pt in &path.points {
            cell_edges.entry((pt.row, pt.col)).or_default().push(idx);
        }
    }

    // For each edge, collect cells where another edge also passes
    let mut result: HashMap<usize, Vec<(i64, i64)>> = HashMap::new();
    for (&idx, path) in paths {
        let crossings: Vec<(i64, i64)> = path
            .points
            .iter()
            .filter(|pt| {
                cell_edges
                    .get(&(pt.row, pt.col))
                    .map(|edges| edges.len() >= 2)
                    .unwrap_or(false)
            })
            .map(|pt| (pt.row, pt.col))
            .collect();
        if !crossings.is_empty() {
            result.insert(idx, crossings);
        }
    }
    result
}
```

This is the **single source of truth** for crossing data — derived from final paths, not grid cell flags.

---

## Proposed Solution: Dual-Grid System

### Core Idea

Introduce a **render grid** with **double the resolution** of the design grid (routing grid). Every design grid point at `(row, col)` maps to render grid point `(row*2, col*2)`. Between every pair of adjacent design points, there's a **mid-point** at odd render-grid coordinates.

```
Design grid:     (0,0)  (0,1)  (0,2)
                   |      |      |
Render grid:  (0,0) (0,1) (0,2) (0,3) (0,4)
                     ↑           ↑
                  mid-point   mid-point
```

The routing algorithm continues to work on the **design grid** (integer coordinates, A* pathfinding, cell occupancy). The render grid is a **coordinate transformation layer** used only during SVG generation.

### Key Properties

| Aspect | Design Grid | Render Grid |
|--------|-------------|-------------|
| Purpose | Routing, pathfinding, collision | SVG coordinate generation |
| Resolution | `cell_size` px between points | `cell_size / 2` px between points |
| Coordinate space | Integer `(row, col)` | Integer `(row*2, col*2)` for design points; `(row*2±1, col*2±1)` for mid-points |
| Used by | A*, commit, deadlock, ports | Edge path builder, hop arcs |
| Grid structure | `Grid` (cells, states, costs) | Pure coordinate math — no new grid allocation |

The render grid is **not a stored data structure**. It's a coordinate mapping:

```rust
fn design_to_render(row: i64, col: i64) -> (i64, i64) {
    (row * 2, col * 2)
}

fn render_to_world(grid: &Grid, rrow: i64, rcol: i64) -> (f64, f64) {
    let x = rcol as f64 * (grid.cell_size as f64 / 2.0) + grid.offset_x as f64;
    let y = rrow as f64 * (grid.cell_size as f64 / 2.0) + grid.offset_y as f64;
    (x, y)
}

fn mid_point(a: (i64, i64), b: (i64, i64)) -> (i64, i64) {
    // In render-grid space, the midpoint between two adjacent design points
    ((a.0 + b.0) / 2, (a.1 + b.1) / 2)
}
```

---

## Edge Path Model

### Current Model

An edge is a single SVG `<path>` with:
- `M` (move to start)
- `L` (line segments between simplified bend points)
- `Q` (quadratic Bezier at corners for rounding)

Crossings are separate `<circle>` elements drawn on top.

### New Model: Structured Edge Segments

An edge path becomes a **sequence of typed segments**:

```rust
enum EdgeSegment {
    /// Straight line between two render-grid points
    Line { from: RenderPoint, to: RenderPoint },
    
    /// Rounded bend (quadratic Bezier)
    Bend {
        entry: RenderPoint,      // line ends here
        control: RenderPoint,    // grid corner (Bezier control point)
        exit: RenderPoint,       // next line starts here
    },
    
    /// Crossing hop arc (semicircular arc over another edge)
    Hop {
        entry: RenderPoint,      // mid-point before crossing
        apex: RenderPoint,       // crossing point (design grid)
        exit: RenderPoint,       // mid-point after crossing
    },
    
    /// Arrowhead or marker at path endpoint
    Marker { position: RenderPoint, direction: (f64, f64) },
}
```

### How Hops Work

When edge path passes through a crossing cell at design-grid point `P`:

1. Find the two adjacent design-grid points `A` (before crossing) and `B` (after crossing).
2. Compute mid-points in render-grid space:
   - `entry = midpoint(A, P)` — halfway between previous point and crossing
   - `exit = midpoint(P, B)` — halfway between crossing and next point
3. Generate a semicircular SVG arc from `entry` to `exit`, with the arc bulging perpendicular to the edge direction.

```
Before (current):
    A ————————— P ————————— B
                ⬤ (white circle overlay)

After (new):
    A ———— entry ⌒ exit ———— B
              ↑   arc   ↑
           mid-point  mid-point
```

SVG arc command:

```
L entry_x entry_y A rx ry 0 0 1 exit_x exit_y L ...
```

Where `rx = ry = cell_size / 2` (half-cell radius), sweep flag determines arc direction (always hop "over" — away from the other edge's direction).

### Hop Direction Detection

The arc must bulge in the correct direction. At a crossing, two edges intersect orthogonally (guaranteed by the grid routing). The hopping edge's arc should bulge perpendicular to the *other* edge's direction:

- If hopping edge travels horizontally and crossed edge travels vertically → arc bulges upward (or downward, configurable)
- If hopping edge travels vertically and crossed edge travels horizontally → arc bulges left (or right)

The `cell.owner` and `cell.crossed_by` fields already identify which edge was routed first. Convention: the **second edge** (`crossed_by`) hops over the **first edge** (`owner`). This matches the visual metaphor — the later-routed edge "jumps over" the earlier one.

---

## SVG Path Generation

### Current Flow

```
GridPoints → grid_to_world() → simplify_path() → generate_rounded_polyline() → SVG <path d="...">
```

### New Flow

```
GridPoints + CrossingInfo
    ↓
build_edge_segments()          // classify each vertex: line / bend / hop
    ↓
Vec<EdgeSegment>
    ↓
segments_to_svg_path()         // render each segment to SVG commands
    ↓
SVG <path d="M ... L ... Q ... A ... L ...">
```

### `build_edge_segments()` Algorithm

```
Input: simplified world points[], crossing_points set, grid reference
Output: Vec<EdgeSegment>

for each point p[i] in simplified path:
    if p[i] is a crossing point:
        prev = p[i-1] (in render-grid coords)
        curr = p[i]   (in render-grid coords)  
        next = p[i+1] (in render-grid coords)
        
        entry_mid = midpoint(prev_render, curr_render)
        exit_mid  = midpoint(curr_render, next_render)
        
        emit Line { to: entry_mid }
        emit Hop  { entry: entry_mid, apex: curr, exit: exit_mid }
        // next Line will start from exit_mid
        
    else if direction changes at p[i] (bend):
        compute entry/exit with corner_radius (same as current Q logic)
        emit Bend { entry, control: p[i], exit }
        
    else:
        emit Line { to: p[i] }
```

### SVG Command Mapping

| Segment | SVG Commands |
|---------|-------------|
| `Line` | `L x y` |
| `Bend` | `L entry_x entry_y Q ctrl_x ctrl_y exit_x exit_y` |
| `Hop` | `L entry_x entry_y A r r 0 0 sweep exit_x exit_y` |
| `Marker` | handled via `marker-start` / `marker-end` attributes (unchanged) |

---

## What Gets Removed

1. **`render/crossing.rs`** — entire file. No more overlay circles. Hops are now part of edge paths.
2. **Crossing SVG layer in `svg.rs`** — the `<!-- Crossings -->` group and `render_crossings()` call.
3. **`config.render_crossings`** — no longer needed as a toggle (hops are always rendered as arcs when crossings exist). Could repurpose as `render_hops: bool` if needed.

---

## What Gets Modified

### `render/edges.rs`

- Add `RenderPoint` struct (or reuse `Point` with render-grid precision).
- Add `EdgeSegment` enum.
- New `build_edge_segments()` function replacing direct `simplify_path() → generate_rounded_polyline()` pipeline.
- New `segments_to_svg_path()` function generating SVG `d` attribute from segments.
- `render_edge()` updated to use new pipeline. Takes additional crossing info parameter.
- `generate_rounded_polyline()` retained as helper for non-crossing bend segments, or inlined into segment builder.

### `render/svg.rs`

- Remove crossing layer rendering.
- Pass crossing information (set of crossing grid points per edge) to `render_edge()`.
- Remove `render_crossings` import and call.

### `grid/builder.rs`

- Add `render_to_world(rrow, rcol)` method (half-cell precision coordinate conversion).
- Or: add `grid_to_world_f64(row: f64, col: f64)` accepting fractional grid coordinates.

### `routing/commit.rs` (Bug fix — Phase 0)

- `uncommit_path()` rewritten to handle crossing cells: promote `crossed_by` to `owner` when removing first occupant, clear crossing flag when removing second.
- New `reconcile_crossings()` function: full grid crossing rebuild from authoritative `paths` HashMap.
- New `compute_crossing_points()` function: derives per-edge crossing point lists from path data (replaces grid-scan approach for renderer).

### `pipeline.rs` (Bug fix — Phase 0)

- Insert `reconcile_crossings()` call after Phase 5d (post-reroute).
- Insert `reconcile_crossings()` call after Phase 6 (post-deadlock).
- Call `compute_crossing_points()` after final reconciliation, store in `RoutingResult`.

### `routing/astar.rs`

- **No changes.** Routing stays on design grid. The render grid is purely a rendering concern.

### `config.rs`

- `render_crossings: bool` → rename to `render_hops: bool` or deprecate.
- Optional: `hop_radius: f64` config (default: `cell_size * 0.5`).

---

## What Gets Added

### `render/segments.rs` (new file)

Core segment model and builders:

```rust
pub struct RenderPoint { pub x: f64, pub y: f64 }

pub enum EdgeSegment {
    Line { from: RenderPoint, to: RenderPoint },
    Bend { entry: RenderPoint, control: RenderPoint, exit: RenderPoint },
    Hop { entry: RenderPoint, exit: RenderPoint, radius: f64, sweep: bool },
}

pub fn build_edge_segments(
    grid_points: &[GridPoint],
    grid: &Grid,
    crossing_set: &HashSet<(i64, i64)>,
    corner_radius: f64,
) -> Vec<EdgeSegment>;

pub fn segments_to_svg_path(segments: &[EdgeSegment]) -> String;
```

### Crossing Data Plumbing

Currently `render_crossings()` scans the entire grid for `cell.crossing == true`. This is unreliable after rerouting (see Bug Fix section above).

New approach: derive crossing data from the authoritative `paths` HashMap after all routing mutations are complete, using `compute_crossing_points()` (added in Phase 0).

```rust
// In RoutingResult:
pub crossing_points: HashMap<usize, Vec<(i64, i64)>>
// edge_idx → list of grid points where this edge crosses another
```

This data is computed once after final reconciliation. It does not depend on mutable grid cell flags, making it immune to stale-crossing bugs.

---

## Edge Cases

### 1. Hop at a Bend

If crossing and direction change happen at the same grid point (crossing + bend), the hop takes priority. The bend is absorbed into the arc — the arc entry/exit points already handle the directional change.

### 2. Adjacent Crossings

Two consecutive grid points are both crossings. Each gets its own hop arc. Mid-points between them are at render-grid resolution (half-cell apart), so arcs don't overlap.

### 3. Crossing Near Edge Endpoints

If crossing is at the first or last grid point (on a node connector), skip hop — the edge starts/ends at that node. In practice this shouldn't happen (connector cells are on node boundaries, crossings happen in open grid space).

### 4. ER Glyph Trimming + Hops

ER edges trim start/end for glyphs. Hops in the interior of the path are unaffected. If a hop lands in the trimmed region, it's skipped (same as current crossing overlay logic — it would be hidden under the glyph).

### 5. Self-Crossing Edges

A single edge crossing itself (rare, only in complex routing). Treat same as any crossing — the edge hops over its own earlier segment.

---

## Implementation Order

### Phase 0: Fix Crossing Metadata Staleness (Bug Fix — prerequisite)

1. Update `uncommit_path()` in `routing/commit.rs` to handle crossing cells:
   - When removing the `owner` edge from a crossing cell: promote `crossed_by` to `owner`, clear `crossing`.
   - When removing the `crossed_by` edge from a crossing cell: clear `crossed_by` and `crossing`, keep `owner`.
2. Add `reconcile_crossings()` in `routing/commit.rs` (or new `routing/crossing_state.rs`).
3. Call `reconcile_crossings()` after all reroute passes in `pipeline.rs` (after Phase 5d, before deadlock).
4. Call `reconcile_crossings()` after deadlock resolver (after Phase 6).
5. Add `compute_crossing_points()` as path-based crossing derivation (replaces grid-scan approach).
6. Tests:
   - Unit test: uncommit one edge from a crossing cell → crossing flag cleared, other edge promoted.
   - Unit test: uncommit second edge from crossing → crossing flag cleared, owner keeps cell.
   - Integration test: reroute a crossing-heavy fixture → verify `cell.crossing` flags match actual path overlaps.
   - Integration test: `compute_crossing_points()` output matches grid crossing state after reconciliation.

**Goal:** Crossing metadata is always consistent with actual paths. Existing circle-overlay rendering becomes correct. New arc-based rendering (Phase 3) inherits correctness.

### Phase 1: Segment Model + Bend Migration

1. Create `render/segments.rs` with `EdgeSegment` enum and `RenderPoint`.
2. Implement `build_edge_segments()` for **Lines and Bends only** (no hops yet).
3. Implement `segments_to_svg_path()` generating `M`, `L`, `Q` commands.
4. Wire into `render_edge()` — output should be **identical** to current.
5. Run full test suite — verify SVG output unchanged.

**Goal:** Refactor to segment model without changing any visual output.

### Phase 2: Crossing Data Extraction

1. Add `crossing_points: HashMap<usize, Vec<(i64, i64)>>` to `RoutingResult`.
2. Populate using `compute_crossing_points()` from Phase 0 (path-based derivation, not grid cell flags).
3. Call after final reconciliation in pipeline, so data reflects post-reroute truth.
4. Pass `crossing_points` through to `render_edge()`.
5. Tests: verify crossing data matches for known crossing fixtures.

**Goal:** Plumb crossing data without changing rendering. Data source is the authoritative `paths` HashMap, not mutable grid state.

### Phase 3: Hop Arcs

1. Add `Hop` variant handling in `build_edge_segments()`.
2. Implement render-grid mid-point calculation for hop entry/exit.
3. Implement SVG arc (`A` command) generation in `segments_to_svg_path()`.
4. Detect arc sweep direction from edge travel direction at crossing.
5. Remove `render/crossing.rs` and the overlay circle layer from `svg.rs`.
6. Update `render_crossings` config (rename or repurpose).

**Goal:** Proper semicircular hop arcs integrated into edge paths.

### Phase 4: Polish + Config

1. Add `hop_radius` config option (default: half cell size).
2. Handle edge cases (hop at bend, adjacent crossings, near endpoints).
3. Update tests — crossing tests now verify arc presence in path `d` attribute.
4. Visual QA with benchmark fixtures (b01-b12) + crossing-heavy test cases.
5. Update docs.

---

## Risk Assessment

| Risk | Mitigation |
|------|-----------|
| Stale crossing metadata (existing bug) | Phase 0 fixes `uncommit_path()` + adds `reconcile_crossings()` safety net. Both unit and integration tests verify consistency. |
| SVG arc math wrong (sweep direction) | Unit test each direction combo (H→V, V→H, each quadrant) |
| Regression in non-crossing edges | Phase 1 produces identical output — diff test catches regressions |
| ER glyph interaction | ER trim logic runs on final segment list, not raw points |
| Performance — reconciliation scan | O(grid_cells) clear + O(total_path_points) rebuild. Negligible vs. A* routing cost. Only runs twice (after reroute, after deadlock). |
| Performance — segment building | O(n) per edge, same as current simplify+polyline |
| Mid-point at half-pixel | Use f64 throughout (already the case in world coords) |
| Reroute changes crossing set for other edges | `reconcile_crossings()` rebuilds from all paths, so transitive effects are captured. `compute_crossing_points()` re-derives per-edge crossing data from final state. |

---

## Visual Comparison

### Before (Current)

```
Edge A: ───────────────────────────
Edge B:        │
               │
        ───────⬤───────           ← white circle overlay at crossing
               │
               │
```

### After (New)

```
Edge A: ───────────────────────────
Edge B:        │
               │
        ───────╭─╮───────         ← semicircular arc hop, part of Edge B's path
               │
               │
```

The arc is part of Edge B's `<path d="...">`, not a separate element. Edge A passes straight through. Edge B's path includes `... L mid1 A r r 0 0 1 mid2 L ...`.
