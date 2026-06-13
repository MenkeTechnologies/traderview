//! Markup vs margin — the conversion small businesses get wrong.
//!
//! **Markup** is profit as a percent of **cost**; **margin** is the same
//! profit as a percent of **price**. They are never equal (except at zero):
//! a 50% markup on a $50 cost is a $75 price — but that's only a 33% margin.
//! Pricing off the wrong one quietly erodes profit.
//!
//! Given the unit cost and one of {price, markup%, margin%}, this returns
//! all four figures so the two can be compared directly:
//!
//!   * markup% = profit / cost × 100
//!   * margin% = profit / price × 100
//!   * from markup: price = cost × (1 + markup/100)
//!   * from margin: price = cost / (1 − margin/100)   (margin must be < 100)
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Cost + price are given.
    Price,
    /// Cost + markup% are given.
    Markup,
    /// Cost + margin% are given.
    Margin,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MarkupInput {
    pub mode: Mode,
    pub cost_usd: f64,
    /// Selling price (Price mode).
    #[serde(default)]
    pub price_usd: f64,
    /// Markup percent of cost (Markup mode).
    #[serde(default)]
    pub markup_pct: f64,
    /// Margin percent of price (Margin mode).
    #[serde(default)]
    pub margin_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarkupResult {
    pub cost_usd: f64,
    pub price_usd: f64,
    pub profit_usd: f64,
    pub markup_pct: f64,
    pub margin_pct: f64,
    /// False when the inputs are infeasible (e.g. margin ≥ 100%).
    pub feasible: bool,
}

pub fn analyze(i: &MarkupInput) -> MarkupResult {
    let cost = i.cost_usd.max(0.0);
    let mut feasible = true;

    let price = match i.mode {
        Mode::Price => i.price_usd.max(0.0),
        Mode::Markup => cost * (1.0 + i.markup_pct / 100.0),
        Mode::Margin => {
            // price = cost / (1 − margin); margin ≥ 100% is impossible.
            if i.margin_pct >= 100.0 {
                feasible = false;
                0.0
            } else {
                cost / (1.0 - i.margin_pct / 100.0)
            }
        }
    };

    let profit = price - cost;
    let markup_pct = if cost > 0.0 { profit / cost * 100.0 } else { 0.0 };
    let margin_pct = if price > 0.0 { profit / price * 100.0 } else { 0.0 };

    MarkupResult {
        cost_usd: cost,
        price_usd: price,
        profit_usd: profit,
        markup_pct,
        margin_pct,
        feasible,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn price_mode_computes_markup_and_margin() {
        // cost 60, price 100 → profit 40, markup 66.67%, margin 40%.
        let r = analyze(&MarkupInput {
            mode: Mode::Price,
            cost_usd: 60.0,
            price_usd: 100.0,
            markup_pct: 0.0,
            margin_pct: 0.0,
        });
        assert!((r.profit_usd - 40.0).abs() < 1e-9);
        assert!((r.markup_pct - (40.0 / 60.0 * 100.0)).abs() < 1e-9);
        assert!((r.margin_pct - 40.0).abs() < 1e-9);
    }

    #[test]
    fn markup_mode_recovers_price() {
        // cost 60, markup 66.666…% → price 100.
        let r = analyze(&MarkupInput {
            mode: Mode::Markup,
            cost_usd: 60.0,
            price_usd: 0.0,
            markup_pct: 40.0 / 60.0 * 100.0,
            margin_pct: 0.0,
        });
        assert!((r.price_usd - 100.0).abs() < 1e-9);
        assert!((r.margin_pct - 40.0).abs() < 1e-9);
    }

    #[test]
    fn margin_mode_recovers_price() {
        // cost 60, margin 40% → price = 60 / 0.6 = 100.
        let r = analyze(&MarkupInput {
            mode: Mode::Margin,
            cost_usd: 60.0,
            price_usd: 0.0,
            markup_pct: 0.0,
            margin_pct: 40.0,
        });
        assert!((r.price_usd - 100.0).abs() < 1e-9);
        assert!((r.markup_pct - (40.0 / 60.0 * 100.0)).abs() < 1e-9);
        assert!(r.feasible);
    }

    #[test]
    fn markup_never_equals_margin() {
        // cost 50, price 100 → markup 100%, margin 50%.
        let r = analyze(&MarkupInput {
            mode: Mode::Price,
            cost_usd: 50.0,
            price_usd: 100.0,
            markup_pct: 0.0,
            margin_pct: 0.0,
        });
        assert!((r.markup_pct - 100.0).abs() < 1e-9);
        assert!((r.margin_pct - 50.0).abs() < 1e-9);
    }

    #[test]
    fn margin_at_or_above_100_is_infeasible() {
        let r = analyze(&MarkupInput {
            mode: Mode::Margin,
            cost_usd: 60.0,
            price_usd: 0.0,
            markup_pct: 0.0,
            margin_pct: 100.0,
        });
        assert!(!r.feasible);
        assert!(r.price_usd.abs() < 1e-9);
    }

    #[test]
    fn zero_cost_guards_markup() {
        let r = analyze(&MarkupInput {
            mode: Mode::Price,
            cost_usd: 0.0,
            price_usd: 100.0,
            markup_pct: 0.0,
            margin_pct: 0.0,
        });
        assert!(r.markup_pct.abs() < 1e-9);
        assert!((r.margin_pct - 100.0).abs() < 1e-9); // all profit
    }

    #[test]
    fn markup_to_margin_roundtrip() {
        // From a 25% markup, derive price+margin, then re-derive markup.
        let a = analyze(&MarkupInput {
            mode: Mode::Markup,
            cost_usd: 80.0,
            price_usd: 0.0,
            markup_pct: 25.0,
            margin_pct: 0.0,
        });
        let b = analyze(&MarkupInput {
            mode: Mode::Margin,
            cost_usd: 80.0,
            price_usd: 0.0,
            markup_pct: 0.0,
            margin_pct: a.margin_pct,
        });
        assert!((b.markup_pct - 25.0).abs() < 1e-6);
        assert!((b.price_usd - a.price_usd).abs() < 1e-6);
    }

    #[test]
    fn keystone_double_is_fifty_percent_margin() {
        // Keystone pricing = 100% markup (double cost) → exactly 50% margin.
        let r = analyze(&MarkupInput {
            mode: Mode::Markup,
            cost_usd: 25.0,
            price_usd: 0.0,
            markup_pct: 100.0,
            margin_pct: 0.0,
        });
        assert!((r.price_usd - 50.0).abs() < 1e-9);
        assert!((r.margin_pct - 50.0).abs() < 1e-9);
    }
}
