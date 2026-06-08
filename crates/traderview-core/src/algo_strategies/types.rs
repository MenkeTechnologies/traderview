//! Shared types for the algo strategy family. Every `Strategy` impl
//! returns these so the engine + sizing + persistence layers don't have
//! to care which strategy produced the signal.

use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SideMode {
    Long,
    Short,
    Both,
}

impl Default for SideMode {
    fn default() -> Self { Self::Long }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sizing {
    pub risk_pct_per_trade: f64,
    pub max_pos_pct: f64,
}

impl Default for Sizing {
    fn default() -> Self {
        Self { risk_pct_per_trade: 0.01, max_pos_pct: 0.20 }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EntrySignal {
    pub side: Side,
    pub entry_price: f64,
    /// Implied stop distance per share (used by `size_shares` to back out
    /// position size from the strategy's chosen risk budget). Strategies
    /// that don't use ATR set this to `(entry - stop).abs()`.
    pub stop_distance: f64,
    pub trigger_index: usize,
    pub stop_price: f64,
    pub take_profit_price: f64,
    /// Strategy kind that produced this signal — for journaling.
    pub kind: &'static str,
    /// Strategy-specific diagnostic — RSI / ROC / VWAP / Donchian band,
    /// etc. Free-form JSON so adding a strategy doesn't require a
    /// migration to the wire schema.
    pub diagnostic: Json,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExitSignal {
    pub reason: &'static str,
    pub exit_price: f64,
    pub trigger_index: usize,
}

/// Risk-first share sizing. `stop_distance` is per-share dollars at risk
/// (e.g. `atr * stop_mult` for momentum, or `(entry - vwap).abs() * k` for
/// VWAP scalp). Result is capped by `max_pos_pct * equity / entry`.
pub fn size_shares(
    equity: f64,
    entry_price: f64,
    stop_distance: f64,
    sizing: &Sizing,
) -> u64 {
    if entry_price <= 0.0 || stop_distance <= 0.0 || equity <= 0.0 {
        return 0;
    }
    let risk_dollars = equity * sizing.risk_pct_per_trade;
    let risk_qty = (risk_dollars / stop_distance).floor();
    let cap_qty = ((equity * sizing.max_pos_pct) / entry_price).floor();
    risk_qty.min(cap_qty).max(0.0) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_shares_uses_stop_distance_then_caps() {
        // $100k equity, 1% risk = $1000, stop_distance $2 ⇒ 500 shares.
        // max_pos_pct 0.20 @ $100/sh ⇒ 200-share cap. Cap wins.
        let qty = size_shares(
            100_000.0,
            100.0,
            2.0,
            &Sizing { risk_pct_per_trade: 0.01, max_pos_pct: 0.20 },
        );
        assert_eq!(qty, 200);
    }

    #[test]
    fn size_shares_zero_on_bad_inputs() {
        let s = Sizing::default();
        assert_eq!(size_shares(0.0, 100.0, 1.0, &s), 0);
        assert_eq!(size_shares(10_000.0, 0.0, 1.0, &s), 0);
        assert_eq!(size_shares(10_000.0, 100.0, 0.0, &s), 0);
    }
}
