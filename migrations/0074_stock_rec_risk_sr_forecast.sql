-- Risk badge + support/resistance + 3-month forecast band for stored
-- stock recommendations. Backfills default to NULL / 0 — the next
-- nightly compute repopulates every row it touches, and the
-- leaderboard tolerates NULLs on the new columns for old rows.

ALTER TABLE stock_recommendations
    ADD COLUMN risk_level          TEXT,
    ADD COLUMN annualized_vol_pct  NUMERIC(8, 4) NOT NULL DEFAULT 0,
    ADD COLUMN support             NUMERIC(20, 8),
    ADD COLUMN resistance          NUMERIC(20, 8),
    ADD COLUMN forecast_3m_low     NUMERIC(20, 8),
    ADD COLUMN forecast_3m_high    NUMERIC(20, 8);
