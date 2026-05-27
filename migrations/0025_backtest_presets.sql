-- 0025 — Shareable backtest parameter presets.
--
-- `preset` is the full Body JSON the existing /backtest/run endpoint accepts
-- (symbol, preset variant + params, days, capital, fee). `slug` is a
-- short URL-safe id; public presets are findable by slug across users.
-- Fork: creates a NEW row copying ast and metadata, owned by the forker
-- with origin_id pointing to the source.

CREATE TABLE backtest_presets (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    description     TEXT,
    preset          JSONB NOT NULL,
    is_public       BOOLEAN NOT NULL DEFAULT FALSE,
    slug            TEXT NOT NULL UNIQUE,
    origin_id       UUID REFERENCES backtest_presets(id) ON DELETE SET NULL,
    fork_count      INTEGER NOT NULL DEFAULT 0,
    run_count       INTEGER NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, name)
);
CREATE INDEX backtest_presets_user_idx    ON backtest_presets(user_id, updated_at DESC);
CREATE INDEX backtest_presets_public_idx  ON backtest_presets(is_public, fork_count DESC) WHERE is_public;
