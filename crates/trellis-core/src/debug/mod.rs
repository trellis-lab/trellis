/// Structured debug log for the trellis rendering pipeline.
///
/// Populated in `run_pipeline()` when `config.debug_log_path` is `Some`.
/// Serialised to JSON via `debug::writer::write_debug_log`.
///
/// Only compiled when the `debug-log` feature is enabled.
pub mod writer;

use serde::Serialize;

// ─── Top-level ────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize)]
pub struct DebugLog {
    /// Schema version — increment on breaking changes.
    pub version: u8,
    pub input_file: Option<String>,
    pub diagram_type: String,
    pub cell_size: i32,
    pub phases: PipelinePhases,
}

impl DebugLog {
    pub fn new(diagram_type: String, cell_size: i32) -> Self {
        Self {
            version: 1,
            input_file: None,
            diagram_type,
            cell_size,
            phases: PipelinePhases::default(),
        }
    }
}

// ─── Phase structs ────────────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize)]
pub struct PipelinePhases {
    pub placement: PlacementPhase,
    pub grid: GridPhase,
    pub ports: PortsPhase,
    pub routing: RoutingPhase,
    pub quality_reroute: QualityReroutePhase,
    pub port_swap: PortSwapPhase,
    pub crossing_reroute: CrossingReroutePhase,
    pub deadlock: DeadlockPhase,
    pub labels: LabelsPhase,
    pub crossings: CrossingsPhase,
}

// ─── Placement ────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize)]
pub struct PlacementPhase {
    /// "sugiyama" | "force-directed" | "row-flow" | "class"
    pub algorithm: String,
    /// Force-directed only.
    pub iterations: Option<u32>,
    pub node_positions: Vec<NodePos>,
}

#[derive(Debug, Serialize)]
pub struct NodePos {
    pub id: String,
    pub x: f64,
    pub y: f64,
}

// ─── Grid ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize)]
pub struct GridPhase {
    pub cols: usize,
    pub rows: usize,
    pub offset_x: i32,
    pub offset_y: i32,
}

// ─── Ports ────────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize)]
pub struct PortsPhase {
    pub strategy: String,
    /// Edges pinned by the straight-edge pre-pass.
    pub straight_edge_prepass: Vec<StraightEdgePin>,
    /// Final port assignment per node (populated with available data;
    /// candidate lists require P4 instrumentation in the assigner modules).
    pub assignments: Vec<NodePortLog>,
}

#[derive(Debug, Serialize)]
pub struct StraightEdgePin {
    pub edge_index: usize,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct NodePortLog {
    pub node_id: String,
    pub edges: Vec<EdgePortLog>,
}

#[derive(Debug, Serialize)]
pub struct EdgePortLog {
    pub edge_index: usize,
    pub edge_label: Option<String>,
    /// Populated by P4 instrumentation inside the assigner; empty until then.
    pub candidates: Vec<PortCandidate>,
    pub selected: PortAssignment,
    /// Populated by P4 instrumentation inside the assigner; empty until then.
    pub rejection_reasons: Vec<RejectionNote>,
}

#[derive(Debug, Serialize)]
pub struct PortCandidate {
    /// "Top" | "Right" | "Bottom" | "Left"
    pub side: String,
    pub connector: (i32, i32),
    pub score: f64,
    pub rank: usize,
}

#[derive(Debug, Serialize)]
pub struct PortAssignment {
    pub side: String,
    pub connector: (i32, i32),
}

#[derive(Debug, Serialize)]
pub struct RejectionNote {
    pub connector: (i32, i32),
    /// "occupied" | "below-score" | "congestion"
    pub reason: String,
}

// ─── Routing ──────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize)]
pub struct RoutingPhase {
    pub edges: Vec<EdgeRoutingLog>,
}

#[derive(Debug, Serialize)]
pub struct EdgeRoutingLog {
    pub edge_index: usize,
    pub edge_label: Option<String>,
    /// Populated by P5 instrumentation inside the router; 0.0 until then.
    pub priority_score: f64,
    /// Populated by P5 instrumentation inside the router; empty until then.
    pub attempts: Vec<RoutingAttempt>,
    /// Index into `attempts` for the selected route.
    pub selected_attempt: usize,
    /// Final committed path as (row, col) pairs.
    pub final_path: Vec<(i64, i64)>,
    pub bend_count: usize,
    pub path_length: usize,
    /// Total A* cost of the selected route.
    pub total_cost: f64,
}

#[derive(Debug, Serialize)]
pub struct RoutingAttempt {
    pub source_port: (i32, i32),
    pub target_port: (i32, i32),
    pub result: RoutingAttemptResult,
    pub cost: Option<f64>,
    pub bend_count: Option<usize>,
    /// "no-path" | "occupied-cells: N" | "cost-above-threshold"
    pub rejection_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub enum RoutingAttemptResult {
    Success,
    Failed,
}

// ─── Quality reroute ──────────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize)]
pub struct QualityReroutePhase {
    pub threshold_used: usize,
    /// "disabled" | "fixed(N)" | "auto(median=M)"
    pub threshold_source: String,
    pub rerouted_edges: Vec<RerouteLog>,
}

#[derive(Debug, Serialize)]
pub struct RerouteLog {
    pub edge_index: usize,
    pub original_bends: usize,
    pub best_alternative_bends: usize,
    /// "improved" | "no-improvement-kept-original"
    pub outcome: String,
}

// ─── Port swap ────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize)]
pub struct PortSwapPhase {
    pub swaps: Vec<PortSwapLog>,
}

#[derive(Debug, Serialize)]
pub struct PortSwapLog {
    pub node_id: String,
    pub side: String,
    pub edge_a: usize,
    pub edge_b: usize,
    pub bends_before: usize,
    pub bends_after: usize,
}

// ─── Crossing reroute ─────────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize)]
pub struct CrossingReroutePhase {
    pub enabled: bool,
    pub edges_rerouted: usize,
    pub details: Vec<CrossingRerouteLog>,
}

#[derive(Debug, Serialize)]
pub struct CrossingRerouteLog {
    pub edge_index: usize,
    pub crossings_before: usize,
    pub crossings_after: usize,
    /// "improved" | "no-improvement-kept-original"
    pub outcome: String,
    /// One entry per (src_side, tgt_side) combination tried.
    pub attempts: Vec<CrossingAttempt>,
}

#[derive(Debug, Serialize)]
pub struct CrossingAttempt {
    /// "Top" | "Right" | "Bottom" | "Left"
    pub src_side: String,
    /// "Top" | "Right" | "Bottom" | "Left"
    pub tgt_side: String,
    pub result: CrossingAttemptResult,
    /// Human-readable rejection reason when result is not NewBest.
    pub rejection_reason: Option<String>,
    /// Crossing count against all other paths (None when route failed).
    pub crossings_after: Option<usize>,
    pub bend_count: Option<usize>,
    pub path_length: Option<usize>,
}

#[derive(Debug, Serialize)]
pub enum CrossingAttemptResult {
    /// `ports_for_sides` returned None — side has no connectors.
    NoConnectors,
    /// A* returned no path — all routes blocked by obstacles.
    NoPath,
    /// Path found but bend count or length exceeds budget.
    BudgetExceeded,
    /// Path found but does not reduce crossings below current best.
    StillCrossing,
    /// Path reduces crossings — new best candidate recorded.
    NewBest,
}

// ─── Deadlock ─────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize)]
pub struct DeadlockPhase {
    pub triggered: bool,
    /// "rip-up-reroute" | "grid-expansion" | "crossing-fallback"
    pub resolution_method: Option<String>,
    pub edges_affected: Vec<usize>,
}

// ─── Labels ───────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize)]
pub struct LabelsPhase {
    pub labels: Vec<LabelPlacementLog>,
}

#[derive(Debug, Serialize)]
pub struct LabelPlacementLog {
    pub edge_index: usize,
    pub text: String,
    pub position: (f64, f64),
    pub collision_resolved: bool,
}

// ─── Crossings ────────────────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize)]
pub struct CrossingsPhase {
    /// "None" | "Arc" | "Rectangular" | "Skip"
    pub style: String,
    pub crossings: Vec<CrossingLog>,
}

#[derive(Debug, Serialize)]
pub struct CrossingLog {
    pub owner_edge: String,
    pub hopper_edge: String,
    pub cell: (usize, usize),
    pub hop_rendered: bool,
}
