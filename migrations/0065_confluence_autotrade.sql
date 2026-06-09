-- 0065 — confluence autotrade pipeline.
--
-- Wires the confluence dashboard's ranked output directly into the paper
-- account. When `enabled = true`, the autotrade runner picks symbols
-- whose confluence score crosses `min_score` AND have at least
-- `min_distinct_sources` independent scanners hitting, then submits a
-- paper-market buy for `notional_usd / quote` shares against the user's
-- default paper account.
--
-- Cooldown prevents the same symbol firing twice while it's still hot in
-- the ranking — without it, every poll would re-buy the top symbol.
--
-- `max_open_positions` caps simultaneous autotrade exposure so a single
-- confluence flood (e.g. an earnings day) can't 10× the account.

CREATE TABLE IF NOT EXISTS confluence_autotrade_config (
    user_id              UUID         PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    enabled              BOOLEAN      NOT NULL DEFAULT FALSE,
    min_score            DOUBLE PRECISION NOT NULL DEFAULT 8.0,
    min_distinct_sources INT          NOT NULL DEFAULT 3,
    notional_usd         DOUBLE PRECISION NOT NULL DEFAULT 1000.0,
    cooldown_minutes     INT          NOT NULL DEFAULT 240,
    max_open_positions   INT          NOT NULL DEFAULT 10,
    updated_at           TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- Per-fire audit log so we can show the user every autotrade decision +
-- the score that drove it. The paper_order_id is nullable because we log
-- rejected attempts too (cooldown, max-positions cap, quote unavailable).
CREATE TABLE IF NOT EXISTS confluence_autotrade_log (
    id                   BIGSERIAL    PRIMARY KEY,
    user_id              UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    symbol               TEXT         NOT NULL,
    score                DOUBLE PRECISION NOT NULL,
    distinct_sources     INT          NOT NULL,
    notional_usd         DOUBLE PRECISION NOT NULL,
    action               TEXT         NOT NULL,  -- 'submitted' | 'skipped_cooldown' | 'skipped_cap' | 'skipped_quote' | 'skipped_open'
    paper_order_id       UUID         REFERENCES paper_orders(id) ON DELETE SET NULL,
    reason               TEXT,
    fired_at             TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS confluence_autotrade_log_user_time_idx
    ON confluence_autotrade_log (user_id, fired_at DESC);

CREATE INDEX IF NOT EXISTS confluence_autotrade_log_user_symbol_time_idx
    ON confluence_autotrade_log (user_id, symbol, fired_at DESC);

-- Rollback (manual):
--   DROP TABLE confluence_autotrade_log;
--   DROP TABLE confluence_autotrade_config;
