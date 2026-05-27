-- 0026 — Trading goals (monthly / quarterly / yearly targets).
--
-- Each goal binds an account (optional - null = all accounts), a period
-- window (start_date..end_date inclusive), and up to three target metrics
-- (net P/L, win rate, max drawdown %). Progress is computed at query time
-- against closed trades whose opened_at falls inside the window.

CREATE TABLE trading_goals (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                 UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    account_id              UUID REFERENCES accounts(id) ON DELETE CASCADE,
    name                    TEXT NOT NULL,
    period                  TEXT NOT NULL,    -- 'monthly' | 'quarterly' | 'yearly' | 'custom'
    start_date              DATE NOT NULL,
    end_date                DATE NOT NULL CHECK (end_date >= start_date),
    target_pnl              NUMERIC(20, 2),
    target_win_rate         REAL,             -- 0..1
    target_max_drawdown_pct REAL,             -- positive number; cap to stay under
    notes                   TEXT,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX trading_goals_user_idx        ON trading_goals(user_id, start_date DESC);
CREATE INDEX trading_goals_account_idx     ON trading_goals(account_id);
