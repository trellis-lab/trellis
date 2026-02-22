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
/// Ratio of the cylinder ellipse's vertical radius to its horizontal radius.
/// Derived from the C4 v4 reference SVG (ry=22.2, rx=225.5 → ≈ 0.0985).
/// Keeping this ratio constant satisfies "the ellipse curve is always the same".
const CYLINDER_RY_RATIO: f64 = 22.2 / 225.5;
/// Cubic-bezier kappa for a quarter-ellipse approximation (4/3 · tan(π/8) ≈ 0.5523).
const KAPPA: f64 = 0.5523;
/// Corner radius for the main box
const BOX_RADIUS: f64 = 4.0;
/// Horizontal padding inside the box (must match PADDING_X in c4_diagram.rs)
const BOX_PADDING_X: f64 = 16.0;
/// Approximate character width for description/technology text at FONT_SIZE_DESC.
/// Must match RENDER_CHAR_WIDTH_DESC in c4_diagram.rs.
const CHAR_WIDTH_DESC: f64 = 5.5;
/// Maximum lines per text block for Person nodes.
/// Spec: "No text can have more than 5 lines."
/// Must match `PERSON_MAX_LINES` in `c4_diagram.rs`.
const MAX_TEXT_LINES: usize = 5;

// ── Colours ───────────────────────────────────────────────────────────────────

/// Stroke / text colour for Person elements (C4 v4: white fill, coloured stroke).
const COLOUR_PERSON_STROKE: &str = "#08427b";
const COLOUR_EXT_STROKE: &str = "#999999";
/// Stroke / text colour for Database elements (C4 v4: white fill, coloured stroke).
const COLOUR_DB_STROKE: &str = "#438dd5";
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

// ── Stroke settings────────────────────────────────────────────────────────────

const SVG_STROKE_WIDTH: f64 = 4.0;
const SVG_STROKE_WIDTH_NARROW: f64 = 1.0;

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

/// Render a Person element per C4 v4 standard:
/// - White fill with `#08427b` stroke/text
/// - Head ellipse with diameter = width/2, overlapping the box top by 10 px
/// - Rounded rectangle body (corner radius = box_h / 3)
/// - Two vertical leg lines at the bottom
/// - Text order: Caption (bold, +2), Type (italic, −1), Description (default)
fn render_person(node: &Node, c4_type: C4NodeType) -> String {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    let stroke = if c4_type.is_external() {
        COLOUR_EXT_STROKE
    } else {
        COLOUR_PERSON_STROKE
    }; // #08427b
    let fill = "#ffffff";
    let text_colour = stroke;

    let dashed = if c4_type.is_external() {
        " stroke-dasharray=\"6,3\""
    } else {
        ""
    };

    // Head: diameter = 80% of width/2 → radius = width/4
    let head_r = w * 0.8 / 4.0;
    let head_cx = x + w / 2.0;
    // Head top is at node top (y); center is one radius below.
    let head_cy = y + head_r;

    // Box starts where the head overlaps it by 10 px:
    //   head bottom = head_cy + head_r = box_y + 10  →  box_y = y + 2*head_r − 10
    let box_y = y + 2.0 * head_r - 10.0;
    let box_h = h - (box_y - y);

    // Rounded corners proportional to box height (matches the spec example).
    let person_rx = (box_h / 3.0).min(w / 2.0);

    // Leg lines: at 20 % and 80 % of width, from ~44 % down the box to the bottom.
    let leg_left_x = x + w * 0.2;
    let leg_right_x = x + w * 0.8;
    let leg_y_top = box_y + box_h * 0.44;
    let leg_y_bottom = box_y + box_h;

    let mut svg = String::with_capacity(512);

    // 1. Rounded rectangle body
    svg.push_str(&format!(
        "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" \
         fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\"{} rx=\"{:.1}\"/>\n",
        x, box_y, w, box_h, fill, stroke, SVG_STROKE_WIDTH, dashed, person_rx
    ));

    // 2. Leg lines (drawn over the rectangle border)
    svg.push_str(&format!(
        "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" \
         stroke=\"{}\" opacity=\"0.3\" stroke-width=\"{}\"{}/>\n",
        leg_left_x, leg_y_bottom, leg_left_x, leg_y_top, stroke, SVG_STROKE_WIDTH_NARROW, dashed
    ));
    svg.push_str(&format!(
        "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" \
         stroke=\"{}\" opacity=\"0.3\" stroke-width=\"{}\"{}/>\n",
        leg_right_x, leg_y_bottom, leg_right_x, leg_y_top, stroke, SVG_STROKE_WIDTH_NARROW, dashed
    ));

    // 3. Head ellipse (drawn last so it sits on top of the box border)
    svg.push_str(&format!(
        "<ellipse cx=\"{:.1}\" cy=\"{:.1}\" rx=\"{:.1}\" ry=\"{:.1}\" \
         fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\"{}/>\n",
        head_cx, head_cy, head_r, head_r, fill, stroke, SVG_STROKE_WIDTH, dashed
    ));

    // ── Text labels ──────────────────────────────────────────────────────────
    // Order (C4 v4 generic spec): Caption → Type → Description, with 4 px gaps.
    // 10 px visual gap above the first line and below the last line.
    // The bottom gap is enforced by the placement code sizing the box height;
    // the top gap is achieved by computing the baseline from the cap-height:
    //   text_y = box_y + 10 + caption_size * 0.75  (cap-height ≈ 75 % of font size)
    let cx = x + w / 2.0;
    // Caption (name) — bold, default + 2
    let caption_size = FONT_SIZE_LABEL + 2.0;
    // Baseline positioned so the visual top of the caption is 10 px below box_y.
    let mut text_y = box_y + 20.0 + caption_size * 0.75;
    for line in wrap_text_truncated(&node.label, w) {
        svg.push_str(&format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" \
             font-family=\"Arial, Helvetica, sans-serif\" \
             font-size=\"{:.0}\" font-weight=\"bold\" fill=\"{}\">{}</text>\n",
            cx,
            text_y,
            caption_size,
            text_colour,
            escape_xml(&line)
        ));
        text_y += LINE_HEIGHT;
    }
    text_y += 4.0; // 4 px gap

    // Type descriptor — italic, default − 1
    let type_label = if c4_type.is_external() {
        "[Person, External]"
    } else {
        "[Person]"
    };
    let type_size = FONT_SIZE_LABEL - 1.0;
    svg.push_str(&format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" \
         font-family=\"Arial, Helvetica, sans-serif\" \
         font-size=\"{:.0}\" font-style=\"italic\" fill=\"{}\">{}</text>\n",
        cx, text_y, type_size, text_colour, type_label
    ));
    text_y += LINE_HEIGHT + 4.0; // 4 px gap

    // Description — default size, word-wrapped and truncated to MAX_TEXT_LINES
    if let Some(desc) = &node.c4_description {
        for line in wrap_text_truncated(desc, w) {
            svg.push_str(&format!(
                "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" \
                 font-family=\"Arial, Helvetica, sans-serif\" \
                 font-size=\"{:.0}\" fill=\"{}\">{}</text>\n",
                cx,
                text_y,
                FONT_SIZE_LABEL,
                text_colour,
                escape_xml(&line)
            ));
            text_y += LINE_HEIGHT;
        }
    }

    svg
}

/// Render a Database (cylinder) element per C4 v4:
/// - White fill with `#438dd5` stroke and text (all DB variants)
/// - Two SVG `<path>` elements matching the C4 v4 reference SVG shape:
///     1. **Body** – closed outline: top half-ellipse arc + straight sides + bottom half-ellipse arc
///     2. **Lid**  – bottom arc of the top ellipse (visible interior rim)
/// - Ellipse ry = rx × CYLINDER_RY_RATIO, so "the ellipse curve is always the same"
/// - Top padding = ry*2 + 10 px (spec: top padding = cap height + 10 px)
fn render_cylinder(node: &Node, c4_type: C4NodeType) -> String {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    // C4 v4: white fill, #438dd5 stroke and text for all database shapes.
    let fill = "#ffffff";
    let stroke = COLOUR_DB_STROKE;
    let text_colour = COLOUR_DB_STROKE;

    let dashed = if c4_type.is_external() {
        " stroke-dasharray=\"6,3\""
    } else {
        ""
    };

    // Geometry: rx proportional to width; ry keeps a fixed ratio (always the same curve).
    let cx = x + w / 2.0;
    let rx = w / 2.0;
    let ry = rx * CYLINDER_RY_RATIO;
    let x1 = x + w; // right edge
    let yh = y + h; // bottom edge
    let top_cy = y + ry; // centre of top ellipse
    let bot_cy = yh - ry; // centre of bottom ellipse

    // Pre-compute bezier control-point offsets (kappa × radius).
    let rxk = rx * KAPPA;
    let ryk = ry * KAPPA;

    let mut svg = String::with_capacity(768);

    // ── Path 1: Body (closed outline) ────────────────────────────────────────
    //
    //  Start at left of top ellipse → quarter-arc to top-centre →
    //  quarter-arc to right of top ellipse → straight down right side →
    //  quarter-arc to bottom-centre → quarter-arc to left of bottom ellipse → close.
    //
    //  Matches the body <path> in the C4 v4 reference SVG (two kappa-based
    //  cubic beziers per half-ellipse).
    svg.push_str(&format!(
        "<path d=\"\
         M {:.1} {:.1} \
         C {:.1} {:.1} {:.1} {:.1} {:.1} {:.1} \
         C {:.1} {:.1} {:.1} {:.1} {:.1} {:.1} \
         L {:.1} {:.1} \
         C {:.1} {:.1} {:.1} {:.1} {:.1} {:.1} \
         C {:.1} {:.1} {:.1} {:.1} {:.1} {:.1} Z\" \
         fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\"{}/>\n",
        // Start: left of top ellipse
        x,
        top_cy,
        // Q1: left → top-centre  (kappa above centre)
        x,
        top_cy - ryk,
        cx - rxk,
        y,
        cx,
        y,
        // Q2: top-centre → right  (kappa above centre)
        cx + rxk,
        y,
        x1,
        top_cy - ryk,
        x1,
        top_cy,
        // Straight down right side
        x1,
        bot_cy,
        // Q3: right → bottom-centre  (kappa below centre)
        x1,
        bot_cy + ryk,
        cx + rxk,
        yh,
        cx,
        yh,
        // Q4: bottom-centre → left  (kappa below centre)
        cx - rxk,
        yh,
        x,
        bot_cy + ryk,
        x,
        bot_cy,
        fill,
        stroke,
        SVG_STROKE_WIDTH,
        dashed,
    ));

    // ── Path 2: Lid (bottom arc of top ellipse — visible interior rim) ────────
    //
    //  Starts at right of top ellipse → arc to bottom-centre → arc to left.
    //  fill="none" so only the stroke is visible (matches the reference SVG).
    let y2ry = y + 2.0 * ry; // bottom of the top ellipse
    svg.push_str(&format!(
        "<path d=\"\
         M {:.1} {:.1} \
         C {:.1} {:.1} {:.1} {:.1} {:.1} {:.1} \
         C {:.1} {:.1} {:.1} {:.1} {:.1} {:.1}\" \
         fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"{}/>\n",
        // Start: right of top ellipse
        x1,
        top_cy,
        // Arc right → bottom-centre  (kappa below centre)
        x1,
        top_cy + ryk,
        cx + rxk,
        y2ry,
        cx,
        y2ry,
        // Arc bottom-centre → left  (kappa below centre)
        cx - rxk,
        y2ry,
        x,
        top_cy + ryk,
        x,
        top_cy,
        stroke,
        SVG_STROKE_WIDTH,
        dashed,
    ));

    // Text starts below the top cap with a 10 px gap (spec: top padding = cap height + 10 px).
    render_c4_labels(&mut svg, node, c4_type, x, y + ry * 2.0, w, text_colour);

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
    let mut svg = String::with_capacity(512);

    svg.push_str(&format!(
        "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" \
         fill=\"{}\" stroke=\"{}\" stroke-width=\"{}\" rx=\"{}\"/>\n",
        x, y, w, h, fill, stroke, SVG_STROKE_WIDTH, BOX_RADIUS
    ));

    render_c4_labels(&mut svg, node, c4_type, x, y + 10.0, w, text_colour);

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
         fill=\"#f5f5f5\" stroke=\"#999\" stroke-width=\"{}\" rx=\"{}\"/>\n\
         <text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" dominant-baseline=\"central\" \
         font-family=\"Arial, Helvetica, sans-serif\" font-size=\"13\" fill=\"#333\">{}</text>\n",
        x,
        y,
        w,
        h,
        BOX_RADIUS,
        SVG_STROKE_WIDTH,
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
    _c4_type: C4NodeType,
    box_x: f64,
    box_y: f64,
    box_w: f64,
    text_colour: &str,
) {
    let cx = box_x + box_w / 2.0;
    let mut text_y = box_y + 16.0;

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

    // Type descriptor (italic)
    /*     let type_label = c4_type_label(c4_type);
    svg.push_str(&format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" \
         font-family=\"Arial, Helvetica, sans-serif\" \
         font-size=\"{:.0}\" font-style=\"italic\" fill=\"{}\">{}</text>\n",
        cx, text_y, FONT_SIZE_TYPE, text_colour, type_label
    ));
    text_y += LINE_HEIGHT + 2.0; */

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

/// Return (fill, text_colour, stroke) for box/cylinder/queue elements.
fn box_colours(c4_type: C4NodeType) -> (&'static str, &'static str, &'static str) {
    if c4_type.is_external() {
        return (COLOUR_EXT_TEXT, COLOUR_EXT_FILL, COLOUR_EXT_FILL);
    }
    match c4_type {
        C4NodeType::System
        | C4NodeType::SystemDb
        | C4NodeType::SystemQueue
        | C4NodeType::SystemExt
        | C4NodeType::SystemDbExt
        | C4NodeType::SystemQueueExt => {
            (COLOUR_SYSTEM_TEXT, COLOUR_SYSTEM_FILL, COLOUR_SYSTEM_FILL)
        }

        C4NodeType::Container
        | C4NodeType::ContainerDb
        | C4NodeType::ContainerQueue
        | C4NodeType::ContainerExt
        | C4NodeType::ContainerDbExt
        | C4NodeType::ContainerQueueExt => (
            COLOUR_CONTAINER_TEXT,
            COLOUR_CONTAINER_FILL,
            COLOUR_CONTAINER_FILL,
        ),

        C4NodeType::Component
        | C4NodeType::ComponentDb
        | C4NodeType::ComponentQueue
        | C4NodeType::ComponentExt
        | C4NodeType::ComponentDbExt
        | C4NodeType::ComponentQueueExt => (
            COLOUR_COMPONENT_TEXT,
            COLOUR_COMPONENT_FILL,
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

/// Word-wrap `text` to fit `box_w`, then truncate to [`MAX_TEXT_LINES`] lines.
///
/// If wrapping produces more than the limit, the last kept line is replaced with
/// `"…"` so the rendered block never exceeds 5 lines (spec requirement).
fn wrap_text_truncated(text: &str, box_w: f64) -> Vec<String> {
    let mut lines = wrap_text(text, box_w);
    if lines.len() > MAX_TEXT_LINES {
        lines.truncate(MAX_TEXT_LINES - 1);
        lines.push("\u{2026}".to_string()); // U+2026 HORIZONTAL ELLIPSIS
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
        // New C4 v4 shape: ellipse head, rect body, two leg lines
        assert!(svg.contains("<ellipse"), "missing head ellipse");
        assert!(svg.contains("<rect"), "missing body rect");
        assert!(svg.contains("<line"), "missing leg lines");
        // No old shoulder arc
        assert!(!svg.contains("<path"), "unexpected path (old shoulder arc)");
        // Correct colours: white fill, #08427b stroke and text
        assert!(svg.contains("#ffffff"), "missing white fill");
        assert!(svg.contains("#08427b"), "missing stroke colour");
        // Text labels
        assert!(svg.contains("[Person]"), "missing type label");
        assert!(svg.contains("alice"), "missing name label");
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
        // C4 v4: two <path> elements (body + lid), no rect or ellipse
        assert_eq!(svg.matches("<path").count(), 2, "expected body + lid paths");
        assert!(
            !svg.contains("<ellipse"),
            "cylinder must not use ellipse elements"
        );
        assert!(
            !svg.contains("<rect"),
            "cylinder must not use rect elements"
        );
        // Correct colours: white fill, #438dd5 stroke/text
        assert!(svg.contains("#ffffff"), "missing white fill");
        assert!(
            svg.contains(COLOUR_DB_STROKE),
            "missing #438dd5 stroke colour"
        );
        // Lid path must have fill="none"
        assert!(
            svg.contains("fill=\"none\""),
            "lid path must have fill=none"
        );
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
