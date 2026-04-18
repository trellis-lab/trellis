use trellis_parser::{Direction, NodeShape};

use super::assignment::Side;

/// Inputs to the shape-specific side-priority override.
///
/// `effective_dir` is computed once via `effective_direction(ctx)` from
/// `ports/mod.rs`, which already encodes `FlowBias`, `diagram_type`, and
/// `graph.direction`. `None` means bias is inactive.
pub struct ShapePortContext {
    pub shape: NodeShape,
    /// `None` → bias disabled; `Some(dir)` → apply override for this direction.
    pub effective_dir: Option<Direction>,
    /// `true` = source endpoint (edge leaves this node).
    /// `false` = target endpoint (edge enters this node).
    pub is_source: bool,
}

/// Returns a shape-specific side-priority override, or `None` to fall back
/// to the caller's default (corner-distance) priority list.
///
/// When `Some` is returned the array lists sides in preference order; the
/// caller tries them left to right and stops at the first side with a free
/// connector.
pub fn shape_side_priority(ctx: &ShapePortContext) -> Option<[Side; 4]> {
    let dir = ctx.effective_dir?; // None → no bias → caller uses its own list
    match ctx.shape {
        NodeShape::Diamond => Some(diamond_priority(dir, ctx.is_source)),
        // Add new shapes here.
        _ => None,
    }
}

/// Side-priority table for diamond nodes.
///
/// Vertical flow (TB/BT): incoming edges favour the axis-aligned poles (Top/Bottom);
/// outgoing branch edges favour the perpendicular poles (Right/Left).
/// Horizontal flow (LR/RL): roles are swapped.
///
/// BT/RL mirror TB/LR: the "near" pole is the one the main flow arrives from,
/// so for BT incoming that is Bottom, not Top.
fn diamond_priority(dir: Direction, is_source: bool) -> [Side; 4] {
    use Direction::*;
    use Side::*;
    match (dir, is_source) {
        // Vertical main flow — TB
        (TB, false) => [Top, Bottom, Right, Left],
        (TB, true) => [Right, Left, Top, Bottom],
        // Vertical main flow — BT (reversed: near pole is Bottom)
        (BT, false) => [Bottom, Top, Right, Left],
        (BT, true) => [Right, Left, Bottom, Top],
        // Horizontal main flow — LR
        (LR, false) => [Right, Left, Top, Bottom],
        (LR, true) => [Top, Bottom, Right, Left],
        // Horizontal main flow — RL (reversed: near pole is Left)
        (RL, false) => [Left, Right, Top, Bottom],
        (RL, true) => [Top, Bottom, Left, Right],
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_parser::Direction::*;
    use Side::*;

    fn ctx(shape: NodeShape, dir: Option<Direction>, is_source: bool) -> ShapePortContext {
        ShapePortContext {
            shape,
            effective_dir: dir,
            is_source,
        }
    }

    // ── bias disabled ────────────────────────────────────────────────────────

    #[test]
    fn bias_none_returns_none() {
        let result = shape_side_priority(&ctx(NodeShape::Diamond, None, false));
        assert!(result.is_none());
    }

    #[test]
    fn non_diamond_shape_returns_none() {
        let result = shape_side_priority(&ctx(NodeShape::Rectangle, Some(TB), false));
        assert!(result.is_none());
    }

    // ── diamond TB ───────────────────────────────────────────────────────────

    #[test]
    fn diamond_tb_incoming_prefers_top() {
        let p = shape_side_priority(&ctx(NodeShape::Diamond, Some(TB), false)).unwrap();
        assert_eq!(p[0], Top);
        assert_eq!(p[1], Bottom);
    }

    #[test]
    fn diamond_tb_outgoing_prefers_right() {
        let p = shape_side_priority(&ctx(NodeShape::Diamond, Some(TB), true)).unwrap();
        assert_eq!(p[0], Right);
        assert_eq!(p[1], Left);
    }

    // ── diamond BT ───────────────────────────────────────────────────────────

    #[test]
    fn diamond_bt_incoming_prefers_bottom() {
        let p = shape_side_priority(&ctx(NodeShape::Diamond, Some(BT), false)).unwrap();
        assert_eq!(p[0], Bottom);
        assert_eq!(p[1], Top);
    }

    #[test]
    fn diamond_bt_outgoing_prefers_right() {
        let p = shape_side_priority(&ctx(NodeShape::Diamond, Some(BT), true)).unwrap();
        assert_eq!(p[0], Right);
        assert_eq!(p[1], Left);
    }

    // ── diamond LR ───────────────────────────────────────────────────────────

    #[test]
    fn diamond_lr_incoming_prefers_right() {
        let p = shape_side_priority(&ctx(NodeShape::Diamond, Some(LR), false)).unwrap();
        assert_eq!(p[0], Right);
        assert_eq!(p[1], Left);
    }

    #[test]
    fn diamond_lr_outgoing_prefers_top() {
        let p = shape_side_priority(&ctx(NodeShape::Diamond, Some(LR), true)).unwrap();
        assert_eq!(p[0], Top);
        assert_eq!(p[1], Bottom);
    }

    // ── diamond RL ───────────────────────────────────────────────────────────

    #[test]
    fn diamond_rl_incoming_prefers_left() {
        let p = shape_side_priority(&ctx(NodeShape::Diamond, Some(RL), false)).unwrap();
        assert_eq!(p[0], Left);
        assert_eq!(p[1], Right);
    }

    #[test]
    fn diamond_rl_outgoing_prefers_top() {
        let p = shape_side_priority(&ctx(NodeShape::Diamond, Some(RL), true)).unwrap();
        assert_eq!(p[0], Top);
        assert_eq!(p[1], Bottom);
    }

    // ── all four sides present ───────────────────────────────────────────────

    #[test]
    fn diamond_priority_contains_all_four_sides() {
        for dir in [TB, BT, LR, RL] {
            for is_src in [false, true] {
                let p = shape_side_priority(&ctx(NodeShape::Diamond, Some(dir), is_src)).unwrap();
                let mut seen = [false; 4];
                for side in &p {
                    match side {
                        Top => seen[0] = true,
                        Bottom => seen[1] = true,
                        Left => seen[2] = true,
                        Right => seen[3] = true,
                    }
                }
                assert!(
                    seen.iter().all(|&s| s),
                    "missing side for dir={dir:?} is_src={is_src}"
                );
            }
        }
    }
}
