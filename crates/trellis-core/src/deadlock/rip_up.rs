use std::collections::HashMap;

use crate::config::{RoutingCosts, TrellisConfig};
use crate::grid::{CellState, Grid};
use crate::ports::EdgePorts;
use crate::routing::astar::{route_edge, GridPoint, RoutedPath};
use crate::routing::commit::{build_other_cell_owners, commit_path, restore_path, uncommit_path};
use crate::routing::RoutingResult;

/// Find edges that are blocking the path of a failed edge.
///
/// Runs a modified A* that traverses occupied cells (with high cost)
/// and records which edge owners are encountered along the cheapest path.
/// Returns the edge indices of the most frequently encountered blocking edges (up to 3).
pub fn find_blocking_edges(
    grid: &Grid,
    failed_edge_idx: usize,
    port_assignments: &HashMap<usize, EdgePorts>,
) -> Vec<usize> {
    let ports = match port_assignments.get(&failed_edge_idx) {
        Some(p) => p,
        None => return vec![],
    };

    let source = GridPoint {
        row: ports.source_port.grid_row,
        col: ports.source_port.grid_col,
    };
    let target = GridPoint {
        row: ports.target_port.grid_row,
        col: ports.target_port.grid_col,
    };

    // Use BFS/A*-like exploration to find which occupied cells lie between source and target.
    // We look at occupied cells near the straight-line corridor between source and target.
    let mut edge_frequency: HashMap<usize, usize> = HashMap::new();

    // Explore cells in the bounding box between source and target (with some margin)
    let min_row = source.row.min(target.row) - 2;
    let max_row = source.row.max(target.row) + 2;
    let min_col = source.col.min(target.col) - 2;
    let max_col = source.col.max(target.col) + 2;

    for row in min_row..=max_row {
        for col in min_col..=max_col {
            if !grid.in_bounds(row, col) {
                continue;
            }
            if let Some(cell) = grid.get(row as usize, col as usize) {
                if cell.state == CellState::Occupied {
                    if let Some(owner) = &cell.owner {
                        // Parse edge index from owner string "edge_N"
                        if let Some(idx) = parse_edge_index(owner) {
                            *edge_frequency.entry(idx).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
    }

    // Sort by frequency (most blocking first) and take up to 3
    let mut blocking: Vec<(usize, usize)> = edge_frequency.into_iter().collect();
    blocking.sort_by_key(|a| std::cmp::Reverse(a.1));
    blocking.into_iter().take(3).map(|(idx, _)| idx).collect()
}

/// Parse an edge index from an owner string like "edge_5"
fn parse_edge_index(owner: &str) -> Option<usize> {
    owner.strip_prefix("edge_").and_then(|s| s.parse().ok())
}

/// Rip-up and reroute: remove blocking edges, route the failed edge,
/// then re-route the removed edges.
///
/// Returns the path for the failed edge if successful.
pub fn rip_up_and_reroute(
    grid: &mut Grid,
    failed_edge_idx: usize,
    port_assignments: &HashMap<usize, EdgePorts>,
    config: &TrellisConfig,
    result: &mut RoutingResult,
    max_iterations: usize,
) -> Option<RoutedPath> {
    let ports = port_assignments.get(&failed_edge_idx)?;

    let source = GridPoint {
        row: ports.source_port.grid_row,
        col: ports.source_port.grid_col,
    };
    let target = GridPoint {
        row: ports.target_port.grid_row,
        col: ports.target_port.grid_col,
    };

    for _iteration in 0..max_iterations {
        let blocking_edges = find_blocking_edges(grid, failed_edge_idx, port_assignments);

        if blocking_edges.is_empty() {
            return None; // No edges blocking — problem is structural
        }

        // Save old paths for potential rollback
        let mut saved_paths: Vec<(usize, RoutedPath)> = Vec::new();

        // Uncommit blocking edges. Each path is removed from result.paths before
        // uncommitting, so result.paths contains only the remaining committed edges —
        // used to keep shared cells occupied.
        for &blocking_idx in &blocking_edges {
            if let Some(path) = result.paths.remove(&blocking_idx) {
                let edge_id = format!("edge_{}", blocking_idx);
                let other_owners =
                    build_other_cell_owners(&result.paths, &std::collections::HashSet::new());
                uncommit_path(grid, &path.points, &edge_id, &other_owners);
                result.total_bends = result.total_bends.saturating_sub(path.bend_count);
                saved_paths.push((blocking_idx, path));
            }
        }

        // Temporarily free source/target cells
        let source_state = save_and_free_cell(grid, source, &config.routing_costs);
        let target_state = save_and_free_cell(grid, target, &config.routing_costs);

        // Try routing the failed edge
        let failed_path = route_edge(grid, source, target, &config.routing_costs);

        restore_cell(grid, source, source_state);
        restore_cell(grid, target, target_state);

        if let Some(path) = failed_path {
            // Commit the failed edge's new path
            let edge_id = format!("edge_{}", failed_edge_idx);
            commit_path(grid, &path.points, &edge_id);
            result.total_bends += path.bend_count;

            // Try to re-route all blocking edges
            let mut all_rerouted = true;
            let mut rerouted_paths: Vec<(usize, RoutedPath)> = Vec::new();

            for &blocking_idx in &blocking_edges {
                if let Some(blocking_ports) = port_assignments.get(&blocking_idx) {
                    let b_source = GridPoint {
                        row: blocking_ports.source_port.grid_row,
                        col: blocking_ports.source_port.grid_col,
                    };
                    let b_target = GridPoint {
                        row: blocking_ports.target_port.grid_row,
                        col: blocking_ports.target_port.grid_col,
                    };

                    let bs = save_and_free_cell(grid, b_source, &config.routing_costs);
                    let bt = save_and_free_cell(grid, b_target, &config.routing_costs);

                    let re_path = route_edge(grid, b_source, b_target, &config.routing_costs);

                    restore_cell(grid, b_source, bs);
                    restore_cell(grid, b_target, bt);

                    if let Some(rp) = re_path {
                        let b_edge_id = format!("edge_{}", blocking_idx);
                        commit_path(grid, &rp.points, &b_edge_id);
                        result.total_bends += rp.bend_count;
                        rerouted_paths.push((blocking_idx, rp));
                    } else {
                        all_rerouted = false;
                        break;
                    }
                }
            }

            if all_rerouted {
                // Success! Record all paths
                result.paths.insert(failed_edge_idx, path.clone());
                for (idx, rp) in rerouted_paths {
                    result.paths.insert(idx, rp);
                }
                return Some(path);
            }

            // Rollback: uncommit what we just committed.
            // Use result.paths (original committed edges) as the "other" set —
            // cells shared with originals are kept occupied, cells only in the
            // just-committed rerouted paths are freed.
            let rollback_others =
                build_other_cell_owners(&result.paths, &std::collections::HashSet::new());

            // Uncommit rerouted blocking edges
            for (idx, rp) in &rerouted_paths {
                let eid = format!("edge_{}", idx);
                uncommit_path(grid, &rp.points, &eid, &rollback_others);
                result.total_bends = result.total_bends.saturating_sub(rp.bend_count);
            }

            // Uncommit the failed edge
            uncommit_path(grid, &path.points, &edge_id, &rollback_others);
            result.total_bends = result.total_bends.saturating_sub(path.bend_count);

            // Restore original blocking edge paths. Use restore_path to
            // force-reclaim any cells whose owner was transferred during
            // uncommit — otherwise future uncommits of these edges would
            // miss those cells (ghost occupied cells).
            for (idx, sp) in saved_paths {
                let eid = format!("edge_{}", idx);
                restore_path(grid, &sp.points, &eid);
                result.total_bends += sp.bend_count;
                result.paths.insert(idx, sp);
            }
        } else {
            // Failed edge couldn't be routed even with blocking edges removed.
            for (idx, sp) in saved_paths {
                let eid = format!("edge_{}", idx);
                restore_path(grid, &sp.points, &eid);
                result.total_bends += sp.bend_count;
                result.paths.insert(idx, sp);
            }
        }
    }

    None
}

/// Save a cell's state and temporarily mark it as free for routing
fn save_and_free_cell(
    grid: &mut Grid,
    point: GridPoint,
    costs: &RoutingCosts,
) -> Option<CellState> {
    if !grid.in_bounds(point.row, point.col) {
        return None;
    }
    let row = point.row as usize;
    let col = point.col as usize;
    if let Some(cell) = grid.get(row, col) {
        let original_state = cell.state;
        if original_state == CellState::Blocked {
            if let Some(cell) = grid.get_mut(row, col) {
                cell.state = CellState::Free;
                cell.cost = costs.base_cost;
            }
        }
        Some(original_state)
    } else {
        None
    }
}

/// Restore a cell's original state after routing
fn restore_cell(grid: &mut Grid, point: GridPoint, original_state: Option<CellState>) {
    if let Some(state) = original_state {
        if state == CellState::Blocked && grid.in_bounds(point.row, point.col) {
            if let Some(cell) = grid.get_mut(point.row as usize, point.col as usize) {
                if cell.state != CellState::Occupied {
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
    use crate::grid::Grid;

    #[test]
    fn test_parse_edge_index() {
        assert_eq!(parse_edge_index("edge_0"), Some(0));
        assert_eq!(parse_edge_index("edge_42"), Some(42));
        assert_eq!(parse_edge_index("node_A"), None);
        assert_eq!(parse_edge_index("edge_"), None);
    }

    #[test]
    fn test_find_blocking_edges_empty_grid() {
        let grid = Grid::new(10, 10, 10, 0, 0);
        let port_assignments = HashMap::new();
        let result = find_blocking_edges(&grid, 0, &port_assignments);
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_blocking_edges_with_occupied_cells() {
        let mut grid = Grid::new(10, 10, 10, 0, 0);

        // Mark some cells as occupied by edge_1
        for col in 2..8 {
            if let Some(cell) = grid.get_mut(5, col) {
                cell.state = CellState::Occupied;
                cell.owner = Some("edge_1".to_string());
            }
        }

        // Create port assignments: edge 0 needs to cross from row 3 to row 7 at col 5
        let mut port_assignments = HashMap::new();
        port_assignments.insert(
            0,
            EdgePorts {
                source_port: crate::ports::Port {
                    x: 50.0,
                    y: 30.0,
                    grid_row: 3,
                    grid_col: 5,
                    side: crate::ports::Side::Bottom,
                },
                target_port: crate::ports::Port {
                    x: 50.0,
                    y: 70.0,
                    grid_row: 7,
                    grid_col: 5,
                    side: crate::ports::Side::Top,
                },
            },
        );

        let blocking = find_blocking_edges(&grid, 0, &port_assignments);
        assert!(!blocking.is_empty());
        assert!(blocking.contains(&1));
    }
}
