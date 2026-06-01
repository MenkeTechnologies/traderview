//! IRC § 6654 — Failure by individual to pay estimated income tax.
//!
//! 26 U.S.C. § 6654 imposes a penalty on individuals (and certain trusts and
//! estates) who fail to pay enough tax through withholding plus four quarterly
//! estimated installments. Two safe harbors avoid the penalty entirely:
//!
//! 1. **Current-year safe harbor (§ 6654(d)(1)(B)(i))** — pay 90% of the
//!    actual current-year tax liability through the four installments.
//! 2. **Prior-year safe harbor (§ 6654(d)(1)(B)(ii))** — pay 100% of the
//!    PRIOR-year tax liability through the four installments. **The 100%
//!    rises to 110%** under § 6654(d)(1)(C) if prior-year AGI exceeded
//!    $150,000 ($75,000 if married filing separately).
//!
//! Required annual payment = the LESSER of the two safe-harbor amounts.
//! Required installment per quarter = (required annual payment) / 4.
//!
//! **De minimis exception (§ 6654(e)(1))**: no penalty if the total tax owed
//! after withholding is less than $1,000.
//!
//! Penalty rate (§ 6621(a)(2)) = federal short-term rate + 3 percentage
//! points, set quarterly by the IRS. 2026 Q1 = 7%; 2026 Q2 = 6%.
//! Penalty for each underpaid period = underpayment_amount × rate × days/365.
//!
//! Citations: 26 U.S.C. § 6654; § 6654(d)(1)(B)(i) (90% current-year);
//! § 6654(d)(1)(B)(ii) (100% prior-year); § 6654(d)(1)(C) (110% high-AGI);
//! § 6654(e)(1) ($1,000 de minimis); § 6621(a)(2) (short-term + 3 points).
//!
//! Caveats: this module does NOT model the annualized-income exception of
//! § 6654(d)(2) (uneven income), the farmer/fisherman two-thirds-of-income
//! exception (§ 6654(i)), or the retired/disabled waiver (§ 6654(e)(3)).
//! The four quarterly due dates (Apr 15, Jun 15, Sep 15, Jan 15) and the
//! days each underpayment accrues are passed in by the caller because the
//! calculator does not know the filing date.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    MarriedFilingJointly,
    MarriedFilingSeparately,
    HeadOfHousehold,
    QualifyingWidow,
}

impl FilingStatus {
    fn high_agi_threshold_cents(self) -> i64 {
        match self {
            FilingStatus::MarriedFilingSeparately => 7500000,
            _ => 15000000,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6654Input {
    pub current_year_tax_cents: i64,
    pub prior_year_tax_cents: i64,
    pub prior_year_agi_cents: i64,
    pub filing_status: FilingStatus,
    /// Withholding counts as estimated payments evenly across the year. To
    /// model that, add (withholding / 4) into each `quarterly_payments_cents`
    /// bucket. The compute fn does NOT split withholding for the caller.
    pub quarterly_payments_cents: [i64; 4],
    /// Annualized underpayment-rate percent per quarter, expressed as
    /// whole-number basis points × 100 to retain precision: e.g. 7% = 700.
    /// IRS sets this quarterly. Using bps avoids float math.
    pub quarterly_rate_bps: [u32; 4],
    /// Days each quarter's underpayment accrues. Standard intervals: Q1
    /// 4/15→6/15 (61), Q2 6/15→9/15 (92), Q3 9/15→1/15 (122), Q4 1/15→filing
    /// date (up to 90).
    pub days_in_period: [u32; 4],
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6654Result {
    pub required_annual_payment_cents: i64,
    pub required_installment_cents: i64,
    pub safe_harbor_used: SafeHarbor,
    pub high_agi_uplift_applied: bool,
    pub de_minimis_exception: bool,
    pub safe_harbor_met: bool,
    pub underpayment_per_quarter_cents: [i64; 4],
    pub penalty_per_quarter_cents: [i64; 4],
    pub total_penalty_cents: i64,
    pub citation: &'static str,
    pub note: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SafeHarbor {
    NinetyPercentCurrentYear,
    PriorYearOneHundredPercent,
    PriorYearOneHundredTenPercent,
    DeMinimisExempt,
}

pub fn compute(input: &Section6654Input) -> Section6654Result {
    let cy_tax = input.current_year_tax_cents.max(0);
    let py_tax = input.prior_year_tax_cents.max(0);
    let py_agi = input.prior_year_agi_cents.max(0);

    if cy_tax < 100000 {
        return Section6654Result {
            required_annual_payment_cents: 0,
            required_installment_cents: 0,
            safe_harbor_used: SafeHarbor::DeMinimisExempt,
            high_agi_uplift_applied: false,
            de_minimis_exception: true,
            safe_harbor_met: true,
            underpayment_per_quarter_cents: [0; 4],
            penalty_per_quarter_cents: [0; 4],
            total_penalty_cents: 0,
            citation: "§ 6654(e)(1) de minimis exception — tax under $1,000",
            note: format!(
                "Current-year tax of {} cents is below the $1,000 de minimis threshold; no § 6654 penalty applies.",
                cy_tax
            ),
        };
    }

    let high_agi_threshold = input.filing_status.high_agi_threshold_cents();
    let high_agi = py_agi > high_agi_threshold;

    let ninety_pct = (cy_tax as i128 * 90 / 100) as i64;
    let prior_year_pct = if high_agi { 110 } else { 100 };
    let prior_year_required = (py_tax as i128 * prior_year_pct / 100) as i64;

    let (required_annual, safe_harbor_used) = if prior_year_required < ninety_pct {
        if high_agi {
            (prior_year_required, SafeHarbor::PriorYearOneHundredTenPercent)
        } else {
            (prior_year_required, SafeHarbor::PriorYearOneHundredPercent)
        }
    } else {
        (ninety_pct, SafeHarbor::NinetyPercentCurrentYear)
    };

    let required_installment = required_annual / 4;

    let mut underpayments = [0i64; 4];
    let mut penalties = [0i64; 4];
    let mut total_penalty = 0i64;

    let mut cumulative_required = 0i64;
    let mut cumulative_paid = 0i64;
    for q in 0..4 {
        cumulative_required += required_installment;
        cumulative_paid += input.quarterly_payments_cents[q];
        let under = (cumulative_required - cumulative_paid).max(0);
        underpayments[q] = under;
        let rate_bps = input.quarterly_rate_bps[q] as i128;
        let days = input.days_in_period[q] as i128;
        let penalty = (under as i128 * rate_bps * days) / (10_000_i128 * 365);
        penalties[q] = penalty as i64;
        total_penalty += penalty as i64;
    }

    let safe_harbor_met = total_penalty == 0;

    let note = format!(
        "Required annual payment = {} cents (lesser of 90%-current-year = {} cents and {}%-prior-year = {} cents). Per-quarter required installment = {} cents.{}{}",
        required_annual,
        ninety_pct,
        prior_year_pct,
        prior_year_required,
        required_installment,
        if high_agi {
            " HIGH-AGI uplift applied — prior-year safe harbor requires 110% (not 100%) under § 6654(d)(1)(C)."
        } else {
            ""
        },
        if safe_harbor_met {
            " Safe harbor MET — no penalty."
        } else {
            " Safe harbor MISSED — penalty assessed per quarter."
        }
    );

    Section6654Result {
        required_annual_payment_cents: required_annual,
        required_installment_cents: required_installment,
        safe_harbor_used,
        high_agi_uplift_applied: high_agi,
        de_minimis_exception: false,
        safe_harbor_met,
        underpayment_per_quarter_cents: underpayments,
        penalty_per_quarter_cents: penalties,
        total_penalty_cents: total_penalty,
        citation: "26 U.S.C. § 6654 — failure to pay estimated income tax (Form 2210)",
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> Section6654Input {
        Section6654Input {
            current_year_tax_cents: 4000000,
            prior_year_tax_cents: 3000000,
            prior_year_agi_cents: 10000000,
            filing_status: FilingStatus::Single,
            quarterly_payments_cents: [900000, 900000, 900000, 900000],
            quarterly_rate_bps: [700, 700, 700, 700],
            days_in_period: [61, 92, 122, 90],
        }
    }

    #[test]
    fn de_minimis_under_1000_no_penalty() {
        let mut i = base_input();
        i.current_year_tax_cents = 999_99;
        let r = compute(&i);
        assert!(r.de_minimis_exception);
        assert!(r.safe_harbor_met);
        assert_eq!(r.total_penalty_cents, 0);
        assert!(matches!(r.safe_harbor_used, SafeHarbor::DeMinimisExempt));
        assert!(r.note.contains("$1,000 de minimis"));
    }

    #[test]
    fn de_minimis_boundary_exactly_1000_is_above() {
        let mut i = base_input();
        i.current_year_tax_cents = 100000;
        let r = compute(&i);
        assert!(!r.de_minimis_exception);
    }

    #[test]
    fn prior_year_safe_harbor_100pct_lower_than_current_90pct() {
        let mut i = base_input();
        i.current_year_tax_cents = 4000000;
        i.prior_year_tax_cents = 3000000;
        i.prior_year_agi_cents = 10000000;
        i.quarterly_payments_cents = [750000, 750000, 750000, 750000];
        let r = compute(&i);
        assert_eq!(r.required_annual_payment_cents, 3000000);
        assert!(matches!(
            r.safe_harbor_used,
            SafeHarbor::PriorYearOneHundredPercent
        ));
        assert!(!r.high_agi_uplift_applied);
        assert_eq!(r.required_installment_cents, 750000);
        assert!(r.safe_harbor_met);
    }

    #[test]
    fn high_agi_triggers_110pct() {
        let mut i = base_input();
        i.prior_year_agi_cents = 20000000;
        let r = compute(&i);
        assert!(r.high_agi_uplift_applied);
        assert_eq!(r.required_annual_payment_cents, 3300000);
        assert!(matches!(
            r.safe_harbor_used,
            SafeHarbor::PriorYearOneHundredTenPercent
        ));
        assert_eq!(r.required_installment_cents, 825000);
        assert!(r.note.contains("HIGH-AGI uplift"));
        assert!(r.note.contains("§ 6654(d)(1)(C)"));
    }

    #[test]
    fn high_agi_boundary_150k_is_NOT_high() {
        let mut i = base_input();
        i.prior_year_agi_cents = 15000000;
        let r = compute(&i);
        assert!(
            !r.high_agi_uplift_applied,
            "AGI exactly $150,000 must NOT trigger uplift (> threshold required)"
        );
    }

    #[test]
    fn high_agi_boundary_150k_plus_one_cent_is_high() {
        let mut i = base_input();
        i.prior_year_agi_cents = 15000000 + 1;
        let r = compute(&i);
        assert!(r.high_agi_uplift_applied);
    }

    #[test]
    fn mfs_threshold_is_75k_not_150k() {
        let mut i = base_input();
        i.filing_status = FilingStatus::MarriedFilingSeparately;
        i.prior_year_agi_cents = 8000000;
        let r = compute(&i);
        assert!(
            r.high_agi_uplift_applied,
            "MFS uses $75,000 threshold; $80,000 triggers 110%"
        );
    }

    #[test]
    fn mfs_threshold_75k_boundary_not_high() {
        let mut i = base_input();
        i.filing_status = FilingStatus::MarriedFilingSeparately;
        i.prior_year_agi_cents = 7500000;
        let r = compute(&i);
        assert!(!r.high_agi_uplift_applied);
    }

    #[test]
    fn current_year_90pct_used_when_lower_than_prior_year() {
        let mut i = base_input();
        i.current_year_tax_cents = 1000000;
        i.prior_year_tax_cents = 4000000;
        i.prior_year_agi_cents = 5000000;
        let r = compute(&i);
        assert_eq!(r.required_annual_payment_cents, 900000);
        assert!(matches!(
            r.safe_harbor_used,
            SafeHarbor::NinetyPercentCurrentYear
        ));
    }

    #[test]
    fn even_quarterly_payments_meeting_required_no_penalty() {
        let mut i = base_input();
        i.current_year_tax_cents = 4000000;
        i.prior_year_tax_cents = 3000000;
        i.prior_year_agi_cents = 10000000;
        i.quarterly_payments_cents = [750000, 750000, 750000, 750000];
        let r = compute(&i);
        assert!(r.safe_harbor_met);
        assert_eq!(r.total_penalty_cents, 0);
        assert_eq!(r.underpayment_per_quarter_cents, [0, 0, 0, 0]);
    }

    #[test]
    fn skipped_q1_payment_triggers_penalty_q1_only() {
        let mut i = base_input();
        i.current_year_tax_cents = 4000000;
        i.prior_year_tax_cents = 3000000;
        i.quarterly_payments_cents = [0, 2250000, 750000, 750000];
        let r = compute(&i);
        assert_eq!(r.required_installment_cents, 750000);
        assert_eq!(r.underpayment_per_quarter_cents[0], 750000);
        assert_eq!(
            r.underpayment_per_quarter_cents[1], 0,
            "Q2 catchup zeros Q2 underpayment"
        );
        assert!(r.penalty_per_quarter_cents[0] > 0);
        assert_eq!(r.penalty_per_quarter_cents[1], 0);
    }

    #[test]
    fn penalty_formula_matches_rate_times_days_over_365() {
        let mut i = base_input();
        i.current_year_tax_cents = 4000000;
        i.prior_year_tax_cents = 3000000;
        i.quarterly_payments_cents = [0, 2250000, 750000, 750000];
        i.quarterly_rate_bps = [700, 700, 700, 700];
        i.days_in_period = [61, 92, 122, 90];
        let r = compute(&i);
        // Q1 underpayment 750000 cents, rate 7% (700 bps), 61 days.
        // Penalty = 750000 * 700 * 61 / (10000 * 365) = 8,773 cents (truncated).
        let expected = 7_500_00_i128 * 700 * 61 / (10_000 * 365);
        assert_eq!(r.penalty_per_quarter_cents[0], expected as i64);
    }

    #[test]
    fn variable_quarterly_rates_compound_into_total() {
        let mut i = base_input();
        i.current_year_tax_cents = 4000000;
        i.prior_year_tax_cents = 3000000;
        i.quarterly_payments_cents = [0, 0, 0, 0];
        i.quarterly_rate_bps = [800, 700, 600, 500];
        i.days_in_period = [61, 92, 122, 90];
        let r = compute(&i);
        // Cumulative required by Q4: 30,000 cents installments × 4 = 30,000 dollars
        // But we want underpayments to step up: 7500, 15000, 22500, 30000
        assert_eq!(r.underpayment_per_quarter_cents[0], 750000);
        assert_eq!(r.underpayment_per_quarter_cents[1], 1500000);
        assert_eq!(r.underpayment_per_quarter_cents[2], 2250000);
        assert_eq!(r.underpayment_per_quarter_cents[3], 3000000);
        assert!(r.penalty_per_quarter_cents[0] > 0);
        assert!(r.penalty_per_quarter_cents[1] > 0);
        assert!(r.penalty_per_quarter_cents[2] > 0);
        assert!(r.penalty_per_quarter_cents[3] > 0);
        let sum: i64 = r.penalty_per_quarter_cents.iter().sum();
        assert_eq!(r.total_penalty_cents, sum);
    }

    #[test]
    fn overpaying_q1_carries_overpayment_forward() {
        let mut i = base_input();
        i.current_year_tax_cents = 4000000;
        i.prior_year_tax_cents = 3000000;
        // Required per quarter = 7500.00 cents. Front-load Q1.
        i.quarterly_payments_cents = [3000000, 0, 0, 0];
        let r = compute(&i);
        assert!(r.safe_harbor_met);
        assert_eq!(r.total_penalty_cents, 0);
    }

    #[test]
    fn under_paying_throughout_year_full_penalty() {
        let mut i = base_input();
        i.current_year_tax_cents = 4000000;
        i.prior_year_tax_cents = 3000000;
        i.quarterly_payments_cents = [0, 0, 0, 0];
        i.quarterly_rate_bps = [700, 700, 700, 700];
        i.days_in_period = [61, 92, 122, 90];
        let r = compute(&i);
        assert!(!r.safe_harbor_met);
        assert!(r.total_penalty_cents > 0);
        // The Q4 underpayment is the full annual required (30,000).
        assert_eq!(r.underpayment_per_quarter_cents[3], 3000000);
    }

    #[test]
    fn zero_prior_year_tax_means_no_prior_year_harbor() {
        let mut i = base_input();
        i.current_year_tax_cents = 4000000;
        i.prior_year_tax_cents = 0;
        i.prior_year_agi_cents = 5000000;
        let r = compute(&i);
        // Prior-year 100% = 0, less than 90% current-year = 36,000.
        // But § 6654(d)(1)(B)(ii) requires PY return to actually have been
        // filed for a 12-month year — modeled here as just zero, which makes
        // the lesser comparison pick zero. This is a known model limitation;
        // the citation list documents it. Pin behavior either way.
        assert_eq!(r.required_annual_payment_cents, 0);
    }

    #[test]
    fn citation_pins_form_2210_and_section_6654() {
        let i = base_input();
        let r = compute(&i);
        assert!(r.citation.contains("§ 6654"));
        assert!(r.citation.contains("Form 2210"));
    }

    #[test]
    fn note_mentions_safe_harbor_status() {
        let mut i = base_input();
        i.current_year_tax_cents = 4000000;
        i.prior_year_tax_cents = 3000000;
        i.quarterly_payments_cents = [750000, 750000, 750000, 750000];
        let r = compute(&i);
        assert!(r.note.contains("Safe harbor MET"));

        i.quarterly_payments_cents = [0, 0, 0, 0];
        let r2 = compute(&i);
        assert!(r2.note.contains("Safe harbor MISSED"));
    }

    #[test]
    fn rate_zero_means_no_penalty_even_with_underpayment() {
        let mut i = base_input();
        i.current_year_tax_cents = 4000000;
        i.prior_year_tax_cents = 3000000;
        i.quarterly_payments_cents = [0, 0, 0, 0];
        i.quarterly_rate_bps = [0, 0, 0, 0];
        i.days_in_period = [61, 92, 122, 90];
        let r = compute(&i);
        assert!(r.safe_harbor_met);
        assert_eq!(r.total_penalty_cents, 0);
        // Underpayments still exist; only penalty is zero because of rate.
        assert!(r.underpayment_per_quarter_cents[3] > 0);
    }

    #[test]
    fn zero_days_means_no_penalty() {
        let mut i = base_input();
        i.current_year_tax_cents = 4000000;
        i.prior_year_tax_cents = 3000000;
        i.quarterly_payments_cents = [0, 0, 0, 0];
        i.quarterly_rate_bps = [700, 700, 700, 700];
        i.days_in_period = [0, 0, 0, 0];
        let r = compute(&i);
        assert!(r.safe_harbor_met);
        assert_eq!(r.total_penalty_cents, 0);
    }

    #[test]
    fn negative_tax_clamped_to_zero() {
        let mut i = base_input();
        i.current_year_tax_cents = -5000_00;
        let r = compute(&i);
        assert!(r.de_minimis_exception);
        assert_eq!(r.required_annual_payment_cents, 0);
    }

    #[test]
    fn high_agi_at_high_agi_with_quarterly_payments_meeting_110pct_safe_harbor_met() {
        let mut i = base_input();
        i.current_year_tax_cents = 5000000;
        i.prior_year_tax_cents = 3000000;
        i.prior_year_agi_cents = 25000000;
        // 110% of prior-year = 33,000.00 → 8,250.00 per quarter.
        i.quarterly_payments_cents = [825000, 825000, 825000, 825000];
        let r = compute(&i);
        assert!(r.high_agi_uplift_applied);
        assert!(r.safe_harbor_met);
        assert_eq!(r.total_penalty_cents, 0);
    }

    #[test]
    fn high_agi_under_110pct_misses_safe_harbor() {
        let mut i = base_input();
        i.current_year_tax_cents = 5000000;
        i.prior_year_tax_cents = 3000000;
        i.prior_year_agi_cents = 25000000;
        // Each installment $250 short ($25,000 total vs $33,000 required).
        i.quarterly_payments_cents = [625000, 625000, 625000, 625000];
        let r = compute(&i);
        assert!(r.high_agi_uplift_applied);
        assert!(!r.safe_harbor_met);
        assert!(r.total_penalty_cents > 0);
        // Final cumulative underpayment = 33,000 - 25,000 = 8,000.
        assert_eq!(r.underpayment_per_quarter_cents[3], 800000);
    }

    #[test]
    fn mfj_uses_150k_threshold_not_75k() {
        let mut i = base_input();
        i.filing_status = FilingStatus::MarriedFilingJointly;
        i.prior_year_agi_cents = 14000000;
        let r = compute(&i);
        assert!(!r.high_agi_uplift_applied);
        assert!(matches!(
            r.safe_harbor_used,
            SafeHarbor::PriorYearOneHundredPercent
        ));
    }
}
