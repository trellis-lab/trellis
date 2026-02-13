use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "trellis")]
#[command(about = "A fast Mermaid diagram renderer", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Render a Mermaid diagram to SVG or PNG
    Render {
        /// Input file path (use '-' for stdin)
        input: PathBuf,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,

        /// Output format (svg or png)
        #[arg(short, long, default_value = "svg")]
        format: String,

        /// Show rendering metrics
        #[arg(long)]
        metrics: bool,
    },

    /// Validate a Mermaid diagram
    Validate {
        /// Input file path
        input: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Render {
            input,
            output,
            format,
            metrics,
        } => {
            // Read input
            let content = fs::read_to_string(&input)
                .with_context(|| format!("Failed to read input file: {:?}", input))?;

            // Parse
            let graph = trellis_parser::parse(&content)
                .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

            // Render
            let config = trellis_core::TrellisConfig::default();
            let output_format = match format.as_str() {
                "svg" => trellis_core::OutputFormat::Svg,
                "png" => trellis_core::OutputFormat::Png,
                _ => anyhow::bail!("Unsupported format: {}", format),
            };

            let result = trellis_core::render(&graph, &config, output_format)
                .map_err(|e| anyhow::anyhow!("Render error: {}", e))?;

            // Write output
            fs::write(&output, &result.data)
                .with_context(|| format!("Failed to write output file: {:?}", output))?;

            println!("Rendered to: {:?}", output);

            if metrics {
                let metrics_json = serde_json::to_string_pretty(&result.metrics)?;
                eprintln!("{}", metrics_json);
            }

            Ok(())
        }

        Commands::Validate { input } => {
            let content = fs::read_to_string(&input)
                .with_context(|| format!("Failed to read input file: {:?}", input))?;

            let graph = trellis_parser::parse(&content)
                .map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;

            println!(
                "OK - {} nodes, {} edges",
                graph.nodes.len(),
                graph.edges.len()
            );

            Ok(())
        }
    }
}
