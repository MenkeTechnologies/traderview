//! Demand for payment (demand letter) — the formal written demand a creditor
//! sends before pursuing collection or suit. It totals the principal, accrued
//! interest, and late fees, computes the pay-by date from the demand date plus
//! the response window, and assembles the letter. Often a prerequisite to small-
//! claims or collection action. Drafting aid, not legal advice.

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct DemandInput {
    pub creditor_name: String,
    pub creditor_address: String,
    #[serde(default)]
    pub creditor_phone: String,
    pub debtor_name: String,
    pub principal_usd: f64,
    #[serde(default)]
    pub accrued_interest_usd: f64,
    #[serde(default)]
    pub late_fees_usd: f64,
    /// What the debt is for (invoice #, goods, services, loan…).
    pub debt_description: String,
    /// Date the demand is sent (YYYY-MM-DD).
    pub demand_date: String,
    /// Days the debtor has to pay before further action.
    pub response_days: i64,
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
pub struct DemandLetter {
    pub title: String,
    pub principal_usd: f64,
    pub accrued_interest_usd: f64,
    pub late_fees_usd: f64,
    pub total_due_usd: f64,
    pub pay_by_date: String,
    pub statutory_citation: String,
    pub clauses: Vec<DocClause>,
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

pub fn generate(i: &DemandInput) -> DemandLetter {
    let total = i.principal_usd + i.accrued_interest_usd + i.late_fees_usd;

    let pay_by = NaiveDate::parse_from_str(&i.demand_date, "%Y-%m-%d")
        .map(|d| (d + Duration::days(i.response_days)).format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    let citation = i.statute_citation.trim();
    let pursuant = if citation.is_empty() {
        format!(
            "This demand and any subsequent action are made under the laws of the State of {}.",
            i.governing_state
        )
    } else {
        format!(
            "This demand and any subsequent action are made under the laws of the State of {} ({}).",
            i.governing_state, citation
        )
    };

    // Build the itemized statement; only show interest / fees lines when nonzero.
    let mut lines = vec![
        format!("Amount owed for: {}", i.debt_description),
        format!("  Principal: {}", money(i.principal_usd)),
    ];
    if i.accrued_interest_usd != 0.0 {
        lines.push(format!("  Accrued interest: {}", money(i.accrued_interest_usd)));
    }
    if i.late_fees_usd != 0.0 {
        lines.push(format!("  Late fees: {}", money(i.late_fees_usd)));
    }
    lines.push(format!("  Total now due: {}", money(total)));

    let clauses = vec![
        DocClause { heading: "To".into(), body: i.debtor_name.clone() },
        DocClause {
            heading: "1. Statement of Debt".into(),
            body: lines.join("\n"),
        },
        DocClause {
            heading: "2. Demand for Payment".into(),
            body: format!(
                "Demand is hereby made for payment of the total amount of {} on or before {} ({} days from the date of this letter). Payment should be made to {} at {}.",
                money(total),
                pay_by,
                i.response_days,
                i.creditor_name,
                i.creditor_address
            ),
        },
        DocClause {
            heading: "3. Consequences of Non-Payment".into(),
            body: "If payment is not received by the date stated above, the undersigned may pursue all available remedies, which may include referral to a collection agency, reporting of the debt, and the commencement of legal proceedings to recover the amount due together with any interest, costs, and fees allowed by law. This letter is an attempt to collect a debt.".into(),
        },
        DocClause { heading: "4. Governing Law".into(), body: pursuant },
        DocClause {
            heading: "Signature".into(),
            body: format!(
                "Signature: ____________________  Date: {}\n{}\n{}{}",
                i.demand_date,
                i.creditor_name,
                i.creditor_address,
                if i.creditor_phone.is_empty() {
                    String::new()
                } else {
                    format!("\nTelephone: {}", i.creditor_phone)
                }
            ),
        },
    ];

    DemandLetter {
        title: "Demand for Payment".into(),
        principal_usd: i.principal_usd,
        accrued_interest_usd: i.accrued_interest_usd,
        late_fees_usd: i.late_fees_usd,
        total_due_usd: total,
        pay_by_date: pay_by,
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

    fn base() -> DemandInput {
        DemandInput {
            creditor_name: "Acme LLC".into(),
            creditor_address: "1 Commerce St".into(),
            creditor_phone: String::new(),
            debtor_name: "John Debtor".into(),
            principal_usd: 5000.0,
            accrued_interest_usd: 150.0,
            late_fees_usd: 50.0,
            debt_description: "Invoice #1042, unpaid".into(),
            demand_date: "2026-06-01".into(),
            response_days: 15,
            governing_state: "New York".into(),
            statute_citation: String::new(),
        }
    }

    #[test]
    fn total_sums_principal_interest_fees() {
        assert!(close(generate(&base()).total_due_usd, 5200.0));
    }

    #[test]
    fn pay_by_is_demand_plus_response() {
        // 2026-06-01 + 15 days = 2026-06-16.
        assert_eq!(generate(&base()).pay_by_date, "2026-06-16");
    }

    #[test]
    fn zero_interest_and_fees_total_is_principal() {
        let d = generate(&DemandInput {
            accrued_interest_usd: 0.0,
            late_fees_usd: 0.0,
            ..base()
        });
        assert!(close(d.total_due_usd, 5000.0));
        // Interest/fees lines omitted from the statement when zero.
        let stmt = d.clauses.iter().find(|c| c.heading.contains("Statement")).unwrap();
        assert!(!stmt.body.contains("Accrued interest"));
        assert!(!stmt.body.contains("Late fees"));
    }

    #[test]
    fn statement_lists_components_when_present() {
        let stmt = generate(&base())
            .clauses
            .into_iter()
            .find(|c| c.heading.contains("Statement"))
            .unwrap();
        assert!(stmt.body.contains("Accrued interest: $150.00"));
        assert!(stmt.body.contains("Late fees: $50.00"));
        assert!(stmt.body.contains("Total now due: $5200.00"));
    }

    #[test]
    fn statute_citation_echoed() {
        let d = generate(&DemandInput {
            statute_citation: "FDCPA 15 U.S.C. § 1692".into(),
            ..base()
        });
        assert_eq!(d.statutory_citation, "FDCPA 15 U.S.C. § 1692");
        assert!(d.clauses.iter().any(|c| c.body.contains("FDCPA 15 U.S.C. § 1692")));
    }

    #[test]
    fn bad_date_yields_empty_pay_by() {
        let d = generate(&DemandInput { demand_date: "x".into(), ..base() });
        assert_eq!(d.pay_by_date, "");
    }
}
