use std::collections::HashMap;
use trellis_parser::Graph;

/// A canonical key for multi-edge detection.
/// Two edges between the same pair of nodes (regardless of direction)
/// share the same canonical key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanonicalKey {
    /// The smaller node id (alphabetically)
    pub node_a: String,
    /// The larger node id (alphabetically)
    pub node_b: String,
}

impl CanonicalKey {
    pub fn new(from: &str, to: &str) -> Self {
        if from <= to {
            Self {
                node_a: from.to_string(),
                node_b: to.to_string(),
            }
        } else {
            Self {
                node_a: to.to_string(),
                node_b: from.to_string(),
            }
        }
    }
}

/// A group of edges that share the same canonical key (multi-edges)
#[derive(Debug, Clone)]
pub struct MultiEdgeGroup {
    pub key: CanonicalKey,
    pub edge_indices: Vec<usize>,
}

/// Detect multi-edges in the graph.
///
/// Multi-edges are multiple edges connecting the same pair of nodes.
/// They need special handling during routing to ensure they don't overlap.
///
/// Returns groups that have 2+ edges. Single edges are not returned.
pub fn detect_multi_edges(graph: &Graph) -> Vec<MultiEdgeGroup> {
    let mut groups: HashMap<CanonicalKey, Vec<usize>> = HashMap::new();

    for (idx, edge) in graph.edges.iter().enumerate() {
        let key = CanonicalKey::new(&edge.from, &edge.to);
        groups.entry(key).or_default().push(idx);
    }

    groups
        .into_iter()
        .filter(|(_, indices)| indices.len() > 1)
        .map(|(key, edge_indices)| MultiEdgeGroup { key, edge_indices })
        .collect()
}

/// Check if a specific edge index is part of a multi-edge group
pub fn is_multi_edge(edge_index: usize, multi_groups: &[MultiEdgeGroup]) -> bool {
    multi_groups
        .iter()
        .any(|g| g.edge_indices.contains(&edge_index))
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_parser::{Edge, EdgeStyle, ArrowHead};

    fn make_edge(from: &str, to: &str) -> Edge {
        Edge {
            from: from.to_string(),
            to: to.to_string(),
            label: None,
            style: EdgeStyle::Solid,
            arrow_head: ArrowHead::Arrow,
            ..Default::default()        }
    }

    #[test]
    fn test_no_multi_edges() {
        let mut graph = Graph::new();
        graph.edges = vec![make_edge("A", "B"), make_edge("B", "C")];

        let groups = detect_multi_edges(&graph);
        assert!(groups.is_empty());
    }

    #[test]
    fn test_detect_multi_edges() {
        let mut graph = Graph::new();
        graph.edges = vec![
            make_edge("A", "B"),
            make_edge("A", "B"), // duplicate
            make_edge("B", "C"),
        ];

        let groups = detect_multi_edges(&graph);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].edge_indices.len(), 2);
        assert!(groups[0].edge_indices.contains(&0));
        assert!(groups[0].edge_indices.contains(&1));
    }

    #[test]
    fn test_canonical_key_direction_independent() {
        let key1 = CanonicalKey::new("A", "B");
        let key2 = CanonicalKey::new("B", "A");
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_is_multi_edge() {
        let groups = vec![MultiEdgeGroup {
            key: CanonicalKey::new("A", "B"),
            edge_indices: vec![0, 1],
        }];

        assert!(is_multi_edge(0, &groups));
        assert!(is_multi_edge(1, &groups));
        assert!(!is_multi_edge(2, &groups));
    }
}
