#!/usr/bin/env zsh
# Vendor uPlot dist files into frontend/lib/. Run once per checkout.
# No curl-pipe-bash — pulls via npm pack and stages.

set -e
HERE="${0:A:h}"
ROOT="${HERE:h}"
LIB="$ROOT/frontend/lib"
mkdir -p "$LIB"

TMPDIR="$(mktemp -d)"
trap "rm -rf $TMPDIR" EXIT

cd "$TMPDIR"
npm pack uplot --silent
TGZ=(uplot-*.tgz)
tar -xzf "$TGZ[1]"

cp package/dist/uPlot.iife.min.js "$LIB/uPlot.iife.min.js"
cp package/dist/uPlot.min.css "$LIB/uPlot.min.css"
print -- "vendored uPlot -> $LIB"
