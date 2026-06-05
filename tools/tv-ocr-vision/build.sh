#!/usr/bin/env bash
# Build the Vision OCR sidecar binary and copy it into a place the Rust
# OCR engine knows to look. Safe to run on Linux (just no-ops).
#
# Idempotent — re-running with no source changes is a sub-second SPM cache hit.

set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"

# Linux / non-Apple builds: bail cleanly so CI doesn't fail.
if [[ "$(uname -s)" != "Darwin" ]]; then
    echo "tv-ocr-vision: not macOS, skipping (Vision is Apple-only)" >&2
    exit 0
fi

if ! command -v swift >/dev/null 2>&1; then
    echo "tv-ocr-vision: 'swift' not on PATH — install Xcode Command Line Tools" >&2
    exit 1
fi

echo "tv-ocr-vision: building (swift build -c release)…"
swift build -c release

SRC="$DIR/.build/release/tv-ocr-vision"
if [[ ! -x "$SRC" ]]; then
    echo "tv-ocr-vision: build succeeded but $SRC missing or not executable" >&2
    exit 1
fi

# Publish to the workspace's target/release-sidecars so the Rust engine can
# find it via a stable relative path regardless of what triggered the build.
WORKSPACE="$(cd "$DIR/../.." && pwd)"
DEST_DIR="$WORKSPACE/target/release-sidecars"
mkdir -p "$DEST_DIR"
cp -f "$SRC" "$DEST_DIR/tv-ocr-vision"

echo "tv-ocr-vision: installed → $DEST_DIR/tv-ocr-vision"
