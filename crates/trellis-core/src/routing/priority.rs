use std::collections::HashMap;
use trellis_parser::Graph;

/// Priority score for an edge, used to determine routing order.
/// Higher priority edges are routed first.
#[derive(Debug, Clone)]
pub struct EdgePriority {
    pub edge_index: usize,
    pub score: f64,
}

/// Calculate routing priority for all edges.
///
/// Uses a hybrid scoring approach:
/// - **Degree factor**: edges connecting high-degree nodes get higher priority
///   (they need more routing space, so routing them first gives better results)
/// - **Distance factor**: shorter edges get higher priority
///   (they're easier to route and blocking them early doesn't waste much space)
/// - **Congestion factor**: edges in dense regions get higher priority
pub fn calculate_priorities(graph: &Graph) -> Vec<EdgePriority> {
    if graph.edges.is_empty() {
        return vec![];
    }

    // Calculate node degrees
    let mut degrees: HashMap<&str, usize> = HashMap::new();
    for edge in &graph.edges {
        *degrees.entry(edge.from.as_str()).or_insert(0) += 1;
        *degrees.entry(edge.to.as_str()).or_insert(0) += 1;
    }

    // Build node position lookup
    let node_pos: HashMap<&str, (f64, f64)> = graph
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), (n.x, n.y)))
        .collect();

    // Calculate raw scores
    let mut priorities: Vec<EdgePriority> = graph
        .edges
        .iter()
        .enumerate()
        .map(|(idx, edge)| {
            let source_degree = *degrees.get(edge.from.as_str()).unwrap_or(&1) as f64;
            let target_degree = *degrees.get(edge.to.as_str()).unwrap_or(&1) as f64;
            let degree_score = source_degree + target_degree;

            // Distance: shorter edges get higher priority (inverse)
            let distance = match (
                node_pos.get(edge.from.as_str()),
                node_pos.get(edge.to.as_str()),
            ) {
                (Some(&(x1, y1)), Some(&(x2, y2))) => {
                    ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
                }
                _ => 100.0,
            };
            let distance_score = 1.0 / (1.0 + distance / 100.0);

            // Hybrid score: weight degree more heavily
            let score = degree_score * 2.0 + distance_score;

            EdgePriority {
                edge_index: idx,
                score,
            }
        })
        .collect();

    // Sort by score descending (highest priority first)
    priorities.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    priorities
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_parser::{ArrowHead, Edge, EdgeStyle, Node, NodeShape};

    fn make_node(id: &str, x: f64, y: f64) -> Node {
        Node {
            id: id.to_string(),
            label: id.to_string(),
            shape: NodeShape::Rectangle,
            width: 80.0,
            height: 40.0,
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
    fn test_empty_graph() {
        let graph = Graph::new();
        let priorities = calculate_priorities(&graph);
        assert!(priorities.is_empty());
    }

    #[test]
    fn test_high_degree_nodes_prioritized() {
        let mut graph = Graph::new();
        // A is a hub (degree 3), D-E is a leaf edge (degree 1 each)
        graph.nodes = vec![
            make_node("A", 100.0, 50.0),
            make_node("B", 200.0, 50.0),
            make_node("C", 100.0, 150.0),
            make_node("D", 300.0, 50.0),
            make_node("E", 400.0, 50.0),
        ];
        graph.edges = vec![
            make_edge("A", "B"), // A degree=3
            make_edge("A", "C"),
            make_edge("A", "D"),
            make_edge("D", "E"), // D degree=2, E degree=1
        ];

        let priorities = calculate_priorities(&graph);
        assert_eq!(priorities.len(), 4);

        // The first edges should involve node A (highest degree)
        let first_edge = priorities[0].edge_index;
        let edge = &graph.edges[first_edge];
        assert!(
            edge.from == "A" || edge.to == "A",
            "First priority edge should involve hub node A"
        );
    }

    #[test]
    fn test_all_edges_present() {
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("A", 0.0, 0.0),
            make_node("B", 100.0, 0.0),
            make_node("C", 0.0, 100.0),
        ];
        graph.edges = vec![
            make_edge("A", "B"),
            make_edge("B", "C"),
            make_edge("A", "C"),
        ];

        let priorities = calculate_priorities(&graph);
        assert_eq!(priorities.len(), 3);

        // All edge indices should be present
        let mut indices: Vec<usize> = priorities.iter().map(|p| p.edge_index).collect();
        indices.sort();
        assert_eq!(indices, vec![0, 1, 2]);
    }
}
