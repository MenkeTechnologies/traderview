//! Simplified Student Aid Index (SAI) estimator — successor to the
//! Expected Family Contribution (EFC) formula under the FAFSA
//! Simplification Act (effective 2024-25 award year).
//!
//! The SAI uses the same general structure as the old EFC: a
//! parent contribution plus a student contribution. Each is built
//! from income (after allowances) and assets (after asset protection
//! allowance). The official formula is hundreds of pages of
//! tables; this is the SIMPLIFIED version commonly used by
//! financial-aid calculators:
//!
//!   Parent income contribution:
//!     income_protection_allowance ≈ $35k (family of 4) + $5k per
//!       additional member. (Real table is more granular by # in
//!       college; we approximate.)
//!     available_income = (parent_agi − income_protection − allowances)
//!     allowances ≈ ~7.65% of AGI for FICA, plus state/local tax
//!       ≈ 5% (rough). We bundle as 12.65% off AGI for simplicity.
//!     parent_income_contribution = graduated schedule:
//!         first $20k @ 22%, next $5k @ 25%, next $7k @ 29%, next
//!         $7k @ 34%, next $7k @ 40%, above @ 47%.
//!
//!   Parent asset contribution:
//!     asset_protection_allowance varies by parent age; we use $0
//!       under the new SAI (parents' APA was removed in 2024-25).
//!     parent_asset_contribution = max(0, parent_assets) × 5.64%
//!
//!   Student contribution:
//!     student_income_allowance ≈ $9,410 (new SAI).
//!     student_income_contribution = max(0, student_agi − allowance) × 50%
//!     student_asset_contribution = student_assets × 20%
//!
//!   SAI = parent_income_contribution + parent_asset_contribution
//!         + student_income_contribution + student_asset_contribution
//!
//! Number of dependents in college NO LONGER divides SAI under SAI
//! (vs old EFC which divided by it) — biggest single change with
//! Simplification Act.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct FafsaInput {
    pub parent_agi_usd: f64,
    #[serde(default)]
    pub parent_assets_usd: f64,
    #[serde(default)]
    pub student_agi_usd: f64,
    #[serde(default)]
    pub student_assets_usd: f64,
    /// Household size (parents + dependent children).
    pub household_size: u32,
    /// Number of dependents in college (informational; SAI no longer
    /// divides by this under Simplification Act).
    #[serde(default = "default_in_college")]
    pub dependents_in_college: u32,
}

fn default_in_college() -> u32 { 1 }

#[derive(Debug, Clone, Serialize)]
pub struct FafsaReport {
    pub parent_income_protection_usd: f64,
    pub parent_available_income_usd: f64,
    pub parent_income_contribution_usd: f64,
    pub parent_asset_contribution_usd: f64,
    pub student_income_contribution_usd: f64,
    pub student_asset_contribution_usd: f64,
    pub sai_usd: f64,
    pub sai_per_student_usd: f64,
    pub aid_tier: &'static str,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn parent_income_protection(household_size: u32) -> f64 {
    // Family of 4 ≈ $35k. +$5k per additional member.
    let base = 35_000.0_f64;
    if household_size <= 4 {
        base * (household_size as f64 / 4.0).max(0.5)
    } else {
        base + (household_size as f64 - 4.0) * 5_000.0
    }
}

pub fn parent_allowances(agi: f64) -> f64 {
    // FICA ~7.65% + state/local ~5% bundled.
    (agi * 0.1265).max(0.0)
}

/// Graduated schedule applied to AVAILABLE income (income after
/// protections + allowances).
pub fn parent_income_contribution_graduated(available_income: f64) -> f64 {
    if available_income <= 0.0 { return 0.0; }
    let brackets = [
        (20_000.0, 0.22),
        (5_000.0, 0.25),
        (7_000.0, 0.29),
        (7_000.0, 0.34),
        (7_000.0, 0.40),
    ];
    let mut remaining = available_income;
    let mut total = 0.0;
    for (width, rate) in brackets {
        let taxed = remaining.min(width);
        total += taxed * rate;
        remaining -= taxed;
        if remaining <= 0.0 { return total; }
    }
    // Above all brackets → top rate 47%.
    total + remaining * 0.47
}

pub fn aid_tier(sai: f64) -> &'static str {
    // Rough categorization tied to Pell Grant thresholds.
    if sai < 0.0 { "max_pell" }
    else if sai < 7_400.0 { "pell_eligible" }
    else if sai < 25_000.0 { "merit_likely" }
    else { "full_pay_likely" }
}

pub fn compute(input: &FafsaInput) -> FafsaReport {
    let income_protection = parent_income_protection(input.household_size);
    let allowances = parent_allowances(input.parent_agi_usd);
    let available_income = input.parent_agi_usd - income_protection - allowances;
    let parent_income_contrib = parent_income_contribution_graduated(available_income.max(0.0));
    let parent_asset_contrib = input.parent_assets_usd.max(0.0) * 0.0564;
    let student_income_allowance = 9_410.0;
    let student_avail_income = (input.student_agi_usd - student_income_allowance).max(0.0);
    let student_income_contrib = student_avail_income * 0.50;
    let student_asset_contrib = input.student_assets_usd.max(0.0) * 0.20;
    let sai = parent_income_contrib
        + parent_asset_contrib
        + student_income_contrib
        + student_asset_contrib;
    // Old EFC formula divided by dependents_in_college; SAI does not.
    // We report SAI as the official number and a derived per-student
    // figure for users still applying the old mental model.
    let sai_per_student = if input.dependents_in_college > 0 {
        sai / input.dependents_in_college as f64
    } else {
        sai
    };
    let tier = aid_tier(sai);
    FafsaReport {
        parent_income_protection_usd: income_protection,
        parent_available_income_usd: available_income.max(0.0),
        parent_income_contribution_usd: parent_income_contrib,
        parent_asset_contribution_usd: parent_asset_contrib,
        student_income_contribution_usd: student_income_contrib,
        student_asset_contribution_usd: student_asset_contrib,
        sai_usd: sai,
        sai_per_student_usd: sai_per_student,
        aid_tier: tier,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> FafsaInput {
        FafsaInput {
            parent_agi_usd: 100_000.0,
            parent_assets_usd: 50_000.0,
            student_agi_usd: 0.0,
            student_assets_usd: 0.0,
            household_size: 4,
            dependents_in_college: 1,
        }
    }

    #[test]
    fn parent_income_protection_family_of_4() {
        assert_eq!(parent_income_protection(4), 35_000.0);
    }

    #[test]
    fn parent_income_protection_family_of_5() {
        assert_eq!(parent_income_protection(5), 40_000.0);
    }

    #[test]
    fn parent_income_protection_small_family() {
        // Family of 2 → 35k × 0.5 = $17,500.
        assert_eq!(parent_income_protection(2), 17_500.0);
    }

    #[test]
    fn parent_allowances_basic() {
        let a = parent_allowances(100_000.0);
        assert!((a - 12_650.0).abs() < 1e-6);
    }

    #[test]
    fn parent_income_contribution_zero_when_available_negative() {
        assert_eq!(parent_income_contribution_graduated(-1000.0), 0.0);
    }

    #[test]
    fn parent_income_contribution_first_bracket() {
        let c = parent_income_contribution_graduated(10_000.0);
        assert!((c - 2_200.0).abs() < 1e-6);
    }

    #[test]
    fn parent_income_contribution_multiple_brackets() {
        // 25k = 20k@22 + 5k@25 = 4400 + 1250 = 5650
        let c = parent_income_contribution_graduated(25_000.0);
        assert!((c - 5_650.0).abs() < 1e-6);
    }

    #[test]
    fn parent_income_contribution_above_all_brackets() {
        // 50k → 20k@22 + 5k@25 + 7k@29 + 7k@34 + 7k@40 + 4k@47
        // = 4400 + 1250 + 2030 + 2380 + 2800 + 1880 = 14740
        let c = parent_income_contribution_graduated(50_000.0);
        assert!((c - 14_740.0).abs() < 1e-6);
    }

    #[test]
    fn aid_tier_max_pell_when_negative_sai() {
        assert_eq!(aid_tier(-100.0), "max_pell");
    }

    #[test]
    fn aid_tier_pell_eligible() {
        assert_eq!(aid_tier(5_000.0), "pell_eligible");
    }

    #[test]
    fn aid_tier_merit_likely() {
        assert_eq!(aid_tier(15_000.0), "merit_likely");
    }

    #[test]
    fn aid_tier_full_pay_likely() {
        assert_eq!(aid_tier(50_000.0), "full_pay_likely");
    }

    #[test]
    fn compute_basic() {
        let r = compute(&input());
        // available = 100k − 35k − 12.65k = 52.35k
        assert!((r.parent_available_income_usd - 52_350.0).abs() < 0.5);
        assert!(r.parent_income_contribution_usd > 0.0);
        assert!((r.parent_asset_contribution_usd - 50_000.0 * 0.0564).abs() < 0.01);
        assert!(r.sai_usd > 0.0);
    }

    #[test]
    fn compute_low_income_zero_sai() {
        let mut i = input();
        i.parent_agi_usd = 25_000.0;
        i.parent_assets_usd = 0.0;
        let r = compute(&i);
        // available negative → zero income contribution; zero assets → 0.
        assert_eq!(r.sai_usd, 0.0);
    }

    #[test]
    fn compute_student_income_contribution_50pct() {
        let mut i = input();
        i.student_agi_usd = 19_410.0;  // $10k above allowance
        let r = compute(&i);
        assert!((r.student_income_contribution_usd - 5_000.0).abs() < 0.01);
    }

    #[test]
    fn compute_student_asset_contribution_20pct() {
        let mut i = input();
        i.student_assets_usd = 10_000.0;
        let r = compute(&i);
        assert!((r.student_asset_contribution_usd - 2_000.0).abs() < 0.01);
    }

    #[test]
    fn compute_sai_per_student_divides_by_dependents() {
        let mut i = input();
        i.dependents_in_college = 2;
        let r = compute(&i);
        assert!((r.sai_per_student_usd - r.sai_usd / 2.0).abs() < 0.01);
    }
}
