-- 0008 — trade plans + user settings + saved filter sets
-- TraderVue parity: pre-trade plans, saved report filters, user preferences.

-- Pre-trade plan that becomes the trade after fills arrive.
CREATE TABLE trade_plans (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_id      UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    symbol          TEXT NOT NULL,
    asset_class     asset_class_t NOT NULL DEFAULT 'stock',
    side            trade_side_t NOT NULL,                    -- intended direction
    intended_qty    NUMERIC(20, 8) NOT NULL,
    intended_entry  NUMERIC(20, 8) NOT NULL,
    stop_loss       NUMERIC(20, 8),
    initial_target  NUMERIC(20, 8),
    setup_notes     TEXT NOT NULL DEFAULT '',
    plan_status     TEXT NOT NULL DEFAULT 'pending',          -- pending | filled | abandoned
    linked_trade_id UUID REFERENCES trades(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    filled_at       TIMESTAMPTZ,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX trade_plans_user_idx ON trade_plans(user_id, created_at DESC);
CREATE INDEX trade_plans_status_idx ON trade_plans(user_id, plan_status);

-- User-saved filter sets (re-applied in the Trades/Reports view).
CREATE TABLE filter_sets (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    payload         JSONB NOT NULL,                           -- arbitrary filter shape
    is_default      BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, name)
);

-- Per-user settings (dashboard layout, default account, base currency override, theme).
CREATE TABLE user_settings (
    user_id         UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    default_account_id UUID REFERENCES accounts(id) ON DELETE SET NULL,
    base_currency   TEXT NOT NULL DEFAULT 'USD',              -- display currency for cross-account totals
    timezone        TEXT NOT NULL DEFAULT 'America/New_York',
    theme           TEXT NOT NULL DEFAULT 'cyberpunk',
    starting_cash   NUMERIC(20, 8) NOT NULL DEFAULT 0,        -- baseline for equity curve
    dashboard_layout JSONB NOT NULL DEFAULT '{}'::jsonb,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Currency conversion rates (only populated when multi-currency accounts exist).
CREATE TABLE fx_rates (
    base            TEXT NOT NULL,
    quote           TEXT NOT NULL,
    day             DATE NOT NULL,
    rate            NUMERIC(20, 10) NOT NULL,
    source          TEXT NOT NULL DEFAULT 'manual',
    PRIMARY KEY (base, quote, day)
);
