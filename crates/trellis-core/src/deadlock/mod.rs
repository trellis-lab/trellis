pub mod expand;
pub mod fallback;
pub mod rip_up;

use std::collections::HashMap;

use crate::config::TrellisConfig;
use crate::grid::Grid;
use crate::ports::EdgePorts;
use crate::routing::astar::RoutedPath;
use crate::routing::RoutingResult;
use trellis_parser::Graph;

/// Maximum number of rip-up iterations before moving to the next level
const MAX_RIP_UP_ITERATIONS: usize = 5;

/// Handle a deadlock situation where an edge could not be routed.
///
/// Implements a 3-level defense:
/// 1. Rip-up and reroute: find blocking edges, remove them, reroute
/// 2. Grid expansion: enlarge grid by 1.5x and reroute everything
/// 3. Crossing fallback: allow routing through occupied cells
///
/// Returns the path for the failed edge if any level succeeds.
pub fn handle_deadlock(
    graph: &Graph,
    grid: &mut Grid,
    failed_edge_idx: usize,
    port_assignments: &HashMap<usize, EdgePorts>,
    config: &TrellisConfig,
    result: &mut RoutingResult,
) -> Option<RoutedPath> {
    // Level 1: Rip-up and reroute
    if let Some(path) = rip_up::rip_up_and_reroute(
        grid,
        failed_edge_idx,
        port_assignments,
        config,
        result,
        MAX_RIP_UP_ITERATIONS,
    ) {
        return Some(path);
    }

    // Level 2: Grid expansion
    if let Some(path) = expand::expand_grid_and_retry(
        graph,
        grid,
        failed_edge_idx,
        port_assignments,
        config,
        result,
    ) {
        return Some(path);
    }

    // Level 3: Crossing fallback
    if let Some(path) = fallback::route_with_crossings_allowed(
        grid,
        failed_edge_idx,
        port_assignments,
        config,
    ) {
        return Some(path);
    }

    None
}
