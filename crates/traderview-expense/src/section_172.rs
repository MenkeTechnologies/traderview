//! IRC §172 — Net Operating Loss deduction.
//!
//! Foundational rule for every business taxpayer (including
//! sole-proprietor traders, S-corp shareholders, partnership partners
//! flowing through losses, and corporations themselves). When deductions
//! exceed gross income, §172 lets the excess offset taxable income in
//! other years.
//!
//! **Three statutory regimes by NOL year:**
//!
//! 1. **Pre-2018 (legacy)**: 2-year carryback / 20-year carryforward;
//!    NO 80% limit. Applies to NOLs from tax years beginning before
//!    January 1, 2018. Most pre-TCJA carryforwards have now expired or
//!    been absorbed.
//!
//! 2. **CARES Act 2018-2020**: 5-year carryback PLUS 100% of taxable
//!    income offset (the 80% limit was temporarily suspended). Applied
//!    to NOLs arising in tax years 2018, 2019, and 2020 only. Sunset
//!    after 2020.
//!
//! 3. **Permanent TCJA (post-2020)**: indefinite carryforward, no
//!    carryback (except farming/insurance 2-year), and 80% of taxable
//!    income limit reapplied. Effective for NOLs in tax years 2021+.
//!
//! **Farming + insurance carve-out**: §172(b)(1)(B) preserves the
//! 2-year carryback for farming losses and certain insurance-company
//! losses even under the post-TCJA permanent regime. Module flags this
//! with the `farming_loss` input.
//!
//! **80% limit math**: under the permanent post-2020 regime, the NOL
//! deduction allowed is the LESSER of:
//!   - the available NOL carryforward (prior carryover + current loss),
//!   - 80% of taxable income BEFORE the NOL deduction.
//!
//! Remaining NOL carries forward indefinitely.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section172Input {
    /// NOL generated in the CURRENT tax year (positive number — the
    /// magnitude of excess deductions over gross income). Zero in a
    /// profit year.
    pub current_year_nol: Decimal,
    /// Taxable income before the §172 deduction in the current year.
    /// The amount the NOL can offset. Zero in a loss year (there's
    /// nothing to absorb).
    pub current_year_taxable_income_before_nol: Decimal,
    /// Sum of prior-year NOL carryforwards entering the current year.
    pub prior_year_nol_carryforward: Decimal,
    /// Current tax year for regime selection.
    pub tax_year: i32,
    /// True if the loss is a farming-business or insurance-company
    /// NOL eligible for the 2-year carryback under §172(b)(1)(B).
    pub farming_or_insurance_loss: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Section172Regime {
    /// Pre-TCJA: 2-year carryback, 20-year carryforward, no 80% limit.
    Pre2018Legacy,
    /// CARES Act 2018-2020: 5-year carryback, 100% taxable income
    /// offset.
    CaresAct,
    /// Permanent TCJA post-2020: indefinite carryforward, no carryback
    /// (except farming/insurance 2-year), 80% taxable income limit.
    PermanentTcja,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section172Result {
    pub regime: Section172Regime,
    /// NOL available to deduct = prior_carryforward + current-year loss
    /// (but current-year loss is typically carried forward, not absorbed
    /// in the same year — kept separate fields).
    pub total_nol_available: Decimal,
    /// 80% × taxable_income — the post-2020 limit. Equals taxable
    /// income for CARES Act years and pre-2018 (no limit applies).
    pub eighty_percent_limit_dollars: Decimal,
    /// True if the 80% limit binds (i.e., NOL exceeds the limit).
    pub eighty_percent_limit_binds: bool,
    /// NOL deduction allowed this year — min(available, limit).
    pub nol_deduction_allowed: Decimal,
    /// Taxable income after applying the NOL deduction.
    pub taxable_income_after_nol: Decimal,
    /// NOL remaining to carry to next year.
    pub nol_carryforward_to_next_year: Decimal,
    /// True if the loss is farming/insurance eligible for the 2-year
    /// carryback under §172(b)(1)(B).
    pub farming_2yr_carryback_available: bool,
    /// True if CARES Act 5-year carryback applies (2018-2020 NOLs only).
    pub cares_act_5yr_carryback_available: bool,
    pub note: String,
}

pub fn compute(input: &Section172Input) -> Section172Result {
    // Step 1: Classify regime by tax year.
    let regime = classify_regime(input.tax_year);

    // Step 2: Compute the 80% limit dollars.
    let eighty_percent_limit = match regime {
        Section172Regime::PermanentTcja => {
            input.current_year_taxable_income_before_nol * Decimal::from(80) / Decimal::from(100)
        }
        Section172Regime::CaresAct | Section172Regime::Pre2018Legacy => {
            // No 80% limit under these regimes; effectively 100% of taxable income.
            input.current_year_taxable_income_before_nol
        }
    };

    // Step 3: Compute available NOL.
    // Current-year NOL doesn't reduce current-year TI in a loss year
    // (there's no TI in a loss year). It enters the carryforward.
    // Prior-year carryforward IS available to absorb against current TI.
    let total_nol_available = input.prior_year_nol_carryforward + input.current_year_nol;

    // Step 4: NOL deduction = min(available_for_absorption, limit).
    // For absorption purposes: only the prior carryforward absorbs
    // against current-year TI; current-year NOL flows directly to
    // carryforward.
    let available_for_absorption = input.prior_year_nol_carryforward;
    let nol_deduction = available_for_absorption.min(eighty_percent_limit);
    let limit_binds = available_for_absorption > eighty_percent_limit
        && matches!(regime, Section172Regime::PermanentTcja);

    // Step 5: Taxable income after NOL.
    let taxable_income_after =
        (input.current_year_taxable_income_before_nol - nol_deduction).max(Decimal::ZERO);

    // Step 6: Carryforward = prior - absorbed + current year loss.
    let carryforward = total_nol_available - nol_deduction;

    let farming_carryback = input.farming_or_insurance_loss;
    let cares_carryback = matches!(regime, Section172Regime::CaresAct);

    let note = build_note(input, regime, nol_deduction, carryforward, limit_binds);

    Section172Result {
        regime,
        total_nol_available,
        eighty_percent_limit_dollars: eighty_percent_limit,
        eighty_percent_limit_binds: limit_binds,
        nol_deduction_allowed: nol_deduction,
        taxable_income_after_nol: taxable_income_after,
        nol_carryforward_to_next_year: carryforward,
        farming_2yr_carryback_available: farming_carryback,
        cares_act_5yr_carryback_available: cares_carryback,
        note,
    }
}

fn classify_regime(year: i32) -> Section172Regime {
    if year < 2018 {
        Section172Regime::Pre2018Legacy
    } else if year <= 2020 {
        Section172Regime::CaresAct
    } else {
        Section172Regime::PermanentTcja
    }
}

fn build_note(
    input: &Section172Input,
    regime: Section172Regime,
    nol_deduction: Decimal,
    carryforward: Decimal,
    limit_binds: bool,
) -> String {
    let regime_phrase = match regime {
        Section172Regime::Pre2018Legacy => "pre-TCJA (2-yr carryback / 20-yr carryforward / no 80% limit)",
        Section172Regime::CaresAct => "CARES Act (5-yr carryback / 100% offset / 2018-2020 only)",
        Section172Regime::PermanentTcja => "permanent TCJA post-2020 (no carryback, indefinite carryforward, 80% limit)",
    };
    let farming_phrase = if input.farming_or_insurance_loss {
        " + §172(b)(1)(B) farming/insurance 2-year carryback available"
    } else {
        ""
    };
    let limit_phrase = if limit_binds {
        " — 80% limit BINDS (excess carries forward indefinitely)"
    } else {
        ""
    };
    format!(
        "{} year {}{}: NOL deduction ${} allowed; ${} carries forward{}",
        regime_phrase,
        input.tax_year,
        farming_phrase,
        nol_deduction.round_dp(2),
        carryforward.round_dp(2),
        limit_phrase
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section172Input {
        Section172Input {
            current_year_nol: Decimal::ZERO,
            current_year_taxable_income_before_nol: dec!(100_000),
            prior_year_nol_carryforward: dec!(50_000),
            tax_year: 2026,
            farming_or_insurance_loss: false,
        }
    }

    #[test]
    fn permanent_tcja_post_2020_classified() {
        let r = compute(&base());
        assert_eq!(r.regime, Section172Regime::PermanentTcja);
    }

    #[test]
    fn cares_act_2018_2020_classified() {
        let mut i = base();
        i.tax_year = 2020;
        let r = compute(&i);
        assert_eq!(r.regime, Section172Regime::CaresAct);
        assert!(r.cares_act_5yr_carryback_available);
    }

    #[test]
    fn pre_2018_legacy_classified() {
        let mut i = base();
        i.tax_year = 2017;
        let r = compute(&i);
        assert_eq!(r.regime, Section172Regime::Pre2018Legacy);
    }

    #[test]
    fn tcja_80_percent_limit_does_not_bind_when_nol_small() {
        // $50k NOL ≤ 80% × $100k = $80k → full $50k deducted.
        let r = compute(&base());
        assert_eq!(r.eighty_percent_limit_dollars, dec!(80_000));
        assert!(!r.eighty_percent_limit_binds);
        assert_eq!(r.nol_deduction_allowed, dec!(50_000));
        assert_eq!(r.taxable_income_after_nol, dec!(50_000));
        assert_eq!(r.nol_carryforward_to_next_year, Decimal::ZERO);
    }

    #[test]
    fn tcja_80_percent_limit_binds_when_nol_large() {
        // $200k NOL > $80k limit → only $80k absorbed; $120k carries.
        let mut i = base();
        i.prior_year_nol_carryforward = dec!(200_000);
        let r = compute(&i);
        assert!(r.eighty_percent_limit_binds);
        assert_eq!(r.nol_deduction_allowed, dec!(80_000));
        assert_eq!(r.taxable_income_after_nol, dec!(20_000));
        assert_eq!(r.nol_carryforward_to_next_year, dec!(120_000));
    }

    #[test]
    fn cares_act_100_percent_offset_no_limit_binding() {
        // 2020 + $200k NOL: under CARES, no 80% limit. Full $100k TI
        // absorbed; $100k carries.
        let mut i = base();
        i.tax_year = 2020;
        i.prior_year_nol_carryforward = dec!(200_000);
        let r = compute(&i);
        assert!(!r.eighty_percent_limit_binds);
        assert_eq!(r.nol_deduction_allowed, dec!(100_000));
        assert_eq!(r.taxable_income_after_nol, Decimal::ZERO);
        assert_eq!(r.nol_carryforward_to_next_year, dec!(100_000));
    }

    #[test]
    fn pre_2018_no_80_percent_limit_full_absorption() {
        // 2017: no 80% limit. Full TI absorbed.
        let mut i = base();
        i.tax_year = 2017;
        i.prior_year_nol_carryforward = dec!(150_000);
        let r = compute(&i);
        assert!(!r.eighty_percent_limit_binds);
        assert_eq!(r.nol_deduction_allowed, dec!(100_000));
        assert_eq!(r.nol_carryforward_to_next_year, dec!(50_000));
    }

    #[test]
    fn current_year_nol_flows_to_carryforward_not_absorption() {
        // Current year loss $50k + prior $0 + TI $0 (loss year).
        // Cannot absorb against own loss-year TI; $50k carries forward.
        let mut i = base();
        i.current_year_nol = dec!(50_000);
        i.current_year_taxable_income_before_nol = Decimal::ZERO;
        i.prior_year_nol_carryforward = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.nol_deduction_allowed, Decimal::ZERO);
        assert_eq!(r.nol_carryforward_to_next_year, dec!(50_000));
    }

    #[test]
    fn current_year_loss_combined_with_prior_carryforward() {
        // Current loss $30k + prior $50k = $80k total available.
        // Current year TI $100k → 80% limit $80k. Prior carryforward
        // absorbs against current TI: min($50k, $80k) = $50k allowed.
        // Carryforward = $50k - $50k + $30k = $30k.
        let mut i = base();
        i.current_year_nol = dec!(30_000);
        i.prior_year_nol_carryforward = dec!(50_000);
        let r = compute(&i);
        assert_eq!(r.total_nol_available, dec!(80_000));
        assert_eq!(r.nol_deduction_allowed, dec!(50_000));
        assert_eq!(r.nol_carryforward_to_next_year, dec!(30_000));
    }

    #[test]
    fn farming_loss_flag_surfaces_2_year_carryback() {
        let mut i = base();
        i.farming_or_insurance_loss = true;
        let r = compute(&i);
        assert!(r.farming_2yr_carryback_available);
        assert!(r.note.contains("farming/insurance 2-year carryback"));
    }

    #[test]
    fn non_farming_loss_no_carryback_flag_post_2020() {
        // Post-2020 default: no carryback unless farming.
        let r = compute(&base());
        assert!(!r.farming_2yr_carryback_available);
        assert!(!r.cares_act_5yr_carryback_available);
    }

    #[test]
    fn cares_year_flags_5yr_carryback_regardless_of_farming() {
        // 2019 NOL gets 5-year carryback under CARES regardless of
        // farming status.
        let mut i = base();
        i.tax_year = 2019;
        let r = compute(&i);
        assert!(r.cares_act_5yr_carryback_available);
    }

    #[test]
    fn tcja_2021_boundary_first_post_cares_year() {
        // 2021 = first year of permanent TCJA regime (no carryback, 80%
        // limit). CARES sunset after 2020.
        let mut i = base();
        i.tax_year = 2021;
        let r = compute(&i);
        assert_eq!(r.regime, Section172Regime::PermanentTcja);
        assert!(!r.cares_act_5yr_carryback_available);
    }

    #[test]
    fn tcja_2018_boundary_first_cares_year() {
        // 2018 = first year of CARES Act treatment for NOLs.
        let mut i = base();
        i.tax_year = 2018;
        let r = compute(&i);
        assert_eq!(r.regime, Section172Regime::CaresAct);
    }

    #[test]
    fn tcja_2017_boundary_last_pre_tcja_year() {
        // 2017 = last year of pre-TCJA legacy regime.
        let mut i = base();
        i.tax_year = 2017;
        let r = compute(&i);
        assert_eq!(r.regime, Section172Regime::Pre2018Legacy);
    }

    #[test]
    fn zero_taxable_income_no_absorption() {
        // Profit year but $0 TI → no NOL absorption; full carryforward
        // continues.
        let mut i = base();
        i.current_year_taxable_income_before_nol = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.nol_deduction_allowed, Decimal::ZERO);
        assert_eq!(r.nol_carryforward_to_next_year, dec!(50_000));
    }

    #[test]
    fn zero_nol_no_op() {
        let mut i = base();
        i.prior_year_nol_carryforward = Decimal::ZERO;
        i.current_year_nol = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.nol_deduction_allowed, Decimal::ZERO);
        assert_eq!(r.taxable_income_after_nol, dec!(100_000));
        assert_eq!(r.nol_carryforward_to_next_year, Decimal::ZERO);
    }

    #[test]
    fn taxable_income_after_nol_never_negative() {
        // Defensive: even pathological inputs shouldn't produce negative
        // TI. NOL deduction is bounded by available_for_absorption AND
        // the 80% limit, both ≤ TI.
        let mut i = base();
        i.prior_year_nol_carryforward = dec!(10_000_000);
        let r = compute(&i);
        assert!(r.taxable_income_after_nol >= Decimal::ZERO);
    }

    #[test]
    fn very_large_nol_no_precision_loss() {
        // $1B taxable income with $5B NOL carryforward. Post-2020 limit
        // = $800M; NOL deduction = $800M; carryforward = $4.2B.
        let mut i = base();
        i.current_year_taxable_income_before_nol = dec!(1_000_000_000);
        i.prior_year_nol_carryforward = dec!(5_000_000_000);
        let r = compute(&i);
        assert_eq!(r.eighty_percent_limit_dollars, dec!(800_000_000));
        assert_eq!(r.nol_deduction_allowed, dec!(800_000_000));
        assert_eq!(r.nol_carryforward_to_next_year, dec!(4_200_000_000));
    }

    #[test]
    fn eighty_percent_limit_boundary_exact_no_binding() {
        // NOL exactly at 80% limit. Deduction = NOL; limit doesn't bind
        // (NOL <= limit, not >).
        let mut i = base();
        i.prior_year_nol_carryforward = dec!(80_000);
        let r = compute(&i);
        assert!(!r.eighty_percent_limit_binds);
        assert_eq!(r.nol_deduction_allowed, dec!(80_000));
        assert_eq!(r.taxable_income_after_nol, dec!(20_000));
    }

    #[test]
    fn eighty_percent_limit_boundary_one_over_binds() {
        // $80,001 > $80,000 → binds.
        let mut i = base();
        i.prior_year_nol_carryforward = dec!(80_001);
        let r = compute(&i);
        assert!(r.eighty_percent_limit_binds);
        assert_eq!(r.nol_deduction_allowed, dec!(80_000));
        assert_eq!(r.nol_carryforward_to_next_year, dec!(1));
    }

    #[test]
    fn note_describes_regime_and_carryforward_amount() {
        let r = compute(&base());
        assert!(r.note.contains("permanent TCJA"));
        assert!(r.note.contains("80%"));
    }

    #[test]
    fn note_describes_limit_binding_when_applicable() {
        let mut i = base();
        i.prior_year_nol_carryforward = dec!(200_000);
        let r = compute(&i);
        assert!(r.note.contains("80% limit BINDS"));
    }

    #[test]
    fn note_for_cares_act_describes_5yr_carryback() {
        let mut i = base();
        i.tax_year = 2020;
        let r = compute(&i);
        assert!(r.note.contains("CARES"));
        assert!(r.note.contains("5-yr"));
    }
}
