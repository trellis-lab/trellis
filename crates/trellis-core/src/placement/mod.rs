pub mod sugiyama;
pub mod snap;

use trellis_parser::Graph;

/// Layer spacing in pixels (distance between layers)
pub const LAYER_SPACING: f64 = 100.0;

/// Node spacing in pixels (distance between nodes within a layer)
pub const NODE_SPACING: f64 = 80.0;

/// Place all nodes in the graph by assigning (x, y) coordinates.
/// Currently only supports flowcharts via Sugiyama layout.
///
/// `cell_size` is used to snap node dimensions to grid-point multiples.
pub fn place_nodes(graph: &mut Graph, cell_size: i32) {
    snap_node_dimensions_to_grid(graph, cell_size);

    match graph.diagram_type {
        trellis_parser::DiagramType::Flowchart => {
            sugiyama::layout(graph);
        }
        // Other diagram types will be implemented in M10
        _ => {
            sugiyama::layout(graph);
        }
    }

    // Snap top-left positions to grid points
    snap_node_positions_to_grid(graph, cell_size);
}

/// Snap node top-left positions to the nearest grid points.
fn snap_node_positions_to_grid(graph: &mut Graph, cell_size: i32) {
    let cs = cell_size as f64;
    for node in &mut graph.nodes {
        node.x = (node.x / cs).round() * cs;
        node.y = (node.y / cs).round() * cs;
    }
}

/// Snap node dimensions to grid-aligned sizes.
///
/// "Width = N * cell_size" means N grid points along the edge,
/// i.e., pixel width = (N-1) * cell_size.
/// Minimum: 5 grid points wide (4*cs pixels), 3 grid points tall (2*cs pixels).
fn snap_node_dimensions_to_grid(graph: &mut Graph, cell_size: i32) {
    let cs = cell_size as f64;
    for node in &mut graph.nodes {
        // Calculate how many grid points needed to cover the text-based dimension.
        // N grid points span (N-1) cell intervals = (N-1)*cs pixels.
        // So N = ceil(pixels / cs) + 1.
        let w_points = ((node.width / cs).ceil() as i32 + 1).max(5); // min 5 grid points
        let h_points = ((node.height / cs).ceil() as i32 + 1).max(3); // min 3 grid points
        node.width = (w_points - 1) as f64 * cs;
        node.height = (h_points - 1) as f64 * cs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_parser::{Node, NodeShape};

    fn make_node(id: &str, w: f64, h: f64) -> Node {
        Node {
            id: id.to_string(),
            label: id.to_string(),
            shape: NodeShape::Rectangle,
            width: w,
            height: h,
            x: 0.0,
            y: 0.0,
        }
    }

    #[test]
    fn test_snap_dimensions_minimum() {
        let mut graph = Graph::new();
        graph.nodes = vec![make_node("A", 10.0, 10.0)];
        snap_node_dimensions_to_grid(&mut graph, 10);
        // min 5 grid points wide = 4*10 = 40 pixels
        assert_eq!(graph.nodes[0].width, 40.0);
        // min 3 grid points tall = 2*10 = 20 pixels
        assert_eq!(graph.nodes[0].height, 20.0);
    }

    #[test]
    fn test_snap_dimensions_rounds_up() {
        let mut graph = Graph::new();
        // width=55 needs ceil(55/10)+1 = 6+1 = 7 grid points = 60 pixels
        graph.nodes = vec![make_node("A", 55.0, 25.0)];
        snap_node_dimensions_to_grid(&mut graph, 10);
        assert_eq!(graph.nodes[0].width, 60.0);
        // height=25 needs ceil(25/10)+1 = 3+1 = 4 grid points = 30 pixels
        assert_eq!(graph.nodes[0].height, 30.0);
    }

    #[test]
    fn test_snap_dimensions_exact_multiple() {
        let mut graph = Graph::new();
        // width=40 = exactly 4 cell intervals, ceil(40/10)+1 = 5 grid points = 40 pixels
        graph.nodes = vec![make_node("A", 40.0, 20.0)];
        snap_node_dimensions_to_grid(&mut graph, 10);
        assert_eq!(graph.nodes[0].width, 40.0);
        assert_eq!(graph.nodes[0].height, 20.0);
    }
}
