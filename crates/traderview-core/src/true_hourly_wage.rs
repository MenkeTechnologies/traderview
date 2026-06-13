//! True hourly wage ("Your Money or Your Life") — what a job really pays once
//! job-related costs come out of pay and job-related time (commute, prep,
//! decompression) is added to the hours.
//!
//! ```text
//! net annual  = salary − taxes − work expenses
//! real hours  = (work + commute + extra) per week × weeks
//! true hourly = net annual / real hours
//! ```
//!
//! The nominal hourly (salary ÷ paid hours) almost always overstates it.

use serde::{Deserialize, Serialize};

fn d_weeks() -> f64 {
    50.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrueWageInput {
    pub gross_annual_salary_usd: f64,
    #[serde(default = "d_weeks")]
    pub weeks_worked: f64,
    pub weekly_work_hours: f64,
    /// Commute hours per week.
    #[serde(default)]
    pub weekly_commute_hours: f64,
    /// Extra job-related hours per week (prep, decompression, on-call).
    #[serde(default)]
    pub weekly_extra_hours: f64,
    /// Annual taxes on the salary.
    #[serde(default)]
    pub annual_taxes_usd: f64,
    /// Annual job-related expenses (commuting cost, work clothes, meals, etc.).
    #[serde(default)]
    pub annual_work_expenses_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TrueWageResult {
    /// Salary − taxes − work expenses.
    pub net_annual_usd: f64,
    /// Paid hours per year (work hours × weeks).
    pub nominal_annual_hours: f64,
    /// All job-related hours per year (work + commute + extra × weeks).
    pub total_annual_hours: f64,
    /// Salary / nominal hours.
    pub nominal_hourly_usd: f64,
    /// Net annual / total job-related hours.
    pub true_hourly_usd: f64,
    /// How much the true wage falls short of the nominal, percent.
    pub erosion_pct: f64,
}

pub fn analyze(input: &TrueWageInput) -> TrueWageResult {
    let weeks = input.weeks_worked.max(0.0);
    let net = input.gross_annual_salary_usd - input.annual_taxes_usd - input.annual_work_expenses_usd;

    let nominal_hours = input.weekly_work_hours * weeks;
    let total_hours = (input.weekly_work_hours + input.weekly_commute_hours + input.weekly_extra_hours) * weeks;

    let nominal_hourly = if nominal_hours > 0.0 {
        input.gross_annual_salary_usd / nominal_hours
    } else {
        0.0
    };
    let true_hourly = if total_hours > 0.0 {
        net / total_hours
    } else {
        0.0
    };
    let erosion = if nominal_hourly > 0.0 {
        (1.0 - true_hourly / nominal_hourly) * 100.0
    } else {
        0.0
    };

    TrueWageResult {
        net_annual_usd: net,
        nominal_annual_hours: nominal_hours,
        total_annual_hours: total_hours,
        nominal_hourly_usd: nominal_hourly,
        true_hourly_usd: true_hourly,
        erosion_pct: erosion,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn base() -> TrueWageInput {
        TrueWageInput {
            gross_annual_salary_usd: 60_000.0,
            weeks_worked: 50.0,
            weekly_work_hours: 40.0,
            weekly_commute_hours: 5.0,
            weekly_extra_hours: 0.0,
            annual_taxes_usd: 12_000.0,
            annual_work_expenses_usd: 3_000.0,
        }
    }

    #[test]
    fn net_annual() {
        assert!(close(analyze(&base()).net_annual_usd, 45_000.0));
    }

    #[test]
    fn nominal_hourly() {
        // 60,000 / (40 × 50) = 30.
        assert!(close(analyze(&base()).nominal_hourly_usd, 30.0));
    }

    #[test]
    fn true_hourly() {
        // 45,000 / (45 × 50 = 2,250) = 20.
        assert!(close(analyze(&base()).true_hourly_usd, 20.0));
    }

    #[test]
    fn erosion() {
        // 1 − 20/30 = 33.33%.
        assert!(close(analyze(&base()).erosion_pct, 100.0 / 3.0));
    }

    #[test]
    fn hours_breakdown() {
        let r = analyze(&base());
        assert!(close(r.nominal_annual_hours, 2_000.0));
        assert!(close(r.total_annual_hours, 2_250.0));
    }

    #[test]
    fn no_costs_no_commute_equals_nominal() {
        let r = analyze(&TrueWageInput {
            weekly_commute_hours: 0.0,
            annual_taxes_usd: 0.0,
            annual_work_expenses_usd: 0.0,
            ..base()
        });
        assert!(close(r.true_hourly_usd, r.nominal_hourly_usd));
        assert!(close(r.erosion_pct, 0.0));
    }

    #[test]
    fn longer_commute_lowers_true_wage() {
        let short = analyze(&base());
        let long = analyze(&TrueWageInput {
            weekly_commute_hours: 15.0,
            ..base()
        });
        assert!(long.true_hourly_usd < short.true_hourly_usd);
    }

    #[test]
    fn expenses_lower_net_and_wage() {
        let more = analyze(&TrueWageInput {
            annual_work_expenses_usd: 10_000.0,
            ..base()
        });
        assert!(more.true_hourly_usd < analyze(&base()).true_hourly_usd);
    }
}
