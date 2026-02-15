use std::collections::HashMap;

use crate::config::TrellisConfig;
use crate::grid::{build_grid, calculate_grid_extent, Grid};
use crate::ports::{assign_ports, EdgePorts};
use crate::routing::astar::RoutedPath;
use crate::routing::RoutingResult;
use trellis_parser::Graph;

/// Grid expansion factor
const EXPANSION_FACTOR: f64 = 1.5;

/// Expand the grid by 1.5x and retry routing all edges from scratch.
///
/// This is the Level 2 deadlock resolution: if rip-up fails, we create a larger
/// grid with more routing space and re-route everything.
///
/// On success, the grid is replaced with the expanded version and all paths are updated.
/// Returns the path for the failed edge if it was successfully routed.
pub fn expand_grid_and_retry(
    graph: &Graph,
    grid: &mut Grid,
    failed_edge_idx: usize,
    _port_assignments: &HashMap<usize, EdgePorts>,
    config: &TrellisConfig,
    result: &mut RoutingResult,
) -> Option<RoutedPath> {
    // Clone the graph and scale node coordinates
    let mut scaled_graph = graph.clone();

    // Scale all node positions by EXPANSION_FACTOR relative to centroid
    // Node positions are top-left, so use center of each node for centroid
    let centroid_x = scaled_graph
        .nodes
        .iter()
        .map(|n| n.x + n.width / 2.0)
        .sum::<f64>()
        / scaled_graph.nodes.len().max(1) as f64;
    let centroid_y = scaled_graph
        .nodes
        .iter()
        .map(|n| n.y + n.height / 2.0)
        .sum::<f64>()
        / scaled_graph.nodes.len().max(1) as f64;

    for node in &mut scaled_graph.nodes {
        let node_cx = node.x + node.width / 2.0;
        let node_cy = node.y + node.height / 2.0;
        let new_cx = centroid_x + (node_cx - centroid_x) * EXPANSION_FACTOR;
        let new_cy = centroid_y + (node_cy - centroid_y) * EXPANSION_FACTOR;
        node.x = new_cx - node.width / 2.0;
        node.y = new_cy - node.height / 2.0;
    }

    // Rebuild grid and ports with expanded coordinates
    let cell_size = config.cell_size;
    let extent = calculate_grid_extent(&scaled_graph, cell_size);
    let mut new_grid = build_grid(&scaled_graph, cell_size, &extent);
    let new_port_assignments =
        assign_ports(&scaled_graph, cell_size, extent.offset_x, extent.offset_y);

    // Route all edges on the new grid (without deadlock handling to avoid infinite recursion)
    let new_result =
        crate::routing::route_all_edges_no_deadlock(&scaled_graph, &mut new_grid, &new_port_assignments, config);

    // Check if the failed edge was routed
    if let Some(failed_path) = new_result.paths.get(&failed_edge_idx) {
        let path = failed_path.clone();

        // Replace the grid and result with the expanded versions
        *grid = new_grid;
        *result = new_result;

        return Some(path);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expansion_factor() {
        assert!((EXPANSION_FACTOR - 1.5).abs() < 0.01);
    }
}
