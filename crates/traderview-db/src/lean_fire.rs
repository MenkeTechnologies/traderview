//! Lean FIRE calculator.
//!
//! Lean FIRE is the FIRE variant aimed at minimalist retirement —
//! typically annual expenses ≤ $40k (loose community-published
//! threshold; some define < $25k or < median household expenses).
//! Low spending → smaller portfolio target → faster timeline → but
//! less margin for error if expenses surprise.
//!
//! This module is purposely focused on the minimalist case: validates
//! the user's expense plan is within the lean tier, computes the FI
//! number + years-to-target + classifies the user's expense level as
//! ultralean / lean / borderline / not_lean.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LeanFireInput {
    pub current_nw_usd: f64,
    pub annual_expenses_usd: f64,
    #[serde(default = "default_swr")]
    pub safe_withdrawal_rate_pct: f64,
    #[serde(default = "default_return")]
    pub expected_real_return_pct: f64,
    #[serde(default)]
    pub monthly_contribution_usd: f64,
}

fn default_swr() -> f64 { 4.0 }
fn default_return() -> f64 { 5.0 }

#[derive(Debug, Clone, Serialize)]
pub struct LeanFireReport {
    pub fi_number_usd: f64,
    pub current_vs_fi_delta_usd: f64,
    pub is_lean_fi: bool,
    pub years_to_fi: Option<f64>,
    pub expense_tier: &'static str,
    pub upper_lean_threshold_usd: f64,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

const ULTRALEAN_CAP: f64 = 25_000.0;
const LEAN_CAP: f64 = 40_000.0;
const BORDERLINE_CAP: f64 = 55_000.0;

pub fn fi_number(annual_expenses: f64, swr_pct: f64) -> f64 {
    if swr_pct <= 0.0 { return 0.0; }
    annual_expenses / (swr_pct / 100.0)
}

pub fn classify_expense_tier(annual_expenses: f64) -> &'static str {
    if annual_expenses <= ULTRALEAN_CAP { "ultralean" }
    else if annual_expenses <= LEAN_CAP { "lean" }
    else if annual_expenses <= BORDERLINE_CAP { "borderline" }
    else { "not_lean" }
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

pub fn compute(input: &LeanFireInput) -> LeanFireReport {
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
    LeanFireReport {
        fi_number_usd: fi,
        current_vs_fi_delta_usd: delta,
        is_lean_fi: is_fi,
        years_to_fi: years,
        expense_tier: tier,
        upper_lean_threshold_usd: LEAN_CAP,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> LeanFireInput {
        LeanFireInput {
            current_nw_usd: 200_000.0,
            annual_expenses_usd: 30_000.0,
            safe_withdrawal_rate_pct: 4.0,
            expected_real_return_pct: 5.0,
            monthly_contribution_usd: 1_500.0,
        }
    }

    #[test]
    fn fi_number_basic() {
        assert_eq!(fi_number(30_000.0, 4.0), 750_000.0);
    }

    #[test]
    fn classify_expense_tier_thresholds() {
        assert_eq!(classify_expense_tier(20_000.0), "ultralean");
        assert_eq!(classify_expense_tier(25_000.0), "ultralean");
        assert_eq!(classify_expense_tier(35_000.0), "lean");
        assert_eq!(classify_expense_tier(40_000.0), "lean");
        assert_eq!(classify_expense_tier(50_000.0), "borderline");
        assert_eq!(classify_expense_tier(80_000.0), "not_lean");
    }

    #[test]
    fn years_until_target_already_there() {
        assert_eq!(years_until_target(500_000.0, 0.0, 5.0, 100_000.0), Some(0.0));
    }

    #[test]
    fn years_until_target_impossible() {
        assert!(years_until_target(100.0, 0.0, 0.0, 1_000_000.0).is_none());
    }

    #[test]
    fn compute_lean_30k_fi() {
        let r = compute(&input());
        assert_eq!(r.fi_number_usd, 750_000.0);
        assert_eq!(r.expense_tier, "lean");
        assert_eq!(r.upper_lean_threshold_usd, 40_000.0);
    }

    #[test]
    fn compute_not_yet_at_200k() {
        let r = compute(&input());
        assert!(!r.is_lean_fi);
        assert!(r.current_vs_fi_delta_usd < 0.0);
    }

    #[test]
    fn compute_already_fi_at_800k() {
        let mut i = input();
        i.current_nw_usd = 800_000.0;
        let r = compute(&i);
        assert!(r.is_lean_fi);
        assert_eq!(r.years_to_fi, Some(0.0));
    }

    #[test]
    fn compute_ultralean_classification() {
        let mut i = input();
        i.annual_expenses_usd = 20_000.0;
        let r = compute(&i);
        assert_eq!(r.expense_tier, "ultralean");
    }

    #[test]
    fn compute_not_lean_classification() {
        let mut i = input();
        i.annual_expenses_usd = 80_000.0;
        let r = compute(&i);
        assert_eq!(r.expense_tier, "not_lean");
    }

    #[test]
    fn compute_years_to_fi_positive_for_realistic() {
        let r = compute(&input());
        assert!(r.years_to_fi.is_some());
        let y = r.years_to_fi.unwrap();
        assert!(y > 5.0 && y < 30.0);
    }
}
