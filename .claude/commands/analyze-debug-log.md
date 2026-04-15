# analyze-debug-log

Read the debug log at $ARGUMENTS (JSON file produced by `trellis --debug-log`).
Parse all phases. Use the structured log as the primary source for all answers.

## SVG ↔ JSON ID mapping

When the SVG was rendered with `--features debug-log`, every element carries a
data attribute that maps directly to a log entry:

| SVG attribute | JSON path | Meaning |
|---|---|---|
| `data-edge-index="N"` | `phases.routing.edges[i].edge_index == N` | Routed edge N |
| `data-edge-index="N" data-fallback="true"` | edge N has no entry in `routing.edges` (unrouted) | Fallback straight line |
| `data-node-id="X"` | `phases.placement.node_positions[i].id == X` | Node X |

Use these when the user points at an SVG element ("this edge", "that node"):
- Identify the `data-edge-index` or `data-node-id` from the SVG.
- Look up the corresponding log entry by matching `edge_index` or node `id`.
- Cite both the SVG attribute and the JSON path in your answer.

If both an SVG file and the JSON log are available, read both and cross-reference
to give concrete answers (e.g. "edge index 3 in the SVG corresponds to
`routing.edges[2]` with 4 bends and path length 18").

## If a question is provided

Answer using data from the log. Cite the phase name and edge index when referencing
decisions. Examples of answerable questions:

- "Why was edge 3 routed around node B?" → check `routing.edges[i].final_path` where
  `edge_index == 3`; cross-reference other edges' paths for grid obstacles.
- "Which ports were considered for node X?" → check `ports.assignments` for `node_id == X`;
  `candidates` list (populated by P4 instrumentation).
- "Why did the quality rerouter skip edge 5?" → compare `quality_reroute.threshold_used`
  vs `routing.edges[i].bend_count` where `edge_index == 5`.
- "Did deadlock trigger?" → check `deadlock.triggered` + `deadlock.resolution_method`.
- "How many crossings were found and which style rendered them?" →
  check `crossings.style` + `crossings.crossings[]`.

## If no question is provided

Print a summary:

```
Diagram type : <diagram_type>
Cell size    : <cell_size>
Nodes        : <placement.node_positions.len()>
Edges        : <routing.edges.len()>
Grid         : <grid.cols> × <grid.rows>  (offset <grid.offset_x>, <grid.offset_y>)

Routing
  Routed      : <count of edges with final_path.len() > 0>
  Failed      : <edges with empty final_path>
  Total bends : <sum of routing.edges[].bend_count>
  Avg bends   : <mean>
  Max bends   : <max edge bend_count, edge index>

Quality reroute
  Threshold   : <quality_reroute.threshold_source>
  Rerouted    : <quality_reroute.rerouted_edges.len()>

Crossing reroute
  Enabled     : <crossing_reroute.enabled>
  Edges moved : <crossing_reroute.edges_rerouted>

Crossings
  Style       : <crossings.style>
  Count       : <crossings.crossings.len()>

Deadlock      : <deadlock.triggered ? "YES – " + deadlock.resolution_method : "no">
Labels        : <labels.labels.len()>
Port strategy : <ports.strategy>
Straight pins : <ports.straight_edge_prepass.len()>
```

## Report generation

If the user asks to "generate a report" or passes a second argument as an output path,
write the summary above plus any per-edge anomalies (bend count > 2× average, failed
routes, crossing edges) to a Markdown file at that path.
