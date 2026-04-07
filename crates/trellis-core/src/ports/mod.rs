pub mod assignment;
pub mod auto;
pub mod barycenter;
pub mod common;
pub mod crossing_greedy;
pub mod iterative;
pub mod median;
pub mod prepass;
pub mod stats;
pub mod two_phase;

pub use common::compute_topo_rank;

use std::collections::HashMap;

pub use assignment::{assign_ports, DefaultPortAssigner, EdgePorts, Port, Side};
pub use auto::AutoPortAssigner;
pub use barycenter::BarycenterPortAssigner;
pub use crossing_greedy::CrossingGreedyPortAssigner;
pub use iterative::IterativeSwapAssigner;
pub use median::MedianPortAssigner;
pub use prepass::{apply_pinned, straight_edge_prepass, PinnedPortMap, PinnedPorts};
pub use two_phase::TwoPhaseAssigner;

use crate::config::{FlowBias, PortAssignmentStrategy};
use crate::grid::Grid;

/// Context provided to port assignment strategies.
///
/// Bundled as a struct so future strategies can receive additional data
/// without breaking the trait signature.
pub struct PortAssignmentContext<'a> {
    pub graph: &'a trellis_parser::Graph,
    pub cell_size: i32,
    pub offset_x: i32,
    pub offset_y: i32,
    /// Grid with node footprints committed (no edges yet).
    /// Used by the straight-edge pre-pass and available to strategies.
    pub grid: &'a Grid,
    /// Edges whose ports were locked in by the straight-edge pre-pass.
    /// Each `PortAssigner` must apply these after its normal logic.
    pub pinned_ports: PinnedPortMap,
    /// Flow-direction bias setting from config.
    pub flow_bias: FlowBias,
    /// Topological rank per node (BFS from roots).
    /// Used for back-edge detection when `flow_bias` is active.
    pub topo_rank: std::collections::HashMap<String, usize>,
    /// When true, strategies may emit diagnostic info to stderr.
    pub print_metrics: bool,
}

/// Trait for port assignment strategies.
///
/// Each implementation assigns source and target connection points (ports)
/// to all edges in a graph, returning a map from edge index to port pair.
pub trait PortAssigner {
    fn assign_ports(&self, ctx: &PortAssignmentContext) -> HashMap<usize, EdgePorts>;
}

/// Resolve the effective layout direction for flow-aware port assignment.
///
/// Returns `Some(direction)` when the bias should be applied, `None` for uniform sectors.
pub fn effective_direction(ctx: &PortAssignmentContext) -> Option<trellis_parser::Direction> {
    match ctx.flow_bias {
        FlowBias::None => None,
        FlowBias::Strong => Some(ctx.graph.direction),
        FlowBias::Auto => {
            // Apply bias only for Flowchart (Sugiyama) diagrams; not ER/C4/Class
            if ctx.graph.diagram_type == trellis_parser::DiagramType::Flowchart {
                Some(ctx.graph.direction)
            } else {
                None
            }
        }
    }
}

/// Resolve a config enum value to a concrete port assigner.
pub fn create_port_assigner(strategy: PortAssignmentStrategy) -> Box<dyn PortAssigner> {
    match strategy {
        PortAssignmentStrategy::Default => Box::new(DefaultPortAssigner),
        PortAssignmentStrategy::Barycenter => Box::new(BarycenterPortAssigner),
        PortAssignmentStrategy::Median => Box::new(MedianPortAssigner),
        PortAssignmentStrategy::CrossingGreedy => Box::new(CrossingGreedyPortAssigner),
        PortAssignmentStrategy::IterativeSwap => Box::new(IterativeSwapAssigner {
            initial_strategy: PortAssignmentStrategy::CrossingGreedy,
        }),
        PortAssignmentStrategy::TwoPhase => Box::new(TwoPhaseAssigner),
        PortAssignmentStrategy::Auto => Box::new(AutoPortAssigner),
    }
}

/// Returns true if the given strategy benefits from the pipeline refinement loop.
///
/// For `Auto`, the answer depends on the graph — use `auto::auto_needs_refinement()`
/// instead. This function returns `false` for `Auto` to avoid unconditionally
/// triggering the refinement loop; the pipeline checks Auto separately.
pub fn needs_refinement(strategy: PortAssignmentStrategy) -> bool {
    matches!(
        strategy,
        PortAssignmentStrategy::IterativeSwap | PortAssignmentStrategy::TwoPhase
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;
    use trellis_parser::{ArrowHead, Edge, EdgeStyle, Graph, Node, NodeShape};

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
        let grid = Grid::new(50, 50, 10, 0, 0);
        let ctx = PortAssignmentContext {
            graph: &graph,
            cell_size: 10,
            offset_x: 0,
            offset_y: 0,
            grid: &grid,
            pinned_ports: HashMap::new(),
            flow_bias: FlowBias::None,
            topo_rank: HashMap::new(),
            print_metrics: false,
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

    #[test]
    fn all_strategies_produce_same_count() {
        let mut graph = Graph::new();
        graph.nodes = vec![
            Node {
                id: "A".into(),
                label: "A".into(),
                shape: NodeShape::Rectangle,
                width: 80.0,
                height: 20.0,
                x: 40.0,
                y: 30.0,
                ..Default::default()
            },
            Node {
                id: "B".into(),
                label: "B".into(),
                shape: NodeShape::Rectangle,
                width: 40.0,
                height: 20.0,
                x: 20.0,
                y: 180.0,
                ..Default::default()
            },
            Node {
                id: "C".into(),
                label: "C".into(),
                shape: NodeShape::Rectangle,
                width: 40.0,
                height: 20.0,
                x: 100.0,
                y: 180.0,
                ..Default::default()
            },
        ];
        graph.edges = vec![
            Edge {
                from: "A".into(),
                to: "B".into(),
                label: None,
                style: EdgeStyle::Solid,
                arrow_head: ArrowHead::Arrow,
                ..Default::default()
            },
            Edge {
                from: "A".into(),
                to: "C".into(),
                label: None,
                style: EdgeStyle::Solid,
                arrow_head: ArrowHead::Arrow,
                ..Default::default()
            },
        ];

        let strategies = [
            PortAssignmentStrategy::Default,
            PortAssignmentStrategy::Barycenter,
            PortAssignmentStrategy::Median,
            PortAssignmentStrategy::CrossingGreedy,
            PortAssignmentStrategy::IterativeSwap,
            PortAssignmentStrategy::TwoPhase,
            PortAssignmentStrategy::Auto,
        ];

        let grid = Grid::new(50, 50, 10, 0, 0);
        for strategy in &strategies {
            let assigner = create_port_assigner(*strategy);
            let ctx = PortAssignmentContext {
                graph: &graph,
                cell_size: 10,
                offset_x: 0,
                offset_y: 0,
                grid: &grid,
                pinned_ports: HashMap::new(),
                flow_bias: FlowBias::None,
                topo_rank: HashMap::new(),
                print_metrics: false,
            };
            let ports = assigner.assign_ports(&ctx);
            assert_eq!(
                ports.len(),
                2,
                "Strategy {:?} should produce 2 port assignments",
                strategy
            );
        }
    }

    #[test]
    fn needs_refinement_true_for_multi_round() {
        assert!(needs_refinement(PortAssignmentStrategy::IterativeSwap));
        assert!(needs_refinement(PortAssignmentStrategy::TwoPhase));
    }

    #[test]
    fn needs_refinement_false_for_single_round() {
        assert!(!needs_refinement(PortAssignmentStrategy::Default));
        assert!(!needs_refinement(PortAssignmentStrategy::Barycenter));
        assert!(!needs_refinement(PortAssignmentStrategy::Median));
        assert!(!needs_refinement(PortAssignmentStrategy::CrossingGreedy));
        // Auto handles refinement internally via auto_needs_refinement()
        assert!(!needs_refinement(PortAssignmentStrategy::Auto));
    }
}
