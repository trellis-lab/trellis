use serde::{Deserialize, Serialize};

// ─── per-field default functions ──────────────────────────────────────────────

fn default_cell_size() -> i32 {
    10
}
fn default_density_factor() -> f64 {
    1.5
}
fn default_safety_multiplier() -> f64 {
    1.2
}
fn default_corner_radius() -> f64 {
    8.0
}
fn default_render_crossings() -> bool {
    false
}
fn default_show_grid() -> bool {
    true
}
fn default_show_edge_labels() -> bool {
    false
}
fn default_decomposition() -> DecompositionMode {
    DecompositionMode::None
}
fn default_decomposition_threshold() -> usize {
    50
}
fn default_port_assignment() -> PortAssignmentStrategy {
    PortAssignmentStrategy::Default
}
fn default_port_refinement_rounds() -> usize {
    0
}
fn default_routing_costs() -> RoutingCosts {
    RoutingCosts::default()
}

fn default_base_cost() -> f64 {
    1.0
}
fn default_bend_cost() -> f64 {
    2.0
}
fn default_adjacent_cost() -> f64 {
    0.5
}
fn default_crossing_cost() -> f64 {
    10.0
}
fn default_blocked_cost() -> f64 {
    1000.0
}
fn default_perpendicular_cost() -> f64 {
    6.0
}

// ─── config structs ───────────────────────────────────────────────────────────

/// Configuration for the Trellis rendering engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrellisConfig {
    /// Grid cell size in pixels
    #[serde(default = "default_cell_size")]
    pub cell_size: i32,

    /// Density factor for grid calculation (higher = more space)
    #[serde(default = "default_density_factor")]
    pub density_factor: f64,

    /// Safety multiplier for grid extent
    #[serde(default = "default_safety_multiplier")]
    pub safety_multiplier: f64,

    /// Cost constants for A* routing
    #[serde(default = "default_routing_costs")]
    pub routing_costs: RoutingCosts,

    /// Decomposition settings for large graphs
    #[serde(default = "default_decomposition")]
    pub decomposition: DecompositionMode,

    /// Threshold for triggering decomposition (number of nodes)
    #[serde(default = "default_decomposition_threshold")]
    pub decomposition_threshold: usize,

    /// Corner radius for rounded edges (in pixels)
    #[serde(default = "default_corner_radius")]
    pub corner_radius: f64,

    /// Enable/disable rendering line jumps
    #[serde(default = "default_render_crossings")]
    pub render_crossings: bool,

    /// Enable/disable rendering of the grid system
    #[serde(default = "default_show_grid")]
    pub show_grid: bool,

    /// Enable/disable displaying edge captions
    #[serde(default = "default_show_edge_labels")]
    pub show_edge_labels: bool,

    /// Port assignment algorithm
    #[serde(default = "default_port_assignment")]
    pub port_assignment: PortAssignmentStrategy,

    /// Maximum port refinement rounds after routing (0 = disabled).
    /// Used by multi-round strategies (IterativeSwap, TwoPhase) to
    /// re-route edges after swapping crossing ports.
    #[serde(default = "default_port_refinement_rounds")]
    pub port_refinement_rounds: usize,
}

/// A* routing cost constants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingCosts {
    #[serde(default = "default_base_cost")]
    pub base_cost: f64,
    #[serde(default = "default_bend_cost")]
    pub bend_cost: f64,
    #[serde(default = "default_adjacent_cost")]
    pub adjacent_cost: f64,
    #[serde(default = "default_crossing_cost")]
    pub crossing_cost: f64,
    #[serde(default = "default_blocked_cost")]
    pub blocked_cost: f64,
    /// Penalty for moving parallel to a node boundary near connector points.
    /// Forces edges to approach/leave nodes perpendicularly.
    #[serde(default = "default_perpendicular_cost")]
    pub perpendicular_cost: f64,
}

/// Port assignment algorithm selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PortAssignmentStrategy {
    #[default]
    Default,
    /// Barycenter ordering — orders by weighted average position of target neighbourhood
    Barycenter,
    /// Median ordering — orders by median position, robust to outliers
    Median,
    /// Crossing-count greedy — minimises port inversions that cause crossings
    CrossingGreedy,
    /// Route-then-swap — routes, finds crossings, swaps ports, re-routes (multi-round)
    IterativeSwap,
    /// Two-phase: fast crossing estimate + targeted re-route of crossing edges (multi-round)
    TwoPhase,
    /// Auto: analyses graph stats and selects the best strategy automatically
    Auto,
}

/// Decomposition mode for handling large graphs
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum DecompositionMode {
    #[default]
    None,
    Single,
    Multi,
}

impl Default for TrellisConfig {
    fn default() -> Self {
        Self {
            cell_size: default_cell_size(),
            density_factor: default_density_factor(),
            safety_multiplier: default_safety_multiplier(),
            routing_costs: RoutingCosts::default(),
            decomposition: DecompositionMode::None,
            decomposition_threshold: default_decomposition_threshold(),
            corner_radius: default_corner_radius(),
            render_crossings: default_render_crossings(),
            show_grid: default_show_grid(),
            show_edge_labels: default_show_edge_labels(),
            port_assignment: default_port_assignment(),
            port_refinement_rounds: default_port_refinement_rounds(),
        }
    }
}

impl Default for RoutingCosts {
    fn default() -> Self {
        Self {
            base_cost: default_base_cost(),
            bend_cost: default_bend_cost(),
            adjacent_cost: default_adjacent_cost(),
            crossing_cost: default_crossing_cost(),
            blocked_cost: default_blocked_cost(),
            perpendicular_cost: default_perpendicular_cost(),
        }
    }
}

pub enum ConfigurationType {
    Basic,
    Benchmark,
}

pub fn configuration_factory(config_type: ConfigurationType) -> TrellisConfig {
    match config_type {
        ConfigurationType::Basic => TrellisConfig::default(),
        ConfigurationType::Benchmark => TrellisConfig {
            cell_size: 10,
            density_factor: 1.5,
            safety_multiplier: 1.2,
            routing_costs: RoutingCosts::default(),
            decomposition: DecompositionMode::None,
            decomposition_threshold: 50,
            corner_radius: 8.0,
            render_crossings: true,
            show_grid: false,
            show_edge_labels: true,
            port_assignment: PortAssignmentStrategy::Default,
            port_refinement_rounds: 0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn port_assignment_defaults_when_absent() {
        let config: TrellisConfig = serde_json::from_str("{}").unwrap();
        assert!(matches!(
            config.port_assignment,
            PortAssignmentStrategy::Default
        ));
    }

    #[test]
    fn port_assignment_parses_from_json() {
        let config: TrellisConfig =
            serde_json::from_str(r#"{"port_assignment": "Default"}"#).unwrap();
        assert!(matches!(
            config.port_assignment,
            PortAssignmentStrategy::Default
        ));
    }
}
