//! Supertrend Cross — Olivier Seban's ATR-banded trend reversal.
//!
//! Entry fires the bar the Supertrend trend flag flips:
//!   long entry  on `trend[i-1]` == -1 → `trend[i]` == 1
//!   short entry on `trend[i-1]` ==  1 → `trend[i]` == -1
//!
//! Exit: opposite flip OR the supertrend value itself (acts as a trailing
//! stop — every bar a long is open, super_trend = the lower band).

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::indicators;
use crate::models::PriceBar;
use crate::supertrend;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    pub atr_period: usize,
    pub multiplier: f64,
    /// Backup take-profit (ATR multiple) — Supertrend doesn't define one
    /// natively, so we still size against `atr_stop_mult` and target a
    /// reasonable R-multiple beyond entry.
    pub atr_take_profit_mult: f64,
    pub atr_stop_mult: f64,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            atr_period: 10,
            multiplier: 3.0,
            atr_take_profit_mult: 3.0,
            atr_stop_mult: 2.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Supertrend {
    pub rules: Rules,
}

impl Supertrend {
    pub fn new(rules: Rules) -> Self {
        Self { rules }
    }
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone()).unwrap_or_default();
        Self { rules }
    }
}

fn st_bars(bars: &[PriceBar]) -> Vec<supertrend::Bar> {
    use rust_decimal::prelude::ToPrimitive;
    bars.iter()
        .map(|b| supertrend::Bar {
            high: b.high.to_f64().unwrap_or(0.0),
            low: b.low.to_f64().unwrap_or(0.0),
            close: b.close.to_f64().unwrap_or(0.0),
        })
        .collect()
}

fn atr_f64_vec(bars: &[PriceBar], period: usize) -> Vec<f64> {
    let closes = indicators::closes(bars);
    let highs = indicators::highs(bars);
    let lows = indicators::lows(bars);
    indicators::atr(&highs, &lows, &closes, period)
        .into_iter()
        .map(|v| v.unwrap_or(0.0))
        .collect()
}

impl Strategy for Supertrend {
    fn kind(&self) -> StrategyKind {
        StrategyKind::Supertrend
    }

    fn min_bars(&self) -> usize {
        self.rules.atr_period + 3
    }

    fn evaluate_entry(&self, bars: &[PriceBar], side_mode: SideMode) -> Option<EntrySignal> {
        if bars.len() < self.min_bars() {
            return None;
        }
        let atr_v = atr_f64_vec(bars, self.rules.atr_period);
        let report = supertrend::compute(&st_bars(bars), &atr_v, self.rules.multiplier);
        let i = bars.len() - 1;
        let prev = i - 1;

        let trend_now = report.get(i)?.trend;
        let trend_prev = report.get(prev)?.trend;
        if trend_now == 0 || trend_prev == 0 {
            return None;
        }
        let close_now = bars[i].close;
        let close_f = close_now_f64(close_now);
        let st_value = report.get(i)?.super_trend;
        let atr_now = atr_v[i];
        if atr_now <= 0.0 {
            return None;
        }

        let want_long = matches!(side_mode, SideMode::Long | SideMode::Both)
            && trend_prev == -1
            && trend_now == 1;
        let want_short = matches!(side_mode, SideMode::Short | SideMode::Both)
            && trend_prev == 1
            && trend_now == -1;

        if want_long {
            // Stop at supertrend value (the lower band on uptrend),
            // never further than atr_stop_mult * ATR.
            let st_stop = st_value;
            let atr_stop = close_f - self.rules.atr_stop_mult * atr_now;
            let stop = st_stop.max(atr_stop);
            let stop_distance = (close_f - stop).max(0.01);
            Some(EntrySignal {
                side: Side::Buy,
                entry_price: close_f,
                stop_distance,
                trigger_index: i,
                stop_price: stop.max(0.01),
                take_profit_price: close_f + self.rules.atr_take_profit_mult * atr_now,
                kind: "supertrend",
                diagnostic: serde_json::json!({
                    "super_trend": st_value,
                    "trend_prev": trend_prev,
                    "trend_now": trend_now,
                    "atr": atr_now,
                }),
            })
        } else if want_short {
            let st_stop = st_value;
            let atr_stop = close_f + self.rules.atr_stop_mult * atr_now;
            let stop = st_stop.min(atr_stop);
            let stop_distance = (stop - close_f).max(0.01);
            Some(EntrySignal {
                side: Side::Sell,
                entry_price: close_f,
                stop_distance,
                trigger_index: i,
                stop_price: stop,
                take_profit_price: (close_f - self.rules.atr_take_profit_mult * atr_now).max(0.01),
                kind: "supertrend",
                diagnostic: serde_json::json!({
                    "super_trend": st_value,
                    "trend_prev": trend_prev,
                    "trend_now": trend_now,
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
        _anchor_high: f64,
        _anchor_low: f64,
    ) -> Option<ExitSignal> {
        if bars.len() < self.min_bars() {
            return None;
        }
        let atr_v = atr_f64_vec(bars, self.rules.atr_period);
        let report = supertrend::compute(&st_bars(bars), &atr_v, self.rules.multiplier);
        let i = bars.len() - 1;
        let prev = i - 1;
        let trend_now = report.get(i)?.trend;
        let trend_prev = report.get(prev)?.trend;
        let close_f = close_now_f64(bars[i].close);

        match side {
            Side::Buy => {
                if trend_prev == 1 && trend_now == -1 {
                    return Some(ExitSignal {
                        reason: "supertrend_flip",
                        exit_price: close_f,
                        trigger_index: i,
                    });
                }
            }
            Side::Sell => {
                if trend_prev == -1 && trend_now == 1 {
                    return Some(ExitSignal {
                        reason: "supertrend_flip",
                        exit_price: close_f,
                        trigger_index: i,
                    });
                }
            }
        }
        None
    }
}

fn close_now_f64(d: rust_decimal::Decimal) -> f64 {
    use rust_decimal::prelude::ToPrimitive;
    d.to_f64().unwrap_or(0.0)
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

    /// 30 declining bars (Supertrend = -1), then a sharp 8-bar uptrend
    /// that forces the trend to flip.
    fn down_then_up_window() -> Vec<PriceBar> {
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        for i in 0..30 {
            let p = 100.0 - i as f64 * 0.3;
            bars.push(bar(
                t,
                &format!("{p:.2}"),
                &format!("{:.2}", p + 0.1),
                &format!("{:.2}", p - 0.2),
                &format!("{:.2}", p - 0.1),
                1_000_000,
            ));
            t += 60;
        }
        for i in 0..8 {
            let p = 91.0 + (i as f64 + 1.0) * 1.2;
            bars.push(bar(
                t,
                &format!("{:.2}", p - 0.4),
                &format!("{:.2}", p + 0.5),
                &format!("{:.2}", p - 0.5),
                &format!("{p:.2}"),
                1_500_000,
            ));
            t += 60;
        }
        bars
    }

    fn first_long(bars: &[PriceBar], strat: &Supertrend) -> Option<EntrySignal> {
        for end in strat.min_bars()..=bars.len() {
            if let Some(s) = strat.evaluate_entry(&bars[..end], SideMode::Long) {
                return Some(s);
            }
        }
        None
    }

    #[test]
    fn entry_fires_on_supertrend_flip_long() {
        let strat = Supertrend::new(Rules::default());
        let bars = down_then_up_window();
        let sig = first_long(&bars, &strat).expect("trend flip must fire long");
        assert_eq!(sig.side, Side::Buy);
        let prev = sig
            .diagnostic
            .get("trend_prev")
            .and_then(|v| v.as_i64())
            .unwrap();
        let now = sig
            .diagnostic
            .get("trend_now")
            .and_then(|v| v.as_i64())
            .unwrap();
        assert_eq!(prev, -1);
        assert_eq!(now, 1);
    }

    #[test]
    fn entry_blocked_on_persistent_uptrend_no_flip() {
        let strat = Supertrend::new(Rules::default());
        // Pure uptrend — trend is 1 from the first bar, never flips.
        let bars: Vec<PriceBar> = (0..50)
            .map(|i| {
                let p = 100.0 + i as f64 * 0.5;
                bar(
                    1_700_000_000 + i * 60,
                    &format!("{p:.2}"),
                    &format!("{:.2}", p + 0.3),
                    &format!("{:.2}", p - 0.2),
                    &format!("{:.2}", p + 0.1),
                    1_000_000,
                )
            })
            .collect();
        assert!(first_long(&bars, &strat).is_none());
    }

    #[test]
    fn kind_and_min_bars() {
        let strat = Supertrend::new(Rules::default());
        assert_eq!(strat.kind().as_str(), "supertrend");
        assert_eq!(strat.min_bars(), 13);
    }
}
