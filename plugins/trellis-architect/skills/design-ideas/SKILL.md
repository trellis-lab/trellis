---
name: design-ideas
description: This skill should be used when the user asks to "generate design ideas", "ta:design-ideas", "propose architecture candidates", or has an approved 02-requirements/requirements.md and is ready to explore architecture options. Produces 2–4 distinct candidate architectures with C4 Container diagrams, gated by user review of all candidates.
version: 0.1.0
---

# ta:design-ideas

Third workflow phase. Generate 2–4 distinct candidate architectures from
`02-requirements/requirements.md`, produce `03-design-ideas/design-ideas.md`.

Read `../../shared/conventions.md` and `../../shared/c4-cheatsheet.md` first —
every candidate diagram uses Trellis-specific container shapes where they fit.

## Preconditions

Require `approvals.requirements.approved: true`. Otherwise point at
`ta:define-requirements`.

## Procedure

1. **Generate 2–4 candidates** that are *genuinely distinct* — different
   architectural style, technology family, or deployment model, not
   cosmetic variations of the same idea (e.g. "monolith," "microservices via
   API gateway," "serverless event-driven" is distinct; three microservice
   layouts differing only in database vendor is not). Ground each in the
   actual FR/NFR set, not generic best practice.

2. **Per candidate**, write:
   - Overview — one paragraph, the core idea.
   - `C4Container` diagram, own `fig-xx` id.
   - Key technology choices.
   - Pros/cons scored qualitatively against the NFRs (not a weighted number —
     that's `ta:select-design`'s job).
   - Rough cost/complexity, one line with the reason.

3. **Don't pre-select a winner here.** This phase presents options; it
   doesn't argue for one. Save advocacy for the comparison summary at most,
   and keep that neutral too.

4. **Interview only where a candidate needs a technology decision the user
   must make before it can be drawn concretely** (e.g. "should Candidate A
   assume a managed queue or self-hosted?"). Don't over-interview — this
   phase is exploratory, not another requirements pass.

5. **Gate**: user reviews all candidate diagrams together. On approval:
   - `approvals.design-ideas: {approved: true, date: <today>}`.
   - Flip this phase's `proposed` ADRs (if any — usually none; the real
     decision ADRs come from `ta:select-design`) to `accepted`.
   - `phase: select-design`.
   - Next command: `ta:select-design`.

## Notes

- Approval here means "these are reasonable options to choose between," not
  "this is the final design" — don't let the user's enthusiasm for one
  candidate skip the comparison step in `ta:select-design`.
