//! Wage converter — hourly ↔ salary across every pay period.
//!
//! Converts an hourly rate to an annual salary (or back) and breaks it out
//! per week, two weeks, month, and year, given the hours worked per week and
//! weeks worked per year (drop weeks for unpaid time off).
//!
//!   * annual hours = hours/week × weeks/year
//!   * hourly → salary: annual = hourly × annual hours
//!   * salary → hourly: hourly = salary / annual hours
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    HourlyToSalary,
    SalaryToHourly,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WageInput {
    pub mode: Mode,
    /// Hourly rate (HourlyToSalary) or annual salary (SalaryToHourly).
    pub amount_usd: f64,
    pub hours_per_week: f64,
    pub weeks_per_year: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct WageResult {
    pub hourly_usd: f64,
    pub weekly_usd: f64,
    pub biweekly_usd: f64,
    pub monthly_usd: f64,
    pub annual_usd: f64,
    pub annual_hours: f64,
}

pub fn analyze(i: &WageInput) -> WageResult {
    let hours_week = i.hours_per_week.max(0.0);
    let weeks = i.weeks_per_year.max(0.0);
    let annual_hours = hours_week * weeks;

    let hourly = match i.mode {
        Mode::HourlyToSalary => i.amount_usd,
        Mode::SalaryToHourly => {
            if annual_hours > 0.0 {
                i.amount_usd / annual_hours
            } else {
                0.0
            }
        }
    };

    let annual = hourly * annual_hours;
    let weekly = hourly * hours_week;

    WageResult {
        hourly_usd: hourly,
        weekly_usd: weekly,
        biweekly_usd: weekly * 2.0,
        monthly_usd: annual / 12.0,
        annual_usd: annual,
        annual_hours,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hourly(rate: f64, h: f64, w: f64) -> WageInput {
        WageInput { mode: Mode::HourlyToSalary, amount_usd: rate, hours_per_week: h, weeks_per_year: w }
    }

    #[test]
    fn hourly_to_annual_full_time() {
        // $30/hr × 2080 = 62,400.
        let r = analyze(&hourly(30.0, 40.0, 52.0));
        assert!((r.annual_hours - 2080.0).abs() < 1e-9);
        assert!((r.annual_usd - 62_400.0).abs() < 1e-6);
    }

    #[test]
    fn monthly_is_annual_over_12() {
        let r = analyze(&hourly(30.0, 40.0, 52.0));
        assert!((r.monthly_usd - 62_400.0 / 12.0).abs() < 1e-6);
    }

    #[test]
    fn weekly_and_biweekly() {
        let r = analyze(&hourly(30.0, 40.0, 52.0));
        assert!((r.weekly_usd - 1_200.0).abs() < 1e-6); // 30 × 40
        assert!((r.biweekly_usd - 2_400.0).abs() < 1e-6);
    }

    #[test]
    fn salary_to_hourly() {
        // 104,000 / 2080 = 50.
        let r = analyze(&WageInput {
            mode: Mode::SalaryToHourly,
            amount_usd: 104_000.0,
            hours_per_week: 40.0,
            weeks_per_year: 52.0,
        });
        assert!((r.hourly_usd - 50.0).abs() < 1e-9);
    }

    #[test]
    fn part_time_halves_annual() {
        let full = analyze(&hourly(30.0, 40.0, 52.0));
        let half = analyze(&hourly(30.0, 20.0, 52.0));
        assert!((half.annual_usd - full.annual_usd / 2.0).abs() < 1e-6);
    }

    #[test]
    fn fewer_weeks_reduces_annual() {
        let full = analyze(&hourly(30.0, 40.0, 52.0));
        let unpaid = analyze(&hourly(30.0, 40.0, 50.0));
        assert!(unpaid.annual_usd < full.annual_usd);
    }

    #[test]
    fn roundtrip_hourly_salary_hourly() {
        let to_salary = analyze(&hourly(42.5, 40.0, 52.0));
        let back = analyze(&WageInput {
            mode: Mode::SalaryToHourly,
            amount_usd: to_salary.annual_usd,
            hours_per_week: 40.0,
            weeks_per_year: 52.0,
        });
        assert!((back.hourly_usd - 42.5).abs() < 1e-9);
    }

    #[test]
    fn zero_hours_salary_to_hourly_guards() {
        let r = analyze(&WageInput {
            mode: Mode::SalaryToHourly,
            amount_usd: 100_000.0,
            hours_per_week: 0.0,
            weeks_per_year: 52.0,
        });
        assert!(r.hourly_usd.abs() < 1e-9);
    }
}
