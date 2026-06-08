-- 0052 — Algorithmic momentum trading.
--
-- Persists per-user algo strategies + every run, order, and fill they
-- produce. Order / fill rows mirror what the broker (internal sim, or
-- Alpaca paper / live) returned so the UI can render a strategy P&L
-- without round-tripping the broker on every page load.
--
-- Defaults are tuned for safety: every new strategy starts paper-locked
-- for 30 days (paper_locked_until), risks 1% of equity per trade, and
-- has its kill switch disengaged. Promoting to Alpaca live requires the
-- paper-lock to have expired AND the user to explicitly flip broker_mode.

CREATE TABLE algo_strategies (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name                        TEXT NOT NULL,
    enabled                     BOOLEAN NOT NULL DEFAULT FALSE,
    -- 'sec10' (ultra-scalp, 10-second bars) | 'min1' (1-minute bars, default)
    timeframe                   TEXT NOT NULL DEFAULT 'min1'
        CHECK (timeframe IN ('sec10', 'min1')),
    -- 'watchlist' (user-supplied watchlist_id) | 'autoscan' (top-N by RVOL)
    universe_mode               TEXT NOT NULL DEFAULT 'watchlist'
        CHECK (universe_mode IN ('watchlist', 'autoscan')),
    watchlist_id                UUID REFERENCES watchlists(id) ON DELETE SET NULL,
    autoscan_top_n              INTEGER NOT NULL DEFAULT 25
        CHECK (autoscan_top_n BETWEEN 1 AND 500),
    -- 'long' | 'short' | 'both' — restricts which sides the strategy can take
    side_mode                   TEXT NOT NULL DEFAULT 'long'
        CHECK (side_mode IN ('long', 'short', 'both')),
    -- Per-strategy entry / exit rules. JSONB so the schema can evolve
    -- without a migration; see traderview-core::momentum_strategy::Rules.
    entry_rules                 JSONB NOT NULL DEFAULT '{}'::jsonb,
    exit_rules                  JSONB NOT NULL DEFAULT '{}'::jsonb,
    -- Sizing: risk_pct_per_trade (default 0.01 = 1% of equity), max_pos_pct,
    -- vol_target_sigma, kelly_fraction. See momentum_strategy::Sizing.
    sizing                      JSONB NOT NULL DEFAULT '{"risk_pct_per_trade": 0.01}'::jsonb,
    -- Risk gates: daily_loss_limit_pct, max_drawdown_pct,
    -- max_concurrent_positions, max_symbol_concentration_pct.
    risk_gates                  JSONB NOT NULL DEFAULT
        '{"daily_loss_limit_pct": 0.03, "max_drawdown_pct": 0.10,
          "max_concurrent_positions": 5, "max_symbol_concentration_pct": 0.20}'::jsonb,
    -- Execution venue. internal_sim = traderview-db::paper module.
    broker_mode                 TEXT NOT NULL DEFAULT 'internal_sim'
        CHECK (broker_mode IN ('internal_sim', 'alpaca_paper', 'alpaca_live')),
    -- New strategies are paper-locked for 30 days regardless of broker_mode;
    -- the engine refuses to send to alpaca_live until now() > paper_locked_until.
    paper_locked_until          TIMESTAMPTZ NOT NULL DEFAULT (now() + INTERVAL '30 days'),
    -- Manual halt. Engine polls this every tick; flipping it true cancels
    -- all working orders and refuses to submit new ones until released.
    kill_switch                 BOOLEAN NOT NULL DEFAULT FALSE,
    kill_reason                 TEXT,
    last_kill_at                TIMESTAMPTZ,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX algo_strategies_user_idx ON algo_strategies(user_id, enabled);

CREATE TABLE algo_runs (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    strategy_id                 UUID NOT NULL REFERENCES algo_strategies(id) ON DELETE CASCADE,
    started_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    stopped_at                  TIMESTAMPTZ,
    -- Reason for the run terminating. NULL while running.
    stopped_reason              TEXT
        CHECK (stopped_reason IN ('user', 'kill_switch', 'risk_breach',
                                  'broker_error', 'engine_error', 'shutdown')),
    bars_processed              BIGINT NOT NULL DEFAULT 0,
    signals_emitted             BIGINT NOT NULL DEFAULT 0,
    orders_submitted            BIGINT NOT NULL DEFAULT 0,
    fills_received              BIGINT NOT NULL DEFAULT 0,
    pnl_realized                NUMERIC(20, 8) NOT NULL DEFAULT 0,
    pnl_unrealized_at_stop      NUMERIC(20, 8),
    last_error                  TEXT
);
CREATE INDEX algo_runs_strategy_idx ON algo_runs(strategy_id, started_at DESC);
-- Per-strategy partial unique: only one open run at a time per strategy.
CREATE UNIQUE INDEX algo_runs_one_open_per_strategy
    ON algo_runs(strategy_id) WHERE stopped_at IS NULL;

CREATE TABLE algo_orders (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id                      UUID NOT NULL REFERENCES algo_runs(id) ON DELETE CASCADE,
    strategy_id                 UUID NOT NULL REFERENCES algo_strategies(id) ON DELETE CASCADE,
    -- We generate this and pass it as Alpaca's client_order_id so we can
    -- reconcile a fill back to a row even if the broker WS drops.
    client_order_id             UUID NOT NULL UNIQUE,
    -- Broker-assigned id; NULL until the POST /v2/orders response lands.
    broker_order_id             TEXT UNIQUE,
    symbol                      TEXT NOT NULL,
    side                        TEXT NOT NULL CHECK (side IN ('buy', 'sell')),
    -- 'market' | 'limit' | 'stop' | 'stop_limit' | 'trailing_stop'
    order_type                  TEXT NOT NULL,
    -- 'simple' | 'bracket' | 'oco' | 'oto'
    order_class                 TEXT NOT NULL DEFAULT 'simple',
    qty                         NUMERIC(20, 8) NOT NULL,
    limit_price                 NUMERIC(20, 8),
    stop_price                  NUMERIC(20, 8),
    -- Alpaca order lifecycle: new, partially_filled, filled, canceled,
    -- expired, replaced, rejected, accepted, pending_new, stopped,
    -- suspended, calculated, done_for_day.
    status                      TEXT NOT NULL DEFAULT 'pending_submit',
    submitted_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- Verbatim request / response bodies for debugging broker quirks.
    raw_request                 JSONB,
    raw_response                JSONB,
    error                       TEXT
);
CREATE INDEX algo_orders_run_idx ON algo_orders(run_id, submitted_at DESC);
CREATE INDEX algo_orders_status_idx ON algo_orders(status)
    WHERE status NOT IN ('filled', 'canceled', 'expired', 'rejected');

CREATE TABLE algo_fills (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id                    UUID NOT NULL REFERENCES algo_orders(id) ON DELETE CASCADE,
    -- Broker fill id (Alpaca exposes one per trade_updates fill event).
    broker_fill_id              TEXT UNIQUE,
    fill_qty                    NUMERIC(20, 8) NOT NULL,
    fill_price                  NUMERIC(20, 8) NOT NULL,
    fill_value                  NUMERIC(20, 8) NOT NULL,
    commission                  NUMERIC(20, 8) NOT NULL DEFAULT 0,
    filled_at                   TIMESTAMPTZ NOT NULL DEFAULT now(),
    raw                         JSONB
);
CREATE INDEX algo_fills_order_idx ON algo_fills(order_id, filled_at DESC);

CREATE TABLE algo_kill_switch_audit (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    strategy_id                 UUID NOT NULL REFERENCES algo_strategies(id) ON DELETE CASCADE,
    actor_user_id               UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- 'engaged' (kill_switch -> TRUE) | 'released' (-> FALSE)
    action                      TEXT NOT NULL CHECK (action IN ('engaged', 'released')),
    reason                      TEXT,
    at                          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX algo_kill_switch_audit_strategy_idx
    ON algo_kill_switch_audit(strategy_id, at DESC);

-- Rollback (manual):
--   DROP TABLE algo_kill_switch_audit, algo_fills, algo_orders, algo_runs, algo_strategies;
