//! ADX pullback — Linda Raschke's "Holy Grail" (Street Smarts, 1996).
//!
//! Trend-CONTINUATION, not reversal: in an established trend (ADX above
//! adx_min with the directional line confirming), wait for price to
//! pull back and TOUCH the EMA(ema_period), then enter on the close
//! breaking the prior bar's high (long) — the resumption, not the dip.
//!
//! Entry (long):
//!   ADX(adx_period) > adx_min AND +DI > −DI
//!   AND some bar within the last `touch_age` bars had low ≤ EMA20
//!   AND this bar's close > prior bar's high (resumption trigger)
//! Entry (short): full mirror (high ≥ EMA20 touch, close < prior low).
//!
//! Stop:   the pullback extreme (min low / max high over the touch
//!         window) ∓ an ATR buffer — below it the pullback became a
//!         reversal and the continuation thesis is dead.
//! Target: ATR-multiple beyond entry.
//! Exit:   close crossing the EMA by a full ATR buffer against the
//!         position — trend support broken.

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::indicators;
use crate::models::PriceBar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    pub adx_period: usize,
    /// Raschke's threshold is 30 — stricter than the generic 25.
    pub adx_min: f64,
    pub ema_period: usize,
    /// The EMA touch must have happened within this many bars.
    pub touch_age: usize,
    pub atr_period: usize,
    /// ATR buffer beyond the pullback extreme for the stop.
    pub atr_stop_buffer: f64,
    pub atr_take_profit_mult: f64,
    /// Exit when close crosses the EMA against the position by this
    /// many ATRs.
    pub atr_exit_buffer: f64,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            adx_period: 14,
            adx_min: 30.0,
            ema_period: 20,
            touch_age: 3,
            atr_period: 14,
            atr_stop_buffer: 0.25,
            atr_take_profit_mult: 3.0,
            atr_exit_buffer: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdxPullback {
    pub rules: Rules,
}

impl AdxPullback {
    pub fn new(rules: Rules) -> Self {
        Self { rules }
    }
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone()).unwrap_or_default();
        Self { rules }
    }
}

impl Strategy for AdxPullback {
    fn kind(&self) -> StrategyKind {
        StrategyKind::AdxPullback
    }

    fn min_bars(&self) -> usize {
        (self.rules.adx_period * 2)
            .max(self.rules.ema_period)
            .max(self.rules.atr_period)
            + self.rules.touch_age
            + 2
    }

    fn evaluate_entry(&self, bars: &[PriceBar], side_mode: SideMode) -> Option<EntrySignal> {
        if bars.len() < self.min_bars() {
            return None;
        }
        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let ema = indicators::ema(&closes, self.rules.ema_period);
        let adx = indicators::adx(&highs, &lows, &closes, self.rules.adx_period);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);

        let i = bars.len() - 1;
        let adx_now = adx.adx.get(i).copied().flatten()?;
        let plus_di = adx.plus_di.get(i).copied().flatten()?;
        let minus_di = adx.minus_di.get(i).copied().flatten()?;
        let atr_now = atr.get(i).copied().flatten()?;
        if atr_now <= 0.0 || adx_now < self.rules.adx_min {
            return None;
        }
        // EMA touch within the freshness window (the trigger bar itself
        // doesn't count — it's the resumption, not the dip).
        let touch_from = i.saturating_sub(self.rules.touch_age);
        let touched_long = (touch_from..i).any(|j| {
            ema.get(j)
                .copied()
                .flatten()
                .is_some_and(|e| lows[j] <= e)
        });
        let touched_short = (touch_from..i).any(|j| {
            ema.get(j)
                .copied()
                .flatten()
                .is_some_and(|e| highs[j] >= e)
        });
        let close_now = closes[i];
        let diagnostic = serde_json::json!({
            "adx": adx_now, "plus_di": plus_di, "minus_di": minus_di,
            "ema": ema.get(i).copied().flatten(), "atr": atr_now,
        });

        let want_long = matches!(side_mode, SideMode::Long | SideMode::Both)
            && plus_di > minus_di
            && touched_long
            && close_now > highs[i - 1];
        let want_short = matches!(side_mode, SideMode::Short | SideMode::Both)
            && minus_di > plus_di
            && touched_short
            && close_now < lows[i - 1];

        if want_long {
            let pullback_low = lows[touch_from..i].iter().cloned().fold(f64::MAX, f64::min);
            let stop = (pullback_low - self.rules.atr_stop_buffer * atr_now).max(0.01);
            Some(EntrySignal {
                side: Side::Buy,
                entry_price: close_now,
                stop_distance: (close_now - stop).max(0.01),
                trigger_index: i,
                stop_price: stop,
                take_profit_price: close_now + self.rules.atr_take_profit_mult * atr_now,
                kind: "adx_pullback",
                diagnostic,
            })
        } else if want_short {
            let pullback_high = highs[touch_from..i].iter().cloned().fold(f64::MIN, f64::max);
            let stop = pullback_high + self.rules.atr_stop_buffer * atr_now;
            Some(EntrySignal {
                side: Side::Sell,
                entry_price: close_now,
                stop_distance: (stop - close_now).max(0.01),
                trigger_index: i,
                stop_price: stop,
                take_profit_price: (close_now
                    - self.rules.atr_take_profit_mult * atr_now)
                    .max(0.01),
                kind: "adx_pullback",
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
        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let ema = indicators::ema(&closes, self.rules.ema_period);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);
        let i = bars.len() - 1;
        let ema_now = ema.get(i).copied().flatten()?;
        let atr_now = atr.get(i).copied().flatten()?;
        let close_now = closes[i];
        let buffer = self.rules.atr_exit_buffer * atr_now;
        let broken = match side {
            Side::Buy => close_now < ema_now - buffer,
            Side::Sell => close_now > ema_now + buffer,
        };
        broken.then_some(ExitSignal {
            reason: "ema_support_broken",
            exit_price: close_now,
            trigger_index: i,
        })
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
            high: Decimal::from_str(&format!("{:.4}", p + 0.3)).unwrap(),
            low: Decimal::from_str(&format!("{:.4}", p - 0.3)).unwrap(),
            close: Decimal::from_str(&format!("{p:.4}")).unwrap(),
            volume: Decimal::from(1_000_000u64),
            source: "test".into(),
        }
    }

    /// Pre-verified against the exact indicator implementations: at the
    /// resumption bar (close 126.5 > prior high 125.3) ADX = 70.2,
    /// +DI 43.5 > −DI 31.0, and bars 65–66 touched EMA20 (126.2/126.1).
    fn pullback_resumption() -> Vec<PriceBar> {
        let mut closes: Vec<f64> = vec![100.0; 20];
        closes.extend((0..40).map(|i| 100.0 + 0.8 * (i + 1) as f64)); // trend to 132
        closes.extend((0..7).map(|i| 132.0 - 1.0 * (i + 1) as f64)); // pullback to 125
        closes.push(126.5); // resumption: breaks prior high 125.3
        let mut t = 1_700_000_000_i64;
        closes
            .into_iter()
            .map(|p| {
                let b = bar(t, p);
                t += 60;
                b
            })
            .collect()
    }

    #[test]
    fn pullback_touch_plus_resumption_fires_long() {
        let bars = pullback_resumption();
        let strat = AdxPullback::new(Rules::default());
        let sig = strat
            .evaluate_entry(&bars, SideMode::Long)
            .expect("verified pullback resumption must fire");
        assert_eq!(sig.side, Side::Buy);
        let adx = sig.diagnostic.get("adx").and_then(|v| v.as_f64()).unwrap();
        assert!(adx > 30.0);
        // Stop sits below the pullback extreme (124.7), not below entry.
        assert!(sig.stop_price < 124.7);
    }

    #[test]
    fn no_entry_during_the_dip_itself() {
        // Same fixture minus the resumption bar: price is AT the EMA
        // touch but no prior-high break yet — the dip is not the entry.
        let mut bars = pullback_resumption();
        bars.pop();
        assert!(AdxPullback::new(Rules::default())
            .evaluate_entry(&bars, SideMode::Long)
            .is_none());
    }

    #[test]
    fn no_entry_without_an_ema_touch() {
        // Shallow 2-bar dip that stays far above EMA20, then a break of
        // the prior high: trend + trigger but NO touch → no setup.
        let mut closes: Vec<f64> = vec![100.0; 20];
        closes.extend((0..40).map(|i| 100.0 + 0.8 * (i + 1) as f64));
        closes.extend([131.0, 130.5, 132.0]); // dip holds ~5 above EMA
        let mut t = 1_700_000_000_i64;
        let bars: Vec<PriceBar> = closes
            .into_iter()
            .map(|p| {
                let b = bar(t, p);
                t += 60;
                b
            })
            .collect();
        assert!(AdxPullback::new(Rules::default())
            .evaluate_entry(&bars, SideMode::Long)
            .is_none());
    }

    #[test]
    fn long_exits_when_ema_support_breaks() {
        // Trend up, then a hard slide a full ATR below the EMA.
        let mut closes: Vec<f64> = vec![100.0; 20];
        closes.extend((0..40).map(|i| 100.0 + 0.8 * (i + 1) as f64));
        closes.extend((0..15).map(|i| 132.0 - 2.0 * (i + 1) as f64)); // to 102
        let mut t = 1_700_000_000_i64;
        let bars: Vec<PriceBar> = closes
            .into_iter()
            .map(|p| {
                let b = bar(t, p);
                t += 60;
                b
            })
            .collect();
        let e = AdxPullback::new(Rules::default())
            .evaluate_exit(&bars, Side::Buy, 0.0, 0.0)
            .expect("hard break must exit");
        assert_eq!(e.reason, "ema_support_broken");
    }

    #[test]
    fn factory_reaches_adx_pullback() {
        let s = from_kind("adx_pullback", &serde_json::json!({})).expect("registered");
        assert_eq!(s.kind(), StrategyKind::AdxPullback);
    }
}
