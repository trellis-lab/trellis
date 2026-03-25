#!/usr/bin/env bash
# docker/intellij-entrypoint.sh
#
# Runs inside the trellis-intellij-builder container.
# Produces a plugin .zip and copies it to /workspace/dist/.
#
# Gradle and the IntelliJ sandbox are cached in the mounted
# trellis-gradle-cache volume to avoid re-downloading on every run.
set -euo pipefail

WASM_DIR="/workspace/editors/intellij/src/main/resources/wasm"
DIST_DIR="/workspace/dist"

# ── Guard: WASM must already be built ─────────────────────────────────────────
if [[ ! -f "$WASM_DIR/trellis_wasm_bg.wasm" ]]; then
    echo "Error: WASM binary not found at $WASM_DIR/trellis_wasm_bg.wasm" >&2
    echo "       Run scripts/build-wasm-docker.sh before building the plugin." >&2
    exit 1
fi

cd /workspace/editors/intellij

# ── Ensure Gradle wrapper exists ──────────────────────────────────────────────
# gradlew is not committed to source control; generate it on first run using
# the Gradle installation baked into the Docker image.
if [[ ! -f "gradlew" ]]; then
    echo "--> gradlew not found; generating Gradle wrapper (${GRADLE_VERSION})…"
    gradle wrapper --gradle-version "${GRADLE_VERSION}" --distribution-type bin
fi

# Ensure gradlew is executable (Windows hosts may strip the execute bit)
chmod +x gradlew

# ── Build plugin ───────────────────────────────────────────────────────────────
echo "==> Building IntelliJ plugin…"
echo "    (first run downloads Gradle + IntelliJ sandbox – this can take a few minutes)"
./gradlew buildPlugin --no-daemon

# ── Copy output to dist/ ──────────────────────────────────────────────────────
mkdir -p "$DIST_DIR"
cp build/distributions/*.zip "$DIST_DIR/"

echo ""
echo "==> IntelliJ plugin built:"
ls "$DIST_DIR"/*.zip
