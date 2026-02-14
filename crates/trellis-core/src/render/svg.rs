use crate::config::TrellisConfig;
use crate::grid::Grid;
use crate::render::crossing::render_crossings;
use crate::render::edges::{arrow_marker_defs, render_edge, render_fallback_edge};
use crate::render::grid::render_grid_dot;
use crate::render::nodes::render_nodes;
use crate::routing::RoutingResult;
use trellis_parser::Graph;

/// Build the complete SVG document from graph, grid, and routing data.
pub fn build_svg(
    graph: &Graph,
    grid: &Grid,
    routing_result: &RoutingResult,
    config: &TrellisConfig,
) -> Vec<u8> {
    if graph.nodes.is_empty() {
        return b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"400\" height=\"300\">\
                 <rect width=\"400\" height=\"300\" fill=\"white\"/></svg>"
            .to_vec();
    }

    // Calculate viewBox from node positions and routed paths
    let (vx, vy, vw, vh) = calculate_viewbox(graph, grid, routing_result);

    let mut svg = String::with_capacity(4096);

    // SVG header
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" \
         width=\"{:.0}\" height=\"{:.0}\" \
         viewBox=\"{:.1} {:.1} {:.1} {:.1}\">\n",
        vw, vh, vx, vy, vw, vh,
    ));

    // Style block
    svg.push_str(
        "<style>\n\
        text { user-select: none; }\n\
    </style>\n",
    );

    // Background
    svg.push_str("<rect width=\"100%\" height=\"100%\" fill=\"white\"/>\n");

    // Show grid
    for i in 0..grid.rows {
        for j in 0..grid.cols {
            let dot_svg = render_grid_dot(i, j, grid);
            svg.push_str(&dot_svg);
        }
    }

    // Marker definitions (arrowheads)
    svg.push_str(arrow_marker_defs());
    svg.push('\n');

    // Z-order: 1. subgraph backgrounds (future M9), 2. edges, 3. nodes, 4. labels (future M8)

    // --- Edges ---
    svg.push_str("<!-- Edges -->\n");
    svg.push_str("<g class=\"edges\">\n");

    // Routed edges
    for (edge_idx, path) in &routing_result.paths {
        if let Some(edge) = graph.edges.get(*edge_idx) {
            let edge_svg = render_edge(edge, &path.points, grid, config.corner_radius);
            svg.push_str("  ");
            svg.push_str(&edge_svg);
            svg.push('\n');
        }
    }

    // Fallback edges (straight lines for unrouted edges)
    for (edge_idx, edge) in graph.edges.iter().enumerate() {
        if routing_result.paths.contains_key(&edge_idx) {
            continue;
        }
        let from_node = graph.nodes.iter().find(|n| n.id == edge.from);
        let to_node = graph.nodes.iter().find(|n| n.id == edge.to);
        if let (Some(f), Some(t)) = (from_node, to_node) {
            let edge_svg = render_fallback_edge(edge, f.x, f.y, t.x, t.y);
            svg.push_str("  ");
            svg.push_str(&edge_svg);
            svg.push('\n');
        }
    }

    svg.push_str("</g>\n");

    // --- Crossing indicators (on top of edges, under nodes) ---
    let crossings_svg = render_crossings(grid);
    if !crossings_svg.is_empty() {
        svg.push_str("<!-- Crossings -->\n");
        svg.push_str("<g class=\"crossings\">\n");
        for line in crossings_svg.lines() {
            svg.push_str("  ");
            svg.push_str(line);
            svg.push('\n');
        }
        svg.push_str("</g>\n");
    }

    // --- Nodes (on top of edges) ---
    svg.push_str("<!-- Nodes -->\n");
    svg.push_str("<g class=\"nodes\">\n");
    let nodes_svg = render_nodes(&graph.nodes);
    for line in nodes_svg.lines() {
        svg.push_str("  ");
        svg.push_str(line);
        svg.push('\n');
    }
    svg.push_str("</g>\n");

    svg.push_str("</svg>");
    svg.into_bytes()
}

/// Calculate the viewBox (origin x, origin y, width, height) from nodes and routed paths.
fn calculate_viewbox(
    graph: &Graph,
    grid: &Grid,
    routing_result: &RoutingResult,
) -> (f64, f64, f64, f64) {
    let padding = 40.0;

    // Start with node bounds
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;

    for node in &graph.nodes {
        let left = node.x - node.width / 2.0;
        let right = node.x + node.width / 2.0;
        let top = node.y - node.height / 2.0;
        let bottom = node.y + node.height / 2.0;

        min_x = min_x.min(left);
        max_x = max_x.max(right);
        min_y = min_y.min(top);
        max_y = max_y.max(bottom);
    }

    // Also consider routed edge paths
    for path in routing_result.paths.values() {
        for point in &path.points {
            let (x, y) = grid.grid_to_world(point.row as usize, point.col as usize);
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
    }

    let origin_x = min_x - padding;
    let origin_y = min_y - padding;
    let width = (max_x - min_x + 2.0 * padding).ceil();
    let height = (max_y - min_y + 2.0 * padding).ceil();

    // Ensure minimum dimensions
    (origin_x, origin_y, width.max(200.0), height.max(150.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph_svg() {
        let graph = Graph::new();
        let grid = Grid::new(1, 1, 10.0, 0.0, 0.0);
        let routing = RoutingResult {
            paths: std::collections::HashMap::new(),
            crossings: 0,
            total_bends: 0,
            failed_routes: 0,
        };
        let config = TrellisConfig::default();

        let svg = build_svg(&graph, &grid, &routing, &config);
        let svg_str = String::from_utf8(svg).unwrap();
        assert!(svg_str.contains("<svg"));
        assert!(svg_str.contains("</svg>"));
    }
}
