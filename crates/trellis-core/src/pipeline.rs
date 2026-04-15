use std::collections::HashMap;

use crate::{
    config::TrellisConfig,
    grid::{build_grid, calculate_grid_extent, Grid},
    labels, placement,
    ports::{
        compute_topo_rank, create_port_assigner, needs_refinement, straight_edge_prepass,
        EdgePorts, PortAssignmentContext,
    },
    routing::{self, commit::reconcile_crossings, RoutingResult},
    types::*,
};
use trellis_parser::{DiagramType, Graph};

#[cfg(feature = "debug-log")]
use crate::debug::{
    CrossingLog, CrossingReroutePhase, CrossingsPhase, DeadlockPhase, DebugLog, EdgeRoutingLog,
    GridPhase, LabelPlacementLog, LabelsPhase, NodePos, PlacementPhase, PortAssignment,
    PortsPhase, QualityReroutePhase, RoutingPhase, StraightEdgePin,
};

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

// ─── Inner pipeline ───────────────────────────────────────────────────────────

/// All outputs produced by one full pipeline run.
/// Shared by `render` and `render_with_validation`.
struct PipelineOutput {
    /// The graph after placement (node positions mutated).
    graph: Graph,
    /// Raw SVG bytes.
    svg_data: Vec<u8>,
    /// Aggregated rendering metrics.
    metrics: RenderMetrics,
    /// Committed routing result (paths, crossings, bends).
    routing_result: RoutingResult,
    /// Source/target port for every routed edge.
    port_assignments: HashMap<usize, EdgePorts>,
    /// Final committed routing grid.
    grid: Grid,
}

/// Run the full pipeline and return every intermediate result.
///
/// Both `render` and `render_with_validation` call this function; the latter
/// keeps the extra fields, the former drops them after extracting SVG + metrics.
fn run_pipeline(
    graph: &Graph,
    config: &TrellisConfig,
    render_ms: u64,
) -> Result<PipelineOutput, RenderError> {
    let mut graph = graph.clone();
    let cell_size = config.cell_size;

    // Debug log — constructed only when --debug-log is active and feature compiled in.
    #[cfg(feature = "debug-log")]
    let mut debug_log: Option<DebugLog> = config.debug_log_path.as_ref().map(|_| {
        let dt = format!("{:?}", graph.diagram_type);
        DebugLog::new(dt, cell_size)
    });

    // Phase 2: Node placement
    let subgraph_data = placement::place_nodes(&mut graph, cell_size);

    // P3 — capture placement phase
    #[cfg(feature = "debug-log")]
    if let Some(ref mut log) = debug_log {
        let algorithm = match graph.diagram_type {
            DiagramType::Flowchart => "sugiyama",
            DiagramType::ClassDiagram => "class",
            DiagramType::ErDiagram => "force-directed",
            DiagramType::C4Diagram => "row-flow",
        };
        log.phases.placement = PlacementPhase {
            algorithm: algorithm.to_string(),
            iterations: None, // populated by P3 inner instrumentation
            node_positions: graph
                .nodes
                .iter()
                .map(|n| NodePos {
                    id: n.id.clone(),
                    x: n.x,
                    y: n.y,
                })
                .collect(),
        };
    }

    // Phase 2.5: Resolve subgraph edges (flowchart only)
    if graph.diagram_type == DiagramType::Flowchart {
        if let Some((ref _tree, ref boxes)) = subgraph_data {
            placement::subgraph::resolve_subgraph_edges(&mut graph, boxes);
        }
    }

    // Phase 3: Grid construction (node footprints only — no edges yet)
    let extent = calculate_grid_extent(&graph, cell_size);
    let prepass_grid = build_grid(&graph, cell_size, &extent);

    // P3 — capture grid phase
    #[cfg(feature = "debug-log")]
    if let Some(ref mut log) = debug_log {
        log.phases.grid = GridPhase {
            cols: prepass_grid.cols,
            rows: prepass_grid.rows,
            offset_x: prepass_grid.offset_x,
            offset_y: prepass_grid.offset_y,
        };
    }

    // Phase 3a: Straight-edge pre-pass — pin ports for axis-aligned node pairs
    let pinned = straight_edge_prepass(
        &graph,
        &prepass_grid,
        cell_size,
        extent.offset_x,
        extent.offset_y,
    );

    // Phase 4: Port assignment
    // Collect pinned edge indices before moving the map into the context so
    // Phase 5b (port_swap) can use them to guard straight edges.
    let pinned_indices: std::collections::HashSet<usize> = pinned.keys().cloned().collect();
    let topo_rank = compute_topo_rank(&graph);
    let assigner = create_port_assigner(config.port_assignment);
    let port_ctx = PortAssignmentContext {
        graph: &graph,
        cell_size,
        offset_x: extent.offset_x,
        offset_y: extent.offset_y,
        grid: &prepass_grid,
        pinned_ports: pinned,
        flow_bias: config.flow_bias,
        topo_rank,
        print_metrics: config.print_metrics,
    };
    let port_assignments = assigner.assign_ports(&port_ctx);

    // P4 — capture ports phase (final assignments; candidate lists need inner instrumentation)
    #[cfg(feature = "debug-log")]
    if let Some(ref mut log) = debug_log {
        use crate::ports::assignment::Side;
        let strategy_name = format!("{:?}", config.port_assignment);
        let straight_pins: Vec<StraightEdgePin> = pinned_indices
            .iter()
            .map(|&idx| StraightEdgePin {
                edge_index: idx,
                reason: "straight-edge-prepass".to_string(),
            })
            .collect();

        // Build per-node assignment summary from the flat port_assignments map.
        let mut by_node: std::collections::BTreeMap<String, Vec<(usize, &crate::ports::EdgePorts)>> =
            std::collections::BTreeMap::new();
        for (edge_idx, ep) in &port_assignments {
            if let Some(edge) = graph.edges.get(*edge_idx) {
                by_node
                    .entry(edge.from.clone())
                    .or_default()
                    .push((*edge_idx, ep));
            }
        }
        let assignments = by_node
            .into_iter()
            .map(|(node_id, edges)| {
                use crate::debug::EdgePortLog;
                let edge_logs = edges
                    .into_iter()
                    .map(|(edge_idx, ep)| {
                        let side_str = |s: &Side| match s {
                            Side::Top => "Top",
                            Side::Right => "Right",
                            Side::Bottom => "Bottom",
                            Side::Left => "Left",
                        };
                        EdgePortLog {
                            edge_index: edge_idx,
                            edge_label: graph
                                .edges
                                .get(edge_idx)
                                .and_then(|e| e.label.clone()),
                            candidates: vec![],
                            selected: PortAssignment {
                                side: side_str(&ep.source_port.side).to_string(),
                                connector: (
                                    ep.source_port.grid_col as i32,
                                    ep.source_port.grid_row as i32,
                                ),
                            },
                            rejection_reasons: vec![],
                        }
                    })
                    .collect();
                crate::debug::NodePortLog {
                    node_id,
                    edges: edge_logs,
                }
            })
            .collect();

        log.phases.ports = PortsPhase {
            strategy: strategy_name,
            straight_edge_prepass: straight_pins,
            assignments,
        };
    }

    // Phase 5-6: Edge routing (A* pathfinding), with optional refinement loop
    let use_refinement = if config.port_assignment == crate::config::PortAssignmentStrategy::Auto {
        config.port_refinement_rounds > 0
            && crate::ports::auto::auto_needs_refinement(
                &graph,
                cell_size,
                extent.offset_x,
                extent.offset_y,
            )
    } else {
        needs_refinement(config.port_assignment) && config.port_refinement_rounds > 0
    };

    #[allow(unused_mut)]
    let (mut port_assignments, mut grid, mut routing_result) = if use_refinement {
        crate::ports::iterative::refine_ports(
            &graph,
            config,
            port_assignments,
            config.port_refinement_rounds,
        )
    } else {
        // Reuse the pre-pass grid for routing (still has only node footprints).
        let mut grid = prepass_grid;
        let result = routing::route_all_edges(&graph, &mut grid, &port_assignments, config);
        (port_assignments, grid, result)
    };

    // P5 — capture routing phase (final paths; individual A* attempts need inner instrumentation)
    #[cfg(feature = "debug-log")]
    if let Some(ref mut log) = debug_log {
        let edge_logs: Vec<EdgeRoutingLog> = routing_result
            .paths
            .iter()
            .map(|(&edge_idx, routed)| {
                let edge_label = graph.edges.get(edge_idx).and_then(|e| e.label.clone());
                let final_path = routed
                    .points
                    .iter()
                    .map(|p| (p.row, p.col))
                    .collect::<Vec<_>>();
                let path_length = routed.points.len();
                EdgeRoutingLog {
                    edge_index: edge_idx,
                    edge_label,
                    priority_score: 0.0, // needs inner instrumentation
                    attempts: vec![],    // needs inner instrumentation
                    selected_attempt: 0,
                    final_path,
                    bend_count: routed.bend_count,
                    path_length,
                    total_cost: routed.total_cost,
                }
            })
            .collect();
        log.phases.routing = RoutingPhase { edges: edge_logs };
    }

    // Phase 5a: Quality rerouting — rip-up edges with excessive bends and
    // try alternative port-side combinations to find a lower-bend route.
    if config.bend_threshold != crate::config::BendThreshold::Disabled {
        routing::quality_reroute::quality_reroute(
            &graph,
            &mut grid,
            &mut port_assignments,
            &mut routing_result.paths,
            config,
        );
        // Recompute aggregate bend count from the (possibly updated) paths.
        routing_result.total_bends = routing_result.paths.values().map(|p| p.bend_count).sum();
    }

    // P6 — capture quality reroute phase summary
    #[cfg(feature = "debug-log")]
    if let Some(ref mut log) = debug_log {
        use crate::config::BendThreshold;
        let (threshold_used, threshold_source) = match config.bend_threshold {
            BendThreshold::Disabled => (0, "disabled".to_string()),
            BendThreshold::Fixed(n) => (n, format!("fixed({n})")),
            BendThreshold::Auto => {
                // Mirror the auto formula from quality_reroute.rs
                let bends: Vec<usize> = routing_result.paths.values().map(|p| p.bend_count).collect();
                let median = if bends.is_empty() {
                    0
                } else {
                    let mut sorted = bends.clone();
                    sorted.sort_unstable();
                    sorted[sorted.len() / 2]
                };
                let t = median.saturating_add(2).max(2);
                (t, format!("auto(median={median})"))
            }
        };
        log.phases.quality_reroute = QualityReroutePhase {
            threshold_used,
            threshold_source,
            rerouted_edges: vec![], // per-edge detail needs inner instrumentation
        };
    }

    // Phase 5b: Port-swap pass — for each node side, detect adjacent port pairs
    // whose edges geometrically cross and swap them if doing so reduces bends.
    // Straight edges (pinned by the pre-pass) are never disturbed.
    routing::port_swap::port_swap(
        &graph,
        &mut grid,
        &mut port_assignments,
        &mut routing_result.paths,
        config,
        &pinned_indices,
    );
    routing_result.total_bends = routing_result.paths.values().map(|p| p.bend_count).sum();

    // P6 — capture port swap phase (swap details need inner instrumentation)
    #[cfg(feature = "debug-log")]
    if let Some(ref mut log) = debug_log {
        use crate::debug::PortSwapPhase;
        log.phases.port_swap = PortSwapPhase { swaps: vec![] };
    }

    // Phase 5c: Crossing-reduction reroute — rip up any edge that still
    // crosses another committed path and try all 16 side combinations,
    // keeping whichever reduces total crossings.  Runs after port_swap so
    // it only needs to handle crossings that the swap pass could not fix
    // (e.g. two edges that don't share a node side).
    #[cfg(feature = "debug-log")]
    let mut cr_details: Vec<crate::debug::CrossingRerouteLog> = Vec::new();

    let crossing_improved = if config.crossing_reroute {
        let n = routing::crossing_reroute::crossing_reroute(
            &graph,
            &mut grid,
            &mut port_assignments,
            &mut routing_result.paths,
            config,
            #[cfg(feature = "debug-log")]
            &mut cr_details,
        );
        routing_result.total_bends = routing_result.paths.values().map(|p| p.bend_count).sum();
        n
    } else {
        0
    };

    // P6 — capture crossing reroute phase
    #[cfg(feature = "debug-log")]
    if let Some(ref mut log) = debug_log {
        log.phases.crossing_reroute = CrossingReroutePhase {
            enabled: config.crossing_reroute,
            edges_rerouted: crossing_improved,
            details: cr_details,
        };
    }

    // Phase 5d: Second quality-reroute pass — runs only when crossing_reroute
    // moved at least one edge (the grid state has changed, so the median bend
    // count may be lower and previously-below-threshold edges can now qualify).
    if config.bend_threshold != crate::config::BendThreshold::Disabled && crossing_improved > 0 {
        routing::quality_reroute::quality_reroute(
            &graph,
            &mut grid,
            &mut port_assignments,
            &mut routing_result.paths,
            config,
        );
        routing_result.total_bends = routing_result.paths.values().map(|p| p.bend_count).sum();
    }

    // Phase 5e: Reconcile crossing metadata — rebuild grid crossing flags from
    // authoritative paths after all reroute passes. Ensures the grid reflects
    // actual committed paths (guards against stale flags from rerouting).
    reconcile_crossings(&mut grid, &routing_result.paths);

    // P7 — capture deadlock phase
    #[cfg(feature = "debug-log")]
    if let Some(ref mut log) = debug_log {
        log.phases.deadlock = DeadlockPhase {
            triggered: routing_result.deadlock_recoveries > 0,
            resolution_method: if routing_result.deadlock_recoveries > 0 {
                Some("rip-up-reroute".to_string())
            } else {
                None
            },
            edges_affected: vec![], // per-edge list needs deadlock instrumentation
        };
    }

    // Collect metrics
    let routed_count = routing_result.paths.len();
    let avg_edge_length = if routed_count > 0 {
        routing_result.total_path_length as f64 / routed_count as f64
    } else {
        0.0
    };
    #[cfg(feature = "diagnostics")]
    let avg_routing_cost = if routed_count > 0 {
        routing_result.total_routing_cost / routed_count as f64
    } else {
        0.0
    };
    #[cfg(feature = "diagnostics")]
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
        #[cfg(feature = "diagnostics")]
        grid_utilization: grid.utilization(),
        grid_rows: grid.rows,
        grid_cols: grid.cols,
        cell_size,
        port_count: port_assignments.len() * 2,
        total_edge_length: routing_result.total_path_length,
        total_routing_cost: routing_result.total_routing_cost,
        avg_edge_length,
        #[cfg(feature = "diagnostics")]
        avg_routing_cost,
        #[cfg(feature = "diagnostics")]
        max_edge_length: routing_result.max_path_length,
        #[cfg(feature = "diagnostics")]
        max_bends_per_edge: routing_result.max_bends_per_edge,
        #[cfg(feature = "diagnostics")]
        avg_detour_factor,
        deadlock_recoveries: routing_result.deadlock_recoveries,
    };

    // Phase 8: Edge label placement
    let label_placements = labels::place_all_labels(&graph, &routing_result.paths, &grid);

    // P7 — capture labels phase
    #[cfg(feature = "debug-log")]
    if let Some(ref mut log) = debug_log {
        // label_placements is a Vec<LabelPlacement>; we don't have edge_index here,
        // so emit by placement order (index == edge iteration order from place_all_labels).
        let logs: Vec<LabelPlacementLog> = label_placements
            .iter()
            .enumerate()
            .map(|(i, lp)| LabelPlacementLog {
                edge_index: i,
                text: lp.text.clone(),
                position: (lp.x, lp.y),
                collision_resolved: false, // inner flag not yet exposed
            })
            .collect();
        log.phases.labels = LabelsPhase { labels: logs };
    }

    // P7 — capture crossings phase from grid after reconcile
    #[cfg(feature = "debug-log")]
    if let Some(ref mut log) = debug_log {
        use crate::config::CrossingStyle;
        let style_str = match config.crossing_style {
            CrossingStyle::None => "None",
            CrossingStyle::Arc => "Arc",
            CrossingStyle::Rectangular => "Rectangular",
            CrossingStyle::Skip => "Skip",
        };
        let mut crossings: Vec<CrossingLog> = Vec::new();
        for row in 0..grid.rows {
            for col in 0..grid.cols {
                if let Some(cell) = grid.get(row, col) {
                    if cell.crossing {
                        if let (Some(owner), Some(hopper)) =
                            (&cell.owner, &cell.crossed_by)
                        {
                            let hop_rendered = !matches!(
                                config.crossing_style,
                                CrossingStyle::None
                            );
                            crossings.push(CrossingLog {
                                owner_edge: owner.clone(),
                                hopper_edge: hopper.clone(),
                                cell: (row, col),
                                hop_rendered,
                            });
                        }
                    }
                }
            }
        }
        log.phases.crossings = CrossingsPhase {
            style: style_str.to_string(),
            crossings,
        };
    }

    // Phase 9: SVG rendering
    let svg_data = crate::render::svg::build_svg(
        &graph,
        &grid,
        &routing_result,
        config,
        &label_placements,
        subgraph_data.as_ref(),
    );

    // Write debug log if active
    #[cfg(feature = "debug-log")]
    if let (Some(path), Some(log)) = (&config.debug_log_path, debug_log) {
        if let Err(e) = crate::debug::writer::write_debug_log(path, &log) {
            eprintln!("Warning: failed to write debug log to {:?}: {}", path, e);
        }
    }

    Ok(PipelineOutput {
        graph,
        svg_data,
        metrics,
        routing_result,
        port_assignments,
        grid,
    })
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Main rendering pipeline.
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

    let render_ms = elapsed_ms(start);
    let pipeline = run_pipeline(graph, config, render_ms)?;

    let data = format_output(pipeline.svg_data, format)?;

    Ok(RenderResult {
        format,
        data,
        metrics: pipeline.metrics,
    })
}

/// Render a diagram and return both the `RenderResult` and `ValidationData`
/// containing the intermediate pipeline outputs needed for quality analysis.
///
/// Only available when the `diagnostics` feature is enabled (default).
/// Not compiled into WASM builds.
#[cfg(feature = "diagnostics")]
pub fn render_with_validation(
    graph: &Graph,
    config: &TrellisConfig,
    format: OutputFormat,
) -> Result<(RenderResult, ValidationData), RenderError> {
    #[cfg(not(target_arch = "wasm32"))]
    let start = std::time::Instant::now();
    #[cfg(target_arch = "wasm32")]
    let start = ();

    let render_ms = elapsed_ms(start);
    let pipeline = run_pipeline(graph, config, render_ms)?;

    let svg_bytes = pipeline.svg_data.clone();
    let data = format_output(pipeline.svg_data, format)?;

    let validation = ValidationData {
        graph: pipeline.graph,
        routing_result: pipeline.routing_result,
        port_assignments: pipeline.port_assignments,
        grid: pipeline.grid,
        svg: svg_bytes,
    };

    Ok((
        RenderResult {
            format,
            data,
            metrics: pipeline.metrics,
        },
        validation,
    ))
}

// ─── Private helpers ──────────────────────────────────────────────────────────

fn format_output(svg_data: Vec<u8>, format: OutputFormat) -> Result<Vec<u8>, RenderError> {
    match format {
        OutputFormat::Svg => Ok(svg_data),
        #[cfg(feature = "png")]
        OutputFormat::Png => crate::render::png::svg_to_png(&svg_data).map_err(|e| RenderError {
            message: format!("PNG conversion failed: {}", e),
        }),
        #[cfg(not(feature = "png"))]
        OutputFormat::Png => Err(RenderError {
            message: "PNG output is not supported in this build (compile with feature 'png')"
                .to_string(),
        }),
    }
}

/// Count the number of distinct layers in the placed graph.
fn count_layers(graph: &Graph) -> usize {
    if graph.nodes.is_empty() {
        return 0;
    }

    let is_horizontal = matches!(
        graph.direction,
        trellis_parser::Direction::LR | trellis_parser::Direction::RL
    );

    let mut layer_values: Vec<i64> = graph
        .nodes
        .iter()
        .map(|n| {
            let val = if is_horizontal { n.x } else { n.y };
            (val * 10.0).round() as i64
        })
        .collect();
    layer_values.sort();
    layer_values.dedup();
    layer_values.len()
}

/// Error type for rendering failures.
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
