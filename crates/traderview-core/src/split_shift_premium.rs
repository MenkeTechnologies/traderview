//! Split-shift premium — when an employer schedules a work day with an unpaid,
//! non-meal gap (a "split shift"), California law owes the employee one additional
//! hour of pay at the minimum wage. That premium is offset by the amount by which
//! the employee's earnings for the day already exceed the minimum wage: premium =
//! max(minimum wage − (regular rate − minimum wage) × hours worked, 0). This
//! computes the one-hour premium, the offset, and the net premium owed. No existing
//! generator computes this minimum-wage offset. Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SplitShiftInput {
    pub employer_name: String,
    pub employee_name: String,
    /// Applicable minimum wage, per hour.
    pub min_wage_usd: f64,
    /// Employee's regular rate of pay, per hour.
    pub regular_rate_usd: f64,
    /// Hours actually worked on the split-shift day.
    pub hours_worked: f64,
    pub shift_date: String,
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
pub struct SplitShiftPremium {
    pub title: String,
    /// One hour of pay at the minimum wage.
    pub one_hour_premium_usd: f64,
    /// Earnings above the minimum wage for the hours worked.
    pub earnings_above_min_usd: f64,
    /// One-hour premium less the offset, floored at 0.
    pub net_premium_usd: f64,
    /// True when the offset fully eliminated the premium.
    pub fully_offset: bool,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &SplitShiftInput) -> SplitShiftPremium {
    let one_hour = cents(i.min_wage_usd);
    let above_min = cents((i.regular_rate_usd - i.min_wage_usd).max(0.0) * i.hours_worked);
    let net = cents((one_hour - above_min).max(0.0));
    let fully_offset = net <= 0.0;

    let calc_body = format!(
        "The split-shift premium is one hour at the minimum wage of {} = {}. The employee's earnings above the minimum wage are {} (({} − {}) × {} hours), which offsets the premium. The net split-shift premium owed is {}.",
        money(i.min_wage_usd),
        money(one_hour),
        money(above_min),
        money(i.regular_rate_usd),
        money(i.min_wage_usd),
        i.hours_worked,
        money(net)
    );

    let citation = i.statute_citation.trim();
    let governing = if citation.is_empty() {
        format!("The split-shift premium is owed under the wage laws of the State of {}.", i.state)
    } else {
        format!("The split-shift premium is owed under the wage laws of the State of {} ({}).", i.state, citation)
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
            heading: "1. Split Shift".into(),
            body: "A split shift is a schedule interrupted by an unpaid, non-meal period. For each split-shift day, the employee is owed one additional hour of pay at the minimum wage, reduced by earnings already above the minimum wage that day.".into(),
        },
        DocClause { heading: "2. Calculation".into(), body: calc_body },
        DocClause {
            heading: "3. Payment".into(),
            body: if fully_offset {
                "No split-shift premium is owed: the employee's earnings above the minimum wage fully offset the one-hour premium.".to_string()
            } else {
                format!("The employer owes the employee a split-shift premium of {} for the shift.", money(net))
            },
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

    SplitShiftPremium {
        title: "Split-Shift Premium Statement".into(),
        one_hour_premium_usd: one_hour,
        earnings_above_min_usd: above_min,
        net_premium_usd: net,
        fully_offset,
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

    fn base() -> SplitShiftInput {
        SplitShiftInput {
            employer_name: "Acme Co".into(),
            employee_name: "Pat Worker".into(),
            min_wage_usd: 16.0,
            regular_rate_usd: 16.0,
            hours_worked: 8.0,
            shift_date: "2026-06-20".into(),
            date: "2026-07-01".into(),
            state: "California".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn full_premium_at_minimum_wage() {
        let d = generate(&base());
        assert!(close(d.one_hour_premium_usd, 16.0));
        assert!(close(d.earnings_above_min_usd, 0.0));
        assert!(close(d.net_premium_usd, 16.0));
        assert!(!d.fully_offset);
    }

    #[test]
    fn fully_offset_above_minimum() {
        let d = generate(&SplitShiftInput { regular_rate_usd: 18.0, ..base() });
        // (18−16) × 8 = 16 ≥ 16 → offset fully.
        assert!(close(d.earnings_above_min_usd, 16.0));
        assert!(close(d.net_premium_usd, 0.0));
        assert!(d.fully_offset);
    }

    #[test]
    fn partial_offset() {
        let d = generate(&SplitShiftInput { regular_rate_usd: 17.0, ..base() });
        // (17−16) × 8 = 8 → premium 16 − 8 = 8.
        assert!(close(d.earnings_above_min_usd, 8.0));
        assert!(close(d.net_premium_usd, 8.0));
        assert!(!d.fully_offset);
    }

    #[test]
    fn fewer_hours_less_offset() {
        let d = generate(&SplitShiftInput { regular_rate_usd: 20.0, hours_worked: 4.0, ..base() });
        // (20−16) × 4 = 16 → fully offset.
        assert!(close(d.net_premium_usd, 0.0));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&SplitShiftInput { statute_citation: "Cal. Code Regs. tit. 8 § 11040".into(), ..base() });
        assert_eq!(d.statutory_citation, "Cal. Code Regs. tit. 8 § 11040");
        assert!(d.clauses.iter().any(|c| c.body.contains("Cal. Code Regs. tit. 8 § 11040")));
    }
}
