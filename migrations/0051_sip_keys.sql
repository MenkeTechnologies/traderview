-- SIP-feed (consolidated tape) data-source credentials.
--
-- The free Finnhub / IEX-only / 15-min-delayed paths are fine for most
-- charting, but real-time scalping needs the consolidated SIP tape
-- (CTA Plan for NYSE/AMEX, UTP Plan for Nasdaq, OPRA for options).
-- We support the two most common SIP providers + a per-user opt-in
-- flag on Alpaca's SIP feed (since their Live tier with SIP costs
-- more than the default IEX-only feed).
--
-- All new columns are nullable / default-false so existing rows keep
-- their previous behavior; the user has to opt in by entering a key.

ALTER TABLE user_settings
    ADD COLUMN IF NOT EXISTS polygon_api_key      TEXT,
    ADD COLUMN IF NOT EXISTS databento_api_key    TEXT,
    ADD COLUMN IF NOT EXISTS alpaca_use_sip_feed  BOOLEAN NOT NULL DEFAULT FALSE;

-- Rollback (manual):
--   ALTER TABLE user_settings DROP COLUMN polygon_api_key, DROP COLUMN databento_api_key, DROP COLUMN alpaca_use_sip_feed;
