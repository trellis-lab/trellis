---
name: adr
description: This skill should be used when the user asks to "record a decision", "create an ADR", "ta:adr new", "supersede ADR-000x", "ta:adr supersede", "list ADRs", "ta:adr list", or when any trellis-architect phase skill needs to capture a significant architectural decision. Usable in every phase, not only select-design.
version: 0.1.0
---

# ta:adr

Manage Architecture Decision Records in `adr/`. Significant decisions surface
in any phase (e.g. "we must use the corporate SSO" during expectations) — this
skill is invoked directly by the user or by another phase skill mid-interview,
not only at `select-design`.

Read `../../shared/conventions.md` §ADRs before acting.

## `ta:adr new`

1. Read `.ta/state.yaml`, take `adr_counter` as the new id (zero-padded to 4
   digits), and the current `phase`.
2. Instantiate `../../shared/templates/adr.md`:
   - Front matter: `status: proposed`, today's date, current `phase`,
     `supersedes: null`, `superseded-by: null`, `requirements`/`risks` arrays
     populated with any ids the decision traces to (ask if unclear whether
     there are any — don't guess).
   - Body: fill Context, Decision drivers, Options considered (at least two —
     if only one option was ever viable, say so explicitly in Context instead
     of inventing a straw-man second option), Decision, Consequences.
3. Save as `adr/ADR-NNNN-<slug>.md` (`<slug>` is a short kebab-case title).
4. Update `state.yaml`: increment `adr_counter`; append to `decisions`:
   `{date, phase, summary: <one line>, adr: ADR-NNNN}`.
5. Tell the user the ADR is `proposed` — it becomes `accepted` automatically
   when the phase it belongs to is approved (the phase skill's gate step does
   this; `ta:adr` does not flip status itself unless asked to accept out of
   band).

## `ta:adr supersede ADR-NNNN`

1. Read the target ADR; confirm it's `accepted` (a `proposed` or already-
   `superseded` ADR can't be superseded — say why and stop).
2. Create a new ADR via the `new` procedure above, with one difference:
   front matter `supersedes: ADR-NNNN`.
3. Edit the old ADR's front matter: `status: superseded`,
   `superseded-by: ADR-MMMM` (the new id). Never delete or renumber the old
   file — it stays as a permanent record.
4. Append to `state.yaml` `decisions` as in step 4 above, with the summary
   noting what it supersedes.
5. If the superseded decision is referenced from a phase document (e.g.
   `04-selected-design/selected-design.md` links the old ADR id), update that link to point at
   the new id and note in-line that it supersedes the old one — don't leave a
   dangling reference to a superseded decision presented as current.

## `ta:adr list`

Scan `adr/*.md`, read each front matter, and render a register table:

| Id | Title | Status | Phase | Requirements | Risks |
|---|---|---|---|---|---|
| ADR-0000 | Record architecture decisions | accepted | expectations | — | — |
| ADR-0001 | | | | | |

Sort by id ascending. This is the same table `ta:documentation` embeds in the
final document's Decision Register — keep the format identical so that step
can copy it directly.

## Cross-cutting rules

- One decision per ADR. If a phase skill invoking this mid-interview surfaces
  two independent decisions, create two ADRs, not one with two "decisions."
- Never edit an `accepted` or `superseded` ADR's Decision/Consequences sections
  after the fact — front-matter status and `superseded-by` are the only fields
  that change post-acceptance.
- `adr_counter` only increments, even across `supersede` calls — it is never
  decremented or reused.
