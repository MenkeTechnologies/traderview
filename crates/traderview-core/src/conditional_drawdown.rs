//! Conditional Drawdown at Risk (CDaR) — Chekhlov, Uryasev, Zabarankin
//! (2003).
//!
//! Tail analogue of Value-at-Risk for the drawdown distribution:
//! CDaR(α) is the average of the WORST `α` fraction of drawdown
//! observations along an equity curve. Used to bound the tail of the
//! drawdown distribution rather than just the maximum drawdown
//! (which is a single-point estimator).
//!
//!   1. Build per-bar drawdown series: DD_t = (peak_t − value_t) / peak_t.
//!   2. Sort DD descending; take the top `⌈α·N⌉` observations.
//!   3. CDaR = mean of those tail observations.
//!
//! Also returns max drawdown and average drawdown for comparison.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct CdarReport {
    pub max_drawdown: f64,
    pub average_drawdown: f64,
    pub conditional_drawdown_at_risk: f64,
    pub n_observations: usize,
    pub n_tail: usize,
}

pub fn compute(equity_curve: &[f64], alpha: f64) -> Option<CdarReport> {
    if equity_curve.len() < 2
        || !alpha.is_finite()
        || !(0.0..1.0).contains(&alpha) || alpha == 0.0
    {
        return None;
    }
    let mut dds = Vec::with_capacity(equity_curve.len());
    let mut peak = f64::NEG_INFINITY;
    let mut any_valid = false;
    for v in equity_curve {
        if !v.is_finite() { continue; }
        any_valid = true;
        if *v > peak { peak = *v; }
        let dd = if peak > 0.0 { (peak - v).max(0.0) / peak } else { 0.0 };
        dds.push(dd);
    }
    if !any_valid || dds.is_empty() { return None; }
    let n = dds.len();
    let max_dd = dds.iter().copied().fold(0.0_f64, f64::max);
    let avg_dd = dds.iter().sum::<f64>() / n as f64;
    let mut sorted = dds.clone();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    let k = ((n as f64) * alpha).ceil().max(1.0) as usize;
    let k = k.min(n);
    let cdar = sorted[..k].iter().sum::<f64>() / k as f64;
    Some(CdarReport {
        max_drawdown: max_dd,
        average_drawdown: avg_dd,
        conditional_drawdown_at_risk: cdar,
        n_observations: n,
        n_tail: k,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(&[], 0.05).is_none());
        assert!(compute(&[1.0], 0.05).is_none());
        assert!(compute(&[1.0, 1.0], 0.0).is_none());
        assert!(compute(&[1.0, 1.0], 1.0).is_none());
        assert!(compute(&[1.0, 1.0], -0.1).is_none());
        assert!(compute(&[1.0, 1.0], f64::NAN).is_none());
    }

    #[test]
    fn monotonic_increasing_equity_yields_zero_drawdown() {
        let curve: Vec<f64> = (1..=20).map(|i| i as f64).collect();
        let r = compute(&curve, 0.10).unwrap();
        assert_eq!(r.max_drawdown, 0.0);
        assert_eq!(r.average_drawdown, 0.0);
        assert_eq!(r.conditional_drawdown_at_risk, 0.0);
    }

    #[test]
    fn cdar_ge_max_drawdown_never() {
        // CDaR averages the WORST k observations; max_drawdown is the
        // single worst → CDaR ≤ max_drawdown always.
        let mut curve = vec![100.0_f64];
        for _ in 0..30 { curve.push(curve.last().unwrap() * 0.99); }
        for _ in 0..30 { curve.push(curve.last().unwrap() * 1.005); }
        let r = compute(&curve, 0.05).unwrap();
        assert!(r.conditional_drawdown_at_risk <= r.max_drawdown + 1e-12);
    }

    #[test]
    fn cdar_ge_average_drawdown() {
        // Averaging the worst-k is ≥ averaging all.
        let mut curve = vec![100.0_f64];
        for _ in 0..30 { curve.push(curve.last().unwrap() * 0.99); }
        for _ in 0..30 { curve.push(curve.last().unwrap() * 1.005); }
        let r = compute(&curve, 0.10).unwrap();
        assert!(r.conditional_drawdown_at_risk >= r.average_drawdown - 1e-12);
    }

    #[test]
    fn known_drawdown_recovered() {
        // 100 → 80 → 100 → 90: max drawdown = (100-80)/100 = 20%.
        let curve = vec![100.0, 80.0, 100.0, 90.0];
        let r = compute(&curve, 0.50).unwrap();
        assert!((r.max_drawdown - 0.20).abs() < 1e-9);
    }

    #[test]
    fn nan_inputs_skipped() {
        let curve = vec![100.0, f64::NAN, 80.0, 100.0];
        let r = compute(&curve, 0.50).unwrap();
        assert_eq!(r.n_observations, 3);
        assert!((r.max_drawdown - 0.20).abs() < 1e-9);
    }

    #[test]
    fn smaller_alpha_focuses_tighter_tail() {
        // CDaR(0.01) ≥ CDaR(0.50) for any drawdown distribution.
        let mut curve = vec![100.0_f64];
        for i in 0..100 {
            let drop = 1.0 - (i as f64 * 0.005).min(0.5);
            curve.push(100.0 * drop);
        }
        let r_1 = compute(&curve, 0.01).unwrap();
        let r_50 = compute(&curve, 0.50).unwrap();
        assert!(r_1.conditional_drawdown_at_risk >= r_50.conditional_drawdown_at_risk - 1e-9);
    }
}
