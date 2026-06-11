//! VWAP Scalp — fade extreme z-score deviations from session VWAP.
//!
//! Distinct from `mean_reversion` (which gates on Connors RSI). This
//! strategy is pure VWAP-mean-reversion: the larger the z-score, the
//! tighter the stop, and the take-profit is always at VWAP.
//!
//! Entry (long):
//!   z-score (close - vwap) / sigma  <=  -z_min
//!   close > prior_low + atr * recovery_buffer   (early-rebound filter
//!                                                to avoid the falling-knife trap)
//!
//! Entry (short): mirror — z >= +z_min and price slightly off the high.
//!
//! Exit: ATR(period) × atr_stop_mult trailing stop OR price crosses VWAP.
//! Tight stops are the defining trait of a scalp.

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::indicators;
use crate::models::PriceBar;
use crate::session_vwap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    pub z_min: f64,
    pub recovery_buffer: f64,
    pub atr_period: usize,
    pub atr_stop_mult: f64,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            // 2.0σ is the mainstream intraday-scalp threshold (Bollinger's
            // own default); 2.5 catches only extreme tails.
            z_min: 2.0,
            recovery_buffer: 0.10,
            atr_period: 14,
            atr_stop_mult: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VwapScalp {
    pub rules: Rules,
}

impl VwapScalp {
    pub fn new(rules: Rules) -> Self {
        Self { rules }
    }
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone()).unwrap_or_default();
        Self { rules }
    }
}

fn session_bars(bars: &[PriceBar]) -> Vec<session_vwap::Bar> {
    use rust_decimal::prelude::ToPrimitive;
    bars.iter()
        .enumerate()
        .map(|(i, b)| session_vwap::Bar {
            high: b.high.to_f64().unwrap_or(0.0),
            low: b.low.to_f64().unwrap_or(0.0),
            close: b.close.to_f64().unwrap_or(0.0),
            volume: b.volume.to_f64().unwrap_or(0.0),
            is_session_start: i == 0,
        })
        .collect()
}

impl Strategy for VwapScalp {
    fn kind(&self) -> StrategyKind {
        StrategyKind::VwapScalp
    }

    fn min_bars(&self) -> usize {
        self.rules.atr_period.max(20) + 2
    }

    fn evaluate_entry(&self, bars: &[PriceBar], side_mode: SideMode) -> Option<EntrySignal> {
        if bars.len() < self.min_bars() {
            return None;
        }
        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);
        let svw = session_vwap::compute(&session_bars(bars));

        let i = bars.len() - 1;
        let prev = i - 1;
        let close_now = closes[i];
        let close_prev = closes[prev];
        let atr_now = atr.get(i).copied().flatten()?;
        if atr_now <= 0.0 {
            return None;
        }
        let vwap_now = svw.vwap.get(i).copied().flatten()?;
        let upper_2 = svw.upper_2.get(i).copied().flatten()?;
        let sigma = ((upper_2 - vwap_now) / 2.0).abs();
        if sigma <= 0.0 {
            return None;
        }
        let z = (close_now - vwap_now) / sigma;

        let want_long = matches!(side_mode, SideMode::Long | SideMode::Both)
            && z <= -self.rules.z_min
            && close_now > close_prev + self.rules.recovery_buffer * atr_now;
        let want_short = matches!(side_mode, SideMode::Short | SideMode::Both)
            && z >= self.rules.z_min
            && close_now < close_prev - self.rules.recovery_buffer * atr_now;

        if want_long {
            let stop = close_now - self.rules.atr_stop_mult * atr_now;
            let stop_distance = (close_now - stop).max(0.01);
            Some(EntrySignal {
                side: Side::Buy,
                entry_price: close_now,
                stop_distance,
                trigger_index: i,
                stop_price: stop.max(0.01),
                take_profit_price: vwap_now,
                kind: "vwap_scalp",
                diagnostic: serde_json::json!({
                    "vwap": vwap_now,
                    "z": z,
                    "sigma": sigma,
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
                take_profit_price: vwap_now,
                kind: "vwap_scalp",
                diagnostic: serde_json::json!({
                    "vwap": vwap_now,
                    "z": z,
                    "sigma": sigma,
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
        let svw = session_vwap::compute(&session_bars(bars));

        let i = bars.len() - 1;
        let prev = i - 1;
        let close_now = closes[i];
        let close_prev = closes[prev];
        let vwap_now = svw.vwap.get(i).copied().flatten()?;
        let vwap_prev = svw.vwap.get(prev).copied().flatten()?;
        let atr_now = atr.get(i).copied().flatten()?;

        match side {
            Side::Buy => {
                // Trail off the HIGH-water mark for a long. The prior
                // code anchored to anchor_low, which is the trade's
                // worst-case mark — the stop sat below it and could
                // never trail toward profit. Symmetric fix for shorts
                // below.
                let trail = anchor_high - self.rules.atr_stop_mult * atr_now;
                if lows[i] <= trail {
                    return Some(ExitSignal {
                        reason: "atr_trailing_stop",
                        exit_price: trail.max(0.01),
                        trigger_index: i,
                    });
                }
                if close_prev < vwap_prev && close_now >= vwap_now {
                    return Some(ExitSignal {
                        reason: "vwap_cross",
                        exit_price: vwap_now,
                        trigger_index: i,
                    });
                }
            }
            Side::Sell => {
                // Trail off the LOW-water mark for a short.
                let trail = anchor_low + self.rules.atr_stop_mult * atr_now;
                if highs[i] >= trail {
                    return Some(ExitSignal {
                        reason: "atr_trailing_stop",
                        exit_price: trail,
                        trigger_index: i,
                    });
                }
                if close_prev > vwap_prev && close_now <= vwap_now {
                    return Some(ExitSignal {
                        reason: "vwap_cross",
                        exit_price: vwap_now,
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

    /// 30 noisy bars to seed VWAP sigma → 5 sharp down bars (z << -z_min)
    /// → 1 recovery bar (close > prior_close + buffer*atr).
    fn vwap_oversold_window() -> Vec<PriceBar> {
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        for i in 0..30 {
            let p = 100.0 + ((i as f64 * 0.5).sin() * 0.3);
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
        for i in 0..5 {
            let p = 100.0 - (i as f64 + 1.0) * 0.6;
            bars.push(bar(
                t,
                &format!("{:.2}", p + 0.1),
                &format!("{:.2}", p + 0.2),
                &format!("{:.2}", p - 0.2),
                &format!("{p:.2}"),
                1_500_000,
            ));
            t += 60;
        }
        // Recovery bar — small upward tick beyond the noise buffer.
        bars.push(bar(t, "97.00", "97.50", "96.90", "97.30", 1_500_000));
        bars
    }

    fn first_long(bars: &[PriceBar], strat: &VwapScalp) -> Option<EntrySignal> {
        for end in strat.min_bars()..=bars.len() {
            if let Some(s) = strat.evaluate_entry(&bars[..end], SideMode::Long) {
                return Some(s);
            }
        }
        None
    }

    #[test]
    fn entry_fires_on_oversold_z_with_recovery_tick() {
        let strat = VwapScalp::new(Rules::default());
        let bars = vwap_oversold_window();
        let sig = first_long(&bars, &strat).expect("oversold VWAP + recovery must fire");
        assert_eq!(sig.side, Side::Buy);
        let z = sig.diagnostic.get("z").and_then(|v| v.as_f64()).unwrap();
        assert!(z <= -2.0, "z {z} should be <= -2.0");
    }

    #[test]
    fn entry_blocked_on_flat_window() {
        let strat = VwapScalp::new(Rules::default());
        let bars: Vec<PriceBar> = (0..60)
            .map(|i| {
                bar(
                    1_700_000_000 + i * 60,
                    "100.00",
                    "100.05",
                    "99.95",
                    "100.00",
                    1_000_000,
                )
            })
            .collect();
        assert!(first_long(&bars, &strat).is_none());
    }

    #[test]
    fn kind_and_min_bars() {
        let strat = VwapScalp::new(Rules::default());
        assert_eq!(strat.kind().as_str(), "vwap_scalp");
        assert_eq!(strat.min_bars(), 22);
    }
}
