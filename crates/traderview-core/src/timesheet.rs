//! Timesheet — records an hourly employee's hours for a pay period and computes
//! gross pay: regular hours at the hourly rate plus overtime hours at the rate
//! times the overtime multiplier (1.5× by default). Assembles the timesheet with
//! a certification and approval line. Drafting aid, not payroll/tax advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct TimesheetInput {
    pub company_name: String,
    pub employee_name: String,
    pub period_start: String,
    pub period_end: String,
    pub regular_hours: f64,
    pub hourly_rate_usd: f64,
    #[serde(default)]
    pub overtime_hours: f64,
    #[serde(default = "default_ot")]
    pub overtime_multiplier: f64,
}

fn default_ot() -> f64 {
    1.5
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Timesheet {
    pub title: String,
    pub total_hours: f64,
    pub regular_pay_usd: f64,
    pub overtime_pay_usd: f64,
    pub gross_pay_usd: f64,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &TimesheetInput) -> Timesheet {
    let mult = if i.overtime_multiplier > 0.0 { i.overtime_multiplier } else { 1.5 };
    let regular_pay = cents(i.regular_hours * i.hourly_rate_usd);
    let overtime_pay = cents(i.overtime_hours * i.hourly_rate_usd * mult);
    let gross_pay = cents(regular_pay + overtime_pay);
    let total_hours = i.regular_hours + i.overtime_hours;

    let ot_line = if i.overtime_hours > 0.0 {
        format!(
            "\n  Overtime: {:.2} h × {} × {:.2} = {}",
            i.overtime_hours, money(i.hourly_rate_usd), mult, money(overtime_pay)
        )
    } else {
        String::new()
    };

    let clauses = vec![
        DocClause {
            heading: "Header".into(),
            body: format!(
                "Employee: {}\nCompany: {}\nPay period: {} to {}",
                i.employee_name, i.company_name, i.period_start, i.period_end
            ),
        },
        DocClause {
            heading: "1. Hours".into(),
            body: format!(
                "Regular hours: {:.2}\nOvertime hours: {:.2}\nTotal hours: {:.2}",
                i.regular_hours, i.overtime_hours, total_hours
            ),
        },
        DocClause {
            heading: "2. Pay".into(),
            body: format!(
                "Regular: {:.2} h × {} = {}{}\nGross pay: {}",
                i.regular_hours, money(i.hourly_rate_usd), money(regular_pay), ot_line, money(gross_pay)
            ),
        },
        DocClause {
            heading: "3. Certification".into(),
            body: "The employee certifies that the hours recorded above are a true and accurate record of time worked during the pay period.".into(),
        },
        DocClause {
            heading: "Approval".into(),
            body: format!(
                "Employee: ____________________  Date: __________\n{}\n\nApproved by: ____________________  Date: __________",
                i.employee_name
            ),
        },
    ];

    Timesheet {
        title: "Timesheet".into(),
        total_hours,
        regular_pay_usd: regular_pay,
        overtime_pay_usd: overtime_pay,
        gross_pay_usd: gross_pay,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> TimesheetInput {
        TimesheetInput {
            company_name: "Acme Inc".into(),
            employee_name: "Lee Hours".into(),
            period_start: "2026-06-01".into(),
            period_end: "2026-06-14".into(),
            regular_hours: 80.0,
            hourly_rate_usd: 25.0,
            overtime_hours: 5.0,
            overtime_multiplier: 1.5,
        }
    }

    #[test]
    fn pay_breakdown() {
        let d = generate(&base());
        assert!(close(d.regular_pay_usd, 2_000.0));
        assert!(close(d.overtime_pay_usd, 187.5));
        assert!(close(d.gross_pay_usd, 2_187.5));
        assert!(close(d.total_hours, 85.0));
    }

    #[test]
    fn no_overtime() {
        let d = generate(&TimesheetInput { overtime_hours: 0.0, ..base() });
        assert!(close(d.overtime_pay_usd, 0.0));
        assert!(close(d.gross_pay_usd, 2_000.0));
        let c = d.clauses.iter().find(|c| c.heading == "2. Pay").unwrap();
        assert!(!c.body.contains("Overtime:"));
    }

    #[test]
    fn double_time_multiplier() {
        let d = generate(&TimesheetInput { overtime_multiplier: 2.0, ..base() });
        // 5 × 25 × 2 = 250.
        assert!(close(d.overtime_pay_usd, 250.0));
        assert!(close(d.gross_pay_usd, 2_250.0));
    }

    #[test]
    fn zero_multiplier_defaults_to_1_5() {
        let d = generate(&TimesheetInput { overtime_multiplier: 0.0, ..base() });
        assert!(close(d.overtime_pay_usd, 187.5));
    }

    #[test]
    fn hours_clause_lists_totals() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "1. Hours").unwrap();
        assert!(c.body.contains("Total hours: 85.00"));
    }
}
