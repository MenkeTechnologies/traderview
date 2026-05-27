-- 0023 — Compound strategy alerts.
--
-- `ast` is a JSONB tree of And/Or/Not over Leaf conditions; the evaluator
-- in traderview-db reads metrics from cached price_bars + live quotes +
-- breadth snapshot. `last_truth` lets us fire only on the false→true edge
-- so a persistent condition doesn't spam every 60s.

CREATE TABLE strategy_alerts (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name                TEXT NOT NULL,
    enabled             BOOLEAN NOT NULL DEFAULT TRUE,
    ast                 JSONB NOT NULL,
    webhook_ids         UUID[] NOT NULL DEFAULT '{}'::uuid[],
    last_truth          BOOLEAN,
    last_evaluated_at   TIMESTAMPTZ,
    last_fired_at       TIMESTAMPTZ,
    fire_count          INTEGER NOT NULL DEFAULT 0,
    last_eval_error     TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX strategy_alerts_user_idx ON strategy_alerts(user_id, enabled);

CREATE TABLE strategy_alert_fires (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    alert_id        UUID NOT NULL REFERENCES strategy_alerts(id) ON DELETE CASCADE,
    fired_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    snapshot        JSONB                              -- metric values at fire time
);
CREATE INDEX strategy_alert_fires_alert_idx ON strategy_alert_fires(alert_id, fired_at DESC);
