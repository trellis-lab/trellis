use crate::{
    config::TrellisConfig,
    grid::{build_grid, calculate_grid_extent},
    labels,
    placement,
    ports::assign_ports,
    routing,
    types::*,
};
use trellis_parser::Graph;
use std::time::Instant;

/// Main rendering pipeline
///
/// Runs the full pipeline: placement → grid → ports → routing → SVG/PNG.
pub fn render(graph: &Graph, config: &TrellisConfig, format: OutputFormat) -> Result<RenderResult, RenderError> {
    let start = Instant::now();

    // Clone the graph so we can mutate it during placement
    let mut graph = graph.clone();

    let cell_size = config.cell_size;

    // Phase 2: Node placement (Sugiyama for flowcharts, subgraph-aware if needed)
    let subgraph_data = placement::place_nodes(&mut graph, cell_size);

    // Phase 2.5: Resolve subgraph edges (create virtual nodes for edges targeting subgraphs)
    if let Some((ref _tree, ref boxes)) = subgraph_data {
        placement::subgraph::resolve_subgraph_edges(&mut graph, boxes);
    }

    // Phase 3: Grid construction
    let extent = calculate_grid_extent(&graph, cell_size);
    let mut grid = build_grid(&graph, cell_size, &extent);

    // Phase 4: Port assignment
    let port_assignments = assign_ports(&graph, cell_size, extent.offset_x, extent.offset_y);

    // Phase 5-6: Edge routing (A* pathfinding)
    let routing_result = routing::route_all_edges(&graph, &mut grid, &port_assignments, config);

    let elapsed = start.elapsed();

    // Collect metrics
    let metrics = RenderMetrics {
        nodes: graph.nodes.len(),
        edges: graph.edges.len(),
        layers: count_layers(&graph),
        crossings: routing_result.crossings,
        bends: routing_result.total_bends,
        render_ms: elapsed.as_millis() as u64,
        grid_utilization: grid.utilization(),
        grid_rows: grid.rows,
        grid_cols: grid.cols,
        cell_size,
        port_count: port_assignments.len() * 2, // source + target for each edge
    };

    // Phase 8: Edge label placement
    let label_placements = labels::place_all_labels(&graph, &routing_result.paths, &grid);

    // Phase 9: SVG rendering
    let svg_data = crate::render::svg::build_svg(
        &graph,
        &grid,
        &routing_result,
        config,
        &label_placements,
        subgraph_data.as_ref(),
    );

    let data = match format {
        OutputFormat::Svg => svg_data,
        OutputFormat::Png => {
            crate::render::png::svg_to_png(&svg_data).map_err(|e| RenderError {
                message: format!("PNG conversion failed: {}", e),
            })?
        }
    };

    Ok(RenderResult {
        format,
        data,
        metrics,
    })
}

/// Count the number of distinct layers in the placed graph
fn count_layers(graph: &Graph) -> usize {
    if graph.nodes.is_empty() {
        return 0;
    }

    // For TB/BT layouts, layers are distinguished by y; for LR/RL by x
    let is_horizontal = matches!(graph.direction, trellis_parser::Direction::LR | trellis_parser::Direction::RL);

    let mut layer_values: Vec<i64> = graph
        .nodes
        .iter()
        .map(|n| {
            let val = if is_horizontal { n.x } else { n.y };
            (val * 10.0).round() as i64 // avoid float comparison issues
        })
        .collect();
    layer_values.sort();
    layer_values.dedup();
    layer_values.len()
}

/// Error type for rendering failures
#[derive(Debug, Clone)]
pub struct RenderError {
    pub message: String,
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Render error: {}", self.message)
    }
}

impl std::error::Error for RenderError {}
