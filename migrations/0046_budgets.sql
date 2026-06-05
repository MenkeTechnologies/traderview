-- 0046: Personal finance / budgeting.
--
-- Two tables:
--   budgets               — per-category monthly spending caps.
--   budget_savings_goals  — one row per user, the monthly amount they
--                           want to save (income − expense ≥ target).
--
-- Both surfaces feed the `budget_status` dashboard widget and the
-- full `#budget` view, which read live actuals from the existing
-- `expense_transactions` table.

CREATE TABLE IF NOT EXISTS budgets (
    user_id        UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- Foreign key on expense_categories.code (TEXT primary key from
    -- migration 0029). Deleting a category cascades the budget away —
    -- a budget on a deleted category is meaningless.
    category_code  TEXT         NOT NULL REFERENCES expense_categories(code) ON DELETE CASCADE,
    monthly_limit  NUMERIC(14, 2) NOT NULL CHECK (monthly_limit >= 0),
    -- Soft pause — when true, the category still shows in reports
    -- but doesn't trigger an "over budget" flag. Useful for
    -- seasonal categories (holiday gifts, car maintenance) that
    -- spike one month and stay quiet the rest.
    paused         BOOLEAN      NOT NULL DEFAULT FALSE,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, category_code)
);

CREATE INDEX IF NOT EXISTS idx_budgets_user
    ON budgets(user_id);

-- ------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS budget_savings_goals (
    user_id        UUID         PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    monthly_target NUMERIC(14, 2) NOT NULL DEFAULT 0 CHECK (monthly_target >= 0),
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
