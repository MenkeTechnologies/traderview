//! Risk computations — R-multiple, position sizing, risk amount derivation.

use crate::models::{AssetClass, TradeSide};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Compute the dollar risk on a trade given entry, stop, qty, and asset spec.
///
/// `risk_amount = |entry - stop| * qty * unit_value`
/// where `unit_value` depends on asset class (multiplier for options,
/// tick_value/tick_size for futures, 1 for stocks/forex).
pub fn risk_amount(
    asset_class: AssetClass,
    _side: TradeSide,
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
