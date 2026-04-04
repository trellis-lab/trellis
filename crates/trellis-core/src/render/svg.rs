use std::collections::HashMap;

use crate::config::TrellisConfig;
use crate::grid::Grid;
use crate::labels::LabelPlacement;
use crate::placement::subgraph::VIRTUAL_PREFIX;
use crate::render::c4_boundary::render_c4_boundaries;
use crate::render::c4_shapes::{c4_marker_defs, render_c4_node};
use crate::render::class_shapes::{class_marker_defs, render_class_node};
use crate::render::crossing::render_crossings;
use crate::render::edges::{arrow_marker_defs, render_edge, render_fallback_edge};
use crate::render::er_shapes::{er_marker_defs, render_er_node};
use crate::render::grid::render_grid_dot;
use crate::render::nodes::{render_node, render_nodes};
use crate::render::subgraph::render_subgraph_backgrounds;
use crate::routing::RoutingResult;
use crate::types::{BoundingBox, SubgraphTree};
use trellis_parser::{DiagramType, Graph, NodeShape};

/// Build the complete SVG document from graph, grid, routing data, and label placements.
pub fn build_svg(
    graph: &Graph,
    grid: &Grid,
    routing_result: &RoutingResult,
    config: &TrellisConfig,
    label_placements: &[LabelPlacement],
    subgraph_data: Option<&(SubgraphTree, HashMap<String, BoundingBox>)>,
) -> Vec<u8> {
    if graph.nodes.is_empty() {
        return b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"400\" height=\"300\">\
                 <rect width=\"400\" height=\"300\" fill=\"white\"/></svg>"
            .to_vec();
    }

    // Calculate viewBox from node positions, routed paths, and subgraph boxes
    let (vx, vy, vw, vh) = calculate_viewbox(graph, grid, routing_result, subgraph_data);

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
    svg.push_str(&format!(
        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"white\"/>\n",
        vx, vy, vw, vh
    ));

    // Show grid
    for i in 0..grid.rows {
        for j in 0..grid.cols {
            let dot_svg = render_grid_dot(i, j, grid);
            svg.push_str(&dot_svg);
        }
    }

    // Marker definitions (arrowheads + diagram-specific markers)
    svg.push_str(arrow_marker_defs());
    svg.push('\n');
    if graph.diagram_type == DiagramType::ClassDiagram {
        svg.push_str(class_marker_defs());
        svg.push('\n');
    }
    if graph.diagram_type == DiagramType::ErDiagram {
        svg.push_str(er_marker_defs());
        svg.push('\n');
    }
    if graph.diagram_type == DiagramType::C4Diagram {
        svg.push_str(c4_marker_defs());
        svg.push('\n');
    }

    // Z-order: 1. boundaries/subgraph backgrounds, 2. edges, 3. nodes, 4. edge labels

    // --- C4 boundary frames (below everything) ---
    // Bounding boxes come from the subgraph_data computed during placement,
    // so this branch only fires when boundaries are present.
    if graph.diagram_type == DiagramType::C4Diagram {
        if let Some((_, boxes)) = subgraph_data {
            let boundaries_svg = render_c4_boundaries(graph, boxes);
            if !boundaries_svg.is_empty() {
                svg.push_str("<!-- C4 Boundaries -->\n");
                svg.push_str("<g class=\"c4-boundaries\">\n");
                for line in boundaries_svg.lines() {
                    svg.push_str("  ");
                    svg.push_str(line);
                    svg.push('\n');
                }
                svg.push_str("</g>\n");
            }
        }
    } else {
        // --- Subgraph backgrounds (below everything) ---
        if let Some((tree, boxes)) = subgraph_data {
            let sg_svg = render_subgraph_backgrounds(tree, boxes);
            if !sg_svg.is_empty() {
                svg.push_str("<!-- Subgraph Backgrounds -->\n");
                svg.push_str("<g class=\"subgraphs\">\n");
                for line in sg_svg.lines() {
                    svg.push_str("  ");
                    svg.push_str(line);
                    svg.push('\n');
                }
                svg.push_str("</g>\n");
            }
        }
    }

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
    if config.render_crossings {
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
    }

    // --- Nodes (on top of edges) ---
    svg.push_str("<!-- Nodes -->\n");
    svg.push_str("<g class=\"nodes\">\n");
    // Filter out virtual subgraph nodes
    let visible_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| !n.id.starts_with(VIRTUAL_PREFIX))
        .cloned()
        .collect();

    if graph.diagram_type == DiagramType::ClassDiagram {
        // Class diagram: use dedicated three-compartment renderer for ClassBox nodes
        for node in &visible_nodes {
            let node_svg = if node.shape == NodeShape::ClassBox {
                render_class_node(node)
            } else {
                render_node(node)
            };
            for line in node_svg.lines() {
                svg.push_str("  ");
                svg.push_str(line);
                svg.push('\n');
            }
        }
    } else if graph.diagram_type == DiagramType::ErDiagram {
        // ER diagram: use dedicated entity box renderer for ErBox nodes
        for node in &visible_nodes {
            let node_svg = if node.shape == NodeShape::ErBox {
                render_er_node(node)
            } else {
                render_node(node)
            };
            for line in node_svg.lines() {
                svg.push_str("  ");
                svg.push_str(line);
                svg.push('\n');
            }
        }
    } else if graph.diagram_type == DiagramType::C4Diagram {
        // C4 diagram: use dedicated C4 element renderer; skip boundary nodes
        for node in &visible_nodes {
            let is_boundary = node.c4_type.map(|t| t.is_boundary()).unwrap_or(false);
            if is_boundary {
                continue; // drawn separately as boundary frames
            }
            let node_svg = render_c4_node(node);
            for line in node_svg.lines() {
                svg.push_str("  ");
                svg.push_str(line);
                svg.push('\n');
            }
        }
    } else {
        let nodes_svg = render_nodes(&visible_nodes);
        for line in nodes_svg.lines() {
            svg.push_str("  ");
            svg.push_str(line);
            svg.push('\n');
        }
    }
    svg.push_str("</g>\n");

    // --- Multiplicity labels for class diagram edges ---
    if graph.diagram_type == DiagramType::ClassDiagram {
        let has_mult = graph
            .edges
            .iter()
            .any(|e| e.source_multiplicity.is_some() || e.target_multiplicity.is_some());
        if has_mult {
            svg.push_str("<!-- Multiplicity Labels -->\n");
            svg.push_str("<g class=\"multiplicity-labels\">\n");
            for (edge_idx, path) in &routing_result.paths {
                if let Some(edge) = graph.edges.get(*edge_idx) {
                    render_multiplicity_labels(&mut svg, edge, &path.points, grid);
                }
            }
            svg.push_str("</g>\n");
        }
    }

    // --- Edge labels (on top of everything) ---
    if config.show_edge_labels && !label_placements.is_empty() {
        svg.push_str("<!-- Edge Labels -->\n");
        svg.push_str("<g class=\"edge-labels\">\n");
        for label in label_placements {
            svg.push_str("  ");
            svg.push_str(&render_label(label));
            svg.push('\n');
        }
        svg.push_str("</g>\n");
    }

    svg.push_str("</svg>");
    svg.into_bytes()
}

/// Render multiplicity labels near the endpoints of a class edge.
fn render_multiplicity_labels(
    svg: &mut String,
    edge: &trellis_parser::Edge,
    grid_points: &[crate::routing::astar::GridPoint],
    grid: &Grid,
) {
    if grid_points.len() < 2 {
        return;
    }

    let offset = 14.0_f64; // pixels from the endpoint

    // Source multiplicity — near the first point
    if let Some(ref mult) = edge.source_multiplicity {
        let p0 = &grid_points[0];
        let p1 = &grid_points[1];
        let (x0, y0) = grid.grid_to_world(p0.row as usize, p0.col as usize);
        let (x1, y1) = grid.grid_to_world(p1.row as usize, p1.col as usize);
        let (lx, ly) = label_offset_point(x0, y0, x1, y1, offset);
        svg.push_str(&format!(
            "  <text x=\"{:.1}\" y=\"{:.1}\" font-size=\"10\" fill=\"#555\">{}</text>\n",
            lx, ly, mult
        ));
    }

    // Target multiplicity — near the last point
    if let Some(ref mult) = edge.target_multiplicity {
        let n = grid_points.len();
        let pn = &grid_points[n - 1];
        let pn1 = &grid_points[n - 2];
        let (xn, yn) = grid.grid_to_world(pn.row as usize, pn.col as usize);
        let (xn1, yn1) = grid.grid_to_world(pn1.row as usize, pn1.col as usize);
        let (lx, ly) = label_offset_point(xn, yn, xn1, yn1, offset);
        svg.push_str(&format!(
            "  <text x=\"{:.1}\" y=\"{:.1}\" font-size=\"10\" fill=\"#555\">{}</text>\n",
            lx, ly, mult
        ));
    }
}

/// Calculate a label position offset from point A towards B by `offset` pixels,
/// then shifted perpendicularly.
fn label_offset_point(ax: f64, ay: f64, bx: f64, by_: f64, offset: f64) -> (f64, f64) {
    let dx = bx - ax;
    let dy = by_ - ay;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.001 {
        return (ax + offset, ay - offset);
    }
    let nx = dx / len;
    let ny = dy / len;
    // Move towards B by offset, then shift perpendicular
    (ax + nx * offset - ny * 6.0, ay + ny * offset + nx * 6.0)
}

/// Line height used when rendering multi-line edge labels.
/// Must match `LABEL_LINE_HEIGHT` in `labels/placement.rs` so sizing is consistent.
const LABEL_LINE_HEIGHT_PX: f64 = 14.0;

/// Render a single edge label as SVG: background rectangle + text.
///
/// `label.text` may contain `\n` for multi-line labels (e.g. C4 edges that
/// combine the relation label with a technology annotation).  Each line is
/// emitted as a `<tspan>` element so the text wraps correctly in SVG.
fn render_label(label: &LabelPlacement) -> String {
    let padding = 3.0;
    let bg_x = label.x;
    let bg_y = label.y;
    let bg_w = label.width + 2.0 * padding;
    let bg_h = label.height + 2.0 * padding;
    let text_x = label.x + label.width / 2.0 + padding;

    let lines: Vec<&str> = label.text.lines().collect();
    let line_count = lines.len().max(1);

    // Vertically centre the text block inside the background rect.
    let total_text_h = line_count as f64 * LABEL_LINE_HEIGHT_PX;
    let first_baseline_y =
        label.y + padding + (label.height - total_text_h) / 2.0 + LABEL_LINE_HEIGHT_PX * 0.8;

    let mut text_svg = String::from(
        "<text text-anchor=\"middle\" \
         font-family=\"Arial, Helvetica, sans-serif\" font-size=\"12\" fill=\"#333\">",
    );
    for (i, line) in lines.iter().enumerate() {
        if i == 0 {
            text_svg.push_str(&format!(
                "<tspan x=\"{:.1}\" y=\"{:.1}\">{}</tspan>",
                text_x,
                first_baseline_y,
                escape_xml(line)
            ));
        } else {
            text_svg.push_str(&format!(
                "<tspan x=\"{:.1}\" dy=\"{:.1}\">{}</tspan>",
                text_x,
                LABEL_LINE_HEIGHT_PX,
                escape_xml(line)
            ));
        }
    }
    text_svg.push_str("</text>");

    format!(
        "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" \
         fill=\"white\" stroke=\"#ccc\" stroke-width=\"0.5\" rx=\"2\"/>{}",
        bg_x, bg_y, bg_w, bg_h, text_svg
    )
}

/// Escape XML special characters in text.
fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Calculate the viewBox (origin x, origin y, width, height) from nodes, routed paths,
/// and subgraph bounding boxes.
fn calculate_viewbox(
    graph: &Graph,
    grid: &Grid,
    routing_result: &RoutingResult,
    subgraph_data: Option<&(SubgraphTree, HashMap<String, BoundingBox>)>,
) -> (f64, f64, f64, f64) {
    let padding = grid.cell_size as f64;

    // Start with node bounds
    let mut max_x = f64::NEG_INFINITY;
    let mut max_y = f64::NEG_INFINITY;
    let mut min_x = f64::INFINITY;
    let mut min_y = f64::INFINITY;

    for node in &graph.nodes {
        let left = node.x;
        let right = node.x + node.width;
        let top = node.y;
        let bottom = node.y + node.height;

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

    // Also consider subgraph bounding boxes
    if let Some((_tree, boxes)) = subgraph_data {
        for bbox in boxes.values() {
            min_x = min_x.min(bbox.x);
            max_x = max_x.max(bbox.x + bbox.width);
            min_y = min_y.min(bbox.y);
            max_y = max_y.max(bbox.y + bbox.height);
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
        let grid = Grid::new(1, 1, 10, 0, 0);
        let routing = RoutingResult {
            paths: std::collections::HashMap::new(),
            crossings: 0,
            total_bends: 0,
            failed_routes: 0,
            total_path_length: 0,
            total_routing_cost: 0.0,
            #[cfg(feature = "diagnostics")]
            max_path_length: 0,
            #[cfg(feature = "diagnostics")]
            max_bends_per_edge: 0,
            #[cfg(feature = "diagnostics")]
            sum_manhattan_distance: 0,
            deadlock_recoveries: 0,
        };
        let config = TrellisConfig::default();

        let svg = build_svg(&graph, &grid, &routing, &config, &[], None);
        let svg_str = String::from_utf8(svg).unwrap();
        assert!(svg_str.contains("<svg"));
        assert!(svg_str.contains("</svg>"));
    }
}
