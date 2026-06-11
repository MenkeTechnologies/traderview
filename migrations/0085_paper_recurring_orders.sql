-- Auto-invest: recurring notional buys on the paper account ("$500 of
-- SPY weekly"). The background pass submits due orders through the
-- normal paper fill path and advances next_run_at by the cadence FROM
-- THE SCHEDULED TIME so the schedule never drifts later.
CREATE TABLE IF NOT EXISTS paper_recurring_orders (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID NOT NULL,
    account_id   UUID NOT NULL REFERENCES paper_accounts(id) ON DELETE CASCADE,
    symbol       TEXT NOT NULL,
    notional_usd NUMERIC NOT NULL,
    cadence      TEXT NOT NULL CHECK (cadence IN ('daily', 'weekly', 'monthly')),
    enabled      BOOLEAN NOT NULL DEFAULT TRUE,
    next_run_at  TIMESTAMPTZ NOT NULL,
    last_status  TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS paper_recurring_due_idx
    ON paper_recurring_orders (enabled, next_run_at);
