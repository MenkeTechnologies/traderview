-- 0027 — Forced post-trade reflection for high-|R| trades.
--
-- Five fixed questions per review so the data shape stays comparable
-- over hundreds of trades. UNIQUE (user_id, trade_id) means one review
-- per (user, trade) — re-opening a review overwrites the previous one.

CREATE TABLE trade_reviews (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    trade_id            UUID NOT NULL REFERENCES trades(id) ON DELETE CASCADE,
    entry_per_plan      BOOLEAN,
    exit_per_plan       BOOLEAN,
    would_change        TEXT,
    mood_at_exit        SMALLINT CHECK (mood_at_exit IS NULL OR mood_at_exit BETWEEN -2 AND 2),
    setup_tag           TEXT,                       -- free-form classifier
    completed_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, trade_id)
);
CREATE INDEX trade_reviews_user_idx ON trade_reviews(user_id, completed_at DESC);
