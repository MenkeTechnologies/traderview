//! Opening Range Breakout — Toby Crabel's canonical day-trade setup.
//!
//! Treats `bars[0..opening_bars]` as the session's opening range. Entry
//! fires on the FIRST close that breaks the OR high (long) or OR low
//! (short), provided RVOL on that bar is at least `rvol_min`. Once a
//! signal fires the same direction won't re-trigger within this window —
//! the engine moves to exit-management.
//!
//! Exit: ATR(14) × atr_stop_mult trailing stop, anchored to high-water
//! mark (long) or low-water mark (short).

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::indicators;
use crate::models::PriceBar;
use crate::opening_range::{self, OhlcBar, OrbConfig};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    /// How many bars make up the opening range. Default 15 (15 × 1-min
    /// bars = 15-minute ORB). For 10s bars set ~90 (15 min).
    pub opening_bars: usize,
    /// Reject wick-only pierces — only count CLOSE-based breakouts.
    pub close_only: bool,
    pub rvol_lookback: usize,
    pub rvol_min: f64,
    pub atr_period: usize,
    pub atr_stop_mult: f64,
    pub atr_take_profit_mult: f64,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            opening_bars: 15,
            close_only: true,
            rvol_lookback: 20,
            rvol_min: 1.5,
            atr_period: 14,
            atr_stop_mult: 2.0,
            atr_take_profit_mult: 3.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Orb { pub rules: Rules }

impl Orb {
    pub fn new(rules: Rules) -> Self { Self { rules } }
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone()).unwrap_or_default();
        Self { rules }
    }
}

fn ohlc_bars(bars: &[PriceBar]) -> Vec<OhlcBar> {
    use rust_decimal::prelude::ToPrimitive;
    bars.iter()
        .map(|b| OhlcBar {
            high: b.high.to_f64().unwrap_or(0.0),
            low: b.low.to_f64().unwrap_or(0.0),
            close: b.close.to_f64().unwrap_or(0.0),
        })
        .collect()
}

fn rolling_rvol(volumes: &[f64], i: usize, lookback: usize) -> Option<f64> {
    if i < lookback { return None; }
    let avg = volumes[i - lookback..i].iter().sum::<f64>() / lookback as f64;
    if avg <= 0.0 { return None; }
    Some(volumes[i] / avg)
}

impl Strategy for Orb {
    fn kind(&self) -> StrategyKind { StrategyKind::Orb }

    fn min_bars(&self) -> usize {
        self.rules.opening_bars
            .max(self.rules.rvol_lookback + 1)
            .max(self.rules.atr_period + 1)
            + 1
    }

    fn evaluate_entry(&self, bars: &[PriceBar], side_mode: SideMode) -> Option<EntrySignal> {
        if bars.len() < self.min_bars() {
            return None;
        }
        let cfg = OrbConfig {
            opening_bars: self.rules.opening_bars,
            atr: 0.0,
            close_only: self.rules.close_only,
        };
        let report = opening_range::detect(&ohlc_bars(bars), &cfg);
        if report.opening_range <= 0.0 {
            return None;
        }
        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let vols = indicators::volumes(bars);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);

        let i = bars.len() - 1;
        let atr_now = atr.get(i).copied().flatten()?;
        if atr_now <= 0.0 { return None; }
        let close_now = closes[i];
        let rvol_now = rolling_rvol(&vols, i, self.rules.rvol_lookback)?;
        if rvol_now < self.rules.rvol_min { return None; }

        // Only fire on the FIRST breakout bar — `report.upper_break.bar_index`
        // must be the LATEST bar for the signal to be fresh. Stops the engine
        // from re-entering N times after the breakout already happened.
        let upper_fresh = report
            .upper_break
            .map(|b| b.bar_index == i)
            .unwrap_or(false);
        let lower_fresh = report
            .lower_break
            .map(|b| b.bar_index == i)
            .unwrap_or(false);

        let want_long = matches!(side_mode, SideMode::Long | SideMode::Both) && upper_fresh;
        let want_short = matches!(side_mode, SideMode::Short | SideMode::Both) && lower_fresh;

        if want_long {
            let stop = close_now - self.rules.atr_stop_mult * atr_now;
            let tp = close_now + self.rules.atr_take_profit_mult * atr_now;
            let stop_distance = (close_now - stop).max(0.01);
            Some(EntrySignal {
                side: Side::Buy,
                entry_price: close_now,
                stop_distance,
                trigger_index: i,
                stop_price: stop.max(0.01),
                take_profit_price: tp,
                kind: "orb",
                diagnostic: serde_json::json!({
                    "opening_high": report.opening_high,
                    "opening_low": report.opening_low,
                    "opening_range": report.opening_range,
                    "rvol": rvol_now,
                    "atr": atr_now,
                }),
            })
        } else if want_short {
            let stop = close_now + self.rules.atr_stop_mult * atr_now;
            let tp = (close_now - self.rules.atr_take_profit_mult * atr_now).max(0.01);
            let stop_distance = (stop - close_now).max(0.01);
            Some(EntrySignal {
                side: Side::Sell,
                entry_price: close_now,
                stop_distance,
                trigger_index: i,
                stop_price: stop,
                take_profit_price: tp,
                kind: "orb",
                diagnostic: serde_json::json!({
                    "opening_high": report.opening_high,
                    "opening_low": report.opening_low,
                    "opening_range": report.opening_range,
                    "rvol": rvol_now,
                    "atr": atr_now,
                }),
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
        if bars.len() < self.min_bars() {
            return None;
        }
        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);
        let i = bars.len() - 1;
        let atr_now = atr.get(i).copied().flatten()?;

        match side {
            Side::Buy => {
                let stop = anchor_high - self.rules.atr_stop_mult * atr_now;
                if lows[i] <= stop {
                    return Some(ExitSignal {
                        reason: "atr_trailing_stop",
                        exit_price: stop.max(0.01),
                        trigger_index: i,
                    });
                }
            }
            Side::Sell => {
                let stop = anchor_low + self.rules.atr_stop_mult * atr_now;
                if highs[i] >= stop {
                    return Some(ExitSignal {
                        reason: "atr_trailing_stop",
                        exit_price: stop,
                        trigger_index: i,
                    });
                }
            }
        }
        let _ = closes;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BarInterval;
    use chrono::{TimeZone, Utc};
    use rust_decimal::Decimal;
    use std::str::FromStr;

    fn bar(t: i64, o: &str, h: &str, l: &str, c: &str, v: u64) -> PriceBar {
        PriceBar {
            symbol: "TEST".into(),
            interval: BarInterval::M1,
            bar_time: Utc.timestamp_opt(t, 0).unwrap(),
            open: Decimal::from_str(o).unwrap(),
            high: Decimal::from_str(h).unwrap(),
            low: Decimal::from_str(l).unwrap(),
            close: Decimal::from_str(c).unwrap(),
            volume: Decimal::from(v),
            source: "test".into(),
        }
    }

    /// 15-bar opening range pinned around 100.0 ± 0.5, then several
    /// inside-range bars to build the RVOL baseline, then a single
    /// breakout bar that closes above the OR high with elevated volume.
    fn breakout_window() -> Vec<PriceBar> {
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        // Opening range: 15 bars, highs cluster around 100.4, lows 99.6.
        for i in 0..15 {
            let p = 100.0 + ((i as f64 * 0.7).sin() * 0.4);
            bars.push(bar(
                t,
                &format!("{p:.2}"),
                &format!("{:.2}", p + 0.1),
                &format!("{:.2}", p - 0.1),
                &format!("{p:.2}"),
                1_000_000,
            ));
            t += 60;
        }
        // 20 inside-range bars — used by RVOL baseline + so the breakout
        // isn't on the very first post-OR bar (more realistic).
        for i in 0..20 {
            let p = 100.0 + ((i as f64 * 0.5).cos() * 0.3);
            bars.push(bar(
                t,
                &format!("{p:.2}"),
                &format!("{:.2}", p + 0.1),
                &format!("{:.2}", p - 0.1),
                &format!("{p:.2}"),
                1_000_000,
            ));
            t += 60;
        }
        // Breakout bar — closes ABOVE the OR high (~100.4) with 3× volume.
        bars.push(bar(t, "100.3", "101.2", "100.2", "101.0", 4_000_000));
        bars
    }

    #[test]
    fn entry_fires_on_first_close_break_above_or_high() {
        let strat = Orb::new(Rules::default());
        let bars = breakout_window();
        let sig = strat
            .evaluate_entry(&bars, SideMode::Long)
            .expect("breakout bar must produce long entry");
        assert_eq!(sig.side, Side::Buy);
        assert_eq!(sig.trigger_index, bars.len() - 1);
        let rvol = sig.diagnostic.get("rvol").and_then(|v| v.as_f64()).unwrap();
        assert!(rvol >= 1.5);
    }

    #[test]
    fn entry_blocked_when_breakout_volume_insufficient() {
        let strat = Orb::new(Rules::default());
        let mut bars = breakout_window();
        let last = bars.len() - 1;
        bars[last].volume = Decimal::from(500_000); // below RVOL min
        assert!(
            strat.evaluate_entry(&bars, SideMode::Long).is_none(),
            "low-volume breakout must be vetoed"
        );
    }

    #[test]
    fn entry_blocked_when_close_stays_inside_or_range() {
        let strat = Orb::new(Rules::default());
        let mut bars = breakout_window();
        let last = bars.len() - 1;
        // Pull the close back inside the OR range — no breakout.
        bars[last].close = Decimal::from_str("100.10").unwrap();
        bars[last].high = Decimal::from_str("100.30").unwrap();
        assert!(strat.evaluate_entry(&bars, SideMode::Long).is_none());
    }

    #[test]
    fn entry_only_fires_on_the_breakout_bar_not_later_inside_bars() {
        let strat = Orb::new(Rules::default());
        let mut bars = breakout_window();
        // Add 5 more inside-range bars after the breakout — the strategy
        // must NOT keep firing entries.
        let mut t = bars.last().unwrap().bar_time.timestamp() + 60;
        for _ in 0..5 {
            bars.push(bar(t, "100.5", "100.6", "100.4", "100.5", 1_000_000));
            t += 60;
        }
        assert!(
            strat.evaluate_entry(&bars, SideMode::Long).is_none(),
            "ORB must only fire on the breakout bar itself"
        );
    }

    #[test]
    fn kind_and_min_bars() {
        let strat = Orb::new(Rules::default());
        assert_eq!(strat.kind().as_str(), "orb");
        // opening_bars 15 wins → 16.
        assert_eq!(strat.min_bars(), 22); // rvol_lookback (20) + 1 = 21, opening_bars 15 < 21
    }
}
