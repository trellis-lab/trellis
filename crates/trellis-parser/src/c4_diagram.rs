//! C4 diagram parser for Mermaid C4Context, C4Container, C4Component,
//! C4Dynamic, and C4Deployment diagrams.
//!
//! Supported syntax elements:
//! - `Person(alias, label[, description])`
//! - `Person_Ext(alias, label[, description])`
//! - `System(alias, label[, description])`
//! - `SystemDb(alias, label[, description])`
//! - `SystemQueue(alias, label[, description])`
//! - `System_Ext(alias, label[, description])` (and `SystemDb_Ext`, `SystemQueue_Ext`)
//! - `Container(alias, label[, technology[, description]])`
//! - `ContainerDb`, `ContainerQueue`, and `_Ext` variants
//! - `Component`, `ComponentDb`, `ComponentQueue`, and `_Ext` variants
//! - `Enterprise_Boundary(alias, label)`, `System_Boundary(alias, label)`,
//!   `Container_Boundary(alias, label)`, `Deployment_Node(alias, label)`
//! - `Rel(from, to, label[, technology])`
//! - `BiRel(from, to, label[, technology])`
//! - `Rel_U`, `Rel_D`, `Rel_L`, `Rel_R`, `Rel_Back`
//! - `UpdateLayoutConfig($c4ShapeInRow, $c4BoundaryInRow)` (ignored for layout)

use crate::{
    ast::{C4NodeType, C4RelType, DiagramType, Edge, Graph, Node, NodeShape, Subgraph},
    tokenizer::Token,
    ParseError,
};

// ── Sizing constants ──────────────────────────────────────────────────────────

/// Approx. character width for label width estimation (pixels)
const CHAR_WIDTH: f64 = 7.5;
/// Header height for a C4 element box (label line)
const HEADER_HEIGHT: f64 = 32.0;
/// Line height for description / technology rows
const LINE_HEIGHT: f64 = 18.0;
/// Horizontal padding inside the box
const PADDING_X: f64 = 16.0;
/// Vertical padding inside the box
const PADDING_Y: f64 = 10.0;
/// Minimum element width
const MIN_WIDTH: f64 = 200.0;
/// Minimum person node height
const MIN_PERSON_HEIGHT: f64 = 200.0;
/// Minimum default element height
const MIN_DEFAULT_HEIGHT: f64 = 100.0;
/// Approximate character width for description/technology text at 10 px (FONT_SIZE_DESC).
/// Must match CHAR_WIDTH_DESC in c4_shapes.rs.
const RENDER_CHAR_WIDTH_DESC: f64 = 5.5;
/// Approximate character width for 13 px label text (Person description).
/// Must match CHAR_WIDTH_LABEL in c4_shapes.rs.
const RENDER_CHAR_WIDTH_LABEL: f64 = 7.0;
/// Approximate character width for the Person caption (15 px bold).
/// Must match CHAR_WIDTH_CAPTION in c4_shapes.rs.
const RENDER_CHAR_WIDTH_CAPTION: f64 = 8.5;
/// Renderer line height used inside the Person box.
/// Must match `LINE_HEIGHT` in `c4_shapes.rs`.
const PERSON_RENDERER_LH: f64 = 14.0;
/// Maximum text lines per text block (caption or description) in a Person node.
/// Spec: "No text can have more than 5 lines."
const PERSON_MAX_LINES: usize = 5;
/// Base box height for a Person node with no description and a single-line caption.
///
/// Derived from the C4 v4 `render_person` text layout (20 px visual top gap):
///   top_offset(31) + caption_line(14) + gap(4) + type_visual_bottom(3) + bottom_gap(10) = 62
///   → rounded up to 63 for a comfortable fit.
///
/// - top_offset = 20 px visual gap + cap-height of 15 pt caption (15 × 0.75 ≈ 11)
/// - type_visual_bottom = 12 pt × 0.25 ≈ 3 px
/// - bottom_gap = 10 px (spec requirement)
const PERSON_BASE_BOX_H: f64 = 63.0;

// ── Public entry point ────────────────────────────────────────────────────────

/// Parse a C4 diagram from the token stream.
pub fn parse_c4_diagram(tokens: &[Token]) -> Result<Graph, ParseError> {
    let mut graph = Graph::new();
    graph.diagram_type = DiagramType::C4Diagram;

    let mut i = 0;
    // Skip the leading directive token
    if !tokens.is_empty() {
        i = 1;
    }

    // Stack of in-progress Subgraph objects (one per open boundary block).
    // The innermost open boundary is at the top (last element).
    let mut sg_stack: Vec<Subgraph> = Vec::new();

    while i < tokens.len() {
        let content = tokens[i].content.trim().to_string();
        let line = tokens[i].line_number;

        // Closing brace completes the innermost boundary
        if content == "}" {
            if let Some(finished_sg) = sg_stack.pop() {
                // Add to parent boundary, or to the top-level graph
                if let Some(parent) = sg_stack.last_mut() {
                    parent.subgraphs.push(finished_sg);
                } else {
                    graph.subgraphs.push(finished_sg);
                }
            }
            i += 1;
            continue;
        }

        if content.is_empty() {
            i += 1;
            continue;
        }

        // Detect whether this line opens a boundary block: it ends with `{`
        // after the closing paren, e.g. `System_Boundary(sb, "label") {`
        let opens_block = content.ends_with('{');
        // Strip the trailing `{` for clean parsing
        let clean = if opens_block {
            content[..content.len() - 1].trim().to_string()
        } else {
            content.clone()
        };

        // Try to parse each supported statement kind
        if let Some(node) = try_parse_element(&clean, line)? {
            let is_boundary = node.c4_type.map(|t| t.is_boundary()).unwrap_or(false);

            // If we're inside a boundary and this is a regular element, record it
            if !is_boundary {
                if let Some(current_sg) = sg_stack.last_mut() {
                    current_sg.nodes.push(node.id.clone());
                }
            }

            // If this line opens a block, push a new Subgraph onto the stack
            if opens_block && is_boundary {
                sg_stack.push(Subgraph {
                    id: node.id.clone(),
                    label: Some(node.label.clone()),
                    nodes: Vec::new(),
                    subgraphs: Vec::new(),
                });
            }

            // Avoid duplicate node IDs
            if !graph.nodes.iter().any(|n| n.id == node.id) {
                graph.nodes.push(node);
            }
        } else if let Some(edge) = try_parse_relation(&clean, line)? {
            // Ensure endpoint nodes exist (create placeholder if not)
            ensure_node(&mut graph, &edge.from);
            ensure_node(&mut graph, &edge.to);
            graph.edges.push(edge);
        }
        // UpdateLayoutConfig, UpdateElementStyle, UpdateRelStyle — silently ignored

        i += 1;
    }

    // Flush any unclosed boundary blocks (malformed input tolerance)
    while let Some(sg) = sg_stack.pop() {
        if let Some(parent) = sg_stack.last_mut() {
            parent.subgraphs.push(sg);
        } else {
            graph.subgraphs.push(sg);
        }
    }

    Ok(graph)
}

// ── Element parsing ───────────────────────────────────────────────────────────

/// Try to parse a C4 element definition from a single line.
/// Returns `Ok(Some(node))` on success, `Ok(None)` if the line is not an element.
fn try_parse_element(line: &str, line_no: usize) -> Result<Option<Node>, ParseError> {
    // Keyword is the part before the first `(`
    let paren_pos = match line.find('(') {
        Some(p) => p,
        None => return Ok(None),
    };

    let keyword = line[..paren_pos].trim();
    let c4_type = match keyword {
        "Person" => C4NodeType::Person,
        "Person_Ext" => C4NodeType::PersonExt,
        "System" => C4NodeType::System,
        "SystemDb" => C4NodeType::SystemDb,
        "SystemQueue" => C4NodeType::SystemQueue,
        "System_Ext" => C4NodeType::SystemExt,
        "SystemDb_Ext" => C4NodeType::SystemDbExt,
        "SystemQueue_Ext" => C4NodeType::SystemQueueExt,
        "Container" => C4NodeType::Container,
        "ContainerDb" => C4NodeType::ContainerDb,
        "ContainerQueue" => C4NodeType::ContainerQueue,
        "Container_Ext" => C4NodeType::ContainerExt,
        "ContainerDb_Ext" => C4NodeType::ContainerDbExt,
        "ContainerQueue_Ext" => C4NodeType::ContainerQueueExt,
        "Component" => C4NodeType::Component,
        "ComponentDb" => C4NodeType::ComponentDb,
        "ComponentQueue" => C4NodeType::ComponentQueue,
        "Component_Ext" => C4NodeType::ComponentExt,
        "ComponentDb_Ext" => C4NodeType::ComponentDbExt,
        "ComponentQueue_Ext" => C4NodeType::ComponentQueueExt,
        "Enterprise_Boundary" => C4NodeType::EnterpriseBoundary,
        "System_Boundary" => C4NodeType::SystemBoundary,
        "Container_Boundary" => C4NodeType::ContainerBoundary,
        "Deployment_Node" | "Node" | "Node_L" | "Node_R" => C4NodeType::DeploymentNode,
        _ => return Ok(None),
    };

    let args_raw = extract_args(line, paren_pos, line_no)?;
    let args = split_c4_args(&args_raw);

    // alias is always the first argument
    if args.is_empty() {
        return Err(ParseError {
            message: format!(
                "C4 element '{}' requires at least an alias argument",
                keyword
            ),
            line: line_no,
            column: 0,
        });
    }

    let alias = args[0].trim().to_string();
    let label = args
        .get(1)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| alias.clone());

    // For Container/Component: args are (alias, label, technology, description)
    // For System/Person: args are (alias, label, description)
    let (technology, description) = match c4_type {
        C4NodeType::Container
        | C4NodeType::ContainerDb
        | C4NodeType::ContainerQueue
        | C4NodeType::ContainerExt
        | C4NodeType::ContainerDbExt
        | C4NodeType::ContainerQueueExt
        | C4NodeType::Component
        | C4NodeType::ComponentDb
        | C4NodeType::ComponentQueue
        | C4NodeType::ComponentExt
        | C4NodeType::ComponentDbExt
        | C4NodeType::ComponentQueueExt => {
            let tech = args.get(2).map(|s| s.trim().to_string());
            let desc = args.get(3).map(|s| s.trim().to_string());
            (tech, desc)
        }
        _ => {
            let desc = args.get(2).map(|s| s.trim().to_string());
            (None, desc)
        }
    };

    let (width, height) = size_c4_node(c4_type, &label, &description, &technology);

    let node = Node {
        id: alias.clone(),
        label,
        shape: NodeShape::C4Box,
        width,
        height,
        c4_type: Some(c4_type),
        c4_description: description,
        c4_technology: technology,
        ..Default::default()
    };

    Ok(Some(node))
}

// ── Relation parsing ──────────────────────────────────────────────────────────

/// Try to parse a C4 relationship from a single line.
fn try_parse_relation(line: &str, line_no: usize) -> Result<Option<Edge>, ParseError> {
    let paren_pos = match line.find('(') {
        Some(p) => p,
        None => return Ok(None),
    };

    let keyword = line[..paren_pos].trim();
    let (rel_type, bidirectional) = match keyword {
        "Rel" => (C4RelType::Rel, false),
        "BiRel" => (C4RelType::Rel, true),
        "Rel_Back" => (C4RelType::RelBack, false),
        "Rel_U" | "Rel_Up" => (C4RelType::RelU, false),
        "Rel_D" | "Rel_Down" => (C4RelType::RelD, false),
        "Rel_L" | "Rel_Left" => (C4RelType::RelL, false),
        "Rel_R" | "Rel_Right" => (C4RelType::RelR, false),
        _ => return Ok(None),
    };

    let args_raw = extract_args(line, paren_pos, line_no)?;
    let args = split_c4_args(&args_raw);

    // Rel(from, to, label[, technology])
    if args.len() < 3 {
        return Err(ParseError {
            message: format!("Rel '{}' requires at least (from, to, label)", keyword),
            line: line_no,
            column: 0,
        });
    }

    let from = args[0].trim().to_string();
    let to = args[1].trim().to_string();
    let label = strip_quotes(args[2].trim());
    let technology = args.get(3).map(|s| strip_quotes(s.trim()));

    let edge = Edge {
        from,
        to,
        label: Some(label),
        c4_rel_type: Some(rel_type),
        c4_technology: technology,
        c4_bidirectional: bidirectional,
        ..Default::default()
    };

    Ok(Some(edge))
}

// ── Sizing ────────────────────────────────────────────────────────────────────

/// Calculate the pixel size of a C4 element node.
///
/// **Person** nodes have a fixed width (`MIN_WIDTH`) per the C4 v4 spec
/// ("The width of the elements are constant").  Caption and description are
/// word-wrapped into that width and both capped at `PERSON_MAX_LINES`.
///
/// **All other** C4 elements have a label-driven width (minimum `MIN_WIDTH`).
fn size_c4_node(
    c4_type: C4NodeType,
    label: &str,
    description: &Option<String>,
    technology: &Option<String>,
) -> (f64, f64) {
    // Person: constant width.  Everyone else: label-driven, minimum MIN_WIDTH.
    let label_width = label.len() as f64 * CHAR_WIDTH + PADDING_X * 2.0;
    let width = if c4_type.is_person() {
        MIN_WIDTH
    } else {
        label_width.max(MIN_WIDTH)
    };

    let height = if c4_type.is_person() {
        // ── Person geometry (C4 v4) ──────────────────────────────────────────
        // Head: radius = width/4.  The head overlaps the box top by 10 px, so
        //   head_space = 2×(width/4) − 10 = width/2 − 10
        let head_space = width / 2.0 - 10.0;

        // Caption (15 px bold) — use the caption-specific char width.
        let caption_cpl = ((width - PADDING_X * 2.0) / RENDER_CHAR_WIDTH_CAPTION).max(5.0) as usize;
        // PERSON_BASE_BOX_H already reserves one caption line; each extra line
        // adds PERSON_RENDERER_LH (14 px).
        let caption_lines = word_wrap_line_count(label, caption_cpl).min(PERSON_MAX_LINES);
        let caption_extra = caption_lines.saturating_sub(1) as f64 * PERSON_RENDERER_LH;

        // Description (13 px) — use the label char width.
        let desc_cpl = ((width - PADDING_X * 2.0) / RENDER_CHAR_WIDTH_LABEL).max(5.0) as usize;
        // Description word-wrap (4 px type→desc gap + lines × 14 px).
        let desc_extra = if let Some(desc) = description {
            let desc_lines = word_wrap_line_count(desc, desc_cpl).min(PERSON_MAX_LINES);
            4.0 + PERSON_RENDERER_LH * desc_lines as f64
        } else {
            0.0
        };

        (head_space + PERSON_BASE_BOX_H + caption_extra + desc_extra).max(MIN_PERSON_HEIGHT)
    } else {
        // ── All other C4 elements ─────────────────────────────────────────────
        // Height: header + optional tech row + word-wrapped description rows.
        let mut h = HEADER_HEIGHT;
        if technology.is_some() {
            h += LINE_HEIGHT;
        }
        if let Some(desc) = description {
            let chars_per_line =
                ((width - PADDING_X * 2.0) / RENDER_CHAR_WIDTH_DESC).max(5.0) as usize;
            let lines = word_wrap_line_count(desc, chars_per_line);
            h += LINE_HEIGHT * lines as f64;
        }
        h.max(MIN_DEFAULT_HEIGHT + PADDING_Y)
    };

    (width, height)
}

/// Count the number of lines produced by word-wrapping `text` at `chars_per_line`.
///
/// Uses the same word-break algorithm as `wrap_text` in `c4_shapes.rs` so that
/// the parser's height estimate exactly matches the rendered output.
fn word_wrap_line_count(text: &str, chars_per_line: usize) -> usize {
    let mut line_count = 0usize;
    let mut current_len = 0usize;
    for word in text.split_whitespace() {
        if current_len == 0 {
            current_len = word.len();
        } else if current_len + 1 + word.len() <= chars_per_line {
            current_len += 1 + word.len();
        } else {
            line_count += 1;
            current_len = word.len();
        }
    }
    if current_len > 0 {
        line_count += 1;
    }
    line_count.max(1)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Ensure a node with the given ID exists; insert a placeholder if not.
fn ensure_node(graph: &mut Graph, id: &str) {
    if !graph.nodes.iter().any(|n| n.id == id) {
        let (w, h) = size_c4_node(C4NodeType::System, id, &None, &None);
        graph.nodes.push(Node {
            id: id.to_string(),
            label: id.to_string(),
            shape: NodeShape::C4Box,
            width: w,
            height: h,
            c4_type: Some(C4NodeType::System),
            ..Default::default()
        });
    }
}

/// Extract the raw argument string from inside the first `(...)` on the line.
fn extract_args(line: &str, paren_pos: usize, line_no: usize) -> Result<String, ParseError> {
    let after_open = &line[paren_pos + 1..];
    // Find matching closing paren (handling nested parens)
    let mut depth = 1usize;
    let mut end = None;
    for (idx, ch) in after_open.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(idx);
                    break;
                }
            }
            _ => {}
        }
    }
    match end {
        Some(e) => Ok(after_open[..e].to_string()),
        None => Err(ParseError {
            message: "Unmatched '(' in C4 element".to_string(),
            line: line_no,
            column: paren_pos,
        }),
    }
}

/// Split a comma-separated C4 argument list, respecting quoted strings.
fn split_c4_args(args_raw: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '"';

    for ch in args_raw.chars() {
        match ch {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = ch;
                // Don't include the quote in the token
            }
            c if in_quotes && c == quote_char => {
                in_quotes = false;
            }
            ',' if !in_quotes => {
                args.push(current.trim().to_string());
                current = String::new();
            }
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        args.push(current.trim().to_string());
    }
    args
}

/// Strip surrounding quotes from a string slice.
fn strip_quotes(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::tokenize;

    fn parse_c4(input: &str) -> Graph {
        let tokens = tokenize(input);
        parse_c4_diagram(&tokens).expect("C4 parse failed")
    }

    #[test]
    fn test_parse_person() {
        let g = parse_c4("C4Context\n    Person(alice, \"Alice\", \"A user\")\n");
        assert_eq!(g.diagram_type, DiagramType::C4Diagram);
        let node = g
            .nodes
            .iter()
            .find(|n| n.id == "alice")
            .expect("alice not found");
        assert_eq!(node.label, "Alice");
        assert_eq!(node.c4_type, Some(C4NodeType::Person));
        assert_eq!(node.c4_description.as_deref(), Some("A user"));
    }

    #[test]
    fn test_parse_system() {
        let g = parse_c4("C4Context\n    System(myapp, \"My App\", \"The main system\")\n");
        let node = g
            .nodes
            .iter()
            .find(|n| n.id == "myapp")
            .expect("myapp not found");
        assert_eq!(node.c4_type, Some(C4NodeType::System));
    }

    #[test]
    fn test_parse_system_ext() {
        let g = parse_c4("C4Context\n    System_Ext(ext, \"External\")\n");
        let node = g
            .nodes
            .iter()
            .find(|n| n.id == "ext")
            .expect("ext not found");
        assert_eq!(node.c4_type, Some(C4NodeType::SystemExt));
        assert!(node.c4_type.unwrap().is_external());
    }

    #[test]
    fn test_parse_container_with_tech() {
        let g =
            parse_c4("C4Container\n    Container(api, \"API\", \"Rust/Actix\", \"REST API\")\n");
        let node = g
            .nodes
            .iter()
            .find(|n| n.id == "api")
            .expect("api not found");
        assert_eq!(node.c4_type, Some(C4NodeType::Container));
        assert_eq!(node.c4_technology.as_deref(), Some("Rust/Actix"));
        assert_eq!(node.c4_description.as_deref(), Some("REST API"));
    }

    #[test]
    fn test_parse_rel() {
        let g = parse_c4(
            "C4Context\n    Person(alice, \"Alice\")\n    System(app, \"App\")\n    Rel(alice, app, \"Uses\")\n",
        );
        assert_eq!(g.edges.len(), 1);
        let edge = &g.edges[0];
        assert_eq!(edge.from, "alice");
        assert_eq!(edge.to, "app");
        assert_eq!(edge.label.as_deref(), Some("Uses"));
        assert_eq!(edge.c4_rel_type, Some(C4RelType::Rel));
        assert!(!edge.c4_bidirectional);
    }

    #[test]
    fn test_parse_birel() {
        let g = parse_c4(
            "C4Context\n    Person(alice, \"Alice\")\n    System(app, \"App\")\n    BiRel(alice, app, \"Interacts\")\n",
        );
        let edge = &g.edges[0];
        assert!(edge.c4_bidirectional);
    }

    #[test]
    fn test_parse_rel_with_technology() {
        let g = parse_c4(
            "C4Context\n    System(a, \"A\")\n    System(b, \"B\")\n    Rel(a, b, \"Calls\", \"HTTPS\")\n",
        );
        let edge = &g.edges[0];
        assert_eq!(edge.c4_technology.as_deref(), Some("HTTPS"));
    }

    #[test]
    fn test_parse_rel_directional() {
        let g = parse_c4(
            "C4Context\n    System(a, \"A\")\n    System(b, \"B\")\n    Rel_U(a, b, \"Up\")\n",
        );
        let edge = &g.edges[0];
        assert_eq!(edge.c4_rel_type, Some(C4RelType::RelU));
    }

    #[test]
    fn test_parse_boundary() {
        let g = parse_c4(
            "C4Context\n    Enterprise_Boundary(eb, \"Enterprise\") {\n        System(app, \"App\")\n    }\n",
        );
        // Boundary node still appears in graph.nodes
        let boundary = g
            .nodes
            .iter()
            .find(|n| n.id == "eb")
            .expect("boundary not found");
        assert_eq!(boundary.c4_type, Some(C4NodeType::EnterpriseBoundary));
        assert!(boundary.c4_type.unwrap().is_boundary());
        // Containment is tracked in graph.subgraphs
        assert_eq!(g.subgraphs.len(), 1);
        assert_eq!(g.subgraphs[0].id, "eb");
        assert!(g.subgraphs[0].nodes.contains(&"app".to_string()));
    }

    #[test]
    fn test_parse_database() {
        let g = parse_c4(
            "C4Container\n    ContainerDb(db, \"Database\", \"PostgreSQL\", \"Stores data\")\n",
        );
        let node = g.nodes.iter().find(|n| n.id == "db").expect("db not found");
        assert_eq!(node.c4_type, Some(C4NodeType::ContainerDb));
        assert!(node.c4_type.unwrap().is_db());
    }

    #[test]
    fn test_split_c4_args_quoted() {
        let args = split_c4_args("alice, \"Alice Smith\", \"A user with, comma\"");
        assert_eq!(args.len(), 3);
        assert_eq!(args[1], "Alice Smith");
        assert_eq!(args[2], "A user with, comma");
    }

    #[test]
    fn test_ensure_placeholder_node_created() {
        let g = parse_c4("C4Context\n    Rel(alice, app, \"Uses\")\n");
        // Both alice and app should be created as placeholder nodes
        assert!(g.nodes.iter().any(|n| n.id == "alice"));
        assert!(g.nodes.iter().any(|n| n.id == "app"));
    }
}
