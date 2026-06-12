-- Per-user digest delivery hour (UTC) + idempotent delivery marker.
-- Separate table rather than user_settings: the digest owns its state,
-- and last_sent_on makes delivery exactly-once-per-day across process
-- restarts (the in-memory dedup the drift watches use is fine at 12h
-- cadence; a daily digest re-sent on every restart would not be).
CREATE TABLE IF NOT EXISTS digest_prefs (
    user_id      UUID PRIMARY KEY,
    hour_utc     INT NOT NULL DEFAULT 12 CHECK (hour_utc BETWEEN 0 AND 23),
    last_sent_on DATE
);
