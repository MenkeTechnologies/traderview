//! IRC § 199A — Qualified Business Income (QBI) deduction.
//! Made PERMANENT by One Big Beautiful Bill Act of 2025
//! (Pub. L. 119-21). 20% deduction of QBI for owners of
//! pass-through entities (sole proprietorships + S-corps +
//! partnerships + LLCs taxed as such). Trader-critical for:
//! - Trader-traders with Section 475(f) MTM election
//!   (trading is generally an SSTB EXCEPT § 475(f) electing
//!   traders may qualify under § 199A as non-SSTB).
//! - Trader-landlords with rental real estate qualifying as
//!   trade or business under Rev. Proc. 2019-38 safe harbor.
//!
//! Distinct from siblings `section_469` (passive activity
//! rules), `section_1411` (NIIT — investment income tax),
//! and `section_475c2` (§ 475(f)(1) MTM trader election).
//!
//! **§ 199A(a) basic 20% deduction** — deduction = LESSER of:
//! 1. 20% × QBI (combined QBI amount), OR
//! 2. 20% × (Taxable Income − Net Capital Gain).
//!
//! **§ 199A(b)(2) W-2 wage / UBIA phase-in limitation** —
//! applies when TI exceeds threshold. Limits 20% × QBI to
//! GREATER of:
//! 1. 50% × W-2 wages, OR
//! 2. 25% × W-2 wages + 2.5% × Unadjusted Basis Immediately
//!    after Acquisition (UBIA) of qualified property.
//!
//! **§ 199A(e)(2) 2026 thresholds** (indexed):
//! - Single / Head of Household: **$201,750** phase-in
//!   begin; **$276,750** phase-out complete (phase-in window
//!   $75,000 expanded by OBBBA from $50,000).
//! - Married Filing Jointly / Qualifying Surviving Spouse:
//!   **$403,500** phase-in begin; **$553,500** phase-out
//!   complete (phase-in window $150,000 expanded by OBBBA
//!   from $100,000).
//!
//! **§ 199A(d)(2) Specified Service Trade or Business
//! (SSTB)** — health, law, accounting, actuarial science,
//! performing arts, consulting, athletics, financial
//! services, brokerage services, investing/investment
//! management/trading (excluded from full QBI deduction
//! above threshold). SSTB deduction phases out completely
//! above upper threshold.
//!
//! **OBBBA 2025 changes (Pub. L. 119-21)**:
//! - Made § 199A PERMANENT (expiration date removed from
//!   § 199A(i)).
//! - Expanded phase-in window ($50K → $75K single; $100K →
//!   $150K joint).
//! - **§ 199A(b) NEW minimum deduction of $400** if QBI ≥
//!   $1,000 AND taxpayer materially participates in trade
//!   or business.
//!
//! **Rev. Proc. 2019-38 rental real estate safe harbor** —
//! rental real estate enterprise treated as trade or business
//! for § 199A purposes if criteria met (250+ hours / year of
//! rental services performed + separate books/records +
//! contemporaneous records).
//!
//! Citations: 26 USC § 199A(a)-(i); Pub. L. 119-21 (OBBBA
//! 2025; § 199A permanence + $400 minimum + expanded phase-
//! in); Rev. Proc. 2019-38 (rental real estate safe harbor);
//! IRS Form 8995 / 8995-A; § 475(f) trader exception.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    HeadOfHousehold,
    MarriedFilingJointly,
    QualifyingSurvivingSpouse,
    MarriedFilingSeparately,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section199AInput {
    pub filing_status: FilingStatus,
    /// Taxable income before QBI deduction in cents.
    pub taxable_income_cents: i64,
    /// Net capital gain in cents (subtracted for § 199A(a)(1)
    /// (B) overall limit calculation).
    pub net_capital_gain_cents: i64,
    /// Combined Qualified Business Income amount in cents.
    pub qbi_cents: i64,
    /// W-2 wages paid by qualified trade or business in
    /// cents (for § 199A(b)(2) phase-in limit).
    pub w2_wages_cents: i64,
    /// Unadjusted Basis Immediately after Acquisition (UBIA)
    /// of qualified property in cents.
    pub ubia_qualified_property_cents: i64,
    /// Whether business is a Specified Service Trade or
    /// Business under § 199A(d)(2) (health + law + financial
    /// services + investment management + trading; subject
    /// to phase-out above threshold).
    pub is_sstb: bool,
    /// Whether taxpayer materially participates in trade or
    /// business (for OBBBA $400 minimum deduction).
    pub materially_participates: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section199AResult {
    pub qbi_deduction_cents: i64,
    pub phase_in_threshold_cents: i64,
    pub phase_out_complete_cents: i64,
    pub phase_in_window_cents: i64,
    pub below_phase_in_threshold: bool,
    pub above_phase_out_complete: bool,
    pub sstb_phaseout_engaged: bool,
    pub w2_ubia_limit_engaged: bool,
    pub obbba_400_minimum_applied: bool,
    pub overall_20_percent_cap_engaged: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section199AInput) -> Section199AResult {
    let (phase_in_threshold, phase_out_complete, phase_in_window): (i64, i64, i64) = match input
        .filing_status
    {
        FilingStatus::Single | FilingStatus::HeadOfHousehold => {
            (20_175_000_000, 27_675_000_000, 7_500_000_000)
        }
        FilingStatus::MarriedFilingJointly | FilingStatus::QualifyingSurvivingSpouse => {
            (40_350_000_000, 55_350_000_000, 15_000_000_000)
        }
        FilingStatus::MarriedFilingSeparately => (20_175_000_000, 27_675_000_000, 7_500_000_000),
    };

    let taxable_income = input.taxable_income_cents.max(0);
    let net_capital_gain = input.net_capital_gain_cents.max(0).min(taxable_income);
    let qbi = input.qbi_cents.max(0);
    let w2_wages = input.w2_wages_cents.max(0);
    let ubia = input.ubia_qualified_property_cents.max(0);

    let twenty_percent_qbi = qbi.saturating_mul(20) / 100;
    let twenty_percent_overall = taxable_income
        .saturating_sub(net_capital_gain)
        .saturating_mul(20)
        / 100;

    let below_threshold = taxable_income <= phase_in_threshold;
    let above_phase_out = taxable_income >= phase_out_complete;

    let sstb_phaseout = input.is_sstb && !below_threshold;
    let w2_ubia_phaseout = !below_threshold;

    let mut qbi_limited = twenty_percent_qbi;

    if w2_ubia_phaseout {
        let w2_only_limit = w2_wages.saturating_mul(50) / 100;
        let w2_plus_ubia_limit =
            (w2_wages.saturating_mul(25) / 100).saturating_add(ubia.saturating_mul(25) / 1000);
        let w2_limit = w2_only_limit.max(w2_plus_ubia_limit);

        if above_phase_out {
            qbi_limited = qbi_limited.min(w2_limit);
        } else {
            let excess_over_threshold = taxable_income.saturating_sub(phase_in_threshold);
            let phase_in_pct = if phase_in_window > 0 {
                excess_over_threshold.saturating_mul(10000) / phase_in_window
            } else {
                10000
            };
            let phase_in_pct = phase_in_pct.min(10000);
            let limited_difference = twenty_percent_qbi.saturating_sub(w2_limit);
            let reduction = limited_difference.saturating_mul(phase_in_pct) / 10000;
            qbi_limited = qbi_limited.saturating_sub(reduction);
        }
    }

    if sstb_phaseout {
        if above_phase_out {
            qbi_limited = 0;
        } else {
            let excess_over_threshold = taxable_income.saturating_sub(phase_in_threshold);
            let phase_in_pct = if phase_in_window > 0 {
                excess_over_threshold.saturating_mul(10000) / phase_in_window
            } else {
                10000
            };
            let phase_in_pct = phase_in_pct.min(10000);
            let reduction = qbi_limited.saturating_mul(phase_in_pct) / 10000;
            qbi_limited = qbi_limited.saturating_sub(reduction);
        }
    }

    let mut deduction = qbi_limited.min(twenty_percent_overall);
    let overall_cap_engaged = twenty_percent_qbi > twenty_percent_overall;

    let obbba_min_applies =
        qbi >= 100_000 && input.materially_participates && !above_phase_out && taxable_income > 0;
    if obbba_min_applies {
        let minimum = 40_000_i64;
        if deduction < minimum {
            deduction = minimum;
        }
    }

    let notes: Vec<String> = vec![
        "26 USC § 199A(a) — QBI deduction = LESSER of (1) 20% × QBI (combined QBI amount) or (2) 20% × (Taxable Income − Net Capital Gain)"
            .to_string(),
        "26 USC § 199A(b)(2) W-2 wage / UBIA phase-in limitation — applies when TI exceeds threshold; limits 20% × QBI to GREATER of (a) 50% × W-2 wages or (b) 25% × W-2 wages + 2.5% × UBIA"
            .to_string(),
        "26 USC § 199A(d)(2) SSTB — Specified Service Trade or Business (health + law + accounting + actuarial + performing arts + consulting + athletics + financial / brokerage / investment management / trading); phases out completely above upper threshold"
            .to_string(),
        "OBBBA 2025 (Pub. L. 119-21) — made § 199A PERMANENT (expiration removed from § 199A(i)); expanded phase-in window ($50K → $75K single; $100K → $150K joint); NEW § 199A(b) $400 minimum deduction if QBI ≥ $1,000 AND material participation"
            .to_string(),
        "Rev. Proc. 2019-38 — rental real estate enterprise safe harbor: treated as trade or business for § 199A if 250+ hours/year of rental services + separate books/records + contemporaneous records"
            .to_string(),
    ];

    Section199AResult {
        qbi_deduction_cents: deduction,
        phase_in_threshold_cents: phase_in_threshold,
        phase_out_complete_cents: phase_out_complete,
        phase_in_window_cents: phase_in_window,
        below_phase_in_threshold: below_threshold,
        above_phase_out_complete: above_phase_out,
        sstb_phaseout_engaged: sstb_phaseout,
        w2_ubia_limit_engaged: w2_ubia_phaseout,
        obbba_400_minimum_applied: obbba_min_applies && deduction == 40_000,
        overall_20_percent_cap_engaged: overall_cap_engaged,
        citation: "26 USC § 199A(a)-(i); Pub. L. 119-21 (OBBBA 2025); Rev. Proc. 2019-38; IRS Form 8995 / 8995-A; § 475(f)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single_below_threshold() -> Section199AInput {
        Section199AInput {
            filing_status: FilingStatus::Single,
            taxable_income_cents: 15_000_000_000,
            net_capital_gain_cents: 0,
            qbi_cents: 10_000_000_000,
            w2_wages_cents: 0,
            ubia_qualified_property_cents: 0,
            is_sstb: false,
            materially_participates: true,
        }
    }

    fn mfj_below_threshold() -> Section199AInput {
        Section199AInput {
            filing_status: FilingStatus::MarriedFilingJointly,
            taxable_income_cents: 30_000_000_000,
            net_capital_gain_cents: 0,
            qbi_cents: 20_000_000_000,
            w2_wages_cents: 0,
            ubia_qualified_property_cents: 0,
            is_sstb: false,
            materially_participates: true,
        }
    }

    #[test]
    fn single_below_threshold_full_20_percent() {
        let r = check(&single_below_threshold());
        assert_eq!(r.qbi_deduction_cents, 2_000_000_000);
        assert!(r.below_phase_in_threshold);
        assert!(!r.above_phase_out_complete);
        assert!(!r.sstb_phaseout_engaged);
        assert!(!r.w2_ubia_limit_engaged);
    }

    #[test]
    fn single_at_201750_phase_in_threshold_boundary() {
        let mut i = single_below_threshold();
        i.taxable_income_cents = 20_175_000_000;
        let r = check(&i);
        assert!(r.below_phase_in_threshold);
        assert!(!r.w2_ubia_limit_engaged);
    }

    #[test]
    fn single_just_over_phase_in_threshold_engages_w2_limit() {
        let mut i = single_below_threshold();
        i.taxable_income_cents = 20_175_000_001;
        let r = check(&i);
        assert!(!r.below_phase_in_threshold);
        assert!(r.w2_ubia_limit_engaged);
    }

    #[test]
    fn single_at_276750_phase_out_complete_boundary() {
        let mut i = single_below_threshold();
        i.taxable_income_cents = 27_675_000_000;
        let r = check(&i);
        assert!(r.above_phase_out_complete);
    }

    #[test]
    fn mfj_at_403500_phase_in_threshold_boundary() {
        let mut i = mfj_below_threshold();
        i.taxable_income_cents = 40_350_000_000;
        let r = check(&i);
        assert!(r.below_phase_in_threshold);
    }

    #[test]
    fn mfj_at_553500_phase_out_complete_boundary() {
        let mut i = mfj_below_threshold();
        i.taxable_income_cents = 55_350_000_000;
        let r = check(&i);
        assert!(r.above_phase_out_complete);
    }

    #[test]
    fn filing_status_threshold_truth_table() {
        for (status, exp_phase_in, exp_phase_out, exp_window) in [
            (
                FilingStatus::Single,
                20_175_000_000_i64,
                27_675_000_000_i64,
                7_500_000_000_i64,
            ),
            (
                FilingStatus::HeadOfHousehold,
                20_175_000_000,
                27_675_000_000,
                7_500_000_000,
            ),
            (
                FilingStatus::MarriedFilingJointly,
                40_350_000_000,
                55_350_000_000,
                15_000_000_000,
            ),
            (
                FilingStatus::QualifyingSurvivingSpouse,
                40_350_000_000,
                55_350_000_000,
                15_000_000_000,
            ),
            (
                FilingStatus::MarriedFilingSeparately,
                20_175_000_000,
                27_675_000_000,
                7_500_000_000,
            ),
        ] {
            let mut i = single_below_threshold();
            i.filing_status = status;
            let r = check(&i);
            assert_eq!(r.phase_in_threshold_cents, exp_phase_in);
            assert_eq!(r.phase_out_complete_cents, exp_phase_out);
            assert_eq!(r.phase_in_window_cents, exp_window);
        }
    }

    #[test]
    fn obbba_400_minimum_applies_with_material_participation_and_qbi_over_1000() {
        let mut i = single_below_threshold();
        i.qbi_cents = 100_000;
        i.taxable_income_cents = 1_000_000;
        let r = check(&i);
        assert!(r.obbba_400_minimum_applied);
        assert_eq!(r.qbi_deduction_cents, 40_000);
    }

    #[test]
    fn obbba_400_minimum_not_applied_without_material_participation() {
        let mut i = single_below_threshold();
        i.qbi_cents = 100_000;
        i.materially_participates = false;
        let r = check(&i);
        assert!(!r.obbba_400_minimum_applied);
    }

    #[test]
    fn obbba_400_minimum_not_applied_below_1000_qbi() {
        let mut i = single_below_threshold();
        i.qbi_cents = 99_999;
        i.taxable_income_cents = 1_000_000;
        let r = check(&i);
        assert!(!r.obbba_400_minimum_applied);
    }

    #[test]
    fn overall_20_percent_cap_engaged_when_qbi_exceeds_ti() {
        let mut i = single_below_threshold();
        i.taxable_income_cents = 5_000_000_000;
        i.qbi_cents = 10_000_000_000;
        let r = check(&i);
        assert!(r.overall_20_percent_cap_engaged);
        assert_eq!(r.qbi_deduction_cents, 1_000_000_000);
    }

    #[test]
    fn net_capital_gain_reduces_overall_cap() {
        let mut i = single_below_threshold();
        i.taxable_income_cents = 10_000_000_000;
        i.net_capital_gain_cents = 5_000_000_000;
        i.qbi_cents = 10_000_000_000;
        let r = check(&i);
        assert_eq!(r.qbi_deduction_cents, 1_000_000_000);
        assert!(r.overall_20_percent_cap_engaged);
    }

    #[test]
    fn sstb_above_phase_out_zero_deduction() {
        let mut i = single_below_threshold();
        i.is_sstb = true;
        i.taxable_income_cents = 30_000_000_000;
        let r = check(&i);
        assert!(r.sstb_phaseout_engaged);
        assert!(r.above_phase_out_complete);
        assert_eq!(r.qbi_deduction_cents, 0);
    }

    #[test]
    fn sstb_below_threshold_no_phaseout() {
        let mut i = single_below_threshold();
        i.is_sstb = true;
        let r = check(&i);
        assert!(!r.sstb_phaseout_engaged);
        assert_eq!(r.qbi_deduction_cents, 2_000_000_000);
    }

    #[test]
    fn non_sstb_above_phase_out_w2_limit_applies() {
        let mut i = single_below_threshold();
        i.is_sstb = false;
        i.taxable_income_cents = 30_000_000_000;
        i.w2_wages_cents = 1_000_000_000;
        let r = check(&i);
        assert!(!r.sstb_phaseout_engaged);
        assert!(r.w2_ubia_limit_engaged);
        assert!(r.above_phase_out_complete);
        assert_eq!(r.qbi_deduction_cents, 500_000_000);
    }

    #[test]
    fn non_sstb_w2_plus_ubia_alternative() {
        let mut i = single_below_threshold();
        i.taxable_income_cents = 30_000_000_000;
        i.w2_wages_cents = 0;
        i.ubia_qualified_property_cents = 11_000_000_000;
        let r = check(&i);
        assert!(r.w2_ubia_limit_engaged);
        assert_eq!(r.qbi_deduction_cents, 275_000_000);
    }

    #[test]
    fn obbba_2025_permanence_in_notes() {
        let r = check(&single_below_threshold());
        assert!(r.notes.iter().any(|n| n.contains("OBBBA 2025")
            && n.contains("PERMANENT")
            && n.contains("$400 minimum")));
    }

    #[test]
    fn notes_pin_section_199a_a_lesser_of_formula() {
        let r = check(&single_below_threshold());
        assert!(r.notes.iter().any(|n| n.contains("§ 199A(a)")
            && n.contains("LESSER")
            && n.contains("20% × QBI")
            && n.contains("Net Capital Gain")));
    }

    #[test]
    fn notes_pin_w2_ubia_limit_section_199a_b_2() {
        let r = check(&single_below_threshold());
        assert!(r.notes.iter().any(|n| n.contains("§ 199A(b)(2)")
            && n.contains("50% × W-2")
            && n.contains("25% × W-2")
            && n.contains("2.5% × UBIA")));
    }

    #[test]
    fn notes_pin_sstb_categories_section_199a_d_2() {
        let r = check(&single_below_threshold());
        assert!(r.notes.iter().any(|n| n.contains("§ 199A(d)(2)")
            && n.contains("health")
            && n.contains("law")
            && n.contains("trading")));
    }

    #[test]
    fn notes_pin_rental_real_estate_safe_harbor() {
        let r = check(&single_below_threshold());
        assert!(r.notes.iter().any(|n| n.contains("Rev. Proc. 2019-38")
            && n.contains("250+ hours")
            && n.contains("rental real estate")));
    }

    #[test]
    fn citation_pins_authorities() {
        let r = check(&single_below_threshold());
        assert!(r.citation.contains("§ 199A(a)-(i)"));
        assert!(r.citation.contains("Pub. L. 119-21"));
        assert!(r.citation.contains("Rev. Proc. 2019-38"));
        assert!(r.citation.contains("Form 8995"));
        assert!(r.citation.contains("§ 475(f)"));
    }

    #[test]
    fn defensive_negative_qbi_clamped() {
        let mut i = single_below_threshold();
        i.qbi_cents = -1_000_000_000;
        let r = check(&i);
        assert_eq!(r.qbi_deduction_cents, 0);
    }

    #[test]
    fn defensive_negative_taxable_income_clamped() {
        let mut i = single_below_threshold();
        i.taxable_income_cents = -1_000_000;
        let r = check(&i);
        assert_eq!(r.qbi_deduction_cents, 0);
    }

    #[test]
    fn defensive_negative_w2_wages_clamped() {
        let mut i = single_below_threshold();
        i.taxable_income_cents = 30_000_000_000;
        i.w2_wages_cents = -1_000_000_000;
        let r = check(&i);
        assert_eq!(r.qbi_deduction_cents, 0);
    }

    #[test]
    fn mfj_uniquely_double_phase_in_window_invariant() {
        let r_mfj = check(&mfj_below_threshold());
        let r_single = check(&single_below_threshold());
        assert_eq!(
            r_mfj.phase_in_window_cents,
            2 * r_single.phase_in_window_cents
        );
    }

    #[test]
    fn mfj_uniquely_double_phase_in_threshold_invariant() {
        let r_mfj = check(&mfj_below_threshold());
        let r_single = check(&single_below_threshold());
        assert_eq!(
            r_mfj.phase_in_threshold_cents,
            2 * r_single.phase_in_threshold_cents
        );
    }

    #[test]
    fn sstb_uniquely_phases_out_invariant() {
        let mut i_sstb = single_below_threshold();
        i_sstb.is_sstb = true;
        i_sstb.taxable_income_cents = 30_000_000_000;
        let r_sstb = check(&i_sstb);
        assert!(r_sstb.sstb_phaseout_engaged);
        assert_eq!(r_sstb.qbi_deduction_cents, 0);

        let mut i_non_sstb = single_below_threshold();
        i_non_sstb.is_sstb = false;
        i_non_sstb.taxable_income_cents = 30_000_000_000;
        i_non_sstb.w2_wages_cents = 5_000_000_000;
        let r_non_sstb = check(&i_non_sstb);
        assert!(!r_non_sstb.sstb_phaseout_engaged);
        assert!(r_non_sstb.qbi_deduction_cents > 0);
    }

    #[test]
    fn obbba_phase_in_window_uniquely_expanded() {
        let r_single = check(&single_below_threshold());
        let r_mfj = check(&mfj_below_threshold());
        assert_eq!(r_single.phase_in_window_cents, 7_500_000_000);
        assert_eq!(r_mfj.phase_in_window_cents, 15_000_000_000);
    }

    #[test]
    fn three_eight_percent_test_does_not_match_section_1411() {
        let r = check(&single_below_threshold());
        assert_eq!(r.qbi_deduction_cents, 2_000_000_000);
    }
}
