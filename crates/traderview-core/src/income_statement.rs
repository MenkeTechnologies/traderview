//! Income statement (profit & loss) — the standard multi-step business P&L
//! cascade: revenue less cost of goods sold gives gross profit; less operating
//! expenses gives operating income (EBIT); less interest gives pre-tax income;
//! less income tax gives net income. It also reports the margin at each level as
//! a percentage of revenue. Distinct from the trading P&L modules and from the
//! financial-ratios module; this assembles the statement itself. Drafting aid,
//! not accounting advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct IncomeStatementInput {
    pub company_name: String,
    pub period_label: String,
    pub revenue_usd: f64,
    #[serde(default)]
    pub cogs_usd: f64,
    #[serde(default)]
    pub operating_expenses_usd: f64,
    #[serde(default)]
    pub interest_expense_usd: f64,
    /// Income tax rate on pre-tax income, percent.
    #[serde(default)]
    pub tax_rate_pct: f64,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct IncomeStatement {
    pub title: String,
    /// Revenue − COGS.
    pub gross_profit_usd: f64,
    /// Gross profit − operating expenses (EBIT).
    pub operating_income_usd: f64,
    /// Operating income − interest.
    pub pretax_income_usd: f64,
    /// Tax on pre-tax income (0 when pre-tax income is not positive).
    pub income_tax_usd: f64,
    /// Pre-tax income − tax.
    pub net_income_usd: f64,
    pub gross_margin_pct: f64,
    pub operating_margin_pct: f64,
    pub pretax_margin_pct: f64,
    pub net_margin_pct: f64,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &IncomeStatementInput) -> IncomeStatement {
    let gross = cents(i.revenue_usd - i.cogs_usd);
    let ebit = cents(gross - i.operating_expenses_usd);
    let pretax = cents(ebit - i.interest_expense_usd);
    // Tax applies only to positive pre-tax income.
    let tax = if pretax > 0.0 {
        cents(pretax * i.tax_rate_pct / 100.0)
    } else {
        0.0
    };
    let net = cents(pretax - tax);

    let margin = |x: f64| -> f64 {
        if i.revenue_usd != 0.0 {
            cents(x / i.revenue_usd * 100.0)
        } else {
            0.0
        }
    };
    let gm = margin(gross);
    let om = margin(ebit);
    let pm = margin(pretax);
    let nm = margin(net);

    let statement_body = format!(
        "Revenue {}\n  − Cost of goods sold {}\n  = Gross profit {} ({:.2}% margin)\n  − Operating expenses {}\n  = Operating income (EBIT) {} ({:.2}% margin)\n  − Interest expense {}\n  = Pre-tax income {} ({:.2}% margin)\n  − Income tax ({:.2}%) {}\n  = Net income {} ({:.2}% margin)",
        money(i.revenue_usd),
        money(i.cogs_usd),
        money(gross), gm,
        money(i.operating_expenses_usd),
        money(ebit), om,
        money(i.interest_expense_usd),
        money(pretax), pm,
        i.tax_rate_pct, money(tax),
        money(net), nm
    );

    let clauses = vec![
        DocClause {
            heading: "Statement".into(),
            body: format!("Company: {}\nPeriod: {}\nDate: {}", i.company_name, i.period_label, i.date),
        },
        DocClause { heading: "Income Statement".into(), body: statement_body },
        DocClause {
            heading: "Summary".into(),
            body: format!(
                "Gross profit {}, operating income {}, pre-tax income {}, net income {}.",
                money(gross), money(ebit), money(pretax), money(net)
            ),
        },
    ];

    IncomeStatement {
        title: "Income Statement (P&L)".into(),
        gross_profit_usd: gross,
        operating_income_usd: ebit,
        pretax_income_usd: pretax,
        income_tax_usd: tax,
        net_income_usd: net,
        gross_margin_pct: gm,
        operating_margin_pct: om,
        pretax_margin_pct: pm,
        net_margin_pct: nm,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> IncomeStatementInput {
        IncomeStatementInput {
            company_name: "Acme Co".into(),
            period_label: "FY2025".into(),
            revenue_usd: 1_000_000.0,
            cogs_usd: 600_000.0,
            operating_expenses_usd: 250_000.0,
            interest_expense_usd: 20_000.0,
            tax_rate_pct: 21.0,
            date: "2026-01-31".into(),
        }
    }

    #[test]
    fn cascade_and_margins() {
        let d = generate(&base());
        assert!(close(d.gross_profit_usd, 400_000.0));
        assert!(close(d.operating_income_usd, 150_000.0));
        assert!(close(d.pretax_income_usd, 130_000.0));
        assert!(close(d.income_tax_usd, 27_300.0));
        assert!(close(d.net_income_usd, 102_700.0));
        assert!(close(d.gross_margin_pct, 40.0));
        assert!(close(d.operating_margin_pct, 15.0));
        assert!(close(d.pretax_margin_pct, 13.0));
        assert!(close(d.net_margin_pct, 10.27));
    }

    #[test]
    fn no_tax_on_pretax_loss() {
        let d = generate(&IncomeStatementInput { operating_expenses_usd: 500_000.0, ..base() });
        // gross 400k − opex 500k = −100k EBIT; pretax −120k → no tax.
        assert!(close(d.pretax_income_usd, -120_000.0));
        assert!(close(d.income_tax_usd, 0.0));
        assert!(close(d.net_income_usd, -120_000.0));
    }

    #[test]
    fn net_equals_pretax_minus_tax() {
        let d = generate(&base());
        assert!(close(d.net_income_usd, d.pretax_income_usd - d.income_tax_usd));
    }

    #[test]
    fn zero_revenue_zero_margins() {
        let d = generate(&IncomeStatementInput { revenue_usd: 0.0, cogs_usd: 0.0, operating_expenses_usd: 0.0, interest_expense_usd: 0.0, ..base() });
        assert!(close(d.gross_margin_pct, 0.0));
        assert!(close(d.net_income_usd, 0.0));
    }

    #[test]
    fn statement_lists_all_levels() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "Income Statement").unwrap();
        assert!(c.body.contains("Gross profit"));
        assert!(c.body.contains("Operating income"));
        assert!(c.body.contains("Net income"));
    }
}
