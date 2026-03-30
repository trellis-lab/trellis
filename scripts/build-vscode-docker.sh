#!/usr/bin/env bash
# scripts/build-vscode-docker.sh
#
# Builds the Trellis VS Code extension (.vsix) inside Docker.
# The WASM binary must already be present (run build-wasm-docker.sh first).
#
# Usage:
#   ./scripts/build-vscode-docker.sh
#
# Output: dist/trellis-preview-<version>.vsix
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
IMAGE_NAME="trellis-vscode-builder"
WASM_FILE="$REPO_ROOT/editors/vscode/wasm/trellis_wasm_bg.wasm"

# ── Sanity checks ─────────────────────────────────────────────────────────────
if ! command -v docker &>/dev/null; then
    echo "Error: docker not found. Install Docker Desktop from https://www.docker.com/products/docker-desktop/" >&2
    exit 1
fi

if ! docker info &>/dev/null; then
    echo "Error: Docker daemon is not running. Start Docker Desktop and try again." >&2
    exit 1
fi

if [[ ! -f "$WASM_FILE" ]]; then
    echo "Error: WASM binary not found at $WASM_FILE" >&2
    echo "       Run ./scripts/build-wasm-docker.sh first." >&2
    exit 1
fi

# ── Build image ───────────────────────────────────────────────────────────────
echo "==> Building Docker image '$IMAGE_NAME'…"
docker build \
    --file "$REPO_ROOT/docker/Dockerfile.vscode-builder" \
    --tag  "$IMAGE_NAME" \
    "$REPO_ROOT"

# ── Build extension ───────────────────────────────────────────────────────────
echo ""
echo "==> Building VS Code extension inside Docker…"
docker run --rm \
    --volume "$REPO_ROOT:/workspace" \
    "$IMAGE_NAME"

echo ""
echo "==> Output:"
ls "$REPO_ROOT/dist/"*.vsix 2>/dev/null || true
