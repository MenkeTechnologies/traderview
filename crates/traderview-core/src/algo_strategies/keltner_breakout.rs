//! Keltner Channel Breakout — Chester Keltner's volatility-channel rule.
//!
//! Keltner upper = EMA(close, period) + mult * ATR(period)
//! Keltner lower = EMA(close, period) - mult * ATR(period)
//!
//! Entry:
//!   long  on  close > upper band AND prior close <= upper band
//!             (fresh breakout, not a sustained trend already in motion).
//!   short on  close < lower band AND prior close >= lower band.
//!
//! Stop:   middle band (EMA) — gives the channel room to expand.
//! Target: 2x channel width beyond entry (default; tunable).
//!
//! Exit: opposite-band touch OR price closes back through the EMA.

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::indicators;
use crate::models::PriceBar;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    pub period: usize,
    pub atr_period: usize,
    pub multiplier: f64,
    /// Take-profit distance as a multiple of channel half-width
    /// (mult * ATR). Default 2.0 = a 1.5x ATR breakout targets a 3x
    /// ATR move from EMA.
    pub take_profit_mult: f64,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            period: 20,
            atr_period: 20,
            multiplier: 1.5,
            take_profit_mult: 2.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeltnerBreakout {
    pub rules: Rules,
}

impl KeltnerBreakout {
    pub fn new(rules: Rules) -> Self {
        Self { rules }
    }
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone()).unwrap_or_default();
        Self { rules }
    }
}

fn keltner_bands(
    bars: &[PriceBar],
    period: usize,
    atr_period: usize,
    mult: f64,
) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    let closes = indicators::closes(bars);
    let highs = indicators::highs(bars);
    let lows = indicators::lows(bars);
    let ema = indicators::ema(&closes, period);
    let atr = indicators::atr(&highs, &lows, &closes, atr_period);
    let mut upper = Vec::with_capacity(bars.len());
    let mut lower = Vec::with_capacity(bars.len());
    for i in 0..bars.len() {
        let m = ema.get(i).copied().flatten();
        let a = atr.get(i).copied().flatten();
        match (m, a) {
            (Some(m), Some(a)) => {
                upper.push(Some(m + mult * a));
                lower.push(Some(m - mult * a));
            }
            _ => {
                upper.push(None);
                lower.push(None);
            }
        }
    }
    (upper, ema, lower)
}

impl Strategy for KeltnerBreakout {
    fn kind(&self) -> StrategyKind {
        StrategyKind::KeltnerBreakout
    }

    fn min_bars(&self) -> usize {
        self.rules.period.max(self.rules.atr_period) + 3
    }

    fn evaluate_entry(&self, bars: &[PriceBar], side_mode: SideMode) -> Option<EntrySignal> {
        if bars.len() < self.min_bars() {
            return None;
        }
        let (upper, mid, lower) = keltner_bands(
            bars,
            self.rules.period,
            self.rules.atr_period,
            self.rules.multiplier,
        );
        let i = bars.len() - 1;
        let prev = i - 1;
        let close_now = bars[i].close.to_f64().unwrap_or(0.0);
        let close_prev = bars[prev].close.to_f64().unwrap_or(0.0);
        let upper_now = upper[i]?;
        let upper_prev = upper[prev]?;
        let mid_now = mid[i]?;
        let lower_now = lower[i]?;
        let lower_prev = lower[prev]?;
        let half_width = (upper_now - mid_now).abs();
        if half_width <= 0.0 {
            return None;
        }

        let want_long = matches!(side_mode, SideMode::Long | SideMode::Both)
            && close_prev <= upper_prev
            && close_now > upper_now;
        let want_short = matches!(side_mode, SideMode::Short | SideMode::Both)
            && close_prev >= lower_prev
            && close_now < lower_now;

        if want_long {
            // Stop at the middle band; never less than 0.5x half-width.
            let stop = mid_now.min(close_now - 0.5 * half_width).max(0.01);
            Some(EntrySignal {
                side: Side::Buy,
                entry_price: close_now,
                stop_distance: (close_now - stop).max(0.01),
                trigger_index: i,
                stop_price: stop,
                take_profit_price: close_now + self.rules.take_profit_mult * half_width,
                kind: "keltner_breakout",
                diagnostic: serde_json::json!({
                    "upper": upper_now, "mid": mid_now, "lower": lower_now,
                    "half_width": half_width,
                }),
            })
        } else if want_short {
            let stop = mid_now.max(close_now + 0.5 * half_width);
            Some(EntrySignal {
                side: Side::Sell,
                entry_price: close_now,
                stop_distance: (stop - close_now).max(0.01),
                trigger_index: i,
                stop_price: stop,
                take_profit_price: (close_now - self.rules.take_profit_mult * half_width).max(0.01),
                kind: "keltner_breakout",
                diagnostic: serde_json::json!({
                    "upper": upper_now, "mid": mid_now, "lower": lower_now,
                    "half_width": half_width,
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
        let (upper, mid, lower) = keltner_bands(
            bars,
            self.rules.period,
            self.rules.atr_period,
            self.rules.multiplier,
        );
        let i = bars.len() - 1;
        let mid_now = mid[i]?;
        let upper_now = upper[i]?;
        let lower_now = lower[i]?;
        let close_now = bars[i].close.to_f64().unwrap_or(0.0);

        match side {
            Side::Buy => {
                // Check the more specific exit reason FIRST. Since
                // lower < mid by construction, the prior `close < mid`
                // branch always fired before `close < lower` could —
                // every below-the-band exit got mislabeled as a
                // mid-break. The trade still exits either way; this
                // only fixes the audit/UI label.
                if close_now < lower_now {
                    Some(ExitSignal {
                        reason: "keltner_lower_touch_long",
                        exit_price: close_now,
                        trigger_index: i,
                    })
                } else if close_now < mid_now {
                    Some(ExitSignal {
                        reason: "keltner_mid_break_long",
                        exit_price: close_now,
                        trigger_index: i,
                    })
                } else {
                    None
                }
            }
            Side::Sell => {
                if close_now > upper_now {
                    Some(ExitSignal {
                        reason: "keltner_upper_touch_short",
                        exit_price: close_now,
                        trigger_index: i,
                    })
                } else if close_now > mid_now {
                    Some(ExitSignal {
                        reason: "keltner_mid_break_short",
                        exit_price: close_now,
                        trigger_index: i,
                    })
                } else {
                    None
                }
            }
        }
    }
}
