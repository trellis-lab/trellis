use std::collections::{BTreeSet, HashMap};
use trellis_parser::Node;

use super::assignment::{EdgePorts, Port, Side};
use super::common::{
    angle_to_side, angle_to_side_flow_aware, calculate_angle, enumerate_connectors, Connector,
};
use super::prepass::apply_pinned;
use super::{effective_direction, PortAssigner, PortAssignmentContext};

/// Trellis Basic port assignment algorithm.
///
/// Assigns each edge's port by finding the node face geometrically closest
/// to the other node's center, then greedily claiming the nearest free
/// connector from the center of that face outward. Ports are booked
/// sequentially so later edges see earlier claims.
pub struct TrellisBasicAssigner;

impl PortAssigner for TrellisBasicAssigner {
    fn assign_ports(&self, ctx: &PortAssignmentContext) -> HashMap<usize, EdgePorts> {
        assign_trellis_basic(ctx)
    }
}

// ─── Corner enum ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Corner {
    Tl,
    Tr,
    Bl,
    Br,
}

fn corner_rank(c: Corner) -> u8 {
    match c {
        Corner::Tl => 0,
        Corner::Tr => 1,
        Corner::Bl => 2,
        Corner::Br => 3,
    }
}

/// Two sides that share the given corner.
fn corner_sides(c: Corner) -> (Side, Side) {
    match c {
        Corner::Tl => (Side::Top, Side::Left),
        Corner::Tr => (Side::Top, Side::Right),
        Corner::Bl => (Side::Bottom, Side::Left),
        Corner::Br => (Side::Bottom, Side::Right),
    }
}

/// The two corners of `side` in connector-index order (index 0, index N-1).
fn corners_of_side(side: Side) -> (Corner, Corner) {
    match side {
        Side::Top => (Corner::Tl, Corner::Tr),
        Side::Bottom => (Corner::Bl, Corner::Br),
        Side::Left => (Corner::Tl, Corner::Bl),
        Side::Right => (Corner::Tr, Corner::Br),
    }
}

/// Outward unit normal for each side.
fn side_normal(side: Side) -> (f64, f64) {
    match side {
        Side::Top => (0.0, -1.0),
        Side::Bottom => (0.0, 1.0),
        Side::Left => (-1.0, 0.0),
        Side::Right => (1.0, 0.0),
    }
}

// ─── Geometry helpers ─────────────────────────────────────────────────────────

/// Squared Euclidean distance from each corner of `node` to `(o_cx, o_cy)`.
/// Returns an unsorted array for direct lookup by corner.
fn compute_corner_dists(node: &Node, o_cx: f64, o_cy: f64) -> [(Corner, f64); 4] {
    let x0 = node.x;
    let x1 = node.x + node.width;
    let y0 = node.y;
    let y1 = node.y + node.height;
    [
        (Corner::Tl, (x0 - o_cx).powi(2) + (y0 - o_cy).powi(2)),
        (Corner::Tr, (x1 - o_cx).powi(2) + (y0 - o_cy).powi(2)),
        (Corner::Bl, (x0 - o_cx).powi(2) + (y1 - o_cy).powi(2)),
        (Corner::Br, (x1 - o_cx).powi(2) + (y1 - o_cy).powi(2)),
    ]
}

fn lookup_corner_dist(corner: Corner, dists: &[(Corner, f64); 4]) -> f64 {
    dists
        .iter()
        .find(|(c, _)| *c == corner)
        .map_or(f64::MAX, |(_, d)| *d)
}

/// Select the primary side for `corner` given the N→O direction vector.
///
/// Uses dot product with outward normals. Diagonal tie (equal scores) is
/// broken by `angle_to_side_flow_aware` when a flow direction is active,
/// otherwise `angle_to_side` with uniform 90° sectors.
fn primary_side_for_corner(
    corner: Corner,
    n_to_o: (f64, f64),
    flow_dir: Option<trellis_parser::Direction>,
) -> Side {
    let (sa, sb) = corner_sides(corner);
    let na = side_normal(sa);
    let nb = side_normal(sb);
    let score_a = na.0 * n_to_o.0 + na.1 * n_to_o.1;
    let score_b = nb.0 * n_to_o.0 + nb.1 * n_to_o.1;
    if score_a > score_b {
        sa
    } else if score_b > score_a {
        sb
    } else {
        // Diagonal tie: use flow-aware sectors when a direction is active,
        // uniform sectors otherwise.
        let angle = n_to_o.1.atan2(n_to_o.0).to_degrees();
        let angle = ((angle % 360.0) + 360.0) % 360.0;
        match flow_dir {
            Some(dir) => angle_to_side_flow_aware(angle, dir),
            None => angle_to_side(angle),
        }
    }
}

/// Build priority side list from `sorted_corners` (closest first).
///
/// Each corner contributes its primary side; duplicates are dropped so the
/// list contains at most 4 distinct sides in closest-to-farthest order.
fn build_priority_sides(
    sorted_corners: &[(Corner, f64)],
    n_to_o: (f64, f64),
    flow_dir: Option<trellis_parser::Direction>,
) -> Vec<Side> {
    let mut result: Vec<Side> = Vec::with_capacity(4);
    for (corner, _) in sorted_corners {
        let side = primary_side_for_corner(*corner, n_to_o, flow_dir);
        if !result.contains(&side) {
            result.push(side);
        }
    }
    result
}

/// For `side`, return `(near_idx, far_idx)` where `near_idx` is the connector
/// index of the corner of `side` closest to O, and `far_idx` is the opposite.
///
/// `corners_of_side` maps to indices (0, n-1) in the connector list.
fn near_far_end_indices(side: Side, dists: &[(Corner, f64); 4], n: usize) -> (usize, usize) {
    debug_assert!(n > 0);
    let (c0, c1) = corners_of_side(side);
    let d0 = lookup_corner_dist(c0, dists);
    let d1 = lookup_corner_dist(c1, dists);
    // c0 maps to index 0, c1 maps to index n-1
    if d0 <= d1 {
        (0, n - 1) // c0 is near (idx 0), c1 is far (idx n-1)
    } else {
        (n - 1, 0) // c1 is near (idx n-1), c0 is far (idx 0)
    }
}

// ─── Connector search ─────────────────────────────────────────────────────────

/// Walk connectors from `from_idx` to `to_idx` (inclusive) and return the
/// index of the first connector not in `claimed`. Returns `None` if all are
/// claimed.
fn first_unclaimed(
    connectors: &[Connector],
    from_idx: usize,
    to_idx: usize,
    claimed: &BTreeSet<(i64, i64)>,
) -> Option<usize> {
    if connectors.is_empty() {
        return None;
    }
    let step: i64 = if to_idx >= from_idx { 1 } else { -1 };
    let mut i = from_idx as i64;
    loop {
        let c = &connectors[i as usize];
        if !claimed.contains(&(c.grid_row, c.grid_col)) {
            return Some(i as usize);
        }
        if i == to_idx as i64 {
            return None;
        }
        i += step;
    }
}

/// Find the best connector for this edge-endpoint using the priority side list.
///
/// For each side (in priority order):
///   1. Primary sweep: center → near end
///   2. Fallback 1:   center → far end
///
/// If all priority sides are full, falls back to a linear scan of all sides.
/// Returns `(side, connector)` — always succeeds for graphs with reasonable
/// node sizes.
fn find_connector(
    node: &Node,
    priority_sides: &[Side],
    dists: &[(Corner, f64); 4],
    claimed: &BTreeSet<(i64, i64)>,
    cell_size: i32,
    offset_x: i32,
    offset_y: i32,
) -> (Side, Connector) {
    for &side in priority_sides {
        let connectors = enumerate_connectors(node, side, cell_size, offset_x, offset_y);
        if connectors.is_empty() {
            continue;
        }
        let n = connectors.len();
        let center_idx = n / 2;
        let (near_idx, far_idx) = near_far_end_indices(side, dists, n);

        // Primary sweep: center → near end
        if let Some(idx) = first_unclaimed(&connectors, center_idx, near_idx, claimed) {
            return (side, connectors[idx].clone());
        }

        // Fallback 1: center → far end (center already claimed, naturally skipped)
        if let Some(idx) = first_unclaimed(&connectors, center_idx, far_idx, claimed) {
            return (side, connectors[idx].clone());
        }
    }

    // Fallback 2: linear scan of all four sides
    for &side in &[Side::Top, Side::Right, Side::Bottom, Side::Left] {
        let connectors = enumerate_connectors(node, side, cell_size, offset_x, offset_y);
        for conn in connectors {
            if !claimed.contains(&(conn.grid_row, conn.grid_col)) {
                return (side, conn);
            }
        }
    }

    // Ultimate last resort (pathological case: all connectors claimed)
    let connectors = enumerate_connectors(node, Side::Top, cell_size, offset_x, offset_y);
    let idx = connectors.len() / 2;
    (Side::Top, connectors[idx].clone())
}

// ─── Main algorithm ───────────────────────────────────────────────────────────

fn assign_trellis_basic(ctx: &PortAssignmentContext) -> HashMap<usize, EdgePorts> {
    let graph = ctx.graph;
    let cell_size = ctx.cell_size;
    let offset_x = ctx.offset_x;
    let offset_y = ctx.offset_y;
    let flow_dir = effective_direction(ctx);

    let node_map: HashMap<&str, &Node> =
        graph.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    // Per-node claimed connector sets (BTreeSet for determinism).
    let mut claimed: HashMap<String, BTreeSet<(i64, i64)>> = HashMap::new();

    // Pre-claim connectors locked by the straight-edge pre-pass.
    let mut skip_set: BTreeSet<usize> = BTreeSet::new();
    for (&edge_idx, pp) in &ctx.pinned_ports {
        let edge = &graph.edges[edge_idx];
        claimed
            .entry(edge.from.clone())
            .or_default()
            .insert((pp.source.grid_row, pp.source.grid_col));
        claimed
            .entry(edge.to.clone())
            .or_default()
            .insert((pp.target.grid_row, pp.target.grid_col));
        skip_set.insert(edge_idx);
    }

    // Collect per-node work items: (edge_idx, other_node_id, is_source, angle).
    // Using owned Strings to avoid lifetime entanglement with `claimed`.
    let mut per_node: HashMap<String, Vec<(usize, String, bool, f64)>> = HashMap::new();
    for (edge_idx, edge) in graph.edges.iter().enumerate() {
        if skip_set.contains(&edge_idx) {
            continue;
        }
        if let (Some(src), Some(tgt)) = (
            node_map.get(edge.from.as_str()),
            node_map.get(edge.to.as_str()),
        ) {
            let angle_src = calculate_angle(src, tgt);
            let angle_tgt = calculate_angle(tgt, src);
            per_node
                .entry(edge.from.clone())
                .or_default()
                .push((edge_idx, edge.to.clone(), true, angle_src));
            per_node
                .entry(edge.to.clone())
                .or_default()
                .push((edge_idx, edge.from.clone(), false, angle_tgt));
        }
    }

    // Sort each node's work items: angle ascending, then edge_idx ascending.
    for items in per_node.values_mut() {
        items.sort_by(|a, b| {
            a.3.partial_cmp(&b.3)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
    }

    // Process nodes in deterministic order.
    let mut node_ids: Vec<String> = per_node.keys().cloned().collect();
    node_ids.sort_unstable();

    let mut result: HashMap<usize, EdgePorts> = HashMap::new();

    for node_id in &node_ids {
        let node = match node_map.get(node_id.as_str()) {
            Some(n) => n,
            None => continue,
        };

        let n_cx = node.x + node.width / 2.0;
        let n_cy = node.y + node.height / 2.0;

        // Clone items so we can mutate `claimed` inside the loop.
        let items = per_node[node_id].clone();

        for (edge_idx, other_id, is_source, _angle) in items {
            let other = match node_map.get(other_id.as_str()) {
                Some(n) => n,
                None => continue,
            };

            let o_cx = other.x + other.width / 2.0;
            let o_cy = other.y + other.height / 2.0;

            // Step 1: corner distances.
            let dists = compute_corner_dists(node, o_cx, o_cy);

            // Sort a copy for priority-list construction.
            let mut sorted_dists = dists;
            sorted_dists.sort_by(|a, b| {
                a.1.partial_cmp(&b.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| corner_rank(a.0).cmp(&corner_rank(b.0)))
            });

            // N→O direction vector (degenerate self-loop defaults to rightward).
            let dx = o_cx - n_cx;
            let dy = o_cy - n_cy;
            let n_to_o = if dx == 0.0 && dy == 0.0 {
                (1.0, 0.0)
            } else {
                (dx, dy)
            };

            // Step 2: build priority side list.
            let priority_sides = build_priority_sides(&sorted_dists, n_to_o, flow_dir);

            // Steps 3-4: find the best free connector.
            let node_claimed = claimed.entry(node_id.clone()).or_default();
            let (side, conn) = find_connector(
                node,
                &priority_sides,
                &dists,
                node_claimed,
                cell_size,
                offset_x,
                offset_y,
            );
            node_claimed.insert((conn.grid_row, conn.grid_col));

            let port = Port {
                x: conn.x,
                y: conn.y,
                grid_row: conn.grid_row,
                grid_col: conn.grid_col,
                side,
            };

            let entry = result.entry(edge_idx).or_insert_with(|| EdgePorts {
                source_port: Port {
                    x: 0.0,
                    y: 0.0,
                    grid_row: 0,
                    grid_col: 0,
                    side: Side::Top,
                },
                target_port: Port {
                    x: 0.0,
                    y: 0.0,
                    grid_row: 0,
                    grid_col: 0,
                    side: Side::Top,
                },
            });

            if is_source {
                entry.source_port = port;
            } else {
                entry.target_port = port;
            }
        }
    }

    apply_pinned(&mut result, &ctx.pinned_ports);
    result
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FlowBias;
    use crate::grid::Grid;
    use crate::ports::{PinnedPortMap, PinnedPorts, PortAssignmentContext};
    use trellis_parser::{ArrowHead, Edge, EdgeStyle, Graph, Node, NodeShape};

    fn make_rect(id: &str, x: f64, y: f64, w: f64, h: f64) -> Node {
        Node {
            id: id.to_string(),
            label: id.to_string(),
            shape: NodeShape::Rectangle,
            width: w,
            height: h,
            x,
            y,
            ..Default::default()
        }
    }

    fn make_edge(from: &str, to: &str) -> Edge {
        Edge {
            from: from.to_string(),
            to: to.to_string(),
            label: None,
            style: EdgeStyle::Solid,
            arrow_head: ArrowHead::Arrow,
            ..Default::default()
        }
    }

    fn make_ctx<'a>(
        graph: &'a Graph,
        grid: &'a Grid,
        pinned_ports: PinnedPortMap,
    ) -> PortAssignmentContext<'a> {
        PortAssignmentContext {
            graph,
            cell_size: 10,
            offset_x: 0,
            offset_y: 0,
            grid,
            pinned_ports,
            flow_bias: FlowBias::None,
            topo_rank: HashMap::new(),
            print_metrics: false,
        }
    }

    /// B directly to the right of A → source port on Right side of A,
    /// target port on Left side of B.
    #[test]
    fn direct_right_gets_right_side() {
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_rect("A", 10.0, 80.0, 40.0, 20.0),
            make_rect("B", 210.0, 80.0, 40.0, 20.0),
        ];
        graph.edges = vec![make_edge("A", "B")];

        let grid = Grid::new(50, 50, 10, 0, 0);
        let ctx = make_ctx(&graph, &grid, HashMap::new());
        let ports = TrellisBasicAssigner.assign_ports(&ctx);

        assert_eq!(ports.len(), 1);
        assert_eq!(ports[&0].source_port.side, Side::Right, "A→B source should exit Right");
        assert_eq!(ports[&0].target_port.side, Side::Left, "A→B target should enter Left");
    }

    /// Single edge where B is directly above A: port assigned at center index
    /// of the Top side (not displaced to an edge).
    #[test]
    fn center_port_preferred() {
        // A: 40×20 at (60,80); center=(80,90).  B: 40×20 at (60,0); center=(80,10).
        // B is directly above A → Top side selected.
        // Top connectors (cell_size=10, gc=6, gr=8, w_pts=5):
        //   cols 7,8,9 → center_idx=1 → col 8.
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_rect("A", 60.0, 80.0, 40.0, 20.0),
            make_rect("B", 60.0, 0.0, 40.0, 20.0),
        ];
        graph.edges = vec![make_edge("A", "B")];

        let grid = Grid::new(50, 50, 10, 0, 0);
        let ctx = make_ctx(&graph, &grid, HashMap::new());
        let ports = TrellisBasicAssigner.assign_ports(&ctx);

        let src = &ports[&0].source_port;
        assert_eq!(src.side, Side::Top);
        // Center col for A top side: gc=6, w_pts=5 → connectors cols 7..9, center=col 8
        assert_eq!(src.grid_col, 8, "center connector should be at col 8");
    }

    /// Two edges both going upward from A to B1, B2. First edge claims center;
    /// second is displaced to an adjacent slot.
    #[test]
    fn second_edge_displaced_from_center() {
        // A: 60×20 at (0,80); Top connectors: gc=0, gr=8, w_pts=7 → cols 1..5 (5 connectors).
        // center_idx=2 → col 3.
        // B1 directly above left  → angle≈270° → first processed (smaller angle wins if equal).
        // B2 directly above right → same angle class, both Top, second claims non-center slot.
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_rect("A", 0.0, 80.0, 60.0, 20.0),   // center (30,90)
            make_rect("B1", 0.0, 0.0, 20.0, 20.0),    // center (10,10) — upper-left
            make_rect("B2", 40.0, 0.0, 20.0, 20.0),   // center (50,10) — upper-right
        ];
        // edge 0: A→B1, edge 1: A→B2
        graph.edges = vec![make_edge("A", "B1"), make_edge("A", "B2")];

        let grid = Grid::new(50, 50, 10, 0, 0);
        let ctx = make_ctx(&graph, &grid, HashMap::new());
        let ports = TrellisBasicAssigner.assign_ports(&ctx);

        let col0 = ports[&0].source_port.grid_col;
        let col1 = ports[&1].source_port.grid_col;

        // Both should be on the Top side
        assert_eq!(ports[&0].source_port.side, Side::Top);
        assert_eq!(ports[&1].source_port.side, Side::Top);

        // The two ports must be at different columns
        assert_ne!(col0, col1, "two edges must not share the same connector");
    }

    /// B is at upper-right of A → TR corner closest → primary walk on Right side
    /// goes toward TR (top end, index 0). With center claimed, next slot is above center.
    #[test]
    fn near_corner_direction() {
        // A: 40×60 at (10,80); center=(30,110).
        // Right connectors: gc=1,gr=8,w_pts=5,h_pts=7 → right_col=5, rows 9..13 (5 slots).
        // center_idx=2 → row 11.
        //
        // B1 at (210,100,40,20): center=(230,110) — exactly horizontal from A center.
        //   angle(A→B1) = atan2(0,200) = 0°. Processed first.
        //   d(TR)=(50-230)²+(80-110)²=32400+900=33300
        //   d(BR)=(50-230)²+(140-110)²=32400+900=33300  → tie → near=0(TR), far=4(BR)
        //   Primary sweep center→near: [2,1,0] → row 11 (center) free → assigned row 11.
        //
        // B2 at (210,40,40,20): center=(230,50) — upper-right of A.
        //   angle(A→B2) = atan2(-60,200) ≈ 343.3°. Processed second.
        //   d(TR)=33300, d(BR)=40500 → TR nearer → near=0(TR end), far=4(BR end).
        //   Primary sweep center→near: [2(claimed),1,0] → row 10 → assigned (above center).
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_rect("A", 10.0, 80.0, 40.0, 60.0),   // center (30,110)
            make_rect("B1", 210.0, 100.0, 40.0, 20.0), // exactly right → angle 0° → processed first
            make_rect("B2", 210.0, 40.0, 40.0, 20.0),  // upper-right   → angle 343° → processed second
        ];
        graph.edges = vec![make_edge("A", "B1"), make_edge("A", "B2")];

        let grid = Grid::new(50, 50, 10, 0, 0);
        let ctx = make_ctx(&graph, &grid, HashMap::new());
        let ports = TrellisBasicAssigner.assign_ports(&ctx);

        let row_b1 = ports[&0].source_port.grid_row;
        let row_b2 = ports[&1].source_port.grid_row;

        assert_eq!(ports[&0].source_port.side, Side::Right);
        assert_eq!(ports[&1].source_port.side, Side::Right);
        // B1 (direct right, angle 0°, processed first) gets center (row 11)
        assert_eq!(row_b1, 11, "B1 should get center row");
        // B2 (upper-right) walks toward TR end → assigned above center (row < 11)
        assert!(row_b2 < row_b1, "B2 (upper-right) should be assigned above center (toward TR)");
    }

    /// When the primary side is at capacity, the algorithm spills to the next
    /// priority side.
    #[test]
    fn full_primary_side_spills_to_next() {
        // A: 40×20 at (0,0). Right side: gc=0, gr=0, w_pts=5, h_pts=3 → rows 1..1 → 1 connector.
        // B1 directly right  → claims the single Right connector.
        // B2 also right-ish  → Right full → spills to Top or Bottom.
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_rect("A", 0.0, 0.0, 40.0, 20.0),    // center (20,10)
            make_rect("B1", 200.0, 0.0, 40.0, 20.0),  // directly right
            make_rect("B2", 200.0, 20.0, 40.0, 20.0), // slightly below-right
        ];
        graph.edges = vec![make_edge("A", "B1"), make_edge("A", "B2")];

        let grid = Grid::new(50, 50, 10, 0, 0);
        let ctx = make_ctx(&graph, &grid, HashMap::new());
        let ports = TrellisBasicAssigner.assign_ports(&ctx);

        // Both edges must get distinct connectors
        let s0 = &ports[&0].source_port;
        let s1 = &ports[&1].source_port;
        assert_ne!(
            (s0.grid_row, s0.grid_col),
            (s1.grid_row, s1.grid_col),
            "spilled edge must not reuse the full side's connector"
        );
    }

    /// Diamond nodes have exactly one connector per side (the visual tip).
    /// The algorithm must still assign a port without panicking.
    #[test]
    fn diamond_single_port_per_side() {
        let mut graph = Graph::new();
        graph.nodes = vec![
            Node {
                id: "A".into(),
                label: "A".into(),
                shape: NodeShape::Diamond,
                width: 40.0,
                height: 40.0,
                x: 0.0,
                y: 0.0,
                ..Default::default()
            },
            make_rect("B", 200.0, 0.0, 40.0, 20.0),
        ];
        graph.edges = vec![make_edge("A", "B")];

        let grid = Grid::new(50, 50, 10, 0, 0);
        let ctx = make_ctx(&graph, &grid, HashMap::new());
        let ports = TrellisBasicAssigner.assign_ports(&ctx);

        assert_eq!(ports.len(), 1);
        // B is to the right of A → Right side of diamond
        assert_eq!(ports[&0].source_port.side, Side::Right);
    }

    /// Pre-pass pinned connectors must not be reassigned to other edges.
    #[test]
    fn pinned_ports_not_reassigned() {
        // Two edges from A. One is pinned by the pre-pass. The other must get a
        // different connector, not the pinned one.
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_rect("A", 0.0, 80.0, 40.0, 20.0), // center (20,90)
            make_rect("B", 0.0, 0.0, 40.0, 20.0),  // above A
            make_rect("C", 0.0, 0.0, 40.0, 20.0),  // also above A (same position for test)
        ];
        graph.edges = vec![make_edge("A", "B"), make_edge("A", "C")];

        // Pin edge 0 to a specific connector on the Top side of A.
        // A top side: gc=(0-0)/10=0, gr=8, w_pts=5 → connectors cols 1..3 → center col 2.
        let pinned_src = Port {
            x: 20.0,
            y: 80.0,
            grid_row: 8,
            grid_col: 2,
            side: Side::Top,
        };
        let pinned_tgt = Port {
            x: 20.0,
            y: 20.0,
            grid_row: 2,
            grid_col: 2,
            side: Side::Bottom,
        };
        let mut pinned: PinnedPortMap = HashMap::new();
        pinned.insert(
            0,
            PinnedPorts {
                source: pinned_src.clone(),
                target: pinned_tgt,
            },
        );

        let grid = Grid::new(50, 50, 10, 0, 0);
        let ctx = make_ctx(&graph, &grid, pinned);
        let ports = TrellisBasicAssigner.assign_ports(&ctx);

        // Edge 0 keeps its pinned port
        assert_eq!(ports[&0].source_port.grid_col, pinned_src.grid_col);
        assert_eq!(ports[&0].source_port.grid_row, pinned_src.grid_row);

        // Edge 1 must not steal the pinned connector
        let s1 = &ports[&1].source_port;
        assert_ne!(
            (s1.grid_row, s1.grid_col),
            (pinned_src.grid_row, pinned_src.grid_col),
            "edge 1 must not be assigned the pinned connector"
        );
    }
}
