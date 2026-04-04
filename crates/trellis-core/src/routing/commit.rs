use crate::config::RoutingCosts;
use crate::grid::{CellState, Grid};
use crate::routing::astar::GridPoint;
use crate::routing::cost::ALL_DIRECTIONS;

/// Commit a routed path to the grid, marking cells as occupied
/// and increasing costs of adjacent cells.
///
/// This ensures that subsequent edge routes will avoid this path
/// and maintain separation between edges.
pub fn commit_path(grid: &mut Grid, path: &[GridPoint], edge_id: &str, costs: &RoutingCosts) {
    for point in path {
        if !grid.in_bounds(point.row, point.col) {
            continue;
        }

        let row = point.row as usize;
        let col = point.col as usize;

        if let Some(cell) = grid.get_mut(row, col) {
            if cell.state == CellState::Free {
                cell.state = CellState::Occupied;
                cell.owner = Some(edge_id.to_string());
            } else if cell.state == CellState::Occupied {
                // Two paths share this cell – mark it as a crossing so the
                // renderer draws a bridge and the invariant check accepts it.
                cell.crossing = true;
                cell.crossed_by = Some(edge_id.to_string());
            }
        }

        // Increase cost of adjacent cells to encourage separation
        for &dir in &ALL_DIRECTIONS {
            let (dr, dc) = dir.delta();
            let nr = point.row + dr;
            let nc = point.col + dc;

            if grid.in_bounds(nr, nc) {
                if let Some(adj_cell) = grid.get_mut(nr as usize, nc as usize) {
                    if adj_cell.state == CellState::Free {
                        adj_cell.cost += costs.adjacent_cost;
                    }
                }
            }
        }
    }
}

/// Uncommit (release) a previously committed path from the grid.
/// Used by rip-up-and-reroute in deadlock handling (M7).
pub fn uncommit_path(grid: &mut Grid, path: &[GridPoint], edge_id: &str, costs: &RoutingCosts) {
    for point in path {
        if !grid.in_bounds(point.row, point.col) {
            continue;
        }

        let row = point.row as usize;
        let col = point.col as usize;

        if let Some(cell) = grid.get_mut(row, col) {
            if cell.state == CellState::Occupied && cell.owner.as_deref() == Some(edge_id) {
                cell.state = CellState::Free;
                cell.owner = None;
                cell.cost = costs.base_cost;
            }
        }
    }

    // Recalculate adjacent costs (simplified: just reset to base)
    for point in path {
        for &dir in &ALL_DIRECTIONS {
            let (dr, dc) = dir.delta();
            let nr = point.row + dr;
            let nc = point.col + dc;

            if grid.in_bounds(nr, nc) {
                if let Some(adj_cell) = grid.get_mut(nr as usize, nc as usize) {
                    if adj_cell.state == CellState::Free {
                        adj_cell.cost = costs.base_cost;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RoutingCosts;
    use crate::grid::Grid;

    fn default_costs() -> RoutingCosts {
        RoutingCosts::default()
    }

    #[test]
    fn test_commit_path_marks_occupied() {
        let mut grid = Grid::new(10, 10, 10, 0, 0);
        let path = vec![
            GridPoint { row: 5, col: 0 },
            GridPoint { row: 5, col: 1 },
            GridPoint { row: 5, col: 2 },
        ];

        commit_path(&mut grid, &path, "edge_0", &default_costs());

        for p in &path {
            let cell = grid.get(p.row as usize, p.col as usize).unwrap();
            assert_eq!(cell.state, CellState::Occupied);
            assert_eq!(cell.owner.as_deref(), Some("edge_0"));
        }
    }

    #[test]
    fn test_commit_increases_adjacent_costs() {
        let mut grid = Grid::new(10, 10, 10, 0, 0);
        let path = vec![GridPoint { row: 5, col: 5 }];

        let original_cost = grid.get(4, 5).unwrap().cost;
        commit_path(&mut grid, &path, "edge_0", &default_costs());

        // Adjacent cells should have increased cost
        let adj_cost = grid.get(4, 5).unwrap().cost;
        assert!(adj_cost > original_cost);
    }

    #[test]
    fn test_uncommit_restores_free() {
        let mut grid = Grid::new(10, 10, 10, 0, 0);
        let path = vec![GridPoint { row: 5, col: 0 }, GridPoint { row: 5, col: 1 }];

        let costs = default_costs();
        commit_path(&mut grid, &path, "edge_0", &costs);
        uncommit_path(&mut grid, &path, "edge_0", &costs);

        for p in &path {
            let cell = grid.get(p.row as usize, p.col as usize).unwrap();
            assert_eq!(cell.state, CellState::Free);
            assert!(cell.owner.is_none());
        }
    }

    #[test]
    fn test_commit_does_not_overwrite_blocked() {
        let mut grid = Grid::new(10, 10, 10, 0, 0);
        grid.get_mut(5, 5).unwrap().state = CellState::Blocked;

        let path = vec![GridPoint { row: 5, col: 5 }];
        commit_path(&mut grid, &path, "edge_0", &default_costs());

        // Should still be blocked
        assert_eq!(grid.get(5, 5).unwrap().state, CellState::Blocked);
    }
}
