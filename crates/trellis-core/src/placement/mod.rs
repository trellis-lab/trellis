pub mod class;
pub mod snap;
pub mod subgraph;
pub mod sugiyama;

use std::collections::HashMap;
use trellis_parser::{Graph, NodeShape};

use crate::types::{BoundingBox, SubgraphTree};

/// Layer spacing in pixels (distance between layers)
pub const LAYER_SPACING: f64 = 100.0;

/// Node spacing in pixels (distance between nodes within a layer)
pub const NODE_SPACING: f64 = 80.0;

/// Place all nodes in the graph by assigning (x, y) coordinates.
///
/// If the graph contains subgraphs, uses recursive bottom-up placement
/// and returns the subgraph tree and bounding boxes for rendering.
///
/// `cell_size` is used to snap node dimensions to grid-point multiples.
pub fn place_nodes(
    graph: &mut Graph,
    cell_size: i32,
) -> Option<(SubgraphTree, HashMap<String, BoundingBox>)> {
    snap_node_dimensions_to_grid(graph, cell_size);

    if !graph.subgraphs.is_empty() {
        // Subgraph-aware placement (M9)
        let result = subgraph::place_with_subgraphs(graph, cell_size);
        snap_node_positions_to_grid(graph, cell_size);
        Some(result)
    } else {
        match graph.diagram_type {
            trellis_parser::DiagramType::Flowchart => {
                sugiyama::layout(graph);
            }
            trellis_parser::DiagramType::ClassDiagram => {
                class::place_class_diagram(graph);
            }
            _ => {
                sugiyama::layout(graph);
            }
        }
        snap_node_positions_to_grid(graph, cell_size);
        None
    }
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
        let mut w_points = ((node.width / cs).ceil() as i32 + 1).max(5); // min 5 grid points
        let mut h_points = ((node.height / cs).ceil() as i32 + 1).max(3); // min 3 grid points

        // Make it always odd number
        if w_points % 2 == 0 {
            w_points += 1;
        }

        if h_points % 2 == 0 {
            h_points += 1;
        }

        // Correct circles and double-circles: force equal width and height
        if node.shape == NodeShape::Circle || node.shape == NodeShape::DoubleCircle {
            let max = w_points.max(h_points);
            w_points = max;
            h_points = max;
        }

        // Cylinders need extra height so the top ellipse cap does not overlap the label.
        // +2 grid points = 2 * cell_size extra pixels (stays odd since 2 is even).
        if node.shape == NodeShape::Cylinder {
            h_points += 2;
        }

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
            ..Default::default()        }
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
        // height=25 needs ceil(25/10)+1 = 3+1+1 = 5 grid points = 40 pixels
        assert_eq!(graph.nodes[0].height, 40.0);
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

    #[test]
    fn test_snap_dimensions_cylinder_extra_height() {
        let mut graph = Graph::new();
        // Same pixel size as the rectangle in test_snap_dimensions_exact_multiple,
        // but shape=Cylinder → h_points gets +2 extra grid points = +2*10 = +20 px.
        let mut node = make_node("A", 40.0, 20.0);
        node.shape = NodeShape::Cylinder;
        graph.nodes = vec![node];
        snap_node_dimensions_to_grid(&mut graph, 10);
        // width unchanged
        assert_eq!(graph.nodes[0].width, 40.0);
        // height: base h_points = 3 (from test above) + 2 = 5 → (5-1)*10 = 40 px
        assert_eq!(graph.nodes[0].height, 40.0);
    }
}
