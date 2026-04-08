use criterion::{criterion_group, BenchmarkId, Criterion};
use std::path::Path;
use trellis_core::{
    config::{configuration_factory, ConfigurationType, PortAssignmentStrategy, TrellisConfig},
    pipeline::render,
    types::OutputFormat,
};
use trellis_parser::parse;

use PortAssignmentStrategy as PA;

/// Fixture descriptor: (name, file, recommended port assignment strategy).
///
/// The strategy reflects the graph topology:
/// - `Default`        — simple/linear graphs, no crossing risk
/// - `CrossingGreedy` — moderate complexity, fan-out, multi-edges
/// - `TwoPhase`       — complex hub topology at manageable size
/// - `Auto`           — let the selector decide (meta-benchmark)
const FIXTURES: &[(&str, &str, PortAssignmentStrategy)] = &[
    // ── Basic topology fixtures ──────────────────────────────
    ("b01_linear_chain", "b01.mmd", PA::Default),
    ("b02_wide_branch", "b02.mmd", PA::CrossingGreedy),
    ("b03_k33_bipartite", "b03.mmd", PA::CrossingGreedy),
    ("b04_diamond", "b04.mmd", PA::Default),
    ("b05_star", "b05.mmd", PA::CrossingGreedy),
    ("b06_multi_edge", "b06.mmd", PA::CrossingGreedy),
    ("b07_cycle", "b07.mmd", PA::Default),
    ("b08_nested_subgraph", "b08.mmd", PA::Default),
    ("b09_subgraph_edges", "b09.mmd", PA::Default),
    // ── Larger / typed diagrams ──────────────────────────────
    ("b10_50node_flowchart", "b10.mmd", PA::CrossingGreedy),
    ("b11_100node_er", "b11.mmd", PA::CrossingGreedy),
    ("b12_class_hierarchy", "b12.mmd", PA::CrossingGreedy),
    ("b13_graph_edge_labels", "b13.mmd", PA::Default),
    ("b14_all_flowchart_shapes", "b14.mmd", PA::Default),
    ("b15_class_edges", "b15.mmd", PA::CrossingGreedy),
    ("b16_all_er_edges", "b16.mmd", PA::CrossingGreedy),
    ("b17_c4_long_texts", "b17.mmd", PA::Default),
    ("b18_c4_support", "b18.mmd", PA::Default),
    ("b19_c4_deployments", "b19.mmd", PA::Default),
    // ── Port assignment algorithm fixtures ────────────────────
    ("b20_port_fan_out", "b20.mmd", PA::CrossingGreedy),
    ("b21_port_inversion", "b21.mmd", PA::CrossingGreedy),
    ("b22_port_congestion", "b22.mmd", PA::CrossingGreedy),
    ("b23_port_bipartite", "b23.mmd", PA::CrossingGreedy),
    ("b24_port_diamond_chain", "b24.mmd", PA::TwoPhase),
    // ── Auto mode fixtures ───────────────────────────────────
    ("b25_auto_hub_topology", "b25.mmd", PA::Auto),
    ("b26_auto_simple_chain", "b26.mmd", PA::Auto),
    ("b27_auto_moderate", "b27.mmd", PA::Auto),
    ("b28_auto_dense_mesh", "b28.mmd", PA::Auto),
    ("b29_auto_class_hierarchy", "b29.mmd", PA::Auto),
];

fn fixture_path(file: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(file)
}

/// Build a benchmark config with the given port assignment strategy.
fn bench_config(strategy: PortAssignmentStrategy) -> TrellisConfig {
    let mut config = configuration_factory(ConfigurationType::Benchmark);
    config.port_assignment = strategy;
    // Enable refinement rounds for multi-round strategies
    if matches!(strategy, PA::IterativeSwap | PA::TwoPhase | PA::Auto) {
        config.port_refinement_rounds = 3;
    }
    config
}

// ---------------------------------------------------------------------------
// Timing benchmarks
// ---------------------------------------------------------------------------

/// Full-pipeline benchmark: parse → placement → grid → ports → routing → SVG.
///
/// Each fixture runs with its recommended port assignment strategy.
fn bench_render_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_pipeline");

    for &(name, file, strategy) in FIXTURES {
        let source = match std::fs::read_to_string(fixture_path(file)) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("Skipping missing fixture: {}", file);
                continue;
            }
        };

        group.bench_with_input(BenchmarkId::new("render", name), &source, |b, src| {
            b.iter(|| {
                let graph = parse(src).expect("parse failed");
                let config = bench_config(strategy);
                render(&graph, &config, OutputFormat::Svg).expect("render failed")
            });
        });
    }

    group.finish();
}

/// Port assignment strategy comparison benchmark.
///
/// For a subset of interesting fixtures, compares Default vs CrossingGreedy
/// vs TwoPhase vs Auto to measure the cost/benefit of each strategy.
fn bench_port_strategies(c: &mut Criterion) {
    let comparison_fixtures: &[(&str, &str)] = &[
        ("b03_k33_bipartite", "b03.mmd"),
        ("b05_star", "b05.mmd"),
        ("b10_50node_flowchart", "b10.mmd"),
        ("b20_port_fan_out", "b20.mmd"),
        ("b24_port_diamond_chain", "b24.mmd"),
        ("b25_auto_hub_topology", "b25.mmd"),
        ("b28_auto_dense_mesh", "b28.mmd"),
    ];

    let strategies = [
        ("Default", PA::Default),
        ("CrossingGreedy", PA::CrossingGreedy),
        ("TwoPhase", PA::TwoPhase),
        ("Auto", PA::Auto),
    ];

    let mut group = c.benchmark_group("port_strategies");

    for &(fixture_name, file) in comparison_fixtures {
        let source = match std::fs::read_to_string(fixture_path(file)) {
            Ok(s) => s,
            Err(_) => continue,
        };

        for &(strategy_name, strategy) in &strategies {
            let id = format!("{}/{}", fixture_name, strategy_name);
            group.bench_with_input(BenchmarkId::new("render", &id), &source, |b, src| {
                b.iter(|| {
                    let graph = parse(src).expect("parse failed");
                    let config = bench_config(strategy);
                    render(&graph, &config, OutputFormat::Svg).expect("render failed")
                });
            });
        }
    }

    group.finish();
}

/// Parse-only benchmark (isolates parser from layout/rendering cost)
fn bench_parse_only(c: &mut Criterion) {
    let fixtures: &[(&str, &str)] = &[
        ("b01", "b01.mmd"),
        ("b10", "b10.mmd"),
        ("b11", "b11.mmd"),
        ("b12", "b12.mmd"),
    ];

    let mut group = c.benchmark_group("parse_only");

    for &(name, file) in fixtures {
        let source = match std::fs::read_to_string(fixture_path(file)) {
            Ok(s) => s,
            Err(_) => continue,
        };

        group.bench_with_input(BenchmarkId::new("parse", name), &source, |b, src| {
            b.iter(|| parse(src).expect("parse failed"));
        });
    }

    group.finish();
}

criterion_group!(
    timing_benches,
    bench_render_pipeline,
    bench_port_strategies,
    bench_parse_only
);

// ---------------------------------------------------------------------------
// One-shot routing quality report (not a timing benchmark)
//
// Columns:
//   strategy  – port assignment algorithm used
//   edges     – number of edges in the graph
//   avg_len   – average routed path length in grid steps
//   max_len   – longest single routed path in grid steps
//   dtour     – average detour factor (actual / manhattan); 1.0 = optimal
//   avg_cost  – average A* routing cost per edge
//   bends     – total bend count across all edges
//   max_bnds  – bend count of the worst single edge
//   crossings – grid cells where two paths overlap
//   dl_rec    – edges that needed the deadlock recovery handler
// ---------------------------------------------------------------------------
fn print_routing_quality_table() {
    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/tmp");
    std::fs::create_dir_all(&out_dir).expect("failed to create target/tmp");

    eprintln!();
    eprintln!(
        "=== Routing Quality Metrics ===\n\
         {:<30} {:<15} {:>5} {:>8} {:>8} {:>6} {:>8} {:>8} {:>9} {:>8} {:>6}",
        "fixture",
        "strategy",
        "edges",
        "avg_len",
        "max_len",
        "dtour",
        "avg_cost",
        "bends",
        "crossings",
        "max_bnds",
        "dl_rec"
    );
    eprintln!("{}", "-".repeat(117));

    for &(name, file, strategy) in FIXTURES {
        let source = match std::fs::read_to_string(fixture_path(file)) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let graph = parse(&source).expect("parse failed");
        let config = bench_config(strategy);
        let result = render(&graph, &config, OutputFormat::Svg).expect("render failed");

        let svg_path = out_dir.join(format!("{}.svg", name));
        std::fs::write(&svg_path, &result.data).expect("failed to write SVG");

        let m = &result.metrics;

        eprintln!(
            "{:<30} {:<15} {:>5} {:>8.1} {:>8} {:>6.2} {:>8.1} {:>8} {:>9} {:>8} {:>6}",
            name,
            format!("{:?}", strategy),
            m.edges,
            m.avg_edge_length,
            m.max_edge_length,
            m.avg_detour_factor,
            m.avg_routing_cost,
            m.bends,
            m.crossings,
            m.max_bends_per_edge,
            m.deadlock_recoveries,
        );
    }

    eprintln!("{}", "-".repeat(117));
    eprintln!(
        "  dtour = total_actual_steps / total_manhattan_steps; \
         dl_rec = edges needing deadlock recovery\n"
    );
}

// ---------------------------------------------------------------------------
// Custom main: quality table once, then Criterion timing benchmarks
// ---------------------------------------------------------------------------
fn main() {
    print_routing_quality_table();
    timing_benches();
}
