# Diagram Theme Support — Strategy Document

**Status:** Proposal  
**Date:** 2026-04-13  
**Scope:** `trellis-core`, `trellis-cli`, `trellis-wasm`

---

## 1. Problem Statement

All colors in Trellis are hardcoded as string literals scattered across 11 render files (~70+ hex values). Users cannot change visual style without recompiling. There is no concept of light/dark mode, branding, or consistent palette.

This document defines a strategy for introducing **6 built-in named themes** (3 light, 3 dark) with a clean architecture that allows future extensibility.

---

## 2. Color Token Model

Rather than passing individual hex values, all render functions will receive a `&Theme` reference. The theme exposes a flat set of **semantic color tokens**:

### 2.1 Global Tokens

| Token | Meaning |
|---|---|
| `background` | SVG canvas fill |
| `node_fill` | Default node interior |
| `node_stroke` | Default node border |
| `node_text` | Default node label text |
| `edge_stroke` | Line color for edges |
| `edge_fallback_stroke` | Unrouted / fallback edge |
| `edge_label_text` | Text on edge labels |
| `edge_label_bg` | Background behind edge label |
| `edge_label_border` | Border of edge label bubble |
| `arrow_fill` | Arrowhead fill |
| `crossing_bg` | Hop decoration background |
| `crossing_stroke` | Hop arc/rect stroke |
| `grid_dot` | Debug grid dot fill |
| `multiplicity_text` | Source/target multiplicity label |

### 2.2 Flowchart Shape Tokens

Flowchart nodes are colored by **shape role**, not by individual node. Six shape roles cover all current shapes:

| Token | Covers |
|---|---|
| `shape_process_fill / _stroke` | Rectangle, RoundedRect, Subroutine, Asymmetric, Parallelogram, Trapezoid |
| `shape_decision_fill / _stroke` | Diamond |
| `shape_terminal_fill / _stroke` | Circle, DoubleCircle, Stadium |
| `shape_storage_fill / _stroke` | Cylinder |
| `shape_io_fill / _stroke` | (reserved for future I/O shape) |
| `shape_special_fill / _stroke` | Hexagon |

### 2.3 Class Diagram Tokens

| Token | Meaning |
|---|---|
| `class_box_fill` | Class box background |
| `class_box_stroke` | Class box border |
| `class_header_text` | Class name + stereotype |
| `class_member_text` | Method / attribute text |
| `class_separator` | Horizontal divider line |
| `class_marker_fill` | Arrow marker interior |

### 2.4 ER Diagram Tokens

| Token | Meaning |
|---|---|
| `er_box_fill` | Entity box background |
| `er_box_stroke` | Entity box border |
| `er_entity_text` | Entity name |
| `er_attribute_text` | Attribute rows |
| `er_glyph_stroke` | Crow's foot line color |

### 2.5 C4 Diagram Tokens

C4 uses element-type-specific colors because they carry semantic meaning (Person vs System vs External). Theme defines one palette set; the render layer applies opacity/tint variants per type.

| Token | Meaning |
|---|---|
| `c4_person_fill / _stroke` | Person element |
| `c4_system_fill / _stroke` | Internal system |
| `c4_container_fill / _stroke` | Container |
| `c4_component_fill / _stroke` | Component |
| `c4_external_fill / _stroke` | Any `*_Ext` variant |
| `c4_deployment_fill / _stroke` | Deployment node |
| `c4_boundary_enterprise_fill / _stroke` | Enterprise boundary |
| `c4_boundary_system_fill / _stroke` | System boundary |
| `c4_boundary_container_fill / _stroke` | Container boundary |
| `c4_boundary_deployment_fill / _stroke` | Deployment boundary |
| `c4_text` | All C4 element text |
| `c4_description_text` | Description / technology lines |

### 2.6 Subgraph Tokens

Subgraphs nest up to 4 levels deep. Theme defines 4 color pairs:

| Token | Meaning |
|---|---|
| `subgraph_fill[0..4]` | Background per depth level |
| `subgraph_stroke[0..4]` | Border per depth level |
| `subgraph_label[0..4]` | Label text per depth level |

---

## 3. The Six Built-In Themes

### Light Themes

#### L1 — `default` (current palette, codified)
The existing hardcoded colors extracted into a theme struct. Zero visual change on upgrade. Serves as migration safety net.

| Role | Fill | Stroke |
|---|---|---|
| Canvas | `#ffffff` | — |
| Process node | `#e8f4fd` | `#4a90d9` |
| Decision node | `#fff3e0` | `#e67e22` |
| Terminal node | `#e8f5e9` | `#43a047` |
| Storage node | `#e8f4fd` | `#4a90d9` |
| Special node | `#f3e5f5` | `#8e24aa` |
| Edge | — | `#555555` |
| Class box | `#f5f5f5` | `#555555` |
| ER box | `#f0f7ff` | `#336699` |

#### L2 — `paper`
Warm off-white canvas. Sepia-toned greys for neutral elements. Amber accents for decisions. Subtle, document-like feel. Suitable for technical documentation exports.

| Role | Fill | Stroke |
|---|---|---|
| Canvas | `#faf8f5` | — |
| Process node | `#f0ece4` | `#8b7355` |
| Decision node | `#fef3c7` | `#d97706` |
| Terminal node | `#ecfdf5` | `#059669` |
| Storage node | `#f0ece4` | `#8b7355` |
| Special node | `#fdf4ff` | `#9333ea` |
| Edge | — | `#6b5744` |
| Class box | `#f5f0e8` | `#7c6a56` |
| ER box | `#fef9ef` | `#a0845c` |

#### L3 — `blueprint`
White canvas with saturated blue accent scheme throughout. All shapes share a single blue family. Inspired by engineering blueprint / technical drawing aesthetic. High contrast text. Good for presentations.

| Role | Fill | Stroke |
|---|---|---|
| Canvas | `#f8faff` | — |
| Process node | `#dbeafe` | `#2563eb` |
| Decision node | `#ede9fe` | `#7c3aed` |
| Terminal node | `#d1fae5` | `#059669` |
| Storage node | `#dbeafe` | `#2563eb` |
| Special node | `#fce7f3` | `#db2777` |
| Edge | — | `#374151` |
| Class box | `#eff6ff` | `#1d4ed8` |
| ER box | `#eff6ff` | `#1e40af` |

---

### Dark Themes

#### D1 — `dark`
Dark grey canvas (`#1e1e1e`, VS Code-style). Muted fills, lighter strokes. Text in near-white. The canonical dark mode.

| Role | Fill | Stroke |
|---|---|---|
| Canvas | `#1e1e1e` | — |
| Process node | `#1e3a5f` | `#5b9bd5` |
| Decision node | `#3d2b00` | `#e0a020` |
| Terminal node | `#1a3d2b` | `#4caf79` |
| Storage node | `#1e3a5f` | `#5b9bd5` |
| Special node | `#2d1a40` | `#b07de0` |
| Edge | — | `#9e9e9e` |
| Class box | `#2a2a2a` | `#888888` |
| ER box | `#1a2d40` | `#5b8db8` |
| Text (global) | `#e0e0e0` | — |

#### D2 — `midnight`
Deep navy canvas (`#0d1117`, GitHub dark style). Cool blue-grey palette. Cyan/teal accents. Very low eye strain. Optimized for extended viewing.

| Role | Fill | Stroke |
|---|---|---|
| Canvas | `#0d1117` | — |
| Process node | `#0d2137` | `#58a6ff` |
| Decision node | `#1a1000` | `#e3b341` |
| Terminal node | `#0d2618` | `#3fb950` |
| Storage node | `#0d2137` | `#58a6ff` |
| Special node | `#1e0d2e` | `#bc8cff` |
| Edge | — | `#8b949e` |
| Class box | `#161b22` | `#30363d` |
| ER box | `#0d1e2e` | `#388bfd` |
| Text (global) | `#c9d1d9` | — |

#### D3 — `forest`
Dark green canvas. Earth-tone palette. Warm amber for decisions, moss green for terminals. Comfortable for developers who prefer green-on-dark aesthetic.

| Role | Fill | Stroke |
|---|---|---|
| Canvas | `#0f1a0f` | — |
| Process node | `#0d2210` | `#4ade80` |
| Decision node | `#1f1500` | `#fbbf24` |
| Terminal node | `#0d2218` | `#34d399` |
| Storage node | `#0d2210` | `#4ade80` |
| Special node | `#1a0d22` | `#c084fc` |
| Edge | — | `#86efac` |
| Class box | `#111a11` | `#4ade80` |
| ER box | `#0d1a0d` | `#22c55e` |
| Text (global) | `#d1fae5` | — |

---

## 4. Rust Architecture

### 4.1 New Module: `trellis-core/src/theme.rs`

```rust
/// Flat color palette. All values are CSS hex strings (e.g. "#ff0000").
#[derive(Debug, Clone)]
pub struct Theme {
    // --- Global ---
    pub background: &'static str,
    pub node_fill: &'static str,
    pub node_stroke: &'static str,
    pub node_text: &'static str,
    pub edge_stroke: &'static str,
    pub edge_fallback_stroke: &'static str,
    pub edge_label_text: &'static str,
    pub edge_label_bg: &'static str,
    pub edge_label_border: &'static str,
    pub arrow_fill: &'static str,
    pub crossing_bg: &'static str,
    pub crossing_stroke: &'static str,
    pub grid_dot: &'static str,
    pub multiplicity_text: &'static str,

    // --- Flowchart shape roles ---
    pub shape_process_fill: &'static str,
    pub shape_process_stroke: &'static str,
    pub shape_decision_fill: &'static str,
    pub shape_decision_stroke: &'static str,
    pub shape_terminal_fill: &'static str,
    pub shape_terminal_stroke: &'static str,
    pub shape_storage_fill: &'static str,
    pub shape_storage_stroke: &'static str,
    pub shape_special_fill: &'static str,
    pub shape_special_stroke: &'static str,

    // --- Class diagram ---
    pub class_box_fill: &'static str,
    pub class_box_stroke: &'static str,
    pub class_header_text: &'static str,
    pub class_member_text: &'static str,
    pub class_separator: &'static str,
    pub class_marker_fill: &'static str,

    // --- ER diagram ---
    pub er_box_fill: &'static str,
    pub er_box_stroke: &'static str,
    pub er_entity_text: &'static str,
    pub er_attribute_text: &'static str,
    pub er_glyph_stroke: &'static str,

    // --- C4 diagram ---
    pub c4_person_fill: &'static str,
    pub c4_person_stroke: &'static str,
    pub c4_system_fill: &'static str,
    pub c4_system_stroke: &'static str,
    pub c4_container_fill: &'static str,
    pub c4_container_stroke: &'static str,
    pub c4_component_fill: &'static str,
    pub c4_component_stroke: &'static str,
    pub c4_external_fill: &'static str,
    pub c4_external_stroke: &'static str,
    pub c4_deployment_fill: &'static str,
    pub c4_deployment_stroke: &'static str,
    pub c4_boundary_enterprise_fill: &'static str,
    pub c4_boundary_enterprise_stroke: &'static str,
    pub c4_boundary_system_fill: &'static str,
    pub c4_boundary_system_stroke: &'static str,
    pub c4_boundary_container_fill: &'static str,
    pub c4_boundary_container_stroke: &'static str,
    pub c4_boundary_deployment_fill: &'static str,
    pub c4_boundary_deployment_stroke: &'static str,
    pub c4_text: &'static str,
    pub c4_description_text: &'static str,

    // --- Subgraph (4 depth levels) ---
    pub subgraph_fill: [&'static str; 4],
    pub subgraph_stroke: [&'static str; 4],
    pub subgraph_label: [&'static str; 4],
}

/// Named theme selector.
#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeName {
    #[default]
    Default,
    Paper,
    Blueprint,
    Dark,
    Midnight,
    Forest,
}

impl ThemeName {
    pub fn theme(&self) -> &'static Theme {
        match self {
            ThemeName::Default   => &themes::DEFAULT,
            ThemeName::Paper     => &themes::PAPER,
            ThemeName::Blueprint => &themes::BLUEPRINT,
            ThemeName::Dark      => &themes::DARK,
            ThemeName::Midnight  => &themes::MIDNIGHT,
            ThemeName::Forest    => &themes::FOREST,
        }
    }
}

/// Sub-module holding the 6 static theme instances.
pub mod themes { /* ... one pub static per theme ... */ }
```

### 4.2 Config Integration (`config.rs`)

Add one field to `TrellisConfig`:

```rust
/// Visual color theme. Options: default, paper, blueprint, dark, midnight, forest.
pub theme: ThemeName,
```

Default: `ThemeName::Default` (zero visual change on upgrade).

TOML key: `theme = "dark"`.

### 4.3 Render Function Signature Changes

All render entry points receive `theme: &Theme` as an additional parameter. Thread it through the call chain:

```
pipeline::render(graph, config, format)
  └─ svg::build_svg(graph, layout, &config.theme.theme(), ...)
       ├─ nodes::render_node(node, ..., theme)
       ├─ class_shapes::render_class_node(node, ..., theme)
       ├─ er_shapes::render_er_node(node, ..., theme)
       ├─ c4_shapes::render_c4_node(node, ..., theme)
       ├─ c4_boundary::render_boundary(boundary, ..., theme)
       ├─ subgraph::render_subgraph(subgraph, depth, ..., theme)
       ├─ edges::render_edge(edge, ..., theme)
       └─ crossing::render_crossing(..., theme)
```

No render function constructs hex strings directly — all values come from `theme.*` fields.

### 4.4 Stroke Width Policy

Stroke widths are **not** theme tokens. They are geometry constants and remain hardcoded. Only color values move into `Theme`.

---

## 5. Implementation Plan

### Phase 1 — Scaffold (no visual change)
1. Create `trellis-core/src/theme.rs` with `Theme` struct, `ThemeName` enum, and `themes::DEFAULT` populated with all current hardcoded values.
2. Add `theme: ThemeName` to `TrellisConfig` with `Default` variant.
3. Wire `theme.theme()` call in `pipeline.rs` and pass `&Theme` into `build_svg`.
4. Thread `theme` parameter down through all render functions.
5. Replace every hardcoded hex string in render files with the corresponding `theme.*` field.
6. All 410 tests pass. Zero visual diff on `default` theme.

### Phase 2 — Remaining 5 themes
1. Add `themes::PAPER`, `themes::BLUEPRINT`, `themes::DARK`, `themes::MIDNIGHT`, `themes::FOREST` static instances.
2. Verify serde round-trips for `ThemeName` in TOML.
3. Add CLI `--theme <name>` flag (delegates to config field).
4. Add WASM `render(src, config_json)` — theme name already flows through config JSON.
5. Update `default.toml` with `# theme = "default"` comment and valid options list.

### Phase 3 — Tests and fixtures
1. Add one integration test per theme: render `b01.mmd`, assert SVG contains expected canvas background color.
2. Add benchmark fixture `b25.mmd` with all diagram types mixed (flowchart + subgraphs) to smoke-test theme coverage.
3. Update docs.

---

## 6. Non-Goals

- **Custom user-defined themes via TOML color tables** — deferred. Token model is designed to support this later (swap `&'static str` for `String`, add TOML `[theme.custom]` table), but not in this iteration.
- **Per-node color overrides** — deferred. AST `Node` could carry optional `fill`/`stroke` fields that override theme values.
- **CSS class-based SVG output** — considered but rejected for now. Inline attributes are simpler, work in all SVG consumers (including resvg/PNG), and avoid introducing a class-name contract.
- **Automatic dark mode detection** — out of scope. Theme is always explicitly selected.

---

## 7. File Impact Summary

| File | Change |
|---|---|
| `trellis-core/src/theme.rs` | **New** — Theme struct + 6 static instances |
| `trellis-core/src/config.rs` | Add `theme: ThemeName` field |
| `trellis-core/src/pipeline.rs` | Pass `theme` to `build_svg` |
| `trellis-core/src/render/svg.rs` | Accept `&Theme`, use tokens |
| `trellis-core/src/render/nodes.rs` | Replace ~18 hex strings |
| `trellis-core/src/render/class_shapes.rs` | Replace ~8 hex strings |
| `trellis-core/src/render/er_shapes.rs` | Replace ~6 hex strings |
| `trellis-core/src/render/er_glyphs.rs` | Replace ~1 hex string |
| `trellis-core/src/render/c4_shapes.rs` | Replace 12 constants |
| `trellis-core/src/render/c4_boundary.rs` | Replace 8 constants |
| `trellis-core/src/render/subgraph.rs` | Replace 3 color arrays |
| `trellis-core/src/render/edges.rs` | Replace ~4 hex strings |
| `trellis-core/src/render/crossing.rs` | Replace ~2 hex strings |
| `trellis-core/src/render/grid.rs` | Replace 1 hex string |
| `trellis-cli/src/main.rs` | Add `--theme` CLI flag |
| `trellis-cli/default.toml` | Add `theme` documentation |
| `trellis-wasm/src/lib.rs` | No change (theme flows via config JSON) |
| `tests/integration/` | Add 6 theme smoke tests |

---

## 8. Token Count Reference

Total color tokens in `Theme` struct:

- Global: 14
- Flowchart shape roles: 10
- Class diagram: 6
- ER diagram: 5
- C4 elements: 16
- C4 boundaries: 8
- Subgraph (3 arrays × 4): 12

**Total: 71 tokens per theme × 6 themes = 426 static color values**

All defined as `&'static str` in `theme.rs`. Zero runtime allocation.
