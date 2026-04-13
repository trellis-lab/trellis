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

use crate::theme::Theme;
use crate::types::BoundingBox;

/// Corner radius for the boundary rectangle
const CORNER_R: f64 = 8.0;
/// Font size for the boundary name line
const FONT_SIZE_NAME: f64 = 12.0;
/// Font size for the boundary type line (e.g. "[Enterprise Boundary]")
const FONT_SIZE_TYPE: f64 = 11.0;
/// Height reserved for the two-line label strip at the bottom of the frame.
/// Must match `LABEL_HEIGHT` in `placement/c4.rs` → `compute_bboxes`.
const LABEL_STRIP_HEIGHT: f64 = 36.0;
/// Vertical distance between the two label baselines.
const LABEL_LINE_GAP: f64 = 16.0;

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
pub fn render_c4_boundaries(
    graph: &Graph,
    boxes: &HashMap<String, BoundingBox>,
    theme: &Theme,
) -> String {
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

        let (bg, stroke) = boundary_colours(boundary.c4_type, theme);

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

        // Two-line label at the bottom-left of the boundary frame:
        //   Line 1 (bold)  — boundary name
        //   Line 2 (italic) — boundary type, e.g. "[Enterprise Boundary]"
        let label_x = bbox.x + 10.0;
        let bottom = bbox.y + bbox.height;
        // Place lines inside the LABEL_STRIP_HEIGHT area reserved at the frame bottom.
        let line1_y = bottom - LABEL_STRIP_HEIGHT + LABEL_LINE_GAP; // = bottom − 20
        let line2_y = line1_y + LABEL_LINE_GAP - 2.0; // = bottom − 6

        svg.push_str(&format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" \
             font-family=\"Arial, Helvetica, sans-serif\" \
             font-size=\"{:.0}\" font-weight=\"bold\" fill=\"{}\">{}</text>\n",
            label_x,
            line1_y,
            FONT_SIZE_NAME,
            stroke,
            escape_xml(&boundary.label),
        ));

        let type_label = boundary_type_label(boundary.c4_type);
        svg.push_str(&format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" \
             font-family=\"Arial, Helvetica, sans-serif\" \
             font-size=\"{:.0}\" font-style=\"italic\" fill=\"{}\">{}</text>\n",
            label_x, line2_y, FONT_SIZE_TYPE, stroke, type_label,
        ));
    }

    svg
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Return the bracketed type label string for a boundary node.
fn boundary_type_label(c4_type: Option<C4NodeType>) -> &'static str {
    match c4_type {
        Some(C4NodeType::EnterpriseBoundary) => "[Enterprise Boundary]",
        Some(C4NodeType::SystemBoundary) => "[System Boundary]",
        Some(C4NodeType::ContainerBoundary) => "[Container Boundary]",
        Some(C4NodeType::DeploymentNode) => "[Deployment Node]",
        _ => "[Boundary]",
    }
}

fn boundary_colours<'t>(
    c4_type: Option<C4NodeType>,
    theme: &'t Theme,
) -> (&'t str, &'t str) {
    match c4_type {
        Some(C4NodeType::EnterpriseBoundary) => (
            theme.c4_boundary_enterprise_fill,
            theme.c4_boundary_enterprise_stroke,
        ),
        Some(C4NodeType::SystemBoundary) => (
            theme.c4_boundary_system_fill,
            theme.c4_boundary_system_stroke,
        ),
        Some(C4NodeType::ContainerBoundary) => (
            theme.c4_boundary_container_fill,
            theme.c4_boundary_container_stroke,
        ),
        Some(C4NodeType::DeploymentNode) => (
            theme.c4_boundary_deployment_fill,
            theme.c4_boundary_deployment_stroke,
        ),
        _ => (theme.c4_external_fill, theme.c4_external_stroke),
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
    use crate::theme::themes::DEFAULT;
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
        let svg = render_c4_boundaries(&graph, &boxes, &DEFAULT);
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
        let svg = render_c4_boundaries(&graph, &boxes, &DEFAULT);
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
        let svg = render_c4_boundaries(&graph, &boxes, &DEFAULT);
        // Should be empty because no contained elements were placed
        assert!(svg.is_empty(), "empty boundary should produce no SVG");
    }

    #[test]
    fn test_boundary_renders_type_label() {
        let mut graph = Graph::new();
        graph
            .nodes
            .push(make_boundary("eb", C4NodeType::EnterpriseBoundary));
        let mut boxes = HashMap::new();
        boxes.insert("eb".to_string(), make_bbox(0.0, 0.0, 200.0, 150.0));
        let svg = render_c4_boundaries(&graph, &boxes, &DEFAULT);
        // Both name and type label must appear
        assert!(svg.contains("eb"), "boundary name must be rendered");
        assert!(
            svg.contains("[Enterprise Boundary]"),
            "boundary type label must be rendered"
        );
        // Type line must be italic
        assert!(
            svg.contains("font-style=\"italic\""),
            "type label must use italic style"
        );
    }

    #[test]
    fn test_boundary_uses_correct_colours() {
        let mut graph = Graph::new();
        graph
            .nodes
            .push(make_boundary("cb", C4NodeType::ContainerBoundary));
        let mut boxes = HashMap::new();
        boxes.insert("cb".to_string(), make_bbox(0.0, 0.0, 100.0, 80.0));
        let svg = render_c4_boundaries(&graph, &boxes, &DEFAULT);
        assert!(svg.contains(DEFAULT.c4_boundary_container_fill));
        assert!(svg.contains(DEFAULT.c4_boundary_container_stroke));
    }
}
