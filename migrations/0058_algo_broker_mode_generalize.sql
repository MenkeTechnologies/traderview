-- 0058 — Generalize broker_mode from Alpaca-specific names to a
-- broker-agnostic {internal_sim, paper, live} set. Each broker
-- adapter resolves its own endpoint from (account.broker,
-- broker_mode) at runtime — see traderview_db::broker_dispatcher.
--
-- Migration plan:
--   1. Migrate existing rows so the new CHECK passes:
--        'alpaca_paper' -> 'paper'
--        'alpaca_live'  -> 'live'
--   2. Drop the old CHECK; add the new one.

UPDATE algo_strategies SET broker_mode = 'paper' WHERE broker_mode = 'alpaca_paper';
UPDATE algo_strategies SET broker_mode = 'live'  WHERE broker_mode = 'alpaca_live';

ALTER TABLE algo_strategies
    DROP CONSTRAINT IF EXISTS algo_strategies_broker_mode_check;

ALTER TABLE algo_strategies
    ADD CONSTRAINT algo_strategies_broker_mode_check
        CHECK (broker_mode IN ('internal_sim', 'paper', 'live'));

-- Rollback (manual): re-CHECK the old set + UPDATE 'paper'/'live' back
-- to 'alpaca_paper'/'alpaca_live' (lossy — loses which broker the user
-- picked through the account row, but the original 0052 schema only
-- recognized Alpaca anyway).
