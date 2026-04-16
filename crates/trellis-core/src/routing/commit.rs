use std::collections::{BTreeMap, HashMap, HashSet};

use crate::grid::{CellState, Grid};
use crate::routing::astar::{GridPoint, RoutedPath};

/// Commit a routed path to the grid, marking cells as occupied.
///
/// Only Free cells are claimed; cells already Occupied (by a previously
/// committed edge that crosses this one) are left unchanged.  Blocked cells
/// (node interiors / corners) are never touched.
///
/// Adjacent cost inflation is NOT stored on cells — the A* cost function
/// (`movement_cost`) derives the adjacent penalty from live cell states at
/// query time, so pre-cached values would only go stale after reroutes.
pub fn commit_path(grid: &mut Grid, path: &[GridPoint], edge_id: &str) {
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
            }
            // Occupied or Blocked cells are left as-is.
            // Crossings between edges are detected purely from the `paths`
            // map at render time via `compute_crossing_points`.
        }
    }
}

/// Build a map of `(row, col) → edge_id` for all cells used by committed
/// paths *excluding* the given indices.
///
/// Used by `uncommit_path` callers to identify cells that must stay Occupied
/// because another committed edge still passes through them.
pub fn build_other_cell_owners(
    paths: &BTreeMap<usize, RoutedPath>,
    excluded: &HashSet<usize>,
) -> HashMap<(i64, i64), String> {
    let mut map: HashMap<(i64, i64), String> = HashMap::new();
    for (&idx, path) in paths {
        if excluded.contains(&idx) {
            continue;
        }
        let id = format!("edge_{}", idx);
        for pt in &path.points {
            // First writer wins: lowest-index edge becomes the canonical owner
            // for any cell shared by multiple edges.
            map.entry((pt.row, pt.col)).or_insert_with(|| id.clone());
        }
    }
    map
}

/// Uncommit (release) a previously committed path from the grid.
///
/// For each cell in `path`:
/// - If only this edge uses the cell (not in `other_cell_owners`): free it.
/// - If another committed edge also passes through the cell: keep it Occupied
///   and update the owner to that other edge so future uncommits work correctly.
///
/// `other_cell_owners` is built by the caller via `build_other_cell_owners`,
/// which excludes all edges being uncommitted in the same operation (e.g. both
/// sides of a port-swap).  This ensures cells shared only between simultaneously
/// uncommitted edges are freed, while cells shared with stable edges are kept.
pub fn uncommit_path(
    grid: &mut Grid,
    path: &[GridPoint],
    edge_id: &str,
    other_cell_owners: &HashMap<(i64, i64), String>,
) {
    for point in path {
        if !grid.in_bounds(point.row, point.col) {
            continue;
        }

        let row = point.row as usize;
        let col = point.col as usize;

        if let Some(cell) = grid.get_mut(row, col) {
            if cell.owner.as_deref() == Some(edge_id) {
                match other_cell_owners.get(&(point.row, point.col)) {
                    Some(other_id) => {
                        // Cell is also used by another committed edge.
                        // Keep it Occupied; transfer ownership so the other
                        // edge's future uncommit can free it correctly.
                        cell.owner = Some(other_id.clone());
                    }
                    None => {
                        // This edge is the sole occupant — free the cell.
                        cell.state = CellState::Free;
                        cell.owner = None;
                    }
                }
            }
        }
    }
}

/// Restore a previously committed path after a rejected reroute attempt.
///
/// Unlike `commit_path`, this function force-sets `cell.owner` even on cells
/// that are already Occupied.  This is necessary when a rollback needs to undo
/// an ownership transfer that `uncommit_path` performed during the attempt:
///
/// ```text
/// edge_A owned cell (r,c) → uncommit transferred owner to edge_C
///                          → attempt rejected → commit_path restores A's path
///                          → but commit_path sees Occupied → leaves owner as edge_C
///                          → next uncommit of A misses (r,c) → ghost cell
/// ```
///
/// `restore_path` fixes this by always writing `edge_id` as owner for any
/// Occupied cell on the path.  Blocked cells (node interiors) are never touched.
pub fn restore_path(grid: &mut Grid, path: &[GridPoint], edge_id: &str) {
    for point in path {
        if !grid.in_bounds(point.row, point.col) {
            continue;
        }

        let row = point.row as usize;
        let col = point.col as usize;

        if let Some(cell) = grid.get_mut(row, col) {
            match cell.state {
                CellState::Free => {
                    cell.state = CellState::Occupied;
                    cell.owner = Some(edge_id.to_string());
                }
                CellState::Occupied => {
                    // Force-restore ownership. This cell may have had its owner
                    // transferred to another edge during the failed attempt's
                    // uncommit. Reclaim it so future uncommits of this edge work.
                    cell.owner = Some(edge_id.to_string());
                }
                CellState::Blocked => {
                    // Never touch node interior cells.
                }
            }
        }
    }
}

/// Derive per-edge crossing-point lists purely from the `paths` map.
///
/// For every grid cell shared by two or more distinct edges, the edge with the
/// **higher** index is designated as the "hopper" that renders the arc or
/// bridge decoration.  The lower-index edge (committed first) passes straight
/// through unchanged.
///
/// This function is self-contained: it does not read any grid crossing metadata
/// (which could be stale after reroutes) — crossing state is always derived
/// fresh from the authoritative `paths` map.
pub fn compute_crossing_points(
    paths: &BTreeMap<usize, RoutedPath>,
) -> HashMap<usize, Vec<(i64, i64)>> {
    // Map each cell to all edge indices whose path includes it.
    let mut cell_edges: HashMap<(i64, i64), Vec<usize>> = HashMap::new();
    for (&edge_idx, path) in paths {
        for pt in &path.points {
            cell_edges
                .entry((pt.row, pt.col))
                .or_default()
                .push(edge_idx);
        }
    }

    // For each crossing cell, assign the hop to the higher-index edge.
    let mut result: HashMap<usize, Vec<(i64, i64)>> = HashMap::new();
    for ((row, col), edges) in &cell_edges {
        if edges.len() < 2 {
            continue;
        }

        let mut sorted = edges.clone();
        sorted.sort_unstable();
        sorted.dedup();
        if sorted.len() < 2 {
            // Same edge visited this cell twice (tight U-turn) — not a crossing.
            continue;
        }

        // sorted[0] = lower index (owner / passes straight)
        // sorted[1] = higher index (hopper / renders arc)
        let hopper_idx = sorted[1];
        result.entry(hopper_idx).or_default().push((*row, *col));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{CellState, Grid};

    fn make_path(points: &[(i64, i64)]) -> Vec<GridPoint> {
        points
            .iter()
            .map(|&(row, col)| GridPoint { row, col })
            .collect()
    }

    fn routed(points: &[(i64, i64)]) -> RoutedPath {
        RoutedPath {
            points: make_path(points),
            bend_count: 0,
            total_cost: 0.0,
        }
    }

    #[test]
    fn test_commit_path_marks_occupied() {
        let mut grid = Grid::new(10, 10, 10, 0, 0);
        let path = make_path(&[(5, 0), (5, 1), (5, 2)]);

        commit_path(&mut grid, &path, "edge_0");

        for p in &path {
            let cell = grid.get(p.row as usize, p.col as usize).unwrap();
            assert_eq!(cell.state, CellState::Occupied);
            assert_eq!(cell.owner.as_deref(), Some("edge_0"));
        }
    }

    #[test]
    fn test_commit_does_not_overwrite_blocked() {
        let mut grid = Grid::new(10, 10, 10, 0, 0);
        grid.get_mut(5, 5).unwrap().state = CellState::Blocked;

        commit_path(&mut grid, &make_path(&[(5, 5)]), "edge_0");

        assert_eq!(grid.get(5, 5).unwrap().state, CellState::Blocked);
    }

    #[test]
    fn test_uncommit_frees_sole_occupant() {
        let mut grid = Grid::new(10, 10, 10, 0, 0);
        let path = make_path(&[(5, 0), (5, 1)]);

        commit_path(&mut grid, &path, "edge_0");
        uncommit_path(&mut grid, &path, "edge_0", &HashMap::new());

        for p in &path {
            let cell = grid.get(p.row as usize, p.col as usize).unwrap();
            assert_eq!(cell.state, CellState::Free);
            assert!(cell.owner.is_none());
        }
    }

    #[test]
    fn test_uncommit_keeps_shared_cell_occupied() {
        let mut grid = Grid::new(10, 10, 10, 0, 0);

        // Simulate edge_0 owns (5,5) but edge_1 also passes through it.
        let cell = grid.get_mut(5, 5).unwrap();
        cell.state = CellState::Occupied;
        cell.owner = Some("edge_0".to_string());

        let path = make_path(&[(5, 5)]);
        // other_cell_owners says edge_1 also uses (5,5)
        let mut other = HashMap::new();
        other.insert((5_i64, 5_i64), "edge_1".to_string());

        uncommit_path(&mut grid, &path, "edge_0", &other);

        let cell = grid.get(5, 5).unwrap();
        // Cell must stay Occupied, transferred to edge_1
        assert_eq!(cell.state, CellState::Occupied);
        assert_eq!(cell.owner.as_deref(), Some("edge_1"));
    }

    #[test]
    fn test_compute_crossing_points_only_hopper_gets_arc() {
        // edge 0 horizontal, edge 1 vertical — they share (3,2).
        let mut paths = BTreeMap::new();
        paths.insert(0usize, routed(&[(3, 0), (3, 1), (3, 2)]));
        paths.insert(1usize, routed(&[(2, 2), (3, 2), (4, 2)]));

        let crossing_pts = compute_crossing_points(&paths);

        // edge 1 (higher index) is the hopper.
        assert!(
            !crossing_pts.contains_key(&0),
            "owner (edge_0) must not hop"
        );
        assert!(
            crossing_pts.contains_key(&1),
            "hopper (edge_1) must receive arc"
        );
        assert!(crossing_pts[&1].contains(&(3, 2)));
    }

    #[test]
    fn test_compute_crossing_points_no_self_crossing_on_revisit() {
        // A single path that revisits the same cell (tight U-turn).
        let mut paths = BTreeMap::new();
        paths.insert(0usize, routed(&[(3, 0), (3, 1), (4, 1), (3, 1), (3, 2)]));

        let crossing_pts = compute_crossing_points(&paths);
        assert!(
            crossing_pts.is_empty(),
            "single-edge revisit must not produce a crossing"
        );
    }

    #[test]
    fn test_build_other_cell_owners_excludes_given_indices() {
        let mut paths = BTreeMap::new();
        paths.insert(0usize, routed(&[(0, 0), (0, 1)]));
        paths.insert(1usize, routed(&[(0, 1), (0, 2)]));
        paths.insert(2usize, routed(&[(0, 2), (0, 3)]));

        let excluded: HashSet<usize> = [1usize].iter().cloned().collect();
        let owners = build_other_cell_owners(&paths, &excluded);

        // (0,0) owned by edge_0 ✓
        assert_eq!(owners.get(&(0, 0)).map(|s| s.as_str()), Some("edge_0"));
        // (0,1) shared by 0 and 1, but 1 is excluded → edge_0 is owner
        assert_eq!(owners.get(&(0, 1)).map(|s| s.as_str()), Some("edge_0"));
        // (0,2) shared by 1 (excluded) and 2 → edge_2 is owner
        assert_eq!(owners.get(&(0, 2)).map(|s| s.as_str()), Some("edge_2"));
        // (0,3) owned by edge_2 ✓
        assert_eq!(owners.get(&(0, 3)).map(|s| s.as_str()), Some("edge_2"));
    }

    #[test]
    fn test_restore_path_reclaims_transferred_owner() {
        // Reproduces the ghost-cell bug: edge_A owns (5,5), uncommit transfers
        // owner to edge_C, then a plain commit_path of edge_A's path leaves the
        // owner as edge_C. restore_path must force it back to edge_A.
        let mut grid = Grid::new(10, 10, 10, 0, 0);

        // edge_A commits (5,5) first.
        commit_path(&mut grid, &make_path(&[(5, 5)]), "edge_a");
        assert_eq!(grid.get(5, 5).unwrap().owner.as_deref(), Some("edge_a"));

        // Uncommit edge_A with edge_C listed as other owner → transfers to edge_C.
        let mut other = HashMap::new();
        other.insert((5_i64, 5_i64), "edge_c".to_string());
        uncommit_path(&mut grid, &make_path(&[(5, 5)]), "edge_a", &other);
        assert_eq!(grid.get(5, 5).unwrap().owner.as_deref(), Some("edge_c"));

        // Simulate reject: restore edge_A's original path.
        // commit_path would NOT reclaim the owner (sees Occupied, leaves as edge_C).
        // restore_path MUST reclaim it.
        restore_path(&mut grid, &make_path(&[(5, 5)]), "edge_a");

        let cell = grid.get(5, 5).unwrap();
        assert_eq!(cell.state, CellState::Occupied);
        assert_eq!(
            cell.owner.as_deref(),
            Some("edge_a"),
            "restore_path must reclaim ownership from transferred edge_c"
        );
    }

    #[test]
    fn test_restore_path_does_not_touch_blocked() {
        let mut grid = Grid::new(10, 10, 10, 0, 0);
        grid.get_mut(5, 5).unwrap().state = CellState::Blocked;

        restore_path(&mut grid, &make_path(&[(5, 5)]), "edge_0");

        assert_eq!(grid.get(5, 5).unwrap().state, CellState::Blocked);
        assert!(grid.get(5, 5).unwrap().owner.is_none());
    }
}
