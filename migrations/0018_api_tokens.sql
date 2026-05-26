-- 0018 — Personal Access Tokens (PATs) for the public API.
--
-- Token wire format:  pat_<24-char-prefix>_<32-char-secret>
--   * `prefix` is stored plain for fast lookup
--   * `hash` is the argon2 hash of `prefix` + '_' + `secret`
-- Only the prefix is recoverable after creation — the secret is shown to the
-- user once at creation time and never again.

CREATE TABLE api_tokens (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,                       -- human label e.g. "n8n staging"
    prefix          TEXT NOT NULL UNIQUE,                -- 24 chars, plaintext (lookup key)
    hash            TEXT NOT NULL,                       -- argon2(prefix + '_' + secret)
    scopes          TEXT[] NOT NULL DEFAULT '{read}',    -- 'read' | 'write' | 'admin'
    expires_at      TIMESTAMPTZ,                         -- NULL = never
    revoked_at      TIMESTAMPTZ,                         -- NULL = active
    last_used_at    TIMESTAMPTZ,
    use_count       BIGINT NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX api_tokens_user_idx ON api_tokens(user_id, revoked_at);
