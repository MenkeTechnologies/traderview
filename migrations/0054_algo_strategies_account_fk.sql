-- 0054 — Bind algo strategies to a real broker account.
--
-- Every algo strategy now produces fills against an existing `accounts`
-- row, so the standard `executions` → `trades::rollup_account` pipeline
-- materializes algo activity into the same tables real broker trades
-- live in. All existing dashboards / reports / portfolio views surface
-- algo trades automatically once strategies are bound.
--
-- Column is added NULLABLE here because any rows created during the
-- 0052/0053 dev window pre-date the FK. The route layer treats NULL as
-- invalid (refuses to start / submit), so unbound strategies are inert
-- until the user picks an account.

ALTER TABLE algo_strategies
    ADD COLUMN IF NOT EXISTS account_id UUID
        REFERENCES accounts(id) ON DELETE RESTRICT;

CREATE INDEX IF NOT EXISTS algo_strategies_account_idx
    ON algo_strategies(account_id) WHERE account_id IS NOT NULL;

-- Rollback (manual):
--   DROP INDEX algo_strategies_account_idx;
--   ALTER TABLE algo_strategies DROP COLUMN account_id;
