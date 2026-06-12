//! 529 college savings planner.
//!
//! Two-part calculation:
//!
//!   1. Project total cost of a 4-year college education at the year
//!      the child starts:
//!      cost_year_t = annual_cost_today × (1 + tuition_inflation)^t
//!      summed over the 4 years of college. Tuition inflation has
//!      historically been ~5%/yr but is dropping toward ~3% recently.
//!
//!   2. Given current 529 balance, expected investment return, and
//!      years until college starts, compute monthly contribution
//!      required to reach the target.
//!      FV = current × (1+r)^n + m × ((1+r/12)^(12n) − 1) / (r/12)
//!      solve for m.
//!
//! Inputs:
//!   - child_age_years, college_start_age (default 18)
//!   - annual_cost_today_usd
//!   - tuition_inflation_pct (default 5)
//!   - years_in_college (default 4)
//!   - current_529_balance_usd
//!   - expected_annual_return_pct
//!
//! Compute returns:
//!   - years_until_college
//!   - per-year projected costs (vec of 4)
//!   - total_projected_cost_usd
//!   - projected_savings_at_start_usd (current balance grown)
//!   - shortfall_usd = total − projected savings
//!   - required_monthly_contribution_usd to close shortfall
//!   - on_track if current monthly contribution input (optional) ≥ required
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct College529Input {
    pub child_age_years: u32,
    #[serde(default = "default_start_age")]
    pub college_start_age: u32,
    pub annual_cost_today_usd: f64,
    #[serde(default = "default_tuition_infl")]
    pub tuition_inflation_pct: f64,
    #[serde(default = "default_years_college")]
    pub years_in_college: u32,
    #[serde(default)]
    pub current_529_balance_usd: f64,
    #[serde(default = "default_return")]
    pub expected_annual_return_pct: f64,
    #[serde(default)]
    pub current_monthly_contribution_usd: f64,
}

fn default_start_age() -> u32 { 18 }
fn default_tuition_infl() -> f64 { 5.0 }
fn default_years_college() -> u32 { 4 }
fn default_return() -> f64 { 6.0 }

#[derive(Debug, Clone, Serialize)]
pub struct YearCost {
    pub year_index: u32,
    pub year_age: u32,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct College529Report {
    pub years_until_college: u32,
    pub per_year_cost: Vec<YearCost>,
    pub total_projected_cost_usd: f64,
    pub projected_savings_at_start_usd: f64,
    pub shortfall_usd: f64,
    pub required_monthly_contribution_usd: f64,
    pub current_monthly_contribution_usd: f64,
    pub on_track: bool,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn future_value(current: f64, monthly: f64, annual_return_pct: f64, years: f64) -> f64 {
    if years <= 0.0 {
        return current;
    }
    let r = annual_return_pct / 100.0;
    let monthly_r = r / 12.0;
    let months = years * 12.0;
    let lump = current * (1.0 + r).powf(years);
    let annuity = if monthly_r.abs() < 1e-12 {
        monthly * months
    } else {
        monthly * ((1.0 + monthly_r).powf(months) - 1.0) / monthly_r
    };
    lump + annuity
}

pub fn required_monthly(current: f64, annual_return_pct: f64, years: f64, target: f64) -> f64 {
    if years <= 0.0 {
        return (target - current).max(0.0);
    }
    let r = annual_return_pct / 100.0;
    let monthly_r = r / 12.0;
    let months = years * 12.0;
    let lump = current * (1.0 + r).powf(years);
    let needed = (target - lump).max(0.0);
    if needed <= 0.0 {
        return 0.0;
    }
    if monthly_r.abs() < 1e-12 {
        return needed / months;
    }
    let annuity_factor = ((1.0 + monthly_r).powf(months) - 1.0) / monthly_r;
    needed / annuity_factor
}

pub fn compute(input: &College529Input) -> College529Report {
    let years_until = (input.college_start_age as i32 - input.child_age_years as i32).max(0) as u32;
    let infl_factor = 1.0 + input.tuition_inflation_pct / 100.0;
    let per_year: Vec<YearCost> = (0..input.years_in_college)
        .map(|i| {
            let years_out = years_until + i;
            let cost = input.annual_cost_today_usd * infl_factor.powi(years_out as i32);
            YearCost {
                year_index: i + 1,
                year_age: input.college_start_age + i,
                cost_usd: cost,
            }
        })
        .collect();
    let total_cost: f64 = per_year.iter().map(|y| y.cost_usd).sum();
    let projected_savings = future_value(
        input.current_529_balance_usd,
        0.0,
        input.expected_annual_return_pct,
        years_until as f64,
    );
    let shortfall = (total_cost - projected_savings).max(0.0);
    let required = required_monthly(
        input.current_529_balance_usd,
        input.expected_annual_return_pct,
        years_until as f64,
        total_cost,
    );
    let on_track = input.current_monthly_contribution_usd >= required - 0.005;
    College529Report {
        years_until_college: years_until,
        per_year_cost: per_year,
        total_projected_cost_usd: total_cost,
        projected_savings_at_start_usd: projected_savings,
        shortfall_usd: shortfall,
        required_monthly_contribution_usd: required,
        current_monthly_contribution_usd: input.current_monthly_contribution_usd,
        on_track,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> College529Input {
        College529Input {
            child_age_years: 5,
            college_start_age: 18,
            annual_cost_today_usd: 30_000.0,
            tuition_inflation_pct: 5.0,
            years_in_college: 4,
            current_529_balance_usd: 20_000.0,
            expected_annual_return_pct: 6.0,
            current_monthly_contribution_usd: 0.0,
        }
    }

    #[test]
    fn future_value_zero_years() {
        assert_eq!(future_value(10_000.0, 500.0, 6.0, 0.0), 10_000.0);
    }

    #[test]
    fn future_value_known() {
        // $10k × 1.06^10 ≈ $17,908.
        let fv = future_value(10_000.0, 0.0, 6.0, 10.0);
        assert!((fv - 17_908.0).abs() < 5.0);
    }

    #[test]
    fn required_monthly_already_overfunded() {
        // Current grows past target on its own.
        let r = required_monthly(100_000.0, 6.0, 10.0, 50_000.0);
        assert_eq!(r, 0.0);
    }

    #[test]
    fn required_monthly_basic() {
        // Need $100k in 10 years at 6% with $0 start.
        let r = required_monthly(0.0, 6.0, 10.0, 100_000.0);
        // Published value ≈ $610/mo
        assert!(r > 580.0 && r < 650.0, "got {r}");
    }

    #[test]
    fn compute_years_until_basic() {
        let r = compute(&input());
        assert_eq!(r.years_until_college, 13);
    }

    #[test]
    fn compute_per_year_costs_inflate() {
        let r = compute(&input());
        assert_eq!(r.per_year_cost.len(), 4);
        // Each year more expensive than the last.
        for w in r.per_year_cost.windows(2) {
            assert!(w[1].cost_usd > w[0].cost_usd);
        }
        // First year (age 18) = $30k × 1.05^13
        let expected_first = 30_000.0 * 1.05_f64.powi(13);
        assert!((r.per_year_cost[0].cost_usd - expected_first).abs() < 1.0);
    }

    #[test]
    fn compute_total_cost_sums_per_year() {
        let r = compute(&input());
        let sum: f64 = r.per_year_cost.iter().map(|y| y.cost_usd).sum();
        assert!((r.total_projected_cost_usd - sum).abs() < 0.01);
    }

    #[test]
    fn compute_shortfall_zero_when_fully_funded() {
        let mut i = input();
        i.current_529_balance_usd = 1_000_000.0;
        let r = compute(&i);
        assert_eq!(r.shortfall_usd, 0.0);
        assert_eq!(r.required_monthly_contribution_usd, 0.0);
    }

    #[test]
    fn compute_on_track_when_contribution_above_required() {
        let mut i = input();
        i.current_monthly_contribution_usd = 5_000.0;  // way over
        let r = compute(&i);
        assert!(r.on_track);
    }

    #[test]
    fn compute_not_on_track_when_contribution_below_required() {
        let mut i = input();
        i.current_monthly_contribution_usd = 0.0;
        let r = compute(&i);
        // Required > 0 → not on track at zero.
        assert!(!r.on_track);
    }

    #[test]
    fn compute_child_at_college_age_zero_years_until() {
        let mut i = input();
        i.child_age_years = 18;
        let r = compute(&i);
        assert_eq!(r.years_until_college, 0);
    }

    #[test]
    fn compute_zero_inflation_constant_cost_per_year() {
        let mut i = input();
        i.tuition_inflation_pct = 0.0;
        let r = compute(&i);
        for y in &r.per_year_cost {
            assert!((y.cost_usd - 30_000.0).abs() < 1e-6);
        }
        assert_eq!(r.total_projected_cost_usd, 120_000.0);
    }
}
