//! Aroon Oscillator — Tushar Chande companion to the Aroon Up/Down.
//!
//!   Aroon Up   = (period − bars_since_highest_high) / period × 100
//!   Aroon Down = (period − bars_since_lowest_low)   / period × 100
//!   Aroon Osc  = Aroon Up − Aroon Down
//!
//! Range −100..=+100. +100 = brand-new period-high just printed and no
//! period-low recently. −100 = mirror. Zero-line cross is the textbook
//! trend-direction flip. Standard period = 25.
//!
//! Distinct from `crate::aroon` which emits the two underlying components;
//! this module emits the oscillator only (for chart overlays that just
//! want a single signed line).
//!
//! Pure compute.

use crate::aroon;

pub fn compute(bars: &[aroon::Bar], period: usize) -> Vec<f64> {
    let pts = aroon::compute(bars, period);
    pts.into_iter().map(|p| p.oscillator).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> aroon::Bar {
        aroon::Bar { high: h, low: l }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 25).is_empty());
    }

    #[test]
    fn rising_series_oscillator_positive() {
        let bars: Vec<aroon::Bar> = (1..=40)
            .map(|i| b(100.0 + i as f64, 99.0 + i as f64))
            .collect();
        let out = compute(&bars, 25);
        assert!(
            out[39] > 50.0,
            "uptrend should yield strong + oscillator, got {}",
            out[39]
        );
    }

    #[test]
    fn falling_series_oscillator_negative() {
        let bars: Vec<aroon::Bar> = (1..=40)
            .map(|i| b(200.0 - i as f64, 199.0 - i as f64))
            .collect();
        let out = compute(&bars, 25);
        assert!(out[39] < -50.0);
    }

    #[test]
    fn range_bounded_minus_100_to_plus_100() {
        let bars: Vec<aroon::Bar> = (0..200)
            .map(|i| {
                let mid = 100.0 + (i as f64 * 0.5).sin() * 5.0;
                b(mid + 1.0, mid - 1.0)
            })
            .collect();
        let out = compute(&bars, 25);
        for v in &out {
            assert!((-100.0..=100.0).contains(v), "out of [-100,100]: {v}");
        }
    }

    #[test]
    fn huge_period_safe() {
        let bars = vec![b(101.0, 99.0); 5];
        // aroon's guard returns all-default for n < period+1; default oscillator = 0.0.
        let out = compute(&bars, usize::MAX);
        for v in &out {
            assert_eq!(*v, 0.0);
        }
    }
}
