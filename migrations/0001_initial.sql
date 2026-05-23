-- traderview initial schema
-- Multi-user web mode: real users, password_hash via argon2.
-- Desktop mode: src-tauri creates a single 'local' user on first launch and auto-logs in.

CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- ---------------------------------------------------------------------------
-- users
-- ---------------------------------------------------------------------------
CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email           TEXT,                                   -- nullable for desktop 'local' user
    password_hash   TEXT,                                   -- nullable for desktop 'local' user
    display_name    TEXT NOT NULL DEFAULT '',
    is_local        BOOLEAN NOT NULL DEFAULT FALSE,         -- true = desktop auto-user
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX users_email_lower_idx ON users (lower(email)) WHERE email IS NOT NULL;

-- ---------------------------------------------------------------------------
-- accounts (broker accounts; one user can have many)
-- ---------------------------------------------------------------------------
CREATE TABLE accounts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    broker          TEXT NOT NULL,                          -- 'webull', 'ibkr', 'tos', ...
    name            TEXT NOT NULL,
    base_currency   TEXT NOT NULL DEFAULT 'USD',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX accounts_user_id_idx ON accounts(user_id);

-- ---------------------------------------------------------------------------
-- executions (one row per fill, the atom)
-- ---------------------------------------------------------------------------
CREATE TYPE side_t AS ENUM ('buy', 'sell', 'short', 'cover');

CREATE TABLE executions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    symbol          TEXT NOT NULL,
    side            side_t NOT NULL,
    qty             NUMERIC(20, 8) NOT NULL CHECK (qty > 0),
    price           NUMERIC(20, 8) NOT NULL CHECK (price >= 0),
    fee             NUMERIC(20, 8) NOT NULL DEFAULT 0,
    executed_at     TIMESTAMPTZ NOT NULL,
    broker_order_id TEXT,                                   -- null if broker didn't provide
    raw             JSONB NOT NULL DEFAULT '{}'::jsonb,     -- original row for re-parse
    import_id       UUID,                                   -- back-ref to imports
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX executions_account_executed_idx ON executions(account_id, executed_at);
CREATE INDEX executions_symbol_idx ON executions(symbol);
-- Dedupe key. broker_order_id can repeat across users so we scope by account.
-- NULLs collide as distinct under default PG behavior, which is what we want
-- for brokers that don't supply an id (we fall back to import_id-based dedupe).
CREATE UNIQUE INDEX executions_dedupe_idx
    ON executions(account_id, broker_order_id, executed_at, symbol, side, qty, price)
    WHERE broker_order_id IS NOT NULL;

-- ---------------------------------------------------------------------------
-- trades (FIFO-derived from executions, materialized)
-- ---------------------------------------------------------------------------
CREATE TYPE trade_status_t AS ENUM ('open', 'closed');
CREATE TYPE trade_side_t AS ENUM ('long', 'short');

CREATE TABLE trades (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    symbol          TEXT NOT NULL,
    side            trade_side_t NOT NULL,
    status          trade_status_t NOT NULL,
    opened_at       TIMESTAMPTZ NOT NULL,
    closed_at       TIMESTAMPTZ,
    qty             NUMERIC(20, 8) NOT NULL,
    entry_avg       NUMERIC(20, 8) NOT NULL,
    exit_avg        NUMERIC(20, 8),
    gross_pnl       NUMERIC(20, 8),
    fees            NUMERIC(20, 8) NOT NULL DEFAULT 0,
    net_pnl         NUMERIC(20, 8),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX trades_account_opened_idx ON trades(account_id, opened_at DESC);
CREATE INDEX trades_account_closed_idx ON trades(account_id, closed_at DESC);
CREATE INDEX trades_symbol_idx ON trades(symbol);

-- Link table: which executions compose which trade.
CREATE TABLE trade_executions (
    trade_id        UUID NOT NULL REFERENCES trades(id) ON DELETE CASCADE,
    execution_id    UUID NOT NULL REFERENCES executions(id) ON DELETE CASCADE,
    qty_used        NUMERIC(20, 8) NOT NULL,                -- a single execution can split across trades
    PRIMARY KEY (trade_id, execution_id)
);

-- ---------------------------------------------------------------------------
-- tags
-- ---------------------------------------------------------------------------
CREATE TABLE tags (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    color           TEXT NOT NULL DEFAULT '#00e5ff',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, name)
);

CREATE TABLE trade_tags (
    trade_id        UUID NOT NULL REFERENCES trades(id) ON DELETE CASCADE,
    tag_id          UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (trade_id, tag_id)
);

-- ---------------------------------------------------------------------------
-- journal
-- ---------------------------------------------------------------------------
CREATE TABLE journal_entries (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    trade_id        UUID REFERENCES trades(id) ON DELETE CASCADE,
    day             DATE,                                   -- set for per-day entries; null for per-trade
    body_md         TEXT NOT NULL DEFAULT '',
    mood            SMALLINT,                               -- -2..+2 optional
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (trade_id IS NOT NULL OR day IS NOT NULL)
);
CREATE INDEX journal_user_day_idx ON journal_entries(user_id, day);
CREATE INDEX journal_trade_idx ON journal_entries(trade_id);

-- ---------------------------------------------------------------------------
-- imports (audit + dedupe)
-- ---------------------------------------------------------------------------
CREATE TABLE imports (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id      UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    source          TEXT NOT NULL,                          -- 'webull', 'manual', ...
    filename        TEXT NOT NULL,
    sha256          TEXT NOT NULL,
    row_count       INTEGER NOT NULL DEFAULT 0,
    imported_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (account_id, sha256)
);

-- Wire executions.import_id back to imports now that the table exists.
ALTER TABLE executions
    ADD CONSTRAINT executions_import_fk
    FOREIGN KEY (import_id) REFERENCES imports(id) ON DELETE SET NULL;
