//! Swing Failure Pattern (SFP) — ICT / Tom Hougaard false-break signal.
//!
//! A bar takes out a prior N-bar swing high/low (intrabar) but CLOSES
//! BACK INSIDE the prior range — signaling the breakout failed and
//! the market is likely to reverse in the opposite direction.
//!
//! Bullish SFP (long signal):
//!   - bar.low < min(low over lookback bars)
//!   - bar.close > min(low over lookback bars)
//!     (took out the prior low intrabar but closed back above it)
//!
//! Bearish SFP (short signal): mirrored.
//!
//! Pure compute. Default lookback = 20.
//! Companion to `turtle_soup`, `pinball_setup`, `hikkake_pattern`,
//! `key_reversal_bar`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SfpReport {
    pub bullish: Vec<bool>,
    pub bearish: Vec<bool>,
    pub lookback: usize,
}

pub fn compute(bars: &[Bar], lookback: usize) -> SfpReport {
    let n = bars.len();
    let mut report = SfpReport {
        bullish: vec![false; n],
        bearish: vec![false; n],
        lookback,
    };
    if lookback < 2 || n < lookback + 1 { return report; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    for i in lookback..n {
        let win = &bars[i - lookback..i];
        let prior_low = win.iter().fold(f64::INFINITY, |a, b| a.min(b.low));
        let prior_high = win.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
        let bar = bars[i];
        if bar.low < prior_low && bar.close > prior_low {
            report.bullish[i] = true;
        }
        if bar.high > prior_high && bar.close < prior_high {
            report.bearish[i] = true;
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 1);
        assert!(!r.bullish.iter().any(|x| *x));
        let r2 = compute(&bars[..5], 20);
        assert!(!r2.bullish.iter().any(|x| *x));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        let r = compute(&bars, 20);
        assert!(!r.bullish.iter().any(|x| *x));
    }

    #[test]
    fn flat_market_no_signal() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 20);
        assert!(!r.bullish.iter().any(|x| *x));
        assert!(!r.bearish.iter().any(|x| *x));
    }

    #[test]
    fn bullish_sfp_detected() {
        // 20 bars with low=99, then a bar with low=95 (breaks prior low)
        // but close=101 (above prior low 99).
        let mut bars = vec![b(101.0, 99.0, 100.0); 20];
        bars.push(b(102.0, 95.0, 101.0));
        let r = compute(&bars, 20);
        assert!(r.bullish[20]);
    }

    #[test]
    fn bearish_sfp_detected() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 20];
        bars.push(b(110.0, 100.0, 99.5));
        let r = compute(&bars, 20);
        assert!(r.bearish[20]);
    }

    #[test]
    fn true_breakout_close_below_prior_low_not_sfp() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 20];
        bars.push(b(101.0, 90.0, 92.0));    // close 92 < prior low 99
        let r = compute(&bars, 20);
        assert!(!r.bullish[20]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 20);
        assert_eq!(r.bullish.len(), 30);
        assert_eq!(r.bearish.len(), 30);
    }
}
