pub mod sugiyama;
pub mod snap;

use trellis_parser::Graph;

/// Layer spacing in pixels (distance between layers)
pub const LAYER_SPACING: f64 = 100.0;

/// Node spacing in pixels (distance between nodes within a layer)
pub const NODE_SPACING: f64 = 80.0;

/// Place all nodes in the graph by assigning (x, y) coordinates.
/// Currently only supports flowcharts via Sugiyama layout.
pub fn place_nodes(graph: &mut Graph) {
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
