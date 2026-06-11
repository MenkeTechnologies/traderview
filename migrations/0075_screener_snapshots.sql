-- Periodic screener snapshots: the background refresher stores each
-- run's full payload so views serve history and shape-flip changes
-- without recomputing, mirroring the golden-stars persistence model.
CREATE TABLE IF NOT EXISTS screener_snapshots (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    screener    TEXT        NOT NULL,
    payload     JSONB       NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_screener_snapshots_latest
    ON screener_snapshots (screener, created_at DESC);
