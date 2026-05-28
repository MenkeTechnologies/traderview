//! Hikkake Pattern — Daniel Chesler / Linda Raschke false-breakout
//! variant of the inside bar setup.
//!
//! Setup:
//!   Bar 0: "mother" bar
//!   Bar 1: inside bar (low > bar0.low AND high < bar0.high)
//!   Bar 2: false breakout — closes BEYOND the inside bar's extreme
//!     in the OPPOSITE direction of the eventual signal
//!
//! Resolution (within confirm_bars after bar 2):
//!   - For LONG hikkake: bar 2 closes below inside-bar low (downside
//!     false break), but a subsequent close > inside-bar HIGH triggers
//!     the bullish setup.
//!   - For SHORT hikkake: bar 2 closes above inside-bar high (upside
//!     false break), but a subsequent close < inside-bar LOW triggers
//!     bearish.
//!
//! Pure compute. Default confirm_bars = 3.
//! Companion to `inside_bar_breakout`, `turtle_soup`, `pinball_setup`,
//! `harami_pattern`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HikkakeReport {
    pub long_signal: Vec<bool>,
    pub short_signal: Vec<bool>,
    pub confirm_bars: usize,
}

pub fn compute(bars: &[Bar], confirm_bars: usize) -> HikkakeReport {
    let n = bars.len();
    let mut report = HikkakeReport {
        long_signal: vec![false; n],
        short_signal: vec![false; n],
        confirm_bars,
    };
    if confirm_bars < 1 || n < confirm_bars + 3 { return report; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    for i in 2..(n - confirm_bars) {
        let mother = bars[i - 2];
        let inside = bars[i - 1];
        let trigger = bars[i];
        // Inside-bar requirement.
        if !(inside.high < mother.high && inside.low > mother.low) { continue; }
        // Downside false break → look for upside reclaim.
        if trigger.close < inside.low {
            for k in 1..=confirm_bars {
                if bars[i + k].close > inside.high {
                    report.long_signal[i + k] = true;
                    break;
                }
            }
        }
        // Upside false break → look for downside reclaim.
        if trigger.close > inside.high {
            for k in 1..=confirm_bars {
                if bars[i + k].close < inside.low {
                    report.short_signal[i + k] = true;
                    break;
                }
            }
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
        let r = compute(&bars, 0);
        assert!(!r.long_signal.iter().any(|x| *x));
        let r2 = compute(&bars[..5], 3);
        assert!(!r2.long_signal.iter().any(|x| *x));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        let r = compute(&bars, 3);
        assert!(!r.long_signal.iter().any(|x| *x));
    }

    #[test]
    fn flat_market_no_signal() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 3);
        assert!(!r.long_signal.iter().any(|x| *x));
        assert!(!r.short_signal.iter().any(|x| *x));
    }

    #[test]
    fn classic_long_hikkake_detected() {
        // Bar 0 mother: high=110, low=100, close=105.
        // Bar 1 inside: high=108, low=102, close=105 (inside mother).
        // Bar 2 trigger: close 101 (below inside.low=102 → downside false break).
        // Bar 3 confirm: close 109 (above inside.high=108 → triggers long).
        let bars = vec![
            b(110.0, 100.0, 105.0),
            b(108.0, 102.0, 105.0),
            b(108.0, 100.5, 101.0),
            b(110.0, 105.0, 109.0),
            b(110.0, 105.0, 110.0),
            b(110.0, 105.0, 110.0),
        ];
        let r = compute(&bars, 3);
        assert!(r.long_signal[3]);
    }

    #[test]
    fn classic_short_hikkake_detected() {
        let bars = vec![
            b(110.0, 100.0, 105.0),
            b(108.0, 102.0, 105.0),
            b(110.5, 104.0, 110.0),    // upside false break (close > inside.high)
            b(105.0, 100.0, 101.0),     // downside reclaim (close < inside.low)
            b(101.0, 95.0, 96.0),
            b(96.0, 90.0, 91.0),
        ];
        let r = compute(&bars, 3);
        assert!(r.short_signal[3]);
    }

    #[test]
    fn no_inside_bar_no_signal() {
        // Bar 1 wider than bar 0 → no inside bar.
        let bars = vec![
            b(108.0, 102.0, 105.0),
            b(112.0, 100.0, 106.0),
            b(108.0, 100.5, 101.0),
            b(110.0, 105.0, 109.0),
            b(110.0, 105.0, 110.0),
            b(110.0, 105.0, 110.0),
        ];
        let r = compute(&bars, 3);
        assert!(!r.long_signal.iter().any(|x| *x));
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 3);
        assert_eq!(r.long_signal.len(), 30);
        assert_eq!(r.short_signal.len(), 30);
    }
}
