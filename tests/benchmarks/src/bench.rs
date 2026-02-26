use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::path::Path;
use trellis_core::{config::TrellisConfig, pipeline::render, types::OutputFormat};
use trellis_parser::parse;

fn load_fixture(name: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(manifest_dir).join("fixtures").join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Failed to read fixture: {}", path.display()))
}

/// Full-pipeline benchmark: parse → placement → grid → ports → routing → SVG
fn bench_render_pipeline(c: &mut Criterion) {
    // B01–B12 fixture files in tests/benchmarks/fixtures/
    let fixtures: &[(&str, &str)] = &[
        ("b01_linear_chain",       "b01.mmd"),
        ("b02_wide_branch",        "b02.mmd"),
        ("b03_k33_bipartite",      "b03.mmd"),
        ("b04_diamond",            "b04.mmd"),
        ("b05_star",               "b05.mmd"),
        ("b06_multi_edge",         "b06.mmd"),
        ("b07_cycle",              "b07.mmd"),
        ("b08_nested_subgraph",    "b08.mmd"),
        ("b09_subgraph_edges",     "b09.mmd"),
        ("b10_50node_flowchart",   "b10.mmd"),
        ("b11_100node_er",         "b11.mmd"),
        ("b12_class_hierarchy",    "b12.mmd"),
    ];

    let mut group = c.benchmark_group("render_pipeline");

    for &(name, file) in fixtures {
        // Skip fixtures that don't exist yet (e.g., b11 ER if not yet present)
        let source = match std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures").join(file),
        ) {
            Ok(s) => s,
            Err(_) => {
                eprintln!("Skipping missing fixture: {}", file);
                continue;
            }
        };

        group.bench_with_input(
            BenchmarkId::new("render", name),
            &source,
            |b, src| {
                b.iter(|| {
                    let graph = parse(src).expect("parse failed");
                    let config = TrellisConfig::default();
                    render(&graph, &config, OutputFormat::Svg).expect("render failed")
                });
            },
        );
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
        let source = match std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures").join(file),
        ) {
            Ok(s) => s,
            Err(_) => continue,
        };

        group.bench_with_input(BenchmarkId::new("parse", name), &source, |b, src| {
            b.iter(|| parse(src).expect("parse failed"));
        });
    }

    group.finish();
}

criterion_group!(benches, bench_render_pipeline, bench_parse_only);
criterion_main!(benches);
