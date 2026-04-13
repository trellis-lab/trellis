use std::collections::{BTreeSet, HashMap};
use trellis_parser::Graph;

use crate::config::TrellisConfig;
use crate::grid::{build_grid, calculate_grid_extent, Grid};
use crate::routing;

use super::assignment::EdgePorts;
use super::common::build_edge_side_map;
use super::{create_port_assigner, PortAssigner, PortAssignmentContext};

/// Route-Then-Swap port assigner.
///
/// Uses any single-round assigner (CrossingGreedy by default) for the initial
/// assignment, then runs up to `max_rounds` refinement loops:
///   1. Route all edges
///   2. Find crossing edge pairs from the grid
///   3. Swap port assignments for crossing pairs that share a node
///   4. Re-route with the new ports
///
/// Stops early if crossings reach zero or no improvement is made.
pub struct IterativeSwapAssigner {
    /// The initial assigner to use for the first port assignment.
    pub initial_strategy: crate::config::PortAssignmentStrategy,
}

impl PortAssigner for IterativeSwapAssigner {
    fn assign_ports(&self, ctx: &PortAssignmentContext) -> HashMap<usize, EdgePorts> {
        // The initial assignment comes from the chosen single-round strategy.
        let initial = create_port_assigner(self.initial_strategy);
        initial.assign_ports(ctx)
    }
}

/// Find edge pairs that cross on the grid.
///
/// Scans cells marked as crossings (where `owner` and `crossed_by` differ)
/// and returns the set of unique (edge_a, edge_b) pairs.
pub fn find_crossing_edge_pairs(grid: &Grid) -> Vec<(String, String)> {
    let mut pairs: BTreeSet<(String, String)> = BTreeSet::new();

    for row in 0..grid.rows {
        for col in 0..grid.cols {
            if let Some(cell) = grid.get(row, col) {
                if cell.crossing {
                    if let (Some(owner), Some(crossed_by)) = (&cell.owner, &cell.crossed_by) {
                        // Canonical ordering to avoid duplicates
                        let pair = if owner < crossed_by {
                            (owner.clone(), crossed_by.clone())
                        } else {
                            (crossed_by.clone(), owner.clone())
                        };
                        pairs.insert(pair);
                    }
                }
            }
        }
    }

    pairs.into_iter().collect()
}

/// Parse edge index from edge ID string (format: "edge_N").
fn parse_edge_index(edge_id: &str) -> Option<usize> {
    edge_id.strip_prefix("edge_").and_then(|s| s.parse().ok())
}

/// Find the shared node between two edges (if any).
///
/// Two edges share a node if one edge's source or target equals
/// the other edge's source or target.
fn find_shared_node(edge_a_idx: usize, edge_b_idx: usize, graph: &Graph) -> Option<String> {
    let a = graph.edges.get(edge_a_idx)?;
    let b = graph.edges.get(edge_b_idx)?;

    if a.from == b.from || a.from == b.to {
        Some(a.from.clone())
    } else if a.to == b.from || a.to == b.to {
        Some(a.to.clone())
    } else {
        None
    }
}

/// Swap port assignments for two edges on a shared node.
///
/// For the shared node, swaps which edge gets which port.
/// The ports on the other end (non-shared node) remain unchanged.
pub fn apply_port_swaps(
    port_assignments: &mut HashMap<usize, EdgePorts>,
    swaps: &[(usize, usize, String)], // (edge_a, edge_b, shared_node_id)
    graph: &Graph,
) {
    for &(edge_a, edge_b, ref shared_node) in swaps {
        let (port_a, port_b) = {
            let pa = match port_assignments.get(&edge_a) {
                Some(p) => p.clone(),
                None => continue,
            };
            let pb = match port_assignments.get(&edge_b) {
                Some(p) => p.clone(),
                None => continue,
            };
            (pa, pb)
        };

        let a_is_source_on_shared = graph
            .edges
            .get(edge_a)
            .map(|e| e.from == *shared_node)
            .unwrap_or(false);
        let b_is_source_on_shared = graph
            .edges
            .get(edge_b)
            .map(|e| e.from == *shared_node)
            .unwrap_or(false);

        // Extract the port on the shared node for each edge
        let a_shared_port = if a_is_source_on_shared {
            &port_a.source_port
        } else {
            &port_a.target_port
        };
        let b_shared_port = if b_is_source_on_shared {
            &port_b.source_port
        } else {
            &port_b.target_port
        };

        // Swap: edge_a gets edge_b's shared-side port and vice versa
        let a_new_shared = b_shared_port.clone();
        let b_new_shared = a_shared_port.clone();

        if let Some(pa) = port_assignments.get_mut(&edge_a) {
            if a_is_source_on_shared {
                pa.source_port = a_new_shared;
            } else {
                pa.target_port = a_new_shared;
            }
        }
        if let Some(pb) = port_assignments.get_mut(&edge_b) {
            if b_is_source_on_shared {
                pb.source_port = b_new_shared;
            } else {
                pb.target_port = b_new_shared;
            }
        }
    }
}

/// Run the iterative refinement loop.
///
/// Takes an initial port assignment and repeatedly:
///   1. Builds grid + routes all edges
///   2. Finds crossing pairs
///   3. Swaps ports for crossing pairs
///
/// Returns the best (lowest-crossing) port assignment and the final grid + routing result.
pub fn refine_ports(
    graph: &Graph,
    config: &TrellisConfig,
    initial_ports: HashMap<usize, EdgePorts>,
    max_rounds: usize,
) -> (HashMap<usize, EdgePorts>, Grid, routing::RoutingResult) {
    let cell_size = config.cell_size;
    let extent = calculate_grid_extent(graph, cell_size);

    let mut best_ports = initial_ports;
    let mut best_grid;
    let mut best_result;

    // Do the first routing pass
    let mut grid = build_grid(graph, cell_size, &extent);
    let result = routing::route_all_edges(graph, &mut grid, &best_ports, config);

    if result.crossings == 0 || max_rounds == 0 {
        return (best_ports, grid, result);
    }

    let mut best_crossings = result.crossings;
    best_grid = grid;
    best_result = result;

    for _round in 0..max_rounds {
        // Find crossing edge pairs
        let crossing_pairs = find_crossing_edge_pairs(&best_grid);
        if crossing_pairs.is_empty() {
            break;
        }

        // Build swap list: for each crossing pair, find the shared node
        let mut swaps: Vec<(usize, usize, String)> = Vec::new();
        for (edge_a_id, edge_b_id) in &crossing_pairs {
            let edge_a = match parse_edge_index(edge_a_id) {
                Some(idx) => idx,
                None => continue,
            };
            let edge_b = match parse_edge_index(edge_b_id) {
                Some(idx) => idx,
                None => continue,
            };

            if let Some(shared_node) = find_shared_node(edge_a, edge_b, graph) {
                swaps.push((edge_a, edge_b, shared_node));
            }
        }

        if swaps.is_empty() {
            break; // no swappable crossings
        }

        // Apply swaps
        let mut candidate_ports = best_ports.clone();
        apply_port_swaps(&mut candidate_ports, &swaps, graph);

        // Re-route with swapped ports
        let mut grid = build_grid(graph, cell_size, &extent);
        let result = routing::route_all_edges(graph, &mut grid, &candidate_ports, config);

        if result.crossings < best_crossings {
            best_crossings = result.crossings;
            best_ports = candidate_ports;
            best_grid = grid;
            best_result = result;

            if best_crossings == 0 {
                break;
            }
        } else {
            break; // no improvement
        }
    }

    (best_ports, best_grid, best_result)
}

/// Estimate total inversion count across all nodes/sides without routing.
///
/// This is the fast cost function used by TwoPhase (phase 1) and Annealing.
pub fn estimate_inversions(graph: &Graph, cell_size: i32, offset_x: i32, offset_y: i32) -> usize {
    let (node_map, per_node_sides) = build_edge_side_map(
        graph,
        cell_size,
        offset_x,
        offset_y,
        None,
        &std::collections::HashMap::new(),
    );

    let mut total = 0;
    for sides in per_node_sides.values() {
        for (&side, edges) in sides {
            total += super::common::count_inversions(edges, side, &node_map);
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{CellState, Grid};

    #[test]
    fn parse_edge_index_valid() {
        assert_eq!(parse_edge_index("edge_0"), Some(0));
        assert_eq!(parse_edge_index("edge_42"), Some(42));
    }

    #[test]
    fn parse_edge_index_invalid() {
        assert_eq!(parse_edge_index("foo"), None);
        assert_eq!(parse_edge_index("edge_"), None);
    }

    #[test]
    fn find_crossing_pairs_empty_grid() {
        let grid = Grid::new(5, 5, 10, 0, 0);
        let pairs = find_crossing_edge_pairs(&grid);
        assert!(pairs.is_empty());
    }

    #[test]
    fn find_crossing_pairs_detects_crossing() {
        let mut grid = Grid::new(5, 5, 10, 0, 0);
        // Simulate a crossing at (2,2)
        let cell = grid.get_mut(2, 2).unwrap();
        cell.state = CellState::Occupied;
        cell.owner = Some("edge_0".to_string());
        cell.crossing = true;
        cell.crossed_by = Some("edge_1".to_string());

        let pairs = find_crossing_edge_pairs(&grid);
        assert_eq!(pairs.len(), 1);
        let (a, b) = &pairs[0];
        assert!(
            (a == "edge_0" && b == "edge_1") || (a == "edge_1" && b == "edge_0"),
            "Pair should contain edge_0 and edge_1"
        );
    }

    #[test]
    fn find_shared_node_finds_common_source() {
        let mut graph = Graph::new();
        graph.edges = vec![
            trellis_parser::Edge {
                from: "A".into(),
                to: "B".into(),
                ..Default::default()
            },
            trellis_parser::Edge {
                from: "A".into(),
                to: "C".into(),
                ..Default::default()
            },
        ];
        assert_eq!(find_shared_node(0, 1, &graph), Some("A".to_string()));
    }

    #[test]
    fn find_shared_node_none_when_unrelated() {
        let mut graph = Graph::new();
        graph.edges = vec![
            trellis_parser::Edge {
                from: "A".into(),
                to: "B".into(),
                ..Default::default()
            },
            trellis_parser::Edge {
                from: "C".into(),
                to: "D".into(),
                ..Default::default()
            },
        ];
        assert_eq!(find_shared_node(0, 1, &graph), None);
    }

    #[test]
    fn estimate_inversions_zero_for_simple_graph() {
        let mut graph = Graph::new();
        graph.nodes = vec![
            trellis_parser::Node {
                id: "A".into(),
                label: "A".into(),
                width: 40.0,
                height: 20.0,
                x: 60.0,
                y: 0.0,
                ..Default::default()
            },
            trellis_parser::Node {
                id: "B".into(),
                label: "B".into(),
                width: 40.0,
                height: 20.0,
                x: 60.0,
                y: 100.0,
                ..Default::default()
            },
        ];
        graph.edges = vec![trellis_parser::Edge {
            from: "A".into(),
            to: "B".into(),
            ..Default::default()
        }];

        assert_eq!(estimate_inversions(&graph, 10, 0, 0), 0);
    }
}
