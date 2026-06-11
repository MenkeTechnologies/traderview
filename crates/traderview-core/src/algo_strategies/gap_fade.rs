//! Gap fade — fade moderate opening gaps back toward the prior close.
//!
//! The documented gap statistic: MODERATE gaps (roughly 1–4%) fill the
//! same session more often than not, while LARGE gaps tend to continue
//! (that regime is gap-and-go, a different trade). This strategy takes
//! only the fade window:
//!
//! Entry:
//!   long  on a gap DOWN of gap_min_pct..=gap_max_pct vs the prior
//!         session close, after `confirm_bars` bars of the new session,
//!         once price reclaims above the session open (the fade has
//!         started) and the gap hasn't already filled. Only within the
//!         first `max_entry_bars` bars — a stale gap is dead.
//!   short on the mirrored gap UP.
//!
//! Target: the prior close — the gap fill IS the trade.
//! Stop:   ATR-multiple beyond entry.
//! Exit:   close crossing the prior close ("gap_filled").
//!
//! Sessions are found from bar timestamps (UTC date change splits US
//! RTH sessions correctly: prior close ~20:00Z, next open ~13:30Z the
//! following UTC day), so the strategy needs a window spanning the
//! prior session's tail.

use super::{EntrySignal, ExitSignal, Side, SideMode, Strategy, StrategyKind};
use crate::indicators;
use crate::models::PriceBar;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules {
    /// Smallest gap worth fading, percent of prior close.
    pub gap_min_pct: f64,
    /// Largest gap to fade — beyond this gaps tend to CONTINUE.
    pub gap_max_pct: f64,
    /// Bars of the new session to wait before entering.
    pub confirm_bars: usize,
    /// Entry window: no fresh entries after this many session bars.
    pub max_entry_bars: usize,
    pub atr_period: usize,
    pub atr_stop_mult: f64,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            gap_min_pct: 1.0,
            gap_max_pct: 4.0,
            confirm_bars: 3,
            max_entry_bars: 30,
            atr_period: 14,
            atr_stop_mult: 1.5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GapFade {
    pub rules: Rules,
}

/// (session_start index, prior session close, session open, gap %).
struct Gap {
    session_start: usize,
    prior_close: f64,
    gap_pct: f64,
}

impl GapFade {
    pub fn new(rules: Rules) -> Self {
        Self { rules }
    }
    pub fn from_json(entry_rules: &serde_json::Value) -> Self {
        let rules = serde_json::from_value::<Rules>(entry_rules.clone()).unwrap_or_default();
        Self { rules }
    }

    /// Locate the most recent session boundary in the window.
    fn gap(&self, bars: &[PriceBar]) -> Option<Gap> {
        let last_date = bars.last()?.bar_time.date_naive();
        let session_start = bars
            .iter()
            .position(|b| b.bar_time.date_naive() == last_date)?;
        if session_start == 0 {
            return None; // no prior-session tail in the window
        }
        let prior_close = bars[session_start - 1].close.to_f64().unwrap_or(0.0);
        let open = bars[session_start].open.to_f64().unwrap_or(0.0);
        if prior_close <= 0.0 || open <= 0.0 {
            return None;
        }
        Some(Gap {
            session_start,
            prior_close,
            gap_pct: (open / prior_close - 1.0) * 100.0,
        })
    }
}

impl Strategy for GapFade {
    fn kind(&self) -> StrategyKind {
        StrategyKind::GapFade
    }

    fn min_bars(&self) -> usize {
        self.rules.atr_period + self.rules.confirm_bars + 2
    }

    fn evaluate_entry(&self, bars: &[PriceBar], side_mode: SideMode) -> Option<EntrySignal> {
        if bars.len() < self.min_bars() {
            return None;
        }
        let g = self.gap(bars)?;
        let i = bars.len() - 1;
        let session_pos = i - g.session_start;
        if session_pos < self.rules.confirm_bars || session_pos > self.rules.max_entry_bars {
            return None;
        }
        let magnitude = g.gap_pct.abs();
        if magnitude < self.rules.gap_min_pct || magnitude > self.rules.gap_max_pct {
            return None;
        }
        let closes = indicators::closes(bars);
        let highs = indicators::highs(bars);
        let lows = indicators::lows(bars);
        let atr = indicators::atr(&highs, &lows, &closes, self.rules.atr_period);
        let atr_now = atr.get(i).copied().flatten()?;
        if atr_now <= 0.0 {
            return None;
        }
        let close_now = closes[i];
        let open = bars[g.session_start].open.to_f64().unwrap_or(0.0);

        let diagnostic = serde_json::json!({
            "gap_pct": g.gap_pct, "prior_close": g.prior_close,
            "session_open": open, "session_pos": session_pos, "atr": atr_now,
        });
        if g.gap_pct < 0.0 {
            // Gap DOWN → fade long toward the prior close.
            if !matches!(side_mode, SideMode::Long | SideMode::Both)
                || close_now <= open          // fade hasn't started
                || close_now >= g.prior_close // gap already filled
            {
                return None;
            }
            let stop = (close_now - self.rules.atr_stop_mult * atr_now).max(0.01);
            Some(EntrySignal {
                side: Side::Buy,
                entry_price: close_now,
                stop_distance: (close_now - stop).max(0.01),
                trigger_index: i,
                stop_price: stop,
                take_profit_price: g.prior_close,
                kind: "gap_fade",
                diagnostic,
            })
        } else {
            // Gap UP → fade short toward the prior close.
            if !matches!(side_mode, SideMode::Short | SideMode::Both)
                || close_now >= open
                || close_now <= g.prior_close
            {
                return None;
            }
            let stop = close_now + self.rules.atr_stop_mult * atr_now;
            Some(EntrySignal {
                side: Side::Sell,
                entry_price: close_now,
                stop_distance: (stop - close_now).max(0.01),
                trigger_index: i,
                stop_price: stop,
                take_profit_price: g.prior_close,
                kind: "gap_fade",
                diagnostic,
            })
        }
    }

    fn evaluate_exit(
        &self,
        bars: &[PriceBar],
        side: Side,
        _anchor_high: f64,
        _anchor_low: f64,
    ) -> Option<ExitSignal> {
        let g = self.gap(bars)?;
        let i = bars.len() - 1;
        let close_now = bars[i].close.to_f64().unwrap_or(0.0);
        let filled = match side {
            Side::Buy => close_now >= g.prior_close,
            Side::Sell => close_now <= g.prior_close,
        };
        filled.then_some(ExitSignal {
            reason: "gap_filled",
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

    fn bar(t: i64, o: f64, c: f64) -> PriceBar {
        let (lo, hi) = if o < c { (o, c) } else { (c, o) };
        PriceBar {
            symbol: "TEST".into(),
            interval: BarInterval::M1,
            bar_time: Utc.timestamp_opt(t, 0).unwrap(),
            open: Decimal::from_str(&format!("{o:.4}")).unwrap(),
            high: Decimal::from_str(&format!("{:.4}", hi + 0.1)).unwrap(),
            low: Decimal::from_str(&format!("{:.4}", lo - 0.1)).unwrap(),
            close: Decimal::from_str(&format!("{c:.4}")).unwrap(),
            volume: Decimal::from(1_000_000u64),
            source: "test".into(),
        }
    }

    const DAY1: i64 = 1_700_000_000; // mid-session UTC
    const DAY2: i64 = DAY1 + 86_400;

    /// Prior session flat at 100, next session opens gapped and drifts.
    fn window(gap_open: f64, drift_per_bar: f64, session_bars: usize) -> Vec<PriceBar> {
        let mut bars: Vec<PriceBar> = (0..30)
            .map(|i| bar(DAY1 + i * 60, 100.0, 100.0))
            .collect();
        let mut p = gap_open;
        for i in 0..session_bars {
            let next = gap_open + drift_per_bar * (i + 1) as f64;
            bars.push(bar(DAY2 + i as i64 * 60, p, next));
            p = next;
        }
        bars
    }

    #[test]
    fn moderate_gap_down_reclaim_fades_long_targeting_prior_close() {
        // 2% gap down (open 98), reclaiming +0.15/bar. After confirm_bars
        // the close is back above the open and below 100 → long, target
        // exactly the prior close.
        let bars = window(98.0, 0.15, 6);
        let strat = GapFade::new(Rules::default());
        let sig = strat
            .evaluate_entry(&bars, SideMode::Long)
            .expect("moderate reclaimed gap must fade");
        assert_eq!(sig.side, Side::Buy);
        assert!((sig.take_profit_price - 100.0).abs() < 1e-9);
        let gap = sig.diagnostic.get("gap_pct").and_then(|v| v.as_f64()).unwrap();
        assert!((gap + 2.0).abs() < 1e-9);
    }

    #[test]
    fn large_gap_is_gap_and_go_territory_no_fade() {
        // 6% gap down exceeds gap_max_pct — continuation regime.
        let bars = window(94.0, 0.15, 6);
        assert!(GapFade::new(Rules::default())
            .evaluate_entry(&bars, SideMode::Long)
            .is_none());
    }

    #[test]
    fn tiny_gap_is_noise_no_fade() {
        // 0.3% gap is under gap_min_pct.
        let bars = window(99.7, 0.05, 6);
        assert!(GapFade::new(Rules::default())
            .evaluate_entry(&bars, SideMode::Long)
            .is_none());
    }

    #[test]
    fn no_entry_before_reclaim_or_after_fill() {
        // Still sinking (close below open): no fade confirmation.
        let sinking = window(98.0, -0.1, 6);
        let strat = GapFade::new(Rules::default());
        assert!(strat.evaluate_entry(&sinking, SideMode::Long).is_none());
        // Already filled (close ran past 100): nothing left to capture.
        let filled = window(98.0, 0.6, 6);
        assert!(strat.evaluate_entry(&filled, SideMode::Long).is_none());
    }

    #[test]
    fn long_exits_when_gap_fills() {
        let bars = window(98.0, 0.3, 8); // close reaches 100.4 by bar 8
        let strat = GapFade::new(Rules::default());
        let e = strat
            .evaluate_exit(&bars, Side::Buy, 0.0, 0.0)
            .expect("gap fill must exit");
        assert_eq!(e.reason, "gap_filled");
    }

    #[test]
    fn gap_up_fades_short() {
        // 2% gap up (open 102) fading down −0.15/bar.
        let bars = window(102.0, -0.15, 6);
        let sig = GapFade::new(Rules::default())
            .evaluate_entry(&bars, SideMode::Short)
            .expect("reclaimed gap-up must fade short");
        assert_eq!(sig.side, Side::Sell);
        assert!((sig.take_profit_price - 100.0).abs() < 1e-9);
    }

    #[test]
    fn factory_reaches_gap_fade() {
        let s = from_kind("gap_fade", &serde_json::json!({})).expect("registered");
        assert_eq!(s.kind(), StrategyKind::GapFade);
    }
}
