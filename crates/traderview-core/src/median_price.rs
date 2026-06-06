//! Median Price (MP) — midpoint of the bar range.
//!
//!   MP_t = (high + low) / 2
//!
//! The simplest "representative price" that ignores the close. Used
//! as the price input for Awesome Oscillator, Bill Williams' Alligator,
//! and many smoothing-only studies.
//!
//! Pure compute. Companion to `typical_price`, `weighted_close`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
}

pub fn compute(bars: &[Bar]) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if bars
        .iter()
        .any(|b| !b.high.is_finite() || !b.low.is_finite())
    {
        return out;
    }
    for (i, bar) in bars.iter().enumerate() {
        out[i] = Some((bar.high + bar.low) / 2.0);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> Bar {
        Bar { high: h, low: l }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[]).is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![b(101.0, 99.0), b(f64::NAN, 99.0)];
        assert!(compute(&bars).iter().all(|x| x.is_none()));
    }

    #[test]
    fn midpoint_of_range() {
        let bars = vec![b(110.0, 100.0)];
        assert!((compute(&bars)[0].unwrap() - 105.0).abs() < 1e-9);
    }

    #[test]
    fn zero_range_returns_value() {
        let bars = vec![b(100.0, 100.0)];
        assert!((compute(&bars)[0].unwrap() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(101.0, 99.0); 30];
        assert_eq!(compute(&bars).len(), 30);
    }
}
