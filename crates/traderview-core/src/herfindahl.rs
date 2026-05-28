//! Herfindahl-Hirschman Index (HHI) — portfolio concentration measure.
//!
//!   HHI = Σ w_i²        (raw, summed over positive weights, in [0, 1])
//!   HHI_scaled  = HHI · 10_000                       (regulator scale 0..10_000)
//!   Effective N = 1 / HHI                            ("equivalent equal positions")
//!
//! Used regulatorily (DOJ antitrust threshold = 1500), in portfolio
//! analysis (rule-of-thumb: HHI > 0.20 = concentrated), and as a risk
//! constraint in mean-variance optimization (penalize HHI to spread
//! exposure).
//!
//! Weights must be NON-NEGATIVE; long-short portfolios should pass
//! absolute weights normalized to gross exposure.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct HhiReport {
    pub hhi: f64,
    pub hhi_scaled: f64,
    pub effective_n: f64,
    pub n_positions: usize,
    pub max_weight: f64,
}

pub fn compute(weights: &[f64]) -> Option<HhiReport> {
    if weights.is_empty() { return None; }
    let mut sum_w = 0.0_f64;
    let mut sum_w2 = 0.0_f64;
    let mut max_w = 0.0_f64;
    let mut n = 0_usize;
    for w in weights {
        if !w.is_finite() { return None; }
        if *w < 0.0 { return None; }
        if *w > 0.0 {
            sum_w += w;
            sum_w2 += w * w;
            if *w > max_w { max_w = *w; }
            n += 1;
        }
    }
    if sum_w <= 0.0 { return None; }
    // Normalize if not already summing to 1.
    let normalized = if (sum_w - 1.0).abs() > 1e-9 {
        sum_w2 / (sum_w * sum_w)
    } else {
        sum_w2
    };
    if !normalized.is_finite() || normalized <= 0.0 {
        return None;
    }
    Some(HhiReport {
        hhi: normalized,
        hhi_scaled: normalized * 10_000.0,
        effective_n: 1.0 / normalized,
        n_positions: n,
        max_weight: max_w / sum_w,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        assert!(compute(&[]).is_none());
    }

    #[test]
    fn nan_or_negative_returns_none() {
        assert!(compute(&[0.5, f64::NAN]).is_none());
        assert!(compute(&[0.5, -0.5]).is_none());
    }

    #[test]
    fn all_zero_returns_none() {
        assert!(compute(&[0.0, 0.0, 0.0]).is_none());
    }

    #[test]
    fn single_full_weight_yields_hhi_one() {
        let r = compute(&[1.0]).unwrap();
        assert!((r.hhi - 1.0).abs() < 1e-12);
        assert_eq!(r.effective_n, 1.0);
        assert_eq!(r.n_positions, 1);
    }

    #[test]
    fn equal_weights_yield_inverse_n() {
        // 4 equal positions → HHI = 4·(0.25)² = 0.25 → effective_n = 4.
        let r = compute(&[0.25; 4]).unwrap();
        assert!((r.hhi - 0.25).abs() < 1e-12);
        assert!((r.effective_n - 4.0).abs() < 1e-12);
    }

    #[test]
    fn unnormalized_weights_are_normalized_internally() {
        // 4 equal positions of weight 5 (sum = 20) → still HHI = 0.25.
        let r = compute(&[5.0; 4]).unwrap();
        assert!((r.hhi - 0.25).abs() < 1e-12);
    }

    #[test]
    fn hhi_scaled_matches_regulatory_convention() {
        // 4 equal positions → DOJ scale = 2500.
        let r = compute(&[0.25; 4]).unwrap();
        assert!((r.hhi_scaled - 2_500.0).abs() < 1e-9);
    }

    #[test]
    fn concentrated_portfolio_yields_high_hhi() {
        // One 80% position + 4×5%: HHI = 0.64 + 4·0.0025 = 0.65.
        let r = compute(&[0.80, 0.05, 0.05, 0.05, 0.05]).unwrap();
        assert!((r.hhi - 0.65).abs() < 1e-9);
        assert!(r.effective_n < 2.0);    // very concentrated
    }

    #[test]
    fn max_weight_normalized_to_total() {
        let r = compute(&[10.0, 20.0, 70.0]).unwrap();
        assert!((r.max_weight - 0.70).abs() < 1e-9);
    }

    #[test]
    fn zero_weights_excluded_from_n_positions() {
        let r = compute(&[0.5, 0.5, 0.0, 0.0]).unwrap();
        assert_eq!(r.n_positions, 2);
    }
}
