//! FIRE (Financial Independence / Retire Early) goal calculator.
//!
//! Standard projection: given current portfolio, monthly contribution,
//! expected return, and a target — compute years-to-target, required
//! savings to hit by a given date, year-by-year projection table,
//! withdrawal-rate sustainability per Trinity Study (4% rule), and a
//! sensitivity table.
//!
//! All return inputs are nominal annual %; the user can subtract
//! expected inflation manually for a real-return calc.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct FireInput {
    pub current_portfolio_usd: f64,
    pub monthly_contribution_usd: f64,
    pub expected_annual_return_pct: f64,
    pub target_net_worth_usd: f64,
    pub current_age: u32,
    pub target_retirement_age: u32,
    /// 4.0 default per the original Trinity Study (Bengen 1994).
    #[serde(default = "default_withdrawal_rate")]
    pub safe_withdrawal_rate_pct: f64,
}

fn default_withdrawal_rate() -> f64 {
    4.0
}

#[derive(Debug, Clone, Serialize)]
pub struct YearProjection {
    pub age: u32,
    pub year_index: u32,
    pub start_balance_usd: f64,
    pub contributions_usd: f64,
    pub growth_usd: f64,
    pub end_balance_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SensitivityCell {
    pub return_pct_delta: f64,
    pub contribution_pct_delta: f64,
    pub years_to_target: Option<f64>,
    pub final_nw_at_target_age: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FireReport {
    pub input: FireInputEcho,
    pub years_to_target: Option<f64>,
    pub final_net_worth_at_target_age: f64,
    pub required_monthly_savings_for_target_date: Option<f64>,
    pub safe_withdrawal_income_annual_usd: f64,
    pub yearly_projection: Vec<YearProjection>,
    pub sensitivity: Vec<SensitivityCell>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FireInputEcho {
    pub current_portfolio_usd: f64,
    pub monthly_contribution_usd: f64,
    pub expected_annual_return_pct: f64,
    pub target_net_worth_usd: f64,
    pub current_age: u32,
    pub target_retirement_age: u32,
    pub safe_withdrawal_rate_pct: f64,
}

// ─── Pure compute ──────────────────────────────────────────────────────────

/// Future value of a portfolio with monthly contributions over N years.
///   FV = current × (1 + r)^n + monthly × ((1 + r/12)^(12n) - 1) / (r/12)
/// Uses monthly compounding for the contribution annuity, annual for
/// the lump-sum portion.
pub fn future_value(
    current_portfolio: f64,
    monthly_contribution: f64,
    annual_return_pct: f64,
    years: f64,
) -> f64 {
    if years <= 0.0 {
        return current_portfolio;
    }
    let r = annual_return_pct / 100.0;
    let monthly_r = r / 12.0;
    let months = years * 12.0;
    let lump_growth = current_portfolio * (1.0 + r).powf(years);
    let annuity_growth = if monthly_r.abs() < 1e-12 {
        monthly_contribution * months
    } else {
        monthly_contribution * ((1.0 + monthly_r).powf(months) - 1.0) / monthly_r
    };
    lump_growth + annuity_growth
}

/// Years required to grow `current` + `monthly` contributions at `annual_return`
/// up to `target`. Uses iterative search (5,000-step cap @ monthly resolution).
/// Returns `None` if the target is never reached within 100 years (e.g.
/// contributions + return aren't enough).
pub fn years_to_reach_target(
    current: f64,
    monthly: f64,
    annual_return_pct: f64,
    target: f64,
) -> Option<f64> {
    if current >= target {
        return Some(0.0);
    }
    if monthly <= 0.0 && annual_return_pct <= 0.0 {
        return None;
    }
    let mut years = 0.0_f64;
    let step = 1.0 / 12.0; // monthly granularity
    while years < 100.0 {
        years += step;
        let fv = future_value(current, monthly, annual_return_pct, years);
        if fv >= target {
            return Some(years);
        }
    }
    None
}

/// Required monthly contribution to hit `target` in `years` given
/// `current` + `annual_return`. Solves the annuity equation backward.
pub fn required_monthly_savings(
    current: f64,
    annual_return_pct: f64,
    years: f64,
    target: f64,
) -> Option<f64> {
    if years <= 0.0 {
        return None;
    }
    let r = annual_return_pct / 100.0;
    let monthly_r = r / 12.0;
    let months = years * 12.0;
    let lump_growth = current * (1.0 + r).powf(years);
    let needed_from_contributions = target - lump_growth;
    if needed_from_contributions <= 0.0 {
        return Some(0.0);
    }
    if monthly_r.abs() < 1e-12 {
        return Some(needed_from_contributions / months);
    }
    let annuity_factor = ((1.0 + monthly_r).powf(months) - 1.0) / monthly_r;
    if annuity_factor <= 0.0 {
        return None;
    }
    Some(needed_from_contributions / annuity_factor)
}

/// Year-by-year projection. Each row reports the start balance, that
/// year's contributions, growth, and end balance. Useful for charting.
pub fn yearly_projection(
    current: f64,
    monthly_contribution: f64,
    annual_return_pct: f64,
    current_age: u32,
    target_age: u32,
) -> Vec<YearProjection> {
    if target_age <= current_age {
        return Vec::new();
    }
    let years = target_age - current_age;
    let mut out: Vec<YearProjection> = Vec::with_capacity(years as usize);
    let mut balance = current;
    let r = annual_return_pct / 100.0;
    let annual_contributions = monthly_contribution * 12.0;
    for i in 0..years {
        let start_balance = balance;
        let growth = balance * r;
        balance = balance * (1.0 + r) + annual_contributions;
        out.push(YearProjection {
            age: current_age + i + 1,
            year_index: i + 1,
            start_balance_usd: start_balance,
            contributions_usd: annual_contributions,
            growth_usd: growth,
            end_balance_usd: balance,
        });
    }
    out
}

/// Trinity Study safe-withdrawal income: target NW × withdrawal_rate / 100.
pub fn safe_withdrawal_income(net_worth: f64, withdrawal_rate_pct: f64) -> f64 {
    if net_worth <= 0.0 || withdrawal_rate_pct <= 0.0 {
        return 0.0;
    }
    net_worth * withdrawal_rate_pct / 100.0
}

/// Build the full report.
pub fn compute(input: &FireInput) -> FireReport {
    let years_to_retirement = if input.target_retirement_age > input.current_age {
        (input.target_retirement_age - input.current_age) as f64
    } else {
        0.0
    };
    let final_nw = future_value(
        input.current_portfolio_usd,
        input.monthly_contribution_usd,
        input.expected_annual_return_pct,
        years_to_retirement,
    );
    let years_to_target = years_to_reach_target(
        input.current_portfolio_usd,
        input.monthly_contribution_usd,
        input.expected_annual_return_pct,
        input.target_net_worth_usd,
    );
    let required_monthly = required_monthly_savings(
        input.current_portfolio_usd,
        input.expected_annual_return_pct,
        years_to_retirement,
        input.target_net_worth_usd,
    );
    let income = safe_withdrawal_income(input.target_net_worth_usd, input.safe_withdrawal_rate_pct);
    let projection = yearly_projection(
        input.current_portfolio_usd,
        input.monthly_contribution_usd,
        input.expected_annual_return_pct,
        input.current_age,
        input.target_retirement_age,
    );
    let mut sensitivity: Vec<SensitivityCell> = Vec::new();
    for ret_delta in &[-2.0, 0.0, 2.0] {
        for contrib_delta in &[-20.0, 0.0, 20.0] {
            let r = input.expected_annual_return_pct + ret_delta;
            let c = input.monthly_contribution_usd * (1.0 + contrib_delta / 100.0);
            let yrs = years_to_reach_target(
                input.current_portfolio_usd,
                c,
                r,
                input.target_net_worth_usd,
            );
            let fv = future_value(input.current_portfolio_usd, c, r, years_to_retirement);
            sensitivity.push(SensitivityCell {
                return_pct_delta: *ret_delta,
                contribution_pct_delta: *contrib_delta,
                years_to_target: yrs,
                final_nw_at_target_age: fv,
            });
        }
    }
    FireReport {
        input: FireInputEcho {
            current_portfolio_usd: input.current_portfolio_usd,
            monthly_contribution_usd: input.monthly_contribution_usd,
            expected_annual_return_pct: input.expected_annual_return_pct,
            target_net_worth_usd: input.target_net_worth_usd,
            current_age: input.current_age,
            target_retirement_age: input.target_retirement_age,
            safe_withdrawal_rate_pct: input.safe_withdrawal_rate_pct,
        },
        years_to_target,
        final_net_worth_at_target_age: final_nw,
        required_monthly_savings_for_target_date: required_monthly,
        safe_withdrawal_income_annual_usd: income,
        yearly_projection: projection,
        sensitivity,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn future_value_zero_years_returns_current() {
        assert_eq!(future_value(100_000.0, 500.0, 7.0, 0.0), 100_000.0);
    }

    #[test]
    fn future_value_compounds_lump_sum() {
        // $100k × 1.07^10 ≈ $196,715
        let fv = future_value(100_000.0, 0.0, 7.0, 10.0);
        assert!((fv - 196_715.0).abs() < 100.0, "fv = {fv}");
    }

    #[test]
    fn future_value_zero_return_linear_contributions() {
        // Zero return + $500/month × 12 months × 10 years = $60,000 plus current.
        let fv = future_value(0.0, 500.0, 0.0, 10.0);
        assert!((fv - 60_000.0).abs() < 1.0);
    }

    #[test]
    fn years_to_reach_target_already_above() {
        assert_eq!(
            years_to_reach_target(1_000_000.0, 0.0, 7.0, 100_000.0),
            Some(0.0)
        );
    }

    #[test]
    fn years_to_reach_target_finds_finite() {
        // $100k start + $500/m at 7% to reach $1M.
        let y = years_to_reach_target(100_000.0, 500.0, 7.0, 1_000_000.0).unwrap();
        assert!(y > 20.0 && y < 35.0, "expected ~25-30y, got {y}");
    }

    #[test]
    fn years_to_reach_target_none_when_impossible() {
        // No contributions + zero return + low start → never reaches high target.
        let y = years_to_reach_target(100.0, 0.0, 0.0, 1_000_000.0);
        assert!(y.is_none());
    }

    #[test]
    fn required_monthly_savings_basic() {
        // To grow $0 to $1M in 30 years at 7%, requires ~$815/m
        let m = required_monthly_savings(0.0, 7.0, 30.0, 1_000_000.0).unwrap();
        assert!(m > 700.0 && m < 900.0, "expected ~$815/m, got {m}");
    }

    #[test]
    fn required_monthly_savings_zero_when_already_funded() {
        // Current grows to target on its own.
        let m = required_monthly_savings(1_000_000.0, 7.0, 10.0, 1_500_000.0).unwrap();
        assert_eq!(m, 0.0);
    }

    #[test]
    fn yearly_projection_correct_year_count() {
        let p = yearly_projection(100_000.0, 500.0, 7.0, 30, 65);
        assert_eq!(p.len(), 35);
        assert_eq!(p[0].age, 31);
        assert_eq!(p.last().unwrap().age, 65);
    }

    #[test]
    fn yearly_projection_empty_when_target_age_below_current() {
        assert!(yearly_projection(100_000.0, 500.0, 7.0, 50, 40).is_empty());
    }

    #[test]
    fn safe_withdrawal_income_4pct_of_million_is_40k() {
        assert!((safe_withdrawal_income(1_000_000.0, 4.0) - 40_000.0).abs() < 1e-9);
    }

    #[test]
    fn safe_withdrawal_income_zero_on_invalid_inputs() {
        assert_eq!(safe_withdrawal_income(0.0, 4.0), 0.0);
        assert_eq!(safe_withdrawal_income(1_000_000.0, 0.0), 0.0);
        assert_eq!(safe_withdrawal_income(-100.0, 4.0), 0.0);
    }

    #[test]
    fn compute_full_report_structure() {
        let r = compute(&FireInput {
            current_portfolio_usd: 100_000.0,
            monthly_contribution_usd: 500.0,
            expected_annual_return_pct: 7.0,
            target_net_worth_usd: 1_000_000.0,
            current_age: 30,
            target_retirement_age: 65,
            safe_withdrawal_rate_pct: 4.0,
        });
        assert!(r.years_to_target.is_some());
        assert!(r.final_net_worth_at_target_age > 1_000_000.0);
        // Should have 3 × 3 = 9 sensitivity cells.
        assert_eq!(r.sensitivity.len(), 9);
        // 35 year-by-year projection rows.
        assert_eq!(r.yearly_projection.len(), 35);
        // $40k/y safe withdrawal on a $1M target at 4%.
        assert!((r.safe_withdrawal_income_annual_usd - 40_000.0).abs() < 1e-9);
    }
}
