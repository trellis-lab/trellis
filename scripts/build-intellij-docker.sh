#!/usr/bin/env bash
# scripts/build-intellij-docker.sh
#
# Builds the Trellis IntelliJ plugin (.zip) inside Docker.
# The WASM binary must already be present (run build-wasm-docker.sh first).
#
# Gradle and the IntelliJ sandbox are cached in a named Docker volume
# (trellis-gradle-cache) so subsequent builds are fast.
#
# Usage:
#   ./scripts/build-intellij-docker.sh               # normal build
#   ./scripts/build-intellij-docker.sh --clean-cache # wipe Gradle cache then build
#
# If you see "Input/output error" from Gradle, run with --clean-cache to recover.
#
# Output: dist/trellis-intellij-<version>.zip
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
IMAGE_NAME="trellis-intellij-builder"
GRADLE_CACHE_VOLUME="trellis-gradle-cache"
WASM_FILE="$REPO_ROOT/editors/intellij/src/main/resources/wasm/trellis_wasm_bg.wasm"

# ── Flags ─────────────────────────────────────────────────────────────────────
CLEAN_CACHE=false
for arg in "$@"; do
    [[ "$arg" == "--clean-cache" ]] && CLEAN_CACHE=true
done

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

# ── Optionally wipe the Gradle cache volume ───────────────────────────────────
if [[ "$CLEAN_CACHE" == true ]]; then
    echo "==> Removing Gradle cache volume '$GRADLE_CACHE_VOLUME'…"
    docker volume rm "$GRADLE_CACHE_VOLUME" 2>/dev/null || true
    echo "    Cache cleared. Gradle and IntelliJ sandbox will be re-downloaded."
    echo ""
fi

# ── Build image ───────────────────────────────────────────────────────────────
echo "==> Building Docker image '$IMAGE_NAME'…"
docker build \
    --file "$REPO_ROOT/docker/Dockerfile.intellij-builder" \
    --tag  "$IMAGE_NAME" \
    "$REPO_ROOT"

# ── Build plugin ──────────────────────────────────────────────────────────────
echo ""
echo "==> Building IntelliJ plugin inside Docker…"
echo "    Gradle cache volume: $GRADLE_CACHE_VOLUME"
echo "    (first run downloads Gradle + IntelliJ sandbox; subsequent runs use the cache)"
docker run --rm \
    --volume "$REPO_ROOT:/workspace" \
    --volume "$GRADLE_CACHE_VOLUME:/root/.gradle" \
    "$IMAGE_NAME"

echo ""
echo "==> Output:"
ls "$REPO_ROOT/dist/"*.zip 2>/dev/null || true
