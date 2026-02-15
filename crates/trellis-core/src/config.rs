use serde::{Deserialize, Serialize};

/// Configuration for the Trellis rendering engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrellisConfig {
    /// Grid cell size in pixels
    pub cell_size: i32,

    /// Density factor for grid calculation (higher = more space)
    pub density_factor: f64,

    /// Safety multiplier for grid extent
    pub safety_multiplier: f64,

    /// Cost constants for A* routing
    pub routing_costs: RoutingCosts,

    /// Decomposition settings for large graphs
    pub decomposition: DecompositionMode,

    /// Threshold for triggering decomposition (number of nodes)
    pub decomposition_threshold: usize,

    /// Corner radius for rounded edges (in pixels)
    pub corner_radius: f64,
}

/// A* routing cost constants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingCosts {
    pub base_cost: f64,
    pub bend_cost: f64,
    pub adjacent_cost: f64,
    pub crossing_cost: f64,
    pub blocked_cost: f64,
}

/// Decomposition mode for handling large graphs
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DecompositionMode {
    None,
    Single,
    Multi,
}

impl Default for TrellisConfig {
    fn default() -> Self {
        Self {
            cell_size: 10,
            density_factor: 1.5,
            safety_multiplier: 1.2,
            routing_costs: RoutingCosts::default(),
            decomposition: DecompositionMode::None,
            decomposition_threshold: 50,
            corner_radius: 8.0,
        }
    }
}

impl Default for RoutingCosts {
    fn default() -> Self {
        Self {
            base_cost: 1.0,
            bend_cost: 2.0,
            adjacent_cost: 0.5,
            crossing_cost: 10.0,
            blocked_cost: 1000.0,
        }
    }
}
