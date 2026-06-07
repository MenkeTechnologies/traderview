-- Broker normalization for multi-broker trade filtering.
--
-- Today `accounts.broker` is free-text ('webull', 'IBKR', 'TD Ameritrade',
-- etc.) so:
--   * Per-broker rollups can't be done in SQL without fragile string compare.
--   * Webull-typo and webull collide in the UI selector.
--   * No broker metadata exists anywhere.
--
-- This migration:
--   1. Creates a `brokers` table (one row per distinct broker per user).
--   2. Adds `accounts.broker_id UUID` FK.
--   3. Backfills `broker_id` from the existing free-text column (lowercased,
--      whitespace-trimmed key — eliminates Webull-vs-webull dupes).
--   4. Leaves `accounts.broker` text in place for back-compat. The frontend
--      reads from `broker_id` going forward; the text column is read-only
--      legacy and slated for removal in a future migration once every
--      consumer is updated.

CREATE TABLE brokers (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- Normalized lower-snake key used for deduplication. Examples:
    --   'webull', 'ibkr', 'tos', 'tasty', 'schwab', 'fidelity'.
    slug            TEXT NOT NULL,
    -- Display label the UI shows ("Webull", "Interactive Brokers"). Free
    -- text so the user can rename without breaking the slug.
    display_name    TEXT NOT NULL,
    -- Optional metadata.
    home_url        TEXT,
    notes           TEXT,
    is_default      BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, slug)
);
CREATE INDEX brokers_user_idx ON brokers(user_id);
-- Enforce one default broker per user via a partial unique index.
CREATE UNIQUE INDEX brokers_one_default_per_user_idx
    ON brokers(user_id) WHERE is_default = TRUE;

-- Backfill: one row per (user_id, normalized broker text) seen in accounts.
INSERT INTO brokers (user_id, slug, display_name)
SELECT DISTINCT
    a.user_id,
    LOWER(REGEXP_REPLACE(TRIM(a.broker), '[^a-zA-Z0-9]+', '_', 'g'))    AS slug,
    -- Display name preserves the user's original casing on the first row
    -- we encounter — picked deterministically by sorting account id.
    (
        SELECT a2.broker
        FROM accounts a2
        WHERE a2.user_id = a.user_id
          AND LOWER(REGEXP_REPLACE(TRIM(a2.broker), '[^a-zA-Z0-9]+', '_', 'g'))
              = LOWER(REGEXP_REPLACE(TRIM(a.broker), '[^a-zA-Z0-9]+', '_', 'g'))
        ORDER BY a2.created_at ASC, a2.id ASC
        LIMIT 1
    )                                                                    AS display_name
FROM accounts a
WHERE a.broker IS NOT NULL
  AND TRIM(a.broker) <> ''
ON CONFLICT (user_id, slug) DO NOTHING;

ALTER TABLE accounts
    ADD COLUMN broker_id UUID REFERENCES brokers(id) ON DELETE SET NULL;
CREATE INDEX accounts_broker_idx ON accounts(broker_id)
    WHERE broker_id IS NOT NULL;
CREATE INDEX accounts_user_broker_idx ON accounts(user_id, broker_id)
    WHERE broker_id IS NOT NULL;

-- Wire each existing account to its broker via the same normalization.
UPDATE accounts a
SET broker_id = b.id
FROM brokers b
WHERE a.user_id = b.user_id
  AND a.broker IS NOT NULL
  AND TRIM(a.broker) <> ''
  AND b.slug = LOWER(REGEXP_REPLACE(TRIM(a.broker), '[^a-zA-Z0-9]+', '_', 'g'));

-- Set the first broker per user as default if no flag was set yet.
UPDATE brokers b
SET is_default = TRUE
WHERE b.id = (
    SELECT id FROM brokers b2
    WHERE b2.user_id = b.user_id
    ORDER BY b2.created_at ASC, b2.id ASC
    LIMIT 1
)
AND NOT EXISTS (
    SELECT 1 FROM brokers b3
    WHERE b3.user_id = b.user_id AND b3.is_default = TRUE
);

-- Rollback (manual):
--   ALTER TABLE accounts DROP COLUMN broker_id;
--   DROP TABLE brokers;
