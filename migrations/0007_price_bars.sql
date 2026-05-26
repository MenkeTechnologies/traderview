-- 0007 — price bars cache (OHLCV)
-- Backs chart rendering, trade replay, MFE/MAE computation.

CREATE TYPE bar_interval_t AS ENUM ('1m', '5m', '15m', '1h', '1d', '1w');

CREATE TABLE price_bars (
    symbol          TEXT NOT NULL,
    interval        bar_interval_t NOT NULL,
    bar_time        TIMESTAMPTZ NOT NULL,
    open            NUMERIC(20, 8) NOT NULL,
    high            NUMERIC(20, 8) NOT NULL,
    low             NUMERIC(20, 8) NOT NULL,
    close           NUMERIC(20, 8) NOT NULL,
    volume          NUMERIC(28, 8) NOT NULL DEFAULT 0,
    source          TEXT NOT NULL DEFAULT 'yfinance',
    fetched_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (symbol, interval, bar_time)
);
CREATE INDEX price_bars_symbol_interval_time_idx ON price_bars(symbol, interval, bar_time DESC);

-- Track which (symbol, interval, range) was last fetched, so the fetcher
-- can avoid redundant network calls.
CREATE TABLE price_fetch_log (
    symbol          TEXT NOT NULL,
    interval        bar_interval_t NOT NULL,
    range_start     TIMESTAMPTZ NOT NULL,
    range_end       TIMESTAMPTZ NOT NULL,
    fetched_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    bar_count       INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (symbol, interval, range_start, range_end)
);

-- Symbol metadata (resolved from yfinance / exchange feed)
CREATE TABLE symbols (
    symbol          TEXT PRIMARY KEY,
    name            TEXT,
    exchange        TEXT,
    asset_class     asset_class_t NOT NULL DEFAULT 'stock',
    currency        TEXT NOT NULL DEFAULT 'USD',
    multiplier      NUMERIC(20, 8) NOT NULL DEFAULT 1,
    tick_size       NUMERIC(20, 8),
    tick_value      NUMERIC(20, 8),
    last_refreshed  TIMESTAMPTZ NOT NULL DEFAULT now()
);
