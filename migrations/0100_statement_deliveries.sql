-- Month-end statement push: exactly-once per (user, month) across
-- restarts, same idea as digest_prefs.last_sent_on but keyed by the
-- statement month it covers.
CREATE TABLE statement_deliveries (
    user_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    month        TEXT NOT NULL,
    delivered_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, month)
);
