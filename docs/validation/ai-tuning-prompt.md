# AI Agent Tuning Prompt Template

Use this template when feeding structured feedback to an AI agent to improve
Trellis routing, port assignment, and node placement algorithms.

---

## Context

Trellis is a Mermaid diagram renderer that uses A\* maze routing on a grid.
Its quality knobs live in two places:

| Location | Field | Effect |
|---|---|---|
| `TrellisConfig` (`trellis-core/src/config.rs`) | `routing_costs.bend_cost` | Penalty per bend in A\* path cost |
| `TrellisConfig` | `routing_costs.crossing_cost` | Penalty for crossing an occupied cell |
| `TrellisConfig` | `port_assignment` | Strategy: `Default` / `Barycenter` / `Median` / `CrossingGreedy` |
| `TrellisConfig` | `cell_size` | Grid resolution (larger = more routing room) |
| Iterative port refinement (`ports/iterative.rs`) | `max_rounds` | Number of crossing-reduction rounds |

Quality flags emitted by `trellis-validate`:

| Flag | Meaning |
|---|---|
| `high_detour` | Routed path is ≥ 2× the Manhattan distance |
| `excessive_bends` | Path has ≥ 4 direction changes |
| `avoidable_crossing` | Edge crosses another edge's cell |

---

## Inputs to provide to the AI agent

### 1 — Diagram source (`.mmd`)

```
flowchart TD
    A[Start] --> B[Process]
    B --> C{Decision}
    C -- yes --> D[End]
    C -- no  --> B
```

### 2 — Current `TrellisConfig` (TOML)

```toml
cell_size        = 10
corner_radius    = 8.0
show_edge_labels = true
render_crossings = false
port_assignment  = "Default"

[routing_costs]
bend_cost     = 2.0
crossing_cost = 10.0
```

### 3 — Quality report (`{fixture}.json`)

Produced by `trellis evaluate <fixture>.mmd -o ./reports/`.

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
      "path_cells": [[3,4],[3,5],[4,5],[5,5],[5,6]],
      "quality_score": 0.42,
      "flags": ["high_detour", "avoidable_crossing"]
    }
  ],
  "global_metrics": {
    "total_edges": 4,
    "routed_edges": 4,
    "failed_edges": 0,
    "total_crossings": 1,
    "total_bends": 9,
    "avg_quality_score": 0.61,
    "avg_detour_factor": 1.8,
    "flagged_edges": 2
  }
}
```

### 4 — Human feedback (`feedback.json`)

Produced by `trellis generate-review ./reports/` → open `review.html` in a browser
→ annotate edges → click **Download feedback.json**.

```json
{
  "b05.mmd": {
    "edges": {
      "A-->B": {
        "improvable": true,
        "note": "unnecessary detour around D; should go straight south"
      },
      "C-->E": {
        "improvable": true,
        "note": "could use south port instead of east to avoid crossing B-->D"
      }
    }
  }
}
```

---

## Prompt template

Copy the block below, fill in the `<…>` placeholders with your actual files,
and send it to the AI agent.

---

```
You are a routing algorithm tuning assistant for Trellis, a Mermaid diagram
renderer that uses A* maze routing on a grid.

## Diagram source
<paste contents of fixture.mmd>

## Current TrellisConfig (TOML)
<paste current config.toml or the relevant fields>

## Quality report
<paste contents of fixture.json>

## Human feedback
<paste contents of feedback.json>

## Your task

1. Analyse the flagged edges using the quality report and the human notes.
2. Identify the most likely algorithmic root cause for each problem:
   - high_detour  →  routing_costs.bend_cost may be too low relative to
                     crossing_cost, causing A* to prefer indirect paths to
                     avoid crossings; or port placement forces an indirect
                     exit direction.
   - excessive_bends  →  port assignment may be placing the source/target
                         port on the wrong side (e.g. North when South would
                         give a straighter path).
   - avoidable_crossing  →  routing priority order or crossing_cost may need
                            adjustment; or iterative port refinement is not
                            running enough rounds.
3. Propose a concrete change to TrellisConfig or algorithm parameters.
   Present it as a diff against the current TOML.
4. For each proposed change, predict its effect on the flagged edges and
   estimate whether it could harm currently-good edges (quality_score ≥ 0.8).
5. If multiple changes compete, rank them by expected impact / risk.

Respond with:
- A short diagnosis paragraph per flagged edge.
- A proposed config diff (TOML format).
- A brief explanation of the predicted before/after quality scores.
```

---

## Interpreting the agent's proposals

After the agent proposes a config change:

1. Apply the diff to your `config.toml`.
2. Re-run `trellis evaluate-batch ./fixtures/ -o ./reports-v2/ --annotated`.
3. Compare quality scores:
   ```
   trellis evaluate-batch ./fixtures/ -o ./reports-v2/ --compare-strategies
   ```
4. Open `trellis generate-review ./reports-v2/` and review the delta visually.
5. If the average quality improves and no good edges regress, commit the config.

Repeat with updated `feedback.json` from the new review session.

---

## Strategy selection guide

When `--compare-strategies` shows one port-assignment strategy consistently
wins for a given graph topology, lock it in via `port_assignment` in config:

| Topology | Recommended strategy |
|---|---|
| Dense fan-out from one node | `CrossingGreedy` |
| Bipartite / layered graphs  | `Barycenter` |
| Sparse / few crossings      | `Default` |
| Long chains with outliers   | `Median` |

For mixed topologies, `Barycenter` is the safe default.
