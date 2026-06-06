//! IRC § 1341 — Computation of tax where taxpayer restores substantial
//! amount held under claim of right.
//!
//! 26 U.S.C. § 1341 codifies the **claim-of-right doctrine**. When a
//! taxpayer reported income in a prior year because they appeared to have
//! an unrestricted right to it, and then in a later year is required to
//! return that money (commission clawback, bonus disgorgement, Social
//! Security overpayment recovery, repayment of erroneously paid wages,
//! repayment of an embezzler/fraudster recovery, etc.), the repayment
//! creates a deduction problem: the income was taxed at the prior year's
//! rate but the deduction would only benefit the current year's rate.
//! § 1341 lets the taxpayer choose **the LESSER** of two computations.
//!
//! **Threshold (§ 1341(a)(3))**: the repayment must exceed **$3,000**.
//! At or below $3,000, § 1341 does not apply — the deduction (if any) is
//! taken under § 165 in the year of repayment, period.
//!
//! **Two methods (§ 1341(a)(4)/(5))**:
//!
//! 1. **Method A — Deduction**: take the repayment as a deduction in the
//!    current year (under § 165 or whichever other section is applicable).
//!    Pay the resulting current-year tax.
//! 2. **Method B — Credit (claim-of-right)**: compute current-year tax
//!    WITHOUT the deduction, then REDUCE that tax by the amount of tax
//!    that was originally paid on the now-repaid income in the prior year.
//!    The credit is refundable — if it exceeds current-year tax, the
//!    difference is refunded.
//!
//! § 1341(a) requires the taxpayer to use whichever method produces the
//! LESSER tax for the current year. The relief is automatic — no election
//! is required; the IRC mandates the lesser computation.
//!
//! **Trader-relevant scenarios**: clawback of erroneously credited gains,
//! return of fraudulent broker proceeds, IRS-required disgorgement, claw-
//! back of insider-trading-tainted profits (after SEC settlement), repay-
//! ment of margin-loan-funded distributions that exceeded basis.
//!
//! Citations: 26 U.S.C. § 1341; § 1341(a)(1) (claim-of-right requirement);
//! § 1341(a)(2) (deduction availability); § 1341(a)(3) ($3,000 threshold);
//! § 1341(a)(4) (deduction method); § 1341(a)(5) (credit method —
//! refundable); § 1341(b)(2) (lesser-of-the-two-tax computation).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section1341Input {
    pub repayment_amount_cents: i64,
    /// Current-year tax as it would be computed if NO § 1341 relief and NO
    /// § 165 deduction for the repayment (the "no-relief baseline").
    /// Used as the starting point for the credit method.
    pub current_year_tax_without_relief_cents: i64,
    /// Current-year tax as it would be computed if the repayment is taken
    /// as a deduction in the current year (Method A — deduction method).
    pub current_year_tax_with_deduction_cents: i64,
    /// Decrease in prior-year tax that WOULD HAVE RESULTED had the now-
    /// repaid income been excluded from prior-year gross income. This is
    /// the "amount of tax originally paid on the now-repaid income" — the
    /// credit under Method B.
    pub prior_year_tax_decrease_cents: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    DeductionMethodA,
    CreditMethodB,
    BelowThresholdNoRelief,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1341Result {
    pub threshold_met: bool,
    pub deduction_method_tax_cents: i64,
    pub credit_method_tax_cents: i64,
    /// The refund triggered if the § 1341 credit exceeds current-year tax.
    /// Zero unless the credit method yields negative current-year tax.
    pub credit_method_refund_cents: i64,
    pub better_method: Method,
    pub final_tax_cents: i64,
    /// Savings vs. taking the deduction under § 165 alone (no § 1341
    /// election available). Always non-negative; zero when deduction
    /// method already beat the credit method.
    pub savings_vs_deduction_only_cents: i64,
    pub citation: &'static str,
    pub note: String,
}

pub fn compute(input: &Section1341Input) -> Section1341Result {
    let repayment = input.repayment_amount_cents.max(0);
    let cy_without = input.current_year_tax_without_relief_cents.max(0);
    let cy_with_ded = input.current_year_tax_with_deduction_cents.max(0);
    let py_decrease = input.prior_year_tax_decrease_cents.max(0);

    // § 1341(a)(3): threshold $3,000 (300,000 cents).
    if repayment <= 300_000 {
        return Section1341Result {
            threshold_met: false,
            deduction_method_tax_cents: cy_with_ded,
            credit_method_tax_cents: cy_without,
            credit_method_refund_cents: 0,
            better_method: Method::BelowThresholdNoRelief,
            final_tax_cents: cy_with_ded,
            savings_vs_deduction_only_cents: 0,
            citation:
                "§ 1341(a)(3) — repayment must exceed $3,000; § 1341 unavailable below threshold",
            note: format!(
                "Repayment of {} cents is at or below the $3,000 threshold; § 1341 does NOT apply. The taxpayer takes the deduction (if any) under § 165 in the year of repayment.",
                repayment
            ),
        };
    }

    // Method A — deduction.
    let method_a = cy_with_ded;
    // Method B — credit. Credit may exceed current-year tax → refund.
    let raw_credit_tax = cy_without - py_decrease;
    let method_b = raw_credit_tax.max(0);
    let credit_refund = if raw_credit_tax < 0 {
        -raw_credit_tax
    } else {
        0
    };

    let (better, final_tax) = if method_a <= method_b {
        (Method::DeductionMethodA, method_a)
    } else {
        (Method::CreditMethodB, method_b)
    };

    let savings_vs_deduction_only = (cy_with_ded - final_tax).max(0);

    let note = format!(
        "Method A (deduction): current-year tax = {} cents. Method B (credit): current-year tax = max(0, {} − {}) = {} cents; refund if credit exceeds tax = {} cents. § 1341(b)(2) mandates the lesser of the two: {:?} wins with {} cents. Savings vs. § 165-deduction-only = {} cents.",
        method_a,
        cy_without,
        py_decrease,
        method_b,
        credit_refund,
        better,
        final_tax,
        savings_vs_deduction_only,
    );

    Section1341Result {
        threshold_met: true,
        deduction_method_tax_cents: method_a,
        credit_method_tax_cents: method_b,
        credit_method_refund_cents: credit_refund,
        better_method: better,
        final_tax_cents: final_tax,
        savings_vs_deduction_only_cents: savings_vs_deduction_only,
        citation:
            "26 U.S.C. § 1341 — claim-of-right relief (Method A § 1341(a)(4) deduction; Method B § 1341(a)(5) refundable credit; § 1341(b)(2) lesser-of)",
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        repayment: i64,
        cy_without: i64,
        cy_with_ded: i64,
        py_decrease: i64,
    ) -> Section1341Input {
        Section1341Input {
            repayment_amount_cents: repayment,
            current_year_tax_without_relief_cents: cy_without,
            current_year_tax_with_deduction_cents: cy_with_ded,
            prior_year_tax_decrease_cents: py_decrease,
        }
    }

    #[test]
    fn below_3000_threshold_section_1341_unavailable() {
        let r = compute(&input(2_999_99, 50_000_00, 49_000_00, 30_000_00));
        assert!(!r.threshold_met);
        assert!(matches!(r.better_method, Method::BelowThresholdNoRelief));
        assert_eq!(r.final_tax_cents, 49_000_00);
        assert!(r.citation.contains("$3,000"));
        assert!(r.note.contains("does NOT apply"));
    }

    #[test]
    fn exactly_3000_threshold_unavailable_strict_greater_than() {
        // § 1341(a)(3) requires repayment > $3,000 — exactly $3,000 is
        // below the strict-greater-than threshold.
        let r = compute(&input(3_000_00, 50_000_00, 49_000_00, 30_000_00));
        assert!(!r.threshold_met);
    }

    #[test]
    fn one_cent_above_3000_threshold_met() {
        let r = compute(&input(3_000_01, 50_000_00, 49_000_00, 30_000_00));
        assert!(r.threshold_met);
    }

    #[test]
    fn credit_method_beats_deduction_when_prior_year_rate_higher() {
        // Repayment $50K, current-year tax $80K without relief, $70K with
        // deduction ($10K saved at current rate). Prior-year tax decrease
        // if income excluded = $20K (taxed at higher prior-year rate).
        // Method A: $70K. Method B: $80K − $20K = $60K. Credit wins.
        let r = compute(&input(50_000_00, 80_000_00, 70_000_00, 20_000_00));
        assert!(r.threshold_met);
        assert!(matches!(r.better_method, Method::CreditMethodB));
        assert_eq!(r.final_tax_cents, 60_000_00);
        assert_eq!(r.deduction_method_tax_cents, 70_000_00);
        assert_eq!(r.credit_method_tax_cents, 60_000_00);
        assert_eq!(r.savings_vs_deduction_only_cents, 10_000_00);
    }

    #[test]
    fn deduction_method_beats_credit_when_current_year_rate_higher() {
        // Now the current-year rate is HIGHER (less common). Repayment
        // $50K, cy_without $80K, cy_with_ded $50K (saved $30K at higher
        // current rate). Prior-year decrease $20K. Method A: $50K. Method
        // B: $80K − $20K = $60K. Deduction wins.
        let r = compute(&input(50_000_00, 80_000_00, 50_000_00, 20_000_00));
        assert!(matches!(r.better_method, Method::DeductionMethodA));
        assert_eq!(r.final_tax_cents, 50_000_00);
        assert_eq!(r.savings_vs_deduction_only_cents, 0);
    }

    #[test]
    fn methods_tie_picks_deduction() {
        // When the two methods produce identical tax, deduction is picked
        // because it's simpler administratively (no Form 1040 line-13
        // credit adjustment required).
        let r = compute(&input(50_000_00, 80_000_00, 60_000_00, 20_000_00));
        assert!(matches!(r.better_method, Method::DeductionMethodA));
        assert_eq!(r.final_tax_cents, 60_000_00);
    }

    #[test]
    fn credit_method_creates_refund_when_credit_exceeds_tax() {
        // cy_without = $20K, py_decrease = $30K. Method B: max(0, -10K) = 0,
        // with $10K refund.
        let r = compute(&input(100_000_00, 20_000_00, 18_000_00, 30_000_00));
        assert!(matches!(r.better_method, Method::CreditMethodB));
        assert_eq!(r.credit_method_tax_cents, 0);
        assert_eq!(r.credit_method_refund_cents, 10_000_00);
        assert_eq!(r.final_tax_cents, 0);
    }

    #[test]
    fn deduction_better_no_refund_from_credit() {
        let r = compute(&input(50_000_00, 80_000_00, 40_000_00, 20_000_00));
        assert!(matches!(r.better_method, Method::DeductionMethodA));
        assert_eq!(r.credit_method_refund_cents, 0); // refund only manifests if credit method picked
    }

    #[test]
    fn no_prior_year_tax_decrease_falls_back_to_deduction() {
        // If the prior year had no tax decrease available (e.g., the prior
        // year was a loss year with no tax to recover), credit method
        // yields cy_without — never beats deduction method.
        let r = compute(&input(50_000_00, 80_000_00, 70_000_00, 0));
        assert!(matches!(r.better_method, Method::DeductionMethodA));
        assert_eq!(r.final_tax_cents, 70_000_00);
    }

    #[test]
    fn large_repayment_with_high_prior_year_rate() {
        // $1M repayment, prior-year decrease $400K. Current-year tax
        // without $300K, with deduction $200K (saved $100K). Credit:
        // $300K − $400K = −$100K → refund $100K. Credit wins.
        let r = compute(&input(1_000_000_00, 300_000_00, 200_000_00, 400_000_00));
        assert!(matches!(r.better_method, Method::CreditMethodB));
        assert_eq!(r.final_tax_cents, 0);
        assert_eq!(r.credit_method_refund_cents, 100_000_00);
        assert_eq!(r.savings_vs_deduction_only_cents, 200_000_00);
    }

    #[test]
    fn citation_pins_section_1341() {
        let r = compute(&input(50_000_00, 80_000_00, 70_000_00, 20_000_00));
        assert!(r.citation.contains("§ 1341"));
        assert!(r.citation.contains("§ 1341(a)(4)"));
        assert!(r.citation.contains("§ 1341(a)(5)"));
        assert!(r.citation.contains("§ 1341(b)(2)"));
    }

    #[test]
    fn note_describes_lesser_of_computation() {
        let r = compute(&input(50_000_00, 80_000_00, 70_000_00, 20_000_00));
        assert!(r.note.contains("§ 1341(b)(2)"));
        assert!(r.note.contains("lesser of the two"));
    }

    #[test]
    fn zero_repayment_below_threshold() {
        let r = compute(&input(0, 80_000_00, 80_000_00, 0));
        assert!(!r.threshold_met);
    }

    #[test]
    fn negative_inputs_clamped() {
        let r = compute(&input(-1, -1, -1, -1));
        assert!(!r.threshold_met);
        assert_eq!(r.final_tax_cents, 0);
    }

    #[test]
    fn savings_vs_deduction_only_nonnegative() {
        for (rep, cy_no, cy_d, py) in [
            (50_000_00, 80_000_00, 70_000_00, 20_000_00),
            (50_000_00, 80_000_00, 50_000_00, 20_000_00),
            (50_000_00, 80_000_00, 60_000_00, 20_000_00),
            (50_000_00, 80_000_00, 40_000_00, 20_000_00),
        ] {
            let r = compute(&input(rep, cy_no, cy_d, py));
            assert!(
                r.savings_vs_deduction_only_cents >= 0,
                "savings should be non-negative"
            );
        }
    }

    #[test]
    fn refund_only_when_credit_method_picked() {
        // High py_decrease but deduction method still wins because
        // cy_with_ded is very low — refund should still be tracked but
        // final_tax uses deduction-method.
        let r = compute(&input(50_000_00, 30_000_00, 10_000_00, 100_000_00));
        // Method A: $10K. Method B: max(0, 30K − 100K) = 0 + refund $70K.
        // Method B's "tax" = 0 < $10K → credit wins.
        assert!(matches!(r.better_method, Method::CreditMethodB));
        assert_eq!(r.final_tax_cents, 0);
        assert_eq!(r.credit_method_refund_cents, 70_000_00);
    }

    #[test]
    fn trader_clawback_scenario_50k() {
        // Trader received $50K commission in 2024 (taxed at 37%), repaid
        // in 2026 when in 24% bracket. Prior-year decrease = $50K * 37%
        // = $18,500. Current-year deduction value = $50K * 24% = $12,000.
        // cy_without = $60K, cy_with_ded = $48K. Method A: $48K. Method
        // B: $60K − $18.5K = $41.5K. Credit wins by $6,500.
        let r = compute(&input(50_000_00, 60_000_00, 48_000_00, 18_500_00));
        assert!(matches!(r.better_method, Method::CreditMethodB));
        assert_eq!(r.final_tax_cents, 41_500_00);
        assert_eq!(r.savings_vs_deduction_only_cents, 6_500_00);
    }

    #[test]
    fn social_security_overpayment_repayment_path() {
        // SSA overpayment of $10,000 received 2023 (taxed at 22%), repaid
        // 2026 in 22% bracket. Prior-year decrease = $2,200. cy_with_ded
        // = cy_without - $2,200 = same as Method B → tie. Picks deduction.
        let r = compute(&input(10_000_00, 20_000_00, 17_800_00, 2_200_00));
        assert!(matches!(r.better_method, Method::DeductionMethodA));
        assert_eq!(r.final_tax_cents, 17_800_00);
    }

    #[test]
    fn equal_methods_select_deduction() {
        // Bracket parity → tie → pick deduction (simpler).
        let r = compute(&input(10_000_00, 10_000_00, 7_800_00, 2_200_00));
        assert_eq!(r.deduction_method_tax_cents, r.credit_method_tax_cents);
        assert!(matches!(r.better_method, Method::DeductionMethodA));
    }
}
