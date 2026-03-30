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

# Resolve wasm-pack binary – on Windows/Git Bash the .exe suffix is needed in scripts
if command -v wasm-pack &>/dev/null; then
    WASM_PACK=wasm-pack
elif command -v wasm-pack.exe &>/dev/null; then
    WASM_PACK=wasm-pack.exe
else
    echo "Error: wasm-pack not found. Install from https://rustwasm.github.io/wasm-pack/installer/" >&2
    exit 1
fi

echo "==> Building trellis-wasm ($WASM_PACK_PROFILE)..."

# ── VS Code extension target (ES module / web) ────────────────────────────────
VSCODE_OUT="$REPO_ROOT/editors/vscode/wasm"
echo "--> Target: web  →  $VSCODE_OUT"
$WASM_PACK build $RELEASE_FLAG \
    --target web \
    --out-dir "$VSCODE_OUT" \
    --out-name trellis_wasm \
    "$WASM_CRATE"

# ── IntelliJ plugin target (web / ES module – loaded by JCEF's Chromium engine)
IJ_OUT="$REPO_ROOT/editors/intellij/src/main/resources/wasm"
echo "--> Target: web  →  $IJ_OUT"
$WASM_PACK build $RELEASE_FLAG \
    --target web \
    --out-dir "$IJ_OUT" \
    --out-name trellis_wasm \
    "$WASM_CRATE"

# ── Optional: wasm-opt size optimisation (release only) ───────────────────────
if [[ "$WASM_PACK_PROFILE" == "release" ]]; then
    for WASM_FILE in \
        "$VSCODE_OUT/trellis_wasm_bg.wasm" \
        "$IJ_OUT/trellis_wasm_bg.wasm"; do
        WASM_OPT=$(command -v wasm-opt || command -v wasm-opt.exe || true)
        if [[ -n "$WASM_OPT" ]]; then
            echo "--> Running wasm-opt on $WASM_FILE"
            "$WASM_OPT" -O3 "$WASM_FILE" -o "$WASM_FILE"
        else
            echo "--> wasm-opt not found, skipping size optimisation for $WASM_FILE"
        fi
    done
fi

echo "==> WASM build complete."
echo "    VS Code:   $VSCODE_OUT"
echo "    IntelliJ:  $IJ_OUT"
