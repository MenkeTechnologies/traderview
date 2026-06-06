//! Probability of Informed Trading (PIN) — Easley, Kiefer, O'Hara,
//! Paperman (1996), method-of-moments approximation.
//!
//! The EKOP model decomposes daily order flow into:
//!   - With probability (1 − α), no information event:
//!     buy arrivals ~ Poisson(ε), sell arrivals ~ Poisson(ε)
//!   - With probability α · δ, bad-news event:
//!     buys ~ Poisson(ε), sells ~ Poisson(ε + μ)
//!   - With probability α · (1 − δ), good-news event:
//!     buys ~ Poisson(ε + μ), sells ~ Poisson(ε)
//!
//! Probability of Informed Trading:
//!
//!   PIN = (α · μ) / (α · μ + 2ε)
//!
//! Full MLE is non-trivial (Poisson mixture). This module ships a
//! method-of-moments approximation that is robust and fast:
//!   - ε̂ = mean( min(B_t, S_t) )
//!   - μ̂ = mean( |B_t − S_t| )
//!   - α̂ = fraction of days where |B_t − S_t| exceeds 1 · sample-stdev
//!     of B−S
//!
//! While simpler than MLE, the MoM estimator captures the same core
//! signal: days with abnormal order imbalance reflect informed trading
//! events.
//!
//! Pure compute. Companion to `vpin`, `weighted_midprice`, `kyles_lambda`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DailyOrderFlow {
    pub buys: f64,
    pub sells: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PinReport {
    pub alpha: f64,
    pub mu: f64,
    pub epsilon: f64,
    pub pin: f64,
    pub mean_imbalance: f64,
    pub n_days: usize,
}

pub fn estimate(flow: &[DailyOrderFlow]) -> Option<PinReport> {
    if flow.len() < 10 {
        return None;
    }
    if flow
        .iter()
        .any(|d| !d.buys.is_finite() || !d.sells.is_finite() || d.buys < 0.0 || d.sells < 0.0)
    {
        return None;
    }
    let n = flow.len();
    let n_f = n as f64;
    // ε̂ = mean of min(B,S) (uninformed-only days have low imbalance).
    let epsilon: f64 = flow.iter().map(|d| d.buys.min(d.sells)).sum::<f64>() / n_f;
    // μ̂ = mean of |B - S|.
    let imbalances: Vec<f64> = flow.iter().map(|d| (d.buys - d.sells).abs()).collect();
    let mean_imbalance: f64 = imbalances.iter().sum::<f64>() / n_f;
    let mu = mean_imbalance;
    // α̂ = fraction of days where |B - S| > 1·σ_{B-S}.
    let imb_mean: f64 = flow.iter().map(|d| d.buys - d.sells).sum::<f64>() / n_f;
    let imb_var: f64 = flow
        .iter()
        .map(|d| (d.buys - d.sells - imb_mean).powi(2))
        .sum::<f64>()
        / n_f;
    let imb_sd = imb_var.max(0.0).sqrt();
    let threshold = imb_sd;
    let alpha = if threshold > 0.0 {
        imbalances.iter().filter(|i| **i > threshold).count() as f64 / n_f
    } else {
        0.0
    };
    let pin_denom = alpha * mu + 2.0 * epsilon;
    let pin = if pin_denom > 0.0 {
        (alpha * mu) / pin_denom
    } else {
        0.0
    };
    Some(PinReport {
        alpha,
        mu,
        epsilon,
        pin,
        mean_imbalance,
        n_days: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(b: f64, s: f64) -> DailyOrderFlow {
        DailyOrderFlow { buys: b, sells: s }
    }

    #[test]
    fn too_short_returns_none() {
        let flow = vec![d(100.0, 100.0); 5];
        assert!(estimate(&flow).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        let mut flow = vec![d(100.0, 100.0); 20];
        flow[5] = d(f64::NAN, 100.0);
        assert!(estimate(&flow).is_none());
    }

    #[test]
    fn negative_counts_return_none() {
        let mut flow = vec![d(100.0, 100.0); 20];
        flow[5] = d(-10.0, 100.0);
        assert!(estimate(&flow).is_none());
    }

    #[test]
    fn balanced_flow_yields_low_pin() {
        // Every day has B ≈ S, no informed events.
        let flow: Vec<_> = (0..50)
            .map(|i| d(100.0 + (i % 3) as f64, 100.0 + (i % 3) as f64))
            .collect();
        let r = estimate(&flow).unwrap();
        // PIN should be very low since μ ≈ 0.
        assert!(
            r.pin < 0.1,
            "balanced flow PIN should be low, got {}",
            r.pin
        );
    }

    #[test]
    fn heavy_imbalance_yields_higher_pin() {
        // Build two flows: one fully balanced, one with frequent strong
        // imbalances. The MoM PIN should be higher on the imbalanced one.
        let balanced: Vec<DailyOrderFlow> = (0..40).map(|_| d(100.0, 100.0)).collect();
        let mut imbalanced: Vec<DailyOrderFlow> = (0..40).map(|_| d(100.0, 100.0)).collect();
        // Every other day, large buy-side imbalance.
        for i in (1..40).step_by(2) {
            imbalanced[i] = d(2000.0, 100.0);
        }
        let r_bal = estimate(&balanced).unwrap();
        let r_imb = estimate(&imbalanced).unwrap();
        assert!(
            r_imb.pin > r_bal.pin,
            "imbalanced PIN {} should exceed balanced PIN {}",
            r_imb.pin,
            r_bal.pin
        );
        // And the imbalanced PIN should be appreciable (> 0.05).
        assert!(
            r_imb.pin > 0.05,
            "imbalanced PIN should be > 0.05, got {}",
            r_imb.pin
        );
    }

    #[test]
    fn pin_in_unit_range() {
        let mut state: u64 = 42;
        let flow: Vec<_> = (0..100)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let b = ((state >> 32) as f64 / u32::MAX as f64) * 200.0;
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let s = ((state >> 32) as f64 / u32::MAX as f64) * 200.0;
                d(b, s)
            })
            .collect();
        let r = estimate(&flow).unwrap();
        assert!((0.0..=1.0).contains(&r.pin), "PIN {} out of [0,1]", r.pin);
        assert!((0.0..=1.0).contains(&r.alpha));
    }

    #[test]
    fn epsilon_is_mean_min_volume() {
        let mut flow = vec![
            d(100.0, 50.0), // min 50
            d(80.0, 120.0), // min 80
            d(60.0, 60.0),  // min 60
            d(40.0, 200.0), // min 40
            d(150.0, 50.0), // min 50
        ];
        flow.extend(std::iter::repeat_n(d(100.0, 100.0), 10));
        let r = estimate(&flow).unwrap();
        let expected_eps =
            flow.iter().map(|d| d.buys.min(d.sells)).sum::<f64>() / flow.len() as f64;
        assert!((r.epsilon - expected_eps).abs() < 1e-12);
    }

    #[test]
    fn n_days_reported_correctly() {
        let flow = vec![d(100.0, 100.0); 25];
        let r = estimate(&flow).unwrap();
        assert_eq!(r.n_days, 25);
    }
}
