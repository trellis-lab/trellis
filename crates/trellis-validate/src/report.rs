use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use trellis_core::{grid::Grid, ports::EdgePorts, routing::RoutingResult};
use trellis_parser::Graph;

use crate::scoring::{score_all_edges, EdgeQuality};

// --- Report types -------------------------------------------------------------

/// A grid cell coordinate `[row, col]` as it appears in the JSON report.
pub type CellCoord = [i64; 2];

/// Per-edge entry in the JSON report.
///
/// Matches the schema defined in `docs/validation/edge-quality-feedback.md`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeReport {
    /// `"source-->target"`
    pub id: String,
    pub source: String,
    pub target: String,
    pub bends: usize,
    pub detour_factor: f64,
    pub crossings: usize,
    pub port_side_source: String,
    pub port_side_target: String,
    /// Sequence of `[row, col]` grid coordinates for the routed path.
    pub path_cells: Vec<CellCoord>,
    pub quality_score: f64,
    pub flags: Vec<String>,
}

/// Summary statistics across all edges in a single diagram.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalMetrics {
    /// Total number of edges in the graph.
    pub total_edges: usize,
    /// Edges that were successfully routed.
    pub routed_edges: usize,
    /// Edges that could not be routed.
    pub failed_edges: usize,
    /// Total crossing cells across all paths.
    pub total_crossings: usize,
    /// Total bends across all routed paths.
    pub total_bends: usize,
    /// Mean quality score over all routed edges (0.0 if none).
    pub avg_quality_score: f64,
    /// Mean detour factor over edges with non-zero Manhattan distance.
    /// 0.0 if no such edges exist.
    pub avg_detour_factor: f64,
    /// Number of routed edges that carry at least one quality flag.
    pub flagged_edges: usize,
}

/// The top-level JSON report for one diagram.
///
/// Stable contract consumed by the AI tuning agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramReport {
    /// Base filename of the input diagram (e.g. `"b05.mmd"`).
    pub fixture: String,
    pub edges: Vec<EdgeReport>,
    pub global_metrics: GlobalMetrics,
}

// --- Public API ---------------------------------------------------------------

/// Generate a `DiagramReport` for one rendered diagram.
///
/// * `fixture_name`     — base filename used as the `fixture` field (e.g. `"b05.mmd"`).
/// * `graph`            — the parsed graph (provides node/edge counts and IDs).
/// * `routing_result`   — committed routing output (paths, crossings, bends).
/// * `port_assignments` — source/target port for every routed edge.
/// * `grid`             — final routing grid (used to read cell coordinates and crossing flags).
pub fn generate_report(
    fixture_name: &str,
    graph: &Graph,
    routing_result: &RoutingResult,
    port_assignments: &HashMap<usize, EdgePorts>,
    grid: &Grid,
) -> DiagramReport {
    let scored = score_all_edges(graph, routing_result, port_assignments, grid);

    let edges: Vec<EdgeReport> = scored
        .iter()
        .enumerate()
        .map(|(list_idx, q)| {
            // Recover the original edge_idx so we can look up the path.
            // score_all_edges returns entries sorted by edge_idx; we need the
            // actual edge_idx to retrieve the path from routing_result.paths.
            let edge_idx = find_edge_idx(graph, q, list_idx);
            let path_cells = routing_result
                .paths
                .get(&edge_idx)
                .map(|p| {
                    p.points
                        .iter()
                        .map(|pt| [pt.row, pt.col])
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            edge_report_from_quality(q, path_cells)
        })
        .collect();

    let global_metrics = compute_global_metrics(graph, routing_result, &edges);

    DiagramReport {
        fixture: fixture_name.to_string(),
        edges,
        global_metrics,
    }
}

// --- Private helpers ----------------------------------------------------------

/// Recover the edge index for an `EdgeQuality` entry.
///
/// `score_all_edges` sorts by edge_idx; we walk `graph.edges` to match the
/// from/to pair, skipping already-matched indices via the sorted list position.
fn find_edge_idx(graph: &Graph, q: &EdgeQuality, hint: usize) -> usize {
    // Fast path: edge at position `hint` often matches (sorted by index).
    if let Some(e) = graph.edges.get(hint) {
        if e.from == q.source && e.to == q.target {
            return hint;
        }
    }
    // Fallback: linear scan for the first matching from/to pair.
    graph
        .edges
        .iter()
        .position(|e| e.from == q.source && e.to == q.target)
        .unwrap_or(hint)
}

fn edge_report_from_quality(q: &EdgeQuality, path_cells: Vec<CellCoord>) -> EdgeReport {
    EdgeReport {
        id: q.edge_id.clone(),
        source: q.source.clone(),
        target: q.target.clone(),
        bends: q.bends,
        detour_factor: q.detour_factor,
        crossings: q.crossings,
        port_side_source: q.port_side_source.clone(),
        port_side_target: q.port_side_target.clone(),
        path_cells,
        quality_score: q.quality_score,
        flags: q.flags.clone(),
    }
}

fn compute_global_metrics(
    graph: &Graph,
    routing_result: &RoutingResult,
    edges: &[EdgeReport],
) -> GlobalMetrics {
    let total_edges = graph.edges.len();
    let routed_edges = routing_result.paths.len();
    let failed_edges = routing_result.failed_routes;
    let total_crossings = routing_result.crossings;
    let total_bends = routing_result.total_bends;

    let avg_quality_score = if edges.is_empty() {
        0.0
    } else {
        edges.iter().map(|e| e.quality_score).sum::<f64>() / edges.len() as f64
    };

    let finite_detour: Vec<f64> = edges
        .iter()
        .filter(|e| e.detour_factor.is_finite())
        .map(|e| e.detour_factor)
        .collect();
    let avg_detour_factor = if finite_detour.is_empty() {
        0.0
    } else {
        finite_detour.iter().sum::<f64>() / finite_detour.len() as f64
    };

    let flagged_edges = edges.iter().filter(|e| !e.flags.is_empty()).count();

    GlobalMetrics {
        total_edges,
        routed_edges,
        failed_edges,
        total_crossings,
        total_bends,
        avg_quality_score,
        avg_detour_factor,
        flagged_edges,
    }
}

// --- Tests --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_core::{
        grid::Grid,
        ports::assignment::{EdgePorts, Port, Side},
        routing::{
            astar::{GridPoint, RoutedPath},
            RoutingResult,
        },
    };
    use trellis_parser::{ast::Edge, Graph};

    fn make_graph_with_edges(pairs: &[(&str, &str)]) -> Graph {
        let mut g = Graph::new();
        for (from, to) in pairs {
            g.edges.push(Edge {
                from: from.to_string(),
                to: to.to_string(),
                ..Default::default()
            });
        }
        g
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

    fn make_path(points: Vec<(i64, i64)>, bends: usize) -> RoutedPath {
        RoutedPath {
            points: points
                .into_iter()
                .map(|(r, c)| GridPoint { row: r, col: c })
                .collect(),
            total_cost: 0.0,
            bend_count: bends,
        }
    }

    fn make_routing_result(paths: HashMap<usize, RoutedPath>) -> RoutingResult {
        RoutingResult {
            paths,
            crossings: 0,
            total_bends: 0,
            failed_routes: 0,
            total_path_length: 0,
            total_routing_cost: 0.0,
            max_path_length: 0,
            max_bends_per_edge: 0,
            sum_manhattan_distance: 0,
            deadlock_recoveries: 0,
        }
    }

    #[test]
    fn report_fixture_name_preserved() {
        let graph = make_graph_with_edges(&[("A", "B")]);
        let mut paths = HashMap::new();
        paths.insert(0, make_path(vec![(0, 0), (1, 0)], 0));
        let rr = make_routing_result(paths);
        let mut pa = HashMap::new();
        pa.insert(0, make_ports(0, 0, 1, 0));
        let grid = Grid::new(5, 5, 10, 0, 0);

        let report = generate_report("b05.mmd", &graph, &rr, &pa, &grid);

        assert_eq!(report.fixture, "b05.mmd");
    }

    #[test]
    fn report_edge_fields_match_schema() {
        let graph = make_graph_with_edges(&[("A", "B")]);
        let mut paths = HashMap::new();
        paths.insert(0, make_path(vec![(0, 0), (1, 0), (2, 0)], 0));
        let rr = make_routing_result(paths);
        let mut pa = HashMap::new();
        pa.insert(0, make_ports(0, 0, 2, 0));
        let grid = Grid::new(5, 5, 10, 0, 0);

        let report = generate_report("test.mmd", &graph, &rr, &pa, &grid);

        assert_eq!(report.edges.len(), 1);
        let e = &report.edges[0];
        assert_eq!(e.id, "A-->B");
        assert_eq!(e.source, "A");
        assert_eq!(e.target, "B");
        assert_eq!(e.path_cells, vec![[0, 0], [1, 0], [2, 0]]);
        assert_eq!(e.port_side_source, "South");
        assert_eq!(e.port_side_target, "North");
    }

    #[test]
    fn global_metrics_counts_correctly() {
        let graph = make_graph_with_edges(&[("A", "B"), ("C", "D")]);
        let mut paths = HashMap::new();
        paths.insert(0, make_path(vec![(0, 0), (1, 0)], 0));
        paths.insert(1, make_path(vec![(2, 0), (3, 0)], 1));
        let mut rr = make_routing_result(paths);
        rr.total_bends = 1;
        let mut pa = HashMap::new();
        pa.insert(0, make_ports(0, 0, 1, 0));
        pa.insert(1, make_ports(2, 0, 3, 0));
        let grid = Grid::new(10, 10, 10, 0, 0);

        let report = generate_report("multi.mmd", &graph, &rr, &pa, &grid);

        let gm = &report.global_metrics;
        assert_eq!(gm.total_edges, 2);
        assert_eq!(gm.routed_edges, 2);
        assert_eq!(gm.failed_edges, 0);
        assert_eq!(gm.total_bends, 1);
        assert!(gm.avg_quality_score > 0.0);
    }

    #[test]
    fn report_serialises_to_valid_json() {
        let graph = make_graph_with_edges(&[("X", "Y")]);
        let mut paths = HashMap::new();
        paths.insert(0, make_path(vec![(0, 0), (0, 1)], 0));
        let rr = make_routing_result(paths);
        let mut pa = HashMap::new();
        pa.insert(0, make_ports(0, 0, 0, 1));
        let grid = Grid::new(5, 5, 10, 0, 0);

        let report = generate_report("x.mmd", &graph, &rr, &pa, &grid);
        let json = serde_json::to_string_pretty(&report).expect("serialisation failed");

        assert!(json.contains("\"fixture\""));
        assert!(json.contains("\"edges\""));
        assert!(json.contains("\"global_metrics\""));
        assert!(json.contains("\"path_cells\""));
        assert!(json.contains("X-->Y"));
    }

    #[test]
    fn self_loop_edge_serialises_with_finite_detour() {
        // Test that a self-loop edge (A-->A) produces valid JSON with detour_factor=1.0
        let graph = make_graph_with_edges(&[("A", "A")]);
        let mut paths = HashMap::new();
        paths.insert(0, make_path(vec![(0, 0)], 0));
        let rr = make_routing_result(paths);
        let mut pa = HashMap::new();
        pa.insert(0, make_ports(0, 0, 0, 0));  // same position
        let grid = Grid::new(5, 5, 10, 0, 0);

        let report = generate_report("self-loop.mmd", &graph, &rr, &pa, &grid);
        let json = serde_json::to_string_pretty(&report).expect("serialisation failed");

        // Verify JSON is valid and contains expected fields
        assert!(json.contains("\"detour_factor\": 1.0"), "detour_factor should be 1.0, got:\n{}", json);
        assert!(!json.contains("\"detour_factor\": null"), "detour_factor should not be null");

        // Verify the edge report has the correct detour_factor
        assert_eq!(report.edges.len(), 1);
        assert_eq!(report.edges[0].detour_factor, 1.0);
        assert!(report.edges[0].detour_factor.is_finite());
    }
}
