-- 0042 — Tax dashboard extensions.
--
-- Adds two pieces of structure the dashboard needs:
--
-- 1. `expense_categories.kind` — distinguishes income vs expense
--    categories so the dashboard can sum income from labelled
--    transactions instead of inferring from amount sign (cards parse
--    refunds as positive but they aren't income; some bank parsers
--    encode income as negative). Default 'expense'; user marks any
--    category as 'income' to flip how it contributes.
--
-- 2. `estimated_tax_payments` — per-user, per-quarter estimated tax
--    payment log. IRS quarterly dues are Apr 15 / Jun 15 / Sep 15 /
--    Jan 15 (next year). The dashboard surfaces these as
--    scheduled-vs-paid alongside the Schedule C/E totals.

ALTER TABLE expense_categories
    ADD COLUMN IF NOT EXISTS kind TEXT NOT NULL DEFAULT 'expense'
        CHECK (kind IN ('income', 'expense'));

CREATE TABLE IF NOT EXISTS estimated_tax_payments (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tax_year    INTEGER NOT NULL,                          -- e.g. 2026
    quarter     SMALLINT NOT NULL CHECK (quarter BETWEEN 1 AND 4),
    paid_at     DATE NOT NULL,
    amount      NUMERIC(20, 2) NOT NULL,
    method      TEXT NOT NULL DEFAULT '',                  -- 'EFTPS' / 'IRS Direct Pay' / etc.
    note        TEXT NOT NULL DEFAULT '',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS estimated_tax_payments_user_year_idx
    ON estimated_tax_payments (user_id, tax_year, quarter);
