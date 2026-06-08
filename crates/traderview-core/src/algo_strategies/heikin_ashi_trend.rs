//! Heikin-Ashi Trend — noise-filtered trend follower.
//!
//! HA candles smooth out wicks and noise. Entry fires when:
//!   - last `green_run` HA candles are all bullish (long), and
//!   - close > EMA(slow)                                  (trend confirm)
//!
//! Mirror for short with bearish HA run + close < EMA(slow). Exit on
//! first opposing HA candle or close back through EMA(slow).
//!
//! Slower turnover than VWAP/momentum — designed for 5-min+ bars where
//! HA's noise filter actually buys you discretion.

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::heikin_ashi;
use crate::indicators;
use crate::models::PriceBar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    pub ema_slow: usize,
    /// Number of consecutive same-direction HA candles required.
    pub green_run: usize,
    pub atr_period: usize,
    pub atr_stop_mult: f64,
    pub atr_take_profit_mult: f64,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            ema_slow: 21,
            green_run: 3,
            atr_period: 14,
            atr_stop_mult: 2.0,
            atr_take_profit_mult: 4.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HeikinAshiTrend {
    pub rules: Rules,
}

impl HeikinAshiTrend {
    pub fn new(rules: Rules) -> Self {
        Self { rules }
    }
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone()).unwrap_or_default();
        Self { rules }
    }
}

fn ha_bars(bars: &[PriceBar]) -> Vec<heikin_ashi::Bar> {
    use rust_decimal::prelude::ToPrimitive;
    bars.iter()
        .map(|b| heikin_ashi::Bar {
            open: b.open.to_f64().unwrap_or(0.0),
            high: b.high.to_f64().unwrap_or(0.0),
            low: b.low.to_f64().unwrap_or(0.0),
            close: b.close.to_f64().unwrap_or(0.0),
        })
        .collect()
}

impl Strategy for HeikinAshiTrend {
    fn kind(&self) -> StrategyKind {
        StrategyKind::HeikinAshiTrend
    }

    fn min_bars(&self) -> usize {
        self.rules.ema_slow.max(self.rules.atr_period + 1) + self.rules.green_run
    }

    fn evaluate_entry(&self, bars: &[PriceBar], side_mode: SideMode) -> Option<EntrySignal> {
        if bars.len() < self.min_bars() {
            return None;
        }
        let ha = heikin_ashi::compute(&ha_bars(bars));
        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let ema = indicators::ema(&closes, self.rules.ema_slow);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);

        let i = bars.len() - 1;
        let close_now = closes[i];
        let atr_now = atr.get(i).copied().flatten()?;
        if atr_now <= 0.0 {
            return None;
        }
        let ema_now = ema.get(i).copied().flatten()?;

        let last_n = &ha[i + 1 - self.rules.green_run..=i];
        let all_bull = last_n.iter().all(|h| h.is_bull());
        let all_bear = last_n.iter().all(|h| h.is_bear());

        let want_long =
            matches!(side_mode, SideMode::Long | SideMode::Both) && all_bull && close_now > ema_now;
        let want_short = matches!(side_mode, SideMode::Short | SideMode::Both)
            && all_bear
            && close_now < ema_now;

        if want_long {
            let stop = close_now - self.rules.atr_stop_mult * atr_now;
            let stop_distance = (close_now - stop).max(0.01);
            Some(EntrySignal {
                side: Side::Buy,
                entry_price: close_now,
                stop_distance,
                trigger_index: i,
                stop_price: stop.max(0.01),
                take_profit_price: close_now + self.rules.atr_take_profit_mult * atr_now,
                kind: "heikin_ashi_trend",
                diagnostic: serde_json::json!({
                    "ha_run_len": self.rules.green_run,
                    "ema_slow": ema_now,
                    "atr": atr_now,
                }),
            })
        } else if want_short {
            let stop = close_now + self.rules.atr_stop_mult * atr_now;
            let stop_distance = (stop - close_now).max(0.01);
            Some(EntrySignal {
                side: Side::Sell,
                entry_price: close_now,
                stop_distance,
                trigger_index: i,
                stop_price: stop,
                take_profit_price: (close_now - self.rules.atr_take_profit_mult * atr_now)
                    .max(0.01),
                kind: "heikin_ashi_trend",
                diagnostic: serde_json::json!({
                    "ha_run_len": self.rules.green_run,
                    "ema_slow": ema_now,
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
        let ha = heikin_ashi::compute(&ha_bars(bars));
        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let ema = indicators::ema(&closes, self.rules.ema_slow);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);

        let i = bars.len() - 1;
        let close_now = closes[i];
        let atr_now = atr.get(i).copied().flatten()?;
        let ema_now = ema.get(i).copied().flatten()?;
        let ha_now = ha.get(i)?;

        match side {
            Side::Buy => {
                let trail = anchor_high - self.rules.atr_stop_mult * atr_now;
                if lows[i] <= trail {
                    return Some(ExitSignal {
                        reason: "atr_trailing_stop",
                        exit_price: trail.max(0.01),
                        trigger_index: i,
                    });
                }
                if ha_now.is_bear() {
                    return Some(ExitSignal {
                        reason: "ha_bear_candle",
                        exit_price: close_now,
                        trigger_index: i,
                    });
                }
                if close_now < ema_now {
                    return Some(ExitSignal {
                        reason: "ema_loss",
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
                if ha_now.is_bull() {
                    return Some(ExitSignal {
                        reason: "ha_bull_candle",
                        exit_price: close_now,
                        trigger_index: i,
                    });
                }
                if close_now > ema_now {
                    return Some(ExitSignal {
                        reason: "ema_gain",
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

    /// Long flat seed (price below EMA), then 10 strongly bullish bars
    /// (each open < close, EMA lifts under price) producing a run of
    /// green HA candles.
    fn ha_uptrend_window() -> Vec<PriceBar> {
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        for _ in 0..30 {
            bars.push(bar(t, "100.00", "100.10", "99.90", "100.00", 1_000_000));
            t += 60;
        }
        for i in 0..10 {
            let p_open = 100.0 + i as f64 * 0.6;
            let p_close = p_open + 0.5;
            bars.push(bar(
                t,
                &format!("{p_open:.2}"),
                &format!("{:.2}", p_close + 0.2),
                &format!("{:.2}", p_open - 0.1),
                &format!("{p_close:.2}"),
                1_500_000,
            ));
            t += 60;
        }
        bars
    }

    fn first_long(bars: &[PriceBar], strat: &HeikinAshiTrend) -> Option<EntrySignal> {
        for end in strat.min_bars()..=bars.len() {
            if let Some(s) = strat.evaluate_entry(&bars[..end], SideMode::Long) {
                return Some(s);
            }
        }
        None
    }

    #[test]
    fn entry_fires_on_green_run_above_ema() {
        let strat = HeikinAshiTrend::new(Rules::default());
        let bars = ha_uptrend_window();
        let sig = first_long(&bars, &strat).expect("HA green run must fire long");
        assert_eq!(sig.side, Side::Buy);
        let ema = sig
            .diagnostic
            .get("ema_slow")
            .and_then(|v| v.as_f64())
            .unwrap();
        assert!(sig.entry_price > ema, "close must be above EMA on entry");
    }

    #[test]
    fn entry_blocked_on_choppy_window_no_green_run() {
        let strat = HeikinAshiTrend::new(Rules::default());
        // Alternating up/down bars — never 3 green HA candles in a row.
        let bars: Vec<PriceBar> = (0..60)
            .map(|i| {
                let (o, c): (f64, f64) = if i % 2 == 0 {
                    (100.0, 99.5)
                } else {
                    (99.5, 100.0)
                };
                bar(
                    1_700_000_000 + i * 60,
                    &format!("{o:.2}"),
                    &format!("{:.2}", o.max(c) + 0.2),
                    &format!("{:.2}", o.min(c) - 0.2),
                    &format!("{c:.2}"),
                    1_000_000,
                )
            })
            .collect();
        assert!(first_long(&bars, &strat).is_none());
    }

    #[test]
    fn kind_and_min_bars() {
        let strat = HeikinAshiTrend::new(Rules::default());
        assert_eq!(strat.kind().as_str(), "heikin_ashi_trend");
        assert_eq!(strat.min_bars(), 24);
    }
}
