use clap::{Parser, Subcommand};
use rayon::prelude::*;
use std::ffi::OsStr;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use trellis_core::{PortAssignmentStrategy, ThemeName, TrellisConfig};

// ─── CLI definition ───────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "trellis")]
#[command(about = "A fast Mermaid diagram renderer")]
#[command(version)]
#[command(disable_help_subcommand = true)]
struct Cli {
    /// Load configuration from FILE instead of ~/.trellis/config.toml
    #[arg(long, global = true, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Color theme: default, paper, blueprint, dark, midnight, forest
    #[arg(long, global = true, value_name = "THEME")]
    theme: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Render a Mermaid diagram to SVG or PNG
    #[command(after_help = "\
EXAMPLES
  trellis render diagram.mmd -o diagram.svg
  trellis render diagram.mmd -o diagram.png -f png
  trellis render diagram.mmd -o diagram.svg --metrics
  cat diagram.mmd | trellis render - -f svg > diagram.svg")]
    Render {
        /// Input .mmd file path, or '-' to read from stdin
        input: PathBuf,

        /// Output file path, or '-' to write to stdout
        #[arg(short, long, default_value = "-", value_name = "FILE")]
        output: PathBuf,

        /// Output format: svg or png
        #[arg(short, long, default_value = "svg", value_name = "FMT")]
        format: String,

        /// Print rendering metrics as JSON to stderr
        #[arg(long)]
        metrics: bool,

        /// Write structured pipeline debug log to FILE.
        /// Omit FILE to use <output>.debug.json in the same directory.
        /// Requires build with --features debug-log.
        #[arg(long, value_name = "FILE", num_args = 0..=1, hide = true)]
        debug_log: Option<Option<PathBuf>>,
    },

    /// Render all .mmd files in a directory (parallel)
    #[command(after_help = "\
EXAMPLES
  trellis render-batch diagrams/ -o out/
  trellis render-batch diagrams/ -o out/ -f png")]
    RenderBatch {
        /// Directory containing .mmd source files (searched recursively)
        input_dir: PathBuf,

        /// Directory to write rendered output files (created if absent)
        #[arg(short, long, value_name = "DIR")]
        output_dir: PathBuf,

        /// Output format: svg or png
        #[arg(short, long, default_value = "svg", value_name = "FMT")]
        format: String,
    },

    /// Validate a Mermaid diagram and print a node/edge summary
    #[command(after_help = "\
EXAMPLES
  trellis validate diagram.mmd
  echo $?   # 0 = valid, 1 = parse error")]
    Validate {
        /// Input .mmd file path, or '-' to read from stdin
        input: PathBuf,
    },

    /// Preprocess a Markdown file: replace Mermaid code blocks with rendered images
    #[command(after_help = "\
EXAMPLES
  trellis preprocess doc.md -o out.md
  trellis preprocess doc.md -o out.md --image-dir assets/diagrams -f png")]
    Preprocess {
        /// Input Markdown file
        input: PathBuf,

        /// Output Markdown file
        #[arg(short, long, value_name = "FILE")]
        output: PathBuf,

        /// Directory to write rendered image files (created if absent)
        #[arg(long, default_value = "img", value_name = "DIR")]
        image_dir: PathBuf,

        /// Output format for images: svg or png
        #[arg(short, long, default_value = "svg", value_name = "FMT")]
        format: String,
    },

    /// Render a diagram and produce a quality report (.svg + .json)
    #[command(after_help = "\
EXAMPLES
  trellis evaluate diagram.mmd -o ./reports/
  trellis evaluate diagram.mmd -o ./reports/ --annotated")]
    Evaluate {
        /// Input .mmd file
        input: PathBuf,

        /// Output directory (created if absent)
        #[arg(short, long, value_name = "DIR")]
        output_dir: PathBuf,

        /// Also write an annotated SVG with quality colour overlay ({name}.annotated.svg)
        #[arg(long)]
        annotated: bool,
    },

    /// Evaluate all .mmd files in a directory, producing .svg + .json per fixture
    #[command(after_help = "\
EXAMPLES
  trellis evaluate-batch ./fixtures/ -o ./reports/
  trellis evaluate-batch ./fixtures/ -o ./reports/ --annotated
  trellis evaluate-batch ./fixtures/ -o ./reports/ --compare-strategies
  trellis --config custom.toml evaluate-batch ./fixtures/ -o ./reports/")]
    EvaluateBatch {
        /// Directory containing .mmd source files (searched recursively)
        input_dir: PathBuf,

        /// Output directory (created if absent)
        #[arg(short, long, value_name = "DIR")]
        output_dir: PathBuf,

        /// Also write annotated SVGs with quality colour overlay ({name}.annotated.svg)
        #[arg(long)]
        annotated: bool,

        /// Run every port-assignment strategy and write per-strategy outputs +
        /// a summary CSV ({name}_strategies.csv) for A/B comparison
        #[arg(long)]
        compare_strategies: bool,
    },

    /// Generate an interactive HTML review tool from a reports directory
    #[command(after_help = "\
EXAMPLES
  trellis generate-review ./reports/
  trellis generate-review ./reports/ -o custom-review.html

The input directory must contain .json reports produced by `evaluate` or
`evaluate-batch`.  For each .json file a matching .annotated.svg is used when
present; otherwise the plain .svg is used as a fallback.

Open the output HTML in any browser, click coloured edge overlays to annotate
them, then download feedback.json for the AI tuning agent.")]
    GenerateReview {
        /// Directory containing .json (and optionally .annotated.svg) files
        input_dir: PathBuf,

        /// Output HTML file [default: <input_dir>/review.html]
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Print a comprehensive usage reference for all commands
    Help,
}

// ─── error / exit-code handling ───────────────────────────────────────────────

enum AppError {
    /// Exit code 1 – diagram syntax is invalid
    Parse(String),
    /// Exit code 2 – rendering or I/O failure
    Render(String),
}

impl AppError {
    fn exit_code(&self) -> i32 {
        match self {
            AppError::Parse(_) => 1,
            AppError::Render(_) => 2,
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Parse(msg) => write!(f, "Parse error: {}", msg),
            AppError::Render(msg) => write!(f, "Render error: {}", msg),
        }
    }
}

// ─── helpers ──────────────────────────────────────────────────────────────────

/// Resolve `~/.trellis/config.toml` without an external crate.
fn default_config_path() -> Option<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()?;
    Some(PathBuf::from(home).join(".trellis").join("config.toml"))
}

/// Load `TrellisConfig` from a TOML file.  Missing fields fall back to defaults.
fn load_config(path: Option<&PathBuf>) -> TrellisConfig {
    let resolved = path.cloned().or_else(default_config_path);
    let Some(p) = resolved else {
        return TrellisConfig::default();
    };
    if !p.exists() {
        return TrellisConfig::default();
    }
    match fs::read_to_string(&p) {
        Err(e) => {
            eprintln!("Warning: cannot read config {:?}: {}", p, e);
            TrellisConfig::default()
        }
        Ok(content) => match toml::from_str::<TrellisConfig>(&content) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Warning: cannot parse config {:?}: {}", p, e);
                TrellisConfig::default()
            }
        },
    }
}

/// Read text from a file path or from stdin when path is `-`.
fn read_input(path: &PathBuf) -> Result<String, AppError> {
    if path == &PathBuf::from("-") {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| AppError::Render(format!("failed to read stdin: {}", e)))?;
        Ok(buf)
    } else {
        fs::read_to_string(path)
            .map_err(|e| AppError::Render(format!("failed to read {:?}: {}", path, e)))
    }
}

/// Write bytes to a file path or to stdout when path is `-`.
fn write_output(path: &PathBuf, data: &[u8]) -> Result<(), AppError> {
    if path == &PathBuf::from("-") {
        std::io::stdout()
            .write_all(data)
            .map_err(|e| AppError::Render(format!("failed to write to stdout: {}", e)))
    } else {
        fs::write(path, data)
            .map_err(|e| AppError::Render(format!("failed to write {:?}: {}", path, e)))
    }
}

/// Parse a format string into `OutputFormat`.
fn parse_format(format: &str) -> Result<trellis_core::OutputFormat, AppError> {
    match format {
        "svg" => Ok(trellis_core::OutputFormat::Svg),
        "png" => Ok(trellis_core::OutputFormat::Png),
        other => Err(AppError::Render(format!(
            "unsupported format '{}' (use svg or png)",
            other
        ))),
    }
}

/// Count all subgraphs recursively (including nested ones).
fn count_subgraphs(subgraphs: &[trellis_parser::Subgraph]) -> usize {
    subgraphs
        .iter()
        .map(|sg| 1 + count_subgraphs(&sg.subgraphs))
        .sum()
}

// ─── command implementations ──────────────────────────────────────────────────

/// Derive the default debug-log path from the output path:
/// `<stem>.debug.json` in the same directory.
#[cfg(feature = "debug-log")]
fn derive_default_debug_path(output: &Path) -> PathBuf {
    let stem = output
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    output.with_file_name(format!("{stem}.debug.json"))
}

fn cmd_render(
    input: &PathBuf,
    output: &PathBuf,
    format: &str,
    print_metrics: bool,
    #[cfg_attr(not(feature = "debug-log"), allow(unused_variables))]
    debug_log_flag: &Option<Option<PathBuf>>,
    config: &TrellisConfig,
) -> Result<(), AppError> {
    let content = read_input(input)?;

    let graph = trellis_parser::parse(&content).map_err(|e| AppError::Parse(e.to_string()))?;

    let output_format = parse_format(format)?;

    let mut config = config.clone();
    config.print_metrics = print_metrics;

    #[cfg(feature = "debug-log")]
    {
        config.debug_log_path = match debug_log_flag {
            None => None,
            Some(None) => Some(derive_default_debug_path(output)),
            Some(Some(p)) => Some(p.clone()),
        };
    }

    let result = trellis_core::render(&graph, &config, output_format)
        .map_err(|e| AppError::Render(e.to_string()))?;

    write_output(output, &result.data)?;

    // Only print path confirmation when writing to a real file
    if output != &PathBuf::from("-") {
        println!("Rendered to: {:?}", output);
    }

    if print_metrics {
        let json = serde_json::to_string_pretty(&result.metrics)
            .map_err(|e| AppError::Render(format!("metrics serialisation failed: {}", e)))?;
        eprintln!("{}", json);
    }

    Ok(())
}

fn cmd_render_batch(
    input_dir: &PathBuf,
    output_dir: &PathBuf,
    format: &str,
    config: &TrellisConfig,
) -> Result<(), AppError> {
    fs::create_dir_all(output_dir).map_err(|e| {
        AppError::Render(format!("cannot create output dir {:?}: {}", output_dir, e))
    })?;

    let output_format = parse_format(format)?;
    let ext = format;

    // Collect all .mmd files in the input directory tree
    let files: Vec<PathBuf> = walkdir::WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension() == Some(OsStr::new("mmd")))
        .map(|e| e.path().to_path_buf())
        .collect();

    if files.is_empty() {
        eprintln!("No .mmd files found in {:?}", input_dir);
        return Ok(());
    }

    println!("Rendering {} file(s)…", files.len());

    // Parallel rendering with rayon
    let errors: Vec<String> = files
        .par_iter()
        .filter_map(|file| {
            let stem = file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("diagram");
            let out_file = output_dir.join(format!("{}.{}", stem, ext));

            let result: Result<(), String> = (|| {
                let content = fs::read_to_string(file)
                    .map_err(|e| format!("cannot read {:?}: {}", file, e))?;
                let graph = trellis_parser::parse(&content)
                    .map_err(|e| format!("parse error in {:?}: {}", file, e))?;
                let rendered = trellis_core::render(&graph, config, output_format)
                    .map_err(|e| format!("render error in {:?}: {}", file, e))?;
                fs::write(&out_file, &rendered.data)
                    .map_err(|e| format!("cannot write {:?}: {}", out_file, e))?;
                println!("  {:?} -> {:?}", file, out_file);
                Ok(())
            })();

            result.err()
        })
        .collect();

    if errors.is_empty() {
        println!("Done – {} file(s) rendered.", files.len());
        Ok(())
    } else {
        for e in &errors {
            eprintln!("Error: {}", e);
        }
        Err(AppError::Render(format!(
            "{} file(s) failed to render",
            errors.len()
        )))
    }
}

fn cmd_validate(input: &PathBuf) -> Result<(), AppError> {
    let content = read_input(input)?;

    let graph = trellis_parser::parse(&content).map_err(|e| AppError::Parse(e.to_string()))?;

    let subgraph_count = count_subgraphs(&graph.subgraphs);
    if subgraph_count > 0 {
        println!(
            "OK – {} nodes, {} edges, {} subgraphs",
            graph.nodes.len(),
            graph.edges.len(),
            subgraph_count,
        );
    } else {
        println!(
            "OK – {} nodes, {} edges",
            graph.nodes.len(),
            graph.edges.len(),
        );
    }

    Ok(())
}

/// Preprocess a Markdown file: find fenced ```mermaid … ``` blocks, render each
/// to `image_dir`, and replace the block with a Markdown image reference.
fn cmd_preprocess(
    input: &PathBuf,
    output: &PathBuf,
    image_dir: &Path,
    format: &str,
    config: &TrellisConfig,
) -> Result<(), AppError> {
    let content = read_input(input)?;
    let output_format = parse_format(format)?;
    let ext = format;

    fs::create_dir_all(image_dir)
        .map_err(|e| AppError::Render(format!("cannot create image dir {:?}: {}", image_dir, e)))?;

    let mut out_lines: Vec<String> = Vec::new();
    let mut diagram_index: u32 = 0;
    let mut in_mermaid = false;
    let mut fence_char = '`';
    let mut mermaid_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        if !in_mermaid {
            // Detect opening fence  ```mermaid  or  ~~~mermaid
            let trimmed = line.trim_start();
            if (trimmed.starts_with("```mermaid") || trimmed.starts_with("~~~mermaid"))
                && (trimmed.len() == "```mermaid".len()
                    || trimmed
                        .chars()
                        .nth("```mermaid".len())
                        .is_none_or(|c| c.is_whitespace()))
            {
                fence_char = trimmed.chars().next().unwrap_or('`');
                in_mermaid = true;
                mermaid_lines.clear();
            } else {
                out_lines.push(line.to_string());
            }
        } else {
            // Detect closing fence (same character, ≥3 of them, optional trailing space)
            let trimmed = line.trim_start();
            let is_close = trimmed.starts_with(&fence_char.to_string().repeat(3))
                && trimmed.trim_end_matches(fence_char).trim().is_empty();

            if is_close {
                in_mermaid = false;
                diagram_index += 1;
                let img_name = format!("diagram_{}.{}", diagram_index, ext);
                let img_path = image_dir.join(&img_name);

                // Build the mermaid source
                let source = mermaid_lines.join("\n");

                match trellis_parser::parse(&source) {
                    Err(e) => {
                        eprintln!(
                            "Warning: failed to parse diagram {} – skipping: {}",
                            diagram_index, e
                        );
                        // Emit original block unchanged
                        out_lines.push(format!("{}mermaid", fence_char.to_string().repeat(3)));
                        out_lines.append(&mut mermaid_lines);
                        out_lines.push(fence_char.to_string().repeat(3));
                    }
                    Ok(graph) => match trellis_core::render(&graph, config, output_format) {
                        Err(e) => {
                            eprintln!(
                                "Warning: failed to render diagram {} – skipping: {}",
                                diagram_index, e
                            );
                            out_lines.push(format!("{}mermaid", fence_char.to_string().repeat(3)));
                            out_lines.append(&mut mermaid_lines);
                            out_lines.push(fence_char.to_string().repeat(3));
                        }
                        Ok(rendered) => {
                            fs::write(&img_path, &rendered.data).map_err(|e| {
                                AppError::Render(format!(
                                    "cannot write image {:?}: {}",
                                    img_path, e
                                ))
                            })?;
                            // Emit a Markdown image reference using the image_dir path
                            let ref_path = image_dir.join(&img_name);
                            out_lines.push(format!(
                                "![diagram {}]({})",
                                diagram_index,
                                ref_path.display()
                            ));
                            mermaid_lines.clear();
                        }
                    },
                }
            } else {
                mermaid_lines.push(line.to_string());
            }
        }
    }

    // If the file ended without a closing fence, emit the unclosed block as-is
    if in_mermaid {
        out_lines.push(format!("{}mermaid", fence_char.to_string().repeat(3)));
        out_lines.extend(mermaid_lines);
    }

    let output_content = out_lines.join("\n");
    write_output(output, output_content.as_bytes())?;

    println!(
        "Preprocessed {:?} → {:?}  ({} diagram(s) rendered)",
        input, output, diagram_index
    );

    Ok(())
}

// ─── evaluate commands ────────────────────────────────────────────────────────

/// Evaluate a single diagram: render + quality report.
/// Writes `{stem}.svg` and `{stem}.json` to `output_dir`.
/// Optionally writes `{stem}.annotated.svg`.
fn cmd_evaluate(
    input: &PathBuf,
    output_dir: &PathBuf,
    annotated: bool,
    config: &TrellisConfig,
) -> Result<(), AppError> {
    fs::create_dir_all(output_dir).map_err(|e| {
        AppError::Render(format!("cannot create output dir {:?}: {}", output_dir, e))
    })?;

    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("diagram");
    let fixture_name = input
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("diagram.mmd");

    let content = read_input(input)?;
    let graph = trellis_parser::parse(&content).map_err(|e| AppError::Parse(e.to_string()))?;

    let (render_result, validation) =
        trellis_core::render_with_validation(&graph, config, trellis_core::OutputFormat::Svg)
            .map_err(|e| AppError::Render(e.to_string()))?;

    // Write SVG
    let svg_path = output_dir.join(format!("{}.svg", stem));
    fs::write(&svg_path, &render_result.data)
        .map_err(|e| AppError::Render(format!("cannot write {:?}: {}", svg_path, e)))?;

    // Build and write JSON report
    let report = trellis_validate::report::generate_report(
        fixture_name,
        &validation.graph,
        &validation.routing_result,
        &validation.port_assignments,
    );
    let json = serde_json::to_string_pretty(&report)
        .map_err(|e| AppError::Render(format!("JSON serialisation failed: {}", e)))?;
    let json_path = output_dir.join(format!("{}.json", stem));
    fs::write(&json_path, json.as_bytes())
        .map_err(|e| AppError::Render(format!("cannot write {:?}: {}", json_path, e)))?;

    // Optionally write annotated SVG
    if annotated {
        let ann = trellis_validate::annotated_svg::annotate_svg(
            &validation.svg,
            &report,
            &validation.grid,
        );
        let ann_path = output_dir.join(format!("{}.annotated.svg", stem));
        fs::write(&ann_path, &ann)
            .map_err(|e| AppError::Render(format!("cannot write {:?}: {}", ann_path, e)))?;
        println!("  annotated SVG → {:?}", ann_path);
    }

    println!("  {:?} → {:?}", input, svg_path);
    println!("  report     → {:?}", json_path);
    println!(
        "  {} edges | avg quality {:.2} | {} flagged",
        report.global_metrics.routed_edges,
        report.global_metrics.avg_quality_score,
        report.global_metrics.flagged_edges,
    );

    Ok(())
}

/// All port-assignment strategies available for `--compare-strategies`.
const COMPARE_STRATEGIES: &[(PortAssignmentStrategy, &str)] = &[
    (PortAssignmentStrategy::Default, "default"),
    (PortAssignmentStrategy::Barycenter, "barycenter"),
    (PortAssignmentStrategy::Median, "median"),
    (PortAssignmentStrategy::CrossingGreedy, "crossing_greedy"),
];

fn cmd_evaluate_batch(
    input_dir: &PathBuf,
    output_dir: &PathBuf,
    annotated: bool,
    compare_strategies: bool,
    config: &TrellisConfig,
) -> Result<(), AppError> {
    fs::create_dir_all(output_dir).map_err(|e| {
        AppError::Render(format!("cannot create output dir {:?}: {}", output_dir, e))
    })?;

    let files: Vec<PathBuf> = walkdir::WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension() == Some(OsStr::new("mmd")))
        .map(|e| e.path().to_path_buf())
        .collect();

    if files.is_empty() {
        eprintln!("No .mmd files found in {:?}", input_dir);
        return Ok(());
    }

    println!(
        "Evaluating {} file(s){}…",
        files.len(),
        if compare_strategies {
            " × 4 strategies"
        } else {
            ""
        }
    );

    let errors: Vec<String> = files
        .par_iter()
        .flat_map(|file| {
            let mut errs = Vec::new();

            let stem = file
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("diagram");
            let fixture_name = file
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("diagram.mmd");

            let content = match fs::read_to_string(file) {
                Ok(c) => c,
                Err(e) => {
                    errs.push(format!("cannot read {:?}: {}", file, e));
                    return errs;
                }
            };
            let graph = match trellis_parser::parse(&content) {
                Ok(g) => g,
                Err(e) => {
                    errs.push(format!("parse error in {:?}: {}", file, e));
                    return errs;
                }
            };

            if compare_strategies {
                // Run every strategy and collect per-strategy results for the CSV
                let mut csv_rows: Vec<String> = Vec::new();

                for &(strategy, label) in COMPARE_STRATEGIES {
                    let mut cfg = config.clone();
                    cfg.port_assignment = strategy;

                    match trellis_core::render_with_validation(
                        &graph,
                        &cfg,
                        trellis_core::OutputFormat::Svg,
                    ) {
                        Err(e) => errs.push(format!("render error {:?} ({}): {}", file, label, e)),
                        Ok((result, validation)) => {
                            let report = trellis_validate::report::generate_report(
                                fixture_name,
                                &validation.graph,
                                &validation.routing_result,
                                &validation.port_assignments,
                            );

                            // Write {stem}_{strategy}.svg
                            let svg_path = output_dir.join(format!("{}_{}.svg", stem, label));
                            if let Err(e) = fs::write(&svg_path, &result.data) {
                                errs.push(format!("cannot write {:?}: {}", svg_path, e));
                            }

                            // Write {stem}_{strategy}.json
                            if let Ok(json) = serde_json::to_string_pretty(&report) {
                                let json_path = output_dir.join(format!("{}_{}.json", stem, label));
                                if let Err(e) = fs::write(&json_path, json.as_bytes()) {
                                    errs.push(format!("cannot write {:?}: {}", json_path, e));
                                }
                            }

                            // Write annotated SVG if requested
                            if annotated {
                                let ann = trellis_validate::annotated_svg::annotate_svg(
                                    &validation.svg,
                                    &report,
                                    &validation.grid,
                                );
                                let ann_path =
                                    output_dir.join(format!("{}_{}.annotated.svg", stem, label));
                                if let Err(e) = fs::write(&ann_path, &ann) {
                                    errs.push(format!("cannot write {:?}: {}", ann_path, e));
                                }
                            }

                            csv_rows.push(format!(
                                "{},{},{:.4},{},{},{}",
                                stem,
                                label,
                                report.global_metrics.avg_quality_score,
                                report.global_metrics.total_crossings,
                                report.global_metrics.total_bends,
                                report.global_metrics.flagged_edges,
                            ));
                        }
                    }
                }

                // Write summary CSV
                if !csv_rows.is_empty() {
                    let header = "fixture,strategy,avg_quality,crossings,bends,flagged_edges";
                    let csv = format!("{}\n{}\n", header, csv_rows.join("\n"));
                    let csv_path = output_dir.join(format!("{}_strategies.csv", stem));
                    if let Err(e) = fs::write(&csv_path, csv.as_bytes()) {
                        errs.push(format!("cannot write {:?}: {}", csv_path, e));
                    } else {
                        println!("  {:?} comparison → {:?}", file, csv_path);
                    }
                }
            } else {
                // Single-strategy mode
                match trellis_core::render_with_validation(
                    &graph,
                    config,
                    trellis_core::OutputFormat::Svg,
                ) {
                    Err(e) => errs.push(format!("render error in {:?}: {}", file, e)),
                    Ok((result, validation)) => {
                        let report = trellis_validate::report::generate_report(
                            fixture_name,
                            &validation.graph,
                            &validation.routing_result,
                            &validation.port_assignments,
                        );

                        let svg_path = output_dir.join(format!("{}.svg", stem));
                        if let Err(e) = fs::write(&svg_path, &result.data) {
                            errs.push(format!("cannot write {:?}: {}", svg_path, e));
                            return errs;
                        }

                        if let Ok(json) = serde_json::to_string_pretty(&report) {
                            let json_path = output_dir.join(format!("{}.json", stem));
                            if let Err(e) = fs::write(&json_path, json.as_bytes()) {
                                errs.push(format!("cannot write {:?}: {}", json_path, e));
                                return errs;
                            }
                        }

                        if annotated {
                            let ann = trellis_validate::annotated_svg::annotate_svg(
                                &validation.svg,
                                &report,
                                &validation.grid,
                            );
                            let ann_path = output_dir.join(format!("{}.annotated.svg", stem));
                            if let Err(e) = fs::write(&ann_path, &ann) {
                                errs.push(format!("cannot write {:?}: {}", ann_path, e));
                            }
                        }

                        println!(
                            "  {:?} → {}.svg + {}.json (quality {:.2})",
                            file, stem, stem, report.global_metrics.avg_quality_score
                        );
                    }
                }
            }

            errs
        })
        .collect();

    if errors.is_empty() {
        println!("Done – {} file(s) evaluated.", files.len());
        Ok(())
    } else {
        for e in &errors {
            eprintln!("Error: {}", e);
        }
        Err(AppError::Render(format!("{} file(s) failed", errors.len())))
    }
}

// ─── generate-review ──────────────────────────────────────────────────────────

/// Load every `*.json` report from `input_dir`, pair with the matching
/// `*.annotated.svg` (falling back to `*.svg`), then write a self-contained
/// `review.html` that the reviewer opens in a browser.
fn cmd_generate_review(input_dir: &PathBuf, output: Option<&PathBuf>) -> Result<(), AppError> {
    // Collect all JSON report files
    let json_files: Vec<PathBuf> = walkdir::WalkDir::new(input_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension() == Some(OsStr::new("json")))
        .map(|e| e.path().to_path_buf())
        .collect();

    if json_files.is_empty() {
        eprintln!("No .json report files found in {:?}", input_dir);
        return Ok(());
    }

    println!("Loading {} report(s)…", json_files.len());

    // Load each report + matching SVG
    struct Entry {
        fixture_name: String,
        /// Strategy label derived from the JSON filename, e.g. "barycenter"
        /// for `b01_barycenter.json` when the fixture is `b01.mmd`.
        /// `None` when the JSON filename matches the fixture name exactly.
        strategy_label: Option<String>,
        svg: String,
        report: trellis_validate::report::DiagramReport,
    }

    let mut entries: Vec<Entry> = Vec::new();

    for json_path in &json_files {
        let stem = json_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("diagram");

        // Parse report
        let json_content = fs::read_to_string(json_path)
            .map_err(|e| AppError::Render(format!("cannot read {:?}: {}", json_path, e)))?;
        let report: trellis_validate::report::DiagramReport =
            serde_json::from_str(&json_content)
                .map_err(|e| AppError::Render(format!("cannot parse {:?}: {}", json_path, e)))?;

        // Detect strategy label: if the JSON stem is `{fixture_stem}_{strategy}`,
        // extract the suffix. e.g. "b01_barycenter" with fixture "b01.mmd" → "barycenter".
        let fixture_stem = report
            .fixture
            .trim_end_matches(".mmd")
            .trim_end_matches(".mermaid");
        let strategy_label = if stem != fixture_stem {
            let prefix = format!("{}_", fixture_stem);
            if stem.starts_with(&prefix) {
                Some(stem[prefix.len()..].to_string())
            } else {
                None
            }
        } else {
            None
        };

        // Prefer annotated SVG; fall back to plain SVG
        let svg_path = {
            let ann = input_dir.join(format!("{}.annotated.svg", stem));
            if ann.exists() {
                ann
            } else {
                input_dir.join(format!("{}.svg", stem))
            }
        };

        let svg = if svg_path.exists() {
            fs::read_to_string(&svg_path)
                .map_err(|e| AppError::Render(format!("cannot read {:?}: {}", svg_path, e)))?
        } else {
            eprintln!(
                "  Warning: no SVG found for {:?} (expected {:?})",
                json_path, svg_path
            );
            format!(
                "<svg xmlns='http://www.w3.org/2000/svg' width='200' height='60'>\
                 <text x='10' y='30' font-family='sans-serif' font-size='12' fill='#888'>\
                 SVG not found for {}</text></svg>",
                stem
            )
        };

        let fixture_name = report.fixture.clone();
        entries.push(Entry {
            fixture_name,
            strategy_label,
            svg,
            report,
        });
    }

    // Sort: primary key = fixture_name, secondary = strategy_label (None first)
    entries.sort_by(|a, b| {
        a.fixture_name
            .cmp(&b.fixture_name)
            .then_with(|| a.strategy_label.cmp(&b.strategy_label))
    });

    // Build review entries
    let review_entries: Vec<trellis_validate::review_html::ReviewEntry<'_>> = entries
        .iter()
        .map(|e| trellis_validate::review_html::ReviewEntry {
            fixture_name: &e.fixture_name,
            annotated_svg: &e.svg,
            report: &e.report,
            strategy_label: e.strategy_label.as_deref(),
        })
        .collect();

    let html = trellis_validate::review_html::generate_review_html(&review_entries);

    let out_path = output
        .cloned()
        .unwrap_or_else(|| input_dir.join("review.html"));

    fs::write(&out_path, &html)
        .map_err(|e| AppError::Render(format!("cannot write {:?}: {}", out_path, e)))?;

    println!(
        "Review tool written to {:?}  ({} fixture(s))",
        out_path,
        entries.len()
    );
    Ok(())
}

// ─── help ─────────────────────────────────────────────────────────────────────

fn cmd_help() {
    let version = env!("CARGO_PKG_VERSION");
    println!(
        "\
trellis {version} – A fast Mermaid diagram renderer

USAGE
  trellis [--config <FILE>] <COMMAND> [OPTIONS]

GLOBAL OPTIONS
  --config <FILE>   Load configuration from FILE
                    (default: ~/.trellis/config.toml, silently ignored if absent)
  -h, --help        Show brief help for the given command
  -V, --version     Print version

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

COMMAND: render
  Render a single Mermaid diagram to SVG or PNG.

  USAGE
    trellis render [OPTIONS] <INPUT>

  ARGUMENTS
    <INPUT>              Path to a .mmd file, or '-' to read from stdin

  OPTIONS
    -o, --output <FILE>  Output file path, or '-' to write to stdout [default: -]
    -f, --format <FMT>   Output format: svg | png                    [default: svg]
        --metrics        Print rendering metrics as JSON to stderr

  EXAMPLES
    trellis render diagram.mmd -o diagram.svg
    trellis render diagram.mmd -o diagram.png -f png
    trellis render diagram.mmd -o diagram.svg --metrics
    cat diagram.mmd | trellis render - -f svg > diagram.svg

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

COMMAND: render-batch
  Recursively find every .mmd file in a directory and render each
  one in parallel, writing results to an output directory.

  USAGE
    trellis render-batch [OPTIONS] <INPUT_DIR>

  ARGUMENTS
    <INPUT_DIR>              Source directory (searched recursively for .mmd files)

  OPTIONS
    -o, --output-dir <DIR>   Output directory (created if absent)   [required]
    -f, --format <FMT>       Output format: svg | png               [default: svg]

  EXAMPLES
    trellis render-batch diagrams/ -o out/
    trellis render-batch diagrams/ -o out/ -f png

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

COMMAND: validate
  Parse a Mermaid diagram and print a node / edge / subgraph
  summary.  Exits with code 1 if the diagram cannot be parsed.

  USAGE
    trellis validate <INPUT>

  ARGUMENTS
    <INPUT>   Path to a .mmd file, or '-' to read from stdin

  EXAMPLES
    trellis validate diagram.mmd
    echo $?   # 0 = valid, 1 = parse error

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

COMMAND: evaluate
  Render a diagram and produce a per-edge quality report.
  Writes <stem>.svg and <stem>.json to the output directory.
  Quality scores: 1.0 = perfect, 0.0 = worst.
  Flags: high_detour (>=2x Manhattan), excessive_bends (>=4), avoidable_crossing.

  USAGE
    trellis evaluate [OPTIONS] <INPUT>

  ARGUMENTS
    <INPUT>                  Input .mmd file

  OPTIONS
    -o, --output-dir <DIR>   Output directory (created if absent)    [required]
        --annotated          Also write <stem>.annotated.svg with
                             colour-coded quality overlay

  EXAMPLES
    trellis evaluate diagram.mmd -o ./reports/
    trellis evaluate diagram.mmd -o ./reports/ --annotated

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

COMMAND: evaluate-batch
  Evaluate all .mmd files in a directory, writing .svg + .json per fixture.
  Run with --compare-strategies to produce per-strategy outputs and a
  summary CSV suitable for A/B algorithm comparison.

  USAGE
    trellis [--config <FILE>] evaluate-batch [OPTIONS] <INPUT_DIR>

  ARGUMENTS
    <INPUT_DIR>              Source directory (searched recursively for .mmd files)

  OPTIONS
    -o, --output-dir <DIR>   Output directory (created if absent)    [required]
        --annotated          Also write annotated SVGs
        --compare-strategies Run Default/Barycenter/Median/CrossingGreedy and
                             write <stem>_<strategy>.svg + .json + a
                             <stem>_strategies.csv summary

  GLOBAL OPTIONS (place before the subcommand)
    --config <FILE>          Load config from FILE instead of ~/.trellis/config.toml

  EXAMPLES
    trellis evaluate-batch ./fixtures/ -o ./reports/
    trellis evaluate-batch ./fixtures/ -o ./reports/ --annotated
    trellis evaluate-batch ./fixtures/ -o ./reports/ --compare-strategies
    trellis --config custom.toml evaluate-batch ./fixtures/ -o ./reports/

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

COMMAND: generate-review
  Generate a self-contained HTML review tool from a reports directory.
  Load .json reports produced by `evaluate` or `evaluate-batch`, pair
  each with the matching .annotated.svg (or plain .svg as fallback), and
  write a single review.html file.

  Open the HTML in a browser to:
    - Browse all fixtures side-by-side.
    - Click a coloured edge overlay to open the review panel.
    - Toggle 'improvable' and write a free-text routing note.
    - Download feedback.json for the AI tuning agent.

  USAGE
    trellis generate-review [OPTIONS] <INPUT_DIR>

  ARGUMENTS
    <INPUT_DIR>              Directory containing .json + .svg files

  OPTIONS
    -o, --output <FILE>      Output HTML file
                             [default: <input_dir>/review.html]

  EXAMPLES
    trellis evaluate-batch ./fixtures/ -o ./reports/ --annotated
    trellis generate-review ./reports/
    trellis generate-review ./reports/ -o review.html

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

COMMAND: preprocess
  Scan a Markdown file for fenced ```mermaid ... ``` blocks, render
  each to an image file, and replace the block with a Markdown
  image reference (![diagram N](img/diagram_N.svg)).

  USAGE
    trellis preprocess [OPTIONS] <INPUT>

  ARGUMENTS
    <INPUT>                  Input Markdown file

  OPTIONS
    -o, --output <FILE>      Output Markdown file                    [required]
        --image-dir <DIR>    Directory for rendered images           [default: img]
    -f, --format <FMT>       Image format: svg | png                 [default: svg]

  EXAMPLES
    trellis preprocess doc.md -o out.md
    trellis preprocess doc.md -o out.md --image-dir assets/imgs -f png

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

CONFIG FILE
  ~/.trellis/config.toml  (loaded automatically; partial files are fine)

  All fields are optional -- missing ones fall back to built-in defaults.
  Override at runtime with: trellis --config /path/to/config.toml <COMMAND>

  EXAMPLE config.toml
    cell_size        = 10      # grid cell size in pixels
    corner_radius    = 8.0     # rounded edge corners
    show_edge_labels = true    # show edge captions
    crossing_style = \"Arc\"    # crossing decoration: None | Arc | Rectangular | Skip

    [routing_costs]
    bend_cost     = 2.0
    crossing_cost = 10.0

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

EXIT CODES
  0  Success
  1  Parse error  (invalid diagram syntax)
  2  Render error (rendering or I/O failure)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

PANDOC INTEGRATION
  Use the bundled Lua filter to render Mermaid blocks when running Pandoc:

    pandoc doc.md --lua-filter filters/trellis-filter.lua -o doc.html
    pandoc doc.md --lua-filter filters/trellis-filter.lua -o doc.pdf

  For HTML output the SVG is embedded inline.
  For all other formats a temporary PNG file is used.
"
    );
}

// ─── entry point ─────────────────────────────────────────────────────────────

fn run() -> i32 {
    let cli = Cli::parse();
    let mut config = load_config(cli.config.as_ref());

    // --theme flag overrides any theme set in config file
    if let Some(ref theme_str) = cli.theme {
        let parsed = match theme_str.as_str() {
            "default" => Some(ThemeName::Default),
            "paper" => Some(ThemeName::Paper),
            "blueprint" => Some(ThemeName::Blueprint),
            "dark" => Some(ThemeName::Dark),
            "midnight" => Some(ThemeName::Midnight),
            "forest" => Some(ThemeName::Forest),
            other => {
                eprintln!(
                    "Warning: unknown theme '{}' — using default. \
                     Valid themes: default, paper, blueprint, dark, midnight, forest",
                    other
                );
                None
            }
        };
        if let Some(t) = parsed {
            config.theme = t;
        }
    }

    let result = match &cli.command {
        Commands::Render {
            input,
            output,
            format,
            metrics,
            debug_log,
        } => cmd_render(input, output, format, *metrics, debug_log, &config),

        Commands::RenderBatch {
            input_dir,
            output_dir,
            format,
        } => cmd_render_batch(input_dir, output_dir, format, &config),

        Commands::Validate { input } => cmd_validate(input),

        Commands::Evaluate {
            input,
            output_dir,
            annotated,
        } => cmd_evaluate(input, output_dir, *annotated, &config),

        Commands::EvaluateBatch {
            input_dir,
            output_dir,
            annotated,
            compare_strategies,
        } => cmd_evaluate_batch(
            input_dir,
            output_dir,
            *annotated,
            *compare_strategies,
            &config,
        ),

        Commands::Preprocess {
            input,
            output,
            image_dir,
            format,
        } => cmd_preprocess(input, output, image_dir, format, &config),

        Commands::GenerateReview { input_dir, output } => {
            cmd_generate_review(input_dir, output.as_ref())
        }

        Commands::Help => {
            cmd_help();
            return 0;
        }
    };

    match result {
        Ok(()) => 0,
        Err(e) => {
            let code = e.exit_code();
            eprintln!("{}", e);
            code
        }
    }
}

fn main() {
    std::process::exit(run());
}
