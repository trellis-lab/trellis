use serde::{Deserialize, Serialize};

/// Output format for rendered diagrams
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum OutputFormat {
    #[default]
    Svg,
    Png,
}

/// Rendering result containing the output data
#[derive(Debug, Clone)]
pub struct RenderResult {
    pub format: OutputFormat,
    pub data: Vec<u8>,
    pub metrics: RenderMetrics,
}

/// Metrics collected during rendering
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RenderMetrics {
    pub nodes: usize,
    pub edges: usize,
    pub layers: usize,
    pub crossings: usize,
    pub bends: usize,
    pub render_ms: u64,
    pub grid_utilization: f64,
    pub grid_rows: usize,
    pub grid_cols: usize,
    pub cell_size: i32,
    pub port_count: usize,
}
