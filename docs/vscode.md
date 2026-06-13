---
layout: default
title: VS Code Extension
nav_order: 4
---

# VS Code Extension

The Trellis VS Code extension renders Mermaid diagrams live as you type — clean, overlap-free layouts right inside your editor. It works on standalone `.mmd` / `.mermaid` files and on ` ```mermaid ` code blocks inside Markdown.

---

## Install

From the VS Code Quick Open (`Ctrl+P` / `Cmd+P`):

```
ext install trellislab.trellis-mermaid
```

Or search **"Trellis"** in the Extensions panel (`Ctrl+Shift+X`) and click **Install**.

→ [Marketplace page](https://marketplace.visualstudio.com/items?itemName=trellislab.trellis-mermaid)

---

## Usage

### Preview a diagram file

1. Open a `.mmd` or `.mermaid` file.
2. Press **`Ctrl+Shift+V`** (**Trellis: Open Preview to the Side**), or run **Trellis: Open Preview** from the Command Palette.
3. The preview updates live as you edit.

### Preview a Mermaid block in Markdown

Open any Markdown file containing a ` ```mermaid ` block. A CodeLens appears above each block — click it (**Trellis: Preview Mermaid Block**) to render that diagram.

### Export

Export from the Command Palette (`Ctrl+Shift+P`) or the export button in the preview toolbar:

| Command | Result |
|---|---|
| `Trellis: Export Diagram…` | Pick a format interactively |
| `Trellis: Export as SVG` | Save as `.svg` |
| `Trellis: Export as PNG` | Save as `.png` |
| `Trellis: Export as Draw.io` | Save as `.drawio` mxfile |

---

## Settings

Configure under **Settings → Extensions → Trellis**:

| Setting | Default | Description |
|---|---|---|
| Auto Preview | `true` | Re-render on every keystroke / file save |
| Show Edge Labels | `true` | Display labels on diagram edges |
| Show Title | `true` | Display the diagram title above the diagram |
| Crossing Style | `Skip` | How crossing edges are drawn (`None`, `Arc`, `Rectangular`, `Skip`) |
| Corner Radius | `8` | Node corner roundness in pixels (`0` = sharp) |
| Label Font Size | `10` | Font size in pixels for edge labels |

---

## Licensing

Live preview and SVG/PNG/Draw.io export work without a license.

A license key unlocks commercial features and removes any usage limits. Enter your key from the Command Palette:

```
Trellis: Enter License Key…
```

Paste the key from your Lemon Squeezy purchase email. To subscribe or manage seats, see the [Licensing](licensing) page. To free or move a seat between machines, see [Troubleshooting → Deregistering a device](troubleshooting#deregistering-a-device-on-lemon-squeezy).
