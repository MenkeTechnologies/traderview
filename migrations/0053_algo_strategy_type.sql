-- 0053 — Strategy type discriminator on algo_strategies.
--
-- Lets a single algo_strategies row pick which Strategy impl in
-- traderview-core::algo_strategies the engine will run. Defaults to
-- 'momentum' so every pre-existing row continues to behave identically.
--
-- The CHECK list is the closed set of strategies the factory in
-- traderview-core::algo_strategies::from_kind accepts. New kinds added
-- there need to be added here in a follow-up migration before they can
-- be persisted.

ALTER TABLE algo_strategies
    ADD COLUMN IF NOT EXISTS strategy_type TEXT NOT NULL DEFAULT 'momentum'
        CHECK (strategy_type IN ('momentum', 'mean_reversion', 'orb',
                                 'donchian_trend', 'bb_squeeze'));

CREATE INDEX IF NOT EXISTS algo_strategies_kind_idx
    ON algo_strategies(strategy_type, enabled);

-- Rollback (manual):
--   ALTER TABLE algo_strategies DROP COLUMN strategy_type;
--   DROP INDEX algo_strategies_kind_idx;
