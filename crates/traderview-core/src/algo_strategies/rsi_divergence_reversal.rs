//! RSI divergence reversal — exhaustion turns at confirmed swings.
//!
//! Entry:
//!   long  on a CONFIRMED bullish divergence (price lower swing-low,
//!         RSI higher low — selling exhausted) whose confirming swing
//!         is at most `max_age_bars` old, with the divergence low's
//!         RSI below `rsi_oversold` (quality gate: divergences off a
//!         mid-range RSI are noise).
//!   short on the mirrored bearish divergence above `rsi_overbought`.
//!
//! Stop:   beyond the divergence's extreme (the swing that "proved"
//!         exhaustion) minus/plus an ATR buffer — if price takes that
//!         level out, the divergence thesis is dead.
//! Target: ATR-multiple beyond entry.
//!
//! Exit:   RSI reaching the OPPOSITE extreme — the reversal played out.
//!
//! Swing detection, divergence pairing, and RSI all reuse the shared
//! cores (swing_points, rsi_divergence, indicators::rsi).

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::models::PriceBar;
use crate::rsi_divergence::{Divergence, DivergenceKind, PriceRsiPoint};
use crate::{indicators, swing_points};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    pub rsi_period: usize,
    /// Bars on each side that must be strictly beyond a swing point.
    pub swing_lookback: usize,
    /// Divergence low must print RSI below this for a long.
    pub rsi_oversold: f64,
    /// Divergence high must print RSI above this for a short.
    pub rsi_overbought: f64,
    /// Maximum bars since the divergence CONFIRMED (swing + lookback).
    pub max_age_bars: usize,
    pub atr_period: usize,
    /// ATR buffer beyond the divergence extreme for the stop.
    pub atr_stop_mult: f64,
    pub atr_take_profit_mult: f64,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            rsi_period: 14,
            swing_lookback: 5,
            rsi_oversold: 35.0,
            rsi_overbought: 65.0,
            max_age_bars: 4,
            atr_period: 14,
            atr_stop_mult: 1.0,
            atr_take_profit_mult: 3.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RsiDivergenceReversal {
    pub rules: Rules,
}

impl RsiDivergenceReversal {
    pub fn new(rules: Rules) -> Self {
        Self { rules }
    }
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone()).unwrap_or_default();
        Self { rules }
    }

    /// Confirmed divergences over the window, via the shared cores.
    fn divergences(&self, bars: &[PriceBar]) -> Vec<Divergence> {
        let swing_bars: Vec<swing_points::Bar> = bars
            .iter()
            .map(|b| swing_points::Bar {
                high: b.high.to_f64().unwrap_or(0.0),
                low: b.low.to_f64().unwrap_or(0.0),
            })
            .collect();
        let swings = swing_points::detect(&swing_bars, self.rules.swing_lookback);
        let closes = indicators::closes(bars);
        let rsi = indicators::rsi(&closes, self.rules.rsi_period);
        // The detector compares swing prices to swing RSI — build the
        // series from the swings themselves (price = the swing's own
        // high/low, RSI at that bar).
        let series: Vec<PriceRsiPoint> = swings
            .iter()
            .filter_map(|s| {
                Some(PriceRsiPoint {
                    bar_index: s.index,
                    price: s.price,
                    rsi: rsi.get(s.index).copied().flatten()?,
                })
            })
            .collect();
        crate::rsi_divergence::detect(&swings, &series)
    }
}

impl Strategy for RsiDivergenceReversal {
    fn kind(&self) -> StrategyKind {
        StrategyKind::RsiDivergence
    }

    fn min_bars(&self) -> usize {
        self.rules.rsi_period + self.rules.swing_lookback * 4 + 5
    }

    fn evaluate_entry(&self, bars: &[PriceBar], side_mode: SideMode) -> Option<EntrySignal> {
        if bars.len() < self.min_bars() {
            return None;
        }
        let i = bars.len() - 1;
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let closes = indicators::closes(bars);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);
        let atr_now = atr.get(i).copied().flatten()?;
        if atr_now <= 0.0 {
            return None;
        }
        let close_f = bars[i].close.to_f64().unwrap_or(0.0);

        for d in self.divergences(bars).iter().rev() {
            // Freshness: the divergence confirms swing_lookback bars
            // after its closing swing; stale setups don't fire.
            let confirmed_at = d.to_bar + self.rules.swing_lookback;
            if confirmed_at > i || i - confirmed_at > self.rules.max_age_bars {
                continue;
            }
            match d.kind {
                DivergenceKind::Bullish
                    if matches!(side_mode, SideMode::Long | SideMode::Both)
                        && d.to_rsi < self.rules.rsi_oversold =>
                {
                    let stop = (d.to_price - self.rules.atr_stop_mult * atr_now).max(0.01);
                    return Some(EntrySignal {
                        side: Side::Buy,
                        entry_price: close_f,
                        stop_distance: (close_f - stop).max(0.01),
                        trigger_index: i,
                        stop_price: stop,
                        take_profit_price: close_f + self.rules.atr_take_profit_mult * atr_now,
                        kind: "rsi_divergence",
                        diagnostic: serde_json::json!({
                            "from_price": d.from_price, "to_price": d.to_price,
                            "from_rsi": d.from_rsi, "to_rsi": d.to_rsi,
                            "confirmed_at": confirmed_at, "atr": atr_now,
                        }),
                    });
                }
                DivergenceKind::Bearish
                    if matches!(side_mode, SideMode::Short | SideMode::Both)
                        && d.to_rsi > self.rules.rsi_overbought =>
                {
                    let stop = d.to_price + self.rules.atr_stop_mult * atr_now;
                    return Some(EntrySignal {
                        side: Side::Sell,
                        entry_price: close_f,
                        stop_distance: (stop - close_f).max(0.01),
                        trigger_index: i,
                        stop_price: stop,
                        take_profit_price: (close_f
                            - self.rules.atr_take_profit_mult * atr_now)
                            .max(0.01),
                        kind: "rsi_divergence",
                        diagnostic: serde_json::json!({
                            "from_price": d.from_price, "to_price": d.to_price,
                            "from_rsi": d.from_rsi, "to_rsi": d.to_rsi,
                            "confirmed_at": confirmed_at, "atr": atr_now,
                        }),
                    });
                }
                _ => {}
            }
        }
        None
    }

    fn evaluate_exit(
        &self,
        bars: &[PriceBar],
        side: Side,
        _anchor_high: f64,
        _anchor_low: f64,
    ) -> Option<ExitSignal> {
        if bars.len() <= self.rules.rsi_period {
            return None;
        }
        let closes = indicators::closes(bars);
        let rsi = indicators::rsi(&closes, self.rules.rsi_period);
        let i = bars.len() - 1;
        let rsi_now = rsi.get(i).copied().flatten()?;
        let close_f = bars[i].close.to_f64().unwrap_or(0.0);
        match side {
            Side::Buy if rsi_now >= self.rules.rsi_overbought => Some(ExitSignal {
                reason: "rsi_reached_overbought",
                exit_price: close_f,
                trigger_index: i,
            }),
            Side::Sell if rsi_now <= self.rules.rsi_oversold => Some(ExitSignal {
                reason: "rsi_reached_oversold",
                exit_price: close_f,
                trigger_index: i,
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algo_strategies::from_kind;
    use crate::models::BarInterval;
    use chrono::{TimeZone, Utc};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn bar(t: i64, p: f64) -> PriceBar {
        PriceBar {
            symbol: "TEST".into(),
            interval: BarInterval::M1,
            bar_time: Utc.timestamp_opt(t, 0).unwrap(),
            open: Decimal::from_str(&format!("{p:.4}")).unwrap(),
            high: Decimal::from_str(&format!("{:.4}", p + 0.2)).unwrap(),
            low: Decimal::from_str(&format!("{:.4}", p - 0.2)).unwrap(),
            close: Decimal::from_str(&format!("{p:.4}")).unwrap(),
            volume: Decimal::from(1_000_000u64),
            source: "test".into(),
        }
    }

    /// Pre-verified numerically against the shared cores: swing lows at
    /// bars 19 (low 79.8, RSI 0.0) and 35 (low 78.8, RSI 19.5) — lower
    /// price low, higher RSI low, confirming at bar 40 of 44.
    fn bullish_divergence_window() -> Vec<PriceBar> {
        let mut closes: Vec<f64> = vec![100.0; 10];
        closes.extend((0..10).map(|i| 100.0 - 2.0 * (i + 1) as f64)); // steep to 80
        closes.extend((0..6).map(|i| 80.0 + (i + 1) as f64)); // bounce to 86
        closes.extend((0..10).map(|i| 86.0 - 0.7 * (i + 1) as f64)); // grind to 79
        closes.extend((0..8).map(|i| 79.0 + 0.8 * (i + 1) as f64)); // confirm
        let mut t = 1_700_000_000_i64;
        closes
            .into_iter()
            .map(|p| {
                let b = bar(t, p);
                t += 60;
                b
            })
            .collect()
    }

    fn first_long(bars: &[PriceBar], strat: &RsiDivergenceReversal) -> Option<EntrySignal> {
        for end in strat.min_bars()..=bars.len() {
            if let Some(s) = strat.evaluate_entry(&bars[..end], SideMode::Long) {
                return Some(s);
            }
        }
        None
    }

    #[test]
    fn confirmed_bullish_divergence_fires_long_below_oversold() {
        let bars = bullish_divergence_window();
        let strat = RsiDivergenceReversal::new(Rules::default());
        let sig = first_long(&bars, &strat).expect("verified divergence must fire");
        assert_eq!(sig.side, Side::Buy);
        let to_rsi = sig.diagnostic.get("to_rsi").and_then(|v| v.as_f64()).unwrap();
        let from_rsi = sig.diagnostic.get("from_rsi").and_then(|v| v.as_f64()).unwrap();
        assert!(to_rsi > from_rsi, "RSI must improve at the lower low");
        assert!(to_rsi < 35.0, "quality gate: divergence low under oversold");
        // Stop sits beyond the divergence low — thesis-invalidation level.
        let to_price = sig.diagnostic.get("to_price").and_then(|v| v.as_f64()).unwrap();
        assert!(sig.stop_price < to_price);
    }

    #[test]
    fn stale_divergence_does_not_fire() {
        // Extend the same window far past max_age_bars: the only entry
        // windows that fire are within the freshness budget.
        let mut bars = bullish_divergence_window();
        let mut t = 1_700_000_000_i64 + 60 * bars.len() as i64;
        for i in 0..20 {
            bars.push(bar(t, 85.4 + 0.1 * i as f64));
            t += 60;
        }
        let strat = RsiDivergenceReversal::new(Rules::default());
        // Full-window evaluation (oldest divergence now 20+ bars stale).
        assert!(strat.evaluate_entry(&bars, SideMode::Long).is_none());
    }

    #[test]
    fn monotone_trend_has_no_divergence() {
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        for i in 0..80 {
            bars.push(bar(t, 100.0 + 0.3 * i as f64 + 0.005 * (i * i) as f64));
            t += 60;
        }
        let strat = RsiDivergenceReversal::new(Rules::default());
        assert!(first_long(&bars, &strat).is_none());
    }

    #[test]
    fn long_exits_when_rsi_reaches_overbought() {
        // Sustained rally drives Wilder RSI through 65.
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        let strat = RsiDivergenceReversal::new(Rules::default());
        let mut exited = None;
        for i in 0..40 {
            bars.push(bar(t, 100.0 + 0.8 * i as f64));
            t += 60;
            if bars.len() > strat.rules.rsi_period {
                if let Some(e) = strat.evaluate_exit(&bars, Side::Buy, 0.0, 0.0) {
                    exited = Some(e);
                    break;
                }
            }
        }
        assert_eq!(exited.expect("rally must exit").reason, "rsi_reached_overbought");
    }

    #[test]
    fn factory_reaches_rsi_divergence() {
        let s = from_kind("rsi_divergence", &serde_json::json!({})).expect("registered");
        assert_eq!(s.kind(), StrategyKind::RsiDivergence);
    }
}
