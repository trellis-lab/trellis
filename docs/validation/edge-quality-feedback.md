# Edge Quality Feedback System

Goal: render all fixture diagrams, identify edges that could be routed better for readability, and feed structured feedback to an AI agent that fine-tunes routing, port assignment, and node placement algorithms.

---

## Option A: Annotated SVG Export (Lightweight)

Add a `--diagnostic` flag to the CLI that renders SVGs with per-edge quality annotations:

- **Color-code edges** by quality score (green = good, yellow = suboptimal, red = bad) based on detour factor, bend count, and crossing involvement.
- **Add tooltip `<title>` elements** on each edge path with metrics: detour factor, bends, crossings, manhattan vs actual length.
- **Edge IDs in SVG** (`data-edge-id="A-->B"`) so they are selectable and searchable.

The reviewer opens the SVG in a browser, visually inspects, and the colour coding highlights problem edges immediately. No new tooling needed.

**Pros:** Minimal code change, immediate value, no external dependencies.
**Cons:** Feedback is visual-only, no structured data for the AI agent to consume.

---

## Option B: JSON Report + Annotated SVG (Medium)

Extend Option A with a companion JSON report per fixture:

```json
{
  "fixture": "b05.mmd",
  "edges": [
    {
      "id": "A-->B",
      "source": "A",
      "target": "B",
      "bends": 4,
      "detour_factor": 2.3,
      "crossings": 1,
      "port_side_source": "South",
      "port_side_target": "North",
      "path_cells": [[3,4],[3,5],[4,5]],
      "quality_score": 0.42,
      "flags": ["high_detour", "avoidable_crossing"]
    }
  ],
  "global_metrics": {}
}
```

New CLI command: `trellis evaluate-batch ./fixtures/ -o ./reports/`
- Renders all fixtures.
- Produces `{name}.svg` (annotated) + `{name}.json` (structured report).
- The AI agent reads the JSON, compares across algorithm runs, and suggests parameter changes.

**Pros:** Machine-readable, composable, can diff between algorithm runs.
**Cons:** More code, quality heuristics need tuning, no human annotation yet.

---

## Option C: Interactive Review Tool (Full)

Build a web-based review UI (single HTML file + JS, no framework):

1. `trellis evaluate-batch` produces annotated SVGs + JSON (Option B).
2. The HTML tool loads all fixtures side-by-side.
3. Reviewer clicks an edge to toggle it as "improvable" and writes to `feedback.json`.
4. Each flagged edge gets a free-text note (e.g. "should route left of node C instead").
5. The feedback JSON is the input for the AI tuning agent.

```json
{
  "b05.mmd": {
    "edges": {
      "A-->B": { "improvable": true, "note": "unnecessary detour around D" },
      "C-->E": { "improvable": true, "note": "could use south port instead" }
    }
  }
}
```

AI agent receives: fixture `.mmd` + `feedback.json` + current algorithm config, proposes config/algorithm changes, re-renders, human reviews delta.

**Pros:** Full feedback loop, structured + human-readable, iterative.
**Cons:** Most development effort, needs a simple web UI.

---

## Option D: Diff-Based Comparison (Algorithm A/B Testing)

Focus on comparing outputs across algorithm configurations:

1. `trellis evaluate-batch` renders all fixtures with every port assignment strategy (Default, Barycenter, Median, CrossingGreedy).
2. Produces a comparison matrix: `{fixture}_{algorithm}.svg` + a summary CSV.
3. An overlay SVG highlights edges that differ between strategies (same edge, different routing).
4. Human picks the best version per fixture to build a "ground truth" set.

This turns the problem into supervised learning: the AI agent gets (fixture, best_algorithm) pairs and can learn which algorithm suits which graph topology.

**Pros:** Directly actionable for algorithm selection, ground-truth dataset.
**Cons:** Only compares existing algorithms, does not capture "none of these are good".

---

## Recommended Approach

Start with **Option B**, evolve to **C**:

1. Option B gives immediate value: batch-render, per-edge quality scores, and JSON for the AI agent.
2. The JSON format from B becomes the data model for C's interactive UI.
3. Option D's A/B comparison can be added as a mode on top of B (`--compare-strategies`).

### Implementation Order

1. **`diagnostics` feature flag in `trellis-core`** — gate the display-only post-routing stats loop and `grid.utilization()` behind `#[cfg(feature = "diagnostics")]`. Update `trellis-wasm` to compile with `default-features = false`. This keeps the production and WASM builds lean before adding any new code.

2. **Create `crates/trellis-validate`** — add the crate to the workspace with dependencies on `trellis-core`, `trellis-parser`, and `serde_json`. No logic yet, just the skeleton.

3. **Quality scoring (`scoring.rs`)** — implement `score_edge` and `score_all_edges` using detour factor, bend count, and crossing involvement to produce a 0–1 quality score and human-readable flags (`high_detour`, `avoidable_crossing`, `excessive_bends`).

4. **JSON report (`report.rs`)** — implement `generate_report` producing the per-edge `DiagramReport`. The JSON schema defined here becomes the stable contract for the AI agent.

5. **Annotated SVG (`annotated_svg.rs`)** — colour-code edge paths by quality score, attach `data-edge-id` and `<title>` tooltips to each `<path>` element.

6. **CLI `evaluate` and `evaluate-batch` commands** — wire up `trellis-validate` in `trellis-cli`. Single-file and batch modes, both producing `{name}.svg` + `{name}.json`. Add `--compare-strategies` flag (Option D) as a batch mode variant.

7. **Interactive HTML review tool (`review_html.rs`)** — generate a self-contained HTML file that loads all fixture SVGs, makes edges clickable, and exports `feedback.json` via browser download (localStorage → download button).

8. **AI agent prompt template** — write `docs/validation/ai-tuning-prompt.md` describing how to feed `feedback.json` + fixture `.mmd` + current `TrellisConfig` to an AI agent and interpret the proposed changes.

---

## Architecture Conclusions

### Separate `trellis-validate` crate — Yes, for the analysis layer

The data collection already belongs in `trellis-core` because the algorithms need it: `crossings` from
`grid.count_crossings()` drives the iterative port refinement early-exit in `ports/iterative.rs`. That
cannot move.

What does not belong in `trellis-core` is the *analysis layer* built on top of that data:

- Quality scoring (detour factor thresholds, bend-count classification, 0–1 quality score)
- Per-edge JSON serialisation for the report format
- Annotated SVG generation (diagnostic colour overlays, `data-edge-id` attributes)
- The `feedback.json` schema and reader
- HTML review tool generation

A new crate `crates/trellis-validate` with a clean dependency direction:

```
trellis-validate → trellis-core + trellis-parser
trellis-cli      → trellis-validate  (only for the `evaluate` command)
trellis-wasm     → trellis-core only  (validate never included in WASM)
```

This lets `trellis-validate` pull in heavier dependencies freely (e.g. `serde_json`, templating for HTML)
without bloating `trellis-core` or the WASM binary size.

### Switching off edge statistics — partially possible

Three categories of stats, with different constraints:

| Stat | Computation cost | Used by algorithm? | Can be gated? |
|---|---|---|---|
| `total_bends`, `total_path_length`, `total_routing_cost` | Free `+=` per edge | Yes — iterative refinement scoring | No |
| `grid.count_crossings()` | O(grid cells) scan | **Yes** — iterative refinement early-exit | No |
| `grid.utilization()` | O(grid cells) scan | No — display only | **Yes** |
| `max_path_length`, `max_bends_per_edge`, `sum_manhattan_distance` | O(edges) post-loop | No — display only | **Yes** |
| `avg_detour_factor`, `avg_routing_cost` | Division only | No — display only | **Yes** |

The existing `print_metrics: bool` config flag already gates the *display*, but all values are always
*computed*. The cleanest improvement is a `diagnostics` Cargo feature that gates the post-routing stats
loop and the two O(grid cells) display-only scans:

```toml
# trellis-core/Cargo.toml
[features]
default = ["diagnostics"]
diagnostics = []
```

`trellis-wasm` would compile with `default-features = false`, the same pattern already used for the
`png` feature. This is primarily a WASM bundle size and speed optimisation rather than a correctness
concern — the stats computation is genuinely cheap for typical diagram sizes.
