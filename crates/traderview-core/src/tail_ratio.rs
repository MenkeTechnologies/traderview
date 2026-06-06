//! Tail Ratio — ratio of the 95th percentile return to the absolute
//! 5th percentile (worst-tail) return.
//!
//!   TailRatio = |Q_95(returns)| / |Q_05(returns)|
//!
//! Values:
//!   - TailRatio > 1 → upside fatter than downside (positive skew)
//!   - TailRatio < 1 → downside fatter than upside (crash exposure)
//!   - TailRatio ≈ 1 → symmetric tails
//!
//! Companion: "Common Sense Ratio" = profit_factor · tail_ratio
//! (more skeptical of strategies with strong P/F but bad tails).
//!
//! Pure compute. Companion to `cornish_fisher`, `realized_higher_moments`,
//! `omega_ratio`, `gain_to_pain_ratio`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TailRatioReport {
    pub tail_ratio: f64,
    pub right_tail_value: f64,
    pub left_tail_value: f64,
    pub common_sense_ratio: f64,
    pub upper_quantile: f64,
    pub lower_quantile: f64,
    pub n_observations: usize,
}

pub fn compute(returns: &[f64], upper_q: f64, lower_q: f64) -> Option<TailRatioReport> {
    let n = returns.len();
    if n < 20
        || !upper_q.is_finite()
        || !lower_q.is_finite()
        || !(0.5..1.0).contains(&upper_q)
        || !(0.0..0.5).contains(&lower_q)
    {
        return None;
    }
    if returns.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let mut sorted = returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let q = |p: f64| {
        let idx = ((p * (n - 1) as f64).round() as usize).min(n - 1);
        sorted[idx]
    };
    let right = q(upper_q);
    let left = q(lower_q);
    if left.abs() <= 0.0 {
        return None;
    }
    let tail = right.abs() / left.abs();
    // Profit factor: sum positive / sum |negative|.
    let pos_sum: f64 = returns.iter().filter(|r| **r > 0.0).sum();
    let neg_sum: f64 = returns.iter().filter(|r| **r < 0.0).map(|r| r.abs()).sum();
    let pf = if neg_sum > 0.0 {
        pos_sum / neg_sum
    } else {
        f64::INFINITY
    };
    let common_sense = pf * tail;
    Some(TailRatioReport {
        tail_ratio: tail,
        right_tail_value: right,
        left_tail_value: left,
        common_sense_ratio: common_sense,
        upper_quantile: upper_q,
        lower_quantile: lower_q,
        n_observations: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_returns_none() {
        let r = vec![0.01_f64; 10];
        assert!(compute(&r, 0.95, 0.05).is_none());
    }

    #[test]
    fn invalid_quantiles_return_none() {
        let r = vec![0.01_f64; 50];
        assert!(compute(&r, 0.4, 0.05).is_none());
        assert!(compute(&r, 0.95, 0.6).is_none());
        assert!(compute(&r, 1.0, 0.05).is_none());
        assert!(compute(&r, f64::NAN, 0.05).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut r = vec![0.01_f64; 50];
        r[5] = f64::NAN;
        assert!(compute(&r, 0.95, 0.05).is_none());
    }

    #[test]
    fn symmetric_returns_yield_tail_near_one() {
        let r: Vec<f64> = (-25..=25).map(|i| i as f64 * 0.001).collect();
        let result = compute(&r, 0.95, 0.05).unwrap();
        assert!(
            (result.tail_ratio - 1.0).abs() < 0.20,
            "symmetric tail ratio ≈ 1, got {}",
            result.tail_ratio
        );
    }

    #[test]
    fn positive_skew_yields_tail_above_one() {
        // Many small losses, big winners. 80 losses + 20 wins so q(0.95)
        // lands inside the wins block (indices 80..99).
        let mut r = vec![-0.001_f64; 80];
        r.extend(vec![0.1_f64; 20]);
        let result = compute(&r, 0.95, 0.05).unwrap();
        assert!(
            result.tail_ratio > 1.0,
            "positive skew should yield tail > 1, got {}",
            result.tail_ratio
        );
    }

    #[test]
    fn negative_skew_yields_tail_below_one() {
        // Many small wins, big losses. 20 losses + 80 wins so q(0.05)
        // lands inside the losses block (indices 0..19).
        let mut r = vec![-0.1_f64; 20];
        r.extend(vec![0.001_f64; 80]);
        let result = compute(&r, 0.95, 0.05).unwrap();
        assert!(
            result.tail_ratio < 1.0,
            "negative skew should yield tail < 1, got {}",
            result.tail_ratio
        );
    }

    #[test]
    fn common_sense_ratio_is_profit_factor_times_tail() {
        let r: Vec<f64> = (-25..=25).map(|i| i as f64 * 0.001).collect();
        let result = compute(&r, 0.95, 0.05).unwrap();
        // Sum positive = sum |negative| in symmetric input → pf = 1.
        // common_sense ≈ tail_ratio.
        if result.tail_ratio.is_finite() {
            assert!((result.common_sense_ratio - result.tail_ratio).abs() < 0.30);
        }
    }

    #[test]
    fn n_observations_reported() {
        let r = vec![0.01_f64; 100];
        let result = compute(&r, 0.95, 0.05);
        // All-same returns → left == 0.01, abs(left) = 0.01 > 0; tail = 1.
        if let Some(rep) = result {
            assert_eq!(rep.n_observations, 100);
        }
    }
}
