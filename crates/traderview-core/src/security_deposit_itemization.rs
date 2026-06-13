//! Security-deposit itemization statement — the itemized accounting a landlord
//! must send a departing tenant: the deposit held, any required interest, each
//! deduction with its dollar amount, and the balance returned (or still owed).
//!
//! Nearly every state requires this in writing within a fixed number of days of
//! move-out (commonly 14–30); failing to itemize can forfeit the landlord's
//! right to withhold anything. This computes the totals and the statutory
//! deadline date and assembles the statement. Drafting aid, not legal advice.

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Deduction {
    pub description: String,
    pub amount_usd: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DepositInput {
    pub landlord_name: String,
    pub landlord_address: String,
    #[serde(default)]
    pub landlord_phone: String,
    pub tenant_name: String,
    #[serde(default)]
    pub tenant_forwarding_address: String,
    pub premises_address: String,
    pub deposit_held_usd: f64,
    /// Interest owed on the deposit where the state requires it (else 0).
    #[serde(default)]
    pub interest_owed_usd: f64,
    #[serde(default)]
    pub deductions: Vec<Deduction>,
    /// Date the tenancy ended / possession returned (YYYY-MM-DD).
    pub tenancy_end_date: String,
    /// Statutory days to return the deposit, counted from tenancy end.
    pub return_deadline_days: i64,
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
pub struct DepositStatement {
    pub title: String,
    pub deposit_held_usd: f64,
    pub interest_owed_usd: f64,
    pub total_deductions_usd: f64,
    /// Amount refunded to the tenant (0 when deductions exceed the deposit).
    pub balance_returned_usd: f64,
    /// Amount the tenant still owes (0 when the deposit covers the deductions).
    pub balance_owed_by_tenant_usd: f64,
    pub return_deadline_date: String,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &DepositInput) -> DepositStatement {
    let total_deductions: f64 = i.deductions.iter().map(|d| d.amount_usd).sum();
    let gross = i.deposit_held_usd + i.interest_owed_usd;
    let balance = gross - total_deductions;
    let balance_returned = balance.max(0.0);
    let balance_owed = (-balance).max(0.0);

    let deadline = NaiveDate::parse_from_str(&i.tenancy_end_date, "%Y-%m-%d")
        .map(|d| {
            (d + Duration::days(i.return_deadline_days))
                .format("%Y-%m-%d")
                .to_string()
        })
        .unwrap_or_default();

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!(
            "This statement is furnished pursuant to the security-deposit law of the State of {}.",
            i.state
        )
    } else {
        format!(
            "This statement is furnished pursuant to the security-deposit law of the State of {} ({}).",
            i.state, citation
        )
    };

    let to_body = if i.tenant_forwarding_address.trim().is_empty() {
        i.tenant_name.clone()
    } else {
        format!("{}\n{}", i.tenant_name, i.tenant_forwarding_address.trim())
    };

    // Itemized deductions block — one line each, then the total.
    let deductions_body = if i.deductions.is_empty() {
        "No deductions are claimed. The full deposit is being returned.".to_string()
    } else {
        let mut lines: Vec<String> = i
            .deductions
            .iter()
            .map(|d| format!("  • {} — {}", d.description, money(d.amount_usd)))
            .collect();
        lines.push(format!("Total deductions: {}", money(total_deductions)));
        lines.join("\n")
    };

    let balance_body = if balance_owed > 0.0 {
        format!(
            "Your deductions exceed the deposit held. After applying the {} deposit{} against {} in charges, a balance of {} remains owed by the tenant. Payment is requested by {}.",
            money(i.deposit_held_usd),
            if i.interest_owed_usd > 0.0 {
                format!(" plus {} interest", money(i.interest_owed_usd))
            } else {
                String::new()
            },
            money(total_deductions),
            money(balance_owed),
            deadline
        )
    } else {
        format!(
            "After applying {} in itemized deductions against the {} deposit{}, a balance of {} is being returned to the tenant. This refund must be delivered on or before {} ({} days after the tenancy ended).",
            money(total_deductions),
            money(i.deposit_held_usd),
            if i.interest_owed_usd > 0.0 {
                format!(" plus {} interest", money(i.interest_owed_usd))
            } else {
                String::new()
            },
            money(balance_returned),
            deadline,
            i.return_deadline_days
        )
    };

    let clauses = vec![
        DocClause { heading: "To".into(), body: to_body },
        DocClause {
            heading: "1. Deposit Held".into(),
            body: format!(
                "A security deposit of {} was held by {} for the premises located at {}.",
                money(i.deposit_held_usd),
                i.landlord_name,
                i.premises_address
            ),
        },
        DocClause {
            heading: "2. Itemized Deductions".into(),
            body: deductions_body,
        },
        DocClause {
            heading: "3. Balance".into(),
            body: balance_body,
        },
        DocClause { heading: "4. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signature".into(),
            body: format!(
                "Signature of owner of premises or agent: ____________________  Date: {}\n{}\n{}{}",
                i.tenancy_end_date,
                i.landlord_name,
                i.landlord_address,
                if i.landlord_phone.is_empty() {
                    String::new()
                } else {
                    format!("\nTelephone: {}", i.landlord_phone)
                }
            ),
        },
    ];

    DepositStatement {
        title: "Security Deposit Itemization Statement".into(),
        deposit_held_usd: i.deposit_held_usd,
        interest_owed_usd: i.interest_owed_usd,
        total_deductions_usd: total_deductions,
        balance_returned_usd: balance_returned,
        balance_owed_by_tenant_usd: balance_owed,
        return_deadline_date: deadline,
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

    fn base() -> DepositInput {
        DepositInput {
            landlord_name: "Acme Property Mgmt".into(),
            landlord_address: "1 Main St".into(),
            landlord_phone: String::new(),
            tenant_name: "Jane Doe".into(),
            tenant_forwarding_address: "99 New Ave".into(),
            premises_address: "42 Rental Rd".into(),
            deposit_held_usd: 1500.0,
            interest_owed_usd: 0.0,
            deductions: vec![
                Deduction { description: "Carpet cleaning".into(), amount_usd: 200.0 },
                Deduction { description: "Wall repair".into(), amount_usd: 350.0 },
            ],
            tenancy_end_date: "2026-06-30".into(),
            return_deadline_days: 21,
            state: "California".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn totals_and_balance_returned() {
        let d = generate(&base());
        assert!(close(d.total_deductions_usd, 550.0));
        assert!(close(d.balance_returned_usd, 950.0));
        assert!(close(d.balance_owed_by_tenant_usd, 0.0));
    }

    #[test]
    fn deadline_is_end_plus_days() {
        // 2026-06-30 + 21 days = 2026-07-21.
        assert_eq!(generate(&base()).return_deadline_date, "2026-07-21");
    }

    #[test]
    fn deductions_exceeding_deposit_owe_balance() {
        let d = generate(&DepositInput {
            deposit_held_usd: 1000.0,
            deductions: vec![Deduction {
                description: "Major damage".into(),
                amount_usd: 1200.0,
            }],
            ..base()
        });
        assert!(close(d.total_deductions_usd, 1200.0));
        assert!(close(d.balance_returned_usd, 0.0));
        assert!(close(d.balance_owed_by_tenant_usd, 200.0));
    }

    #[test]
    fn interest_adds_to_returnable_balance() {
        let d = generate(&DepositInput {
            interest_owed_usd: 30.0,
            ..base()
        });
        // 1500 + 30 − 550 = 980 returned.
        assert!(close(d.balance_returned_usd, 980.0));
    }

    #[test]
    fn no_deductions_returns_full_deposit() {
        let d = generate(&DepositInput {
            deductions: vec![],
            ..base()
        });
        assert!(close(d.total_deductions_usd, 0.0));
        assert!(close(d.balance_returned_usd, 1500.0));
        assert!(d.clauses.iter().any(|c| c.body.contains("full deposit is being returned")));
    }

    #[test]
    fn statute_citation_echoed_when_supplied() {
        let d = generate(&DepositInput {
            statute_citation: "Cal. Civ. Code § 1950.5".into(),
            ..base()
        });
        assert_eq!(d.statutory_citation, "Cal. Civ. Code § 1950.5");
        assert!(d.clauses.iter().any(|c| c.body.contains("Cal. Civ. Code § 1950.5")));
    }

    #[test]
    fn forwarding_address_included_in_to_clause() {
        let to = generate(&base()).clauses.into_iter().find(|c| c.heading == "To").unwrap();
        assert!(to.body.contains("Jane Doe"));
        assert!(to.body.contains("99 New Ave"));
    }

    #[test]
    fn bad_date_yields_empty_deadline() {
        let d = generate(&DepositInput {
            tenancy_end_date: "not-a-date".into(),
            ..base()
        });
        assert_eq!(d.return_deadline_date, "");
    }
}
