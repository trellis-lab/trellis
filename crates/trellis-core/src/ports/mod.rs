pub mod assignment;

use std::collections::HashMap;
use trellis_parser::Graph;

pub use assignment::{assign_ports, DefaultPortAssigner, EdgePorts, Port, Side};

use crate::config::PortAssignmentStrategy;

/// Context provided to port assignment strategies.
///
/// Bundled as a struct so future strategies can receive additional data
/// (e.g. Grid, diagram direction) without breaking the trait signature.
pub struct PortAssignmentContext<'a> {
    pub graph: &'a Graph,
    pub cell_size: i32,
    pub offset_x: i32,
    pub offset_y: i32,
}

/// Trait for port assignment strategies.
///
/// Each implementation assigns source and target connection points (ports)
/// to all edges in a graph, returning a map from edge index to port pair.
pub trait PortAssigner {
    fn assign_ports(&self, ctx: &PortAssignmentContext) -> HashMap<usize, EdgePorts>;
}

/// Resolve a config enum value to a concrete port assigner.
pub fn create_port_assigner(strategy: PortAssignmentStrategy) -> Box<dyn PortAssigner> {
    match strategy {
        PortAssignmentStrategy::Default => Box::new(DefaultPortAssigner),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_parser::{ArrowHead, Edge, EdgeStyle, Node, NodeShape};

    #[test]
    fn default_assigner_matches_free_function() {
        let mut graph = Graph::new();
        graph.nodes = vec![
            Node {
                id: "A".into(),
                label: "A".into(),
                shape: NodeShape::Rectangle,
                width: 40.0,
                height: 20.0,
                x: 60.0,
                y: 30.0,
                ..Default::default()
            },
            Node {
                id: "B".into(),
                label: "B".into(),
                shape: NodeShape::Rectangle,
                width: 40.0,
                height: 20.0,
                x: 60.0,
                y: 130.0,
                ..Default::default()
            },
        ];
        graph.edges = vec![Edge {
            from: "A".into(),
            to: "B".into(),
            label: None,
            style: EdgeStyle::Solid,
            arrow_head: ArrowHead::Arrow,
            ..Default::default()
        }];

        let free_fn_result = assign_ports(&graph, 10, 0, 0);

        let assigner = DefaultPortAssigner;
        let ctx = PortAssignmentContext {
            graph: &graph,
            cell_size: 10,
            offset_x: 0,
            offset_y: 0,
        };
        let trait_result = assigner.assign_ports(&ctx);

        assert_eq!(free_fn_result.len(), trait_result.len());
        for (idx, expected) in &free_fn_result {
            let actual = &trait_result[idx];
            assert_eq!(expected.source_port.grid_row, actual.source_port.grid_row);
            assert_eq!(expected.source_port.grid_col, actual.source_port.grid_col);
            assert_eq!(expected.target_port.grid_row, actual.target_port.grid_row);
            assert_eq!(expected.target_port.grid_col, actual.target_port.grid_col);
        }
    }
}
