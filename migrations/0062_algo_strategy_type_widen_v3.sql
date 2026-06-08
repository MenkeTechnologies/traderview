-- 0062 — Widen algo_strategies.strategy_type CHECK to include the three
-- newest families: ma_cross_adx, keltner_breakout, ichimoku_cloud.

ALTER TABLE algo_strategies
    DROP CONSTRAINT IF EXISTS algo_strategies_strategy_type_check;

ALTER TABLE algo_strategies
    ADD CONSTRAINT algo_strategies_strategy_type_check
        CHECK (strategy_type IN (
            'momentum', 'mean_reversion', 'orb',
            'donchian_trend', 'bb_squeeze',
            'ttm_squeeze', 'vwap_scalp', 'supertrend', 'heikin_ashi_trend',
            'connors_rsi2', 'order_block_sweep', 'pead', 'pairs',
            'ma_cross_adx', 'keltner_breakout', 'ichimoku_cloud'
        ));

-- Rollback (manual):
--   ALTER TABLE algo_strategies DROP CONSTRAINT algo_strategies_strategy_type_check;
--   ALTER TABLE algo_strategies ADD CONSTRAINT algo_strategies_strategy_type_check
--       CHECK (strategy_type IN (
--           'momentum','mean_reversion','orb','donchian_trend','bb_squeeze',
--           'ttm_squeeze','vwap_scalp','supertrend','heikin_ashi_trend',
--           'connors_rsi2','order_block_sweep','pead','pairs'));
