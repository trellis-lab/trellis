#!/usr/bin/env bash
# docker/wasm-entrypoint.sh
#
# Runs inside the trellis-wasm-builder container.
# Builds the trellis-wasm crate for every editor target and writes the output
# directly into the mounted workspace so the host filesystem is updated.
#
# Usage (via docker run):
#   docker run --rm -v "$(pwd):/workspace" trellis-wasm-builder           # dev
#   docker run --rm -v "$(pwd):/workspace" trellis-wasm-builder --release  # optimised
set -euo pipefail

RELEASE_FLAG=""
WASM_PACK_PROFILE="dev"
if [[ "${1:-}" == "--release" ]]; then
    RELEASE_FLAG="--release"
    WASM_PACK_PROFILE="release"
fi

WASM_CRATE="/workspace/crates/trellis-wasm"

# Editor output directories (host paths via the volume mount)
VSCODE_OUT="/workspace/editors/vscode/wasm"
IJ_OUT="/workspace/editors/intellij/src/main/resources/wasm"

echo "==> Building trellis-wasm ($WASM_PACK_PROFILE) …"

mkdir -p "$VSCODE_OUT" "$IJ_OUT"

# ── VS Code extension (web / ES module) ───────────────────────────────────────
echo "--> Target: web  →  $VSCODE_OUT"
wasm-pack build $RELEASE_FLAG \
    --target web \
    --out-dir "$VSCODE_OUT" \
    --out-name trellis_wasm \
    "$WASM_CRATE"

# ── IntelliJ plugin (web / ES module – loaded by JCEF's Chromium engine) ──────
echo "--> Target: web  →  $IJ_OUT"
wasm-pack build $RELEASE_FLAG \
    --target web \
    --out-dir "$IJ_OUT" \
    --out-name trellis_wasm \
    "$WASM_CRATE"

# ── Optional: wasm-opt size optimisation (release builds only) ────────────────
if [[ "$WASM_PACK_PROFILE" == "release" ]] && command -v wasm-opt &>/dev/null; then
    for WASM_FILE in \
        "$VSCODE_OUT/trellis_wasm_bg.wasm" \
        "$IJ_OUT/trellis_wasm_bg.wasm"; do
        echo "--> wasm-opt -O3  $WASM_FILE"
        wasm-opt -O3 "$WASM_FILE" -o "$WASM_FILE"
    done
fi

echo ""
echo "==> WASM build complete."
echo "    VS Code :  $VSCODE_OUT"
echo "    IntelliJ:  $IJ_OUT"
