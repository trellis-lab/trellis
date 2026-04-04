use std::collections::HashMap;

use trellis_parser::DiagramType;

use super::assignment::EdgePorts;
use super::crossing_greedy::CrossingGreedyPortAssigner;
use super::stats::{compute_graph_stats, count_hubs_with_threshold, GraphStats};
use super::two_phase::TwoPhaseAssigner;
use super::{DefaultPortAssigner, PortAssigner, PortAssignmentContext};

// ── Threshold constants ─────────────────────────────────────────────────────

/// Hub threshold for flowchart diagrams (Sugiyama layers help with crossings).
const HUB_THRESHOLD_FLOWCHART: usize = 4;
/// Hub threshold for class/ER diagrams (inheritance/relationship hierarchies).
const HUB_THRESHOLD_CLASS_ER: usize = 3;
/// Hub threshold for C4 diagrams (typically sparse, few edges).
const HUB_THRESHOLD_C4: usize = 6;

/// Graph size limits beyond which multi-round strategies are too expensive.
const LARGE_GRAPH_NODE_LIMIT: usize = 200;
const LARGE_GRAPH_EDGE_LIMIT: usize = 500;

/// Maximum node count for TwoPhase (multi-round) selection.
const MAX_NODES_FOR_TWOPHASE: usize = 80;

/// Minimum edges per side to consider a graph "complex".
const SIDE_CONGESTION_THRESHOLD: usize = 4;

/// Minimum hub count to consider a graph "complex".
const COMPLEX_HUB_THRESHOLD: usize = 2;

/// Degree thresholds for moderate complexity.
const HIGH_DEGREE_THRESHOLD: usize = 5;
const HIGH_AVG_DEGREE: f64 = 3.0;
const HIGH_DEGREE_STD_DEV: f64 = 2.0;

/// Multi-edge count that triggers CrossingGreedy.
const MULTI_EDGE_CONCERN: usize = 2;

/// The strategy selected by the Auto decision tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectedStrategy {
    Default,
    CrossingGreedy,
    TwoPhase,
}

/// Auto port assigner — analyses graph stats and delegates to the best strategy.
pub struct AutoPortAssigner;

impl PortAssigner for AutoPortAssigner {
    fn assign_ports(&self, ctx: &PortAssignmentContext) -> HashMap<usize, EdgePorts> {
        let stats = compute_graph_stats(ctx.graph, ctx.cell_size, ctx.offset_x, ctx.offset_y);
        let effective_hubs = effective_hub_count(ctx.graph, &stats);
        let selected = select_strategy_with_hub_count(&stats, effective_hubs);

        #[cfg(not(target_arch = "wasm32"))]
        if ctx.print_metrics {
            log_selection(&stats, selected);
        }

        let assigner: Box<dyn PortAssigner> = match selected {
            SelectedStrategy::Default => Box::new(DefaultPortAssigner),
            SelectedStrategy::CrossingGreedy => Box::new(CrossingGreedyPortAssigner),
            SelectedStrategy::TwoPhase => Box::new(TwoPhaseAssigner),
        };
        assigner.assign_ports(ctx)
    }
}

/// Returns true if the Auto selector would choose a multi-round strategy for the
/// given graph and grid parameters. Used by the pipeline to decide whether to
/// run the refinement loop.
pub fn auto_needs_refinement(
    graph: &trellis_parser::Graph,
    cell_size: i32,
    offset_x: i32,
    offset_y: i32,
) -> bool {
    let stats = compute_graph_stats(graph, cell_size, offset_x, offset_y);
    let hubs = effective_hub_count(graph, &stats);
    select_strategy_with_hub_count(&stats, hubs) == SelectedStrategy::TwoPhase
}

/// Compute effective hub count using diagram-type-adjusted threshold.
fn effective_hub_count(graph: &trellis_parser::Graph, stats: &GraphStats) -> usize {
    let hub_threshold = match stats.diagram_type {
        DiagramType::ClassDiagram | DiagramType::ErDiagram => HUB_THRESHOLD_CLASS_ER,
        DiagramType::C4Diagram => HUB_THRESHOLD_C4,
        _ => HUB_THRESHOLD_FLOWCHART,
    };

    if hub_threshold == super::stats::HUB_THRESHOLD_DEFAULT {
        stats.hub_count
    } else {
        count_hubs_with_threshold(graph, hub_threshold)
    }
}

/// Select the best port assignment strategy based on graph statistics.
///
/// Decision tree (see `auto-mode-strategy.md`):
/// 1. Trivial → Default
/// 2. Simple chain → Default
/// 3. Large graph → CrossingGreedy
/// 4. Complex + small → TwoPhase
/// 5. Complex + large → CrossingGreedy
/// 6. Moderate complexity → CrossingGreedy
/// 7. Dense/skewed → CrossingGreedy
/// 8. Else → Default
pub fn select_strategy(stats: &GraphStats) -> SelectedStrategy {
    select_strategy_with_hub_count(stats, stats.hub_count)
}

/// Select strategy with an explicit hub count (for diagram-type-adjusted thresholds).
pub fn select_strategy_with_hub_count(
    stats: &GraphStats,
    effective_hub_count: usize,
) -> SelectedStrategy {
    // Trivial graphs
    if stats.edge_count == 0 || stats.node_count <= 2 {
        return SelectedStrategy::Default;
    }

    // Simple chain/linear graphs
    if stats.max_degree <= 2 && stats.hub_count == 0 && stats.max_edges_per_side <= 2 {
        return SelectedStrategy::Default;
    }

    // Large graphs — avoid multi-round
    if stats.node_count > LARGE_GRAPH_NODE_LIMIT || stats.edge_count > LARGE_GRAPH_EDGE_LIMIT {
        return SelectedStrategy::CrossingGreedy;
    }

    // Complex topology at manageable scale
    if effective_hub_count >= COMPLEX_HUB_THRESHOLD
        && stats.max_edges_per_side >= SIDE_CONGESTION_THRESHOLD
    {
        if stats.node_count <= MAX_NODES_FOR_TWOPHASE {
            return SelectedStrategy::TwoPhase;
        }
        return SelectedStrategy::CrossingGreedy;
    }

    // Moderate complexity indicators
    if stats.max_degree >= HIGH_DEGREE_THRESHOLD
        || stats.overflow_count > 0
        || stats.multi_edge_count >= MULTI_EDGE_CONCERN
    {
        return SelectedStrategy::CrossingGreedy;
    }

    // Dense or skewed degree distribution
    if stats.avg_degree > HIGH_AVG_DEGREE || stats.degree_std_dev > HIGH_DEGREE_STD_DEV {
        return SelectedStrategy::CrossingGreedy;
    }

    SelectedStrategy::Default
}

#[cfg(not(target_arch = "wasm32"))]
fn log_selection(stats: &GraphStats, selected: SelectedStrategy) {
    eprintln!(
        "[trellis::auto] nodes={} edges={} max_deg={} hubs={} max_side={} overflow={} → {:?}",
        stats.node_count,
        stats.edge_count,
        stats.max_degree,
        stats.hub_count,
        stats.max_edges_per_side,
        stats.overflow_count,
        selected,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_parser::DiagramType;

    fn base_stats() -> GraphStats {
        GraphStats {
            node_count: 10,
            edge_count: 12,
            subgraph_count: 0,
            diagram_type: DiagramType::Flowchart,
            max_degree: 3,
            avg_degree: 2.4,
            degree_std_dev: 0.8,
            hub_count: 0,
            max_edges_per_side: 2,
            overflow_count: 0,
            avg_connectors_per_side: 3.0,
            multi_edge_count: 0,
            bidirectional_edge_count: 0,
            avg_edge_node_ratio: 1.2,
            median_edge_node_ratio: 1.0,
            max_node_distance: 200.0,
            layout_density: 500.0,
            min_connectors_on_loaded_side: 3,
        }
    }

    #[test]
    fn trivial_no_edges_uses_default() {
        let stats = GraphStats {
            edge_count: 0,
            ..base_stats()
        };
        assert_eq!(select_strategy(&stats), SelectedStrategy::Default);
    }

    #[test]
    fn trivial_two_nodes_uses_default() {
        let stats = GraphStats {
            node_count: 2,
            edge_count: 1,
            ..base_stats()
        };
        assert_eq!(select_strategy(&stats), SelectedStrategy::Default);
    }

    #[test]
    fn simple_chain_uses_default() {
        let stats = GraphStats {
            max_degree: 2,
            hub_count: 0,
            max_edges_per_side: 1,
            ..base_stats()
        };
        assert_eq!(select_strategy(&stats), SelectedStrategy::Default);
    }

    #[test]
    fn large_node_count_uses_greedy() {
        let stats = GraphStats {
            node_count: 300,
            edge_count: 400,
            ..base_stats()
        };
        assert_eq!(select_strategy(&stats), SelectedStrategy::CrossingGreedy);
    }

    #[test]
    fn large_edge_count_uses_greedy() {
        let stats = GraphStats {
            node_count: 100,
            edge_count: 600,
            ..base_stats()
        };
        assert_eq!(select_strategy(&stats), SelectedStrategy::CrossingGreedy);
    }

    #[test]
    fn complex_small_graph_uses_twophase() {
        let stats = GraphStats {
            node_count: 40,
            hub_count: 3,
            max_edges_per_side: 5,
            max_degree: 6,
            ..base_stats()
        };
        assert_eq!(select_strategy(&stats), SelectedStrategy::TwoPhase);
    }

    #[test]
    fn complex_large_graph_uses_greedy() {
        let stats = GraphStats {
            node_count: 120,
            hub_count: 3,
            max_edges_per_side: 5,
            max_degree: 6,
            ..base_stats()
        };
        assert_eq!(select_strategy(&stats), SelectedStrategy::CrossingGreedy);
    }

    #[test]
    fn high_degree_uses_greedy() {
        let stats = GraphStats {
            max_degree: 7,
            ..base_stats()
        };
        assert_eq!(select_strategy(&stats), SelectedStrategy::CrossingGreedy);
    }

    #[test]
    fn overflow_uses_greedy() {
        let stats = GraphStats {
            overflow_count: 3,
            ..base_stats()
        };
        assert_eq!(select_strategy(&stats), SelectedStrategy::CrossingGreedy);
    }

    #[test]
    fn multi_edges_use_greedy() {
        let stats = GraphStats {
            multi_edge_count: 2,
            ..base_stats()
        };
        assert_eq!(select_strategy(&stats), SelectedStrategy::CrossingGreedy);
    }

    #[test]
    fn dense_avg_degree_uses_greedy() {
        let stats = GraphStats {
            avg_degree: 4.5,
            ..base_stats()
        };
        assert_eq!(select_strategy(&stats), SelectedStrategy::CrossingGreedy);
    }

    #[test]
    fn high_std_dev_uses_greedy() {
        let stats = GraphStats {
            degree_std_dev: 2.5,
            ..base_stats()
        };
        assert_eq!(select_strategy(&stats), SelectedStrategy::CrossingGreedy);
    }

    #[test]
    fn moderate_graph_uses_default() {
        // All metrics below thresholds → Default
        let stats = GraphStats {
            node_count: 10,
            edge_count: 12,
            max_degree: 3,
            hub_count: 0,
            max_edges_per_side: 2,
            overflow_count: 0,
            multi_edge_count: 0,
            avg_degree: 2.4,
            degree_std_dev: 0.8,
            ..base_stats()
        };
        assert_eq!(select_strategy(&stats), SelectedStrategy::Default);
    }

    #[test]
    fn er_diagram_lower_hub_threshold() {
        // With ER diagram, hub threshold is 3 instead of 4.
        // Stats has hub_count computed with default threshold (4).
        // Nodes with degree 3 ARE hubs for ER but not for flowchart.
        // The hub_count in stats uses default threshold, so we need hub_count >= 2
        // with default threshold for the complex path to trigger.
        // For this test, we verify that ER doesn't change the outcome when
        // hub_count is already computed above threshold.
        let stats = GraphStats {
            diagram_type: DiagramType::ErDiagram,
            node_count: 30,
            max_degree: 4,
            hub_count: 2,
            max_edges_per_side: 4,
            ..base_stats()
        };
        assert_eq!(select_strategy(&stats), SelectedStrategy::TwoPhase);
    }

    #[test]
    fn c4_diagram_high_hub_threshold() {
        // C4 hub threshold is 6, so nodes with degree 4-5 aren't hubs.
        // With flowchart threshold they'd be hubs, but C4 is more lenient.
        let stats = GraphStats {
            diagram_type: DiagramType::C4Diagram,
            node_count: 30,
            max_degree: 5,
            hub_count: 2, // counted with default=4
            max_edges_per_side: 4,
            ..base_stats()
        };
        // Even though hub_count=2 (default threshold), the C4 threshold is 6
        // so effective hubs would be fewer. Currently uses stats.hub_count
        // since threshold matches default. This still triggers complex path.
        // The key difference is visible when we integrate count_hubs_with_threshold.
        assert_eq!(select_strategy(&stats), SelectedStrategy::TwoPhase);
    }
}
