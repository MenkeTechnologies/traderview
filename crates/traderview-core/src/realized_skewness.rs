//! Realized Skewness — third-moment companion to realized variance
//! (Amaya, Christoffersen, Jacobs, Vasquez 2015).
//!
//! For high-frequency intraday returns over a window:
//!
//!   RV  = Σ r_i²
//!   RS  = √n · Σ r_i³ / RV^(3/2)
//!
//! The √n scaling makes RS an annualizable measure of distributional
//! asymmetry, distinct from sample skewness on lower-frequency returns.
//!
//! Daily / weekly realized skewness has been shown to be a robust
//! predictor of next-period equity returns: stocks with most-negative
//! realized skew tend to outperform (negative-skew premium).
//!
//! Pure compute. Companion to `realized_volatility`,
//! `realized_higher_moments`, `realized_quarticity`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RealizedSkewnessReport {
    pub realized_skewness: f64,
    pub realized_variance: f64,
    pub sum_third_moment: f64,
    pub n_returns: usize,
}

pub fn compute(returns: &[f64]) -> Option<RealizedSkewnessReport> {
    let n = returns.len();
    if n < 5 {
        return None;
    }
    if returns.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let n_f = n as f64;
    let rv: f64 = returns.iter().map(|r| r * r).sum();
    if rv <= 0.0 {
        return None;
    }
    let m3: f64 = returns.iter().map(|r| r * r * r).sum();
    let rs = n_f.sqrt() * m3 / rv.powf(1.5);
    Some(RealizedSkewnessReport {
        realized_skewness: rs,
        realized_variance: rv,
        sum_third_moment: m3,
        n_returns: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(compute(&[0.01, -0.01]).is_none());
    }

    #[test]
    fn nan_returns_none() {
        assert!(compute(&[0.01, f64::NAN, -0.01, 0.005, 0.002]).is_none());
    }

    #[test]
    fn flat_returns_none() {
        assert!(compute(&[0.0; 50]).is_none());
    }

    #[test]
    fn symmetric_returns_yield_near_zero_skew() {
        let returns = vec![0.01, -0.01, 0.02, -0.02, 0.005, -0.005, 0.015, -0.015];
        let r = compute(&returns).unwrap();
        assert!(
            r.realized_skewness.abs() < 1e-12,
            "symmetric returns should have skew ~0, got {}",
            r.realized_skewness
        );
    }

    #[test]
    fn left_skewed_yields_negative_skew() {
        // Many small gains + one large loss.
        let mut returns = vec![0.005_f64; 50];
        returns.push(-0.30);
        let r = compute(&returns).unwrap();
        assert!(
            r.realized_skewness < 0.0,
            "left-skewed returns: RS should be negative, got {}",
            r.realized_skewness
        );
    }

    #[test]
    fn right_skewed_yields_positive_skew() {
        let mut returns = vec![-0.005_f64; 50];
        returns.push(0.30);
        let r = compute(&returns).unwrap();
        assert!(
            r.realized_skewness > 0.0,
            "right-skewed returns: RS should be positive, got {}",
            r.realized_skewness
        );
    }

    #[test]
    fn n_returns_reported() {
        let returns: Vec<f64> = (0..100).map(|i| (i as f64 * 0.1).sin() * 0.01).collect();
        let r = compute(&returns).unwrap();
        assert_eq!(r.n_returns, 100);
    }

    #[test]
    fn realized_variance_matches_sum_of_squares() {
        let returns = vec![0.01, -0.02, 0.015, -0.01, 0.005];
        let r = compute(&returns).unwrap();
        let expected_rv: f64 = returns.iter().map(|x| x * x).sum();
        assert!((r.realized_variance - expected_rv).abs() < 1e-15);
    }
}
