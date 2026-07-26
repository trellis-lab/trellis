---
name: documentation
description: This skill should be used when the user asks to "assemble the final documentation", "ta:documentation", "generate the architecture PDF", "produce the final design doc", or has an approved 05-risk-analysis.md and is ready to publish. Assembles 06-final/architecture.md from all phases, offers draw.io polish for diagrams, and renders the PDF via the trellis-pandoc Docker image.
version: 0.1.0
---

# ta:documentation

Sixth and final workflow phase. Assemble `06-final/architecture.md` from
`ta.config.md` + phases 01–05, optionally polish diagrams in draw.io, then
render to PDF with the `trellis-pandoc` Docker image.

Read `../../shared/conventions.md` first.

## Preconditions

Require `approvals.risk-analysis.approved: true`. Otherwise point at
`ta:risk-analysis`.

## Procedure

1. **Assemble `06-final/architecture.md`** from `../../shared/templates/architecture.md`,
   filling each section from the corresponding phase document:
   - Pandoc YAML front matter: title (project name), author(s), date, `toc:
     true`, `geometry` — from `ta.config.md`.
   - Executive Summary — write fresh (one page max): problem, chosen approach,
     key risks, current status. Nothing else in this document is written
     fresh; this section is.
   - Context — from `01-expectations.md`.
   - Requirements — summarized from `02-requirements.md` (link to the full
     tables rather than duplicating every row, unless the user wants them
     inline).
   - Architecture — from `04-selected-design.md`, all diagrams.
   - Decision Register — run `ta:adr list`'s table generation and embed it
     directly; include full text of every `accepted` ADR as an appendix, in
     id order. Proposed or superseded ADRs are listed in the register (with
     their status) but only accepted ones get full appendix text.
   - Risk Register — from `05-risk-analysis.md`: SWIFT table + residual-risk
     summary.
   - Glossary — build from terms used across phase docs that a reader outside
     the project wouldn't know; ask the user to review it rather than
     guessing what needs defining.

2. **Diagram finalization.** For each `fig-xx` in the assembled document,
   offer draw.io polish:
   - If the user wants manual polish: `ta:export <fig-id> drawio`, they edit
     in draw.io, then re-render the edited `.drawio` to an image (PNG/SVG) and
     place it in `06-final/assets/`. Replace the `mermaid` block in
     `architecture.md` with an image reference to that asset — this is the
     one place a diagram is allowed to stop being a live `mermaid` block.
   - Diagrams the user doesn't polish stay as `mermaid` blocks — the pandoc
     Lua filter renders them to inline SVG automatically at PDF time; no
     manual export needed for those.
   - Track which figures were replaced with images vs. left as `mermaid` so
     the report in step 4 can list both.

3. **Render the PDF:**

   ```bash
   docker pull ghcr.io/trellis-lab/trellis-pandoc:latest   # first run / update
   docker run --rm -v "$(pwd):/data" ghcr.io/trellis-lab/trellis-pandoc:latest \
     06-final/architecture.md -o 06-final/architecture.pdf
   ```

   Apply any pandoc options recorded in `ta.config.md` (e.g. `--pdf-engine=
   xelatex`) by adding them to the `docker run` command. Requires
   `environment.docker: true` — if false, tell the user to install Docker or
   run the Lua filter directly with a local `pandoc` + `trellis` binary (see
   the Trellis docs' pandoc guide) and stop here.

4. **Gate — final sign-off**, distinct from every prior phase gate: ask the
   user to review the rendered PDF itself (not just the source markdown), not
   only the diagrams. On approval:
   - `approvals.documentation: {approved: true, date: <today>}`.
   - `phase: done`.
   - Report the final path: `06-final/architecture.pdf`.

## Notes

- If the PDF review surfaces a change to content already approved in an
  earlier phase (not just a formatting fix), that's a change to the source
  phase document and its approval, not a silent edit at assembly time — flag
  it to the user and re-run that phase's gate before re-assembling.
- `06-final/assets/` accumulates both `ta:export` ad-hoc renders and the
  round-tripped draw.io images from step 2 — keep only the ones actually
  referenced from `architecture.md` to avoid the folder becoming a dumping
  ground; mention any orphaned files found during assembly.
