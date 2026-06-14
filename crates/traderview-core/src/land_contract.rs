//! Land contract / contract for deed — an owner-financed installment sale where
//! the seller keeps legal title until the buyer pays in full (the buyer holds an
//! equitable, possessory interest meanwhile). It reuses the shared amortization
//! helper to compute the monthly payment on the seller-financed balance, the
//! total of payments, and the total interest, plus the maturity date. Distinct
//! from the purchase agreement (cash/bank-financed) and the promissory note.
//! Drafting aid, not legal advice.

use crate::promissory_note::monthly_payment;
use chrono::{Months, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LandContractInput {
    pub seller_name: String,
    pub buyer_name: String,
    pub property_address: String,
    pub purchase_price_usd: f64,
    pub down_payment_usd: f64,
    pub annual_rate_pct: f64,
    pub term_months: u32,
    pub start_date: String,
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
pub struct LandContract {
    pub title: String,
    pub financed_balance_usd: f64,
    pub monthly_payment_usd: f64,
    pub total_of_payments_usd: f64,
    pub total_interest_usd: f64,
    pub maturity_date: String,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &LandContractInput) -> LandContract {
    let financed = cents((i.purchase_price_usd - i.down_payment_usd).max(0.0));
    let payment = cents(monthly_payment(financed, i.annual_rate_pct, i.term_months));
    let total_of_payments = cents(payment * i.term_months as f64);
    let total_interest = cents((total_of_payments - financed).max(0.0));

    let maturity = NaiveDate::parse_from_str(&i.start_date, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.checked_add_months(Months::new(i.term_months)))
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!("This contract is governed by the laws of the State of {}.", i.state)
    } else {
        format!("This contract is governed by the laws of the State of {} ({}).", i.state, citation)
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!("Seller (Vendor): {}\nBuyer (Vendee): {}\nProperty: {}", i.seller_name, i.buyer_name, i.property_address),
        },
        DocClause {
            heading: "1. Sale and Price".into(),
            body: format!(
                "The Seller agrees to sell the property to the Buyer on installments for {}. The Buyer pays {} down, leaving a seller-financed balance of {}.",
                money(i.purchase_price_usd), money(i.down_payment_usd), money(financed)
            ),
        },
        DocClause {
            heading: "2. Payments".into(),
            body: format!(
                "The Buyer shall pay the balance in {} monthly installments of {} each, including interest, with a final maturity date of {}. Total of payments: {} (of which {} is interest).",
                i.term_months, money(payment), maturity, money(total_of_payments), money(total_interest)
            ),
        },
        DocClause {
            heading: "3. Interest".into(),
            body: format!("Interest accrues on the unpaid balance at {:.3}% per annum.", i.annual_rate_pct),
        },
        DocClause {
            heading: "4. Title".into(),
            body: "The Seller retains legal title to the property until the Buyer has paid the full purchase price. The Buyer holds equitable title and is entitled to possession, and is responsible for taxes, insurance, and maintenance during the contract term. Upon full payment, the Seller shall deliver a deed conveying marketable title.".into(),
        },
        DocClause {
            heading: "5. Default and Forfeiture".into(),
            body: "If the Buyer defaults and fails to cure within any period required by law, the Seller may, subject to applicable law, declare the contract forfeited, retain payments made as liquidated damages, and recover possession, or pursue any other remedy available.".into(),
        },
        DocClause { heading: "6. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Seller: ____________________  Date: __________\n{}\n\nBuyer: ____________________  Date: __________\n{}",
                i.seller_name, i.buyer_name
            ),
        },
    ];

    LandContract {
        title: "Land Contract (Contract for Deed)".into(),
        financed_balance_usd: financed,
        monthly_payment_usd: payment,
        total_of_payments_usd: total_of_payments,
        total_interest_usd: total_interest,
        maturity_date: maturity,
        statutory_citation: citation.to_string(),
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.02
    }

    fn base() -> LandContractInput {
        LandContractInput {
            seller_name: "Sol Seller".into(),
            buyer_name: "Bea Buyer".into(),
            property_address: "9 Country Rd".into(),
            purchase_price_usd: 200_000.0,
            down_payment_usd: 20_000.0,
            annual_rate_pct: 6.0,
            term_months: 360,
            start_date: "2026-08-01".into(),
            state: "Ohio".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn financed_and_payment() {
        let d = generate(&base());
        assert!(close(d.financed_balance_usd, 180_000.0));
        assert!(close(d.monthly_payment_usd, 1_079.19));
    }

    #[test]
    fn totals() {
        let d = generate(&base());
        // 1,079.19 × 360 = 388,508.40; interest 208,508.40 (payment rounded).
        assert!(close(d.total_of_payments_usd, 388_508.40));
        assert!(close(d.total_interest_usd, 208_508.40));
    }

    #[test]
    fn maturity_date() {
        // 2026-08-01 + 360 months = 2056-08-01.
        assert_eq!(generate(&base()).maturity_date, "2056-08-01");
    }

    #[test]
    fn payment_clause_breakdown() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "2. Payments").unwrap();
        assert!(c.body.contains("$1079.19"));
        assert!(c.body.contains("360 monthly installments"));
    }

    #[test]
    fn title_clause_retains_until_paid() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "4. Title").unwrap();
        assert!(c.body.contains("retains legal title"));
        assert!(c.body.contains("equitable title"));
    }

    #[test]
    fn forfeiture_clause_present() {
        assert!(generate(&base()).clauses.iter().any(|c| c.heading.contains("Forfeiture")));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&LandContractInput { statute_citation: "Ohio Rev. Code § 5313".into(), ..base() });
        assert_eq!(d.statutory_citation, "Ohio Rev. Code § 5313");
        assert!(d.clauses.iter().any(|c| c.body.contains("Ohio Rev. Code § 5313")));
    }
}
