# Trellis Architect — Shared Conventions

Every `ta:*` skill follows these rules instead of restating them. Read this file at
the start of any phase skill.

## Diagrams

- Mermaid only. Prefer C4 for architecture: `C4Context` in expectations,
  `C4Container` from design-ideas onward, `C4Component` when a container's
  internals need detail in select-design or documentation.
- Use Trellis-specific container shapes (`ContainerFrontend`, `ContainerApp`,
  `ContainerFolder`, `ContainerGateway`, `ContainerBucket`, each with an `_Ext`
  variant) whenever an element's role matches one. See `c4-cheatsheet.md`.
- Every diagram gets a caption and a stable id: `**Figure fig-XX.** <caption>`
  directly below the closing ` ``` `. Never renumber an existing `fig-xx` — new
  diagrams take the next number even if an earlier one is removed.
- Write diagrams directly as ` ```mermaid ` fenced blocks in the phase document.
  Nothing else touches the file between blocks.

## Review loop

After writing or updating a diagram:
1. Tell the user which file changed and which `fig-xx` to look at.
2. Ask them to open the file in VS Code and preview with the Trellis extension
   (`Ctrl+Shift+V` on a `.mmd` file, or the CodeLens "Trellis: Preview Mermaid
   Block" above a ` ```mermaid ` block in Markdown).
3. Wait for explicit approval before writing it to `state.yaml` and advancing.
   Approval language ("looks good", "approved", "go ahead") is what unlocks the
   phase gate — silence or a request for changes does not.
- Never advance a phase without a recorded approval in `.ta/state.yaml`.

## Rendering (for `ta:export`, and any skill previewing outside the extension)

- Prefer the Trellis MCP `render` tool when the MCP server is configured
  (`environment.mcp: true` in `state.yaml`); fall back to the `trellis` CLI.
- License gate: `TRELLIS_KEY` unset → PNG only (both CLI and MCP). SVG, HTML, and
  Draw.io need a key. The VS Code extension's live preview and SVG/PNG/Draw.io
  export are license-free regardless — only CLI/MCP-side non-PNG rendering is
  gated. If `environment.license: false`, tell the user PNG is all that's
  available outside the extension and point at the extension's own export
  commands as the license-free path to SVG/PNG/Draw.io.
- CLI: `trellis render <input> -o <output> [-f svg|png|html|ascii|drawio]`.
  MCP: `render(mermaid, format="png"|"svg"|"html"|"drawio", theme=...)`.

## Documents

- Markdown, one H1 per file.
- Requirement ids (`FR-xx`, `NFR-xx`), risk ids (`R-xx`), and ADR ids
  (`ADR-NNNN`) are stable once written — never reused, even if the item is later
  dropped (mark it superseded/withdrawn in place instead).
- Keep traceability links live in both directions: expectation → requirement →
  design element → risk. When a phase document references an id from an earlier
  phase, link it (`[FR-03](02-requirements.md#fr-03)`), don't restate its text.

## Interviews

- One focused question at a time. Group only tightly related sub-questions
  (e.g. "who are the primary user roles, and how do they differ?").
- Record answers verbatim in the document's "Raw notes" appendix before
  synthesizing them into the structured sections above it. Synthesis is allowed
  to reorganize and summarize; the raw notes are the audit trail back to what
  the user actually said.

## ADRs

- MADR-style: context, decision drivers, options considered, decision,
  consequences.
- One decision per ADR. Immutable once `accepted` — a change of mind is a new
  ADR that supersedes the old one, never an edit to history.
- Status lifecycle: `proposed → accepted → superseded | deprecated`. `proposed`
  at creation inside a phase; flips to `accepted` when that phase's gate is
  approved.
- Ids: `ADR-NNNN`, zero-padded to 4 digits, taken from `state.yaml`
  `adr_counter` and never reused (counter only increments).
- Phase documents link ADR ids instead of restating rationale. ADRs link back to
  the requirement ids that drove them and the risk ids that triggered them.
- Include a Mermaid diagram inside an ADR only when the decision is structural
  enough to need one — most ADRs don't.
- Full template: `templates/adr.md`. Full workflow: `skills/adr/SKILL.md`.

## State file (`.ta/state.yaml`)

Every skill reads this before acting and writes to it after a gate is approved
or an ADR is created. Never advance `phase` without a recorded approval. Schema
and full field reference: `skills/initialize/SKILL.md`.
