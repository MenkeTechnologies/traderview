//! Risk computations — R-multiple, position sizing, risk amount derivation.

use crate::models::AssetClass;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Compute the dollar risk on a trade given entry, stop, qty, and asset spec.
///
/// `risk_amount = |entry - stop| * qty * unit_value`
/// where `unit_value` depends on asset class (multiplier for options,
/// tick_value/tick_size for futures, 1 for stocks/forex).
///
/// Direction-symmetric: `|entry - stop|` makes it independent of long/short.
pub fn risk_amount(
    asset_class: AssetClass,
    qty: Decimal,
    entry: Decimal,
    stop: Decimal,
    multiplier: Decimal,
    tick_size: Option<Decimal>,
    tick_value: Option<Decimal>,
) -> Decimal {
    let delta = (entry - stop).abs();
    match asset_class {
        AssetClass::Stock | AssetClass::Forex => delta * qty,
        AssetClass::Option => delta * qty * multiplier,
        AssetClass::Future => match (tick_size, tick_value) {
            (Some(ts), Some(tv)) if !ts.is_zero() => (delta / ts) * tv * qty,
            _ => delta * multiplier * qty,
        },
    }
}

/// Position size to achieve a target dollar risk given entry, stop, and spec.
/// Inverse of [`risk_amount`].
pub fn position_size(
    target_risk: Decimal,
    asset_class: AssetClass,
    entry: Decimal,
    stop: Decimal,
    multiplier: Decimal,
    tick_size: Option<Decimal>,
    tick_value: Option<Decimal>,
) -> Option<Decimal> {
    let delta = (entry - stop).abs();
    if delta.is_zero() {
        return None;
    }
    let per_unit = match asset_class {
        AssetClass::Stock | AssetClass::Forex => delta,
        AssetClass::Option => delta * multiplier,
        AssetClass::Future => match (tick_size, tick_value) {
            (Some(ts), Some(tv)) if !ts.is_zero() => (delta / ts) * tv,
            _ => delta * multiplier,
        },
    };
    if per_unit.is_zero() {
        None
    } else {
        Some(target_risk / per_unit)
    }
}

/// One R-multiple summary across a trade list (delegated to stats).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RiskSummary {
    pub trades_with_r: usize,
    pub avg_r: f64,
    pub max_r: f64,
    pub min_r: f64,
    pub expectancy_r: f64,
}

pub fn risk_summary<'a, I>(trades: I) -> RiskSummary
where
    I: IntoIterator<Item = &'a crate::models::Trade>,
{
    let mut s = RiskSummary {
        max_r: f64::NEG_INFINITY,
        min_r: f64::INFINITY,
        ..Default::default()
    };
    let mut sum = 0.0;
    for t in trades {
        if let Some(r) = t.r_multiple() {
            let r = r.to_string().parse::<f64>().unwrap_or(0.0);
            s.trades_with_r += 1;
            sum += r;
            if r > s.max_r {
                s.max_r = r;
            }
            if r < s.min_r {
                s.min_r = r;
            }
        }
    }
    if s.trades_with_r == 0 {
        return RiskSummary::default();
    }
    s.avg_r = sum / s.trades_with_r as f64;
    s.expectancy_r = s.avg_r;
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    // ─── risk_amount ──────────────────────────────────────────────────────

    #[test]
    fn risk_amount_stock_long_basic() {
        // 100 shares, entry 50, stop 49 → $1 stop × 100 = $100 risk.
        let r = risk_amount(
            AssetClass::Stock,
            d("100"),
            d("50"),
            d("49"),
            Decimal::ONE,
            None,
            None,
        );
        assert_eq!(r, d("100"));
    }

    #[test]
    fn risk_amount_stock_short_uses_abs_delta() {
        // Short at 50, stop at 51 → |50-51|*100 = $100. Sign of side
        // doesn't change the dollar risk.
        let r = risk_amount(
            AssetClass::Stock,
            d("100"),
            d("50"),
            d("51"),
            Decimal::ONE,
            None,
            None,
        );
        assert_eq!(r, d("100"));
    }

    #[test]
    fn risk_amount_option_applies_multiplier() {
        // 1 contract, entry $5, stop $4 → $1 × 1 × 100 = $100.
        let r = risk_amount(
            AssetClass::Option,
            d("1"),
            d("5"),
            d("4"),
            d("100"),
            None,
            None,
        );
        assert_eq!(r, d("100"));
    }

    #[test]
    fn risk_amount_future_uses_tick_size_value() {
        // ES futures: 4 ticks risk × $12.50/tick × 1 contract = $50.
        let r = risk_amount(
            AssetClass::Future,
            d("1"),
            d("4000"),
            d("3999"),
            Decimal::ONE,
            Some(d("0.25")),  // tick_size
            Some(d("12.50")), // tick_value
        );
        assert_eq!(r, d("50.00"), "4 ticks × $12.50 × 1 = $50");
    }

    #[test]
    fn risk_amount_future_falls_back_to_multiplier_when_no_ticks() {
        // No tick spec → delta × multiplier × qty.
        let r = risk_amount(
            AssetClass::Future,
            d("1"),
            d("4000"),
            d("3990"),
            d("50"), // CME E-mini ES multiplier
            None,
            None,
        );
        assert_eq!(r, d("500"), "10 × 50 × 1");
    }

    #[test]
    fn risk_amount_zero_when_stop_equals_entry() {
        let r = risk_amount(
            AssetClass::Stock,
            d("100"),
            d("50"),
            d("50"),
            Decimal::ONE,
            None,
            None,
        );
        assert_eq!(r, Decimal::ZERO);
    }

    // ─── position_size ────────────────────────────────────────────────────

    #[test]
    fn position_size_inverse_of_risk_amount() {
        // $200 risk, $1 stop on stock → 200 shares.
        let qty = position_size(
            d("200"),
            AssetClass::Stock,
            d("50"),
            d("49"),
            Decimal::ONE,
            None,
            None,
        )
        .unwrap();
        assert_eq!(qty, d("200"));
        // Round-trip: risk_amount(qty=200, $1 stop) → $200.
        let back = risk_amount(
            AssetClass::Stock,
            qty,
            d("50"),
            d("49"),
            Decimal::ONE,
            None,
            None,
        );
        assert_eq!(back, d("200"));
    }

    #[test]
    fn position_size_returns_none_when_stop_equals_entry() {
        // Zero-stop trade is unsizable.
        let qty = position_size(
            d("200"),
            AssetClass::Stock,
            d("50"),
            d("50"),
            Decimal::ONE,
            None,
            None,
        );
        assert!(qty.is_none(), "infinite shares for $0 stop is wrong");
    }

    #[test]
    fn position_size_option_accounts_for_multiplier() {
        // $500 risk, $1 stop on options with 100x multiplier → 5 contracts.
        let qty = position_size(
            d("500"),
            AssetClass::Option,
            d("5"),
            d("4"),
            d("100"),
            None,
            None,
        )
        .unwrap();
        assert_eq!(qty, d("5"));
    }

    // ─── risk_summary ─────────────────────────────────────────────────────

    #[test]
    fn risk_summary_empty_input_returns_zeros() {
        let s = risk_summary(std::iter::empty());
        assert_eq!(s.trades_with_r, 0);
        assert_eq!(s.avg_r, 0.0);
        assert_eq!(s.max_r, 0.0); // default, not -Inf
    }
}
