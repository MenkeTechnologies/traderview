//! Promissory note — a borrower's written promise to repay a fixed loan on
//! defined terms. Used for owner financing, intra-family loans, business notes,
//! and seller carry-back. This amortizes the loan (level monthly payment, total
//! interest, total of payments, maturity date) and assembles the note's
//! operative clauses. Drafting aid, not legal advice.

use chrono::{Months, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct NoteInput {
    pub lender_name: String,
    pub borrower_name: String,
    pub principal_usd: f64,
    pub annual_rate_pct: f64,
    pub term_months: u32,
    /// Note date / first payment basis (YYYY-MM-DD).
    pub start_date: String,
    /// Flat late charge per missed installment (0 = none).
    #[serde(default)]
    pub late_fee_usd: f64,
    pub governing_state: String,
    #[serde(default)]
    pub statute_citation: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PromissoryNote {
    pub title: String,
    pub monthly_payment_usd: f64,
    pub total_of_payments_usd: f64,
    pub total_interest_usd: f64,
    pub maturity_date: String,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

/// Level monthly payment on a fully-amortizing loan. Zero rate → straight
/// principal split; zero term → zero payment.
pub fn monthly_payment(principal: f64, annual_rate_pct: f64, term_months: u32) -> f64 {
    if term_months == 0 {
        return 0.0;
    }
    let n = term_months as i32;
    let r = annual_rate_pct / 100.0 / 12.0;
    if r == 0.0 {
        principal / n as f64
    } else {
        principal * r / (1.0 - (1.0 + r).powi(-n))
    }
}

pub fn generate(i: &NoteInput) -> PromissoryNote {
    let payment = monthly_payment(i.principal_usd, i.annual_rate_pct, i.term_months);
    let total_of_payments = payment * i.term_months as f64;
    let total_interest = (total_of_payments - i.principal_usd).max(0.0);

    let maturity = NaiveDate::parse_from_str(&i.start_date, "%Y-%m-%d")
        .ok()
        .and_then(|d| d.checked_add_months(Months::new(i.term_months)))
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!(
            "This Note shall be governed by the laws of the State of {}.",
            i.governing_state
        )
    } else {
        format!(
            "This Note shall be governed by the laws of the State of {} ({}).",
            i.governing_state, citation
        )
    };

    let mut clauses = vec![
        DocClause {
            heading: "Parties".into(),
            body: format!(
                "Borrower: {}\nLender: {}\nPrincipal Amount: {}\nAnnual Interest Rate: {:.3}%\nTerm: {} months",
                i.borrower_name,
                i.lender_name,
                money(i.principal_usd),
                i.annual_rate_pct,
                i.term_months
            ),
        },
        DocClause {
            heading: "1. Promise to Pay".into(),
            body: format!(
                "For value received, {} (\"Borrower\") promises to pay to the order of {} (\"Lender\") the principal sum of {}, together with interest as set forth below.",
                i.borrower_name, i.lender_name, money(i.principal_usd)
            ),
        },
        DocClause {
            heading: "2. Interest".into(),
            body: format!(
                "Interest shall accrue on the unpaid principal balance at the rate of {:.3}% per annum.",
                i.annual_rate_pct
            ),
        },
        DocClause {
            heading: "3. Repayment".into(),
            body: format!(
                "Borrower shall repay this Note in {} equal monthly installments of {} each, beginning on {}, with a final maturity date of {}, on which date the entire remaining balance, if any, shall be due and payable. Total of payments over the term: {} (of which {} is interest).",
                i.term_months,
                money(payment),
                i.start_date,
                maturity,
                money(total_of_payments),
                money(total_interest)
            ),
        },
        DocClause {
            heading: "4. Prepayment".into(),
            body: "Borrower may prepay this Note in whole or in part at any time without penalty. Each prepayment shall be applied first to accrued interest and then to principal.".into(),
        },
    ];

    let mut next = 5;
    if i.late_fee_usd > 0.0 {
        clauses.push(DocClause {
            heading: format!("{next}. Late Charge"),
            body: format!(
                "If any installment is not paid within ten (10) days of its due date, Borrower shall pay a late charge of {} for that installment.",
                money(i.late_fee_usd)
            ),
        });
        next += 1;
    }

    clauses.push(DocClause {
        heading: format!("{next}. Default and Acceleration"),
        body: "If Borrower fails to pay any installment when due and the default continues, Lender may declare the entire unpaid principal balance and accrued interest immediately due and payable, and may pursue any remedy available at law or in equity.".into(),
    });
    next += 1;

    clauses.push(DocClause {
        heading: format!("{next}. Governing Law"),
        body: pursuant,
    });

    clauses.push(DocClause {
        heading: "Signatures".into(),
        body: format!(
            "Borrower: ____________________  Date: __________\n{}\n\nLender: ____________________  Date: __________\n{}",
            i.borrower_name, i.lender_name
        ),
    });

    PromissoryNote {
        title: "Promissory Note".into(),
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

    fn base() -> NoteInput {
        NoteInput {
            lender_name: "Jane Lender".into(),
            borrower_name: "John Borrower".into(),
            principal_usd: 10_000.0,
            annual_rate_pct: 6.0,
            term_months: 12,
            start_date: "2026-01-01".into(),
            late_fee_usd: 0.0,
            governing_state: "Texas".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn monthly_payment_amortizes() {
        // $10k, 6%/yr, 12 months → ~$860.66.
        let n = generate(&base());
        assert!((n.monthly_payment_usd - 860.66).abs() < 0.01);
    }

    #[test]
    fn totals_consistent() {
        let n = generate(&base());
        // total ≈ 10327.97, interest ≈ 327.97.
        assert!((n.total_of_payments_usd - 10_327.97).abs() < 0.05);
        assert!((n.total_interest_usd - 327.97).abs() < 0.05);
        // Identity: total = principal + interest.
        assert!((n.total_of_payments_usd - (10_000.0 + n.total_interest_usd)).abs() < 1e-6);
    }

    #[test]
    fn maturity_is_start_plus_term() {
        // 2026-01-01 + 12 months = 2027-01-01.
        assert_eq!(generate(&base()).maturity_date, "2027-01-01");
    }

    #[test]
    fn zero_rate_is_straight_principal_split() {
        let n = generate(&NoteInput {
            principal_usd: 12_000.0,
            annual_rate_pct: 0.0,
            ..base()
        });
        assert!((n.monthly_payment_usd - 1_000.0).abs() < 1e-6);
        assert!((n.total_interest_usd - 0.0).abs() < 1e-6);
    }

    #[test]
    fn late_fee_clause_only_when_set() {
        let without = generate(&base());
        assert!(!without.clauses.iter().any(|c| c.heading.contains("Late Charge")));
        let with = generate(&NoteInput { late_fee_usd: 25.0, ..base() });
        assert!(with.clauses.iter().any(|c| c.heading.contains("Late Charge")));
    }

    #[test]
    fn clause_numbering_stays_sequential_with_late_fee() {
        // With the optional late-charge clause present, Governing Law must still
        // be numbered one past it (no gap, no collision).
        let with = generate(&NoteInput { late_fee_usd: 25.0, ..base() });
        assert!(with.clauses.iter().any(|c| c.heading == "5. Late Charge"));
        assert!(with.clauses.iter().any(|c| c.heading == "6. Default and Acceleration"));
        assert!(with.clauses.iter().any(|c| c.heading == "7. Governing Law"));
    }

    #[test]
    fn statute_citation_echoed() {
        let n = generate(&NoteInput {
            statute_citation: "Tex. Fin. Code § 302".into(),
            ..base()
        });
        assert_eq!(n.statutory_citation, "Tex. Fin. Code § 302");
        assert!(n.clauses.iter().any(|c| c.body.contains("Tex. Fin. Code § 302")));
    }

    #[test]
    fn bad_date_yields_empty_maturity() {
        let n = generate(&NoteInput { start_date: "nope".into(), ..base() });
        assert_eq!(n.maturity_date, "");
    }
}
