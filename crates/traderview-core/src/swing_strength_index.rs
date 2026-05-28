//! Swing Strength Index — measures the magnitude of each ZigZag swing
//! relative to the prior swing.
//!
//! Given a ZigZag-style swing series (alternating pivot highs/lows):
//!   swing_magnitude_t = |pivot[t].price - pivot[t-1].price|
//!   strength_ratio_t  = swing_magnitude_t / swing_magnitude_{t-1}
//!     (ratio > 1.0 = current swing stronger than prior)
//!   strength_z_t      = (strength_ratio_t - SMA(ratios, z_period))
//!                       / stdev(ratios, z_period)
//!
//! Used to flag impulse vs corrective legs: ratio > 1 generally
//! indicates an impulse leg (trend continuation), ratio < 1 suggests
//! corrective.
//!
//! Pure compute. Default z_period = 10.
//! Companion to `zigzag`, `weiss_wave`, `pivot_points`, `gartley_pattern`.

use serde::{Deserialize, Serialize};

pub use crate::gartley_pattern::Pivot;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SwingStrength {
    pub pivot_index: usize,
    pub magnitude: f64,
    pub ratio_vs_prior: Option<f64>,
    pub zscore: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SwingStrengthReport {
    pub swings: Vec<SwingStrength>,
    pub z_period: usize,
}

pub fn compute(pivots: &[Pivot], z_period: usize) -> SwingStrengthReport {
    let mut report = SwingStrengthReport {
        swings: Vec::new(),
        z_period,
    };
    if pivots.len() < 2 || z_period < 3 { return report; }
    if pivots.iter().any(|p| !p.price.is_finite() || p.price <= 0.0) { return report; }
    let mut ratios: Vec<f64> = Vec::new();
    let mut magnitudes = Vec::new();
    for w in pivots.windows(2) {
        let mag = (w[1].price - w[0].price).abs();
        magnitudes.push(mag);
    }
    for (i, &mag) in magnitudes.iter().enumerate() {
        let ratio = if i > 0 && magnitudes[i - 1] > 0.0 {
            Some(mag / magnitudes[i - 1])
        } else { None };
        if let Some(r) = ratio { ratios.push(r); }
        let zscore = if ratios.len() >= z_period {
            let win = &ratios[ratios.len() - z_period..];
            let p_f = z_period as f64;
            let mean: f64 = win.iter().sum::<f64>() / p_f;
            let var: f64 = win.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / p_f;
            let std = var.max(0.0).sqrt();
            if std > 0.0 {
                Some((ratio.unwrap() - mean) / std)
            } else { Some(0.0) }
        } else { None };
        report.swings.push(SwingStrength {
            pivot_index: i + 1,    // pivot at index i+1 is the "current swing" endpoint
            magnitude: mag,
            ratio_vs_prior: ratio,
            zscore,
        });
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(idx: usize, price: f64, is_high: bool) -> Pivot {
        Pivot { index: idx, price, is_high }
    }

    #[test]
    fn empty_or_invalid_returns_empty() {
        let r = compute(&[], 10);
        assert!(r.swings.is_empty());
        let pivots = vec![p(0, 100.0, true)];
        let r2 = compute(&pivots, 10);
        assert!(r2.swings.is_empty());
        let r3 = compute(&[p(0, 100.0, true), p(1, 105.0, false)], 1);
        assert!(r3.swings.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let pivots = vec![p(0, f64::NAN, true), p(1, 100.0, false)];
        let r = compute(&pivots, 10);
        assert!(r.swings.is_empty());
    }

    #[test]
    fn first_swing_has_no_ratio() {
        let pivots = vec![p(0, 100.0, true), p(1, 110.0, false)];
        let r = compute(&pivots, 10);
        assert_eq!(r.swings.len(), 1);
        assert!(r.swings[0].ratio_vs_prior.is_none());
    }

    #[test]
    fn equal_swings_yield_unit_ratio() {
        let pivots = vec![
            p(0, 100.0, true),
            p(10, 90.0, false),    // mag 10
            p(20, 100.0, true),    // mag 10
            p(30, 90.0, false),    // mag 10
        ];
        let r = compute(&pivots, 10);
        assert_eq!(r.swings.len(), 3);
        // Swings 1 and 2 have ratio 1.0 vs prior.
        assert!((r.swings[1].ratio_vs_prior.unwrap() - 1.0).abs() < 1e-9);
        assert!((r.swings[2].ratio_vs_prior.unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn expanding_swings_yield_greater_than_one_ratios() {
        let pivots = vec![
            p(0, 100.0, true),
            p(10, 95.0, false),    // mag 5
            p(20, 105.0, true),    // mag 10 (ratio 2.0)
            p(30, 90.0, false),    // mag 15 (ratio 1.5)
        ];
        let r = compute(&pivots, 10);
        assert!(r.swings[1].ratio_vs_prior.unwrap() > 1.0);
        assert!(r.swings[2].ratio_vs_prior.unwrap() > 1.0);
    }

    #[test]
    fn magnitude_correct() {
        let pivots = vec![p(0, 100.0, true), p(10, 87.5, false)];
        let r = compute(&pivots, 10);
        assert!((r.swings[0].magnitude - 12.5).abs() < 1e-9);
    }
}
