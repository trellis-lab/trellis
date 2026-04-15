use std::collections::HashMap;

use crate::config::{RoutingCosts, TrellisConfig};
use crate::grid::{CellState, Grid};
use crate::ports::EdgePorts;
use crate::routing::astar::{route_edge, GridPoint, RoutedPath};
use crate::routing::commit::commit_path;

/// Route an edge allowing crossings through occupied cells.
///
/// This is the Level 3 (last resort) deadlock resolution. It temporarily
/// modifies the grid so that occupied cells are passable (with high but finite
/// cost), routes the edge, then marks crossing points for visual rendering.
///
/// Returns the path (which may cross existing edges).
pub fn route_with_crossings_allowed(
    grid: &mut Grid,
    failed_edge_idx: usize,
    port_assignments: &HashMap<usize, EdgePorts>,
    config: &TrellisConfig,
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

    // Save original states of occupied cells so we can restore them
    let mut saved_cells: Vec<(usize, usize, CellState, f64)> = Vec::new();

    for row in 0..grid.rows {
        for col in 0..grid.cols {
            if let Some(cell) = grid.get(row, col) {
                if cell.state == CellState::Occupied {
                    saved_cells.push((row, col, cell.state, cell.cost));
                    // Temporarily make occupied cells passable with crossing cost
                    if let Some(cell_mut) = grid.get_mut(row, col) {
                        cell_mut.cost = config.routing_costs.crossing_cost;
                    }
                }
            }
        }
    }

    // Use modified costs that allow crossing: occupied cells get crossing_cost
    // instead of being treated as impassable
    let crossing_costs = RoutingCosts {
        blocked_cost: config.routing_costs.blocked_cost,
        // Other costs stay the same — the key change is that occupied cells
        // already have their cost set to crossing_cost above, and movement_cost
        // returns crossing_cost + base_cost for Occupied cells, which is below
        // blocked_cost, so A* will traverse them.
        ..config.routing_costs.clone()
    };

    // Temporarily free source/target if blocked
    let source_state = save_and_free_cell(grid, source, &config.routing_costs);
    let target_state = save_and_free_cell(grid, target, &config.routing_costs);

    let path = route_edge(grid, source, target, &crossing_costs);

    restore_cell(grid, source, source_state);
    restore_cell(grid, target, target_state);

    // Restore original occupied cell costs
    for (row, col, _state, cost) in &saved_cells {
        if let Some(cell) = grid.get_mut(*row, *col) {
            // Only restore if still occupied (not overwritten)
            if cell.state == CellState::Occupied {
                cell.cost = *cost;
            }
        }
    }

    if let Some(path) = path {
        let edge_id = format!("edge_{}", failed_edge_idx);
        commit_path(grid, &path.points, &edge_id);

        return Some(path);
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
    use crate::ports::{Port, Side};

    #[test]
    fn test_crossing_fallback_routes_through_occupied() {
        let mut grid = Grid::new(10, 10, 10, 0, 0);

        // Create a wall of occupied cells at col 5, rows 0-9
        for row in 0..10 {
            if let Some(cell) = grid.get_mut(row, 5) {
                cell.state = CellState::Occupied;
                cell.owner = Some("edge_99".to_string());
            }
        }

        let mut port_assignments = HashMap::new();
        port_assignments.insert(
            0,
            EdgePorts {
                source_port: Port {
                    x: 30.0,
                    y: 50.0,
                    grid_row: 5,
                    grid_col: 3,
                    side: Side::Right,
                },
                target_port: Port {
                    x: 70.0,
                    y: 50.0,
                    grid_row: 5,
                    grid_col: 7,
                    side: Side::Left,
                },
            },
        );

        let config = TrellisConfig::default();
        let path = route_with_crossings_allowed(&mut grid, 0, &port_assignments, &config);

        assert!(path.is_some(), "Crossing fallback should find a path");
        let path = path.unwrap();
        assert!(!path.points.is_empty());

        // Verify the path crosses through at least one previously-occupied cell (col 5).
        // Crossing detection is now paths-based; the wall at col 5 was committed as Occupied,
        // so any path point at col 5 indicates a crossing.
        let crosses_wall = path
            .points
            .iter()
            .any(|pt| pt.col == 5 && grid.in_bounds(pt.row, pt.col));
        assert!(crosses_wall, "Path should cross through the wall at col 5");
    }
}
