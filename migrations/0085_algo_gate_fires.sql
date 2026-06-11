-- Gate-fire audit: every time a risk gate skips an entry, one row.
-- Answers "which gates actually fire, how often, on what" so gate
-- configs are tuned from data instead of vibes.
CREATE TABLE IF NOT EXISTS algo_gate_fires (
    id          BIGSERIAL PRIMARY KEY,
    strategy_id UUID NOT NULL REFERENCES algo_strategies(id) ON DELETE CASCADE,
    gate        TEXT NOT NULL,
    detail      TEXT NOT NULL,
    fired_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS algo_gate_fires_strategy_idx
    ON algo_gate_fires (strategy_id, fired_at DESC);
