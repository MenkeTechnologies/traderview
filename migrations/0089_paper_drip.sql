-- DRIP: per-account dividend reinvestment. When ON, each positive
-- dividend credit immediately buys more of the paying symbol at
-- market (fractional shares). paper_dividends.reinvested records
-- whether the credit was reinvested.
ALTER TABLE paper_accounts
    ADD COLUMN IF NOT EXISTS drip BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE paper_dividends
    ADD COLUMN IF NOT EXISTS reinvested BOOLEAN NOT NULL DEFAULT FALSE;
