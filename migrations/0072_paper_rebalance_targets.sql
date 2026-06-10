-- 0072 — paper-account rebalance targets.
--
-- The existing `rebalance_targets` table is tied to broker accounts
-- (account_id). For paper-account investing workflows we want target
-- weight sets specific to the paper account — typically the same user
-- has multiple target portfolios (e.g. "60-40", "Boglehead 3-fund",
-- "magic-formula top 20") and switches between them.
--
-- Target weights are stored as a JSON object: { "AAPL": 0.05, ... }.
-- Weights are fractions (0..1); sum should normally equal 1 minus any
-- explicit cash allocation (stored as the `cash_target_pct` column).

CREATE TABLE IF NOT EXISTS paper_rebalance_targets (
    id                   UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id              UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name                 TEXT         NOT NULL,
    targets              JSONB        NOT NULL DEFAULT '{}'::jsonb,
    cash_target_pct      DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    drift_threshold_pct  DOUBLE PRECISION NOT NULL DEFAULT 5.0,
    max_trades           INT          NOT NULL DEFAULT 20,
    notes                TEXT,
    created_at           TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ  NOT NULL DEFAULT now(),
    UNIQUE (user_id, name)
);

CREATE INDEX IF NOT EXISTS paper_rebalance_targets_user_idx
    ON paper_rebalance_targets (user_id);

-- Rollback (manual):
--   DROP TABLE paper_rebalance_targets;
