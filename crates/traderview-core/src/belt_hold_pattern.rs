//! Belt Hold Pattern — single-bar reversal at extremes.
//!
//! Bullish Belt Hold (yorikiri):
//!   - Bullish marubozu-like body (close > open, body ≥ 80% of range)
//!   - Open at the bar's LOW (or within `wick_pct` of low)
//!   - Long lower shadow nearly absent
//!   - Trend filter: prior `trend_period` bars in downtrend (close at
//!     end < close at start)
//!
//! Bearish Belt Hold: mirrored — opens at the bar's HIGH, bearish body,
//! prior trend up.
//!
//! Pure compute. Default trend_period = 5, wick_pct = 0.05.
//! Companion to `spinning_top_marubozu`, `hanging_man_shooting_star`,
//! `kicker_pattern`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub open: f64, pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BeltHoldReport {
    pub bullish: Vec<bool>,
    pub bearish: Vec<bool>,
    pub trend_period: usize,
    pub wick_pct: f64,
}

pub fn compute(bars: &[Bar], trend_period: usize, wick_pct: f64) -> BeltHoldReport {
    let n = bars.len();
    let mut report = BeltHoldReport {
        bullish: vec![false; n],
        bearish: vec![false; n],
        trend_period,
        wick_pct,
    };
    if trend_period < 2 || !wick_pct.is_finite() || wick_pct <= 0.0
        || n < trend_period + 1 { return report; }
    if bars.iter().any(|b| !b.open.is_finite() || !b.high.is_finite()
        || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    for i in trend_period..n {
        let bar = bars[i];
        let range = bar.high - bar.low;
        if range <= 0.0 { continue; }
        let body = (bar.close - bar.open).abs();
        if body < 0.8 * range { continue; }
        let prior_start_close = bars[i - trend_period].close;
        let prior_end_close = bars[i - 1].close;
        // Bullish: in downtrend, opens at low, bullish body.
        if prior_end_close < prior_start_close
            && bar.close > bar.open
            && (bar.open - bar.low) <= wick_pct * range {
            report.bullish[i] = true;
        }
        // Bearish: in uptrend, opens at high, bearish body.
        if prior_end_close > prior_start_close
            && bar.close < bar.open
            && (bar.high - bar.open) <= wick_pct * range {
            report.bearish[i] = true;
        }
    }
    report
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
        let r = compute(&bars, 1, 0.05);
        assert!(!r.bullish.iter().any(|x| *x));
        let r2 = compute(&bars, 5, 0.0);
        assert!(!r2.bullish.iter().any(|x| *x));
        let r3 = compute(&bars[..3], 5, 0.05);
        assert!(!r3.bullish.iter().any(|x| *x));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![bar(100.0, 101.0, 99.0, 100.0); 10];
        bars[5] = bar(f64::NAN, 101.0, 99.0, 100.0);
        let r = compute(&bars, 5, 0.05);
        assert!(!r.bullish.iter().any(|x| *x));
    }

    #[test]
    fn bullish_belt_hold_at_downtrend_bottom_detected() {
        // 5 down bars then a bullish belt hold (open at low, full body).
        let mut bars: Vec<_> = (0..5).map(|i| {
            let p = 100.0 - i as f64;
            bar(p, p + 0.5, p - 0.5, p - 0.4)
        }).collect();
        // Belt hold: open=95.0=low, high=105, low=95, close=104.5.
        // body = 9.5 of range 10 = 95%; lower wick = 0.
        bars.push(bar(95.0, 105.0, 95.0, 104.5));
        let r = compute(&bars, 5, 0.05);
        assert!(r.bullish[5]);
    }

    #[test]
    fn bearish_belt_hold_at_uptrend_top_detected() {
        let mut bars: Vec<_> = (0..5).map(|i| {
            let p = 100.0 + i as f64;
            bar(p, p + 0.5, p - 0.5, p + 0.4)
        }).collect();
        // Belt hold: open=110=high, close=100.5, low=100.
        bars.push(bar(110.0, 110.0, 100.0, 100.5));
        let r = compute(&bars, 5, 0.05);
        assert!(r.bearish[5]);
    }

    #[test]
    fn belt_hold_without_trend_context_skipped() {
        // Same belt-hold shape but in flat market → no signal.
        let mut bars = vec![bar(100.0, 101.0, 99.0, 100.0); 5];
        bars.push(bar(95.0, 105.0, 95.0, 104.5));
        let r = compute(&bars, 5, 0.05);
        // Flat market → prior_end == prior_start → neither direction met.
        assert!(!r.bullish[5]);
        assert!(!r.bearish[5]);
    }

    #[test]
    fn small_body_rejected() {
        let mut bars: Vec<_> = (0..5).map(|i| {
            let p = 100.0 - i as f64;
            bar(p, p + 0.5, p - 0.5, p - 0.4)
        }).collect();
        // Body 5 of range 10 = 50% < 80% threshold.
        bars.push(bar(95.0, 105.0, 95.0, 100.0));
        let r = compute(&bars, 5, 0.05);
        assert!(!r.bullish[5]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars, 5, 0.05);
        assert_eq!(r.bullish.len(), 10);
        assert_eq!(r.bearish.len(), 10);
    }
}
