-- Buying power: gross entry-basis exposure may not exceed
-- margin_multiplier × entry-basis equity. 1 = cash account,
-- 2 = Reg-T initial margin (the default), 4 = day-trading.
-- Before this, cash could go arbitrarily negative — unlimited free
-- leverage that silently invalidated every return number.
ALTER TABLE paper_accounts
    ADD COLUMN IF NOT EXISTS margin_multiplier NUMERIC NOT NULL DEFAULT 2;
