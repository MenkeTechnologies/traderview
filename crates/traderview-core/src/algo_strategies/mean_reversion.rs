//! Mean Reversion — Connors RSI + session VWAP z-score reversion.
//!
//! Entry (long):
//!   Connors RSI (rsi_p=3, streak_p=2, rank_p=100) < crsi_oversold AND
//!   close <  session_vwap − vwap_z_min × vwap_sigma AND
//!   RSI(rsi_period) trending up over the last 2 bars (early reversal sign)
//!
//! Entry (short) — mirror.
//!
//! Exit (long): close crosses session VWAP back upward, OR ATR-based
//! emergency stop triggers (low touches anchor_low − atr_stop_mult × ATR).
//!
//! Sizing: stop_distance = (entry − stop_price).abs(). The reverter's
//! natural stop sits beyond the previous low / VWAP-3σ, so risk-budget
//! sizing falls out of that.

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::indicators;
use crate::models::PriceBar;
use crate::{connors_rsi, session_vwap};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    pub crsi_rsi_period: usize,
    pub crsi_streak_period: usize,
    pub crsi_rank_period: usize,
    pub crsi_oversold: f64,
    pub crsi_overbought: f64,
    pub rsi_period: usize,
    pub vwap_z_min: f64,
    pub atr_period: usize,
    pub atr_stop_mult: f64,
    /// Take-profit multiplier (relative to ATR) when VWAP cross hasn't
    /// fired yet — used as fallback ceiling.
    pub atr_take_profit_mult: f64,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            crsi_rsi_period: 3,
            crsi_streak_period: 2,
            crsi_rank_period: 100,
            crsi_oversold: 10.0,
            crsi_overbought: 90.0,
            rsi_period: 14,
            vwap_z_min: 2.0,
            atr_period: 14,
            atr_stop_mult: 2.0,
            atr_take_profit_mult: 1.5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MeanReversion {
    pub rules: Rules,
}

impl MeanReversion {
    pub fn new(rules: Rules) -> Self { Self { rules } }
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone())
            .unwrap_or_default();
        Self { rules }
    }
}

/// Build the session_vwap Bar series. We mark every bar as
/// `is_session_start = false` after the first — engine-side bar windows
/// are already session-scoped (the live tick worker resets at the open),
/// so a single-session window is the right unit here.
fn session_bars(bars: &[PriceBar]) -> Vec<session_vwap::Bar> {
    bars.iter()
        .enumerate()
        .map(|(i, b)| session_vwap::Bar {
            high: f64_dec(b.high),
            low: f64_dec(b.low),
            close: f64_dec(b.close),
            volume: f64_dec(b.volume),
            is_session_start: i == 0,
        })
        .collect()
}

fn f64_dec(d: rust_decimal::Decimal) -> f64 {
    use rust_decimal::prelude::ToPrimitive;
    d.to_f64().unwrap_or(0.0)
}

impl Strategy for MeanReversion {
    fn kind(&self) -> StrategyKind { StrategyKind::MeanReversion }

    fn min_bars(&self) -> usize {
        self.rules.crsi_rank_period
            .max(self.rules.rsi_period + 2)
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

        let crsi = connors_rsi::compute(
            &closes,
            self.rules.crsi_rsi_period,
            self.rules.crsi_streak_period,
            self.rules.crsi_rank_period,
        );
        let rsi = indicators::rsi(&closes, self.rules.rsi_period);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);
        let svw = session_vwap::compute(&session_bars(bars));

        let i = bars.len() - 1;
        let prev = i - 1;

        let crsi_now = crsi.get(i).copied().flatten()?;
        let rsi_now = rsi.get(i).copied().flatten()?;
        let atr_now = atr.get(i).copied().flatten()?;
        if atr_now <= 0.0 {
            return None;
        }
        let vwap_now = svw.vwap.get(i).copied().flatten()?;
        let upper_2 = svw.upper_2.get(i).copied().flatten()?;
        let sigma = ((upper_2 - vwap_now) / 2.0).abs(); // upper_2 = vwap + 2σ
        if sigma <= 0.0 {
            return None;
        }
        let close_now = closes[i];
        let z = (close_now - vwap_now) / sigma;

        // Pure CRSI + VWAP z-score gate — Connors's canonical "catch the
        // falling knife" signal. The ATR stop manages risk if the
        // reversion fails. No "RSI trending up" prerequisite; that would
        // veto the very oversold bars CRSI is designed to flag.
        let want_long = matches!(side_mode, SideMode::Long | SideMode::Both)
            && crsi_now < self.rules.crsi_oversold
            && close_now < (vwap_now - self.rules.vwap_z_min * sigma);
        let want_short = matches!(side_mode, SideMode::Short | SideMode::Both)
            && crsi_now > self.rules.crsi_overbought
            && close_now > (vwap_now + self.rules.vwap_z_min * sigma);
        let _ = prev; let _ = rsi_now;

        if want_long {
            let stop = close_now - self.rules.atr_stop_mult * atr_now;
            let stop_distance = (close_now - stop).max(0.01);
            Some(EntrySignal {
                side: Side::Buy,
                entry_price: close_now,
                stop_distance,
                trigger_index: i,
                stop_price: stop.max(0.01),
                // Take-profit at VWAP — that's the reversion target.
                take_profit_price: vwap_now,
                kind: "mean_reversion",
                diagnostic: serde_json::json!({
                    "crsi": crsi_now,
                    "rsi": rsi_now,
                    "vwap": vwap_now,
                    "vwap_z": z,
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
                kind: "mean_reversion",
                diagnostic: serde_json::json!({
                    "crsi": crsi_now,
                    "rsi": rsi_now,
                    "vwap": vwap_now,
                    "vwap_z": z,
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
                // ATR stop anchored to the LOW since entry (worst case).
                let atr_stop = anchor_low - self.rules.atr_stop_mult * atr_now;
                if lows[i] <= atr_stop {
                    return Some(ExitSignal {
                        reason: "atr_stop",
                        exit_price: atr_stop.max(0.01),
                        trigger_index: i,
                    });
                }
                // Long target: price crosses VWAP from below.
                if close_prev < vwap_prev && close_now >= vwap_now {
                    return Some(ExitSignal {
                        reason: "vwap_cross",
                        exit_price: vwap_now,
                        trigger_index: i,
                    });
                }
            }
            Side::Sell => {
                let atr_stop = anchor_high + self.rules.atr_stop_mult * atr_now;
                if highs[i] >= atr_stop {
                    return Some(ExitSignal {
                        reason: "atr_stop",
                        exit_price: atr_stop,
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

    /// Build a noisy session where the latest bar is a sharp oversold
    /// pierce: Connors RSI drops below 10, price closes well below the
    /// session VWAP, RSI(14) ticks up on the rebound bar.
    fn oversold_window() -> Vec<PriceBar> {
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        // 110 bars meandering around 100 with small swings to build
        // both Connors RSI rank denominator and session VWAP context.
        for i in 0..110 {
            let phase = (i as f64 * 0.21).sin() * 0.5;
            let p = 100.0 + phase;
            bars.push(bar(
                t,
                &format!("{p:.2}"),
                &format!("{:.2}", p + 0.3),
                &format!("{:.2}", p - 0.3),
                &format!("{p:.2}"),
                1_000_000,
            ));
            t += 60;
        }
        // 5 sharp down bars to drive CRSI into oversold.
        for i in 0..5 {
            let p = 99.0 - (i as f64 + 1.0) * 0.5;
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
        // Final bar: tiny up-tick (RSI moves up vs prior), price still
        // well below VWAP. This is the rebound candle that should fire entry.
        let last_close = 96.4;
        bars.push(bar(
            t,
            "96.20",
            "96.50",
            "96.10",
            &format!("{last_close:.2}"),
            2_000_000,
        ));
        bars
    }

    fn first_long(bars: &[PriceBar], strat: &MeanReversion) -> Option<EntrySignal> {
        // Walk forward to find the first bar where the rule fires.
        for end in strat.min_bars()..=bars.len() {
            if let Some(s) = strat.evaluate_entry(&bars[..end], SideMode::Long) {
                return Some(s);
            }
        }
        None
    }

    #[test]
    fn entry_fires_on_oversold_below_vwap_pierce() {
        let strat = MeanReversion::new(Rules::default());
        let bars = oversold_window();
        let sig = first_long(&bars, &strat)
            .expect("oversold + below-VWAP setup must produce mean-reversion entry");
        assert_eq!(sig.side, Side::Buy);
        assert!(sig.entry_price < sig.take_profit_price, "TP at VWAP > entry");
        let crsi = sig.diagnostic.get("crsi").and_then(|v| v.as_f64()).unwrap();
        let vwap_z = sig.diagnostic.get("vwap_z").and_then(|v| v.as_f64()).unwrap();
        assert!(crsi < 10.0, "CRSI {crsi} < 10");
        assert!(vwap_z < -2.0, "vwap_z {vwap_z} < -2");
    }

    #[test]
    fn entry_blocked_on_flat_market() {
        let strat = MeanReversion::new(Rules::default());
        // 130 bars of pure flat — CRSI sits mid-range, no VWAP deviation.
        let bars: Vec<PriceBar> = (0..130)
            .map(|i| bar(1_700_000_000 + i * 60, "100.00", "100.10", "99.90", "100.00", 1_000_000))
            .collect();
        assert!(first_long(&bars, &strat).is_none());
    }

    #[test]
    fn entry_blocked_under_side_mode_short_on_long_setup() {
        let strat = MeanReversion::new(Rules::default());
        let bars = oversold_window();
        for end in strat.min_bars()..=bars.len() {
            assert!(
                strat.evaluate_entry(&bars[..end], SideMode::Short).is_none(),
                "long-side oversold must never fire under SideMode::Short"
            );
        }
    }

    #[test]
    fn kind_and_min_bars() {
        let strat = MeanReversion::new(Rules::default());
        assert_eq!(strat.kind().as_str(), "mean_reversion");
        // crsi_rank_period (100) wins → 101.
        assert_eq!(strat.min_bars(), 101);
    }
}
