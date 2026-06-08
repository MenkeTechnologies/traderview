-- 0055 — Widen algo_strategies.strategy_type CHECK to include the four
-- new families: ttm_squeeze, vwap_scalp, supertrend, heikin_ashi_trend.
--
-- Postgres doesn't let you ALTER a CHECK constraint in place; the
-- shortest atomic path is DROP + ADD inside a single migration so the
-- visible window with no constraint never spans more than one
-- statement of a transactional migration runner.

ALTER TABLE algo_strategies
    DROP CONSTRAINT IF EXISTS algo_strategies_strategy_type_check;

ALTER TABLE algo_strategies
    ADD CONSTRAINT algo_strategies_strategy_type_check
        CHECK (strategy_type IN (
            'momentum', 'mean_reversion', 'orb',
            'donchian_trend', 'bb_squeeze',
            'ttm_squeeze', 'vwap_scalp', 'supertrend', 'heikin_ashi_trend'
        ));

-- Rollback (manual):
--   ALTER TABLE algo_strategies DROP CONSTRAINT algo_strategies_strategy_type_check;
--   ALTER TABLE algo_strategies ADD CONSTRAINT algo_strategies_strategy_type_check
--       CHECK (strategy_type IN ('momentum','mean_reversion','orb','donchian_trend','bb_squeeze'));
