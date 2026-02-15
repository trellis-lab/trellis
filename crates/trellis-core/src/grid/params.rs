use trellis_parser::Graph;

/// Calculate the optimal cell size based on node dimensions and edge density.
///
/// Follows the spec: cellSize = floor(minDimension / R), where R depends on edge density.
pub fn suggest_cell_size(graph: &Graph) -> i32 {
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
/// Offsets are snapped to `cell_size` multiples so the grid stays aligned with nodes.
pub fn calculate_grid_extent(graph: &Graph, cell_size: i32) -> GridExtent {
    if graph.nodes.is_empty() {
        return GridExtent {
            width: 100.0,
            height: 100.0,
            offset_x: 0,
            offset_y: 0,
        };
    }

    // Bounding box from node positions (nodes store top-left coordinates)
    let min_x = graph
        .nodes
        .iter()
        .map(|n| n.x)
        .fold(f64::INFINITY, f64::min);
    let max_x = graph
        .nodes
        .iter()
        .map(|n| n.x + n.width)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_y = graph
        .nodes
        .iter()
        .map(|n| n.y)
        .fold(f64::INFINITY, f64::min);
    let max_y = graph
        .nodes
        .iter()
        .map(|n| n.y + n.height)
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

    let cs = cell_size as f64;

    // Snap offsets to cell_size multiples (round down so the grid origin is before the nodes)
    let raw_offset_x = min_x - (bounding_width * (k - 1.0) / 2.0);
    let raw_offset_y = min_y - (bounding_height * (k - 1.0) / 2.0);
    let offset_x = (raw_offset_x / cs).floor() as i32 * cell_size;
    let offset_y = (raw_offset_y / cs).floor() as i32 * cell_size;

    // Recompute width/height to cover from offset to max + margin, snapped up
    let needed_w = max_x - offset_x as f64 + bounding_width * (k - 1.0) / 2.0;
    let needed_h = max_y - offset_y as f64 + bounding_height * (k - 1.0) / 2.0;

    GridExtent {
        width: (needed_w / cs).ceil() * cs,
        height: (needed_h / cs).ceil() * cs,
        offset_x,
        offset_y,
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

        let cell_size = suggest_cell_size(&graph);
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

        let cell_size = suggest_cell_size(&graph);
        // min_dimension = 30, density = 3.5 > 3.0 → R=6, cell = floor(30/6) = 5
        assert_eq!(cell_size, 5);
    }

    #[test]
    fn test_cell_size_minimum() {
        let mut graph = Graph::new();
        graph.nodes = vec![make_node("A", 10.0, 10.0, 0.0, 0.0)];
        graph.edges = vec![];

        let cell_size = suggest_cell_size(&graph);
        // min_dimension = 10, density = 0 → R=4, cell = floor(10/4) = 2 → max(2, 5) = 5
        assert_eq!(cell_size, 5);
    }

    #[test]
    fn test_grid_extent_basic() {
        let mut graph = Graph::new();
        // Top-left coordinates: A at (0,0), B at (200,100)
        graph.nodes = vec![
            make_node("A", 80.0, 40.0, 0.0, 0.0),    // left 0, right 80, top 0, bottom 40
            make_node("B", 80.0, 40.0, 200.0, 100.0), // left 200, right 280, top 100, bottom 140
        ];
        graph.edges = vec![make_edge("A", "B")];

        let cell_size = 10;
        let extent = calculate_grid_extent(&graph, cell_size);
        // bounding: width = 280, height = 140
        // density = 0.5, K = 1.5
        // offsets snapped to cell_size multiples, width/height snapped up
        assert!(extent.width >= 280.0, "width {} should cover bounding box", extent.width);
        assert!(extent.height >= 140.0, "height {} should cover bounding box", extent.height);
        assert_eq!(extent.offset_x % cell_size, 0, "offset_x should be cell_size-aligned");
        assert_eq!(extent.offset_y % cell_size, 0, "offset_y should be cell_size-aligned");
        // width and height should be multiples of cell_size
        assert_eq!((extent.width as i32) % cell_size, 0, "width should be cell_size-aligned");
    }

    #[test]
    fn test_grid_extent_empty() {
        let graph = Graph::new();
        let extent = calculate_grid_extent(&graph, 10);
        assert_eq!(extent.width, 100.0);
        assert_eq!(extent.height, 100.0);
    }
}
