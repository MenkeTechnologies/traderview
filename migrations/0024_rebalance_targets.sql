-- 0024 — Saved portfolio rebalance targets (per-user named target sets).
--
-- `targets` is a JSONB array of { symbol, weight, price? }. The compute
-- endpoint accepts ad-hoc targets; this table just persists named templates
-- so a user can reload "60/40 stocks/bonds" or "sector rotation #3".

CREATE TABLE rebalance_targets (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    account_id      UUID REFERENCES accounts(id) ON DELETE SET NULL,
    targets         JSONB NOT NULL,
    max_trades      INTEGER NOT NULL DEFAULT 20
        CHECK (max_trades >= 0 AND max_trades <= 200),
    notes           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, name)
);
CREATE INDEX rebalance_targets_user_idx ON rebalance_targets(user_id, updated_at DESC);
