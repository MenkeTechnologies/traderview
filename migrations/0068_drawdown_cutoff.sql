-- 0068 — drawdown auto-cutoff.
--
-- Per-user equity-trailing rule: when current equity (cash + MTM
-- positions across configured brokers) falls below
-- (high_water_mark × (1 - max_drawdown_pct/100)), automatically fire
-- the multi-broker kill-switch ONCE and pin auto_killed_at. After a
-- fire, the rule is dormant until the user explicitly clicks "Reset"
-- in the UI (which clears auto_killed_at and re-seeds high_water_mark
-- from current equity). That prevents a re-fire loop if the kill
-- itself doesn't fully flatten the book.
--
-- enabled = FALSE by default — this is destructive automation; opt-in
-- only.

CREATE TABLE IF NOT EXISTS drawdown_cutoff_config (
    user_id              UUID         PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    enabled              BOOLEAN      NOT NULL DEFAULT FALSE,
    max_drawdown_pct     DOUBLE PRECISION NOT NULL DEFAULT 5.0,
    high_water_mark      DOUBLE PRECISION,
    last_equity          DOUBLE PRECISION,
    last_evaluated_at    TIMESTAMPTZ,
    auto_killed_at       TIMESTAMPTZ,
    updated_at           TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- Per-evaluation audit so the user can see every check that ran +
-- every fire that happened. Even a no-op evaluation gets a row so the
-- UI can render "evaluated, no breach" history.
CREATE TABLE IF NOT EXISTS drawdown_cutoff_log (
    id                   BIGSERIAL    PRIMARY KEY,
    user_id              UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    evaluated_at         TIMESTAMPTZ  NOT NULL DEFAULT now(),
    current_equity       DOUBLE PRECISION NOT NULL,
    high_water_mark      DOUBLE PRECISION NOT NULL,
    drawdown_pct         DOUBLE PRECISION NOT NULL,
    threshold_pct        DOUBLE PRECISION NOT NULL,
    action               TEXT         NOT NULL  -- 'evaluated' | 'fired' | 'skipped_disabled' | 'skipped_already_fired'
);

CREATE INDEX IF NOT EXISTS drawdown_cutoff_log_user_time_idx
    ON drawdown_cutoff_log (user_id, evaluated_at DESC);

-- Rollback (manual):
--   DROP TABLE drawdown_cutoff_log;
--   DROP TABLE drawdown_cutoff_config;
