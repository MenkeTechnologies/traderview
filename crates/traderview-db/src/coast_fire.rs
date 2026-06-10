//! Coast FIRE calculator.
//!
//! "Coast FI" is the point at which your portfolio is large enough
//! to compound on its own (no new contributions) to your full FI
//! number by your target retirement age. Once you hit Coast FI, you
//! can stop saving for retirement — though you still need to cover
//! living expenses with current income, you don't need to put any
//! more in for compounding to do the work.
//!
//! Coast FI math:
//!
//!   FI_number     = annual_expenses / SWR
//!   Coast_FI_now  = FI_number / (1+r)^years_to_retirement
//!
//! Reports:
//!   - fi_number_usd               — Trinity-Study target portfolio
//!   - coast_fi_today_usd          — required NW today to coast to FI
//!   - current_vs_coast_delta_usd  — how close you are
//!   - is_coast_fi                 — already at or above the coast number
//!   - projected_nw_at_retirement_usd if no more contributions
//!   - years_until_coast_fi        — at current contributions
//!   - per_year_coast_required     — coast number you need at each age
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CoastFireInput {
    pub current_nw_usd: f64,
    pub current_age: u32,
    pub target_retirement_age: u32,
    pub annual_expenses_usd: f64,
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
pub struct YearCoastReq {
    pub age: u32,
    pub coast_required_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoastFireReport {
    pub fi_number_usd: f64,
    pub coast_fi_today_usd: f64,
    pub current_vs_coast_delta_usd: f64,
    pub is_coast_fi: bool,
    pub projected_nw_at_retirement_no_contributions_usd: f64,
    pub years_until_coast_fi: Option<f64>,
    pub per_year_coast_required: Vec<YearCoastReq>,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn fi_number(annual_expenses: f64, swr_pct: f64) -> f64 {
    if swr_pct <= 0.0 { return 0.0; }
    annual_expenses / (swr_pct / 100.0)
}

pub fn coast_required_at(target_fi: f64, real_return_pct: f64, years_until: f64) -> f64 {
    if years_until <= 0.0 { return target_fi; }
    let r = real_return_pct / 100.0;
    target_fi / (1.0 + r).powf(years_until)
}

pub fn future_value_no_contribution(current: f64, return_pct: f64, years: f64) -> f64 {
    if years <= 0.0 { return current; }
    let r = return_pct / 100.0;
    current * (1.0 + r).powf(years)
}

/// Years from now until the user's portfolio (with current contributions)
/// reaches the Coast-FI number — i.e. when they can stop contributing.
/// Returns None if they never reach it within 100 years.
pub fn years_until_coast(
    current_nw: f64,
    monthly_contribution: f64,
    real_return_pct: f64,
    fi_number: f64,
    target_retirement_age: u32,
    current_age: u32,
) -> Option<f64> {
    if current_age >= target_retirement_age { return None; }
    let r = real_return_pct / 100.0;
    let monthly_r = r / 12.0;
    let mut years = 0.0_f64;
    let step = 1.0 / 12.0;
    while years < 100.0 {
        years += step;
        // Where am I after `years` of contributing?
        let months = years * 12.0;
        let lump = current_nw * (1.0 + r).powf(years);
        let annuity = if monthly_r.abs() < 1e-12 {
            monthly_contribution * months
        } else {
            monthly_contribution * ((1.0 + monthly_r).powf(months) - 1.0) / monthly_r
        };
        let nw = lump + annuity;
        // What's the coast requirement at this age?
        let years_remaining = (target_retirement_age - current_age) as f64 - years;
        if years_remaining <= 0.0 {
            // At retirement age now — need to BE at FI, not just coasting.
            if nw >= fi_number { return Some(years); }
            return None;
        }
        let coast_req = fi_number / (1.0 + r).powf(years_remaining);
        if nw >= coast_req { return Some(years); }
    }
    None
}

pub fn compute(input: &CoastFireInput) -> CoastFireReport {
    let years_until = if input.target_retirement_age > input.current_age {
        (input.target_retirement_age - input.current_age) as f64
    } else { 0.0 };
    let target_fi = fi_number(input.annual_expenses_usd, input.safe_withdrawal_rate_pct);
    let coast_today = coast_required_at(target_fi, input.expected_real_return_pct, years_until);
    let delta = input.current_nw_usd - coast_today;
    let is_coast = input.current_nw_usd >= coast_today - 0.005;
    let projected_no_contrib = future_value_no_contribution(
        input.current_nw_usd, input.expected_real_return_pct, years_until,
    );
    let years_to_coast = years_until_coast(
        input.current_nw_usd,
        input.current_monthly_contribution_usd,
        input.expected_real_return_pct,
        target_fi,
        input.target_retirement_age,
        input.current_age,
    );
    let mut per_year: Vec<YearCoastReq> = Vec::new();
    for age in input.current_age..=input.target_retirement_age {
        let yrs = (input.target_retirement_age - age) as f64;
        let req = coast_required_at(target_fi, input.expected_real_return_pct, yrs);
        per_year.push(YearCoastReq { age, coast_required_usd: req });
    }
    CoastFireReport {
        fi_number_usd: target_fi,
        coast_fi_today_usd: coast_today,
        current_vs_coast_delta_usd: delta,
        is_coast_fi: is_coast,
        projected_nw_at_retirement_no_contributions_usd: projected_no_contrib,
        years_until_coast_fi: years_to_coast,
        per_year_coast_required: per_year,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> CoastFireInput {
        CoastFireInput {
            current_nw_usd: 100_000.0,
            current_age: 30,
            target_retirement_age: 65,
            annual_expenses_usd: 40_000.0,
            safe_withdrawal_rate_pct: 4.0,
            expected_real_return_pct: 5.0,
            current_monthly_contribution_usd: 1_000.0,
        }
    }

    #[test]
    fn fi_number_basic() {
        assert_eq!(fi_number(40_000.0, 4.0), 1_000_000.0);
    }

    #[test]
    fn fi_number_zero_swr() {
        assert_eq!(fi_number(40_000.0, 0.0), 0.0);
    }

    #[test]
    fn coast_required_at_full_target_when_zero_years() {
        assert_eq!(coast_required_at(1_000_000.0, 5.0, 0.0), 1_000_000.0);
    }

    #[test]
    fn coast_required_at_basic() {
        // $1M discounted at 5% real for 30 years ≈ $231,377
        let c = coast_required_at(1_000_000.0, 5.0, 30.0);
        assert!((c - 231_377.0).abs() < 100.0, "got {c}");
    }

    #[test]
    fn future_value_no_contribution_basic() {
        let fv = future_value_no_contribution(100_000.0, 5.0, 10.0);
        // 100k × 1.05^10 ≈ 162,889
        assert!((fv - 162_889.0).abs() < 10.0);
    }

    #[test]
    fn years_until_coast_already_there() {
        // $1M at 5% real / 35 years to retirement / FI=$1M.
        // Coast number today = $1M / 1.05^35 ≈ $181k. NW $1M >> coast.
        let y = years_until_coast(1_000_000.0, 0.0, 5.0, 1_000_000.0, 65, 30);
        assert_eq!(y, Some(0.0_f64 + 1.0 / 12.0));  // first iter passes
    }

    #[test]
    fn years_until_coast_never_with_zero_contribution_too_low() {
        // $100 at 5% / 30 years, FI $1M. Coast at 30 = $231k.
        // $100 × 1.05^30 ≈ $432 — never reaches $231k.
        let y = years_until_coast(100.0, 0.0, 5.0, 1_000_000.0, 65, 30);
        assert!(y.is_none());
    }

    #[test]
    fn compute_coast_today_correct() {
        let r = compute(&input());
        // FI = $1M, 35y at 5% → coast ≈ $181,290
        assert!((r.coast_fi_today_usd - 181_290.0).abs() < 200.0);
    }

    #[test]
    fn compute_not_coast_fi_at_100k() {
        let r = compute(&input());
        assert!(!r.is_coast_fi);
        assert!(r.current_vs_coast_delta_usd < 0.0);
    }

    #[test]
    fn compute_is_coast_fi_when_above_threshold() {
        let mut i = input();
        i.current_nw_usd = 300_000.0;
        let r = compute(&i);
        assert!(r.is_coast_fi);
    }

    #[test]
    fn compute_per_year_count() {
        let r = compute(&input());
        // 30..=65 inclusive = 36 entries
        assert_eq!(r.per_year_coast_required.len(), 36);
        assert_eq!(r.per_year_coast_required[0].age, 30);
        assert_eq!(r.per_year_coast_required.last().unwrap().age, 65);
    }

    #[test]
    fn compute_per_year_increases_with_age() {
        let r = compute(&input());
        for w in r.per_year_coast_required.windows(2) {
            assert!(w[1].coast_required_usd >= w[0].coast_required_usd);
        }
    }

    #[test]
    fn compute_already_at_retirement_age_zero_years() {
        let mut i = input();
        i.current_age = 65;
        let r = compute(&i);
        assert_eq!(r.coast_fi_today_usd, r.fi_number_usd);
    }
}
