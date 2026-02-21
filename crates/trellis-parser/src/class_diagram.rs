use crate::ast::*;
use crate::text_metrics;
use crate::tokenizer::{Token, TokenType};
use std::collections::HashMap;

/// Parse class diagram tokens into a Graph.
pub fn parse_class_diagram(tokens: &[Token]) -> Result<Graph, crate::ParseError> {
    let mut graph = Graph::new();
    graph.diagram_type = DiagramType::ClassDiagram;
    graph.direction = Direction::TB;

    let mut node_map: HashMap<String, usize> = HashMap::new();

    // Multi-line class body parsing state
    let mut in_class_body: Option<String> = None; // current class id
    let mut brace_depth: u32 = 0;

    for token in tokens {
        match token.token_type {
            TokenType::Directive => continue,
            TokenType::SubgraphStart | TokenType::SubgraphEnd => continue,
            TokenType::Statement => {
                let line = token.content.trim();

                // Handle class body lines while inside { ... }
                if let Some(ref class_id) = in_class_body.clone() {
                    if line == "}" || line.ends_with('}') {
                        brace_depth -= 1;
                        if brace_depth == 0 {
                            // Process the closing brace content if any
                            let content = line.trim_end_matches('}').trim();
                            if !content.is_empty() {
                                parse_class_member(&mut graph, &mut node_map, class_id, content);
                            }
                            in_class_body = None;
                        }
                        continue;
                    }
                    if line.contains('{') {
                        brace_depth += 1;
                    }
                    // Parse stereotype line
                    if line.starts_with("<<") && line.ends_with(">>") {
                        let stereotype = line[2..line.len() - 2].trim().to_string();
                        if let Some(&idx) = node_map.get(class_id) {
                            graph.nodes[idx].stereotype = Some(stereotype);
                        }
                        continue;
                    }
                    // Parse member line inside class body
                    parse_class_member(&mut graph, &mut node_map, class_id, line);
                    continue;
                }

                // Not inside a class body — parse top-level statements

                // Detect "class ClassName" or "class ClassName { ... }"
                if let Some(rest) = line.strip_prefix("class ") {
                    let rest = rest.trim();
                    // Possible: "class Foo", "class Foo { ... }", "class Foo { <<stereo>> ... }"
                    let (class_id, has_open_brace, inline_content) =
                        split_class_header(rest);
                    ensure_class_node(&mut graph, &mut node_map, &class_id);
                    if has_open_brace {
                        // Check if body closes on same line
                        if let Some(close_pos) = inline_content.find('}') {
                            let body = &inline_content[..close_pos];
                            parse_class_body_inline(&mut graph, &mut node_map, &class_id, body);
                        } else {
                            // Multi-line body
                            in_class_body = Some(class_id.clone());
                            brace_depth = 1;
                            // Inline content after '{' on same line
                            if !inline_content.is_empty() {
                                parse_class_body_inline(
                                    &mut graph,
                                    &mut node_map,
                                    &class_id,
                                    &inline_content,
                                );
                            }
                        }
                    }
                    continue;
                }

                // Detect "ClassName : member" (attribute/method definition)
                if let Some(colon_pos) = line.find(" : ") {
                    let class_id = line[..colon_pos].trim().to_string();
                    let member = line[colon_pos + 3..].trim();
                    if !class_id.contains(' ') && !class_id.contains('<') && !class_id.contains('-')
                        && !class_id.contains('.')
                    {
                        ensure_class_node(&mut graph, &mut node_map, &class_id);
                        parse_class_member(&mut graph, &mut node_map, &class_id, member);
                        continue;
                    }
                }

                // Detect note statements (skip them)
                if line.starts_with("note ") {
                    continue;
                }

                // Detect namespace blocks (skip for now)
                if line.starts_with("namespace ") {
                    continue;
                }

                // Try to parse a relation
                if let Some((from, edge_type, to, source_mult, target_mult, label)) =
                    parse_relation(line)
                {
                    ensure_class_node(&mut graph, &mut node_map, &from);
                    ensure_class_node(&mut graph, &mut node_map, &to);

                    let style = if matches!(
                        edge_type,
                        ClassEdgeType::Realization | ClassEdgeType::Dependency | ClassEdgeType::Link
                    ) {
                        EdgeStyle::Dotted
                    } else {
                        EdgeStyle::Solid
                    };

                    graph.edges.push(Edge {
                        from,
                        to,
                        label,
                        style,
                        arrow_head: ArrowHead::None,
                        class_edge_type: Some(edge_type),
                        source_multiplicity: source_mult,
                        target_multiplicity: target_mult,
                    });
                }
            }
        }
    }

    // After parsing all tokens: calculate node sizes based on content
    recalculate_class_node_sizes(&mut graph);

    Ok(graph)
}

// ── Class header parsing ─────────────────────────────────────────────

/// Split "ClassName" or "ClassName { ... }" into (id, has_brace, after_brace).
fn split_class_header(rest: &str) -> (String, bool, String) {
    if let Some(brace_pos) = rest.find('{') {
        let class_id = rest[..brace_pos].trim().to_string();
        let after = rest[brace_pos + 1..].trim().to_string();
        (class_id, true, after)
    } else {
        (rest.to_string(), false, String::new())
    }
}

// ── Inline body parsing ──────────────────────────────────────────────

/// Parse the content between `{` and `}` on a single line.
fn parse_class_body_inline(
    graph: &mut Graph,
    node_map: &mut HashMap<String, usize>,
    class_id: &str,
    body: &str,
) {
    for part in body.split('\n') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if part.starts_with("<<") && part.ends_with(">>") {
            let stereotype = part[2..part.len() - 2].trim().to_string();
            if let Some(&idx) = node_map.get(class_id) {
                graph.nodes[idx].stereotype = Some(stereotype);
            }
        } else {
            parse_class_member(graph, node_map, class_id, part);
        }
    }
}

// ── Member parsing ───────────────────────────────────────────────────

/// Parse a single class member line (attribute or method).
fn parse_class_member(
    graph: &mut Graph,
    node_map: &mut HashMap<String, usize>,
    class_id: &str,
    line: &str,
) {
    let line = line.trim();
    if line.is_empty() {
        return;
    }

    // Stereotype inside body: <<interface>>
    if line.starts_with("<<") && line.ends_with(">>") {
        let stereotype = line[2..line.len() - 2].trim().to_string();
        if let Some(&idx) = node_map.get(class_id) {
            graph.nodes[idx].stereotype = Some(stereotype);
        }
        return;
    }

    let (visibility, rest, is_static, is_abstract) = parse_visibility_prefix(line);

    // If rest ends with `(...)` it's a method
    if let Some(paren_start) = rest.find('(') {
        let name_and_type = rest[..paren_start].trim();
        let params_with_end = &rest[paren_start..];
        let params = if let Some(close) = params_with_end.find(')') {
            params_with_end[1..close].to_string()
        } else {
            String::new()
        };

        // name_and_type may be "returnType methodName" or just "methodName"
        let (return_type, name) = split_type_name(name_and_type);

        // Return type annotation after closing paren (Mermaid: `name() ReturnType`)
        let return_type_final = if return_type.is_empty() {
            // Check for return type after closing paren
            if let Some(close) = params_with_end.find(')') {
                let after_close = params_with_end[close + 1..].trim();
                if !after_close.is_empty() {
                    after_close.to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            return_type
        };

        if let Some(&idx) = node_map.get(class_id) {
            graph.nodes[idx].class_methods.push(ClassMethod {
                visibility,
                return_type: return_type_final,
                name,
                params,
                is_static,
                is_abstract,
            });
        }
    } else {
        // Attribute: "type name" or just "name"
        let (attr_type, name) = split_type_name(rest);
        if name.is_empty() {
            return;
        }
        if let Some(&idx) = node_map.get(class_id) {
            graph.nodes[idx].class_attributes.push(ClassAttribute {
                visibility,
                attr_type,
                name,
                is_static,
                is_abstract,
            });
        }
    }
}

/// Parse visibility prefix (+, -, #, ~) and modifiers ($=static, *=abstract).
/// Returns (visibility, remaining, is_static, is_abstract).
fn parse_visibility_prefix(s: &str) -> (ClassVisibility, &str, bool, bool) {
    let s = s.trim();
    let mut is_static = false;
    let mut is_abstract = false;

    // Check for modifier prefixes after visibility
    let (vis, rest) = if let Some(rest) = s.strip_prefix('+') {
        (ClassVisibility::Public, rest.trim_start())
    } else if let Some(rest) = s.strip_prefix('-') {
        (ClassVisibility::Private, rest.trim_start())
    } else if let Some(rest) = s.strip_prefix('#') {
        (ClassVisibility::Protected, rest.trim_start())
    } else if let Some(rest) = s.strip_prefix('~') {
        (ClassVisibility::Package, rest.trim_start())
    } else {
        (ClassVisibility::Public, s)
    };

    // Check for $ (static) or * (abstract) modifier
    let rest = if let Some(rest) = rest.strip_prefix('$') {
        is_static = true;
        rest.trim_start()
    } else if let Some(rest) = rest.strip_prefix('*') {
        is_abstract = true;
        rest.trim_start()
    } else {
        rest
    };

    (vis, rest, is_static, is_abstract)
}

/// Split "type name" into (type, name). If only one word, treat as (name="", type=word)
/// unless the word is a name without a type.
fn split_type_name(s: &str) -> (String, String) {
    let s = s.trim();
    let parts: Vec<&str> = s.splitn(2, ' ').collect();
    if parts.len() == 2 {
        (parts[0].to_string(), parts[1].trim().to_string())
    } else if parts.len() == 1 && !parts[0].is_empty() {
        // Single word — treat as name with no type
        (String::new(), parts[0].to_string())
    } else {
        (String::new(), String::new())
    }
}

// ── Relation parsing ──────────────────────────────────────────────────

/// Return type for `parse_relation`: (from, edge_type, to, source_mult, target_mult, label).
type RelationParts =
    (String, ClassEdgeType, String, Option<String>, Option<String>, Option<String>);

/// Try to parse a class diagram relation line.
///
/// Returns `(from, edge_type, to, source_mult, target_mult, label)` or None.
///
/// Supported syntaxes:
/// - `ClassA <|-- ClassB`
/// - `ClassA --|> ClassB`
/// - `ClassA *-- ClassB`  (and reversed `--*`)
/// - `ClassA o-- ClassB`  (and reversed `--o`)
/// - `ClassA --> ClassB`
/// - `ClassA <-- ClassB`
/// - `ClassA -- ClassB`
/// - `ClassA <|.. ClassB`
/// - `ClassA ..|> ClassB`
/// - `ClassA ..> ClassB`
/// - `ClassA <.. ClassB`
/// - `ClassA .. ClassB`
/// - `ClassA "1" --> "0..*" ClassB : label`
pub fn parse_relation(line: &str) -> Option<RelationParts> {
    let line = line.trim();

    // Extract label at the end: "... : label"
    let (line_no_label, label) = if let Some(colon_pos) = find_relation_colon(line) {
        let lbl = line[colon_pos + 1..].trim().to_string();
        (&line[..colon_pos], if lbl.is_empty() { None } else { Some(lbl) })
    } else {
        (line, None)
    };
    let line_no_label = line_no_label.trim();

    // Try to extract multiplicity quotes from each side
    // Pattern: `ClassName "mult" ARROW "mult" ClassName`
    // or: `ClassName ARROW ClassName`
    let (lhs, source_mult, rhs, target_mult) = extract_multiplicities(line_no_label)?;

    let lhs = lhs.trim();
    let rhs = rhs.trim();

    // Try each arrow pattern (order matters — more specific first)
    let patterns: &[(&str, ClassEdgeType, bool)] = &[
        // Inheritance
        ("<|--", ClassEdgeType::Inheritance, false),
        ("--|>", ClassEdgeType::Inheritance, true),
        // Realization (dotted inheritance)
        ("<|..", ClassEdgeType::Realization, false),
        ("..|>", ClassEdgeType::Realization, true),
        // Composition
        ("*--", ClassEdgeType::Composition, false),
        ("--*", ClassEdgeType::Composition, true),
        // Aggregation
        ("o--", ClassEdgeType::Aggregation, false),
        ("--o", ClassEdgeType::Aggregation, true),
        // Dependency (dotted arrow)
        ("<..", ClassEdgeType::Dependency, false),
        ("..>", ClassEdgeType::Dependency, true),
        // Link (plain dotted)
        ("..", ClassEdgeType::Link, false),
        // Association (arrow variants)
        ("<--", ClassEdgeType::Association, false),
        ("-->", ClassEdgeType::Association, true),
        ("--", ClassEdgeType::Association, false),
    ];

    for (arrow, edge_type, arrow_points_right) in patterns {
        if let Some(arrow_pos) = lhs.rfind(arrow) {
            // Split: everything left of arrow is class A
            let class_a = lhs[..arrow_pos].trim();
            if class_a.is_empty() || class_a.contains(' ') {
                continue;
            }
            // rhs is class B
            if rhs.is_empty() || rhs.contains(' ') {
                continue;
            }
            let (from, to) = (class_a.to_string(), rhs.to_string());
            // Swap multiplicities to match from/to direction
            let (sm, tm) = if *arrow_points_right {
                (target_mult, source_mult)
            } else {
                (source_mult, target_mult)
            };
            return Some((from, *edge_type, to, sm, tm, label));
        }
    }

    // Fallback: try with arrow in the middle of the whole string
    for (arrow, edge_type, arrow_points_right) in patterns {
        if let Some(arrow_pos) = line_no_label.find(arrow) {
            let left = line_no_label[..arrow_pos].trim();
            let right = line_no_label[arrow_pos + arrow.len()..].trim();

            // Strip quotes if present
            let left = left.trim_matches('"').trim();
            let right = right.trim_matches('"').trim();

            if left.is_empty() || right.is_empty() {
                continue;
            }
            // left might be "ClassA "mult"" — we need just ClassA
            let (class_a, s_mult) = strip_class_and_mult(left);
            let (class_b, t_mult) = strip_class_and_mult(right);

            if class_a.is_empty() || class_b.is_empty() {
                continue;
            }

            let (from, to) = (class_a, class_b);
            let (sm, tm) = if *arrow_points_right {
                (t_mult, s_mult)
            } else {
                (s_mult, t_mult)
            };
            return Some((from, *edge_type, to, sm, tm, label));
        }
    }

    None
}

/// Find the colon that separates a relation from its label.
/// Skips colons inside quoted strings.
fn find_relation_colon(line: &str) -> Option<usize> {
    let mut in_quotes = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => in_quotes = !in_quotes,
            ':' if !in_quotes => return Some(i),
            _ => {}
        }
    }
    None
}

/// Extract `ClassName "mult" ... "mult" ClassName` structure.
/// Returns `(lhs_with_arrow, source_mult, rhs, target_mult)`.
fn extract_multiplicities(
    line: &str,
) -> Option<(String, Option<String>, String, Option<String>)> {
    // Try to find: `ClassName "mult" ARROW "mult" ClassName`
    // We look for quoted sections and extract them.
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_content = String::new();

    for c in line.chars() {
        if c == '"' {
            if in_quotes {
                // End of quote
                parts.push(current.clone());
                parts.push(format!("\"{}\"", quote_content));
                current.clear();
                quote_content.clear();
                in_quotes = false;
            } else {
                // Start of quote
                parts.push(current.clone());
                current.clear();
                in_quotes = true;
            }
        } else if in_quotes {
            quote_content.push(c);
        } else {
            current.push(c);
        }
    }
    parts.push(current);

    // Simplest case: no quotes → lhs = line, no multiplicities
    if parts.iter().all(|p| !p.starts_with('"')) {
        return Some((line.to_string(), None, String::new(), None));
    }

    // Expected structure: classA [smult] arrow [tmult] classB
    // parts alternates: non-quoted, quoted, non-quoted, quoted, non-quoted
    let non_quoted: Vec<&str> = parts
        .iter()
        .filter(|p| !p.starts_with('"'))
        .map(|p| p.trim())
        .collect();
    let quoted: Vec<&str> = parts
        .iter()
        .filter(|p| p.starts_with('"'))
        .map(|p| p.trim_matches('"'))
        .collect();

    // "ClassA "smult" ARROW "tmult" ClassB" — the only structured case we handle
    let (class_a, source_mult, arrow_part, target_mult, class_b) =
        if let (3, 2) = (non_quoted.len(), quoted.len()) {
            (
                non_quoted[0].to_string(),
                Some(quoted[0].to_string()),
                non_quoted[1].to_string(),
                Some(quoted[1].to_string()),
                non_quoted[2].to_string(),
            )
        } else {
            // (2,1) or any other pattern — fall back to simple (no multiplicity extracted)
            return Some((line.to_string(), None, String::new(), None));
        };

    if class_a.is_empty() || class_b.is_empty() {
        return Some((line.to_string(), None, String::new(), None));
    }

    // lhs = "classA ARROW" for the pattern matcher
    let lhs = format!("{} {}", class_a, arrow_part);
    Some((lhs, source_mult, class_b, target_mult))
}

/// Strip "ClassName "mult"" → (class, Some(mult)) or just "ClassName" → (class, None)
fn strip_class_and_mult(s: &str) -> (String, Option<String>) {
    let s = s.trim();
    if let Some(q_start) = s.find('"') {
        let class = s[..q_start].trim().to_string();
        if let Some(q_end) = s[q_start + 1..].find('"') {
            let mult = s[q_start + 1..q_start + 1 + q_end].to_string();
            return (class, Some(mult));
        }
    }
    (s.to_string(), None)
}

// ── Node creation ────────────────────────────────────────────────────

/// Ensure a class node exists in the graph. Creates with ClassBox shape if new.
fn ensure_class_node(graph: &mut Graph, node_map: &mut HashMap<String, usize>, id: &str) {
    if node_map.contains_key(id) {
        return;
    }
    let idx = graph.nodes.len();
    graph.nodes.push(Node {
        id: id.to_string(),
        label: id.to_string(),
        shape: NodeShape::ClassBox,
        width: 120.0,  // Will be recalculated
        height: 60.0,  // Will be recalculated
        ..Default::default()
    });
    node_map.insert(id.to_string(), idx);
}

/// Recalculate node widths and heights based on class content.
fn recalculate_class_node_sizes(graph: &mut Graph) {
    let char_width = 7.5_f64; // approx pixels per character
    let line_height = 18.0_f64;
    let header_height = 36.0_f64; // name + stereotype
    let padding_x = 20.0_f64;
    let min_width = 100.0_f64;

    for node in &mut graph.nodes {
        if node.shape != NodeShape::ClassBox {
            continue;
        }

        // Width: max of label, stereotype, all attr/method lines
        let mut max_chars = node.label.len();

        if let Some(ref stereo) = node.stereotype {
            let stereo_display = format!("<<{}>>", stereo);
            max_chars = max_chars.max(stereo_display.len());
        }

        for attr in &node.class_attributes {
            let display_len =
                1 + attr.attr_type.len() + 1 + attr.name.len(); // "+type name"
            max_chars = max_chars.max(display_len);
        }

        for method in &node.class_methods {
            let display_len =
                1 + method.return_type.len() + 1 + method.name.len() + 2 + method.params.len();
            max_chars = max_chars.max(display_len);
        }

        let computed_width = (max_chars as f64 * char_width + padding_x).max(min_width);
        node.width = text_metrics::calculate_text_width(&node.label)
            .max(computed_width)
            .max(min_width);

        // Height: header + separator + attrs + separator + methods
        let n_attrs = node.class_attributes.len();
        let n_methods = node.class_methods.len();
        let n_member_lines = (n_attrs + n_methods).max(1); // at least 1 empty line per compartment
        let compartment_lines = if n_attrs == 0 { 1 } else { n_attrs };
        let method_lines = if n_methods == 0 { 1 } else { n_methods };
        node.height = header_height
            + (compartment_lines as f64) * line_height
            + 2.0  // separator lines
            + (method_lines as f64) * line_height
            + padding_x;
        let _ = n_member_lines;
    }
}

// ── Unit tests ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer;

    fn parse(input: &str) -> Graph {
        let tokens = tokenizer::tokenize(input);
        parse_class_diagram(&tokens).expect("parse failed")
    }

    #[test]
    fn test_inheritance_relation() {
        let g = parse("classDiagram\n    Animal <|-- Dog\n");
        assert_eq!(g.nodes.len(), 2);
        assert_eq!(g.edges.len(), 1);
        let edge = &g.edges[0];
        assert_eq!(edge.from, "Animal");
        assert_eq!(edge.to, "Dog");
        assert_eq!(edge.class_edge_type, Some(ClassEdgeType::Inheritance));
    }

    #[test]
    fn test_inheritance_reversed() {
        let g = parse("classDiagram\n    Dog --|> Animal\n");
        assert_eq!(g.edges.len(), 1);
        let edge = &g.edges[0];
        assert_eq!(edge.from, "Dog");
        assert_eq!(edge.to, "Animal");
        assert_eq!(edge.class_edge_type, Some(ClassEdgeType::Inheritance));
    }

    #[test]
    fn test_composition_relation() {
        let g = parse("classDiagram\n    Car *-- Engine\n");
        assert_eq!(g.edges[0].class_edge_type, Some(ClassEdgeType::Composition));
    }

    #[test]
    fn test_aggregation_relation() {
        let g = parse("classDiagram\n    Team o-- Member\n");
        assert_eq!(g.edges[0].class_edge_type, Some(ClassEdgeType::Aggregation));
    }

    #[test]
    fn test_association_relation() {
        let g = parse("classDiagram\n    A --> B\n");
        assert_eq!(g.edges[0].class_edge_type, Some(ClassEdgeType::Association));
    }

    #[test]
    fn test_realization_relation() {
        let g = parse("classDiagram\n    Interface <|.. Implementation\n");
        assert_eq!(g.edges[0].class_edge_type, Some(ClassEdgeType::Realization));
        assert_eq!(g.edges[0].style, EdgeStyle::Dotted);
    }

    #[test]
    fn test_dependency_relation() {
        let g = parse("classDiagram\n    A ..> B\n");
        assert_eq!(g.edges[0].class_edge_type, Some(ClassEdgeType::Dependency));
    }

    #[test]
    fn test_link_relation() {
        let g = parse("classDiagram\n    A .. B\n");
        assert_eq!(g.edges[0].class_edge_type, Some(ClassEdgeType::Link));
    }

    #[test]
    fn test_relation_with_label() {
        let g = parse("classDiagram\n    A --> B : uses\n");
        assert_eq!(g.edges[0].label, Some("uses".to_string()));
    }

    #[test]
    fn test_member_attribute() {
        let g = parse("classDiagram\n    Animal : +String name\n");
        assert_eq!(g.nodes.len(), 1);
        let node = &g.nodes[0];
        assert_eq!(node.class_attributes.len(), 1);
        let attr = &node.class_attributes[0];
        assert_eq!(attr.visibility, ClassVisibility::Public);
        assert_eq!(attr.attr_type, "String");
        assert_eq!(attr.name, "name");
    }

    #[test]
    fn test_member_method() {
        let g = parse("classDiagram\n    Animal : +makeSound()\n");
        assert_eq!(g.nodes[0].class_methods.len(), 1);
        let method = &g.nodes[0].class_methods[0];
        assert_eq!(method.visibility, ClassVisibility::Public);
        assert_eq!(method.name, "makeSound");
    }

    #[test]
    fn test_private_attribute() {
        let g = parse("classDiagram\n    Foo : -int count\n");
        assert_eq!(g.nodes[0].class_attributes[0].visibility, ClassVisibility::Private);
    }

    #[test]
    fn test_protected_attribute() {
        let g = parse("classDiagram\n    Foo : #String data\n");
        assert_eq!(g.nodes[0].class_attributes[0].visibility, ClassVisibility::Protected);
    }

    #[test]
    fn test_stereotype_inline() {
        let g = parse("classDiagram\n    class Foo {\n        <<interface>>\n    }\n");
        // We need full multi-line parsing for this — just ensure node exists
        assert!(!g.nodes.is_empty());
    }

    #[test]
    fn test_multiplicity() {
        let g = parse("classDiagram\n    A \"1\" --> \"0..*\" B\n");
        // Should parse the relation at least
        assert_eq!(g.edges.len(), 1);
    }

    #[test]
    fn test_b12_fixture() {
        let input = "classDiagram\n    \
                     Animal <|-- Dog\n    \
                     Animal <|-- Cat\n    \
                     Animal : +String name\n    \
                     Animal : +int age\n    \
                     Animal : +makeSound()\n    \
                     Dog : +String breed\n    \
                     Dog : +bark()\n    \
                     Cat : +bool indoor\n    \
                     Cat : +meow()\n";
        let g = parse(input);
        assert_eq!(g.nodes.len(), 3); // Animal, Dog, Cat
        assert_eq!(g.edges.len(), 2); // 2 inheritance edges

        let animal = g.nodes.iter().find(|n| n.id == "Animal").unwrap();
        assert_eq!(animal.class_attributes.len(), 2); // name, age
        assert_eq!(animal.class_methods.len(), 1); // makeSound

        let dog = g.nodes.iter().find(|n| n.id == "Dog").unwrap();
        assert_eq!(dog.class_attributes.len(), 1); // breed
        assert_eq!(dog.class_methods.len(), 1); // bark
    }
}
