---
name: initialize
description: This skill should be used when the user asks to "start a new architecture design", "initialize a trellis-architect project", "scaffold ta project", "ta:initialize", or names a target directory for a new design workflow. Creates the workspace structure, detects the Trellis environment, and seeds the state file and meta-ADR that every other ta skill depends on.
version: 0.1.0
---

# ta:initialize

Scaffold a new Trellis Architect workspace in a target directory and record the
detected environment. Every other `ta:*` skill reads `.ta/state.yaml`, created
here — run this first in any new project.

Read `../../shared/conventions.md` before acting; it defines diagram, ADR, and
id rules referenced below.

## Procedure

1. **Get the target directory.** If not given, ask for it. Confirm before
   writing if the directory already exists and is non-empty (this is not a
   fresh scaffold — check for an existing `.ta/state.yaml` first; if found,
   stop and tell the user to use `ta:status` instead of re-initializing).

2. **Get the project name and author(s).** Short questions, used in
   `ta.config.md` and later in the pandoc front matter of the final document.

3. **Create the structure:**

   ```
   <target-dir>/
   ├── .ta/state.yaml
   ├── 00-input/
   ├── 01-expectations/
   │   └── expectations.md      (copied from templates/expectations.md)
   ├── 02-requirements/
   │   └── requirements.md      (copied from templates/requirements.md)
   ├── 03-design-ideas/
   │   └── design-ideas.md      (copied from templates/design-ideas.md)
   ├── 04-selected-design/
   │   └── selected-design.md   (copied from templates/selected-design.md)
   ├── 05-risk-analysis/
   │   └── risk-analysis.md     (copied from templates/risk-analysis.md)
   ├── adr/
   │   └── ADR-0000-record-architecture-decisions.md
   ├── 06-final/
   │   ├── architecture.md      (copied from templates/architecture.md, filled later)
   │   └── assets/
   └── ta.config.md
   ```

   Every phase gets its own numbered folder, even though it holds a single
   file today — keeps room for supporting material (input excerpts, working
   notes) without a later rename. Copy each template from
   `../../shared/templates/` into its folder, substituting `<project name>`
   placeholders. Leave phase content as template placeholders — later skills
   fill them in during their own interviews. `00-input/` starts empty; tell
   the user to drop initial requirement docs or notes there before running
   `ta:refine-expectations`.

4. **Seed the meta-ADR.** Instantiate `../../shared/templates/adr.md` as
   `adr/ADR-0000-record-architecture-decisions.md`: status `accepted`, phase
   `expectations`, decision "We will use Architecture Decision Records to
   document significant decisions, following the MADR-style format described
   in the trellis-architect plugin conventions." No requirements/risks links —
   it predates the project's own requirements.

5. **Detect the environment:**
   - **Trellis binary**: check `TRELLIS_BIN` env var, else `which trellis`
     (or `where trellis` on Windows). Record the resolved path, or `null` if
     neither resolves.
   - **License**: check `TRELLIS_KEY` in the environment or a `.env` file in
     the target directory. Record `true`/`false` — don't print the key value.
   - **MCP**: check whether a Trellis MCP tool (name containing `trellis`,
     exposing a `render` tool) is already available in this session. If
     unsure, ask the user whether they've configured the Trellis MCP server;
     record their answer.
   - **Docker**: check `docker info` exits 0. Record `true`/`false`.
   - If `trellis_bin` is null and MCP is false, warn the user: no render path
     is available yet, and diagram review will be limited to the VS Code
     extension until one is configured. This does not block initialization.
   - If `license` is false, warn that CLI/MCP rendering is PNG-only until a
     `TRELLIS_KEY` is set (VS Code extension export is unaffected — see
     conventions.md).

6. **Write `.ta/state.yaml`:**

   ```yaml
   project: <name>
   phase: expectations
   approvals:
     expectations: {approved: false, date: null}
     requirements: {approved: false, date: null}
     design-ideas: {approved: false, date: null}
     select-design: {approved: false, date: null}
     risk-analysis: {approved: false, date: null}
     documentation: {approved: false, date: null}
   environment:
     trellis_bin: <path or null>
     license: <true|false>
     mcp: <true|false>
     docker: <true|false>
   decisions: []
   adr_counter: 1
   ```

   `phase` is one of: `expectations | requirements | design-ideas |
   select-design | risk-analysis | documentation | done`. `adr_counter` starts
   at `1` — `0` was consumed by the seed ADR, which is not logged in
   `decisions` (it predates the workflow it documents).

7. **Write `ta.config.md`:** project name, author(s), and placeholders for
   pandoc options (`pdf-engine`, theme) and Trellis CLI theme choice — filled
   in properly by `ta:documentation`, but stub them now so the file exists.

8. **Report back**: structure created, environment findings, and the next
   command (`ta:refine-expectations`, after the user has dropped materials
   into `00-input/`).

## Notes

- Never overwrite an existing `.ta/state.yaml`. Re-running initialize on an
  already-initialized directory is a bug in the calling flow, not a reset
  button — refuse and point at `ta:status`.
- `adr_counter` and `decisions` are the only two fields any skill other than
  `ta:adr` and `ta:initialize` should touch directly for ADR bookkeeping; phase
  skills otherwise only touch their own `approvals.<phase>` entry.
