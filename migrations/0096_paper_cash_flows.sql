-- Deposits / withdrawals — the missing account-funding ops. Signed
-- amounts (deposit positive); the statement uses the period's net
-- flow for a modified-Dietz return so a deposit never reads as gain.
CREATE TABLE paper_cash_flows (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    paper_account_id UUID NOT NULL REFERENCES paper_accounts(id) ON DELETE CASCADE,
    amount           NUMERIC NOT NULL,
    note             TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX paper_cash_flows_account_idx ON paper_cash_flows(paper_account_id, created_at DESC);
