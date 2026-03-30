use clap::{Parser, Subcommand};
use rayon::prelude::*;
use std::ffi::OsStr;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use trellis_core::TrellisConfig;

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
        other => Err(AppError::Render(format!("unsupported format '{}' (use svg or png)", other))),
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

fn cmd_render(
    input: &PathBuf,
    output: &PathBuf,
    format: &str,
    print_metrics: bool,
    config: &TrellisConfig,
) -> Result<(), AppError> {
    let content = read_input(input)?;

    let graph = trellis_parser::parse(&content)
        .map_err(|e| AppError::Parse(e.to_string()))?;

    let output_format = parse_format(format)?;

    let result = trellis_core::render(&graph, config, output_format)
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
    fs::create_dir_all(output_dir)
        .map_err(|e| AppError::Render(format!("cannot create output dir {:?}: {}", output_dir, e)))?;

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

    let graph = trellis_parser::parse(&content)
        .map_err(|e| AppError::Parse(e.to_string()))?;

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
                    || trimmed.chars().nth("```mermaid".len()).is_none_or(|c| c.is_whitespace()))
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
    render_crossings = false   # draw line-jump bridges

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
    let config = load_config(cli.config.as_ref());

    let result = match &cli.command {
        Commands::Render {
            input,
            output,
            format,
            metrics,
        } => cmd_render(input, output, format, *metrics, &config),

        Commands::RenderBatch {
            input_dir,
            output_dir,
            format,
        } => cmd_render_batch(input_dir, output_dir, format, &config),

        Commands::Validate { input } => cmd_validate(input),

        Commands::Preprocess {
            input,
            output,
            image_dir,
            format,
        } => cmd_preprocess(input, output, image_dir, format, &config),

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
