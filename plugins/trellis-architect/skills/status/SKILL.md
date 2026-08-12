---
name: status
description: This skill should be used when the user asks "where am I in the design", "ta:status", "what's next", "show open questions", "list pending approvals", or wants a summary of workflow progress in a trellis-architect project. Reads .ta/state.yaml and reports phase, approvals, open questions, and proposed ADRs.
version: 0.1.0
---

# ta:status

Report current position in the workflow without changing anything. Safe to run
at any time, in any phase.

## Procedure

1. Locate `.ta/state.yaml` in the current or given project directory. If
   missing, tell the user this directory hasn't been initialized and suggest
   `ta:initialize`.
2. Read `state.yaml` and report:
   - **Current phase** and which document that maps to (e.g. `design-ideas` →
     `03-design-ideas/design-ideas.md`).
   - **Approvals**: which phases are approved (with date) vs. pending.
   - **Environment**: `trellis_bin`, `license`, `mcp`, `docker` from the
     `environment` block — flag any that block the current phase's rendering
     needs (e.g. no `trellis_bin` and `mcp: false` means no CLI/MCP render
     path, only the VS Code extension).
3. Scan the current phase's document (and `01-expectations/expectations.md`
   always, since its "Open questions" section tends to stay live longest) for
   open items:
   - Any `## Open questions` section with unresolved bullets.
   - Any `<placeholder>`-style text left unfilled (a template that wasn't
     completed).
4. Scan `adr/` for ADRs with `status: proposed` in their front matter — these
   haven't been accepted at a phase gate yet. List id, title, phase.
5. Suggest the next command:
   - If the current phase isn't approved yet and its document has no open
     placeholders → suggest reviewing diagrams and approving (tell the user
     which skill re-runs the gate check, i.e. the current phase's own skill).
   - If open questions or placeholders remain → suggest continuing the
     interview for the current phase's skill.
   - If the current phase is approved → suggest the next phase's skill, in
     order: `initialize → refine-expectations → define-requirements →
     design-ideas → select-design → risk-analysis → documentation`.
   - If `phase: done` → point at `06-final/architecture.pdf`.

## Output shape

Keep it scannable — a short status block, not prose:

```
Phase: design-ideas (not yet approved)
Approvals: expectations ✅ 2026-07-20 · requirements ✅ 2026-07-22 · design-ideas ⏳
Environment: trellis_bin=/usr/local/bin/trellis · license=true · mcp=true · docker=true
Open questions: 2 in 03-design-ideas/design-ideas.md
Proposed ADRs: none
Next: finish the design-ideas interview, then review fig-03/fig-04 and approve.
```
