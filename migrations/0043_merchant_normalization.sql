-- 0043: Merchant normalization + learned category mappings.
--
-- The OCR engine sees raw text from a receipt — "WAL-MART STORE 4892",
-- "Wal*mart #482", "WAL MART INC". These all refer to the same merchant
-- but downstream rollups (top-merchants panel, duplicate detection,
-- learned-category lookups) need to treat them as one.
--
-- Two tables:
--
--   merchant_aliases — translates a noisy OCR merchant string into a
--     stable canonical form. Pre-seeded with the most common chains
--     and extended at runtime when the user merges merchants in the UI.
--     Per-user so different users can have different canonical names
--     (a small business might want "WALMART → Walmart [Local store]"
--     while a contractor uses "WALMART → Walmart [Materials]").
--
--   learned_merchant_categories — when the user changes the category
--     on a line item, remember which category they picked for that
--     merchant. Next OCR on the same merchant defaults to the
--     most-frequently-chosen category for that merchant's items.
--     Stored as (user, merchant, category) with a use_count counter
--     so we can rank ties.

CREATE TABLE IF NOT EXISTS merchant_aliases (
    -- Per-user so canonical decisions don't bleed across accounts.
    user_id        UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- The user-facing display name everything rolls up to.
    -- Free-form (user can edit) — case sensitivity preserved.
    canonical      TEXT        NOT NULL,

    -- Patterns matching raw OCR merchant strings. Each entry is a
    -- case-insensitive POSIX regex (`~*`). The most common entry is a
    -- single literal substring like 'WAL.?MART'. Multiple patterns
    -- per canonical lets us cover all the noise variants of one chain
    -- in a single row.
    alias_patterns TEXT[]      NOT NULL DEFAULT '{}',

    -- Tracking — when the alias was added and how many times the
    -- canonicalizer has hit. Useful for cleanup ("which aliases are
    -- actually getting used?") and ranking.
    use_count      INTEGER     NOT NULL DEFAULT 0,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (user_id, canonical)
);

-- Look up by user. The canonicalizer needs to scan every alias row
-- for the active user on each OCR call — a per-user index is enough.
CREATE INDEX IF NOT EXISTS idx_merchant_aliases_user
    ON merchant_aliases(user_id);

-- ------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS learned_merchant_categories (
    user_id           UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- The canonical merchant — joins back to merchant_aliases.canonical
    -- when canonicalize() resolves the raw OCR name. We store the text
    -- (not an FK) because the user can rename the canonical and we want
    -- learned categories to follow the rename via UPDATE … SET
    -- merchant_canonical = new instead of a cascade.
    merchant_canonical TEXT        NOT NULL,

    -- The category the user chose. Foreign-keyed because deleting a
    -- category SHOULD wipe its learned mappings (otherwise re-OCR
    -- would re-apply a deleted category). NB: expense_categories uses
    -- a TEXT primary key (`code`), not an integer surrogate — see
    -- migration 0029.
    category_code     TEXT        NOT NULL REFERENCES expense_categories(code) ON DELETE CASCADE,

    -- How many times the user has confirmed this (merchant, category)
    -- pair. UPSERT-incremented on every category-change PATCH the
    -- learning hook fires. We pick the highest use_count as the
    -- default suggestion on next OCR.
    use_count         INTEGER     NOT NULL DEFAULT 1,

    last_used         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (user_id, merchant_canonical, category_code)
);

-- Hot lookup: "what's the best-guess category for this merchant?"
-- Includes use_count DESC so the ORDER BY in the lookup query can
-- skip a sort.
CREATE INDEX IF NOT EXISTS idx_learned_merchant_categories_lookup
    ON learned_merchant_categories(user_id, merchant_canonical, use_count DESC);
