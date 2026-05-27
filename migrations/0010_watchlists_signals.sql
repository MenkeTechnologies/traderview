-- 0010 — watchlists, quote snapshots, computed signals cache.
-- Yahoo Finance + StockInvest.us-style market research layer.

CREATE TABLE watchlists (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    position        INTEGER NOT NULL DEFAULT 0,
    is_default      BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, name)
);
CREATE INDEX watchlists_user_idx ON watchlists(user_id, position);

CREATE TABLE watchlist_symbols (
    watchlist_id    UUID NOT NULL REFERENCES watchlists(id) ON DELETE CASCADE,
    symbol          TEXT NOT NULL,
    position        INTEGER NOT NULL DEFAULT 0,
    added_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (watchlist_id, symbol)
);
CREATE INDEX watchlist_symbols_symbol_idx ON watchlist_symbols(symbol);

-- ---------------------------------------------------------------------------
-- Quote snapshot cache (live price / day-change). TTL enforced in app code.
-- ---------------------------------------------------------------------------
CREATE TABLE quote_snapshots (
    symbol          TEXT PRIMARY KEY,
    price           NUMERIC(20, 8) NOT NULL,
    prev_close      NUMERIC(20, 8),
    change_pct      NUMERIC(10, 4),
    day_high        NUMERIC(20, 8),
    day_low         NUMERIC(20, 8),
    volume          BIGINT,
    market_state    TEXT,
    fetched_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ---------------------------------------------------------------------------
-- Computed signal results — recomputed on demand from price_bars.
-- A row per (symbol, computed_at) lets us track signal duration over time.
-- ---------------------------------------------------------------------------
CREATE TABLE signal_results (
    symbol          TEXT NOT NULL,
    computed_at     TIMESTAMPTZ NOT NULL,
    -- StockInvest-style composite (-10 .. +10).
    score           INTEGER NOT NULL,
    summary         TEXT NOT NULL,            -- 'buy' | 'sell' | 'hold'
    -- Detail blob: { sma20, sma50, sma200, ema12, ema26, macd, signal, hist,
    --                rsi14, adx14, stoch_k, stoch_d, bb_upper, bb_mid, bb_lower,
    --                pivot, r1, r2, r3, s1, s2, s3,
    --                signals: [{ name, side, weight, ... }] }
    detail          JSONB NOT NULL DEFAULT '{}'::jsonb,
    PRIMARY KEY (symbol, computed_at)
);
CREATE INDEX signal_results_symbol_idx ON signal_results(symbol, computed_at DESC);
CREATE INDEX signal_results_score_idx ON signal_results(score, computed_at DESC);
