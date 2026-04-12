use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use trellis_core::{
    grid::Grid,
    ports::{EdgePorts, Side},
    routing::{astar::RoutedPath, RoutingResult},
};
use trellis_parser::{ast::Edge, Graph};

// --- Thresholds ---------------------------------------------------------------

/// Detour factor at or above which the edge is flagged `high_detour`.
/// 2.0 means the routed path is at least twice the Manhattan distance.
const DETOUR_THRESHOLD: f64 = 2.0;

/// Bend count at or above which the edge is flagged `excessive_bends`.
const BEND_THRESHOLD: usize = 4;

// --- Output types -------------------------------------------------------------

/// Quality assessment of a single routed edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeQuality {
    /// Human-readable edge identifier: `"source-->target"`.
    pub edge_id: String,
    pub source: String,
    pub target: String,
    /// Number of direction changes in the routed path.
    pub bends: usize,
    /// `actual_path_length / manhattan_distance`.  1.0 is theoretically optimal;
    /// values > 1 indicate detour due to obstacles or congestion.
    /// 1.0 when the manhattan distance is zero (source == target, self-loop).
    pub detour_factor: f64,
    /// Number of grid cells on this edge's path that are marked as crossing points.
    pub crossings: usize,
    /// Which side of the source node this edge departs from.
    pub port_side_source: String,
    /// Which side of the target node this edge arrives at.
    pub port_side_target: String,
    /// Number of grid steps in the routed path (number of cells − 1).
    pub path_length: usize,
    /// Straight-line (Manhattan) distance in grid steps between source and target ports.
    pub manhattan_distance: usize,
    /// Composite quality score in the range [0.0, 1.0].  1.0 = perfect; 0.0 = worst.
    pub quality_score: f64,
    /// Human-readable flags identifying specific quality problems.
    /// Possible values: `"high_detour"`, `"excessive_bends"`, `"avoidable_crossing"`.
    pub flags: Vec<String>,
}

// --- Public API ---------------------------------------------------------------

/// Score a single routed edge.
///
/// * `edge_idx` — index of the edge in `graph.edges`; used to look up cells in the grid.
/// * `edge`     — the parsed edge (provides `from`/`to` node IDs).
/// * `path`     — the committed routed path for this edge.
/// * `ports`    — source and target port assignments.
/// * `grid`     — the final committed routing grid (used to count crossing cells).
pub fn score_edge(
    edge_idx: usize,
    edge: &Edge,
    path: &RoutedPath,
    ports: &EdgePorts,
    grid: &Grid,
) -> EdgeQuality {
    let path_length = path.points.len().saturating_sub(1);
    let manhattan_distance = ((ports.source_port.grid_row - ports.target_port.grid_row)
        .unsigned_abs()
        + (ports.source_port.grid_col - ports.target_port.grid_col).unsigned_abs())
        as usize;

    let detour_factor = if manhattan_distance > 0 {
        path_length as f64 / manhattan_distance as f64
    } else {
        1.0
    };

    let crossings = count_edge_crossings(edge_idx, path, grid);

    let quality_score = compute_score(path.bend_count, detour_factor, crossings);
    let flags = build_flags(path.bend_count, detour_factor, crossings);

    EdgeQuality {
        edge_id: format!("{}-->{}", edge.from, edge.to),
        source: edge.from.clone(),
        target: edge.to.clone(),
        bends: path.bend_count,
        detour_factor,
        crossings,
        port_side_source: side_name(ports.source_port.side),
        port_side_target: side_name(ports.target_port.side),
        path_length,
        manhattan_distance,
        quality_score,
        flags,
    }
}

/// Score all successfully routed edges.
///
/// Edges that failed to route (not present in `routing_result.paths`) are skipped.
/// Results are returned in edge-index order.
pub fn score_all_edges(
    graph: &Graph,
    routing_result: &RoutingResult,
    port_assignments: &HashMap<usize, EdgePorts>,
    grid: &Grid,
) -> Vec<EdgeQuality> {
    let mut scores: Vec<(usize, EdgeQuality)> = routing_result
        .paths
        .iter()
        .filter_map(|(&edge_idx, path)| {
            let edge = graph.edges.get(edge_idx)?;
            let ports = port_assignments.get(&edge_idx)?;
            Some((edge_idx, score_edge(edge_idx, edge, path, ports, grid)))
        })
        .collect();

    scores.sort_by_key(|(idx, _)| *idx);
    scores.into_iter().map(|(_, q)| q).collect()
}

// --- Private helpers ----------------------------------------------------------

/// Count the number of cells on `path` that are marked as crossing points
/// and whose owner or `crossed_by` matches `"edge_{edge_idx}"`.
fn count_edge_crossings(edge_idx: usize, path: &RoutedPath, grid: &Grid) -> usize {
    let id = format!("edge_{}", edge_idx);
    path.points
        .iter()
        .filter(|pt| {
            if !grid.in_bounds(pt.row, pt.col) {
                return false;
            }
            let Some(cell) = grid.get(pt.row as usize, pt.col as usize) else {
                return false;
            };
            cell.crossing
                && (cell.owner.as_deref() == Some(id.as_str())
                    || cell.crossed_by.as_deref() == Some(id.as_str()))
        })
        .count()
}

/// Compute a composite quality score in [0.0, 1.0] (higher = better).
///
/// Three independent penalty terms, each capped at its weight:
/// | Term     | Weight | Saturates at               |
/// |----------|--------|----------------------------|
/// | Detour   | 0.40   | detour_factor ≥ 3.0        |
/// | Bends    | 0.30   | bends ≥ 6                  |
/// | Crossing | 0.30   | any crossing present       |
fn compute_score(bends: usize, detour_factor: f64, crossings: usize) -> f64 {
    let detour_penalty = if detour_factor.is_finite() {
        ((detour_factor - 1.0).max(0.0) / 2.0).min(1.0) * 0.40
    } else {
        0.40
    };

    let bend_penalty = (bends as f64 / 6.0).min(1.0) * 0.30;

    let crossing_penalty = if crossings > 0 { 0.30 } else { 0.0 };

    (1.0 - detour_penalty - bend_penalty - crossing_penalty).max(0.0)
}

/// Build human-readable flag strings for the edge.
fn build_flags(bends: usize, detour_factor: f64, crossings: usize) -> Vec<String> {
    let mut flags = Vec::new();
    if detour_factor.is_finite() && detour_factor >= DETOUR_THRESHOLD {
        flags.push("high_detour".to_string());
    }
    if bends >= BEND_THRESHOLD {
        flags.push("excessive_bends".to_string());
    }
    if crossings > 0 {
        flags.push("avoidable_crossing".to_string());
    }
    flags
}

fn side_name(side: Side) -> String {
    match side {
        Side::Top => "North",
        Side::Bottom => "South",
        Side::Left => "West",
        Side::Right => "East",
    }
    .to_string()
}

// --- Tests --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_core::{
        grid::Grid,
        ports::assignment::{EdgePorts, Port, Side},
        routing::astar::{GridPoint, RoutedPath},
    };
    use trellis_parser::ast::Edge;

    fn make_path(points: Vec<(i64, i64)>, bend_count: usize) -> RoutedPath {
        RoutedPath {
            points: points
                .into_iter()
                .map(|(r, c)| GridPoint { row: r, col: c })
                .collect(),
            total_cost: 0.0,
            bend_count,
        }
    }

    fn make_ports(sr: i64, sc: i64, tr: i64, tc: i64) -> EdgePorts {
        EdgePorts {
            source_port: Port {
                x: 0.0,
                y: 0.0,
                grid_row: sr,
                grid_col: sc,
                side: Side::Bottom,
            },
            target_port: Port {
                x: 0.0,
                y: 0.0,
                grid_row: tr,
                grid_col: tc,
                side: Side::Top,
            },
        }
    }

    fn make_edge(from: &str, to: &str) -> Edge {
        Edge {
            from: from.to_string(),
            to: to.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn perfect_straight_edge_scores_high() {
        // 4-step path, 4 manhattan steps, 0 bends, no crossings
        let path = make_path(vec![(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)], 0);
        let ports = make_ports(0, 0, 4, 0);
        let grid = Grid::new(10, 10, 10, 0, 0);
        let edge = make_edge("A", "B");

        let q = score_edge(0, &edge, &path, &ports, &grid);

        assert_eq!(q.path_length, 4);
        assert_eq!(q.manhattan_distance, 4);
        assert!((q.detour_factor - 1.0).abs() < 1e-9);
        assert_eq!(q.bends, 0);
        assert_eq!(q.crossings, 0);
        assert!(q.quality_score > 0.9, "score was {}", q.quality_score);
        assert!(q.flags.is_empty());
    }

    #[test]
    fn high_detour_flagged() {
        // source=(0,0) target=(0,4): Manhattan = 4.
        // Path detours: right 4, down 4, left 4 = 12 steps → detour = 3.0 ≥ threshold.
        let path = make_path(
            vec![
                (0, 0),
                (0, 1),
                (0, 2),
                (0, 3),
                (0, 4),
                (1, 4),
                (2, 4),
                (3, 4),
                (3, 3),
                (3, 2),
                (3, 1),
                (3, 0),
                (3, -1),
            ],
            4,
        );
        let ports = make_ports(0, 0, 0, 4);
        let grid = Grid::new(10, 10, 10, 0, 0);
        let edge = make_edge("A", "B");

        let q = score_edge(0, &edge, &path, &ports, &grid);

        assert!(
            q.detour_factor >= DETOUR_THRESHOLD,
            "detour_factor was {}",
            q.detour_factor
        );
        assert!(q.flags.contains(&"high_detour".to_string()));
    }

    #[test]
    fn excessive_bends_flagged() {
        let path = make_path(vec![(0, 0), (1, 0), (2, 0)], BEND_THRESHOLD);
        let ports = make_ports(0, 0, 2, 0);
        let grid = Grid::new(10, 10, 10, 0, 0);
        let edge = make_edge("A", "B");

        let q = score_edge(0, &edge, &path, &ports, &grid);

        assert!(q.flags.contains(&"excessive_bends".to_string()));
    }

    #[test]
    fn edge_id_format() {
        let path = make_path(vec![(0, 0), (1, 0)], 0);
        let ports = make_ports(0, 0, 1, 0);
        let grid = Grid::new(5, 5, 10, 0, 0);
        let edge = make_edge("Foo", "Bar");

        let q = score_edge(0, &edge, &path, &ports, &grid);

        assert_eq!(q.edge_id, "Foo-->Bar");
        assert_eq!(q.port_side_source, "South");
        assert_eq!(q.port_side_target, "North");
    }

    #[test]
    fn score_all_edges_returns_sorted_results() {
        use std::collections::HashMap;
        use trellis_core::routing::RoutingResult;
        use trellis_parser::Graph;

        let mut graph = Graph::new();
        graph.edges.push(make_edge("A", "B"));
        graph.edges.push(make_edge("C", "D"));

        let path0 = make_path(vec![(0, 0), (1, 0)], 0);
        let path1 = make_path(vec![(2, 0), (3, 0)], 0);

        let mut paths = HashMap::new();
        paths.insert(0usize, path0);
        paths.insert(1usize, path1);

        let routing_result = RoutingResult {
            paths,
            crossings: 0,
            total_bends: 0,
            failed_routes: 0,
            total_path_length: 2,
            total_routing_cost: 0.0,
            max_path_length: 0,
            max_bends_per_edge: 0,
            sum_manhattan_distance: 0,
            deadlock_recoveries: 0,
        };

        let mut port_assignments = HashMap::new();
        port_assignments.insert(0usize, make_ports(0, 0, 1, 0));
        port_assignments.insert(1usize, make_ports(2, 0, 3, 0));

        let grid = Grid::new(10, 10, 10, 0, 0);
        let scores = score_all_edges(&graph, &routing_result, &port_assignments, &grid);

        assert_eq!(scores.len(), 2);
        assert_eq!(scores[0].source, "A");
        assert_eq!(scores[1].source, "C");
    }

    #[test]
    fn self_loop_detour_factor_is_finite() {
        // Test that a self-loop (source == target) produces a finite detour_factor
        // Path with 1 point (self-loop), manhattan distance = 0
        let path = make_path(vec![(0, 0)], 0);
        let ports = make_ports(0, 0, 0, 0); // same position
        let grid = Grid::new(5, 5, 10, 0, 0);
        let edge = make_edge("A", "A");

        let q = score_edge(0, &edge, &path, &ports, &grid);

        assert!(
            q.detour_factor.is_finite(),
            "detour_factor should be finite"
        );
        assert_eq!(q.manhattan_distance, 0);
        assert_eq!(q.detour_factor, 1.0);
    }
}
