# Debug Log — Implementation Strategy

## Overview

Add structured JSON debug logging to the trellis pipeline, gated behind a Cargo
feature flag. Log covers all pipeline phases with decision rationale. Exposed
via hidden CLI arg. Analysed via a project-level Claude Code skill.

---

## 1. Cargo Feature Flag

File: `crates/trellis-core/Cargo.toml`

```toml
[features]
default = ["png", "diagnostics"]
png     = ["dep:resvg"]
debug-log = ["dep:serde_json"]   # new — NOT in default
```

- `debug-log` excluded from `default` → never ships in release builds.
- `trellis-cli/Cargo.toml` adds `trellis-core = { ..., features = ["debug-log"] }`
  only when built with `--features debug-log`.
- All debug-log code wrapped in `#[cfg(feature = "debug-log")]`.

---

## 2. Data Model

New module: `crates/trellis-core/src/debug/mod.rs`

### Top-level struct

```rust
pub struct DebugLog {
    pub version: u8,                   // schema version, currently 1
    pub input_file: Option<String>,
    pub diagram_type: String,
    pub cell_size: usize,
    pub phases: PipelinePhases,
}
```

### Phase structs

```rust
pub struct PipelinePhases {
    pub placement:     PlacementPhase,
    pub grid:          GridPhase,
    pub ports:         PortsPhase,
    pub routing:       RoutingPhase,
    pub quality_reroute: QualityReroutePhase,
    pub port_swap:     PortSwapPhase,
    pub crossing_reroute: CrossingReroutePhase,
    pub deadlock:      DeadlockPhase,
    pub labels:        LabelsPhase,
    pub crossings:     CrossingsPhase,
}
```

#### PlacementPhase
```rust
pub struct PlacementPhase {
    pub algorithm: String,         // "sugiyama" | "force-directed" | "row-flow" | "class"
    pub iterations: Option<u32>,   // force-directed only
    pub node_positions: Vec<NodePos>,
}
pub struct NodePos { pub id: String, pub x: f64, pub y: f64 }
```

#### GridPhase
```rust
pub struct GridPhase {
    pub cols: usize,
    pub rows: usize,
    pub offset_x: i32,
    pub offset_y: i32,
}
```

#### PortsPhase
```rust
pub struct PortsPhase {
    pub strategy: String,
    pub straight_edge_prepass: Vec<StraightEdgePin>,
    pub assignments: Vec<NodePortLog>,
}
pub struct StraightEdgePin {
    pub edge_index: usize,
    pub reason: String,
}
pub struct NodePortLog {
    pub node_id: String,
    pub edges: Vec<EdgePortLog>,
}
pub struct EdgePortLog {
    pub edge_index: usize,
    pub edge_label: Option<String>,
    pub candidates: Vec<PortCandidate>,
    pub selected: PortAssignment,
    pub rejection_reasons: Vec<RejectionNote>,
}
pub struct PortCandidate {
    pub side: String,            // "Top" | "Right" | "Bottom" | "Left"
    pub connector: (i32, i32),
    pub score: f64,
    pub rank: usize,
}
pub struct PortAssignment {
    pub side: String,
    pub connector: (i32, i32),
}
pub struct RejectionNote {
    pub connector: (i32, i32),
    pub reason: String,          // "occupied" | "below-score" | "congestion"
}
```

#### RoutingPhase
```rust
pub struct RoutingPhase {
    pub edges: Vec<EdgeRoutingLog>,
}
pub struct EdgeRoutingLog {
    pub edge_index: usize,
    pub edge_label: Option<String>,
    pub priority_score: f64,
    pub attempts: Vec<RoutingAttempt>,
    pub selected_attempt: usize,   // index into attempts
    pub final_path: Vec<(i32, i32)>,
    pub bend_count: usize,
    pub path_length: usize,
}
pub struct RoutingAttempt {
    pub source_port: (i32, i32),
    pub target_port: (i32, i32),
    pub result: RoutingAttemptResult,
    pub cost: Option<f64>,
    pub bend_count: Option<usize>,
    pub rejection_reason: Option<String>,
    // "no-path" | "occupied-cells: N" | "cost-above-threshold"
}
pub enum RoutingAttemptResult { Success, Failed }
```

#### QualityReroutePhase
```rust
pub struct QualityReroutePhase {
    pub threshold_used: usize,
    pub threshold_source: String,   // "disabled" | "fixed(N)" | "auto(median=M)"
    pub rerouted_edges: Vec<RerouteLog>,
}
pub struct RerouteLog {
    pub edge_index: usize,
    pub original_bends: usize,
    pub best_alternative_bends: usize,
    pub outcome: String,   // "improved" | "no-improvement-kept-original"
}
```

#### PortSwapPhase
```rust
pub struct PortSwapPhase {
    pub swaps: Vec<PortSwapLog>,
}
pub struct PortSwapLog {
    pub node_id: String,
    pub side: String,
    pub edge_a: usize,
    pub edge_b: usize,
    pub bends_before: usize,
    pub bends_after: usize,
}
```

#### CrossingReroutePhase
```rust
pub struct CrossingReroutePhase {
    pub enabled: bool,
    pub edges_rerouted: usize,
    pub details: Vec<CrossingRerouteLog>,
}
pub struct CrossingRerouteLog {
    pub edge_index: usize,
    pub crossings_before: usize,
    pub crossings_after: usize,
    pub outcome: String,
}
```

#### DeadlockPhase
```rust
pub struct DeadlockPhase {
    pub triggered: bool,
    pub resolution_method: Option<String>,
    // "rip-up-reroute" | "grid-expansion" | "crossing-fallback"
    pub edges_affected: Vec<usize>,
}
```

#### LabelsPhase
```rust
pub struct LabelsPhase {
    pub labels: Vec<LabelPlacementLog>,
}
pub struct LabelPlacementLog {
    pub edge_index: usize,
    pub text: String,
    pub position: (f64, f64),
    pub collision_resolved: bool,
}
```

#### CrossingsPhase
```rust
pub struct CrossingsPhase {
    pub style: String,    // "None" | "Arc" | "Rectangular" | "Skip"
    pub crossings: Vec<CrossingLog>,
}
pub struct CrossingLog {
    pub owner_edge: usize,
    pub hopper_edge: usize,
    pub cell: (i32, i32),
    pub hop_rendered: bool,
}
```

---

## 3. Passing the Log Through the Pipeline

`DebugLog` is constructed in `run_pipeline()` when config signals debug mode,
passed by mutable reference to each phase function.

### Config change (`crates/trellis-core/src/config.rs`)

```rust
pub struct TrellisConfig {
    // ... existing fields ...
    #[cfg(feature = "debug-log")]
    pub debug_log_path: Option<std::path::PathBuf>,
}
```

### Pipeline change (`pipeline.rs`)

```rust
#[cfg(feature = "debug-log")]
let mut debug_log = config.debug_log_path.as_ref().map(|_| DebugLog::new(
    graph.diagram_type.to_string(), config.cell_size
));

// Each phase receives: Option<&mut DebugLog>
// Pattern inside phase functions:
#[cfg(feature = "debug-log")]
if let Some(log) = debug_log.as_mut() {
    log.phases.placement = /* ... */;
}

// End of run_pipeline() — write file
#[cfg(feature = "debug-log")]
if let (Some(path), Some(log)) = (&config.debug_log_path, debug_log) {
    crate::debug::writer::write_debug_log(path, &log)?;
}
```

Phase functions get an extra parameter:
```rust
fn some_phase_fn(
    ...,
    #[cfg(feature = "debug-log")] debug: Option<&mut DebugLog>,
)
```

Use `#[cfg_attr(not(feature = "debug-log"), allow(unused_variables))]` to keep
signatures clean in non-debug builds.

---

## 4. CLI Changes (`crates/trellis-cli/src/main.rs`)

Hidden arg on `Render` subcommand — not shown in `--help`:

```rust
/// Write debug log to FILE (default: <output>.debug.json)
#[arg(long, value_name = "FILE", hide = true)]
debug_log: Option<Option<PathBuf>>,
```

Default path logic (in render handler):

```rust
#[cfg(feature = "debug-log")]
let debug_log_path = match args.debug_log {
    None => None,                          // flag absent → no log
    Some(None) => Some(derive_default_debug_path(&output_path)),
    Some(Some(p)) => Some(p),
};

fn derive_default_debug_path(output: &Path) -> PathBuf {
    let stem = output.file_stem().unwrap_or_default();
    let name = format!("{}.debug.json", stem.to_string_lossy());
    output.with_file_name(name)
}
```

`--debug-log` with no value → default path. `--debug-log path/to/file.json` →
custom path.

CLI must be compiled with `--features debug-log` to expose the flag at all.

---

## 5. File Structure

```
crates/trellis-core/src/
  debug/
    mod.rs          ← DebugLog struct + all phase structs + DebugLog::new()
    writer.rs       ← write_debug_log(path, log) → serde_json::to_writer_pretty
```

Entire `debug/` module gated:

```rust
// crates/trellis-core/src/lib.rs
#[cfg(feature = "debug-log")]
pub mod debug;
```

---

## 6. Instrumentation Sites

| Phase | File(s) to modify | What to capture |
|---|---|---|
| Placement | `placement/mod.rs`, `placement/force_directed.rs` | algorithm name, iterations, node positions after |
| Grid | `grid/mod.rs` | cols, rows, offsets |
| Straight-edge prepass | `ports/mod.rs` | pinned edge indices + reason |
| Port assignment | `ports/assignment.rs`, `ports/barycenter.rs`, `ports/median.rs`, `ports/crossing_greedy.rs` | per-edge candidate list + scores + selection |
| Routing | `routing/mod.rs`, `routing/astar.rs` | per-edge: priority, attempts, cost, bends, rejection |
| Quality reroute | `routing/quality_reroute.rs` | threshold, rerouted edges, outcome |
| Port swap | `routing/port_swap.rs` | swapped pairs + bend delta |
| Crossing reroute | `routing/crossing_reroute.rs` | edges touched, crossing counts |
| Deadlock | `deadlock/mod.rs` | triggered flag, method, affected edges |
| Labels | `labels/mod.rs` | position, collision flag |
| Crossings | `routing/commit.rs` (`reconcile_crossings`) | crossing pairs, cell, hop flag |

---

## 7. Claude Code Skill

File: `crates/trellis-core/.claude/skills/analyze-debug-log.md`  
Or at project level: `.claude/commands/analyze-debug-log.md`

**Recommended location**: `.claude/commands/analyze-debug-log.md`  
(Consistent with existing `check-analysis.md`, `validate-rust.md` pattern.)

The skill:
1. Accepts a path argument (file path to `.debug.json`).
2. Reads and parses the JSON.
3. Answers user questions about routing decisions, port selections, cost scores,
   rejection reasons — using the structured log as primary source.
4. Can surface: why edge X was routed around Y, which ports were considered for
   node Z and why option A beat B, why deadlock triggered, which edges had
   excessive bends and how rerouting resolved them.
5. Generate a report of the findings. (file path to `.report.md`)

Skill template content (brief):
```markdown
# analyze-debug-log

Read the debug log at $ARGUMENTS (JSON file produced by trellis --debug-log).
Parse all phases. Answer the user's question about pipeline decisions using
data from the log. Cite phase name and edge index when referencing decisions.
If no question given, print a summary: diagram type, node count, edge count,
routing success rate, bend stats, crossing count, deadlock triggered y/n.
```

---

## 8. README Update

Add to the `render` command reference section (hidden section or dev appendix):

```
  --debug-log [FILE]    Write structured pipeline debug log to FILE.
                        Default: <output>.debug.json (same directory as output).
                        Requires build with --features debug-log.
```

---

## 9. Implementation Phases

| Phase | Scope | Notes |
|---|---|---|
| **P1** | Feature flag + data model + writer | No instrumentation yet; confirms compile |
| **P2** | Config plumbing + CLI arg + default path logic | Smoke-test with empty log |
| **P3** | Placement + Grid instrumentation | Low-risk, read-only capture |
| **P4** | Port assignment instrumentation | Most complex; touch 4 assigner files |
| **P5** | Routing instrumentation (A* attempts + costs) | `astar.rs` returns attempt list |
| **P6** | Reroute phases (quality, port-swap, crossing) | Augment existing reroute fns |
| **P7** | Deadlock + Labels + Crossings | Finishes full pipeline coverage |
| **P8** | Claude Code skill | Write + smoke-test with real log output |
| **P9** | README update | Final polish |

Each phase: `cargo test --workspace` + `cargo clippy --workspace` before next.

---

## 10. Constraints & Guard Rails

- No `debug-log` code reaches stable release (`default` feature never includes it).
- `--help` output never mentions `--debug-log` (`hide = true` in clap).
- Log write failure is non-fatal: log warning to stderr, continue rendering.
- JSON uses `serde_json::to_writer_pretty` for human + AI readability.
- `DebugLog` derives `serde::Serialize` only (no Deserialize needed).
- Passing `Option<&mut DebugLog>` keeps zero overhead when feature is compiled
  in but no `--debug-log` flag given (Option is None, branches skip).
