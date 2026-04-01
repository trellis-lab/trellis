#!/usr/bin/env bash
# scripts/build-pandoc-docker.sh
#
# Builds the Trellis Pandoc Docker image which bundles the Trellis CLI
# with Pandoc for full Markdown-to-PDF/HTML rendering with Mermaid support.
#
# Usage:
#   ./scripts/build-pandoc-docker.sh
#
# Then render documents:
#   docker run --rm -v "$(pwd):/data" trellis-pandoc input.md -o output.pdf
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
IMAGE_NAME="trellis-pandoc"

# ── Sanity checks ─────────────────────────────────────────────────────────────
if ! command -v docker &>/dev/null; then
    echo "Error: docker not found. Install Docker Desktop from https://www.docker.com/products/docker-desktop/" >&2
    exit 1
fi

if ! docker info &>/dev/null; then
    echo "Error: Docker daemon is not running. Start Docker Desktop and try again." >&2
    exit 1
fi

# ── Check Lua filter exists ──────────────────────────────────────────────────
FILTER="$REPO_ROOT/filters/trellis-filter.lua"
if [[ ! -f "$FILTER" ]]; then
    echo "Error: Pandoc Lua filter not found at $FILTER" >&2
    echo "       The trellis-filter.lua should exist from M14 (CLI features)." >&2
    exit 1
fi

# ── Build image ──────────────────────────────────────────────────────────────
echo "==> Building Docker image '$IMAGE_NAME'…"
echo "    (first run compiles the Trellis CLI from source — this may take a few minutes)"
docker build \
    --file "$REPO_ROOT/docker/Dockerfile.pandoc" \
    --tag  "$IMAGE_NAME" \
    "$REPO_ROOT"

echo ""
echo "==> Done. Usage:"
echo "    docker run --rm -v \"\$(pwd):/data\" $IMAGE_NAME input.md -o output.pdf"
echo "    docker run --rm -v \"\$(pwd):/data\" $IMAGE_NAME input.md -o output.html"
