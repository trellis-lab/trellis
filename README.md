# Trellis

**Mermaid diagrams that actually look good.**

> Early access — [join the discussion](https://github.com/trellis-lab/trellis/discussions)

Trellis is a drop-in replacement for the default Mermaid renderer. It produces clean, overlap-free diagrams — no tangled edges, no manual layout tweaking, no post-processing in a drawing tool.

→ [trellislab.net](https://trellislab.net) — live comparison and interactive editor

---

## Why Trellis

Standard Mermaid uses an aging layout engine that frequently produces diagrams with overlapping and crossing edges. The result is diagrams that are hard to read and not presentation-ready without significant manual effort.

Trellis replaces that engine. Every edge finds its own clear corridor. Diagrams come out clean the first time.

**Standard Mermaid syntax. Zero migration.** Existing `.mmd` files work without changes.

---

## Supported diagram types

- Flowchart
- Class diagram
- Entity-relationship (ER)
- C4 architecture diagram

---

## Installation

### VS Code extension

Install from the marketplace:

```
ext install trellislab.trellis-mermaid
```

Or search **"Trellis"** in the VS Code Extensions panel. Live preview updates as you type.

→ [Marketplace page](https://marketplace.visualstudio.com/items?itemName=trellislab.trellis-mermaid)

---

### CLI binary

Download the pre-built binary for your platform from the [releases page](https://github.com/trellis-lab/trellis/releases):

| Platform | File |
|---|---|
| macOS (Apple Silicon) | `trellis-vX.Y.Z-aarch64-apple-darwin.tar.gz` |
| macOS (Intel) | `trellis-vX.Y.Z-x86_64-apple-darwin.tar.gz` |
| Linux x86_64 | `trellis-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz` |
| Windows x86_64 | `trellis-vX.Y.Z-x86_64-pc-windows-gnu.zip` |

Extract and add the binary to your `PATH`.

---

### Docker

```bash
docker pull trellis-lab/trellis:latest
docker run --rm -v "$(pwd):/data" trellis-lab/trellis:latest input.mmd -o output.svg
```

---

## CLI usage

```bash
# Render to SVG
trellis render diagram.mmd -o diagram.svg

# Render to PNG
trellis render diagram.mmd -o diagram.png

# Render from stdin
echo "flowchart LR
  A --> B --> C" | trellis render - -o diagram.svg

# Render all diagrams in a directory
trellis render-batch ./diagrams -o ./output

# Validate syntax only
trellis validate diagram.mmd
```

---

## Output formats

- **SVG** — scalable, embeddable, publication-ready
- **PNG** — rasterized for presentations and documentation

---

## Pandoc integration

Trellis includes a Pandoc filter for rendering Mermaid code blocks inside Markdown documents directly to PDF or HTML — no pre-processing step needed.

---

## Live editor

Try Trellis in the browser without installing anything:

→ [trellislab.net/editor.html](https://trellislab.net/editor.html)

---

## Releases and changelog

→ [github.com/trellis-lab/trellis/releases](https://github.com/trellis-lab/trellis/releases)

---

## Issues and feedback

→ [github.com/trellis-lab/trellis/issues](https://github.com/trellis-lab/trellis/issues)
