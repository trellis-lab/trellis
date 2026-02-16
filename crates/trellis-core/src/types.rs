use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Output format for rendered diagrams
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum OutputFormat {
    #[default]
    Svg,
    Png,
}

/// Axis-aligned bounding box for subgraph frames
#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// A node in the flattened subgraph tree
#[derive(Debug, Clone)]
pub struct SubgraphTreeNode {
    pub id: String,
    pub label: Option<String>,
    /// Direct child subgraph IDs
    pub children: Vec<String>,
    /// Node IDs that belong directly to this subgraph (not to a deeper child)
    pub direct_node_ids: Vec<String>,
}

/// The full subgraph tree with a virtual ROOT containing top-level subgraphs
#[derive(Debug, Clone)]
pub struct SubgraphTree {
    /// Map from subgraph ID (including "ROOT") to tree node
    pub nodes: HashMap<String, SubgraphTreeNode>,
    /// The root ID is always "ROOT"
    pub root_id: String,
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
