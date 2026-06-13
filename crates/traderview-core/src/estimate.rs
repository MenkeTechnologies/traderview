//! Estimate / quote generator — a non-binding price quote a business sends
//! before work is authorized. It shares the line-item math with the invoice
//! (one `line_items` implementation, so the numbers reconcile identically) and
//! adds a validity window: the quote is good through the valid-until date,
//! computed from the estimate date plus the validity period. Drafting aid.

use crate::line_items::{self, LineItem, LineResult};
use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct EstimateInput {
    pub business_name: String,
    #[serde(default)]
    pub business_address: String,
    pub client_name: String,
    #[serde(default)]
    pub client_address: String,
    pub estimate_number: String,
    /// Estimate issue date (YYYY-MM-DD).
    pub estimate_date: String,
    /// Days the quote remains valid (valid-until = estimate date + N).
    pub valid_days: i64,
    pub line_items: Vec<LineItem>,
    #[serde(default)]
    pub tax_rate_pct: f64,
    #[serde(default)]
    pub discount_pct: f64,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EstimateDocument {
    pub business_name: String,
    pub business_address: String,
    pub client_name: String,
    pub client_address: String,
    pub estimate_number: String,
    pub estimate_date: String,
    /// Estimate date + valid_days; empty if the date can't be parsed.
    pub valid_until: String,
    pub valid_days: i64,
    pub lines: Vec<LineResult>,
    pub subtotal_usd: f64,
    pub discount_pct: f64,
    pub discount_amount_usd: f64,
    pub tax_rate_pct: f64,
    pub tax_amount_usd: f64,
    pub total_usd: f64,
    pub notes: String,
}

pub fn generate(i: &EstimateInput) -> EstimateDocument {
    let t = line_items::compute(&i.line_items, i.discount_pct, i.tax_rate_pct);

    let valid_until = NaiveDate::parse_from_str(&i.estimate_date, "%Y-%m-%d")
        .map(|d| (d + Duration::days(i.valid_days)).format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    EstimateDocument {
        business_name: i.business_name.clone(),
        business_address: i.business_address.clone(),
        client_name: i.client_name.clone(),
        client_address: i.client_address.clone(),
        estimate_number: i.estimate_number.clone(),
        estimate_date: i.estimate_date.clone(),
        valid_until,
        valid_days: i.valid_days,
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

    fn item(desc: &str, qty: f64, price: f64) -> LineItem {
        LineItem { description: desc.into(), quantity: qty, unit_price_usd: price }
    }

    fn base() -> EstimateInput {
        EstimateInput {
            business_name: "Acme LLC".into(),
            business_address: "1 Industrial Way".into(),
            client_name: "Beta Corp".into(),
            client_address: "2 Market St".into(),
            estimate_number: "EST-001".into(),
            estimate_date: "2026-06-01".into(),
            valid_days: 30,
            line_items: vec![item("Consulting (hrs)", 10.0, 150.0), item("Materials", 2.0, 75.0)],
            tax_rate_pct: 0.0,
            discount_pct: 0.0,
            notes: String::new(),
        }
    }

    #[test]
    fn subtotal_and_total_match_shared_math() {
        let d = generate(&base());
        assert!((d.subtotal_usd - 1_650.0).abs() < 1e-9);
        assert!((d.total_usd - 1_650.0).abs() < 1e-9);
    }

    #[test]
    fn discount_then_tax() {
        let d = generate(&EstimateInput { discount_pct: 10.0, tax_rate_pct: 8.25, ..base() });
        assert!((d.discount_amount_usd - 165.0).abs() < 1e-9);
        assert!((d.tax_amount_usd - 122.51).abs() < 1e-9);
        assert!((d.total_usd - 1_607.51).abs() < 1e-9);
    }

    #[test]
    fn valid_until_is_date_plus_validity() {
        // 2026-06-01 + 30 days = 2026-07-01.
        assert_eq!(generate(&base()).valid_until, "2026-07-01");
    }

    #[test]
    fn bad_date_yields_empty_valid_until() {
        let d = generate(&EstimateInput { estimate_date: "x".into(), ..base() });
        assert_eq!(d.valid_until, "");
    }

    #[test]
    fn lines_carry_amounts() {
        let d = generate(&base());
        assert_eq!(d.lines.len(), 2);
        assert!((d.lines[0].amount_usd - 1_500.0).abs() < 1e-9);
        assert!((d.lines[1].amount_usd - 150.0).abs() < 1e-9);
    }
}
