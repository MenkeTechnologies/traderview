#!/usr/bin/env bash
# Vendor uPlot dist files into frontend/lib/. Run once per checkout.
# No curl-pipe-bash — pulls via npm pack and stages.
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/cyberpunk.sh

LIB="frontend/lib"
mkdir -p "$LIB"

TMPDIR_X="$(mktemp -d)"
trap 'rm -rf "$TMPDIR_X"' EXIT

cyber_banner
cyber_status "OPERATION" "VENDOR uPlot // npm pack → frontend/lib"
echo

cyber_section "FETCH"
( cd "$TMPDIR_X" && npm pack uplot --silent )
TGZ=("$TMPDIR_X"/uplot-*.tgz)
cyber_ok "fetched $(basename "${TGZ[0]}")"
echo

cyber_section "EXTRACT"
( cd "$TMPDIR_X" && tar -xzf "$(basename "${TGZ[0]}")" )
cyber_ok "extracted"
echo

cyber_section "STAGE"
cp "$TMPDIR_X/package/dist/uPlot.iife.min.js" "$LIB/uPlot.iife.min.js"
cp "$TMPDIR_X/package/dist/uPlot.min.css"     "$LIB/uPlot.min.css"
cyber_ok "staged uPlot.iife.min.js + uPlot.min.css → $LIB"

cyber_tagline "uPlot ONLINE."
cyber_line
