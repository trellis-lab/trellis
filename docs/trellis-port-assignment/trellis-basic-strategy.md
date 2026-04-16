# Trellis Basic — Port Assignment Strategy

## Overview

**Strategy name**: `TrellisBasic`  
**Config key**: `"TrellisBasic"` (TOML), `PortAssignmentStrategy::TrellisBasic` (Rust)  
**Type**: Single-round (no routing feedback)  
**File**: `crates/trellis-core/src/ports/trellis_basic.rs`  
**Default**: Yes — replaces `Default` as the out-of-the-box strategy

Trellis Basic assigns each edge's port by finding the node face geometrically closest to the
other node's center, then greedily claiming the nearest free connector from the center of that
face outward. Ports are booked sequentially as edges are processed, so later edges see earlier
claims and skip them.

This differs from the `Default` assigner, which distributes all edges on a side evenly after
grouping. Trellis Basic instead processes edges one by one, placing each port as close to the
face's center as possible given what is already claimed — producing shorter, more direct paths.

---

## Geometry Definitions

For a node `N` with top-left corner at `(N.x, N.y)`, width `N.width`, height `N.height`:

```
Corners (used for distance calculation only — not valid port positions):
  TL = (N.x,           N.y)
  TR = (N.x + N.width, N.y)
  BL = (N.x,           N.y + N.height)
  BR = (N.x + N.width, N.y + N.height)

N's center:
  N_cx = N.x + N.width  / 2
  N_cy = N.y + N.height / 2
```

Connectors (valid port positions) are the non-corner grid points on each face, as enumerated
by `enumerate_connectors()` in `ports/common.rs`. Corners are **never** assigned as ports.

Connector index ordering per side:
```
Top    — left to right  (col ascending, row = top row)
Bottom — left to right  (col ascending, row = bottom row)
Left   — top to bottom  (row ascending, col = left col)
Right  — top to bottom  (row ascending, col = right col)
```

Each corner is an endpoint of exactly two sides:
```
TL → Top  side index 0   (leftmost)   | Left  side index 0   (topmost)
TR → Top  side index N-1 (rightmost)  | Right side index 0   (topmost)
BL → Bottom side index 0 (leftmost)   | Left  side index N-1 (bottommost)
BR → Bottom side index N-1(rightmost) | Right side index N-1 (bottommost)
```

---

## Algorithm

### Step 0 — Edge processing order

For each node `N`, collect all edges incident to it (both as source and target).
Sort these edge-endpoints by angle (from `N`'s center to the other node's center),
ascending. Ties broken by edge index ascending. This order determines which edge
claims ports first, making the algorithm deterministic.

Maintain a per-node claimed set `claimed: BTreeSet<(grid_row, grid_col)>` initialised
to empty. Update it as each edge-endpoint is assigned a connector.

`BTreeSet` is used (not `HashSet`) for consistency with the codebase convention of avoiding
hash-based collections where determinism matters. Although `claimed` is only queried via
`contains` (never iterated), `BTreeSet` prevents any future code path from silently
introducing iteration-order non-determinism.

### Step 1 — Corner distances

For edge-endpoint `(N, O)` (node `N` connecting to node `O`):

Compute `O`'s center:
```
O_cx = O.x + O.width  / 2
O_cy = O.y + O.height / 2
```

Compute squared Euclidean distance from each corner of `N` to `(O_cx, O_cy)`:
```
d(TL) = (N.x           - O_cx)² + (N.y           - O_cy)²
d(TR) = (N.x + N.width - O_cx)² + (N.y           - O_cy)²
d(BL) = (N.x           - O_cx)² + (N.y + N.height - O_cy)²
d(BR) = (N.x + N.width - O_cx)² + (N.y + N.height - O_cy)²
```

Sort corners by distance ascending → ranked list `[C₁, C₂, C₃, C₄]`.

### Step 2 — Side selection from closest corner

For the closest corner `C₁`, determine its primary side:

Each corner belongs to two sides. Pick the one whose outward normal points most
toward `O`. This is computed via dot product with the normalised `N→O` vector:

```
N_to_O = (O_cx - N_cx, O_cy - N_cy)

Outward normals (unit vectors):
  Top    = ( 0, -1)
  Bottom = ( 0, +1)
  Left   = (-1,  0)
  Right  = (+1,  0)

For corner C₁ belonging to sides (S_a, S_b):
  score(S) = dot(normal(S), N_to_O)   (un-normalised, sign is sufficient)
  primary_side = argmax(score(S_a), score(S_b))
```

Tie (diagonal case, score equal): prefer the side whose direction matches
`angle_to_side(angle(N→O))` — consistent with the rest of the codebase.

Build an **ordered side priority list** by iterating `C₁ → C₄` and appending each
corner's primary side only if not already in the list:

```
priority_sides = []
for Cᵢ in [C₁, C₂, C₃, C₄]:
    s = primary_side(Cᵢ, N→O)
    if s not in priority_sides:
        priority_sides.append(s)
```

This gives at most 4 distinct sides ordered from closest to farthest.

### Step 3 — Walk direction within a side

For side `S` selected from `priority_sides`, determine which corner of `S` is closest
to `O` (= the corner with smaller `d(C)` among the two corners of `S`):

```
corners_of_side(Top)    = (TL, TR)  →  indices (0, N-1)
corners_of_side(Bottom) = (BL, BR)  →  indices (0, N-1)
corners_of_side(Left)   = (TL, BL)  →  indices (0, N-1)
corners_of_side(Right)  = (TR, BR)  →  indices (0, N-1)
```

Let `near_end_idx` = connector index of the closer corner on side `S`,
and `far_end_idx` = connector index of the farther corner.

Center index: `center_idx = connectors.len() / 2`

### Step 4 — Greedy connector search

Connectors for side `S` are the list returned by `enumerate_connectors(N, S, ...)`.

Search in this order:

**Primary sweep** (center toward near end):
```
for i in walk(center_idx → near_end_idx):   // inclusive, step toward near end
    if connector[i] not in claimed:
        assign connector[i]
        claimed.insert(connector[i])
        return
```

**Fallback 1** (center toward far end — all near-end slots taken):
```
for i in walk(center_idx → far_end_idx):    // inclusive, step toward far end
    if connector[i] not in claimed:
        assign connector[i]
        claimed.insert(connector[i])
        return
```

Note: center_idx is visited in both sweeps. On the primary sweep it is the first
candidate. On fallback 1 it has already been checked and is claimed, so it is
naturally skipped.

**Fallback 2** (entire primary side full — try next side in priority list):
```
for S_next in priority_sides[1..]:
    repeat Steps 3–4 for S_next
    if assignment succeeded: return
```

**Ultimate fallback** (all four sides full — pathological case):
Scan all four sides' connectors linearly and assign the first unclaimed connector
found. This should not occur in practice for graphs with reasonable node sizes.

---

## Pseudocode

```
fn assign_trellis_basic(ctx: PortAssignmentContext) -> HashMap<usize, EdgePorts>:
    node_map = build node lookup
    claimed  = HashMap<node_id, BTreeSet<(row, col)>>()
    result   = HashMap<edge_index, EdgePorts>()

    // Collect all (edge_index, node_id, other_node_id, is_source) tuples
    // Sort per-node by angle(node → other_node) asc, then edge_index asc
    per_node_work = group_and_sort_edge_endpoints(graph, node_map)

    for node_id in per_node_work:
        node    = node_map[node_id]
        n_cx    = node.x + node.width  / 2
        n_cy    = node.y + node.height / 2
        node_claimed = claimed.entry(node_id).or_default()

        for (edge_idx, other_id, is_source) in per_node_work[node_id]:
            other   = node_map[other_id]
            o_cx    = other.x + other.width  / 2
            o_cy    = other.y + other.height / 2

            // Step 1: corner distances
            corners = [(TL, d_TL), (TR, d_TR), (BL, d_BL), (BR, d_BR)]
            corners.sort_by(|a, b| a.dist.cmp(b.dist))

            // Step 2: side priority list
            n_to_o = (o_cx - n_cx, o_cy - n_cy)
            priority_sides = build_priority_sides(corners, n_to_o)

            // Steps 3-4: find first free connector
            port = None
            for side in priority_sides:
                connectors = enumerate_connectors(node, side, ...)
                if connectors.is_empty(): continue

                center_idx  = connectors.len() / 2
                (near_idx, far_idx) = near_far_corner_indices(side, corners)

                // Primary sweep: center → near end
                port = first_unclaimed(connectors, center_idx, near_idx, node_claimed)
                if port.is_some(): break

                // Fallback 1: center → far end
                port = first_unclaimed(connectors, center_idx, far_idx, node_claimed)
                if port.is_some(): break

            if port.is_none():
                port = scan_all_unclaimed(node, node_claimed, ...)   // ultimate fallback

            node_claimed.insert((port.grid_row, port.grid_col))
            store port in result[edge_idx] as source or target

    apply_pinned_ports(result, ctx.pinned_ports)    // honour straight-edge pre-pass locks
    result
```

---

## Walk Helper

```
fn first_unclaimed(
    connectors: &[Connector],
    from_idx:   usize,
    to_idx:     usize,
    claimed:    &HashSet<(i64, i64)>,
) -> Option<&Connector>:

    step = if to_idx >= from_idx { +1 } else { -1 }
    i    = from_idx
    loop:
        c = &connectors[i]
        if (c.grid_row, c.grid_col) not in claimed:
            return Some(c)
        if i == to_idx: return None
        i += step
```

---

## Interaction with the Pre-pass

The `straight_edge_prepass` in `prepass.rs` locks ports for edges that can route
in a straight line (no bends). These pinned ports are stored in `ctx.pinned_ports`
and applied **after** the main assignment loop via `apply_pinned()`. Trellis Basic
must mark pinned connectors as claimed before processing remaining edges to avoid
assigning the same slot twice:

```
// Before the main loop, pre-claim pinned connectors
for (edge_idx, pinned) in &ctx.pinned_ports:
    src_node = node_map[graph.edges[edge_idx].from]
    tgt_node = node_map[graph.edges[edge_idx].to]
    claimed[src_node.id].insert((pinned.source_port.grid_row, pinned.source_port.grid_col))  // BTreeSet
    claimed[tgt_node.id].insert((pinned.target_port.grid_row, pinned.target_port.grid_col))  // BTreeSet
    // Skip this edge in the main loop
    skip_set.insert(edge_idx)
```

---

## Diamond Nodes

`enumerate_connectors` returns exactly one connector per side for `NodeShape::Diamond`
(the visual tip of the diamond on each side). Trellis Basic handles diamonds correctly
because the walk logic degenerates to a single-candidate check per side; the priority
list still determines which side is tried first.

---

## Fallback Chain Summary

```
1. Primary side, center → near corner  (closest to O)
2. Primary side, center → far corner   (away from O)
3. Next side in priority order, same two sweeps
4. ... (repeat for remaining sides)
5. Ultimate: linear scan of all connectors on all sides
```

---

## Comparison with Existing Strategies

| Property | Default | TrellisBasic |
|---|---|---|
| Side selection | Angle → 90° sector | Corner distance → dot product |
| Port position within side | Even distribution, all edges at once | Greedy, center-first, sequential |
| Port contention | None (each side independently even-spaced) | Claimed set per node; later edges get off-center ports |
| Self-loops | Handled by angle fallback | Priority list naturally selects adjacent side |
| Multi-edge sides | Evenly spread | Packed near center first |
| Overflow (more edges than connectors) | `handle_overflow` → clockwise spill | Priority list → next corner's side |
| Crossing avoidance | None explicit | Implicit: center-biased ports keep paths short |

---

## Implementation Notes

### New file

`crates/trellis-core/src/ports/trellis_basic.rs`

Struct:
```rust
pub struct TrellisBasicAssigner;

impl PortAssigner for TrellisBasicAssigner {
    fn assign_ports(&self, ctx: &PortAssignmentContext) -> HashMap<usize, EdgePorts> {
        assign_trellis_basic(ctx)
    }
}
```

### Config changes

Add variant to `PortAssignmentStrategy` in `config.rs` and mark it `#[default]`:
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum PortAssignmentStrategy {
    #[default]
    TrellisBasic,     // Greedy center-first with corner-distance side selection (DEFAULT)
    Default,          // Legacy angle-based even-distribution (kept for regression testing)
    Barycenter,
    Median,
    CrossingGreedy,
    IterativeSwap,
    TwoPhase,
    Auto,
}
```

Update `create_port_assigner` in `ports/mod.rs`:
```rust
PortAssignmentStrategy::TrellisBasic => Box::new(TrellisBasicAssigner),
```

Update `default_port_assignment()` in `config.rs`:
```rust
fn default_port_assignment() -> PortAssignmentStrategy {
    PortAssignmentStrategy::TrellisBasic
}
```

Add arm to `all_strategies_produce_same_count` test in `ports/mod.rs`.

### Reused helpers from `ports/common.rs`

- `enumerate_connectors(node, side, cell_size, offset_x, offset_y)` — connector list
- `calculate_angle(from, to)` — for edge sorting by angle
- No dependency on `build_edge_side_map`, `sort_edges_on_side`, or `handle_overflow`
  (Trellis Basic replaces all three with its own logic)

### Private helpers needed in `trellis_basic.rs`

- `corner_distances(node, o_cx, o_cy) -> [(Corner, f64); 4]`
- `primary_side(corner, n_to_o, topo_rank) -> Side`
- `build_priority_sides(sorted_corners, n_to_o) -> Vec<Side>`
- `near_far_end_indices(side, sorted_corners) -> (usize, usize)`
- `first_unclaimed(connectors, from, to, claimed) -> Option<usize>`
- `scan_all_unclaimed(node, claimed, ...) -> Connector`

### TOML example

```toml
port_assignment = "TrellisBasic"
```

---

## Test Cases

### Unit tests (in `trellis_basic.rs`)

| Test | What it checks |
|---|---|
| `direct_right_gets_right_side` | Node B directly right of A → port on Right side of A |
| `center_port_preferred` | Single edge → port at center index of selected side |
| `second_edge_displaced_from_center` | Two edges → second edge gets next slot from center |
| `near_corner_direction` | B at upper-right of A → primary walk toward TR end of Right side |
| `full_primary_side_spills_to_next` | Side at capacity → port assigned on second priority side |
| `diamond_single_port_per_side` | Diamond node → port at tip of selected face |
| `pinned_ports_excluded_from_claimed` | Pre-pass locked ports not reassigned |

### Integration fixtures

Create `tests/benchmarks/fixtures/b25.mmd` — a graph where center-biased ports
produce shorter average path lengths than even-spacing:

```
flowchart LR
  A --> B
  A --> C
  A --> D
  B --> E
  C --> E
  D --> E
```

Assert `total_path_length(TrellisBasic) <= total_path_length(Default)`.
