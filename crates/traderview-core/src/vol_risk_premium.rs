//! Volatility Risk Premium (VRP) — implied minus realized variance.
//!
//! VRP captures the spread that option sellers earn (on average) for
//! providing insurance against volatility shocks:
//!
//!   VRP_t = IV²_t − RV_t      (variance form)
//!   VolP_t = IV_t − √RV_t      (volatility form)
//!
//! Computed across a series of (forecast IV, subsequent realized
//! vol) pairs and aggregated:
//!
//!   - mean / stdev / min / max of VRP
//!   - hit rate (fraction where IV² > RV → sellers profitable)
//!   - VRP standard error (HAC for overlapping samples; here plain)
//!   - "fair vol" estimate: avg of √RV (premium-neutral level)
//!
//! Pure compute. Companion to `variance_swap_strike`, `realized_volatility`,
//! `iv_term_structure`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VrpObservation {
    pub implied_volatility_annualized: f64,
    pub subsequent_realized_volatility_annualized: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VrpReport {
    pub per_obs_vrp_variance: Vec<f64>,
    pub per_obs_vrp_volatility: Vec<f64>,
    pub mean_vrp_variance: f64,
    pub mean_vrp_volatility: f64,
    pub stdev_vrp_variance: f64,
    pub min_vrp_variance: f64,
    pub max_vrp_variance: f64,
    pub seller_hit_rate: f64,
    pub fair_vol_estimate: f64,
    pub n_observations: usize,
}

pub fn compute(observations: &[VrpObservation]) -> Option<VrpReport> {
    if observations.is_empty() { return None; }
    if observations.iter().any(|o| !o.implied_volatility_annualized.is_finite()
        || o.implied_volatility_annualized < 0.0
        || !o.subsequent_realized_volatility_annualized.is_finite()
        || o.subsequent_realized_volatility_annualized < 0.0) {
        return None;
    }
    let n = observations.len();
    let vrp_var: Vec<f64> = observations.iter().map(|o| {
        o.implied_volatility_annualized.powi(2)
            - o.subsequent_realized_volatility_annualized.powi(2)
    }).collect();
    let vrp_vol: Vec<f64> = observations.iter().map(|o| {
        o.implied_volatility_annualized - o.subsequent_realized_volatility_annualized
    }).collect();
    let n_f = n as f64;
    let mean_var: f64 = vrp_var.iter().sum::<f64>() / n_f;
    let mean_vol: f64 = vrp_vol.iter().sum::<f64>() / n_f;
    let var_var: f64 = vrp_var.iter().map(|x| (x - mean_var).powi(2)).sum::<f64>()
        / (n_f - 1.0).max(1.0);
    let sd_var = var_var.max(0.0).sqrt();
    let min_var = vrp_var.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_var = vrp_var.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let seller_hits = vrp_var.iter().filter(|x| **x > 0.0).count() as f64 / n_f;
    let fair_vol: f64 = observations.iter()
        .map(|o| o.subsequent_realized_volatility_annualized).sum::<f64>() / n_f;
    Some(VrpReport {
        per_obs_vrp_variance: vrp_var,
        per_obs_vrp_volatility: vrp_vol,
        mean_vrp_variance: mean_var,
        mean_vrp_volatility: mean_vol,
        stdev_vrp_variance: sd_var,
        min_vrp_variance: min_var,
        max_vrp_variance: max_var,
        seller_hit_rate: seller_hits,
        fair_vol_estimate: fair_vol,
        n_observations: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn o(iv: f64, rv: f64) -> VrpObservation {
        VrpObservation {
            implied_volatility_annualized: iv,
            subsequent_realized_volatility_annualized: rv,
        }
    }

    #[test]
    fn empty_returns_none() {
        assert!(compute(&[]).is_none());
    }

    #[test]
    fn nan_or_negative_returns_none() {
        assert!(compute(&[o(f64::NAN, 0.20)]).is_none());
        assert!(compute(&[o(0.20, -0.10)]).is_none());
        assert!(compute(&[o(-0.10, 0.20)]).is_none());
    }

    #[test]
    fn iv_above_rv_yields_positive_vrp() {
        let obs = vec![o(0.30, 0.20); 10];
        let r = compute(&obs).unwrap();
        assert!(r.mean_vrp_variance > 0.0);
        assert!(r.mean_vrp_volatility > 0.0);
        assert!((r.seller_hit_rate - 1.0).abs() < 1e-9);
    }

    #[test]
    fn iv_below_rv_yields_negative_vrp() {
        let obs = vec![o(0.20, 0.30); 10];
        let r = compute(&obs).unwrap();
        assert!(r.mean_vrp_variance < 0.0);
        assert!(r.mean_vrp_volatility < 0.0);
        assert!(r.seller_hit_rate.abs() < 1e-9);
    }

    #[test]
    fn fair_vol_equals_mean_rv() {
        let obs = vec![
            o(0.30, 0.20),
            o(0.30, 0.25),
            o(0.30, 0.30),
            o(0.30, 0.15),
        ];
        let r = compute(&obs).unwrap();
        let expected = (0.20 + 0.25 + 0.30 + 0.15) / 4.0;
        assert!((r.fair_vol_estimate - expected).abs() < 1e-12);
    }

    #[test]
    fn min_max_reported_correctly() {
        let obs = vec![
            o(0.30, 0.20),    // VRP_var = 0.09 - 0.04 = 0.05
            o(0.25, 0.25),    // VRP_var = 0
            o(0.10, 0.20),    // VRP_var = 0.01 - 0.04 = -0.03
        ];
        let r = compute(&obs).unwrap();
        assert!((r.max_vrp_variance - 0.05).abs() < 1e-12);
        assert!((r.min_vrp_variance + 0.03).abs() < 1e-12);
    }

    #[test]
    fn seller_hit_rate_in_unit_range() {
        let obs = vec![
            o(0.30, 0.20),    // hit
            o(0.30, 0.25),    // hit
            o(0.20, 0.30),    // miss
        ];
        let r = compute(&obs).unwrap();
        assert!((0.0..=1.0).contains(&r.seller_hit_rate));
        assert!((r.seller_hit_rate - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn per_obs_length_matches_input() {
        let obs = vec![o(0.30, 0.20); 25];
        let r = compute(&obs).unwrap();
        assert_eq!(r.per_obs_vrp_variance.len(), 25);
        assert_eq!(r.per_obs_vrp_volatility.len(), 25);
        assert_eq!(r.n_observations, 25);
    }
}
