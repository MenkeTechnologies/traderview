#!/usr/bin/env bash
# Run the targeted test suite — fast crates (no DB) + frontend JS tests.
# Skips traderview-web (axum routes need a real PG) — exercise those by
# running the full app instead.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> Rust unit tests (core + import + expense + ocr + db)"
cargo test \
    -p traderview-core \
    -p traderview-import \
    -p traderview-expense \
    -p traderview-ocr \
    -p traderview-db \
    --lib \
    "$@"

echo
echo "==> Frontend JS tests (node --test)"
node --test frontend/tests/*.test.mjs

echo
echo "==> ALL TESTS PASSED"
