//! C4 boundary frame renderer.
//!
//! Boundaries (Enterprise_Boundary, System_Boundary, Container_Boundary)
//! are rendered as dashed rectangles enclosing the elements placed inside
//! them.
//!
//! The bounding boxes are computed in `placement::c4::compute_c4_subgraph_data`
//! and passed in as a `HashMap<String, BoundingBox>` so that placement and
//! rendering share a single source of truth.

use std::collections::HashMap;
use trellis_parser::{C4NodeType, Graph, Node};

use crate::types::BoundingBox;

/// Corner radius for the boundary rectangle
const CORNER_R: f64 = 8.0;
/// Font size for the boundary label
const FONT_SIZE: f64 = 12.0;
/// Vertical offset to centre the label text inside the label strip
const LABEL_HEIGHT: f64 = 20.0;

// ── Colours by boundary type ──────────────────────────────────────────────────
const COLOUR_ENTERPRISE_BG: &str = "#ffffff";
const COLOUR_ENTERPRISE_STROKE: &str = "#f9a825";
const COLOUR_SYSTEM_BG: &str = "#ffffff";
const COLOUR_SYSTEM_STROKE: &str = "#388e3c";
const COLOUR_CONTAINER_BG: &str = "#ffffff";
const COLOUR_CONTAINER_STROKE: &str = "#1565c0";
const COLOUR_DEPLOYMENT_BG: &str = "#ffffff";
const COLOUR_DEPLOYMENT_STROKE: &str = "#616161";

const SVG_BOUNDARY_STROKE_WIDTH: f64 = 2.0;

// ── Public API ────────────────────────────────────────────────────────────────

/// Render all C4 boundary frames as SVG.
///
/// `boxes` contains the pre-computed bounding boxes (from
/// `placement::c4::compute_c4_subgraph_data`) keyed by boundary node ID.
///
/// Boundaries that have no entry in `boxes` (i.e. no contained elements
/// were placed) are silently skipped.  Returns an empty string when there
/// are no boundaries in the graph.
pub fn render_c4_boundaries(graph: &Graph, boxes: &HashMap<String, BoundingBox>) -> String {
    let boundaries: Vec<&Node> = graph
        .nodes
        .iter()
        .filter(|n| n.c4_type.map(|t| t.is_boundary()).unwrap_or(false))
        .collect();

    if boundaries.is_empty() {
        return String::new();
    }

    let mut svg = String::with_capacity(256 * boundaries.len());

    for boundary in &boundaries {
        let bbox = match boxes.get(&boundary.id) {
            Some(b) => b,
            None => continue, // boundary has no contained elements
        };

        let (bg, stroke) = boundary_colours(boundary.c4_type);

        // Background / border rectangle
        svg.push_str(&format!(
            "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" \
             fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\" \
             rx=\"{}\"/>\n",
            bbox.x,
            bbox.y,
            bbox.width,
            bbox.height,
            bg,
            stroke,
            SVG_BOUNDARY_STROKE_WIDTH,
            CORNER_R
        ));

        // Label in the top-left corner of the boundary strip
        let label_x = bbox.x + 10.0;
        let label_y = bbox.y + bbox.height - LABEL_HEIGHT / 2.0;
        svg.push_str(&format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" \
             font-family=\"Arial, Helvetica, sans-serif\" \
             font-size=\"{:.0}\" font-weight=\"bold\" fill=\"{}\">{}</text>\n",
            label_x,
            label_y,
            FONT_SIZE,
            stroke,
            escape_xml(&boundary.label),
        ));
    }

    svg
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn boundary_colours(c4_type: Option<C4NodeType>) -> (&'static str, &'static str) {
    match c4_type {
        Some(C4NodeType::EnterpriseBoundary) => (COLOUR_ENTERPRISE_BG, COLOUR_ENTERPRISE_STROKE),
        Some(C4NodeType::SystemBoundary) => (COLOUR_SYSTEM_BG, COLOUR_SYSTEM_STROKE),
        Some(C4NodeType::ContainerBoundary) => (COLOUR_CONTAINER_BG, COLOUR_CONTAINER_STROKE),
        Some(C4NodeType::DeploymentNode) => (COLOUR_DEPLOYMENT_BG, COLOUR_DEPLOYMENT_STROKE),
        _ => ("#ffffff", "#999999"),
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_parser::{C4NodeType, Node, NodeShape};

    fn make_boundary(id: &str, c4_type: C4NodeType) -> Node {
        Node {
            id: id.to_string(),
            label: id.to_string(),
            shape: NodeShape::C4Box,
            c4_type: Some(c4_type),
            ..Default::default()
        }
    }

    fn make_bbox(x: f64, y: f64, w: f64, h: f64) -> BoundingBox {
        BoundingBox {
            x,
            y,
            width: w,
            height: h,
        }
    }

    #[test]
    fn test_no_boundaries_returns_empty() {
        let mut graph = Graph::new();
        graph.nodes.push(Node {
            id: "sys".to_string(),
            c4_type: Some(C4NodeType::System),
            ..Default::default()
        });
        let boxes = HashMap::new();
        let svg = render_c4_boundaries(&graph, &boxes);
        assert!(svg.is_empty());
    }

    #[test]
    fn test_boundary_rendered() {
        let mut graph = Graph::new();
        graph
            .nodes
            .push(make_boundary("eb", C4NodeType::EnterpriseBoundary));
        let mut boxes = HashMap::new();
        boxes.insert("eb".to_string(), make_bbox(16.0, -4.0, 200.0, 150.0));
        let svg = render_c4_boundaries(&graph, &boxes);
        assert!(svg.contains("<rect"), "should contain a rect element");
        assert!(svg.contains("eb"), "should contain the boundary label");
    }

    #[test]
    fn test_boundary_skipped_when_no_bbox() {
        let mut graph = Graph::new();
        graph
            .nodes
            .push(make_boundary("sb", C4NodeType::SystemBoundary));
        let boxes = HashMap::new(); // no entry for "sb"
        let svg = render_c4_boundaries(&graph, &boxes);
        // Should be empty because no contained elements were placed
        assert!(svg.is_empty(), "empty boundary should produce no SVG");
    }

    #[test]
    fn test_boundary_uses_correct_colours() {
        let mut graph = Graph::new();
        graph
            .nodes
            .push(make_boundary("cb", C4NodeType::ContainerBoundary));
        let mut boxes = HashMap::new();
        boxes.insert("cb".to_string(), make_bbox(0.0, 0.0, 100.0, 80.0));
        let svg = render_c4_boundaries(&graph, &boxes);
        assert!(svg.contains(COLOUR_CONTAINER_BG));
        assert!(svg.contains(COLOUR_CONTAINER_STROKE));
    }
}
