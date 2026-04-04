use std::collections::HashMap;
use trellis_parser::Graph;

use super::common::{
    assign_connectors, build_edge_side_map, rebalance_sides, NodeEdgeInfo,
};
use super::assignment::{EdgePorts, Side};
use super::{PortAssigner, PortAssignmentContext};

/// Median port assignment algorithm.
///
/// Same as barycenter but uses the median position instead of the mean.
/// More robust when one far-off node skews the average.
pub struct MedianPortAssigner;

impl PortAssigner for MedianPortAssigner {
    fn assign_ports(&self, ctx: &PortAssignmentContext) -> HashMap<usize, EdgePorts> {
        assign_ports_median(ctx.graph, ctx.cell_size, ctx.offset_x, ctx.offset_y)
    }
}

fn assign_ports_median(
    graph: &Graph,
    cell_size: i32,
    offset_x: i32,
    offset_y: i32,
) -> HashMap<usize, EdgePorts> {
    let mut port_assignments: HashMap<usize, EdgePorts> = HashMap::new();

    let (node_map, mut per_node_sides) =
        build_edge_side_map(graph, cell_size, offset_x, offset_y);

    // Build adjacency list
    let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in &graph.edges {
        adjacency
            .entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
        adjacency
            .entry(edge.to.as_str())
            .or_default()
            .push(edge.from.as_str());
    }

    for node in &graph.nodes {
        let sides = match per_node_sides.get_mut(node.id.as_str()) {
            Some(s) => s,
            None => continue,
        };

        rebalance_sides(sides, node, cell_size, offset_x, offset_y, 0.8, 0.4);

        for (&side, edge_list) in sides.iter_mut() {
            sort_by_median(edge_list, side, &node_map, &adjacency);
        }

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

fn sort_by_median(
    edges: &mut [NodeEdgeInfo],
    side: Side,
    node_map: &HashMap<&str, &trellis_parser::Node>,
    adjacency: &HashMap<&str, Vec<&str>>,
) {
    edges.sort_by(|a, b| {
        let ma = compute_median(&a.other_node_id, side, node_map, adjacency);
        let mb = compute_median(&b.other_node_id, side, node_map, adjacency);
        ma.partial_cmp(&mb).unwrap()
    });
}

/// Compute the median position of a node and its 1-hop neighbours.
fn compute_median(
    node_id: &str,
    side: Side,
    node_map: &HashMap<&str, &trellis_parser::Node>,
    adjacency: &HashMap<&str, Vec<&str>>,
) -> f64 {
    let pos_fn = |id: &str| -> Option<f64> {
        node_map.get(id).map(|n| match side {
            Side::Top | Side::Bottom => n.x + n.width / 2.0,
            Side::Left | Side::Right => n.y + n.height / 2.0,
        })
    };

    let mut positions: Vec<f64> = Vec::new();

    if let Some(p) = pos_fn(node_id) {
        positions.push(p);
    }

    if let Some(neighbours) = adjacency.get(node_id) {
        for &neighbour in neighbours {
            if let Some(p) = pos_fn(neighbour) {
                positions.push(p);
            }
        }
    }

    if positions.is_empty() {
        return 0.0;
    }

    positions.sort_by(|a, b| a.partial_cmp(b).unwrap());
    positions[positions.len() / 2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::PortAssignmentContext;
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
    fn median_assigns_all_edges() {
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

        let assigner = MedianPortAssigner;
        let ctx = PortAssignmentContext {
            graph: &graph,
            cell_size: 10,
            offset_x: 0,
            offset_y: 0,
            print_metrics: false,
        };
        let ports = assigner.assign_ports(&ctx);
        assert_eq!(ports.len(), 3);

        for i in 0..3 {
            assert_eq!(ports[&i].source_port.side, Side::Bottom);
        }
    }

    #[test]
    fn median_robust_to_outlier() {
        // A -> B1, A -> B2
        // B1 has one neighbour far right (outlier), B2 has no extra neighbours
        // Median should be less affected by the outlier than barycenter
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("A", 80.0, 20.0, 60.0, 0.0),
            make_node("B1", 40.0, 20.0, 50.0, 100.0),
            make_node("B2", 40.0, 20.0, 90.0, 100.0),
            make_node("C", 40.0, 20.0, 500.0, 200.0), // far outlier
        ];
        graph.edges = vec![
            make_edge("A", "B1"),
            make_edge("A", "B2"),
            make_edge("B1", "C"),
        ];

        let assigner = MedianPortAssigner;
        let ctx = PortAssignmentContext {
            graph: &graph,
            cell_size: 10,
            offset_x: 0,
            offset_y: 0,
            print_metrics: false,
        };
        let ports = assigner.assign_ports(&ctx);
        assert_eq!(ports.len(), 3);
    }
}
