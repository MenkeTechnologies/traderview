-- 0011 — Warrior Trading + Zendoo parity: paper trading, alerts, hotkeys,
-- scan presets, sector cache, risk goals on user_settings.

-- ---------------------------------------------------------------------------
-- Paper trading (Warrior Trading simulator)
-- ---------------------------------------------------------------------------
CREATE TABLE paper_accounts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    starting_cash   NUMERIC(20, 8) NOT NULL DEFAULT 200000,
    cash            NUMERIC(20, 8) NOT NULL DEFAULT 200000,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    reset_at        TIMESTAMPTZ,
    UNIQUE (user_id, name)
);

CREATE TYPE paper_order_status_t AS ENUM ('pending', 'filled', 'cancelled', 'rejected');
CREATE TYPE paper_order_type_t   AS ENUM ('market', 'limit', 'stop', 'stop_limit');

CREATE TABLE paper_orders (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    paper_account_id UUID NOT NULL REFERENCES paper_accounts(id) ON DELETE CASCADE,
    symbol          TEXT NOT NULL,
    side            side_t NOT NULL,
    qty             NUMERIC(20, 8) NOT NULL,
    order_type      paper_order_type_t NOT NULL DEFAULT 'market',
    limit_price     NUMERIC(20, 8),
    stop_price      NUMERIC(20, 8),
    status          paper_order_status_t NOT NULL DEFAULT 'pending',
    filled_price    NUMERIC(20, 8),
    filled_qty      NUMERIC(20, 8),
    fee             NUMERIC(20, 8) NOT NULL DEFAULT 0,
    submitted_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    filled_at       TIMESTAMPTZ,
    cancel_at       TIMESTAMPTZ,
    reject_reason   TEXT
);
CREATE INDEX paper_orders_account_idx ON paper_orders(paper_account_id, submitted_at DESC);

CREATE TABLE paper_positions (
    paper_account_id UUID NOT NULL REFERENCES paper_accounts(id) ON DELETE CASCADE,
    symbol          TEXT NOT NULL,
    qty             NUMERIC(20, 8) NOT NULL,    -- + long, - short
    avg_price       NUMERIC(20, 8) NOT NULL,
    realized_pnl    NUMERIC(20, 8) NOT NULL DEFAULT 0,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (paper_account_id, symbol)
);

-- ---------------------------------------------------------------------------
-- Alert rules (Zendoo audio + browser notification alerts)
-- ---------------------------------------------------------------------------
CREATE TYPE alert_trigger_t AS ENUM (
    'price_above', 'price_below',
    'pct_up', 'pct_down',
    'volume_surge',
    'new_high_of_day', 'new_low_of_day',
    'rsi_above', 'rsi_below',
    'cross_sma50', 'cross_sma200'
);

CREATE TABLE alert_rules (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    symbol          TEXT NOT NULL,
    trigger         alert_trigger_t NOT NULL,
    threshold       NUMERIC(20, 8),                 -- price / % / multiple
    sound           TEXT NOT NULL DEFAULT 'bell',   -- bell | chime | voice
    voice_text      TEXT,                           -- for sound = 'voice'
    enabled         BOOLEAN NOT NULL DEFAULT TRUE,
    triggered_at    TIMESTAMPTZ,                    -- last firing
    trigger_count   INTEGER NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX alert_rules_user_idx ON alert_rules(user_id, enabled);

-- ---------------------------------------------------------------------------
-- Hotkeys (Warrior Trading DAS-style key bindings, repurposed for journal UX)
-- ---------------------------------------------------------------------------
CREATE TABLE hotkeys (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,        -- 'Sell all' / 'Tag setup' / etc.
    combo           TEXT NOT NULL,        -- 'ctrl+z', 'shift+f1', etc.
    action          TEXT NOT NULL,        -- 'paper_sell_all' | 'jump_to_trade' | ...
    payload         JSONB NOT NULL DEFAULT '{}'::jsonb,
    UNIQUE (user_id, combo)
);

-- ---------------------------------------------------------------------------
-- Scan presets (user-saved scanner configurations)
-- ---------------------------------------------------------------------------
CREATE TABLE scan_presets (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    kind            TEXT NOT NULL,        -- 'gappers' | 'momentum' | 'hod' | 'low_float' | '52w_high' | 'custom'
    criteria        JSONB NOT NULL DEFAULT '{}'::jsonb,
    sound_on_hit    BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, name)
);

-- ---------------------------------------------------------------------------
-- Sector strength cache (11 SPDR sector ETFs)
-- ---------------------------------------------------------------------------
CREATE TABLE sector_strength (
    sector          TEXT PRIMARY KEY,        -- 'XLF', 'XLK', 'XLE', ...
    label           TEXT NOT NULL,
    price           NUMERIC(20, 8) NOT NULL,
    change_pct      NUMERIC(10, 4) NOT NULL,
    rs_vs_spy       NUMERIC(10, 4),
    fetched_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ---------------------------------------------------------------------------
-- User settings — risk dashboard fields
-- ---------------------------------------------------------------------------
ALTER TABLE user_settings
    ADD COLUMN daily_profit_goal NUMERIC(20, 8) NOT NULL DEFAULT 0,
    ADD COLUMN daily_max_loss    NUMERIC(20, 8) NOT NULL DEFAULT 0;
