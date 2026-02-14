pub mod astar;
pub mod commit;
pub mod cost;
pub mod multi_edge;
pub mod priority;

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
    /// Total number of crossings detected
    pub crossings: usize,
    /// Total number of bends across all paths
    pub total_bends: usize,
    /// Number of edges that could not be routed
    pub failed_routes: usize,
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
            route_single_edge(graph, grid, edge_idx, port_assignments, config, &mut result, deadlock_enabled);
            routed.insert(edge_idx, true);
        }
    }

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
    let source_state = save_and_free_cell(grid, source);
    let target_state = save_and_free_cell(grid, target);

    match route_edge(grid, source, target, &config.routing_costs) {
        Some(path) => {
            result.total_bends += path.bend_count;
            let edge_id = format!("edge_{}", edge_idx);
            commit_path(grid, &path.points, &edge_id);
            result.paths.insert(edge_idx, path);
        }
        None => {
            // Restore cells before deadlock handling (it manages its own cell states)
            restore_cell(grid, source, source_state);
            restore_cell(grid, target, target_state);

            if deadlock_enabled {
                // M7: 3-level deadlock handling
                match deadlock::handle_deadlock(
                    graph, grid, edge_idx, port_assignments, config, result,
                ) {
                    Some(path) => {
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

/// Save a cell's state and temporarily mark it as free for routing
fn save_and_free_cell(grid: &mut Grid, point: GridPoint) -> Option<crate::grid::CellState> {
    if !grid.in_bounds(point.row, point.col) {
        return None;
    }

    let row = point.row as usize;
    let col = point.col as usize;

    if let Some(cell) = grid.get(row, col) {
        let original_state = cell.state;
        if original_state == crate::grid::CellState::Blocked {
            if let Some(cell) = grid.get_mut(row, col) {
                cell.state = crate::grid::CellState::Free;
                cell.cost = 1.0;
            }
        }
        Some(original_state)
    } else {
        None
    }
}

/// Restore a cell's original state after routing
fn restore_cell(
    grid: &mut Grid,
    point: GridPoint,
    original_state: Option<crate::grid::CellState>,
) {
    if let Some(state) = original_state {
        if state == crate::grid::CellState::Blocked
            && grid.in_bounds(point.row, point.col)
        {
            if let Some(cell) = grid.get_mut(point.row as usize, point.col as usize) {
                // Don't restore to blocked if we committed a path through it
                if cell.state != crate::grid::CellState::Occupied {
                    cell.state = state;
                    cell.cost = f64::INFINITY;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::build_grid;
    use crate::grid::params::{calculate_cell_size, calculate_grid_extent};
    use crate::placement;
    use crate::ports::assign_ports;

    /// Helper: parse, place, build grid, assign ports, route, return result
    fn route_fixture(mermaid: &str) -> (RoutingResult, Grid) {
        let mut graph = trellis_parser::parse(mermaid).expect("parse failed");
        placement::place_nodes(&mut graph);

        let cell_size = calculate_cell_size(&graph);
        let extent = calculate_grid_extent(&graph);
        let mut grid = build_grid(&graph, cell_size, &extent);
        let port_assignments = assign_ports(&graph, cell_size, extent.offset_x, extent.offset_y);
        let config = TrellisConfig::default();

        let result = route_all_edges(&graph, &mut grid, &port_assignments, &config);
        (result, grid)
    }

    #[test]
    fn test_b01_linear_chain_all_edges_routed() {
        let (result, _grid) = route_fixture(
            "graph TB\n    A --> B\n    B --> C\n    C --> D\n    D --> E",
        );

        assert_eq!(result.paths.len(), 4, "All 4 edges should be routed");
        assert_eq!(result.failed_routes, 0, "No edges should fail");
    }

    #[test]
    fn test_b01_paths_are_orthogonal() {
        let (result, _grid) = route_fixture(
            "graph TB\n    A --> B\n    B --> C\n    C --> D\n    D --> E",
        );

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
                    prev.row, prev.col, curr.row, curr.col
                );
            }
        }
    }

    #[test]
    fn test_b01_no_path_overlap() {
        let (result, _grid) = route_fixture(
            "graph TB\n    A --> B\n    B --> C\n    C --> D\n    D --> E",
        );

        // Collect all points from all paths (excluding start/end which may share ports)
        let mut all_points: Vec<(i64, i64)> = Vec::new();
        for (_, path) in &result.paths {
            // Skip first and last points (ports can be shared)
            for point in path.points.iter().skip(1).take(path.points.len().saturating_sub(2)) {
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
        let graph = trellis_parser::parse(
            "graph TB\n    A --> B\n    A --> B\n    B --> C",
        )
        .unwrap();

        let groups = multi_edge::detect_multi_edges(&graph);
        assert_eq!(groups.len(), 1, "Should detect 1 multi-edge group");
        assert_eq!(groups[0].edge_indices.len(), 2);
    }

    #[test]
    fn test_b04_diamond_routed() {
        let (result, _grid) = route_fixture(
            "graph TB\n    A --> B\n    A --> C\n    B --> D\n    C --> D",
        );

        assert_eq!(result.paths.len(), 4, "All diamond edges should be routed");
        assert_eq!(result.failed_routes, 0);
    }
}
