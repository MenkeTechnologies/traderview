//! Purchase order (PO) — the buyer-side document that authorizes a purchase
//! from a vendor. It shares the line-item math with the invoice and estimate
//! (one `line_items` implementation), adds optional shipping to the grand total,
//! and computes the expected delivery date from the order date plus the lead
//! time. Drafting aid.

use crate::line_items::{self, LineItem, LineResult};
use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PurchaseOrderInput {
    pub buyer_name: String,
    #[serde(default)]
    pub buyer_address: String,
    pub vendor_name: String,
    #[serde(default)]
    pub vendor_address: String,
    pub po_number: String,
    pub order_date: String,
    /// Lead time in days (expected delivery = order date + N).
    pub delivery_days: i64,
    #[serde(default)]
    pub ship_to: String,
    pub line_items: Vec<LineItem>,
    #[serde(default)]
    pub tax_rate_pct: f64,
    #[serde(default)]
    pub discount_pct: f64,
    #[serde(default)]
    pub shipping_usd: f64,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PurchaseOrder {
    pub buyer_name: String,
    pub buyer_address: String,
    pub vendor_name: String,
    pub vendor_address: String,
    pub po_number: String,
    pub order_date: String,
    /// Order date + delivery_days; empty if the date can't be parsed.
    pub expected_delivery_date: String,
    pub delivery_days: i64,
    pub ship_to: String,
    pub lines: Vec<LineResult>,
    pub subtotal_usd: f64,
    pub discount_pct: f64,
    pub discount_amount_usd: f64,
    pub tax_rate_pct: f64,
    pub tax_amount_usd: f64,
    pub shipping_usd: f64,
    /// Grand total = discounted subtotal + tax + shipping.
    pub total_usd: f64,
    pub notes: String,
}

pub fn generate(i: &PurchaseOrderInput) -> PurchaseOrder {
    let t = line_items::compute(&i.line_items, i.discount_pct, i.tax_rate_pct);
    let shipping = i.shipping_usd.max(0.0);
    let total = line_items::cents(t.total_usd + shipping);

    let expected_delivery_date = NaiveDate::parse_from_str(&i.order_date, "%Y-%m-%d")
        .map(|d| (d + Duration::days(i.delivery_days)).format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    PurchaseOrder {
        buyer_name: i.buyer_name.clone(),
        buyer_address: i.buyer_address.clone(),
        vendor_name: i.vendor_name.clone(),
        vendor_address: i.vendor_address.clone(),
        po_number: i.po_number.clone(),
        order_date: i.order_date.clone(),
        expected_delivery_date,
        delivery_days: i.delivery_days,
        ship_to: i.ship_to.clone(),
        lines: t.lines,
        subtotal_usd: t.subtotal_usd,
        discount_pct: t.discount_pct,
        discount_amount_usd: t.discount_amount_usd,
        tax_rate_pct: t.tax_rate_pct,
        tax_amount_usd: t.tax_amount_usd,
        shipping_usd: shipping,
        total_usd: total,
        notes: i.notes.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(desc: &str, qty: f64, price: f64) -> LineItem {
        LineItem { description: desc.into(), quantity: qty, unit_price_usd: price }
    }

    fn base() -> PurchaseOrderInput {
        PurchaseOrderInput {
            buyer_name: "Acme LLC".into(),
            buyer_address: "1 Industrial Way".into(),
            vendor_name: "Parts Co".into(),
            vendor_address: "9 Supply Rd".into(),
            po_number: "PO-001".into(),
            order_date: "2026-06-01".into(),
            delivery_days: 7,
            ship_to: "1 Industrial Way, Dock B".into(),
            line_items: vec![item("Widget", 10.0, 150.0), item("Bracket", 2.0, 75.0)],
            tax_rate_pct: 0.0,
            discount_pct: 0.0,
            shipping_usd: 50.0,
            notes: String::new(),
        }
    }

    #[test]
    fn shipping_added_to_grand_total() {
        let d = generate(&base());
        // subtotal 1,650 + shipping 50 = 1,700.
        assert!((d.subtotal_usd - 1_650.0).abs() < 1e-9);
        assert!((d.shipping_usd - 50.0).abs() < 1e-9);
        assert!((d.total_usd - 1_700.0).abs() < 1e-9);
    }

    #[test]
    fn discount_tax_then_shipping() {
        // 1,650 − 10% = 1,485; +8.25% tax (122.51) = 1,607.51; + 50 shipping = 1,657.51.
        let d = generate(&PurchaseOrderInput { discount_pct: 10.0, tax_rate_pct: 8.25, ..base() });
        assert!((d.total_usd - 1_657.51).abs() < 1e-9);
    }

    #[test]
    fn expected_delivery_is_order_plus_lead() {
        // 2026-06-01 + 7 days = 2026-06-08.
        assert_eq!(generate(&base()).expected_delivery_date, "2026-06-08");
    }

    #[test]
    fn negative_shipping_clamped() {
        let d = generate(&PurchaseOrderInput { shipping_usd: -10.0, ..base() });
        assert!((d.shipping_usd - 0.0).abs() < 1e-9);
        assert!((d.total_usd - 1_650.0).abs() < 1e-9);
    }

    #[test]
    fn bad_date_yields_empty_delivery() {
        let d = generate(&PurchaseOrderInput { order_date: "x".into(), ..base() });
        assert_eq!(d.expected_delivery_date, "");
    }
}
