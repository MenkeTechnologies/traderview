-- 0020 — Custom dashboards for multi-monitor / per-workflow boards.
--
-- `layout` is a JSONB array of widget objects:
--   [{ "id": "uuid", "kind": "quote", "params": { "symbol": "SPY" },
--      "x": 0, "y": 0, "w": 4, "h": 2 }, ...]
-- The frontend grid is 12 columns; w/h are grid cells.

CREATE TABLE dashboards (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    layout      JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX dashboards_user_idx ON dashboards(user_id, updated_at DESC);
