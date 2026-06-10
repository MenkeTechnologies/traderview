//! Barista FIRE calculator.
//!
//! "Barista FI" is the point at which your portfolio is large enough
//! to cover the **gap** between your living expenses and a low-stress
//! part-time job's after-tax income — the canonical example being a
//! Starbucks barista job that pays modest wages but covers healthcare.
//! You can quit the high-stress career and let the portfolio + part-time
//! income carry you to traditional retirement.
//!
//! Math:
//!   gap_annual_usd = annual_expenses_usd − annual_part_time_income_usd
//!   barista_fi_now = gap_annual_usd / SWR_pct   (= a smaller FI number)
//!   coast_today    = barista_fi_now / (1+r)^years_until_retirement
//!     (similar discounting to coast-fire — at retirement age you stop
//!     working part-time too, so portfolio must cover FULL expenses then;
//!     barista_fi is a near-term-only number)
//!
//! Reports:
//!   - full_fi_number_usd   — what the user would need without part-time
//!   - barista_fi_number_usd — what they need with part-time income
//!   - barista_savings_usd   — full_fi − barista_fi
//!   - current_vs_barista_delta_usd
//!   - is_barista_fi flag
//!   - years_until_barista_fi at current contribution rate
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct BaristaFireInput {
    pub current_nw_usd: f64,
    pub current_age: u32,
    pub annual_expenses_usd: f64,
    pub annual_part_time_income_usd: f64,
    #[serde(default = "default_swr")]
    pub safe_withdrawal_rate_pct: f64,
    #[serde(default = "default_return")]
    pub expected_real_return_pct: f64,
    #[serde(default)]
    pub current_monthly_contribution_usd: f64,
}

fn default_swr() -> f64 { 4.0 }
fn default_return() -> f64 { 5.0 }

#[derive(Debug, Clone, Serialize)]
pub struct BaristaFireReport {
    pub full_fi_number_usd: f64,
    pub barista_fi_number_usd: f64,
    pub barista_savings_usd: f64,
    pub gap_annual_usd: f64,
    pub current_vs_barista_delta_usd: f64,
    pub is_barista_fi: bool,
    pub years_until_barista_fi: Option<f64>,
    pub status: &'static str,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn fi_number(annual_expenses: f64, swr_pct: f64) -> f64 {
    if swr_pct <= 0.0 { return 0.0; }
    annual_expenses / (swr_pct / 100.0)
}

/// Months to grow `current` + monthly contributions to `target` at
/// `annual_return_pct` real return. Returns None if never reaches in 100 years.
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

pub fn compute(input: &BaristaFireInput) -> BaristaFireReport {
    let full_fi = fi_number(input.annual_expenses_usd, input.safe_withdrawal_rate_pct);
    let gap = (input.annual_expenses_usd - input.annual_part_time_income_usd).max(0.0);
    let barista_fi = fi_number(gap, input.safe_withdrawal_rate_pct);
    let savings = (full_fi - barista_fi).max(0.0);
    let delta = input.current_nw_usd - barista_fi;
    let is_barista = input.current_nw_usd >= barista_fi - 0.005;
    let years_to_barista = years_until_target(
        input.current_nw_usd,
        input.current_monthly_contribution_usd,
        input.expected_real_return_pct,
        barista_fi,
    );
    let _ = input.current_age;
    let status: &'static str = if barista_fi <= 0.0 { "no_gap" }
        else if is_barista { "barista_fi" }
        else { "not_yet" };
    BaristaFireReport {
        full_fi_number_usd: full_fi,
        barista_fi_number_usd: barista_fi,
        barista_savings_usd: savings,
        gap_annual_usd: gap,
        current_vs_barista_delta_usd: delta,
        is_barista_fi: is_barista,
        years_until_barista_fi: years_to_barista,
        status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> BaristaFireInput {
        BaristaFireInput {
            current_nw_usd: 200_000.0,
            current_age: 35,
            annual_expenses_usd: 40_000.0,
            annual_part_time_income_usd: 25_000.0,
            safe_withdrawal_rate_pct: 4.0,
            expected_real_return_pct: 5.0,
            current_monthly_contribution_usd: 500.0,
        }
    }

    #[test]
    fn fi_number_basic() {
        assert_eq!(fi_number(40_000.0, 4.0), 1_000_000.0);
    }

    #[test]
    fn years_until_target_already_there() {
        assert_eq!(years_until_target(500_000.0, 0.0, 5.0, 100_000.0), Some(0.0));
    }

    #[test]
    fn years_until_target_finite() {
        let y = years_until_target(100_000.0, 1_000.0, 5.0, 500_000.0).unwrap();
        assert!(y > 5.0 && y < 25.0);
    }

    #[test]
    fn years_until_target_none_when_impossible() {
        assert!(years_until_target(100.0, 0.0, 0.0, 1_000_000.0).is_none());
    }

    #[test]
    fn compute_full_fi_basic() {
        let r = compute(&input());
        assert_eq!(r.full_fi_number_usd, 1_000_000.0);
    }

    #[test]
    fn compute_barista_fi_smaller_than_full() {
        let r = compute(&input());
        // gap = 40k − 25k = 15k. barista = 15k / 4% = 375k.
        assert_eq!(r.barista_fi_number_usd, 375_000.0);
        assert!(r.barista_fi_number_usd < r.full_fi_number_usd);
        assert_eq!(r.barista_savings_usd, 625_000.0);
    }

    #[test]
    fn compute_gap_zero_when_part_time_covers_expenses() {
        let mut i = input();
        i.annual_part_time_income_usd = 45_000.0;
        let r = compute(&i);
        assert_eq!(r.gap_annual_usd, 0.0);
        assert_eq!(r.barista_fi_number_usd, 0.0);
        assert_eq!(r.status, "no_gap");
    }

    #[test]
    fn compute_not_yet_at_200k() {
        let r = compute(&input());
        assert!(!r.is_barista_fi);
        assert!(r.current_vs_barista_delta_usd < 0.0);
        assert_eq!(r.status, "not_yet");
    }

    #[test]
    fn compute_already_barista_fi_at_400k() {
        let mut i = input();
        i.current_nw_usd = 400_000.0;
        let r = compute(&i);
        assert!(r.is_barista_fi);
        assert_eq!(r.status, "barista_fi");
        assert_eq!(r.years_until_barista_fi, Some(0.0));
    }

    #[test]
    fn compute_years_to_barista_positive_at_200k() {
        let r = compute(&input());
        // Need to grow 200k → 375k with $500/mo at 5% real. Should be < 20 years.
        assert!(r.years_until_barista_fi.is_some());
        let y = r.years_until_barista_fi.unwrap();
        assert!(y > 5.0 && y < 30.0);
    }
}
