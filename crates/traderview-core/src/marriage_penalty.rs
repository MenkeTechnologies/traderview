//! Marriage penalty / bonus — the difference between a couple's federal income
//! tax filing jointly and the tax they would owe as two single filers.
//!
//! Each spouse's income is run through the single schedule (with the single
//! standard deduction); their combined income is run through the MFJ schedule
//! (with the MFJ standard deduction). The difference is the penalty (positive,
//! joint costs more) or the bonus (negative, joint costs less).
//!
//! 2026 brackets are exactly 2× the single brackets through the 32% rate, so a
//! penalty arises only when combined income reaches the 37% bracket
//! ($768,700 MFJ vs 2 × $640,600 single); a bonus is common whenever the two
//! incomes are unequal. Schedules are the web-verified 2026 figures (IRS
//! Rev. Proc. 2025-32 / Tax Foundation); standard deductions are overridable.

use serde::{Deserialize, Serialize};

/// 2026 ordinary brackets as (lower bound, marginal rate).
const SINGLE: [(f64, f64); 7] = [
    (0.0, 0.10),
    (12_400.0, 0.12),
    (50_400.0, 0.22),
    (105_700.0, 0.24),
    (201_775.0, 0.32),
    (256_225.0, 0.35),
    (640_600.0, 0.37),
];
const MFJ: [(f64, f64); 7] = [
    (0.0, 0.10),
    (24_800.0, 0.12),
    (100_800.0, 0.22),
    (211_400.0, 0.24),
    (403_550.0, 0.32),
    (512_450.0, 0.35),
    (768_700.0, 0.37),
];

fn d_std_single() -> f64 {
    16_100.0
}
fn d_std_mfj() -> f64 {
    32_200.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarriagePenaltyInput {
    pub spouse_a_income_usd: f64,
    pub spouse_b_income_usd: f64,
    /// Single standard deduction (2026 default).
    #[serde(default = "d_std_single")]
    pub std_deduction_single_usd: f64,
    /// MFJ standard deduction (2026 default).
    #[serde(default = "d_std_mfj")]
    pub std_deduction_mfj_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MarriagePenaltyResult {
    pub combined_income_usd: f64,
    pub joint_taxable_usd: f64,
    pub spouse_a_taxable_usd: f64,
    pub spouse_b_taxable_usd: f64,
    pub joint_tax_usd: f64,
    pub spouse_a_tax_usd: f64,
    pub spouse_b_tax_usd: f64,
    pub single_total_tax_usd: f64,
    /// Joint tax − two-single tax. Positive = penalty, negative = bonus.
    pub marriage_penalty_usd: f64,
    /// Penalty as a share of combined income. None when income is zero.
    pub penalty_pct_of_income: Option<f64>,
    pub joint_effective_rate_pct: Option<f64>,
    pub single_effective_rate_pct: Option<f64>,
    pub is_penalty: bool,
    pub is_bonus: bool,
}

/// Progressive tax from a (lower bound, rate) schedule.
fn tax_from(taxable: f64, schedule: &[(f64, f64)]) -> f64 {
    let mut tax = 0.0;
    for (i, &(lo, rate)) in schedule.iter().enumerate() {
        if taxable <= lo {
            break;
        }
        let hi = schedule.get(i + 1).map(|&(b, _)| b).unwrap_or(f64::INFINITY);
        tax += (taxable.min(hi) - lo) * rate;
    }
    tax
}

pub fn analyze(input: &MarriagePenaltyInput) -> MarriagePenaltyResult {
    let combined = input.spouse_a_income_usd + input.spouse_b_income_usd;

    let a_taxable = (input.spouse_a_income_usd - input.std_deduction_single_usd).max(0.0);
    let b_taxable = (input.spouse_b_income_usd - input.std_deduction_single_usd).max(0.0);
    let joint_taxable = (combined - input.std_deduction_mfj_usd).max(0.0);

    let a_tax = tax_from(a_taxable, &SINGLE);
    let b_tax = tax_from(b_taxable, &SINGLE);
    let single_total = a_tax + b_tax;
    let joint_tax = tax_from(joint_taxable, &MFJ);
    let penalty = joint_tax - single_total;

    let pct = |x: f64| {
        if combined > 0.0 {
            Some(x / combined * 100.0)
        } else {
            None
        }
    };

    MarriagePenaltyResult {
        combined_income_usd: combined,
        joint_taxable_usd: joint_taxable,
        spouse_a_taxable_usd: a_taxable,
        spouse_b_taxable_usd: b_taxable,
        joint_tax_usd: joint_tax,
        spouse_a_tax_usd: a_tax,
        spouse_b_tax_usd: b_tax,
        single_total_tax_usd: single_total,
        marriage_penalty_usd: penalty,
        penalty_pct_of_income: pct(penalty),
        joint_effective_rate_pct: pct(joint_tax),
        single_effective_rate_pct: pct(single_total),
        is_penalty: penalty > 1e-6,
        is_bonus: penalty < -1e-6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.5
    }

    fn base() -> MarriagePenaltyInput {
        MarriagePenaltyInput {
            spouse_a_income_usd: 60_000.0,
            spouse_b_income_usd: 60_000.0,
            std_deduction_single_usd: 16_100.0,
            std_deduction_mfj_usd: 32_200.0,
        }
    }

    #[test]
    fn equal_modest_incomes_neutral() {
        // Both in the 2×-structure range → joint == two singles.
        let r = analyze(&base());
        assert!(close(r.single_total_tax_usd, 10_040.0));
        assert!(close(r.joint_tax_usd, 10_040.0));
        assert!(close(r.marriage_penalty_usd, 0.0));
        assert!(!r.is_penalty && !r.is_bonus);
    }

    #[test]
    fn single_earner_gets_bonus() {
        let r = analyze(&MarriagePenaltyInput {
            spouse_a_income_usd: 200_000.0,
            spouse_b_income_usd: 0.0,
            ..base()
        });
        // single $36,734 vs joint $26,340 → bonus of $10,394.
        assert!(close(r.single_total_tax_usd, 36_734.0));
        assert!(close(r.joint_tax_usd, 26_340.0));
        assert!(close(r.marriage_penalty_usd, -10_394.0));
        assert!(r.is_bonus);
    }

    #[test]
    fn high_combined_income_penalty() {
        let r = analyze(&MarriagePenaltyInput {
            spouse_a_income_usd: 500_000.0,
            spouse_b_income_usd: 500_000.0,
            ..base()
        });
        // two singles $276,268.50 vs joint $280,250.50 → penalty $3,982.
        assert!(close(r.single_total_tax_usd, 276_268.50));
        assert!(close(r.joint_tax_usd, 280_250.50));
        assert!(close(r.marriage_penalty_usd, 3_982.0));
        assert!(r.is_penalty);
    }

    #[test]
    fn zero_income_zero_tax() {
        let r = analyze(&MarriagePenaltyInput {
            spouse_a_income_usd: 0.0,
            spouse_b_income_usd: 0.0,
            ..base()
        });
        assert!(close(r.joint_tax_usd, 0.0));
        assert!(close(r.marriage_penalty_usd, 0.0));
        assert!(r.penalty_pct_of_income.is_none());
    }

    #[test]
    fn standard_deduction_override_changes_taxable() {
        let r = analyze(&MarriagePenaltyInput {
            std_deduction_single_usd: 0.0,
            std_deduction_mfj_usd: 0.0,
            ..base()
        });
        // No deduction → joint taxable equals combined income.
        assert!(close(r.joint_taxable_usd, 120_000.0));
        assert!(close(r.spouse_a_taxable_usd, 60_000.0));
    }

    #[test]
    fn bonus_flag_excludes_penalty() {
        let r = analyze(&MarriagePenaltyInput {
            spouse_a_income_usd: 300_000.0,
            spouse_b_income_usd: 20_000.0,
            ..base()
        });
        assert!(r.is_bonus);
        assert!(!r.is_penalty);
        assert!(r.marriage_penalty_usd < 0.0);
    }

    #[test]
    fn joint_effective_rate() {
        let r = analyze(&base());
        // 10,040 / 120,000 = 8.3667%.
        assert!((r.joint_effective_rate_pct.unwrap() - 8.3667).abs() < 1e-3);
    }

    #[test]
    fn penalty_pct_tracks_sign() {
        let r = analyze(&MarriagePenaltyInput {
            spouse_a_income_usd: 500_000.0,
            spouse_b_income_usd: 500_000.0,
            ..base()
        });
        // 3,982 / 1,000,000 ≈ 0.3982%.
        assert!((r.penalty_pct_of_income.unwrap() - 0.3982).abs() < 1e-3);
    }
}
