# Trellis

[![CI](https://github.com/trellis-mermaid/trellis/actions/workflows/ci.yml/badge.svg)](https://github.com/trellis-mermaid/trellis/actions/workflows/ci.yml)
[![Release](https://github.com/trellis-mermaid/trellis/actions/workflows/release.yml/badge.svg)](https://github.com/trellis-mermaid/trellis/releases)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)

A fast Mermaid diagram renderer built in Rust, using VLSI maze routing (A* pathfinding) to produce overlap-free orthogonal edge layouts.

Supports: **flowchart**, **class**, **ER**, and **C4** diagram types.

## Installation

### Pre-built binaries

Download the latest release from the [Releases page](https://github.com/trellis-mermaid/trellis/releases):

| Platform | Archive |
|---|---|
| Linux x86_64 | `trellis-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz` |
| macOS x86_64 | `trellis-vX.Y.Z-x86_64-apple-darwin.tar.gz` |
| macOS ARM (Apple Silicon) | `trellis-vX.Y.Z-aarch64-apple-darwin.tar.gz` |
| Windows x86_64 | `trellis-vX.Y.Z-x86_64-pc-windows-gnu.zip` |

Extract the archive and add `trellis` to your PATH.

### Build from source

```bash
cargo install --path crates/trellis-cli
```

### Docker

```bash
# Pull the pre-built image
docker pull ghcr.io/trellis-mermaid/trellis:latest

# Render a diagram
docker run --rm -v "$(pwd):/data" ghcr.io/trellis-mermaid/trellis:latest input.mmd -o output.svg
```

## Usage

### Render a diagram

```bash
trellis render input.mmd -o output.svg
trellis render input.mmd -o output.png
```

### Render from stdin

```bash
echo "flowchart LR
  A --> B --> C" | trellis render - -o diagram.svg
```

### Batch render a directory

```bash
trellis render-batch ./diagrams -o ./output
```

### Validate (parse only, no render)

```bash
trellis validate input.mmd
```

### Use with Pandoc (Markdown documents with Mermaid diagrams)

Render Markdown documents containing \`\`\`mermaid code blocks to PDF or HTML:

```bash
# Using Docker (recommended — no local dependencies)
docker run --rm -v "$(pwd):/data" trellis-pandoc document.md -o document.pdf
docker run --rm -v "$(pwd):/data" trellis-pandoc document.md -o document.html

# Using the Lua filter directly (requires Pandoc + trellis in PATH)
pandoc --lua-filter=filters/trellis-filter.lua document.md -o document.pdf
```

Build the Pandoc Docker image locally:

```bash
./scripts/build-pandoc-docker.sh
```

### Diagram quality validation

The `trellis-validate` crate analyses routed diagrams and produces per-edge
quality reports.  The `evaluate` and `evaluate-batch` CLI commands (step 6)
are the primary interface; the library API is described below for embedding.

```bash
# Render and evaluate a single diagram (produces diagram.svg + diagram.json)
trellis evaluate diagram.mmd -o ./reports/

# Evaluate every fixture in a directory
trellis evaluate-batch ./fixtures/ -o ./reports/
```

The library exposes three functions for programmatic use:

```rust
// 1. Score every routed edge (detour factor, bends, crossings → 0–1 score + flags)
trellis_validate::scoring::score_all_edges(&graph, &routing_result, &port_assignments, &grid)

// 2. Build the full JSON report (stable AI-agent contract)
trellis_validate::report::generate_report("diagram.mmd", &graph, &routing_result, &port_assignments, &grid)

// 3. Inject a colour-coded quality overlay into a rendered SVG
trellis_validate::annotated_svg::annotate_svg(&svg_bytes, &report, &grid)
```

**Quality score** (0.0 – 1.0, higher is better):

| Score   | Overlay colour | Meaning |
|---------|---------------|---------|
| ≥ 0.80  | Green          | Edge routes cleanly |
| 0.50 – 0.79 | Yellow    | Suboptimal — detour or extra bends |
| < 0.50  | Red            | Poor — high detour, many bends, or crossing |

**Flags** on individual edges:

| Flag | Condition |
|---|---|
| `high_detour` | Routed path ≥ 2× the Manhattan distance |
| `excessive_bends` | ≥ 4 direction changes |
| `avoidable_crossing` | Edge passes through a grid cell shared with another edge |

**JSON report schema** (`{name}.json`):

```json
{
  "fixture": "diagram.mmd",
  "edges": [
    {
      "id": "A-->B",
      "source": "A",
      "target": "B",
      "bends": 2,
      "detour_factor": 1.4,
      "crossings": 0,
      "port_side_source": "South",
      "port_side_target": "North",
      "path_cells": [[3,4],[4,4],[5,4]],
      "quality_score": 0.87,
      "flags": []
    }
  ],
  "global_metrics": {
    "total_edges": 5,
    "routed_edges": 5,
    "failed_edges": 0,
    "total_crossings": 0,
    "total_bends": 6,
    "avg_quality_score": 0.84,
    "avg_detour_factor": 1.3,
    "flagged_edges": 0
  }
}
```

The `evaluate` and `evaluate-batch` CLI commands that produce `{name}.svg` +
`{name}.json` pairs are wired up to this library in the next step.

### Configuration

Use a TOML config file for custom settings:

```bash
trellis render input.mmd -o output.svg --config trellis.toml
```

## Project Structure

```
trellis/
├── crates/
│   ├── trellis-parser/    # Mermaid syntax parser (nom-based)
│   ├── trellis-core/      # Core rendering pipeline (placement → grid → routing → SVG)
│   ├── trellis-wasm/      # WASM bindings (wasm-bindgen)
│   ├── trellis-cli/       # Command-line interface
│   └── trellis-validate/  # Edge quality scoring, JSON reports, annotated SVG
├── docker/
│   ├── Dockerfile.wasm-builder      # Builds WASM (Rust + wasm-pack)
│   ├── Dockerfile.vscode-builder    # Builds the VS Code .vsix (Node.js + vsce)
│   ├── Dockerfile.intellij-builder  # Builds the IntelliJ .zip (JDK 21 + Gradle)
│   ├── Dockerfile.cli-builder       # Cross-compiles CLI (Linux + Windows)
│   └── Dockerfile.pandoc            # Full Pandoc + Trellis for document rendering
├── editors/
│   ├── vscode/            # VS Code extension
│   └── intellij/          # IntelliJ platform plugin
├── scripts/
│   ├── build-wasm.sh / build-wasm-docker.sh
│   ├── build-vscode-docker.sh
│   ├── build-intellij-docker.sh
│   ├── build-cli.sh
│   └── build-pandoc-docker.sh
├── tests/
│   └── benchmarks/
│       └── fixtures/      # Benchmark test files (b01-b12.mmd)
└── docs/                  # Specifications and implementation plan
```

## Development

### Build

```bash
cargo build --workspace
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run benchmarks
cargo bench -p trellis-benchmarks

# Static analysis
cargo clippy --workspace
```

### Building WASM (required by both editor extensions)

```bash
# Docker (recommended — no Rust/wasm-pack needed locally)
./scripts/build-wasm-docker.sh           # dev build
./scripts/build-wasm-docker.sh --release # optimised release build

# Native (requires Rust + wasm-pack installed)
./scripts/build-wasm.sh
./scripts/build-wasm.sh --release
```

### Building CLI binaries

```bash
# Docker: cross-compile Linux + Windows
./scripts/build-cli.sh

# Native: host platform only
./scripts/build-cli.sh --native
```

## Editor Extensions

### VS Code Extension

```bash
# Build with Docker
./scripts/build-wasm-docker.sh && ./scripts/build-vscode-docker.sh

# Install
code --install-extension dist/trellis-preview-0.1.0.vsix
```

**Development:** Open the repo in VS Code, press **F5** to launch the Extension Development Host, open a `.mmd` file, and press **Ctrl+Shift+V** to preview.

### IntelliJ Plugin

```bash
# Build with Docker
./scripts/build-wasm-docker.sh && ./scripts/build-intellij-docker.sh

# Install: Settings → Plugins → ⚙ → Install Plugin from Disk…
# Select dist/trellis-intellij-0.1.0.zip
```

**Development:**

```bash
cd editors/intellij
./gradlew runIde
```

**Settings** (Settings → Tools → Trellis):

| Setting | Default | Description |
|---|---|---|
| Auto Preview | `true` | Re-render on every keystroke / file save |
| Default Direction | `TB` | Flowchart layout direction (`TB`, `LR`, `BT`, `RL`) |
| Default Theme | `default` | Colour theme (`default`, `dark`, `neutral`) |

## CI/CD

The project uses GitHub Actions for continuous integration and release automation:

- **CI** (`ci.yml`): Runs on every push to `main`/`develop` and on PRs. Tests on Linux, macOS, and Windows. Builds WASM, CLI binaries (4 platforms), VS Code extension, and IntelliJ plugin.
- **Release** (`release.yml`): Triggered by pushing a version tag (`v*`). Creates a draft GitHub Release with pre-built CLI binaries, editor extensions, and pushes the Docker image to GHCR.

### Creating a release

```bash
git tag v0.1.0
git push origin v0.1.0
# → GitHub Actions builds all artifacts and creates a draft release
```

## Milestones

| Milestone | Status | Description |
|---|---|---|
| M1 | ✅ | Project skeleton |
| M2 | ✅ | Flowchart parser |
| M3 | ✅ | Sugiyama placement |
| M4 | ✅ | Grid + ports |
| M5 | ✅ | A* edge routing |
| M6 | ✅ | SVG output |
| M7 | ✅ | Deadlock handling (rip-up & reroute) |
| M8 | ✅ | Edge labels |
| M9 | ✅ | Subgraph visualisation |
| M10 | ✅ | Class diagram |
| M11 | ✅ | ER diagram |
| M12 | ✅ | C4 diagram types |
| M13 | ✅ | Benchmarks (B01–B12) |
| M14 | ✅ | CLI (render-batch, validate, stdin, Pandoc filter) |
| M15 | ✅ | WASM + VS Code extension |
| M16 | ✅ | IntelliJ plugin |
| M17 | ✅ | Docker + CI/CD |
| M18 | ⬜ | Decomposition (50+ node graphs) |
| M19 | ⬜ | Commercial features (licensing, freemium) |

See [docs/trellis-implementation-plan.md](docs/trellis-implementation-plan.md) for the full roadmap.

## License

MIT OR Apache-2.0
