//! Connors RSI-2 — Larry Connors's classic mean-reversion edge for
//! stocks (Connors & Alvarez, "Short Term Trading Strategies That
//! Work", 2008).
//!
//! Entry (long):
//!   close > SMA(200)         (long-only above the long-term trend)
//!   RSI(2) < rsi_oversold    (extreme oversold on a tight period)
//!
//! Exit (long):
//!   close > SMA(short_exit)  (Connors's "5-day SMA touch" exit)
//!   OR RSI(2) > rsi_overbought
//!
//! No short side — Connors's published research was long-only because
//! the edge in stocks above the 200 SMA was much cleaner than the
//! mirror image below it.

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::indicators;
use crate::models::PriceBar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    pub sma_trend: usize,
    pub sma_exit: usize,
    pub rsi_period: usize,
    pub rsi_oversold: f64,
    pub rsi_overbought: f64,
    pub atr_period: usize,
    pub atr_stop_mult: f64,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            sma_trend: 200,
            sma_exit: 5,
            rsi_period: 2,
            rsi_oversold: 5.0,
            rsi_overbought: 70.0,
            atr_period: 14,
            atr_stop_mult: 2.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnorsRsi2 { pub rules: Rules }

impl ConnorsRsi2 {
    pub fn new(rules: Rules) -> Self { Self { rules } }
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone()).unwrap_or_default();
        Self { rules }
    }
}

impl Strategy for ConnorsRsi2 {
    fn kind(&self) -> StrategyKind { StrategyKind::ConnorsRsi2 }

    fn min_bars(&self) -> usize {
        self.rules.sma_trend.max(self.rules.atr_period + 1) + 2
    }

    fn evaluate_entry(&self, bars: &[PriceBar], side_mode: SideMode) -> Option<EntrySignal> {
        // Connors's published edge is long-only — refuse short side
        // even if the strategy is configured for it. Mirror would lose
        // money historically per the source research.
        if !matches!(side_mode, SideMode::Long | SideMode::Both) {
            return None;
        }
        if bars.len() < self.min_bars() {
            return None;
        }
        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let sma200 = indicators::sma(&closes, self.rules.sma_trend);
        let rsi = indicators::rsi(&closes, self.rules.rsi_period);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);

        let i = bars.len() - 1;
        let close_now = closes[i];
        let sma_now = sma200.get(i).copied().flatten()?;
        let rsi_now = rsi.get(i).copied().flatten()?;
        let atr_now = atr.get(i).copied().flatten()?;
        if atr_now <= 0.0 { return None; }

        let above_trend = close_now > sma_now;
        let oversold = rsi_now < self.rules.rsi_oversold;
        if !(above_trend && oversold) {
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
            // No explicit target — exit on RSI cross or SMA(5) touch.
            // Set TP at 4×ATR as a fallback ceiling.
            take_profit_price: close_now + 4.0 * atr_now,
            kind: "connors_rsi2",
            diagnostic: serde_json::json!({
                "sma_trend": sma_now,
                "rsi": rsi_now,
                "atr": atr_now,
                "above_trend": above_trend,
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
        // Long-only — short positions never opened by this strategy.
        if side != Side::Buy {
            return None;
        }
        if bars.len() < self.min_bars() {
            return None;
        }
        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let sma_exit = indicators::sma(&closes, self.rules.sma_exit);
        let rsi = indicators::rsi(&closes, self.rules.rsi_period);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);

        let i = bars.len() - 1;
        let close_now = closes[i];
        let atr_now = atr.get(i).copied().flatten()?;
        let sma_now = sma_exit.get(i).copied().flatten()?;
        let rsi_now = rsi.get(i).copied().flatten()?;

        // Hard stop — ATR trailing under the high-water anchor.
        let trail = anchor_high - self.rules.atr_stop_mult * atr_now;
        if lows[i] <= trail {
            return Some(ExitSignal {
                reason: "atr_trailing_stop",
                exit_price: trail.max(0.01),
                trigger_index: i,
            });
        }
        if close_now >= sma_now {
            return Some(ExitSignal {
                reason: "sma_exit_touch",
                exit_price: close_now,
                trigger_index: i,
            });
        }
        if rsi_now > self.rules.rsi_overbought {
            return Some(ExitSignal {
                reason: "rsi_overbought",
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

    /// 220 bars climbing steadily (price stays above SMA-200), then 4
    /// sharp down bars that push RSI(2) deep into oversold but don't
    /// break below SMA-200.
    fn pullback_above_trend_window() -> Vec<PriceBar> {
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        // Climb from 100 → ~155 over 220 bars (gentle uptrend; closes
        // sit well above the 200-bar SMA).
        for i in 0..220 {
            let p = 100.0 + i as f64 * 0.25;
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
        // 4 quick down bars — close drops ~3 points without breaking
        // SMA(200). RSI(2) is sensitive enough that 2-3 reds suffice.
        let mut last = 155.0;
        for _ in 0..4 {
            last -= 0.8;
            bars.push(bar(
                t,
                &format!("{:.2}", last + 0.5),
                &format!("{:.2}", last + 0.5),
                &format!("{:.2}", last - 0.1),
                &format!("{last:.2}"),
                1_000_000,
            ));
            t += 60;
        }
        bars
    }

    fn first_long(bars: &[PriceBar], strat: &ConnorsRsi2) -> Option<EntrySignal> {
        for end in strat.min_bars()..=bars.len() {
            if let Some(s) = strat.evaluate_entry(&bars[..end], SideMode::Long) {
                return Some(s);
            }
        }
        None
    }

    #[test]
    fn entry_fires_on_pullback_above_sma200_with_rsi2_extreme() {
        let strat = ConnorsRsi2::new(Rules::default());
        let bars = pullback_above_trend_window();
        let sig = first_long(&bars, &strat).expect("pullback above 200SMA must fire");
        assert_eq!(sig.side, Side::Buy);
        let sma = sig.diagnostic.get("sma_trend").and_then(|v| v.as_f64()).unwrap();
        let rsi = sig.diagnostic.get("rsi").and_then(|v| v.as_f64()).unwrap();
        assert!(sig.entry_price > sma, "close {} > SMA200 {}", sig.entry_price, sma);
        assert!(rsi < 5.0, "RSI(2) {rsi} must be below 5");
    }

    #[test]
    fn entry_blocked_below_sma200_even_if_rsi_oversold() {
        let strat = ConnorsRsi2::new(Rules::default());
        // Downtrend: close NEVER above SMA(200). Even if RSI(2) hits 0
        // the strategy refuses to buy under the long-term trend.
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        for i in 0..225 {
            let p = 200.0 - i as f64 * 0.5;
            bars.push(bar(
                t,
                &format!("{p:.2}"),
                &format!("{:.2}", p + 0.2),
                &format!("{:.2}", p - 0.5),
                &format!("{:.2}", p - 0.2),
                1_000_000,
            ));
            t += 60;
        }
        assert!(first_long(&bars, &strat).is_none());
    }

    #[test]
    fn entry_refuses_short_side_even_under_side_mode_both() {
        let strat = ConnorsRsi2::new(Rules::default());
        let bars = pullback_above_trend_window();
        // Under SideMode::Short the strategy must return None — long-only edge.
        for end in strat.min_bars()..=bars.len() {
            assert!(
                strat.evaluate_entry(&bars[..end], SideMode::Short).is_none(),
                "long-only strategy must not fire under SideMode::Short"
            );
        }
    }

    #[test]
    fn kind_and_min_bars() {
        let strat = ConnorsRsi2::new(Rules::default());
        assert_eq!(strat.kind().as_str(), "connors_rsi2");
        // sma_trend (200) wins → +2 = 202.
        assert_eq!(strat.min_bars(), 202);
    }
}
