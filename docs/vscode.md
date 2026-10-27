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

### Inspect diagram

Click the **Inspect Diagram** button in the preview toolbar to open an interactive inspector that lets you explore the diagram structure. Select nodes to view:
- Node labels and descriptions (for C4 diagrams)
- Connected edges with their labels
- Neighboring nodes

Click the background or close button to deselect.

### Pan and zoom

The preview supports interactive pan and zoom:
- **Zoom:** Scroll with your cursor centered on the diagram (cursor-anchored zoom). Your zoom level and pan position are saved per document.
- **Pan:** Click and drag to pan around the diagram.
- **Fit to view:** When you open the preview, the diagram automatically scales to fit the entire viewport. Double-click to cycle through zoom modes: fit-page → fit-width → actual size.

The preview re-renders sharply as you zoom in for fine details. When you select a node, the preview smoothly pans to keep it centered in view.

### Unsupported diagram types

The preview falls back to Mermaid.js 12 for diagram types Trellis doesn't yet support (e.g. sequence, state, gantt, pie, journey, gitGraph, mindmap). This means you can still preview and work with these diagrams — they just use Mermaid's default layout and look instead of Trellis's overlap-free routing. (Sequence diagrams can also be stepped through on a C4 model with [Trellis Flow](#trellis-flow).)

### Export

Export from the Command Palette (`Ctrl+Shift+P`) or the export button in the preview toolbar:

| Command | Result |
|---|---|
| `Trellis: Export Diagram…` | Pick a format interactively |
| `Trellis: Export as SVG` | Save as `.svg` |
| `Trellis: Export as PNG` | Save as `.png` |
| `Trellis: Export as Draw.io` | Save as `.drawio` mxfile |

### Anonymize

To share a diagram in a bug report without exposing confidential names, anonymize it from the Command Palette. Works on `.mmd` files and Markdown mermaid blocks, for flowchart, class, ER, C4 and architecture diagrams. Requires a license; see [`trellis anonymize`](cli#anonymize) for what is replaced and kept.

| Command | Result |
|---|---|
| `Trellis: Anonymize Diagram in Place` | Replace the diagram in the editor; undoable, nothing is saved until you save |
| `Trellis: Copy Anonymized Diagram` | Copy the anonymized diagram to the clipboard |
| `Trellis: Save Anonymized Diagram…` | Save an anonymized copy, by default as `<name>.anon.mmd` |

---

## Trellis Flow

Trellis Flow plays a Mermaid `sequenceDiagram` on top of your C4 model, one message at a time. Each step lights up the C4 nodes and relations it uses and fades out the rest, so you can walk through how a request moves through the system.

> **Commercial feature.** Trellis Flow requires a license. Without one, the Trellis view shows a short demo and how to enter a license key.

### Getting started

1. Click the **Trellis** icon in the activity bar.
2. Pick a source folder (**Trellis Flow: Use Folder as Source**). Trellis scans it for C4 models and sequence diagrams in `.mmd`, `.mermaid` and Markdown files.
3. Open a sequence from the **Sequences** pane. Trellis matches its participants to the C4 model.

The sidebar has four panes: **Source**, **C4 Model**, **Sequences** and **Validation**. Use the tree's Ctrl/Cmd+F filter to search. If a folder holds several C4 models, choose one with **Use C4 Model** or pin it with **Set as C4 Model**.

### Stepping through a sequence

- **Right / Left arrow** — next / previous step; **Space** — play / pause.
- Participants missing from the C4 model appear as dashed temporary nodes; messages with no matching relation appear as dashed edges. The **Validation** pane lists them with "did you mean" hints.
- Async messages (`-)`, `--)`) are drawn with a hollow arrowhead.
- The details column shows a mini sequence diagram with pinned participant names, notes, and `loop` / `alt` / `opt` / `par` / `critical` / `break` / `rect` blocks as nested, foldable sections. The step card lists the current step's notes and blocks with links to their source lines.
- Drag the handle between the diagram and the details column to resize it; double-click to reset.
- With **Follow Step** on, the view pans to the current step when you are zoomed in and the step is out of view.

### Live reload

Saving the sequence or C4 file keeps you on the same step. During playback, Flow pauses and asks before reloading. A reload bar offers **Reload** and **Show changes**, which tags new, changed and removed messages in the mini sequence. If a file has a syntax error, the last valid version stays on screen.

---

## Settings

Configure under **Settings → Extensions → Trellis**:

| Setting | Default | Description |
|---|---|---|
| Auto Preview | `true` | Re-render on every keystroke / file save |
| Show Edge Labels | `true` | Display labels on diagram edges |
| Show Title | `true` | Display the diagram title above the diagram |
| Theme | `default` | Color theme: `default`, `paper`, `blueprint`, `dark`, `midnight`, `forest` |
| Crossing Style | `Skip` | How crossing edges are drawn (`None`, `Arc`, `Rectangular`, `Skip`) |
| Corner Radius | `8` | Node corner roundness in pixels (`0` = sharp) |
| Label Font Size | `10` | Font size in pixels for edge labels |
| Cell Size | `40` | Grid cell size in pixels (internal grid resolution) |
| C4 Gap X | `120` | Horizontal spacing between C4 element nodes in the same row, in pixels |
| C4 Gap Y | `80` | Vertical spacing between C4 rows and sections, in pixels |
| Show Grid | `false` | Display the background grid |
| HTML Orientation | `Landscape` | Inspector panel layout in HTML export: `Landscape` or `Portrait` |

### Trellis Flow settings

Under **Settings → Extensions → Trellis**, prefixed `trellisFlow.`:

| Setting | Default | Description |
|---|---|---|
| `theme` | `auto` | Flow diagram theme. `auto` follows VS Code (`default` for light, `classicblueprint` for dark), `inherit` uses the Trellis Theme setting, or name a theme |
| `speed` | `1` | Playback speed multiplier: `0.5`, `1`, `2` |
| `travelDot` | `true` | Animate a dot along the active edge while playing |
| `followStep` | `true` | Pan the diagram to the current step when zoomed in |
| `falloff.window` | `2` | How many steps away a node or edge still shows as part of the flow |
| `falloff.curve` | `exp` | How quickly elements fade with distance: `linear` or `exp` |
| `falloff.floor` | `0.1` | Lowest opacity for elements outside the falloff window |
| `sequenceFontSize` | `10` | Font size (px) of the mini sequence diagram; spacing scales with it |
| `sequenceFontWeight` | `normal` | Font weight of the mini sequence diagram: `normal` or `bold` |
| `sequenceLineWidth` | `1.2` | Line width (px) of the mini sequence diagram's arrows |
| `scan.maxFiles` | `500` | Maximum number of files scanned for C4 models and sequences |
| `scan.exclude` | `node_modules`, `.git`, `target`, `out`, `dist` | Glob patterns the scan skips |

---

## Icons in architecture diagrams

The extension bundles a **built-in Iconify set only** — `mdi:*` and `logos:aws-*`. Unlike the CLI, it does **not** download icons on demand, so `architecture-beta` diagrams using other prefixes (`devicon`, `simple-icons`, `carbon`, …) show those icons only when rendered with the CLI or Docker. See [Troubleshooting → Icons don't appear in the VS Code preview](troubleshooting#icons-dont-appear-in-the-vs-code-preview) and the full [icon prefix list](mermaid-support#icons).

---

## Licensing

Live preview and SVG/PNG/Draw.io export work without a license.

A license key unlocks commercial features (Trellis Flow, anonymize) and removes any usage limits. Enter your key from the Command Palette:

```
Trellis: Enter License Key…
```

Paste the key from your Lemon Squeezy purchase email. To subscribe or manage seats, see the [Licensing](licensing) page. To free or move a seat between machines, see [Troubleshooting → Deregistering a device](troubleshooting#deregistering-a-device-on-lemon-squeezy).
