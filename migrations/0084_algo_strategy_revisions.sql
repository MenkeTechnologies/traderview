-- Config revision history: one row per strategy save, snapshotting the
-- PRIOR config before it's overwritten. When drift detection fires,
-- "what changed before performance fell off" has an answer.
CREATE TABLE IF NOT EXISTS algo_strategy_revisions (
    id           BIGSERIAL PRIMARY KEY,
    strategy_id  UUID NOT NULL REFERENCES algo_strategies(id) ON DELETE CASCADE,
    -- Snapshot of the config BEFORE the update that created this row.
    name         TEXT NOT NULL,
    timeframe    TEXT NOT NULL,
    side_mode    TEXT NOT NULL,
    strategy_type TEXT NOT NULL,
    entry_rules  JSONB NOT NULL,
    exit_rules   JSONB NOT NULL,
    sizing       JSONB NOT NULL,
    risk_gates   JSONB NOT NULL,
    broker_mode  TEXT NOT NULL,
    replaced_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS algo_strategy_revisions_idx
    ON algo_strategy_revisions (strategy_id, replaced_at DESC);
