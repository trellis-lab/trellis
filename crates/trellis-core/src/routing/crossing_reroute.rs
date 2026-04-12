use std::collections::{HashMap, HashSet};

use trellis_parser::{Direction, Graph};

use crate::config::TrellisConfig;
use crate::grid::Grid;
use crate::ports::assignment::{EdgePorts, Port, Side};
use crate::ports::common::enumerate_connectors;

use super::astar::{route_edge, GridPoint, RoutedPath};
use super::commit::{commit_path, uncommit_path};

/// Maximum additional bends a reroute may introduce relative to the original.
const MAX_EXTRA_BENDS: usize = 2;

/// Maximum path-length growth factor a reroute may introduce.
const MAX_LENGTH_FACTOR: f64 = 1.3;

/// Attempt to reduce crossings by rerouting each edge that geometrically
/// crosses any other committed path.
///
/// For every edge with at least one crossing, the edge is uncommitted and all
/// 16 source/target side combinations are tried.  The combination that yields
/// the lowest total crossing count against *all other* committed paths is kept,
/// provided it strictly reduces crossings.  If no combination improves things,
/// the original path is recommitted unchanged.
///
/// To avoid O(16 · n) HashSet rebuilds per candidate, the occupied-cell sets
/// for all other paths are pre-built once per candidate before the 16 trials.
///
/// Returns the number of edges that were improved.
pub fn crossing_reroute(
    graph: &Graph,
    grid: &mut Grid,
    port_assignments: &mut HashMap<usize, EdgePorts>,
    paths: &mut HashMap<usize, RoutedPath>,
    config: &TrellisConfig,
) -> usize {
    // Collect edges that have at least one crossing with any other path.
    // Sort worst-first so edges with more crossings are processed first.
    let mut candidates: Vec<(usize, usize)> = paths
        .keys()
        .filter_map(|&idx| {
            let crossings = crossing_count_for_edge(idx, paths);
            if crossings > 0 {
                Some((idx, crossings))
            } else {
                None
            }
        })
        .collect();

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

        let edge_id = format!("edge_{}", edge_idx);
        uncommit_path(grid, &current_path.points, &edge_id, &config.routing_costs);

        // Pre-build occupied-cell sets for all other committed paths.
        // These are stable for the duration of the 16 trials below.
        let other_sets: Vec<HashSet<(i64, i64)>> = paths
            .iter()
            .filter(|(&idx, _)| idx != edge_idx)
            .map(|(_, p)| p.points.iter().map(|pt| (pt.row, pt.col)).collect())
            .collect();

        let crossings_before: usize = other_sets
            .iter()
            .map(|s| {
                current_path
                    .points
                    .iter()
                    .filter(|pt| s.contains(&(pt.row, pt.col)))
                    .count()
            })
            .sum();

        let current_len = current_path.points.len().saturating_sub(1);
        let bend_budget = current_path.bend_count.saturating_add(MAX_EXTRA_BENDS);
        let len_budget = (current_len as f64 * MAX_LENGTH_FACTOR).ceil() as usize;

        // Fix 3: restrict sides to those consistent with the graph flow direction.
        // Back-flow sides (e.g. Top for a source in TB) are excluded so that
        // crossing_reroute never routes an edge against the layout direction.
        let src_sides = allowed_source_sides(graph.direction);
        let tgt_sides = allowed_target_sides(graph.direction);

        let mut best_crossings = crossings_before;
        let mut best_ports = current_ports.clone();
        let mut best_path = current_path.clone();

        for &src_side in &src_sides {
            for &tgt_side in &tgt_sides {
                let candidate_ports = match ports_for_sides(
                    src_node,
                    tgt_node,
                    src_side,
                    tgt_side,
                    config.cell_size,
                    grid.offset_x,
                    grid.offset_y,
                    grid,
                ) {
                    Some(p) => p,
                    None => continue,
                };

                let source = GridPoint {
                    row: candidate_ports.source_port.grid_row,
                    col: candidate_ports.source_port.grid_col,
                };
                let target = GridPoint {
                    row: candidate_ports.target_port.grid_row,
                    col: candidate_ports.target_port.grid_col,
                };

                let src_state = save_and_free_cell(grid, source, &config.routing_costs);
                let tgt_state = save_and_free_cell(grid, target, &config.routing_costs);

                let try_path = route_edge(grid, source, target, &config.routing_costs);

                restore_cell(grid, source, src_state);
                restore_cell(grid, target, tgt_state);

                if let Some(path) = try_path {
                    let new_len = path.points.len().saturating_sub(1);

                    // Fix 1: reject paths that bloat bends or detour excessively.
                    if path.bend_count > bend_budget || new_len > len_budget {
                        continue;
                    }

                    let crossings: usize = other_sets
                        .iter()
                        .map(|s| {
                            path.points
                                .iter()
                                .filter(|pt| s.contains(&(pt.row, pt.col)))
                                .count()
                        })
                        .sum();

                    if crossings < best_crossings {
                        best_crossings = crossings;
                        best_ports = candidate_ports;
                        best_path = path;
                    }
                }
            }
        }

        commit_path(grid, &best_path.points, &edge_id, &config.routing_costs);

        if best_crossings < crossings_before {
            port_assignments.insert(edge_idx, best_ports);
            paths.insert(edge_idx, best_path);
            improved += 1;
        }
    }

    improved
}

/// Return the source-side candidates consistent with the graph flow direction.
///
/// The back-flow side (e.g. Top in TB) is excluded: allowing a forward edge to
/// exit from there almost always produces a long detour that routes against the
/// intended layout direction.
fn allowed_source_sides(direction: Direction) -> Vec<Side> {
    match direction {
        // TB flows downward — source exits Bottom, Left, or Right; not Top.
        Direction::TB => vec![Side::Bottom, Side::Left, Side::Right],
        // BT flows upward — source exits Top, Left, or Right; not Bottom.
        Direction::BT => vec![Side::Top, Side::Left, Side::Right],
        // LR flows rightward — source exits Right, Top, or Bottom; not Left.
        Direction::LR => vec![Side::Right, Side::Top, Side::Bottom],
        // RL flows leftward — source exits Left, Top, or Bottom; not Right.
        Direction::RL => vec![Side::Left, Side::Top, Side::Bottom],
    }
}

/// Return the target-side candidates consistent with the graph flow direction.
///
/// The back-flow side (e.g. Bottom in TB) is excluded for the same reason.
fn allowed_target_sides(direction: Direction) -> Vec<Side> {
    match direction {
        // TB flows downward — target enters Top, Left, or Right; not Bottom.
        Direction::TB => vec![Side::Top, Side::Left, Side::Right],
        // BT flows upward — target enters Bottom, Left, or Right; not Top.
        Direction::BT => vec![Side::Bottom, Side::Left, Side::Right],
        // LR flows rightward — target enters Left, Top, or Bottom; not Right.
        Direction::LR => vec![Side::Left, Side::Top, Side::Bottom],
        // RL flows leftward — target enters Right, Top, or Bottom; not Left.
        Direction::RL => vec![Side::Right, Side::Top, Side::Bottom],
    }
}

/// Count how many cells in edge `idx`'s path are shared with any other path.
fn crossing_count_for_edge(idx: usize, paths: &HashMap<usize, RoutedPath>) -> usize {
    let path = match paths.get(&idx) {
        Some(p) => p,
        None => return 0,
    };
    let self_set: HashSet<(i64, i64)> = path.points.iter().map(|p| (p.row, p.col)).collect();
    paths
        .iter()
        .filter(|(&other_idx, _)| other_idx != idx)
        .flat_map(|(_, other)| other.points.iter())
        .filter(|pt| self_set.contains(&(pt.row, pt.col)))
        .count()
}

/// Pick a connector on `side` of a node, preferring cells not already occupied.
fn pick_connector<'a>(
    connectors: &'a [crate::ports::common::Connector],
    grid: &Grid,
) -> &'a crate::ports::common::Connector {
    use crate::grid::CellState;

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
        &connectors[connectors.len() / 2]
    } else {
        &connectors[free_indices[free_indices.len() / 2]]
    }
}

/// Build an `EdgePorts` for a given source/target side combination.
/// Returns `None` if either node has no connectors on the requested side.
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

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TrellisConfig;
    use crate::grid::{build_grid, params::calculate_grid_extent};
    use crate::placement;
    use crate::ports::assign_ports;
    use crate::routing::route_all_edges;

    fn route_fixture(
        mermaid: &str,
    ) -> (
        HashMap<usize, RoutedPath>,
        Grid,
        HashMap<usize, EdgePorts>,
        Graph,
        TrellisConfig,
    ) {
        let mut graph = trellis_parser::parse(mermaid).expect("parse failed");
        let config = TrellisConfig::default();
        let cell_size = config.cell_size;
        placement::place_nodes(&mut graph, cell_size);
        let extent = calculate_grid_extent(&graph, cell_size);
        let port_assignments = assign_ports(&graph, cell_size, extent.offset_x, extent.offset_y);
        let mut grid = build_grid(&graph, cell_size, &extent);
        let result = route_all_edges(&graph, &mut grid, &port_assignments, &config);
        (result.paths, grid, port_assignments, graph, config)
    }

    #[test]
    fn crossing_reroute_does_not_increase_crossings_on_chain() {
        let mermaid = "graph TB\n    A --> B\n    B --> C\n    C --> D";
        let (mut paths, mut grid, mut ports, graph, config) = route_fixture(mermaid);
        let crossings_before: usize = paths
            .keys()
            .map(|&i| crossing_count_for_edge(i, &paths))
            .sum::<usize>()
            / 2; // each crossing counted from both sides
        crossing_reroute(&graph, &mut grid, &mut ports, &mut paths, &config);
        let crossings_after: usize = paths
            .keys()
            .map(|&i| crossing_count_for_edge(i, &paths))
            .sum::<usize>()
            / 2;
        assert!(
            crossings_after <= crossings_before,
            "crossings increased: {} -> {}",
            crossings_before,
            crossings_after
        );
    }

    #[test]
    fn crossing_reroute_preserves_all_edges() {
        let mermaid = "graph TB\n    A --> B\n    A --> C\n    B --> D\n    C --> D";
        let (mut paths, mut grid, mut ports, graph, config) = route_fixture(mermaid);
        let count_before = paths.len();
        crossing_reroute(&graph, &mut grid, &mut ports, &mut paths, &config);
        assert_eq!(paths.len(), count_before, "no edges should be dropped");
    }

    #[test]
    fn crossing_reroute_noop_on_single_edge() {
        let mermaid = "graph TB\n    A --> B";
        let (mut paths, mut grid, mut ports, graph, config) = route_fixture(mermaid);
        let improved = crossing_reroute(&graph, &mut grid, &mut ports, &mut paths, &config);
        assert_eq!(improved, 0, "single edge cannot cross anything");
    }
}
