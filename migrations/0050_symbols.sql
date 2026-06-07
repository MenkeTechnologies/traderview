-- Extend the existing `symbols` table (migration 0007) with the
-- richer Finnhub fields so the global frontend autocomplete can show
-- a human description alongside each ticker. Drops nothing — the old
-- price-bars consumers keep reading `name`, `exchange`, etc.
--
-- New columns are nullable + IF NOT EXISTS so re-running on a partial
-- previous install of this migration (e.g. an aborted seed left the
-- columns half-applied) doesn't fail.

ALTER TABLE symbols
    ADD COLUMN IF NOT EXISTS description      TEXT,
    ADD COLUMN IF NOT EXISTS display_symbol   TEXT,
    ADD COLUMN IF NOT EXISTS type             TEXT,
    ADD COLUMN IF NOT EXISTS mic              TEXT,
    ADD COLUMN IF NOT EXISTS figi             TEXT,
    ADD COLUMN IF NOT EXISTS isin             TEXT,
    ADD COLUMN IF NOT EXISTS share_class_figi TEXT,
    ADD COLUMN IF NOT EXISTS fetched_at       TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- Case-insensitive prefix searches sort by lowered symbol.
CREATE INDEX IF NOT EXISTS symbols_lower_idx ON symbols (LOWER(symbol));

-- Rollback (manual): the ALTER above is additive so reverting just
-- means leaving the new columns NULL on every row, or
--   ALTER TABLE symbols DROP COLUMN description, ... DROP COLUMN fetched_at;
