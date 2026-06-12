//! Historical-Simulation Value at Risk — distribution-free VaR/ES from
//! the empirical quantile of past returns.
//!
//! Given a series of historical log or arithmetic returns, sorts them
//! ascending and reads off the (1-α) quantile via Type-7 interpolation.
//! Expected Shortfall = mean of returns ≤ the VaR threshold.
//!
//! VaR and ES are reported as positive magnitudes of loss.
//!
//! Pure compute. Companion to `monte_carlo_var`, `evt_value_at_risk`,
//! `cornish_fisher_var`, `expected_shortfall`.

#[derive(Debug, Clone, serde::Serialize)]
pub struct Report {
    pub var: f64,
    pub expected_shortfall: f64,
    pub n: usize,
}

pub fn compute(returns: &[f64], confidence: f64) -> Option<Report> {
    let n = returns.len();
    if n < 2 || !confidence.is_finite() {
        return None;
    }
    if !(0.5..1.0).contains(&confidence) {
        return None;
    }
    if returns.iter().any(|x| !x.is_finite()) {
        return None;
    }
    let mut sorted: Vec<f64> = returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let alpha = 1.0 - confidence;
    let h = alpha * (n as f64 - 1.0);
    let lo = h.floor() as usize;
    let hi = (lo + 1).min(n - 1);
    let frac = h - lo as f64;
    let quantile = sorted[lo] * (1.0 - frac) + sorted[hi] * frac;
    let var = -quantile;
    let tail: Vec<f64> = sorted.iter().copied().filter(|&r| r <= quantile).collect();
    let es = if tail.is_empty() {
        var
    } else {
        -(tail.iter().sum::<f64>() / tail.len() as f64)
    };
    Some(Report {
        var,
        expected_shortfall: es,
        n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        let r = vec![0.01_f64; 10];
        assert!(compute(&[], 0.95).is_none());
        assert!(compute(&r, 0.4).is_none());
        assert!(compute(&r, 1.0).is_none());
        assert!(compute(&r, f64::NAN).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let mut r = vec![0.01_f64; 100];
        r[5] = f64::NAN;
        assert!(compute(&r, 0.95).is_none());
    }

    #[test]
    fn symmetric_returns_yield_finite_var() {
        // Symmetric distribution around 0 → 95% VaR is some positive #.
        let r: Vec<f64> = (-100_i32..=100).map(|x| x as f64 / 100.0).collect();
        let rep = compute(&r, 0.95).unwrap();
        assert!(rep.var > 0.0);
        assert!(rep.expected_shortfall >= rep.var - 1e-9);
    }

    #[test]
    fn es_is_at_least_var() {
        let r: Vec<f64> = (0..1000)
            .map(|i| {
                let x = i as f64 / 1000.0;
                // 99% small, 1% large losses
                if x < 0.99 {
                    0.0
                } else {
                    -0.5
                }
            })
            .collect();
        let rep = compute(&r, 0.95).unwrap();
        assert!(rep.expected_shortfall >= rep.var);
    }

    #[test]
    fn higher_confidence_yields_higher_var() {
        let r: Vec<f64> = (-500_i32..=500).map(|x| x as f64 / 1000.0).collect();
        let r95 = compute(&r, 0.95).unwrap();
        let r99 = compute(&r, 0.99).unwrap();
        assert!(r99.var >= r95.var);
    }

    #[test]
    fn constant_returns_yield_zero_var() {
        let r = vec![0.005_f64; 200];
        let rep = compute(&r, 0.95).unwrap();
        // Constant +0.5%: quantile = 0.005, so VaR = -0.005.
        assert!((rep.var + 0.005).abs() < 1e-9);
    }
}
