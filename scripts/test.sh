#!/usr/bin/env bash
# Run the targeted test suite — fast crates (no DB) + frontend JS tests.
# traderview-web's full route handlers need a real PG, but its inline unit
# tests (rate_limit, receipt_routes helpers, etc.) don't touch the pool
# and run in milliseconds.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> Rust unit tests (core + import + expense + ocr + db + web helpers)"
cargo test \
    -p traderview-core \
    -p traderview-import \
    -p traderview-expense \
    -p traderview-ocr \
    -p traderview-db \
    -p traderview-web \
    --lib \
    "$@"

echo
echo "==> Frontend JS tests (node --test)"
node --test frontend/tests/*.test.mjs

echo
echo "==> ALL TESTS PASSED"
