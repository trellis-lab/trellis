use crate::ast::*;
use crate::text_metrics;
use crate::tokenizer::{Token, TokenType};
use std::collections::HashMap;

/// Parse flowchart tokens into a Graph
pub fn parse_flowchart(tokens: &[Token]) -> Result<Graph, crate::ParseError> {
    let mut graph = Graph::new();
    graph.diagram_type = DiagramType::Flowchart;

    // Parse direction from directive
    for token in tokens {
        if token.token_type == TokenType::Directive {
            graph.direction = parse_direction(&token.content);
            break;
        }
    }

    let mut node_map: HashMap<String, usize> = HashMap::new();
    let mut subgraph_stack: Vec<Subgraph> = Vec::new();

    for token in tokens {
        match token.token_type {
            TokenType::Directive => continue,

            TokenType::SubgraphStart => {
                let sg = parse_subgraph_start(&token.content);
                subgraph_stack.push(sg);
            }

            TokenType::SubgraphEnd => {
                if let Some(finished) = subgraph_stack.pop() {
                    if let Some(parent) = subgraph_stack.last_mut() {
                        parent.subgraphs.push(finished);
                    } else {
                        graph.subgraphs.push(finished);
                    }
                }
            }

            TokenType::Statement => {
                parse_statement(
                    &token.content,
                    &mut graph,
                    &mut node_map,
                    &mut subgraph_stack,
                    token.line_number,
                )?;
            }
        }
    }

    // Close any unclosed subgraphs
    while let Some(finished) = subgraph_stack.pop() {
        if let Some(parent) = subgraph_stack.last_mut() {
            parent.subgraphs.push(finished);
        } else {
            graph.subgraphs.push(finished);
        }
    }

    Ok(graph)
}

/// Extract direction from a directive like "graph TB" or "flowchart LR"
fn parse_direction(directive: &str) -> Direction {
    let parts: Vec<&str> = directive.split_whitespace().collect();
    if parts.len() >= 2 {
        match parts[1].to_uppercase().as_str() {
            "TB" | "TD" => Direction::TB,
            "BT" => Direction::BT,
            "LR" => Direction::LR,
            "RL" => Direction::RL,
            _ => Direction::TB,
        }
    } else {
        Direction::TB
    }
}

/// Parse a subgraph start line: `subgraph ID [label]` or `subgraph ID label text`
fn parse_subgraph_start(line: &str) -> Subgraph {
    let rest = line
        .strip_prefix("subgraph")
        .or_else(|| line.strip_prefix("Subgraph"))
        .or_else(|| line.strip_prefix("SUBGRAPH"))
        .unwrap_or(line)
        .trim();

    let (id, label) = if rest.is_empty() {
        ("unnamed".to_string(), None)
    } else if let Some(bracket_start) = rest.find('[') {
        let id = rest[..bracket_start].trim().to_string();
        let label = rest
            .find(']')
            .map(|bracket_end| rest[bracket_start + 1..bracket_end].to_string());
        (id, label)
    } else {
        let mut parts = rest.splitn(2, ' ');
        let id = parts.next().unwrap_or("").to_string();
        let label = parts.next().map(|s| s.trim().to_string());
        (id, label)
    };

    Subgraph {
        id,
        label,
        nodes: Vec::new(),
        subgraphs: Vec::new(),
    }
}

// ── Node reference parsing ──────────────────────────────────────────

/// A parsed node reference from a statement
struct NodeRef {
    id: String,
    label: Option<String>,
    shape: Option<NodeShape>,
}

/// Maps a node shape specifier to a `NodeShape`.
/// Each entry is `(open_delimiter, close_delimiter, shape)`.
/// `parse_node_ref` sorts these by open-delimiter length (descending) at runtime,
/// so the declaration order here does not matter.
const NODE_SHAPES: &[(&str, &str, NodeShape)] = &[
    ("(((", ")))", NodeShape::DoubleCircle),
    ("((", "))", NodeShape::Circle),
    ("([", "])", NodeShape::Stadium),
    ("{{", "}}", NodeShape::Hexagon),
    ("[[", "]]", NodeShape::Subroutine),
    ("[(", ")]", NodeShape::Cylinder),
    ("[/", "/]", NodeShape::Parallelogram),
    ("[\\", "\\]", NodeShape::ParallelogramAlt),
    ("[/", "\\]", NodeShape::Trapezoid),
    ("[\\", "/]", NodeShape::TrapezoidAlt),
    ("[", "]", NodeShape::Rectangle),
    ("(", ")", NodeShape::RoundedRectangle),
    ("{", "}", NodeShape::Diamond),
    (">", "]", NodeShape::Asymmetric),
    // TODO: Add custom shape string support - Future release
];

/// Parse a node reference: `ID`, `ID[label]`, `ID(label)`, `ID{label}`,
/// `ID((label))`, or `ID{{label}}`
fn parse_node_ref(input: &str) -> Option<(NodeRef, &str)> {
    let input = input.trim_start();
    if input.is_empty() {
        return None;
    }

    // Parse ID: starts with letter/underscore, continues with alphanumeric/underscore/hyphen
    let id_end = input
        .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .unwrap_or(input.len());

    if id_end == 0 {
        return None;
    }

    let id = &input[..id_end];
    let rest = &input[id_end..];

    // Sort by open-delimiter length descending so that longer (more-specific)
    // delimiters like "((" are always tried before shorter ones like "(".
    let mut sorted_shapes = NODE_SHAPES.to_vec();
    sorted_shapes.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    for (open, close, shape) in sorted_shapes {
        if let Some(inner) = rest.strip_prefix(open) {
            if let Some(close_pos) = inner.find(close) {
                let label = strip_quotes(inner[..close_pos].trim());
                return Some((
                    NodeRef {
                        id: id.to_string(),
                        label: Some(label),
                        shape: Some(shape),
                    },
                    &rest[open.len() + close_pos + close.len()..],
                ));
            }
        }
    }

    // No shape → just the ID
    Some((
        NodeRef {
            id: id.to_string(),
            label: None,
            shape: None,
        },
        rest,
    ))
}

/// Strip surrounding double quotes from a label
fn strip_quotes(s: &str) -> String {
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

// ── Arrow parsing ───────────────────────────────────────────────────

/// Arrow information extracted from parsing
struct ArrowInfo {
    style: EdgeStyle,
    arrow_head: ArrowHead,
    label: Option<String>,
}

/// Parse an arrow pattern and return the ArrowInfo + remaining input.
///
/// Supports: `-->`, `---`, `-.->`, `-.-`, `==>`, `===`,
/// `--text-->`, `-->|text|`, `--|text|-->`, and analogous forms.
fn parse_arrow(input: &str) -> Option<(ArrowInfo, &str)> {
    let input = input.trim_start();

    // ── Dotted arrows ──

    // -.-> (dotted arrow)
    if let Some(rest) = input.strip_prefix("-.->") {
        let (label, rest) = parse_pipe_label(rest);
        return Some((
            ArrowInfo {
                style: EdgeStyle::Dotted,
                arrow_head: ArrowHead::Arrow,
                label,
            },
            rest,
        ));
    }

    // -.- (dotted, no arrow) — but not if it continues as -.->
    if input.starts_with("-.-") && !input.starts_with("-.->") {
        let rest = &input[3..];
        let (label, rest) = parse_pipe_label(rest);
        return Some((
            ArrowInfo {
                style: EdgeStyle::Dotted,
                arrow_head: ArrowHead::None,
                label,
            },
            rest,
        ));
    }

    // -.text.-> (dotted arrow with inline label)
    if input.starts_with("-.") && !input.starts_with("-.->") && !input.starts_with("-.-") {
        if let Some(pos) = input[2..].find(".->") {
            let label_text = strip_pipes_str(input[2..2 + pos].trim());
            let rest = &input[2 + pos + 3..];
            return Some((
                ArrowInfo {
                    style: EdgeStyle::Dotted,
                    arrow_head: ArrowHead::Arrow,
                    label: Some(label_text),
                },
                rest,
            ));
        }
    }

    // ── Thick arrows ──

    // ==> (thick arrow)
    if let Some(rest) = input.strip_prefix("==>") {
        let (label, rest) = parse_pipe_label(rest);
        return Some((
            ArrowInfo {
                style: EdgeStyle::Thick,
                arrow_head: ArrowHead::Arrow,
                label,
            },
            rest,
        ));
    }

    // === (thick, no arrow) — but not if it continues as ===>
    if input.starts_with("===") && !input[3..].starts_with('>') {
        let rest = &input[3..];
        let (label, rest) = parse_pipe_label(rest);
        return Some((
            ArrowInfo {
                style: EdgeStyle::Thick,
                arrow_head: ArrowHead::None,
                label,
            },
            rest,
        ));
    }

    // ==text==> (thick arrow with inline label)
    if input.starts_with("==") && !input.starts_with("==>") && !input.starts_with("===") {
        if let Some(pos) = input[2..].find("==>") {
            let label_text = strip_pipes_str(input[2..2 + pos].trim());
            let rest = &input[2 + pos + 3..];
            return Some((
                ArrowInfo {
                    style: EdgeStyle::Thick,
                    arrow_head: ArrowHead::Arrow,
                    label: Some(label_text),
                },
                rest,
            ));
        }
    }

    // ── Solid arrows ──

    // --> (solid arrow)
    if let Some(rest) = input.strip_prefix("-->") {
        let (label, rest) = parse_pipe_label(rest);
        return Some((
            ArrowInfo {
                style: EdgeStyle::Solid,
                arrow_head: ArrowHead::Arrow,
                label,
            },
            rest,
        ));
    }

    // --- (solid, no arrow) — but not if followed by - or >
    if input.starts_with("---") && !input[3..].starts_with('-') && !input[3..].starts_with('>') {
        let rest = &input[3..];
        let (label, rest) = parse_pipe_label(rest);
        return Some((
            ArrowInfo {
                style: EdgeStyle::Solid,
                arrow_head: ArrowHead::None,
                label,
            },
            rest,
        ));
    }

    // --text--> or --|text|--> (solid arrow with label)
    if input.starts_with("--") && !input.starts_with("-->") && !input.starts_with("---") {
        // Look for --> ending
        if let Some(pos) = input[2..].find("-->") {
            let label_text = strip_pipes_str(input[2..2 + pos].trim());
            let rest = &input[2 + pos + 3..];
            return Some((
                ArrowInfo {
                    style: EdgeStyle::Solid,
                    arrow_head: ArrowHead::Arrow,
                    label: Some(label_text),
                },
                rest,
            ));
        }
        // Look for --- ending
        if let Some(pos) = input[2..].find("---") {
            let label_text = strip_pipes_str(input[2..2 + pos].trim());
            let rest = &input[2 + pos + 3..];
            return Some((
                ArrowInfo {
                    style: EdgeStyle::Solid,
                    arrow_head: ArrowHead::None,
                    label: Some(label_text),
                },
                rest,
            ));
        }
    }

    None
}

/// Try to parse `|label|` immediately after an arrow
fn parse_pipe_label(input: &str) -> (Option<String>, &str) {
    if let Some(input_line) = input.strip_prefix('|') {
        if let Some(close) = input_line.find('|') {
            let label = input[1..1 + close].to_string();
            return (Some(label), &input[1 + close + 1..]);
        }
    }
    (None, input)
}

/// Strip pipe delimiters: `|text|` → `text`, otherwise return as-is
fn strip_pipes_str(s: &str) -> String {
    if s.len() >= 2 && s.starts_with('|') && s.ends_with('|') {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

// ── Statement parsing ───────────────────────────────────────────────

/// Keywords to skip (style/class directives, not node/edge definitions)
const SKIP_PREFIXES: &[&str] = &[
    "style ",
    "classDef ",
    "classdef ",
    "class ",
    "click ",
    "linkStyle ",
    "linkstyle ",
];

/// Parse a statement line into nodes and edges.
///
/// A statement is a chain: `NodeRef --> NodeRef --> NodeRef ...`
fn parse_statement(
    content: &str,
    graph: &mut Graph,
    node_map: &mut HashMap<String, usize>,
    subgraph_stack: &mut [Subgraph],
    line_number: usize,
) -> Result<(), crate::ParseError> {
    let trimmed = content.trim();

    // Skip style/class/click directives
    let lower = trimmed.to_lowercase();
    for prefix in SKIP_PREFIXES {
        if lower.starts_with(prefix) {
            return Ok(());
        }
    }

    // Parse first node
    let (first_ref, mut remaining) = match parse_node_ref(trimmed) {
        Some(result) => result,
        None => {
            return Err(crate::ParseError {
                message: format!("Expected node definition, got: {}", trimmed),
                line: line_number,
                column: 1,
            });
        }
    };

    let first_id = ensure_node(graph, node_map, &first_ref, subgraph_stack);

    // Parse chain: (arrow + node)*
    let mut prev_id = first_id;

    loop {
        remaining = remaining.trim_start();
        if remaining.is_empty() {
            break;
        }

        // Try to parse an arrow
        let (arrow, rest) = match parse_arrow(remaining) {
            Some(result) => result,
            None => break,
        };
        remaining = rest;

        // Parse next node
        let (next_ref, rest) = match parse_node_ref(remaining) {
            Some(result) => result,
            None => {
                return Err(crate::ParseError {
                    message: format!("Expected node after arrow at line {}", line_number),
                    line: line_number,
                    column: 1,
                });
            }
        };
        remaining = rest;

        let next_id = ensure_node(graph, node_map, &next_ref, subgraph_stack);

        // Create edge
        graph.edges.push(Edge {
            from: prev_id.clone(),
            to: next_id.clone(),
            label: arrow.label,
            style: arrow.style,
            arrow_head: arrow.arrow_head,
        });

        prev_id = next_id;
    }

    Ok(())
}

/// Ensure a node exists in the graph. Creates it if new, updates shape/label
/// if re-defined. Adds to the current subgraph context.
fn ensure_node(
    graph: &mut Graph,
    node_map: &mut HashMap<String, usize>,
    node_ref: &NodeRef,
    subgraph_stack: &mut [Subgraph],
) -> String {
    let id = node_ref.id.clone();

    if let Some(&idx) = node_map.get(&id) {
        // Node exists — update only if new info is provided
        if let Some(ref label) = node_ref.label {
            graph.nodes[idx].label = label.clone();
            graph.nodes[idx].width = text_metrics::calculate_text_width(label);
            graph.nodes[idx].height = text_metrics::calculate_text_height(label);
        }
        if let Some(shape) = node_ref.shape {
            graph.nodes[idx].shape = shape;
        }
    } else {
        // New node
        let label = node_ref.label.clone().unwrap_or_else(|| id.clone());
        let width = text_metrics::calculate_text_width(&label);
        let height = text_metrics::calculate_text_height(&label);
        let shape = node_ref.shape.unwrap_or(NodeShape::Rectangle);

        graph.nodes.push(Node {
            id: id.clone(),
            label,
            shape,
            width,
            height,
            x: 0.0,
            y: 0.0,
        });
        node_map.insert(id.clone(), graph.nodes.len() - 1);
    }

    // Add to current (deepest) subgraph if inside one
    if let Some(sg) = subgraph_stack.last_mut() {
        if !sg.nodes.contains(&id) {
            sg.nodes.push(id.clone());
        }
    }

    id
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::tokenize;

    #[test]
    fn test_parse_direction() {
        assert_eq!(parse_direction("graph TB"), Direction::TB);
        assert_eq!(parse_direction("graph TD"), Direction::TB);
        assert_eq!(parse_direction("flowchart LR"), Direction::LR);
        assert_eq!(parse_direction("graph BT"), Direction::BT);
        assert_eq!(parse_direction("graph RL"), Direction::RL);
    }

    #[test]
    fn test_parse_node_ref_plain() {
        let (node, rest) = parse_node_ref("A --> B").unwrap();
        assert_eq!(node.id, "A");
        assert!(node.label.is_none());
        assert!(node.shape.is_none());
        assert_eq!(rest, " --> B");
    }

    #[test]
    fn test_parse_node_ref_rectangle() {
        let (node, rest) = parse_node_ref("A[My Label] --> B").unwrap();
        assert_eq!(node.id, "A");
        assert_eq!(node.label.as_deref(), Some("My Label"));
        assert_eq!(node.shape, Some(NodeShape::Rectangle));
        assert_eq!(rest, " --> B");
    }

    #[test]
    fn test_parse_node_ref_rounded() {
        let (node, _) = parse_node_ref("A(Rounded Label)").unwrap();
        assert_eq!(node.id, "A");
        assert_eq!(node.label.as_deref(), Some("Rounded Label"));
        assert_eq!(node.shape, Some(NodeShape::RoundedRectangle));
    }

    #[test]
    fn test_parse_node_ref_diamond() {
        let (node, _) = parse_node_ref("A{Decision}").unwrap();
        assert_eq!(node.id, "A");
        assert_eq!(node.label.as_deref(), Some("Decision"));
        assert_eq!(node.shape, Some(NodeShape::Diamond));
    }

    #[test]
    fn test_parse_node_ref_circle() {
        let (node, _) = parse_node_ref("A((Circle))").unwrap();
        assert_eq!(node.id, "A");
        assert_eq!(node.label.as_deref(), Some("Circle"));
        assert_eq!(node.shape, Some(NodeShape::Circle));
    }

    #[test]
    fn test_parse_node_ref_hexagon() {
        let (node, _) = parse_node_ref("A{{Hexagon}}").unwrap();
        assert_eq!(node.id, "A");
        assert_eq!(node.label.as_deref(), Some("Hexagon"));
        assert_eq!(node.shape, Some(NodeShape::Hexagon));
    }

    #[test]
    fn test_parse_node_ref_cylinder() {
        let (node, rest) = parse_node_ref("A[(Database)] --> B").unwrap();
        assert_eq!(node.id, "A");
        assert_eq!(node.label.as_deref(), Some("Database"));
        assert_eq!(node.shape, Some(NodeShape::Cylinder));
        assert_eq!(rest, " --> B");
    }

    #[test]
    fn test_parse_node_ref_stadium() {
        let (node, _) = parse_node_ref("A([Stadium])").unwrap();
        assert_eq!(node.shape, Some(NodeShape::Stadium));
        assert_eq!(node.label.as_deref(), Some("Stadium"));
    }

    #[test]
    fn test_parse_node_ref_subroutine() {
        let (node, _) = parse_node_ref("A[[Sub]]").unwrap();
        assert_eq!(node.shape, Some(NodeShape::Subroutine));
        assert_eq!(node.label.as_deref(), Some("Sub"));
    }

    #[test]
    fn test_parse_node_ref_asymmetric() {
        let (node, _) = parse_node_ref("A>Flag]").unwrap();
        assert_eq!(node.shape, Some(NodeShape::Asymmetric));
        assert_eq!(node.label.as_deref(), Some("Flag"));
    }

    #[test]
    fn test_parse_node_ref_double_circle() {
        let (node, _) = parse_node_ref("A(((DC)))").unwrap();
        assert_eq!(node.shape, Some(NodeShape::DoubleCircle));
        assert_eq!(node.label.as_deref(), Some("DC"));
    }

    #[test]
    fn test_parse_node_ref_parallelogram() {
        let (node, _) = parse_node_ref("A[/Para/]").unwrap();
        assert_eq!(node.shape, Some(NodeShape::Parallelogram));
        assert_eq!(node.label.as_deref(), Some("Para"));
    }

    #[test]
    fn test_parse_node_ref_parallelogram_alt() {
        let (node, _) = parse_node_ref("A[\\Para\\]").unwrap();
        assert_eq!(node.shape, Some(NodeShape::ParallelogramAlt));
        assert_eq!(node.label.as_deref(), Some("Para"));
    }

    #[test]
    fn test_parse_node_ref_trapezoid() {
        let (node, _) = parse_node_ref("A[/Trap\\]").unwrap();
        assert_eq!(node.shape, Some(NodeShape::Trapezoid));
        assert_eq!(node.label.as_deref(), Some("Trap"));
    }

    #[test]
    fn test_parse_node_ref_trapezoid_alt() {
        let (node, _) = parse_node_ref("A[\\Trap/]").unwrap();
        assert_eq!(node.shape, Some(NodeShape::TrapezoidAlt));
        assert_eq!(node.label.as_deref(), Some("Trap"));
    }

    #[test]
    fn test_parse_arrow_solid() {
        let (arrow, rest) = parse_arrow("-->B").unwrap();
        assert_eq!(arrow.style, EdgeStyle::Solid);
        assert_eq!(arrow.arrow_head, ArrowHead::Arrow);
        assert!(arrow.label.is_none());
        assert_eq!(rest, "B");
    }

    #[test]
    fn test_parse_arrow_solid_no_head() {
        let (arrow, rest) = parse_arrow("--- B").unwrap();
        assert_eq!(arrow.style, EdgeStyle::Solid);
        assert_eq!(arrow.arrow_head, ArrowHead::None);
        assert!(arrow.label.is_none());
        assert_eq!(rest, " B");
    }

    #[test]
    fn test_parse_arrow_dotted() {
        let (arrow, _) = parse_arrow("-.-> B").unwrap();
        assert_eq!(arrow.style, EdgeStyle::Dotted);
        assert_eq!(arrow.arrow_head, ArrowHead::Arrow);
    }

    #[test]
    fn test_parse_arrow_thick() {
        let (arrow, _) = parse_arrow("==> B").unwrap();
        assert_eq!(arrow.style, EdgeStyle::Thick);
        assert_eq!(arrow.arrow_head, ArrowHead::Arrow);
    }

    #[test]
    fn test_parse_arrow_with_pipe_label() {
        let (arrow, rest) = parse_arrow("-->|Yes| B").unwrap();
        assert_eq!(arrow.style, EdgeStyle::Solid);
        assert_eq!(arrow.label.as_deref(), Some("Yes"));
        assert_eq!(rest, " B");
    }

    #[test]
    fn test_parse_arrow_with_inline_label() {
        let (arrow, rest) = parse_arrow("--text--> B").unwrap();
        assert_eq!(arrow.style, EdgeStyle::Solid);
        assert_eq!(arrow.arrow_head, ArrowHead::Arrow);
        assert_eq!(arrow.label.as_deref(), Some("text"));
        assert_eq!(rest, " B");
    }

    #[test]
    fn test_b01_linear_chain() {
        let input = "graph TB\n    A --> B\n    B --> C\n    C --> D\n    D --> E\n";
        let tokens = tokenize(input);
        let graph = parse_flowchart(&tokens).unwrap();

        assert_eq!(graph.direction, Direction::TB);
        assert_eq!(graph.diagram_type, DiagramType::Flowchart);
        assert_eq!(graph.nodes.len(), 5);
        assert_eq!(graph.edges.len(), 4);

        assert_eq!(graph.nodes[0].id, "A");
        assert_eq!(graph.nodes[4].id, "E");
        assert_eq!(graph.edges[0].from, "A");
        assert_eq!(graph.edges[0].to, "B");
        assert_eq!(graph.edges[3].from, "D");
        assert_eq!(graph.edges[3].to, "E");
    }

    #[test]
    fn test_b02_wide_branch() {
        let input = "graph TB\n    A --> B1\n    A --> B2\n    A --> B3\n    A --> B4\n    A --> B5\n    A --> B6\n";
        let tokens = tokenize(input);
        let graph = parse_flowchart(&tokens).unwrap();

        assert_eq!(graph.nodes.len(), 7);
        assert_eq!(graph.edges.len(), 6);

        // All edges originate from A
        for edge in &graph.edges {
            assert_eq!(edge.from, "A");
        }
    }

    #[test]
    fn test_b04_diamond() {
        let input = "graph TB\n    A --> B\n    A --> C\n    B --> D\n    C --> D\n";
        let tokens = tokenize(input);
        let graph = parse_flowchart(&tokens).unwrap();

        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(graph.edges.len(), 4);
    }

    #[test]
    fn test_b07_cycle() {
        let input = "graph TB\n    A --> B\n    B --> C\n    C --> D\n    D --> A\n";
        let tokens = tokenize(input);
        let graph = parse_flowchart(&tokens).unwrap();

        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(graph.edges.len(), 4);

        // Verify the cycle: last edge goes D → A
        assert_eq!(graph.edges[3].from, "D");
        assert_eq!(graph.edges[3].to, "A");
    }

    #[test]
    fn test_b08_nested_subgraph() {
        let input = "graph TB\n    A --> B\n    subgraph S1\n        B --> C\n        subgraph S2\n            C --> D\n        end\n    end\n    D --> E\n";
        let tokens = tokenize(input);
        let graph = parse_flowchart(&tokens).unwrap();

        assert_eq!(graph.nodes.len(), 5);
        assert_eq!(graph.edges.len(), 4);

        // One top-level subgraph: S1
        assert_eq!(graph.subgraphs.len(), 1);
        assert_eq!(graph.subgraphs[0].id, "S1");

        // S1 contains B, C, and has nested S2
        assert!(graph.subgraphs[0].nodes.contains(&"B".to_string()));
        assert!(graph.subgraphs[0].nodes.contains(&"C".to_string()));
        assert_eq!(graph.subgraphs[0].subgraphs.len(), 1);

        // S2 contains C, D
        let s2 = &graph.subgraphs[0].subgraphs[0];
        assert_eq!(s2.id, "S2");
        assert!(s2.nodes.contains(&"C".to_string()));
        assert!(s2.nodes.contains(&"D".to_string()));
    }

    #[test]
    fn test_node_shapes() {
        let input = "graph TB\n    A[Rectangle] --> B(Rounded)\n    B --> C{Diamond}\n    C --> D((Circle))\n    D --> E{{Hexagon}}\n";
        let tokens = tokenize(input);
        let graph = parse_flowchart(&tokens).unwrap();

        assert_eq!(graph.nodes.len(), 5);
        assert_eq!(graph.nodes[0].shape, NodeShape::Rectangle);
        assert_eq!(graph.nodes[0].label, "Rectangle");
        assert_eq!(graph.nodes[1].shape, NodeShape::RoundedRectangle);
        assert_eq!(graph.nodes[2].shape, NodeShape::Diamond);
        assert_eq!(graph.nodes[3].shape, NodeShape::Circle);
        assert_eq!(graph.nodes[4].shape, NodeShape::Hexagon);
    }

    #[test]
    fn test_edge_styles() {
        let input = "graph TB\n    A --> B\n    B -.-> C\n    C ==> D\n";
        let tokens = tokenize(input);
        let graph = parse_flowchart(&tokens).unwrap();

        assert_eq!(graph.edges[0].style, EdgeStyle::Solid);
        assert_eq!(graph.edges[0].arrow_head, ArrowHead::Arrow);
        assert_eq!(graph.edges[1].style, EdgeStyle::Dotted);
        assert_eq!(graph.edges[1].arrow_head, ArrowHead::Arrow);
        assert_eq!(graph.edges[2].style, EdgeStyle::Thick);
        assert_eq!(graph.edges[2].arrow_head, ArrowHead::Arrow);
    }

    #[test]
    fn test_edge_labels() {
        let input = "graph TB\n    A -->|Yes| B\n    B --No--> C\n";
        let tokens = tokenize(input);
        let graph = parse_flowchart(&tokens).unwrap();

        assert_eq!(graph.edges[0].label.as_deref(), Some("Yes"));
        assert_eq!(graph.edges[1].label.as_deref(), Some("No"));
    }

    #[test]
    fn test_chained_edges() {
        let input = "graph TB\n    A --> B --> C --> D\n";
        let tokens = tokenize(input);
        let graph = parse_flowchart(&tokens).unwrap();

        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(graph.edges.len(), 3);
        assert_eq!(graph.edges[0].from, "A");
        assert_eq!(graph.edges[0].to, "B");
        assert_eq!(graph.edges[1].from, "B");
        assert_eq!(graph.edges[1].to, "C");
        assert_eq!(graph.edges[2].from, "C");
        assert_eq!(graph.edges[2].to, "D");
    }

    #[test]
    fn test_text_metrics_set() {
        let input = "graph TB\n    A[Hello World] --> B\n";
        let tokens = tokenize(input);
        let graph = parse_flowchart(&tokens).unwrap();

        // A has label "Hello World" → width/height should be calculated
        assert!(graph.nodes[0].width > 0.0);
        assert!(graph.nodes[0].height > 0.0);
        // B has implicit label "B"
        assert!(graph.nodes[1].width > 0.0);
        assert!(graph.nodes[1].height > 0.0);
    }

    #[test]
    fn test_skip_style_directives() {
        let input = "graph TB\n    A --> B\n    style A fill:#f9f\n";
        let tokens = tokenize(input);
        let graph = parse_flowchart(&tokens).unwrap();
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
    }
}
