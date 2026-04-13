use std::collections::{BTreeMap, HashMap};

use crate::config::RoutingCosts;
use crate::grid::{CellState, Grid};
use crate::routing::astar::{GridPoint, RoutedPath};
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
///
/// Handles crossing cells correctly:
/// - If this edge was the first occupant (`owner`) at a crossing, the second
///   occupant (`crossed_by`) is promoted to `owner` and the crossing flag is
///   cleared. The cell stays `Occupied`.
/// - If this edge was the second occupant (`crossed_by`), the `crossed_by`
///   field and `crossing` flag are cleared. The original `owner` keeps the cell.
/// - Otherwise the cell is freed normally.
pub fn uncommit_path(grid: &mut Grid, path: &[GridPoint], edge_id: &str, costs: &RoutingCosts) {
    for point in path {
        if !grid.in_bounds(point.row, point.col) {
            continue;
        }

        let row = point.row as usize;
        let col = point.col as usize;

        if let Some(cell) = grid.get_mut(row, col) {
            if cell.owner.as_deref() == Some(edge_id) {
                if cell.crossing {
                    // Edge was the first occupant. Promote crossed_by to owner,
                    // clear crossing state. Cell stays Occupied.
                    cell.owner = cell.crossed_by.take();
                    cell.crossing = false;
                } else {
                    // Normal non-crossing cell: free it.
                    cell.state = CellState::Free;
                    cell.owner = None;
                    cell.cost = costs.base_cost;
                }
            } else if cell.crossed_by.as_deref() == Some(edge_id) {
                // Edge was the second occupant. Owner keeps the cell; clear crossing.
                cell.crossed_by = None;
                cell.crossing = false;
            }
        }
    }

    // Reset adjacent cell costs for newly freed cells
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

/// Rebuild all crossing metadata on the grid from the authoritative `paths` map.
///
/// Clears every `crossing` / `crossed_by` flag, then re-derives them by
/// scanning which cells are shared by two or more committed edge paths.
/// Call this after any reroute pass so the grid reflects the actual paths.
pub fn reconcile_crossings(grid: &mut Grid, paths: &BTreeMap<usize, RoutedPath>) {
    // Step 1: Clear all crossing metadata.
    for row in 0..grid.rows {
        for col in 0..grid.cols {
            if let Some(cell) = grid.get_mut(row, col) {
                cell.crossing = false;
                cell.crossed_by = None;
            }
        }
    }

    // Step 2: Build cell → edge-index list from paths.
    let mut cell_edges: HashMap<(usize, usize), Vec<usize>> = HashMap::new();
    for (&edge_idx, path) in paths {
        for pt in &path.points {
            if grid.in_bounds(pt.row, pt.col) {
                cell_edges
                    .entry((pt.row as usize, pt.col as usize))
                    .or_default()
                    .push(edge_idx);
            }
        }
    }

    // Step 3: Mark cells shared by 2+ edges as crossings.
    for ((row, col), edges) in &cell_edges {
        if edges.len() >= 2 {
            if let Some(cell) = grid.get_mut(*row, *col) {
                cell.crossing = true;
                let mut sorted = edges.clone();
                sorted.sort_unstable();
                // owner is already set by commit_path; just set crossed_by.
                cell.crossed_by = Some(format!("edge_{}", sorted[1]));
            }
        }
    }
}

/// Derive per-edge crossing-point lists from the `paths` map.
///
/// Only the **second** occupant at each crossing cell (the edge identified by
/// `cell.crossed_by` after `reconcile_crossings`) receives a hop arc.  The
/// first occupant (`cell.owner`) passes straight through unchanged.
///
/// This ensures exactly one edge hops at every crossing — the grid's
/// `crossed_by` field is the authoritative source for which edge that is.
pub fn compute_crossing_points(
    paths: &BTreeMap<usize, RoutedPath>,
    grid: &Grid,
) -> HashMap<usize, Vec<(i64, i64)>> {
    // Build cell → edge list.
    let mut cell_edges: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (&edge_idx, path) in paths {
        for pt in &path.points {
            cell_edges
                .entry((pt.row, pt.col))
                .or_default()
                .push(edge_idx);
        }
    }

    // For each crossing cell, give the hop arc only to the second occupant.
    let mut result: HashMap<usize, Vec<(i64, i64)>> = HashMap::new();
    for ((row, col), edges) in &cell_edges {
        if edges.len() < 2 {
            continue;
        }

        // `reconcile_crossings` sets `crossed_by` to "edge_N" for the second
        // occupant. Parse that to find which edge index hops.
        let hopper = if grid.in_bounds(*row, *col) {
            grid.get(*row as usize, *col as usize)
                .and_then(|cell| cell.crossed_by.as_deref())
                .and_then(|id| id.strip_prefix("edge_"))
                .and_then(|s| s.parse::<usize>().ok())
                .filter(|idx| edges.contains(idx))
        } else {
            None
        };

        // Fallback: if grid has no crossing metadata yet, assign the hop to
        // the edge with the highest index (last-routed approximation).
        let hopper_idx = hopper.unwrap_or_else(|| *edges.iter().max().unwrap());
        result.entry(hopper_idx).or_default().push((*row, *col));
    }
    result
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

    #[test]
    fn test_uncommit_owner_at_crossing_promotes_crossed_by() {
        let mut grid = Grid::new(10, 10, 10, 0, 0);
        let costs = default_costs();

        // Simulate a crossing at (5,5): edge_0 is owner, edge_1 is crossed_by
        {
            let cell = grid.get_mut(5, 5).unwrap();
            cell.state = CellState::Occupied;
            cell.owner = Some("edge_0".to_string());
            cell.crossing = true;
            cell.crossed_by = Some("edge_1".to_string());
        }

        let path = vec![GridPoint { row: 5, col: 5 }];
        uncommit_path(&mut grid, &path, "edge_0", &costs);

        let cell = grid.get(5, 5).unwrap();
        // Cell stays Occupied under edge_1
        assert_eq!(cell.state, CellState::Occupied);
        assert_eq!(cell.owner.as_deref(), Some("edge_1"));
        assert!(!cell.crossing);
        assert!(cell.crossed_by.is_none());
    }

    #[test]
    fn test_uncommit_crossed_by_clears_crossing_keeps_owner() {
        let mut grid = Grid::new(10, 10, 10, 0, 0);
        let costs = default_costs();

        // Simulate a crossing at (5,5): edge_0 is owner, edge_1 is crossed_by
        {
            let cell = grid.get_mut(5, 5).unwrap();
            cell.state = CellState::Occupied;
            cell.owner = Some("edge_0".to_string());
            cell.crossing = true;
            cell.crossed_by = Some("edge_1".to_string());
        }

        let path = vec![GridPoint { row: 5, col: 5 }];
        uncommit_path(&mut grid, &path, "edge_1", &costs);

        let cell = grid.get(5, 5).unwrap();
        // Cell stays Occupied under edge_0
        assert_eq!(cell.state, CellState::Occupied);
        assert_eq!(cell.owner.as_deref(), Some("edge_0"));
        assert!(!cell.crossing);
        assert!(cell.crossed_by.is_none());
    }

    #[test]
    fn test_compute_crossing_points_only_hopper_gets_arc() {
        use crate::routing::astar::RoutedPath;
        use std::collections::BTreeMap;

        // edge 0 horizontal, edge 1 vertical — they cross at (3,2).
        let path_a = RoutedPath {
            points: vec![
                GridPoint { row: 3, col: 0 },
                GridPoint { row: 3, col: 1 },
                GridPoint { row: 3, col: 2 }, // shared
            ],
            bend_count: 0,
            total_cost: 0.0,
        };
        let path_b = RoutedPath {
            points: vec![
                GridPoint { row: 2, col: 2 },
                GridPoint { row: 3, col: 2 }, // shared
                GridPoint { row: 4, col: 2 },
            ],
            bend_count: 0,
            total_cost: 0.0,
        };

        let mut paths = BTreeMap::new();
        paths.insert(0usize, path_a);
        paths.insert(1usize, path_b);

        // Set up grid with crossing metadata: edge_1 is the hopper.
        let mut grid = Grid::new(10, 10, 10, 0, 0);
        {
            let cell = grid.get_mut(3, 2).unwrap();
            cell.state = CellState::Occupied;
            cell.owner = Some("edge_0".to_string());
            cell.crossing = true;
            cell.crossed_by = Some("edge_1".to_string());
        }

        let crossing_pts = compute_crossing_points(&paths, &grid);

        // Only edge 1 (the hopper) should have the arc.
        assert!(!crossing_pts.contains_key(&0), "owner should not hop");
        assert!(crossing_pts.contains_key(&1), "crossed_by edge should hop");
        assert!(crossing_pts[&1].contains(&(3, 2)));
    }

    #[test]
    fn test_reconcile_crossings_rebuilds_flags() {
        use crate::routing::astar::RoutedPath;
        use std::collections::BTreeMap;

        let mut grid = Grid::new(10, 10, 10, 0, 0);

        // Pre-corrupt: mark a cell as crossing that shouldn't be
        {
            let cell = grid.get_mut(1, 1).unwrap();
            cell.state = CellState::Occupied;
            cell.crossing = true;
            cell.crossed_by = Some("stale".to_string());
        }

        let path_a = RoutedPath {
            points: vec![GridPoint { row: 3, col: 0 }, GridPoint { row: 3, col: 2 }],
            bend_count: 0,
            total_cost: 0.0,
        };
        let path_b = RoutedPath {
            points: vec![GridPoint { row: 2, col: 1 }, GridPoint { row: 4, col: 1 }],
            bend_count: 0,
            total_cost: 0.0,
        };

        let mut paths = BTreeMap::new();
        paths.insert(0usize, path_a);
        paths.insert(1usize, path_b);

        reconcile_crossings(&mut grid, &paths);

        // Stale crossing at (1,1) should be cleared
        let stale = grid.get(1, 1).unwrap();
        assert!(!stale.crossing);
        assert!(stale.crossed_by.is_none());
    }
}
