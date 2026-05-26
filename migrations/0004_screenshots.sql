-- 0004 — screenshots attached to trades or journal entries
-- TraderVue parity: chart/setup screenshots per trade.

CREATE TABLE screenshots (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    trade_id        UUID REFERENCES trades(id) ON DELETE CASCADE,
    journal_id      UUID REFERENCES journal_entries(id) ON DELETE CASCADE,
    filename        TEXT NOT NULL,                            -- original upload name
    mime_type       TEXT NOT NULL,
    size_bytes      BIGINT NOT NULL,
    bytes           BYTEA NOT NULL,                           -- file content (web mode)
    caption         TEXT NOT NULL DEFAULT '',
    position        INTEGER NOT NULL DEFAULT 0,               -- display order within owner
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (trade_id IS NOT NULL OR journal_id IS NOT NULL)
);
CREATE INDEX screenshots_trade_idx ON screenshots(trade_id) WHERE trade_id IS NOT NULL;
CREATE INDEX screenshots_journal_idx ON screenshots(journal_id) WHERE journal_id IS NOT NULL;
CREATE INDEX screenshots_user_idx ON screenshots(user_id);
