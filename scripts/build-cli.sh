#!/usr/bin/env bash
# scripts/build-cli.sh
#
# Cross-compiles the Trellis CLI for multiple platforms.
#
# Docker mode (default): builds Linux x86_64 + Windows x86_64 inside Docker.
# Native mode (--native): builds for the host platform only (no Docker needed).
# CI mode (--ci): builds for a specific target (used by GitHub Actions matrix).
#
# Usage:
#   ./scripts/build-cli.sh                              # Docker: Linux + Windows
#   ./scripts/build-cli.sh --native                     # Host platform only
#   ./scripts/build-cli.sh --ci --target <TRIPLE>       # Single target for CI
#
# macOS builds (x86_64 + ARM) are handled by GitHub Actions runners
# since cross-compiling for macOS from Linux requires osxcross.
#
# Output: dist/<target>/trellis[.exe]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
IMAGE_NAME="trellis-cli-builder"
DIST_DIR="$REPO_ROOT/dist"

MODE="docker"
CI_TARGET=""

# ── Parse arguments ──────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case "$1" in
        --native)  MODE="native"; shift ;;
        --ci)      MODE="ci"; shift ;;
        --target)  CI_TARGET="$2"; shift 2 ;;
        -h|--help)
            echo "Usage: $0 [--native | --ci --target <TRIPLE>]"
            echo ""
            echo "Modes:"
            echo "  (default)    Docker: Linux x86_64 + Windows x86_64"
            echo "  --native     Host platform only (no Docker)"
            echo "  --ci         Single target (for CI matrix builds)"
            echo ""
            echo "Targets (CI mode):"
            echo "  x86_64-unknown-linux-gnu"
            echo "  x86_64-apple-darwin"
            echo "  aarch64-apple-darwin"
            echo "  x86_64-pc-windows-gnu"
            exit 0
            ;;
        *) echo "Unknown argument: $1" >&2; exit 1 ;;
    esac
done

# ── Native mode ──────────────────────────────────────────────────────────────
if [[ "$MODE" == "native" ]]; then
    echo "==> Building trellis CLI (native, release)…"
    cd "$REPO_ROOT"
    cargo build --release -p trellis-cli

    mkdir -p "$DIST_DIR"
    if [[ -f target/release/trellis.exe ]]; then
        cp target/release/trellis.exe "$DIST_DIR/"
    else
        cp target/release/trellis "$DIST_DIR/"
    fi

    echo "==> Done. Binary in: $DIST_DIR/"
    exit 0
fi

# ── CI mode ──────────────────────────────────────────────────────────────────
if [[ "$MODE" == "ci" ]]; then
    if [[ -z "$CI_TARGET" ]]; then
        echo "Error: --ci mode requires --target <TRIPLE>" >&2
        exit 1
    fi

    echo "==> Building trellis CLI for $CI_TARGET (release)…"
    cd "$REPO_ROOT"
    cargo build --release -p trellis-cli --target "$CI_TARGET"

    OUT_DIR="$DIST_DIR/$CI_TARGET"
    mkdir -p "$OUT_DIR"

    BIN_NAME="trellis"
    if [[ "$CI_TARGET" == *"windows"* ]]; then
        BIN_NAME="trellis.exe"
    fi
    cp "target/$CI_TARGET/release/$BIN_NAME" "$OUT_DIR/"

    echo "==> Done. Binary: $OUT_DIR/$BIN_NAME"
    exit 0
fi

# ── Docker mode ──────────────────────────────────────────────────────────────
if ! command -v docker &>/dev/null; then
    echo "Error: docker not found. Install Docker Desktop from https://www.docker.com/products/docker-desktop/" >&2
    exit 1
fi

if ! docker info &>/dev/null; then
    echo "Error: Docker daemon is not running. Start Docker Desktop and try again." >&2
    exit 1
fi

echo "==> Building Docker image '$IMAGE_NAME'…"
echo "    (first run installs cross-compilation toolchains — this may take a few minutes)"
docker build \
    --file "$REPO_ROOT/docker/Dockerfile.cli-builder" \
    --tag  "$IMAGE_NAME" \
    "$REPO_ROOT"

echo ""
echo "==> Cross-compiling CLI binaries inside Docker…"
docker run --rm \
    --volume "$REPO_ROOT:/workspace" \
    "$IMAGE_NAME"

echo ""
echo "==> Output:"
ls "$DIST_DIR"/*/trellis* 2>/dev/null || echo "(no binaries found)"
