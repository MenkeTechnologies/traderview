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
echo "==> Frontend JS tests (vitest)"
# vitest covers the *.spec.js files. Skipped if frontend/node_modules is
# missing — run `cd frontend && pnpm install` (or npm install) to enable.
if [ -d frontend/node_modules ]; then
    (cd frontend && npx vitest run)
else
    echo "    skipped: frontend/node_modules missing. Run 'cd frontend && pnpm install' to enable."
fi

echo
echo "==> Backend integration tests (real Postgres via postgresql_embedded)"
# Integration tests download an embedded PG on first run (~80MB, cached).
# Single-threaded because the harness shares one PG instance per binary.
# Skip via NO_DB_TESTS=1 in environments where PG cannot start.
if [ "${NO_DB_TESTS:-}" = "1" ]; then
    echo "    skipped: NO_DB_TESTS=1"
else
    cargo test -p traderview-db --test integration -- --test-threads=1
fi

echo
echo "==> ALL TESTS PASSED"
