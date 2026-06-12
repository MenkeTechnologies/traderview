-- Short borrow fees — the cost side of the cash sweep. Daily debit on
-- equity-short ENTRY notional (avg_price × |qty|; marked-value GC
-- rates need a quote dependency the deterministic pass deliberately
-- avoids). Audit rows share paper_interest, distinguished by kind.
ALTER TABLE paper_accounts
    ADD COLUMN IF NOT EXISTS borrow_apy_pct NUMERIC NOT NULL DEFAULT 0;

ALTER TABLE paper_interest
    ADD COLUMN IF NOT EXISTS kind TEXT NOT NULL DEFAULT 'cash_sweep';
