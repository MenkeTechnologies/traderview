-- IRC §1212(b) capital loss carryover ledger.
--
-- One row per (user, tax year) snapshotting the Schedule D / Capital
-- Loss Carryover Worksheet result. The carryovers are character-preserving
-- (short-term stays short-term, long-term stays long-term) per
-- §1212(b)(1)(A)/(B), and §1212(b)(2) absorbs short-term first against
-- the $3,000 / $1,500-MFS ordinary-income deduction.
--
-- The compute lives in `traderview-expense::section_1212`. This table
-- just persists the result so next year's compute can read prior-year
-- ST/LT carryovers without re-running every closed lot.

CREATE TABLE capital_loss_carryovers (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tax_year                    INTEGER NOT NULL,
    filing_status               TEXT NOT NULL,
    -- Inputs that produced the row, snapshotted for audit / re-compute.
    st_gains_year               NUMERIC(28, 2) NOT NULL DEFAULT 0,
    st_losses_year              NUMERIC(28, 2) NOT NULL DEFAULT 0,
    lt_gains_year               NUMERIC(28, 2) NOT NULL DEFAULT 0,
    lt_losses_year              NUMERIC(28, 2) NOT NULL DEFAULT 0,
    prior_st_carryover          NUMERIC(28, 2) NOT NULL DEFAULT 0,
    prior_lt_carryover          NUMERIC(28, 2) NOT NULL DEFAULT 0,
    -- Outputs.
    deductible_against_ordinary NUMERIC(28, 2) NOT NULL,
    st_absorbed_by_deduction    NUMERIC(28, 2) NOT NULL,
    lt_absorbed_by_deduction    NUMERIC(28, 2) NOT NULL,
    st_carryover_next_year      NUMERIC(28, 2) NOT NULL,
    lt_carryover_next_year      NUMERIC(28, 2) NOT NULL,
    combined_net_gain_loss      NUMERIC(28, 2) NOT NULL,
    note                        TEXT NOT NULL DEFAULT '',
    computed_at                 TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, tax_year)
);
CREATE INDEX capital_loss_carryovers_user_year_idx
    ON capital_loss_carryovers(user_id, tax_year DESC);
