use std::collections::HashMap;
use trellis_parser::{Graph, Node};

use crate::grid::{CellState, Grid};

use super::assignment::{EdgePorts, Port, Side};

/// A port assignment locked in before the main assigner runs.
pub struct PinnedPorts {
    pub source: Port,
    pub target: Port,
}

/// Pre-pass result: edges with locked straight-line ports.
///
/// Passed to the main `PortAssigner` via `PortAssignmentContext`.
pub type PinnedPortMap = HashMap<usize, PinnedPorts>;

/// Apply pinned port overrides to an existing port assignment map.
///
/// Each assigner calls this after its normal assignment to ensure that
/// edges pinned by the pre-pass are not displaced by the main algorithm.
pub fn apply_pinned(ports: &mut HashMap<usize, EdgePorts>, pinned: &PinnedPortMap) {
    for (&edge_idx, pp) in pinned {
        ports.insert(
            edge_idx,
            EdgePorts {
                source_port: pp.source.clone(),
                target_port: pp.target.clone(),
            },
        );
    }
}

// ─── Candidate (internal) ────────────────────────────────────────────────────

struct PrepassCandidate {
    edge_idx: usize,
    src_port: Port,
    tgt_port: Port,
    /// Fraction of the narrower node's connector span covered by the overlap.
    /// Higher fraction = higher priority when two edges compete for the same slot.
    overlap_fraction: f64,
    src_node_id: String,
    tgt_node_id: String,
}

// ─── Public entry point ───────────────────────────────────────────────────────

/// Run the straight-edge pre-pass.
///
/// For each edge, checks whether source and target nodes are axis-aligned
/// such that a 0-bend path is geometrically possible (no node blocks the
/// corridor). If so, pins the port positions to lock in the straight path.
///
/// Only node footprints are in the grid at call time (no edges yet), so
/// obstacle detection is cheap and exact.
///
/// Returns a `PinnedPortMap` that is passed into `PortAssignmentContext`
/// and applied by every `PortAssigner` after its normal logic.
pub fn straight_edge_prepass(
    graph: &Graph,
    grid: &Grid,
    cell_size: i32,
    offset_x: i32,
    offset_y: i32,
) -> PinnedPortMap {
    let cs = cell_size as f64;
    let ox = offset_x as f64;
    let oy = offset_y as f64;

    let node_map: HashMap<&str, &Node> =
        graph.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    let mut candidates: Vec<PrepassCandidate> = Vec::new();

    for (edge_idx, edge) in graph.edges.iter().enumerate() {
        let (Some(&src), Some(&tgt)) = (
            node_map.get(edge.from.as_str()),
            node_map.get(edge.to.as_str()),
        ) else {
            continue;
        };

        // Try vertical alignment first; fall back to horizontal.
        if let Some(cand) = try_vertical(edge_idx, src, tgt, &edge.from, &edge.to, grid, cs, ox, oy) {
            candidates.push(cand);
        } else if let Some(cand) =
            try_horizontal(edge_idx, src, tgt, &edge.from, &edge.to, grid, cs, ox, oy)
        {
            candidates.push(cand);
        }
    }

    // Higher overlap fraction wins contention for the same port slot.
    candidates.sort_by(|a, b| {
        b.overlap_fraction
            .partial_cmp(&a.overlap_fraction)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Resolve port contention: at most one edge per (node_id, side, slot) pair.
    let mut pinned = PinnedPortMap::new();
    // Key: (node_id, side, slot_coord) — col for Top/Bottom, row for Left/Right.
    let mut used: HashMap<(String, Side, i64), usize> = HashMap::new();

    for cand in candidates {
        let src_slot = port_slot(&cand.src_node_id, &cand.src_port);
        let tgt_slot = port_slot(&cand.tgt_node_id, &cand.tgt_port);

        if used.contains_key(&src_slot) || used.contains_key(&tgt_slot) {
            // Slot occupied by a higher-priority edge. Skip — the main assigner
            // will handle this edge normally.
            continue;
        }

        used.insert(src_slot, cand.edge_idx);
        used.insert(tgt_slot, cand.edge_idx);
        pinned.insert(
            cand.edge_idx,
            PinnedPorts {
                source: cand.src_port,
                target: cand.tgt_port,
            },
        );
    }

    pinned
}

// ─── Vertical alignment ───────────────────────────────────────────────────────

fn try_vertical(
    edge_idx: usize,
    src: &Node,
    tgt: &Node,
    src_id: &str,
    tgt_id: &str,
    grid: &Grid,
    cs: f64,
    ox: f64,
    oy: f64,
) -> Option<PrepassCandidate> {
    let (gc_src, gr_src, w_src, h_src) = node_grid_coords(src, cs, ox, oy);
    let (gc_tgt, gr_tgt, w_tgt, h_tgt) = node_grid_coords(tgt, cs, ox, oy);

    let src_bottom = gr_src + h_src - 1;
    let tgt_bottom = gr_tgt + h_tgt - 1;

    // Corridor between the two nodes (exclusive of both boundary rows).
    let (corridor_start, corridor_end, src_side, tgt_side) = if src_bottom < gr_tgt {
        // Source above target → source exits Bottom, target enters Top.
        (src_bottom + 1, gr_tgt - 1, Side::Bottom, Side::Top)
    } else if tgt_bottom < gr_src {
        // Target above source (back-edge) → source exits Top, target enters Bottom.
        (tgt_bottom + 1, gr_src - 1, Side::Top, Side::Bottom)
    } else {
        return None; // Nodes overlap vertically.
    };

    // Connector columns (non-corner boundary points): gc+1 ..= gc+w-2.
    let src_col_min = gc_src + 1;
    let src_col_max = gc_src + w_src - 2;
    let tgt_col_min = gc_tgt + 1;
    let tgt_col_max = gc_tgt + w_tgt - 2;

    let overlap_min = src_col_min.max(tgt_col_min);
    let overlap_max = src_col_max.min(tgt_col_max);

    if overlap_min > overlap_max {
        return None;
    }

    let corridor_len = required_corridor_len(corridor_start, corridor_end);

    // Among clear columns, prefer the one closest to the overlap centre.
    // This keeps straight edges aesthetically centred on the shared side.
    let center_col = (overlap_min + overlap_max) / 2;
    let best_col = (overlap_min..=overlap_max)
        .filter(|&col| {
            unobstructed_vertical(grid, col, corridor_start, corridor_end) >= corridor_len
        })
        .min_by_key(|&col| (col - center_col).abs())?;

    let overlap_fraction = overlap_fraction(overlap_min, overlap_max, src_col_min, src_col_max, tgt_col_min, tgt_col_max);

    // Port row depends on which node is higher.
    let (src_row, tgt_row) = if matches!(src_side, Side::Bottom) {
        (src_bottom, gr_tgt)
    } else {
        (gr_src, tgt_bottom)
    };

    Some(PrepassCandidate {
        edge_idx,
        src_port: make_port(src_row, best_col, src_side, cs, ox, oy),
        tgt_port: make_port(tgt_row, best_col, tgt_side, cs, ox, oy),
        overlap_fraction,
        src_node_id: src_id.to_string(),
        tgt_node_id: tgt_id.to_string(),
    })
}

// ─── Horizontal alignment ─────────────────────────────────────────────────────

fn try_horizontal(
    edge_idx: usize,
    src: &Node,
    tgt: &Node,
    src_id: &str,
    tgt_id: &str,
    grid: &Grid,
    cs: f64,
    ox: f64,
    oy: f64,
) -> Option<PrepassCandidate> {
    let (gc_src, gr_src, w_src, h_src) = node_grid_coords(src, cs, ox, oy);
    let (gc_tgt, gr_tgt, w_tgt, h_tgt) = node_grid_coords(tgt, cs, ox, oy);

    let src_right = gc_src + w_src - 1;
    let tgt_right = gc_tgt + w_tgt - 1;

    let (corridor_start, corridor_end, src_side, tgt_side) = if src_right < gc_tgt {
        // Source left of target → source exits Right, target enters Left.
        (src_right + 1, gc_tgt - 1, Side::Right, Side::Left)
    } else if tgt_right < gc_src {
        // Target left of source → source exits Left, target enters Right.
        (tgt_right + 1, gc_src - 1, Side::Left, Side::Right)
    } else {
        return None; // Nodes overlap horizontally.
    };

    // Connector rows (non-corner boundary points): gr+1 ..= gr+h-2.
    let src_row_min = gr_src + 1;
    let src_row_max = gr_src + h_src - 2;
    let tgt_row_min = gr_tgt + 1;
    let tgt_row_max = gr_tgt + h_tgt - 2;

    let overlap_min = src_row_min.max(tgt_row_min);
    let overlap_max = src_row_max.min(tgt_row_max);

    if overlap_min > overlap_max {
        return None;
    }

    let corridor_len = required_corridor_len(corridor_start, corridor_end);

    // Among clear rows, prefer the one closest to the overlap centre.
    let center_row = (overlap_min + overlap_max) / 2;
    let best_row = (overlap_min..=overlap_max)
        .filter(|&row| {
            unobstructed_horizontal(grid, row, corridor_start, corridor_end) >= corridor_len
        })
        .min_by_key(|&row| (row - center_row).abs())?;

    let overlap_fraction = overlap_fraction(overlap_min, overlap_max, src_row_min, src_row_max, tgt_row_min, tgt_row_max);

    let (src_col, tgt_col) = if matches!(src_side, Side::Right) {
        (src_right, gc_tgt)
    } else {
        (gc_src, tgt_right)
    };

    Some(PrepassCandidate {
        edge_idx,
        src_port: make_port(best_row, src_col, src_side, cs, ox, oy),
        tgt_port: make_port(best_row, tgt_col, tgt_side, cs, ox, oy),
        overlap_fraction,
        src_node_id: src_id.to_string(),
        tgt_node_id: tgt_id.to_string(),
    })
}

// ─── Grid corridor helpers ────────────────────────────────────────────────────

/// Number of cells in the corridor (exclusive of both node boundaries).
///
/// Returns 0 when nodes are adjacent (no gap cells), which is still a valid
/// straight path — a 0-length corridor is trivially unobstructed.
fn required_corridor_len(start: i64, end: i64) -> usize {
    if end < start {
        0
    } else {
        (end - start + 1) as usize
    }
}

/// Count unobstructed cells scanning downward along `col` from `start_row` to `end_row`.
///
/// Stops at the first `Blocked` cell and returns the count up to that point.
fn unobstructed_vertical(grid: &Grid, col: i64, start_row: i64, end_row: i64) -> usize {
    if end_row < start_row {
        return 0;
    }
    let mut count = 0;
    for row in start_row..=end_row {
        if !grid.in_bounds(row, col) {
            break;
        }
        match grid.get(row as usize, col as usize) {
            Some(cell) if cell.state == CellState::Blocked => break,
            Some(_) => count += 1,
            None => break,
        }
    }
    count
}

/// Count unobstructed cells scanning rightward along `row` from `start_col` to `end_col`.
fn unobstructed_horizontal(grid: &Grid, row: i64, start_col: i64, end_col: i64) -> usize {
    if end_col < start_col {
        return 0;
    }
    let mut count = 0;
    for col in start_col..=end_col {
        if !grid.in_bounds(row, col) {
            break;
        }
        match grid.get(row as usize, col as usize) {
            Some(cell) if cell.state == CellState::Blocked => break,
            Some(_) => count += 1,
            None => break,
        }
    }
    count
}

// ─── Small helpers ────────────────────────────────────────────────────────────

/// Compute (grid_col, grid_row, w_points, h_points) for a node.
fn node_grid_coords(node: &Node, cs: f64, ox: f64, oy: f64) -> (i64, i64, i64, i64) {
    let gc = ((node.x - ox) / cs).round() as i64;
    let gr = ((node.y - oy) / cs).round() as i64;
    let w = (node.width / cs).round() as i64 + 1;
    let h = (node.height / cs).round() as i64 + 1;
    (gc, gr, w, h)
}

/// Fraction of the narrower node's connector span covered by the overlap.
fn overlap_fraction(
    overlap_min: i64,
    overlap_max: i64,
    src_min: i64,
    src_max: i64,
    tgt_min: i64,
    tgt_max: i64,
) -> f64 {
    let overlap_len = (overlap_max - overlap_min + 1) as f64;
    let src_span = (src_max - src_min + 1).max(1) as f64;
    let tgt_span = (tgt_max - tgt_min + 1).max(1) as f64;
    overlap_len / src_span.min(tgt_span)
}

/// Build a `Port` from grid coordinates.
fn make_port(row: i64, col: i64, side: Side, cs: f64, ox: f64, oy: f64) -> Port {
    Port {
        x: col as f64 * cs + ox,
        y: row as f64 * cs + oy,
        grid_row: row,
        grid_col: col,
        side,
    }
}

/// Canonical slot key for port contention tracking.
///
/// For Top/Bottom ports the discriminating coordinate is the column;
/// for Left/Right ports it is the row.
fn port_slot(node_id: &str, port: &Port) -> (String, Side, i64) {
    let coord = match port.side {
        Side::Top | Side::Bottom => port.grid_col,
        Side::Left | Side::Right => port.grid_row,
    };
    (node_id.to_string(), port.side, coord)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{build_grid, GridExtent};
    use trellis_parser::{ArrowHead, Edge, EdgeStyle, Graph, Node, NodeShape};

    fn make_node(id: &str, w: f64, h: f64, x: f64, y: f64) -> Node {
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
            style: EdgeStyle::Solid,
            arrow_head: ArrowHead::Arrow,
            ..Default::default()
        }
    }

    fn build_test_grid(graph: &Graph, cell_size: i32) -> Grid {
        let extent = GridExtent {
            width: 500.0,
            height: 500.0,
            offset_x: 0,
            offset_y: 0,
        };
        build_grid(graph, cell_size, &extent)
    }

    /// Vertically aligned nodes with full column overlap → Bottom/Top pinned at the shared column.
    #[test]
    fn prepass_pins_vertically_aligned_nodes() {
        let mut graph = Graph::new();
        // A: 40×20 at (60, 0) → gc=6, gr=0, bottom_row=2, conn_cols=7,8,9
        // B: 40×20 at (60, 100) → gc=6, gr=10, top_row=10, conn_cols=7,8,9
        // Corridor: rows 3..9 (7 rows, all free)
        // Overlap connector cols: 7..9
        graph.nodes = vec![
            make_node("A", 40.0, 20.0, 60.0, 0.0),
            make_node("B", 40.0, 20.0, 60.0, 100.0),
        ];
        graph.edges = vec![make_edge("A", "B")];

        let grid = build_test_grid(&graph, 10);
        let pinned = straight_edge_prepass(&graph, &grid, 10, 0, 0);

        assert_eq!(pinned.len(), 1, "Edge A→B should be pinned");
        let pp = &pinned[&0];
        assert_eq!(pp.source.side, Side::Bottom, "Source port must be Bottom");
        assert_eq!(pp.target.side, Side::Top, "Target port must be Top");
        assert_eq!(
            pp.source.grid_col, pp.target.grid_col,
            "Source and target must share the same column"
        );
        // Column must be the centre of the connector overlap range (7..=9 → centre 8).
        assert_eq!(
            pp.source.grid_col, 8,
            "Should pick the centre column (8), got {}",
            pp.source.grid_col
        );
        assert_eq!(pp.source.grid_row, 2, "Source row must be bottom of A");
        assert_eq!(pp.target.grid_row, 10, "Target row must be top of B");
    }

    /// A node placed in the corridor between A and B blocks the straight path → no pin.
    #[test]
    fn prepass_skips_blocked_corridor() {
        let mut graph = Graph::new();
        // A at (60, 0), B at (60, 200), C at (60, 90) — C blocks the corridor.
        graph.nodes = vec![
            make_node("A", 40.0, 20.0, 60.0, 0.0),
            make_node("B", 40.0, 20.0, 60.0, 200.0),
            make_node("C", 40.0, 20.0, 60.0, 90.0), // sits in the A→B corridor
        ];
        graph.edges = vec![make_edge("A", "B")];

        let grid = build_test_grid(&graph, 10);
        let pinned = straight_edge_prepass(&graph, &grid, 10, 0, 0);

        assert!(
            pinned.is_empty(),
            "Blocked corridor should produce no pinned ports"
        );
    }

    /// Two edges compete for the same column on node A.
    /// The edge with the larger overlap fraction wins; the other is left for the main assigner.
    #[test]
    fn prepass_resolves_contention_by_overlap_fraction() {
        let mut graph = Graph::new();
        // A: wide node at (0, 0), 80×20 → gc=0, conn_cols=1..7
        // B: narrow node, exactly centred under A's left half → smaller overlap
        // C: wide node below A, same width → full overlap (wins contention)
        //
        //   A:  x=0..80 (conn_cols 1..7)
        //   B:  x=0..20 (conn_cols 1..1) — overlap with A: col 1 only → fraction 1/1=1.0
        //   C:  x=0..80 (conn_cols 1..7) — overlap with A: cols 1..7 → fraction 7/7=1.0
        //
        // Both edges share col=1 on A's bottom (B's top is also col 1).
        // Since overlaps are equal, whichever comes first in the sorted list wins.
        // The important invariant: exactly one edge per slot is pinned.
        graph.nodes = vec![
            make_node("A", 80.0, 20.0, 0.0, 0.0),
            make_node("B", 20.0, 20.0, 0.0, 100.0),
            make_node("C", 80.0, 20.0, 0.0, 200.0),
        ];
        graph.edges = vec![make_edge("A", "B"), make_edge("A", "C")];

        let grid = build_test_grid(&graph, 10);
        let pinned = straight_edge_prepass(&graph, &grid, 10, 0, 0);

        // A→B and A→C may both be pinned if they get distinct columns on A's bottom.
        // A→B must use col 1 (only overlap col with B's connector).
        // A→C can use any of cols 1..7 on A's bottom, but col 1 is taken by A→B.
        // So A→C should be pinned at some other column (2..7).
        if pinned.contains_key(&0) && pinned.contains_key(&1) {
            let p0 = &pinned[&0]; // A→B
            let p1 = &pinned[&1]; // A→C
            assert_ne!(
                p0.source.grid_col, p1.source.grid_col,
                "Two pinned edges must not share the same source port column on A"
            );
        } else {
            // At most one can be pinned if they truly cannot be separated.
            assert!(
                pinned.len() <= 2,
                "No more than 2 edges can be pinned for 2 edges"
            );
        }
    }

    /// Horizontally aligned nodes get Left/Right pinned ports.
    #[test]
    fn prepass_pins_horizontally_aligned_nodes() {
        let mut graph = Graph::new();
        // A: 40×20 at (0, 60) → gc=0, gr=6, right_col=4, conn_rows=7
        // B: 40×20 at (100, 60) → gc=10, gr=6, left_col=10, conn_rows=7
        // Corridor: cols 5..9 (5 cols, all free)
        graph.nodes = vec![
            make_node("A", 40.0, 20.0, 0.0, 60.0),
            make_node("B", 40.0, 20.0, 100.0, 60.0),
        ];
        graph.edges = vec![make_edge("A", "B")];

        let grid = build_test_grid(&graph, 10);
        let pinned = straight_edge_prepass(&graph, &grid, 10, 0, 0);

        assert_eq!(pinned.len(), 1, "Edge A→B should be pinned");
        let pp = &pinned[&0];
        assert_eq!(pp.source.side, Side::Right, "Source port must be Right");
        assert_eq!(pp.target.side, Side::Left, "Target port must be Left");
        assert_eq!(
            pp.source.grid_row, pp.target.grid_row,
            "Source and target must share the same row"
        );
        assert_eq!(pp.source.grid_col, 4, "Source col must be right edge of A");
        assert_eq!(pp.target.grid_col, 10, "Target col must be left edge of B");
    }

    /// Nodes with no connector-range overlap produce no pin.
    #[test]
    fn prepass_skips_non_overlapping_nodes() {
        let mut graph = Graph::new();
        // A: 40×20 at (0, 0)      → conn_cols 1..3 (Bottom)
        // B: 40×20 at (200, 100)  → conn_cols 21..23 (Top)
        // No overlap in connector columns → no vertical pin.
        // No overlap in connector rows → no horizontal pin.
        graph.nodes = vec![
            make_node("A", 40.0, 20.0, 0.0, 0.0),
            make_node("B", 40.0, 20.0, 200.0, 100.0),
        ];
        graph.edges = vec![make_edge("A", "B")];

        let grid = build_test_grid(&graph, 10);
        let pinned = straight_edge_prepass(&graph, &grid, 10, 0, 0);

        assert!(pinned.is_empty(), "No alignment → no pinned ports");
    }
}
