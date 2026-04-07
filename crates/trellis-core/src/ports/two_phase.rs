use std::collections::HashMap;
use trellis_parser::Graph;

use super::assignment::{EdgePorts, Side};
use super::common::{
    assign_connectors, build_edge_side_map, count_inversions, rebalance_sides, NodeEdgeInfo,
};
use super::prepass::apply_pinned;
use super::{PortAssigner, PortAssignmentContext};

/// Two-phase port assigner.
///
/// **Phase 1** (inside `assign_ports`): uses CrossingGreedy per side to minimise
/// inversions, then applies a global inversion estimate to catch inter-node
/// ordering issues. This runs without any routing — purely geometric.
///
/// **Phase 2** (in the pipeline refinement loop): after routing, crossing edge
/// pairs are identified and their ports are swapped using the shared
/// `iterative::refine_ports()` function. This is handled externally by the
/// pipeline, not inside this assigner.
pub struct TwoPhaseAssigner;

impl PortAssigner for TwoPhaseAssigner {
    fn assign_ports(&self, ctx: &PortAssignmentContext) -> HashMap<usize, EdgePorts> {
        let mut ports =
            assign_ports_two_phase(ctx.graph, ctx.cell_size, ctx.offset_x, ctx.offset_y);
        apply_pinned(&mut ports, &ctx.pinned_ports);
        ports
    }
}

/// Phase 1: crossing-greedy ordering with global inversion awareness.
///
/// 1. Build edge-side map (same as all algorithms)
/// 2. Rebalance sides
/// 3. For each side, run the crossing-greedy minimisation (from Algorithm 3)
/// 4. Apply a second pass: for each node, check if swapping any adjacent pair
///    of edges on the same side would reduce the global inversion estimate
fn assign_ports_two_phase(
    graph: &Graph,
    cell_size: i32,
    offset_x: i32,
    offset_y: i32,
) -> HashMap<usize, EdgePorts> {
    let mut port_assignments: HashMap<usize, EdgePorts> = HashMap::new();

    let (node_map, mut per_node_sides) =
        build_edge_side_map(graph, cell_size, offset_x, offset_y);

    // Phase 1a: rebalance + crossing-greedy per side
    for node in &graph.nodes {
        let sides = match per_node_sides.get_mut(node.id.as_str()) {
            Some(s) => s,
            None => continue,
        };

        rebalance_sides(sides, node, cell_size, offset_x, offset_y, 0.8, 0.4);

        for (&side, edge_list) in sides.iter_mut() {
            minimize_crossings_greedy(edge_list, side, &node_map);
        }
    }

    // Phase 1b: global refinement pass — try swapping adjacent pairs on each side
    // to reduce total inversions across the entire graph
    let mut improved = true;
    let mut max_passes = 3; // limit passes to avoid excessive computation
    while improved && max_passes > 0 {
        improved = false;
        max_passes -= 1;

        let current_total = total_inversions(&per_node_sides, &node_map);
        if current_total == 0 {
            break;
        }

        for node in &graph.nodes {
            let sides = match per_node_sides.get_mut(node.id.as_str()) {
                Some(s) => s,
                None => continue,
            };

            let side_keys: Vec<Side> = sides.keys().copied().collect();
            for side in side_keys {
                let edges = match sides.get_mut(&side) {
                    Some(e) if e.len() >= 2 => e,
                    _ => continue,
                };

                // Try each adjacent swap
                for i in 0..edges.len() - 1 {
                    let before = count_inversions(edges, side, &node_map);
                    edges.swap(i, i + 1);
                    let after = count_inversions(edges, side, &node_map);

                    if after < before {
                        improved = true;
                    } else {
                        // Swap back — no improvement
                        edges.swap(i, i + 1);
                    }
                }
            }
        }
    }

    // Assign connectors from the optimised ordering
    for node in &graph.nodes {
        let sides = match per_node_sides.get(node.id.as_str()) {
            Some(s) => s,
            None => continue,
        };

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

/// Compute total inversions across all nodes and sides.
fn total_inversions(
    per_node_sides: &HashMap<&str, HashMap<Side, Vec<NodeEdgeInfo>>>,
    node_map: &HashMap<&str, &trellis_parser::Node>,
) -> usize {
    let mut total = 0;
    for sides in per_node_sides.values() {
        for (&side, edges) in sides {
            total += count_inversions(edges, side, node_map);
        }
    }
    total
}

/// Crossing-greedy minimisation for a single side.
///
/// Reuses the same logic as CrossingGreedyPortAssigner but operates on
/// a mutable slice directly.
fn minimize_crossings_greedy(
    edges: &mut [NodeEdgeInfo],
    side: Side,
    node_map: &HashMap<&str, &trellis_parser::Node>,
) {
    use super::common::would_cross;

    let k = edges.len();
    if k <= 1 {
        return;
    }

    // For small k, exhaustive search (same as crossing_greedy.rs)
    if k < 8 {
        let mut best_inversions = count_inversions(edges, side, node_map);
        if best_inversions == 0 {
            return;
        }

        let mut best_perm: Vec<usize> = (0..k).collect();
        let mut current_perm: Vec<usize> = (0..k).collect();

        let mut c = vec![0usize; k];
        let mut i = 1;
        while i < k {
            if c[i] < i {
                if i % 2 == 0 {
                    current_perm.swap(0, i);
                } else {
                    current_perm.swap(c[i], i);
                }

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

        let ordered: Vec<NodeEdgeInfo> =
            best_perm.iter().map(|&idx| edges[idx].clone()).collect();
        edges.clone_from_slice(&ordered);
    } else {
        // Greedy adjacent-swap for larger k
        let mut changed = true;
        let mut max_passes = k * k;
        while changed && max_passes > 0 {
            changed = false;
            max_passes -= 1;
            for i in 0..k - 1 {
                if would_cross(&edges[i], &edges[i + 1], side, node_map) {
                    edges.swap(i, i + 1);
                    changed = true;
                }
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
    fn two_phase_assigns_all_edges() {
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

        let assigner = TwoPhaseAssigner;
        let grid = Grid::new(50, 50, 10, 0, 0);
        let ctx = PortAssignmentContext {
            graph: &graph,
            cell_size: 10,
            offset_x: 0,
            offset_y: 0,
            grid: &grid,
            pinned_ports: HashMap::new(),
            print_metrics: false,
        };
        let ports = assigner.assign_ports(&ctx);
        assert_eq!(ports.len(), 3);

        for i in 0..3 {
            assert_eq!(ports[&i].source_port.side, Side::Bottom);
        }
    }

    #[test]
    fn two_phase_resolves_inversions() {
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("A", 80.0, 20.0, 60.0, 0.0),
            make_node("B1", 40.0, 20.0, 120.0, 100.0),
            make_node("B2", 40.0, 20.0, 20.0, 100.0),
        ];
        graph.edges = vec![make_edge("A", "B1"), make_edge("A", "B2")];

        let assigner = TwoPhaseAssigner;
        let grid = Grid::new(50, 50, 10, 0, 0);
        let ctx = PortAssignmentContext {
            graph: &graph,
            cell_size: 10,
            offset_x: 0,
            offset_y: 0,
            grid: &grid,
            pinned_ports: HashMap::new(),
            print_metrics: false,
        };
        let ports = assigner.assign_ports(&ctx);

        // B2 (left target) should get the left port, B1 (right target) the right port
        let x_b1 = ports[&0].source_port.x;
        let x_b2 = ports[&1].source_port.x;
        assert!(
            x_b2 < x_b1,
            "B2 (left target) should have leftward port: b2={} b1={}",
            x_b2,
            x_b1
        );
    }

    #[test]
    fn two_phase_handles_diamond_chain() {
        // Diamond: Start -> A, Start -> B, A -> End, B -> End
        // Plus chain: A -> B
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("Start", 80.0, 20.0, 60.0, 0.0),
            make_node("A", 40.0, 20.0, 30.0, 100.0),
            make_node("B", 40.0, 20.0, 110.0, 100.0),
            make_node("End", 80.0, 20.0, 60.0, 200.0),
        ];
        graph.edges = vec![
            make_edge("Start", "A"),
            make_edge("Start", "B"),
            make_edge("A", "B"),
            make_edge("A", "End"),
            make_edge("B", "End"),
        ];

        let assigner = TwoPhaseAssigner;
        let grid = Grid::new(50, 50, 10, 0, 0);
        let ctx = PortAssignmentContext {
            graph: &graph,
            cell_size: 10,
            offset_x: 0,
            offset_y: 0,
            grid: &grid,
            pinned_ports: HashMap::new(),
            print_metrics: false,
        };
        let ports = assigner.assign_ports(&ctx);
        assert_eq!(ports.len(), 5);
    }
}
