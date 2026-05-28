//! Stick Sandwich — 3-bar reversal pattern with matching outer closes.
//!
//! Bullish Stick Sandwich:
//!   Bar 1: bearish bar
//!   Bar 2: bullish bar whose body is ENTIRELY ABOVE bar 1's close
//!     (gap up between the two closing prices not required, but body of
//!     bar 2 sits above bar 1's body)
//!   Bar 3: bearish bar whose close matches bar 1's close (within
//!     tolerance)
//!
//! Bearish Stick Sandwich: mirrored.
//!
//! Pure compute. Default tolerance_pct = 0.3.
//! Companion to `harami_pattern`, `tweezer_top_bottom`,
//! `morning_evening_star`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub open: f64, pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StickSandwichReport {
    pub bullish: Vec<bool>,
    pub bearish: Vec<bool>,
    pub tolerance_pct: f64,
}

pub fn compute(bars: &[Bar], tolerance_pct: f64) -> StickSandwichReport {
    let n = bars.len();
    let mut report = StickSandwichReport {
        bullish: vec![false; n],
        bearish: vec![false; n],
        tolerance_pct,
    };
    if n < 3 || !tolerance_pct.is_finite() || tolerance_pct <= 0.0 { return report; }
    if bars.iter().any(|b| !b.open.is_finite() || !b.high.is_finite()
        || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    let tol_factor = tolerance_pct / 100.0;
    for i in 2..n {
        let (b1, b2, b3) = (bars[i - 2], bars[i - 1], bars[i]);
        let tol = b1.close.abs() * tol_factor;
        // Bullish: b1 bearish, b2 bullish above b1's body, b3 bearish
        // closing matching b1.close.
        if b1.close < b1.open
            && b2.close > b2.open
            && b2.open >= b1.close
            && b3.close < b3.open
            && (b3.close - b1.close).abs() <= tol {
            report.bullish[i] = true;
        }
        // Bearish: b1 bullish, b2 bearish below b1's body, b3 bullish
        // closing matching b1.close.
        if b1.close > b1.open
            && b2.close < b2.open
            && b2.open <= b1.close
            && b3.close > b3.open
            && (b3.close - b1.close).abs() <= tol {
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
    fn empty_returns_empty() {
        let r = compute(&[], 0.3);
        assert!(r.bullish.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.0),
                        bar(f64::NAN, 101.0, 99.0, 100.0),
                        bar(100.0, 101.0, 99.0, 100.0)];
        let r = compute(&bars, 0.3);
        assert!(!r.bullish.iter().any(|x| *x));
    }

    #[test]
    fn bullish_stick_sandwich_detected() {
        // Bar 1: bearish 110→100.
        // Bar 2: bullish opens 100 (≥ b1.close=100), closes 105.
        // Bar 3: bearish closes 100.1 (within 0.3% of b1.close=100).
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(100.0, 105.5, 99.8, 105.0),
            bar(106.0, 106.5, 99.5, 100.1),
        ];
        let r = compute(&bars, 0.3);
        assert!(r.bullish[2]);
    }

    #[test]
    fn bearish_stick_sandwich_detected() {
        let bars = vec![
            bar(100.0, 110.5, 99.5, 110.0),
            bar(110.0, 110.2, 105.0, 105.5),
            bar(104.0, 110.5, 103.5, 109.9),
        ];
        let r = compute(&bars, 0.3);
        assert!(r.bearish[2]);
    }

    #[test]
    fn b3_close_too_far_rejects() {
        let bars = vec![
            bar(110.0, 110.5, 99.5, 100.0),
            bar(100.0, 105.5, 99.8, 105.0),
            bar(106.0, 106.5, 95.0, 95.0),    // close 95 ≠ b1.close 100
        ];
        let r = compute(&bars, 0.3);
        assert!(!r.bullish[2]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 10];
        let r = compute(&bars, 0.3);
        assert_eq!(r.bullish.len(), 10);
        assert_eq!(r.bearish.len(), 10);
    }
}
