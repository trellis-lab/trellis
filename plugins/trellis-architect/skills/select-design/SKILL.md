---
name: select-design
description: This skill should be used when the user asks to "select a design", "ta:select-design", "compare the candidates", "pick the architecture", or has approved 03-design-ideas/design-ideas.md and needs to choose or combine candidates. Runs a weighted scoring comparison, supports hybrid designs, and records the choice as ADRs.
version: 0.1.0
---

# ta:select-design

Fourth workflow phase. Compare candidates from `03-design-ideas/design-ideas.md`,
select or combine, produce `04-selected-design/selected-design.md`. The
decision itself is captured as ADRs — this document links them, it doesn't
restate their rationale.

Read `../../shared/conventions.md` and `../../shared/c4-cheatsheet.md` first.

## Preconditions

Require `approvals.design-ideas.approved: true`. Otherwise point at
`ta:design-ideas`.

## Procedure

1. **Build the weighted scoring matrix**: rows = prioritized requirements
   (weight derived from MoSCoW — Must > Should > Could; ask the user to
   confirm or adjust numeric weights rather than inventing a scale silently),
   columns = candidates. Score each candidate per requirement; ask the user
   to validate scores they'd disagree with rather than presenting the matrix
   as a fait accompli.

2. **Support hybrids explicitly.** If the user wants to combine elements
   (e.g. Candidate A's frontend with Candidate B's data layer), that's a
   first-class outcome, not a fallback — draft the merged container diagram
   and state which elements came from which candidate, in place of a single
   "winner" column.

3. **Record the decision as ADRs** (via `ta:adr new`), not as prose in this
   document:
   - One ADR for the overall architecture choice. "Options considered" =
     the candidates from `03-design-ideas/design-ideas.md` (rejected ones
     included, with why they lost). `requirements:` front matter = the
     highest-weighted rows that drove the outcome.
   - One additional ADR per significant technology/pattern choice *inside*
     the winner (e.g. choice of message broker, choice of auth pattern) —
     only for choices material enough to matter later if revisited, not every
     minor detail.
   - `04-selected-design/selected-design.md` links these ADR ids in the
     Decision Summary section instead of duplicating their rationale.

4. **Draft the refined diagrams**: `C4Container` for the selected/merged
   design (own `fig-xx`), plus `C4Component` for any container whose internals
   need detail at this stage (skip components with no interesting internal
   structure yet — that can wait for `ta:documentation` or stay implicit).

5. **Gate**: user approval here means accepting the ADRs, not just liking the
   diagram — say so explicitly when asking. On approval:
   - Flip all ADRs created in this phase from `proposed` to `accepted`.
   - `approvals.select-design: {approved: true, date: <today>}`.
   - `phase: risk-analysis`.
   - Next command: `ta:risk-analysis`.

## Notes

- If the user changes their mind about a choice already recorded as an
  accepted ADR mid-phase, that's a supersession (`ta:adr supersede`), not an
  edit — even within the same phase, before the phase gate closes.
