-- Cash sweep interest: idle paper cash accrues at a per-account APY
-- (0 = off). last_interest_on makes the daily credit exactly-once
-- across restarts; paper_interest is the audit trail.
ALTER TABLE paper_accounts
    ADD COLUMN IF NOT EXISTS cash_apy_pct     NUMERIC NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS last_interest_on DATE;

CREATE TABLE paper_interest (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    paper_account_id UUID NOT NULL REFERENCES paper_accounts(id) ON DELETE CASCADE,
    credited_on      DATE NOT NULL,
    amount           NUMERIC NOT NULL,
    apy_pct          NUMERIC NOT NULL,
    days             INT NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX paper_interest_account_idx ON paper_interest(paper_account_id, credited_on DESC);
