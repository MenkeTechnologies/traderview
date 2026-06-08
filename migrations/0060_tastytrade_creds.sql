-- 0060 — Tastytrade brokerage API credentials.
--
-- Tastytrade auth: POST /sessions with {login, password, remember-me}
-- mints a session-token. We store the long-lived token OR the
-- username+password so the dispatcher can re-mint a token on demand.
-- (Both columns nullable; UI lets users pick one path.)
--
-- account_number selects which Tastytrade account the strategy
-- submits against — different from the account.id we use internally.
--
-- Sandbox flag (default true) routes to api.cert.tastyworks.com;
-- false to api.tastyworks.com.

ALTER TABLE user_settings
    ADD COLUMN IF NOT EXISTS tastytrade_login          TEXT,
    ADD COLUMN IF NOT EXISTS tastytrade_password       TEXT,
    ADD COLUMN IF NOT EXISTS tastytrade_session_token  TEXT,
    ADD COLUMN IF NOT EXISTS tastytrade_account_number TEXT,
    ADD COLUMN IF NOT EXISTS tastytrade_sandbox        BOOLEAN NOT NULL DEFAULT TRUE;

-- Rollback (manual):
--   ALTER TABLE user_settings DROP COLUMN tastytrade_login,
--                              DROP COLUMN tastytrade_password,
--                              DROP COLUMN tastytrade_session_token,
--                              DROP COLUMN tastytrade_account_number,
--                              DROP COLUMN tastytrade_sandbox;
