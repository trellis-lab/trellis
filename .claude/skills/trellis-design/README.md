# Trellis Design System

## Overview

**Trellis** is a fast, overlap-free Mermaid-compatible diagram renderer built in Rust. It uses VLSI maze routing (A* pathfinding) to produce orthogonal edge layouts without overlapping edges — a significant improvement over the aging Dagre layout engine used by standard Mermaid.

**Website:** https://trellislabs.net  
**GitHub:** https://github.com/trellis-mermaid/trellis  
**License:** MIT OR Apache-2.0

### Products / Surfaces

| Surface | Description |
|---|---|
| **CLI** | `trellis render input.mmd -o output.svg` — cross-platform binary |
| **WASM Library** | `trellis-wasm` — browser-side rendering via `render()` and `render_with_metrics()` |
| **VS Code Extension** | Live preview of `.mmd` files in the editor |
| **IntelliJ Plugin** | Live preview in JetBrains IDEs |
| **Pandoc Filter** | Renders Mermaid blocks in Markdown → PDF/HTML |
| **Web App (planned)** | Mermaid editor + landing page using trellis-wasm |

### Sources Used

- **Codebase:** `trellis/` — mounted via File System Access API (path prefix: `trellis/`)
- **Uploaded images:** `uploads/logo.png`, `uploads/logo-text.png`
- **Codebase README:** `trellis/README.md`
- **Tech spec:** `trellis/docs/trellis-spec.md`
- **WASM API:** `trellis/crates/trellis-wasm/src/lib.rs`
- **Placeholder web page:** `trellis/public/index.html`

---

## CONTENT FUNDAMENTALS

### Tone & Voice

- **Technical but clear.** Trellis targets developers and architects who write Diagrams-as-Code (DaaC). Copy is precise, benefit-led, and never fluffy.
- **First-person plural absent.** Prefers declarative statements: "Renders overlap-free diagrams" not "We render..."
- **Lowercase product names in prose:** "trellis render", "trellis-wasm", "trellis-validate" (follow Rust crate naming conventions in code contexts).
- **Emoji: never used.** Iconography is handled via `→` arrows (ASCII/Unicode) in terminal/CLI contexts only.
- **Casing:** Sentence case for headings. ALL CAPS only for badges/status labels. Title Case for table headers.
- **Numbers:** Always concrete. "50+ node graphs", "A* pathfinding", "0–1 quality score" — avoids vague qualifiers like "many" or "fast" without backing.
- **Features described as outcomes:** "Zero JavaScript required", "Overlap-free by construction", "Drop-in Mermaid syntax compatibility."
- **CLI examples are canonical marketing.** Code blocks in README are treated as product showcases — clean, idiomatic, no unnecessary flags.

### Copy Examples (from source)

- "A fast Mermaid diagram renderer built in Rust, using VLSI maze routing (A* pathfinding) to produce overlap-free orthogonal edge layouts."
- "Pixel-perfect orthogonal edges. Zero JavaScript required."
- "Drop-in Mermaid syntax compatibility."
- "Render Markdown documents containing mermaid code blocks to PDF or HTML."

---

## VISUAL FOUNDATIONS

### Color System

Blueprint drawing aesthetic — technical drafting rooms, Cyanotype prints, architectural drawings.

| Token | Hex | Use |
|---|---|---|
| `--prussian-blue` | `#003153` | Darkest bg, hero sections |
| `--blueprint-blue` | `#1B3F6E` | Primary surface, nav, cards |
| `--cyanotype-azure` | `#4A8AB5` | Accent, links, highlights |
| `--blueprint-white` | `#E8F4FB` | Foreground text, borders on dark |
| `--ink-white` | `#FFFFFF` | Pure white for contrast |
| `--grid-line` | `rgba(72,140,181,0.12)` | Background grid pattern |

### Typography

- **Display / Headings:** `Courier Prime` or `JetBrains Mono` — monospace slab serif, echoing technical documentation and blueprint annotations. Google Fonts substitution used (see flag below).
- **Body:** `Inter` — clean, readable at small sizes for technical prose.
- **Code:** `JetBrains Mono` — for Mermaid source, CLI commands, JSON.
- **Weight scale:** 400 (body), 500 (label), 700 (heading/display).
- **Letter-spacing:** Tight (`-0.02em`) on large headings. `0.1em` on uppercase labels/badges.

> ⚑ **Font substitution:** The original product does not specify a font. `Courier Prime` and `JetBrains Mono` are Google Fonts substitutes matching the monospace slab aesthetic inferred from the logo's "Trellis" wordmark. Please provide brand font files if different.

### Logo

- **Mark:** A stylized "T" formed by two orthogonal arrows — one routing left-then-up, one routing right. Visually references the VLSI maze routing concept.
- **Wordmark:** "Trellis" in a slab serif monospace face, consistent with technical/blueprint aesthetic.
- **Variants:** `logo.png` (mark only), `logo-text.png` (mark + wordmark), SVG variants: narrow, narrow-with-text, wide.
- **Usage:** Logo is always black on white/light, or white on dark blueprint backgrounds. No color-tinted logos.

### Background Patterns

- **Blueprint grid:** `repeating-linear-gradient` creating a fine square grid at ~40px intervals, color `rgba(72,140,181,0.12)` on dark, `rgba(0,49,83,0.06)` on light. This is the signature motif.
- **No photography, no illustrations.** Diagram SVG outputs act as the primary visual content.

### Spacing & Layout

- Base unit: `8px`. Scale: 4, 8, 12, 16, 24, 32, 48, 64, 96.
- Border radius: `4px` (inputs/code), `8px` (cards), `12px` (panels), `0` (strict blueprint mode).
- Max content width: `1200px`. Editor layout: split `50/50` or `40/60` (code | preview).

### Shadows & Elevation

- No drop shadows in blueprint mode — elevation is communicated by border weight (`1px` vs `2px`) and background color steps.
- `inset 0 0 0 1px rgba(72,140,181,0.3)` — subtle inner ring on active/focused states.

### Animation

- Minimal. Fade transitions only: `opacity 150ms ease`. No bounces, no springs.
- Editor preview re-render: instant swap (no animation) to feel like a real compiler.
- Hover states: lighter background (`--cyanotype-azure` at 10% opacity overlay).
- Press states: slight darken (`--prussian-blue` deepened 10%).

### Borders & Dividers

- Primary border: `1px solid rgba(72,140,181,0.25)` on dark surfaces.
- Active/selected: `1px solid #4A8AB5`.
- Dividers: `1px solid rgba(232,244,251,0.1)`.

### Iconography

See **ICONOGRAPHY** section below.

### Cards

- Background: `--blueprint-blue` (`#1B3F6E`).
- Border: `1px solid rgba(72,140,181,0.25)`.
- Border radius: `8px`.
- No shadow. Hover: border color lifts to `#4A8AB5`.

### Corner Radii

| Element | Radius |
|---|---|
| Buttons | `6px` |
| Inputs / code blocks | `4px` |
| Cards / panels | `8px` |
| Modal dialogs | `12px` |
| Badges / pills | `999px` |

---

## ICONOGRAPHY

- **No icon font or sprite sheet found** in the codebase. The CLI uses `→` arrows as ASCII icons in feature lists.
- **CDN:** Lucide Icons (stroke-based, `1.5px` weight, `16–20px` size) is the closest match to the product's minimalist technical aesthetic. Used via CDN: `https://unpkg.com/lucide@latest`.
- **Usage:** Icons appear in feature lists, editor toolbar buttons, and status indicators. Always `16px` or `20px`, stroke color inherits from text.
- **No emoji.** Unicode arrows (`→`, `←`, `↑`) may appear in CLI/terminal contexts only.
- **SVG diagrams** generated by Trellis itself are the primary "imagery" of the brand — showcased in hero sections and feature demos.

---

## FILE INDEX

```
README.md                          ← You are here
SKILL.md                           ← Agent skill definition
colors_and_type.css                ← CSS custom properties (colors, type, spacing)
assets/
  logo.png                         ← Mark only (black on transparent)
  logo-text.png                    ← Mark + wordmark (black on transparent)
  trellis-logo-narrow.svg          ← SVG mark
  trellis-logo-narrow-text.svg     ← SVG mark + wordmark
  trellis-logo-narrow-text_bg.svg  ← SVG with background
  trellis-logo-wide.svg            ← Wide layout SVG
preview/
  colors-palette.html              ← Brand color swatches
  colors-semantic.html             ← Semantic color tokens
  type-scale.html                  ← Type scale specimen
  type-code.html                   ← Code / mono type specimen
  spacing.html                     ← Spacing tokens
  components-buttons.html          ← Button variants
  components-badges.html           ← Badge / pill variants
  components-inputs.html           ← Input fields
  components-cards.html            ← Card variants
  logo-variants.html               ← Logo & wordmark variants
ui_kits/
  trellis-app/
    index.html                     ← Interactive Mermaid editor + landing page
    Header.jsx                     ← Top nav component
    Editor.jsx                     ← Split-pane code editor
    Landing.jsx                    ← Marketing sections
    Footer.jsx                     ← Footer component
```
