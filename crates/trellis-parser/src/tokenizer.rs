use crate::ast::DiagramType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    /// Diagram type directive: `graph TB`, `flowchart LR`
    Directive,
    /// Subgraph start: `subgraph id [label]`
    SubgraphStart,
    /// Subgraph end: `end`
    SubgraphEnd,
    /// Node/edge statement
    Statement,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub content: String,
    pub line_number: usize,
}

/// Tokenize Mermaid input into classified tokens.
///
/// Processes line by line: strips comments, splits on semicolons,
/// and classifies each token.
pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();

    for (line_idx, line) in input.lines().enumerate() {
        let line = strip_comment(line);

        // Split on semicolons for multiple statements per line
        for part in line.split(';') {
            let trimmed = part.trim();
            if trimmed.is_empty() {
                continue;
            }

            let token_type = classify_line(trimmed);
            tokens.push(Token {
                token_type,
                content: trimmed.to_string(),
                line_number: line_idx + 1,
            });
        }
    }

    tokens
}

/// Strip Mermaid comments (%% ...) from a line
fn strip_comment(line: &str) -> &str {
    match line.find("%%") {
        Some(pos) => &line[..pos],
        None => line,
    }
}

/// Classify a trimmed, non-empty line into a token type
fn classify_line(trimmed: &str) -> TokenType {
    let lower = trimmed.to_lowercase();

    // Flowchart directives
    if lower.starts_with("graph ") || lower.starts_with("flowchart ")
        || lower == "graph" || lower == "flowchart"
    {
        return TokenType::Directive;
    }

    // Other diagram type directives
    if lower.starts_with("classdiagram") || lower.starts_with("erdiagram") {
        return TokenType::Directive;
    }

    // C4 diagram directives (C4Context, C4Container, C4Component, C4Dynamic, C4Deployment)
    if lower.starts_with("c4context")
        || lower.starts_with("c4container")
        || lower.starts_with("c4component")
        || lower.starts_with("c4dynamic")
        || lower.starts_with("c4deployment")
    {
        return TokenType::Directive;
    }

    // Subgraph boundaries
    if lower.starts_with("subgraph ") || lower == "subgraph" {
        return TokenType::SubgraphStart;
    }

    if lower == "end" {
        return TokenType::SubgraphEnd;
    }

    TokenType::Statement
}

/// Detect diagram type from the first directive token
pub fn detect_diagram_type(tokens: &[Token]) -> DiagramType {
    for token in tokens {
        if token.token_type == TokenType::Directive {
            let lower = token.content.to_lowercase();
            if lower.starts_with("graph") || lower.starts_with("flowchart") {
                return DiagramType::Flowchart;
            }
            if lower.starts_with("classdiagram") {
                return DiagramType::ClassDiagram;
            }
            if lower.starts_with("erdiagram") {
                return DiagramType::ErDiagram;
            }
            if lower.starts_with("c4context")
                || lower.starts_with("c4container")
                || lower.starts_with("c4component")
                || lower.starts_with("c4dynamic")
                || lower.starts_with("c4deployment")
            {
                return DiagramType::C4Diagram;
            }
        }
    }
    DiagramType::Flowchart
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_comment() {
        assert_eq!(strip_comment("A --> B %% this is a comment"), "A --> B ");
        assert_eq!(strip_comment("A --> B"), "A --> B");
        assert_eq!(strip_comment("%% full comment"), "");
    }

    #[test]
    fn test_classify_line() {
        assert_eq!(classify_line("graph TB"), TokenType::Directive);
        assert_eq!(classify_line("flowchart LR"), TokenType::Directive);
        assert_eq!(classify_line("subgraph S1"), TokenType::SubgraphStart);
        assert_eq!(classify_line("end"), TokenType::SubgraphEnd);
        assert_eq!(classify_line("A --> B"), TokenType::Statement);
    }

    #[test]
    fn test_tokenize_simple() {
        let input = "graph TB\n    A --> B\n    B --> C\n";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].token_type, TokenType::Directive);
        assert_eq!(tokens[1].token_type, TokenType::Statement);
        assert_eq!(tokens[2].token_type, TokenType::Statement);
    }

    #[test]
    fn test_tokenize_with_comments() {
        let input = "graph TB\n    %% comment line\n    A --> B\n";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenType::Directive);
        assert_eq!(tokens[1].token_type, TokenType::Statement);
    }

    #[test]
    fn test_tokenize_semicolons() {
        let input = "graph TB\n    A --> B; B --> C\n";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[1].content, "A --> B");
        assert_eq!(tokens[2].content, "B --> C");
    }

    #[test]
    fn test_tokenize_subgraphs() {
        let input = "graph TB\n    subgraph S1\n        A --> B\n    end\n";
        let tokens = tokenize(input);
        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[1].token_type, TokenType::SubgraphStart);
        assert_eq!(tokens[2].token_type, TokenType::Statement);
        assert_eq!(tokens[3].token_type, TokenType::SubgraphEnd);
    }

    #[test]
    fn test_detect_diagram_type() {
        let flowchart_tokens = tokenize("graph TB\n    A --> B\n");
        assert_eq!(detect_diagram_type(&flowchart_tokens), DiagramType::Flowchart);

        let flowchart_tokens2 = tokenize("flowchart LR\n    A --> B\n");
        assert_eq!(detect_diagram_type(&flowchart_tokens2), DiagramType::Flowchart);
    }
}
