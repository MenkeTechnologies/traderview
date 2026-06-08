//! TTM Squeeze Momentum — John Carter ("Trade the Markets", 2007).
//!
//! Entry fires the bar AFTER a squeeze release while the linear-regression
//! momentum histogram is positive and accelerating (long) or negative and
//! decelerating (short). The squeeze (BB inside KC) signals coiled
//! volatility; the directional break is the play.
//!
//! Entry (long):
//!   squeeze_on[i-1] == true        (squeeze WAS on)
//!   squeeze_on[i]   == false       (released this bar)
//!   momentum[i]     >  0
//!   momentum[i]     >  momentum[i-1]   (still expanding)
//!
//! Entry (short): mirror — momentum < 0 and falling.
//!
//! Exit: momentum crosses zero in the unfavorable direction, OR
//! ATR(period) × atr_stop_mult trailing stop against the anchor high/low.

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::indicators;
use crate::models::PriceBar;
use crate::ttm_squeeze;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    pub period: usize,
    pub bb_mult: f64,
    pub kc_mult: f64,
    /// Window after a release in which entry is still valid. TTM
    /// momentum lags 1-3 bars behind the BB/KC release, so requiring the
    /// release on the EXACT current bar misses real entries.
    pub release_lookback: usize,
    pub atr_period: usize,
    pub atr_stop_mult: f64,
    pub atr_take_profit_mult: f64,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            period: 20,
            bb_mult: 2.0,
            kc_mult: 1.5,
            release_lookback: 5,
            atr_period: 14,
            atr_stop_mult: 2.0,
            atr_take_profit_mult: 3.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TtmSqueeze { pub rules: Rules }

impl TtmSqueeze {
    pub fn new(rules: Rules) -> Self { Self { rules } }
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone()).unwrap_or_default();
        Self { rules }
    }
}

fn ttm_bars(bars: &[PriceBar]) -> Vec<ttm_squeeze::Bar> {
    use rust_decimal::prelude::ToPrimitive;
    bars.iter()
        .map(|b| ttm_squeeze::Bar {
            high: b.high.to_f64().unwrap_or(0.0),
            low: b.low.to_f64().unwrap_or(0.0),
            close: b.close.to_f64().unwrap_or(0.0),
        })
        .collect()
}

impl Strategy for TtmSqueeze {
    fn kind(&self) -> StrategyKind { StrategyKind::TtmSqueeze }

    fn min_bars(&self) -> usize {
        self.rules.period.max(self.rules.atr_period + 1) + 2
    }

    fn evaluate_entry(&self, bars: &[PriceBar], side_mode: SideMode) -> Option<EntrySignal> {
        if bars.len() < self.min_bars() {
            return None;
        }
        let report = ttm_squeeze::compute(
            &ttm_bars(bars),
            self.rules.period,
            self.rules.bb_mult,
            self.rules.kc_mult,
        );
        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);

        let i = bars.len() - 1;
        let prev = i - 1;

        let squeeze_now = report.squeeze_on.get(i).copied().flatten()?;
        let mom_now = report.momentum.get(i).copied().flatten()?;
        let mom_prev = report.momentum.get(prev).copied().flatten()?;
        let atr_now = atr.get(i).copied().flatten()?;
        if atr_now <= 0.0 { return None; }
        let close_now = closes[i];

        // Squeeze must be CURRENTLY off (released) AND there must have
        // been at least one squeeze_on=true bar inside the last
        // release_lookback bars — the coil that preceded the move.
        if squeeze_now { return None; }
        let lookback_start = i.saturating_sub(self.rules.release_lookback);
        let had_squeeze_recently = (lookback_start..i)
            .any(|k| report.squeeze_on.get(k).copied().flatten() == Some(true));
        if !had_squeeze_recently {
            return None;
        }
        let just_released = had_squeeze_recently;

        let want_long = matches!(side_mode, SideMode::Long | SideMode::Both)
            && mom_now > 0.0
            && mom_now > mom_prev;
        let want_short = matches!(side_mode, SideMode::Short | SideMode::Both)
            && mom_now < 0.0
            && mom_now < mom_prev;

        if want_long {
            let stop = close_now - self.rules.atr_stop_mult * atr_now;
            let tp = close_now + self.rules.atr_take_profit_mult * atr_now;
            let stop_distance = (close_now - stop).max(0.01);
            Some(EntrySignal {
                side: Side::Buy,
                entry_price: close_now,
                stop_distance,
                trigger_index: i,
                stop_price: stop.max(0.01),
                take_profit_price: tp,
                kind: "ttm_squeeze",
                diagnostic: serde_json::json!({
                    "squeeze_just_released": just_released,
                    "momentum": mom_now,
                    "momentum_prev": mom_prev,
                    "atr": atr_now,
                }),
            })
        } else if want_short {
            let stop = close_now + self.rules.atr_stop_mult * atr_now;
            let tp = (close_now - self.rules.atr_take_profit_mult * atr_now).max(0.01);
            let stop_distance = (stop - close_now).max(0.01);
            Some(EntrySignal {
                side: Side::Sell,
                entry_price: close_now,
                stop_distance,
                trigger_index: i,
                stop_price: stop,
                take_profit_price: tp,
                kind: "ttm_squeeze",
                diagnostic: serde_json::json!({
                    "squeeze_just_released": just_released,
                    "momentum": mom_now,
                    "momentum_prev": mom_prev,
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
        let report = ttm_squeeze::compute(
            &ttm_bars(bars),
            self.rules.period,
            self.rules.bb_mult,
            self.rules.kc_mult,
        );
        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);

        let i = bars.len() - 1;
        let prev = i - 1;
        let close_now = closes[i];
        let atr_now = atr.get(i).copied().flatten()?;
        let mom_now = report.momentum.get(i).copied().flatten()?;
        let mom_prev = report.momentum.get(prev).copied().flatten()?;

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
                if mom_prev > 0.0 && mom_now <= 0.0 {
                    return Some(ExitSignal {
                        reason: "momentum_flip_negative",
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
                if mom_prev < 0.0 && mom_now >= 0.0 {
                    return Some(ExitSignal {
                        reason: "momentum_flip_positive",
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

    /// 50 very tight bars (squeeze on) → 5 expanding bars with rising
    /// closes (squeeze releases + momentum positive + accelerating).
    fn squeeze_release_window() -> Vec<PriceBar> {
        let mut bars = Vec::new();
        let mut t = 1_700_000_000_i64;
        for _ in 0..50 {
            bars.push(bar(t, "100.00", "100.05", "99.95", "100.00", 1_000_000));
            t += 60;
        }
        for i in 0..5 {
            let p = 100.0 + (i as f64 + 1.0) * 1.2;
            bars.push(bar(
                t,
                &format!("{:.2}", p - 0.3),
                &format!("{:.2}", p + 0.8),
                &format!("{:.2}", p - 0.6),
                &format!("{p:.2}"),
                2_000_000,
            ));
            t += 60;
        }
        bars
    }

    fn first_long(bars: &[PriceBar], strat: &TtmSqueeze) -> Option<EntrySignal> {
        for end in strat.min_bars()..=bars.len() {
            if let Some(s) = strat.evaluate_entry(&bars[..end], SideMode::Long) {
                return Some(s);
            }
        }
        None
    }

    #[test]
    fn entry_fires_on_squeeze_release_with_positive_accelerating_momentum() {
        let strat = TtmSqueeze::new(Rules::default());
        let bars = squeeze_release_window();
        let sig = first_long(&bars, &strat).expect("squeeze release must fire entry");
        assert_eq!(sig.side, Side::Buy);
        let mom = sig.diagnostic.get("momentum").and_then(|v| v.as_f64()).unwrap();
        let mom_prev = sig.diagnostic.get("momentum_prev").and_then(|v| v.as_f64()).unwrap();
        assert!(mom > 0.0 && mom > mom_prev, "momentum {mom} > prev {mom_prev}");
    }

    #[test]
    fn entry_blocked_on_pure_squeeze_with_no_release() {
        let strat = TtmSqueeze::new(Rules::default());
        // 80 bars of pure squeeze — never releases.
        let bars: Vec<PriceBar> = (0..80)
            .map(|i| bar(1_700_000_000 + i * 60, "100.00", "100.05", "99.95", "100.00", 1_000_000))
            .collect();
        assert!(first_long(&bars, &strat).is_none());
    }

    #[test]
    fn kind_and_min_bars() {
        let strat = TtmSqueeze::new(Rules::default());
        assert_eq!(strat.kind().as_str(), "ttm_squeeze");
        assert_eq!(strat.min_bars(), 22);
    }
}
