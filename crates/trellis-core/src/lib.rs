pub mod config;
pub mod pipeline;
pub mod placement;
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
}
