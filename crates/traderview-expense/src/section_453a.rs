//! IRC §453A — Special rules for nondealer installment obligations.
//!
//! Pairs with `section_453` (installment-method election baseline).
//! §453 lets a nondealer report gain on a deferred-payment sale as
//! payments are received instead of in full at closing. §453A then
//! imposes a non-deductible **interest charge** on the deferred tax
//! liability attributable to the portion of large outstanding
//! installment obligations exceeding $5 million at year-end. This
//! offsets the time-value benefit of the deferral.
//!
//! **Applicability gates** (all must be true):
//!
//! - Per-sale gate: sales price > $150,000 (§453A(b)(1)).
//! - Aggregate gate: aggregate face amount of all installment
//!   obligations from the taxpayer's nondealer sales arising during
//!   the year AND outstanding at year-end exceeds $5,000,000
//!   (§453A(b)(2)(A)).
//! - Property type: not a "personal-use" property and not residential
//!   lots / timeshares sold by a dealer (§453A(b)(3),(4)).
//! - Taxpayer type: nondealer.
//!
//! **Computation** (§453A(c)):
//!
//!   Applicable_Percentage = (Aggregate_Face − $5,000,000) / Aggregate_Face
//!   Deferred_Tax_Liability = Unrecognized_Gain × Max_Applicable_Tax_Rate
//!   Interest_Charge = Applicable_Percentage × Deferred_Tax_Liability
//!                     × Underpayment_Rate(§6621)
//!
//! The underpayment rate under §6621 is the federal short-term rate
//! plus 3 percentage points, set quarterly by the IRS via revenue
//! ruling. The interest charge is a non-deductible additional tax
//! reported on Form 1040 Schedule 2 line 8c (individual) or on the
//! relevant corporate return line.
//!
//! Sources:
//! [Cornell LII 26 U.S.C. § 453A](https://www.law.cornell.edu/uscode/text/26/453A),
//! [IRS LB&I — Interest on Deferred Tax Liability practice unit](https://www.irs.gov/pub/fatca/int_practice_units/interest-on-deferred-tax-liability.pdf),
//! [RPB CPA — Navigating §453A $5M threshold](https://rpbcpa.com/navigating-section-453a-tax-implications-of-installment-sales-of-5-million/),
//! [Accounting Insights — §453A special rules](https://accountinginsights.org/irc-section-453a-special-rules-for-installment-sales-and-deferred-tax/).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicabilityResult {
    /// §453A interest charge applies — all gates passed.
    Applicable,
    /// Per-sale gate not met (sales price ≤ $150,000).
    BelowPerSaleThreshold,
    /// Aggregate gate not met (year-end face ≤ $5M).
    BelowAggregateThreshold,
    /// Taxpayer is a dealer — §453A does not apply (dealers use
    /// §453(l) accrual; §453A is non-dealer-only).
    DealerExcluded,
    /// Property is residential lots or timeshares sold by a dealer —
    /// §453A(b)(4) excluded.
    ResidentialLotsOrTimesharesExcluded,
    /// Property is personal-use — §453A(b)(3) excluded.
    PersonalUseExcluded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section453aInput {
    pub sales_price_dollars: i64,
    /// Aggregate face amount of all qualifying installment
    /// obligations outstanding at year-end (across all sales the
    /// taxpayer made during the year that fall under §453A).
    pub aggregate_year_end_face_obligations_dollars: i64,
    /// Unrecognized gain on the installment note(s) at year-end —
    /// the gross-profit-not-yet-reported portion.
    pub unrecognized_gain_dollars: i64,
    /// Maximum tax rate applicable to the character of the deferred
    /// gain (in basis points: 2100 = 21% C-corp, 3700 = 37% top
    /// ordinary individual, 2000 = 20% top LTCG, etc.).
    pub maximum_applicable_tax_rate_bp: u32,
    /// Federal short-term underpayment rate under IRC §6621 (in
    /// basis points; equal to short-term AFR + 300bp). Set
    /// quarterly by IRS via revenue ruling.
    pub underpayment_rate_bp: u32,
    pub taxpayer_is_dealer: bool,
    pub property_is_personal_use: bool,
    pub property_is_residential_lots_or_timeshares: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section453aResult {
    pub applicability: ApplicabilityResult,
    /// (Aggregate face − $5M) / Aggregate face, in basis points.
    pub applicable_percentage_bp: u32,
    /// Unrecognized gain × max applicable tax rate.
    pub deferred_tax_liability_dollars: i64,
    /// Final non-deductible interest charge.
    pub interest_charge_dollars: i64,
    pub citation: String,
    pub note: String,
}

const PER_SALE_THRESHOLD_DOLLARS: i64 = 150_000;
const AGGREGATE_THRESHOLD_DOLLARS: i64 = 5_000_000;

pub fn compute(input: &Section453aInput) -> Section453aResult {
    // Gate evaluation in §453A(b) order: exclusions before threshold
    // tests because exclusions are categorical.
    let applicability = if input.taxpayer_is_dealer {
        ApplicabilityResult::DealerExcluded
    } else if input.property_is_personal_use {
        ApplicabilityResult::PersonalUseExcluded
    } else if input.property_is_residential_lots_or_timeshares {
        ApplicabilityResult::ResidentialLotsOrTimesharesExcluded
    } else if input.sales_price_dollars <= PER_SALE_THRESHOLD_DOLLARS {
        ApplicabilityResult::BelowPerSaleThreshold
    } else if input.aggregate_year_end_face_obligations_dollars <= AGGREGATE_THRESHOLD_DOLLARS {
        ApplicabilityResult::BelowAggregateThreshold
    } else {
        ApplicabilityResult::Applicable
    };

    // Applicable percentage — only meaningful when applicable.
    let applicable_pct_bp = if matches!(applicability, ApplicabilityResult::Applicable) {
        let numerator =
            input.aggregate_year_end_face_obligations_dollars - AGGREGATE_THRESHOLD_DOLLARS;
        let denom = input.aggregate_year_end_face_obligations_dollars;
        ((numerator as i128 * 10_000 / denom as i128).max(0) as u64).min(10_000) as u32
    } else {
        0
    };

    let deferred_tax = ((input.unrecognized_gain_dollars.max(0) as i128)
        * (input.maximum_applicable_tax_rate_bp as i128)
        / 10_000) as i64;

    let interest_charge = if matches!(applicability, ApplicabilityResult::Applicable) {
        // applicable_% × deferred_tax × underpayment_rate.
        let step1 = (deferred_tax as i128) * (applicable_pct_bp as i128) / 10_000;
        let step2 = step1 * (input.underpayment_rate_bp as i128) / 10_000;
        step2.max(0) as i64
    } else {
        0
    };

    let applicability_label = match applicability {
        ApplicabilityResult::Applicable => "§453A interest charge applies",
        ApplicabilityResult::BelowPerSaleThreshold => {
            "Per-sale price ≤ $150,000 — §453A(b)(1) gate not met"
        }
        ApplicabilityResult::BelowAggregateThreshold => {
            "Aggregate year-end face ≤ $5M — §453A(b)(2)(A) gate not met"
        }
        ApplicabilityResult::DealerExcluded => {
            "Dealer — §453A is non-dealer-only; dealers use §453(l)"
        }
        ApplicabilityResult::ResidentialLotsOrTimesharesExcluded => {
            "Residential lots / timeshares sold by dealer — §453A(b)(4) excluded"
        }
        ApplicabilityResult::PersonalUseExcluded => "Personal-use property — §453A(b)(3) excluded",
    };

    let note = format!(
        "Sales price ${}; aggregate year-end face ${}; unrecognized gain ${} × {}.{}% max rate = ${} deferred tax; applicable % {}.{}%; underpayment rate (§6621) {}.{}%; interest charge ${}. {}.",
        input.sales_price_dollars,
        input.aggregate_year_end_face_obligations_dollars,
        input.unrecognized_gain_dollars,
        input.maximum_applicable_tax_rate_bp / 100,
        input.maximum_applicable_tax_rate_bp % 100,
        deferred_tax,
        applicable_pct_bp / 100,
        applicable_pct_bp % 100,
        input.underpayment_rate_bp / 100,
        input.underpayment_rate_bp % 100,
        interest_charge,
        applicability_label,
    );

    Section453aResult {
        applicability,
        applicable_percentage_bp: applicable_pct_bp,
        deferred_tax_liability_dollars: deferred_tax,
        interest_charge_dollars: interest_charge,
        citation:
            "IRC §453A Special rules for nondealers (Omnibus Budget Reconciliation Act of 1987 §10202(d); current form post-1988); §453A(b) applicability gates: $150k per-sale floor + $5M aggregate year-end face + nondealer + non-personal-use + non-residential-lots/timeshares; §453A(c) applicable % = (aggregate − $5M) / aggregate; interest rate = IRC §6621 federal underpayment rate (short-term AFR + 3 pp); non-deductible additional tax reported on Form 1040 Schedule 2 line 8c"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Section453aInput {
        Section453aInput {
            sales_price_dollars: 10_000_000,
            aggregate_year_end_face_obligations_dollars: 10_000_000,
            unrecognized_gain_dollars: 8_000_000,
            maximum_applicable_tax_rate_bp: 2000, // 20% LTCG
            underpayment_rate_bp: 800,            // 8% — 5% AFR + 3% spread (illustrative)
            taxpayer_is_dealer: false,
            property_is_personal_use: false,
            property_is_residential_lots_or_timeshares: false,
        }
    }

    // ── Applicability gates ────────────────────────────────────────

    #[test]
    fn baseline_applicable() {
        let r = compute(&baseline());
        assert_eq!(r.applicability, ApplicabilityResult::Applicable);
    }

    #[test]
    fn per_sale_at_150k_exact_below_threshold() {
        let mut i = baseline();
        i.sales_price_dollars = 150_000;
        let r = compute(&i);
        assert_eq!(
            r.applicability,
            ApplicabilityResult::BelowPerSaleThreshold,
            "§453A(b)(1) requires price > $150k (strict greater)"
        );
        assert_eq!(r.interest_charge_dollars, 0);
    }

    #[test]
    fn per_sale_one_dollar_over_meets_threshold() {
        // $150,001 sales price + $10M aggregate + non-excluded = applicable.
        let mut i = baseline();
        i.sales_price_dollars = 150_001;
        let r = compute(&i);
        assert_eq!(r.applicability, ApplicabilityResult::Applicable);
    }

    #[test]
    fn aggregate_at_5m_exact_below_threshold() {
        let mut i = baseline();
        i.aggregate_year_end_face_obligations_dollars = 5_000_000;
        let r = compute(&i);
        assert_eq!(
            r.applicability,
            ApplicabilityResult::BelowAggregateThreshold,
            "§453A(b)(2)(A) requires aggregate > $5M (strict greater)"
        );
    }

    #[test]
    fn aggregate_one_dollar_over_meets_threshold() {
        let mut i = baseline();
        i.aggregate_year_end_face_obligations_dollars = 5_000_001;
        let r = compute(&i);
        assert_eq!(r.applicability, ApplicabilityResult::Applicable);
    }

    // ── Exclusions ─────────────────────────────────────────────────

    #[test]
    fn dealer_excluded() {
        let mut i = baseline();
        i.taxpayer_is_dealer = true;
        let r = compute(&i);
        assert_eq!(r.applicability, ApplicabilityResult::DealerExcluded);
        assert_eq!(r.interest_charge_dollars, 0);
    }

    #[test]
    fn personal_use_excluded() {
        let mut i = baseline();
        i.property_is_personal_use = true;
        let r = compute(&i);
        assert_eq!(r.applicability, ApplicabilityResult::PersonalUseExcluded);
        assert_eq!(r.interest_charge_dollars, 0);
    }

    #[test]
    fn residential_lots_or_timeshares_excluded() {
        let mut i = baseline();
        i.property_is_residential_lots_or_timeshares = true;
        let r = compute(&i);
        assert_eq!(
            r.applicability,
            ApplicabilityResult::ResidentialLotsOrTimesharesExcluded
        );
    }

    #[test]
    fn dealer_exclusion_short_circuits_before_thresholds() {
        // Dealer + below thresholds — should still report
        // DealerExcluded, not BelowPerSaleThreshold.
        let mut i = baseline();
        i.taxpayer_is_dealer = true;
        i.sales_price_dollars = 50_000;
        i.aggregate_year_end_face_obligations_dollars = 100_000;
        let r = compute(&i);
        assert_eq!(r.applicability, ApplicabilityResult::DealerExcluded);
    }

    // ── Applicable percentage formula ──────────────────────────────

    #[test]
    fn applicable_pct_10m_aggregate_50_pct() {
        // ($10M − $5M) / $10M = 50%.
        let r = compute(&baseline());
        assert_eq!(r.applicable_percentage_bp, 5000);
    }

    #[test]
    fn applicable_pct_20m_aggregate_75_pct() {
        // ($20M − $5M) / $20M = 75%.
        let mut i = baseline();
        i.aggregate_year_end_face_obligations_dollars = 20_000_000;
        let r = compute(&i);
        assert_eq!(r.applicable_percentage_bp, 7500);
    }

    #[test]
    fn applicable_pct_50m_aggregate_90_pct() {
        // ($50M − $5M) / $50M = 90%.
        let mut i = baseline();
        i.aggregate_year_end_face_obligations_dollars = 50_000_000;
        let r = compute(&i);
        assert_eq!(r.applicable_percentage_bp, 9000);
    }

    #[test]
    fn applicable_pct_approaches_100_at_very_large_aggregate() {
        // $5B aggregate → 99.9%.
        let mut i = baseline();
        i.aggregate_year_end_face_obligations_dollars = 5_000_000_000;
        let r = compute(&i);
        assert_eq!(r.applicable_percentage_bp, 9990);
    }

    // ── Deferred tax liability ─────────────────────────────────────

    #[test]
    fn deferred_tax_at_20_pct_ltcg() {
        // $8M × 20% = $1.6M.
        let r = compute(&baseline());
        assert_eq!(r.deferred_tax_liability_dollars, 1_600_000);
    }

    #[test]
    fn deferred_tax_at_37_pct_ordinary() {
        // $8M × 37% = $2.96M.
        let mut i = baseline();
        i.maximum_applicable_tax_rate_bp = 3700;
        let r = compute(&i);
        assert_eq!(r.deferred_tax_liability_dollars, 2_960_000);
    }

    #[test]
    fn deferred_tax_at_21_pct_ccorp() {
        let mut i = baseline();
        i.maximum_applicable_tax_rate_bp = 2100;
        let r = compute(&i);
        assert_eq!(r.deferred_tax_liability_dollars, 1_680_000);
    }

    // ── Interest charge — full integration ─────────────────────────

    #[test]
    fn baseline_interest_charge_math() {
        // applicable% 50% × deferred $1.6M × underpayment 8% = $64,000.
        let r = compute(&baseline());
        assert_eq!(r.interest_charge_dollars, 64_000);
    }

    #[test]
    fn interest_charge_scales_with_underpayment_rate() {
        // Doubling underpayment rate doubles interest charge.
        let mut i = baseline();
        i.underpayment_rate_bp = 1600; // 16%
        let r = compute(&i);
        assert_eq!(r.interest_charge_dollars, 128_000);
    }

    #[test]
    fn interest_charge_zero_when_not_applicable() {
        let mut i = baseline();
        i.sales_price_dollars = 100_000; // below $150k
        let r = compute(&i);
        assert_eq!(r.interest_charge_dollars, 0);
        assert_eq!(r.applicable_percentage_bp, 0);
    }

    #[test]
    fn interest_charge_zero_with_zero_unrecognized_gain() {
        // Even if applicable, zero deferred tax → zero interest.
        let mut i = baseline();
        i.unrecognized_gain_dollars = 0;
        let r = compute(&i);
        assert_eq!(r.applicability, ApplicabilityResult::Applicable);
        assert_eq!(r.interest_charge_dollars, 0);
    }

    // ── Citation contents ──────────────────────────────────────────

    #[test]
    fn citation_mentions_obra_1987_origin() {
        let r = compute(&baseline());
        assert!(r
            .citation
            .contains("Omnibus Budget Reconciliation Act of 1987"));
    }

    #[test]
    fn citation_mentions_5m_aggregate_threshold() {
        let r = compute(&baseline());
        assert!(r.citation.contains("$5M aggregate"));
    }

    #[test]
    fn citation_mentions_section_6621_underpayment_rate() {
        let r = compute(&baseline());
        assert!(r.citation.contains("§6621"));
        assert!(r.citation.contains("AFR"));
    }

    #[test]
    fn citation_mentions_schedule_2_line_8c() {
        let r = compute(&baseline());
        assert!(r.citation.contains("Schedule 2 line 8c"));
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn note_applicable_describes_charge() {
        let r = compute(&baseline());
        assert!(r.note.contains("§453A interest charge applies"));
    }

    #[test]
    fn note_per_sale_failure_explains() {
        let mut i = baseline();
        i.sales_price_dollars = 100_000;
        let r = compute(&i);
        assert!(r.note.contains("§453A(b)(1)"));
    }

    #[test]
    fn note_aggregate_failure_explains() {
        let mut i = baseline();
        i.aggregate_year_end_face_obligations_dollars = 4_000_000;
        let r = compute(&i);
        assert!(r.note.contains("§453A(b)(2)(A)"));
    }

    // ── Defensive ──────────────────────────────────────────────────

    #[test]
    fn very_large_billion_dollar_sale_no_precision_loss() {
        // $1B sale, $800M gain, 37% rate, 8% underpayment.
        let mut i = baseline();
        i.sales_price_dollars = 1_000_000_000;
        i.aggregate_year_end_face_obligations_dollars = 1_000_000_000;
        i.unrecognized_gain_dollars = 800_000_000;
        i.maximum_applicable_tax_rate_bp = 3700;
        i.underpayment_rate_bp = 800;
        let r = compute(&i);
        // applicable% = ($1B − $5M) / $1B = 99.5% (9950 bp).
        assert_eq!(r.applicable_percentage_bp, 9950);
        // deferred tax = $800M × 37% = $296M.
        assert_eq!(r.deferred_tax_liability_dollars, 296_000_000);
        // interest = 99.5% × $296M × 8% = $23,561,600.
        assert_eq!(r.interest_charge_dollars, 23_561_600);
    }

    #[test]
    fn negative_unrecognized_gain_clamped_to_zero_deferred_tax() {
        // Pathological negative gain → deferred tax clamps to 0.
        let mut i = baseline();
        i.unrecognized_gain_dollars = -1_000_000;
        let r = compute(&i);
        assert_eq!(r.deferred_tax_liability_dollars, 0);
        assert_eq!(r.interest_charge_dollars, 0);
    }
}
