//! Gain-to-Pain Ratio (Jack Schwager, "Hedge Fund Market Wizards").
//!
//!   GPR = sum_of_monthly_returns / sum_of_absolute_monthly_losses
//!
//! Numerator is the total cumulative arithmetic return; denominator is
//! the sum of absolute values of all negative-return months.
//!
//! Schwager benchmarks:
//!   - GPR > 1.0 → acceptable hedge fund
//!   - GPR > 1.5 → very good
//!   - GPR > 2.0 → exceptional
//!
//! Companion: "Gain-to-Pain Index" = same ratio but uses ulcer-style
//! squared losses in denominator (more punishment for outliers).
//!
//! Pure compute. Companion to `gain_pain_ratio` (which exists with a
//! different definition), `omega_ratio`, `sortino_ratio`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GainToPainReport {
    pub gain_to_pain_ratio: f64,
    pub gain_to_pain_index: f64,
    pub sum_returns: f64,
    pub sum_absolute_losses: f64,
    pub n_periods: usize,
    pub n_losing_periods: usize,
}

pub fn compute(returns: &[f64]) -> Option<GainToPainReport> {
    if returns.is_empty() {
        return None;
    }
    if returns.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let sum: f64 = returns.iter().sum();
    let losses: Vec<f64> = returns.iter().filter(|r| **r < 0.0).copied().collect();
    let sum_abs_losses: f64 = losses.iter().map(|l| l.abs()).sum();
    let sum_sq_losses: f64 = losses.iter().map(|l| l * l).sum();
    let gpr = if sum_abs_losses > 0.0 {
        sum / sum_abs_losses
    } else {
        f64::INFINITY
    };
    let gpi = if sum_sq_losses > 0.0 {
        sum / sum_sq_losses.sqrt()
    } else {
        f64::INFINITY
    };
    Some(GainToPainReport {
        gain_to_pain_ratio: gpr,
        gain_to_pain_index: gpi,
        sum_returns: sum,
        sum_absolute_losses: sum_abs_losses,
        n_periods: returns.len(),
        n_losing_periods: losses.len(),
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
    fn nan_returns_none() {
        assert!(compute(&[0.01, f64::NAN, 0.02]).is_none());
    }

    #[test]
    fn all_gains_infinite_gpr() {
        let r = vec![0.01, 0.02, 0.005, 0.015];
        let result = compute(&r).unwrap();
        assert!(result.gain_to_pain_ratio.is_infinite());
        assert_eq!(result.n_losing_periods, 0);
    }

    #[test]
    fn mixed_returns_gpr_computed() {
        // Sum = 0.04, losses = -0.01, -0.02 → abs sum 0.03 → GPR = 0.04/0.03 ≈ 1.333.
        let r = vec![0.03, -0.01, 0.04, -0.02];
        let result = compute(&r).unwrap();
        assert_eq!(result.n_losing_periods, 2);
        assert!((result.sum_returns - 0.04).abs() < 1e-12);
        assert!((result.sum_absolute_losses - 0.03).abs() < 1e-12);
        assert!((result.gain_to_pain_ratio - (0.04 / 0.03)).abs() < 1e-9);
    }

    #[test]
    fn all_losses_negative_gpr() {
        let r = vec![-0.01, -0.02, -0.005];
        let result = compute(&r).unwrap();
        assert!(result.gain_to_pain_ratio < 0.0);
    }

    #[test]
    fn n_losing_periods_counted_strictly_below_zero() {
        let r = vec![0.01, 0.0, -0.01, 0.02];
        let result = compute(&r).unwrap();
        // Zero return → not a loss.
        assert_eq!(result.n_losing_periods, 1);
    }

    #[test]
    fn gpi_uses_squared_losses() {
        // Single big loss vs many small ones with same |sum|:
        // big-loss version has larger sum_sq → smaller GPI.
        let small = vec![0.10, -0.01, -0.01, -0.01, -0.01, -0.01]; // 5x -0.01
        let big = vec![0.10, -0.05]; // 1x -0.05
        let s = compute(&small).unwrap();
        let b = compute(&big).unwrap();
        // sum_abs_losses both 0.05; sum_sq for small = 5·0.0001 = 0.0005;
        // for big = 0.0025. So sqrt(0.0025) > sqrt(0.0005), making GPI smaller.
        assert!(
            b.gain_to_pain_index < s.gain_to_pain_index,
            "concentrated loss should depress GPI more, big={}, small={}",
            b.gain_to_pain_index,
            s.gain_to_pain_index
        );
    }

    #[test]
    fn n_periods_reported() {
        let r = vec![0.01; 25];
        let result = compute(&r).unwrap();
        assert_eq!(result.n_periods, 25);
    }
}
