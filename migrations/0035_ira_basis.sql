-- IRS Form 8606 nondeductible IRA basis ledger.
--
-- Multi-year persistence so the §408(d)(2) pro-rata rule can pull last
-- year's basis automatically. UNIQUE on (user_id, tax_year) makes
-- re-computation idempotent. Mirrors the discipline of
-- 0034_capital_loss_carryover.sql.

CREATE TABLE ira_basis_history (
    id                              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tax_year                        INTEGER NOT NULL,
    -- Form 8606 line 1 + line 2 inputs snapshot.
    nondeductible_contributions     NUMERIC(28, 2) NOT NULL DEFAULT 0,
    prior_basis                     NUMERIC(28, 2) NOT NULL DEFAULT 0,
    year_end_aggregate_value        NUMERIC(28, 2) NOT NULL DEFAULT 0,
    distributions_this_year         NUMERIC(28, 2) NOT NULL DEFAULT 0,
    conversions_to_roth             NUMERIC(28, 2) NOT NULL DEFAULT 0,
    -- Form 8606 outputs.
    line_3_total_basis_available    NUMERIC(28, 2) NOT NULL,
    line_9_proration_denominator    NUMERIC(28, 2) NOT NULL,
    line_10_proration_ratio         NUMERIC(10, 5) NOT NULL,
    line_11_nontaxable_conversion   NUMERIC(28, 2) NOT NULL,
    line_12_nontaxable_distribution NUMERIC(28, 2) NOT NULL,
    line_13_total_nontaxable        NUMERIC(28, 2) NOT NULL,
    line_14_basis_carryover         NUMERIC(28, 2) NOT NULL,
    line_15c_taxable_distribution   NUMERIC(28, 2) NOT NULL,
    line_18_taxable_conversion      NUMERIC(28, 2) NOT NULL,
    total_taxable                   NUMERIC(28, 2) NOT NULL,
    note                            TEXT NOT NULL DEFAULT '',
    computed_at                     TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, tax_year)
);
CREATE INDEX ira_basis_history_user_year_idx
    ON ira_basis_history(user_id, tax_year DESC);
