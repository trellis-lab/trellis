use crate::theme::Theme;
use trellis_parser::{Node, NodeShape};

/// Render a single node as an SVG element string.
/// Node positions (x, y) are top-left coordinates.
pub fn render_node(node: &Node, theme: &Theme) -> String {
    let w = node.width;
    let h = node.height;
    let left = node.x;
    let top = node.y;
    let cx = left + w / 2.0;
    let cy = top + h / 2.0;
    // label_cy may be shifted down for shapes whose top decoration overlaps the centre
    let mut label_cy = cy;

    let mut svg = String::new();

    // Shape
    match node.shape {
        NodeShape::Rectangle => {
            svg.push_str(&format!(
                "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" \
                 fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
                left, top, w, h, theme.shape_process_fill, theme.shape_process_stroke,
            ));
        }
        NodeShape::RoundedRectangle => {
            svg.push_str(&format!(
                "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" rx=\"8\" \
                 fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
                left, top, w, h, theme.shape_process_fill, theme.shape_process_stroke,
            ));
        }
        NodeShape::Diamond => {
            // Diamond: four points at top, right, bottom, left of center
            let points = format!(
                "{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                cx,
                top, // top
                left + w,
                cy, // right
                cx,
                top + h, // bottom
                left,
                cy, // left
            );
            svg.push_str(&format!(
                "<polygon points=\"{}\" \
                 fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
                points, theme.shape_decision_fill, theme.shape_decision_stroke,
            ));
        }
        NodeShape::Circle => {
            let r = w.max(h) / 2.0;
            svg.push_str(&format!(
                "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{:.1}\" \
                 fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
                cx, cy, r, theme.shape_terminal_fill, theme.shape_terminal_stroke,
            ));
        }
        NodeShape::Hexagon => {
            // Hexagon: flat-top style
            let dx = w / 4.0;
            let points = format!(
                "{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                left + dx,
                top, // top-left
                left + w - dx,
                top, // top-right
                left + w,
                cy, // right
                left + w - dx,
                top + h, // bottom-right
                left + dx,
                top + h, // bottom-left
                left,
                cy, // left
            );
            svg.push_str(&format!(
                "<polygon points=\"{}\" \
                 fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
                points, theme.shape_special_fill, theme.shape_special_stroke,
            ));
        }
        NodeShape::Stadium => {
            // Stadium: rectangle with fully-rounded ends (rx = half height)
            svg.push_str(&format!(
                "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" rx=\"{:.1}\" \
                 fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
                left,
                top,
                w,
                h,
                h / 2.0,
                theme.shape_terminal_fill,
                theme.shape_terminal_stroke,
            ));
        }
        NodeShape::Subroutine => {
            // Subroutine: rectangle with inner vertical lines 6 px from each side
            svg.push_str(&format!(
                "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" \
                 fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
                left, top, w, h, theme.shape_process_fill, theme.shape_process_stroke,
            ));
            let inset = 6.0;
            svg.push_str(&format!(
                "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" \
                 stroke=\"{}\" stroke-width=\"1.5\"/>\n",
                left + inset,
                top,
                left + inset,
                top + h,
                theme.shape_process_stroke,
            ));
            svg.push_str(&format!(
                "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" \
                 stroke=\"{}\" stroke-width=\"1.5\"/>\n",
                left + w - inset,
                top,
                left + w - inset,
                top + h,
                theme.shape_process_stroke,
            ));
        }
        NodeShape::Asymmetric => {
            // Asymmetric (flag/tag): left side has an inward notch at mid-height, right side is flat
            let tip = h / 3.0;
            let right = left + w;
            let points = format!(
                "{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                left,
                top, // top-left
                right,
                top, // top-right
                right,
                top + h, // bottom-right
                left,
                top + h, // bottom-left
                left + tip,
                cy, // inward notch pointing right (into the shape)
            );
            svg.push_str(&format!(
                "<polygon points=\"{}\" \
                 fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
                points, theme.shape_process_fill, theme.shape_process_stroke,
            ));
        }
        NodeShape::Parallelogram => {
            // Parallelogram: both sides slant like / (leans right)
            let skew = h / 4.0;
            let right = left + w;
            let points = format!(
                "{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                left + skew,
                top, // top-left
                right,
                top, // top-right
                right - skew,
                top + h, // bottom-right
                left,
                top + h, // bottom-left
            );
            svg.push_str(&format!(
                "<polygon points=\"{}\" \
                 fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
                points, theme.shape_process_fill, theme.shape_process_stroke,
            ));
        }
        NodeShape::ParallelogramAlt => {
            // Parallelogram alt: both sides slant like \ (leans left)
            let skew = h / 4.0;
            let right = left + w;
            let points = format!(
                "{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                left,
                top, // top-left
                right - skew,
                top, // top-right
                right,
                top + h, // bottom-right
                left + skew,
                top + h, // bottom-left
            );
            svg.push_str(&format!(
                "<polygon points=\"{}\" \
                 fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
                points, theme.shape_process_fill, theme.shape_process_stroke,
            ));
        }
        NodeShape::TrapezoidAlt => {
            // Trapezoid [/label\]: wider at top, narrower at bottom
            let skew = h / 4.0;
            let right = left + w;
            let points = format!(
                "{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                left,
                top, // top-left
                right,
                top, // top-right
                right - skew,
                top + h, // bottom-right
                left + skew,
                top + h, // bottom-left
            );
            svg.push_str(&format!(
                "<polygon points=\"{}\" \
                 fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
                points, theme.shape_process_fill, theme.shape_process_stroke,
            ));
        }
        NodeShape::Trapezoid => {
            // Trapezoid alt [\label/]: wider at bottom, narrower at top
            let skew = h / 4.0;
            let right = left + w;
            let points = format!(
                "{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                left + skew,
                top, // top-left
                right - skew,
                top, // top-right
                right,
                top + h, // bottom-right
                left,
                top + h, // bottom-left
            );
            svg.push_str(&format!(
                "<polygon points=\"{}\" \
                 fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
                points, theme.shape_process_fill, theme.shape_process_stroke,
            ));
        }
        NodeShape::DoubleCircle => {
            // Double circle: two concentric circles; inner ring drawn with fill=none
            let r = w.max(h) / 2.0;
            svg.push_str(&format!(
                "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{:.1}\" \
                 fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
                cx, cy, r, theme.shape_terminal_fill, theme.shape_terminal_stroke,
            ));
            svg.push_str(&format!(
                "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{:.1}\" \
                 fill=\"none\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
                cx,
                cy,
                r - 5.0,
                theme.shape_terminal_stroke,
            ));
        }
        NodeShape::Cylinder => {
            // Cylinder: two vertical side lines, a full ellipse on top, and a
            // bottom half-arc. Drawn as:
            //   1. Body path  – left wall + bottom semicircle + right wall,
            //                   closed with a straight line (hidden by the top ellipse).
            //   2. Top ellipse – drawn on top of the body to show the top face.
            let ry = (h / 5.0).max(5.0);
            let rx = w / 2.0;
            let right = left + w;
            // Body path:
            //   M top-left  →  L bottom-left  →  arc bottom-right  →  L top-right  →  Z
            // In SVG (y-down), sweep=0 (counter-clockwise) from left to right
            // traces the LOWER arc — the one that bulges downward (∪).
            svg.push_str(&format!(
                "<path d=\"M {:.1},{:.1} L {:.1},{:.1} \
                            A {:.1},{:.1} 0 0 0 {:.1},{:.1} \
                            L {:.1},{:.1} Z\" \
                 fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
                left,
                top + ry, // M top-left
                left,
                top + h - ry, // L bottom-left
                rx,
                ry, // A radii
                right,
                top + h - ry, // A end bottom-right (through bottom)
                right,
                top + ry, // L top-right
                theme.shape_storage_fill,
                theme.shape_storage_stroke,
            ));
            // Top ellipse — slightly different fill to suggest the top face.
            // Its fill hides the straight closing line of the body path.
            svg.push_str(&format!(
                "<ellipse cx=\"{:.1}\" cy=\"{:.1}\" rx=\"{:.1}\" ry=\"{:.1}\" \
                 fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
                cx,
                top + ry,
                rx,
                ry,
                theme.shape_storage_top_fill,
                theme.shape_storage_stroke,
            ));
            // Push the label below the top ellipse so it stays readable.
            label_cy += 10.0;
        }
        // ClassBox, ErBox, and C4Box are rendered by dedicated renderers; fall back here.
        NodeShape::ClassBox | NodeShape::ErBox | NodeShape::C4Box => {
            svg.push_str(&format!(
                "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" \
                 fill=\"{}\" stroke=\"{}\" stroke-width=\"1.5\"/>\n",
                left, top, w, h, theme.fallback_box_fill, theme.fallback_box_stroke,
            ));
        }
    }

    // Label text
    let escaped_label = escape_xml(&node.label);
    svg.push_str(&format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" dominant-baseline=\"central\" \
         font-family=\"Arial, Helvetica, sans-serif\" font-size=\"12\" fill=\"{}\">{}</text>\n",
        cx, label_cy, theme.node_text, escaped_label,
    ));

    svg
}

/// Render all nodes as SVG elements.
pub fn render_nodes(nodes: &[Node], theme: &Theme) -> String {
    let mut svg = String::new();
    for node in nodes {
        svg.push_str(&render_node(node, theme));
    }
    svg
}

/// Escape special XML characters in text.
fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::themes::DEFAULT;

    fn make_node(id: &str, shape: NodeShape, x: f64, y: f64) -> Node {
        Node {
            id: id.to_string(),
            label: id.to_string(),
            shape,
            width: 60.0,
            height: 30.0,
            x,
            y,
            ..Default::default()
        }
    }

    #[test]
    fn test_render_rectangle() {
        let node = make_node("A", NodeShape::Rectangle, 100.0, 50.0);
        let svg = render_node(&node, &DEFAULT);
        assert!(svg.contains("<rect"));
        assert!(!svg.contains("rx="));
        assert!(svg.contains("text-anchor=\"middle\""));
    }

    #[test]
    fn test_render_rounded_rectangle() {
        let node = make_node("B", NodeShape::RoundedRectangle, 100.0, 50.0);
        let svg = render_node(&node, &DEFAULT);
        assert!(svg.contains("rx=\"8\""));
    }

    #[test]
    fn test_render_diamond() {
        let node = make_node("C", NodeShape::Diamond, 100.0, 50.0);
        let svg = render_node(&node, &DEFAULT);
        assert!(svg.contains("<polygon"));
    }

    #[test]
    fn test_render_circle() {
        let node = make_node("D", NodeShape::Circle, 100.0, 50.0);
        let svg = render_node(&node, &DEFAULT);
        assert!(svg.contains("<circle"));
    }

    #[test]
    fn test_render_hexagon() {
        let node = make_node("E", NodeShape::Hexagon, 100.0, 50.0);
        let svg = render_node(&node, &DEFAULT);
        assert!(svg.contains("<polygon"));
        // Hexagon has 6 points (6 pairs of coordinates)
    }

    #[test]
    fn test_render_cylinder() {
        let node = make_node("F", NodeShape::Cylinder, 100.0, 50.0);
        let svg = render_node(&node, &DEFAULT);
        // Body uses a path (left wall + bottom arc + right wall)
        assert!(svg.contains("<path"));
        // One top-face ellipse
        assert_eq!(svg.matches("<ellipse").count(), 1);
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("A & B"), "A &amp; B");
        assert_eq!(escape_xml("<b>"), "&lt;b&gt;");
    }

    #[test]
    fn test_render_multiple_nodes() {
        let nodes = vec![
            make_node("A", NodeShape::Rectangle, 100.0, 50.0),
            make_node("B", NodeShape::RoundedRectangle, 200.0, 50.0),
        ];
        let svg = render_nodes(&nodes, &DEFAULT);
        assert!(svg.contains(">A</text>"));
        assert!(svg.contains(">B</text>"));
    }
}
