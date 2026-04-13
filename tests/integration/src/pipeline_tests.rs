/// Full-pipeline integration tests that verify end-to-end rendering for each
/// diagram type and produce per-fixture metrics useful for documentation.
///
/// Run with:
///   cargo test -p trellis-integration -- --nocapture
/// to see the metrics table printed to stdout.
#[cfg(test)]
use trellis_core::{config::TrellisConfig, pipeline::render, types::OutputFormat};
#[cfg(test)]
use trellis_parser::parse;

// ──────────────────────────────────────────────────────────────────────────────
// Helper
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
fn render_svg(mermaid: &str) -> trellis_core::types::RenderMetrics {
    let graph = parse(mermaid).expect("parse failed");
    let config = TrellisConfig::default();
    let result = render(&graph, &config, OutputFormat::Svg).expect("render failed");
    result.metrics
}

#[cfg(test)]
fn render_and_assert_svg_contains(
    mermaid: &str,
    needle: &str,
) -> trellis_core::types::RenderMetrics {
    let graph = parse(mermaid).expect("parse failed");
    let config = TrellisConfig::default();
    let result = render(&graph, &config, OutputFormat::Svg).expect("render failed");
    let svg = String::from_utf8(result.data.clone()).expect("SVG is not valid UTF-8");
    assert!(
        svg.contains(needle),
        "SVG output does not contain expected string {:?}",
        needle
    );
    result.metrics
}

// ──────────────────────────────────────────────────────────────────────────────
// Flowchart pipeline tests (B01–B09)
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod flowchart {
    use super::*;

    #[test]
    fn b01_linear_chain_renders() {
        let m = render_svg("graph TB\n    A --> B\n    B --> C\n    C --> D\n    D --> E");
        assert_eq!(m.nodes, 5);
        assert_eq!(m.edges, 4);
        assert!(m.render_ms < 5000, "render should finish under 5 s");
        eprintln!(
            "[B01] nodes={} edges={} crossings={} bends={} render_ms={}",
            m.nodes, m.edges, m.crossings, m.bends, m.render_ms
        );
    }

    #[test]
    fn b02_wide_branch_renders() {
        let m = render_svg(
            "graph TB\n    A --> B1\n    A --> B2\n    A --> B3\n    A --> B4\n    A --> B5\n    A --> B6",
        );
        assert_eq!(m.nodes, 7);
        assert_eq!(m.edges, 6);
        assert_eq!(m.crossings, 0, "wide branch should have no crossings");
        eprintln!(
            "[B02] nodes={} edges={} crossings={} bends={} render_ms={}",
            m.nodes, m.edges, m.crossings, m.bends, m.render_ms
        );
    }

    #[test]
    fn b03_k33_renders_without_panic() {
        // K3,3 is non-planar; crossings are expected but the pipeline must not panic
        let m = render_svg(
            "graph TB\n    A1 --> B1\n    A1 --> B2\n    A1 --> B3\n    A2 --> B1\n    A2 --> B2\n    A2 --> B3\n    A3 --> B1\n    A3 --> B2\n    A3 --> B3",
        );
        assert_eq!(m.nodes, 6);
        assert_eq!(m.edges, 9);
        eprintln!(
            "[B03] nodes={} edges={} crossings={} bends={} render_ms={}",
            m.nodes, m.edges, m.crossings, m.bends, m.render_ms
        );
    }

    #[test]
    fn b04_diamond_no_crossings() {
        let m = render_svg("graph TB\n    A --> B\n    A --> C\n    B --> D\n    C --> D");
        assert_eq!(m.nodes, 4);
        assert_eq!(m.edges, 4);
        assert_eq!(m.crossings, 0, "diamond should have no crossings");
        eprintln!(
            "[B04] nodes={} edges={} crossings={} bends={} render_ms={}",
            m.nodes, m.edges, m.crossings, m.bends, m.render_ms
        );
    }

    #[test]
    fn b05_star_renders() {
        let m = render_svg(
            "graph TB\n    A --> B\n    B --> C\n    C --> A\n    A --> D\n    D --> E\n    E --> F",
        );
        assert_eq!(m.nodes, 6);
        assert_eq!(m.edges, 6);
        eprintln!(
            "[B05] nodes={} edges={} crossings={} bends={} render_ms={}",
            m.nodes, m.edges, m.crossings, m.bends, m.render_ms
        );
    }

    #[test]
    fn b06_multi_edge_renders() {
        let m = render_svg("graph TB\n    A --> B\n    A --> B\n    B --> C");
        assert_eq!(m.nodes, 3);
        assert_eq!(m.edges, 3);
        eprintln!(
            "[B06] nodes={} edges={} crossings={} bends={} render_ms={}",
            m.nodes, m.edges, m.crossings, m.bends, m.render_ms
        );
    }

    #[test]
    fn b07_cycle_renders() {
        let m = render_svg("graph TB\n    A --> B\n    B --> C\n    C --> A");
        assert_eq!(m.nodes, 3);
        assert_eq!(m.edges, 3);
        eprintln!(
            "[B07] nodes={} edges={} crossings={} bends={} render_ms={}",
            m.nodes, m.edges, m.crossings, m.bends, m.render_ms
        );
    }

    #[test]
    fn b08_nested_subgraph_renders_frame() {
        let svg_fragment = render_and_assert_svg_contains(
            "graph TB\n    subgraph outer\n        subgraph inner\n            A --> B\n        end\n        C --> D\n    end",
            "subgraph",
        );
        eprintln!(
            "[B08] nodes={} edges={} crossings={} bends={} render_ms={}",
            svg_fragment.nodes,
            svg_fragment.edges,
            svg_fragment.crossings,
            svg_fragment.bends,
            svg_fragment.render_ms
        );
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Class diagram pipeline tests (B12)
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod class_diagram {
    use super::*;

    #[test]
    fn b12_class_hierarchy_renders() {
        let m = render_and_assert_svg_contains(
            "classDiagram\n    Animal <|-- Dog\n    Animal <|-- Cat\n    Animal : +String name\n    Animal : +int age\n    Animal : +makeSound()\n    Dog : +String breed\n    Dog : +bark()\n    Cat : +bool indoor\n    Cat : +meow()\n",
            "Animal",
        );
        assert_eq!(m.nodes, 3);
        eprintln!(
            "[B12] nodes={} edges={} crossings={} bends={} render_ms={}",
            m.nodes, m.edges, m.crossings, m.bends, m.render_ms
        );
    }

    #[test]
    fn class_diagram_inheritance_marker_in_svg() {
        // The SVG should contain an inheritance triangle marker
        render_and_assert_svg_contains(
            "classDiagram\n    Vehicle <|-- Car\n    Vehicle : +String make\n",
            "inherit",
        );
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// ER diagram pipeline tests (B11)
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod er_diagram {
    use super::*;

    #[test]
    fn b11_er_simple_renders() {
        let m = render_svg(
            "erDiagram\n    CUSTOMER ||--o{ ORDER : places\n    ORDER ||--|{ LINE-ITEM : contains\n",
        );
        assert!(m.nodes >= 2, "should have at least 2 entities");
        eprintln!(
            "[B11] nodes={} edges={} crossings={} bends={} render_ms={}",
            m.nodes, m.edges, m.crossings, m.bends, m.render_ms
        );
    }

    #[test]
    fn er_diagram_crow_foot_marker_in_svg() {
        render_and_assert_svg_contains("erDiagram\n    A ||--o{ B : rel\n", "er-");
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// C4 diagram pipeline tests
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod c4_diagram {
    use super::*;

    #[test]
    fn c4_context_renders() {
        let m = render_svg(
            "C4Context\n    Person(user, \"User\", \"A person\")\n    System(sys, \"System\", \"The system\")\n    Rel(user, sys, \"Uses\")\n",
        );
        assert!(m.nodes >= 2);
        eprintln!(
            "[C4] nodes={} edges={} crossings={} bends={} render_ms={}",
            m.nodes, m.edges, m.crossings, m.bends, m.render_ms
        );
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Metrics summary (printed only with --nocapture)
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod metrics_summary {
    use super::*;

    /// Runs all B01–B12 fixtures and prints a summary table of render metrics.
    /// Execute with: cargo test -p trellis-integration metrics_table -- --nocapture
    #[test]
    fn metrics_table() {
        let fixtures: &[(&str, &str)] = &[
            ("B01 linear chain",    "graph TB\n    A --> B\n    B --> C\n    C --> D\n    D --> E"),
            ("B02 wide branch",     "graph TB\n    A --> B1\n    A --> B2\n    A --> B3\n    A --> B4\n    A --> B5\n    A --> B6"),
            ("B03 K3,3 bipartite",  "graph TB\n    A1 --> B1\n    A1 --> B2\n    A1 --> B3\n    A2 --> B1\n    A2 --> B2\n    A2 --> B3\n    A3 --> B1\n    A3 --> B2\n    A3 --> B3"),
            ("B04 diamond",         "graph TB\n    A --> B\n    A --> C\n    B --> D\n    C --> D"),
            ("B05 star/cycle",      "graph TB\n    A --> B\n    B --> C\n    C --> A\n    A --> D\n    D --> E\n    E --> F"),
            ("B06 multi-edge",      "graph TB\n    A --> B\n    A --> B\n    B --> C"),
            ("B07 cycle",           "graph TB\n    A --> B\n    B --> C\n    C --> A"),
            ("B12 class hierarchy", "classDiagram\n    Animal <|-- Dog\n    Animal <|-- Cat\n    Animal : +String name\n    Animal : +makeSound()\n"),
            ("B11 ER simple",       "erDiagram\n    CUSTOMER ||--o{ ORDER : places\n    ORDER ||--|{ LINE-ITEM : contains\n"),
        ];

        eprintln!();
        eprintln!(
            "{:<25} {:>6} {:>6} {:>9} {:>6} {:>10}",
            "Fixture", "Nodes", "Edges", "Crossings", "Bends", "Render ms"
        );
        eprintln!("{}", "-".repeat(70));

        for (name, mermaid) in fixtures {
            let graph = parse(mermaid).expect("parse failed");
            let config = TrellisConfig::default();
            let result = render(&graph, &config, OutputFormat::Svg).expect("render failed");
            let m = &result.metrics;
            eprintln!(
                "{:<25} {:>6} {:>6} {:>9} {:>6} {:>10}",
                name, m.nodes, m.edges, m.crossings, m.bends, m.render_ms
            );
        }

        eprintln!("{}", "-".repeat(70));
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Theme smoke tests — each theme renders a simple diagram without panicking
// and embeds its background colour in the SVG output.
// ──────────────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod themes {
    use super::*;
    use trellis_core::ThemeName;

    const SIMPLE: &str = "graph TB\n    A --> B\n    B --> C";

    fn render_with_theme(theme: ThemeName) -> String {
        let graph = parse(SIMPLE).expect("parse failed");
        let mut config = TrellisConfig::default();
        config.theme = theme;
        let result = render(&graph, &config, OutputFormat::Svg).expect("render failed");
        String::from_utf8(result.data).expect("SVG is not valid UTF-8")
    }

    #[test]
    fn default_theme_renders() {
        let svg = render_with_theme(ThemeName::Default);
        assert!(svg.contains("<svg"), "missing SVG root");
        // Default background is white
        assert!(svg.contains("white"), "missing default background");
    }

    #[test]
    fn paper_theme_renders() {
        let svg = render_with_theme(ThemeName::Paper);
        assert!(svg.contains("<svg"), "missing SVG root");
        // Paper theme has a warm cream background
        assert!(svg.contains("#faf8f5"), "missing paper background colour");
    }

    #[test]
    fn blueprint_theme_renders() {
        let svg = render_with_theme(ThemeName::Blueprint);
        assert!(svg.contains("<svg"), "missing SVG root");
        // Blueprint theme has a light blue-white background
        assert!(svg.contains("#f8faff"), "missing blueprint background colour");
    }

    #[test]
    fn dark_theme_renders() {
        let svg = render_with_theme(ThemeName::Dark);
        assert!(svg.contains("<svg"), "missing SVG root");
        // Dark theme has a dark background
        assert!(svg.contains("#1e1e1e"), "missing dark background colour");
    }

    #[test]
    fn midnight_theme_renders() {
        let svg = render_with_theme(ThemeName::Midnight);
        assert!(svg.contains("<svg"), "missing SVG root");
        // Midnight theme has a deep navy background
        assert!(svg.contains("#0d1117"), "missing midnight background colour");
    }

    #[test]
    fn forest_theme_renders() {
        let svg = render_with_theme(ThemeName::Forest);
        assert!(svg.contains("<svg"), "missing SVG root");
        // Forest theme has a dark green background
        assert!(svg.contains("#0f1a0f"), "missing forest background colour");
    }
}
