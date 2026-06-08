-- 0061 — IBKR Client Portal Web API + Schwab Trader API credentials.
--
-- IBKR Client Portal API auth: usually a local gateway process holds
-- session cookies after the user logs in via web. For headless setups
-- the user passes an OAuth bearer token. We store both so the
-- dispatcher can build either kind of client. base_url defaults to
-- https://localhost:5000/v1/api but lets the user override (cloud
-- deployments etc.).
--
-- Schwab Trader API auth: OAuth 2.0 — access_token + refresh_token.
-- The refresh_token is long-lived (7 days per Schwab policy); access
-- token expires in 30 minutes and the dispatcher refreshes it on
-- demand. account_hash is the per-account identifier (not the
-- human-readable account number) used in API paths.

ALTER TABLE user_settings
    ADD COLUMN IF NOT EXISTS ibkr_account_id      TEXT,
    ADD COLUMN IF NOT EXISTS ibkr_base_url         TEXT,
    ADD COLUMN IF NOT EXISTS ibkr_bearer_token     TEXT,
    ADD COLUMN IF NOT EXISTS schwab_client_id      TEXT,
    ADD COLUMN IF NOT EXISTS schwab_client_secret  TEXT,
    ADD COLUMN IF NOT EXISTS schwab_access_token   TEXT,
    ADD COLUMN IF NOT EXISTS schwab_refresh_token  TEXT,
    ADD COLUMN IF NOT EXISTS schwab_account_hash   TEXT;

-- Rollback (manual):
--   ALTER TABLE user_settings DROP COLUMN ibkr_account_id,
--                              DROP COLUMN ibkr_base_url,
--                              DROP COLUMN ibkr_bearer_token,
--                              DROP COLUMN schwab_client_id,
--                              DROP COLUMN schwab_client_secret,
--                              DROP COLUMN schwab_access_token,
--                              DROP COLUMN schwab_refresh_token,
--                              DROP COLUMN schwab_account_hash;
