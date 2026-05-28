//! Empirical Distribution Function (ECDF) + Dvoretzky-Kiefer-Wolfowitz
//! Confidence Band.
//!
//! For a sample {x_1, …, x_n}, the empirical CDF is:
//!
//!   F̂_n(x) = (1/n) · |{i : x_i ≤ x}|
//!
//! The DKW inequality (Massart 1990) gives a finite-sample, uniform
//! confidence band on F̂_n:
//!
//!   P(sup_x |F̂_n(x) − F(x)| > ε) ≤ 2 · exp(−2n·ε²)
//!
//! For confidence 1 − α, half-width:
//!
//!   ε_α = √(ln(2/α) / (2n))
//!
//! Reports the unique sorted x-values + cumulative probabilities +
//! upper/lower bands clipped to [0, 1].
//!
//! Use cases:
//!   - Distribution-free band on return CDF for visualization
//!   - Pre-test for quantile-based VaR estimates
//!   - Sample CDF input to other tests (KS, AD, etc.)
//!
//! Pure compute. Companion to `kolmogorov_smirnov_2sample`,
//! `anderson_darling_normality`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcdfPoint {
    pub x: f64,
    pub probability: f64,
    pub lower_band: f64,
    pub upper_band: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EcdfReport {
    pub points: Vec<EcdfPoint>,
    pub dkw_half_width: f64,
    pub confidence: f64,
    pub n_observations: usize,
}

pub fn compute(sample: &[f64], confidence: f64) -> Option<EcdfReport> {
    let n = sample.len();
    if n < 5 || !confidence.is_finite() || !(0.5..1.0).contains(&confidence) {
        return None;
    }
    if sample.iter().any(|x| !x.is_finite()) { return None; }
    let mut sorted = sample.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let alpha = 1.0 - confidence;
    let epsilon = ((2.0_f64 / alpha).ln() / (2.0 * n as f64)).sqrt();
    let n_f = n as f64;
    let mut points = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j + 1 < n && sorted[j + 1] == sorted[i] { j += 1; }
        let prob = (j + 1) as f64 / n_f;
        points.push(EcdfPoint {
            x: sorted[i],
            probability: prob,
            lower_band: (prob - epsilon).max(0.0),
            upper_band: (prob + epsilon).min(1.0),
        });
        i = j + 1;
    }
    Some(EcdfReport {
        points,
        dkw_half_width: epsilon,
        confidence,
        n_observations: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(compute(&[1.0, 2.0], 0.95).is_none());
    }

    #[test]
    fn invalid_confidence_returns_none() {
        assert!(compute(&[1.0; 10], 0.4).is_none());
        assert!(compute(&[1.0; 10], 1.0).is_none());
        assert!(compute(&[1.0; 10], f64::NAN).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let s = vec![1.0, f64::NAN, 3.0, 4.0, 5.0];
        assert!(compute(&s, 0.95).is_none());
    }

    #[test]
    fn probabilities_monotone_in_unit_range() {
        let s: Vec<f64> = (1..=20).map(|i| i as f64).collect();
        let r = compute(&s, 0.95).unwrap();
        for w in r.points.windows(2) {
            assert!(w[1].probability >= w[0].probability);
        }
        for p in &r.points {
            assert!((0.0..=1.0).contains(&p.probability));
        }
    }

    #[test]
    fn confidence_band_brackets_probability() {
        let s: Vec<f64> = (1..=50).map(|i| i as f64).collect();
        let r = compute(&s, 0.95).unwrap();
        for p in &r.points {
            assert!(p.lower_band <= p.probability);
            assert!(p.upper_band >= p.probability);
            assert!((0.0..=1.0).contains(&p.lower_band));
            assert!((0.0..=1.0).contains(&p.upper_band));
        }
    }

    #[test]
    fn dkw_width_shrinks_with_n() {
        let small: Vec<f64> = (1..=50).map(|i| i as f64).collect();
        let large: Vec<f64> = (1..=5000).map(|i| i as f64).collect();
        let r_small = compute(&small, 0.95).unwrap();
        let r_large = compute(&large, 0.95).unwrap();
        assert!(r_large.dkw_half_width < r_small.dkw_half_width);
    }

    #[test]
    fn duplicate_values_collapsed() {
        let s = vec![1.0, 1.0, 1.0, 2.0, 2.0, 3.0];
        let r = compute(&s, 0.95).unwrap();
        assert_eq!(r.points.len(), 3);
        // Probability at x=1 = 3/6 = 0.5.
        assert!((r.points[0].probability - 0.5).abs() < 1e-12);
        // Probability at x=2 = 5/6.
        assert!((r.points[1].probability - 5.0 / 6.0).abs() < 1e-12);
        // Probability at x=3 = 1.0.
        assert!((r.points[2].probability - 1.0).abs() < 1e-12);
    }

    #[test]
    fn n_observations_reported() {
        let s: Vec<f64> = (0..30).map(|i| i as f64).collect();
        let r = compute(&s, 0.95).unwrap();
        assert_eq!(r.n_observations, 30);
    }
}
