//! Business invoice generator.
//!
//! Turns a business's billing details and a list of line items into a
//! finished invoice with the arithmetic done correctly:
//!
//!   * each line's **amount** = quantity × unit price,
//!   * **subtotal** = sum of the line amounts,
//!   * an optional **discount** comes off the subtotal,
//!   * **tax** is charged on the discounted subtotal, and
//!   * the **total** = discounted subtotal + tax.
//!
//! The due date is the invoice date plus the payment terms (net-N). All
//! money is rounded to cents at each step the way an invoice prints, so the
//! displayed lines, subtotal, tax, and total reconcile exactly. Pure compute.

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};

// Line-item math is shared with the estimate and purchase-order generators —
// one implementation in `line_items` so every billing document reconciles
// identically. These aliases keep the invoice's public type names and JSON
// shape unchanged.
pub use crate::line_items::{LineItem as InvoiceLineItem, LineResult as InvoiceLineResult};

#[derive(Debug, Clone, Deserialize)]
pub struct InvoiceInput {
    pub business_name: String,
    #[serde(default)]
    pub business_address: String,
    pub client_name: String,
    #[serde(default)]
    pub client_address: String,
    pub invoice_number: String,
    /// Invoice issue date (YYYY-MM-DD).
    pub invoice_date: String,
    /// Net-N payment terms in days (due date = invoice date + N).
    pub payment_terms_days: i64,
    pub line_items: Vec<InvoiceLineItem>,
    #[serde(default)]
    pub tax_rate_pct: f64,
    #[serde(default)]
    pub discount_pct: f64,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct InvoiceDocument {
    pub business_name: String,
    pub business_address: String,
    pub client_name: String,
    pub client_address: String,
    pub invoice_number: String,
    pub invoice_date: String,
    /// Invoice date + payment terms; empty if the date can't be parsed.
    pub due_date: String,
    pub payment_terms_days: i64,
    pub lines: Vec<InvoiceLineResult>,
    pub subtotal_usd: f64,
    pub discount_pct: f64,
    pub discount_amount_usd: f64,
    pub tax_rate_pct: f64,
    pub tax_amount_usd: f64,
    pub total_usd: f64,
    pub notes: String,
}

pub fn generate(i: &InvoiceInput) -> InvoiceDocument {
    let t = crate::line_items::compute(&i.line_items, i.discount_pct, i.tax_rate_pct);

    let due_date = NaiveDate::parse_from_str(&i.invoice_date, "%Y-%m-%d")
        .map(|d| (d + Duration::days(i.payment_terms_days)).format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    InvoiceDocument {
        business_name: i.business_name.clone(),
        business_address: i.business_address.clone(),
        client_name: i.client_name.clone(),
        client_address: i.client_address.clone(),
        invoice_number: i.invoice_number.clone(),
        invoice_date: i.invoice_date.clone(),
        due_date,
        payment_terms_days: i.payment_terms_days,
        lines: t.lines,
        subtotal_usd: t.subtotal_usd,
        discount_pct: t.discount_pct,
        discount_amount_usd: t.discount_amount_usd,
        tax_rate_pct: t.tax_rate_pct,
        tax_amount_usd: t.tax_amount_usd,
        total_usd: t.total_usd,
        notes: i.notes.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(desc: &str, qty: f64, price: f64) -> InvoiceLineItem {
        InvoiceLineItem { description: desc.into(), quantity: qty, unit_price_usd: price }
    }

    fn base() -> InvoiceInput {
        InvoiceInput {
            business_name: "Acme LLC".into(),
            business_address: "1 Industrial Way".into(),
            client_name: "Beta Corp".into(),
            client_address: "2 Market St".into(),
            invoice_number: "INV-001".into(),
            invoice_date: "2026-06-01".into(),
            payment_terms_days: 30,
            line_items: vec![item("Consulting (hrs)", 10.0, 150.0), item("Materials", 2.0, 75.0)],
            tax_rate_pct: 0.0,
            discount_pct: 0.0,
            notes: String::new(),
        }
    }

    #[test]
    fn line_amounts_and_subtotal() {
        let d = generate(&base());
        assert!((d.lines[0].amount_usd - 1_500.0).abs() < 1e-9);
        assert!((d.lines[1].amount_usd - 150.0).abs() < 1e-9);
        assert!((d.subtotal_usd - 1_650.0).abs() < 1e-9);
        assert!((d.total_usd - 1_650.0).abs() < 1e-9);
    }

    #[test]
    fn discount_comes_off_subtotal() {
        // 10% off 1,650 = 165 discount → taxable 1,485.
        let d = generate(&InvoiceInput { discount_pct: 10.0, ..base() });
        assert!((d.discount_amount_usd - 165.0).abs() < 1e-9);
        assert!((d.total_usd - 1_485.0).abs() < 1e-9);
    }

    #[test]
    fn tax_charged_on_discounted_subtotal() {
        // 8.25% on 1,650 (no discount) = 136.125 → rounds to 136.13.
        let d = generate(&InvoiceInput { tax_rate_pct: 8.25, ..base() });
        assert!((d.tax_amount_usd - 136.13).abs() < 1e-9);
        assert!((d.total_usd - 1_786.13).abs() < 1e-9);
    }

    #[test]
    fn discount_and_tax_combined() {
        // 1,650 − 10% (165) = 1,485 taxable; 8.25% = 122.5125 → 122.51; total 1,607.51.
        let d = generate(&InvoiceInput { discount_pct: 10.0, tax_rate_pct: 8.25, ..base() });
        assert!((d.discount_amount_usd - 165.0).abs() < 1e-9);
        assert!((d.tax_amount_usd - 122.51).abs() < 1e-9);
        assert!((d.total_usd - 1_607.51).abs() < 1e-9);
    }

    #[test]
    fn due_date_is_invoice_date_plus_terms() {
        // 2026-06-01 + 30 days = 2026-07-01.
        let d = generate(&base());
        assert_eq!(d.due_date, "2026-07-01");
    }

    #[test]
    fn empty_invoice_is_all_zero() {
        let d = generate(&InvoiceInput { line_items: vec![], ..base() });
        assert!(d.subtotal_usd.abs() < 1e-9);
        assert!(d.total_usd.abs() < 1e-9);
        assert!(d.lines.is_empty());
    }

    #[test]
    fn fractional_quantities_round_to_cents() {
        // 3.5 × 33.33 = 116.655 → 116.66 per invoice rounding.
        let d = generate(&InvoiceInput {
            line_items: vec![item("Hours", 3.5, 33.33)],
            ..base()
        });
        assert!((d.lines[0].amount_usd - 116.66).abs() < 1e-9);
        assert!((d.subtotal_usd - 116.66).abs() < 1e-9);
    }
}
