use trellis_parser::{Node, NodeShape};

/// Render a single node as an SVG element string.
/// Node positions (x, y) are center coordinates.
pub fn render_node(node: &Node) -> String {
    let cx = node.x;
    let cy = node.y;
    let w = node.width;
    let h = node.height;
    let left = cx - w / 2.0;
    let top = cy - h / 2.0;

    let mut svg = String::new();

    // Shape
    match node.shape {
        NodeShape::Rectangle => {
            svg.push_str(&format!(
                "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" \
                 fill=\"#e8f4fd\" stroke=\"#4a90d9\" stroke-width=\"1.5\"/>\n",
                left, top, w, h,
            ));
        }
        NodeShape::RoundedRectangle => {
            svg.push_str(&format!(
                "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" rx=\"8\" \
                 fill=\"#e8f4fd\" stroke=\"#4a90d9\" stroke-width=\"1.5\"/>\n",
                left, top, w, h,
            ));
        }
        NodeShape::Diamond => {
            // Diamond: four points at top, right, bottom, left of center
            let points = format!(
                "{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                cx, top,           // top
                left + w, cy,      // right
                cx, top + h,       // bottom
                left, cy,          // left
            );
            svg.push_str(&format!(
                "<polygon points=\"{}\" \
                 fill=\"#fff3e0\" stroke=\"#e67e22\" stroke-width=\"1.5\"/>\n",
                points,
            ));
        }
        NodeShape::Circle => {
            let r = w.max(h) / 2.0;
            svg.push_str(&format!(
                "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{:.1}\" \
                 fill=\"#e8f5e9\" stroke=\"#43a047\" stroke-width=\"1.5\"/>\n",
                cx, cy, r,
            ));
        }
        NodeShape::Hexagon => {
            // Hexagon: flat-top style
            let dx = w / 4.0;
            let points = format!(
                "{:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1} {:.1},{:.1}",
                left + dx, top,           // top-left
                left + w - dx, top,       // top-right
                left + w, cy,             // right
                left + w - dx, top + h,   // bottom-right
                left + dx, top + h,       // bottom-left
                left, cy,                 // left
            );
            svg.push_str(&format!(
                "<polygon points=\"{}\" \
                 fill=\"#f3e5f5\" stroke=\"#8e24aa\" stroke-width=\"1.5\"/>\n",
                points,
            ));
        }
    }

    // Label text
    let escaped_label = escape_xml(&node.label);
    svg.push_str(&format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" dominant-baseline=\"central\" \
         font-family=\"Arial, Helvetica, sans-serif\" font-size=\"12\">{}</text>\n",
        cx, cy, escaped_label,
    ));

    svg
}

/// Render all nodes as SVG elements.
pub fn render_nodes(nodes: &[Node]) -> String {
    let mut svg = String::new();
    for node in nodes {
        svg.push_str(&render_node(node));
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

    fn make_node(id: &str, shape: NodeShape, x: f64, y: f64) -> Node {
        Node {
            id: id.to_string(),
            label: id.to_string(),
            shape,
            width: 60.0,
            height: 30.0,
            x,
            y,
        }
    }

    #[test]
    fn test_render_rectangle() {
        let node = make_node("A", NodeShape::Rectangle, 100.0, 50.0);
        let svg = render_node(&node);
        assert!(svg.contains("<rect"));
        assert!(!svg.contains("rx="));
        assert!(svg.contains("text-anchor=\"middle\""));
    }

    #[test]
    fn test_render_rounded_rectangle() {
        let node = make_node("B", NodeShape::RoundedRectangle, 100.0, 50.0);
        let svg = render_node(&node);
        assert!(svg.contains("rx=\"8\""));
    }

    #[test]
    fn test_render_diamond() {
        let node = make_node("C", NodeShape::Diamond, 100.0, 50.0);
        let svg = render_node(&node);
        assert!(svg.contains("<polygon"));
    }

    #[test]
    fn test_render_circle() {
        let node = make_node("D", NodeShape::Circle, 100.0, 50.0);
        let svg = render_node(&node);
        assert!(svg.contains("<circle"));
    }

    #[test]
    fn test_render_hexagon() {
        let node = make_node("E", NodeShape::Hexagon, 100.0, 50.0);
        let svg = render_node(&node);
        assert!(svg.contains("<polygon"));
        // Hexagon has 6 points (6 pairs of coordinates)
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
        let svg = render_nodes(&nodes);
        assert!(svg.contains(">A</text>"));
        assert!(svg.contains(">B</text>"));
    }
}
