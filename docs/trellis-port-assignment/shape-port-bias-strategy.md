# Shape Port Bias — Strategy Document

## Overview

**Goal**: Let individual node shapes override the side-priority list that TrellisBasic
derives from corner distances, based on diagram flow direction and edge direction
(incoming vs. outgoing). Diamond nodes are the first case; the design is built for
future shapes without code surgery.

**Scope**: TrellisBasic only. Other strategies (Barycenter, Median, etc.) are
single-pass crossing-minimisers that operate on pre-committed sides and do not
benefit from shape-specific side selection at this stage.

---

## Motivation

TrellisBasic selects a side priority list from corner distances. For `NodeShape::Diamond`
this works geometrically, but it ignores the semantic intent of a diamond (decision
node) in the context of diagram flow direction:

- In a **Top-Down** diagram the "main" path enters a diamond from above (North) and
  exits vertically or laterally. Forcing incoming edges to favour N/S and outgoing
  edges to favour E/W aligns with the conventional flowchart reading direction and
  reduces unnecessary bends.
- In a **Left-Right** diagram the roles flip: main path enters East/West, branches
  exit North/South.

The corner-distance heuristic will usually arrive at the correct side for a single
edge, but it has no knowledge of whether an edge is incoming or outgoing, and it
cannot distinguish "main flow" sides from "branch" sides. Shape-specific overrides
fill that gap.

---

## Design Goals

1. **Zero changes to the core algorithm** — TrellisBasic calls one hook; the hook
   returns either an override or nothing. All shape logic is isolated.
2. **Pure function, no state** — the hook takes a context struct and returns
   `Option<[Side; 4]>`. No trait objects, no dynamic dispatch, no allocations.
3. **Easy to extend** — adding a new shape means adding one `match` arm; no new
   files, traits, or registries required.
4. **Respects `FlowBias` config** — shape bias is suppressed when
   `flow_bias = None` and limited to Sugiyama layouts when `flow_bias = Auto`.
5. **Deterministic** — pure function over immutable inputs; same inputs always
   yield same output.

---

## Existing Infrastructure

`ports/mod.rs` already provides:

```rust
pub fn effective_direction(ctx: &PortAssignmentContext) -> Option<Direction> {
    match ctx.flow_bias {
        FlowBias::None   => None,
        FlowBias::Strong => Some(ctx.graph.direction),
        FlowBias::Auto   => {
            if ctx.graph.diagram_type == DiagramType::Flowchart {
                Some(ctx.graph.direction)
            } else {
                None
            }
        }
    }
}
```

`None` means "no bias active"; `Some(dir)` means "apply bias for this direction".
This already encodes `FlowBias`, `diagram_type`, and `graph.direction` in one call —
no need to duplicate that logic in `shape_bias.rs`.

---

## Module: `ports/shape_bias.rs`

New file: `crates/trellis-core/src/ports/shape_bias.rs`

### Context struct

```rust
/// Inputs to the shape-specific side-priority override.
///
/// `effective_dir` is computed once per edge-endpoint via `effective_direction(ctx)`
/// from `ports/mod.rs`. `None` means bias is inactive (shape override suppressed).
pub struct ShapePortContext {
    /// Visual shape of the node being assigned a port.
    pub shape: NodeShape,
    /// Resolved flow direction, or `None` when bias is disabled.
    ///
    /// Callers obtain this from `effective_direction(&port_ctx)` — that function
    /// already handles `FlowBias::None / Auto / Strong` and the Sugiyama check.
    pub effective_dir: Option<Direction>,
    /// `true`  = this edge leaves the node   (source endpoint)
    /// `false` = this edge enters the node   (target endpoint)
    pub is_source: bool,
}
```

All fields are `Copy` types. No heap allocation.

### Entry-point function

```rust
/// Returns a shape-specific side-priority override, or `None` to fall back
/// to TrellisBasic's default corner-distance priority list.
///
/// The returned array lists the four sides in preference order; TrellisBasic
/// walks them left to right, stopping at the first side with a free connector.
pub fn shape_side_priority(ctx: &ShapePortContext) -> Option<[Side; 4]> {
    let dir = ctx.effective_dir?;   // None → no bias → use default algorithm
    match ctx.shape {
        NodeShape::Diamond => Some(diamond_priority(dir, ctx.is_source)),
        // Future shapes: add arms here.
        _ => None,
    }
}
```

`bias_active()` is not needed — `effective_dir` being `None` is the "inactive"
signal; the `?` operator handles it in one line.

### Diamond rules

```rust
fn diamond_priority(dir: Direction, is_source: bool) -> [Side; 4] {
    use Direction::*;
    use Side::*;
    // is_source == true  → outgoing edge (from diamond)
    // is_source == false → incoming edge (into diamond)
    match (dir, is_source) {
        // Top-Down / Bottom-Up — vertical main flow
        (TB, false) => [North, South, East, West],   // incoming: N/S first
        (TB, true)  => [East, West, North, South],   // outgoing: E/W first
        (BT, false) => [South, North, East, West],   // incoming: S then N
        (BT, true)  => [East, West, South, North],   // outgoing: E/W first
        // Left-Right / Right-Left — horizontal main flow (vice versa)
        (LR, false) => [East, West, North, South],   // incoming: E/W first
        (LR, true)  => [North, South, East, West],   // outgoing: N/S first
        (RL, false) => [West, East, North, South],   // incoming: W then E
        (RL, true)  => [North, South, West, East],   // outgoing: N/S first
    }
}
```

**Rationale for BT/RL orderings:**

- `BT` mirrors `TB`: the "near" pole of a reversed vertical flow is South, so
  incoming edges prefer South before North.
- `RL` mirrors `LR`: near pole is West, so incoming edges prefer West before East.
- Outgoing branch edges in `BT`/`RL` still use the perpendicular axis (E/W and
  N/S respectively) because branch exits are always orthogonal to the main flow
  axis regardless of direction.

---

## Integration with TrellisBasic

In `trellis_basic.rs`, call `effective_direction(ctx)` once per node (or per
edge-endpoint) and pass the result into `ShapePortContext`. No need to touch
`config.flow_bias` or `graph.diagram_type` directly inside `trellis_basic.rs` —
`effective_direction` already encapsulates both.

```rust
// Computed once per PortAssignmentContext, outside the per-edge loop:
let eff_dir = effective_direction(ctx);   // from ports::mod

// Inside the per-edge-endpoint loop, after computing `corners`:
let shape_ctx = ShapePortContext {
    shape: node.shape,
    effective_dir: eff_dir,
    is_source,
};

let priority_sides: Vec<Side> = shape_side_priority(&shape_ctx)
    .map(|arr| arr.to_vec())
    .unwrap_or_else(|| build_priority_sides(&corners, n_to_o));
```

No other changes to TrellisBasic's algorithm. The rest of Steps 3–4 (walk
direction, greedy connector search, fallback chain) operate identically on the
resolved `priority_sides` vector regardless of its origin.

---

## Connector Availability for Diamonds

`enumerate_connectors` returns **exactly one connector per side** for
`NodeShape::Diamond` (the visual tip of each face). This means:

- Each `Side` in `priority_sides` either has its single connector free or not.
- The greedy walk in Steps 3–4 degenerates to a single-candidate check.
- The four-side override list is therefore also a strict fallback chain:
  first available tip wins.

For a diamond with two outgoing edges in a `TB` diagram the assignment will be:

| Edge | Priority tried | Result |
|------|----------------|--------|
| First outgoing  | East   | East tip free → assigned |
| Second outgoing | East → West | East taken → West tip |

If the diagram has three outgoing edges, the third spills to North (or South), which
is the expected fallback behaviour: the shape bias prefers perpendicular sides for
outgoing edges but never blocks assignment entirely.

---

## Config Interaction

Shape bias piggy-backs on `effective_direction()` — the same function already
used by other bias-aware code in TrellisBasic. No new config key.

| `flow_bias` | `diagram_type`  | `effective_direction()` | Shape bias active? |
|-------------|-----------------|-------------------------|--------------------|
| `None`      | any             | `None`                  | No                 |
| `Auto`      | ER / C4 / Class | `None`                  | No                 |
| `Auto`      | Flowchart       | `Some(graph.direction)` | Yes                |
| `Strong`    | any             | `Some(graph.direction)` | Yes                |

---

## Extension Protocol — Adding a New Shape

To add shape-specific port bias for a new shape (e.g., `NodeShape::Circle`):

1. Add a match arm in `shape_side_priority`:
   ```rust
   NodeShape::Circle => circle_priority(ctx),
   ```
2. Implement `circle_priority(ctx: &ShapePortContext) -> Option<[Side; 4]>` in
   the same file, following the same pattern as `diamond_priority`.
3. Return `None` for direction/edge-direction combinations where the default
   algorithm is already correct (opt-in override, not full replacement).
4. Add unit tests (see section below).

No changes needed in `trellis_basic.rs`, `config.rs`, or any other file.
(`mod.rs` already has `pub mod shape_bias;` from Step 2 of the implementation plan.)

---

## Implementation Plan

| Step | File | Change |
|------|------|--------|
| 1 | `ports/shape_bias.rs` | Create — `ShapePortContext`, `shape_side_priority`, `bias_active`, `diamond_priority` |
| 2 | `ports/mod.rs` | `pub mod shape_bias;` |
| 3 | `ports/trellis_basic.rs` | Import `shape_bias`, build `ShapePortContext`, replace `priority_sides` when override returned |
| 4 | `ports/shape_bias.rs` | Unit tests (see below) |
| 5 | `tests/benchmarks/fixtures/b26.mmd` | Integration fixture — diamond in TD flowchart |

---

## Test Cases

### Unit tests (in `shape_bias.rs`)

| Test name | Scenario | Expected |
|-----------|----------|----------|
| `diamond_tb_incoming_prefers_north` | Diamond, TB, `is_source=false` | `[North, South, East, West]` |
| `diamond_tb_outgoing_prefers_east` | Diamond, TB, `is_source=true` | `[East, West, North, South]` |
| `diamond_bt_incoming_prefers_south` | Diamond, BT, `is_source=false` | `[South, North, East, West]` |
| `diamond_lr_incoming_prefers_east` | Diamond, LR, `is_source=false` | `[East, West, North, South]` |
| `diamond_lr_outgoing_prefers_north` | Diamond, LR, `is_source=true` | `[North, South, East, West]` |
| `diamond_rl_outgoing_prefers_north` | Diamond, RL, `is_source=true` | `[North, South, West, East]` |
| `bias_none_returns_none` | Any shape, `effective_dir=None` | `None` |
| `non_diamond_shape_returns_none` | `NodeShape::Rect`, `effective_dir=Some(TB)` | `None` |

### Integration fixture `b26.mmd`

```
flowchart TD
  A[Start] --> D{Decision}
  D -- yes --> B[Branch B]
  D -- no  --> C[Branch C]
  B --> E[End]
  C --> E
```

Assert:
- Port assigned to `D` for edge `A→D`: on North or South side (incoming, TB).
- Ports for `D→B` and `D→C`: on East and West sides respectively (outgoing, TB).
- No connector on the same grid cell used twice on node `D`.

---

## Non-Goals

- Shape bias for `Barycenter`, `Median`, or `CrossingGreedy` strategies — they
  operate on committed sides and crossing counts, not on side selection.
- Per-node config overrides (e.g., `pin_side = "East"`) — out of scope; addressed
  by the existing `pinned_ports` mechanism in `prepass.rs`.
- Dynamic shape bias depending on neighbour count — the geometry stays simple;
  the priority list is a fixed 4-element array per (shape, direction, is_source)
  combination.
