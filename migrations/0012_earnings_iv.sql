-- 0012 — earnings-week IV scanner + straddle/strangle backtest cache.
--
-- The "implied move" for an earnings event is the cost of an at-the-money
-- straddle on the nearest post-earnings expiration, expressed as % of spot.
-- We compare it to the 8-quarter realized-move history to flag overpriced /
-- underpriced premium.

CREATE TABLE earnings_calendar (
    symbol          TEXT NOT NULL,
    earnings_date   DATE NOT NULL,
    fiscal_period   TEXT,                                  -- 'Q1 2026', etc.
    when_announced  TEXT,                                  -- 'bmo' | 'amc' | 'unknown'
    eps_estimate    NUMERIC(20, 8),
    eps_actual      NUMERIC(20, 8),
    revenue_estimate NUMERIC(28, 8),
    revenue_actual  NUMERIC(28, 8),
    fetched_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (symbol, earnings_date)
);
CREATE INDEX earnings_calendar_date_idx ON earnings_calendar(earnings_date);

-- ---------------------------------------------------------------------------
-- Realized historical earnings moves (8-quarter rolling window).
-- ---------------------------------------------------------------------------
CREATE TABLE realized_earnings_moves (
    symbol          TEXT NOT NULL,
    earnings_date   DATE NOT NULL,
    close_before    NUMERIC(20, 8) NOT NULL,    -- last close before earnings
    close_after     NUMERIC(20, 8) NOT NULL,    -- first close after earnings
    abs_move_pct    NUMERIC(10, 4) NOT NULL,    -- |close_after - close_before| / close_before * 100
    direction       TEXT NOT NULL,              -- 'up' | 'down'
    PRIMARY KEY (symbol, earnings_date)
);
CREATE INDEX realized_earnings_moves_symbol_idx ON realized_earnings_moves(symbol);

-- ---------------------------------------------------------------------------
-- Options chain snapshot cache. TTL ~ 15 min during market hours, longer overnight.
-- ---------------------------------------------------------------------------
CREATE TABLE options_chain_snapshots (
    symbol          TEXT NOT NULL,
    expiration      DATE NOT NULL,
    spot_price      NUMERIC(20, 8) NOT NULL,
    fetched_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (symbol, expiration)
);

CREATE TABLE option_quotes (
    symbol          TEXT NOT NULL,
    expiration      DATE NOT NULL,
    strike          NUMERIC(20, 8) NOT NULL,
    option_type     option_type_t NOT NULL,
    bid             NUMERIC(20, 8),
    ask             NUMERIC(20, 8),
    last_price      NUMERIC(20, 8),
    implied_vol     NUMERIC(10, 6),
    volume          BIGINT,
    open_interest   BIGINT,
    in_the_money    BOOLEAN,
    fetched_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (symbol, expiration, strike, option_type),
    FOREIGN KEY (symbol, expiration) REFERENCES options_chain_snapshots(symbol, expiration)
        ON DELETE CASCADE
);

-- ---------------------------------------------------------------------------
-- Computed implied move for each symbol×next-earnings (re-computed on demand).
-- ---------------------------------------------------------------------------
CREATE TABLE implied_moves (
    symbol          TEXT NOT NULL,
    earnings_date   DATE NOT NULL,
    expiration      DATE NOT NULL,             -- straddle expiration used
    spot_price      NUMERIC(20, 8) NOT NULL,
    atm_strike      NUMERIC(20, 8) NOT NULL,
    call_mid        NUMERIC(20, 8) NOT NULL,
    put_mid         NUMERIC(20, 8) NOT NULL,
    implied_move_pct  NUMERIC(10, 4) NOT NULL,   -- (call_mid + put_mid) / spot * 100
    avg_realized_pct  NUMERIC(10, 4),            -- 8q mean
    median_realized_pct NUMERIC(10, 4),          -- 8q median
    realized_sample_size INTEGER NOT NULL DEFAULT 0,
    edge_pct          NUMERIC(10, 4),            -- implied - median realized
    -- Backtest tally over the sampled quarters.
    long_straddle_pnl   NUMERIC(20, 8),          -- avg per-quarter $ on $1 of premium
    short_straddle_pnl  NUMERIC(20, 8),
    long_straddle_winrate  NUMERIC(10, 4),
    short_straddle_winrate NUMERIC(10, 4),
    computed_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (symbol, earnings_date)
);
CREATE INDEX implied_moves_edge_idx ON implied_moves(edge_pct DESC, computed_at DESC);
CREATE INDEX implied_moves_earnings_date_idx ON implied_moves(earnings_date);
