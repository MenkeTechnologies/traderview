//! Statement of account — an accounts-receivable collections document that
//! aggregates multiple outstanding invoices for one customer and ages each by
//! days outstanding into standard buckets (current 0–30, 31–60, 61–90, 90+).
//! Distinct from an invoice (a single charge) and a demand for payment (a single
//! overdue notice): this is the periodic running balance with an aging summary.
//! Drafting aid, not legal/accounting advice.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Invoice {
    pub number: String,
    /// Invoice date (YYYY-MM-DD).
    pub date: String,
    pub amount_usd: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatementInput {
    pub company_name: String,
    pub customer_name: String,
    /// Date the statement is run; aging is measured against this.
    pub statement_date: String,
    pub invoices: Vec<Invoice>,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct InvoiceRow {
    pub number: String,
    pub date: String,
    pub amount_usd: f64,
    pub days_outstanding: i64,
    /// Aging bucket label: "Current", "31–60", "61–90", or "90+".
    pub bucket: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct StatementOfAccount {
    pub title: String,
    pub invoice_count: usize,
    pub total_due_usd: f64,
    pub current_usd: f64,
    pub b31_60_usd: f64,
    pub b61_90_usd: f64,
    pub over_90_usd: f64,
    pub rows: Vec<InvoiceRow>,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

/// Days from invoice date to the statement date (0 if the invoice is future-dated
/// or either date is unparseable).
fn days_outstanding(statement: Option<NaiveDate>, invoice: &str) -> i64 {
    match (statement, NaiveDate::parse_from_str(invoice, "%Y-%m-%d")) {
        (Some(s), Ok(d)) => (s - d).num_days().max(0),
        _ => 0,
    }
}

pub fn generate(i: &StatementInput) -> StatementOfAccount {
    let statement = NaiveDate::parse_from_str(&i.statement_date, "%Y-%m-%d").ok();

    let mut current = 0.0;
    let mut b31_60 = 0.0;
    let mut b61_90 = 0.0;
    let mut over_90 = 0.0;

    let rows: Vec<InvoiceRow> = i
        .invoices
        .iter()
        .map(|inv| {
            let days = days_outstanding(statement, &inv.date);
            let bucket = if days <= 30 {
                current += inv.amount_usd;
                "Current"
            } else if days <= 60 {
                b31_60 += inv.amount_usd;
                "31–60"
            } else if days <= 90 {
                b61_90 += inv.amount_usd;
                "61–90"
            } else {
                over_90 += inv.amount_usd;
                "90+"
            };
            InvoiceRow {
                number: inv.number.clone(),
                date: inv.date.clone(),
                amount_usd: cents(inv.amount_usd),
                days_outstanding: days,
                bucket: bucket.to_string(),
            }
        })
        .collect();

    let total = cents(current + b31_60 + b61_90 + over_90);
    let (current, b31_60, b61_90, over_90) =
        (cents(current), cents(b31_60), cents(b61_90), cents(over_90));

    let aging_body = format!(
        "Aging of the {} outstanding invoice(s) as of {}:\n  • Current (0–30 days): {}\n  • 31–60 days: {}\n  • 61–90 days: {}\n  • Over 90 days: {}\n  Total due: {}",
        rows.len(),
        i.statement_date,
        money(current),
        money(b31_60),
        money(b61_90),
        money(over_90),
        money(total)
    );

    let detail_body = if rows.is_empty() {
        "No outstanding invoices.".to_string()
    } else {
        rows.iter()
            .map(|r| {
                format!(
                    "  • {} dated {}: {} — {} day(s) outstanding ({})",
                    r.number,
                    r.date,
                    money(r.amount_usd),
                    r.days_outstanding,
                    r.bucket
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let mut clauses = vec![
        DocClause {
            heading: "Statement".into(),
            body: format!(
                "From: {}\nTo: {}\nStatement date: {}",
                i.company_name, i.customer_name, i.statement_date
            ),
        },
        DocClause { heading: "Outstanding Invoices".into(), body: detail_body },
        DocClause { heading: "Aging Summary".into(), body: aging_body },
        DocClause {
            heading: "Remittance".into(),
            body: format!(
                "Please remit the total balance of {} to {}. Amounts over 90 days past due may be referred for collection. If your records differ, contact us within 10 days of this statement.",
                money(total),
                i.company_name
            ),
        },
    ];

    let note = i.note.trim();
    if !note.is_empty() {
        clauses.push(DocClause { heading: "Note".into(), body: note.to_string() });
    }

    StatementOfAccount {
        title: "Statement of Account".into(),
        invoice_count: rows.len(),
        total_due_usd: total,
        current_usd: current,
        b31_60_usd: b31_60,
        b61_90_usd: b61_90,
        over_90_usd: over_90,
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

    fn base() -> StatementInput {
        StatementInput {
            company_name: "Acme Supply Co".into(),
            customer_name: "Beta Retail LLC".into(),
            statement_date: "2026-06-13".into(),
            invoices: vec![
                Invoice { number: "INV-001".into(), date: "2026-06-01".into(), amount_usd: 1000.0 },
                Invoice { number: "INV-002".into(), date: "2026-05-01".into(), amount_usd: 2000.0 },
                Invoice { number: "INV-003".into(), date: "2026-03-15".into(), amount_usd: 1500.0 },
                Invoice { number: "INV-004".into(), date: "2026-01-01".into(), amount_usd: 500.0 },
            ],
            note: String::new(),
        }
    }

    #[test]
    fn aging_buckets_and_total() {
        let d = generate(&base());
        assert_eq!(d.invoice_count, 4);
        assert!(close(d.current_usd, 1000.0));
        assert!(close(d.b31_60_usd, 2000.0));
        assert!(close(d.b61_90_usd, 1500.0));
        assert!(close(d.over_90_usd, 500.0));
        assert!(close(d.total_due_usd, 5000.0));
    }

    #[test]
    fn buckets_sum_to_total() {
        let d = generate(&base());
        assert!(close(d.current_usd + d.b31_60_usd + d.b61_90_usd + d.over_90_usd, d.total_due_usd));
    }

    #[test]
    fn ninety_days_is_61_90_boundary() {
        // INV-003 is exactly 90 days out → 61–90, not 90+.
        let d = generate(&base());
        let r = d.rows.iter().find(|r| r.number == "INV-003").unwrap();
        assert_eq!(r.days_outstanding, 90);
        assert_eq!(r.bucket, "61–90");
    }

    #[test]
    fn future_dated_invoice_is_current_zero_days() {
        let mut inp = base();
        inp.invoices = vec![Invoice { number: "INV-X".into(), date: "2026-12-01".into(), amount_usd: 400.0 }];
        let d = generate(&inp);
        assert_eq!(d.rows[0].days_outstanding, 0);
        assert_eq!(d.rows[0].bucket, "Current");
        assert!(close(d.current_usd, 400.0));
    }

    #[test]
    fn empty_invoices_zero_total() {
        let mut inp = base();
        inp.invoices = vec![];
        let d = generate(&inp);
        assert_eq!(d.invoice_count, 0);
        assert!(close(d.total_due_usd, 0.0));
        assert!(d.clauses.iter().any(|c| c.body.contains("No outstanding invoices")));
    }

    #[test]
    fn note_appended_when_present() {
        let mut inp = base();
        inp.note = "Net-30 terms apply.".into();
        let d = generate(&inp);
        assert!(d.clauses.iter().any(|c| c.heading == "Note" && c.body.contains("Net-30")));
    }
}
