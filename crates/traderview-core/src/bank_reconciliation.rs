//! Bank reconciliation — ties a company's book cash balance to the bank statement
//! balance. The bank side adds deposits in transit and subtracts outstanding
//! checks; the book side adds interest earned and subtracts service charges and
//! returned (NSF) items. When the two adjusted balances agree the account is
//! reconciled; any difference flags an error to investigate. No existing generator
//! computes a two-sided cash reconciliation. Drafting aid, not accounting advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct BankReconciliationInput {
    pub company_name: String,
    pub account_label: String,
    pub statement_date: String,
    /// Ending balance per the bank statement.
    pub bank_statement_balance_usd: f64,
    /// Deposits recorded in the books but not yet on the statement.
    #[serde(default)]
    pub deposits_in_transit_usd: f64,
    /// Checks written but not yet cleared.
    #[serde(default)]
    pub outstanding_checks_usd: f64,
    /// Ending balance per the books (general ledger).
    pub book_balance_usd: f64,
    /// Interest credited by the bank, not yet in the books.
    #[serde(default)]
    pub interest_earned_usd: f64,
    /// Bank service charges not yet in the books.
    #[serde(default)]
    pub service_charges_usd: f64,
    /// Returned/NSF items charged back, not yet in the books.
    #[serde(default)]
    pub nsf_returns_usd: f64,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BankReconciliation {
    pub title: String,
    /// Bank balance + deposits in transit − outstanding checks.
    pub adjusted_bank_balance_usd: f64,
    /// Book balance + interest − service charges − NSF returns.
    pub adjusted_book_balance_usd: f64,
    /// Adjusted bank − adjusted book (0 when reconciled).
    pub difference_usd: f64,
    pub reconciled: bool,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &BankReconciliationInput) -> BankReconciliation {
    let adj_bank = cents(
        i.bank_statement_balance_usd + i.deposits_in_transit_usd - i.outstanding_checks_usd,
    );
    let adj_book = cents(
        i.book_balance_usd + i.interest_earned_usd - i.service_charges_usd - i.nsf_returns_usd,
    );
    let difference = cents(adj_bank - adj_book);
    let reconciled = difference.abs() < 0.01;

    let bank_body = format!(
        "Bank statement balance {}\n  + Deposits in transit {}\n  − Outstanding checks {}\n  = Adjusted bank balance {}",
        money(i.bank_statement_balance_usd),
        money(i.deposits_in_transit_usd),
        money(i.outstanding_checks_usd),
        money(adj_bank)
    );

    let book_body = format!(
        "Book balance {}\n  + Interest earned {}\n  − Service charges {}\n  − NSF / returned items {}\n  = Adjusted book balance {}",
        money(i.book_balance_usd),
        money(i.interest_earned_usd),
        money(i.service_charges_usd),
        money(i.nsf_returns_usd),
        money(adj_book)
    );

    let result_body = if reconciled {
        format!(
            "The adjusted bank balance and adjusted book balance both equal {}. The account is reconciled.",
            money(adj_bank)
        )
    } else {
        format!(
            "The adjusted balances differ by {} (bank {} vs book {}). Investigate the discrepancy before posting.",
            money(difference.abs()),
            money(adj_bank),
            money(adj_book)
        )
    };

    let clauses = vec![
        DocClause {
            heading: "Reconciliation".into(),
            body: format!(
                "Company: {}\nAccount: {}\nStatement date: {}\nPrepared: {}",
                i.company_name, i.account_label, i.statement_date, i.date
            ),
        },
        DocClause { heading: "Bank Side".into(), body: bank_body },
        DocClause { heading: "Book Side".into(), body: book_body },
        DocClause { heading: "Result".into(), body: result_body },
    ];

    BankReconciliation {
        title: "Bank Reconciliation".into(),
        adjusted_bank_balance_usd: adj_bank,
        adjusted_book_balance_usd: adj_book,
        difference_usd: difference,
        reconciled,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> BankReconciliationInput {
        BankReconciliationInput {
            company_name: "Acme Co".into(),
            account_label: "Operating checking".into(),
            statement_date: "2026-06-30".into(),
            bank_statement_balance_usd: 10_000.0,
            deposits_in_transit_usd: 2_000.0,
            outstanding_checks_usd: 1_500.0,
            book_balance_usd: 10_650.0,
            interest_earned_usd: 50.0,
            service_charges_usd: 100.0,
            nsf_returns_usd: 100.0,
            date: "2026-07-01".into(),
        }
    }

    #[test]
    fn reconciled_ties_out() {
        let d = generate(&base());
        assert!(close(d.adjusted_bank_balance_usd, 10_500.0));
        assert!(close(d.adjusted_book_balance_usd, 10_500.0));
        assert!(close(d.difference_usd, 0.0));
        assert!(d.reconciled);
    }

    #[test]
    fn out_of_balance_flagged() {
        let d = generate(&BankReconciliationInput { book_balance_usd: 10_000.0, ..base() });
        // adjusted book 9,850 vs bank 10,500 → difference 650.
        assert!(close(d.difference_usd, 650.0));
        assert!(!d.reconciled);
        assert!(d.clauses.iter().any(|c| c.body.contains("Investigate")));
    }

    #[test]
    fn deposits_and_checks_adjust_bank() {
        let d = generate(&base());
        // 10,000 + 2,000 − 1,500 = 10,500.
        assert!(close(d.adjusted_bank_balance_usd, 10_500.0));
    }

    #[test]
    fn interest_and_charges_adjust_book() {
        let d = generate(&base());
        // 10,650 + 50 − 100 − 100 = 10,500.
        assert!(close(d.adjusted_book_balance_usd, 10_500.0));
    }

    #[test]
    fn both_sides_present() {
        let d = generate(&base());
        assert!(d.clauses.iter().any(|c| c.heading == "Bank Side"));
        assert!(d.clauses.iter().any(|c| c.heading == "Book Side"));
    }
}
