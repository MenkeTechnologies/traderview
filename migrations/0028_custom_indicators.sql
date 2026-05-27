-- 0028 — User-defined indicator presets.
--
-- definition shape (validated app-side):
--   { "kind": "sma|ema|rsi|bollinger|macd|atr|vwap", "params": {...} }
-- Multiple presets can be selected to overlay on a chart; the frontend
-- composes them, the backend evaluates one at a time per request.

CREATE TABLE custom_indicators (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    definition      JSONB NOT NULL,
    color           TEXT NOT NULL DEFAULT '#00e5ff',
    is_default      BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, name)
);
CREATE INDEX custom_indicators_user_idx ON custom_indicators(user_id, is_default DESC);
