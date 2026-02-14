pub mod sugiyama;
pub mod snap;

use std::collections::HashMap;
use trellis_parser::{Direction, Graph};
use crate::ports::MIN_PORT_SPACING;

/// Layer spacing in pixels (distance between layers)
pub const LAYER_SPACING: f64 = 100.0;

/// Node spacing in pixels (distance between nodes within a layer)
pub const NODE_SPACING: f64 = 80.0;

/// Place all nodes in the graph by assigning (x, y) coordinates.
/// Currently only supports flowcharts via Sugiyama layout.
pub fn place_nodes(graph: &mut Graph) {
    expand_nodes_for_ports(graph);

    match graph.diagram_type {
        trellis_parser::DiagramType::Flowchart => {
            sugiyama::layout(graph);
        }
        // Other diagram types will be implemented in M10
        _ => {
            sugiyama::layout(graph);
        }
    }
}

/// Expand node dimensions so that high-degree nodes have enough space for port routing.
///
/// For each node, counts its edge degree and ensures the node is wide/tall enough
/// to fit all ports with at least `MIN_PORT_SPACING` between them.
fn expand_nodes_for_ports(graph: &mut Graph) {
    // Count degree per node
    let mut degree: HashMap<&str, usize> = HashMap::new();
    for edge in &graph.edges {
        *degree.entry(edge.from.as_str()).or_default() += 1;
        *degree.entry(edge.to.as_str()).or_default() += 1;
    }

    let is_vertical = matches!(graph.direction, Direction::TB | Direction::BT);

    for node in &mut graph.nodes {
        let deg = degree.get(node.id.as_str()).copied().unwrap_or(0);
        if deg <= 1 {
            continue;
        }

        // Primary axis: the side that fans out (bottom for TB, right for LR)
        // needs to hold up to `deg` ports. Secondary axes hold overflow.
        // Conservative estimate: primary dimension needs all ports,
        // secondary needs roughly a third (overflow from primary).
        let primary_required = (deg as f64 + 1.0) * MIN_PORT_SPACING;
        let secondary_required = ((deg as f64 / 3.0).ceil() + 1.0) * MIN_PORT_SPACING;

        if is_vertical {
            // Width is the primary dimension (bottom/top side holds most ports)
            node.width = node.width.max(primary_required);
            node.height = node.height.max(secondary_required);
        } else {
            // Height is the primary dimension (right/left side holds most ports)
            node.height = node.height.max(primary_required);
            node.width = node.width.max(secondary_required);
        }
    }
}
