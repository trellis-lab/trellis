use std::collections::HashMap;
use trellis_parser::Graph;

use super::common::{
    assign_connectors, build_edge_side_map, sort_edges_on_side,
};
use super::{PortAssigner, PortAssignmentContext};

/// The default (angle-based) port assignment algorithm.
pub struct DefaultPortAssigner;

impl PortAssigner for DefaultPortAssigner {
    fn assign_ports(&self, ctx: &PortAssignmentContext) -> HashMap<usize, EdgePorts> {
        assign_ports(ctx.graph, ctx.cell_size, ctx.offset_x, ctx.offset_y)
    }
}

/// Side of a node where a port can be placed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

/// A port on a node's edge (connection point at an exact grid point)
#[derive(Debug, Clone)]
pub struct Port {
    pub x: f64,
    pub y: f64,
    pub grid_row: i64,
    pub grid_col: i64,
    pub side: Side,
}

/// Port assignments for an edge (source and target ports)
#[derive(Debug, Clone)]
pub struct EdgePorts {
    pub source_port: Port,
    pub target_port: Port,
}

/// Assign ports to all edges in the graph using discrete grid-point connectors.
///
/// For each node, edges are grouped by side (based on the angle to the connected node),
/// overflow is handled, edges are sorted within each side, and connector positions are assigned.
pub fn assign_ports(
    graph: &Graph,
    cell_size: i32,
    offset_x: i32,
    offset_y: i32,
) -> HashMap<usize, EdgePorts> {
    let mut port_assignments: HashMap<usize, EdgePorts> = HashMap::new();

    let (node_map, mut per_node_sides) =
        build_edge_side_map(graph, cell_size, offset_x, offset_y);

    for node in &graph.nodes {
        let sides = match per_node_sides.get_mut(node.id.as_str()) {
            Some(s) => s,
            None => continue,
        };

        // Sort edges on each side by perpendicular axis
        for (&side, edge_list) in sides.iter_mut() {
            sort_edges_on_side(edge_list, side, &node_map);
        }

        // Assign connector positions
        assign_connectors(
            node,
            sides,
            cell_size,
            offset_x,
            offset_y,
            &mut port_assignments,
        );
    }

    port_assignments
}

// Re-export angle_to_side for backward compatibility and tests
pub use super::common::angle_to_side;

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::common::enumerate_connectors;
    use trellis_parser::{ArrowHead, Edge, EdgeStyle, Node, NodeShape};

    fn make_node(id: &str, w: f64, h: f64, x: f64, y: f64) -> Node {
        Node {
            id: id.to_string(),
            label: id.to_string(),
            shape: NodeShape::Rectangle,
            width: w,
            height: h,
            x,
            y,
            ..Default::default()
        }
    }

    fn make_edge(from: &str, to: &str) -> Edge {
        Edge {
            from: from.to_string(),
            to: to.to_string(),
            label: None,
            style: EdgeStyle::Solid,
            arrow_head: ArrowHead::Arrow,
            ..Default::default()
        }
    }

    #[test]
    fn test_angle_to_side_right() {
        assert_eq!(angle_to_side(0.0), Side::Right);
        assert_eq!(angle_to_side(44.9), Side::Right);
        assert_eq!(angle_to_side(350.0), Side::Right);
    }

    #[test]
    fn test_angle_to_side_bottom() {
        assert_eq!(angle_to_side(45.0), Side::Bottom);
        assert_eq!(angle_to_side(90.0), Side::Bottom);
        assert_eq!(angle_to_side(134.9), Side::Bottom);
    }

    #[test]
    fn test_angle_to_side_left() {
        assert_eq!(angle_to_side(135.0), Side::Left);
        assert_eq!(angle_to_side(180.0), Side::Left);
        assert_eq!(angle_to_side(224.9), Side::Left);
    }

    #[test]
    fn test_angle_to_side_top() {
        assert_eq!(angle_to_side(225.0), Side::Top);
        assert_eq!(angle_to_side(270.0), Side::Top);
        assert_eq!(angle_to_side(314.9), Side::Top);
    }

    #[test]
    fn test_enumerate_connectors_top() {
        let node = make_node("A", 40.0, 20.0, 0.0, 0.0);
        let connectors = enumerate_connectors(&node, Side::Top, 10, 0, 0);
        assert_eq!(connectors.len(), 3);
        assert_eq!(connectors[0].grid_col, 1);
        assert_eq!(connectors[1].grid_col, 2);
        assert_eq!(connectors[2].grid_col, 3);
        assert_eq!(connectors[0].grid_row, 0);
    }

    #[test]
    fn test_enumerate_connectors_left() {
        let node = make_node("A", 40.0, 20.0, 0.0, 0.0);
        let connectors = enumerate_connectors(&node, Side::Left, 10, 0, 0);
        assert_eq!(connectors.len(), 1);
        assert_eq!(connectors[0].grid_row, 1);
        assert_eq!(connectors[0].grid_col, 0);
    }

    #[test]
    fn test_enumerate_connectors_with_offset() {
        let node = make_node("A", 40.0, 20.0, 50.0, 30.0);
        let connectors = enumerate_connectors(&node, Side::Bottom, 10, 0, 0);
        assert_eq!(connectors.len(), 3);
        assert_eq!(connectors[0].grid_row, 5);
        assert_eq!(connectors[0].grid_col, 6);
        assert_eq!(connectors[0].x, 60.0);
        assert_eq!(connectors[0].y, 50.0);
    }

    #[test]
    fn test_assign_ports_simple_vertical() {
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("A", 40.0, 20.0, 60.0, 30.0),
            make_node("B", 40.0, 20.0, 60.0, 130.0),
        ];
        graph.edges = vec![make_edge("A", "B")];

        let ports = assign_ports(&graph, 10, 0, 0);
        assert_eq!(ports.len(), 1);

        let edge_ports = &ports[&0];
        assert_eq!(edge_ports.source_port.side, Side::Bottom);
        assert_eq!(edge_ports.target_port.side, Side::Top);
        assert_eq!(edge_ports.source_port.x % 10.0, 0.0);
        assert_eq!(edge_ports.source_port.y % 10.0, 0.0);
    }

    #[test]
    fn test_assign_ports_horizontal() {
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("A", 40.0, 20.0, 10.0, 80.0),
            make_node("B", 40.0, 20.0, 210.0, 80.0),
        ];
        graph.edges = vec![make_edge("A", "B")];

        let ports = assign_ports(&graph, 10, 0, 0);
        let edge_ports = &ports[&0];
        assert_eq!(edge_ports.source_port.side, Side::Right);
        assert_eq!(edge_ports.target_port.side, Side::Left);
    }

    #[test]
    fn test_port_symmetry_for_opposite_edges() {
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("A", 40.0, 20.0, 60.0, 80.0),
            make_node("B", 40.0, 20.0, 60.0, 0.0),
            make_node("C", 40.0, 20.0, 60.0, 180.0),
        ];
        graph.edges = vec![make_edge("A", "B"), make_edge("A", "C")];

        let ports = assign_ports(&graph, 10, 0, 0);

        let ab = &ports[&0];
        assert_eq!(ab.source_port.side, Side::Top);

        let ac = &ports[&1];
        assert_eq!(ac.source_port.side, Side::Bottom);

        assert!((ab.source_port.x - ac.source_port.x).abs() < 0.01);
    }

    #[test]
    fn test_multiple_edges_same_side() {
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("A", 80.0, 20.0, 40.0, 30.0),
            make_node("B1", 40.0, 20.0, 20.0, 180.0),
            make_node("B2", 40.0, 20.0, 70.0, 180.0),
            make_node("B3", 40.0, 20.0, 120.0, 180.0),
        ];
        graph.edges = vec![
            make_edge("A", "B1"),
            make_edge("A", "B2"),
            make_edge("A", "B3"),
        ];

        let ports = assign_ports(&graph, 10, 0, 0);

        for i in 0..3 {
            assert_eq!(ports[&i].source_port.side, Side::Bottom);
        }

        let x0 = ports[&0].source_port.x;
        let x1 = ports[&1].source_port.x;
        let x2 = ports[&2].source_port.x;
        assert!(x0 < x1, "Port for B1 should be left of B2: {} < {}", x0, x1);
        assert!(x1 < x2, "Port for B2 should be left of B3: {} < {}", x1, x2);

        assert_eq!(x0 % 10.0, 0.0);
        assert_eq!(x1 % 10.0, 0.0);
        assert_eq!(x2 % 10.0, 0.0);
    }
}
