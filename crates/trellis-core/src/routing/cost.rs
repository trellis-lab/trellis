use crate::config::RoutingCosts;
use crate::grid::{BoundarySide, CellState, Grid};

/// Direction of movement on the grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Get the (row_delta, col_delta) for this direction
    pub fn delta(self) -> (i64, i64) {
        match self {
            Direction::Up => (-1, 0),
            Direction::Down => (1, 0),
            Direction::Left => (0, -1),
            Direction::Right => (0, 1),
        }
    }

    /// Check if two directions are perpendicular (i.e., moving to the new direction is a bend)
    pub fn is_bend(self, other: Direction) -> bool {
        self != other
    }
}

/// All four orthogonal directions
pub const ALL_DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Down,
    Direction::Left,
    Direction::Right,
];

/// Calculate the cost of moving from one cell to an adjacent cell.
///
/// Takes into account:
/// - Base movement cost
/// - Bend cost (direction change)
/// - Adjacent occupied cell cost increase
/// - Crossing cost (moving through an occupied cell)
/// - Blocked cost (moving through a node cell)
pub fn movement_cost(
    grid: &Grid,
    to_row: usize,
    to_col: usize,
    prev_direction: Option<Direction>,
    new_direction: Direction,
    costs: &RoutingCosts,
) -> f64 {
    let cell = match grid.get(to_row, to_col) {
        Some(c) => c,
        None => return f64::INFINITY,
    };

    match cell.state {
        CellState::Blocked => costs.blocked_cost,
        CellState::Occupied => costs.crossing_cost + costs.base_cost,
        CellState::Free => {
            let mut cost = costs.base_cost;

            // Bend penalty
            if let Some(prev) = prev_direction {
                if prev.is_bend(new_direction) {
                    cost += costs.bend_cost;
                }
            }

            // Adjacent occupied cell penalty.
            // Skipped on connector and approach-zone cells (boundary_side.is_some()).
            // These cells are constrained port exits/entries; penalising adjacency
            // here pushes A* off the straight corridor between co-linear ports,
            // producing the "exit-then-return" Z-shape bend pattern.
            if cell.boundary_side.is_none() {
                cost += adjacent_penalty(grid, to_row, to_col, costs);
            }

            // Perpendicularity penalty: penalize movement parallel to a node boundary
            // on connector cells (directly on the boundary) to ensure the very
            // first/last edge segment is perpendicular.
            // Only applied to connector cells, NOT approach-zone cells, because
            // approach-zone penalties would penalize pass-through traffic and
            // push edges into worse routes (e.g., crossing existing edges).
            if cell.is_boundary_connector {
                if let Some(boundary_side) = cell.boundary_side {
                    if is_parallel_to_boundary(new_direction, boundary_side) {
                        cost += costs.perpendicular_cost * 4.0;
                    }
                }
            }

            cost
        }
    }
}

/// Check if a movement direction is parallel to a node boundary side.
///
/// Top/Bottom boundaries run horizontally, so Left/Right movement is parallel.
/// Left/Right boundaries run vertically, so Up/Down movement is parallel.
fn is_parallel_to_boundary(direction: Direction, boundary: BoundarySide) -> bool {
    match boundary {
        BoundarySide::Top | BoundarySide::Bottom => {
            matches!(direction, Direction::Left | Direction::Right)
        }
        BoundarySide::Left | BoundarySide::Right => {
            matches!(direction, Direction::Up | Direction::Down)
        }
    }
}

/// Calculate penalty for being adjacent to occupied cells.
/// This encourages edges to spread apart rather than bunching together.
fn adjacent_penalty(grid: &Grid, row: usize, col: usize, costs: &RoutingCosts) -> f64 {
    let mut penalty = 0.0;
    for &dir in &ALL_DIRECTIONS {
        let (dr, dc) = dir.delta();
        let nr = row as i64 + dr;
        let nc = col as i64 + dc;
        if grid.in_bounds(nr, nc) {
            if let Some(cell) = grid.get(nr as usize, nc as usize) {
                if cell.state == CellState::Occupied {
                    penalty += costs.adjacent_cost;
                }
            }
        }
    }
    penalty
}

/// Manhattan distance heuristic for A*
pub fn manhattan_distance(from_row: i64, from_col: i64, to_row: i64, to_col: i64) -> f64 {
    ((from_row - to_row).abs() + (from_col - to_col).abs()) as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;

    fn default_costs() -> RoutingCosts {
        RoutingCosts::default()
    }

    #[test]
    fn test_movement_cost_free_cell() {
        let grid = Grid::new(10, 10, 10, 0, 0);
        let cost = movement_cost(&grid, 5, 5, None, Direction::Right, &default_costs());
        assert!((cost - 1.0).abs() < 0.01); // base cost only
    }

    #[test]
    fn test_movement_cost_bend() {
        let grid = Grid::new(10, 10, 10, 0, 0);
        let cost = movement_cost(
            &grid,
            5,
            5,
            Some(Direction::Right),
            Direction::Down,
            &default_costs(),
        );
        assert!((cost - 3.0).abs() < 0.01); // base + bend
    }

    #[test]
    fn test_movement_cost_blocked() {
        let mut grid = Grid::new(10, 10, 10, 0, 0);
        grid.get_mut(5, 5).unwrap().state = CellState::Blocked;
        let cost = movement_cost(&grid, 5, 5, None, Direction::Right, &default_costs());
        assert!((cost - 1000.0).abs() < 0.01);
    }

    #[test]
    fn test_manhattan_distance() {
        assert!((manhattan_distance(0, 0, 3, 4) - 7.0).abs() < 0.01);
        assert!((manhattan_distance(5, 5, 5, 5) - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_perpendicular_penalty_parallel_to_top_boundary_connector() {
        let mut grid = Grid::new(10, 10, 10, 0, 0);
        let cell = grid.get_mut(3, 5).unwrap();
        cell.boundary_side = Some(BoundarySide::Top);
        cell.is_boundary_connector = true;

        let costs = default_costs();
        // Moving Left along a Top boundary connector → parallel → should incur 4x penalty
        let cost = movement_cost(&grid, 3, 5, None, Direction::Left, &costs);
        let expected = costs.base_cost + costs.perpendicular_cost * 4.0;
        assert!(
            (cost - expected).abs() < 0.01,
            "Parallel movement along Top boundary connector should add 4x perpendicular_cost"
        );
    }

    #[test]
    fn test_perpendicular_no_penalty_when_approaching_perpendicularly() {
        let mut grid = Grid::new(10, 10, 10, 0, 0);
        let cell = grid.get_mut(3, 5).unwrap();
        cell.boundary_side = Some(BoundarySide::Top);
        cell.is_boundary_connector = true;

        let costs = default_costs();
        // Moving Down toward a Top boundary → perpendicular → no penalty
        let cost = movement_cost(&grid, 3, 5, None, Direction::Down, &costs);
        assert!(
            (cost - costs.base_cost).abs() < 0.01,
            "Perpendicular movement toward Top boundary should have no extra cost"
        );
    }

    #[test]
    fn test_perpendicular_penalty_parallel_to_right_boundary_connector() {
        let mut grid = Grid::new(10, 10, 10, 0, 0);
        let cell = grid.get_mut(5, 7).unwrap();
        cell.boundary_side = Some(BoundarySide::Right);
        cell.is_boundary_connector = true;

        let costs = default_costs();
        // Moving Up along a Right boundary connector → parallel → should incur 4x penalty
        let cost = movement_cost(&grid, 5, 7, None, Direction::Up, &costs);
        let expected = costs.base_cost + costs.perpendicular_cost * 4.0;
        assert!(
            (cost - expected).abs() < 0.01,
            "Parallel movement along Right boundary connector should add 4x perpendicular_cost"
        );
        // Moving Right toward a Right boundary → perpendicular → no penalty
        let cost = movement_cost(&grid, 5, 7, None, Direction::Right, &costs);
        assert!(
            (cost - costs.base_cost).abs() < 0.01,
            "Perpendicular movement toward Right boundary should have no extra cost"
        );
    }

    #[test]
    fn test_no_perpendicular_penalty_on_plain_free_cell() {
        let grid = Grid::new(10, 10, 10, 0, 0);
        // Cell with no boundary_side → no perpendicular penalty regardless of direction
        let costs = default_costs();
        let cost = movement_cost(&grid, 5, 5, None, Direction::Left, &costs);
        assert!((cost - costs.base_cost).abs() < 0.01);
    }

    #[test]
    fn test_connector_cell_gets_stronger_perpendicular_penalty() {
        let mut grid = Grid::new(10, 10, 10, 0, 0);
        let cell = grid.get_mut(5, 7).unwrap();
        cell.boundary_side = Some(BoundarySide::Left);
        cell.is_boundary_connector = true;

        let costs = default_costs();
        // Moving Down along a Left boundary connector → parallel → 4x penalty
        let cost = movement_cost(&grid, 5, 7, None, Direction::Down, &costs);
        let expected = costs.base_cost + costs.perpendicular_cost * 4.0;
        assert!(
            (cost - expected).abs() < 0.01,
            "Connector cell should get 4x perpendicular_cost, expected {}, got {}",
            expected,
            cost
        );
    }

    #[test]
    fn test_approach_zone_gets_no_perpendicular_penalty() {
        let costs = default_costs();

        // Approach zone cell (not a connector) — should get NO perpendicular penalty
        let mut grid = Grid::new(10, 10, 10, 0, 0);
        grid.get_mut(5, 7).unwrap().boundary_side = Some(BoundarySide::Left);
        let approach_cost = movement_cost(&grid, 5, 7, None, Direction::Down, &costs);
        assert!(
            (approach_cost - costs.base_cost).abs() < 0.01,
            "Approach zone cell should not incur perpendicular penalty, got {}",
            approach_cost
        );

        // Connector cell — should get 4x penalty
        let mut grid2 = Grid::new(10, 10, 10, 0, 0);
        let cell = grid2.get_mut(5, 7).unwrap();
        cell.boundary_side = Some(BoundarySide::Left);
        cell.is_boundary_connector = true;
        let connector_cost = movement_cost(&grid2, 5, 7, None, Direction::Down, &costs);

        assert!(
            connector_cost > approach_cost,
            "Connector penalty ({}) should be stronger than approach zone ({})",
            connector_cost,
            approach_cost
        );
    }
}
