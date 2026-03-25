#!/usr/bin/env bash
# docker/vscode-entrypoint.sh
#
# Runs inside the trellis-vscode-builder container.
# Produces a .vsix package and copies it to /workspace/dist/.
set -euo pipefail

WASM_DIR="/workspace/editors/vscode/wasm"
DIST_DIR="/workspace/dist"

# ── Guard: WASM must already be built ─────────────────────────────────────────
if [[ ! -f "$WASM_DIR/trellis_wasm_bg.wasm" ]]; then
    echo "Error: WASM binary not found at $WASM_DIR/trellis_wasm_bg.wasm" >&2
    echo "       Run scripts/build-wasm-docker.sh before building the extension." >&2
    exit 1
fi

cd /workspace/editors/vscode

# ── Ensure icon exists (vsce fails if media/icon.png is missing) ──────────────
ICON="media/icon.png"
if [[ ! -f "$ICON" ]]; then
    echo "--> media/icon.png not found; copying from images/logo.png"
    mkdir -p media
    cp /workspace/images/logo.png "$ICON"
fi

# ── Install dependencies ───────────────────────────────────────────────────────
echo "==> Installing npm dependencies…"
npm install

# ── Compile TypeScript ─────────────────────────────────────────────────────────
echo "==> Compiling TypeScript…"
npm run compile

# ── Package extension ─────────────────────────────────────────────────────────
echo "==> Packaging VS Code extension…"
npm run package

# ── Copy output to dist/ ──────────────────────────────────────────────────────
mkdir -p "$DIST_DIR"
cp ./*.vsix "$DIST_DIR/"

echo ""
echo "==> VS Code extension built:"
ls "$DIST_DIR"/*.vsix
