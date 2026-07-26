---
name: export
description: This skill should be used when the user asks to "export this diagram", "ta:export", "render fig-0x to svg/png/html/drawio", "get a draw.io version of this diagram", or wants any mermaid block from the ta workflow docs rendered to an output format. Renders via the Trellis MCP server or CLI into 06-final/assets/, respecting license gating.
version: 0.1.0
---

# ta:export

Render any Mermaid block from the workflow's markdown documents to
`png` / `svg` / `html` / `drawio`, saved into `06-final/assets/`. Usable in
any phase — not only during `ta:documentation`.

Read `../../shared/conventions.md` §Rendering first.

## Procedure

1. **Identify the diagram**: by `fig-xx` id or by file + position. Extract the
   Mermaid source from its fenced block.

2. **Identify the target format.** If not given, ask, using this guide (also
   in conventions.md and the MCP `output_format_guide` prompt when available):
   - `png` — quick preview, no license required.
   - `svg` — quick high-quality preview / import into another editor.
   - `html` — interactive review; best for stakeholder walkthroughs.
   - `drawio` — further manual editing in draw.io.

3. **Check the license gate** (`environment.license` in `.ta/state.yaml`):
   - `svg`/`html`/`drawio` require `TRELLIS_KEY`. If `environment.license:
     false`, tell the user and offer two alternatives: fall back to `png`, or
     use the VS Code extension's own export commands (`Trellis: Export as
     SVG` / `Trellis: Export as Draw.io`), which don't require a license.
     Don't silently downgrade to PNG without saying so.
   - `png` always works.

4. **Render**:
   - Prefer the Trellis MCP `render` tool if `environment.mcp: true`:
     `render(mermaid=<source>, format=<target>, theme=<from ta.config.md,
     default "default">)`. PNG returns base64 image content; other formats
     return text — write it to the output file.
   - Otherwise use the CLI: write the Mermaid source to a temp `.mmd` file,
     `trellis render <temp>.mmd -o 06-final/assets/<fig-id>.<ext>
     [-f <format>]` (svg is the CLI default; still pass `-f` explicitly for
     clarity when the target isn't svg).
   - If neither `environment.mcp` nor `environment.trellis_bin` is available,
     tell the user no CLI/MCP render path exists and point at the VS Code
     extension as the only option.

5. **Report** the output path and, for `drawio`, remind the user this file is
   meant for manual editing — re-import per `ta:documentation`'s round-trip
   step once edited, rather than treating it as a final artifact on its own.

## Notes

- This skill doesn't update the phase document — it produces a rendered copy
  alongside it. Replacing a `mermaid` block with an image reference (the
  round-trip step) is `ta:documentation`'s job, done deliberately at assembly
  time, not on every ad-hoc export.
