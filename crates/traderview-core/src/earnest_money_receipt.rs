//! Earnest money receipt — the escrow holder's (or broker's) acknowledgment of
//! the buyer's good-faith deposit on a real-estate purchase. It records the
//! deposit, computes it as a percent of the purchase price and the balance due
//! at closing, and states the escrow/refund terms. Drafting aid, not legal
//! advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct EarnestMoneyInput {
    pub escrow_holder_name: String,
    pub buyer_name: String,
    pub seller_name: String,
    pub property_address: String,
    pub earnest_money_usd: f64,
    pub purchase_price_usd: f64,
    pub received_date: String,
    #[serde(default)]
    pub payment_method: String,
    #[serde(default)]
    pub state: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct EarnestMoneyReceipt {
    pub title: String,
    pub earnest_money_usd: f64,
    pub purchase_price_usd: f64,
    pub earnest_money_pct: f64,
    pub balance_at_closing_usd: f64,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &EarnestMoneyInput) -> EarnestMoneyReceipt {
    let earnest_pct = if i.purchase_price_usd > 0.0 {
        cents(i.earnest_money_usd / i.purchase_price_usd * 100.0)
    } else {
        0.0
    };
    let balance = cents((i.purchase_price_usd - i.earnest_money_usd).max(0.0));

    let method_part = if i.payment_method.trim().is_empty() {
        String::new()
    } else {
        format!(" by {}", i.payment_method.trim())
    };

    let mut clauses = vec![
        DocClause {
            heading: "Header".into(),
            body: format!(
                "Escrow holder: {}\nBuyer: {}\nSeller: {}\nProperty: {}",
                i.escrow_holder_name, i.buyer_name, i.seller_name, i.property_address
            ),
        },
        DocClause {
            heading: "1. Receipt of Funds".into(),
            body: format!(
                "The escrow holder acknowledges receipt from the Buyer of earnest money in the amount of {}{} on {}.",
                money(i.earnest_money_usd), method_part, i.received_date
            ),
        },
        DocClause {
            heading: "2. Application".into(),
            body: format!(
                "The earnest money is {:.2}% of the {} purchase price and will be held in escrow and applied to the purchase at closing, leaving a balance of {} due at closing.",
                earnest_pct, money(i.purchase_price_usd), money(balance)
            ),
        },
        DocClause {
            heading: "3. Refund and Forfeiture".into(),
            body: "If the Buyer terminates the purchase agreement under a valid contingency, the earnest money is refundable to the Buyer; otherwise it is held, applied, or forfeited as provided in the purchase agreement.".into(),
        },
    ];

    if !i.state.trim().is_empty() {
        clauses.push(DocClause {
            heading: "4. Governing Law".into(),
            body: format!("This receipt is given under the law of the State of {}.", i.state.trim()),
        });
    }

    clauses.push(DocClause {
        heading: "Signature".into(),
        body: format!(
            "Received by (escrow holder): ____________________  Date: {}\n{}",
            i.received_date, i.escrow_holder_name
        ),
    });

    EarnestMoneyReceipt {
        title: "Earnest Money Receipt".into(),
        earnest_money_usd: i.earnest_money_usd,
        purchase_price_usd: i.purchase_price_usd,
        earnest_money_pct: earnest_pct,
        balance_at_closing_usd: balance,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> EarnestMoneyInput {
        EarnestMoneyInput {
            escrow_holder_name: "Title Co Escrow".into(),
            buyer_name: "Bea Buyer".into(),
            seller_name: "Sol Seller".into(),
            property_address: "100 Maple Ave".into(),
            earnest_money_usd: 8_000.0,
            purchase_price_usd: 400_000.0,
            received_date: "2026-06-15".into(),
            payment_method: "wire transfer".into(),
            state: "Arizona".into(),
        }
    }

    #[test]
    fn pct_and_balance() {
        let d = generate(&base());
        assert!(close(d.earnest_money_pct, 2.0));
        assert!(close(d.balance_at_closing_usd, 392_000.0));
    }

    #[test]
    fn receipt_clause_has_method_and_amount() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Receipt of Funds")).unwrap();
        assert!(c.body.contains("$8000.00"));
        assert!(c.body.contains("by wire transfer"));
    }

    #[test]
    fn application_clause_breakdown() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading == "2. Application").unwrap();
        assert!(c.body.contains("2.00% of the $400000.00"));
        assert!(c.body.contains("$392000.00 due at closing"));
    }

    #[test]
    fn no_method_omits_by() {
        let c = generate(&EarnestMoneyInput { payment_method: String::new(), ..base() })
            .clauses.into_iter().find(|c| c.heading.contains("Receipt of Funds")).unwrap();
        assert!(!c.body.contains(" by "));
    }

    #[test]
    fn refund_clause_present() {
        assert!(generate(&base()).clauses.iter().any(|c| c.heading.contains("Refund and Forfeiture")));
    }

    #[test]
    fn governing_law_only_with_state() {
        assert!(generate(&base()).clauses.iter().any(|c| c.heading == "4. Governing Law"));
        let none = generate(&EarnestMoneyInput { state: String::new(), ..base() });
        assert!(!none.clauses.iter().any(|c| c.heading == "4. Governing Law"));
    }
}
