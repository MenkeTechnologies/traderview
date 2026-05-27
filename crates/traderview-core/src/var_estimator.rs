//! Value-at-Risk (VAR) estimator — historical + parametric variants.
//!
//! **Historical VAR**: percentile of historical loss distribution.
//!   E.g., 95% VAR = 5th-percentile loss over lookback window.
//!
//! **Parametric (Gaussian) VAR**: assume normal returns, compute from
//! mean + stdev:
//!   VAR_95 = -(μ - 1.645 × σ) × position_size
//!
//! Both return DOLLAR loss at the given confidence level (one-day
//! horizon by default; caller scales with √time for multi-day).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VarReport {
    pub method: String,
    pub confidence: f64,
    pub var_dollars: f64,
    pub expected_shortfall_dollars: f64,
    pub n: usize,
}

pub fn historical(daily_returns: &[f64], position_value: f64, confidence: f64) -> VarReport {
    let mut report = VarReport {
        method: "historical".into(),
        confidence,
        n: daily_returns.len(),
        ..Default::default()
    };
    if daily_returns.len() < 10 || confidence <= 0.0 || confidence >= 1.0 {
        return report;
    }
    let mut sorted = daily_returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let alpha = 1.0 - confidence;
    // Take the worst-tail percentile: for 5% tail of 100 obs, index 4.
    // floor(alpha × n) - 1 with saturating sub to keep at least index 0.
    let idx = ((alpha * sorted.len() as f64).floor() as usize).saturating_sub(1);
    let var_pct = -sorted[idx.min(sorted.len() - 1)];
    report.var_dollars = var_pct * position_value;
    // Expected Shortfall (CVaR) = mean of losses worse than VAR.
    let tail: Vec<f64> = sorted[..=idx.min(sorted.len() - 1)].iter()
        .map(|r| -r).collect();
    if !tail.is_empty() {
        report.expected_shortfall_dollars =
            tail.iter().sum::<f64>() / tail.len() as f64 * position_value;
    }
    report
}

pub fn parametric_gaussian(daily_returns: &[f64], position_value: f64, confidence: f64)
    -> VarReport
{
    let mut report = VarReport {
        method: "parametric_gaussian".into(),
        confidence,
        n: daily_returns.len(),
        ..Default::default()
    };
    if daily_returns.len() < 2 || confidence <= 0.0 || confidence >= 1.0 {
        return report;
    }
    let n = daily_returns.len() as f64;
    let mean = daily_returns.iter().sum::<f64>() / n;
    let var = daily_returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n;
    let stdev = var.sqrt();
    // z-score for confidence (1.645 for 95%, 2.326 for 99%).
    let z = inverse_normal(confidence);
    let var_pct = -(mean - z * stdev);
    report.var_dollars = var_pct.max(0.0) * position_value;
    // ES under Gaussian: ES = -(μ - σ × φ(z)/α) where φ is pdf, α = 1-confidence.
    let alpha = 1.0 - confidence;
    let phi_z = (-0.5 * z * z).exp() / (2.0 * std::f64::consts::PI).sqrt();
    let es_pct = -(mean - stdev * phi_z / alpha);
    report.expected_shortfall_dollars = es_pct.max(0.0) * position_value;
    report
}

/// Approximate inverse normal CDF for common confidence levels.
fn inverse_normal(confidence: f64) -> f64 {
    // Quick lookup for common cases; for everything else use the
    // Beasley-Springer-Moro approximation.
    if (confidence - 0.90).abs() < 1e-6 { return 1.282; }
    if (confidence - 0.95).abs() < 1e-6 { return 1.645; }
    if (confidence - 0.99).abs() < 1e-6 { return 2.326; }
    if (confidence - 0.999).abs() < 1e-6 { return 3.090; }
    // Beasley-Springer-Moro for arbitrary confidence.
    let p = confidence;
    if p < 0.5 {
        return -inverse_normal(1.0 - p);
    }
    let t = (-2.0 * (1.0 - p).ln()).sqrt();
    let c0 = 2.515517;
    let c1 = 0.802853;
    let c2 = 0.010328;
    let d1 = 1.432788;
    let d2 = 0.189269;
    let d3 = 0.001308;
    t - (c0 + c1 * t + c2 * t * t) / (1.0 + d1 * t + d2 * t * t + d3 * t * t * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_default() {
        let r = historical(&[], 10_000.0, 0.95);
        assert_eq!(r.var_dollars, 0.0);
    }

    #[test]
    fn historical_95_var_at_5th_percentile_loss() {
        // 100 daily returns: 95 at 0%, 5 at -1%. VAR_95 should be ~1% × position.
        let mut returns = vec![0.0; 95];
        returns.extend(vec![-0.01; 5]);
        let r = historical(&returns, 10_000.0, 0.95);
        // Should report ~$100 VAR (1% of $10k).
        assert!((r.var_dollars - 100.0).abs() < 0.01);
    }

    #[test]
    fn expected_shortfall_above_var() {
        // ES is always ≥ VAR (it's the mean of the worst tail).
        let returns: Vec<f64> = (0..100).map(|i| (i as f64 - 50.0) * 0.001).collect();
        let r = historical(&returns, 10_000.0, 0.95);
        assert!(r.expected_shortfall_dollars >= r.var_dollars,
            "ES ({}) must be >= VAR ({})", r.expected_shortfall_dollars, r.var_dollars);
    }

    #[test]
    fn parametric_zero_volatility_zero_var() {
        let returns = vec![0.01; 30];    // all same → stdev 0.
        let r = parametric_gaussian(&returns, 10_000.0, 0.95);
        assert_eq!(r.var_dollars, 0.0);
    }

    #[test]
    fn parametric_var_positive_and_uses_z_score() {
        // Generic positive-result check — exact scale depends on construction.
        let returns: Vec<f64> = (0..50).map(|i| {
            (i as f64 - 25.0) / 25.0 * 0.01
        }).collect();
        let r = parametric_gaussian(&returns, 10_000.0, 0.95);
        // Just verify positive (mean 0, stdev > 0 → VAR > 0).
        assert!(r.var_dollars > 0.0);
    }

    #[test]
    fn parametric_99_more_severe_than_95() {
        let returns: Vec<f64> = (0..50).map(|i| {
            (i as f64 - 25.0) / 25.0 * 0.01
        }).collect();
        let r95 = parametric_gaussian(&returns, 10_000.0, 0.95);
        let r99 = parametric_gaussian(&returns, 10_000.0, 0.99);
        assert!(r99.var_dollars > r95.var_dollars);
    }

    #[test]
    fn invalid_confidence_returns_default() {
        let r = historical(&[0.01; 30], 10_000.0, 1.5);
        assert_eq!(r.var_dollars, 0.0);
    }

    #[test]
    fn larger_position_proportional_var() {
        let returns = vec![-0.01; 5];
        let mut all_returns = vec![0.0; 95];
        all_returns.extend(returns);
        let small = historical(&all_returns, 1_000.0, 0.95);
        let big = historical(&all_returns, 10_000.0, 0.95);
        assert!((big.var_dollars / small.var_dollars - 10.0).abs() < 1e-9);
    }

    #[test]
    fn historical_99pct_more_severe_than_95pct() {
        let returns: Vec<f64> = (0..1000).map(|i| {
            -((i as f64).powi(2) / 1_000_000.0).min(0.10)
        }).collect();
        let r95 = historical(&returns, 10_000.0, 0.95);
        let r99 = historical(&returns, 10_000.0, 0.99);
        assert!(r99.var_dollars >= r95.var_dollars,
            "99% VAR ({}) should be ≥ 95% VAR ({})", r99.var_dollars, r95.var_dollars);
    }

    #[test]
    fn inverse_normal_known_values() {
        assert!((inverse_normal(0.95) - 1.645).abs() < 1e-3);
        assert!((inverse_normal(0.99) - 2.326).abs() < 1e-3);
    }
}
