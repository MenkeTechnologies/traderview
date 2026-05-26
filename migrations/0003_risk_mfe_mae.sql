-- 0003 — risk fields + MFE/MAE on trades
-- TraderVue parity: R-multiple reports, exit-efficiency, max favorable/adverse excursion.

ALTER TABLE trades
    -- risk inputs (user-supplied or derived from journal)
    ADD COLUMN stop_loss        NUMERIC(20, 8),                    -- price the user planned to stop at
    ADD COLUMN risk_amount      NUMERIC(20, 8),                    -- $ at risk = (entry - stop) * qty * mult
    ADD COLUMN initial_target   NUMERIC(20, 8),                    -- planned exit
    -- MFE / MAE (computed from price_bars during a trade's open window)
    ADD COLUMN mfe              NUMERIC(20, 8),                    -- max favorable excursion ($)
    ADD COLUMN mae              NUMERIC(20, 8),                    -- max adverse excursion ($)
    ADD COLUMN best_exit_pnl    NUMERIC(20, 8),                    -- theoretical max P&L
    ADD COLUMN exit_efficiency  NUMERIC(10, 6);                    -- net_pnl / best_exit_pnl, 0..1

-- R-multiple is computed on-the-fly (net_pnl / risk_amount) — no column needed,
-- but an index on risk_amount speeds up "all trades where risk > X" filters.
CREATE INDEX trades_risk_amount_idx ON trades(account_id, risk_amount) WHERE risk_amount IS NOT NULL;
