//! Post-Earnings Announcement Drift (PEAD) — the canonical Ball &
//! Brown / Bernard-Thomas anomaly: positive earnings surprises persist;
//! the stock keeps drifting up for ~30-60 days post-announcement.
//!
//! This strategy module handles the TECHNICAL confirmation side. The
//! actual earnings-surprise eligibility check happens in
//! `traderview-db::algo_runner::pead_eligible_symbols`, which queries
//! the existing `earnings_cal` table BEFORE the runner calls into this
//! evaluate_entry. The split keeps the Strategy trait pure (bars in,
//! signal out) while letting PEAD use external fundamental data.
//!
//! Entry (long, given a symbol the runner pre-filtered to "had a
//! recent positive earnings surprise"):
//!   close[i] > max(highs[i-recent_high_lookback..i])   (still in drift
//!                                                       mode — no fade
//!                                                       of the gap)
//!   close[i] > SMA(short_trend)                        (still above
//!                                                       short trend)
//!
//! Exit: ATR trailing stop OR `hold_bars` elapsed since entry.

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::indicators;
use crate::models::PriceBar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    /// Look-back for the "still making higher highs" check.
    pub recent_high_lookback: usize,
    pub short_trend_period: usize,
    pub atr_period: usize,
    pub atr_stop_mult: f64,
    pub atr_take_profit_mult: f64,
    /// Minimum surprise % the runner gate requires. Persisted here so
    /// the UI can display it next to the strategy config; the runner
    /// reads it from this same struct via the entry_rules JSONB column.
    pub min_surprise_pct: f64,
    /// How many days after the earnings event the strategy stays
    /// eligible to enter. Past this window the drift edge fades.
    pub max_days_since_earnings: i32,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            recent_high_lookback: 10,
            short_trend_period: 20,
            atr_period: 14,
            atr_stop_mult: 2.0,
            atr_take_profit_mult: 4.0,
            min_surprise_pct: 5.0,
            max_days_since_earnings: 5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pead { pub rules: Rules }

impl Pead {
    pub fn new(rules: Rules) -> Self { Self { rules } }
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone()).unwrap_or_default();
        Self { rules }
    }
}

impl Strategy for Pead {
    fn kind(&self) -> StrategyKind { StrategyKind::Pead }

    fn min_bars(&self) -> usize {
        self.rules
            .recent_high_lookback
            .max(self.rules.short_trend_period)
            .max(self.rules.atr_period + 1)
            + 2
    }

    fn evaluate_entry(&self, bars: &[PriceBar], side_mode: SideMode) -> Option<EntrySignal> {
        // Long-only — the published PEAD edge is asymmetric (positive
        // surprises drift up more reliably than negative ones drift down).
        if !matches!(side_mode, SideMode::Long | SideMode::Both) {
            return None;
        }
        if bars.len() < self.min_bars() {
            return None;
        }
        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let sma = indicators::sma(&closes, self.rules.short_trend_period);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);

        let i = bars.len() - 1;
        let close_now = closes[i];
        let atr_now = atr.get(i).copied().flatten()?;
        if atr_now <= 0.0 { return None; }
        let sma_now = sma.get(i).copied().flatten()?;
        if close_now <= sma_now { return None; }

        let lo = i.saturating_sub(self.rules.recent_high_lookback);
        let recent_high = highs[lo..i].iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        if close_now <= recent_high {
            return None;
        }

        let stop = close_now - self.rules.atr_stop_mult * atr_now;
        let stop_distance = (close_now - stop).max(0.01);
        Some(EntrySignal {
            side: Side::Buy,
            entry_price: close_now,
            stop_distance,
            trigger_index: i,
            stop_price: stop.max(0.01),
            take_profit_price: close_now + self.rules.atr_take_profit_mult * atr_now,
            kind: "pead",
            diagnostic: serde_json::json!({
                "recent_high": recent_high,
                "sma_short": sma_now,
                "atr": atr_now,
            }),
        })
    }

    fn evaluate_exit(
        &self,
        bars: &[PriceBar],
        side: Side,
        anchor_high: f64,
        _anchor_low: f64,
    ) -> Option<ExitSignal> {
        if side != Side::Buy {
            return None;
        }
        if bars.len() < self.min_bars() {
            return None;
        }
        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let sma = indicators::sma(&closes, self.rules.short_trend_period);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);
        let i = bars.len() - 1;
        let atr_now = atr.get(i).copied().flatten()?;
        let sma_now = sma.get(i).copied().flatten()?;
        let close_now = closes[i];

        let trail = anchor_high - self.rules.atr_stop_mult * atr_now;
        if lows[i] <= trail {
            return Some(ExitSignal {
                reason: "atr_trailing_stop",
                exit_price: trail.max(0.01),
                trigger_index: i,
            });
        }
        if close_now < sma_now {
            return Some(ExitSignal {
                reason: "short_trend_loss",
                exit_price: close_now,
                trigger_index: i,
            });
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

    /// 25 climbing bars where the very last bar makes a NEW high above
    /// the 10-bar lookback window AND closes above SMA(20). Models the
    /// "still drifting up" post-earnings setup.
    fn drift_window() -> Vec<PriceBar> {
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        for i in 0..25 {
            let p = 100.0 + i as f64 * 0.6;
            bars.push(bar(
                t,
                &format!("{p:.2}"),
                &format!("{:.2}", p + 0.3),
                &format!("{:.2}", p - 0.1),
                &format!("{:.2}", p + 0.2),
                1_000_000,
            ));
            t += 60;
        }
        bars
    }

    fn first_long(bars: &[PriceBar], strat: &Pead) -> Option<EntrySignal> {
        for end in strat.min_bars()..=bars.len() {
            if let Some(s) = strat.evaluate_entry(&bars[..end], SideMode::Long) {
                return Some(s);
            }
        }
        None
    }

    #[test]
    fn entry_fires_on_new_high_above_short_trend() {
        let strat = Pead::new(Rules::default());
        let bars = drift_window();
        let sig = first_long(&bars, &strat).expect("drift past recent high + SMA → long");
        assert_eq!(sig.side, Side::Buy);
        let sma = sig.diagnostic.get("sma_short").and_then(|v| v.as_f64()).unwrap();
        assert!(sig.entry_price > sma, "close {} > SMA20 {}", sig.entry_price, sma);
    }

    #[test]
    fn entry_blocked_when_close_below_short_trend() {
        let strat = Pead::new(Rules::default());
        // Pure declining series — close always below SMA(20).
        let bars: Vec<PriceBar> = (0..40)
            .map(|i| {
                let p = 100.0 - i as f64 * 0.5;
                bar(
                    1_700_000_000 + i * 60,
                    &format!("{p:.2}"),
                    &format!("{:.2}", p + 0.2),
                    &format!("{:.2}", p - 0.5),
                    &format!("{:.2}", p - 0.2),
                    1_000_000,
                )
            })
            .collect();
        assert!(first_long(&bars, &strat).is_none());
    }

    #[test]
    fn entry_refuses_short_side() {
        let strat = Pead::new(Rules::default());
        let bars = drift_window();
        for end in strat.min_bars()..=bars.len() {
            assert!(strat.evaluate_entry(&bars[..end], SideMode::Short).is_none());
        }
    }

    #[test]
    fn kind_and_min_bars() {
        let strat = Pead::new(Rules::default());
        assert_eq!(strat.kind().as_str(), "pead");
        assert_eq!(strat.min_bars(), 22); // max(10, 20, 15) + 2 = 22.
    }
}
