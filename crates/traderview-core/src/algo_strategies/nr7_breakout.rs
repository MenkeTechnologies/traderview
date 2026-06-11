//! NR7 breakout — Toby Crabel's volatility-contraction expansion play.
//!
//! A bar whose true range is the narrowest of the last 7 (NR7, or 4 for
//! NR4) precedes directional expansion at a documented above-base rate.
//! The trade is the BREAKOUT of that narrow bar's range, not the bar
//! itself:
//!
//! Entry:
//!   long  on close breaking ABOVE the narrow bar's high within
//!         `max_age_bars` of the pattern;
//!   short on close breaking BELOW its low.
//!   `require_inside_bar` additionally demands the narrow bar be fully
//!   nested in its predecessor (NR+IB — Crabel's tighter combo).
//!
//! Stop:   the opposite side of the narrow bar — the contraction range
//!         IS the risk; if price traverses the whole bar the expansion
//!         picked the other direction.
//! Target: ATR-multiple beyond entry.
//! Exit:   ATR trailing stop from the high/low watermark (expansion
//!         days are ridden, not faded) — same shape as ORB's exit.
//!
//! Pattern detection reuses the shared range_contraction core.

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::indicators;
use crate::models::PriceBar;
use crate::range_contraction::{self, PatternKind};
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    /// "nr7" (stricter, more predictive) or "nr4".
    pub pattern: String,
    /// Demand the narrow bar also be an inside bar (NR+IB combo).
    pub require_inside_bar: bool,
    /// Breakout must happen within this many bars of the pattern.
    pub max_age_bars: usize,
    pub atr_period: usize,
    /// Trailing-stop distance from the watermark, in ATRs.
    pub atr_stop_mult: f64,
    pub atr_take_profit_mult: f64,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            pattern: "nr7".into(),
            require_inside_bar: false,
            max_age_bars: 2,
            atr_period: 14,
            atr_stop_mult: 2.0,
            atr_take_profit_mult: 3.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Nr7Breakout {
    pub rules: Rules,
}

impl Nr7Breakout {
    pub fn new(rules: Rules) -> Self {
        Self { rules }
    }
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone()).unwrap_or_default();
        Self { rules }
    }

    fn wanted_kind(&self) -> PatternKind {
        if self.rules.pattern == "nr4" {
            PatternKind::Nr4
        } else {
            PatternKind::Nr7
        }
    }
}

fn ohlc(bars: &[PriceBar]) -> Vec<range_contraction::OhlcBar> {
    bars.iter()
        .map(|b| range_contraction::OhlcBar {
            high: b.high.to_f64().unwrap_or(0.0),
            low: b.low.to_f64().unwrap_or(0.0),
            close: b.close.to_f64().unwrap_or(0.0),
        })
        .collect()
}

impl Strategy for Nr7Breakout {
    fn kind(&self) -> StrategyKind {
        StrategyKind::Nr7Breakout
    }

    fn min_bars(&self) -> usize {
        (self.rules.atr_period + 1).max(8) + self.rules.max_age_bars + 1
    }

    fn evaluate_entry(&self, bars: &[PriceBar], side_mode: SideMode) -> Option<EntrySignal> {
        if bars.len() < self.min_bars() {
            return None;
        }
        let i = bars.len() - 1;
        let report = range_contraction::detect(&ohlc(bars));
        let wanted = self.wanted_kind();
        // Latest qualifying pattern STRICTLY before the current bar,
        // within the freshness window.
        let hit = report
            .hits
            .iter()
            .rev()
            .find(|h| h.kind == wanted && h.bar_index < i && i - h.bar_index <= self.rules.max_age_bars)?;
        if self.rules.require_inside_bar
            && !report
                .hits
                .iter()
                .any(|h| h.kind == PatternKind::InsideBar && h.bar_index == hit.bar_index)
        {
            return None;
        }
        let nr_high = bars[hit.bar_index].high.to_f64().unwrap_or(0.0);
        let nr_low = bars[hit.bar_index].low.to_f64().unwrap_or(0.0);

        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);
        let atr_now = atr.get(i).copied().flatten()?;
        if atr_now <= 0.0 {
            return None;
        }
        let close_now = closes[i];
        let diagnostic = serde_json::json!({
            "pattern": self.rules.pattern, "nr_bar": hit.bar_index,
            "nr_high": nr_high, "nr_low": nr_low, "nr_range": hit.range,
            "atr": atr_now,
        });

        if matches!(side_mode, SideMode::Long | SideMode::Both) && close_now > nr_high {
            Some(EntrySignal {
                side: Side::Buy,
                entry_price: close_now,
                stop_distance: (close_now - nr_low).max(0.01),
                trigger_index: i,
                stop_price: nr_low,
                take_profit_price: close_now + self.rules.atr_take_profit_mult * atr_now,
                kind: "nr7_breakout",
                diagnostic,
            })
        } else if matches!(side_mode, SideMode::Short | SideMode::Both) && close_now < nr_low {
            Some(EntrySignal {
                side: Side::Sell,
                entry_price: close_now,
                stop_distance: (nr_high - close_now).max(0.01),
                trigger_index: i,
                stop_price: nr_high,
                take_profit_price: (close_now - self.rules.atr_take_profit_mult * atr_now).max(0.01),
                kind: "nr7_breakout",
                diagnostic,
            })
        } else {
            None
        }
    }

    fn evaluate_exit(
        &self,
        bars: &[PriceBar],
        side: Side,
        anchor_high: f64,
        anchor_low: f64,
    ) -> Option<ExitSignal> {
        if bars.len() < self.rules.atr_period + 1 {
            return None;
        }
        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);
        let i = bars.len() - 1;
        let atr_now = atr.get(i).copied().flatten()?;
        let close_now = closes[i];
        let stopped = match side {
            Side::Buy => close_now < anchor_high - self.rules.atr_stop_mult * atr_now,
            Side::Sell => close_now > anchor_low + self.rules.atr_stop_mult * atr_now,
        };
        stopped.then_some(ExitSignal {
            reason: "atr_trailing_stop",
            exit_price: close_now,
            trigger_index: i,
        })
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

    fn bar(t: i64, h: f64, l: f64, c: f64) -> PriceBar {
        PriceBar {
            symbol: "TEST".into(),
            interval: BarInterval::M1,
            bar_time: Utc.timestamp_opt(t, 0).unwrap(),
            open: Decimal::from_str(&format!("{:.4}", (h + l) / 2.0)).unwrap(),
            high: Decimal::from_str(&format!("{h:.4}")).unwrap(),
            low: Decimal::from_str(&format!("{l:.4}")).unwrap(),
            close: Decimal::from_str(&format!("{c:.4}")).unwrap(),
            volume: Decimal::from(1_000_000u64),
            source: "test".into(),
        }
    }

    /// 20 wide bars (range 1.0) around 100, then a 0.2-range NR7 nested
    /// inside its predecessor, then the breakout bar.
    fn nr7_then(breakout_close: f64) -> Vec<PriceBar> {
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        for _ in 0..20 {
            bars.push(bar(t, 100.5, 99.5, 100.0));
            t += 60;
        }
        bars.push(bar(t, 100.1, 99.9, 100.0)); // NR7 + inside bar
        t += 60;
        bars.push(bar(
            t,
            breakout_close.max(100.1) + 0.1,
            breakout_close.min(99.9) - 0.1,
            breakout_close,
        ));
        bars
    }

    #[test]
    fn breakout_above_nr7_high_fires_long_with_range_stop() {
        let bars = nr7_then(100.6);
        let sig = Nr7Breakout::new(Rules::default())
            .evaluate_entry(&bars, SideMode::Long)
            .expect("close above NR7 high must fire");
        assert_eq!(sig.side, Side::Buy);
        // Stop = the NR bar's LOW: the contraction range is the risk.
        assert!((sig.stop_price - 99.9).abs() < 1e-9);
        let nr_range = sig.diagnostic.get("nr_range").and_then(|v| v.as_f64()).unwrap();
        assert!((nr_range - 0.2).abs() < 1e-6);
    }

    #[test]
    fn breakdown_below_nr7_low_fires_short() {
        let bars = nr7_then(99.4);
        let sig = Nr7Breakout::new(Rules::default())
            .evaluate_entry(&bars, SideMode::Short)
            .expect("close below NR7 low must fire short");
        assert_eq!(sig.side, Side::Sell);
        assert!((sig.stop_price - 100.1).abs() < 1e-9);
    }

    #[test]
    fn close_inside_the_narrow_range_is_no_breakout() {
        let bars = nr7_then(100.0);
        assert!(Nr7Breakout::new(Rules::default())
            .evaluate_entry(&bars, SideMode::Both)
            .is_none());
    }

    #[test]
    fn uniform_ranges_never_qualify() {
        // All bars identical range: the detector's strictly-smaller
        // requirement keeps flat tape from minting patterns.
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        for _ in 0..25 {
            bars.push(bar(t, 100.5, 99.5, 100.4));
            t += 60;
        }
        assert!(Nr7Breakout::new(Rules::default())
            .evaluate_entry(&bars, SideMode::Both)
            .is_none());
    }

    #[test]
    fn stale_pattern_does_not_fire() {
        // Push the breakout past max_age_bars: WIDE drifting bars
        // between the NR7 and the break (narrow fillers would mint
        // fresh NR7 hits of their own and legitimately re-arm).
        let mut bars = nr7_then(100.05); // last bar harmless (inside range)
        let t0 = 1_700_000_000_i64 + 60 * bars.len() as i64;
        for k in 0..2 {
            bars.push(bar(t0 + k * 60, 100.5, 99.5, 100.0));
        }
        bars.push(bar(t0 + 180, 100.9, 100.0, 100.8)); // would-be breakout
        assert!(Nr7Breakout::new(Rules::default())
            .evaluate_entry(&bars, SideMode::Long)
            .is_none());
    }

    #[test]
    fn require_inside_bar_filters_non_nested_nr7() {
        // NR7 bar sits ABOVE its predecessor's high (not nested).
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        for _ in 0..20 {
            bars.push(bar(t, 100.5, 99.5, 100.0));
            t += 60;
        }
        bars.push(bar(t, 100.8, 100.6, 100.7)); // NR7, NOT inside
        t += 60;
        bars.push(bar(t, 101.2, 100.7, 101.1)); // breaks NR high
        let plain = Nr7Breakout::new(Rules::default());
        assert!(plain.evaluate_entry(&bars, SideMode::Long).is_some());
        let strict = Nr7Breakout::new(Rules {
            require_inside_bar: true,
            ..Rules::default()
        });
        assert!(strict.evaluate_entry(&bars, SideMode::Long).is_none());
    }

    #[test]
    fn trailing_stop_exits_off_the_watermark() {
        let mut bars = nr7_then(100.6);
        let t0 = 1_700_000_000_i64 + 60 * bars.len() as i64;
        // Slide well below watermark 103 minus 2 ATRs (~1 range unit).
        for k in 0..3 {
            bars.push(bar(t0 + k * 60, 100.0 - k as f64, 99.0 - k as f64, 99.2 - k as f64));
        }
        let e = Nr7Breakout::new(Rules::default())
            .evaluate_exit(&bars, Side::Buy, 103.0, 99.0)
            .expect("deep slide must trail out");
        assert_eq!(e.reason, "atr_trailing_stop");
    }

    #[test]
    fn factory_reaches_nr7_breakout() {
        let s = from_kind("nr7_breakout", &serde_json::json!({})).expect("registered");
        assert_eq!(s.kind(), StrategyKind::Nr7Breakout);
    }
}
