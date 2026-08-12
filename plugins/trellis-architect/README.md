# Trellis Architect (`ta`)

Claude Code plugin: skills guiding a Software Architect from an initial idea
to a validated, PDF-ready architecture design document, using the Trellis
portfolio (CLI, VS Code extension, MCP server, Pandoc Docker image) for
diagram rendering, review, and publishing.

```
initialize → refine expectations → define requirements → collect design ideas
  → select design → risk analysis → final documentation
```

Every phase produces a markdown document with embedded Mermaid diagrams,
reviewed in the Trellis VS Code extension, gated by explicit user approval
before the next phase starts. See `../../docs/ai-plugin/plan.md` and
`definition.md` in the same doc tree for the full design rationale.

## Install

```
/plugin marketplace add <github-owner>/trellis
/plugin install trellis-architect@trellis
```

(Local development: `claude --plugin-dir /path/to/trellis/plugins/trellis-architect`.)

Installing also registers the bundled Trellis MCP server (`.claude-plugin/
plugin.json`'s `mcpServers` block) — set `TRELLIS_BIN` and `TRELLIS_KEY` in
your shell environment beforehand if you want CLI path detection and licensed
formats (SVG/HTML/Draw.io) to work out of the box. The bundled MCP entry
launches `../../mcp/server.py` relative to the plugin's install location: it
assumes the plugin is installed from a checkout of this repo (Option A —
marketplace lives at the `trellis` repo root), not from a plugin fetched
standalone via `git-subdir`. If you install trellis-architect from elsewhere,
configure the Trellis MCP server separately per `../../docs/mcp.md` instead of
relying on this bundling.

## Skills

Invocation form is `trellis-architect:<skill-name>` (Claude Code namespaces
skills by plugin name; there is no separate alias mechanism in the plugin
manifest, so `ta:` below is documentation shorthand, not a second registered
prefix — always use the full `trellis-architect:` form).

| Skill | Purpose |
|---|---|
| `ta:initialize` | Scaffold a new workspace, detect the Trellis environment |
| `ta:refine-expectations` | Interview → `01-expectations/expectations.md` + system context diagram |
| `ta:define-requirements` | FR/NFR → `02-requirements/requirements.md` |
| `ta:design-ideas` | 2–4 candidate architectures → `03-design-ideas/design-ideas.md` |
| `ta:select-design` | Weighted comparison / hybrid → `04-selected-design/selected-design.md` + ADRs |
| `ta:risk-analysis` | SWIFT review → `05-risk-analysis/risk-analysis.md`, feeds mitigations back into the design |
| `ta:documentation` | Assemble `06-final/architecture.md`, render PDF via `trellis-pandoc` |
| `ta:export` | Render any diagram to png/svg/html/drawio |
| `ta:adr` | `new` / `supersede` / `list` — Architecture Decision Records |
| `ta:status` | Report current phase, pending approvals, open questions |

## Workspace layout

```
<target-dir>/
├── .ta/state.yaml          # phase, approvals, environment, decisions, adr_counter
├── 00-input/                # user-provided initial materials
├── 01-expectations/expectations.md
├── 02-requirements/requirements.md
├── 03-design-ideas/design-ideas.md
├── 04-selected-design/selected-design.md
├── 05-risk-analysis/risk-analysis.md
├── adr/ADR-NNNN-*.md
├── 06-final/architecture.{md,pdf}, assets/
└── ta.config.md
```

Full schema and per-phase detail: `skills/initialize/SKILL.md`,
`shared/conventions.md`.

## License gating

PNG rendering is always free. SVG/HTML/Draw.io via the Trellis CLI or MCP
server require `TRELLIS_KEY` (see `../../docs/licensing.md`). The VS Code
extension's own live preview and SVG/PNG/Draw.io export are license-free
regardless — it's the CLI/MCP-side rendering that's gated. `ta:initialize`
records what's available in `.ta/state.yaml`; every skill checks that before
choosing a render path.
