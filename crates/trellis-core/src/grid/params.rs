use trellis_parser::Graph;

/// Calculate the optimal cell size based on node dimensions and edge density.
///
/// Follows the spec: cellSize = floor(minDimension / R), where R depends on edge density.
pub fn calculate_cell_size(graph: &Graph) -> i32 {
    if graph.nodes.is_empty() {
        return 20; // default fallback
    }

    // Find the smallest node dimension
    let min_dimension = graph
        .nodes
        .iter()
        .flat_map(|n| [n.width, n.height])
        .fold(f64::INFINITY, f64::min);

    // Routing factor based on edge density
    let edge_density = if graph.nodes.is_empty() {
        0.0
    } else {
        graph.edges.len() as f64 / graph.nodes.len() as f64
    };

    let r = if edge_density < 1.5 {
        4.0 // sparse graph
    } else if edge_density <= 3.0 {
        5.0 // average
    } else {
        6.0 // dense graph
    };

    let cell_size = (min_dimension / r).floor() as i32;

    // Minimum cell size
    cell_size.max(5)
}

/// Grid extent (width and height in pixels) with safety multiplier.
#[derive(Debug, Clone, Copy)]
pub struct GridExtent {
    pub width: f64,
    pub height: f64,
    pub offset_x: i32,
    pub offset_y: i32,
}

/// Calculate the grid extent based on node positions, with a safety multiplier K.
///
/// K starts at 1.5 and increases based on edge density, max degree, and subgraphs.
pub fn calculate_grid_extent(graph: &Graph) -> GridExtent {
    if graph.nodes.is_empty() {
        return GridExtent {
            width: 100.0,
            height: 100.0,
            offset_x: 0,
            offset_y: 0,
        };
    }

    // Bounding box from node positions (nodes store center coordinates)
    let min_x = graph
        .nodes
        .iter()
        .map(|n| n.x - n.width / 2.0)
        .fold(f64::INFINITY, f64::min);
    let max_x = graph
        .nodes
        .iter()
        .map(|n| n.x + n.width / 2.0)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_y = graph
        .nodes
        .iter()
        .map(|n| n.y - n.height / 2.0)
        .fold(f64::INFINITY, f64::min);
    let max_y = graph
        .nodes
        .iter()
        .map(|n| n.y + n.height / 2.0)
        .fold(f64::NEG_INFINITY, f64::max);

    let bounding_width = max_x - min_x;
    let bounding_height = max_y - min_y;

    // Safety multiplier K
    let edge_density = graph.edges.len() as f64 / graph.nodes.len() as f64;

    // Calculate max degree
    let max_degree = calculate_max_degree(graph);

    let has_subgraphs = !graph.subgraphs.is_empty();

    let mut k = 1.5;
    if edge_density > 2.0 {
        k += 0.3;
    }
    if max_degree > 6 {
        k += 0.2;
    }
    if has_subgraphs {
        k += 0.2;
    }

    GridExtent {
        width: (bounding_width * k).ceil(),
        height: (bounding_height * k).ceil(),
        offset_x: (min_x - (bounding_width * (k - 1.0) / 2.0)).floor() as i32,
        offset_y: (min_y - (bounding_height * (k - 1.0) / 2.0)).floor() as i32,
    }
}

fn calculate_max_degree(graph: &Graph) -> usize {
    if graph.nodes.is_empty() {
        return 0;
    }

    use std::collections::HashMap;
    let mut degrees: HashMap<&str, usize> = HashMap::new();

    for edge in &graph.edges {
        *degrees.entry(edge.from.as_str()).or_insert(0) += 1;
        *degrees.entry(edge.to.as_str()).or_insert(0) += 1;
    }

    degrees.values().copied().max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_parser::{Edge, Node, NodeShape, EdgeStyle, ArrowHead};

    fn make_node(id: &str, w: f64, h: f64, x: f64, y: f64) -> Node {
        Node {
            id: id.to_string(),
            label: id.to_string(),
            shape: NodeShape::Rectangle,
            width: w,
            height: h,
            x,
            y,
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

    #[test]
    fn test_cell_size_sparse_graph() {
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("A", 80.0, 40.0, 0.0, 0.0),
            make_node("B", 80.0, 40.0, 100.0, 100.0),
        ];
        graph.edges = vec![make_edge("A", "B")];

        let cell_size = calculate_cell_size(&graph);
        // min_dimension = 40, density < 1.5 → R=4, cell = floor(40/4) = 10
        assert_eq!(cell_size, 10);
    }

    #[test]
    fn test_cell_size_dense_graph() {
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("A", 60.0, 30.0, 0.0, 0.0),
            make_node("B", 60.0, 30.0, 100.0, 0.0),
        ];
        graph.edges = vec![
            make_edge("A", "B"),
            make_edge("B", "A"),
            make_edge("A", "B"),
            make_edge("B", "A"),
            make_edge("A", "B"),
            make_edge("B", "A"),
            make_edge("A", "B"),
        ];

        let cell_size = calculate_cell_size(&graph);
        // min_dimension = 30, density = 3.5 > 3.0 → R=6, cell = floor(30/6) = 5
        assert_eq!(cell_size, 5);
    }

    #[test]
    fn test_cell_size_minimum() {
        let mut graph = Graph::new();
        graph.nodes = vec![make_node("A", 10.0, 10.0, 0.0, 0.0)];
        graph.edges = vec![];

        let cell_size = calculate_cell_size(&graph);
        // min_dimension = 10, density = 0 → R=4, cell = floor(10/4) = 2 → max(2, 5) = 5
        assert_eq!(cell_size, 5);
    }

    #[test]
    fn test_grid_extent_basic() {
        let mut graph = Graph::new();
        graph.nodes = vec![
            make_node("A", 80.0, 40.0, 40.0, 20.0),  // left edge 0, right 80, top 0, bottom 40
            make_node("B", 80.0, 40.0, 240.0, 120.0), // left 200, right 280, top 100, bottom 140
        ];
        graph.edges = vec![make_edge("A", "B")];

        let extent = calculate_grid_extent(&graph);
        // bounding: width = 280-0 = 280, height = 140-0 = 140
        // density = 0.5, K = 1.5
        assert_eq!(extent.width, (280.0 * 1.5_f64).ceil());
        assert_eq!(extent.height, (140.0 * 1.5_f64).ceil());
    }

    #[test]
    fn test_grid_extent_empty() {
        let graph = Graph::new();
        let extent = calculate_grid_extent(&graph);
        assert_eq!(extent.width, 100.0);
        assert_eq!(extent.height, 100.0);
    }
}
