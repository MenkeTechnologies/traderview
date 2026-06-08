-- 0063 — algo_backtests: persistent history of every backtest a user
-- runs against a strategy. Reads off the same algo_strategies.id so
-- the dashboard can list "all backtests for this strategy" without a
-- second join.
--
-- We persist:
--   * strategy_id (FK), user_id (audit trail)
--   * input config (symbol, interval, range, equity, fees, slippage)
--   * snapshot of the entry_rules JSON used at run time so a result
--     stays meaningful even when the user later edits the strategy
--   * full summary (trades, win_rate, pf, max_dd, sharpe, etc.) — the
--     equity_curve + trade list stay client-side and would balloon
--     the row; if the user wants them again they re-run.

CREATE TABLE IF NOT EXISTS algo_backtests (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    strategy_id      UUID NOT NULL REFERENCES algo_strategies(id) ON DELETE CASCADE,
    user_id          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- Input config
    symbol           TEXT NOT NULL,
    interval         TEXT NOT NULL,
    range_from       TIMESTAMPTZ NOT NULL,
    range_to         TIMESTAMPTZ NOT NULL,
    initial_equity   NUMERIC(20, 8) NOT NULL,
    fee_per_trade    NUMERIC(20, 8) NOT NULL,
    slippage_bps     NUMERIC(20, 8) NOT NULL,
    -- entry_rules snapshot at run time
    entry_rules      JSONB NOT NULL DEFAULT '{}'::jsonb,
    -- Summary stats (denormalized for fast list rendering)
    trades           BIGINT NOT NULL DEFAULT 0,
    wins             BIGINT NOT NULL DEFAULT 0,
    losses           BIGINT NOT NULL DEFAULT 0,
    win_rate         NUMERIC(10, 6) NOT NULL DEFAULT 0,
    avg_r            NUMERIC(20, 8) NOT NULL DEFAULT 0,
    profit_factor    NUMERIC(20, 8) NOT NULL DEFAULT 0,
    total_return_pct NUMERIC(20, 8) NOT NULL DEFAULT 0,
    max_drawdown_pct NUMERIC(20, 8) NOT NULL DEFAULT 0,
    final_equity     NUMERIC(20, 8) NOT NULL DEFAULT 0,
    sharpe           NUMERIC(20, 8) NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS algo_backtests_strategy_idx
    ON algo_backtests (strategy_id, created_at DESC);
CREATE INDEX IF NOT EXISTS algo_backtests_user_idx
    ON algo_backtests (user_id, created_at DESC);

-- Rollback (manual):
--   DROP TABLE algo_backtests;
