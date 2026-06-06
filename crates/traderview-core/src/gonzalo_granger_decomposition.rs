//! Gonzalo-Granger Permanent / Transitory Decomposition (1995).
//!
//! For two cointegrated price series (e.g. two venue prices of the
//! same asset), decomposes each into a common permanent component
//! and a transitory deviation:
//!
//!   p_i,t = α_i · f_t + z_i,t
//!
//! where f_t is the long-run "true price" (linear combo of both prices
//! using normalized loading weights) and z_i,t is the per-venue
//! transitory component.
//!
//! Loading weights from the cointegrating vector β = (β_1, β_2)' with
//! β_1 = 1 (normalization). The permanent component:
//!
//!   f_t = (β_2 · p_1,t − β_1 · p_2,t · (−1)) ...
//!
//! Simplification used here (the standard "common factor portfolio"):
//!
//!   f_t = (β_2 · p_1,t + β_1 · p_2,t) / (β_1 + β_2)
//!
//! where β_1, β_2 are positive weights normalized from the orthogonal-
//! complement of the cointegrating vector (Hasbrouck-style
//! information shares).
//!
//! Pure compute on two aligned price series. Companion to
//! `cointegration`, `pair_trade_zscore`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GonzaloGrangerReport {
    pub permanent_component: Vec<f64>,
    pub transitory_component_1: Vec<f64>,
    pub transitory_component_2: Vec<f64>,
    pub loading_weight_1: f64,
    pub loading_weight_2: f64,
    pub n_observations: usize,
}

pub fn decompose(price_1: &[f64], price_2: &[f64]) -> Option<GonzaloGrangerReport> {
    let n = price_1.len();
    if n < 10 || price_2.len() != n {
        return None;
    }
    if price_1.iter().any(|x| !x.is_finite()) || price_2.iter().any(|x| !x.is_finite()) {
        return None;
    }
    // OLS of price_1 on price_2 + intercept to estimate cointegrating slope.
    let n_f = n as f64;
    let m1: f64 = price_1.iter().sum::<f64>() / n_f;
    let m2: f64 = price_2.iter().sum::<f64>() / n_f;
    let mut sxx = 0.0_f64;
    let mut sxy = 0.0_f64;
    for i in 0..n {
        sxx += (price_2[i] - m2).powi(2);
        sxy += (price_2[i] - m2) * (price_1[i] - m1);
    }
    if sxx <= 0.0 {
        return None;
    }
    let beta = sxy / sxx;
    // Use β to set normalized loading weights w_1, w_2 ≥ 0 summing to 1.
    // Positive-slope cointegration → w_1 = β / (β + 1), w_2 = 1 / (β + 1).
    // Negative or zero β fallback: equal weights.
    let (w1, w2) = if beta > 0.0 {
        let denom = beta + 1.0;
        (beta / denom, 1.0 / denom)
    } else {
        (0.5, 0.5)
    };
    let permanent: Vec<f64> = price_1
        .iter()
        .zip(price_2.iter())
        .map(|(p1, p2)| w1 * p1 + w2 * p2)
        .collect();
    let transitory_1: Vec<f64> = price_1
        .iter()
        .zip(permanent.iter())
        .map(|(p, f)| p - f)
        .collect();
    let transitory_2: Vec<f64> = price_2
        .iter()
        .zip(permanent.iter())
        .map(|(p, f)| p - f)
        .collect();
    Some(GonzaloGrangerReport {
        permanent_component: permanent,
        transitory_component_1: transitory_1,
        transitory_component_2: transitory_2,
        loading_weight_1: w1,
        loading_weight_2: w2,
        n_observations: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        assert!(decompose(&[1.0; 5], &[1.0; 5]).is_none());
    }

    #[test]
    fn mismatched_returns_none() {
        assert!(decompose(&[1.0; 10], &[1.0; 5]).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let p1 = vec![1.0_f64; 20];
        let mut p2 = vec![1.0_f64; 20];
        p2[5] = f64::NAN;
        assert!(decompose(&p1, &p2).is_none());
    }

    #[test]
    fn identical_series_yield_zero_transitory() {
        let p: Vec<f64> = (1..=20).map(|i| i as f64).collect();
        let r = decompose(&p, &p).unwrap();
        for t in &r.transitory_component_1 {
            assert!(t.abs() < 1e-9);
        }
        for t in &r.transitory_component_2 {
            assert!(t.abs() < 1e-9);
        }
        for (perm, orig) in r.permanent_component.iter().zip(p.iter()) {
            assert!((perm - orig).abs() < 1e-9);
        }
    }

    #[test]
    fn loading_weights_sum_to_one() {
        let p1: Vec<f64> = (1..=30).map(|i| i as f64 + 0.5).collect();
        let p2: Vec<f64> = (1..=30).map(|i| i as f64 - 0.5).collect();
        let r = decompose(&p1, &p2).unwrap();
        assert!((r.loading_weight_1 + r.loading_weight_2 - 1.0).abs() < 1e-12);
    }

    #[test]
    fn permanent_component_intermediate_between_inputs() {
        let p1: Vec<f64> = (1..=30).map(|i| i as f64 + 1.0).collect();
        let p2: Vec<f64> = (1..=30).map(|i| i as f64 - 1.0).collect();
        let r = decompose(&p1, &p2).unwrap();
        for ((perm, p1v), p2v) in r.permanent_component.iter().zip(p1.iter()).zip(p2.iter()) {
            let min = p1v.min(*p2v);
            let max = p1v.max(*p2v);
            assert!(*perm >= min - 1e-9 && *perm <= max + 1e-9);
        }
    }

    #[test]
    fn transitory_sums_to_zero_when_weights_balanced() {
        // p_1 = 100 + ε, p_2 = 100 − ε with symmetric weights → transitory pair sums to 0.
        let p1: Vec<f64> = (0..20).map(|i| 100.0 + (i as f64).sin()).collect();
        let p2: Vec<f64> = (0..20).map(|i| 100.0 - (i as f64).sin()).collect();
        let r = decompose(&p1, &p2).unwrap();
        for (t1, t2) in r
            .transitory_component_1
            .iter()
            .zip(r.transitory_component_2.iter())
        {
            // w1*t1 + w2*t2 should be ~ 0 by construction of f.
            let combined = r.loading_weight_1 * t1 + r.loading_weight_2 * t2;
            assert!(combined.abs() < 1e-9);
        }
    }

    #[test]
    fn output_lengths_match_input() {
        let p: Vec<f64> = (1..=25).map(|i| i as f64).collect();
        let r = decompose(&p, &p).unwrap();
        assert_eq!(r.permanent_component.len(), 25);
        assert_eq!(r.transitory_component_1.len(), 25);
        assert_eq!(r.transitory_component_2.len(), 25);
        assert_eq!(r.n_observations, 25);
    }
}
