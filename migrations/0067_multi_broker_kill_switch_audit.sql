-- 0067 — kill-switch audit log.
--
-- Every fire of POST /multi-broker/kill-switch writes one row so the
-- user can later answer "who flattened my book at 9:47 AM and why."
-- Today the actor is always the authenticated user (no service-token
-- impersonation), but the column shape leaves room for a future
-- caller_id when scheduled drawdown auto-cutoff lands.

CREATE TABLE IF NOT EXISTS multi_broker_kill_switch_audit (
    id                 BIGSERIAL    PRIMARY KEY,
    user_id            UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    fired_at           TIMESTAMPTZ  NOT NULL DEFAULT now(),
    brokers_attempted  TEXT         NOT NULL,
    cancelled_orders   INT          NOT NULL DEFAULT 0,
    closed_positions   INT          NOT NULL DEFAULT 0,
    reason             TEXT,
    error_count        INT          NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS multi_broker_kill_switch_audit_user_time_idx
    ON multi_broker_kill_switch_audit (user_id, fired_at DESC);

-- Rollback (manual):
--   DROP TABLE multi_broker_kill_switch_audit;
