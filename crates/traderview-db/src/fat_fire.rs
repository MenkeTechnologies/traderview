//! Fat FIRE calculator.
//!
//! Fat FIRE is the FIRE variant aimed at high-spend retirement —
//! typically annual expenses ≥ $100k. Bigger portfolio target → longer
//! timeline → more margin for inflation / sequence-of-returns risk.
//! Above $250k/yr commonly called "Obese FIRE" by the community.
//!
//! This module validates the user's expense plan is within fat tier,
//! computes the FI number + years-to-target + classifies the spending
//! level as not_fat / borderline / fat / obese.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct FatFireInput {
    pub current_nw_usd: f64,
    pub annual_expenses_usd: f64,
    #[serde(default = "default_swr")]
    pub safe_withdrawal_rate_pct: f64,
    #[serde(default = "default_return")]
    pub expected_real_return_pct: f64,
    #[serde(default)]
    pub monthly_contribution_usd: f64,
}

fn default_swr() -> f64 { 3.5 }  // Fat FIRE more conservative SWR
fn default_return() -> f64 { 5.0 }

#[derive(Debug, Clone, Serialize)]
pub struct FatFireReport {
    pub fi_number_usd: f64,
    pub current_vs_fi_delta_usd: f64,
    pub is_fat_fi: bool,
    pub years_to_fi: Option<f64>,
    pub expense_tier: &'static str,
    pub lower_fat_threshold_usd: f64,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

const FAT_THRESHOLD: f64 = 100_000.0;
const BORDERLINE_THRESHOLD: f64 = 80_000.0;
const OBESE_THRESHOLD: f64 = 250_000.0;

pub fn fi_number(annual_expenses: f64, swr_pct: f64) -> f64 {
    if swr_pct <= 0.0 { return 0.0; }
    annual_expenses / (swr_pct / 100.0)
}

pub fn classify_expense_tier(annual_expenses: f64) -> &'static str {
    if annual_expenses >= OBESE_THRESHOLD { "obese" }
    else if annual_expenses >= FAT_THRESHOLD { "fat" }
    else if annual_expenses >= BORDERLINE_THRESHOLD { "borderline" }
    else { "not_fat" }
}

pub fn years_until_target(
    current: f64,
    monthly_contribution: f64,
    annual_return_pct: f64,
    target: f64,
) -> Option<f64> {
    if current >= target { return Some(0.0); }
    if monthly_contribution <= 0.0 && annual_return_pct <= 0.0 { return None; }
    let r = annual_return_pct / 100.0;
    let monthly_r = r / 12.0;
    let mut years = 0.0_f64;
    let step = 1.0 / 12.0;
    while years < 100.0 {
        years += step;
        let months = years * 12.0;
        let lump = current * (1.0 + r).powf(years);
        let annuity = if monthly_r.abs() < 1e-12 {
            monthly_contribution * months
        } else {
            monthly_contribution * ((1.0 + monthly_r).powf(months) - 1.0) / monthly_r
        };
        if lump + annuity >= target { return Some(years); }
    }
    None
}

pub fn compute(input: &FatFireInput) -> FatFireReport {
    let fi = fi_number(input.annual_expenses_usd, input.safe_withdrawal_rate_pct);
    let delta = input.current_nw_usd - fi;
    let is_fi = input.current_nw_usd >= fi - 0.005 && fi > 0.0;
    let years = years_until_target(
        input.current_nw_usd,
        input.monthly_contribution_usd,
        input.expected_real_return_pct,
        fi,
    );
    let tier = classify_expense_tier(input.annual_expenses_usd);
    FatFireReport {
        fi_number_usd: fi,
        current_vs_fi_delta_usd: delta,
        is_fat_fi: is_fi,
        years_to_fi: years,
        expense_tier: tier,
        lower_fat_threshold_usd: FAT_THRESHOLD,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> FatFireInput {
        FatFireInput {
            current_nw_usd: 500_000.0,
            annual_expenses_usd: 150_000.0,
            safe_withdrawal_rate_pct: 3.5,
            expected_real_return_pct: 5.0,
            monthly_contribution_usd: 5_000.0,
        }
    }

    #[test]
    fn fi_number_basic() {
        // $150k / 3.5% = $4.286M
        let fi = fi_number(150_000.0, 3.5);
        assert!((fi - 4_285_714.0).abs() < 100.0);
    }

    #[test]
    fn classify_expense_tier_thresholds() {
        assert_eq!(classify_expense_tier(60_000.0), "not_fat");
        assert_eq!(classify_expense_tier(80_000.0), "borderline");
        assert_eq!(classify_expense_tier(99_999.0), "borderline");
        assert_eq!(classify_expense_tier(100_000.0), "fat");
        assert_eq!(classify_expense_tier(200_000.0), "fat");
        assert_eq!(classify_expense_tier(250_000.0), "obese");
        assert_eq!(classify_expense_tier(500_000.0), "obese");
    }

    #[test]
    fn years_until_target_already() {
        assert_eq!(years_until_target(5_000_000.0, 0.0, 5.0, 1_000_000.0), Some(0.0));
    }

    #[test]
    fn years_until_target_impossible() {
        assert!(years_until_target(100.0, 0.0, 0.0, 10_000_000.0).is_none());
    }

    #[test]
    fn compute_fat_default_classification() {
        let r = compute(&input());
        assert_eq!(r.expense_tier, "fat");
        assert_eq!(r.lower_fat_threshold_usd, 100_000.0);
    }

    #[test]
    fn compute_not_yet_at_500k_nw() {
        let r = compute(&input());
        assert!(!r.is_fat_fi);
        assert!(r.current_vs_fi_delta_usd < 0.0);
    }

    #[test]
    fn compute_already_fi_at_5m() {
        let mut i = input();
        i.current_nw_usd = 5_000_000.0;
        let r = compute(&i);
        assert!(r.is_fat_fi);
        assert_eq!(r.years_to_fi, Some(0.0));
    }

    #[test]
    fn compute_obese_classification() {
        let mut i = input();
        i.annual_expenses_usd = 300_000.0;
        let r = compute(&i);
        assert_eq!(r.expense_tier, "obese");
    }

    #[test]
    fn compute_not_fat_classification() {
        let mut i = input();
        i.annual_expenses_usd = 60_000.0;
        let r = compute(&i);
        assert_eq!(r.expense_tier, "not_fat");
    }

    #[test]
    fn compute_years_to_fi_long_for_fat() {
        let r = compute(&input());
        assert!(r.years_to_fi.is_some());
        let y = r.years_to_fi.unwrap();
        // Need $4.3M starting from $500k at $5k/mo + 5% real → ~25-35 years.
        assert!(y > 15.0);
    }

    #[test]
    fn compute_fi_uses_user_swr_not_default() {
        let mut i = input();
        i.safe_withdrawal_rate_pct = 4.0;
        let r = compute(&i);
        // $150k / 4% = $3.75M (different from $4.286M default).
        assert!((r.fi_number_usd - 3_750_000.0).abs() < 1.0);
    }
}
