-- Equity history for paper accounts: the background ticker samples
-- cash + marked position value on an interval, skipping unchanged
-- readings, so the sim has a real performance record (equity curve,
-- drawdowns) instead of only a live point-in-time number.
CREATE TABLE IF NOT EXISTS paper_equity_snapshots (
    id               BIGSERIAL PRIMARY KEY,
    paper_account_id UUID        NOT NULL REFERENCES paper_accounts(id) ON DELETE CASCADE,
    equity           NUMERIC     NOT NULL,
    cash             NUMERIC     NOT NULL,
    position_value   NUMERIC     NOT NULL,
    taken_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_paper_equity_snapshots_acct
    ON paper_equity_snapshots (paper_account_id, taken_at);
