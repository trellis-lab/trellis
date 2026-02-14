use crate::{
    config::TrellisConfig,
    grid::{build_grid, calculate_cell_size, calculate_grid_extent},
    placement,
    ports::assign_ports,
    types::*,
};
use trellis_parser::Graph;
use std::time::Instant;

/// Main rendering pipeline
///
/// Runs the full pipeline: placement → grid → ports → (routing → SVG in future milestones).
/// For M4, placement, grid construction, and port assignment are performed.
pub fn render(graph: &Graph, config: &TrellisConfig, format: OutputFormat) -> Result<RenderResult, RenderError> {
    let start = Instant::now();

    // Clone the graph so we can mutate it during placement
    let mut graph = graph.clone();

    // Phase 2: Node placement (Sugiyama for flowcharts)
    placement::place_nodes(&mut graph);

    // Phase 3: Grid construction
    let cell_size = calculate_cell_size(&graph);
    let extent = calculate_grid_extent(&graph);
    let grid = build_grid(&graph, cell_size, &extent);

    // Phase 4: Port assignment
    let port_assignments = assign_ports(&graph, cell_size);

    let elapsed = start.elapsed();

    // Collect metrics
    let metrics = RenderMetrics {
        nodes: graph.nodes.len(),
        edges: graph.edges.len(),
        layers: count_layers(&graph),
        crossings: 0, // Will be computed in M5
        bends: 0,     // Will be computed in M5
        render_ms: elapsed.as_millis() as u64,
        grid_utilization: grid.utilization(),
        grid_rows: grid.rows,
        grid_cols: grid.cols,
        cell_size,
        port_count: port_assignments.len() * 2, // source + target for each edge
    };

    let data = match format {
        OutputFormat::Svg => {
            // Placeholder SVG showing node positions
            generate_debug_svg(&graph, &config)
        }
        OutputFormat::Png => {
            // Placeholder: will be implemented with resvg in M6
            vec![]
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

/// Generate a debug SVG that shows node positions (placeholder until M6 proper rendering)
fn generate_debug_svg(graph: &Graph, _config: &TrellisConfig) -> Vec<u8> {
    if graph.nodes.is_empty() {
        return b"<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"400\" height=\"300\"><rect width=\"400\" height=\"300\" fill=\"white\"/></svg>".to_vec();
    }

    // Calculate viewBox from node positions
    let max_x = graph.nodes.iter().map(|n| n.x + n.width / 2.0).fold(f64::NEG_INFINITY, f64::max);
    let max_y = graph.nodes.iter().map(|n| n.y + n.height / 2.0).fold(f64::NEG_INFINITY, f64::max);
    let vw = (max_x + 50.0).ceil();
    let vh = (max_y + 50.0).ceil();

    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">\n",
        vw, vh, vw, vh,
    );
    svg.push_str("  <rect width=\"100%\" height=\"100%\" fill=\"white\"/>\n");

    // Draw edges as lines
    for edge in &graph.edges {
        let from_node = graph.nodes.iter().find(|n| n.id == edge.from);
        let to_node = graph.nodes.iter().find(|n| n.id == edge.to);
        if let (Some(f), Some(t)) = (from_node, to_node) {
            svg.push_str(&format!(
                "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#999\" stroke-width=\"1\" marker-end=\"url(#arrow)\"/>\n",
                f.x, f.y, t.x, t.y,
            ));
        }
    }

    // Arrow marker
    svg.push_str("  <defs><marker id=\"arrow\" viewBox=\"0 0 10 10\" refX=\"10\" refY=\"5\" markerWidth=\"6\" markerHeight=\"6\" orient=\"auto-start-reverse\"><path d=\"M 0 0 L 10 5 L 0 10 z\" fill=\"#999\"/></marker></defs>\n");

    // Draw nodes
    for node in &graph.nodes {
        let rx = node.x - node.width / 2.0;
        let ry = node.y - node.height / 2.0;
        svg.push_str(&format!(
            "  <rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"4\" fill=\"#e8f4fd\" stroke=\"#4a90d9\" stroke-width=\"1.5\"/>\n",
            rx, ry, node.width, node.height,
        ));
        svg.push_str(&format!(
            "  <text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"central\" font-family=\"Arial\" font-size=\"12\">{}</text>\n",
            node.x, node.y, node.label,
        ));
    }

    svg.push_str("</svg>");
    svg.into_bytes()
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
