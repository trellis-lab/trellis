---
layout: default
title: CLI Reference
nav_order: 3
---

# Trellis CLI Reference

The `trellis` binary renders Mermaid diagrams to SVG, PNG, HTML, ASCII, and Draw.io. It reads from a file or stdin, and writes to a file or stdout.

---

## Installation

### Binary download

| Platform | File |
|---|---|
| macOS (Apple Silicon) | `trellis-vX.Y.Z-aarch64-apple-darwin.tar.gz` |
| macOS (Intel) | `trellis-vX.Y.Z-x86_64-apple-darwin.tar.gz` |
| Linux x86_64 | `trellis-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz` |
| Windows x86_64 | `trellis-vX.Y.Z-x86_64-pc-windows-gnu.zip` |

Download from the [releases page](https://github.com/trellis-lab/trellis/releases), extract, and add `trellis` to your `PATH`.

### Docker

```bash
docker pull ghcr.io/trellis-lab/trellis:latest
```

---

## Quick start

```bash
trellis render diagram.mmd -o diagram.svg
```

Open `diagram.svg` in any browser or embed it in documentation.

---

## Command reference

### `render`

Render a single diagram.

```bash
trellis render <input> -o <output> [-f <format>] [--config <path>]
```

Pass `-` as `<input>` to read from stdin.

**Examples:**

```bash
# Render to SVG (default format)
trellis render input.mmd -o output.svg

# Render to PNG
trellis render input.mmd -o output.png

# Render to interactive HTML
trellis render input.mmd -o output.html -f html

# Render to ASCII (terminal preview)
trellis render input.mmd -o output.txt -f ascii

# Render to Draw.io (editable .mxfile)
trellis render input.mmd -o output.drawio -f drawio

# Render from stdin
echo "flowchart LR
  A --> B --> C" | trellis render - -o diagram.svg

# Render with custom config
trellis render input.mmd -o output.svg --config trellis.toml
```

### `render-batch`

Render all `.mmd` files in a directory.

> **Commercial feature.** `render-batch` requires a valid Trellis license. See [Licensing](licensing).

```bash
trellis render-batch <input-dir> -o <output-dir>
```

**Example:**

```bash
trellis render-batch ./diagrams -o ./output
```

All `.mmd` files in `./diagrams` are rendered to `./output` using the same filename with a `.svg` extension.

### `validate`

Parse a diagram and check for syntax errors — no render output produced.

```bash
trellis validate <input>
```

**Example:**

```bash
trellis validate input.mmd
```

Exits with code `0` on success, non-zero on error.

### `license`

Manage your Trellis license on this device. Required for commercial features such as `render-batch`.

```bash
# Activate this device (consumes one seat)
trellis license activate --license-key YOUR_KEY

# Show current status and device instance
trellis license status

# Deactivate this device (frees the seat)
trellis license deactivate
```

The key is resolved in priority order: `--license-key KEY` > `TRELLIS_KEY` in a `.env` file in the working directory > `TRELLIS_KEY` environment variable. See [Licensing](licensing) for details and the subscribe link.

---

## Flags

| Flag | Description |
|------|-------------|
| `-o <path>` | Output file path |
| `-f <format>` | Output format: `svg` (default), `png`, `html`, `ascii`, `drawio` |
| `--config <path>` | TOML config file |

---

## Output formats

Pick a format with `-f`. Each one targets a different use case:

| Format | Purpose |
|---|---|
| `svg` | Publication-ready vector — embed in web, Markdown, docs sites |
| `png` | Raster image for slide decks, presentations, tools without SVG support |
| `html` | Self-contained interactive viewer — explore large diagrams in a browser |
| `ascii` | Terminal preview and AI-agent workflows — lowest token cost |
| `drawio` | Editable `.mxfile` to hand-tweak in draw.io / Confluence |

### SVG

Scalable vector graphic. Embeds directly in HTML, Markdown, and documentation sites. Renders crisply at any size.

### PNG

Rasterized image. Use for presentations, slide decks, and tools that don't support SVG.

### HTML

Self-contained interactive file. No server or internet connection required. Open in any browser to:

- Click a node to highlight it, all connected edges, and its neighbours.
- Click an edge to highlight it and its two endpoint nodes.
- View an inspector panel with incoming/outgoing edge labels and neighbour names.
- Search nodes by name.
- Pan and zoom with mouse drag and scroll wheel; use the minimap to navigate large diagrams.
- Toggle between light and dark themes.
- Click the background or close button to deselect.

### ASCII

UTF-8 box-drawing diagram using Unicode characters (`─ │ ┌ ┐ └ ┘ ▶ ▼`). Node positions and edge paths match the SVG layout. Useful for terminal preview and AI-agent workflows — significantly lower token cost than SVG or HTML.

> ASCII output is in development and disabled by default in pre-built binaries.

### Draw.io

Editable `.mxfile` you can open directly in [draw.io](https://draw.io), Confluence, or the VS Code draw.io extension. Rendering is diagram-type-aware:

- **Flowchart** — nodes as draw.io shapes with orthogonal routed edges
- **Class diagram** — UML containers with typed relationship arrows
- **ER diagram** — table stencils with crow's-foot notation
- **C4 diagram** — C4-styled shapes, colours, and boundary containers

---

## Themes

| Theme | Description |
|-------|-------------|
| `default` | Default Trellis theme |
| `paper` | Light, print-friendly |
| `blueprint` | Technical blueprint style |
| `dark` | Dark background |
| `midnight` | Deep dark with accent colors |
| `forest` | Green-toned |

Set the theme via a config file:

```toml
# trellis.toml
theme = "paper"
```

---

## Configuration

Create a `trellis.toml` file and pass it with `--config`:

```bash
trellis render input.mmd -o output.svg --config trellis.toml
```

Example config:

```toml
# Colour theme: default, paper, blueprint, dark, midnight, forest
theme = "paper"

# Inspector panel layout for HTML output: Landscape (default) or Portrait
html_orientation = "Landscape"

# Anonymous usage ping — set to false to opt out (default: true)
telemetry = true
```

See [Telemetry](telemetry) for what is collected and how to opt out.

---

## Docker usage

The Docker image wraps the CLI binary. Mount the current directory to `/data`:

```bash
# Pull the image
docker pull ghcr.io/trellis-lab/trellis:latest

# Render a file
docker run --rm -v "$(pwd):/data" ghcr.io/trellis-lab/trellis:latest input.mmd -o output.svg

# Render to PNG
docker run --rm -v "$(pwd):/data" ghcr.io/trellis-lab/trellis:latest input.mmd -o output.png
```

---

## Stdin usage

Pass `-` as the input path to read diagram source from stdin:

```bash
echo "flowchart LR
  A --> B --> C" | trellis render - -o diagram.svg
```

Useful in shell pipelines and CI scripts.

---

## Supported diagram types

- Flowchart
- Class diagram
- Entity-relationship (ER)
- C4 architecture diagram
- Architecture (`architecture-beta`)
