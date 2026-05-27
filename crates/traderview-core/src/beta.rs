//! Single-asset beta vs benchmark estimator.
//!
//! Beta = covariance(asset, benchmark) / variance(benchmark)
//!
//! Helpful when:
//!   - Sizing for market-neutrality (long $X, short $X × beta of benchmark)
//!   - Decomposing P&L: market-driven vs alpha-driven
//!   - Checking if a "low beta" stock has actually been moving with the
//!     market lately (rolling beta vs static beta)
//!
//! Also emits alpha (intercept) so the caller can see the asset's
//! non-market return per period.
//!
//! Pure compute. Caller pre-aligns return series.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BetaReport {
    pub beta: f64,
    /// Alpha (intercept) in same period units as the returns. Caller
    /// annualizes if needed.
    pub alpha: f64,
    /// R-squared — fraction of asset variance explained by benchmark.
    pub r_squared: f64,
    /// Correlation coefficient (same sign as beta).
    pub correlation: f64,
    pub n: usize,
}

pub fn estimate(asset: &[f64], benchmark: &[f64]) -> Option<BetaReport> {
    if asset.len() != benchmark.len() || asset.len() < 2 {
        return None;
    }
    let n = asset.len() as f64;
    let mean_a = asset.iter().sum::<f64>() / n;
    let mean_b = benchmark.iter().sum::<f64>() / n;
    let mut cov = 0.0;
    let mut var_b = 0.0;
    let mut var_a = 0.0;
    for i in 0..asset.len() {
        let da = asset[i] - mean_a;
        let db = benchmark[i] - mean_b;
        cov += da * db;
        var_b += db * db;
        var_a += da * da;
    }
    if var_b == 0.0 {
        return None;
    }
    let beta = cov / var_b;
    let alpha = mean_a - beta * mean_b;
    let correlation = if var_a > 0.0 && var_b > 0.0 {
        cov / (var_a.sqrt() * var_b.sqrt())
    } else {
        0.0
    };
    let r_squared = correlation * correlation;
    Some(BetaReport {
        beta,
        alpha,
        r_squared,
        correlation,
        n: asset.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        assert!(estimate(&[], &[]).is_none());
    }

    #[test]
    fn length_mismatch_returns_none() {
        assert!(estimate(&[1.0, 2.0], &[1.0]).is_none());
    }

    #[test]
    fn zero_variance_benchmark_returns_none() {
        // Benchmark flat → beta undefined (divide by zero).
        assert!(estimate(&[1.0, 2.0, 3.0], &[5.0, 5.0, 5.0]).is_none());
    }

    #[test]
    fn perfect_match_beta_one_r_squared_one() {
        // Asset == benchmark exactly.
        let series = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r = estimate(&series, &series).unwrap();
        assert!((r.beta - 1.0).abs() < 1e-12);
        assert!((r.r_squared - 1.0).abs() < 1e-12);
        assert!((r.alpha - 0.0).abs() < 1e-12);
    }

    #[test]
    fn asset_is_two_times_benchmark_beta_two() {
        let bench = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let asset = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let r = estimate(&asset, &bench).unwrap();
        assert!((r.beta - 2.0).abs() < 1e-9);
        assert!((r.r_squared - 1.0).abs() < 1e-12);
    }

    #[test]
    fn negatively_correlated_asset_negative_beta() {
        let bench = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let asset = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        let r = estimate(&asset, &bench).unwrap();
        assert!((r.beta + 1.0).abs() < 1e-9, "beta should be -1.0");
        assert!(r.correlation < 0.0);
    }

    #[test]
    fn alpha_captures_constant_offset() {
        // Asset = bench + 10 → beta = 1, alpha = 10.
        let bench = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let asset = bench.iter().map(|x| x + 10.0).collect::<Vec<_>>();
        let r = estimate(&asset, &bench).unwrap();
        assert!((r.beta - 1.0).abs() < 1e-9);
        assert!((r.alpha - 10.0).abs() < 1e-9);
    }

    #[test]
    fn r_squared_zero_for_independent_series() {
        // Construct two truly independent-looking series.
        let bench = vec![1.0, -1.0, 1.0, -1.0, 1.0, -1.0];
        let asset = vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        // Asset variance = 0 → correlation = 0 by our formula.
        let r = estimate(&asset, &bench).unwrap();
        assert_eq!(r.r_squared, 0.0);
        assert_eq!(r.correlation, 0.0);
    }

    #[test]
    fn low_beta_stock_significantly_under_one() {
        // Asset moves about 30% as much as bench.
        let bench = vec![10.0, 11.0, 9.0, 12.0, 8.0];
        let asset = bench
            .iter()
            .map(|x| 10.0 + (x - 10.0) * 0.3)
            .collect::<Vec<_>>();
        let r = estimate(&asset, &bench).unwrap();
        assert!((r.beta - 0.3).abs() < 1e-9);
    }
}
