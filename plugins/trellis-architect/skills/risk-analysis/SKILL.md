---
name: risk-analysis
description: This skill should be used when the user asks to "analyze risks", "ta:risk-analysis", "run a SWIFT review", "what could go wrong with this design", or has an approved 04-selected-design/selected-design.md and needs a structured risk pass before final documentation. Walks the selected design with SWIFT guide-words and feeds architectural mitigations back into the design.
version: 0.1.0
---

# ta:risk-analysis

Fifth workflow phase. SWIFT (Structured What-If Technique) review of the
selected design, producing `05-risk-analysis/risk-analysis.md`. Failure
mitigation by design is the point — architectural mitigations feed back into
`04-selected-design/selected-design.md`, not just into a table nobody
revisits.

Read `../../shared/conventions.md` first.

## Preconditions

Require `approvals.select-design.approved: true`. Otherwise point at
`ta:select-design`.

## Procedure

1. **Walk the selected design element by element** (each container/component
   in `04-selected-design/selected-design.md`'s diagrams, each critical
   external dependency).
   For each, apply the guide-words: *fails / is slow / is unavailable / is
   compromised / grows 10x*. Not every guide-word produces a risk for every
   element — skip combinations that are clearly not credible rather than
   padding the table.

2. **Per risk row** (`R-xx`): what-if question, cause, consequence,
   severity × likelihood (ask the user to weigh in on judgment calls — this
   is not purely mechanical), mitigation, mitigation type
   (**architectural change** vs **development activity** — see conventions),
   owner phase (usually `risk-analysis` itself, but a mitigation might belong
   to `documentation` or an operational phase outside this workflow).

3. **Architectural mitigations**: for each risk whose mitigation type is
   "architectural change," update `04-selected-design/selected-design.md`
   (diagram and/or text) to reflect it, then:
   - `ta:adr new` with context = the risk id, unless the mitigation reverses
     an already-accepted decision — in that case `ta:adr supersede` on the
     ADR being reversed instead.
   - Record the change in this document's "Architectural mitigations applied"
     table, linking the ADR.

4. **Residual risk summary**: risks intentionally left unmitigated or
   partially mitigated, with the reasoning the user gave for accepting them.
   Don't silently drop a risk that isn't fully closed — every `R-xx` row ends
   up either mitigated or explicitly accepted as residual.

5. **Gate**: user approves the SWIFT table and residual-risk summary,
   including any diagram changes made to `04-selected-design/selected-design.md`
   along the way. On approval:
   - Flip this phase's ADRs to `accepted`.
   - `approvals.risk-analysis: {approved: true, date: <today>}`.
   - `phase: documentation`.
   - Next command: `ta:documentation`.

## Notes

- If an architectural mitigation changes a diagram in
  `04-selected-design/selected-design.md`, that file keeps its existing
  `fig-xx` ids for unchanged diagrams and only
  assigns a new id if the change is substantial enough to be a genuinely new
  diagram (a redraw of the same diagram with one added component reuses the
  id; a wholly new deployment diagram gets a new one).
