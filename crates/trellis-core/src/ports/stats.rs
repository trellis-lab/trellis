use std::collections::{HashMap, HashSet};

use trellis_parser::{DiagramType, Graph};

use super::assignment::Side;
use super::common::{angle_to_side, calculate_angle, enumerate_connectors};

/// Default hub threshold (nodes with degree >= this are considered hubs).
pub const HUB_THRESHOLD_DEFAULT: usize = 4;

/// Structural and spatial statistics of a placed graph.
///
/// Computed once in O(N + E), used by the Auto selector to pick a strategy.
pub struct GraphStats {
    // ── Size ──────────────────────────────────────────────────
    pub node_count: usize,
    pub edge_count: usize,
    pub subgraph_count: usize,
    pub diagram_type: DiagramType,

    // ── Degree distribution ───────────────────────────────────
    pub max_degree: usize,
    pub avg_degree: f64,
    pub degree_std_dev: f64,
    /// Nodes with degree >= `HUB_THRESHOLD_DEFAULT`.
    pub hub_count: usize,

    // ── Side congestion ───────────────────────────────────────
    /// Maximum edges assigned to any single side across all nodes.
    pub max_edges_per_side: usize,
    /// Edges that exceed the connector capacity of their assigned side.
    pub overflow_count: usize,
    /// Average available connectors per occupied side.
    pub avg_connectors_per_side: f64,

    // ── Edge patterns ─────────────────────────────────────────
    /// Node pairs with 2+ edges between them.
    pub multi_edge_count: usize,
    /// Node pairs where A→B and B→A both exist.
    pub bidirectional_edge_count: usize,
    pub avg_edge_node_ratio: f64,
    pub median_edge_node_ratio: f64,

    // ── Spatial ───────────────────────────────────────────────
    /// Euclidean distance of the furthest pair of connected nodes.
    pub max_node_distance: f64,
    /// Bounding-box area / node_count.
    pub layout_density: f64,
    /// Fewest available connectors on any side that has at least one edge.
    pub min_connectors_on_loaded_side: usize,
}

/// Compute graph statistics from a placed graph in a single pass.
pub fn compute_graph_stats(
    graph: &Graph,
    cell_size: i32,
    offset_x: i32,
    offset_y: i32,
) -> GraphStats {
    let node_count = graph.nodes.len();
    let edge_count = graph.edges.len();
    let subgraph_count = graph.subgraphs.len();
    let diagram_type = graph.diagram_type;

    // ── Node lookup ──────────────────────────────────────────
    let node_map: HashMap<&str, &trellis_parser::Node> =
        graph.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    // ── Degree distribution ──────────────────────────────────
    let mut degrees: HashMap<&str, usize> = HashMap::new();
    for edge in &graph.edges {
        *degrees.entry(edge.from.as_str()).or_default() += 1;
        *degrees.entry(edge.to.as_str()).or_default() += 1;
    }

    let max_degree = degrees.values().copied().max().unwrap_or(0);
    let deg_count = degrees.len().max(1);
    let deg_sum: usize = degrees.values().sum();
    let avg_degree = deg_sum as f64 / deg_count as f64;

    let degree_std_dev = if deg_count > 1 {
        let variance = degrees
            .values()
            .map(|&d| {
                let diff = d as f64 - avg_degree;
                diff * diff
            })
            .sum::<f64>()
            / deg_count as f64;
        variance.sqrt()
    } else {
        0.0
    };

    let hub_count = degrees
        .values()
        .filter(|&&d| d >= HUB_THRESHOLD_DEFAULT)
        .count();

    // ── Edge patterns ────────────────────────────────────────
    let mut edge_pairs: HashMap<(&str, &str), usize> = HashMap::new();
    let mut directed_pairs: HashSet<(&str, &str)> = HashSet::new();

    for edge in &graph.edges {
        let a = edge.from.as_str();
        let b = edge.to.as_str();
        let canonical = if a < b { (a, b) } else { (b, a) };
        *edge_pairs.entry(canonical).or_default() += 1;
        directed_pairs.insert((a, b));
    }

    let multi_edge_count = edge_pairs.values().filter(|&&c| c >= 2).count();
    let bidirectional_edge_count = directed_pairs
        .iter()
        .filter(|&&(a, b)| directed_pairs.contains(&(b, a)))
        .count()
        / 2; // each pair counted twice

    let avg_edge_node_ratio = if node_count > 0 {
        edge_count as f64 / node_count as f64
    } else {
        0.0
    };

    // Per-node edge counts for median
    let mut per_node_edge_counts: Vec<f64> = degrees.values().map(|&d| d as f64).collect();
    per_node_edge_counts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median_edge_node_ratio = if per_node_edge_counts.is_empty() {
        0.0
    } else {
        let mid = per_node_edge_counts.len() / 2;
        if per_node_edge_counts.len().is_multiple_of(2) && per_node_edge_counts.len() > 1 {
            (per_node_edge_counts[mid - 1] + per_node_edge_counts[mid]) / 2.0
        } else {
            per_node_edge_counts[mid]
        }
    };

    // ── Side congestion ──────────────────────────────────────
    let sides_all = [Side::Top, Side::Right, Side::Bottom, Side::Left];
    let mut max_edges_per_side: usize = 0;
    let mut overflow_count: usize = 0;
    let mut total_connectors: usize = 0;
    let mut occupied_side_count: usize = 0;
    let mut min_connectors_on_loaded_side: usize = usize::MAX;

    for node in &graph.nodes {
        // Count edges per side for this node
        let mut side_counts: HashMap<Side, usize> = HashMap::new();
        for edge in &graph.edges {
            let other = if edge.from == node.id {
                node_map.get(edge.to.as_str())
            } else if edge.to == node.id {
                node_map.get(edge.from.as_str())
            } else {
                None
            };

            if let Some(other_node) = other {
                let angle = calculate_angle(node, other_node);
                let side = angle_to_side(angle);
                *side_counts.entry(side).or_default() += 1;
            }
        }

        for &side in &sides_all {
            let edge_count_on_side = side_counts.get(&side).copied().unwrap_or(0);
            if edge_count_on_side == 0 {
                continue;
            }

            max_edges_per_side = max_edges_per_side.max(edge_count_on_side);

            let connectors = enumerate_connectors(node, side, cell_size, offset_x, offset_y).len();
            if edge_count_on_side > connectors {
                overflow_count += edge_count_on_side - connectors;
            }

            total_connectors += connectors;
            occupied_side_count += 1;

            if connectors > 0 {
                min_connectors_on_loaded_side = min_connectors_on_loaded_side.min(connectors);
            }
        }
    }

    let avg_connectors_per_side = if occupied_side_count > 0 {
        total_connectors as f64 / occupied_side_count as f64
    } else {
        0.0
    };

    if min_connectors_on_loaded_side == usize::MAX {
        min_connectors_on_loaded_side = 0;
    }

    // ── Spatial ──────────────────────────────────────────────
    let mut max_node_distance: f64 = 0.0;
    for edge in &graph.edges {
        if let (Some(src), Some(tgt)) = (
            node_map.get(edge.from.as_str()),
            node_map.get(edge.to.as_str()),
        ) {
            let dx = (src.x + src.width / 2.0) - (tgt.x + tgt.width / 2.0);
            let dy = (src.y + src.height / 2.0) - (tgt.y + tgt.height / 2.0);
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > max_node_distance {
                max_node_distance = dist;
            }
        }
    }

    let layout_density = if node_count > 0 && !graph.nodes.is_empty() {
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        for n in &graph.nodes {
            min_x = min_x.min(n.x);
            min_y = min_y.min(n.y);
            max_x = max_x.max(n.x + n.width);
            max_y = max_y.max(n.y + n.height);
        }
        let area = (max_x - min_x).max(1.0) * (max_y - min_y).max(1.0);
        area / node_count as f64
    } else {
        0.0
    };

    GraphStats {
        node_count,
        edge_count,
        subgraph_count,
        diagram_type,
        max_degree,
        avg_degree,
        degree_std_dev,
        hub_count,
        max_edges_per_side,
        overflow_count,
        avg_connectors_per_side,
        multi_edge_count,
        bidirectional_edge_count,
        avg_edge_node_ratio,
        median_edge_node_ratio,
        max_node_distance,
        layout_density,
        min_connectors_on_loaded_side,
    }
}

/// Recount hubs using a custom threshold (for diagram-type adjustments).
pub fn count_hubs_with_threshold(graph: &Graph, threshold: usize) -> usize {
    let mut degrees: HashMap<&str, usize> = HashMap::new();
    for edge in &graph.edges {
        *degrees.entry(edge.from.as_str()).or_default() += 1;
        *degrees.entry(edge.to.as_str()).or_default() += 1;
    }
    degrees.values().filter(|&&d| d >= threshold).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_parser::{Node, NodeShape};

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

    fn make_edge(from: &str, to: &str) -> trellis_parser::Edge {
        trellis_parser::Edge {
            from: from.to_string(),
            to: to.to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn empty_graph_stats() {
        let graph = Graph::new();
        let stats = compute_graph_stats(&graph, 10, 0, 0);
        assert_eq!(stats.node_count, 0);
        assert_eq!(stats.edge_count, 0);
        assert_eq!(stats.max_degree, 0);
        assert_eq!(stats.hub_count, 0);
    }

    #[test]
    fn simple_chain_stats() {
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("A", 40.0, 20.0, 0.0, 0.0),
            make_node("B", 40.0, 20.0, 0.0, 100.0),
            make_node("C", 40.0, 20.0, 0.0, 200.0),
        ];
        graph.edges = vec![make_edge("A", "B"), make_edge("B", "C")];

        let stats = compute_graph_stats(&graph, 10, 0, 0);
        assert_eq!(stats.node_count, 3);
        assert_eq!(stats.edge_count, 2);
        assert_eq!(stats.max_degree, 2); // B has degree 2
        assert_eq!(stats.hub_count, 0);
        assert_eq!(stats.multi_edge_count, 0);
        assert_eq!(stats.bidirectional_edge_count, 0);
    }

    #[test]
    fn hub_detection() {
        let mut graph = Graph::new();
        // Hub node A connects to B, C, D, E (degree 4 = hub)
        graph.nodes = vec![
            make_node("A", 80.0, 20.0, 60.0, 0.0),
            make_node("B", 40.0, 20.0, 0.0, 100.0),
            make_node("C", 40.0, 20.0, 60.0, 100.0),
            make_node("D", 40.0, 20.0, 120.0, 100.0),
            make_node("E", 40.0, 20.0, 180.0, 100.0),
        ];
        graph.edges = vec![
            make_edge("A", "B"),
            make_edge("A", "C"),
            make_edge("A", "D"),
            make_edge("A", "E"),
        ];

        let stats = compute_graph_stats(&graph, 10, 0, 0);
        assert_eq!(stats.max_degree, 4);
        assert_eq!(stats.hub_count, 1); // A is a hub
    }

    #[test]
    fn multi_edge_and_bidirectional_detection() {
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("A", 40.0, 20.0, 0.0, 0.0),
            make_node("B", 40.0, 20.0, 0.0, 100.0),
        ];
        graph.edges = vec![
            make_edge("A", "B"),
            make_edge("A", "B"), // multi-edge
            make_edge("B", "A"), // bidirectional
        ];

        let stats = compute_graph_stats(&graph, 10, 0, 0);
        assert_eq!(stats.multi_edge_count, 1);
        assert_eq!(stats.bidirectional_edge_count, 1);
    }

    #[test]
    fn overflow_counted_on_tiny_node() {
        let mut graph = Graph::new();
        // Very small node (10x10 = 1 cell) with 3 edges on same side
        graph.nodes = vec![
            make_node("A", 10.0, 10.0, 50.0, 0.0),
            make_node("B", 10.0, 10.0, 50.0, 100.0),
            make_node("C", 10.0, 10.0, 60.0, 100.0),
            make_node("D", 10.0, 10.0, 40.0, 100.0),
        ];
        graph.edges = vec![
            make_edge("A", "B"),
            make_edge("A", "C"),
            make_edge("A", "D"),
        ];

        let stats = compute_graph_stats(&graph, 10, 0, 0);
        // All 3 edges point downward → same side, tiny node has few connectors
        assert!(stats.overflow_count > 0 || stats.max_edges_per_side >= 3);
    }

    #[test]
    fn count_hubs_with_custom_threshold() {
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("A", 40.0, 20.0, 0.0, 0.0),
            make_node("B", 40.0, 20.0, 100.0, 0.0),
            make_node("C", 40.0, 20.0, 0.0, 100.0),
            make_node("D", 40.0, 20.0, 100.0, 100.0),
        ];
        graph.edges = vec![
            make_edge("A", "B"),
            make_edge("A", "C"),
            make_edge("A", "D"),
            make_edge("B", "C"),
        ];

        // With threshold 3: A(degree=3) is a hub
        assert_eq!(count_hubs_with_threshold(&graph, 3), 1);
        // With threshold 4: no hubs
        assert_eq!(count_hubs_with_threshold(&graph, 4), 0);
    }

    #[test]
    fn spatial_metrics_computed() {
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("A", 40.0, 20.0, 0.0, 0.0),
            make_node("B", 40.0, 20.0, 200.0, 200.0),
        ];
        graph.edges = vec![make_edge("A", "B")];

        let stats = compute_graph_stats(&graph, 10, 0, 0);
        assert!(stats.max_node_distance > 0.0);
        assert!(stats.layout_density > 0.0);
    }
}
