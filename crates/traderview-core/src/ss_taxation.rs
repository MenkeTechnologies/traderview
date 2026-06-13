//! Taxation of Social Security benefits (IRS Pub 915).
//!
//! How much of a year's benefits are taxable depends on "provisional income":
//!
//! ```text
//! provisional = other income + tax-exempt interest + 50% of benefits
//! ```
//!
//! Below the first threshold none is taxable; between the two thresholds up to
//! 50% is; above the second up to 85% is. The taxable amount is the lesser of
//! the 85% cap and the worksheet's tiered formula. The thresholds ($25k/$34k
//! single, $32k/$44k married-joint) are fixed in statute — never indexed.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    MarriedJoint,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SsTaxInput {
    /// Total Social Security benefits for the year.
    pub social_security_usd: f64,
    /// All other income that lands in AGI (wages, pensions, IRA/401k, etc.).
    pub other_income_usd: f64,
    /// Tax-exempt interest (muni bonds) — counted in provisional income.
    #[serde(default)]
    pub tax_exempt_interest_usd: f64,
    pub filing_status: FilingStatus,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SsTaxResult {
    /// other income + tax-exempt interest + 50% of benefits.
    pub provisional_income_usd: f64,
    pub base1_usd: f64,
    pub base2_usd: f64,
    /// Portion of benefits that is taxable.
    pub taxable_benefits_usd: f64,
    /// Taxable benefits as a percent of total benefits.
    pub taxable_pct: f64,
    /// Benefits that remain tax-free.
    pub nontaxable_benefits_usd: f64,
    /// "none", "up_to_50", or "up_to_85".
    pub tier: String,
}

fn thresholds(status: FilingStatus) -> (f64, f64) {
    match status {
        FilingStatus::Single => (25_000.0, 34_000.0),
        FilingStatus::MarriedJoint => (32_000.0, 44_000.0),
    }
}

pub fn analyze(input: &SsTaxInput) -> SsTaxResult {
    let ss = input.social_security_usd.max(0.0);
    let (base1, base2) = thresholds(input.filing_status);
    let provisional = input.other_income_usd + input.tax_exempt_interest_usd + 0.5 * ss;

    let (taxable, tier) = if provisional <= base1 {
        (0.0, "none")
    } else if provisional <= base2 {
        // Up to 50% tier.
        (
            (0.5 * (provisional - base1)).min(0.5 * ss),
            "up_to_50",
        )
    } else {
        // Up to 85% tier: 85% of the excess over base2, plus the smaller of
        // 50% of benefits or 50% of the band between the thresholds.
        let tier1 = (0.5 * ss).min(0.5 * (base2 - base1));
        let amount = 0.85 * (provisional - base2) + tier1;
        (amount.min(0.85 * ss), "up_to_85")
    };

    SsTaxResult {
        provisional_income_usd: provisional,
        base1_usd: base1,
        base2_usd: base2,
        taxable_benefits_usd: taxable,
        taxable_pct: if ss > 0.0 { taxable / ss * 100.0 } else { 0.0 },
        nontaxable_benefits_usd: ss - taxable,
        tier: tier.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn run(ss: f64, other: f64, exempt: f64, status: FilingStatus) -> SsTaxResult {
        analyze(&SsTaxInput {
            social_security_usd: ss,
            other_income_usd: other,
            tax_exempt_interest_usd: exempt,
            filing_status: status,
        })
    }

    #[test]
    fn below_first_threshold_none() {
        // SS 20k, other 10k → provisional 20k ≤ 25k.
        let r = run(20_000.0, 10_000.0, 0.0, FilingStatus::Single);
        assert!(close(r.provisional_income_usd, 20_000.0));
        assert!(close(r.taxable_benefits_usd, 0.0));
        assert_eq!(r.tier, "none");
    }

    #[test]
    fn fifty_percent_tier() {
        // SS 20k, other 20k → provisional 30k (25k–34k).
        // min(0.5×20k, 0.5×(30k−25k)) = min(10k, 2.5k) = 2,500.
        let r = run(20_000.0, 20_000.0, 0.0, FilingStatus::Single);
        assert!(close(r.taxable_benefits_usd, 2_500.0));
        assert_eq!(r.tier, "up_to_50");
    }

    #[test]
    fn eighty_five_tier_capped() {
        // SS 20k, other 40k → provisional 50k. Formula gives 18,100, capped at
        // 85% × 20k = 17,000.
        let r = run(20_000.0, 40_000.0, 0.0, FilingStatus::Single);
        assert!(close(r.taxable_benefits_usd, 17_000.0));
        assert!(close(r.taxable_pct, 85.0));
        assert_eq!(r.tier, "up_to_85");
    }

    #[test]
    fn eighty_five_tier_below_cap() {
        // SS 20k, other 25k → provisional 35k (just over 34k).
        // 0.85×(35k−34k) + min(10k, 4.5k) = 850 + 4,500 = 5,350.
        let r = run(20_000.0, 25_000.0, 0.0, FilingStatus::Single);
        assert!(close(r.taxable_benefits_usd, 5_350.0));
        assert_eq!(r.tier, "up_to_85");
    }

    #[test]
    fn married_joint_eighty_five() {
        // SS 20k, other 40k → provisional 50k > 44k.
        // 0.85×(50k−44k) + min(10k, 0.5×12k=6k) = 5,100 + 6,000 = 11,100.
        let r = run(20_000.0, 40_000.0, 0.0, FilingStatus::MarriedJoint);
        assert!(close(r.taxable_benefits_usd, 11_100.0));
    }

    #[test]
    fn tax_exempt_interest_raises_provisional() {
        let without = run(20_000.0, 20_000.0, 0.0, FilingStatus::Single);
        let with = run(20_000.0, 20_000.0, 5_000.0, FilingStatus::Single);
        assert!(close(with.provisional_income_usd, without.provisional_income_usd + 5_000.0));
        assert!(with.taxable_benefits_usd > without.taxable_benefits_usd);
    }

    #[test]
    fn nontaxable_is_remainder() {
        let r = run(20_000.0, 40_000.0, 0.0, FilingStatus::Single);
        assert!(close(r.nontaxable_benefits_usd, 20_000.0 - r.taxable_benefits_usd));
    }

    #[test]
    fn zero_benefits_guard() {
        let r = run(0.0, 40_000.0, 0.0, FilingStatus::Single);
        assert!(close(r.taxable_benefits_usd, 0.0));
        assert!(close(r.taxable_pct, 0.0));
    }
}
