use crate::config::RoutingCosts;
use crate::grid::{CellState, Grid};

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

            // Adjacent occupied cell penalty
            cost += adjacent_penalty(grid, to_row, to_col, costs);

            cost
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
pub fn manhattan_distance(
    from_row: i64,
    from_col: i64,
    to_row: i64,
    to_col: i64,
) -> f64 {
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
            &grid, 5, 5,
            Some(Direction::Right), Direction::Down,
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
}
