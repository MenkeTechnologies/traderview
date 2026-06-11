//! Cheapest-to-deliver — gross basis and implied repo across a bond
//! futures deliverable basket.
//!
//!   invoice      = futures × conversion_factor + accrued_at_delivery
//!   gross basis  = clean_price − futures × conversion_factor
//!   implied repo = (invoice − (clean + accrued_now))
//!                  / (clean + accrued_now) × 360/days × 100
//!
//! The CTD is the bond with the HIGHEST implied repo — the short
//! delivers whatever finances best, and every other basket member
//! trades as an option-adjusted spread off it. Coupon flows inside the
//! window are out of scope (caller can fold them into accrued).
//!
//! Pure compute. Companion to `curve_trade`, `bond_convexity`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct DeliverableBond {
    pub name: String,
    pub clean_price: f64,
    pub conversion_factor: f64,
    #[serde(default)]
    pub accrued_now: f64,
    #[serde(default)]
    pub accrued_at_delivery: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CtdInput {
    pub futures_price: f64,
    pub days_to_delivery: f64,
    pub basket: Vec<DeliverableBond>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CtdRow {
    pub name: String,
    pub gross_basis: f64,
    pub implied_repo_pct: f64,
    pub is_ctd: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CtdReport {
    pub rows: Vec<CtdRow>,
    pub ctd_name: String,
    pub ctd_implied_repo_pct: f64,
}

pub fn compute(inp: &CtdInput) -> Option<CtdReport> {
    if !inp.futures_price.is_finite()
        || inp.futures_price <= 0.0
        || !inp.days_to_delivery.is_finite()
        || inp.days_to_delivery <= 0.0
        || inp.basket.is_empty()
        || inp.basket.len() > 100
        || inp.basket.iter().any(|b| {
            !b.clean_price.is_finite()
                || b.clean_price <= 0.0
                || !b.conversion_factor.is_finite()
                || b.conversion_factor <= 0.0
                || !b.accrued_now.is_finite()
                || b.accrued_now < 0.0
                || !b.accrued_at_delivery.is_finite()
                || b.accrued_at_delivery < 0.0
                || b.name.trim().is_empty()
        })
    {
        return None;
    }
    let ann = 360.0 / inp.days_to_delivery;
    let mut rows: Vec<CtdRow> = inp
        .basket
        .iter()
        .map(|b| {
            let invoice = inp.futures_price * b.conversion_factor + b.accrued_at_delivery;
            let cost = b.clean_price + b.accrued_now;
            CtdRow {
                name: b.name.clone(),
                gross_basis: b.clean_price - inp.futures_price * b.conversion_factor,
                implied_repo_pct: (invoice - cost) / cost * ann * 100.0,
                is_ctd: false,
            }
        })
        .collect();
    let ctd_idx = rows
        .iter()
        .enumerate()
        .max_by(|a, b| {
            a.1.implied_repo_pct
                .partial_cmp(&b.1.implied_repo_pct)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)?;
    rows[ctd_idx].is_ctd = true;
    Some(CtdReport {
        ctd_name: rows[ctd_idx].name.clone(),
        ctd_implied_repo_pct: rows[ctd_idx].implied_repo_pct,
        rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bond(name: &str, clean: f64, cf: f64) -> DeliverableBond {
        DeliverableBond {
            name: name.into(),
            clean_price: clean,
            conversion_factor: cf,
            accrued_now: 0.0,
            accrued_at_delivery: 0.0,
        }
    }

    #[test]
    fn ctd_is_the_highest_implied_repo() {
        // Futures 110, 90 days (ann factor exactly 4).
        // A: clean 99, CF 0.9 ⇒ invoice 99, basis 0, IRR 0%.
        // B: clean 98, CF 0.9 ⇒ invoice 99, basis −1, IRR (1/98)·4 ≈ 4.08%.
        let r = compute(&CtdInput {
            futures_price: 110.0,
            days_to_delivery: 90.0,
            basket: vec![bond("A", 99.0, 0.9), bond("B", 98.0, 0.9)],
        })
        .unwrap();
        assert_eq!(r.ctd_name, "B");
        let a = &r.rows[0];
        let b = &r.rows[1];
        assert!(a.gross_basis.abs() < 1e-12);
        assert!((b.gross_basis + 1.0).abs() < 1e-12);
        assert!(a.implied_repo_pct.abs() < 1e-12);
        assert!((b.implied_repo_pct - 1.0 / 98.0 * 4.0 * 100.0).abs() < 1e-9);
        assert!(b.is_ctd && !a.is_ctd);
    }

    #[test]
    fn accrued_interest_flows_through_invoice_and_cost() {
        // Same bond with accrued on both sides: invoice gains the
        // delivery accrued, cost gains today's.
        let mut with = bond("A", 99.0, 0.9);
        with.accrued_now = 1.0;
        with.accrued_at_delivery = 2.0;
        let r = compute(&CtdInput {
            futures_price: 110.0,
            days_to_delivery: 90.0,
            basket: vec![with],
        })
        .unwrap();
        // invoice 99 + 2 = 101; cost 100 ⇒ IRR = (1/100)·4 = 4%.
        assert!((r.rows[0].implied_repo_pct - 4.0).abs() < 1e-9);
    }

    #[test]
    fn negative_carry_baskets_still_rank() {
        // Both bonds finance at negative IRR — CTD is the least bad.
        let r = compute(&CtdInput {
            futures_price: 100.0,
            days_to_delivery: 90.0,
            basket: vec![bond("A", 95.0, 0.9), bond("B", 96.0, 0.9)],
        })
        .unwrap();
        assert!(r.ctd_implied_repo_pct < 0.0);
        assert_eq!(r.ctd_name, "A"); // smaller loss on cheaper bond
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&CtdInput {
            futures_price: 110.0,
            days_to_delivery: 90.0,
            basket: vec![],
        })
        .is_none());
        assert!(compute(&CtdInput {
            futures_price: 0.0,
            days_to_delivery: 90.0,
            basket: vec![bond("A", 99.0, 0.9)],
        })
        .is_none());
        assert!(compute(&CtdInput {
            futures_price: 110.0,
            days_to_delivery: 90.0,
            basket: vec![bond("A", 99.0, 0.0)],
        })
        .is_none());
    }
}
