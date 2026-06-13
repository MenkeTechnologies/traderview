//! Real-estate purchase and sale agreement — a buyer's offer to purchase real
//! property. Distinct from the lease/tenancy documents and from `bill-of-sale`
//! (personal property): it computes the down payment, the financed loan amount,
//! and the earnest money as a percent of price, then assembles the agreement
//! with contingency clauses. Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PurchaseAgreementInput {
    pub buyer_name: String,
    pub seller_name: String,
    pub property_address: String,
    pub purchase_price_usd: f64,
    pub earnest_money_usd: f64,
    /// Down payment as a percent of the purchase price.
    pub down_payment_pct: f64,
    pub closing_date: String,
    #[serde(default)]
    pub financing_contingency: bool,
    #[serde(default)]
    pub inspection_contingency: bool,
    #[serde(default)]
    pub appraisal_contingency: bool,
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
pub struct PurchaseAgreement {
    pub title: String,
    pub purchase_price_usd: f64,
    pub earnest_money_usd: f64,
    pub earnest_money_pct: f64,
    pub down_payment_usd: f64,
    pub loan_amount_usd: f64,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &PurchaseAgreementInput) -> PurchaseAgreement {
    let down_payment = cents(i.purchase_price_usd * i.down_payment_pct / 100.0);
    let loan_amount = cents((i.purchase_price_usd - down_payment).max(0.0));
    let earnest_pct = if i.purchase_price_usd > 0.0 {
        cents(i.earnest_money_usd / i.purchase_price_usd * 100.0)
    } else {
        0.0
    };

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!("This agreement is governed by the laws of the State of {}.", i.state)
    } else {
        format!("This agreement is governed by the laws of the State of {} ({}).", i.state, citation)
    };

    // Contingencies block — list only those that apply.
    let mut conts: Vec<&str> = Vec::new();
    if i.financing_contingency {
        conts.push("financing (the Buyer obtaining a mortgage loan on customary terms)");
    }
    if i.inspection_contingency {
        conts.push("a satisfactory inspection of the property");
    }
    if i.appraisal_contingency {
        conts.push("an appraisal at or above the purchase price");
    }
    let cont_body = if conts.is_empty() {
        "This offer is not subject to any contingencies; it is a non-contingent offer.".to_string()
    } else {
        format!(
            "This offer is contingent on the following, each of which the Buyer may invoke to terminate and recover the earnest money: {}.",
            conts.join("; ")
        )
    };

    let clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!("Buyer: {}\nSeller: {}\nProperty: {}", i.buyer_name, i.seller_name, i.property_address),
        },
        DocClause {
            heading: "1. Purchase Price".into(),
            body: format!("The Buyer agrees to purchase the property for {}.", money(i.purchase_price_usd)),
        },
        DocClause {
            heading: "2. Earnest Money".into(),
            body: format!(
                "The Buyer deposits earnest money of {} ({:.2}% of the purchase price), to be held in escrow and applied to the purchase at closing.",
                money(i.earnest_money_usd), earnest_pct
            ),
        },
        DocClause {
            heading: "3. Financing".into(),
            body: format!(
                "The Buyer will make a down payment of {} ({:.2}% of price) and finance the balance of {} through a mortgage loan.",
                money(down_payment), i.down_payment_pct, money(loan_amount)
            ),
        },
        DocClause {
            heading: "4. Closing".into(),
            body: format!(
                "Closing shall occur on or before {}, at which time the Seller shall convey marketable title by deed and the Buyer shall pay the balance of the purchase price.",
                i.closing_date
            ),
        },
        DocClause { heading: "5. Contingencies".into(), body: cont_body },
        DocClause { heading: "6. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signatures".into(),
            body: format!(
                "Buyer: ____________________  Date: __________\n{}\n\nSeller: ____________________  Date: __________\n{}",
                i.buyer_name, i.seller_name
            ),
        },
    ];

    PurchaseAgreement {
        title: "Real Estate Purchase and Sale Agreement".into(),
        purchase_price_usd: i.purchase_price_usd,
        earnest_money_usd: i.earnest_money_usd,
        earnest_money_pct: earnest_pct,
        down_payment_usd: down_payment,
        loan_amount_usd: loan_amount,
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

    fn base() -> PurchaseAgreementInput {
        PurchaseAgreementInput {
            buyer_name: "Bea Buyer".into(),
            seller_name: "Sol Seller".into(),
            property_address: "100 Maple Ave".into(),
            purchase_price_usd: 400_000.0,
            earnest_money_usd: 8_000.0,
            down_payment_pct: 20.0,
            closing_date: "2026-09-30".into(),
            financing_contingency: true,
            inspection_contingency: true,
            appraisal_contingency: false,
            state: "Washington".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn down_payment_and_loan() {
        let d = generate(&base());
        assert!(close(d.down_payment_usd, 80_000.0));
        assert!(close(d.loan_amount_usd, 320_000.0));
    }

    #[test]
    fn earnest_money_pct() {
        assert!(close(generate(&base()).earnest_money_pct, 2.0));
    }

    #[test]
    fn contingencies_listed() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "5. Contingencies").unwrap();
        assert!(c.body.contains("financing"));
        assert!(c.body.contains("inspection"));
        assert!(!c.body.contains("appraisal"));
    }

    #[test]
    fn non_contingent_when_none() {
        let d = generate(&PurchaseAgreementInput {
            financing_contingency: false,
            inspection_contingency: false,
            appraisal_contingency: false,
            ..base()
        });
        let c = d.clauses.iter().find(|c| c.heading == "5. Contingencies").unwrap();
        assert!(c.body.contains("non-contingent"));
    }

    #[test]
    fn financing_clause_shows_figures() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "3. Financing").unwrap();
        assert!(c.body.contains("$80000.00"));
        assert!(c.body.contains("$320000.00"));
    }

    #[test]
    fn full_cash_purchase() {
        let d = generate(&PurchaseAgreementInput { down_payment_pct: 100.0, ..base() });
        assert!(close(d.down_payment_usd, 400_000.0));
        assert!(close(d.loan_amount_usd, 0.0));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&PurchaseAgreementInput { statute_citation: "RCW 64".into(), ..base() });
        assert_eq!(d.statutory_citation, "RCW 64");
        assert!(d.clauses.iter().any(|c| c.body.contains("RCW 64")));
    }
}
