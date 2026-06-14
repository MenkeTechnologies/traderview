//! Final paycheck / waiting-time penalty — when an employer pays final wages late
//! after a separation, many states impose a penalty equal to the employee's daily
//! wage for each day the payment is late, capped (California Labor Code §203 caps
//! it at 30 days). This computes the daily wage, the capped penalty days, the
//! waiting-time penalty, and the total owed (unpaid final wages plus penalty).
//! No existing generator computes a capped daily-wage penalty. Drafting aid, not
//! legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct FinalPaycheckInput {
    pub employer_name: String,
    pub employee_name: String,
    /// Unpaid final wages owed at separation.
    #[serde(default)]
    pub final_wages_usd: f64,
    /// Employee's hourly rate.
    pub hourly_rate_usd: f64,
    /// Hours in the employee's workday (for the daily wage).
    #[serde(default = "default_hours_per_day")]
    pub hours_per_day: f64,
    /// Calendar days the final payment is late.
    pub days_late: u32,
    /// Maximum penalty days (California §203 is 30).
    #[serde(default = "default_cap_days")]
    pub cap_days: u32,
    pub separation_date: String,
    pub date: String,
    pub state: String,
    #[serde(default)]
    pub statute_citation: String,
}

fn default_hours_per_day() -> f64 {
    8.0
}

fn default_cap_days() -> u32 {
    30
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FinalPaycheck {
    pub title: String,
    /// Hourly rate × hours per day.
    pub daily_wage_usd: f64,
    /// Days late, capped at cap_days.
    pub penalty_days: u32,
    /// True when the cap reduced the penalty days.
    pub cap_applied: bool,
    /// Daily wage × penalty days.
    pub waiting_time_penalty_usd: f64,
    pub final_wages_usd: f64,
    /// Final wages + penalty.
    pub total_owed_usd: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &FinalPaycheckInput) -> FinalPaycheck {
    let daily = cents(i.hourly_rate_usd * i.hours_per_day);
    let penalty_days = i.days_late.min(i.cap_days);
    let cap_applied = i.days_late > i.cap_days;
    let penalty = cents(daily * penalty_days as f64);
    let total = cents(i.final_wages_usd + penalty);

    let cap_line = if cap_applied {
        format!(" The {} days late are capped at the {}-day maximum.", i.days_late, i.cap_days)
    } else {
        String::new()
    };

    let calc_body = format!(
        "The daily wage is {} ({} per hour × {} hours/day). The final payment is {} day(s) late.{} The waiting-time penalty is {} ({} × {} penalty day(s)). Added to unpaid final wages of {}, the total owed is {}.",
        money(daily),
        money(i.hourly_rate_usd),
        i.hours_per_day,
        i.days_late,
        cap_line,
        money(penalty),
        money(daily),
        penalty_days,
        money(i.final_wages_usd),
        money(total)
    );

    let citation = i.statute_citation.trim();
    let governing = if citation.is_empty() {
        format!("The waiting-time penalty is imposed under the wage laws of the State of {}.", i.state)
    } else {
        format!("The waiting-time penalty is imposed under the wage laws of the State of {} ({}).", i.state, citation)
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Employer: {}\nEmployee: {}\nSeparation date: {}\nStatement date: {}",
                i.employer_name, i.employee_name, i.separation_date, i.date
            ),
        },
        DocClause {
            heading: "1. Final Wages".into(),
            body: format!(
                "Unpaid wages owed to the Employee at separation are {}. Final wages are due promptly after separation as required by state law.",
                money(i.final_wages_usd)
            ),
        },
        DocClause { heading: "2. Waiting-Time Penalty".into(), body: calc_body },
        DocClause {
            heading: "3. Total Owed".into(),
            body: format!(
                "The Employer owes the Employee {} in total: {} in unpaid final wages plus {} in waiting-time penalty.",
                money(total), money(i.final_wages_usd), money(penalty)
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

    FinalPaycheck {
        title: "Final Paycheck & Waiting-Time Penalty".into(),
        daily_wage_usd: daily,
        penalty_days,
        cap_applied,
        waiting_time_penalty_usd: penalty,
        final_wages_usd: cents(i.final_wages_usd),
        total_owed_usd: total,
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

    fn base() -> FinalPaycheckInput {
        FinalPaycheckInput {
            employer_name: "Acme Co".into(),
            employee_name: "Pat Former".into(),
            final_wages_usd: 1_500.0,
            hourly_rate_usd: 25.0,
            hours_per_day: 8.0,
            days_late: 10,
            cap_days: 30,
            separation_date: "2026-06-01".into(),
            date: "2026-07-01".into(),
            state: "California".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn penalty_within_cap() {
        let d = generate(&base());
        assert!(close(d.daily_wage_usd, 200.0));
        assert_eq!(d.penalty_days, 10);
        assert!(!d.cap_applied);
        assert!(close(d.waiting_time_penalty_usd, 2_000.0));
        assert!(close(d.total_owed_usd, 3_500.0));
    }

    #[test]
    fn penalty_capped_at_30_days() {
        let d = generate(&FinalPaycheckInput { days_late: 45, ..base() });
        assert_eq!(d.penalty_days, 30);
        assert!(d.cap_applied);
        assert!(close(d.waiting_time_penalty_usd, 6_000.0));
        assert!(close(d.total_owed_usd, 7_500.0));
    }

    #[test]
    fn on_time_no_penalty() {
        let d = generate(&FinalPaycheckInput { days_late: 0, ..base() });
        assert_eq!(d.penalty_days, 0);
        assert!(close(d.waiting_time_penalty_usd, 0.0));
        assert!(close(d.total_owed_usd, 1_500.0));
    }

    #[test]
    fn exactly_at_cap_not_flagged() {
        let d = generate(&FinalPaycheckInput { days_late: 30, ..base() });
        assert_eq!(d.penalty_days, 30);
        assert!(!d.cap_applied);
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&FinalPaycheckInput { statute_citation: "Cal. Lab. Code § 203".into(), ..base() });
        assert_eq!(d.statutory_citation, "Cal. Lab. Code § 203");
        assert!(d.clauses.iter().any(|c| c.body.contains("Cal. Lab. Code § 203")));
    }
}
