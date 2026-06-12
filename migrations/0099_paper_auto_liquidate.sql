-- Opt-in forced liquidation on margin call. Default OFF: a forced
-- sale is destructive and must be chosen, not inherited.
ALTER TABLE paper_accounts
    ADD COLUMN IF NOT EXISTS auto_liquidate BOOLEAN NOT NULL DEFAULT FALSE;
