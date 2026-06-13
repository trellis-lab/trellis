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

Diagram types not in this list (sequence, state, gantt, pie, journey, gitGraph, mindmap, etc.) are not rendered.

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

`subgraph ID [label] … end` blocks are supported.

### Limitations

These are parsed but **ignored** — they don't error, but they have no visual effect:

- `style …` — inline node styling
- `classDef …` / `class …` — CSS-style classes
- `linkStyle …` — per-edge styling
- `click …` — click interactions / links

Also not yet supported (on the roadmap):

- **Custom shape strings** (the newer `id@{ shape: … }` syntax).
- **Markdown-formatted labels** (e.g. `` id["`**bold** _italic_`"] ``) — rendered as plain text for now.

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
