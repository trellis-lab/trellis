---
name: define-requirements
description: This skill should be used when the user asks to "define requirements", "ta:define-requirements", "write FRs and NFRs", or has an approved 01-expectations.md and is ready for the requirements phase. Derives functional and non-functional requirements with traceability back to expectations, gated by user approval.
version: 0.1.0
---

# ta:define-requirements

Second workflow phase. Derive `FR-xx` / `NFR-xx` from `01-expectations.md`,
interview to fill gaps, produce `02-requirements.md`.

Read `../../shared/conventions.md` first.

## Preconditions

Read `.ta/state.yaml`. Require `approvals.expectations.approved: true` — if
not, tell the user expectations must be approved first (`ta:refine-
expectations`). If `phase` is already past `requirements`, point at
`ta:status`.

## Procedure

1. **Derive a first draft** from `01-expectations.md`:
   - Functional requirements (`FR-xx`) from stated goals, user needs, and
     scope-in items.
   - Non-functional requirements (`NFR-xx`), grouped by quality attribute
     (performance, security, availability, and any others the quality-
     expectations section implies — don't force every category to have an
     entry).
   - Every row gets: id, statement, rationale, MoSCoW priority, and a link
     back to the specific expectations-doc line/section it traces to.

2. **Interview to fill gaps** the expectations document doesn't resolve
   precisely enough for a testable requirement statement (vague quality
   expectations like "should be fast" need a concrete NFR — ask for the
   number or the acceptable range). One topic at a time, per conventions.

3. **Priority**: MoSCoW (Must/Should/Could/Won't). Ask the user to confirm
   priorities rather than assigning them unilaterally when it's not obvious
   from the expectations doc.

4. **Traceability check** before the gate: every FR/NFR has a non-empty
   source-expectation link. Any requirement that emerged only during this
   interview (no prior expectations-doc line) gets a note added to
   `01-expectations.md` (its "Known facts" or "Open questions" section, as
   appropriate) so the link target exists — don't leave it dangling.

5. **Diagram**: a use-case or context diagram only if it clarifies how FRs
   group (e.g. by actor or subsystem). Skip it if `01-expectations.md`'s
   context diagram already conveys the same grouping — don't duplicate.

6. **Significant decisions** (e.g. "compliance mandates encryption at rest"):
   `ta:adr new`, same as in every other phase.

7. **Gate**: user reviews (diagram, if drafted) and approves the requirements
   set as a whole — not row-by-row. On approval:
   - `approvals.requirements: {approved: true, date: <today>}`.
   - Flip this phase's `proposed` ADRs to `accepted`.
   - `phase: design-ideas`.
   - Next command: `ta:design-ideas`.

## Notes

- Requirement ids are permanent — a requirement dropped later is marked
  withdrawn in place (e.g. strike-through with a one-line reason), never
  deleted, and never has its id reused by a different requirement.
- Keep FR and NFR numbering independent (`FR-01`, `NFR-01`, not a shared
  counter).
