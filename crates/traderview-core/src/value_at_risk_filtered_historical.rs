//! Filtered Historical Simulation VaR — vol-scale historical residuals
//! by current conditional volatility before quantiling (Hull-White 1998).
//!
//! Steps:
//!   1. Fit an EWMA variance σ²_t = λ·σ²_{t-1} + (1-λ)·r²_{t-1}
//!      across the return series (RiskMetrics, λ = 0.94 default).
//!   2. Standardize residuals: z_t = r_t / σ_t.
//!   3. Rescale residuals by the current (last-bar) σ to produce
//!      forward-looking returns: r̃_t = z_t · σ_T.
//!   4. VaR = -quantile_{1-c}(r̃) ; ES = -mean(r̃ ≤ VaR-quantile).
//!
//! This injects current vol regime into the historical distribution —
//! plain HS-VaR can under- or over-estimate in regime shifts; FHS-VaR
//! adapts each bar.
//!
//! Pure compute. Companion to `value_at_risk_historical`,
//! `garch_1_1`, `expected_shortfall`.

#[derive(Debug)]
pub struct Report {
    pub var: f64,
    pub expected_shortfall: f64,
    pub current_sigma: f64,
    pub n: usize,
}

pub fn compute(returns: &[f64], confidence: f64, lambda: f64) -> Option<Report> {
    let n = returns.len();
    if n < 20 || !confidence.is_finite() || !lambda.is_finite() {
        return None;
    }
    if !(0.5..1.0).contains(&confidence) {
        return None;
    }
    if !(0.5..1.0).contains(&lambda) {
        return None;
    }
    if returns.iter().any(|x| !x.is_finite()) {
        return None;
    }
    // Seed variance: variance of first 10 returns.
    let seed_window = 10.min(n);
    let mu = returns[..seed_window].iter().sum::<f64>() / seed_window as f64;
    let mut var = returns[..seed_window]
        .iter()
        .map(|r| (r - mu).powi(2))
        .sum::<f64>()
        / seed_window as f64;
    if var < 1e-18 {
        var = 1e-18;
    }
    let mut sigmas = Vec::with_capacity(n);
    sigmas.push(var.sqrt());
    for r in returns.iter().take(n - 1) {
        var = lambda * var + (1.0 - lambda) * r * r;
        if var < 1e-18 {
            var = 1e-18;
        }
        sigmas.push(var.sqrt());
    }
    let current_sigma = sigmas[n - 1];
    let mut scaled: Vec<f64> = returns
        .iter()
        .zip(sigmas.iter())
        .map(|(r, s)| (r / s) * current_sigma)
        .collect();
    scaled.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let alpha = 1.0 - confidence;
    let h = alpha * (n as f64 - 1.0);
    let lo = h.floor() as usize;
    let hi = (lo + 1).min(n - 1);
    let frac = h - lo as f64;
    let q = scaled[lo] * (1.0 - frac) + scaled[hi] * frac;
    let var_out = -q;
    let tail: Vec<f64> = scaled.iter().copied().filter(|&r| r <= q).collect();
    let es = if tail.is_empty() {
        var_out
    } else {
        -(tail.iter().sum::<f64>() / tail.len() as f64)
    };
    Some(Report {
        var: var_out,
        expected_shortfall: es,
        current_sigma,
        n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let r = vec![0.01_f64; 100];
        assert!(compute(&r, 0.4, 0.94).is_none());
        assert!(compute(&r, 0.95, 0.3).is_none());
        assert!(compute(&r, 0.95, 1.0).is_none());
        assert!(compute(&r[..5], 0.95, 0.94).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut r = vec![0.01_f64; 100];
        r[5] = f64::NAN;
        assert!(compute(&r, 0.95, 0.94).is_none());
    }

    #[test]
    fn output_is_finite_for_normal_returns() {
        let r: Vec<f64> = (0..200).map(|i| ((i as f64 * 0.3).sin()) * 0.01).collect();
        let rep = compute(&r, 0.95, 0.94).unwrap();
        assert!(rep.var.is_finite());
        assert!(rep.expected_shortfall.is_finite());
        assert!(rep.current_sigma > 0.0);
    }

    #[test]
    fn es_is_at_least_var() {
        let r: Vec<f64> = (0..500).map(|i| ((i as f64 * 0.17).sin()) * 0.02).collect();
        let rep = compute(&r, 0.95, 0.94).unwrap();
        assert!(rep.expected_shortfall >= rep.var - 1e-9);
    }

    #[test]
    fn higher_confidence_yields_higher_var() {
        let r: Vec<f64> = (0..500).map(|i| ((i as f64 * 0.17).sin()) * 0.02).collect();
        let r95 = compute(&r, 0.95, 0.94).unwrap();
        let r99 = compute(&r, 0.99, 0.94).unwrap();
        assert!(r99.var >= r95.var);
    }

    #[test]
    fn vol_spike_increases_var() {
        // 100 calm returns, then 100 high-vol — current_sigma should be high.
        let mut r: Vec<f64> = vec![0.001; 100];
        for i in 0..100 {
            r.push(((i as f64 * 0.5).sin()) * 0.05);
        }
        let rep = compute(&r, 0.95, 0.94).unwrap();
        assert!(rep.current_sigma > 0.005);
    }
}
