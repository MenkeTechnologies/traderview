-- Per-user market-data-provider credentials. Persisted on the server so
-- the user's data-source keys survive process restarts (the in-memory
-- /ticks/configure endpoint only lasts until the next reboot).
--
-- All key/secret columns are nullable — a row exists for every user (see
-- user_settings.user_id PRIMARY KEY) but providers are opt-in. The
-- `data_source_keys` module masks values as "***" when returning to the
-- frontend; never round-trip plaintext keys back to the browser.

ALTER TABLE user_settings
    ADD COLUMN finnhub_api_key   TEXT,
    ADD COLUMN alpaca_key_id     TEXT,
    ADD COLUMN alpaca_secret_key TEXT,
    ADD COLUMN alpaca_paper      BOOLEAN NOT NULL DEFAULT TRUE;
