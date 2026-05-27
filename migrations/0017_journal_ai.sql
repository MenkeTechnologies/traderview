-- 0017 — LLM-powered trade journal analysis.
--
-- Adds:
--   * LLM provider config on user_settings (provider + endpoint + model + API key)
--   * journal_analyses cache table keyed by (user, trade, content_hash) so
--     re-viewing a trade doesn't re-bill the LLM
--
-- API keys are stored encrypted-at-rest by the application layer in real
-- deployments; in single-user / desktop mode they live plain.

ALTER TABLE user_settings
    ADD COLUMN llm_provider     TEXT,           -- 'openai' | 'anthropic' | 'ollama'
    ADD COLUMN llm_endpoint     TEXT,           -- override (e.g. http://localhost:11434 for ollama)
    ADD COLUMN llm_model        TEXT,           -- 'gpt-4o-mini' | 'claude-haiku-4-5' | 'llama3' etc.
    ADD COLUMN llm_api_key      TEXT,           -- bearer credential (NULL for Ollama)
    ADD COLUMN llm_max_tokens   INTEGER DEFAULT 800,
    ADD COLUMN llm_temperature  REAL DEFAULT 0.2;

CREATE TABLE journal_analyses (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    trade_id        UUID NOT NULL REFERENCES trades(id) ON DELETE CASCADE,
    content_hash    TEXT NOT NULL,              -- SHA-256 of the prompt context
    provider        TEXT NOT NULL,
    model           TEXT NOT NULL,
    prompt_tokens   INTEGER,
    response_tokens INTEGER,
    findings        JSONB NOT NULL,             -- structured: mistakes / risk_gaps / suggestions
    raw_response    TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, trade_id, content_hash)
);
CREATE INDEX journal_analyses_user_trade_idx ON journal_analyses(user_id, trade_id, created_at DESC);
