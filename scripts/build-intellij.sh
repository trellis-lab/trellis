#!/usr/bin/env bash
# scripts/build-intellij.sh
#
# Build the Trellis IntelliJ plugin in one step:
#   1. Compile the trellis-wasm crate (bundler target) → plugin resources/wasm/
#   2. Run ./gradlew buildPlugin → build/distributions/*.zip
#
# Requires:
#   - Rust toolchain + wasm-pack  (https://rustwasm.github.io/wasm-pack/installer/)
#   - JDK 17+
#   - Internet access on first run (Gradle downloads dependencies)
#
# Optional:
#   - wasm-opt (https://github.com/WebAssembly/binaryen) for release size optimisation
#
# Usage:
#   ./scripts/build-intellij.sh            # debug WASM + Gradle build
#   ./scripts/build-intellij.sh --release  # optimised WASM + Gradle build
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
WASM_CRATE="$REPO_ROOT/crates/trellis-wasm"
IJ_DIR="$REPO_ROOT/editors/intellij"
IJ_WASM_OUT="$IJ_DIR/src/main/resources/wasm"

RELEASE_FLAG=""
WASM_PACK_PROFILE="dev"
GRADLE_TASK="buildPlugin"

if [[ "${1:-}" == "--release" ]]; then
    RELEASE_FLAG="--release"
    WASM_PACK_PROFILE="release"
fi

# ── Prerequisite checks ───────────────────────────────────────────────────────

if ! command -v wasm-pack &>/dev/null; then
    echo "ERROR: wasm-pack not found." >&2
    echo "       Install it with:  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh" >&2
    exit 1
fi

if ! command -v java &>/dev/null; then
    echo "ERROR: java not found. Install JDK 17+ and ensure it is on PATH." >&2
    exit 1
fi

# ── Step 1 – WASM (bundler target for Chicory / JVM classpath) ───────────────

echo "==> [1/2] Building trellis-wasm ($WASM_PACK_PROFILE) → $IJ_WASM_OUT"
mkdir -p "$IJ_WASM_OUT"

wasm-pack build $RELEASE_FLAG \
    --target bundler \
    --out-dir "$IJ_WASM_OUT" \
    --out-name trellis_wasm \
    "$WASM_CRATE"

# Optional size optimisation (release only)
if [[ "$WASM_PACK_PROFILE" == "release" ]]; then
    WASM_FILE="$IJ_WASM_OUT/trellis_wasm_bg.wasm"
    if command -v wasm-opt &>/dev/null; then
        echo "--> Running wasm-opt -O3 on $WASM_FILE"
        wasm-opt -O3 "$WASM_FILE" -o "$WASM_FILE"
    else
        echo "--> wasm-opt not found, skipping size optimisation."
    fi
fi

echo "    WASM artefacts:"
ls -lh "$IJ_WASM_OUT"/*.wasm 2>/dev/null || true

# ── Step 2 – Gradle buildPlugin ──────────────────────────────────────────────

echo ""
echo "==> [2/2] Running ./gradlew $GRADLE_TASK in $IJ_DIR"
cd "$IJ_DIR"

# Use the wrapper if present; fall back to system gradle
GRADLEW="./gradlew"
if [[ ! -x "$GRADLEW" ]]; then
    if command -v gradle &>/dev/null; then
        GRADLEW="gradle"
    else
        echo "ERROR: Neither ./gradlew nor 'gradle' found." >&2
        echo "       Run: gradle wrapper --gradle-version 8.10  (inside editors/intellij/)" >&2
        exit 1
    fi
fi

"$GRADLEW" "$GRADLE_TASK"

# ── Report output ─────────────────────────────────────────────────────────────

echo ""
echo "==> Build complete."
echo "    Plugin ZIP:"
ls -lh "$IJ_DIR/build/distributions/"*.zip 2>/dev/null \
    || echo "    (no .zip found – check Gradle output above)"
