-- 0064 — soft-delete for algo_strategies.
--
-- Hard-delete via FK CASCADE was wiping the user's entire run +
-- order + fill history whenever a strategy was removed. Even an
-- accidental delete would silently destroy weeks of paper-trading
-- data. We switch to a tombstone column so the row stays + the
-- cascade chain stays intact.
--
-- Rules after this migration:
--   * delete_strategy() now UPDATEs deleted_at = now() instead of
--     issuing DELETE; the row persists.
--   * list_strategies() filters `deleted_at IS NULL` so deleted ones
--     drop out of the strategies table in the UI.
--   * list_active_strategies() (engine) also filters so the runner
--     stops ticking on tombstoned configs.
--   * get_strategy() does NOT filter so the runs panel can still
--     resolve the strategy NAME for historical runs (otherwise a
--     deleted strategy's run history would render with "Unknown
--     strategy").
--   * Restore = `UPDATE algo_strategies SET deleted_at = NULL`. No
--     dedicated route today — manual SQL only.

ALTER TABLE algo_strategies
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS algo_strategies_active_idx
    ON algo_strategies (user_id)
 WHERE deleted_at IS NULL;

-- Rollback (manual):
--   DROP INDEX algo_strategies_active_idx;
--   ALTER TABLE algo_strategies DROP COLUMN deleted_at;
