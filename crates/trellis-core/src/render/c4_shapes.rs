//! C4 diagram shape renderers.
//!
//! Renders the five C4 element kinds:
//! - **Person / Person_Ext** – circle head + shoulder arc above a rounded box
//! - **SystemDb / ContainerDb / ComponentDb** – cylinder shape
//! - **SystemQueue / ContainerQueue / ComponentQueue** – double-frame rectangle
//! - **_Ext variants** – same as above but with dashed border
//! - **All others** – standard rounded rectangle
//!
//! The boundary renderer is in `c4_boundary.rs`.

use trellis_parser::{C4NodeType, Node};

// ── Layout constants ──────────────────────────────────────────────────────────

/// Font size for the element label
const FONT_SIZE_LABEL: f64 = 13.0;
/// Font size for the type descriptor ("Person", "[Database]", etc.)
const FONT_SIZE_TYPE: f64 = 11.0;
/// Font size for description/technology text
const FONT_SIZE_DESC: f64 = 10.0;
/// Line height for description rows
const LINE_HEIGHT: f64 = 14.0;
/// Extra vertical space for Person head above the box
const HEAD_RADIUS: f64 = 16.0;
/// Shoulder half-width
const SHOULDER_W: f64 = 28.0;
/// Ellipse cap height for cylinder
const CYLINDER_CAP: f64 = 14.0;
/// Corner radius for the main box
const BOX_RADIUS: f64 = 4.0;
/// Horizontal padding inside the box (must match PADDING_X in c4_diagram.rs)
const BOX_PADDING_X: f64 = 16.0;
/// Approximate character width for description/technology text at FONT_SIZE_DESC.
/// Must match RENDER_CHAR_WIDTH_DESC in c4_diagram.rs.
const CHAR_WIDTH_DESC: f64 = 5.5;

// ── Colours ───────────────────────────────────────────────────────────────────

const COLOUR_PERSON_FILL: &str = "#08427b";
const COLOUR_PERSON_TEXT: &str = "#ffffff";
const COLOUR_SYSTEM_FILL: &str = "#1168bd";
const COLOUR_SYSTEM_TEXT: &str = "#ffffff";
const COLOUR_CONTAINER_FILL: &str = "#438dd5";
const COLOUR_CONTAINER_TEXT: &str = "#ffffff";
const COLOUR_COMPONENT_FILL: &str = "#85bbf0";
const COLOUR_COMPONENT_TEXT: &str = "#000000";
const COLOUR_EXT_FILL: &str = "#999999";
const COLOUR_EXT_TEXT: &str = "#ffffff";
const COLOUR_DEPLOYMENT_FILL: &str = "#ffffff";
const COLOUR_DEPLOYMENT_TEXT: &str = "#000000";

// ── Public API ────────────────────────────────────────────────────────────────

/// Render a C4 element node as SVG.
pub fn render_c4_node(node: &Node) -> String {
    let c4_type = match node.c4_type {
        Some(t) => t,
        None => return render_c4_default_box(node),
    };

    if c4_type.is_person() {
        render_person(node, c4_type)
    } else if c4_type.is_db() {
        render_cylinder(node, c4_type)
    } else if c4_type.is_queue() {
        render_queue(node, c4_type)
    } else if c4_type.is_boundary() {
        // Boundaries are rendered separately in c4_boundary.rs
        String::new()
    } else {
        render_c4_box(node, c4_type)
    }
}

/// Generate SVG `<defs>` for C4 edge markers (regular open arrow).
///
/// C4 uses simple open arrows — we reuse the existing `arrowhead` marker
/// plus an optional bidirectional variant.
pub fn c4_marker_defs() -> &'static str {
    "<defs>\
     <marker id=\"c4-arrow\" viewBox=\"0 0 10 10\" refX=\"9\" refY=\"5\" \
     markerWidth=\"5\" markerHeight=\"5\" orient=\"auto-start-reverse\">\
     <path d=\"M 0 2 L 9 5 L 0 8\" fill=\"none\" stroke=\"#555\" stroke-width=\"1.5\"/>\
     </marker>\
     </defs>"
}

/// Return SVG marker attributes for a C4 edge.
///
/// `BiRel` edges get both start and end markers; regular `Rel` gets only end.
pub fn c4_edge_markers(edge: &trellis_parser::Edge) -> (String, String) {
    let end = " marker-end=\"url(#c4-arrow)\"".to_string();
    if edge.c4_bidirectional {
        let start = " marker-start=\"url(#c4-arrow)\"".to_string();
        (start, end)
    } else {
        (String::new(), end)
    }
}

// ── Shape renderers ───────────────────────────────────────────────────────────

/// Render a Person element: head circle + shoulder arc + label box.
fn render_person(node: &Node, c4_type: C4NodeType) -> String {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    let (fill, text_colour, stroke) = person_colours(c4_type);
    let dashed = if c4_type.is_external() {
        " stroke-dasharray=\"6,3\""
    } else {
        ""
    };

    // Head sits above the box
    let head_cx = x + w / 2.0;
    let head_cy = y + HEAD_RADIUS;

    // Shoulder arc below the head
    let shoulder_y = head_cy + HEAD_RADIUS + 4.0;
    let left_x = head_cx - SHOULDER_W;
    let right_x = head_cx + SHOULDER_W;

    // The box body starts below the shoulder
    let box_y = shoulder_y + 10.0;
    let box_h = h - (box_y - y);

    let mut svg = String::with_capacity(512);

    // Box body
    svg.push_str(&format!(
        "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" \
         fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"{} rx=\"{}\"/>\n",
        x, box_y, w, box_h, fill, stroke, dashed, BOX_RADIUS
    ));

    // Head circle
    svg.push_str(&format!(
        "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{:.1}\" \
         fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"{}/>\n",
        head_cx, head_cy, HEAD_RADIUS, fill, stroke, dashed
    ));

    // Shoulder arc (simplified as a line for cleanliness)
    svg.push_str(&format!(
        "<path d=\"M {:.1} {:.1} Q {:.1} {:.1} {:.1} {:.1}\" \
         fill=\"none\" stroke=\"{}\" stroke-width=\"1.5\"{}/>\n",
        left_x, box_y, head_cx, shoulder_y, right_x, box_y, stroke, dashed
    ));

    // Type label
    let type_label = if c4_type.is_external() {
        "[Person, External]"
    } else {
        "[Person]"
    };
    let type_y = box_y + 14.0;
    svg.push_str(&format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" \
         font-family=\"Arial, Helvetica, sans-serif\" \
         font-size=\"{:.0}\" font-style=\"italic\" fill=\"{}\">{}</text>\n",
        x + w / 2.0,
        type_y,
        FONT_SIZE_TYPE,
        text_colour,
        type_label
    ));

    // Name label (bold)
    let name_y = type_y + LINE_HEIGHT + 2.0;
    svg.push_str(&format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" \
         font-family=\"Arial, Helvetica, sans-serif\" \
         font-size=\"{:.0}\" font-weight=\"bold\" fill=\"{}\">{}</text>\n",
        x + w / 2.0,
        name_y,
        FONT_SIZE_LABEL,
        text_colour,
        escape_xml(&node.label)
    ));

    // Description (if present)
    if let Some(desc) = &node.c4_description {
        let desc_y = name_y + LINE_HEIGHT;
        svg.push_str(&format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" \
             font-family=\"Arial, Helvetica, sans-serif\" \
             font-size=\"{:.0}\" fill=\"{}\">{}</text>\n",
            x + w / 2.0,
            desc_y,
            FONT_SIZE_DESC,
            text_colour,
            escape_xml(desc)
        ));
    }

    svg
}

/// Render a Database (cylinder) element.
fn render_cylinder(node: &Node, c4_type: C4NodeType) -> String {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    let (fill, text_colour, stroke) = box_colours(c4_type);
    let dashed = if c4_type.is_external() {
        " stroke-dasharray=\"6,3\""
    } else {
        ""
    };
    let cx = x + w / 2.0;
    let rx = w / 2.0;
    let ry = CYLINDER_CAP / 2.0;

    let mut svg = String::with_capacity(512);

    // Cylinder body (rectangle)
    let rect_y = y + ry;
    let rect_h = h - ry;
    svg.push_str(&format!(
        "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" \
         fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"{}/>\n",
        x, rect_y, w, rect_h, fill, stroke, dashed
    ));

    // Bottom ellipse cap
    let bottom_cy = y + h - ry;
    svg.push_str(&format!(
        "<ellipse cx=\"{:.1}\" cy=\"{:.1}\" rx=\"{:.1}\" ry=\"{:.1}\" \
         fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"{}/>\n",
        cx, bottom_cy, rx, ry, fill, stroke, dashed
    ));

    // Top ellipse cap (drawn last so it overlaps body)
    let top_cy = y + ry;
    svg.push_str(&format!(
        "<ellipse cx=\"{:.1}\" cy=\"{:.1}\" rx=\"{:.1}\" ry=\"{:.1}\" \
         fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"{}/>\n",
        cx, top_cy, rx, ry, fill, stroke, dashed
    ));

    // Type label
    render_c4_labels(
        &mut svg,
        node,
        c4_type,
        x,
        y + ry * 2.0 + 6.0,
        w,
        text_colour,
    );

    svg
}

/// Render a Queue (double-frame) element.
fn render_queue(node: &Node, c4_type: C4NodeType) -> String {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    let (fill, text_colour, stroke) = box_colours(c4_type);
    let dashed = if c4_type.is_external() {
        " stroke-dasharray=\"6,3\""
    } else {
        ""
    };
    let offset = 6.0; // double-frame gap

    let mut svg = String::with_capacity(512);

    // Outer frame (back)
    svg.push_str(&format!(
        "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" \
         fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"{} rx=\"{}\"/>\n",
        x + offset,
        y + offset,
        w,
        h,
        fill,
        stroke,
        dashed,
        BOX_RADIUS
    ));
    // Inner frame (front)
    svg.push_str(&format!(
        "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" \
         fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"{} rx=\"{}\"/>\n",
        x, y, w, h, fill, stroke, dashed, BOX_RADIUS
    ));

    render_c4_labels(&mut svg, node, c4_type, x, y, w, text_colour);

    svg
}

/// Render a standard C4 rounded box element.
fn render_c4_box(node: &Node, c4_type: C4NodeType) -> String {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    let (fill, text_colour, stroke) = box_colours(c4_type);
    let dashed = if c4_type.is_external() {
        " stroke-dasharray=\"6,3\""
    } else {
        ""
    };

    let mut svg = String::with_capacity(512);

    svg.push_str(&format!(
        "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" \
         fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"{} rx=\"{}\"/>\n",
        x, y, w, h, fill, stroke, dashed, BOX_RADIUS
    ));

    render_c4_labels(&mut svg, node, c4_type, x, y, w, text_colour);

    svg
}

/// Fallback: plain box for unknown C4 type.
fn render_c4_default_box(node: &Node) -> String {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;
    let cx = x + w / 2.0;
    let cy = y + h / 2.0;

    format!(
        "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" \
         fill=\"#f5f5f5\" stroke=\"#999\" stroke-width=\"1.5\" rx=\"{}\"/>\n\
         <text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" dominant-baseline=\"central\" \
         font-family=\"Arial, Helvetica, sans-serif\" font-size=\"13\" fill=\"#333\">{}</text>\n",
        x,
        y,
        w,
        h,
        BOX_RADIUS,
        cx,
        cy,
        escape_xml(&node.label)
    )
}

// ── Label helpers ─────────────────────────────────────────────────────────────

/// Render type descriptor, name, technology, and description labels inside a C4 box.
fn render_c4_labels(
    svg: &mut String,
    node: &Node,
    c4_type: C4NodeType,
    box_x: f64,
    box_y: f64,
    box_w: f64,
    text_colour: &str,
) {
    let cx = box_x + box_w / 2.0;
    let mut text_y = box_y + 16.0;

    // Type descriptor (italic)
    let type_label = c4_type_label(c4_type);
    svg.push_str(&format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" \
         font-family=\"Arial, Helvetica, sans-serif\" \
         font-size=\"{:.0}\" font-style=\"italic\" fill=\"{}\">{}</text>\n",
        cx, text_y, FONT_SIZE_TYPE, text_colour, type_label
    ));
    text_y += LINE_HEIGHT + 2.0;

    // Element name (bold)
    svg.push_str(&format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" \
         font-family=\"Arial, Helvetica, sans-serif\" \
         font-size=\"{:.0}\" font-weight=\"bold\" fill=\"{}\">{}</text>\n",
        cx,
        text_y,
        FONT_SIZE_LABEL,
        text_colour,
        escape_xml(&node.label)
    ));
    text_y += LINE_HEIGHT + 1.0;

    // Technology (in brackets, smaller)
    if let Some(tech) = &node.c4_technology {
        svg.push_str(&format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" \
             font-family=\"Arial, Helvetica, sans-serif\" \
             font-size=\"{:.0}\" font-style=\"italic\" fill=\"{}\">[{}]</text>\n",
            cx,
            text_y,
            FONT_SIZE_DESC,
            text_colour,
            escape_xml(tech)
        ));
        text_y += LINE_HEIGHT;
    }

    // Description (regular, smaller) — word-wrapped to fit inside the box.
    if let Some(desc) = &node.c4_description {
        for line in wrap_text(desc, box_w) {
            svg.push_str(&format!(
                "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" \
                 font-family=\"Arial, Helvetica, sans-serif\" \
                 font-size=\"{:.0}\" fill=\"{}\">{}</text>\n",
                cx,
                text_y,
                FONT_SIZE_DESC,
                text_colour,
                escape_xml(&line)
            ));
            text_y += LINE_HEIGHT;
        }
    }
}

/// Return the bracketed type label string for a C4 node.
fn c4_type_label(t: C4NodeType) -> &'static str {
    match t {
        C4NodeType::Person => "[Person]",
        C4NodeType::PersonExt => "[Person, External]",
        C4NodeType::System | C4NodeType::SystemExt => "[Software System]",
        C4NodeType::SystemDb | C4NodeType::SystemDbExt => "[Software System, DB]",
        C4NodeType::SystemQueue | C4NodeType::SystemQueueExt => "[Software System, Queue]",
        C4NodeType::Container | C4NodeType::ContainerExt => "[Container]",
        C4NodeType::ContainerDb | C4NodeType::ContainerDbExt => "[Container, DB]",
        C4NodeType::ContainerQueue | C4NodeType::ContainerQueueExt => "[Container, Queue]",
        C4NodeType::Component | C4NodeType::ComponentExt => "[Component]",
        C4NodeType::ComponentDb | C4NodeType::ComponentDbExt => "[Component, DB]",
        C4NodeType::ComponentQueue | C4NodeType::ComponentQueueExt => "[Component, Queue]",
        C4NodeType::EnterpriseBoundary => "[Enterprise Boundary]",
        C4NodeType::SystemBoundary => "[System Boundary]",
        C4NodeType::ContainerBoundary => "[Container Boundary]",
        C4NodeType::DeploymentNode => "[Deployment Node]",
    }
}

// ── Colour helpers ────────────────────────────────────────────────────────────

/// Return (fill, text_colour, stroke) for Person elements.
fn person_colours(c4_type: C4NodeType) -> (&'static str, &'static str, &'static str) {
    if c4_type.is_external() {
        (COLOUR_EXT_FILL, COLOUR_EXT_TEXT, COLOUR_EXT_FILL)
    } else {
        (COLOUR_PERSON_FILL, COLOUR_PERSON_TEXT, COLOUR_PERSON_FILL)
    }
}

/// Return (fill, text_colour, stroke) for box/cylinder/queue elements.
fn box_colours(c4_type: C4NodeType) -> (&'static str, &'static str, &'static str) {
    if c4_type.is_external() {
        return (COLOUR_EXT_FILL, COLOUR_EXT_TEXT, COLOUR_EXT_FILL);
    }
    match c4_type {
        C4NodeType::System
        | C4NodeType::SystemDb
        | C4NodeType::SystemQueue
        | C4NodeType::SystemExt
        | C4NodeType::SystemDbExt
        | C4NodeType::SystemQueueExt => {
            (COLOUR_SYSTEM_FILL, COLOUR_SYSTEM_TEXT, COLOUR_SYSTEM_FILL)
        }

        C4NodeType::Container
        | C4NodeType::ContainerDb
        | C4NodeType::ContainerQueue
        | C4NodeType::ContainerExt
        | C4NodeType::ContainerDbExt
        | C4NodeType::ContainerQueueExt => (
            COLOUR_CONTAINER_FILL,
            COLOUR_CONTAINER_TEXT,
            COLOUR_CONTAINER_FILL,
        ),

        C4NodeType::Component
        | C4NodeType::ComponentDb
        | C4NodeType::ComponentQueue
        | C4NodeType::ComponentExt
        | C4NodeType::ComponentDbExt
        | C4NodeType::ComponentQueueExt => (
            COLOUR_COMPONENT_FILL,
            COLOUR_COMPONENT_TEXT,
            COLOUR_COMPONENT_FILL,
        ),

        C4NodeType::DeploymentNode => (COLOUR_DEPLOYMENT_FILL, COLOUR_DEPLOYMENT_TEXT, "#555555"),

        // Boundaries handled separately
        _ => ("#f5f5f5", "#333333", "#999999"),
    }
}

// ── Utility ───────────────────────────────────────────────────────────────────

/// Word-wrap `text` so each line fits within `box_w` pixels at FONT_SIZE_DESC.
///
/// Uses the same algorithm as `word_wrap_line_count` in the parser so the
/// renderer produces exactly as many lines as the parser reserved height for.
fn wrap_text(text: &str, box_w: f64) -> Vec<String> {
    let inner_w = (box_w - BOX_PADDING_X * 2.0).max(40.0);
    let chars_per_line = (inner_w / CHAR_WIDTH_DESC).max(5.0) as usize;

    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut current_len = 0usize;

    for word in text.split_whitespace() {
        if current_len == 0 {
            current.push_str(word);
            current_len = word.len();
        } else if current_len + 1 + word.len() <= chars_per_line {
            current.push(' ');
            current.push_str(word);
            current_len += 1 + word.len();
        } else {
            lines.push(current.clone());
            current = word.to_string();
            current_len = word.len();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_parser::{C4NodeType, Node, NodeShape};

    fn make_c4_node(id: &str, c4_type: C4NodeType) -> Node {
        Node {
            id: id.to_string(),
            label: id.to_string(),
            shape: NodeShape::C4Box,
            x: 0.0,
            y: 0.0,
            width: 150.0,
            height: 80.0,
            c4_type: Some(c4_type),
            ..Default::default()
        }
    }

    #[test]
    fn test_render_person() {
        let node = make_c4_node("alice", C4NodeType::Person);
        let svg = render_c4_node(&node);
        assert!(svg.contains("<circle"));
        assert!(svg.contains("[Person]"));
        assert!(svg.contains("alice"));
    }

    #[test]
    fn test_render_system() {
        let node = make_c4_node("sys", C4NodeType::System);
        let svg = render_c4_node(&node);
        assert!(svg.contains("<rect"));
        assert!(svg.contains("[Software System]"));
    }

    #[test]
    fn test_render_system_db() {
        let node = make_c4_node("db", C4NodeType::SystemDb);
        let svg = render_c4_node(&node);
        assert!(svg.contains("<ellipse"));
        assert!(svg.contains("[Software System, DB]"));
    }

    #[test]
    fn test_render_container_queue() {
        let node = make_c4_node("q", C4NodeType::ContainerQueue);
        let svg = render_c4_node(&node);
        // Queue renders two rects
        assert_eq!(svg.matches("<rect").count(), 2);
        assert!(svg.contains("[Container, Queue]"));
    }

    #[test]
    fn test_render_ext_uses_dashed() {
        let node = make_c4_node("ext", C4NodeType::SystemExt);
        let svg = render_c4_node(&node);
        assert!(svg.contains("stroke-dasharray"));
    }

    #[test]
    fn test_c4_marker_defs() {
        let defs = c4_marker_defs();
        assert!(defs.contains("c4-arrow"));
    }

    #[test]
    fn test_c4_edge_markers_birel() {
        let edge = trellis_parser::Edge {
            c4_bidirectional: true,
            ..Default::default()
        };
        let (start, end) = c4_edge_markers(&edge);
        assert!(start.contains("marker-start"));
        assert!(end.contains("marker-end"));
    }

    #[test]
    fn test_c4_edge_markers_unidirectional() {
        let edge = trellis_parser::Edge {
            c4_bidirectional: false,
            ..Default::default()
        };
        let (start, end) = c4_edge_markers(&edge);
        assert!(start.is_empty());
        assert!(end.contains("marker-end"));
    }
}
