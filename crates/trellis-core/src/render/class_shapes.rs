use trellis_parser::{ClassVisibility, Node};

/// Line height for attribute/method rows (pixels)
const LINE_HEIGHT: f64 = 18.0;
/// Top compartment height (class name + stereotype)
const HEADER_HEIGHT: f64 = 36.0;
/// Horizontal padding inside the box
const PADDING_X: f64 = 10.0;
/// Font size for class name
const FONT_SIZE_NAME: f64 = 13.0;
/// Font size for members
const FONT_SIZE_MEMBER: f64 = 12.0;

/// Render a class box node as SVG.
///
/// The class box has three compartments separated by horizontal lines:
/// 1. Name compartment: class name (bold) + optional stereotype (italic)
/// 2. Attribute compartment: list of attributes
/// 3. Method compartment: list of methods
pub fn render_class_node(node: &Node) -> String {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    let n_attrs = node.class_attributes.len();
    let n_methods = node.class_methods.len();
    let attr_rows = n_attrs.max(1); // at least 1 empty row
    let method_rows = n_methods.max(1);

    let attr_height = attr_rows as f64 * LINE_HEIGHT;
    let method_height = method_rows as f64 * LINE_HEIGHT;
    // method_height used only to satisfy the pattern; internal layout uses sep positions.
    let _ = method_height;

    let sep1_y = y + HEADER_HEIGHT; // line after name compartment
    let sep2_y = sep1_y + 1.0 + attr_height; // line after attribute compartment

    let mut svg = String::with_capacity(512);

    // Outer box — use node.height (grid-snapped) so the visible box edge aligns
    // exactly with the routing port. The content is top-aligned; any extra space
    // appears as padding at the bottom.
    svg.push_str(&format!(
        "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" \
         fill=\"#f5f5f5\" stroke=\"#555\" stroke-width=\"1.5\"/>\n",
        x, y, w, h
    ));

    // ── Name compartment ──────────────────────────────────────────────

    let cx = x + w / 2.0;

    // Stereotype (if present) — italic, smaller
    let name_label_y = if let Some(ref stereo) = node.stereotype {
        let stereo_display = format!("&lt;&lt;{}&gt;&gt;", stereo);
        svg.push_str(&format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" \
             text-anchor=\"middle\" font-size=\"{:.0}\" font-style=\"italic\" \
             fill=\"#333\">{}</text>\n",
            cx,
            y + 14.0,
            FONT_SIZE_MEMBER,
            stereo_display,
        ));
        y + 28.0 // push name down
    } else {
        y + HEADER_HEIGHT / 2.0 + FONT_SIZE_NAME / 2.0
    };

    // Class name — bold
    svg.push_str(&format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" \
         text-anchor=\"middle\" font-size=\"{:.0}\" font-weight=\"bold\" \
         fill=\"#111\">{}</text>\n",
        cx,
        name_label_y,
        FONT_SIZE_NAME,
        escape_xml(&node.label),
    ));

    // Separator line after name
    svg.push_str(&format!(
        "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" \
         stroke=\"#555\" stroke-width=\"1\"/>\n",
        x, sep1_y, x + w, sep1_y,
    ));

    // ── Attribute compartment ──────────────────────────────────────────

    if node.class_attributes.is_empty() {
        // Empty compartment — just a blank row
        // (already reserved via attr_rows = 1)
    } else {
        for (i, attr) in node.class_attributes.iter().enumerate() {
            let row_y = sep1_y + 1.0 + (i as f64 * LINE_HEIGHT) + LINE_HEIGHT * 0.72;
            let vis_char = visibility_char(attr.visibility);
            let text = if attr.attr_type.is_empty() {
                format!("{}{}", vis_char, escape_xml(&attr.name))
            } else {
                format!(
                    "{}{} {}",
                    vis_char,
                    escape_xml(&attr.attr_type),
                    escape_xml(&attr.name)
                )
            };

            let mut decoration = String::new();
            if attr.is_abstract {
                decoration.push_str(" font-style=\"italic\"");
            }
            if attr.is_static {
                decoration.push_str(" text-decoration=\"underline\"");
            }

            svg.push_str(&format!(
                "<text x=\"{:.1}\" y=\"{:.1}\" \
                 font-size=\"{:.0}\" fill=\"#333\"{}>{}</text>\n",
                x + PADDING_X,
                row_y,
                FONT_SIZE_MEMBER,
                decoration,
                text,
            ));
        }
    }

    // Separator line after attributes
    svg.push_str(&format!(
        "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" \
         stroke=\"#555\" stroke-width=\"1\"/>\n",
        x, sep2_y, x + w, sep2_y,
    ));

    // ── Method compartment ────────────────────────────────────────────

    if node.class_methods.is_empty() {
        // Empty compartment
    } else {
        for (i, method) in node.class_methods.iter().enumerate() {
            let row_y = sep2_y + 1.0 + (i as f64 * LINE_HEIGHT) + LINE_HEIGHT * 0.72;
            let vis_char = visibility_char(method.visibility);
            let params = &method.params;
            let text = if method.return_type.is_empty() {
                format!("{}{}({})", vis_char, escape_xml(&method.name), escape_xml(params))
            } else {
                format!(
                    "{}{}({}) {}",
                    vis_char,
                    escape_xml(&method.name),
                    escape_xml(params),
                    escape_xml(&method.return_type)
                )
            };

            let mut decoration = String::new();
            if method.is_abstract {
                decoration.push_str(" font-style=\"italic\"");
            }
            if method.is_static {
                decoration.push_str(" text-decoration=\"underline\"");
            }

            svg.push_str(&format!(
                "<text x=\"{:.1}\" y=\"{:.1}\" \
                 font-size=\"{:.0}\" fill=\"#333\"{}>{}</text>\n",
                x + PADDING_X,
                row_y,
                FONT_SIZE_MEMBER,
                decoration,
                text,
            ));
        }
    }

    svg
}

/// Convert ClassVisibility to its UML character.
fn visibility_char(vis: ClassVisibility) -> &'static str {
    match vis {
        ClassVisibility::Public => "+",
        ClassVisibility::Private => "-",
        ClassVisibility::Protected => "#",
        ClassVisibility::Package => "~",
    }
}

/// Escape XML special characters.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Generate SVG marker definitions for class diagram edge types.
///
/// Returns a `<defs>` block containing all class-diagram-specific markers.
pub fn class_marker_defs() -> &'static str {
    "<defs>\
     <!-- Inheritance: hollow triangle (open arrowhead at target) -->\
     <marker id=\"inherit-arrow\" viewBox=\"0 0 12 12\" refX=\"12\" refY=\"6\" \
     markerWidth=\"6\" markerHeight=\"6\" orient=\"auto-start-reverse\">\
     <path d=\"M 0 0 L 12 6 L 0 12 Z\" fill=\"white\" stroke=\"#555\" stroke-width=\"1.5\"/>\
     </marker>\
     <!-- Realization: hollow triangle (same as inheritance but used with dotted line) -->\
     <marker id=\"realize-arrow\" viewBox=\"0 0 12 12\" refX=\"12\" refY=\"6\" \
     markerWidth=\"6\" markerHeight=\"6\" orient=\"auto-start-reverse\">\
     <path d=\"M 0 0 L 12 6 L 0 12 Z\" fill=\"white\" stroke=\"#555\" stroke-width=\"1.5\"/>\
     </marker>\
     <!-- Composition: filled diamond at source -->\
     <marker id=\"composition-diamond\" viewBox=\"0 0 20 10\" refX=\"0\" refY=\"5\" \
     markerWidth=\"8\" markerHeight=\"6\" orient=\"auto\">\
     <path d=\"M 0 5 L 8 0 L 16 5 L 8 10 Z\" fill=\"#555\" stroke=\"#555\" stroke-width=\"1\"/>\
     </marker>\
     <!-- Aggregation: hollow diamond at source -->\
     <marker id=\"aggregation-diamond\" viewBox=\"0 0 20 10\" refX=\"0\" refY=\"5\" \
     markerWidth=\"8\" markerHeight=\"6\" orient=\"auto\">\
     <path d=\"M 0 5 L 8 0 L 16 5 L 8 10 Z\" fill=\"white\" stroke=\"#555\" stroke-width=\"1.5\"/>\
     </marker>\
     <!-- Association: open arrowhead -->\
     <marker id=\"assoc-arrow\" viewBox=\"0 0 10 10\" refX=\"10\" refY=\"5\" \
     markerWidth=\"5\" markerHeight=\"5\" orient=\"auto-start-reverse\">\
     <path d=\"M 0 0 L 10 5 L 0 10\" fill=\"none\" stroke=\"#555\" stroke-width=\"1.5\"/>\
     </marker>\
     <!-- Dependency: open arrowhead (same as association but line is dashed) -->\
     <marker id=\"depend-arrow\" viewBox=\"0 0 10 10\" refX=\"10\" refY=\"5\" \
     markerWidth=\"5\" markerHeight=\"5\" orient=\"auto-start-reverse\">\
     <path d=\"M 0 0 L 10 5 L 0 10\" fill=\"none\" stroke=\"#555\" stroke-width=\"1.5\"/>\
     </marker>\
     </defs>"
}

/// Determine the SVG marker-end and marker-start attributes for a class edge.
pub fn class_edge_markers(edge: &trellis_parser::Edge) -> (String, String) {
    use trellis_parser::ClassEdgeType;
    match edge.class_edge_type {
        Some(ClassEdgeType::Inheritance) => (
            String::new(),
            " marker-end=\"url(#inherit-arrow)\"".to_string(),
        ),
        Some(ClassEdgeType::Realization) => (
            String::new(),
            " marker-end=\"url(#realize-arrow)\"".to_string(),
        ),
        Some(ClassEdgeType::Composition) => (
            " marker-start=\"url(#composition-diamond)\"".to_string(),
            String::new(),
        ),
        Some(ClassEdgeType::Aggregation) => (
            " marker-start=\"url(#aggregation-diamond)\"".to_string(),
            String::new(),
        ),
        Some(ClassEdgeType::Association) => (
            String::new(),
            " marker-end=\"url(#assoc-arrow)\"".to_string(),
        ),
        Some(ClassEdgeType::Dependency) => (
            String::new(),
            " marker-end=\"url(#depend-arrow)\"".to_string(),
        ),
        Some(ClassEdgeType::Link) | None => (String::new(), String::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_parser::{ClassAttribute, ClassMethod, ClassVisibility, Node, NodeShape};

    fn make_class_node_with_members() -> Node {
        Node {
            id: "Animal".to_string(),
            label: "Animal".to_string(),
            shape: NodeShape::ClassBox,
            width: 140.0,
            height: 120.0,
            x: 10.0,
            y: 10.0,
            stereotype: Some("abstract".to_string()),
            class_attributes: vec![
                ClassAttribute {
                    visibility: ClassVisibility::Public,
                    attr_type: "String".to_string(),
                    name: "name".to_string(),
                    is_static: false,
                    is_abstract: false,
                },
                ClassAttribute {
                    visibility: ClassVisibility::Private,
                    attr_type: "int".to_string(),
                    name: "age".to_string(),
                    is_static: false,
                    is_abstract: false,
                },
            ],
            class_methods: vec![ClassMethod {
                visibility: ClassVisibility::Public,
                return_type: String::new(),
                name: "makeSound".to_string(),
                params: String::new(),
                is_static: false,
                is_abstract: true,
            }],
        }
    }

    #[test]
    fn test_render_contains_class_name() {
        let node = make_class_node_with_members();
        let svg = render_class_node(&node);
        assert!(svg.contains("Animal"));
    }

    #[test]
    fn test_render_contains_stereotype() {
        let node = make_class_node_with_members();
        let svg = render_class_node(&node);
        assert!(svg.contains("abstract"));
    }

    #[test]
    fn test_render_contains_attribute() {
        let node = make_class_node_with_members();
        let svg = render_class_node(&node);
        assert!(svg.contains("+String name") || svg.contains("+String") && svg.contains("name"));
    }

    #[test]
    fn test_render_contains_method() {
        let node = make_class_node_with_members();
        let svg = render_class_node(&node);
        assert!(svg.contains("makeSound"));
    }

    #[test]
    fn test_render_contains_separators() {
        let node = make_class_node_with_members();
        let svg = render_class_node(&node);
        // Should have at least two separator lines
        let line_count = svg.matches("<line").count();
        assert!(line_count >= 2, "Expected at least 2 separator lines, got {}", line_count);
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("A<B"), "A&lt;B");
        assert_eq!(escape_xml("A&B"), "A&amp;B");
        assert_eq!(escape_xml("normal"), "normal");
    }
}
