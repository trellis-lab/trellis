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
3. Ask for the gate decision via `AskUserQuestion` (see §Interviews & user
   input) — options like "Approved" / "Request changes". Wait for explicit
   approval before writing it to `state.yaml` and advancing. Approval language
   ("looks good", "approved", "go ahead", or the "Approved" option itself) is
   what unlocks the phase gate — silence or a request for changes does not.
- Never advance a phase without a recorded approval in `.ta/state.yaml`.

## Interviews & user input

- Every question put to the user — interview questions, gate approvals,
  confirmations, disambiguation, priority/weighting choices — goes through the
  `AskUserQuestion` tool. Don't ask in plain prose and wait for a free-text
  reply; this applies in every skill, not only the ones with "interview" in
  their procedure.
- One focused question at a time. Group only tightly related sub-questions
  into one `AskUserQuestion` call (e.g. "who are the primary user roles, and
  how do they differ?").
- Shape `options` around what the choice actually is: 2-4 concrete options,
  `multiSelect: true` when more than one can apply (e.g. "which quality
  attributes matter here?"). The tool always offers an "Other" choice for
  free text, so open-ended questions still work — give the 2-3 most likely
  answers as options and let "Other" cover the rest; don't force a narrative
  answer into a false multiple-choice.
- Gate approvals ask with options such as "Approved" / "Request changes" (plus
  "Other" for a qualified answer). Only "Approved" (or equivalent free-text
  approval language typed via "Other") unlocks the gate, per the Review loop
  rule above.
- Record the user's selection or free-text answer verbatim in the document's
  "Raw notes" appendix, same as any other interview answer — asking through
  the tool doesn't replace the written record.
- If `AskUserQuestion` isn't available in the current environment, fall back
  to a plain-text question — treat that as the exception, not the default.

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
  phase, link it with a path relative to the linking document's own folder
  (`[FR-03](../02-requirements/requirements.md#fr-03)`), don't restate its text.

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
