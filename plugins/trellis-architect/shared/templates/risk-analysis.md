# Risk Analysis — <project name>

> Produced by `ta:risk-analysis` from `../04-selected-design/selected-design.md`
> using SWIFT (Structured What-If Technique). See `../conventions.md` for id
> rules.

## SWIFT table

Guide words: fails / is slow / is unavailable / is compromised / grows 10x —
walk the selected design element by element.

| Id | What-if | Cause | Consequence | Severity × Likelihood | Mitigation | Mitigation type | Owner phase |
|---|---|---|---|---|---|---|---|
| R-01 | What if <element> fails? | | | High × Low | | architectural / development | |

**Mitigation type** is exactly one of:
- **architectural change** — feeds back into
  `../04-selected-design/selected-design.md` and gets its own ADR (context =
  this risk id).
- **development activity** — process, testing, or ops practice; no ADR needed
  unless it also changes the architecture.

## Architectural mitigations applied

For each risk mitigated by an architectural change: what changed in
`../04-selected-design/selected-design.md`, and the ADR that records it.

| Risk | Change | ADR |
|---|---|---|
| R-01 | | [ADR-000x](../adr/ADR-000x-<slug>.md) |

## Residual risk summary

<risks accepted as-is, with the reasoning — not every risk gets mitigated to
zero>

## Raw notes

**Q:** <question>
**A:** <answer, verbatim>
