-- 0070 — autotrade exit logic.
--
-- Backtest fixes 20d horizon. Autopilot has no exit — positions just
-- sit. Without an exit rule, realized return ≠ backtest return, and
-- the kelly-sized position keeps drifting whether the signal is still
-- valid or not.
--
-- Two exit rules attached to autotrade-opened paper positions:
--
--   1. Time-stop — flatten after max_holding_days
--      (default 20 = the horizon Kelly defaults to)
--   2. Signal-degradation — if the source confluence row no longer
--      scores ≥ min_score for `degradation_threshold_checks`
--      consecutive evaluations, flatten
--
-- Manual positions (entered by the user, not the autopilot) are
-- untouched — only rows in `autotrade_position_tags` qualify for
-- automatic flattening.

CREATE TABLE IF NOT EXISTS autotrade_position_tags (
    id                            BIGSERIAL    PRIMARY KEY,
    paper_account_id              UUID         NOT NULL REFERENCES paper_accounts(id) ON DELETE CASCADE,
    symbol                        TEXT         NOT NULL,
    opened_by_log_id              BIGINT       REFERENCES confluence_autotrade_log(id) ON DELETE SET NULL,
    opened_at                     TIMESTAMPTZ  NOT NULL DEFAULT now(),
    score_at_open                 DOUBLE PRECISION NOT NULL,
    last_observed_score           DOUBLE PRECISION,
    consecutive_degraded_checks   INT          NOT NULL DEFAULT 0,
    last_evaluated_at             TIMESTAMPTZ,
    UNIQUE (paper_account_id, symbol)
);

CREATE INDEX IF NOT EXISTS autotrade_position_tags_account_idx
    ON autotrade_position_tags (paper_account_id);

ALTER TABLE confluence_autotrade_config
    ADD COLUMN IF NOT EXISTS max_holding_days                INT NOT NULL DEFAULT 20,
    ADD COLUMN IF NOT EXISTS degradation_threshold_checks    INT NOT NULL DEFAULT 3;

-- Log gains a new action type (no schema change — `action` is TEXT):
--   * 'exit_time_stop'    — closed because held > max_holding_days
--   * 'exit_degraded'     — closed because signal degraded for N checks

-- Rollback (manual):
--   DROP TABLE autotrade_position_tags;
--   ALTER TABLE confluence_autotrade_config DROP COLUMN degradation_threshold_checks;
--   ALTER TABLE confluence_autotrade_config DROP COLUMN max_holding_days;
