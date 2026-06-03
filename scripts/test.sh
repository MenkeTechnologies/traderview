#!/usr/bin/env bash
# Run the targeted test suite — fast crates (no DB) + frontend JS tests.
# traderview-web's full route handlers need a real PG, but its inline unit
# tests (rate_limit, receipt_routes helpers, etc.) don't touch the pool
# and run in milliseconds.
set -euo pipefail
cd "$(dirname "$0")/.."
source scripts/cyberpunk.sh

cyber_banner
cyber_status "OPERATION" "RUN ALL TESTS // rust + frontend + integration"
echo

START=$(date +%s)

cyber_section "RUST UNIT TESTS (core + import + expense + ocr + db + web helpers)"
cargo test \
    -p traderview-core \
    -p traderview-import \
    -p traderview-expense \
    -p traderview-ocr \
    -p traderview-db \
    -p traderview-web \
    --lib \
    "$@"
cyber_ok "rust unit tests passed"
echo

cyber_section "FRONTEND JS TESTS (node --test)"
node --test frontend/tests/*.test.mjs
cyber_ok "node tests passed"
echo

cyber_section "FRONTEND JS TESTS (vitest)"
if [ -d frontend/node_modules ]; then
    (cd frontend && npx vitest run)
    cyber_ok "vitest passed"
else
    cyber_warn "skipped: frontend/node_modules missing. Run 'cd frontend && pnpm install' to enable."
fi
echo

cyber_section "BACKEND INTEGRATION TESTS (real Postgres via postgresql_embedded)"
# Integration tests download an embedded PG on first run (~80MB, cached).
# Single-threaded because the harness shares one PG instance per binary.
# Skip via NO_DB_TESTS=1 in environments where PG cannot start.
if [ "${NO_DB_TESTS:-}" = "1" ]; then
    cyber_warn "skipped: NO_DB_TESTS=1"
else
    cargo test -p traderview-db --test integration -- --test-threads=1
    cyber_ok "integration tests passed"
fi

END=$(date +%s)
ELAPSED=$((END - START))

echo
cyber_status "RESULT" "ALL TESTS PASSED // ${ELAPSED}s"
cyber_tagline "SIGNAL CLEAN. SHIP IT."
cyber_line
