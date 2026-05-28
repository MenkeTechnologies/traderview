//! Weighted Close (WC) — double-weights the close in the typical-price
//! formula:
//!
//!   WC_t = (high + low + 2·close) / 4
//!
//! Emphasizes settlement prices vs the high-low range — useful as an
//! alternate price input for indicators that want to bias toward
//! end-of-bar levels (Demark/Wilder-style systems).
//!
//! Pure compute. Companion to `typical_price`, `median_price`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64 }

pub fn compute(bars: &[Bar]) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()) {
        return out;
    }
    for (i, bar) in bars.iter().enumerate() {
        out[i] = Some((bar.high + bar.low + 2.0 * bar.close) / 4.0);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

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
    fn close_at_midpoint_yields_close() {
        let bars = vec![b(105.0, 95.0, 100.0)];
        assert!((compute(&bars)[0].unwrap() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn close_dominates_weighted_close() {
        // h=100, l=100, c=110 → WC = (100+100+220)/4 = 105.
        let bars = vec![b(100.0, 100.0, 110.0)];
        let v = compute(&bars)[0].unwrap();
        assert!((v - 105.0).abs() < 1e-9);
    }

    #[test]
    fn weighted_close_pulled_toward_close_vs_typical() {
        // Typical price: (110 + 90 + 100) / 3 ≈ 100.
        // Weighted close: (110 + 90 + 200) / 4 = 100. Same when c = midpoint.
        // Use asymmetric: h=120, l=80, c=110.
        //   TP = (120 + 80 + 110) / 3 = 103.33
        //   WC = (120 + 80 + 220) / 4 = 105.0   (closer to close)
        let bars = vec![b(120.0, 80.0, 110.0)];
        let wc = compute(&bars)[0].unwrap();
        let tp: f64 = (120.0 + 80.0 + 110.0) / 3.0;
        // WC is closer to close (110) than TP is.
        assert!((wc - 110.0).abs() < (tp - 110.0).abs());
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        assert_eq!(compute(&bars).len(), 30);
    }
}
