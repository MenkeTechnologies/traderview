-- Persisted snapshots of per-symbol Buy/Sell/Hold recommendations + a
-- per-user watcher table that fires when a watched symbol's verdict
-- flips (e.g. strong_buy → hold).
--
-- Why persisted: the panel computes on-demand, but the Golden Stars
-- leaderboard, the verdict-change alerter, and the backtest tooling all
-- need a time-series. One row per (symbol, computed_at) keeps history
-- so we can answer "show me the day this name flipped to strong_buy"
-- and "what's the hit rate of buys generated last quarter."
--
-- All scoring inputs are reproducible from `bars` + the algorithm in
-- `crates/traderview-db/src/stock_recommendation.rs`; we still cache
-- the raw components JSON so the UI can render the panel without
-- re-running the indicator stack on every page load.

CREATE TABLE stock_recommendations (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol          TEXT NOT NULL,
    computed_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- Verdict bucket. Free-form text rather than an ENUM so we can
    -- adjust thresholds (or add new verdicts) without a migration. The
    -- Rust enum is the source of truth for valid values.
    verdict         TEXT NOT NULL,
    score           NUMERIC(6, 2) NOT NULL,           -- 0.00 - 100.00
    stars           SMALLINT NOT NULL CHECK (stars BETWEEN 1 AND 5),
    current_price   NUMERIC(20, 8) NOT NULL,
    target_price    NUMERIC(20, 8) NOT NULL,
    upside_pct      NUMERIC(8, 4) NOT NULL,
    horizon_days    INTEGER NOT NULL DEFAULT 30,
    bars_analyzed   INTEGER NOT NULL,
    -- Full Component[] JSON from the engine. Lets the panel render the
    -- breakdown bars from a single SELECT instead of re-running the
    -- indicator stack on every load.
    components      JSONB NOT NULL DEFAULT '[]'::jsonb
);

-- "Latest by symbol" — the leaderboard hits this hard.
CREATE INDEX stock_recommendations_symbol_computed_idx
    ON stock_recommendations(symbol, computed_at DESC);
-- "Top of the board for a given run" — the cron stamps one row per
-- symbol at the same moment, leaderboard ranks by score within that
-- batch.
CREATE INDEX stock_recommendations_computed_score_idx
    ON stock_recommendations(computed_at DESC, score DESC);

-- Per-user watcher: "alert me when AAPL flips out of strong_buy."
-- `last_verdict` is the verdict we last saw on this symbol; when the
-- nightly compute writes a new row whose verdict differs, the alerter
-- fans out via the user's webhook ids and bumps last_verdict.
CREATE TABLE stock_recommendation_watchers (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    symbol          TEXT NOT NULL,
    -- If non-empty, only fire when the new verdict is in this set
    -- (e.g. `{"strong_buy","buy"}` for "tell me when this becomes a
    -- buy candidate"). Empty/NULL = fire on ANY verdict change.
    fire_on         TEXT[],
    -- Webhook IDs that receive the notification. Same shape as
    -- strategy_alerts.webhook_ids — references existing webhooks table.
    webhook_ids     UUID[] NOT NULL DEFAULT '{}',
    last_verdict    TEXT,
    last_fired_at   TIMESTAMPTZ,
    fire_count      INTEGER NOT NULL DEFAULT 0,
    enabled         BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- One watcher per (user, symbol). A user toggling a watcher off
    -- twice rows = bug magnet; CRUD updates the single row instead.
    UNIQUE (user_id, symbol)
);

CREATE INDEX stock_recommendation_watchers_user_idx
    ON stock_recommendation_watchers(user_id);
CREATE INDEX stock_recommendation_watchers_symbol_enabled_idx
    ON stock_recommendation_watchers(symbol)
    WHERE enabled = TRUE;
