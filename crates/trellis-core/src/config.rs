use crate::theme::ThemeName;
use serde::{Deserialize, Serialize};

// ─── per-field default functions ──────────────────────────────────────────────

fn default_theme() -> ThemeName {
    ThemeName::Default
}

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
fn default_crossing_style() -> CrossingStyle {
    CrossingStyle::None
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
    PortAssignmentStrategy::TrellisBasic
}
fn default_port_refinement_rounds() -> usize {
    0
}
fn default_flow_bias() -> FlowBias {
    FlowBias::None
}
fn default_bend_threshold() -> BendThreshold {
    BendThreshold::Auto
}
fn default_crossing_reroute() -> bool {
    true
}
fn default_show_title() -> bool {
    true
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

    /// Crossing rendering style — how edge crossings are drawn.
    #[serde(default = "default_crossing_style")]
    pub crossing_style: CrossingStyle,

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

    /// Flow-direction bias for port side selection.
    /// Auto applies bias when graph direction is TB or LR.
    #[serde(default = "default_flow_bias")]
    pub flow_bias: FlowBias,

    /// When true, emit diagnostic messages (e.g. Auto strategy selection) to stderr.
    /// Controlled by the CLI `--metrics` flag.
    #[serde(default)]
    pub print_metrics: bool,

    /// Bend threshold that triggers quality rerouting.
    ///
    /// - `Disabled` — quality reroute never runs.
    /// - `Fixed(n)` — reroutes any edge with more than `n` bends.
    /// - `Auto` (default) — computes `max(2, median_bends + 2)` from the
    ///   routed paths at runtime, targeting only the long tail of outliers.
    #[serde(default = "default_bend_threshold")]
    pub bend_threshold: BendThreshold,

    /// Enable crossing-reduction rerouting (Phase 5c).
    ///
    /// When `true` (default), edges with at least one geometric crossing are
    /// ripped up and re-routed through all 16 source/target side combinations.
    /// The new route is kept only if it reduces the total crossing count
    /// against all other committed paths.  Set to `false` to disable.
    #[serde(default = "default_crossing_reroute")]
    pub crossing_reroute: bool,

    /// Color theme for diagram rendering.
    ///
    /// Built-in themes: `default`, `paper`, `blueprint`, `dark`, `midnight`, `forest`.
    #[serde(default = "default_theme")]
    pub theme: ThemeName,

    /// Render the diagram title as a visible caption above the diagram.
    ///
    /// When `false`, the title is still extracted and stored in `Graph.title`
    /// (for tooling / metadata use), and the SVG `<title>` accessibility element
    /// is still emitted — only the visible caption and its viewBox expansion are suppressed.
    #[serde(default = "default_show_title")]
    pub show_title: bool,

    /// Path to write the structured pipeline debug log (JSON).
    ///
    /// `None` means no debug log is written. Set by the `--debug-log` CLI flag;
    /// not meaningful in TOML config files.
    ///
    /// Only compiled when the `debug-log` feature is enabled.
    #[cfg(feature = "debug-log")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub debug_log_path: Option<std::path::PathBuf>,
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
    /// Greedy center-first with corner-distance side selection (default)
    #[default]
    TrellisBasic,
    /// Legacy angle-based even-distribution (kept for regression testing)
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

/// How strongly to bias port side selection toward the layout flow direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FlowBias {
    /// Apply bias for Sugiyama (TB/LR/BT/RL), not for force-directed or C4.
    #[default]
    Auto,
    /// Always apply directional bias regardless of diagram type.
    Strong,
    /// Use original uniform 90° quadrants (old behaviour).
    None,
}

/// Bend count threshold that controls quality rerouting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BendThreshold {
    /// Quality reroute is disabled.
    Disabled,
    /// Reroute any edge whose bend count exceeds this fixed value.
    Fixed(usize),
    /// Compute threshold as `max(2, median_bends + 2)` from the routed paths.
    /// Targets only the long tail of outliers while leaving well-routed edges alone.
    #[default]
    Auto,
}

/// How edge crossings are rendered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CrossingStyle {
    /// Straight lines pass through each other — no special decoration (default).
    #[default]
    None,
    /// Semicircular hop arc: `_͡_` — the second occupant hops over the first.
    Arc,
    /// Rectangular bump: `_|‾|_` — square bridge orthogonal to travel direction.
    Rectangular,
    /// Gap/skip: `-| |-` — the second occupant's stroke is broken at the crossing.
    Skip,
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
            crossing_style: default_crossing_style(),
            show_grid: default_show_grid(),
            show_edge_labels: default_show_edge_labels(),
            port_assignment: default_port_assignment(),
            port_refinement_rounds: default_port_refinement_rounds(),
            flow_bias: FlowBias::None,
            print_metrics: false,
            bend_threshold: BendThreshold::Auto,
            crossing_reroute: true,
            theme: ThemeName::Default,
            show_title: true,
            #[cfg(feature = "debug-log")]
            debug_log_path: None,
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
            crossing_style: CrossingStyle::Arc,
            show_grid: false,
            show_edge_labels: true,
            port_assignment: PortAssignmentStrategy::Default,
            port_refinement_rounds: 0,
            flow_bias: FlowBias::Auto,
            print_metrics: false,
            bend_threshold: BendThreshold::Auto,
            crossing_reroute: true,
            theme: ThemeName::Default,
            show_title: true,
            #[cfg(feature = "debug-log")]
            debug_log_path: None,
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
            PortAssignmentStrategy::TrellisBasic
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

    #[test]
    fn show_title_defaults_true() {
        let config: TrellisConfig = serde_json::from_str("{}").unwrap();
        assert!(config.show_title);
    }

    #[test]
    fn show_title_parses_false() {
        let config: TrellisConfig = serde_json::from_str(r#"{"show_title": false}"#).unwrap();
        assert!(!config.show_title);
    }
}
