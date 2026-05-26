//! Pairwise correlation + beta + spread / z-score for stat-arb pairs trading.
//!
//! All pure functions over `&[f64]` log-returns or prices.

use serde::Serialize;

/// Pearson correlation of two equal-length series.
pub fn pearson(a: &[f64], b: &[f64]) -> Option<f64> {
    let n = a.len().min(b.len());
    if n < 2 { return None; }
    let mean_a = a[..n].iter().sum::<f64>() / n as f64;
    let mean_b = b[..n].iter().sum::<f64>() / n as f64;
    let mut cov = 0.0;
    let mut var_a = 0.0;
    let mut var_b = 0.0;
    for i in 0..n {
        let da = a[i] - mean_a;
        let db = b[i] - mean_b;
        cov   += da * db;
        var_a += da * da;
        var_b += db * db;
    }
    if var_a == 0.0 || var_b == 0.0 { return None; }
    Some(cov / (var_a * var_b).sqrt())
}

/// Log returns from a price series.
pub fn log_returns(prices: &[f64]) -> Vec<f64> {
    let mut out = Vec::with_capacity(prices.len().saturating_sub(1));
    for w in prices.windows(2) {
        if w[0] > 0.0 && w[1] > 0.0 { out.push((w[1] / w[0]).ln()); }
        else { out.push(0.0); }
    }
    out
}

/// Beta of a vs b (slope of OLS regression a_t = α + β·b_t + ε).
pub fn beta(a: &[f64], b: &[f64]) -> Option<f64> {
    let n = a.len().min(b.len());
    if n < 2 { return None; }
    let mean_a = a[..n].iter().sum::<f64>() / n as f64;
    let mean_b = b[..n].iter().sum::<f64>() / n as f64;
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..n {
        num += (a[i] - mean_a) * (b[i] - mean_b);
        den += (b[i] - mean_b).powi(2);
    }
    if den == 0.0 { None } else { Some(num / den) }
}

#[derive(Debug, Clone, Serialize)]
pub struct PairAnalysis {
    pub correlation: f64,
    pub beta: f64,
    pub alpha: f64,
    pub mean_spread: f64,
    pub stdev_spread: f64,
    pub latest_spread: f64,
    pub latest_zscore: f64,
    pub samples: usize,
    pub spread_series: Vec<f64>,
    pub zscore_series: Vec<f64>,
}

/// Pair analysis on price series. Spread = a_price - (alpha + beta * b_price).
/// Z-score is normalized over the whole window.
pub fn pair_analysis(prices_a: &[f64], prices_b: &[f64]) -> Option<PairAnalysis> {
    let n = prices_a.len().min(prices_b.len());
    if n < 30 { return None; }
    let beta_v = beta(prices_a, prices_b)?;
    let mean_a = prices_a[..n].iter().sum::<f64>() / n as f64;
    let mean_b = prices_b[..n].iter().sum::<f64>() / n as f64;
    let alpha_v = mean_a - beta_v * mean_b;
    let mut spread = Vec::with_capacity(n);
    for i in 0..n {
        spread.push(prices_a[i] - (alpha_v + beta_v * prices_b[i]));
    }
    let mean_s = spread.iter().sum::<f64>() / n as f64;
    let var_s = spread.iter().map(|x| (x - mean_s).powi(2)).sum::<f64>() / n as f64;
    let sd_s = var_s.sqrt();
    let z: Vec<f64> = spread.iter().map(|x| if sd_s > 0.0 { (x - mean_s) / sd_s } else { 0.0 }).collect();
    let ra = log_returns(prices_a);
    let rb = log_returns(prices_b);
    let cor = pearson(&ra, &rb)?;
    Some(PairAnalysis {
        correlation: cor, beta: beta_v, alpha: alpha_v,
        mean_spread: mean_s, stdev_spread: sd_s,
        latest_spread: *spread.last().unwrap_or(&0.0),
        latest_zscore: *z.last().unwrap_or(&0.0),
        samples: n,
        spread_series: spread, zscore_series: z,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perfect_correlation_is_one() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        assert!((pearson(&a, &b).unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn negative_correlation() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![5.0, 4.0, 3.0, 2.0, 1.0];
        assert!((pearson(&a, &b).unwrap() + 1.0).abs() < 1e-9);
    }

    #[test]
    fn beta_of_2x_is_2() {
        let b: Vec<f64> = (1..=50).map(|x| x as f64).collect();
        let a: Vec<f64> = b.iter().map(|x| 2.0 * x + 3.0).collect();
        assert!((beta(&a, &b).unwrap() - 2.0).abs() < 1e-9);
    }
}
