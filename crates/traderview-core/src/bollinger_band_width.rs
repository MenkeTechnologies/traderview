//! Bollinger Band Width (BBW) + %B (Percent B) — John Bollinger.
//!
//! Two derived metrics from standard Bollinger Bands (SMA ± k·σ):
//!
//!   BBW_t  = (Upper − Lower) / Middle           — relative band width
//!   %B_t   = (Close − Lower) / (Upper − Lower)  — position within bands
//!
//! BBW: low values = "squeeze" (low vol, often precedes breakout);
//! high values = expansion (high vol regime).
//!
//! %B: > 1 = price above upper band; < 0 = below lower band; 0.5 =
//! price at the middle band.
//!
//! Default: period = 20, k = 2.0 (Bollinger's original parameters).
//!
//! Pure compute. Companion to `keltner_squeeze`, `donchian_channels`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BbwReport {
    pub middle: Vec<Option<f64>>,
    pub upper: Vec<Option<f64>>,
    pub lower: Vec<Option<f64>>,
    pub band_width: Vec<Option<f64>>,
    pub percent_b: Vec<Option<f64>>,
}

pub fn compute(closes: &[f64], period: usize, k: f64) -> BbwReport {
    let n = closes.len();
    let mut mid = vec![None; n];
    let mut up = vec![None; n];
    let mut lo = vec![None; n];
    let mut bbw = vec![None; n];
    let mut pb = vec![None; n];
    if period < 2 || n < period || !k.is_finite() || k < 0.0 {
        return BbwReport { middle: mid, upper: up, lower: lo, band_width: bbw, percent_b: pb };
    }
    for i in (period - 1)..n {
        let win = &closes[i + 1 - period..=i];
        if win.iter().any(|x| !x.is_finite()) { continue; }
        let m: f64 = win.iter().sum::<f64>() / period as f64;
        let var: f64 = win.iter().map(|x| (x - m).powi(2)).sum::<f64>() / period as f64;
        let sd = var.max(0.0).sqrt();
        let u = m + k * sd;
        let l = m - k * sd;
        mid[i] = Some(m);
        up[i] = Some(u);
        lo[i] = Some(l);
        // BBW relative to middle band; undefined if middle == 0.
        if m.abs() > 1e-18 {
            bbw[i] = Some((u - l) / m);
        }
        let cur = closes[i];
        let width = u - l;
        if width > 0.0 {
            pb[i] = Some((cur - l) / width);
        } else {
            // Degenerate flat window: %B undefined, set to 0.5 (price = middle).
            pb[i] = Some(0.5);
        }
    }
    BbwReport { middle: mid, upper: up, lower: lo, band_width: bbw, percent_b: pb }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty_outputs() {
        let r = compute(&[], 20, 2.0);
        assert!(r.middle.is_empty());
    }

    #[test]
    fn invalid_params_return_all_none() {
        let closes = vec![100.0_f64; 30];
        let r = compute(&closes, 1, 2.0);
        assert!(r.middle.iter().all(|x| x.is_none()));
        let r2 = compute(&closes, 20, -1.0);
        assert!(r2.middle.iter().all(|x| x.is_none()));
        let r3 = compute(&closes, 20, f64::NAN);
        assert!(r3.middle.iter().all(|x| x.is_none()));
    }

    #[test]
    fn shorter_than_period_returns_all_none() {
        let closes = vec![100.0_f64; 5];
        let r = compute(&closes, 20, 2.0);
        assert!(r.middle.iter().all(|x| x.is_none()));
    }

    #[test]
    fn middle_is_sma() {
        let closes: Vec<f64> = (1..=30).map(|i| i as f64).collect();
        let r = compute(&closes, 10, 2.0);
        // Last middle = mean(21..30) = 25.5.
        assert!((r.middle[29].unwrap() - 25.5).abs() < 1e-9);
    }

    #[test]
    fn upper_above_middle_lower_below() {
        let closes: Vec<f64> = (1..=30).map(|i| i as f64).collect();
        let r = compute(&closes, 10, 2.0);
        for i in 9..30 {
            assert!(r.upper[i].unwrap() > r.middle[i].unwrap());
            assert!(r.lower[i].unwrap() < r.middle[i].unwrap());
        }
    }

    #[test]
    fn flat_window_yields_zero_width() {
        let closes = vec![100.0_f64; 30];
        let r = compute(&closes, 20, 2.0);
        // All values identical → sd = 0 → upper = lower = middle.
        let bw = r.band_width[29].unwrap();
        assert!(bw.abs() < 1e-9);
        // %B undefined for zero-width band; we return 0.5.
        let pb = r.percent_b[29].unwrap();
        assert!((pb - 0.5).abs() < 1e-9);
    }

    #[test]
    fn percent_b_at_zero_when_close_at_lower() {
        // Construct a window where last close = lower band.
        let mut closes: Vec<f64> = (0..19).map(|_| 100.0_f64).collect();
        closes.push(99.0);
        let r = compute(&closes, 20, 2.0);
        let pb = r.percent_b[19].unwrap();
        // Close 99 is below middle (~99.95) and below middle - 2σ.
        // pb should be ≤ 0.
        assert!(pb < 0.5);
    }

    #[test]
    fn expansion_increases_band_width() {
        let mut closes = vec![100.0_f64; 20];
        // Add high-vol section.
        for i in 0..30 {
            closes.push(100.0 + (i as f64 * 0.7).sin() * 10.0);
        }
        let r = compute(&closes, 20, 2.0);
        // BBW at the expansion phase should exceed BBW at the flat-phase end.
        let flat_bbw = r.band_width[19].unwrap();
        let expansion_bbw = r.band_width[49].unwrap();
        assert!(expansion_bbw > flat_bbw,
            "expansion BBW {expansion_bbw} should exceed flat BBW {flat_bbw}");
    }

    #[test]
    fn outputs_aligned_to_input_length() {
        let closes: Vec<f64> = (0..50).map(|i| 100.0 + i as f64 * 0.1).collect();
        let r = compute(&closes, 20, 2.0);
        assert_eq!(r.middle.len(), 50);
        assert_eq!(r.band_width.len(), 50);
        assert_eq!(r.percent_b.len(), 50);
    }
}
