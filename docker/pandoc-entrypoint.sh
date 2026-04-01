#!/usr/bin/env bash
# docker/pandoc-entrypoint.sh
#
# Runs inside the trellis-pandoc container.
# Wraps Pandoc with the Trellis Lua filter so that ```mermaid code blocks
# are automatically rendered to inline SVG diagrams.
#
# Usage (via docker run):
#   docker run --rm -v "$(pwd):/data" trellis-pandoc input.md -o output.pdf
#   docker run --rm -v "$(pwd):/data" trellis-pandoc input.md -o output.html
#   docker run --rm -v "$(pwd):/data" trellis-pandoc input.md -o output.pdf --pdf-engine=xelatex
set -euo pipefail

FILTER="/usr/local/share/pandoc/filters/trellis-filter.lua"

# ── Guard: trellis CLI must be available ─────────────────────────────────────
if ! command -v trellis &>/dev/null; then
    echo "Error: trellis CLI not found in PATH." >&2
    exit 1
fi

if [[ $# -eq 0 ]]; then
    echo "Usage: docker run --rm -v \"\$(pwd):/data\" trellis-pandoc INPUT.md -o OUTPUT.pdf" >&2
    echo "" >&2
    echo "Converts Markdown to PDF/HTML with Mermaid diagrams rendered by Trellis." >&2
    echo "All standard Pandoc arguments are supported." >&2
    exit 0
fi

echo "==> Running Pandoc with Trellis filter…"
exec pandoc --lua-filter="$FILTER" "$@"
