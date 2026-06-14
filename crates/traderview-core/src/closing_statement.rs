//! Seller's closing / settlement statement — the net sheet at a real-estate
//! closing. From the sale price it subtracts the real-estate commission, the
//! mortgage payoff, the seller's prorated share of annual property tax (days
//! owed ÷ 365), and other closing costs to produce the net proceeds to the
//! seller. Distinct from the purchase agreement (this is the money settlement,
//! not the offer). Drafting aid, not legal/closing advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ClosingInput {
    pub seller_name: String,
    pub buyer_name: String,
    pub property_address: String,
    pub sale_price_usd: f64,
    pub closing_date: String,
    /// Real-estate commission, percent of sale price.
    #[serde(default)]
    pub commission_pct: f64,
    /// Existing mortgage payoff at closing.
    #[serde(default)]
    pub mortgage_payoff_usd: f64,
    #[serde(default)]
    pub annual_property_tax_usd: f64,
    /// Days of the tax year the seller is responsible for (prorated to closing).
    #[serde(default)]
    pub tax_days_owed: f64,
    /// Other seller closing costs (title, escrow, recording, etc.).
    #[serde(default)]
    pub other_costs_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ClosingStatement {
    pub title: String,
    pub sale_price_usd: f64,
    pub commission_usd: f64,
    pub mortgage_payoff_usd: f64,
    pub tax_proration_usd: f64,
    pub other_costs_usd: f64,
    pub total_deductions_usd: f64,
    pub net_to_seller_usd: f64,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &ClosingInput) -> ClosingStatement {
    let commission = cents(i.sale_price_usd * i.commission_pct / 100.0);
    let tax_proration = cents(i.annual_property_tax_usd * i.tax_days_owed / 365.0);
    let total_deductions =
        cents(commission + i.mortgage_payoff_usd + tax_proration + i.other_costs_usd);
    let net_to_seller = cents(i.sale_price_usd - total_deductions);

    // Seller's charges, one line each (omit zero lines except the always-shown
    // commission and payoff anchors).
    let mut lines = vec![
        format!("  Real-estate commission ({:.2}%): {}", i.commission_pct, money(commission)),
        format!("  Mortgage payoff: {}", money(i.mortgage_payoff_usd)),
    ];
    if tax_proration != 0.0 {
        lines.push(format!(
            "  Property-tax proration ({} of 365 days): {}",
            i.tax_days_owed, money(tax_proration)
        ));
    }
    if i.other_costs_usd != 0.0 {
        lines.push(format!("  Other closing costs: {}", money(i.other_costs_usd)));
    }
    lines.push(format!("Total seller charges: {}", money(total_deductions)));

    let clauses = vec![
        DocClause {
            heading: "Header".into(),
            body: format!(
                "Seller: {}\nBuyer: {}\nProperty: {}\nClosing date: {}",
                i.seller_name, i.buyer_name, i.property_address, i.closing_date
            ),
        },
        DocClause {
            heading: "1. Sale Price".into(),
            body: format!("Gross sale price: {}", money(i.sale_price_usd)),
        },
        DocClause { heading: "2. Seller's Charges".into(), body: lines.join("\n") },
        DocClause {
            heading: "3. Net Proceeds to Seller".into(),
            body: format!(
                "Net proceeds to seller (sale price {} less total charges {}): {}.",
                money(i.sale_price_usd),
                money(total_deductions),
                money(net_to_seller)
            ),
        },
        DocClause {
            heading: "Acknowledgment".into(),
            body: format!(
                "Seller: ____________________  Date: {}\n{}\n\nThis statement is for the seller's records and is not a substitute for the official settlement statement prepared by the closing agent.",
                i.closing_date, i.seller_name
            ),
        },
    ];

    ClosingStatement {
        title: "Seller's Closing Statement".into(),
        sale_price_usd: i.sale_price_usd,
        commission_usd: commission,
        mortgage_payoff_usd: i.mortgage_payoff_usd,
        tax_proration_usd: tax_proration,
        other_costs_usd: i.other_costs_usd,
        total_deductions_usd: total_deductions,
        net_to_seller_usd: net_to_seller,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> ClosingInput {
        ClosingInput {
            seller_name: "Sol Seller".into(),
            buyer_name: "Bea Buyer".into(),
            property_address: "100 Maple Ave".into(),
            sale_price_usd: 400_000.0,
            closing_date: "2026-09-30".into(),
            commission_pct: 6.0,
            mortgage_payoff_usd: 250_000.0,
            annual_property_tax_usd: 4_800.0,
            tax_days_owed: 180.0,
            other_costs_usd: 3_000.0,
        }
    }

    #[test]
    fn commission_and_proration() {
        let d = generate(&base());
        assert!(close(d.commission_usd, 24_000.0));
        assert!(close(d.tax_proration_usd, 2_367.12));
    }

    #[test]
    fn total_and_net() {
        let d = generate(&base());
        assert!(close(d.total_deductions_usd, 279_367.12));
        assert!(close(d.net_to_seller_usd, 120_632.88));
    }

    #[test]
    fn charges_clause_lists_lines() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Seller's Charges")).unwrap();
        assert!(c.body.contains("Real-estate commission (6.00%): $24000.00"));
        assert!(c.body.contains("Property-tax proration (180 of 365 days): $2367.12"));
        assert!(c.body.contains("Total seller charges: $279367.12"));
    }

    #[test]
    fn no_tax_proration_omits_line() {
        let d = generate(&ClosingInput { annual_property_tax_usd: 0.0, tax_days_owed: 0.0, ..base() });
        assert!(close(d.tax_proration_usd, 0.0));
        let c = d.clauses.iter().find(|c| c.heading.contains("Seller's Charges")).unwrap();
        assert!(!c.body.contains("proration"));
        // 24,000 + 250,000 + 3,000 = 277,000; net 123,000.
        assert!(close(d.net_to_seller_usd, 123_000.0));
    }

    #[test]
    fn net_proceeds_clause() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Net Proceeds")).unwrap();
        assert!(c.body.contains("$120632.88"));
    }

    #[test]
    fn cash_sale_no_payoff() {
        let d = generate(&ClosingInput { mortgage_payoff_usd: 0.0, ..base() });
        // 24,000 + 2,367.12 + 3,000 = 29,367.12; net 370,632.88.
        assert!(close(d.net_to_seller_usd, 370_632.88));
    }
}
