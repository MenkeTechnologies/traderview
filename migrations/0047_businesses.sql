-- traderview multi-business support.
--
-- Lets a Schedule C filer have N separate businesses (LLC, sole prop,
-- side gig) with per-business expense tracking, dashboards, and
-- (eventually) separate Schedule C output. The Schedule E side already
-- does this via `rental_properties` — this migration brings Schedule C
-- to parity.
--
-- Backwards compat: every existing row gets `business_id = NULL` which
-- the API treats as "default / aggregated". Existing single-business
-- users see no behavioral change until they create a second business.

CREATE TYPE business_entity_type_t AS ENUM (
    'sole_prop',
    'smllc',           -- single-member LLC (Schedule C)
    'llc',             -- multi-member LLC (typically Form 1065)
    's_corp',
    'c_corp',
    'partnership'
);

CREATE TABLE businesses (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id        UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name           TEXT NOT NULL,
    ein            TEXT,                                -- xx-xxxxxxx; optional
    entity_type    business_entity_type_t NOT NULL DEFAULT 'sole_prop',
    naics_code     TEXT,                                -- 6-digit, optional
    principal_addr TEXT,
    -- True for the user's "primary" business — front-end auto-selects
    -- this when no explicit business_id is in the URL/local state.
    is_default     BOOLEAN NOT NULL DEFAULT FALSE,
    started_at     DATE,
    ended_at       DATE,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX businesses_user_idx ON businesses(user_id);
-- Enforce at most one default business per user via a partial unique index.
CREATE UNIQUE INDEX businesses_one_default_per_user_idx
    ON businesses(user_id) WHERE is_default = TRUE;

-- Tag every transaction with the business it belongs to.
-- NULL = "personal / not assigned" (default for existing rows).
ALTER TABLE transactions
    ADD COLUMN business_id UUID REFERENCES businesses(id) ON DELETE SET NULL;
CREATE INDEX transactions_business_idx ON transactions(business_id)
    WHERE business_id IS NOT NULL;

-- Rentals can be owned by a business entity (LLC owns property).
ALTER TABLE rental_properties
    ADD COLUMN business_id UUID REFERENCES businesses(id) ON DELETE SET NULL;
CREATE INDEX rental_properties_business_idx ON rental_properties(business_id)
    WHERE business_id IS NOT NULL;

-- One Schedule C per business per year. NULL business_id = legacy
-- single-business return (preserved for backwards compat).
ALTER TABLE tax_returns
    ADD COLUMN business_id UUID REFERENCES businesses(id) ON DELETE SET NULL;
CREATE INDEX tax_returns_user_year_business_idx
    ON tax_returns(user_id, tax_year, business_id);

-- Receipt items live in JSONB on `receipts.ocr_extracted` and are
-- tagged with `business_id` from the frontend. No DDL needed here —
-- documented for the reader.
--
-- Rollback (manual):
--   ALTER TABLE tax_returns       DROP COLUMN business_id;
--   ALTER TABLE rental_properties DROP COLUMN business_id;
--   ALTER TABLE transactions      DROP COLUMN business_id;
--   DROP TABLE businesses;
--   DROP TYPE  business_entity_type_t;
