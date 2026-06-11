-- 0083 — Paper split adjustment: positions held through a stock split
-- get qty × ratio and avg_price ÷ ratio (value-preserving; shorts scale
-- the same way). One row per account × symbol × split date; the UNIQUE
-- constraint makes the background pass idempotent.

CREATE TABLE paper_splits (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    paper_account_id UUID NOT NULL REFERENCES paper_accounts(id) ON DELETE CASCADE,
    symbol           TEXT NOT NULL,
    split_date       DATE NOT NULL,
    numerator        NUMERIC(20, 8) NOT NULL,
    denominator      NUMERIC(20, 8) NOT NULL,
    qty_before       NUMERIC(20, 8) NOT NULL,
    qty_after        NUMERIC(20, 8) NOT NULL,
    applied_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (paper_account_id, symbol, split_date)
);
CREATE INDEX paper_splits_account_idx ON paper_splits(paper_account_id, split_date DESC);
