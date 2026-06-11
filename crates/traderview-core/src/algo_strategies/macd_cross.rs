//! MACD signal-line crossover — the baseline momentum-turn strategy.
//!
//! Entry:
//!   long  on  MACD line crosses above its signal line; with
//!             `require_zero_side` the cross must happen BELOW the zero
//!             line (catching the turn early, Appel's original reversal
//!             reading) — without it any bullish cross fires.
//!   short on  MACD line crosses below the signal line (mirrored: the
//!             filtered variant requires the cross ABOVE zero).
//!
//! Stop:   ATR-multiple below (long) / above (short) entry.
//! Target: ATR-multiple beyond entry.
//!
//! Exit:   opposite signal-line cross.

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::indicators;
use crate::models::PriceBar;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    pub fast_period: usize,
    pub slow_period: usize,
    pub signal_period: usize,
    /// Only take bullish crosses below zero / bearish crosses above
    /// zero — the early-reversal reading. Off = every cross fires.
    pub require_zero_side: bool,
    pub atr_period: usize,
    pub atr_stop_mult: f64,
    pub atr_take_profit_mult: f64,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
            require_zero_side: false,
            atr_period: 14,
            atr_stop_mult: 1.5,
            atr_take_profit_mult: 3.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MacdCross {
    pub rules: Rules,
}

impl MacdCross {
    pub fn new(rules: Rules) -> Self {
        Self { rules }
    }
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone()).unwrap_or_default();
        Self { rules }
    }

    /// (macd_now, signal_now, macd_prev, signal_prev) at the last bar.
    fn lines(&self, bars: &[PriceBar]) -> Option<(f64, f64, f64, f64)> {
        let closes = indicators::closes(bars);
        let m = indicators::macd(
            &closes,
            self.rules.fast_period,
            self.rules.slow_period,
            self.rules.signal_period,
        );
        let i = bars.len() - 1;
        Some((
            m.line.get(i).copied().flatten()?,
            m.signal.get(i).copied().flatten()?,
            m.line.get(i - 1).copied().flatten()?,
            m.signal.get(i - 1).copied().flatten()?,
        ))
    }
}

impl Strategy for MacdCross {
    fn kind(&self) -> StrategyKind {
        StrategyKind::MacdCross
    }

    fn min_bars(&self) -> usize {
        self.rules.slow_period + self.rules.signal_period + 3
    }

    fn evaluate_entry(&self, bars: &[PriceBar], side_mode: SideMode) -> Option<EntrySignal> {
        if bars.len() < self.min_bars() {
            return None;
        }
        let (macd_now, sig_now, macd_prev, sig_prev) = self.lines(bars)?;
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let closes = indicators::closes(bars);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);
        let i = bars.len() - 1;
        let atr_now = atr.get(i).copied().flatten()?;
        if atr_now <= 0.0 {
            return None;
        }
        let close_f = bars[i].close.to_f64().unwrap_or(0.0);

        let cross_up = macd_prev <= sig_prev && macd_now > sig_now;
        let cross_down = macd_prev >= sig_prev && macd_now < sig_now;
        let zero_ok_long = !self.rules.require_zero_side || macd_now < 0.0;
        let zero_ok_short = !self.rules.require_zero_side || macd_now > 0.0;

        let diagnostic = serde_json::json!({
            "macd": macd_now, "signal": sig_now,
            "histogram": macd_now - sig_now, "atr": atr_now,
        });
        let want_long =
            matches!(side_mode, SideMode::Long | SideMode::Both) && cross_up && zero_ok_long;
        let want_short =
            matches!(side_mode, SideMode::Short | SideMode::Both) && cross_down && zero_ok_short;

        if want_long {
            let stop = (close_f - self.rules.atr_stop_mult * atr_now).max(0.01);
            Some(EntrySignal {
                side: Side::Buy,
                entry_price: close_f,
                stop_distance: (close_f - stop).max(0.01),
                trigger_index: i,
                stop_price: stop,
                take_profit_price: close_f + self.rules.atr_take_profit_mult * atr_now,
                kind: "macd_cross",
                diagnostic,
            })
        } else if want_short {
            let stop = close_f + self.rules.atr_stop_mult * atr_now;
            Some(EntrySignal {
                side: Side::Sell,
                entry_price: close_f,
                stop_distance: (stop - close_f).max(0.01),
                trigger_index: i,
                stop_price: stop,
                take_profit_price: (close_f - self.rules.atr_take_profit_mult * atr_now).max(0.01),
                kind: "macd_cross",
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
        _anchor_high: f64,
        _anchor_low: f64,
    ) -> Option<ExitSignal> {
        if bars.len() < self.min_bars() {
            return None;
        }
        let (macd_now, sig_now, macd_prev, sig_prev) = self.lines(bars)?;
        let close_f = bars[bars.len() - 1].close.to_f64().unwrap_or(0.0);
        let i = bars.len() - 1;
        match side {
            Side::Buy if macd_prev >= sig_prev && macd_now < sig_now => Some(ExitSignal {
                reason: "macd_cross_down",
                exit_price: close_f,
                trigger_index: i,
            }),
            Side::Sell if macd_prev <= sig_prev && macd_now > sig_now => Some(ExitSignal {
                reason: "macd_cross_up",
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

    /// ACCELERATING decline then recovery. The acceleration matters: a
    /// constant-slope decline parks line−signal at +epsilon (both
    /// converge to the same constant), so the strict `prev <=` cross
    /// condition never arms — real tape has curvature.
    fn v_bottom() -> Vec<PriceBar> {
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        let mut bottom = 0.0;
        for i in 0..60 {
            let p = 120.0 - 0.3 * i as f64 - 0.008 * (i * i) as f64;
            bottom = p;
            bars.push(bar(t, p));
            t += 60;
        }
        for i in 0..25 {
            bars.push(bar(t, bottom + (i + 1) as f64 * 0.6));
            t += 60;
        }
        bars
    }

    fn first_long(bars: &[PriceBar], strat: &MacdCross) -> Option<EntrySignal> {
        for end in strat.min_bars()..=bars.len() {
            if let Some(s) = strat.evaluate_entry(&bars[..end], SideMode::Long) {
                return Some(s);
            }
        }
        None
    }

    #[test]
    fn bullish_cross_below_zero_fires_with_and_without_filter() {
        let bars = v_bottom();
        let sig = first_long(&bars, &MacdCross::new(Rules::default()))
            .expect("V-bottom recovery should produce a bullish cross");
        assert_eq!(sig.side, Side::Buy);
        let macd = sig.diagnostic.get("macd").and_then(|v| v.as_f64()).unwrap();
        assert!(macd < 0.0, "cross after a 60-bar slide must sit below zero, got {macd}");
        // The early-reversal filter accepts the same below-zero cross.
        let filtered = MacdCross::new(Rules {
            require_zero_side: true,
            ..Rules::default()
        });
        assert!(first_long(&bars, &filtered).is_some());
    }

    #[test]
    fn zero_filter_blocks_cross_above_zero() {
        // Long rally (MACD well above zero), shallow two-bar dip, resume:
        // the recross happens above zero, so the filtered variant must
        // stay silent while the unfiltered one fires.
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        for i in 0..60 {
            bars.push(bar(t, 100.0 + i as f64 * 0.8));
            t += 60;
        }
        for p in [146.0, 144.5, 143.5, 144.5, 146.5, 149.0, 152.0] {
            bars.push(bar(t, p));
            t += 60;
        }
        let open_sig = first_long(&bars, &MacdCross::new(Rules::default()))
            .expect("dip-resume should recross in an uptrend");
        let macd = open_sig.diagnostic.get("macd").and_then(|v| v.as_f64()).unwrap();
        assert!(macd > 0.0, "premise: this cross sits above zero, got {macd}");
        let filtered = MacdCross::new(Rules {
            require_zero_side: true,
            ..Rules::default()
        });
        assert!(first_long(&bars, &filtered).is_none());
    }

    #[test]
    fn long_exits_on_bearish_cross() {
        // ACCELERATING rally then slide (curvature keeps line strictly
        // above signal at the top — see v_bottom note); once MACD
        // crosses back under its signal line the long must exit.
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        let mut top = 0.0;
        for i in 0..50 {
            let p = 100.0 + 0.3 * i as f64 + 0.01 * (i * i) as f64;
            top = p;
            bars.push(bar(t, p));
            t += 60;
        }
        let strat = MacdCross::new(Rules::default());
        let mut exited = None;
        for i in 0..40 {
            bars.push(bar(t, top - (i + 1) as f64 * 0.9));
            t += 60;
            if let Some(e) = strat.evaluate_exit(&bars, Side::Buy, 0.0, 0.0) {
                exited = Some(e);
                break;
            }
        }
        let e = exited.expect("slide must produce a bearish cross exit");
        assert_eq!(e.reason, "macd_cross_down");
    }

    #[test]
    fn factory_reaches_macd_cross() {
        let s = from_kind("macd_cross", &serde_json::json!({})).expect("registered");
        assert_eq!(s.kind(), StrategyKind::MacdCross);
    }
}
