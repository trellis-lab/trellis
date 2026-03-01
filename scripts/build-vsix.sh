#!/usr/bin/env bash
# scripts/build-vsix.sh
#
# Build the VS Code .vsix extension package inside Docker.
# Requires Docker to be running.
#
# Usage:
#   ./scripts/build-vsix.sh              # outputs to ./dist/
#   ./scripts/build-vsix.sh /some/path   # outputs to /some/path/
#
# The Docker image is rebuilt only when sources change (layer cache).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

IMAGE="trellis-vsix-builder"
DOCKERFILE="$REPO_ROOT/docker/Dockerfile.vsix"
OUTPUT_DIR="${1:-$REPO_ROOT/dist}"

# ── Sanity checks ──────────────────────────────────────────────────────────────
if ! command -v docker &>/dev/null; then
    echo "ERROR: docker is not installed or not in PATH." >&2
    exit 1
fi

if ! docker info &>/dev/null; then
    echo "ERROR: Docker daemon is not running." >&2
    exit 1
fi

# ── Build the Docker image ─────────────────────────────────────────────────────
echo "==> Building Docker image '$IMAGE'..."
echo "    Dockerfile: $DOCKERFILE"
echo "    Context:    $REPO_ROOT"
echo ""

docker build \
    --file  "$DOCKERFILE" \
    --tag   "$IMAGE" \
    --label "org.opencontainers.image.source=https://github.com/trellis/trellis" \
    "$REPO_ROOT"

# ── Extract the .vsix ─────────────────────────────────────────────────────────
mkdir -p "$OUTPUT_DIR"

echo ""
echo "==> Extracting .vsix to '$OUTPUT_DIR'..."

docker run --rm \
    --volume "$OUTPUT_DIR:/output" \
    "$IMAGE"

# ── Report ────────────────────────────────────────────────────────────────────
echo ""
VSIX_FILE=$(ls "$OUTPUT_DIR"/*.vsix 2>/dev/null | head -1 || true)
if [[ -n "$VSIX_FILE" ]]; then
    SIZE=$(du -h "$VSIX_FILE" | cut -f1)
    echo "==> Success!  $VSIX_FILE  ($SIZE)"
    echo ""
    echo "    Install locally:"
    echo "      code --install-extension \"$VSIX_FILE\""
    echo ""
    echo "    Or: VS Code › Extensions › ⋯ › Install from VSIX…"
else
    echo "ERROR: No .vsix file found in '$OUTPUT_DIR'." >&2
    exit 1
fi
