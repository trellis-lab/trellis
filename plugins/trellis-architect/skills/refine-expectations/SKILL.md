---
name: refine-expectations
description: This skill should be used when the user asks to "refine expectations", "ta:refine-expectations", "interview me about the project", or is ready to move past ta:initialize into the first design phase. Interviews the user to capture business goals, users, scope, and constraints into 01-expectations/expectations.md, gated by user approval of the system context diagram.
version: 0.1.0
---

# ta:refine-expectations

First workflow phase. Turn `00-input/` materials and an interview into
`01-expectations/expectations.md`, gated by approval of a system context
diagram.

Read `../../shared/conventions.md` before acting (interview, diagram, and gate
rules) and `../../shared/c4-cheatsheet.md` before drafting the diagram.

## Preconditions

Read `.ta/state.yaml`. If `phase` isn't `expectations`, tell the user this
phase is already past (or not yet reached) and point at `ta:status`.

## Procedure

1. **Read `00-input/`.** Summarize what's there (requirement docs, notes) as
   context, not as ground truth — the interview may contradict or extend it.
   If empty, proceed straight to the interview; note in "Known facts" that no
   input materials were provided.

2. **Interview, one topic at a time**, in this order (skip a topic only if
   the input materials already answer it fully — confirm the answer with the
   user rather than assuming it's still accurate):
   - Business goals — what does success look like, in the sponsor's terms?
   - Users and stakeholders — roles, and how their goals differ.
   - Scope — explicitly ask what's *out* of scope, not just what's in.
   - Constraints — technical, budget, timeline, compliance (ask each
     separately; "any constraints?" gets vague answers).
   - Existing systems — integration points, replacements, coexistence.
   - Quality expectations — in the user's own words (precise NFR wording is
     `ta:define-requirements`'s job, not this one's).
   - Success criteria — how will the user know this succeeded?

   Record every answer verbatim in the "Raw notes" appendix as it's given,
   then synthesize into the structured sections above it.

3. **Surface assumptions.** Anywhere the interview left a gap the document
   needs filled, write it as an explicit assumption (not a silent fact) or an
   open question — the user's later "yes that's right" on an assumption
   promotes it to a known fact; don't do that promotion unprompted.

4. **Significant decisions**: if the interview surfaces one (e.g. a mandated
   technology or vendor), invoke `ta:adr new` immediately rather than only
   noting it in prose — see `../../shared/conventions.md` §ADRs.

5. **Draft the system context diagram** (`C4Context`) reflecting the agreed
   scope: the system, its users, and external systems it talks to. Insert as
   `fig-01` in `01-expectations/expectations.md`.

6. **Gate**: ask the user to open `01-expectations/expectations.md` in VS Code and preview
   `fig-01` with the Trellis extension. On explicit approval:
   - Set `approvals.expectations: {approved: true, date: <today>}` in
     `state.yaml`.
   - Flip any `proposed` ADRs created during this phase to `accepted`.
   - Set `phase: requirements`.
   - Tell the user the next command is `ta:define-requirements`.
   On requested changes, revise and re-ask — don't advance the phase.

## Notes

- If the diagram needs a revision after feedback, keep the same `fig-01` id —
  only new diagrams get new ids.
- Don't let "Open questions" linger unaddressed into the gate — either resolve
  them with the user or explicitly carry them forward as a documented open
  question the user has agreed to defer.
