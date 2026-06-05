-- 0044: Full-text search across receipt OCR text.
--
-- The "where did I buy that?" workflow is non-trivial at scale: a user
-- with 10k receipts can't scroll through them to find the one mentioning
-- "drill bit set" or "anti-seize lubricant". A LIKE '%drill bit%' scan
-- works for hundreds of receipts but degenerates linearly; tsvector +
-- GIN makes it instant at any scale.
--
-- Strategy:
--   * Add `ocr_text_tsv` — a generated tsvector on `ocr_merchant ||
--     ocr_text`. Generated columns auto-recompute on UPDATE, so the
--     OCR completion path doesn't need a separate UPDATE.
--   * Add a GIN index on the tsvector for sublinear search.
--   * Search query uses `plainto_tsquery` so user input never has to
--     escape special characters — "drill bit & screwdriver" is treated
--     as three plain terms.

ALTER TABLE receipts
    ADD COLUMN IF NOT EXISTS ocr_text_tsv tsvector
        GENERATED ALWAYS AS (
            -- Heavy weight on the merchant so a query like "walmart"
            -- ranks merchant-name hits above body mentions.
            setweight(to_tsvector('english', COALESCE(ocr_merchant, '')), 'A') ||
            setweight(to_tsvector('english', COALESCE(ocr_text,     '')), 'B')
        ) STORED;

CREATE INDEX IF NOT EXISTS idx_receipts_ocr_text_tsv
    ON receipts USING GIN (ocr_text_tsv);

-- A per-user index covering (user_id, created_at DESC) is already
-- present from migration 0029; the GIN is purely for the tsv match
-- and the query planner combines them via bitmap AND.
