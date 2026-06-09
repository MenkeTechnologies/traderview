-- 0069 — correlation gate for confluence autotrade.
--
-- Without this, a single news cycle that sends AAPL+MSFT+GOOG+META all
-- to the top of the confluence ranking causes Kelly to size each as
-- independent — net effect 5× exposure to mega-cap tech, beta ~1.1 to
-- QQQ. The gate adds a per-position correlation check: before submitting
-- an order, compute Pearson r of daily % returns between the candidate
-- symbol and each currently open position. If any |r| exceeds
-- `max_pairwise_correlation`, skip the order with action='skipped_correlation'.
--
-- Default 0.85 (returns >85% correlated count as same factor exposure).
-- Default window 60 trading days — long enough to be stable, short
-- enough to reflect current regime.

ALTER TABLE confluence_autotrade_config
    ADD COLUMN IF NOT EXISTS correlation_gate_enabled    BOOLEAN          NOT NULL DEFAULT TRUE,
    ADD COLUMN IF NOT EXISTS max_pairwise_correlation    DOUBLE PRECISION NOT NULL DEFAULT 0.85,
    ADD COLUMN IF NOT EXISTS correlation_window_days     INT              NOT NULL DEFAULT 60;

-- The log already has `reason` (TEXT) which we use to record the offending
-- symbol + r when a correlation skip fires; no extra columns needed.

-- Rollback (manual):
--   ALTER TABLE confluence_autotrade_config DROP COLUMN correlation_window_days;
--   ALTER TABLE confluence_autotrade_config DROP COLUMN max_pairwise_correlation;
--   ALTER TABLE confluence_autotrade_config DROP COLUMN correlation_gate_enabled;
