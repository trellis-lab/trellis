#!/usr/bin/env bash
# scripts/build-wasm.sh
#
# Build the trellis-wasm crate for all editor targets.
# Requires: wasm-pack (https://rustwasm.github.io/wasm-pack/installer/)
# Optional: wasm-opt (https://github.com/WebAssembly/binaryen) for size optimisation.
#
# Usage:
#   ./scripts/build-wasm.sh           # debug build
#   ./scripts/build-wasm.sh --release # optimised release build
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
WASM_CRATE="$REPO_ROOT/crates/trellis-wasm"

RELEASE_FLAG=""
WASM_PACK_PROFILE="dev"
if [[ "${1:-}" == "--release" ]]; then
    RELEASE_FLAG="--release"
    WASM_PACK_PROFILE="release"
fi

echo "==> Building trellis-wasm ($WASM_PACK_PROFILE)..."

# ── VS Code extension target (ES module / web) ────────────────────────────────
VSCODE_OUT="$REPO_ROOT/editors/vscode/wasm"
echo "--> Target: web  →  $VSCODE_OUT"
wasm-pack build $RELEASE_FLAG \
    --target web \
    --out-dir "$VSCODE_OUT" \
    --out-name trellis_wasm \
    "$WASM_CRATE"

# ── IntelliJ plugin target (bundler / node-compatible) ────────────────────────
IJ_OUT="$REPO_ROOT/editors/intellij/src/main/resources/wasm"
echo "--> Target: bundler  →  $IJ_OUT"
wasm-pack build $RELEASE_FLAG \
    --target bundler \
    --out-dir "$IJ_OUT" \
    --out-name trellis_wasm \
    "$WASM_CRATE"

# ── Optional: wasm-opt size optimisation (release only) ───────────────────────
if [[ "$WASM_PACK_PROFILE" == "release" ]]; then
    for WASM_FILE in \
        "$VSCODE_OUT/trellis_wasm_bg.wasm" \
        "$IJ_OUT/trellis_wasm_bg.wasm"; do
        if command -v wasm-opt &>/dev/null; then
            echo "--> Running wasm-opt on $WASM_FILE"
            wasm-opt -O3 "$WASM_FILE" -o "$WASM_FILE"
        else
            echo "--> wasm-opt not found, skipping size optimisation for $WASM_FILE"
        fi
    done
fi

echo "==> WASM build complete."
echo "    VS Code:   $VSCODE_OUT"
echo "    IntelliJ:  $IJ_OUT"
