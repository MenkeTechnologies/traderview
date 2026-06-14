//! Kiddie tax (§ 1(g)) calculator. A dependent child's unearned income is taxed
//! in tiers: the first half of the threshold ($1,300 for 2024 / $1,350 for 2025)
//! is offset by the dependent standard deduction, the next half is taxed at the
//! child's own rate (10%), and anything above the threshold is taxed at the
//! parent's marginal rate. The rule applies under age 18, or to full-time
//! students 19–23. Compares the child's total tax against the parent simply
//! holding the asset and paying long-term cap-gains — the "gift appreciated
//! stock to the kid" strategy only wins outside the kiddie-tax ages. Faithful
//! port of the former client-side calculator. Pure compute, not advice.

use serde::{Deserialize, Serialize};

/// Child's single-filer brackets (cap, rate) as used by the original.
const KID_BRACKETS_SINGLE: [(f64, f64); 5] = [
    (11_600.0, 0.10),
    (47_150.0, 0.12),
    (100_525.0, 0.22),
    (191_950.0, 0.24),
    (f64::INFINITY, 0.37),
];

#[derive(Debug, Clone, Deserialize)]
pub struct KiddieTaxInput {
    pub year: u32,
    pub kid_age: u32,
    pub kid_earned_income_usd: f64,
    pub kid_unearned_income_usd: f64,
    /// Parent's marginal rate as a fraction (0.32 = 32%).
    pub parent_marginal_rate_fraction: f64,
    /// Parent's long-term cap-gains rate as a fraction.
    pub parent_lt_cap_gains_fraction: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct KiddieTaxReport {
    pub kiddie_tax_applies: bool,
    pub eff_std_deduction_usd: f64,
    pub offset_by_std_ded_usd: f64,
    pub kids_rate_amount_usd: f64,
    pub subject_to_parent_rate_usd: f64,
    pub tax_on_earned_usd: f64,
    pub tax_at_kids_rate_usd: f64,
    pub tax_at_parent_rate_usd: f64,
    pub total_kid_tax_usd: f64,
    pub parent_direct_tax_usd: f64,
    /// Parent-direct tax − child's total: positive means gifting to the kid wins.
    pub savings_vs_parent_usd: f64,
    pub valid: bool,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn kid_tax(taxable: f64) -> f64 {
    let mut owe = 0.0;
    let mut last_cap = 0.0;
    for &(cap, rate) in KID_BRACKETS_SINGLE.iter() {
        let slice = (taxable.min(cap) - last_cap).max(0.0);
        owe += slice * rate;
        if taxable <= cap {
            break;
        }
        last_cap = cap;
    }
    owe
}

pub fn generate(i: &KiddieTaxInput) -> KiddieTaxReport {
    let std_ded: f64 = if i.year >= 2025 { 1_350.0 } else { 1_300.0 };
    let tier: f64 = if i.year >= 2025 { 2_700.0 } else { 2_600.0 };
    let half_tier = tier / 2.0;

    let applies = i.kid_age < 18 || (19..=23).contains(&i.kid_age);

    // Dependent standard deduction: greater of the floor or earned + $450.
    let eff_std_ded = std_ded.max(i.kid_earned_income_usd + 450.0);

    let subject_to_parent = (i.kid_unearned_income_usd - tier).max(0.0);
    let kids_rate_amount = (i.kid_unearned_income_usd.min(tier) - half_tier).max(0.0);
    let offset_by_std_ded = i.kid_unearned_income_usd.min(half_tier);
    let tax_on_earned = kid_tax((i.kid_earned_income_usd - eff_std_ded).max(0.0));
    let tax_at_kids_rate = kids_rate_amount * 0.10;
    let tax_at_parent_rate = subject_to_parent * i.parent_marginal_rate_fraction;

    let total_kid_tax = if applies {
        tax_on_earned + tax_at_kids_rate + tax_at_parent_rate
    } else {
        tax_on_earned + kid_tax(i.kid_unearned_income_usd)
    };

    let parent_direct_tax = i.kid_unearned_income_usd * i.parent_lt_cap_gains_fraction;
    let savings_vs_parent = parent_direct_tax - total_kid_tax;

    KiddieTaxReport {
        kiddie_tax_applies: applies,
        eff_std_deduction_usd: round2(eff_std_ded),
        offset_by_std_ded_usd: round2(offset_by_std_ded),
        kids_rate_amount_usd: round2(kids_rate_amount),
        subject_to_parent_rate_usd: round2(subject_to_parent),
        tax_on_earned_usd: round2(tax_on_earned),
        tax_at_kids_rate_usd: round2(tax_at_kids_rate),
        tax_at_parent_rate_usd: round2(tax_at_parent_rate),
        total_kid_tax_usd: round2(total_kid_tax),
        parent_direct_tax_usd: round2(parent_direct_tax),
        savings_vs_parent_usd: round2(savings_vs_parent),
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> KiddieTaxInput {
        KiddieTaxInput {
            year: 2026,
            kid_age: 12,
            kid_earned_income_usd: 0.0,
            kid_unearned_income_usd: 8_000.0,
            parent_marginal_rate_fraction: 0.32,
            parent_lt_cap_gains_fraction: 0.20,
        }
    }

    // Pins cross-checked against the JS compute() in Python.
    #[test]
    fn default_applies_2025_tier() {
        let d = generate(&base());
        assert!(d.valid);
        assert!(d.kiddie_tax_applies);
        assert!(close(d.eff_std_deduction_usd, 1_350.0));
        assert!(close(d.offset_by_std_ded_usd, 1_350.0));
        assert!(close(d.kids_rate_amount_usd, 1_350.0));
        assert!(close(d.subject_to_parent_rate_usd, 5_300.0));
        assert!(close(d.tax_on_earned_usd, 0.0));
        assert!(close(d.tax_at_kids_rate_usd, 135.0));
        assert!(close(d.tax_at_parent_rate_usd, 1_696.0));
        assert!(close(d.total_kid_tax_usd, 1_831.0));
        assert!(close(d.parent_direct_tax_usd, 1_600.0));
        assert!(close(d.savings_vs_parent_usd, -231.0));
    }

    #[test]
    fn exempt_adult_uses_kid_brackets_on_all() {
        let d = generate(&KiddieTaxInput { kid_age: 25, ..base() });
        assert!(!d.kiddie_tax_applies);
        // All $8,000 unearned at the kid's 10% bracket → $800.
        assert!(close(d.total_kid_tax_usd, 800.0));
    }

    #[test]
    fn year_2024_uses_lower_tier() {
        let d = generate(&KiddieTaxInput { year: 2024, ..base() });
        assert!(close(d.kids_rate_amount_usd, 1_300.0));
        assert!(close(d.subject_to_parent_rate_usd, 5_400.0));
        assert!(close(d.total_kid_tax_usd, 1_858.0));
    }

    #[test]
    fn student_19_to_23_applies() {
        let d = generate(&KiddieTaxInput { kid_age: 20, ..base() });
        assert!(d.kiddie_tax_applies);
    }
}
