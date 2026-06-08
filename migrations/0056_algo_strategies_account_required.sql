-- 0056 — Enforce account binding at the SQL level. Unbound strategies
-- are useless (engine refuses to run them, route refuses to start runs)
-- so promoting account_id from nullable to NOT NULL closes the
-- remaining gap. No way to land an unbound row even with a hand-rolled
-- INSERT.
--
-- Cleanup: any pre-0054 dev rows that lack an account_id get deleted
-- here. They couldn't run anyway. The cascading FK on algo_runs +
-- algo_orders + algo_fills + algo_kill_switch_audit drags any
-- dependent rows with them.

DELETE FROM algo_strategies WHERE account_id IS NULL;

ALTER TABLE algo_strategies
    ALTER COLUMN account_id SET NOT NULL;

-- Rollback (manual):
--   ALTER TABLE algo_strategies ALTER COLUMN account_id DROP NOT NULL;
