# Port Assignment Algorithms — Implementation Plan

## Problem Statement

The current port assignment algorithm assigns ports **statically** based on the angle between
node centers. The pipeline is one-way:

```
placement → ports (static) → routing → deadlock recovery → rendering
```

There is no feedback from routing back to port assignment. When the chosen port ordering causes
edge crossings, the router must resolve them via expensive deadlock recovery (rip-up-and-reroute,
grid expansion, or forced crossings with visual bridges).

### Root Cause of Crossing-Inducing Port Assignments

Port ordering on a node's side is a **permutation problem**. For *k* edges on one side there are
*k!* possible orderings. The current algorithm picks one (sorted by perpendicular coordinate of
the target node), but this is often not crossing-minimal.

Example: node A has 3 edges leaving its bottom side to B1, B2, B3. The current algorithm sorts
ports left-to-right matching immediate target x-positions. But when targets are on **different
layers** or route through **intermediate congestion zones**, the geometrically-sorted order forces
paths to cross before they reach the routing grid.

### Current Algorithm Summary (`DefaultPortAssigner`)

Location: `crates/trellis-core/src/ports/assignment.rs`

1. Compute angle from each node center to connected node center
2. Map angle to side (4 quadrants of 90 degrees)
3. Handle overflow: if a side has more edges than connectors, spill to clockwise neighbour
4. Sort edges per side by perpendicular axis of the other node's center
5. Distribute ports evenly across non-corner grid connectors

### How Crossings Are Tracked

- `commit.rs`: when a second edge occupies an already-occupied cell, `cell.crossing = true`
- `grid.count_crossings()`: returns total crossing cell count
- `RoutingResult`: exposes `crossings`, `deadlock_recoveries`, `total_bends`, `total_routing_cost`
- Deadlock handler (3 levels): rip-up-and-reroute → grid expansion → forced crossing fallback

---

## Algorithm Catalogue

### Single-Round Algorithms (no routing feedback)

These replace or enhance the port ordering logic inside `assign_ports()`. They run once before
routing, have no dependency on routing results, and fit cleanly into the `PortAssigner` trait
from the pluggable architecture plan.

---

#### Algorithm 1: Barycenter Ordering

**Strategy**: `Barycenter`

For each side of a node, order edges by the **barycenter** (weighted average position) of the
connected sub-graph, rather than just the immediate target node position.

**Algorithm**:

```
for each node N:
    for each side S of N:
        for each edge E on side S:
            target = E.target_node
            neighbours = nodes connected to target (1-2 hops)
            if S is Top or Bottom:
                barycenter = mean(x_center of target + neighbours)
            else:
                barycenter = mean(y_center of target + neighbours)
        sort edges on S by barycenter ascending
        distribute to connectors (same as current even-spacing logic)
```

**Complexity**: O(E) per node side — negligible overhead.

**When it helps**: Fan-out patterns where a hub connects to nodes that then diverge. The
immediate target positions suggest one order, but the downstream topology suggests another.

**Implementation notes**:
- New file: `crates/trellis-core/src/ports/barycenter.rs`
- Struct: `pub struct BarycenterPortAssigner;`
- Reuse `enumerate_connectors()`, `angle_to_side()`, `handle_overflow()` from `assignment.rs`
  (extract to shared `ports/common.rs` or make `pub(crate)`)
- Only the sorting step changes — replace `sort_edges_on_side()` with barycenter-based sort
- Needs read access to the full `Graph` (already in `PortAssignmentContext`)

---

#### Algorithm 2: Median Ordering

**Strategy**: `Median`

Same as barycenter but uses the **median** position instead of the mean. More robust when one
far-off node skews the average.

**Algorithm**:

```
for each node N:
    for each side S of N:
        for each edge E on side S:
            target = E.target_node
            positions = [perpendicular_center(target)] ++
                        [perpendicular_center(n) for n in target.neighbours]
            sort(positions)
            median = positions[len/2]
        sort edges on S by median ascending
        distribute to connectors
```

**Complexity**: O(E log E) per side due to median computation.

**When it helps**: Asymmetric graphs where one branch goes far in one direction while most
go the other way.

**Implementation notes**:
- New file: `crates/trellis-core/src/ports/median.rs`
- Very similar to barycenter — could share a parameterised implementation with a
  `AggregationFn` closure (`mean` vs `median`)

---

#### Algorithm 3: Crossing-Count Greedy

**Strategy**: `CrossingGreedy`

For each pair of edges on the same side, determine if their port order creates an **inversion**
relative to the order their targets appear on the receiving side. Find the permutation that
minimises inversions.

**Algorithm**:

```
for each node N:
    for each side S of N:
        edges = edges on side S
        k = len(edges)

        // Build crossing matrix
        for i in 0..k:
            for j in (i+1)..k:
                // Check if edge_i and edge_j would cross given current order
                // They cross if: port_order(i) < port_order(j) but
                //                target_port_order(i) > target_port_order(j)
                // i.e., source and target orderings are inverted
                crossing_matrix[i][j] = would_cross(edges[i], edges[j])

        // Find permutation minimising total inversions
        // For small k (< 8): try all k! permutations
        // For larger k: use greedy adjacent-swap (bubble sort minimising crossings)
        optimal_order = minimize_inversions(edges, crossing_matrix)

        distribute optimal_order to connectors
```

**Crossing detection for a pair**: Two edges (A→B) and (A→C) sharing source node A cross if
their source ports are ordered left-to-right but their target ports are ordered right-to-left
(or vice versa). This can be checked by comparing:
- Source side: perpendicular coordinate of target nodes (determines desired port order)
- Target side: the actual port positions on the target nodes

For edges going to **different** target nodes, use the target node's center position as a proxy
for where its port will be.

**Complexity**: O(k^2) per side for crossing matrix, O(k log k) for inversion-minimising sort.
Since k is typically 1-5 edges per side, this is effectively constant.

**When it helps**: Most directly targets the crossing problem. Effective whenever the geometric
sort produces inversions — common in multi-layer flowcharts and class diagrams with inheritance
hierarchies.

**Implementation notes**:
- New file: `crates/trellis-core/src/ports/crossing_greedy.rs`
- Key helper: `fn would_cross(edge_a, edge_b, node_map) -> bool` — compare source-side
  vs target-side orderings
- For the inversion-minimising sort: since k is small, a simple insertion sort tracking
  inversions is sufficient. No need for merge-sort-based inversion counting.
- Reuse all infrastructure from `assignment.rs` except the sorting step

---

#### Algorithm 4: Side Rebalancing

**Strategy**: Not a standalone algorithm — enhancement applicable to any of the above.

Proactively redistribute edges across sides to **balance routing load** before routing begins.

**Algorithm**:

```
for each node N:
    // After initial angle-based side assignment:
    for each side S:
        capacity = enumerate_connectors(N, S).len()
        load = edges_on_side(S).len()
        congestion[S] = load as f64 / capacity.max(1) as f64

    // Rebalance: move borderline edges from congested to light sides
    for each side S where congestion[S] > 0.8:
        neighbour = clockwise_or_counterclockwise with lowest congestion
        if congestion[neighbour] < 0.4:
            // Move edge closest to boundary between S and neighbour
            edge = edge_closest_to_boundary(S, towards=neighbour)
            move edge from S to neighbour
            recalculate congestion
```

**Complexity**: O(E) — single redistribution pass.

**When it helps**: Dense hub nodes where one side accumulates 5+ edges while adjacent sides
are empty. The current overflow handler only triggers when a side is **full** (edges > connectors);
this triggers earlier when congestion is high but not yet at capacity.

**Implementation notes**:
- Can be added as a post-processing step in any `PortAssigner` implementation
- Extract to `ports/common.rs` as `fn rebalance_sides(...)` for reuse
- Threshold values (0.8, 0.4) should be configurable or tuned empirically

---

### Multi-Round Algorithms (with routing feedback)

These require running the routing pipeline (partially or fully) and feeding results back to
port assignment. They need changes to the pipeline loop in `pipeline.rs`, not just a new
`PortAssigner` implementation.

---

#### Algorithm 5: Route-Then-Swap

**Strategy**: `IterativeSwap`

Route all edges with current ports. Identify crossings. Swap port assignments for crossing
edge pairs. Re-route affected edges. Repeat.

**Algorithm**:

```
ports = default_assign_ports(graph)
best_crossings = infinity

for round in 0..max_rounds:          // max_rounds = 2-3
    grid = build_grid(graph)
    result = route_all_edges(graph, grid, ports)

    if result.crossings == 0:
        break                          // optimal — no crossings
    if result.crossings >= best_crossings:
        break                          // no improvement — stop

    best_crossings = result.crossings

    // Find crossing pairs
    crossing_pairs = find_crossing_edge_pairs(grid)

    // For each crossing pair, try swapping their ports on the shared node
    for (edge_a, edge_b) in crossing_pairs:
        shared_node = find_shared_node(edge_a, edge_b)
        if shared_node is None:
            continue                   // crossing between unrelated edges — skip

        // Swap port assignments for edge_a and edge_b on shared_node
        swap(ports[edge_a].port_on(shared_node),
             ports[edge_b].port_on(shared_node))

    // Next round will re-route with swapped ports
```

**Finding crossing pairs from the grid**:

```
fn find_crossing_edge_pairs(grid) -> Vec<(usize, usize)>:
    pairs = HashSet::new()
    for cell in grid.cells:
        if cell.crossing:
            // Cell is occupied by one edge (owner) and crossed by another
            // Need to track all edges passing through each cell
            // Current grid only stores single owner — enhancement needed:
            // store Vec<edge_id> or use a separate crossing map during routing
    pairs
```

**Integration with existing code**:
- Uses `RoutingResult.crossings` to detect when to stop
- Uses the existing uncommit pattern from `deadlock/rip_up.rs` to clear paths before re-routing
- Requires enhancing `commit.rs` to track **both** edge IDs at crossing cells (currently only
  stores `owner` for the first edge; the crossing edge is not recorded)

**Complexity**: O(rounds * E * R) where R is the per-edge routing cost. With 2-3 rounds and
typically only a few affected edges per round, practical overhead is 1.5-2x total routing time.

**When it helps**: Cases where the deadlock handler currently forces crossings (level 3 fallback).
Instead of accepting the crossing, this adjusts the root cause (port positions).

**Pipeline change required** (`pipeline.rs`):

```rust
// Phase 4-5: Iterative port assignment + routing
let mut port_assignments = assigner.assign_ports(&port_ctx);
let mut best_result = None;

for _round in 0..config.port_refinement_rounds {
    let mut grid = build_grid(&graph, cell_size, &extent);
    let result = routing::route_all_edges(&graph, &mut grid, &port_assignments, config);

    if result.crossings == 0 {
        best_result = Some((grid, result));
        break;
    }

    if let Some((_, ref best)) = best_result {
        if result.crossings >= best.crossings {
            break; // no improvement
        }
    }

    // Identify and apply port swaps for crossing edges
    let swaps = find_crossing_swaps(&grid, &port_assignments);
    if swaps.is_empty() {
        best_result = Some((grid, result));
        break;
    }
    apply_port_swaps(&mut port_assignments, &swaps);
    best_result = Some((grid, result));
}
```

**New config field**:

```rust
/// Maximum port refinement rounds (0 = disabled)
#[serde(default = "default_port_refinement_rounds")]
pub port_refinement_rounds: usize,

fn default_port_refinement_rounds() -> usize { 0 }
```

**Implementation notes**:
- New file: `crates/trellis-core/src/ports/iterative.rs` — swap logic and crossing pair detection
- Modify `crates/trellis-core/src/routing/commit.rs` — track second edge ID at crossing cells
- Modify `crates/trellis-core/src/pipeline.rs` — add refinement loop
- The `PortAssigner` trait provides the **initial** assignment; the refinement loop is a
  pipeline-level concern that works with any initial assigner

---

#### Algorithm 6: Simulated Annealing on Port Permutations

**Strategy**: `Annealing`

Treat port ordering as a combinatorial optimisation problem. Explore the space of port
permutations using simulated annealing, evaluating each candidate by a cost function.

**Algorithm**:

```
current = default_assign_ports(graph)
current_score = evaluate(current)     // fast crossing estimate or full routing
best = current
best_score = current_score
T = initial_temperature              // e.g., 10.0
cooling_rate = 0.95

for iteration in 0..max_iterations:  // e.g., 200-1000
    // Generate neighbour: swap two ports on one side of one random node
    candidate = current.clone()
    node = random_node_with_multiple_edges_on_a_side()
    side = random_side_of(node) with >= 2 edges
    (i, j) = random_pair_on(side)
    swap(candidate[node][side][i], candidate[node][side][j])

    candidate_score = evaluate(candidate)
    delta = candidate_score - current_score

    if delta < 0 or random() < exp(-delta / T):
        current = candidate
        current_score = candidate_score
        if current_score < best_score:
            best = current
            best_score = current_score

    T *= cooling_rate

return best
```

**Evaluation function** — two options:

1. **Fast estimate** (no routing): count inversions between port orders on connected node
   pairs across all edges. O(E) per evaluation. Suitable for the inner loop.
   ```
   score = 0
   for each edge (A→B):
       for each other edge (A→C) sharing a side on A:
           if port_order(A→B, A→C) inverted relative to target positions:
               score += 1
   ```

2. **Full routing** (expensive): run `route_all_edges` and use
   `crossings * 100 + total_path_length` as score. Only practical for small graphs or
   final validation.

**Recommended hybrid**: use fast estimate for annealing iterations, validate the final result
with one full routing pass.

**Complexity**: O(iterations * E) with fast estimate, O(iterations * E * R) with full routing.

**When it helps**: Complex graphs where greedy swaps get stuck in local minima. Annealing can
escape by temporarily accepting worse configurations. Most effective for graphs with 20-50
nodes and high connectivity.

**Implementation notes**:
- New file: `crates/trellis-core/src/ports/annealing.rs`
- Depends on a `rand` crate (or use deterministic seed for reproducibility)
- The fast estimate function should be in `ports/common.rs` — reusable by other algorithms
- Temperature schedule and iteration count should be configurable
- WASM consideration: `rand` works on wasm32 with `getrandom` feature; alternatively use a
  simple LCG seeded from input hash for determinism

---

#### Algorithm 7: Two-Phase Estimate + Targeted Re-route

**Strategy**: `TwoPhase`

Combines a fast crossing estimator (no routing) with targeted re-routing of only problematic
edges. Best cost/benefit ratio of the multi-round approaches.

**Algorithm**:

```
Phase 1 — Estimate and optimise (no grid, no routing):

    ports = default_assign_ports(graph)

    // Count estimated crossings using inversion counting
    for each node N:
        for each side S of N:
            edges_on_S = edges assigned to side S
            for each pair (E_i, E_j) in edges_on_S:
                if inverted(E_i, E_j):     // source order != target order
                    estimated_crossings += 1

    // Optimise: run crossing-count greedy (Algorithm 3) per side
    for each node N:
        for each side S of N:
            edges_on_S = edges assigned to side S
            optimal_order = minimize_inversions(edges_on_S)
            reorder ports on S according to optimal_order

Phase 2 — Route and refine (targeted):

    grid = build_grid(graph)
    result = route_all_edges(graph, grid, ports)

    if result.crossings > 0:
        // Only re-route edges involved in actual crossings
        crossing_edges = edges_involved_in_crossings(grid)

        for edge in crossing_edges:
            // Try swapping this edge's port with each neighbour on same side
            for swap_candidate in adjacent_ports_on_same_side(edge):
                uncommit(edge)
                uncommit(swap_candidate)
                swap ports
                re_route(edge)
                re_route(swap_candidate)
                if crossings_reduced:
                    commit swaps
                else:
                    rollback
```

**Complexity**: O(E^2) for estimation + O(E * R) for routing + O(C * R) for refinement where
C is the number of crossing edges (typically small). Total overhead: ~1.2-1.5x current routing.

**When it helps**: Best general-purpose approach. Phase 1 catches most inversions cheaply.
Phase 2 fixes the remaining cases that the estimate missed (e.g., crossings caused by routing
detours rather than port ordering).

**Implementation notes**:
- New file: `crates/trellis-core/src/ports/two_phase.rs`
- Phase 1 runs inside `PortAssigner::assign_ports()` — no pipeline change needed
- Phase 2 needs the same pipeline refinement loop as Algorithm 5 (can share the
  `port_refinement_rounds` config field)
- The inversion-counting function from Phase 1 is the same fast estimate used in Algorithm 6 —
  extract to `ports/common.rs`

---

## Shared Infrastructure

Several algorithms share common building blocks. Extract these to avoid duplication:

### `crates/trellis-core/src/ports/common.rs`

```rust
/// Enumerate non-corner grid connectors on a node side.
/// (Moved from assignment.rs — currently private)
pub(crate) fn enumerate_connectors(node, side, cell_size, offset_x, offset_y) -> Vec<Connector>

/// Map angle to node side (4 quadrants).
/// (Already public in assignment.rs — re-export)
pub fn angle_to_side(angle_deg: f64) -> Side

/// Calculate angle between two node centers.
/// (Currently private in assignment.rs)
pub(crate) fn calculate_angle(from: &Node, to: &Node) -> f64

/// Handle overflow: move excess edges to clockwise neighbour sides.
/// (Currently private in assignment.rs)
pub(crate) fn handle_overflow(sides, node, cell_size, offset_x, offset_y)

/// Distribute N edges evenly across M connectors. Returns connector indices.
pub(crate) fn distribute_evenly(n_edges: usize, n_connectors: usize) -> Vec<usize>

/// Count inversions between source-side and target-side orderings.
/// Used by CrossingGreedy, TwoPhase, and Annealing for fast crossing estimation.
pub(crate) fn count_inversions(edges: &[EdgeOnSide], node_map: &HashMap<&str, &Node>) -> usize

/// Check if two edges on the same side would cross based on target positions.
pub(crate) fn would_cross(edge_a: &EdgeOnSide, edge_b: &EdgeOnSide, node_map: ...) -> bool

/// Rebalance edges across sides (Algorithm 4 — usable as post-processing for any assigner).
pub(crate) fn rebalance_sides(sides, node, cell_size, offset_x, offset_y, threshold: f64)
```

### Grid enhancement for crossing pair tracking

Currently `commit.rs` only stores one `owner` per cell. Multi-round algorithms need to know
**which two edges** cross at a cell.

**Option A**: Add `crossed_by: Option<String>` to `Cell` — set when second edge occupies.
Minimal change, sufficient for pairwise identification.

**Option B**: Add `occupants: Vec<String>` to `Cell` — supports 3+ edges crossing at one
point. More general but more memory.

Recommendation: **Option A** for now. Three-way crossings are extremely rare on a fine grid.

Location: `crates/trellis-core/src/grid/builder.rs` (Cell struct) and
`crates/trellis-core/src/routing/commit.rs` (commit_path).

---

## Configuration

### New fields in `TrellisConfig` (`config.rs`)

```rust
/// Port assignment algorithm (default: angle-based)
#[serde(default = "default_port_assignment")]
pub port_assignment: PortAssignmentStrategy,

/// Maximum port refinement rounds after routing (0 = disabled).
/// Only used by multi-round strategies (IterativeSwap, TwoPhase).
#[serde(default = "default_port_refinement_rounds")]
pub port_refinement_rounds: usize,

fn default_port_assignment() -> PortAssignmentStrategy {
    PortAssignmentStrategy::Default
}

fn default_port_refinement_rounds() -> usize {
    0
}
```

### `PortAssignmentStrategy` enum

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum PortAssignmentStrategy {
    #[default]
    Default,          // Current angle-based algorithm
    Barycenter,       // Algorithm 1
    Median,           // Algorithm 2
    CrossingGreedy,   // Algorithm 3
    IterativeSwap,    // Algorithm 5 (multi-round)
    Annealing,        // Algorithm 6 (multi-round)
    TwoPhase,         // Algorithm 7 (multi-round, recommended)
}
```

### TOML examples

```toml
# Single-round — fast, moderate improvement
port_assignment = "CrossingGreedy"

# Multi-round — best results, ~1.5x render time
port_assignment = "TwoPhase"
port_refinement_rounds = 2

# Multi-round — exhaustive search, slow
port_assignment = "Annealing"
port_refinement_rounds = 0   # annealing handles its own iterations
```

---

## Recommended Implementation Order

### Phase A: Foundation (prerequisite for all algorithms)

1. Implement pluggable architecture from `pluggable-port-assignment-plan.md`
2. Extract shared helpers from `assignment.rs` to `ports/common.rs`
3. Add `crossed_by` field to `Cell` in `grid/builder.rs`

### Phase B: Single-round algorithms

4. **Algorithm 3: CrossingGreedy** — highest impact for lowest effort among single-round options
5. Algorithm 1+4: Barycenter with side rebalancing — good secondary option

### Phase C: Multi-round algorithms

6. **Algorithm 7: TwoPhase** — best cost/benefit ratio
7. Add `port_refinement_rounds` config field and pipeline refinement loop
8. Algorithm 5: IterativeSwap — useful as a simpler alternative to TwoPhase

### Phase D: Advanced (optional)

9. Algorithm 6: Annealing — only if benchmarks show TwoPhase leaves significant crossings
10. Algorithm 2: Median — only if barycenter proves insufficient for specific graph shapes

---

## Testing Strategy

### Per-algorithm unit tests

Each new `PortAssigner` implementation should have tests verifying:
- Correct port side assignment (same as default for simple cases)
- Port positions land on grid points
- Connector exclusion of corners
- Specific crossing-reduction scenarios (construct a graph where default creates a crossing
  but the new algorithm doesn't)

### Crossing regression tests

Create fixture graphs with known crossing counts under the default algorithm:

```
tests/port_assignment/
    crossing_fan_out.mmd    — hub with 4+ fan-out edges (tests barycenter/median)
    crossing_inversion.mmd  — two edges whose target order is inverted (tests greedy)
    crossing_congested.mmd  — dense hub with side overflow (tests rebalancing)
    crossing_multilayer.mmd — 3-layer graph with crossing chains (tests multi-round)
```

For each fixture, assert that the new algorithm produces `crossings <= default_crossings`.

### Benchmark integration

Add port assignment strategy as a parameter to the existing criterion benchmarks (B01-B12):

```rust
for strategy in [Default, CrossingGreedy, TwoPhase] {
    group.bench_with_input(
        BenchmarkId::new("render", format!("{:?}", strategy)),
        &strategy,
        |b, s| b.iter(|| render_with_strategy(graph, *s)),
    );
}
```

This measures both render time overhead and crossing count improvement per strategy.

---

## Comparison Matrix

| Algorithm | Type | Crossing Reduction | Overhead | Complexity | Best For |
|---|---|---|---|---|---|
| Default (current) | Single | Baseline | 0 | O(E) | Simple graphs |
| Barycenter | Single | Low-Medium | ~0 | O(E) | Fan-out patterns |
| Median | Single | Low-Medium | ~0 | O(E log E) | Asymmetric layouts |
| **CrossingGreedy** | **Single** | **Medium** | **~0** | **O(k^2/side)** | **Known inversions** |
| Side Rebalancing | Enhancement | Low | ~0 | O(E) | Hub congestion |
| IterativeSwap | Multi | High | 2-3x routing | O(rounds*E*R) | General reduction |
| Annealing | Multi | Highest | 10-50x routing | O(iter*E*R) | Small complex graphs |
| **TwoPhase** | **Multi** | **High** | **1.2-1.5x routing** | **O(E^2 + E*R)** | **Best cost/benefit** |

Recommended defaults in bold.
