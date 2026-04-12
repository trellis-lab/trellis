pub mod astar;
pub mod commit;
pub mod cost;
pub mod crossing_reroute;
pub mod multi_edge;
pub mod port_swap;
pub mod priority;
pub mod quality_reroute;

use std::collections::HashMap;

use crate::config::TrellisConfig;
use crate::deadlock;
use crate::grid::Grid;
use crate::ports::EdgePorts;
use astar::{route_edge, GridPoint, RoutedPath};
use commit::commit_path;
use multi_edge::{detect_multi_edges, is_multi_edge};
use priority::calculate_priorities;
use trellis_parser::Graph;

/// Result of routing all edges in the graph
#[derive(Debug, Clone)]
pub struct RoutingResult {
    /// Routed paths indexed by edge index
    pub paths: HashMap<usize, RoutedPath>,
    /// Total number of crossing points (grid cells shared by two distinct paths)
    pub crossings: usize,
    /// Total number of bends across all paths
    pub total_bends: usize,
    /// Number of edges that could not be routed
    pub failed_routes: usize,
    /// Total length of all routed paths in grid steps
    pub total_path_length: usize,
    /// Sum of A* routing costs across all routed paths
    pub total_routing_cost: f64,
    /// Longest single routed path in grid steps
    #[cfg(feature = "diagnostics")]
    pub max_path_length: usize,
    /// Largest bend count on any single routed path
    #[cfg(feature = "diagnostics")]
    pub max_bends_per_edge: usize,
    /// Sum of source→target Manhattan distances for all routed edges
    /// (denominator for the average detour factor)
    #[cfg(feature = "diagnostics")]
    pub sum_manhattan_distance: usize,
    /// Number of edges that required the 3-level deadlock recovery handler
    pub deadlock_recoveries: usize,
}

/// Route all edges in the graph using A* pathfinding.
///
/// Edges are routed in priority order. Multi-edges are detected and
/// routed with awareness of each other to prevent overlap.
/// On failure, the 3-level deadlock handler is invoked.
pub fn route_all_edges(
    graph: &Graph,
    grid: &mut Grid,
    port_assignments: &HashMap<usize, EdgePorts>,
    config: &TrellisConfig,
) -> RoutingResult {
    route_all_edges_inner(graph, grid, port_assignments, config, true)
}

/// Route all edges WITHOUT deadlock handling.
/// Used internally by grid expansion to avoid infinite recursion.
pub(crate) fn route_all_edges_no_deadlock(
    graph: &Graph,
    grid: &mut Grid,
    port_assignments: &HashMap<usize, EdgePorts>,
    config: &TrellisConfig,
) -> RoutingResult {
    route_all_edges_inner(graph, grid, port_assignments, config, false)
}

fn route_all_edges_inner(
    graph: &Graph,
    grid: &mut Grid,
    port_assignments: &HashMap<usize, EdgePorts>,
    config: &TrellisConfig,
    deadlock_enabled: bool,
) -> RoutingResult {
    let mut result = RoutingResult {
        paths: HashMap::new(),
        crossings: 0,
        total_bends: 0,
        failed_routes: 0,
        total_path_length: 0,
        total_routing_cost: 0.0,
        #[cfg(feature = "diagnostics")]
        max_path_length: 0,
        #[cfg(feature = "diagnostics")]
        max_bends_per_edge: 0,
        #[cfg(feature = "diagnostics")]
        sum_manhattan_distance: 0,
        deadlock_recoveries: 0,
    };

    if graph.edges.is_empty() {
        return result;
    }

    // Detect multi-edges
    let multi_groups = detect_multi_edges(graph);

    // Calculate routing priorities
    let priorities = calculate_priorities(graph);

    // Track which edges have been routed (for multi-edge groups)
    let mut routed: HashMap<usize, bool> = HashMap::new();

    // Route edges in priority order
    for priority in &priorities {
        let edge_idx = priority.edge_index;

        if routed.contains_key(&edge_idx) {
            continue;
        }

        // Check if this edge is part of a multi-edge group
        if is_multi_edge(edge_idx, &multi_groups) {
            // Find the group and route all edges in the group together
            if let Some(group) = multi_groups
                .iter()
                .find(|g| g.edge_indices.contains(&edge_idx))
            {
                for &group_edge_idx in &group.edge_indices {
                    if routed.contains_key(&group_edge_idx) {
                        continue;
                    }

                    route_single_edge(
                        graph,
                        grid,
                        group_edge_idx,
                        port_assignments,
                        config,
                        &mut result,
                        deadlock_enabled,
                    );
                    routed.insert(group_edge_idx, true);
                }
            }
        } else {
            route_single_edge(
                graph,
                grid,
                edge_idx,
                port_assignments,
                config,
                &mut result,
                deadlock_enabled,
            );
            routed.insert(edge_idx, true);
        }
    }

    // Post-routing: derive per-path stats from the final committed paths.
    // Using the final result.paths (rather than incremental tracking) avoids
    // double-counting during rip-up-and-reroute rollbacks.
    // Gated: O(edges) scan used only for display-only diagnostics metrics.
    #[cfg(feature = "diagnostics")]
    for (&edge_idx, path) in &result.paths {
        let path_len = path.points.len().saturating_sub(1);
        result.max_path_length = result.max_path_length.max(path_len);
        result.max_bends_per_edge = result.max_bends_per_edge.max(path.bend_count);

        if let Some(ports) = port_assignments.get(&edge_idx) {
            let manhattan = ((ports.source_port.grid_row - ports.target_port.grid_row).abs()
                + (ports.source_port.grid_col - ports.target_port.grid_col).abs())
                as usize;
            result.sum_manhattan_distance += manhattan;
        }
    }

    // Count crossing points from the committed grid state
    result.crossings = grid.count_crossings();

    result
}

/// Route a single edge and commit the result to the grid.
/// If routing fails and `deadlock_enabled` is true, triggers the 3-level deadlock handler.
fn route_single_edge(
    graph: &Graph,
    grid: &mut Grid,
    edge_idx: usize,
    port_assignments: &HashMap<usize, EdgePorts>,
    config: &TrellisConfig,
    result: &mut RoutingResult,
    deadlock_enabled: bool,
) {
    let ports = match port_assignments.get(&edge_idx) {
        Some(p) => p,
        None => {
            result.failed_routes += 1;
            return;
        }
    };

    let source = GridPoint {
        row: ports.source_port.grid_row,
        col: ports.source_port.grid_col,
    };
    let target = GridPoint {
        row: ports.target_port.grid_row,
        col: ports.target_port.grid_col,
    };

    // Temporarily mark source and target cells as free if they're blocked
    // (ports are on node boundaries, which may be blocked in the grid)
    let source_state = save_and_free_cell(grid, source, &config.routing_costs);
    let target_state = save_and_free_cell(grid, target, &config.routing_costs);

    match route_edge(grid, source, target, &config.routing_costs) {
        Some(path) => {
            result.total_bends += path.bend_count;
            result.total_path_length += path.points.len().saturating_sub(1);
            result.total_routing_cost += path.total_cost;
            let edge_id = format!("edge_{}", edge_idx);
            commit_path(grid, &path.points, &edge_id, &config.routing_costs);
            result.paths.insert(edge_idx, path);
        }
        None => {
            // Restore cells before deadlock handling (it manages its own cell states)
            restore_cell(grid, source, source_state);
            restore_cell(grid, target, target_state);

            if deadlock_enabled {
                // M7: 3-level deadlock handling
                match deadlock::handle_deadlock(
                    graph,
                    grid,
                    edge_idx,
                    port_assignments,
                    config,
                    result,
                ) {
                    Some(path) => {
                        result.deadlock_recoveries += 1;
                        result.total_bends += path.bend_count;
                        result.total_path_length += path.points.len().saturating_sub(1);
                        result.total_routing_cost += path.total_cost;
                        result.paths.insert(edge_idx, path);
                    }
                    None => {
                        result.failed_routes += 1;
                    }
                }
            } else {
                result.failed_routes += 1;
            }
            return;
        }
    }

    // Restore original cell states for source/target
    restore_cell(grid, source, source_state);
    restore_cell(grid, target, target_state);
}

/// Save a cell's state and temporarily mark it as free for routing.
///
/// Frees both `Blocked` cells (node body) and `Occupied` cells (previously
/// committed edge paths).  Freeing an occupied endpoint is necessary so that
/// A* can land directly on the target connector instead of approaching it from
/// a neighbour, which would add two extra bends.
fn save_and_free_cell(
    grid: &mut Grid,
    point: GridPoint,
    costs: &crate::config::RoutingCosts,
) -> Option<crate::grid::CellState> {
    if !grid.in_bounds(point.row, point.col) {
        return None;
    }

    let row = point.row as usize;
    let col = point.col as usize;

    if let Some(cell) = grid.get(row, col) {
        let original_state = cell.state;
        let needs_free = original_state == crate::grid::CellState::Blocked
            || original_state == crate::grid::CellState::Occupied;
        if needs_free {
            if let Some(cell) = grid.get_mut(row, col) {
                cell.state = crate::grid::CellState::Free;
                cell.cost = costs.base_cost;
            }
        }
        Some(original_state)
    } else {
        None
    }
}

/// Restore a cell's original state after routing
fn restore_cell(grid: &mut Grid, point: GridPoint, original_state: Option<crate::grid::CellState>) {
    if let Some(state) = original_state {
        let was_blocked_or_occupied =
            state == crate::grid::CellState::Blocked || state == crate::grid::CellState::Occupied;
        if was_blocked_or_occupied && grid.in_bounds(point.row, point.col) {
            if let Some(cell) = grid.get_mut(point.row as usize, point.col as usize) {
                // Don't restore if routing committed a new path through this cell.
                if cell.state != crate::grid::CellState::Occupied {
                    cell.state = state;
                    if state == crate::grid::CellState::Blocked {
                        cell.cost = f64::INFINITY;
                    }
                    // Occupied cells have no stored cost; movement_cost computes it dynamically.
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::build_grid;
    use crate::grid::params::calculate_grid_extent;
    use crate::placement;
    use crate::ports::assign_ports;

    /// Helper: parse, place, build grid, assign ports, route, return result
    fn route_fixture(mermaid: &str) -> (RoutingResult, Grid) {
        let mut graph = trellis_parser::parse(mermaid).expect("parse failed");
        let config = TrellisConfig::default();
        let cell_size = config.cell_size;
        placement::place_nodes(&mut graph, cell_size);
        let extent = calculate_grid_extent(&graph, cell_size);
        let mut grid = build_grid(&graph, cell_size, &extent);
        let port_assignments = assign_ports(&graph, cell_size, extent.offset_x, extent.offset_y);

        let result = route_all_edges(&graph, &mut grid, &port_assignments, &config);
        (result, grid)
    }

    #[test]
    fn test_b01_linear_chain_all_edges_routed() {
        let (result, _grid) =
            route_fixture("graph TB\n    A --> B\n    B --> C\n    C --> D\n    D --> E");

        assert_eq!(result.paths.len(), 4, "All 4 edges should be routed");
        assert_eq!(result.failed_routes, 0, "No edges should fail");
    }

    #[test]
    fn test_b01_paths_are_orthogonal() {
        let (result, _grid) =
            route_fixture("graph TB\n    A --> B\n    B --> C\n    C --> D\n    D --> E");

        for (_, path) in &result.paths {
            for i in 1..path.points.len() {
                let prev = &path.points[i - 1];
                let curr = &path.points[i];
                // Each step should be orthogonal (only row or col changes, not both)
                let dr = (curr.row - prev.row).abs();
                let dc = (curr.col - prev.col).abs();
                assert!(
                    (dr == 1 && dc == 0) || (dr == 0 && dc == 1),
                    "Path segment ({},{}) -> ({},{}) is not orthogonal",
                    prev.row,
                    prev.col,
                    curr.row,
                    curr.col
                );
            }
        }
    }

    #[test]
    fn test_b01_no_path_overlap() {
        let (result, _grid) =
            route_fixture("graph TB\n    A --> B\n    B --> C\n    C --> D\n    D --> E");

        // Collect all points from all paths (excluding start/end which may share ports)
        let mut all_points: Vec<(i64, i64)> = Vec::new();
        for (_, path) in &result.paths {
            // Skip first and last points (ports can be shared)
            for point in path
                .points
                .iter()
                .skip(1)
                .take(path.points.len().saturating_sub(2))
            {
                let key = (point.row, point.col);
                assert!(
                    !all_points.contains(&key),
                    "Path overlap detected at ({}, {})",
                    point.row,
                    point.col
                );
                all_points.push(key);
            }
        }
    }

    #[test]
    fn test_b03_k33_most_edges_routed() {
        let (result, _grid) = route_fixture(
            "graph TB\n    A1 --> B1\n    A1 --> B2\n    A1 --> B3\n    A2 --> B1\n    A2 --> B2\n    A2 --> B3\n    A3 --> B1\n    A3 --> B2\n    A3 --> B3",
        );

        // K3,3 is a non-planar, dense bipartite graph. Without deadlock
        // handling (M7), many edges may fail to route as earlier paths block
        // later ones. We expect at least some edges to succeed.
        assert!(
            result.paths.len() >= 2,
            "At least 2 of 9 K3,3 edges should be routed (got {}, failed {})",
            result.paths.len(),
            result.failed_routes
        );
    }

    #[test]
    fn test_b06_multi_edge_detection() {
        let graph =
            trellis_parser::parse("graph TB\n    A --> B\n    A --> B\n    B --> C").unwrap();

        let groups = multi_edge::detect_multi_edges(&graph);
        assert_eq!(groups.len(), 1, "Should detect 1 multi-edge group");
        assert_eq!(groups[0].edge_indices.len(), 2);
    }

    #[test]
    fn test_b04_diamond_routed() {
        let (result, _grid) =
            route_fixture("graph TB\n    A --> B\n    A --> C\n    B --> D\n    C --> D");

        assert_eq!(result.paths.len(), 4, "All diamond edges should be routed");
        assert_eq!(result.failed_routes, 0);
    }
}
