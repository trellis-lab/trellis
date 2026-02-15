use std::collections::HashMap;
use trellis_parser::{Direction, Graph};

use super::{LAYER_SPACING, NODE_SPACING};

/// Maximum iterations for barycenter ordering
const MAX_BARYCENTER_ITERATIONS: usize = 20;

/// Run the full Sugiyama layout algorithm on a flowchart graph.
///
/// Steps:
/// 1. Break cycles (reverse back-edges)
/// 2. Assign layers (longest path)
/// 3. Order within layers (barycenter heuristic)
/// 4. Assign coordinates (direction-dependent, centered)
/// 5. Restore reversed edges
pub fn layout(graph: &mut Graph) {
    if graph.nodes.is_empty() {
        return;
    }

    // Build adjacency from edge list
    let node_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
    let node_index: HashMap<String, usize> = node_ids
        .iter()
        .enumerate()
        .map(|(i, id)| (id.clone(), i))
        .collect();

    // Step 1: Break cycles
    let reversed = break_cycles(&node_ids, &node_index, &mut graph.edges);

    // Step 2: Assign layers
    let layers = assign_layers(&node_ids, &node_index, &graph.edges);

    // Step 3: Order within layers
    let positions = order_within_layers(&node_ids, &node_index, &graph.edges, &layers);

    // Step 4: Assign coordinates
    assign_coordinates(graph, &layers, &positions);

    // Step 5: Restore reversed edges
    for idx in reversed {
        let edge = &mut graph.edges[idx];
        std::mem::swap(&mut edge.from, &mut edge.to);
    }
}

/// Break cycles by reversing back-edges found via DFS.
/// Returns indices of reversed edges.
pub fn break_cycles(
    node_ids: &[String],
    node_index: &HashMap<String, usize>,
    edges: &mut [trellis_parser::Edge],
) -> Vec<usize> {
    let n = node_ids.len();
    // Build adjacency: node_idx -> Vec<(target_idx, edge_idx)>
    let mut adj: Vec<Vec<(usize, usize)>> = vec![Vec::new(); n];
    for (ei, edge) in edges.iter().enumerate() {
        if let (Some(&from), Some(&to)) = (node_index.get(&edge.from), node_index.get(&edge.to)) {
            adj[from].push((to, ei));
        }
    }

    let mut visited = vec![false; n];
    let mut in_stack = vec![false; n];
    let mut reversed = Vec::new();

    for start in 0..n {
        if !visited[start] {
            dfs_break_cycles(
                start,
                &adj,
                &mut visited,
                &mut in_stack,
                edges,
                &mut reversed,
            );
        }
    }

    reversed
}

fn dfs_break_cycles(
    node: usize,
    adj: &[Vec<(usize, usize)>],
    visited: &mut [bool],
    in_stack: &mut [bool],
    edges: &mut [trellis_parser::Edge],
    reversed: &mut Vec<usize>,
) {
    visited[node] = true;
    in_stack[node] = true;

    for &(target, edge_idx) in &adj[node] {
        if in_stack[target] {
            // Back-edge detected → reverse it
            let edge = &mut edges[edge_idx];
            std::mem::swap(&mut edge.from, &mut edge.to);
            reversed.push(edge_idx);
        } else if !visited[target] {
            dfs_break_cycles(target, adj, visited, in_stack, edges, reversed);
        }
    }

    in_stack[node] = false;
}

/// Assign layers using the longest-path algorithm.
/// Returns a map from node id to layer index.
pub fn assign_layers(
    node_ids: &[String],
    node_index: &HashMap<String, usize>,
    edges: &[trellis_parser::Edge],
) -> HashMap<String, usize> {
    let n = node_ids.len();

    // Build adjacency and compute in-degrees
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut in_degree = vec![0usize; n];

    for edge in edges {
        if let (Some(&from), Some(&to)) = (node_index.get(&edge.from), node_index.get(&edge.to)) {
            adj[from].push(to);
            in_degree[to] += 1;
        }
    }

    // Topological sort (Kahn's algorithm)
    let mut queue: Vec<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
    let mut topo_order = Vec::with_capacity(n);
    let mut remaining_in = in_degree.clone();

    let mut head = 0;
    while head < queue.len() {
        let node = queue[head];
        head += 1;
        topo_order.push(node);

        for &target in &adj[node] {
            remaining_in[target] -= 1;
            if remaining_in[target] == 0 {
                queue.push(target);
            }
        }
    }

    // If there are still unvisited nodes (shouldn't happen after cycle breaking,
    // but be safe), add them
    if topo_order.len() < n {
        for i in 0..n {
            if !topo_order.contains(&i) {
                topo_order.push(i);
            }
        }
    }

    // Longest path: layer[node] = max(layer[pred] + 1) for all predecessors
    let mut layer = vec![0usize; n];
    for &node in &topo_order {
        for &target in &adj[node] {
            layer[target] = layer[target].max(layer[node] + 1);
        }
    }

    // Convert to HashMap
    let mut result = HashMap::new();
    for (i, id) in node_ids.iter().enumerate() {
        result.insert(id.clone(), layer[i]);
    }
    result
}

/// Order nodes within each layer using the barycenter heuristic.
/// Returns a map from node id to position within its layer.
pub fn order_within_layers(
    node_ids: &[String],
    _node_index: &HashMap<String, usize>,
    edges: &[trellis_parser::Edge],
    layers: &HashMap<String, usize>,
) -> HashMap<String, usize> {
    let max_layer = layers.values().copied().max().unwrap_or(0);

    // Group nodes by layer
    let mut nodes_in_layer: Vec<Vec<String>> = vec![Vec::new(); max_layer + 1];
    for id in node_ids {
        if let Some(&layer) = layers.get(id) {
            nodes_in_layer[layer].push(id.clone());
        }
    }

    // Build adjacency for quick neighbor lookup
    // forward: source -> targets, backward: target -> sources
    let mut forward: HashMap<String, Vec<String>> = HashMap::new();
    let mut backward: HashMap<String, Vec<String>> = HashMap::new();
    for edge in edges {
        forward
            .entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
        backward
            .entry(edge.to.clone())
            .or_default()
            .push(edge.from.clone());
    }

    // Initial positions: insertion order
    let mut positions: HashMap<String, f64> = HashMap::new();
    for layer_nodes in &nodes_in_layer {
        for (i, id) in layer_nodes.iter().enumerate() {
            positions.insert(id.clone(), i as f64);
        }
    }

    // Iterative barycenter refinement
    for _iter in 0..MAX_BARYCENTER_ITERATIONS {
        let mut improved = false;

        // Top-down pass
        #[allow(clippy::needless_range_loop)] // TODO: fix this warning
        for layer in 1..=max_layer {
            for node_id in &nodes_in_layer[layer] {
                // Get neighbors in the previous layer
                let neighbors: Vec<&String> = backward
                    .get(node_id)
                    .map(|v| {
                        v.iter()
                            .filter(|n| layers.get(*n) == Some(&(layer - 1)))
                            .collect()
                    })
                    .unwrap_or_default();

                if !neighbors.is_empty() {
                    let barycenter: f64 = neighbors
                        .iter()
                        .map(|n| positions.get(*n).copied().unwrap_or(0.0))
                        .sum::<f64>()
                        / neighbors.len() as f64;
                    positions.insert(node_id.clone(), barycenter);
                }
            }

            // Re-assign integer positions by sorting
            let mut layer_nodes = nodes_in_layer[layer].clone();
            layer_nodes.sort_by(|a, b| {
                positions
                    .get(a)
                    .unwrap_or(&0.0)
                    .partial_cmp(positions.get(b).unwrap_or(&0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            for (i, id) in layer_nodes.iter().enumerate() {
                let old = positions.get(id).copied().unwrap_or(0.0);
                if (old - i as f64).abs() > 0.001 {
                    improved = true;
                }
                positions.insert(id.clone(), i as f64);
            }
            nodes_in_layer[layer] = layer_nodes;
        }

        // Bottom-up pass
        if max_layer > 0 {
            for layer in (0..max_layer).rev() {
                for node_id in &nodes_in_layer[layer] {
                    let neighbors: Vec<&String> = forward
                        .get(node_id)
                        .map(|v| {
                            v.iter()
                                .filter(|n| layers.get(*n) == Some(&(layer + 1)))
                                .collect()
                        })
                        .unwrap_or_default();

                    if !neighbors.is_empty() {
                        let barycenter: f64 = neighbors
                            .iter()
                            .map(|n| positions.get(*n).copied().unwrap_or(0.0))
                            .sum::<f64>()
                            / neighbors.len() as f64;
                        positions.insert(node_id.clone(), barycenter);
                    }
                }

                let mut layer_nodes = nodes_in_layer[layer].clone();
                layer_nodes.sort_by(|a, b| {
                    positions
                        .get(a)
                        .unwrap_or(&0.0)
                        .partial_cmp(positions.get(b).unwrap_or(&0.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                for (i, id) in layer_nodes.iter().enumerate() {
                    positions.insert(id.clone(), i as f64);
                }
                nodes_in_layer[layer] = layer_nodes;
            }
        }

        if !improved {
            break;
        }
    }

    // Convert to integer positions
    let mut result = HashMap::new();
    for (id, pos) in &positions {
        result.insert(id.clone(), *pos as usize);
    }
    result
}

/// Assign (x, y) coordinates to all nodes based on layers, positions, and direction.
/// Coordinates are centered per layer.
pub fn assign_coordinates(
    graph: &mut Graph,
    layers: &HashMap<String, usize>,
    positions: &HashMap<String, usize>,
) {
    let max_layer = layers.values().copied().max().unwrap_or(0);

    // Group nodes by layer for centering
    let mut nodes_in_layer: Vec<Vec<usize>> = vec![Vec::new(); max_layer + 1];
    for (i, node) in graph.nodes.iter().enumerate() {
        if let Some(&layer) = layers.get(&node.id) {
            nodes_in_layer[layer].push(i);
        }
    }

    // Sort nodes within each layer by position
    for layer_nodes in &mut nodes_in_layer {
        layer_nodes.sort_by_key(|&i| positions.get(&graph.nodes[i].id).copied().unwrap_or(0));
    }

    // Assign coordinates with centering (node.x/y = top-left corner)
    for (layer_idx, layer_nodes) in nodes_in_layer.iter().enumerate() {
        if layer_nodes.is_empty() {
            continue;
        }

        // Calculate total width of this layer
        let total_width: f64 = layer_nodes
            .iter()
            .map(|&i| graph.nodes[i].width)
            .sum::<f64>()
            + (layer_nodes.len() as f64 - 1.0) * NODE_SPACING;

        let mut offset = -total_width / 2.0;

        for &node_idx in layer_nodes {
            let node = &mut graph.nodes[node_idx];
            let layer = layer_idx;

            match graph.direction {
                Direction::TB => {
                    node.x = offset;
                    node.y = layer as f64 * LAYER_SPACING;
                }
                Direction::BT => {
                    node.x = offset;
                    node.y = -(layer as f64) * LAYER_SPACING;
                }
                Direction::LR => {
                    node.x = layer as f64 * LAYER_SPACING;
                    node.y = offset;
                }
                Direction::RL => {
                    node.x = -(layer as f64) * LAYER_SPACING;
                    node.y = offset;
                }
            }

            offset += node.width + NODE_SPACING;
        }
    }

    // Normalize: shift so minimum top-left coordinate is at a reasonable origin
    if !graph.nodes.is_empty() {
        let min_x = graph
            .nodes
            .iter()
            .map(|n| n.x)
            .fold(f64::INFINITY, f64::min);
        let min_y = graph
            .nodes
            .iter()
            .map(|n| n.y)
            .fold(f64::INFINITY, f64::min);
        let margin = 50.0;
        for node in &mut graph.nodes {
            node.x -= min_x - margin;
            node.y -= min_y - margin;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_parser::{ArrowHead, DiagramType, Edge, EdgeStyle, Node, NodeShape};

    fn make_node(id: &str) -> Node {
        Node {
            id: id.to_string(),
            label: id.to_string(),
            shape: NodeShape::Rectangle,
            width: 60.0,
            height: 40.0,
            x: 0.0,
            y: 0.0,
        }
    }

    fn make_edge(from: &str, to: &str) -> Edge {
        Edge {
            from: from.to_string(),
            to: to.to_string(),
            label: None,
            style: EdgeStyle::Solid,
            arrow_head: ArrowHead::Arrow,
        }
    }

    fn make_graph(nodes: Vec<&str>, edges: Vec<(&str, &str)>) -> Graph {
        Graph {
            nodes: nodes.iter().map(|id| make_node(id)).collect(),
            edges: edges.iter().map(|(f, t)| make_edge(f, t)).collect(),
            subgraphs: Vec::new(),
            direction: Direction::TB,
            diagram_type: DiagramType::Flowchart,
        }
    }

    // ── Cycle breaking tests ──

    #[test]
    fn test_break_cycles_no_cycle() {
        let mut graph = make_graph(vec!["A", "B", "C"], vec![("A", "B"), ("B", "C")]);
        let node_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
        let node_index: HashMap<String, usize> = node_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect();

        let reversed = break_cycles(&node_ids, &node_index, &mut graph.edges);
        assert!(reversed.is_empty());
    }

    #[test]
    fn test_break_cycles_simple_cycle() {
        let mut graph = make_graph(
            vec!["A", "B", "C", "D"],
            vec![("A", "B"), ("B", "C"), ("C", "D"), ("D", "A")],
        );
        let node_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
        let node_index: HashMap<String, usize> = node_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect();

        let reversed = break_cycles(&node_ids, &node_index, &mut graph.edges);
        assert!(!reversed.is_empty(), "At least one edge should be reversed");

        // After cycle breaking, there should be no cycles
        // Verify by checking that assign_layers doesn't loop forever
        let layers = assign_layers(&node_ids, &node_index, &graph.edges);
        assert_eq!(layers.len(), 4);
    }

    // ── Layer assignment tests ──

    #[test]
    fn test_assign_layers_linear() {
        let graph = make_graph(
            vec!["A", "B", "C", "D", "E"],
            vec![("A", "B"), ("B", "C"), ("C", "D"), ("D", "E")],
        );
        let node_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
        let node_index: HashMap<String, usize> = node_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect();

        let layers = assign_layers(&node_ids, &node_index, &graph.edges);

        assert_eq!(layers["A"], 0);
        assert_eq!(layers["B"], 1);
        assert_eq!(layers["C"], 2);
        assert_eq!(layers["D"], 3);
        assert_eq!(layers["E"], 4);
    }

    #[test]
    fn test_assign_layers_diamond() {
        let graph = make_graph(
            vec!["A", "B", "C", "D"],
            vec![("A", "B"), ("A", "C"), ("B", "D"), ("C", "D")],
        );
        let node_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
        let node_index: HashMap<String, usize> = node_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect();

        let layers = assign_layers(&node_ids, &node_index, &graph.edges);

        assert_eq!(layers["A"], 0);
        assert_eq!(layers["B"], 1);
        assert_eq!(layers["C"], 1);
        assert_eq!(layers["D"], 2);
    }

    #[test]
    fn test_assign_layers_wide_branch() {
        let graph = make_graph(
            vec!["A", "B1", "B2", "B3", "B4", "B5", "B6"],
            vec![
                ("A", "B1"),
                ("A", "B2"),
                ("A", "B3"),
                ("A", "B4"),
                ("A", "B5"),
                ("A", "B6"),
            ],
        );
        let node_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
        let node_index: HashMap<String, usize> = node_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect();

        let layers = assign_layers(&node_ids, &node_index, &graph.edges);

        assert_eq!(layers["A"], 0);
        for id in &["B1", "B2", "B3", "B4", "B5", "B6"] {
            assert_eq!(layers[*id], 1, "Node {} should be in layer 1", id);
        }
    }

    // ── Barycenter ordering tests ──

    #[test]
    fn test_order_within_layers_diamond() {
        let graph = make_graph(
            vec!["A", "B", "C", "D"],
            vec![("A", "B"), ("A", "C"), ("B", "D"), ("C", "D")],
        );
        let node_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
        let node_index: HashMap<String, usize> = node_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect();

        let layers = assign_layers(&node_ids, &node_index, &graph.edges);
        let positions = order_within_layers(&node_ids, &node_index, &graph.edges, &layers);

        // B and C should be in layer 1 with positions 0 and 1
        assert!(positions["B"] < 2);
        assert!(positions["C"] < 2);
        assert_ne!(positions["B"], positions["C"]);
    }

    #[test]
    fn test_barycenter_convergence() {
        // Bipartite-like graph: should converge and minimize crossings
        let graph = make_graph(
            vec!["A", "B", "C", "X", "Y", "Z"],
            vec![("A", "X"), ("B", "Y"), ("C", "Z")],
        );
        let node_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
        let node_index: HashMap<String, usize> = node_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i))
            .collect();

        let layers = assign_layers(&node_ids, &node_index, &graph.edges);
        let positions = order_within_layers(&node_ids, &node_index, &graph.edges, &layers);

        // Each node should have a defined position
        for id in &node_ids {
            assert!(
                positions.contains_key(id),
                "Node {} should have a position",
                id
            );
        }
    }

    // ── Full layout tests ──

    #[test]
    fn test_layout_linear_tb() {
        let mut graph = make_graph(vec!["A", "B", "C"], vec![("A", "B"), ("B", "C")]);
        layout(&mut graph);

        // In TB layout, y should increase from A to C
        let a = graph.nodes.iter().find(|n| n.id == "A").unwrap();
        let b = graph.nodes.iter().find(|n| n.id == "B").unwrap();
        let c = graph.nodes.iter().find(|n| n.id == "C").unwrap();

        assert!(a.y < b.y, "A.y ({}) should be less than B.y ({})", a.y, b.y);
        assert!(b.y < c.y, "B.y ({}) should be less than C.y ({})", b.y, c.y);
    }

    #[test]
    fn test_layout_linear_lr() {
        let mut graph = make_graph(vec!["A", "B", "C"], vec![("A", "B"), ("B", "C")]);
        graph.direction = Direction::LR;
        layout(&mut graph);

        let a = graph.nodes.iter().find(|n| n.id == "A").unwrap();
        let b = graph.nodes.iter().find(|n| n.id == "B").unwrap();
        let c = graph.nodes.iter().find(|n| n.id == "C").unwrap();

        assert!(a.x < b.x, "A.x ({}) should be less than B.x ({})", a.x, b.x);
        assert!(b.x < c.x, "B.x ({}) should be less than C.x ({})", b.x, c.x);
    }

    #[test]
    fn test_layout_with_cycle() {
        let mut graph = make_graph(
            vec!["A", "B", "C", "D"],
            vec![("A", "B"), ("B", "C"), ("C", "D"), ("D", "A")],
        );
        layout(&mut graph);

        // Should not panic, all nodes should have coordinates
        for node in &graph.nodes {
            assert!(node.x.is_finite(), "Node {} has non-finite x", node.id);
            assert!(node.y.is_finite(), "Node {} has non-finite y", node.id);
        }
    }

    #[test]
    fn test_layout_single_node() {
        let mut graph = make_graph(vec!["A"], vec![]);
        layout(&mut graph);

        assert!(graph.nodes[0].x.is_finite());
        assert!(graph.nodes[0].y.is_finite());
    }

    #[test]
    fn test_layout_empty_graph() {
        let mut graph = make_graph(vec![], vec![]);
        layout(&mut graph);
        // Should not panic
    }

    #[test]
    fn test_layout_diamond_no_overlap() {
        let mut graph = make_graph(
            vec!["A", "B", "C", "D"],
            vec![("A", "B"), ("A", "C"), ("B", "D"), ("C", "D")],
        );
        layout(&mut graph);

        // B and C should be on the same layer but different x positions
        let b = graph.nodes.iter().find(|n| n.id == "B").unwrap();
        let c = graph.nodes.iter().find(|n| n.id == "C").unwrap();

        assert!(
            (b.y - c.y).abs() < 1.0,
            "B and C should be on the same layer (B.y={}, C.y={})",
            b.y,
            c.y
        );
        assert!(
            (b.x - c.x).abs() > 10.0,
            "B and C should not overlap (B.x={}, C.x={})",
            b.x,
            c.x
        );
    }
}
