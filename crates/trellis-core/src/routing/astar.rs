use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::config::RoutingCosts;
use crate::grid::Grid;
use crate::routing::cost::{manhattan_distance, movement_cost, Direction, ALL_DIRECTIONS};

/// A point on the grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPoint {
    pub row: i64,
    pub col: i64,
}

/// A routed path on the grid
#[derive(Debug, Clone)]
pub struct RoutedPath {
    /// Sequence of grid points from source to target
    pub points: Vec<GridPoint>,
    /// Total cost of the path
    pub total_cost: f64,
    /// Number of bends in the path
    pub bend_count: usize,
}

/// A* search node
#[derive(Debug, Clone)]
struct AStarNode {
    point: GridPoint,
    g_cost: f64,       // actual cost from start
    f_cost: f64,       // g + heuristic
    direction: Option<Direction>, // direction we arrived from
}

impl PartialEq for AStarNode {
    fn eq(&self, other: &Self) -> bool {
        self.f_cost == other.f_cost
    }
}

impl Eq for AStarNode {}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap (lower f_cost = higher priority)
        other
            .f_cost
            .partial_cmp(&self.f_cost)
            .unwrap_or(Ordering::Equal)
    }
}

/// Route a single edge using A* pathfinding on the grid.
///
/// Returns `None` if no path can be found (deadlock situation).
pub fn route_edge(
    grid: &Grid,
    source: GridPoint,
    target: GridPoint,
    costs: &RoutingCosts,
) -> Option<RoutedPath> {
    if source == target {
        return Some(RoutedPath {
            points: vec![source],
            total_cost: 0.0,
            bend_count: 0,
        });
    }

    let mut open_set = BinaryHeap::new();
    let mut g_scores: HashMap<GridPoint, f64> = HashMap::new();
    let mut came_from: HashMap<GridPoint, (GridPoint, Option<Direction>)> = HashMap::new();

    let h = manhattan_distance(source.row, source.col, target.row, target.col) * costs.base_cost;

    open_set.push(AStarNode {
        point: source,
        g_cost: 0.0,
        f_cost: h,
        direction: None,
    });
    g_scores.insert(source, 0.0);

    while let Some(current) = open_set.pop() {
        if current.point == target {
            // Reconstruct path
            let path = reconstruct_path(&came_from, target, source);
            let bend_count = count_bends(&path);
            return Some(RoutedPath {
                points: path,
                total_cost: current.g_cost,
                bend_count,
            });
        }

        let current_g = *g_scores.get(&current.point).unwrap_or(&f64::INFINITY);
        if current.g_cost > current_g {
            continue; // Skip stale entries
        }

        // Explore neighbors
        for &dir in &ALL_DIRECTIONS {
            let (dr, dc) = dir.delta();
            let nr = current.point.row + dr;
            let nc = current.point.col + dc;

            if !grid.in_bounds(nr, nc) {
                continue;
            }

            let neighbor = GridPoint { row: nr, col: nc };

            let move_cost = movement_cost(
                grid,
                nr as usize,
                nc as usize,
                current.direction,
                dir,
                costs,
            );

            if move_cost >= costs.blocked_cost {
                continue; // Don't route through blocked cells
            }

            let tentative_g = current_g + move_cost;
            let existing_g = *g_scores.get(&neighbor).unwrap_or(&f64::INFINITY);

            if tentative_g < existing_g {
                g_scores.insert(neighbor, tentative_g);
                came_from.insert(neighbor, (current.point, Some(dir)));

                let h = manhattan_distance(nr, nc, target.row, target.col) * costs.base_cost;
                open_set.push(AStarNode {
                    point: neighbor,
                    g_cost: tentative_g,
                    f_cost: tentative_g + h,
                    direction: Some(dir),
                });
            }
        }
    }

    None // No path found
}

/// Reconstruct the path from target back to source using the came_from map
fn reconstruct_path(
    came_from: &HashMap<GridPoint, (GridPoint, Option<Direction>)>,
    target: GridPoint,
    source: GridPoint,
) -> Vec<GridPoint> {
    let mut path = vec![target];
    let mut current = target;

    while current != source {
        match came_from.get(&current) {
            Some(&(prev, _)) => {
                path.push(prev);
                current = prev;
            }
            None => break,
        }
    }

    path.reverse();
    path
}

/// Count the number of bends (direction changes) in a path
fn count_bends(path: &[GridPoint]) -> usize {
    if path.len() < 3 {
        return 0;
    }

    let mut bends = 0;
    for i in 1..path.len() - 1 {
        let dr1 = path[i].row - path[i - 1].row;
        let dc1 = path[i].col - path[i - 1].col;
        let dr2 = path[i + 1].row - path[i].row;
        let dc2 = path[i + 1].col - path[i].col;

        if dr1 != dr2 || dc1 != dc2 {
            bends += 1;
        }
    }
    bends
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::RoutingCosts;
    use crate::grid::{CellState, Grid};

    fn default_costs() -> RoutingCosts {
        RoutingCosts::default()
    }

    #[test]
    fn test_route_straight_line() {
        let grid = Grid::new(10, 10, 10.0, 0.0, 0.0);
        let source = GridPoint { row: 0, col: 0 };
        let target = GridPoint { row: 0, col: 5 };

        let result = route_edge(&grid, source, target, &default_costs());
        assert!(result.is_some());

        let path = result.unwrap();
        assert_eq!(path.points.first(), Some(&source));
        assert_eq!(path.points.last(), Some(&target));
        assert_eq!(path.bend_count, 0);
        assert_eq!(path.points.len(), 6); // 0,1,2,3,4,5
    }

    #[test]
    fn test_route_around_obstacle() {
        let mut grid = Grid::new(10, 10, 10.0, 0.0, 0.0);
        // Block a wall at col 3, rows 0-8
        for row in 0..9 {
            grid.get_mut(row, 3).unwrap().state = CellState::Blocked;
            grid.get_mut(row, 3).unwrap().cost = f64::INFINITY;
        }

        let source = GridPoint { row: 5, col: 0 };
        let target = GridPoint { row: 5, col: 6 };

        let result = route_edge(&grid, source, target, &default_costs());
        assert!(result.is_some());

        let path = result.unwrap();
        // Path should go around the wall (through row 9 gap)
        assert_eq!(path.points.first(), Some(&source));
        assert_eq!(path.points.last(), Some(&target));
        // Should not pass through any blocked cell
        for p in &path.points {
            if grid.in_bounds(p.row, p.col) {
                assert_ne!(
                    grid.get(p.row as usize, p.col as usize).unwrap().state,
                    CellState::Blocked,
                    "Path should not go through blocked cell ({}, {})",
                    p.row,
                    p.col
                );
            }
        }
    }

    #[test]
    fn test_route_same_point() {
        let grid = Grid::new(10, 10, 10.0, 0.0, 0.0);
        let point = GridPoint { row: 5, col: 5 };

        let result = route_edge(&grid, point, point, &default_costs());
        assert!(result.is_some());
        assert_eq!(result.unwrap().points.len(), 1);
    }

    #[test]
    fn test_no_path_completely_blocked() {
        let mut grid = Grid::new(5, 5, 10.0, 0.0, 0.0);
        // Block all cells except source and target in separate regions
        for row in 0..5 {
            for col in 0..5 {
                if !(row == 0 && col == 0) && !(row == 4 && col == 4) {
                    grid.get_mut(row, col).unwrap().state = CellState::Blocked;
                    grid.get_mut(row, col).unwrap().cost = f64::INFINITY;
                }
            }
        }

        let source = GridPoint { row: 0, col: 0 };
        let target = GridPoint { row: 4, col: 4 };

        let result = route_edge(&grid, source, target, &default_costs());
        assert!(result.is_none());
    }

    #[test]
    fn test_count_bends() {
        // Straight line: no bends
        let path = vec![
            GridPoint { row: 0, col: 0 },
            GridPoint { row: 0, col: 1 },
            GridPoint { row: 0, col: 2 },
        ];
        assert_eq!(count_bends(&path), 0);

        // L-shape: one bend
        let path = vec![
            GridPoint { row: 0, col: 0 },
            GridPoint { row: 0, col: 1 },
            GridPoint { row: 1, col: 1 },
        ];
        assert_eq!(count_bends(&path), 1);

        // U-shape: two bends
        let path = vec![
            GridPoint { row: 0, col: 0 },
            GridPoint { row: 0, col: 1 },
            GridPoint { row: 1, col: 1 },
            GridPoint { row: 1, col: 0 },
        ];
        assert_eq!(count_bends(&path), 2);
    }
}
