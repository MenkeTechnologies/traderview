-- 0009 — TraderVue-help parity gaps:
--   * Notes templates (auto-applied to new trades / journal entries)
--   * Commission rates per share / per contract on user_settings
--   * Auto-flatten setting (split trade whenever position goes flat)
--   * General notes (no trade, no day)
--   * Postgres full-text search on trades + journal + forum posts

-- ---------------------------------------------------------------------------
-- Notes templates
-- ---------------------------------------------------------------------------
CREATE TABLE note_templates (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    scope           TEXT NOT NULL DEFAULT 'trade',   -- trade | journal
    body_md         TEXT NOT NULL DEFAULT '',
    is_default      BOOLEAN NOT NULL DEFAULT FALSE,  -- auto-apply on new entries
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, name)
);
CREATE INDEX note_templates_user_idx ON note_templates(user_id, scope);

-- ---------------------------------------------------------------------------
-- User settings — add commission rates + auto-flatten
-- ---------------------------------------------------------------------------
ALTER TABLE user_settings
    ADD COLUMN commission_per_share    NUMERIC(20, 8) NOT NULL DEFAULT 0,
    ADD COLUMN commission_per_contract NUMERIC(20, 8) NOT NULL DEFAULT 0,
    ADD COLUMN auto_flatten            BOOLEAN NOT NULL DEFAULT TRUE,
    ADD COLUMN require_account_tag     BOOLEAN NOT NULL DEFAULT FALSE;

-- ---------------------------------------------------------------------------
-- General notes — relax the CHECK so trade_id IS NULL AND day IS NULL is valid.
-- (We re-create the constraint with no requirement, then add a new index for
--  fast lookup of "general" notes.)
-- ---------------------------------------------------------------------------
ALTER TABLE journal_entries DROP CONSTRAINT journal_entries_check;
CREATE INDEX journal_general_idx
    ON journal_entries(user_id, created_at DESC)
    WHERE trade_id IS NULL AND day IS NULL;

-- ---------------------------------------------------------------------------
-- Full-text search — tsvector columns + GIN indexes
-- ---------------------------------------------------------------------------
ALTER TABLE journal_entries
    ADD COLUMN body_tsv tsvector
        GENERATED ALWAYS AS (to_tsvector('english', coalesce(body_md, ''))) STORED;
CREATE INDEX journal_body_tsv_idx ON journal_entries USING GIN (body_tsv);

ALTER TABLE forum_posts
    ADD COLUMN body_tsv tsvector
        GENERATED ALWAYS AS (to_tsvector('english', coalesce(body_md, ''))) STORED;
CREATE INDEX forum_posts_body_tsv_idx ON forum_posts USING GIN (body_tsv);

-- Trades don't have a native body — we search via the symbol + their notes.
-- The symbol gets a trigram index for partial matches in search.
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX trades_symbol_trgm_idx ON trades USING GIN (symbol gin_trgm_ops);
