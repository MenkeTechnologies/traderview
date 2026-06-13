//! Sales commission agreement — engages a salesperson on commission, optionally
//! with a recoverable base draw. It computes the projected commission from the
//! rate and expected sales (plus the draw for a projected period total) and
//! assembles the commission, draw, payment-timing, and chargeback clauses.
//! Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CommissionInput {
    pub company_name: String,
    pub rep_name: String,
    /// Products/territory the rep is engaged to sell.
    pub engagement_description: String,
    pub commission_rate_pct: f64,
    /// Expected sales for the period, used to project commission.
    #[serde(default)]
    pub expected_sales_usd: f64,
    /// Recoverable base draw per period (0 = pure commission).
    #[serde(default)]
    pub base_draw_usd: f64,
    pub payment_terms_days: i64,
    pub start_date: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CommissionAgreement {
    pub title: String,
    pub commission_rate_pct: f64,
    pub expected_sales_usd: f64,
    pub projected_commission_usd: f64,
    pub base_draw_usd: f64,
    pub projected_period_total_usd: f64,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &CommissionInput) -> CommissionAgreement {
    let projected_commission = cents(i.expected_sales_usd * i.commission_rate_pct / 100.0);
    let projected_total = cents(projected_commission + i.base_draw_usd);

    let projection_part = if i.expected_sales_usd > 0.0 {
        format!(
            " On expected sales of {}, the projected commission is {}.",
            money(i.expected_sales_usd), money(projected_commission)
        )
    } else {
        String::new()
    };

    let draw_body = if i.base_draw_usd > 0.0 {
        format!(
            "The Company will advance a recoverable draw of {} per period against future commissions. Draws exceeding commissions earned are carried forward and recovered from later commissions. Projected period total (draw + projected commission): {}.",
            money(i.base_draw_usd), money(projected_total)
        )
    } else {
        "This is a pure-commission arrangement; no base draw is provided.".to_string()
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!("Company: {}\nSales Representative: {}", i.company_name, i.rep_name),
        },
        DocClause {
            heading: "1. Engagement".into(),
            body: format!(
                "The Company engages the Representative to sell {}, beginning {}.",
                i.engagement_description, i.start_date
            ),
        },
        DocClause {
            heading: "2. Commission".into(),
            body: format!(
                "The Representative earns a commission of {:.3}% of net sales generated (net of returns, discounts, and taxes).{}",
                i.commission_rate_pct, projection_part
            ),
        },
        DocClause { heading: "3. Draw".into(), body: draw_body },
        DocClause {
            heading: "4. Payment".into(),
            body: format!(
                "Commissions are earned when the Company collects payment from the customer and are paid within {} days thereafter.",
                i.payment_terms_days
            ),
        },
        DocClause {
            heading: "5. Chargebacks".into(),
            body: "If a sale is refunded, returned, or charged back, any commission paid on it is reversed and offset against future commissions.".into(),
        },
        DocClause {
            heading: "6. Term".into(),
            body: format!(
                "This is an at-will arrangement under the laws of the State of {}; either party may terminate it at any time. Commissions on sales collected before termination remain payable.",
                i.state
            ),
        },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Company: ____________________  Date: __________\n{}\n\nRepresentative: ____________________  Date: __________\n{}",
                i.company_name, i.rep_name
            ),
        },
    ];

    CommissionAgreement {
        title: "Sales Commission Agreement".into(),
        commission_rate_pct: i.commission_rate_pct,
        expected_sales_usd: i.expected_sales_usd,
        projected_commission_usd: projected_commission,
        base_draw_usd: i.base_draw_usd,
        projected_period_total_usd: projected_total,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> CommissionInput {
        CommissionInput {
            company_name: "Acme Inc".into(),
            rep_name: "Sam Sales".into(),
            engagement_description: "Acme widgets in the Northeast territory".into(),
            commission_rate_pct: 8.0,
            expected_sales_usd: 200_000.0,
            base_draw_usd: 2_000.0,
            payment_terms_days: 15,
            start_date: "2026-07-01".into(),
            state: "New York".into(),
        }
    }

    #[test]
    fn projected_commission_and_total() {
        let d = generate(&base());
        assert!(close(d.projected_commission_usd, 16_000.0));
        assert!(close(d.projected_period_total_usd, 18_000.0));
    }

    #[test]
    fn projection_in_commission_clause() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "2. Commission").unwrap();
        assert!(c.body.contains("8.000% of net sales"));
        assert!(c.body.contains("projected commission is $16000.00"));
    }

    #[test]
    fn draw_clause_present_and_total() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "3. Draw").unwrap();
        assert!(c.body.contains("recoverable draw of $2000.00"));
        assert!(c.body.contains("$18000.00"));
    }

    #[test]
    fn pure_commission_when_no_draw() {
        let d = generate(&CommissionInput { base_draw_usd: 0.0, ..base() });
        assert!(close(d.projected_period_total_usd, 16_000.0));
        let c = d.clauses.iter().find(|c| c.heading == "3. Draw").unwrap();
        assert!(c.body.contains("pure-commission"));
    }

    #[test]
    fn no_expected_sales_omits_projection() {
        let c = generate(&CommissionInput { expected_sales_usd: 0.0, ..base() })
            .clauses.into_iter().find(|c| c.heading == "2. Commission").unwrap();
        assert!(!c.body.contains("projected commission"));
    }

    #[test]
    fn chargeback_clause_present() {
        assert!(generate(&base()).clauses.iter().any(|c| c.heading == "5. Chargebacks"));
    }
}
