-- 0019 — Per-PAT rate limit.
--
-- Default 60 requests/minute. Stored on the token, not on the user, so an
-- automation that needs more throughput can be issued a separate token
-- with a higher limit without affecting other integrations.

ALTER TABLE api_tokens
    ADD COLUMN rate_limit_per_min INTEGER NOT NULL DEFAULT 60
        CHECK (rate_limit_per_min > 0 AND rate_limit_per_min <= 10000);
