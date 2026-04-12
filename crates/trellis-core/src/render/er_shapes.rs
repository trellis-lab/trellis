use trellis_parser::{KeyType, Node};

/// Line height for ER attribute rows (pixels)
const LINE_HEIGHT: f64 = 18.0;
/// Header compartment height (entity name)
const HEADER_HEIGHT: f64 = 30.0;
/// Horizontal padding inside the box
const PADDING_X: f64 = 10.0;
/// Font size for entity name
const FONT_SIZE_NAME: f64 = 13.0;
/// Font size for attribute rows
const FONT_SIZE_ATTR: f64 = 11.0;

/// Render an ER entity box as SVG.
///
/// Layout:
/// - Top compartment: entity name (bold, centered)
/// - Separator line
/// - One row per attribute: `[KEY] type name`
pub fn render_er_node(node: &Node) -> String {
    let x = node.x;
    let y = node.y;
    let w = node.width;
    let h = node.height;

    let mut svg = String::with_capacity(512);

    // Outer box
    svg.push_str(&format!(
        "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" \
         fill=\"#f0f7ff\" stroke=\"#336699\" stroke-width=\"1.5\" rx=\"2\"/>\n",
        x, y, w, h
    ));

    // Header separator line
    let sep_y = y + HEADER_HEIGHT;
    svg.push_str(&format!(
        "<line x1=\"{:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" \
         stroke=\"#336699\" stroke-width=\"1\"/>\n",
        x,
        sep_y,
        x + w,
        sep_y
    ));

    // Entity name — bold, centered
    let cx = x + w / 2.0;
    let name_y = y + HEADER_HEIGHT / 2.0 + FONT_SIZE_NAME / 2.0;
    svg.push_str(&format!(
        "<text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" \
         font-family=\"Arial, Helvetica, sans-serif\" \
         font-size=\"{:.0}\" font-weight=\"bold\" fill=\"#111\">{}</text>\n",
        cx,
        name_y,
        FONT_SIZE_NAME,
        escape_xml(&node.label),
    ));

    // Attribute rows
    for (i, attr) in node.er_attributes.iter().enumerate() {
        let row_y = sep_y + (i as f64 + 0.5) * LINE_HEIGHT + FONT_SIZE_ATTR / 2.0;

        // Key indicator text (left-aligned) — bold for PK, italic for FK
        let key_label = key_prefix(&attr.keys);
        let font_weight = if attr.keys.contains(&KeyType::PK) {
            "bold"
        } else {
            "normal"
        };
        let font_style = if attr.keys.contains(&KeyType::FK) {
            "italic"
        } else {
            "normal"
        };
        let text_decoration = if attr.keys.contains(&KeyType::UK) {
            " text-decoration=\"underline\""
        } else {
            ""
        };

        let attr_text = format!("{}{} {}", key_label, attr.attr_type, attr.name);
        svg.push_str(&format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" \
             font-family=\"Arial, Helvetica, sans-serif\" \
             font-size=\"{:.0}\" font-weight=\"{}\" font-style=\"{}\"{} fill=\"#333\">{}</text>\n",
            x + PADDING_X,
            row_y,
            FONT_SIZE_ATTR,
            font_weight,
            font_style,
            text_decoration,
            escape_xml(&attr_text),
        ));
    }

    svg
}

/// Return a key prefix string for display (e.g., "PK ", "FK ", "UK ", "").
fn key_prefix(keys: &[KeyType]) -> &'static str {
    if keys.contains(&KeyType::PK) {
        "PK "
    } else if keys.contains(&KeyType::FK) {
        "FK "
    } else if keys.contains(&KeyType::UK) {
        "UK "
    } else {
        ""
    }
}

/// Escape XML special characters.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_parser::{ErAttribute, Node, NodeShape};

    fn make_er_node(id: &str, attrs: Vec<ErAttribute>) -> Node {
        Node {
            id: id.to_string(),
            label: id.to_string(),
            shape: NodeShape::ErBox,
            x: 10.0,
            y: 20.0,
            width: 150.0,
            height: HEADER_HEIGHT + attrs.len() as f64 * LINE_HEIGHT + 4.0,
            er_attributes: attrs,
            ..Default::default()
        }
    }

    #[test]
    fn test_render_er_node_no_attrs() {
        let node = make_er_node("USER", vec![]);
        let svg = render_er_node(&node);
        assert!(svg.contains("USER"));
        assert!(svg.contains("<rect"));
    }

    #[test]
    fn test_render_er_node_with_pk_attr() {
        let attr = ErAttribute {
            attr_type: "int".to_string(),
            name: "id".to_string(),
            keys: vec![KeyType::PK],
            comment: None,
        };
        let node = make_er_node("ORDER", vec![attr]);
        let svg = render_er_node(&node);
        assert!(svg.contains("PK int id"));
        assert!(svg.contains("font-weight=\"bold\""));
    }

    #[test]
    fn test_render_er_node_with_fk_attr() {
        let attr = ErAttribute {
            attr_type: "int".to_string(),
            name: "user_id".to_string(),
            keys: vec![KeyType::FK],
            comment: None,
        };
        let node = make_er_node("ORDER", vec![attr]);
        let svg = render_er_node(&node);
        assert!(svg.contains("FK int user_id"));
        assert!(svg.contains("font-style=\"italic\""));
    }
}
