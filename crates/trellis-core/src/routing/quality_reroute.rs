use std::collections::HashMap;

use trellis_parser::Graph;

use crate::config::TrellisConfig;
use crate::grid::Grid;
use crate::ports::assignment::{EdgePorts, Port, Side};
use crate::ports::common::enumerate_connectors;

use super::astar::{route_edge, GridPoint, RoutedPath};
use super::commit::{commit_path, uncommit_path};

/// Attempt to improve edges whose bend count exceeds `config.max_acceptable_bends`
/// by trying alternative port-side combinations.
///
/// Returns the number of edges that were improved (re-routed with fewer bends).
///
/// This runs after the primary routing phase and before the deadlock resolver.
/// Only edges that have an existing committed path are candidates; unrouted edges
/// are left for the deadlock resolver.
pub fn quality_reroute(
    graph: &Graph,
    grid: &mut Grid,
    port_assignments: &mut HashMap<usize, EdgePorts>,
    paths: &mut HashMap<usize, RoutedPath>,
    config: &TrellisConfig,
) -> usize {
    if config.max_acceptable_bends == 0 {
        return 0;
    }

    // Collect candidates: routed edges with bend count above threshold
    let mut candidates: Vec<(usize, usize)> = paths
        .iter()
        .filter_map(|(&edge_idx, path)| {
            if path.bend_count > config.max_acceptable_bends {
                Some((edge_idx, path.bend_count))
            } else {
                None
            }
        })
        .collect();

    // Worst first
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

        // Current state
        let current_path = match paths.get(&edge_idx) {
            Some(p) => p.clone(),
            None => continue,
        };
        let current_ports = match port_assignments.get(&edge_idx) {
            Some(p) => p.clone(),
            None => continue,
        };
        let current_bends = current_path.bend_count;

        // Rip up the current path
        let edge_id = format!("edge_{}", edge_idx);
        uncommit_path(grid, &current_path.points, &edge_id, &config.routing_costs);

        let mut best_bends = current_bends;
        let mut best_ports = current_ports.clone();
        let mut best_path = current_path.clone();

        let all_sides = [Side::Top, Side::Bottom, Side::Left, Side::Right];

        // Try all 16 side combinations
        for &src_side in &all_sides {
            for &tgt_side in &all_sides {
                // Skip the current assignment
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

                // Temporarily free source/target boundary cells for routing
                let src_state = save_and_free_cell(grid, source, &config.routing_costs);
                let tgt_state = save_and_free_cell(grid, target, &config.routing_costs);

                let try_path = route_edge(grid, source, target, &config.routing_costs);

                restore_cell(grid, source, src_state);
                restore_cell(grid, target, tgt_state);

                if let Some(path) = try_path {
                    if path.bend_count < best_bends {
                        best_bends = path.bend_count;
                        best_ports = candidate;
                        best_path = path;
                    }
                }
            }
        }

        // Commit the best result
        commit_path(grid, &best_path.points, &edge_id, &config.routing_costs);

        if best_bends < current_bends {
            port_assignments.insert(edge_idx, best_ports);
            paths.insert(edge_idx, best_path);
            improved += 1;
        } else {
            // Restore original (already committed above with best_path == current_path)
        }
    }

    improved
}

/// Pick the median connector on `src_side` of `src_node` and `tgt_side` of `tgt_node`.
///
/// Returns `None` if either side has no connectors (e.g. node is too narrow).
fn ports_for_sides(
    src_node: &trellis_parser::Node,
    tgt_node: &trellis_parser::Node,
    src_side: Side,
    tgt_side: Side,
    cell_size: i32,
    offset_x: i32,
    offset_y: i32,
    _grid: &Grid,
) -> Option<EdgePorts> {
    let src_connectors = enumerate_connectors(src_node, src_side, cell_size, offset_x, offset_y);
    let tgt_connectors = enumerate_connectors(tgt_node, tgt_side, cell_size, offset_x, offset_y);

    if src_connectors.is_empty() || tgt_connectors.is_empty() {
        return None;
    }

    let sc = &src_connectors[src_connectors.len() / 2];
    let tc = &tgt_connectors[tgt_connectors.len() / 2];

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

/// Temporarily free a boundary cell so routing can start/end there.
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
        if original == crate::grid::CellState::Blocked {
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
fn restore_cell(
    grid: &mut Grid,
    point: GridPoint,
    original_state: Option<crate::grid::CellState>,
) {
    if let Some(state) = original_state {
        if state == crate::grid::CellState::Blocked && grid.in_bounds(point.row, point.col) {
            if let Some(cell) = grid.get_mut(point.row as usize, point.col as usize) {
                if cell.state != crate::grid::CellState::Occupied {
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
    use crate::config::TrellisConfig;
    use crate::grid::{build_grid, params::calculate_grid_extent};
    use crate::placement;
    use crate::ports::assign_ports;
    use crate::routing::route_all_edges;

    fn make_config_with_threshold(max_bends: usize) -> TrellisConfig {
        TrellisConfig {
            max_acceptable_bends: max_bends,
            ..TrellisConfig::default()
        }
    }

    #[test]
    fn quality_reroute_disabled_when_zero() {
        let mermaid = "graph TB\n    A --> B\n    B --> C";
        let mut graph = trellis_parser::parse(mermaid).expect("parse failed");
        let config = make_config_with_threshold(0);
        let cell_size = config.cell_size;
        placement::place_nodes(&mut graph, cell_size);
        let extent = calculate_grid_extent(&graph, cell_size);
        let mut grid = build_grid(&graph, cell_size, &extent);
        let mut port_assignments = assign_ports(&graph, cell_size, extent.offset_x, extent.offset_y);
        let mut routing_result = route_all_edges(&graph, &mut grid, &port_assignments, &config);
        let improved = quality_reroute(
            &graph,
            &mut grid,
            &mut port_assignments,
            &mut routing_result.paths,
            &config,
        );
        assert_eq!(improved, 0, "should be 0 when max_acceptable_bends=0");
    }

    #[test]
    fn quality_reroute_leaves_low_bend_edges() {
        // A → B: direct vertical chain → 0 bends → below any reasonable threshold
        let mermaid = "graph TB\n    A --> B";
        let mut graph = trellis_parser::parse(mermaid).expect("parse failed");
        let config = make_config_with_threshold(2);
        let cell_size = config.cell_size;
        placement::place_nodes(&mut graph, cell_size);
        let extent = calculate_grid_extent(&graph, cell_size);
        let mut grid = build_grid(&graph, cell_size, &extent);
        let mut port_assignments = assign_ports(&graph, cell_size, extent.offset_x, extent.offset_y);
        let mut routing_result = route_all_edges(&graph, &mut grid, &port_assignments, &config);
        let total_bends_before: usize = routing_result.paths.values().map(|p| p.bend_count).sum();
        let improved = quality_reroute(
            &graph,
            &mut grid,
            &mut port_assignments,
            &mut routing_result.paths,
            &config,
        );
        let total_bends_after: usize = routing_result.paths.values().map(|p| p.bend_count).sum();
        // Edges below threshold are untouched; total bends shouldn't increase
        assert_eq!(improved, 0);
        assert!(total_bends_after <= total_bends_before);
    }
}
