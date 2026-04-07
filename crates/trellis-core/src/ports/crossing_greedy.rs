use std::collections::HashMap;
use trellis_parser::Graph;

use super::assignment::{EdgePorts, Side};
use super::common::{
    assign_connectors, build_edge_side_map, count_inversions, rebalance_sides, would_cross,
    NodeEdgeInfo,
};
use super::prepass::apply_pinned;
use super::{effective_direction, PortAssigner, PortAssignmentContext};

/// Crossing-count greedy port assignment algorithm.
///
/// For each pair of edges on the same side, determines if their port order
/// creates an inversion relative to the target-side ordering. Finds the
/// permutation that minimises inversions.
pub struct CrossingGreedyPortAssigner;

impl PortAssigner for CrossingGreedyPortAssigner {
    fn assign_ports(&self, ctx: &PortAssignmentContext) -> HashMap<usize, EdgePorts> {
        let direction = effective_direction(ctx);
        let mut ports = assign_ports_crossing_greedy(
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

fn assign_ports_crossing_greedy(
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

    for node in &graph.nodes {
        let sides = match per_node_sides.get_mut(node.id.as_str()) {
            Some(s) => s,
            None => continue,
        };

        rebalance_sides(sides, node, cell_size, offset_x, offset_y, 0.8, 0.4);

        for (&side, edge_list) in sides.iter_mut() {
            minimize_crossings(edge_list, side, &node_map);
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

/// Find the permutation of edges that minimises crossing inversions.
///
/// For small k (< 8): try all k! permutations.
/// For larger k: use greedy adjacent-swap (bubble sort minimising crossings).
fn minimize_crossings(
    edges: &mut [NodeEdgeInfo],
    side: Side,
    node_map: &HashMap<&str, &trellis_parser::Node>,
) {
    let k = edges.len();
    if k <= 1 {
        return;
    }

    if k < 8 {
        // Exhaustive search for small k
        let mut best_inversions = count_inversions(edges, side, node_map);
        if best_inversions == 0 {
            return;
        }

        let mut best_perm: Vec<usize> = (0..k).collect();
        let mut current_perm: Vec<usize> = (0..k).collect();

        // Generate all permutations using Heap's algorithm
        let mut c = vec![0usize; k];
        let mut i = 1;
        while i < k {
            if c[i] < i {
                if i % 2 == 0 {
                    current_perm.swap(0, i);
                } else {
                    current_perm.swap(c[i], i);
                }

                // Evaluate this permutation
                let temp: Vec<NodeEdgeInfo> =
                    current_perm.iter().map(|&idx| edges[idx].clone()).collect();
                let inv = count_inversions(&temp, side, node_map);
                if inv < best_inversions {
                    best_inversions = inv;
                    best_perm = current_perm.clone();
                    if best_inversions == 0 {
                        break;
                    }
                }

                c[i] += 1;
                i = 1;
            } else {
                c[i] = 0;
                i += 1;
            }
        }

        // Apply best permutation
        let ordered: Vec<NodeEdgeInfo> =
            best_perm.iter().map(|&idx| edges[idx].clone()).collect();
        edges.clone_from_slice(&ordered);
    } else {
        // Greedy adjacent-swap for larger k
        greedy_adjacent_swap(edges, side, node_map);
    }
}

/// Bubble-sort style adjacent swap that minimises crossings.
///
/// For each adjacent pair, swap if it reduces inversions.
/// Repeat until no improvement.
fn greedy_adjacent_swap(
    edges: &mut [NodeEdgeInfo],
    side: Side,
    node_map: &HashMap<&str, &trellis_parser::Node>,
) {
    let k = edges.len();
    let mut improved = true;
    let mut max_passes = k * k; // safety bound

    while improved && max_passes > 0 {
        improved = false;
        max_passes -= 1;

        for i in 0..k - 1 {
            // Check if swapping i and i+1 reduces crossings
            if would_cross(&edges[i], &edges[i + 1], side, node_map) {
                edges.swap(i, i + 1);
                improved = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::PortAssignmentContext;
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
    fn crossing_greedy_assigns_all_edges() {
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

        let assigner = CrossingGreedyPortAssigner;
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

        for i in 0..3 {
            assert_eq!(ports[&i].source_port.side, Side::Bottom);
        }

        // Ports should be ordered matching target positions (no inversions)
        let x0 = ports[&0].source_port.x;
        let x1 = ports[&1].source_port.x;
        let x2 = ports[&2].source_port.x;
        assert!(x0 < x1, "B1 port should be left of B2");
        assert!(x1 < x2, "B2 port should be left of B3");
    }

    #[test]
    fn crossing_greedy_resolves_inversion() {
        // Set up a graph where default sorting would create an inversion:
        // A has edges to B1 (left, lower layer) and B2 (right, lower layer)
        // but B1's x > B2's x, so default sort puts B1 right of B2,
        // causing a crossing.
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("A", 80.0, 20.0, 60.0, 0.0),
            // B1 is to the right in x
            make_node("B1", 40.0, 20.0, 120.0, 100.0),
            // B2 is to the left in x
            make_node("B2", 40.0, 20.0, 20.0, 100.0),
        ];
        graph.edges = vec![make_edge("A", "B1"), make_edge("A", "B2")];

        let assigner = CrossingGreedyPortAssigner;
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

        // B2 (left target) should get the left port, B1 (right target) the right port
        let x_b1 = ports[&0].source_port.x; // edge to B1
        let x_b2 = ports[&1].source_port.x; // edge to B2
        assert!(
            x_b2 < x_b1,
            "B2 (left target) should have leftward port: b2={} b1={}",
            x_b2,
            x_b1
        );
    }
}
