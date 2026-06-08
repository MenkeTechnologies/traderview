//! Moving Average Crossover with ADX trend-strength filter.
//!
//! Classic fast/slow EMA crossover gated on ADX > threshold so the
//! strategy only fires when a trend is genuinely strong (filters out
//! sideways chop where MA crosses produce a string of small losers).
//!
//! Entry:
//!   long  on  EMA(fast) crosses above EMA(slow) AND ADX[i] > adx_min
//!             AND plus_di[i] > minus_di[i] (directional confirmation)
//!   short on  EMA(fast) crosses below EMA(slow) AND ADX[i] > adx_min
//!             AND minus_di[i] > plus_di[i]
//!
//! Stop:   ATR-multiple below (long) / above (short) entry.
//! Target: ATR-multiple beyond entry. Tunable via Rules.
//!
//! Exit:   opposite crossover OR ADX drops below trend_lost threshold
//!         (so we don't sit through a strong-→weak trend transition).

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::indicators;
use crate::models::PriceBar;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    pub fast_period: usize,
    pub slow_period: usize,
    pub adx_period: usize,
    /// Minimum ADX to allow entry. 25 is the canonical threshold for
    /// "trending market" — below that the chart is choppy.
    pub adx_min: f64,
    /// Below this ADX after a position is open, exit on the next bar
    /// (trend has weakened past the strategy's edge).
    pub adx_trend_lost: f64,
    pub atr_period: usize,
    pub atr_stop_mult: f64,
    pub atr_take_profit_mult: f64,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            fast_period: 9,
            slow_period: 21,
            adx_period: 14,
            adx_min: 25.0,
            adx_trend_lost: 18.0,
            atr_period: 14,
            atr_stop_mult: 1.5,
            atr_take_profit_mult: 3.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MaCrossAdx {
    pub rules: Rules,
}

impl MaCrossAdx {
    pub fn new(rules: Rules) -> Self { Self { rules } }
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone()).unwrap_or_default();
        Self { rules }
    }
}

impl Strategy for MaCrossAdx {
    fn kind(&self) -> StrategyKind { StrategyKind::MaCrossAdx }

    fn min_bars(&self) -> usize {
        self.rules.slow_period.max(self.rules.adx_period * 2) + 3
    }

    fn evaluate_entry(&self, bars: &[PriceBar], side_mode: SideMode) -> Option<EntrySignal> {
        if bars.len() < self.min_bars() {
            return None;
        }
        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let fast = indicators::ema(&closes, self.rules.fast_period);
        let slow = indicators::ema(&closes, self.rules.slow_period);
        let adx = indicators::adx(&highs, &lows, &closes, self.rules.adx_period);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);

        let i = bars.len() - 1;
        let prev = i - 1;
        let fast_now = fast.get(i).copied().flatten()?;
        let slow_now = slow.get(i).copied().flatten()?;
        let fast_prev = fast.get(prev).copied().flatten()?;
        let slow_prev = slow.get(prev).copied().flatten()?;
        let adx_now = adx.adx.get(i).copied().flatten()?;
        let plus_di = adx.plus_di.get(i).copied().flatten()?;
        let minus_di = adx.minus_di.get(i).copied().flatten()?;
        let atr_now = atr.get(i).copied().flatten()?;
        if atr_now <= 0.0 || adx_now < self.rules.adx_min {
            return None;
        }
        let close_f = bars[i].close.to_f64().unwrap_or(0.0);

        let cross_up = fast_prev <= slow_prev && fast_now > slow_now;
        let cross_down = fast_prev >= slow_prev && fast_now < slow_now;

        let want_long = matches!(side_mode, SideMode::Long | SideMode::Both)
            && cross_up
            && plus_di > minus_di;
        let want_short = matches!(side_mode, SideMode::Short | SideMode::Both)
            && cross_down
            && minus_di > plus_di;

        if want_long {
            let stop = (close_f - self.rules.atr_stop_mult * atr_now).max(0.01);
            Some(EntrySignal {
                side: Side::Buy,
                entry_price: close_f,
                stop_distance: (close_f - stop).max(0.01),
                trigger_index: i,
                stop_price: stop,
                take_profit_price: close_f + self.rules.atr_take_profit_mult * atr_now,
                kind: "ma_cross_adx",
                diagnostic: serde_json::json!({
                    "fast": fast_now, "slow": slow_now,
                    "adx": adx_now, "plus_di": plus_di, "minus_di": minus_di,
                    "atr": atr_now,
                }),
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
                kind: "ma_cross_adx",
                diagnostic: serde_json::json!({
                    "fast": fast_now, "slow": slow_now,
                    "adx": adx_now, "plus_di": plus_di, "minus_di": minus_di,
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
        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let fast = indicators::ema(&closes, self.rules.fast_period);
        let slow = indicators::ema(&closes, self.rules.slow_period);
        let adx = indicators::adx(&highs, &lows, &closes, self.rules.adx_period);
        let i = bars.len() - 1;
        let prev = i - 1;
        let fast_now = fast.get(i).copied().flatten()?;
        let slow_now = slow.get(i).copied().flatten()?;
        let fast_prev = fast.get(prev).copied().flatten()?;
        let slow_prev = slow.get(prev).copied().flatten()?;
        let adx_now = adx.adx.get(i).copied().flatten()?;
        let close_f = bars[i].close.to_f64().unwrap_or(0.0);

        // Trend collapse exit applies to either side.
        if adx_now < self.rules.adx_trend_lost {
            return Some(ExitSignal {
                reason: "adx_trend_lost",
                exit_price: close_f,
                trigger_index: i,
            });
        }
        match side {
            Side::Buy if fast_prev >= slow_prev && fast_now < slow_now => Some(ExitSignal {
                reason: "ma_cross_down",
                exit_price: close_f,
                trigger_index: i,
            }),
            Side::Sell if fast_prev <= slow_prev && fast_now > slow_now => Some(ExitSignal {
                reason: "ma_cross_up",
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
    use crate::algo_strategies::{from_kind, StrategyKind};
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

    /// Strong uptrend window with enough length for ADX to ramp up
    /// AFTER the EMA cross occurs. We use lenient adx_min in the test
    /// so the signal lands deterministically even with synthetic data.
    fn strong_uptrend_window() -> Vec<PriceBar> {
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        for i in 0..30 {
            let p = 100.0 + ((i as f64 * 0.4).sin() * 0.15);
            bars.push(bar(
                t,
                &format!("{p:.2}"),
                &format!("{:.2}", p + 0.15),
                &format!("{:.2}", p - 0.15),
                &format!("{p:.2}"),
                1_000_000,
            ));
            t += 60;
        }
        for i in 0..50 {
            let p = 100.4 + (i as f64 + 1.0) * 0.5;
            bars.push(bar(
                t,
                &format!("{:.2}", p - 0.25),
                &format!("{:.2}", p + 0.35),
                &format!("{:.2}", p - 0.35),
                &format!("{p:.2}"),
                1_500_000,
            ));
            t += 60;
        }
        bars
    }

    fn first_long(bars: &[PriceBar], strat: &MaCrossAdx) -> Option<EntrySignal> {
        for end in strat.min_bars()..=bars.len() {
            if let Some(s) = strat.evaluate_entry(&bars[..end], SideMode::Long) {
                return Some(s);
            }
        }
        None
    }

    #[test]
    fn entry_fires_on_bull_cross_with_strong_adx() {
        // Use a lenient adx_min so synthetic-data smoothing doesn't kill
        // the signal — real trading bumps it back to 25.
        let strat = MaCrossAdx::new(Rules {
            adx_min: 18.0,
            ..Rules::default()
        });
        let bars = strong_uptrend_window();
        let sig = first_long(&bars, &strat).expect("strong trend should fire long");
        assert_eq!(sig.side, Side::Buy);
        let adx = sig.diagnostic.get("adx").and_then(|v| v.as_f64()).unwrap();
        assert!(adx >= 18.0, "adx={adx} should pass gate");
    }

    #[test]
    fn entry_blocked_on_weak_adx_sideways() {
        let strat = MaCrossAdx::new(Rules::default());
        // 120 bars of micro-noise around 100.0 — directional movement
        // is well under what ADX would call a trend (adx_min=25). EMAs
        // may briefly cross but the gate filters them out.
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        for i in 0..120 {
            let p = 100.0 + ((i as f64 * 0.3).sin() * 0.08);
            bars.push(bar(
                t,
                &format!("{p:.4}"),
                &format!("{:.4}", p + 0.04),
                &format!("{:.4}", p - 0.04),
                &format!("{p:.4}"),
                1_000_000,
            ));
            t += 60;
        }
        assert!(first_long(&bars, &strat).is_none());
    }

    #[test]
    fn factory_round_trip_kinds() {
        // Every new strategy MUST be reachable via from_kind() — otherwise
        // a DB row with the new strategy_type would 500 at engine boot.
        for slug in ["ma_cross_adx", "keltner_breakout", "ichimoku_cloud"] {
            let s = from_kind(slug, &serde_json::json!({})).expect(slug);
            let expected = match slug {
                "ma_cross_adx" => StrategyKind::MaCrossAdx,
                "keltner_breakout" => StrategyKind::KeltnerBreakout,
                "ichimoku_cloud" => StrategyKind::IchimokuCloud,
                _ => unreachable!(),
            };
            assert_eq!(s.kind(), expected, "factory returned wrong kind for {slug}");
        }
    }
}
