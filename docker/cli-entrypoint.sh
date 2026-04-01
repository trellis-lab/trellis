#!/usr/bin/env bash
# docker/cli-entrypoint.sh
#
# Runs inside the trellis-cli-builder container.
# Cross-compiles the Trellis CLI for one or all supported targets.
#
# Usage (via docker run):
#   docker run --rm -v "$(pwd):/workspace" trellis-cli-builder
#   docker run --rm -v "$(pwd):/workspace" trellis-cli-builder --target x86_64-unknown-linux-gnu
set -euo pipefail

DIST_DIR="/workspace/dist"
TARGET=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --target) TARGET="$2"; shift 2 ;;
        *) echo "Unknown argument: $1" >&2; exit 1 ;;
    esac
done

build_target() {
    local target="$1"
    local bin_name="trellis"
    local ext=""

    if [[ "$target" == *"windows"* ]]; then
        ext=".exe"
    fi

    echo "==> Building trellis for $target …"

    local cargo_args=(
        build --release
        -p trellis-cli
        --target "$target"
    )

    # Windows cross-compile linker config
    if [[ "$target" == "x86_64-pc-windows-gnu" ]]; then
        export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER="x86_64-w64-mingw32-gcc"
    fi

    cargo "${cargo_args[@]}"

    local out_dir="$DIST_DIR/$target"
    mkdir -p "$out_dir"
    cp "target/$target/release/${bin_name}${ext}" "$out_dir/"

    echo "    -> $out_dir/${bin_name}${ext}"
}

mkdir -p "$DIST_DIR"

if [[ -n "$TARGET" ]]; then
    build_target "$TARGET"
else
    # Build all Docker-supported targets (macOS requires native or CI runner)
    build_target "x86_64-unknown-linux-gnu"
    build_target "x86_64-pc-windows-gnu"
fi

echo ""
echo "==> CLI build complete. Binaries in:"
ls -R "$DIST_DIR"/*/trellis* 2>/dev/null || true
