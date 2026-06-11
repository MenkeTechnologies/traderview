//! Bollinger Squeeze Breakout — John Bollinger's volatility-contraction
//! setup. When bandwidth (BBW) drops to its N-bar low, the band is
//! "squeezed" and an expansion + directional break is statistically
//! likely.
//!
//! Entry (long):
//!   BBW(20, k=2) percentile-rank over `squeeze_lookback` bars <= `squeeze_pct` AND
//!   close > BB.upper (band re-expansion in the long direction)
//!
//! Entry (short): mirror — BBW squeezed AND close < BB.lower.
//!
//! Exit: close crosses BB.middle (target hit), OR ATR trailing stop.
//!
//! Sizing: stop_distance = (entry - stop).abs() where stop = BB.lower
//! (long) or BB.upper (short) — the band the trade departed from is the
//! natural invalidation level.

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::bollinger_band_width;
use crate::indicators;
use crate::models::PriceBar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    pub bb_period: usize,
    pub bb_k: f64,
    /// Window over which BBW percentile rank is computed.
    pub squeeze_lookback: usize,
    /// Squeeze fires when BBW is at or below this percentile of its
    /// trailing lookback. 0.10 = 10th percentile (bottom decile).
    pub squeeze_pct: f64,
    pub atr_period: usize,
    pub atr_stop_mult: f64,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            bb_period: 20,
            bb_k: 2.0,
            squeeze_lookback: 100,
            squeeze_pct: 0.10,
            atr_period: 14,
            atr_stop_mult: 2.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BbSqueeze {
    pub rules: Rules,
}

impl BbSqueeze {
    pub fn new(rules: Rules) -> Self {
        Self { rules }
    }
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone()).unwrap_or_default();
        Self { rules }
    }
}

/// Percentile rank of `value` within `window`, ignoring None entries.
/// Returns None when fewer than `min_obs` finite observations exist.
fn percentile_rank(window: &[Option<f64>], value: f64, min_obs: usize) -> Option<f64> {
    let mut count = 0usize;
    let mut below = 0usize;
    for x in window.iter().flatten() {
        if x.is_finite() {
            count += 1;
            if *x < value {
                below += 1;
            }
        }
    }
    if count < min_obs {
        return None;
    }
    Some(below as f64 / count as f64)
}

impl Strategy for BbSqueeze {
    fn kind(&self) -> StrategyKind {
        StrategyKind::BbSqueeze
    }

    fn min_bars(&self) -> usize {
        self.rules
            .squeeze_lookback
            .max(self.rules.bb_period + 1)
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
        let bbw = bollinger_band_width::compute(&closes, self.rules.bb_period, self.rules.bb_k);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);

        let i = bars.len() - 1;
        let close_now = closes[i];
        let atr_now = atr.get(i).copied().flatten()?;
        if atr_now <= 0.0 {
            return None;
        }
        // Squeeze condition is evaluated on the bar BEFORE the breakout —
        // a breakout bar's own range widens the sigma and lifts its BBW
        // out of the bottom decile, masking the setup. Bollinger's
        // canonical rule: bandwidth was contracted leading into the move.
        let prev = i.saturating_sub(1);
        let bbw_prev = bbw.band_width.get(prev).copied().flatten()?;
        let upper_now = bbw.upper.get(i).copied().flatten()?;
        let lower_now = bbw.lower.get(i).copied().flatten()?;
        let middle_now = bbw.middle.get(i).copied().flatten()?;

        let start = prev.saturating_sub(self.rules.squeeze_lookback);
        let window = &bbw.band_width[start..prev];
        let pct = percentile_rank(window, bbw_prev, self.rules.squeeze_lookback / 2)?;
        if pct > self.rules.squeeze_pct {
            return None;
        }
        let bbw_now = bbw_prev;

        let want_long =
            matches!(side_mode, SideMode::Long | SideMode::Both) && close_now > upper_now;
        let want_short =
            matches!(side_mode, SideMode::Short | SideMode::Both) && close_now < lower_now;

        if want_long {
            // Initial stop at the LOWER band — that's where the trade is
            // invalidated. Fall back to ATR stop if the lower band is
            // unreasonably close (degenerate squeeze).
            let band_stop = lower_now;
            let atr_stop = close_now - self.rules.atr_stop_mult * atr_now;
            let stop = band_stop.min(atr_stop);
            let stop_distance = (close_now - stop).max(0.01);
            // Project TP by the same band-width that triggered the
            // breakout, measured from the middle band. Previously this
            // was `middle_now` which sits BELOW the entry on a long
            // breakout (since entry > upper > middle), making the TP
            // immediately satisfied / wrong-direction. New target sits
            // strictly above entry by the width of the upper half-band.
            let band_width = (upper_now - middle_now).max(0.01);
            let take_profit = close_now + band_width;
            Some(EntrySignal {
                side: Side::Buy,
                entry_price: close_now,
                stop_distance,
                trigger_index: i,
                stop_price: stop.max(0.01),
                take_profit_price: take_profit,
                kind: "bb_squeeze",
                diagnostic: serde_json::json!({
                    "bbw": bbw_now,
                    "bbw_pct": pct,
                    "bb_upper": upper_now,
                    "bb_middle": middle_now,
                    "bb_lower": lower_now,
                    "atr": atr_now,
                }),
            })
        } else if want_short {
            let band_stop = upper_now;
            let atr_stop = close_now + self.rules.atr_stop_mult * atr_now;
            let stop = band_stop.max(atr_stop);
            let stop_distance = (stop - close_now).max(0.01);
            // TP strictly BELOW entry on a short breakout; mirror the
            // long-side fix. Old `middle_now` was ABOVE entry for shorts
            // (entry < lower < middle), making the TP wrong-direction.
            let band_width = (middle_now - lower_now).max(0.01);
            let take_profit = (close_now - band_width).max(0.01);
            Some(EntrySignal {
                side: Side::Sell,
                entry_price: close_now,
                stop_distance,
                trigger_index: i,
                stop_price: stop,
                take_profit_price: take_profit,
                kind: "bb_squeeze",
                diagnostic: serde_json::json!({
                    "bbw": bbw_now,
                    "bbw_pct": pct,
                    "bb_upper": upper_now,
                    "bb_middle": middle_now,
                    "bb_lower": lower_now,
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
        let bbw = bollinger_band_width::compute(&closes, self.rules.bb_period, self.rules.bb_k);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);

        let i = bars.len() - 1;
        let prev = i - 1;
        let close_now = closes[i];
        let close_prev = closes[prev];
        let atr_now = atr.get(i).copied().flatten()?;
        let middle_now = bbw.middle.get(i).copied().flatten()?;
        let middle_prev = bbw.middle.get(prev).copied().flatten()?;

        match side {
            Side::Buy => {
                // ATR trailing stop anchored to high-water.
                let trail = anchor_high - self.rules.atr_stop_mult * atr_now;
                if lows[i] <= trail {
                    return Some(ExitSignal {
                        reason: "atr_trailing_stop",
                        exit_price: trail.max(0.01),
                        trigger_index: i,
                    });
                }
                // Target: price re-crosses the BB middle from above.
                if close_prev > middle_prev && close_now <= middle_now {
                    return Some(ExitSignal {
                        reason: "bb_middle_cross",
                        exit_price: middle_now,
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
                if close_prev < middle_prev && close_now >= middle_now {
                    return Some(ExitSignal {
                        reason: "bb_middle_cross",
                        exit_price: middle_now,
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

    /// 100 bars of high-vol noise (sets BBW baseline), then 30 super-tight
    /// bars (BBW drops to its lookback minimum = squeeze), then one
    /// breakout bar that closes above the upper band.
    fn squeeze_release_window() -> Vec<PriceBar> {
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        // 100 high-vol bars: ±2.0 swings.
        for i in 0..100 {
            let p = 100.0 + (i as f64 * 0.6).sin() * 2.0;
            bars.push(bar(
                t,
                &format!("{p:.2}"),
                &format!("{:.2}", p + 1.0),
                &format!("{:.2}", p - 1.0),
                &format!("{p:.2}"),
                1_000_000,
            ));
            t += 60;
        }
        // 30 tight bars (BBW collapses).
        for _ in 0..30 {
            bars.push(bar(t, "100.00", "100.10", "99.90", "100.00", 1_000_000));
            t += 60;
        }
        // Breakout bar: close well above the (now narrow) upper band.
        bars.push(bar(t, "100.05", "102.00", "100.00", "101.80", 4_000_000));
        bars
    }

    #[test]
    fn entry_fires_on_squeeze_release_long() {
        let strat = BbSqueeze::new(Rules::default());
        let bars = squeeze_release_window();
        let sig = strat
            .evaluate_entry(&bars, SideMode::Long)
            .expect("squeeze release must produce long entry");
        assert_eq!(sig.side, Side::Buy);
        let bbw_pct = sig
            .diagnostic
            .get("bbw_pct")
            .and_then(|v| v.as_f64())
            .unwrap();
        assert!(
            bbw_pct <= 0.10,
            "BBW percentile {bbw_pct} must be in bottom decile"
        );
        let upper = sig
            .diagnostic
            .get("bb_upper")
            .and_then(|v| v.as_f64())
            .unwrap();
        assert!(
            sig.entry_price > upper,
            "entry close must exceed upper band"
        );
    }

    #[test]
    fn entry_blocked_when_no_squeeze_present() {
        let strat = BbSqueeze::new(Rules::default());
        // 130 bars of high vol — BBW never reaches its bottom decile.
        let bars: Vec<PriceBar> = (0..130)
            .map(|i| {
                let p = 100.0 + (i as f64 * 0.5).sin() * 3.0;
                bar(
                    1_700_000_000 + i * 60,
                    &format!("{p:.2}"),
                    &format!("{:.2}", p + 1.5),
                    &format!("{:.2}", p - 1.5),
                    &format!("{p:.2}"),
                    1_000_000,
                )
            })
            .collect();
        assert!(strat.evaluate_entry(&bars, SideMode::Long).is_none());
    }

    #[test]
    fn entry_blocked_when_squeezed_but_no_breakout() {
        let strat = BbSqueeze::new(Rules::default());
        let mut bars = squeeze_release_window();
        let last = bars.len() - 1;
        // Pull the close back BELOW the squeezed mean (~100.0). The
        // squeeze IS present (prior bar is flat) but the breakout
        // condition close > BB.upper isn't met.
        bars[last].close = Decimal::from_str("99.95").unwrap();
        bars[last].high = Decimal::from_str("99.98").unwrap();
        bars[last].low = Decimal::from_str("99.92").unwrap();
        bars[last].open = Decimal::from_str("99.97").unwrap();
        assert!(strat.evaluate_entry(&bars, SideMode::Long).is_none());
    }

    #[test]
    fn kind_and_min_bars() {
        let strat = BbSqueeze::new(Rules::default());
        assert_eq!(strat.kind().as_str(), "bb_squeeze");
        // squeeze_lookback (100) wins → 101.
        assert_eq!(strat.min_bars(), 101);
    }
}
