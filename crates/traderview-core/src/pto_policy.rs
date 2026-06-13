//! Paid-time-off (PTO) accrual policy — states how employees accrue PTO. It
//! computes the annual accrual (rate per pay period × periods per year) in both
//! hours and workdays, applies any carryover cap, and assembles the policy
//! clauses (eligibility, accrual, carryover, usage, separation payout).
//! Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PtoPolicyInput {
    pub company_name: String,
    /// Hours accrued each pay period.
    pub accrual_rate_hours_per_period: f64,
    pub pay_periods_per_year: i32,
    /// Hours in a standard workday (for the hours→days conversion).
    #[serde(default = "default_workday")]
    pub hours_per_workday: f64,
    /// Max hours that carry into the next year (0 = no carryover allowed).
    #[serde(default)]
    pub carryover_cap_hours: f64,
    /// Days of employment before accrual begins.
    #[serde(default)]
    pub waiting_period_days: i64,
    /// Whether unused PTO is paid out at separation.
    #[serde(default)]
    pub payout_on_separation: bool,
    pub state: String,
}

fn default_workday() -> f64 {
    8.0
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PtoPolicy {
    pub title: String,
    pub annual_accrual_hours: f64,
    pub annual_accrual_days: f64,
    pub carryover_cap_hours: f64,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

pub fn generate(i: &PtoPolicyInput) -> PtoPolicy {
    let annual_hours = cents(i.accrual_rate_hours_per_period * i.pay_periods_per_year as f64);
    let workday = if i.hours_per_workday > 0.0 { i.hours_per_workday } else { 8.0 };
    let annual_days = cents(annual_hours / workday);

    let eligibility = if i.waiting_period_days > 0 {
        format!("Employees begin accruing PTO after {} days of employment.", i.waiting_period_days)
    } else {
        "Employees begin accruing PTO from their first day of employment.".to_string()
    };

    let carryover = if i.carryover_cap_hours > 0.0 {
        format!(
            "Employees may carry over up to {:.1} hours of unused PTO into the next year; PTO above the cap is forfeited.",
            i.carryover_cap_hours
        )
    } else {
        "Unused PTO does not carry over to the next year (use-it-or-lose-it), except where state law requires otherwise.".to_string()
    };

    let payout = if i.payout_on_separation {
        "On separation, the Company pays out the employee's accrued, unused PTO.".to_string()
    } else {
        "Accrued PTO is not paid out on separation except where state law requires it.".to_string()
    };

    let clauses = vec![
        DocClause {
            heading: "1. Eligibility".into(),
            body: eligibility,
        },
        DocClause {
            heading: "2. Accrual".into(),
            body: format!(
                "Eligible employees accrue {:.2} hours of PTO per pay period over {} pay periods per year, totaling {:.2} hours ({:.2} workdays) per year.",
                i.accrual_rate_hours_per_period, i.pay_periods_per_year, annual_hours, annual_days
            ),
        },
        DocClause { heading: "3. Carryover".into(), body: carryover },
        DocClause {
            heading: "4. Usage".into(),
            body: "PTO is requested in advance and is subject to manager approval based on business needs. PTO is taken in increments consistent with payroll practice.".into(),
        },
        DocClause { heading: "5. Separation".into(), body: payout },
        DocClause {
            heading: "6. Governing Law".into(),
            body: format!("This policy is administered in accordance with the laws of the State of {}.", i.state),
        },
    ];

    PtoPolicy {
        title: format!("{} — Paid Time Off Policy", i.company_name),
        annual_accrual_hours: annual_hours,
        annual_accrual_days: annual_days,
        carryover_cap_hours: i.carryover_cap_hours,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> PtoPolicyInput {
        PtoPolicyInput {
            company_name: "Acme Inc".into(),
            accrual_rate_hours_per_period: 4.0,
            pay_periods_per_year: 26,
            hours_per_workday: 8.0,
            carryover_cap_hours: 40.0,
            waiting_period_days: 90,
            payout_on_separation: true,
            state: "California".into(),
        }
    }

    #[test]
    fn annual_accrual_hours_and_days() {
        let d = generate(&base());
        // 4 × 26 = 104 hours = 13 workdays.
        assert!(close(d.annual_accrual_hours, 104.0));
        assert!(close(d.annual_accrual_days, 13.0));
    }

    #[test]
    fn accrual_clause_states_totals() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "2. Accrual").unwrap();
        assert!(c.body.contains("104.00 hours"));
        assert!(c.body.contains("13.00 workdays"));
    }

    #[test]
    fn carryover_cap_clause() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "3. Carryover").unwrap();
        assert!(c.body.contains("40.0 hours"));
    }

    #[test]
    fn no_carryover_is_use_it_or_lose_it() {
        let c = generate(&PtoPolicyInput { carryover_cap_hours: 0.0, ..base() })
            .clauses.into_iter().find(|c| c.heading == "3. Carryover").unwrap();
        assert!(c.body.contains("use-it-or-lose-it"));
    }

    #[test]
    fn waiting_period_in_eligibility() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "1. Eligibility").unwrap();
        assert!(c.body.contains("90 days"));
        let none = generate(&PtoPolicyInput { waiting_period_days: 0, ..base() });
        assert!(none.clauses.iter().find(|c| c.heading == "1. Eligibility").unwrap().body.contains("first day"));
    }

    #[test]
    fn payout_toggle() {
        assert!(generate(&base()).clauses.iter().find(|c| c.heading == "5. Separation").unwrap().body.contains("pays out"));
        let no = generate(&PtoPolicyInput { payout_on_separation: false, ..base() });
        assert!(no.clauses.iter().find(|c| c.heading == "5. Separation").unwrap().body.contains("not paid out"));
    }

    #[test]
    fn custom_workday_changes_days() {
        // 104 hours at a 10-hour workday = 10.4 days.
        let d = generate(&PtoPolicyInput { hours_per_workday: 10.0, ..base() });
        assert!(close(d.annual_accrual_days, 10.4));
    }
}
