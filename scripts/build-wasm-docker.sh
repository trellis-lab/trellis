#!/usr/bin/env bash
# scripts/build-wasm-docker.sh
#
# OS-agnostic WASM builder: builds the trellis-wasm crate inside a Docker
# container and copies the output directly into the editor source trees.
#
# Requirements: Docker (Desktop or Engine) — no Rust or wasm-pack needed locally.
#
# Usage:
#   ./scripts/build-wasm-docker.sh           # dev build
#   ./scripts/build-wasm-docker.sh --release # optimised release build
#
# Works from: Git Bash, WSL, macOS Terminal, Linux shell, PowerShell (via bash).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

IMAGE_NAME="trellis-wasm-builder"
RELEASE_FLAG="${1:-}"

# ── Sanity check ──────────────────────────────────────────────────────────────
if ! command -v docker &>/dev/null; then
    echo "Error: docker not found. Install Docker Desktop from https://www.docker.com/products/docker-desktop/" >&2
    exit 1
fi

if ! docker info &>/dev/null; then
    echo "Error: Docker daemon is not running. Start Docker Desktop and try again." >&2
    exit 1
fi

# ── Build the image (cached after the first run) ──────────────────────────────
echo "==> Building Docker image '$IMAGE_NAME' …"
echo "    (this is slow on the first run; subsequent runs use the layer cache)"
docker build \
    --file "$REPO_ROOT/docker/Dockerfile.wasm-builder" \
    --tag  "$IMAGE_NAME" \
    "$REPO_ROOT"

# ── Run the WASM build inside the container ───────────────────────────────────
# The repo root is mounted as /workspace so wasm-pack output lands directly
# in editors/vscode/wasm/ and editors/intellij/.../wasm/ on the host.
echo ""
echo "==> Running WASM build inside Docker …"

# Windows (Git Bash / MSYS2) path fix: convert to Windows format for Docker Desktop
DOCKER_VOLUME="$REPO_ROOT"
if command -v cygpath &>/dev/null; then
    DOCKER_VOLUME="$(cygpath -w "$REPO_ROOT")"
fi

MSYS_NO_PATHCONV=1 docker run --rm \
    --volume "$DOCKER_VOLUME:/workspace" \
    "$IMAGE_NAME" \
    $RELEASE_FLAG

echo ""
echo "==> All done. WASM files are in:"
echo "    editors/vscode/wasm/"
echo "    editors/intellij/src/main/resources/wasm/"
