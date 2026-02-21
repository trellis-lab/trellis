use trellis_parser::{ErCardinality, KeyType, Node};

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
        x, sep_y, x + w, sep_y
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

/// Generate SVG `<defs>` block containing crow's foot marker definitions.
///
/// Crow's foot markers for ER diagrams. Each cardinality type has a
/// `source` (start) and `target` (end) marker variant since the SVG
/// `orient="auto"` flips the marker at the target end.
///
/// Naming convention: `er-{cardinality}-{end}` where end is `start` or `end`.
pub fn er_marker_defs() -> &'static str {
    "<defs>\
     <!-- ER: ExactlyOne (||) — double tick at target end -->\
     <marker id=\"er-exactlyone-end\" viewBox=\"0 0 12 12\" refX=\"12\" refY=\"6\" \
     markerWidth=\"8\" markerHeight=\"8\" orient=\"auto\">\
     <line x1=\"8\" y1=\"0\" x2=\"8\" y2=\"12\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     <line x1=\"11\" y1=\"0\" x2=\"11\" y2=\"12\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     </marker>\
     <marker id=\"er-exactlyone-start\" viewBox=\"0 0 12 12\" refX=\"0\" refY=\"6\" \
     markerWidth=\"8\" markerHeight=\"8\" orient=\"auto-start-reverse\">\
     <line x1=\"1\" y1=\"0\" x2=\"1\" y2=\"12\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     <line x1=\"4\" y1=\"0\" x2=\"4\" y2=\"12\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     </marker>\
     <!-- ER: ZeroOrOne (|o) — tick + circle -->\
     <marker id=\"er-zeroorone-end\" viewBox=\"0 0 16 12\" refX=\"16\" refY=\"6\" \
     markerWidth=\"10\" markerHeight=\"8\" orient=\"auto\">\
     <line x1=\"12\" y1=\"0\" x2=\"12\" y2=\"12\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     <circle cx=\"5\" cy=\"6\" r=\"4\" fill=\"none\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     </marker>\
     <marker id=\"er-zeroorone-start\" viewBox=\"0 0 16 12\" refX=\"0\" refY=\"6\" \
     markerWidth=\"10\" markerHeight=\"8\" orient=\"auto-start-reverse\">\
     <line x1=\"4\" y1=\"0\" x2=\"4\" y2=\"12\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     <circle cx=\"11\" cy=\"6\" r=\"4\" fill=\"none\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     </marker>\
     <!-- ER: OneOrMore (|{) — tick + crow's foot -->\
     <marker id=\"er-oneormore-end\" viewBox=\"0 0 16 14\" refX=\"16\" refY=\"7\" \
     markerWidth=\"10\" markerHeight=\"9\" orient=\"auto\">\
     <line x1=\"12\" y1=\"0\" x2=\"12\" y2=\"14\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     <line x1=\"12\" y1=\"7\" x2=\"2\" y2=\"0\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     <line x1=\"12\" y1=\"7\" x2=\"2\" y2=\"14\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     <line x1=\"12\" y1=\"7\" x2=\"2\" y2=\"7\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     </marker>\
     <marker id=\"er-oneormore-start\" viewBox=\"0 0 16 14\" refX=\"0\" refY=\"7\" \
     markerWidth=\"10\" markerHeight=\"9\" orient=\"auto-start-reverse\">\
     <line x1=\"4\" y1=\"0\" x2=\"4\" y2=\"14\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     <line x1=\"4\" y1=\"7\" x2=\"14\" y2=\"0\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     <line x1=\"4\" y1=\"7\" x2=\"14\" y2=\"14\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     <line x1=\"4\" y1=\"7\" x2=\"14\" y2=\"7\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     </marker>\
     <!-- ER: ZeroOrMore (o{) — circle + crow's foot -->\
     <marker id=\"er-zeroormore-end\" viewBox=\"0 0 20 14\" refX=\"20\" refY=\"7\" \
     markerWidth=\"12\" markerHeight=\"9\" orient=\"auto\">\
     <circle cx=\"16\" cy=\"7\" r=\"3.5\" fill=\"none\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     <line x1=\"12\" y1=\"7\" x2=\"2\" y2=\"0\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     <line x1=\"12\" y1=\"7\" x2=\"2\" y2=\"14\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     <line x1=\"12\" y1=\"7\" x2=\"2\" y2=\"7\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     </marker>\
     <marker id=\"er-zeroormore-start\" viewBox=\"0 0 20 14\" refX=\"0\" refY=\"7\" \
     markerWidth=\"12\" markerHeight=\"9\" orient=\"auto-start-reverse\">\
     <circle cx=\"4\" cy=\"7\" r=\"3.5\" fill=\"none\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     <line x1=\"8\" y1=\"7\" x2=\"18\" y2=\"0\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     <line x1=\"8\" y1=\"7\" x2=\"18\" y2=\"14\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     <line x1=\"8\" y1=\"7\" x2=\"18\" y2=\"7\" stroke=\"#336699\" stroke-width=\"1.5\"/>\
     </marker>\
     </defs>"
}

/// Return the marker ID suffix string for an ER cardinality.
fn cardinality_marker_id(card: ErCardinality) -> &'static str {
    match card {
        ErCardinality::ExactlyOne => "exactlyone",
        ErCardinality::ZeroOrOne => "zeroorone",
        ErCardinality::OneOrMore => "oneormore",
        ErCardinality::ZeroOrMore => "zeroormore",
    }
}

/// Return SVG marker-start and marker-end attribute strings for an ER edge.
///
/// Source cardinality → `marker-start` (at the `from` node end).
/// Target cardinality → `marker-end` (at the `to` node end).
pub fn er_edge_markers(edge: &trellis_parser::Edge) -> (String, String) {
    let src = edge
        .er_source_card
        .map(|c| {
            format!(
                " marker-start=\"url(#er-{}-start)\"",
                cardinality_marker_id(c)
            )
        })
        .unwrap_or_default();

    let tgt = edge
        .er_target_card
        .map(|c| {
            format!(
                " marker-end=\"url(#er-{}-end)\"",
                cardinality_marker_id(c)
            )
        })
        .unwrap_or_default();

    (src, tgt)
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

    #[test]
    fn test_er_marker_defs_contains_all_types() {
        let defs = er_marker_defs();
        assert!(defs.contains("er-exactlyone-end"));
        assert!(defs.contains("er-zeroorone-end"));
        assert!(defs.contains("er-oneormore-end"));
        assert!(defs.contains("er-zeroormore-end"));
        assert!(defs.contains("er-exactlyone-start"));
        assert!(defs.contains("er-zeroorone-start"));
        assert!(defs.contains("er-oneormore-start"));
        assert!(defs.contains("er-zeroormore-start"));
    }
}
