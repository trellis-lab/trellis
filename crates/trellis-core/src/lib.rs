pub mod config;
pub mod grid;
pub mod pipeline;
pub mod placement;
pub mod ports;
pub mod types;

pub use config::*;
pub use pipeline::*;
pub use placement::place_nodes;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_parser::Graph;

    #[test]
    fn test_render_placeholder() {
        let graph = Graph::new();
        let config = TrellisConfig::default();
        let result = render(&graph, &config, OutputFormat::Svg);
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.data.len() > 0);
    }

    #[test]
    fn test_config_default() {
        let config = TrellisConfig::default();
        assert_eq!(config.cell_size, 20.0);
        assert_eq!(config.decomposition_threshold, 50);
    }

    /// Helper: parse a fixture and run the full pipeline, returning metrics
    fn run_fixture(content: &str) -> RenderMetrics {
        let graph = trellis_parser::parse(content).expect("parse failed");
        let config = TrellisConfig::default();
        let result = render(&graph, &config, OutputFormat::Svg).expect("render failed");
        result.metrics
    }

    // --- Grid size tests for B01-B05 fixtures ---

    #[test]
    fn test_b01_grid_and_ports() {
        // B01: linear chain A→B→C→D→E (5 nodes, 4 edges)
        let m = run_fixture("graph TB\n    A --> B\n    B --> C\n    C --> D\n    D --> E");
        assert_eq!(m.nodes, 5);
        assert_eq!(m.edges, 4);
        assert!(m.grid_rows > 0, "grid should have rows");
        assert!(m.grid_cols > 0, "grid should have cols");
        assert!(m.cell_size >= 5.0, "cell size should be at least 5");
        assert_eq!(m.port_count, 8); // 4 edges * 2 ports each
        assert!(m.grid_utilization > 0.0, "grid should have blocked cells from nodes");
    }

    #[test]
    fn test_b02_grid_and_ports() {
        // B02: wide branch A→B1..B6 (7 nodes, 6 edges)
        let m = run_fixture("graph TB\n    A --> B1\n    A --> B2\n    A --> B3\n    A --> B4\n    A --> B5\n    A --> B6");
        assert_eq!(m.nodes, 7);
        assert_eq!(m.edges, 6);
        assert!(m.grid_rows > 0);
        assert!(m.grid_cols > 0);
        assert_eq!(m.port_count, 12); // 6 edges * 2
    }

    #[test]
    fn test_b03_grid_and_ports() {
        // B03: K3,3 bipartite (6 nodes, 9 edges)
        let m = run_fixture("graph TB\n    A1 --> B1\n    A1 --> B2\n    A1 --> B3\n    A2 --> B1\n    A2 --> B2\n    A2 --> B3\n    A3 --> B1\n    A3 --> B2\n    A3 --> B3");
        assert_eq!(m.nodes, 6);
        assert_eq!(m.edges, 9);
        assert!(m.grid_rows > 0);
        assert!(m.grid_cols > 0);
        assert_eq!(m.port_count, 18); // 9 edges * 2
        // Dense graph should have larger grid
        assert!(m.grid_rows * m.grid_cols > 20, "K3,3 should have a reasonably sized grid");
    }

    #[test]
    fn test_b04_grid_and_ports() {
        // B04: diamond A→B, A→C, B→D, C→D (4 nodes, 4 edges)
        let m = run_fixture("graph TB\n    A --> B\n    A --> C\n    B --> D\n    C --> D");
        assert_eq!(m.nodes, 4);
        assert_eq!(m.edges, 4);
        assert!(m.grid_rows > 0);
        assert!(m.grid_cols > 0);
        assert_eq!(m.port_count, 8);
    }

    #[test]
    fn test_b05_grid_and_ports() {
        // B05: graph with cycle A→B→C→A, A→D→E→F (6 nodes, 6 edges)
        let m = run_fixture("graph TB\n    A --> B\n    B --> C\n    C --> A\n    A --> D\n    D --> E\n    E --> F");
        assert_eq!(m.nodes, 6);
        assert_eq!(m.edges, 6);
        assert!(m.grid_rows > 0);
        assert!(m.grid_cols > 0);
        assert_eq!(m.port_count, 12);
    }

    // --- Port symmetry tests ---

    #[test]
    fn test_port_positions_within_node_bounds() {
        let graph = trellis_parser::parse("graph TB\n    A --> B\n    A --> C\n    B --> D\n    C --> D").unwrap();
        let mut graph = graph.clone();
        placement::place_nodes(&mut graph);

        let cell_size = grid::calculate_cell_size(&graph);
        let port_assignments = ports::assign_ports(&graph, cell_size);

        // All ports should be on the boundary of their respective nodes
        for node in &graph.nodes {
            let left = node.x - node.width / 2.0;
            let right = node.x + node.width / 2.0;
            let top = node.y - node.height / 2.0;
            let bottom = node.y + node.height / 2.0;

            for (_, ep) in &port_assignments {
                // Check source ports
                for port in [&ep.source_port, &ep.target_port] {
                    let on_this_node = port.x >= left - 0.01 && port.x <= right + 0.01
                        && port.y >= top - 0.01 && port.y <= bottom + 0.01;
                    if on_this_node {
                        // Port should be on the edge of the node (not inside)
                        let on_edge = (port.x - left).abs() < 0.01
                            || (port.x - right).abs() < 0.01
                            || (port.y - top).abs() < 0.01
                            || (port.y - bottom).abs() < 0.01;
                        assert!(on_edge, "Port ({}, {}) should be on the edge of node {} bounds ({}, {}, {}, {})",
                            port.x, port.y, node.id, left, top, right, bottom);
                    }
                }
            }
        }
    }
}
