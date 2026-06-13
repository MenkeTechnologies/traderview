//! Shared line-item math for billing documents (invoice, estimate, purchase
//! order). One implementation so every document reconciles identically:
//!
//!   * line **amount** = quantity × unit price,
//!   * **subtotal** = sum of the line amounts,
//!   * an optional **discount** comes off the subtotal,
//!   * **tax** is charged on the discounted subtotal, and
//!   * the **total** = discounted subtotal + tax.
//!
//! Every figure is rounded to cents so the displayed lines, subtotal, discount,
//! tax, and total always add up. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LineItem {
    pub description: String,
    pub quantity: f64,
    pub unit_price_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LineResult {
    pub description: String,
    pub quantity: f64,
    pub unit_price_usd: f64,
    pub amount_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LineTotals {
    pub lines: Vec<LineResult>,
    pub subtotal_usd: f64,
    pub discount_pct: f64,
    pub discount_amount_usd: f64,
    pub tax_rate_pct: f64,
    pub tax_amount_usd: f64,
    pub total_usd: f64,
}

/// Round to whole cents.
pub fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

/// Compute per-line amounts and the subtotal/discount/tax/total. Negative
/// discount/tax rates are clamped to zero.
pub fn compute(items: &[LineItem], discount_pct: f64, tax_rate_pct: f64) -> LineTotals {
    let lines: Vec<LineResult> = items
        .iter()
        .map(|li| LineResult {
            description: li.description.clone(),
            quantity: li.quantity,
            unit_price_usd: li.unit_price_usd,
            amount_usd: cents(li.quantity * li.unit_price_usd),
        })
        .collect();

    let subtotal = cents(lines.iter().map(|l| l.amount_usd).sum());
    let discount_pct = discount_pct.max(0.0);
    let discount_amount = cents(subtotal * discount_pct / 100.0);
    let taxable = subtotal - discount_amount;
    let tax_rate = tax_rate_pct.max(0.0);
    let tax_amount = cents(taxable * tax_rate / 100.0);
    let total = cents(taxable + tax_amount);

    LineTotals {
        lines,
        subtotal_usd: subtotal,
        discount_pct,
        discount_amount_usd: discount_amount,
        tax_rate_pct: tax_rate,
        tax_amount_usd: tax_amount,
        total_usd: total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(desc: &str, qty: f64, price: f64) -> LineItem {
        LineItem { description: desc.into(), quantity: qty, unit_price_usd: price }
    }

    #[test]
    fn subtotal_sums_line_amounts() {
        let t = compute(&[item("a", 10.0, 150.0), item("b", 2.0, 75.0)], 0.0, 0.0);
        assert!((t.subtotal_usd - 1_650.0).abs() < 1e-9);
        assert!((t.total_usd - 1_650.0).abs() < 1e-9);
    }

    #[test]
    fn discount_then_tax_on_discounted_base() {
        // 1,650 − 10% (165) = 1,485 taxable; 8.25% = 122.5125 → 122.51; total 1,607.51.
        let t = compute(&[item("a", 10.0, 150.0), item("b", 2.0, 75.0)], 10.0, 8.25);
        assert!((t.discount_amount_usd - 165.0).abs() < 1e-9);
        assert!((t.tax_amount_usd - 122.51).abs() < 1e-9);
        assert!((t.total_usd - 1_607.51).abs() < 1e-9);
    }

    #[test]
    fn negative_rates_clamped() {
        let t = compute(&[item("a", 1.0, 100.0)], -5.0, -3.0);
        assert!((t.discount_amount_usd - 0.0).abs() < 1e-9);
        assert!((t.tax_amount_usd - 0.0).abs() < 1e-9);
        assert!((t.total_usd - 100.0).abs() < 1e-9);
    }
}
