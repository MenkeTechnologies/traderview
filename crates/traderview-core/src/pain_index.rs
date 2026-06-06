//! Pain Index (Becker 1992) — mean absolute drawdown over an equity
//! curve.
//!
//!   pain_index = (1/N) · Σ |DD_t|
//!
//! where DD_t is the per-bar drawdown from the running peak. Distinct
//! from `ulcer_index` (which uses RMS DD) and `risk_adjusted_ratios`
//! (which derives ratios from these denominators).
//!
//! Companion `pain_ratio = (annualized_return − r_f) / pain_index` is
//! also reported.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct PainReport {
    pub pain_index: f64,
    pub pain_ratio: f64,
    pub annualized_return: f64,
    pub n_observations: usize,
}

pub fn compute(
    equity_curve: &[f64],
    period_returns: &[f64],
    risk_free_annual: f64,
    periods_per_year: f64,
) -> Option<PainReport> {
    if equity_curve.len() < 2
        || period_returns.is_empty()
        || !risk_free_annual.is_finite()
        || !periods_per_year.is_finite()
        || periods_per_year <= 0.0
    {
        return None;
    }
    let mut peak = f64::NEG_INFINITY;
    let mut sum_abs_dd = 0.0_f64;
    let mut count = 0_usize;
    for v in equity_curve {
        if !v.is_finite() {
            continue;
        }
        if *v > peak {
            peak = *v;
        }
        let dd = if peak > 0.0 {
            (peak - v).max(0.0) / peak
        } else {
            0.0
        };
        sum_abs_dd += dd;
        count += 1;
    }
    if count == 0 {
        return None;
    }
    let pain = sum_abs_dd / count as f64;
    // Annualized return.
    let clean_rets: Vec<f64> = period_returns
        .iter()
        .copied()
        .filter(|x| x.is_finite())
        .collect();
    if clean_rets.is_empty() {
        return None;
    }
    let mean_ret = clean_rets.iter().sum::<f64>() / clean_rets.len() as f64;
    let annual = mean_ret * periods_per_year;
    let excess = annual - risk_free_annual;
    let ratio = if pain > 0.0 {
        excess / pain
    } else if excess.abs() < 1e-12 {
        0.0
    } else {
        f64::INFINITY * excess.signum()
    };
    Some(PainReport {
        pain_index: pain,
        pain_ratio: ratio,
        annualized_return: annual,
        n_observations: count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(&[], &[0.01], 0.0, 252.0).is_none());
        assert!(compute(&[1.0, 2.0], &[], 0.0, 252.0).is_none());
        assert!(compute(&[1.0, 2.0], &[0.01], f64::NAN, 252.0).is_none());
        assert!(compute(&[1.0, 2.0], &[0.01], 0.0, 0.0).is_none());
    }

    #[test]
    fn monotonic_increasing_yields_zero_pain() {
        let curve: Vec<f64> = (1..=20).map(|i| i as f64).collect();
        let rets = vec![0.05; 19];
        let r = compute(&curve, &rets, 0.0, 252.0).unwrap();
        assert_eq!(r.pain_index, 0.0);
        assert!(r.pain_ratio.is_infinite() && r.pain_ratio > 0.0);
    }

    #[test]
    fn drawdown_curve_yields_positive_pain() {
        let curve = vec![100.0, 80.0, 100.0, 90.0];
        let rets = vec![0.0, -0.2, 0.25, -0.10];
        let r = compute(&curve, &rets, 0.0, 4.0).unwrap();
        // DD series: [0, 0.20, 0, 0.10] → mean = 0.075.
        assert!((r.pain_index - 0.075).abs() < 1e-9);
        assert!(r.pain_ratio.is_finite());
    }

    #[test]
    fn nan_inputs_skipped() {
        let curve = vec![100.0, f64::NAN, 80.0, 100.0];
        let rets = vec![0.0, -0.2, 0.25];
        let r = compute(&curve, &rets, 0.0, 4.0).unwrap();
        assert_eq!(r.n_observations, 3);
    }

    #[test]
    fn larger_drawdown_inflates_pain_index() {
        let small = vec![100.0, 95.0, 100.0];
        let large = vec![100.0, 60.0, 100.0];
        let r_small = compute(&small, &[0.0, -0.05, 0.0526], 0.0, 252.0).unwrap();
        let r_large = compute(&large, &[0.0, -0.4, 0.6667], 0.0, 252.0).unwrap();
        assert!(r_large.pain_index > r_small.pain_index);
    }

    #[test]
    fn risk_free_subtracted_in_ratio() {
        let curve = vec![100.0, 80.0, 100.0];
        let rets = vec![0.0, -0.2, 0.25];
        let mean_ret = rets.iter().sum::<f64>() / 3.0;
        let annual = mean_ret * 252.0;
        let r = compute(&curve, &rets, annual, 252.0).unwrap();
        assert!(r.pain_ratio.abs() < 1e-9 || r.pain_ratio == 0.0);
    }
}
