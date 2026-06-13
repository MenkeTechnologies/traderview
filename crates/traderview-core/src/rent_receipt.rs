//! Rent receipt — the written acknowledgment of a rent payment a landlord gives
//! (and some states require) a tenant. It records the amount paid against the
//! rent due, computes any remaining balance or overpayment credit, and flags
//! paid-in-full, then assembles the receipt. Drafting aid, not legal advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ReceiptInput {
    pub landlord_name: String,
    #[serde(default)]
    pub landlord_address: String,
    #[serde(default)]
    pub landlord_phone: String,
    pub tenant_name: String,
    pub premises_address: String,
    pub amount_paid_usd: f64,
    /// Full rent due for the period (used to compute any balance).
    pub rent_due_usd: f64,
    pub payment_date: String,
    /// Rental period this payment covers (YYYY-MM-DD each).
    pub period_start: String,
    pub period_end: String,
    /// How the payment was made (e.g. "Check #1024", "Cash", "ACH").
    #[serde(default)]
    pub payment_method: String,
    #[serde(default)]
    pub statute_citation: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RentReceipt {
    pub title: String,
    pub amount_paid_usd: f64,
    pub rent_due_usd: f64,
    pub balance_remaining_usd: f64,
    pub overpayment_usd: f64,
    pub paid_in_full: bool,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &ReceiptInput) -> RentReceipt {
    let diff = i.amount_paid_usd - i.rent_due_usd;
    let balance = (-diff).max(0.0);
    let overpayment = diff.max(0.0);
    let paid_in_full = i.amount_paid_usd + 0.005 >= i.rent_due_usd;

    let citation = i.statute_citation.trim();

    let method_line = if i.payment_method.trim().is_empty() {
        String::new()
    } else {
        format!(" by {}", i.payment_method.trim())
    };

    let balance_body = if overpayment > 0.0 {
        format!(
            "Rent due for the period was {}. Payment exceeds the amount due by {}, applied as a credit to the tenant's account.",
            money(i.rent_due_usd),
            money(overpayment)
        )
    } else if balance > 0.0 {
        format!(
            "Rent due for the period was {}. A balance of {} remains outstanding after this payment.",
            money(i.rent_due_usd),
            money(balance)
        )
    } else {
        format!(
            "Rent due for the period was {}. This payment pays the rent IN FULL for the period; no balance remains.",
            money(i.rent_due_usd)
        )
    };

    let mut clauses = vec![
        DocClause {
            heading: "Received From".into(),
            body: format!("{} — tenant of {}", i.tenant_name, i.premises_address),
        },
        DocClause {
            heading: "1. Payment Received".into(),
            body: format!(
                "Received the sum of {}{} on {}.",
                money(i.amount_paid_usd),
                method_line,
                i.payment_date
            ),
        },
        DocClause {
            heading: "2. Period Covered".into(),
            body: format!(
                "This payment is applied to rent for the period {} through {}.",
                i.period_start, i.period_end
            ),
        },
        DocClause { heading: "3. Balance".into(), body: balance_body },
    ];

    if !citation.is_empty() {
        clauses.push(DocClause {
            heading: "4. Governing Law".into(),
            body: format!("This receipt is furnished pursuant to {}.", citation),
        });
    }

    clauses.push(DocClause {
        heading: "Signature".into(),
        body: format!(
            "Received by: ____________________  Date: {}\n{}{}{}",
            i.payment_date,
            i.landlord_name,
            if i.landlord_address.trim().is_empty() {
                String::new()
            } else {
                format!("\n{}", i.landlord_address.trim())
            },
            if i.landlord_phone.trim().is_empty() {
                String::new()
            } else {
                format!("\nTelephone: {}", i.landlord_phone.trim())
            }
        ),
    });

    RentReceipt {
        title: "Rent Receipt".into(),
        amount_paid_usd: i.amount_paid_usd,
        rent_due_usd: i.rent_due_usd,
        balance_remaining_usd: balance,
        overpayment_usd: overpayment,
        paid_in_full,
        statutory_citation: citation.to_string(),
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn base() -> ReceiptInput {
        ReceiptInput {
            landlord_name: "Acme Property Mgmt".into(),
            landlord_address: "1 Main St".into(),
            landlord_phone: String::new(),
            tenant_name: "Jane Doe".into(),
            premises_address: "42 Rental Rd".into(),
            amount_paid_usd: 1500.0,
            rent_due_usd: 1500.0,
            payment_date: "2026-06-01".into(),
            period_start: "2026-06-01".into(),
            period_end: "2026-06-30".into(),
            payment_method: "Check #1024".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn exact_payment_is_paid_in_full() {
        let r = generate(&base());
        assert!(close(r.balance_remaining_usd, 0.0));
        assert!(close(r.overpayment_usd, 0.0));
        assert!(r.paid_in_full);
        assert!(r.clauses.iter().any(|c| c.body.contains("IN FULL")));
    }

    #[test]
    fn partial_payment_leaves_balance() {
        let r = generate(&ReceiptInput { amount_paid_usd: 1000.0, ..base() });
        assert!(close(r.balance_remaining_usd, 500.0));
        assert!(!r.paid_in_full);
        assert!(r.clauses.iter().any(|c| c.body.contains("balance of $500.00")));
    }

    #[test]
    fn overpayment_is_credit() {
        let r = generate(&ReceiptInput { amount_paid_usd: 1600.0, ..base() });
        assert!(close(r.overpayment_usd, 100.0));
        assert!(close(r.balance_remaining_usd, 0.0));
        assert!(r.paid_in_full);
        assert!(r.clauses.iter().any(|c| c.body.contains("credit")));
    }

    #[test]
    fn payment_method_in_body() {
        let c = generate(&base()).clauses.into_iter().find(|c| c.heading.contains("Payment Received")).unwrap();
        assert!(c.body.contains("by Check #1024"));
    }

    #[test]
    fn no_method_omits_by_clause() {
        let c = generate(&ReceiptInput { payment_method: String::new(), ..base() })
            .clauses
            .into_iter()
            .find(|c| c.heading.contains("Payment Received"))
            .unwrap();
        assert!(!c.body.contains(" by "));
    }

    #[test]
    fn citation_adds_governing_law_clause() {
        let with = generate(&ReceiptInput { statute_citation: "Md. Code § 8-211".into(), ..base() });
        assert!(with.clauses.iter().any(|c| c.heading == "4. Governing Law"));
        let without = generate(&base());
        assert!(!without.clauses.iter().any(|c| c.heading == "4. Governing Law"));
    }
}
