pub mod ast;
pub mod c4_diagram;
pub mod class_diagram;
pub mod er_diagram;
pub mod flowchart;
pub mod text_metrics;
pub mod tokenizer;

pub use ast::*;

/// Parse a Mermaid diagram from a string into a Graph AST.
///
/// Detects the diagram type (flowchart, classDiagram, erDiagram) from
/// the first directive and dispatches to the appropriate parser.
/// YAML frontmatter (`--- title: ... ---`) is stripped before tokenizing;
/// the `title` field is extracted and stored on the returned `Graph`.
pub fn parse(input: &str) -> Result<Graph, ParseError> {
    let (title, body) = extract_frontmatter(input);
    let tokens = tokenizer::tokenize(body);
    let diagram_type = tokenizer::detect_diagram_type(&tokens);

    let mut graph = match diagram_type {
        DiagramType::Flowchart => flowchart::parse_flowchart(&tokens)?,
        DiagramType::ClassDiagram => class_diagram::parse_class_diagram(&tokens)?,
        DiagramType::ErDiagram => er_diagram::parse_er_diagram(&tokens)?,
        DiagramType::C4Diagram => c4_diagram::parse_c4_diagram(&tokens)?,
    };
    graph.title = title;
    Ok(graph)
}

/// Extract optional YAML frontmatter. Returns `(title, diagram_body)`.
///
/// Recognises only the `title:` key; all other keys are silently ignored.
/// Returns `(None, input)` unchanged when no valid frontmatter block is found.
fn extract_frontmatter(input: &str) -> (Option<String>, &str) {
    let trimmed = input.trim_start();
    if !trimmed.starts_with("---") {
        return (None, input);
    }
    let after_open = match trimmed.find('\n') {
        Some(pos) => &trimmed[pos + 1..],
        None => return (None, input),
    };
    // Locate the closing "---" marker (must be on its own line).
    let close_marker = "\n---";
    match after_open.find(close_marker) {
        None => (None, input),
        Some(close_pos) => {
            let frontmatter = &after_open[..close_pos];
            let body = &after_open[close_pos + close_marker.len()..];
            let title = parse_title_from_frontmatter(frontmatter);
            (title, body)
        }
    }
}

/// Scan frontmatter lines for `title: <value>`.
fn parse_title_from_frontmatter(fm: &str) -> Option<String> {
    for line in fm.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("title:") {
            let value = rest.trim().to_string();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}

/// Error type for parsing failures
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Parse error at {}:{}: {}",
            self.line, self.column, self.message
        )
    }
}

impl std::error::Error for ParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_flowchart() {
        let result = parse("graph TB\n    A --> B\n    B --> C\n");
        assert!(result.is_ok());
        let graph = result.unwrap();
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);
        assert_eq!(graph.diagram_type, DiagramType::Flowchart);
        assert_eq!(graph.direction, Direction::TB);
    }

    #[test]
    fn test_parse_empty_returns_empty_graph() {
        let result = parse("");
        assert!(result.is_ok());
        let graph = result.unwrap();
        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(graph.edges.len(), 0);
    }

    #[test]
    fn test_parse_class_diagram() {
        let result = parse("classDiagram\n    ClassA <|-- ClassB\n");
        assert!(result.is_ok());
        let graph = result.unwrap();
        assert_eq!(graph.diagram_type, DiagramType::ClassDiagram);
        assert_eq!(graph.nodes.len(), 2);
    }

    #[test]
    fn test_parse_er_diagram() {
        let result = parse("erDiagram\n    CUSTOMER ||--o{ ORDER : places\n");
        assert!(result.is_ok());
        let graph = result.unwrap();
        assert_eq!(graph.diagram_type, DiagramType::ErDiagram);
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn test_graph_creation() {
        let graph = Graph::new();
        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(graph.edges.len(), 0);
    }

    // ── Frontmatter / title tests ─────────────────────────────────────────────

    #[test]
    fn test_title_parsed_from_frontmatter() {
        let input = "---\ntitle: E-Commerce Checkout Flow\n---\nflowchart TD\n    A --> B\n";
        let graph = parse(input).unwrap();
        assert_eq!(graph.title, Some("E-Commerce Checkout Flow".to_string()));
    }

    #[test]
    fn test_no_frontmatter_title_is_none() {
        let graph = parse("graph TB\n    A --> B\n").unwrap();
        assert_eq!(graph.title, None);
    }

    #[test]
    fn test_frontmatter_no_title_key() {
        let input = "---\nfoo: bar\n---\nflowchart TD\n    A --> B\n";
        let graph = parse(input).unwrap();
        assert_eq!(graph.title, None);
    }

    #[test]
    fn test_empty_title_value_is_none() {
        let input = "---\ntitle:\n---\nflowchart TD\n    A --> B\n";
        let graph = parse(input).unwrap();
        assert_eq!(graph.title, None);
    }

    #[test]
    fn test_frontmatter_with_extra_keys() {
        let input = "---\ntitle: Foo\nconfig: {}\n---\ngraph LR\n    A --> B\n";
        let graph = parse(input).unwrap();
        assert_eq!(graph.title, Some("Foo".to_string()));
    }

    #[test]
    fn test_diagram_body_parsed_correctly_with_frontmatter() {
        let input = "---\ntitle: Simple\n---\ngraph TB\n    A --> B\n    B --> C\n";
        let graph = parse(input).unwrap();
        assert_eq!(graph.title, Some("Simple".to_string()));
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);
    }
}
