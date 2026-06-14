//! Reporting-time pay — when an employee reports for a scheduled shift but is sent
//! home early (or given less work than scheduled), California law guarantees pay
//! for half the scheduled shift, with a two-hour minimum and a four-hour maximum,
//! at the regular rate. This computes the reporting-time minimum hours, the hours
//! guaranteed, and any additional pay owed beyond the hours actually worked. No
//! existing generator computes a clamped half-shift minimum guarantee. Drafting
//! aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ReportingTimeInput {
    pub employer_name: String,
    pub employee_name: String,
    /// Hours the employee was scheduled to work.
    pub scheduled_hours: f64,
    /// Hours the employee actually worked before being sent home.
    #[serde(default)]
    pub hours_worked: f64,
    /// Regular rate of pay, per hour.
    pub regular_rate_usd: f64,
    /// Minimum reporting-time hours (California is 2).
    #[serde(default = "default_min_hours")]
    pub min_hours: f64,
    /// Maximum reporting-time hours (California is 4).
    #[serde(default = "default_max_hours")]
    pub max_hours: f64,
    pub shift_date: String,
    pub date: String,
    pub state: String,
    #[serde(default)]
    pub statute_citation: String,
}

fn default_min_hours() -> f64 {
    2.0
}

fn default_max_hours() -> f64 {
    4.0
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ReportingTimePay {
    pub title: String,
    /// Half the scheduled shift, clamped to [min, max].
    pub reporting_min_hours: f64,
    /// The greater of hours worked and the reporting-time minimum.
    pub guaranteed_hours: f64,
    /// Pay for the guaranteed hours.
    pub guaranteed_pay_usd: f64,
    /// Additional pay owed beyond the hours actually worked.
    pub additional_owed_usd: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &ReportingTimeInput) -> ReportingTimePay {
    let half = i.scheduled_hours / 2.0;
    // Clamp half the shift to the [min, max] reporting-time window.
    let reporting_min = half.max(i.min_hours).min(i.max_hours);
    let guaranteed_hours = i.hours_worked.max(reporting_min);
    let guaranteed_pay = cents(guaranteed_hours * i.regular_rate_usd);
    let additional_hours = (reporting_min - i.hours_worked).max(0.0);
    let additional = cents(additional_hours * i.regular_rate_usd);

    let calc_body = format!(
        "Half of the {}-hour scheduled shift is {} hours, which falls within the {}–{} hour reporting-time window, so the reporting-time minimum is {} hours. The employee worked {} hours; the greater of the two is guaranteed, for {} hours of pay, or {} at {} per hour. Additional pay owed beyond hours worked is {}.",
        cents(i.scheduled_hours),
        cents(half),
        cents(i.min_hours),
        cents(i.max_hours),
        cents(reporting_min),
        cents(i.hours_worked),
        cents(guaranteed_hours),
        money(guaranteed_pay),
        money(i.regular_rate_usd),
        money(additional)
    );

    let citation = i.statute_citation.trim();
    let governing = if citation.is_empty() {
        format!("Reporting-time pay is owed under the wage laws of the State of {}.", i.state)
    } else {
        format!("Reporting-time pay is owed under the wage laws of the State of {} ({}).", i.state, citation)
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Employer: {}\nEmployee: {}\nShift date: {}\nStatement date: {}",
                i.employer_name, i.employee_name, i.shift_date, i.date
            ),
        },
        DocClause {
            heading: "1. Reporting-Time Pay".into(),
            body: "An employee who reports for a scheduled shift but is furnished less than half the scheduled hours must be paid for half the shift, with a two-hour minimum and a four-hour maximum, at the regular rate.".into(),
        },
        DocClause { heading: "2. Calculation".into(), body: calc_body },
        DocClause {
            heading: "3. Payment".into(),
            body: format!(
                "The employee is guaranteed {} for the shift. Of that, {} is additional reporting-time pay beyond the hours actually worked.",
                money(guaranteed_pay), money(additional)
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

    ReportingTimePay {
        title: "Reporting-Time Pay Statement".into(),
        reporting_min_hours: cents(reporting_min),
        guaranteed_hours: cents(guaranteed_hours),
        guaranteed_pay_usd: guaranteed_pay,
        additional_owed_usd: additional,
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

    fn base() -> ReportingTimeInput {
        ReportingTimeInput {
            employer_name: "Acme Co".into(),
            employee_name: "Pat Worker".into(),
            scheduled_hours: 8.0,
            hours_worked: 1.0,
            regular_rate_usd: 20.0,
            min_hours: 2.0,
            max_hours: 4.0,
            shift_date: "2026-06-20".into(),
            date: "2026-07-01".into(),
            state: "California".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn half_shift_capped_at_four() {
        let d = generate(&base());
        // half of 8 = 4 → min 4 hours; worked 1 → guaranteed 4.
        assert!(close(d.reporting_min_hours, 4.0));
        assert!(close(d.guaranteed_hours, 4.0));
        assert!(close(d.guaranteed_pay_usd, 80.0));
        assert!(close(d.additional_owed_usd, 60.0));
    }

    #[test]
    fn short_shift_floored_at_two() {
        let d = generate(&ReportingTimeInput { scheduled_hours: 3.0, hours_worked: 0.5, ..base() });
        // half of 3 = 1.5 → floored to 2.
        assert!(close(d.reporting_min_hours, 2.0));
        assert!(close(d.additional_owed_usd, 30.0));
    }

    #[test]
    fn long_shift_capped_at_four() {
        let d = generate(&ReportingTimeInput { scheduled_hours: 10.0, hours_worked: 1.0, ..base() });
        // half of 10 = 5 → capped to 4.
        assert!(close(d.reporting_min_hours, 4.0));
        assert!(close(d.additional_owed_usd, 60.0));
    }

    #[test]
    fn worked_beyond_minimum_no_additional() {
        let d = generate(&ReportingTimeInput { hours_worked: 5.0, ..base() });
        assert!(close(d.additional_owed_usd, 0.0));
        assert!(close(d.guaranteed_hours, 5.0));
        assert!(close(d.guaranteed_pay_usd, 100.0));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&ReportingTimeInput { statute_citation: "IWC Wage Order 5".into(), ..base() });
        assert_eq!(d.statutory_citation, "IWC Wage Order 5");
        assert!(d.clauses.iter().any(|c| c.body.contains("IWC Wage Order 5")));
    }
}
