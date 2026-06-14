//! PTO balance statement — a point-in-time statement of an individual employee's
//! paid-time-off balance, distinct from the PTO policy (which states the accrual
//! rate). It computes hours earned over the periods worked, subtracts hours used,
//! applies any accrual cap, and values the remaining balance for a separation
//! payout. Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PtoBalanceInput {
    pub employer_name: String,
    pub employee_name: String,
    /// Hours accrued each pay period.
    pub accrual_rate_hours_per_period: f64,
    /// Pay periods worked in the measurement window.
    pub periods_worked: f64,
    /// PTO hours used in the window.
    #[serde(default)]
    pub hours_used: f64,
    /// Maximum hours the balance may reach (0 = no cap).
    #[serde(default)]
    pub accrual_cap_hours: f64,
    /// Employee's hourly rate, for the payout value.
    #[serde(default)]
    pub hourly_rate_usd: f64,
    /// Hours in a workday, for the days conversion (usually 8).
    #[serde(default = "default_hours_per_day")]
    pub hours_per_day: f64,
    pub as_of_date: String,
    pub state: String,
    #[serde(default)]
    pub statute_citation: String,
}

fn default_hours_per_day() -> f64 {
    8.0
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PtoBalance {
    pub title: String,
    /// Accrual rate × periods worked.
    pub hours_earned: f64,
    pub hours_used: f64,
    /// Earned − used after applying the cap, floored at 0.
    pub balance_hours: f64,
    /// True when the cap reduced the balance.
    pub cap_applied: bool,
    pub balance_days: f64,
    /// Balance × hourly rate.
    pub payout_value_usd: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &PtoBalanceInput) -> PtoBalance {
    let earned = i.accrual_rate_hours_per_period * i.periods_worked;
    let net = earned - i.hours_used;
    let (capped, cap_applied) = if i.accrual_cap_hours > 0.0 && net > i.accrual_cap_hours {
        (i.accrual_cap_hours, true)
    } else {
        (net, false)
    };
    let balance = capped.max(0.0);
    let balance_days = if i.hours_per_day > 0.0 {
        cents(balance / i.hours_per_day)
    } else {
        0.0
    };
    let payout = cents(balance * i.hourly_rate_usd);

    let cap_line = if cap_applied {
        format!(" The balance is capped at {} hours.", cents(i.accrual_cap_hours))
    } else {
        String::new()
    };

    let calc_body = format!(
        "Earned {} hours ({} hours/period × {} periods) less {} hours used leaves a balance of {} hours ({} workdays).{} At {} per hour, the payout value is {}.",
        cents(earned),
        cents(i.accrual_rate_hours_per_period),
        cents(i.periods_worked),
        cents(i.hours_used),
        cents(balance),
        balance_days,
        cap_line,
        money(i.hourly_rate_usd),
        money(payout)
    );

    let citation = i.statute_citation.trim();
    let governing = if citation.is_empty() {
        format!("PTO is administered under the employer's policy and the laws of the State of {}.", i.state)
    } else {
        format!("PTO is administered under the employer's policy and the laws of the State of {} ({}).", i.state, citation)
    };

    let clauses = vec![
        DocClause {
            heading: "Statement".into(),
            body: format!(
                "Employer: {}\nEmployee: {}\nAs of: {}",
                i.employer_name, i.employee_name, i.as_of_date
            ),
        },
        DocClause { heading: "1. Balance".into(), body: calc_body },
        DocClause {
            heading: "2. Payout".into(),
            body: format!(
                "If paid out, the {} remaining hours are valued at {} per hour for a total of {}, subject to the employer's policy and applicable law on separation.",
                cents(balance), money(i.hourly_rate_usd), money(payout)
            ),
        },
        DocClause { heading: "3. Governing Law".into(), body: governing },
        DocClause {
            heading: "Acknowledgement".into(),
            body: format!(
                "Employee: ____________________  Date: __________\n{}",
                i.employee_name
            ),
        },
    ];

    PtoBalance {
        title: "PTO Balance Statement".into(),
        hours_earned: cents(earned),
        hours_used: cents(i.hours_used),
        balance_hours: cents(balance),
        cap_applied,
        balance_days,
        payout_value_usd: payout,
        statutory_citation: citation.to_string(),
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> PtoBalanceInput {
        PtoBalanceInput {
            employer_name: "Acme Co".into(),
            employee_name: "Sam Worker".into(),
            accrual_rate_hours_per_period: 5.0,
            periods_worked: 20.0,
            hours_used: 30.0,
            accrual_cap_hours: 80.0,
            hourly_rate_usd: 25.0,
            hours_per_day: 8.0,
            as_of_date: "2026-07-01".into(),
            state: "Delaware".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn balance_and_payout() {
        let d = generate(&base());
        assert!(close(d.hours_earned, 100.0));
        assert!(close(d.balance_hours, 70.0));
        assert!(!d.cap_applied);
        assert!(close(d.balance_days, 8.75));
        assert!(close(d.payout_value_usd, 1_750.0));
    }

    #[test]
    fn cap_limits_balance() {
        let d = generate(&PtoBalanceInput { hours_used: 10.0, ..base() });
        // net 90 → capped to 80.
        assert!(d.cap_applied);
        assert!(close(d.balance_hours, 80.0));
        assert!(close(d.balance_days, 10.0));
        assert!(close(d.payout_value_usd, 2_000.0));
    }

    #[test]
    fn no_cap_uses_net() {
        let d = generate(&PtoBalanceInput { accrual_cap_hours: 0.0, ..base() });
        assert!(!d.cap_applied);
        assert!(close(d.balance_hours, 70.0));
    }

    #[test]
    fn overused_floors_at_zero() {
        let d = generate(&PtoBalanceInput { periods_worked: 5.0, hours_used: 40.0, ..base() });
        // earned 25, used 40 → net negative → 0.
        assert!(close(d.balance_hours, 0.0));
        assert!(close(d.payout_value_usd, 0.0));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&PtoBalanceInput { statute_citation: "Cal. Lab. Code § 227.3".into(), ..base() });
        assert_eq!(d.statutory_citation, "Cal. Lab. Code § 227.3");
        assert!(d.clauses.iter().any(|c| c.body.contains("Cal. Lab. Code § 227.3")));
    }
}
