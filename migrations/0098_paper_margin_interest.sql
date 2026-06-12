-- Margin loan interest — the third leg of the funding-cost triangle
-- (sweep credit on idle cash, borrow fee on shorts, loan interest on
-- debit balances). Separate rate from stock borrow: a margin loan and
-- a hard-to-borrow locate are different costs.
ALTER TABLE paper_accounts
    ADD COLUMN IF NOT EXISTS margin_apy_pct NUMERIC NOT NULL DEFAULT 0;
