-- 0059 — Tradier brokerage API credentials.
--
-- Used by the algo broker_dispatcher to instantiate TradierTrading
-- clients for strategies whose account.broker = 'tradier'. The
-- (token, account_number) pair is what Tradier's REST API needs;
-- everything else (sandbox vs prod base URL) is derived from the
-- strategy's broker_mode at runtime.

ALTER TABLE user_settings
    ADD COLUMN IF NOT EXISTS tradier_access_token  TEXT,
    ADD COLUMN IF NOT EXISTS tradier_account_id    TEXT,
    ADD COLUMN IF NOT EXISTS tradier_sandbox       BOOLEAN NOT NULL DEFAULT TRUE;

-- Rollback (manual):
--   ALTER TABLE user_settings DROP COLUMN tradier_access_token,
--                              DROP COLUMN tradier_account_id,
--                              DROP COLUMN tradier_sandbox;
