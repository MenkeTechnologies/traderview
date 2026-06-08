//! Donchian Trend — Richard Donchian / Turtle Traders style.
//!
//! Entry (long): close > Donchian(entry_period).upper AND ADX(14) >
//! adx_min (filters chop). The "fresh breakout" guard is built into
//! donchian::compute — it sets `upper_breakout` only when the close
//! exceeds the PRIOR window's high, so we don't need a separate
//! freshness check on the engine side.
//!
//! Exit (long): close < Donchian(exit_period).lower OR ATR trailing stop
//! anchored to anchor_high. Mirror for short.
//!
//! Sizing: stop_distance = ATR * atr_stop_mult (initial stop is below
//! Donchian.lower for longs; we use ATR as a uniform stop budget).

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::donchian;
use crate::indicators;
use crate::models::PriceBar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    pub entry_period: usize,
    pub exit_period: usize,
    pub adx_period: usize,
    pub adx_min: f64,
    pub atr_period: usize,
    pub atr_stop_mult: f64,
    pub atr_take_profit_mult: f64,
}

impl Default for Rules {
    fn default() -> Self {
        // Classic turtle: 20-bar entry, 10-bar exit. ADX 14 > 20 chop filter.
        Self {
            entry_period: 20,
            exit_period: 10,
            adx_period: 14,
            adx_min: 20.0,
            atr_period: 14,
            atr_stop_mult: 2.0,
            atr_take_profit_mult: 4.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DonchianTrend { pub rules: Rules }

impl DonchianTrend {
    pub fn new(rules: Rules) -> Self { Self { rules } }
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone()).unwrap_or_default();
        Self { rules }
    }
}

fn donchian_bars(bars: &[PriceBar]) -> Vec<donchian::Bar> {
    use rust_decimal::prelude::ToPrimitive;
    bars.iter()
        .map(|b| donchian::Bar {
            high: b.high.to_f64().unwrap_or(0.0),
            low: b.low.to_f64().unwrap_or(0.0),
            close: b.close.to_f64().unwrap_or(0.0),
        })
        .collect()
}

impl Strategy for DonchianTrend {
    fn kind(&self) -> StrategyKind { StrategyKind::DonchianTrend }

    fn min_bars(&self) -> usize {
        self.rules.entry_period
            .max(self.rules.adx_period * 2 + 1)
            .max(self.rules.atr_period + 1)
            + 1
    }

    fn evaluate_entry(&self, bars: &[PriceBar], side_mode: SideMode) -> Option<EntrySignal> {
        if bars.len() < self.min_bars() {
            return None;
        }
        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let donch = donchian::compute(&donchian_bars(bars), self.rules.entry_period);
        let adx = indicators::adx(&highs, &lows, &closes, self.rules.adx_period);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);

        let i = bars.len() - 1;
        let close_now = closes[i];
        let atr_now = atr.get(i).copied().flatten()?;
        if atr_now <= 0.0 { return None; }
        let adx_now = adx.adx.get(i).copied().flatten()?;
        if adx_now < self.rules.adx_min { return None; }
        let point = donch.get(i)?;
        // The Donchian compute returns DonchianPoint::default() (zeros)
        // for bars before the period is reached — filter those out.
        if point.upper == 0.0 && point.lower == 0.0 {
            return None;
        }

        let want_long = matches!(side_mode, SideMode::Long | SideMode::Both)
            && point.upper_breakout;
        let want_short = matches!(side_mode, SideMode::Short | SideMode::Both)
            && point.lower_breakout;

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
                kind: "donchian_trend",
                diagnostic: serde_json::json!({
                    "donchian_upper": point.upper,
                    "donchian_lower": point.lower,
                    "adx": adx_now,
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
                kind: "donchian_trend",
                diagnostic: serde_json::json!({
                    "donchian_upper": point.upper,
                    "donchian_lower": point.lower,
                    "adx": adx_now,
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
        let donch_exit = donchian::compute(&donchian_bars(bars), self.rules.exit_period);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);
        let i = bars.len() - 1;
        let close_now = closes[i];
        let atr_now = atr.get(i).copied().flatten()?;
        let exit_point = donch_exit.get(i)?;

        match side {
            Side::Buy => {
                // First-to-fire: ATR trailing stop (tighter) OR exit-period
                // Donchian low break (looser, the turtle stop).
                let trail = anchor_high - self.rules.atr_stop_mult * atr_now;
                if lows[i] <= trail {
                    return Some(ExitSignal {
                        reason: "atr_trailing_stop",
                        exit_price: trail.max(0.01),
                        trigger_index: i,
                    });
                }
                if exit_point.lower > 0.0 && close_now < exit_point.lower {
                    return Some(ExitSignal {
                        reason: "donchian_exit_low",
                        exit_price: close_now,
                        trigger_index: i,
                    });
                }
            }
            Side::Sell => {
                let trail = anchor_low + self.rules.atr_stop_mult * atr_now;
                if highs[i] >= trail {
                    return Some(ExitSignal {
                        reason: "atr_trailing_stop",
                        exit_price: trail,
                        trigger_index: i,
                    });
                }
                if exit_point.upper > 0.0 && close_now > exit_point.upper {
                    return Some(ExitSignal {
                        reason: "donchian_exit_high",
                        exit_price: close_now,
                        trigger_index: i,
                    });
                }
            }
        }
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

    /// Long base of stable bars, then a clean uptrend that drives ADX
    /// above 20 and breaks the Donchian(20).upper on the final bar.
    fn turtle_breakout_window() -> Vec<PriceBar> {
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        // 30 sideways bars to establish a Donchian high baseline.
        for i in 0..30 {
            let p = 100.0 + ((i as f64 * 0.3).sin() * 0.2);
            bars.push(bar(
                t,
                &format!("{p:.2}"),
                &format!("{:.2}", p + 0.2),
                &format!("{:.2}", p - 0.2),
                &format!("{p:.2}"),
                1_000_000,
            ));
            t += 60;
        }
        // 35 strong uptrend bars to push ADX > 20.
        for i in 0..35 {
            let p = 100.4 + (i as f64 + 1.0) * 0.4;
            bars.push(bar(
                t,
                &format!("{:.2}", p - 0.2),
                &format!("{:.2}", p + 0.3),
                &format!("{:.2}", p - 0.3),
                &format!("{p:.2}"),
                1_500_000,
            ));
            t += 60;
        }
        bars
    }

    fn first_long(bars: &[PriceBar], strat: &DonchianTrend) -> Option<EntrySignal> {
        for end in strat.min_bars()..=bars.len() {
            if let Some(s) = strat.evaluate_entry(&bars[..end], SideMode::Long) {
                return Some(s);
            }
        }
        None
    }

    #[test]
    fn entry_fires_on_donchian_breakout_with_adx_above_20() {
        let strat = DonchianTrend::new(Rules::default());
        let bars = turtle_breakout_window();
        let sig = first_long(&bars, &strat).expect("turtle setup must fire");
        assert_eq!(sig.side, Side::Buy);
        let adx_v = sig.diagnostic.get("adx").and_then(|v| v.as_f64()).unwrap();
        assert!(adx_v >= 20.0, "ADX {adx_v} should clear chop filter");
    }

    #[test]
    fn entry_blocked_on_pure_sideways_low_adx() {
        let strat = DonchianTrend::new(Rules::default());
        // 100 bars of small noise — ADX stays below 20, no breakouts.
        let bars: Vec<PriceBar> = (0..100)
            .map(|i| {
                let p = 100.0 + ((i as f64 * 0.5).sin() * 0.1);
                bar(
                    1_700_000_000 + i * 60,
                    &format!("{p:.3}"),
                    &format!("{:.3}", p + 0.05),
                    &format!("{:.3}", p - 0.05),
                    &format!("{p:.3}"),
                    1_000_000,
                )
            })
            .collect();
        assert!(first_long(&bars, &strat).is_none());
    }

    #[test]
    fn kind_and_min_bars() {
        let strat = DonchianTrend::new(Rules::default());
        assert_eq!(strat.kind().as_str(), "donchian_trend");
        // adx_period (14) * 2 + 1 = 29 wins → 30.
        assert_eq!(strat.min_bars(), 30);
    }
}
