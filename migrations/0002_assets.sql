-- 0002 — multi-asset support (stocks / options / futures / forex)
-- TraderVue parity: per-asset class fields on executions + trades.

CREATE TYPE asset_class_t AS ENUM ('stock', 'option', 'future', 'forex');
CREATE TYPE option_type_t AS ENUM ('call', 'put');

-- ---------------------------------------------------------------------------
-- executions — multi-asset columns
-- ---------------------------------------------------------------------------
ALTER TABLE executions
    ADD COLUMN asset_class      asset_class_t NOT NULL DEFAULT 'stock',
    -- options
    ADD COLUMN option_type      option_type_t,
    ADD COLUMN strike           NUMERIC(20, 8),
    ADD COLUMN expiration       DATE,
    ADD COLUMN multiplier       NUMERIC(20, 8) NOT NULL DEFAULT 1,  -- 100 for US equity options
    -- futures (multiplier doubles as point value; tick_size/value optional)
    ADD COLUMN tick_size        NUMERIC(20, 8),
    ADD COLUMN tick_value       NUMERIC(20, 8),
    -- forex
    ADD COLUMN base_ccy         TEXT,                                -- e.g. 'EUR' for EURUSD
    ADD COLUMN quote_ccy        TEXT,                                -- e.g. 'USD' for EURUSD
    ADD COLUMN pip_size         NUMERIC(20, 8);                       -- 0.0001 or 0.01 (JPY pairs)

-- The same per-asset attributes also live on trades for fast UI render.
ALTER TABLE trades
    ADD COLUMN asset_class      asset_class_t NOT NULL DEFAULT 'stock',
    ADD COLUMN option_type      option_type_t,
    ADD COLUMN strike           NUMERIC(20, 8),
    ADD COLUMN expiration       DATE,
    ADD COLUMN multiplier       NUMERIC(20, 8) NOT NULL DEFAULT 1,
    ADD COLUMN tick_size        NUMERIC(20, 8),
    ADD COLUMN tick_value       NUMERIC(20, 8),
    ADD COLUMN base_ccy         TEXT,
    ADD COLUMN quote_ccy        TEXT,
    ADD COLUMN pip_size         NUMERIC(20, 8);

CREATE INDEX trades_asset_class_idx ON trades(account_id, asset_class);
CREATE INDEX executions_asset_class_idx ON executions(account_id, asset_class);

-- Convenience: composite key for an option leg (so the dedupe-on-symbol
-- doesn't collapse multiple legs of the same underlying).
-- We don't enforce uniqueness — multiple fills on the same option are valid.
CREATE INDEX executions_option_leg_idx
    ON executions(account_id, symbol, expiration, strike, option_type)
    WHERE asset_class = 'option';
