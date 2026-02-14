pub mod ast;
pub mod flowchart;
pub mod text_metrics;
pub mod tokenizer;

pub use ast::*;

/// Parse a Mermaid diagram from a string into a Graph AST.
///
/// Detects the diagram type (flowchart, classDiagram, erDiagram) from
/// the first directive and dispatches to the appropriate parser.
pub fn parse(input: &str) -> Result<Graph, ParseError> {
    let tokens = tokenizer::tokenize(input);
    let diagram_type = tokenizer::detect_diagram_type(&tokens);

    match diagram_type {
        DiagramType::Flowchart => flowchart::parse_flowchart(&tokens),
        DiagramType::ClassDiagram => Err(ParseError {
            message: "Class diagram parsing not yet implemented".to_string(),
            line: 0,
            column: 0,
        }),
        DiagramType::ErDiagram => Err(ParseError {
            message: "ER diagram parsing not yet implemented".to_string(),
            line: 0,
            column: 0,
        }),
    }
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
    fn test_parse_class_diagram_not_implemented() {
        let result = parse("classDiagram\n    ClassA <|-- ClassB\n");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_er_diagram_not_implemented() {
        let result = parse("erDiagram\n    CUSTOMER ||--o{ ORDER : places\n");
        assert!(result.is_err());
    }

    #[test]
    fn test_graph_creation() {
        let graph = Graph::new();
        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(graph.edges.len(), 0);
    }
}
