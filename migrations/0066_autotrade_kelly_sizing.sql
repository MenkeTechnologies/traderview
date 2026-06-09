-- 0066 — Kelly-fractional sizing for confluence autotrade.
--
-- Adds optional sizing strategy to the autotrade config. When
-- sizing_mode = 'fixed_notional' (default), the existing per-symbol
-- notional_usd from 0065 wins. When set to 'half_kelly' or
-- 'quarter_kelly', the per-fire notional is derived from the scanner's
-- backtested mean/stdev (when available) via continuous Kelly applied
-- to current paper account equity, then clamped to kelly_max_fraction.
--
-- When stats aren't yet backtested for the scanner(s) that fired, the
-- pipeline falls back to fixed_notional and logs the reason so the
-- user sees exactly why Kelly couldn't be applied.
--
-- kelly_horizon_days picks which horizon in the scanner backtest table
-- the sizing should reference. 20d is the practical default — long
-- enough to be statistically meaningful, short enough to be tradable.

ALTER TABLE confluence_autotrade_config
    ADD COLUMN IF NOT EXISTS sizing_mode        TEXT             NOT NULL DEFAULT 'fixed_notional',
    ADD COLUMN IF NOT EXISTS kelly_horizon_days INT              NOT NULL DEFAULT 20,
    ADD COLUMN IF NOT EXISTS kelly_max_fraction DOUBLE PRECISION NOT NULL DEFAULT 0.05;

-- Soft constraint — Postgres can't enforce enum semantics on TEXT, but
-- the route handler rejects invalid values before write.

-- Track per-fire how the notional was actually derived so the log shows
-- "kelly_half @ 1.8%" vs "fixed_notional" without polluting `reason`.
ALTER TABLE confluence_autotrade_log
    ADD COLUMN IF NOT EXISTS sizing_used       TEXT,
    ADD COLUMN IF NOT EXISTS kelly_fraction    DOUBLE PRECISION;

-- Rollback (manual):
--   ALTER TABLE confluence_autotrade_log DROP COLUMN kelly_fraction;
--   ALTER TABLE confluence_autotrade_log DROP COLUMN sizing_used;
--   ALTER TABLE confluence_autotrade_config DROP COLUMN kelly_max_fraction;
--   ALTER TABLE confluence_autotrade_config DROP COLUMN kelly_horizon_days;
--   ALTER TABLE confluence_autotrade_config DROP COLUMN sizing_mode;
