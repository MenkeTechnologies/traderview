//! Pay stub — an earnings statement for one pay period. It computes the
//! employee FICA withholding automatically (Social Security 6.2% + Medicare
//! 1.45% of gross) and combines it with the income-tax withholding and any
//! other deductions to produce total deductions and net pay, then assembles the
//! stub. Drafting aid, not payroll/tax advice — FICA ignores the annual wage
//! base and the Additional Medicare surtax.

use serde::{Deserialize, Serialize};

/// Employee Social Security rate.
pub const SOCIAL_SECURITY_RATE: f64 = 0.062;
/// Employee Medicare rate.
pub const MEDICARE_RATE: f64 = 0.0145;

#[derive(Debug, Clone, Deserialize)]
pub struct PayStubInput {
    pub company_name: String,
    pub employee_name: String,
    pub pay_date: String,
    pub period_start: String,
    pub period_end: String,
    pub gross_pay_usd: f64,
    #[serde(default)]
    pub federal_withholding_usd: f64,
    #[serde(default)]
    pub state_withholding_usd: f64,
    /// Other deductions (401k, health premiums, etc.).
    #[serde(default)]
    pub other_deductions_usd: f64,
    /// Optional year-to-date gross for the stub's YTD line.
    #[serde(default)]
    pub ytd_gross_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PayStub {
    pub title: String,
    pub gross_pay_usd: f64,
    pub social_security_usd: f64,
    pub medicare_usd: f64,
    pub federal_withholding_usd: f64,
    pub state_withholding_usd: f64,
    pub other_deductions_usd: f64,
    pub total_deductions_usd: f64,
    pub net_pay_usd: f64,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &PayStubInput) -> PayStub {
    let ss = cents(i.gross_pay_usd * SOCIAL_SECURITY_RATE);
    let medicare = cents(i.gross_pay_usd * MEDICARE_RATE);
    let total_deductions = cents(
        ss + medicare + i.federal_withholding_usd + i.state_withholding_usd + i.other_deductions_usd,
    );
    let net_pay = cents(i.gross_pay_usd - total_deductions);

    let mut deduction_lines = vec![
        format!("  Federal withholding: {}", money(i.federal_withholding_usd)),
        format!("  State withholding: {}", money(i.state_withholding_usd)),
        format!("  Social Security (6.2%): {}", money(ss)),
        format!("  Medicare (1.45%): {}", money(medicare)),
    ];
    if i.other_deductions_usd != 0.0 {
        deduction_lines.push(format!("  Other deductions: {}", money(i.other_deductions_usd)));
    }
    deduction_lines.push(format!("Total deductions: {}", money(total_deductions)));

    let ytd_line = if i.ytd_gross_usd > 0.0 {
        format!("\nYear-to-date gross: {}", money(i.ytd_gross_usd))
    } else {
        String::new()
    };

    let clauses = vec![
        DocClause {
            heading: "Header".into(),
            body: format!(
                "Employer: {}\nEmployee: {}\nPay date: {}\nPay period: {} to {}",
                i.company_name, i.employee_name, i.pay_date, i.period_start, i.period_end
            ),
        },
        DocClause {
            heading: "1. Earnings".into(),
            body: format!("Gross pay: {}{}", money(i.gross_pay_usd), ytd_line),
        },
        DocClause {
            heading: "2. Deductions".into(),
            body: deduction_lines.join("\n"),
        },
        DocClause {
            heading: "3. Net Pay".into(),
            body: format!("Net pay (take-home): {}", money(net_pay)),
        },
    ];

    PayStub {
        title: "Pay Stub / Earnings Statement".into(),
        gross_pay_usd: i.gross_pay_usd,
        social_security_usd: ss,
        medicare_usd: medicare,
        federal_withholding_usd: i.federal_withholding_usd,
        state_withholding_usd: i.state_withholding_usd,
        other_deductions_usd: i.other_deductions_usd,
        total_deductions_usd: total_deductions,
        net_pay_usd: net_pay,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> PayStubInput {
        PayStubInput {
            company_name: "Acme Inc".into(),
            employee_name: "Morgan Pay".into(),
            pay_date: "2026-06-15".into(),
            period_start: "2026-06-01".into(),
            period_end: "2026-06-14".into(),
            gross_pay_usd: 5_000.0,
            federal_withholding_usd: 600.0,
            state_withholding_usd: 200.0,
            other_deductions_usd: 250.0,
            ytd_gross_usd: 60_000.0,
        }
    }

    #[test]
    fn fica_auto_computed() {
        let d = generate(&base());
        assert!(close(d.social_security_usd, 310.0));
        assert!(close(d.medicare_usd, 72.5));
    }

    #[test]
    fn total_deductions_and_net() {
        let d = generate(&base());
        assert!(close(d.total_deductions_usd, 1_432.5));
        assert!(close(d.net_pay_usd, 3_567.5));
    }

    #[test]
    fn deduction_lines_present() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "2. Deductions").unwrap();
        assert!(c.body.contains("Social Security (6.2%): $310.00"));
        assert!(c.body.contains("Medicare (1.45%): $72.50"));
        assert!(c.body.contains("Other deductions: $250.00"));
        assert!(c.body.contains("Total deductions: $1432.50"));
    }

    #[test]
    fn no_other_deductions_omits_line() {
        let d = generate(&PayStubInput { other_deductions_usd: 0.0, ..base() });
        let c = d.clauses.iter().find(|c| c.heading == "2. Deductions").unwrap();
        assert!(!c.body.contains("Other deductions"));
        // 600 + 200 + 310 + 72.5 = 1182.5; net 3817.5.
        assert!(close(d.total_deductions_usd, 1_182.5));
        assert!(close(d.net_pay_usd, 3_817.5));
    }

    #[test]
    fn ytd_line_when_provided() {
        assert!(generate(&base()).clauses.iter().find(|c| c.heading == "1. Earnings").unwrap().body.contains("Year-to-date gross: $60000.00"));
        let no = generate(&PayStubInput { ytd_gross_usd: 0.0, ..base() });
        assert!(!no.clauses.iter().find(|c| c.heading == "1. Earnings").unwrap().body.contains("Year-to-date"));
    }

    #[test]
    fn net_pay_in_clause() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "3. Net Pay").unwrap();
        assert!(c.body.contains("$3567.50"));
    }
}
