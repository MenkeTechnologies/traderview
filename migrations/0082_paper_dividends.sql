-- 0082 — Paper dividend crediting: cash credits for paper positions held
-- through an ex-date (shorts are debited). One row per account × symbol ×
-- ex-date; the UNIQUE constraint makes the background crediting pass
-- idempotent across restarts and overlapping runs.

CREATE TABLE paper_dividends (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    paper_account_id UUID NOT NULL REFERENCES paper_accounts(id) ON DELETE CASCADE,
    symbol           TEXT NOT NULL,
    ex_date          DATE NOT NULL,
    amount_per_share NUMERIC(20, 8) NOT NULL,
    qty              NUMERIC(20, 8) NOT NULL,    -- + long credit, - short debit
    cash_credited    NUMERIC(20, 8) NOT NULL,
    credited_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (paper_account_id, symbol, ex_date)
);
CREATE INDEX paper_dividends_account_idx ON paper_dividends(paper_account_id, ex_date DESC);
