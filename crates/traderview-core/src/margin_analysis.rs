//! Income-statement margin waterfall — gross, operating, pre-tax, and net
//! profit at each step down the P&L, with the margin (percent of revenue) at
//! every level.
//!
//! ```text
//! gross profit     = revenue − COGS
//! operating income = gross profit − operating expenses   (EBIT)
//! pre-tax income   = operating income − interest
//! net income       = pre-tax − tax (tax on positive pre-tax only)
//! ```

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct MarginInput {
    pub revenue_usd: f64,
    pub cogs_usd: f64,
    #[serde(default)]
    pub operating_expenses_usd: f64,
    #[serde(default)]
    pub interest_expense_usd: f64,
    /// Tax rate on positive pre-tax income, percent.
    #[serde(default)]
    pub tax_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MarginResult {
    pub gross_profit_usd: f64,
    pub operating_income_usd: f64,
    pub pretax_income_usd: f64,
    pub tax_usd: f64,
    pub net_income_usd: f64,
    /// Each as a percent of revenue; `None` if revenue ≤ 0.
    pub gross_margin_pct: Option<f64>,
    pub operating_margin_pct: Option<f64>,
    pub pretax_margin_pct: Option<f64>,
    pub net_margin_pct: Option<f64>,
}

pub fn analyze(input: &MarginInput) -> MarginResult {
    let gross = input.revenue_usd - input.cogs_usd;
    let operating = gross - input.operating_expenses_usd;
    let pretax = operating - input.interest_expense_usd;
    let tax = if pretax > 0.0 {
        pretax * input.tax_rate_pct / 100.0
    } else {
        0.0
    };
    let net = pretax - tax;

    let margin = |x: f64| {
        if input.revenue_usd > 0.0 {
            Some(x / input.revenue_usd * 100.0)
        } else {
            None
        }
    };

    MarginResult {
        gross_profit_usd: gross,
        operating_income_usd: operating,
        pretax_income_usd: pretax,
        tax_usd: tax,
        net_income_usd: net,
        gross_margin_pct: margin(gross),
        operating_margin_pct: margin(operating),
        pretax_margin_pct: margin(pretax),
        net_margin_pct: margin(net),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn base() -> MarginInput {
        MarginInput {
            revenue_usd: 1000.0,
            cogs_usd: 600.0,
            operating_expenses_usd: 200.0,
            interest_expense_usd: 50.0,
            tax_rate_pct: 21.0,
        }
    }

    #[test]
    fn gross() {
        let r = analyze(&base());
        assert!(close(r.gross_profit_usd, 400.0));
        assert!(close(r.gross_margin_pct.unwrap(), 40.0));
    }

    #[test]
    fn operating() {
        let r = analyze(&base());
        assert!(close(r.operating_income_usd, 200.0));
        assert!(close(r.operating_margin_pct.unwrap(), 20.0));
    }

    #[test]
    fn pretax() {
        let r = analyze(&base());
        assert!(close(r.pretax_income_usd, 150.0));
        assert!(close(r.pretax_margin_pct.unwrap(), 15.0));
    }

    #[test]
    fn tax_on_pretax() {
        assert!(close(analyze(&base()).tax_usd, 31.5));
    }

    #[test]
    fn net() {
        let r = analyze(&base());
        assert!(close(r.net_income_usd, 118.5));
        assert!(close(r.net_margin_pct.unwrap(), 11.85));
    }

    #[test]
    fn margins_narrow_down_the_waterfall() {
        let r = analyze(&base());
        assert!(
            r.gross_margin_pct.unwrap()
                > r.operating_margin_pct.unwrap()
                && r.operating_margin_pct.unwrap() > r.net_margin_pct.unwrap()
        );
    }

    #[test]
    fn loss_has_no_tax() {
        // Heavy opex drives a pre-tax loss → no tax charged.
        let r = analyze(&MarginInput {
            operating_expenses_usd: 500.0,
            ..base()
        });
        assert!(r.pretax_income_usd < 0.0);
        assert!(close(r.tax_usd, 0.0));
        // Net loss equals pre-tax loss (no tax benefit modeled).
        assert!(close(r.net_income_usd, r.pretax_income_usd));
    }

    #[test]
    fn zero_revenue_guards_margins() {
        let r = analyze(&MarginInput {
            revenue_usd: 0.0,
            ..base()
        });
        assert!(r.gross_margin_pct.is_none());
        assert!(r.net_margin_pct.is_none());
    }
}
