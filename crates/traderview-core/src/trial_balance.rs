//! Trial balance — lists every general-ledger account with its debit or credit
//! balance and verifies that total debits equal total credits, the basic check
//! that the books are in balance under double-entry accounting. Distinct from the
//! bank reconciliation (two-sided cash) and the financial statements; this is the
//! all-accounts debit/credit equality proof. Drafting aid, not accounting advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Account {
    pub name: String,
    #[serde(default)]
    pub debit_usd: f64,
    #[serde(default)]
    pub credit_usd: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrialBalanceInput {
    pub company_name: String,
    pub as_of_date: String,
    pub accounts: Vec<Account>,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AccountRow {
    pub name: String,
    pub debit_usd: f64,
    pub credit_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TrialBalance {
    pub title: String,
    pub account_count: usize,
    pub total_debits_usd: f64,
    pub total_credits_usd: f64,
    /// Total debits − total credits (0 when balanced).
    pub difference_usd: f64,
    pub balanced: bool,
    pub rows: Vec<AccountRow>,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &TrialBalanceInput) -> TrialBalance {
    let rows: Vec<AccountRow> = i
        .accounts
        .iter()
        .map(|a| AccountRow {
            name: a.name.clone(),
            debit_usd: cents(a.debit_usd),
            credit_usd: cents(a.credit_usd),
        })
        .collect();

    let total_debits = cents(i.accounts.iter().map(|a| a.debit_usd).sum());
    let total_credits = cents(i.accounts.iter().map(|a| a.credit_usd).sum());
    let difference = cents(total_debits - total_credits);
    let balanced = difference.abs() < 0.01;

    let detail = if rows.is_empty() {
        "No accounts listed.".to_string()
    } else {
        rows.iter()
            .map(|r| {
                let side = if r.credit_usd > 0.0 && r.debit_usd == 0.0 {
                    format!("credit {}", money(r.credit_usd))
                } else if r.debit_usd > 0.0 && r.credit_usd == 0.0 {
                    format!("debit {}", money(r.debit_usd))
                } else {
                    format!("debit {} / credit {}", money(r.debit_usd), money(r.credit_usd))
                };
                format!("  • {}: {}", r.name, side)
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let result_body = if balanced {
        format!(
            "Total debits {} equal total credits {}. The trial balance is in balance.",
            money(total_debits), money(total_credits)
        )
    } else {
        format!(
            "Total debits {} do not equal total credits {} — a difference of {}. Locate the error before preparing statements.",
            money(total_debits), money(total_credits), money(difference.abs())
        )
    };

    let clauses = vec![
        DocClause {
            heading: "Trial Balance".into(),
            body: format!("Company: {}\nAs of: {}\nPrepared: {}", i.company_name, i.as_of_date, i.date),
        },
        DocClause { heading: "Accounts".into(), body: detail },
        DocClause {
            heading: "Totals".into(),
            body: format!("Total debits: {}\nTotal credits: {}", money(total_debits), money(total_credits)),
        },
        DocClause { heading: "Result".into(), body: result_body },
    ];

    TrialBalance {
        title: "Trial Balance".into(),
        account_count: rows.len(),
        total_debits_usd: total_debits,
        total_credits_usd: total_credits,
        difference_usd: difference,
        balanced,
        rows,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn dr(name: &str, v: f64) -> Account {
        Account { name: name.into(), debit_usd: v, credit_usd: 0.0 }
    }
    fn cr(name: &str, v: f64) -> Account {
        Account { name: name.into(), debit_usd: 0.0, credit_usd: v }
    }

    fn base() -> TrialBalanceInput {
        TrialBalanceInput {
            company_name: "Acme Co".into(),
            as_of_date: "2026-06-30".into(),
            accounts: vec![
                dr("Cash", 5_000.0),
                dr("Accounts receivable", 3_000.0),
                dr("Equipment", 12_000.0),
                cr("Accounts payable", 4_000.0),
                cr("Notes payable", 6_000.0),
                cr("Owner's capital", 10_000.0),
            ],
            date: "2026-07-01".into(),
        }
    }

    #[test]
    fn balanced_books() {
        let d = generate(&base());
        assert_eq!(d.account_count, 6);
        assert!(close(d.total_debits_usd, 20_000.0));
        assert!(close(d.total_credits_usd, 20_000.0));
        assert!(close(d.difference_usd, 0.0));
        assert!(d.balanced);
    }

    #[test]
    fn out_of_balance_flagged() {
        let d = generate(&TrialBalanceInput {
            accounts: vec![dr("Cash", 5_000.0), cr("Accounts payable", 4_000.0)],
            ..base()
        });
        assert!(close(d.difference_usd, 1_000.0));
        assert!(!d.balanced);
        assert!(d.clauses.iter().any(|c| c.body.contains("Locate the error")));
    }

    #[test]
    fn empty_accounts_balanced_zero() {
        let d = generate(&TrialBalanceInput { accounts: vec![], ..base() });
        assert!(close(d.total_debits_usd, 0.0));
        assert!(d.balanced);
        assert!(d.clauses.iter().any(|c| c.body.contains("No accounts")));
    }

    #[test]
    fn rows_preserve_sides() {
        let d = generate(&base());
        assert!(close(d.rows[0].debit_usd, 5_000.0));
        assert!(close(d.rows[3].credit_usd, 4_000.0));
    }

    #[test]
    fn totals_match_sums() {
        let d = generate(&base());
        let dsum: f64 = d.rows.iter().map(|r| r.debit_usd).sum();
        let csum: f64 = d.rows.iter().map(|r| r.credit_usd).sum();
        assert!(close(dsum, d.total_debits_usd));
        assert!(close(csum, d.total_credits_usd));
    }
}
