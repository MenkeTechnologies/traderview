//! Typical Price (TP) — Welles Wilder.
//!
//!   TP_t = (high + low + close) / 3
//!
//! The "true" representative price for a bar that takes intrabar
//! action into account. Used as the building block for CCI, money
//! flow index, accumulation/distribution oscillators, etc.
//!
//! Pure compute. Companion to `weighted_close`, `median_price`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

pub fn compute(bars: &[Bar]) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if bars
        .iter()
        .any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite())
    {
        return out;
    }
    for (i, bar) in bars.iter().enumerate() {
        out[i] = Some((bar.high + bar.low + bar.close) / 3.0);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar {
        Bar {
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[]).is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![b(101.0, 99.0, 100.0), b(f64::NAN, 99.0, 100.0)];
        assert!(compute(&bars).iter().all(|x| x.is_none()));
    }

    #[test]
    fn balanced_bar_yields_close() {
        // h+l+c = 3c when h+l = 2c, i.e. close = midpoint of range.
        let bars = vec![b(105.0, 95.0, 100.0)];
        assert!((compute(&bars)[0].unwrap() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn typical_price_above_close_when_high_dominates() {
        // h=120, l=100, c=100 → TP = 320/3 ≈ 106.67 > 100.
        let bars = vec![b(120.0, 100.0, 100.0)];
        let v = compute(&bars)[0].unwrap();
        assert!(v > 100.0);
        assert!((v - (120.0 + 100.0 + 100.0) / 3.0).abs() < 1e-9);
    }

    #[test]
    fn typical_price_below_close_when_low_dominates() {
        let bars = vec![b(100.0, 80.0, 100.0)];
        let v = compute(&bars)[0].unwrap();
        assert!(v < 100.0);
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        assert_eq!(compute(&bars).len(), 30);
    }
}
