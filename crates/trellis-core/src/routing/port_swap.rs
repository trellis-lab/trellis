use std::collections::{HashMap, HashSet};

use trellis_parser::Graph;

use crate::config::TrellisConfig;
use crate::grid::Grid;
use crate::ports::assignment::{EdgePorts, Port, Side};

use super::astar::{route_edge, GridPoint, RoutedPath};
use super::commit::{commit_path, uncommit_path};

/// Attempt to reduce crossings by swapping the ports of pairs of edges that
/// share a node side and whose paths are geometrically inverted (i.e. they
/// cross each other near that node).
///
/// For each node, each side is examined.  Adjacent port pairs whose connected
/// edges cross — determined by comparing the perpendicular position of each
/// edge's *other* endpoint — are candidates for a swap.  Both edges are
/// uncommitted, their ports exchanged on this node, both re-routed, and the
/// swap is kept only if the combined bend count does not increase.
///
/// `pinned_indices` is the set of edge indices whose ports were locked by the
/// straight-edge pre-pass.  These edges are never swapped; their 0-bend
/// alignment must be preserved.
///
/// Returns the number of swaps that were accepted.
pub fn port_swap(
    graph: &Graph,
    grid: &mut Grid,
    port_assignments: &mut HashMap<usize, EdgePorts>,
    paths: &mut HashMap<usize, RoutedPath>,
    config: &TrellisConfig,
    pinned_indices: &HashSet<usize>,
) -> usize {
    let mut accepted = 0;

    for node in &graph.nodes {
        // Collect all edges that touch this node, grouped by (side, is_source).
        // We track the side as seen from this node and whether this node is the
        // source or target of each edge.
        let mut by_side: HashMap<Side, Vec<usize>> = HashMap::new();

        for (edge_idx, edge) in graph.edges.iter().enumerate() {
            let touches = if edge.from == node.id {
                paths
                    .contains_key(&edge_idx)
                    .then(|| port_assignments.get(&edge_idx).map(|p| p.source_port.side))
            } else if edge.to == node.id {
                paths
                    .contains_key(&edge_idx)
                    .then(|| port_assignments.get(&edge_idx).map(|p| p.target_port.side))
            } else {
                None
            };

            if let Some(Some(side)) = touches {
                by_side.entry(side).or_default().push(edge_idx);
            }
        }

        // Process each side independently.
        for (side, mut edge_indices) in by_side {
            if edge_indices.len() < 2 {
                continue;
            }

            // Sort edges by their port's perpendicular coordinate on this node
            // so that "adjacent" pairs are actually neighbours in port order.
            sort_by_port_position(&mut edge_indices, side, port_assignments);

            // Scan adjacent pairs for inversions.
            let mut i = 0;
            while i + 1 < edge_indices.len() {
                let idx_a = edge_indices[i];
                let idx_b = edge_indices[i + 1];

                // Never swap pinned edges.
                if pinned_indices.contains(&idx_a) || pinned_indices.contains(&idx_b) {
                    i += 1;
                    continue;
                }

                // Only attempt a swap if the two paths actually cross each other.
                let path_a = match paths.get(&idx_a) {
                    Some(p) => p,
                    None => {
                        i += 1;
                        continue;
                    }
                };
                let path_b = match paths.get(&idx_b) {
                    Some(p) => p,
                    None => {
                        i += 1;
                        continue;
                    }
                };
                if path_crossings(&path_a.points, &path_b.points) == 0 {
                    i += 1;
                    continue;
                }

                // Attempt the swap.
                if try_swap(
                    graph,
                    grid,
                    side,
                    idx_a,
                    idx_b,
                    port_assignments,
                    paths,
                    config,
                ) {
                    accepted += 1;
                }
                // Always advance: each pair is visited at most once per pass.
                i += 1;
            }
        }
    }

    accepted
}

// ─── Core swap logic ──────────────────────────────────────────────────────────

/// Uncommit both edges, swap their ports on `node`/`side`, re-route both, and
/// commit.  Keeps the swap only if combined bends do not increase.
///
/// Returns `true` if the swap was accepted.
#[allow(clippy::too_many_arguments)]
fn try_swap(
    _graph: &Graph,
    grid: &mut Grid,
    side: Side,
    idx_a: usize,
    idx_b: usize,
    port_assignments: &mut HashMap<usize, EdgePorts>,
    paths: &mut HashMap<usize, RoutedPath>,
    config: &TrellisConfig,
) -> bool {
    let path_a = match paths.get(&idx_a) {
        Some(p) => p.clone(),
        None => return false,
    };
    let path_b = match paths.get(&idx_b) {
        Some(p) => p.clone(),
        None => return false,
    };
    let ports_a = match port_assignments.get(&idx_a) {
        Some(p) => p.clone(),
        None => return false,
    };
    let ports_b = match port_assignments.get(&idx_b) {
        Some(p) => p.clone(),
        None => return false,
    };

    let crossings_before = path_crossings(&path_a.points, &path_b.points);
    let bends_before = path_a.bend_count + path_b.bend_count;

    let id_a = format!("edge_{}", idx_a);
    let id_b = format!("edge_{}", idx_b);

    uncommit_path(grid, &path_a.points, &id_a, &config.routing_costs);
    uncommit_path(grid, &path_b.points, &id_b, &config.routing_costs);

    // Build swapped port assignments: exchange the port on `side`
    // between the two edges while keeping the other end unchanged.
    let swapped_a = swap_port_on_node(&ports_a, &ports_b, side);
    let swapped_b = swap_port_on_node(&ports_b, &ports_a, side);

    // Re-route both edges with the swapped ports.
    let new_path_a = route_with_ports(grid, &swapped_a, config);
    let new_path_b = route_with_ports(grid, &swapped_b, config);

    match (new_path_a, new_path_b) {
        (Some(new_a), Some(new_b)) => {
            let crossings_after = path_crossings(&new_a.points, &new_b.points);
            let bends_after = new_a.bend_count + new_b.bend_count;

            // Accept if crossings decrease, or crossings are equal and bends decrease.
            let accept = crossings_after < crossings_before
                || (crossings_after == crossings_before && bends_after < bends_before);

            if accept {
                commit_path(grid, &new_a.points, &id_a, &config.routing_costs);
                commit_path(grid, &new_b.points, &id_b, &config.routing_costs);
                port_assignments.insert(idx_a, swapped_a);
                port_assignments.insert(idx_b, swapped_b);
                paths.insert(idx_a, new_a);
                paths.insert(idx_b, new_b);
                true
            } else {
                // Restore originals.
                commit_path(grid, &path_a.points, &id_a, &config.routing_costs);
                commit_path(grid, &path_b.points, &id_b, &config.routing_costs);
                false
            }
        }
        _ => {
            // One or both edges failed to re-route — restore originals.
            commit_path(grid, &path_a.points, &id_a, &config.routing_costs);
            commit_path(grid, &path_b.points, &id_b, &config.routing_costs);
            false
        }
    }
}

/// Count the number of grid cells shared between two paths (i.e. how many
/// times they cross each other).  Uses a HashSet for O(n+m) performance.
fn path_crossings(a: &[GridPoint], b: &[GridPoint]) -> usize {
    let a_set: std::collections::HashSet<(i64, i64)> = a.iter().map(|p| (p.row, p.col)).collect();
    b.iter().filter(|p| a_set.contains(&(p.row, p.col))).count()
}

// ─── Port helpers ─────────────────────────────────────────────────────────────

/// Build a new `EdgePorts` for `edge` by replacing its port on `node`/`side`
/// with the corresponding port from `donor`.
///
/// If `edge` uses `node` as its source and the source port is on `side`,
/// the source port is replaced with `donor`'s source port on `side`.
/// The same logic applies to the target port.  The other end is unchanged.
fn swap_port_on_node(edge: &EdgePorts, donor: &EdgePorts, side: Side) -> EdgePorts {
    // We know the port on `node`/`side` is either the source or the target,
    // depending on which end of the edge `node` is.  Since we only have the
    // `EdgePorts` struct (not the graph edge), we identify the relevant port
    // by side: the port whose `.side` matches `side` is the one to swap.
    let src_matches = edge.source_port.side == side && donor.source_port.side == side;
    let tgt_matches = edge.target_port.side == side && donor.target_port.side == side;

    // Determine which connector index on `side` of `node` to use for the
    // replacement port.  We want to keep the same physical connector position
    // (grid cell) while swapping which edge uses it.
    //
    // Strategy: pick the connector on `side` of `node` that is closest to the
    // donor's port position on that side.  This preserves the "other" edge's
    // original connector and gives this edge the newly vacated slot.
    let donor_connector_pos = if src_matches {
        (donor.source_port.grid_row, donor.source_port.grid_col)
    } else if tgt_matches {
        (donor.target_port.grid_row, donor.target_port.grid_col)
    } else {
        // Nothing to swap.
        return edge.clone();
    };

    // Build the replacement port at the donor's connector position.
    let replacement = Port {
        x: donor_connector_pos.1 as f64, // placeholder; overwritten below
        y: donor_connector_pos.0 as f64,
        grid_row: donor_connector_pos.0,
        grid_col: donor_connector_pos.1,
        side,
    };

    // Fix up x/y from the connector list so they match the grid coordinate
    // system used everywhere else.
    //
    // We do this by finding the matching connector in `enumerate_connectors`
    // (which uses cell_size and offset from the node geometry).  Since we
    // don't have cell_size/offset here, we instead trust that the donor's
    // x/y are already correct and copy them directly.
    let replacement = if src_matches {
        Port {
            x: donor.source_port.x,
            y: donor.source_port.y,
            ..replacement
        }
    } else {
        Port {
            x: donor.target_port.x,
            y: donor.target_port.y,
            ..replacement
        }
    };

    if src_matches {
        EdgePorts {
            source_port: replacement,
            target_port: edge.target_port.clone(),
        }
    } else {
        EdgePorts {
            source_port: edge.source_port.clone(),
            target_port: replacement,
        }
    }
}

/// Route an edge using the given ports, temporarily freeing blocked boundary
/// cells, and return the resulting path (or `None` on failure).
fn route_with_ports(
    grid: &mut Grid,
    ports: &EdgePorts,
    config: &TrellisConfig,
) -> Option<RoutedPath> {
    let source = GridPoint {
        row: ports.source_port.grid_row,
        col: ports.source_port.grid_col,
    };
    let target = GridPoint {
        row: ports.target_port.grid_row,
        col: ports.target_port.grid_col,
    };

    let src_state = save_and_free_cell(grid, source, &config.routing_costs);
    let tgt_state = save_and_free_cell(grid, target, &config.routing_costs);

    let path = route_edge(grid, source, target, &config.routing_costs);

    restore_cell(grid, source, src_state);
    restore_cell(grid, target, tgt_state);

    path
}

/// Sort a list of edge indices by the perpendicular position of their port on
/// `node`/`side` (left-to-right for Top/Bottom, top-to-bottom for Left/Right).
fn sort_by_port_position(
    edge_indices: &mut [usize],
    side: Side,
    port_assignments: &HashMap<usize, EdgePorts>,
) {
    // Sort directly by grid coordinate on the perpendicular axis.
    edge_indices.sort_by(|&a, &b| {
        let pa = port_assignments.get(&a);
        let pb = port_assignments.get(&b);
        let coord_a = port_coord_on_side(pa, side);
        let coord_b = port_coord_on_side(pb, side);
        coord_a
            .partial_cmp(&coord_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Return the perpendicular coordinate (column for Top/Bottom, row for
/// Left/Right) of the port on `side` for this edge.
fn port_coord_on_side(ports: Option<&EdgePorts>, side: Side) -> f64 {
    let ports = match ports {
        Some(p) => p,
        None => return 0.0,
    };

    // Pick whichever port is on the matching side.
    let port = if ports.source_port.side == side {
        &ports.source_port
    } else if ports.target_port.side == side {
        &ports.target_port
    } else {
        return 0.0;
    };

    match side {
        Side::Top | Side::Bottom => port.grid_col as f64,
        Side::Left | Side::Right => port.grid_row as f64,
    }
}

// ─── Cell state helpers (mirrors quality_reroute) ─────────────────────────────

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
    use crate::ports::{assign_ports, straight_edge_prepass};
    use crate::routing::route_all_edges;

    fn route_fixture(
        mermaid: &str,
    ) -> (
        HashMap<usize, RoutedPath>,
        Grid,
        HashMap<usize, EdgePorts>,
        trellis_parser::Graph,
        HashSet<usize>,
        TrellisConfig,
    ) {
        let mut graph = trellis_parser::parse(mermaid).expect("parse failed");
        let config = TrellisConfig::default();
        let cell_size = config.cell_size;
        placement::place_nodes(&mut graph, cell_size);
        let extent = calculate_grid_extent(&graph, cell_size);
        let grid = build_grid(&graph, cell_size, &extent);
        let pinned =
            straight_edge_prepass(&graph, &grid, cell_size, extent.offset_x, extent.offset_y);
        let pinned_indices: HashSet<usize> = pinned.keys().cloned().collect();
        let port_assignments = assign_ports(&graph, cell_size, extent.offset_x, extent.offset_y);
        let mut grid2 = build_grid(&graph, cell_size, &extent);
        let result = route_all_edges(&graph, &mut grid2, &port_assignments, &config);
        (
            result.paths,
            grid2,
            port_assignments,
            graph,
            pinned_indices,
            config,
        )
    }

    #[test]
    fn swap_does_not_increase_bends_on_simple_chain() {
        let mermaid = "graph TB\n    A --> B\n    B --> C\n    C --> D";
        let (mut paths, mut grid, mut ports, graph, pinned, config) = route_fixture(mermaid);
        let bends_before: usize = paths.values().map(|p| p.bend_count).sum();
        port_swap(&graph, &mut grid, &mut ports, &mut paths, &config, &pinned);
        let bends_after: usize = paths.values().map(|p| p.bend_count).sum();
        assert!(
            bends_after <= bends_before,
            "bends increased after port_swap: {} -> {}",
            bends_before,
            bends_after
        );
    }

    #[test]
    fn swap_preserves_all_edges_routed() {
        let mermaid = "graph TB\n    A --> B\n    A --> C\n    B --> D\n    C --> D";
        let (mut paths, mut grid, mut ports, graph, pinned, config) = route_fixture(mermaid);
        let count_before = paths.len();
        port_swap(&graph, &mut grid, &mut ports, &mut paths, &config, &pinned);
        assert_eq!(
            paths.len(),
            count_before,
            "port_swap must not drop any routed edges"
        );
    }

    #[test]
    fn swap_does_not_touch_pinned_edges() {
        // A simple vertically-aligned pair: the straight-edge prepass should pin
        // the A→B edge.  port_swap must leave it untouched.
        let mermaid = "graph TB\n    A --> B\n    A --> C";
        let (mut paths, mut grid, mut ports, graph, pinned, config) = route_fixture(mermaid);

        // Record the port assignment for any pinned edges before the swap.
        let pinned_before: HashMap<usize, EdgePorts> = pinned
            .iter()
            .filter_map(|&idx| ports.get(&idx).map(|p| (idx, p.clone())))
            .collect();

        port_swap(&graph, &mut grid, &mut ports, &mut paths, &config, &pinned);

        // Pinned edges must have identical port assignments after the swap.
        for (idx, before) in &pinned_before {
            let after = ports.get(idx).expect("pinned edge disappeared");
            assert_eq!(
                (before.source_port.grid_row, before.source_port.grid_col),
                (after.source_port.grid_row, after.source_port.grid_col),
                "pinned edge {} source port moved",
                idx
            );
            assert_eq!(
                (before.target_port.grid_row, before.target_port.grid_col),
                (after.target_port.grid_row, after.target_port.grid_col),
                "pinned edge {} target port moved",
                idx
            );
        }
    }

    #[test]
    fn swap_noop_on_single_edge() {
        let mermaid = "graph TB\n    A --> B";
        let (mut paths, mut grid, mut ports, graph, pinned, config) = route_fixture(mermaid);
        let swaps = port_swap(&graph, &mut grid, &mut ports, &mut paths, &config, &pinned);
        assert_eq!(swaps, 0, "single edge: no pairs to swap");
    }

    #[test]
    fn swap_bipartite_does_not_increase_bends() {
        // K2,3 bipartite: multiple fan-out edges sharing node sides.
        let mermaid = "graph TB\n    A1 --> B1\n    A1 --> B2\n    A1 --> B3\n    A2 --> B1\n    A2 --> B2\n    A2 --> B3";
        let (mut paths, mut grid, mut ports, graph, pinned, config) = route_fixture(mermaid);
        let bends_before: usize = paths.values().map(|p| p.bend_count).sum();
        port_swap(&graph, &mut grid, &mut ports, &mut paths, &config, &pinned);
        let bends_after: usize = paths.values().map(|p| p.bend_count).sum();
        assert!(
            bends_after <= bends_before,
            "bends increased on bipartite graph: {} -> {}",
            bends_before,
            bends_after
        );
    }

    #[test]
    fn swap_b22_hub_fan_out_fixes_crossings() {
        // b22: Hub fans out to A-F, all fan into Out. Hub-->E and Hub-->F
        // cross each other (and similarly A/B-->Out). The swap pass must
        // reduce the crossing count to zero on this well-structured graph.
        let mermaid = "graph TB\n    Hub --> A\n    Hub --> B\n    Hub --> C\n    Hub --> D\n    Hub --> E\n    Hub --> F\n    A --> Out\n    B --> Out\n    C --> Out\n    D --> Out\n    E --> Out\n    F --> Out";
        let (mut paths, mut grid, mut ports, graph, pinned, config) = route_fixture(mermaid);

        let crossings_before: usize = graph
            .edges
            .iter()
            .enumerate()
            .flat_map(|(i, _)| graph.edges.iter().enumerate().map(move |(j, _)| (i, j)))
            .filter(|&(i, j)| i < j)
            .filter_map(|(i, j)| {
                let pa = paths.get(&i)?;
                let pb = paths.get(&j)?;
                Some(path_crossings(&pa.points, &pb.points))
            })
            .sum();

        port_swap(&graph, &mut grid, &mut ports, &mut paths, &config, &pinned);

        let crossings_after: usize = graph
            .edges
            .iter()
            .enumerate()
            .flat_map(|(i, _)| graph.edges.iter().enumerate().map(move |(j, _)| (i, j)))
            .filter(|&(i, j)| i < j)
            .filter_map(|(i, j)| {
                let pa = paths.get(&i)?;
                let pb = paths.get(&j)?;
                Some(path_crossings(&pa.points, &pb.points))
            })
            .sum();

        assert!(
            crossings_after < crossings_before,
            "crossings should decrease on b22 fan-out: {} -> {}",
            crossings_before,
            crossings_after
        );
    }
}
