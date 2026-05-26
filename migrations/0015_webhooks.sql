-- 0015 — Outbound webhooks (Discord / Slack / generic) for alert fan-out.

CREATE TYPE webhook_kind_t AS ENUM ('discord', 'slack', 'generic');

CREATE TABLE webhooks (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    kind            webhook_kind_t NOT NULL,
    url             TEXT NOT NULL,
    secret          TEXT,                                  -- optional HMAC secret for generic
    enabled         BOOLEAN NOT NULL DEFAULT TRUE,
    last_fired_at   TIMESTAMPTZ,
    fire_count      INTEGER NOT NULL DEFAULT 0,
    last_status     TEXT,                                  -- "200 OK" / "404 / connection refused"
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, name)
);
CREATE INDEX webhooks_user_idx ON webhooks(user_id, enabled);

-- Fan-out: which webhooks an alert rule should hit when it fires.
ALTER TABLE alert_rules
    ADD COLUMN webhook_ids UUID[] NOT NULL DEFAULT '{}'::uuid[];
