use crate::ast::*;
use crate::tokenizer::{Token, TokenType};
use std::collections::HashMap;

/// Parse ER diagram tokens into a Graph.
///
/// Handles:
/// - Entity definitions: `EntityName { type attrName PK, ... }`
/// - Standalone entity declarations: `EntityName`
/// - Relations with crow's foot cardinality: `EntityA ||--o{ EntityB : "label"`
pub fn parse_er_diagram(tokens: &[Token]) -> Result<Graph, crate::ParseError> {
    let mut graph = Graph::new();
    graph.diagram_type = DiagramType::ErDiagram;
    graph.direction = Direction::TB;

    let mut node_map: HashMap<String, usize> = HashMap::new();

    // Multi-line entity body parsing state
    let mut in_entity_body: Option<String> = None; // current entity id
    let mut brace_depth: u32 = 0;

    for token in tokens {
        match token.token_type {
            TokenType::Directive => continue,
            TokenType::SubgraphStart | TokenType::SubgraphEnd => continue,
            TokenType::Statement => {
                let line = token.content.trim();

                // Handle entity body lines while inside { ... }
                if let Some(ref entity_id) = in_entity_body.clone() {
                    if line == "}" || (line.ends_with('}') && !line.starts_with('{')) {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            let content = line.trim_end_matches('}').trim();
                            if !content.is_empty() {
                                parse_er_attribute(&mut graph, &mut node_map, entity_id, content);
                            }
                            in_entity_body = None;
                        }
                        continue;
                    }
                    if line.contains('{') {
                        brace_depth += 1;
                    }
                    // Parse attribute line inside entity body
                    parse_er_attribute(&mut graph, &mut node_map, entity_id, line);
                    continue;
                }

                // Not inside an entity body — parse top-level statements

                // Try to parse as a relation
                if let Some(relation) = try_parse_relation(line) {
                    let (entity_a, src_card, is_identifying, tgt_card, entity_b, label) = relation;

                    ensure_er_entity(&mut graph, &mut node_map, &entity_a);
                    ensure_er_entity(&mut graph, &mut node_map, &entity_b);

                    let edge = Edge {
                        from: entity_a,
                        to: entity_b,
                        label,
                        style: if is_identifying {
                            EdgeStyle::Solid
                        } else {
                            EdgeStyle::Dotted
                        },
                        arrow_head: ArrowHead::None,
                        er_source_card: Some(src_card),
                        er_target_card: Some(tgt_card),
                        er_identifying: Some(is_identifying),
                        ..Default::default()
                    };
                    graph.edges.push(edge);
                    continue;
                }

                // Detect "EntityName {" — start of entity body
                if let Some(brace_pos) = line.find('{') {
                    let entity_id = line[..brace_pos].trim().to_string();
                    if is_valid_entity_name(&entity_id) {
                        ensure_er_entity(&mut graph, &mut node_map, &entity_id);
                        let after_brace = line[brace_pos + 1..].trim();
                        if after_brace.contains('}') {
                            // Inline body: "EntityName { type attr PK }"
                            let body = after_brace.trim_end_matches('}').trim();
                            if !body.is_empty() {
                                parse_er_attribute(&mut graph, &mut node_map, &entity_id, body);
                            }
                        } else {
                            in_entity_body = Some(entity_id.clone());
                            brace_depth = 1;
                            if !after_brace.is_empty() {
                                parse_er_attribute(
                                    &mut graph,
                                    &mut node_map,
                                    &entity_id,
                                    after_brace,
                                );
                            }
                        }
                        continue;
                    }
                }

                // Standalone entity declaration: just "EntityName"
                if is_valid_entity_name(line) {
                    ensure_er_entity(&mut graph, &mut node_map, line);
                }
            }
        }
    }

    // Calculate node sizes based on attributes
    for node in &mut graph.nodes {
        size_er_node(node);
    }

    Ok(graph)
}

/// Ensure an entity node exists; create it if not.
fn ensure_er_entity(graph: &mut Graph, node_map: &mut HashMap<String, usize>, entity_id: &str) {
    if !node_map.contains_key(entity_id) {
        let idx = graph.nodes.len();
        graph.nodes.push(Node {
            id: entity_id.to_string(),
            label: entity_id.to_string(),
            shape: NodeShape::ErBox,
            ..Default::default()
        });
        node_map.insert(entity_id.to_string(), idx);
    }
}

/// Parse an attribute line inside an entity body.
///
/// Format: `type name [PK] [FK] [UK] ["comment"]`
fn parse_er_attribute(
    graph: &mut Graph,
    node_map: &mut HashMap<String, usize>,
    entity_id: &str,
    line: &str,
) {
    let line = line.trim();
    if line.is_empty() {
        return;
    }

    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.len() < 2 {
        return; // Need at least type and name
    }

    let attr_type = tokens[0].to_string();
    let name = tokens[1].to_string();

    let mut keys = Vec::new();
    let mut comment: Option<String> = None;

    // Parse remaining tokens for key types and comment
    let mut i = 2;
    while i < tokens.len() {
        let tok = tokens[i];
        match tok.to_uppercase().as_str() {
            "PK" => keys.push(KeyType::PK),
            "FK" => keys.push(KeyType::FK),
            "UK" => keys.push(KeyType::UK),
            _ => {
                // Could be a comment in quotes or combined PK,FK etc.
                // Handle comma-separated keys like "PK,FK"
                for part in tok.split(',') {
                    match part.to_uppercase().as_str() {
                        "PK" => keys.push(KeyType::PK),
                        "FK" => keys.push(KeyType::FK),
                        "UK" => keys.push(KeyType::UK),
                        _ => {
                            // Treat as start of comment
                            let rest = tokens[i..].join(" ");
                            let trimmed = rest.trim_matches('"');
                            comment = Some(trimmed.to_string());
                            i = tokens.len(); // exit
                            break;
                        }
                    }
                }
            }
        }
        i += 1;
    }

    let attr = ErAttribute {
        attr_type,
        name,
        keys,
        comment,
    };

    if let Some(&idx) = node_map.get(entity_id) {
        graph.nodes[idx].er_attributes.push(attr);
    }
}

/// Try to parse a line as an ER relation.
///
/// Returns `(entity_a, source_card, is_identifying, target_card, entity_b, label)`.
///
/// Crow's foot notation:
/// - Left-side: `||` ExactlyOne, `|o` ZeroOrOne, `}|` OneOrMore, `}o` ZeroOrMore
/// - Connector: `--` (identifying/solid), `..` (non-identifying/dashed)
/// - Right-side: `||` ExactlyOne, `o|` ZeroOrOne, `|{` OneOrMore, `o{` ZeroOrMore
fn try_parse_relation(
    line: &str,
) -> Option<(String, ErCardinality, bool, ErCardinality, String, Option<String>)> {
    // Characters used in cardinality markers
    const CARD_CHARS: &[u8] = b"|o}{";

    let bytes = line.as_bytes();
    let len = bytes.len();

    // Try to find `--` or `..` surrounded by cardinality characters
    let find_connector = |connector: &[u8; 2]| -> Option<(usize, bool)> {
        let is_identifying = connector == b"--";
        let mut i = 2usize;
        while i + 2 <= len {
            if &bytes[i..i + 2] == connector.as_slice() {
                // Check that chars at [i-2], [i-1] are cardinality chars
                // and chars at [i+2], [i+3] are cardinality chars
                if i >= 2 && i + 4 <= len {
                    let l1 = bytes[i - 2];
                    let l2 = bytes[i - 1];
                    let r1 = bytes[i + 2];
                    let r2 = bytes[i + 3];
                    if CARD_CHARS.contains(&l1)
                        && CARD_CHARS.contains(&l2)
                        && CARD_CHARS.contains(&r1)
                        && CARD_CHARS.contains(&r2)
                    {
                        return Some((i, is_identifying));
                    }
                }
            }
            i += 1;
        }
        None
    };

    let (connector_pos, is_identifying) =
        find_connector(b"--").or_else(|| find_connector(b".."))?;

    // Left cardinality: bytes[connector_pos-2..connector_pos]
    let left_card_bytes = &bytes[connector_pos - 2..connector_pos];
    // Right cardinality: bytes[connector_pos+2..connector_pos+4]
    let right_card_bytes = &bytes[connector_pos + 2..connector_pos + 4];

    let src_card = parse_left_cardinality(left_card_bytes)?;
    let tgt_card = parse_right_cardinality(right_card_bytes)?;

    // Entity A: everything before the cardinality marker (trim)
    let entity_a = line[..connector_pos - 2].trim().to_string();
    // Rest: everything after the right cardinality marker
    let rest = line[connector_pos + 4..].trim();

    if entity_a.is_empty() {
        return None;
    }

    // Parse entity_b and optional label: `EntityB : "label"` or `EntityB : label`
    let (entity_b, label) = if let Some(colon_pos) = rest.find(" : ") {
        let raw_label = rest[colon_pos + 3..].trim();
        let label_text = raw_label.trim_matches('"').to_string();
        (rest[..colon_pos].trim().to_string(), Some(label_text))
    } else {
        (rest.trim().to_string(), None)
    };

    if entity_b.is_empty() {
        return None;
    }

    Some((entity_a, src_card, is_identifying, tgt_card, entity_b, label))
}

/// Parse left-side cardinality marker (2 bytes).
fn parse_left_cardinality(bytes: &[u8]) -> Option<ErCardinality> {
    match bytes {
        b"||" => Some(ErCardinality::ExactlyOne),
        b"|o" => Some(ErCardinality::ZeroOrOne),
        b"}|" => Some(ErCardinality::OneOrMore),
        b"}o" => Some(ErCardinality::ZeroOrMore),
        _ => None,
    }
}

/// Parse right-side cardinality marker (2 bytes).
fn parse_right_cardinality(bytes: &[u8]) -> Option<ErCardinality> {
    match bytes {
        b"||" => Some(ErCardinality::ExactlyOne),
        b"o|" => Some(ErCardinality::ZeroOrOne),
        b"|{" => Some(ErCardinality::OneOrMore),
        b"o{" => Some(ErCardinality::ZeroOrMore),
        _ => None,
    }
}

/// Check if a string is a valid entity name (non-empty, no relation chars).
fn is_valid_entity_name(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    // Entity names are alphanumeric with optional dashes/underscores
    s.chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// Calculate node dimensions based on entity name and attribute count.
fn size_er_node(node: &mut Node) {
    const LINE_HEIGHT: f64 = 18.0;
    const HEADER_HEIGHT: f64 = 30.0;
    const PADDING_X: f64 = 20.0;
    const MIN_WIDTH: f64 = 100.0;
    const CHAR_WIDTH: f64 = 7.5; // approximate char width for size estimation

    let name_width = node.label.len() as f64 * CHAR_WIDTH + PADDING_X * 2.0;
    let max_attr_width = node
        .er_attributes
        .iter()
        .map(|a| {
            let text_len = a.attr_type.len() + 1 + a.name.len() + if a.keys.is_empty() { 0 } else { 4 };
            text_len as f64 * CHAR_WIDTH + PADDING_X * 2.0
        })
        .fold(0.0_f64, f64::max);

    node.width = name_width.max(max_attr_width).max(MIN_WIDTH);
    node.height = HEADER_HEIGHT + node.er_attributes.len() as f64 * LINE_HEIGHT + 4.0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::tokenize;

    fn parse(input: &str) -> Graph {
        let tokens = tokenize(input);
        parse_er_diagram(&tokens).expect("parse failed")
    }

    #[test]
    fn test_parse_simple_relation() {
        let g = parse("erDiagram\n    USER ||--o{ ORDER : places\n");
        assert_eq!(g.nodes.len(), 2);
        assert_eq!(g.edges.len(), 1);
        let e = &g.edges[0];
        assert_eq!(e.from, "USER");
        assert_eq!(e.to, "ORDER");
        assert_eq!(e.er_source_card, Some(ErCardinality::ExactlyOne));
        assert_eq!(e.er_target_card, Some(ErCardinality::ZeroOrMore));
        assert_eq!(e.er_identifying, Some(true));
        assert_eq!(e.label, Some("places".to_string()));
    }

    #[test]
    fn test_parse_non_identifying_relation() {
        let g = parse("erDiagram\n    A }|..o{ B : \"label\"\n");
        assert_eq!(g.edges.len(), 1);
        let e = &g.edges[0];
        assert_eq!(e.er_source_card, Some(ErCardinality::OneOrMore));
        assert_eq!(e.er_target_card, Some(ErCardinality::ZeroOrMore));
        assert_eq!(e.er_identifying, Some(false));
    }

    #[test]
    fn test_cardinality_exactly_one() {
        let g = parse("erDiagram\n    A ||--|| B : has\n");
        assert_eq!(g.edges[0].er_source_card, Some(ErCardinality::ExactlyOne));
        assert_eq!(g.edges[0].er_target_card, Some(ErCardinality::ExactlyOne));
    }

    #[test]
    fn test_cardinality_zero_or_one() {
        let g = parse("erDiagram\n    A |o--o| B : has\n");
        assert_eq!(g.edges[0].er_source_card, Some(ErCardinality::ZeroOrOne));
        assert_eq!(g.edges[0].er_target_card, Some(ErCardinality::ZeroOrOne));
    }

    #[test]
    fn test_cardinality_one_or_more() {
        let g = parse("erDiagram\n    A }|--|{ B : has\n");
        assert_eq!(g.edges[0].er_source_card, Some(ErCardinality::OneOrMore));
        assert_eq!(g.edges[0].er_target_card, Some(ErCardinality::OneOrMore));
    }

    #[test]
    fn test_entity_with_attributes() {
        let g = parse(
            "erDiagram\n    USER {\n        int id PK\n        string name\n        string email UK\n    }\n",
        );
        assert_eq!(g.nodes.len(), 1);
        let node = &g.nodes[0];
        assert_eq!(node.id, "USER");
        assert_eq!(node.er_attributes.len(), 3);
        assert!(node.er_attributes[0].keys.contains(&KeyType::PK));
        assert!(node.er_attributes[1].keys.is_empty());
        assert!(node.er_attributes[2].keys.contains(&KeyType::UK));
    }

    #[test]
    fn test_standalone_entity() {
        let g = parse("erDiagram\n    PRODUCT\n");
        assert_eq!(g.nodes.len(), 1);
        assert_eq!(g.nodes[0].id, "PRODUCT");
    }

    #[test]
    fn test_entity_with_pk_fk() {
        let g = parse(
            "erDiagram\n    ORDER {\n        int id PK\n        int user_id FK\n    }\n",
        );
        let node = &g.nodes[0];
        assert!(node.er_attributes[0].keys.contains(&KeyType::PK));
        assert!(node.er_attributes[1].keys.contains(&KeyType::FK));
    }

    #[test]
    fn test_entity_name_with_dash() {
        // LINE-ITEM is a valid entity name
        let g = parse("erDiagram\n    ORDER ||--|{ LINE-ITEM : contains\n");
        assert_eq!(g.nodes.len(), 2);
        assert!(g.nodes.iter().any(|n| n.id == "LINE-ITEM"));
    }

    #[test]
    fn test_full_b11_fixture() {
        let input = "\
erDiagram
    USER ||--o{ ORDER : places
    ORDER ||--|{ LINE-ITEM : contains
    PRODUCT ||--o{ LINE-ITEM : includes
    USER {
        int id PK
        string name
        string email
    }
    ORDER {
        int id PK
        date order_date
    }
    LINE-ITEM {
        int id PK
        int quantity
    }
    PRODUCT {
        int id PK
        string name
        float price
    }
";
        let g = parse(input);
        assert_eq!(g.nodes.len(), 4);
        assert_eq!(g.edges.len(), 3);
    }
}
