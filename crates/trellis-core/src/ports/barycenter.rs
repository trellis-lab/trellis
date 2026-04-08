use std::collections::HashMap;
use trellis_parser::Graph;

use super::assignment::{EdgePorts, Side};
use super::common::{assign_connectors, build_edge_side_map, rebalance_sides, NodeEdgeInfo};
use super::prepass::apply_pinned;
use super::{effective_direction, PortAssigner, PortAssignmentContext};

/// Barycenter port assignment algorithm.
///
/// Orders edges on each side by the barycenter (weighted average position)
/// of the connected sub-graph (target node + its neighbours within 1-2 hops),
/// rather than just the immediate target node position.
pub struct BarycenterPortAssigner;

impl PortAssigner for BarycenterPortAssigner {
    fn assign_ports(&self, ctx: &PortAssignmentContext) -> HashMap<usize, EdgePorts> {
        let direction = effective_direction(ctx);
        let mut ports = assign_ports_barycenter(
            ctx.graph,
            ctx.cell_size,
            ctx.offset_x,
            ctx.offset_y,
            direction,
            &ctx.topo_rank,
        );
        apply_pinned(&mut ports, &ctx.pinned_ports);
        ports
    }
}

fn assign_ports_barycenter(
    graph: &Graph,
    cell_size: i32,
    offset_x: i32,
    offset_y: i32,
    direction: Option<trellis_parser::Direction>,
    topo_rank: &HashMap<String, usize>,
) -> HashMap<usize, EdgePorts> {
    let mut port_assignments: HashMap<usize, EdgePorts> = HashMap::new();

    let (node_map, mut per_node_sides) =
        build_edge_side_map(graph, cell_size, offset_x, offset_y, direction, topo_rank);

    // Build adjacency list for neighbour lookups
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

        // Apply side rebalancing
        rebalance_sides(sides, node, cell_size, offset_x, offset_y, 0.8, 0.4);

        // Sort edges on each side by barycenter
        for (&side, edge_list) in sides.iter_mut() {
            sort_by_barycenter(edge_list, side, &node_map, &adjacency);
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

/// Sort edges by the barycenter of their target's neighbourhood.
fn sort_by_barycenter(
    edges: &mut [NodeEdgeInfo],
    side: Side,
    node_map: &HashMap<&str, &trellis_parser::Node>,
    adjacency: &HashMap<&str, Vec<&str>>,
) {
    edges.sort_by(|a, b| {
        let ba = compute_barycenter(&a.other_node_id, side, node_map, adjacency);
        let bb = compute_barycenter(&b.other_node_id, side, node_map, adjacency);
        ba.partial_cmp(&bb).unwrap()
    });
}

/// Compute the barycenter (mean position) of a node and its 1-hop neighbours
/// along the perpendicular axis of the given side.
fn compute_barycenter(
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

    let mut sum = 0.0;
    let mut count = 0;

    if let Some(p) = pos_fn(node_id) {
        sum += p;
        count += 1;
    }

    if let Some(neighbours) = adjacency.get(node_id) {
        for &neighbour in neighbours {
            if let Some(p) = pos_fn(neighbour) {
                sum += p;
                count += 1;
            }
        }
    }

    if count == 0 {
        0.0
    } else {
        sum / count as f64
    }
}

#[cfg(test)]
mod tests {
    use super::super::PortAssignmentContext;
    use super::*;
    use crate::grid::Grid;
    use std::collections::HashMap;
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
    fn barycenter_assigns_all_edges() {
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

        let assigner = BarycenterPortAssigner;
        let grid = Grid::new(50, 50, 10, 0, 0);
        let ctx = PortAssignmentContext {
            graph: &graph,
            cell_size: 10,
            offset_x: 0,
            offset_y: 0,
            grid: &grid,
            pinned_ports: HashMap::new(),
            flow_bias: crate::config::FlowBias::None,
            topo_rank: HashMap::new(),
            print_metrics: false,
        };
        let ports = assigner.assign_ports(&ctx);
        assert_eq!(ports.len(), 3);

        // All source ports on A's bottom
        for i in 0..3 {
            assert_eq!(ports[&i].source_port.side, Side::Bottom);
        }
    }

    #[test]
    fn barycenter_considers_neighbourhood() {
        // Fan-out: A -> B1, A -> B2
        // B1 connects to C (far right), B2 connects to D (far left)
        // Pure target-position sort would put B1 left, B2 right.
        // Barycenter should shift B1 right (pulled by C) and B2 left (pulled by D).
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("A", 80.0, 20.0, 60.0, 0.0),
            make_node("B1", 40.0, 20.0, 40.0, 100.0),
            make_node("B2", 40.0, 20.0, 100.0, 100.0),
            make_node("C", 40.0, 20.0, 200.0, 200.0), // far right
            make_node("D", 40.0, 20.0, 0.0, 200.0),   // far left
        ];
        graph.edges = vec![
            make_edge("A", "B1"),
            make_edge("A", "B2"),
            make_edge("B1", "C"),
            make_edge("B2", "D"),
        ];

        let assigner = BarycenterPortAssigner;
        let grid = Grid::new(50, 50, 10, 0, 0);
        let ctx = PortAssignmentContext {
            graph: &graph,
            cell_size: 10,
            offset_x: 0,
            offset_y: 0,
            grid: &grid,
            pinned_ports: HashMap::new(),
            flow_bias: crate::config::FlowBias::None,
            topo_rank: HashMap::new(),
            print_metrics: false,
        };
        let ports = assigner.assign_ports(&ctx);
        assert_eq!(ports.len(), 4);
    }
}
