-- 0022 — Earnings calendar with EPS surprise tracking.
--
-- One row per (symbol, earnings_date). For upcoming events we have estimate
-- but no actual; for past events both, plus 1d and 5d price reactions
-- computed against the existing price_bars cache.

CREATE TABLE earnings_events (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol          TEXT NOT NULL,
    earnings_date   DATE NOT NULL,
    timing          TEXT,                              -- 'amc' | 'bmo' | 'unknown'
    eps_estimate    NUMERIC(20, 6),
    eps_actual      NUMERIC(20, 6),
    revenue_estimate NUMERIC(20, 2),
    revenue_actual   NUMERIC(20, 2),
    surprise_pct    REAL,                              -- (actual - est) / |est| * 100
    price_close_pre NUMERIC(20, 6),                    -- close on earnings_date (if BMO use prior session)
    price_close_1d  NUMERIC(20, 6),
    price_close_5d  NUMERIC(20, 6),
    reaction_1d_pct REAL,
    reaction_5d_pct REAL,
    fetched_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (symbol, earnings_date)
);
CREATE INDEX earnings_events_date_idx     ON earnings_events(earnings_date);
CREATE INDEX earnings_events_symbol_idx   ON earnings_events(symbol, earnings_date DESC);
CREATE INDEX earnings_events_surprise_idx ON earnings_events(surprise_pct DESC NULLS LAST);
