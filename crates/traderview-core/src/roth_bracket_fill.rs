//! Roth bracket-fill — convert just enough to top off a tax bracket.
//!
//! A common Roth-conversion strategy: in a low-income year, convert
//! traditional IRA dollars to Roth up to the **top of your current tax
//! bracket**, but not a dollar more (which would spill into the next, higher
//! bracket). This fills the cheap bracket with conversion income now to avoid
//! larger RMDs taxed at higher rates later.
//!
//!   * headroom = bracket ceiling − current taxable income
//!   * conversion = headroom, capped at the traditional balance
//!   * tax = conversion × the bracket's marginal rate
//!
//! The bracket ceiling and rate are inputs (look up your year's brackets), so
//! the calc stays accurate as brackets change. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct BracketFillInput {
    pub current_taxable_income_usd: f64,
    /// Top of the bracket you want to fill (taxable-income ceiling).
    pub bracket_ceiling_usd: f64,
    /// The bracket's marginal rate (e.g. 12 or 22).
    pub marginal_rate_pct: f64,
    /// Available traditional balance to convert (0 ⇒ no cap, use full headroom).
    #[serde(default)]
    pub traditional_balance_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BracketFillResult {
    /// Room in the bracket before the conversion.
    pub headroom_usd: f64,
    /// Amount to convert (headroom, capped at the traditional balance).
    pub conversion_amount_usd: f64,
    /// Tax owed on the conversion this year.
    pub conversion_tax_usd: f64,
    /// Taxable income after the conversion.
    pub new_taxable_income_usd: f64,
    /// True when income already meets/exceeds the ceiling (no room).
    pub already_at_ceiling: bool,
}

pub fn analyze(i: &BracketFillInput) -> BracketFillResult {
    let headroom = (i.bracket_ceiling_usd - i.current_taxable_income_usd).max(0.0);
    let already = headroom <= 0.0;

    // Cap at the balance only when a positive balance is supplied.
    let conversion = if i.traditional_balance_usd > 0.0 {
        headroom.min(i.traditional_balance_usd)
    } else {
        headroom
    };
    let tax = conversion * i.marginal_rate_pct / 100.0;

    BracketFillResult {
        headroom_usd: headroom,
        conversion_amount_usd: conversion,
        conversion_tax_usd: tax,
        new_taxable_income_usd: i.current_taxable_income_usd + conversion,
        already_at_ceiling: already,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> BracketFillInput {
        BracketFillInput {
            current_taxable_income_usd: 60_000.0,
            bracket_ceiling_usd: 100_000.0,
            marginal_rate_pct: 22.0,
            traditional_balance_usd: 0.0,
        }
    }

    #[test]
    fn headroom_is_ceiling_minus_income() {
        let r = analyze(&base());
        assert!((r.headroom_usd - 40_000.0).abs() < 1e-6);
    }

    #[test]
    fn conversion_fills_headroom_without_balance_cap() {
        let r = analyze(&base());
        assert!((r.conversion_amount_usd - 40_000.0).abs() < 1e-6);
    }

    #[test]
    fn tax_is_conversion_times_rate() {
        // 40k × 22% = 8.8k.
        let r = analyze(&base());
        assert!((r.conversion_tax_usd - 8_800.0).abs() < 1e-6);
    }

    #[test]
    fn balance_caps_the_conversion() {
        let r = analyze(&BracketFillInput { traditional_balance_usd: 25_000.0, ..base() });
        assert!((r.conversion_amount_usd - 25_000.0).abs() < 1e-6);
        assert!((r.conversion_tax_usd - 25_000.0 * 0.22).abs() < 1e-6);
    }

    #[test]
    fn new_taxable_income_after_conversion() {
        let r = analyze(&base());
        assert!((r.new_taxable_income_usd - 100_000.0).abs() < 1e-6); // fills to ceiling
    }

    #[test]
    fn already_at_ceiling_no_room() {
        let r = analyze(&BracketFillInput { current_taxable_income_usd: 105_000.0, ..base() });
        assert!(r.already_at_ceiling);
        assert!(r.headroom_usd.abs() < 1e-9);
        assert!(r.conversion_amount_usd.abs() < 1e-9);
        assert!(r.conversion_tax_usd.abs() < 1e-9);
    }

    #[test]
    fn balance_larger_than_headroom_fills_only_headroom() {
        let r = analyze(&BracketFillInput { traditional_balance_usd: 500_000.0, ..base() });
        assert!((r.conversion_amount_usd - 40_000.0).abs() < 1e-6); // headroom, not balance
    }

    #[test]
    fn effective_rate_matches_marginal() {
        let r = analyze(&base());
        // tax / conversion = marginal rate.
        assert!((r.conversion_tax_usd / r.conversion_amount_usd * 100.0 - 22.0).abs() < 1e-9);
    }
}
