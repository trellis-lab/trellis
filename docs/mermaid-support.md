---
layout: default
title: Mermaid Language Support
nav_order: 2
---

# Mermaid Language Support

Trellis is a drop-in renderer for standard Mermaid syntax — existing `.mmd` files render unchanged. This page lists which diagram types are supported, what flowchart syntax Trellis understands (and what it ignores), and the extra C4 node types Trellis adds on top of standard Mermaid.

---

## Supported diagram types

| Type | Directive(s) |
|---|---|
| Flowchart | `flowchart`, `graph` (with `TB` / `TD` / `BT` / `LR` / `RL` direction) |
| Class diagram | `classDiagram` |
| Entity-relationship | `erDiagram` |
| C4 architecture | `C4Context`, `C4Container`, `C4Component`, `C4Dynamic`, `C4Deployment` |
| Architecture | `architecture-beta` |

Diagram types not in this list (sequence, state, gantt, pie, journey, gitGraph, mindmap, etc.) are not natively supported by Trellis. However, the **VS Code extension falls back to Mermaid.js** rendering for these types, so you can still preview them with Mermaid's default layout. The CLI and other tools will show an error for unsupported types.

---

## Flowchart support

### Node shapes

| Syntax | Shape |
|---|---|
| `id[text]` | Rectangle |
| `id(text)` | Rounded rectangle |
| `id([text])` | Stadium |
| `id((text))` | Circle |
| `id(((text)))` | Double circle |
| `id{text}` | Diamond |
| `id{{text}}` | Hexagon |
| `id[[text]]` | Subroutine |
| `id[(text)]` | Cylinder |
| `id[/text/]` | Parallelogram |
| `id[\text\]` | Parallelogram (alt) |
| `id[/text\]` | Trapezoid |
| `id[\text/]` | Trapezoid (alt) |
| `id>text]` | Asymmetric |

### Edges

Solid, dotted, and thick arrows, with or without arrowheads and labels:

```
A --> B            solid arrow
A --- B            solid line, no arrowhead
A -.-> B           dotted arrow
A -.- B            dotted line
A ==> B            thick arrow
A === B            thick line
A -->|text| B      labelled arrow
A --text--> B      inline-label arrow
A -.text.-> B      labelled dotted arrow
A ==text==> B      labelled thick arrow
```

Multiple sources/targets with `&` are supported:

```
a & b & c --> d
```

### Subgraphs

`subgraph ID [label] … end` blocks are supported, including a bare quoted title with no separate id (`subgraph "Title"`).

### Markdown labels

Node and edge labels support inline markdown when delimited by `` "` ``…`` `" `` — Mermaid's markdown-string syntax:

```
A("`The **cat** in the hat`") -- "`Bold **edge label**`" --> B
```

Supported subset: **bold** (`**text**`), *italic* (`*text*`), and line breaks — matching Mermaid's `htmlLabels: false` rendering. Links, code spans, lists, headings, and strikethrough are out of scope (Mermaid itself rejects them in markdown strings), and a leading list/heading marker (`5. Deploy`, `# Stage 1`, `- item`) is preserved as literal text rather than reinterpreted. A literal line break inside the backticks becomes a line break in the rendered label. Long markdown labels wrap at word boundaries, configurable via `markdown_wrap_width` (default `200.0` px; `0.0` disables wrapping). Plain (non-markdown, non-quoted-string) labels are unaffected — this only applies to the `` "` … `" `` form.

Supported across every output format: SVG and PNG render true bold/italic; `drawio` export sets `html=1` with `<b>`/`<i>` markup; `ascii` output and the `html` inspector's node/edge data show the plain text with styling stripped.

### Limitations

These are parsed but **ignored** — they don't error, but they have no visual effect:

- `style …` — inline node styling
- `classDef …` / `class …` — CSS-style classes
- `linkStyle …` — per-edge styling
- `click …` — click interactions / links

Also not yet supported (on the roadmap):

- **Custom shape strings** (the newer `id@{ shape: … }` syntax).
- **Mindmap diagrams** — markdown labels are supported in the underlying model and expected to extend here without rework.

> Trellis controls colour and appearance through [themes](cli#themes) and config, not inline Mermaid `style`/`classDef` directives.

---

## C4 support

Trellis supports the standard Mermaid C4 vocabulary — `Person`, `System`, `Container`, `Component` (with their `Db`, `Queue`, and `_Ext` variants), boundaries (`Enterprise_Boundary`, `System_Boundary`, `Container_Boundary`), deployment nodes (`Deployment_Node`, `Node`, `Node_L`, `Node_R`), and relationships (`Rel`, `BiRel`, directional `Rel_U/D/L/R`, `Rel_Back`, `UpdateLayoutConfig`).

### Supplementary node types

On top of standard Mermaid, Trellis adds extra `Container`-family node types with purpose-specific shapes. Each also has an `_Ext` (external) variant.

| Keyword | `_Ext` variant | Use for |
|---|---|---|
| `ContainerApp` | `ContainerApp_Ext` | Application / service container |
| `ContainerFrontend` | `ContainerFrontend_Ext` | Frontend / UI container |
| `ContainerGateway` | `ContainerGateway_Ext` | API gateway / edge router |
| `ContainerFolder` | `ContainerFolder_Ext` | Config / file storage |
| `ContainerBucket` | `ContainerBucket_Ext` | Object / blob storage |

**Example:**

```
C4Container
    ContainerGateway(gw, "API Gateway", "Kong", "Routes all traffic")
    ContainerApp(app, "Web App", "Node.js", "Serves the UI")
    ContainerBucket(s3, "Assets", "S3", "Static assets")
    ContainerFolder(cfg, "Config", "YAML", "App config files")
    Rel(gw, app, "Forwards to")
    Rel(app, s3, "Reads assets from")
```

These keywords are Trellis extensions — they render in Trellis but are not part of the official Mermaid C4 specification.

---

## Architecture (`architecture-beta`) support

Trellis renders Mermaid `architecture-beta` diagrams with `group`, `service`, and `junction` elements, orthogonal edge routing, and optional icons. Draw.io export writes architecture diagrams as swimlane containers with embedded icons.

```
architecture-beta
    group cloud(logos:aws)[Cloud]
    service db(logos:postgresql)[Database] in cloud
    service api(logos:aws-lambda)[API] in cloud
    api:R --> L:db
```

### Icons

Icons use Iconify `prefix:name` identifiers (e.g. `logos:aws`, `logos:postgresql`). The **CLI and Docker images download icons on demand** from any of the supported Iconify sets and cache them locally:

`mdi`, `logos`, `vscode-icons`, `devicon`, `carbon`, `tabler`, `heroicons`, `lucide`, `ph`, `ri`, `bi`, `fa`, `fa6-solid`, `fa6-brands`, `simple-icons`, `material-symbols`, `fluent`, `ant-design`

Only these prefixes are allowed — downloads from arbitrary hosts/paths are blocked, and fetched SVGs are size-capped and content-scanned before embedding.

> **VS Code extension:** the extension ships a **built-in icon set only** (`mdi:*` and `logos:aws-*`). It does not download icons on demand. Diagrams using other Iconify prefixes render their icons only via the CLI or Docker. See [Troubleshooting → Icons don't appear in the VS Code preview](troubleshooting#icons-dont-appear-in-the-vs-code-preview).
