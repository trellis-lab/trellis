# Strategy: Diagram Title Support

## Problem

Mermaid supports YAML frontmatter before the diagram body:

```
---
title: E-Commerce Checkout Flow
---
flowchart TD
    A --> B
```

Trellis currently ignores all frontmatter. The tokenizer starts processing from the first non-empty line, so lines like `---`, `title: ...`, and closing `---` pass through as unknown `Statement` tokens or are silently skipped. The `Graph` struct has no field to store the title. No title is rendered in SVG output.

---

## Scope

Only `title` is in scope. Other YAML frontmatter keys (`config`, `theme`, etc.) are deferred.

---

## Format Reference

Mermaid frontmatter is a strict subset of YAML:

```
---
title: <arbitrary UTF-8 text>
---
```

Rules observed in the wild:
- `---` on its own line, no trailing text.
- `title:` key followed by unquoted string (trimmed).
- Closing `---` on its own line.
- Frontmatter must precede the diagram directive (`flowchart`, `graph`, `classDiagram`, etc.).
- No multi-line values. No nested keys.

---

## Affected Components

| Component | File | Change |
|---|---|---|
| Parser — frontmatter strip | `crates/trellis-parser/src/lib.rs` | Extract title before tokenizing |
| AST | `crates/trellis-parser/src/ast.rs` | Add `title: Option<String>` to `Graph` |
| Theme | `crates/trellis-core/src/theme.rs` | Add `title_text: &'static str` token to all 7 themes |
| Config | `crates/trellis-core/src/config.rs` | Add `show_title: bool` (default `true`) |
| SVG renderer | `crates/trellis-core/src/render/svg.rs` | Render `<title>` + visible caption; honour `show_title` |
| ViewBox calculation | `crates/trellis-core/src/render/svg.rs` | Expand height when caption present |

No changes needed in: tokenizer, individual diagram parsers, placement, routing, labels, WASM API (title flows through `Graph`).

---

## Implementation Plan

### Step 1 — AST: Add `title` field to `Graph`

In `crates/trellis-parser/src/ast.rs`:

```rust
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub subgraphs: Vec<Subgraph>,
    pub direction: Direction,
    pub diagram_type: DiagramType,
    pub title: Option<String>,   // NEW
}
```

`Default` impl already derived — new field defaults to `None`. No churn in existing construction sites using `..Default::default()`.

---

### Step 2 — Parser: Strip frontmatter before tokenizing

In `crates/trellis-parser/src/lib.rs`, add a preprocessing step:

```rust
pub fn parse(input: &str) -> Result<Graph, ParseError> {
    let (title, body) = extract_frontmatter(input);
    let tokens = tokenizer::tokenize(body);
    let diagram_type = tokenizer::detect_diagram_type(&tokens);

    let mut graph = match diagram_type {
        DiagramType::Flowchart     => flowchart::parse_flowchart(&tokens)?,
        DiagramType::ClassDiagram  => class_diagram::parse_class_diagram(&tokens)?,
        DiagramType::ErDiagram     => er_diagram::parse_er_diagram(&tokens)?,
        DiagramType::C4Diagram     => c4_diagram::parse_c4_diagram(&tokens)?,
    };
    graph.title = title;
    Ok(graph)
}

/// Extract optional YAML frontmatter. Returns (title, diagram_body).
///
/// Accepts only `title:` key. Silently ignores other keys.
/// Returns the original input unchanged if no frontmatter found.
fn extract_frontmatter(input: &str) -> (Option<String>, &str) {
    let trimmed = input.trim_start();
    if !trimmed.starts_with("---") {
        return (None, input);
    }
    let after_open = match trimmed.find('\n') {
        Some(pos) => &trimmed[pos + 1..],
        None => return (None, input),
    };
    // Find closing "---"
    let close_marker = "\n---";
    match after_open.find(close_marker) {
        None => (None, input),
        Some(close_pos) => {
            let frontmatter = &after_open[..close_pos];
            let body = &after_open[close_pos + close_marker.len()..];
            let title = parse_title_from_frontmatter(frontmatter);
            (title, body)
        }
    }
}

fn parse_title_from_frontmatter(fm: &str) -> Option<String> {
    for line in fm.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("title:") {
            let value = rest.trim().to_string();
            if !value.is_empty() {
                return Some(value);
            }
        }
    }
    None
}
```

**Edge cases:**
- No frontmatter → `(None, original_input)` — zero cost, existing behaviour preserved.
- Frontmatter present but no `title:` key → `(None, body)` — body still parsed correctly.
- Empty `title:` value → treated as absent (`None`).
- Frontmatter with unknown keys → keys silently ignored, title extracted if present.

---

### Step 3 — Theme: Add `title_text` colour token

`theme.rs` `Theme` struct currently has no dedicated title colour. The nearest existing token is `node_text`, but title is a canvas-level element (not inside a node), so it needs its own token for correct contrast on all backgrounds.

Add to `Theme`:

```rust
/// Colour of the visible title caption rendered above the diagram.
pub title_text: &'static str,
```

Set values for all 7 static theme instances:

| Theme | `title_text` | Rationale |
|---|---|---|
| `DEFAULT` | `"#111"` | Near-black on white canvas |
| `PAPER` | `"#1a0f00"` | Dark sepia, matches `class_header_text` |
| `BLUEPRINT` | `"#1e3a5f"` | Dark navy, matches `node_text` |
| `DARK` | `"#e0e0e0"` | Light grey on dark canvas, matches `node_text` |
| `MIDNIGHT` | `"#c9d1d9"` | GitHub-dark foreground, matches `node_text` |
| `FOREST` | `"#d1fae5"` | Pale green on dark green canvas, matches `node_text` |
| `CLASSIC` | `"#E8F4FB"` | Light blue-white on blueprint dark canvas, matches `node_text` |

**Usage in svg.rs:** `theme.title_text` (replaces the erroneous `theme.label_color` reference from the original draft).

---

### Step 4 — Config: Add `show_title` flag

In `crates/trellis-core/src/config.rs`:

```rust
fn default_show_title() -> bool { true }
```

Add to `TrellisConfig`:

```rust
/// Render the diagram title as a visible caption above the diagram.
///
/// When `false`, the title is still extracted and stored in `Graph.title`
/// (for tooling / metadata use), and the SVG `<title>` accessibility element
/// is still emitted — only the visible caption is suppressed.
#[serde(default = "default_show_title")]
pub show_title: bool,
```

Add to both `Default` impl and `configuration_factory` `Benchmark` arm:

```rust
show_title: true,
```

**TOML usage:**
```toml
show_title = false   # suppress visible caption; SVG <title> still written
```

**Design note:** `show_title = false` suppresses only the visual text element and the corresponding viewBox expansion. The `<title>` accessibility element is always emitted when `graph.title` is `Some` — it is invisible in rendered output and has zero layout impact.

---

### Step 5 — SVG Renderer: Emit title

In `crates/trellis-core/src/render/svg.rs`.

#### 5a. `escape_xml` helper

```rust
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}
```

#### 5b. Constants

```rust
const TITLE_FONT_SIZE: f64 = 16.0;
const TITLE_PADDING: f64   = 8.0;
/// Total vertical space consumed by the caption bar.
const TITLE_HEIGHT: f64    = TITLE_FONT_SIZE + TITLE_PADDING * 2.0;
```

#### 5c. ViewBox expansion

After `calculate_viewbox` returns `(vx, vy, vw, vh)`, apply upward expansion when caption will be shown:

```rust
let show_caption = graph.title.is_some() && config.show_title;
let (vy, vh) = if show_caption {
    (vy - TITLE_HEIGHT, vh + TITLE_HEIGHT)
} else {
    (vy, vh)
};
```

The expansion is upward-only (shrinks `vy`, grows `vh`) so the diagram body position is unchanged.

#### 5d. SVG `<title>` element (accessibility)

Always emit when `graph.title` is `Some`, regardless of `show_title`:

```rust
if let Some(ref title) = graph.title {
    svg.push_str(&format!("<title>{}</title>\n", escape_xml(title)));
}
```

#### 5e. Visible caption

Gated on `show_caption`:

```rust
if show_caption {
    if let Some(ref title) = graph.title {
        let tx = vx + vw / 2.0;
        let ty = vy + TITLE_PADDING + TITLE_FONT_SIZE;
        svg.push_str(&format!(
            "<text x=\"{:.1}\" y=\"{:.1}\" \
             font-family=\"Arial, Helvetica, sans-serif\" \
             font-size=\"{:.0}\" font-weight=\"bold\" \
             fill=\"{}\" text-anchor=\"middle\">{}</text>\n",
            tx, ty, TITLE_FONT_SIZE, theme.title_text, escape_xml(title)
        ));
    }
}
```

---

## Test Plan

### Unit tests — `crates/trellis-parser/src/lib.rs`

| Test | Input | Expected |
|---|---|---|
| `title_parsed_from_frontmatter` | `05_complex_ecommerce_checkout.mmd` | `graph.title == Some("E-Commerce Checkout Flow")` |
| `no_frontmatter_title_is_none` | `"graph TB\n A --> B\n"` | `graph.title == None` |
| `frontmatter_no_title_key` | `"---\nfoo: bar\n---\nflowchart TD\n"` | `graph.title == None` |
| `empty_title_value_is_none` | `"---\ntitle:\n---\nflowchart TD\n"` | `graph.title == None` |
| `frontmatter_with_extra_keys` | `"---\ntitle: Foo\nconfig: {}\n---\ngraph LR\n"` | `graph.title == Some("Foo")` |
| `diagram_body_parsed_correctly` | ecommerce fixture | `graph.nodes.len() == 26`, `graph.edges.len() >= 30` |
| `no_frontmatter_body_unchanged` | simple graph | parse succeeds, node count correct |

### Unit tests — `crates/trellis-core/src/config.rs`

| Test | Input | Expected |
|---|---|---|
| `show_title_defaults_true` | `serde_json::from_str("{}")` | `config.show_title == true` |
| `show_title_parses_false` | `r#"{"show_title": false}"#` | `config.show_title == false` |

### Unit tests — `crates/trellis-core/src/theme.rs`

Extend existing `all_six_themes_resolve` test to assert `!theme.title_text.is_empty()` for each theme.

### Integration / smoke test

```bash
# Title visible (default)
cargo run -p trellis-cli -- render tests/mermaid-examples/flowchart/05_complex_ecommerce_checkout.mmd -o /tmp/title_on.svg
# Title suppressed via config
cargo run -p trellis-cli -- render tests/mermaid-examples/flowchart/05_complex_ecommerce_checkout.mmd \
  --config '{"show_title":false}' -o /tmp/title_off.svg
```

Verify:
- `title_on.svg` — contains `<title>E-Commerce Checkout Flow</title>` AND visible `<text>` caption.
- `title_off.svg` — contains `<title>E-Commerce Checkout Flow</title>` but NO `<text>` caption; viewBox height matches `title_on.svg` minus `TITLE_HEIGHT`.

### Regression

```bash
cargo test --workspace
```

All 410 existing tests must pass unchanged.

---

## Non-Goals (deferred)

- YAML `config:` key support.
- Multi-line title values.
- `%%` comment-style directives (`%% title:` syntax in older Mermaid) — add later in tokenizer.
- Per-diagram-type title positioning overrides.
- Title in PNG output — works automatically via SVG→PNG pipeline once SVG is correct.

---

## File Change Summary

| File | Type of change |
|---|---|
| `crates/trellis-parser/src/ast.rs` | Add `title: Option<String>` to `Graph` |
| `crates/trellis-parser/src/lib.rs` | Add `extract_frontmatter()`, `parse_title_from_frontmatter()`; thread title through `parse()` |
| `crates/trellis-core/src/theme.rs` | Add `title_text: &'static str` to `Theme`; set values for all 7 themes |
| `crates/trellis-core/src/config.rs` | Add `show_title: bool` (default `true`); update `Default` impl and `configuration_factory` |
| `crates/trellis-core/src/render/svg.rs` | Add `escape_xml()`, `TITLE_*` constants, viewBox expansion, `<title>` element, visible caption |
