-- 0016 — Per-user chart drawings (trendlines, horizontal levels, fibs, text).
--
-- `kind` tells the frontend how to render `points`:
--   trendline → two {time, price} anchors
--   hline     → one {price} anchor (time ignored)
--   fib       → two anchors (high & low); UI draws standard 0/.236/.382/.5/.618/.786/1 lines
--   text      → one anchor + the `label` field is the visible text
--
-- `points` is a JSONB array — schema validation lives in the application layer.

CREATE TYPE chart_drawing_kind_t AS ENUM ('trendline', 'hline', 'fib', 'text');

CREATE TABLE chart_drawings (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    symbol          TEXT NOT NULL,
    kind            chart_drawing_kind_t NOT NULL,
    points          JSONB NOT NULL,
    label           TEXT,
    color           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX chart_drawings_user_symbol_idx ON chart_drawings(user_id, symbol);
