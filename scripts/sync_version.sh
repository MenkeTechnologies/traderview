#!/usr/bin/env bash
# Propagate the workspace version (Cargo.toml) into every other file that
# carries a hand-written version literal. Run AFTER `cargo set-version` /
# manual edit of the workspace [package] version, BEFORE `git commit` so the
# bump lands in a single commit. Idempotent — safe to run twice.
#
# Files synced:
#   - frontend/index.html   (the topbar version chip literal)
#   - frontend/package.json ("version" field)
#
# Usage:
#   scripts/sync_version.sh            # read version from Cargo.toml
#   scripts/sync_version.sh 1.2.3      # use explicit version

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

if [[ $# -ge 1 ]]; then
    VERSION="$1"
else
    VERSION=$(awk '
        /^\[(workspace\.)?package\]/ { in_pkg = 1; next }
        /^\[/                        { in_pkg = 0 }
        in_pkg && /^version[[:space:]]*=/ {
            match($0, /"[^"]+"/)
            print substr($0, RSTART+1, RLENGTH-2)
            exit
        }
    ' Cargo.toml)
fi

if ! [[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+ ]]; then
    echo "sync_version: refusing to write garbage version '$VERSION'" >&2
    exit 1
fi

echo "syncing version → $VERSION"

# index.html — the literal lives inside the #tv-version anchor. The
# regex captures the opening tag's attributes (which can carry their
# own newlines), then matches `vMAJ.MIN.PATCH[suffix]` immediately
# before `</a>` so a stray vX.Y.Z literal anywhere else on the page
# is untouched.
# -0777 = slurp the whole file. Required because the anchor's opening
# tag and its closing </a> sit on different lines.
V="$VERSION" perl -i -0777 -pe 's{(id="tv-version"[\s\S]*?>)v\d+\.\d+\.\d+[A-Za-z0-9.+-]*(</a>)}{$1.q{v}.$ENV{V}.$2}e' \
    frontend/index.html

# frontend/package.json — top-level "version" field. Anchored to a
# leading double-space so we only touch the root-level key, never a
# nested devDependency version literal.
V="$VERSION" perl -i -pe 's{^(  "version"\s*:\s*)"[^"]*"}{$1.q{"}.$ENV{V}.q{"}}e' \
    frontend/package.json

echo "  frontend/index.html   → v$VERSION"
echo "  frontend/package.json → $VERSION"
