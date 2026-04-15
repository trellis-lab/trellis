use std::collections::{BTreeMap, HashMap, HashSet};

use trellis_parser::Graph;

use crate::config::{BendThreshold, TrellisConfig};
use crate::grid::Grid;
use crate::ports::assignment::{EdgePorts, Port, Side};
use crate::ports::common::enumerate_connectors;

use super::astar::{route_edge, GridPoint, RoutedPath};
use super::commit::{build_other_cell_owners, commit_path, restore_path, uncommit_path};

/// Resolve the effective bend threshold from the config and the current set of
/// routed paths.
///
/// - `Disabled` → `None` (caller skips quality reroute entirely).
/// - `Fixed(n)` → `Some(n)`.
/// - `Auto` → `Some(max(2, median_bends + 2))`.
///
/// The median is computed from a snapshot of the bend counts at the time this
/// function is called, before any rerouting happens.
pub fn resolve_threshold(
    threshold: BendThreshold,
    paths: &BTreeMap<usize, RoutedPath>,
) -> Option<usize> {
    match threshold {
        BendThreshold::Disabled => None,
        BendThreshold::Fixed(n) => Some(n),
        BendThreshold::Auto => {
            if paths.is_empty() {
                return None;
            }
            let mut bend_counts: Vec<usize> = paths.values().map(|p| p.bend_count).collect();
            bend_counts.sort_unstable();
            let median = bend_counts[bend_counts.len() / 2];
            Some(median.saturating_add(2).max(2))
        }
    }
}

/// Attempt to improve edges whose bend count exceeds the resolved threshold by
/// trying alternative port-side combinations.
///
/// Returns the number of edges that were improved (re-routed with fewer bends).
///
/// This runs after the primary routing phase and before the deadlock resolver.
/// Only edges that have an existing committed path are candidates; unrouted
/// edges are left for the deadlock resolver.
pub fn quality_reroute(
    graph: &Graph,
    grid: &mut Grid,
    port_assignments: &mut HashMap<usize, EdgePorts>,
    paths: &mut BTreeMap<usize, RoutedPath>,
    config: &TrellisConfig,
) -> usize {
    let threshold = match resolve_threshold(config.bend_threshold, paths) {
        Some(t) => t,
        None => return 0,
    };

    // Snapshot bend counts before we start mutating paths.
    let mut candidates: Vec<(usize, usize)> = paths
        .iter()
        .filter_map(|(&edge_idx, path)| {
            if path.bend_count >= threshold {
                Some((edge_idx, path.bend_count))
            } else {
                None
            }
        })
        .collect();

    // Worst first.
    candidates.sort_by(|a, b| b.1.cmp(&a.1));

    let mut improved = 0;

    for (edge_idx, _) in candidates {
        let edge = match graph.edges.get(edge_idx) {
            Some(e) => e,
            None => continue,
        };
        let src_node = match graph.nodes.iter().find(|n| n.id == edge.from) {
            Some(n) => n,
            None => continue,
        };
        let tgt_node = match graph.nodes.iter().find(|n| n.id == edge.to) {
            Some(n) => n,
            None => continue,
        };

        let current_path = match paths.get(&edge_idx) {
            Some(p) => p.clone(),
            None => continue,
        };
        let current_ports = match port_assignments.get(&edge_idx) {
            Some(p) => p.clone(),
            None => continue,
        };
        let current_bends = current_path.bend_count;

        // Pre-build point sets for all other committed paths.  Used to count
        // crossings against any candidate path without rebuilding per trial.
        let other_sets: Vec<HashSet<(i64, i64)>> = paths
            .iter()
            .filter(|(&idx, _)| idx != edge_idx)
            .map(|(_, p)| p.points.iter().map(|pt| (pt.row, pt.col)).collect())
            .collect();

        let current_crossings: usize = other_sets
            .iter()
            .map(|s| {
                current_path
                    .points
                    .iter()
                    .filter(|pt| s.contains(&(pt.row, pt.col)))
                    .count()
            })
            .sum();

        let edge_id = format!("edge_{}", edge_idx);
        let excluded: std::collections::HashSet<usize> = [edge_idx].iter().cloned().collect();
        let other_owners = build_other_cell_owners(paths, &excluded);
        uncommit_path(grid, &current_path.points, &edge_id, &other_owners);

        let mut best_bends = current_bends;
        let mut best_crossings = current_crossings;
        let mut best_ports = current_ports.clone();
        let mut best_path = current_path.clone();

        // Exploration costs: suppress adjacent_cost so A* finds the geometrically
        // shortest path (fewest bends) without being deflected by congestion from
        // other committed edges.  The committed result still lives on the real grid.
        let explore_costs = crate::config::RoutingCosts {
            adjacent_cost: 0.0,
            ..config.routing_costs.clone()
        };

        let all_sides = [Side::Top, Side::Bottom, Side::Left, Side::Right];

        // Try all 16 side combinations.
        for &src_side in &all_sides {
            for &tgt_side in &all_sides {
                if src_side == current_ports.source_port.side
                    && tgt_side == current_ports.target_port.side
                {
                    continue;
                }

                let candidate = ports_for_sides(
                    src_node,
                    tgt_node,
                    src_side,
                    tgt_side,
                    config.cell_size,
                    grid.offset_x,
                    grid.offset_y,
                    grid,
                );
                let candidate = match candidate {
                    Some(c) => c,
                    None => continue,
                };

                let source = GridPoint {
                    row: candidate.source_port.grid_row,
                    col: candidate.source_port.grid_col,
                };
                let target = GridPoint {
                    row: candidate.target_port.grid_row,
                    col: candidate.target_port.grid_col,
                };

                let src_state = save_and_free_cell(grid, source, &explore_costs);
                let tgt_state = save_and_free_cell(grid, target, &explore_costs);

                let try_path = route_edge(grid, source, target, &explore_costs);

                restore_cell(grid, source, src_state);
                restore_cell(grid, target, tgt_state);

                if let Some(path) = try_path {
                    let crossings: usize = other_sets
                        .iter()
                        .map(|s| {
                            path.points
                                .iter()
                                .filter(|pt| s.contains(&(pt.row, pt.col)))
                                .count()
                        })
                        .sum();

                    // Accept only if bends strictly improve AND crossings do not
                    // worsen.  This enforces lexicographic priority: never trade
                    // a crossing for a bend reduction.
                    if path.bend_count < best_bends && crossings <= best_crossings {
                        best_bends = path.bend_count;
                        best_crossings = crossings;
                        best_ports = candidate;
                        best_path = path;
                    }
                }
            }
        }

        // Use restore_path: this edge was just uncommitted, so any cells shared
        // with a third edge may have had their owner transferred. restore_path
        // force-reclaims them so future uncommits of this edge work correctly.
        restore_path(grid, &best_path.points, &edge_id);

        if best_bends < current_bends {
            port_assignments.insert(edge_idx, best_ports);
            paths.insert(edge_idx, best_path);
            improved += 1;
        }
    }

    improved
}

/// Pick a connector on each side, preferring grid cells not already occupied by
/// another edge's path.  Falls back to the median connector when all cells on
/// the side are taken.
///
/// Returns `None` if the side has no connectors at all (node too narrow/short).
#[allow(clippy::too_many_arguments)]
fn ports_for_sides(
    src_node: &trellis_parser::Node,
    tgt_node: &trellis_parser::Node,
    src_side: Side,
    tgt_side: Side,
    cell_size: i32,
    offset_x: i32,
    offset_y: i32,
    grid: &Grid,
) -> Option<EdgePorts> {
    let src_connectors = enumerate_connectors(src_node, src_side, cell_size, offset_x, offset_y);
    let tgt_connectors = enumerate_connectors(tgt_node, tgt_side, cell_size, offset_x, offset_y);

    if src_connectors.is_empty() || tgt_connectors.is_empty() {
        return None;
    }

    let sc = pick_connector(&src_connectors, grid);
    let tc = pick_connector(&tgt_connectors, grid);

    Some(EdgePorts {
        source_port: Port {
            x: sc.x,
            y: sc.y,
            grid_row: sc.grid_row,
            grid_col: sc.grid_col,
            side: src_side,
        },
        target_port: Port {
            x: tc.x,
            y: tc.y,
            grid_row: tc.grid_row,
            grid_col: tc.grid_col,
            side: tgt_side,
        },
    })
}

/// From a list of connectors on one node side, return the best available one.
///
/// Preference order:
/// 1. Any connector whose grid cell is not `Occupied` (i.e. not already used
///    as a port by another committed edge).  Among free connectors, the median
///    index is chosen for geometric neutrality.
/// 2. If every connector is occupied, fall back to the median (no choice).
fn pick_connector<'a>(
    connectors: &'a [crate::ports::common::Connector],
    grid: &Grid,
) -> &'a crate::ports::common::Connector {
    use crate::grid::CellState;

    // Collect indices that are not Occupied.
    let free_indices: Vec<usize> = connectors
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            grid.get(c.grid_row as usize, c.grid_col as usize)
                .map(|cell| cell.state != CellState::Occupied)
                .unwrap_or(true)
        })
        .map(|(i, _)| i)
        .collect();

    if free_indices.is_empty() {
        // All occupied — use median as fallback.
        &connectors[connectors.len() / 2]
    } else {
        // Median of the free indices for geometric neutrality.
        &connectors[free_indices[free_indices.len() / 2]]
    }
}

/// Temporarily free a boundary cell so routing can start/end there.
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
        let original = cell.state;
        let needs_free = original == crate::grid::CellState::Blocked
            || original == crate::grid::CellState::Occupied;
        if needs_free {
            if let Some(cell) = grid.get_mut(row, col) {
                cell.state = crate::grid::CellState::Free;
                cell.cost = costs.base_cost;
            }
        }
        Some(original)
    } else {
        None
    }
}

/// Restore a cell's original state after routing.
fn restore_cell(grid: &mut Grid, point: GridPoint, original_state: Option<crate::grid::CellState>) {
    if let Some(state) = original_state {
        let was_blocked_or_occupied =
            state == crate::grid::CellState::Blocked || state == crate::grid::CellState::Occupied;
        if was_blocked_or_occupied && grid.in_bounds(point.row, point.col) {
            if let Some(cell) = grid.get_mut(point.row as usize, point.col as usize) {
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
    use crate::config::{BendThreshold, TrellisConfig};
    use crate::grid::{build_grid, params::calculate_grid_extent};
    use crate::placement;
    use crate::ports::assign_ports;
    use crate::routing::route_all_edges;

    fn make_config(threshold: BendThreshold) -> TrellisConfig {
        TrellisConfig {
            bend_threshold: threshold,
            ..TrellisConfig::default()
        }
    }

    // ── resolve_threshold ────────────────────────────────────────────────────

    #[test]
    fn resolve_disabled_returns_none() {
        let paths = BTreeMap::new();
        assert_eq!(resolve_threshold(BendThreshold::Disabled, &paths), None);
    }

    #[test]
    fn resolve_fixed_returns_value() {
        let paths = BTreeMap::new();
        assert_eq!(resolve_threshold(BendThreshold::Fixed(5), &paths), Some(5));
    }

    #[test]
    fn resolve_auto_empty_paths_returns_none() {
        let paths = BTreeMap::new();
        assert_eq!(resolve_threshold(BendThreshold::Auto, &paths), None);
    }

    #[test]
    fn resolve_auto_all_zero_bends_gives_floor() {
        // median=0 → max(2, 0+2) = 2
        let mut paths = BTreeMap::new();
        paths.insert(
            0,
            RoutedPath {
                points: vec![],
                total_cost: 0.0,
                bend_count: 0,
            },
        );
        paths.insert(
            1,
            RoutedPath {
                points: vec![],
                total_cost: 0.0,
                bend_count: 0,
            },
        );
        assert_eq!(resolve_threshold(BendThreshold::Auto, &paths), Some(2));
    }

    #[test]
    fn resolve_auto_median_two_gives_four() {
        // median=2 → max(2, 2+2) = 4
        let mut paths = BTreeMap::new();
        for i in 0..5usize {
            paths.insert(
                i,
                RoutedPath {
                    points: vec![],
                    total_cost: 0.0,
                    bend_count: 2,
                },
            );
        }
        assert_eq!(resolve_threshold(BendThreshold::Auto, &paths), Some(4));
    }

    #[test]
    fn resolve_auto_outlier_does_not_inflate_threshold() {
        // Even with one extreme outlier the median stays low.
        // bend_counts = [0, 0, 0, 0, 20] → sorted median = 0 → threshold = 2
        let mut paths = BTreeMap::new();
        for i in 0..4usize {
            paths.insert(
                i,
                RoutedPath {
                    points: vec![],
                    total_cost: 0.0,
                    bend_count: 0,
                },
            );
        }
        paths.insert(
            4,
            RoutedPath {
                points: vec![],
                total_cost: 0.0,
                bend_count: 20,
            },
        );
        assert_eq!(resolve_threshold(BendThreshold::Auto, &paths), Some(2));
        // The outlier (20 bends) is above threshold=2 → it will be a candidate.
    }

    #[test]
    fn resolve_auto_uniform_high_bends_raises_threshold() {
        // All edges have 6 bends → median=6 → threshold=8. Nothing gets flagged —
        // this is a global routing problem, not individual outliers.
        let mut paths = BTreeMap::new();
        for i in 0..4usize {
            paths.insert(
                i,
                RoutedPath {
                    points: vec![],
                    total_cost: 0.0,
                    bend_count: 6,
                },
            );
        }
        assert_eq!(resolve_threshold(BendThreshold::Auto, &paths), Some(8));
    }

    // ── quality_reroute integration ──────────────────────────────────────────

    fn route_fixture(
        mermaid: &str,
        threshold: BendThreshold,
    ) -> (
        BTreeMap<usize, RoutedPath>,
        Grid,
        HashMap<usize, EdgePorts>,
        trellis_parser::Graph,
    ) {
        let mut graph = trellis_parser::parse(mermaid).expect("parse failed");
        let config = make_config(threshold);
        let cell_size = config.cell_size;
        placement::place_nodes(&mut graph, cell_size);
        let extent = calculate_grid_extent(&graph, cell_size);
        let mut grid = build_grid(&graph, cell_size, &extent);
        let port_assignments = assign_ports(&graph, cell_size, extent.offset_x, extent.offset_y);
        let result = route_all_edges(&graph, &mut grid, &port_assignments, &config);
        (result.paths, grid, port_assignments, graph)
    }

    #[test]
    fn disabled_never_reroutes() {
        let mermaid = "graph TB\n    A --> B\n    B --> C";
        let (mut paths, mut grid, mut ports, graph) =
            route_fixture(mermaid, BendThreshold::Disabled);
        let config = make_config(BendThreshold::Disabled);
        let improved = quality_reroute(&graph, &mut grid, &mut ports, &mut paths, &config);
        assert_eq!(improved, 0);
    }

    #[test]
    fn fixed_threshold_below_actual_bends_is_noop() {
        // Simple chain: A→B has 0 bends. Fixed(2) means only edges >2 bends are
        // candidates — none here.
        let mermaid = "graph TB\n    A --> B";
        let (mut paths, mut grid, mut ports, graph) =
            route_fixture(mermaid, BendThreshold::Fixed(2));
        let config = make_config(BendThreshold::Fixed(2));
        let bends_before: usize = paths.values().map(|p| p.bend_count).sum();
        let improved = quality_reroute(&graph, &mut grid, &mut ports, &mut paths, &config);
        let bends_after: usize = paths.values().map(|p| p.bend_count).sum();
        assert_eq!(improved, 0);
        assert!(bends_after <= bends_before);
    }

    #[test]
    fn auto_does_not_increase_total_bends() {
        // On a well-routed chain the auto threshold should be above the actual
        // bend counts, so nothing is rerouted and bends cannot increase.
        let mermaid = "graph TB\n    A --> B\n    B --> C\n    C --> D";
        let (mut paths, mut grid, mut ports, graph) = route_fixture(mermaid, BendThreshold::Auto);
        let config = make_config(BendThreshold::Auto);
        let bends_before: usize = paths.values().map(|p| p.bend_count).sum();
        quality_reroute(&graph, &mut grid, &mut ports, &mut paths, &config);
        let bends_after: usize = paths.values().map(|p| p.bend_count).sum();
        assert!(
            bends_after <= bends_before,
            "bends increased after quality reroute"
        );
    }
}
