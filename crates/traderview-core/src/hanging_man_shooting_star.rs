//! Hanging Man / Inverted Shooting Star — single-bar reversal candle
//! patterns at trend extremes.
//!
//! Both have the same body geometry as the existing `Hammer` /
//! `ShootingStar` patterns from `candle_patterns`, but with a TREND
//! CONTEXT filter:
//!
//!   Hanging Man    : Hammer-shape body that appears at the top of
//!                    an uptrend (bearish signal — same shape as
//!                    Hammer but trend reversal direction differs)
//!   Inverted Hammer: ShootingStar-shape body at the bottom of a
//!                    downtrend (bullish signal — small body near low
//!                    with long upper wick, but in a downtrend means
//!                    sellers attempted to push down and failed)
//!
//! Trend context filter: prior `trend_period` bars must show clear
//! direction (close at end > close at start for uptrend, etc.).
//!
//! Pure compute. Default trend_period = 5.
//! Companion to `candle_patterns`, `pinball_setup`,
//! `key_reversal_bar`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub open: f64, pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HangingShootingReport {
    pub hanging_man: Vec<bool>,
    pub inverted_hammer: Vec<bool>,
    pub trend_period: usize,
}

pub fn compute(bars: &[Bar], trend_period: usize) -> HangingShootingReport {
    let n = bars.len();
    let mut report = HangingShootingReport {
        hanging_man: vec![false; n],
        inverted_hammer: vec![false; n],
        trend_period,
    };
    if trend_period < 2 || n < trend_period + 1 { return report; }
    if bars.iter().any(|b| !b.open.is_finite() || !b.high.is_finite()
        || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    for i in trend_period..n {
        let bar = bars[i];
        let prior_first_close = bars[i - trend_period].close;
        let prior_last_close = bars[i - 1].close;
        let in_uptrend = prior_last_close > prior_first_close;
        let in_downtrend = prior_last_close < prior_first_close;
        if in_uptrend && is_hammer_shape(bar) {
            report.hanging_man[i] = true;
        }
        if in_downtrend && is_shooting_star_shape(bar) {
            report.inverted_hammer[i] = true;
        }
    }
    report
}

/// Small body near top, lower wick ≥ 2× body, upper wick small.
fn is_hammer_shape(b: Bar) -> bool {
    let range = b.high - b.low;
    if range <= 0.0 { return false; }
    let body = (b.close - b.open).abs();
    let upper = b.high - b.close.max(b.open);
    let lower = b.close.min(b.open) - b.low;
    body > 0.0 && lower >= 2.0 * body && upper <= body
}

/// Small body near bottom, upper wick ≥ 2× body, lower wick at most
/// equal to body.
fn is_shooting_star_shape(b: Bar) -> bool {
    let range = b.high - b.low;
    if range <= 0.0 { return false; }
    let body = (b.close - b.open).abs();
    let upper = b.high - b.close.max(b.open);
    let lower = b.close.min(b.open) - b.low;
    body > 0.0 && upper >= 2.0 * body && lower <= body
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(o: f64, h: f64, l: f64, c: f64) -> Bar {
        Bar { open: o, high: h, low: l, close: c }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 30];
        let r = compute(&bars, 1);
        assert!(!r.hanging_man.iter().any(|x| *x));
        let r2 = compute(&bars[..3], 5);
        assert!(!r2.hanging_man.iter().any(|x| *x));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        bars[5] = bar(f64::NAN, 101.0, 99.0, 100.5);
        let r = compute(&bars, 5);
        assert!(!r.hanging_man.iter().any(|x| *x));
    }

    #[test]
    fn hanging_man_at_uptrend_top_detected() {
        // 5 rising bars then a hammer-shape bar.
        let mut bars: Vec<_> = (0..5).map(|i| {
            let p = 100.0 + i as f64;
            bar(p, p + 0.5, p - 0.5, p + 0.4)
        }).collect();
        // Hammer-shape: open=105, high=105.1, low=100, close=105.1.
        // body = 0.1, upper = 0 (close at high), lower = 5.0 (50× body).
        bars.push(bar(105.0, 105.1, 100.0, 105.1));
        let r = compute(&bars, 5);
        assert!(r.hanging_man[5]);
    }

    #[test]
    fn inverted_hammer_at_downtrend_bottom_detected() {
        let mut bars: Vec<_> = (0..5).map(|i| {
            let p = 100.0 - i as f64;
            bar(p, p + 0.5, p - 0.5, p - 0.4)
        }).collect();
        // Shooting-star-shape: open=95, high=100, low=94.9, close=95.1.
        // body = 0.1, upper = 4.9 (49× body), lower = 0.1 (= body).
        bars.push(bar(95.0, 100.0, 94.9, 95.1));
        let r = compute(&bars, 5);
        assert!(r.inverted_hammer[5]);
    }

    #[test]
    fn hammer_shape_in_downtrend_not_hanging_man() {
        // Hammer geometry but downtrend → no hanging man (would be a
        // bullish hammer instead, handled by candle_patterns).
        let mut bars: Vec<_> = (0..5).map(|i| {
            let p = 100.0 - i as f64;
            bar(p, p + 0.5, p - 0.5, p - 0.4)
        }).collect();
        bars.push(bar(95.0, 95.5, 90.0, 95.2));
        let r = compute(&bars, 5);
        assert!(!r.hanging_man[5]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars, 5);
        assert_eq!(r.hanging_man.len(), 10);
        assert_eq!(r.inverted_hammer.len(), 10);
    }
}
