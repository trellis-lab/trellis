use criterion::{criterion_group, BenchmarkId, Criterion};
use std::path::Path;
use trellis_core::{
    config::{configuration_factory, ConfigurationType, TrellisConfig},
    pipeline::render,
    types::OutputFormat,
};
use trellis_parser::parse;

/// All benchmark fixtures (name, file)
const FIXTURES: &[(&str, &str)] = &[
    ("b01_linear_chain", "b01.mmd"),
    ("b02_wide_branch", "b02.mmd"),
    ("b03_k33_bipartite", "b03.mmd"),
    ("b04_diamond", "b04.mmd"),
    ("b05_star", "b05.mmd"),
    ("b06_multi_edge", "b06.mmd"),
    ("b07_cycle", "b07.mmd"),
    ("b08_nested_subgraph", "b08.mmd"),
    ("b09_subgraph_edges", "b09.mmd"),
    ("b10_50node_flowchart", "b10.mmd"),
    ("b11_100node_er", "b11.mmd"),
    ("b12_class_hierarchy", "b12.mmd"),
    ("b13_graph_edge_labels", "b13.mmd"),
    ("b14_all_flowchart_shapes", "b14.mmd"),
    ("b15_class_edges", "b15.mmd"),
    ("b16_all_er_edges", "b16.mmd"),
    ("b17_c4_long_texts", "b17.mmd"),
    ("b18_c4_support", "b18.mmd"),
    ("b19_c4_deployments", "b19.mmd"),
];

fn fixture_path(file: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(file)
}

// ---------------------------------------------------------------------------
// Timing benchmarks
// ---------------------------------------------------------------------------

/// Full-pipeline benchmark: parse → placement → grid → ports → routing → SVG
fn bench_render_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_pipeline");

    for &(name, file) in FIXTURES {
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
                let config = configuration_factory(ConfigurationType::Benchmark);
                render(&graph, &config, OutputFormat::Svg).expect("render failed")
            });
        });
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

criterion_group!(timing_benches, bench_render_pipeline, bench_parse_only);

// ---------------------------------------------------------------------------
// One-shot routing quality report (not a timing benchmark)
//
// Columns:
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
         {:<30} {:>5} {:>8} {:>8} {:>6} {:>8} {:>8} {:>9} {:>8} {:>6}",
        "fixture",
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
    eprintln!("{}", "-".repeat(102));

    for &(name, file) in FIXTURES {
        let source = match std::fs::read_to_string(fixture_path(file)) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let graph = parse(&source).expect("parse failed");
        let config = TrellisConfig::default();
        let result = render(&graph, &config, OutputFormat::Svg).expect("render failed");

        let svg_path = out_dir.join(format!("{}.svg", name));
        std::fs::write(&svg_path, &result.data).expect("failed to write SVG");

        let m = &result.metrics;

        eprintln!(
            "{:<30} {:>5} {:>8.1} {:>8} {:>6.2} {:>8.1} {:>8} {:>9} {:>8} {:>6}",
            name,
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

    eprintln!("{}", "-".repeat(102));
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
