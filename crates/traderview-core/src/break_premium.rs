//! Meal & rest break premium pay — under California Labor Code §226.7, an employer
//! that fails to provide a compliant meal or rest break owes the employee one
//! additional hour of pay at the regular rate for each workday the break is missed
//! (a separate premium for meal and for rest, so up to two hours per day). This
//! computes the meal premium, the rest premium, and the total. No existing
//! generator computes per-violation-day premium pay. Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct BreakPremiumInput {
    pub employer_name: String,
    pub employee_name: String,
    /// Employee's regular rate of pay, per hour.
    pub regular_rate_usd: f64,
    /// Workdays on which a compliant meal break was not provided.
    #[serde(default)]
    pub meal_violation_days: u32,
    /// Workdays on which a compliant rest break was not provided.
    #[serde(default)]
    pub rest_violation_days: u32,
    pub period_label: String,
    pub date: String,
    pub state: String,
    #[serde(default)]
    pub statute_citation: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BreakPremium {
    pub title: String,
    /// One hour of pay × meal violation days.
    pub meal_premium_usd: f64,
    /// One hour of pay × rest violation days.
    pub rest_premium_usd: f64,
    /// Total premium hours owed (one per violation).
    pub total_premium_hours: u32,
    /// Meal premium + rest premium.
    pub total_premium_usd: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &BreakPremiumInput) -> BreakPremium {
    let meal = cents(i.regular_rate_usd * i.meal_violation_days as f64);
    let rest = cents(i.regular_rate_usd * i.rest_violation_days as f64);
    let total = cents(meal + rest);
    let hours = i.meal_violation_days + i.rest_violation_days;

    let calc_body = format!(
        "At a regular rate of {}, each missed break is one hour of premium pay. {} meal-break violation day(s) = {}; {} rest-break violation day(s) = {}. Total premium: {} hour(s) of pay, or {}.",
        money(i.regular_rate_usd),
        i.meal_violation_days,
        money(meal),
        i.rest_violation_days,
        money(rest),
        hours,
        money(total)
    );

    let citation = i.statute_citation.trim();
    let governing = if citation.is_empty() {
        format!("Break premiums are owed under the wage laws of the State of {}.", i.state)
    } else {
        format!("Break premiums are owed under the wage laws of the State of {} ({}).", i.state, citation)
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Employer: {}\nEmployee: {}\nPeriod: {}\nStatement date: {}",
                i.employer_name, i.employee_name, i.period_label, i.date
            ),
        },
        DocClause {
            heading: "1. Premium Pay".into(),
            body: "For each workday a compliant meal or rest break was not provided, the employee is owed one additional hour of pay at the regular rate — separately for meal and for rest breaks.".into(),
        },
        DocClause { heading: "2. Calculation".into(), body: calc_body },
        DocClause {
            heading: "3. Payment".into(),
            body: format!(
                "The Employer owes the Employee {} in break-premium pay for the period, payable with the next wage payment.",
                money(total)
            ),
        },
        DocClause { heading: "4. Governing Law".into(), body: governing },
        DocClause {
            heading: "Certification".into(),
            body: format!(
                "Employer: ____________________  Date: __________\n{}",
                i.employer_name
            ),
        },
    ];

    BreakPremium {
        title: "Meal & Rest Break Premium Statement".into(),
        meal_premium_usd: meal,
        rest_premium_usd: rest,
        total_premium_hours: hours,
        total_premium_usd: total,
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

    fn base() -> BreakPremiumInput {
        BreakPremiumInput {
            employer_name: "Acme Co".into(),
            employee_name: "Pat Worker".into(),
            regular_rate_usd: 20.0,
            meal_violation_days: 10,
            rest_violation_days: 5,
            period_label: "Q2 2026".into(),
            date: "2026-07-01".into(),
            state: "California".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn meal_and_rest_premiums() {
        let d = generate(&base());
        assert!(close(d.meal_premium_usd, 200.0));
        assert!(close(d.rest_premium_usd, 100.0));
        assert_eq!(d.total_premium_hours, 15);
        assert!(close(d.total_premium_usd, 300.0));
    }

    #[test]
    fn no_violations_zero() {
        let d = generate(&BreakPremiumInput { meal_violation_days: 0, rest_violation_days: 0, ..base() });
        assert!(close(d.total_premium_usd, 0.0));
        assert_eq!(d.total_premium_hours, 0);
    }

    #[test]
    fn fractional_rate() {
        let d = generate(&BreakPremiumInput { regular_rate_usd: 33.33, meal_violation_days: 8, rest_violation_days: 8, ..base() });
        assert!(close(d.meal_premium_usd, 266.64));
        assert!(close(d.total_premium_usd, 533.28));
    }

    #[test]
    fn meal_only() {
        let d = generate(&BreakPremiumInput { rest_violation_days: 0, ..base() });
        assert!(close(d.rest_premium_usd, 0.0));
        assert!(close(d.total_premium_usd, 200.0));
        assert_eq!(d.total_premium_hours, 10);
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&BreakPremiumInput { statute_citation: "Cal. Lab. Code § 226.7".into(), ..base() });
        assert_eq!(d.statutory_citation, "Cal. Lab. Code § 226.7");
        assert!(d.clauses.iter().any(|c| c.body.contains("Cal. Lab. Code § 226.7")));
    }
}
