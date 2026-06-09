-- 0071 — stop-loss + take-profit on autotrade positions.
--
-- Time-stop and signal-degradation are slow. A position can lose 30%
-- before the time-stop fires at day 20; the confluence score can stay
-- high while MTM craters. SL/TP fire on the price action itself.
--
-- Defaults: SL 5%, TP 15% (3:1 reward-to-risk). Trailing stop is OFF
-- by default — when enabled, replaces SL with a high-water-mark trail
-- that tracks the position's peak and fires when MTM falls
-- trailing_stop_pct below the HWM.
--
-- sweep_exits checks SL/TP/trailing FIRST (price-driven, hard rules),
-- then falls through to time-stop, then signal-degradation.

ALTER TABLE confluence_autotrade_config
    ADD COLUMN IF NOT EXISTS stop_loss_pct           DOUBLE PRECISION NOT NULL DEFAULT 5.0,
    ADD COLUMN IF NOT EXISTS take_profit_pct         DOUBLE PRECISION NOT NULL DEFAULT 15.0,
    ADD COLUMN IF NOT EXISTS trailing_stop_enabled   BOOLEAN          NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS trailing_stop_pct       DOUBLE PRECISION NOT NULL DEFAULT 8.0;

-- Per-position entry price + running peak for trailing stop. Without
-- entry_price stored at open time, we can't compare current MTM against
-- it later — the paper_order's filled_price drifts as the avg-price
-- recomputes on subsequent adds.
ALTER TABLE autotrade_position_tags
    ADD COLUMN IF NOT EXISTS entry_price            DOUBLE PRECISION,
    ADD COLUMN IF NOT EXISTS high_water_mark_price  DOUBLE PRECISION;

-- Log gains two new action types (no schema change — `action` is TEXT):
--   * 'exit_stop_loss'      — MTM ≤ entry × (1 - stop_loss_pct/100)
--   * 'exit_take_profit'    — MTM ≥ entry × (1 + take_profit_pct/100)
--   * 'exit_trailing_stop'  — MTM ≤ hwm × (1 - trailing_stop_pct/100)

-- Rollback (manual):
--   ALTER TABLE autotrade_position_tags DROP COLUMN high_water_mark_price;
--   ALTER TABLE autotrade_position_tags DROP COLUMN entry_price;
--   ALTER TABLE confluence_autotrade_config DROP COLUMN trailing_stop_pct;
--   ALTER TABLE confluence_autotrade_config DROP COLUMN trailing_stop_enabled;
--   ALTER TABLE confluence_autotrade_config DROP COLUMN take_profit_pct;
--   ALTER TABLE confluence_autotrade_config DROP COLUMN stop_loss_pct;
