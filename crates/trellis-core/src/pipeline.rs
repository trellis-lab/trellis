use crate::{
    config::TrellisConfig,
    grid::{build_grid, calculate_grid_extent},
    labels, placement,
    ports::{create_port_assigner, PortAssignmentContext},
    routing,
    types::*,
};
use trellis_parser::{DiagramType, Graph};

/// Returns elapsed milliseconds since `start`. In WASM builds, always returns 0
/// because `std::time::Instant` is unavailable on `wasm32-unknown-unknown`.
#[cfg(not(target_arch = "wasm32"))]
fn elapsed_ms(start: std::time::Instant) -> u64 {
    start.elapsed().as_millis() as u64
}

#[cfg(target_arch = "wasm32")]
fn elapsed_ms(_start: ()) -> u64 {
    0
}

/// Main rendering pipeline
///
/// Runs the full pipeline: placement → grid → ports → routing → SVG/PNG.
pub fn render(
    graph: &Graph,
    config: &TrellisConfig,
    format: OutputFormat,
) -> Result<RenderResult, RenderError> {
    #[cfg(not(target_arch = "wasm32"))]
    let start = std::time::Instant::now();
    #[cfg(target_arch = "wasm32")]
    let start = ();

    // Clone the graph so we can mutate it during placement
    let mut graph = graph.clone();

    let cell_size = config.cell_size;

    // Phase 2: Node placement (Sugiyama for flowcharts, subgraph-aware if needed)
    let subgraph_data = placement::place_nodes(&mut graph, cell_size);

    // Phase 2.5: Resolve subgraph edges (flowchart only — creates virtual nodes for
    // edges that target a subgraph directly rather than an individual node)
    if graph.diagram_type == DiagramType::Flowchart {
        if let Some((ref _tree, ref boxes)) = subgraph_data {
            placement::subgraph::resolve_subgraph_edges(&mut graph, boxes);
        }
    }

    // Phase 3: Grid construction
    let extent = calculate_grid_extent(&graph, cell_size);
    let mut grid = build_grid(&graph, cell_size, &extent);

    // Phase 4: Port assignment
    let assigner = create_port_assigner(config.port_assignment);
    let port_ctx = PortAssignmentContext {
        graph: &graph,
        cell_size,
        offset_x: extent.offset_x,
        offset_y: extent.offset_y,
    };
    let port_assignments = assigner.assign_ports(&port_ctx);

    // Phase 5-6: Edge routing (A* pathfinding)
    let routing_result = routing::route_all_edges(&graph, &mut grid, &port_assignments, config);

    let render_ms = elapsed_ms(start);

    // Collect metrics
    let routed_count = routing_result.paths.len();
    let avg_edge_length = if routed_count > 0 {
        routing_result.total_path_length as f64 / routed_count as f64
    } else {
        0.0
    };
    let avg_routing_cost = if routed_count > 0 {
        routing_result.total_routing_cost / routed_count as f64
    } else {
        0.0
    };
    // avg_detour_factor = total actual steps / total manhattan steps.
    // Using totals (not per-edge average) avoids division-by-zero on zero-length edges.
    let avg_detour_factor = if routing_result.sum_manhattan_distance > 0 {
        routing_result.total_path_length as f64 / routing_result.sum_manhattan_distance as f64
    } else {
        0.0
    };
    let metrics = RenderMetrics {
        nodes: graph.nodes.len(),
        edges: graph.edges.len(),
        layers: count_layers(&graph),
        crossings: routing_result.crossings,
        bends: routing_result.total_bends,
        render_ms,
        grid_utilization: grid.utilization(),
        grid_rows: grid.rows,
        grid_cols: grid.cols,
        cell_size,
        port_count: port_assignments.len() * 2, // source + target for each edge
        total_edge_length: routing_result.total_path_length,
        total_routing_cost: routing_result.total_routing_cost,
        avg_edge_length,
        avg_routing_cost,
        max_edge_length: routing_result.max_path_length,
        max_bends_per_edge: routing_result.max_bends_per_edge,
        avg_detour_factor,
        deadlock_recoveries: routing_result.deadlock_recoveries,
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
        #[cfg(feature = "png")]
        OutputFormat::Png => {
            crate::render::png::svg_to_png(&svg_data).map_err(|e| RenderError {
                message: format!("PNG conversion failed: {}", e),
            })?
        }
        #[cfg(not(feature = "png"))]
        OutputFormat::Png => {
            return Err(RenderError {
                message: "PNG output is not supported in this build (compile with feature 'png')"
                    .to_string(),
            });
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
    let is_horizontal = matches!(
        graph.direction,
        trellis_parser::Direction::LR | trellis_parser::Direction::RL
    );

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
