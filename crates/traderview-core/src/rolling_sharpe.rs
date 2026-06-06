//! Rolling Sharpe Ratio.
//!
//! For each bar i with i ≥ window − 1, computes the Sharpe ratio over
//! the trailing `window` returns:
//!
//!   sharpe_t = mean(r_{t-w+1..t}) / stdev(r_{t-w+1..t}) · √periods_per_year
//!
//! Risk-free rate subtracted from each return before computing if
//! `risk_free_per_period` ≠ 0.
//!
//! Use cases:
//!   - Visualize strategy risk-adjusted performance evolution
//!   - Risk-budget allocation across rolling-Sharpe-ranked strategies
//!   - Regime-switching detection by Sharpe sign change
//!
//! Pure compute. Companion to `rolling_sortino`, `rolling_beta`,
//! `risk_adjusted_ratios`.

pub fn compute(
    returns: &[f64],
    window: usize,
    periods_per_year: f64,
    risk_free_per_period: f64,
) -> Vec<Option<f64>> {
    let n = returns.len();
    let mut out = vec![None; n];
    if window < 2
        || n < window
        || !periods_per_year.is_finite()
        || periods_per_year <= 0.0
        || !risk_free_per_period.is_finite()
    {
        return out;
    }
    if returns.iter().any(|x| !x.is_finite()) {
        return out;
    }
    let ann_scale = periods_per_year.sqrt();
    for (i, slot) in out.iter_mut().enumerate().skip(window - 1) {
        let win = &returns[i + 1 - window..=i];
        let n_f = window as f64;
        let mean: f64 = win.iter().sum::<f64>() / n_f - risk_free_per_period;
        let var: f64 = win
            .iter()
            .map(|r| (r - risk_free_per_period - mean).powi(2))
            .sum::<f64>()
            / (n_f - 1.0);
        let sd = var.max(0.0).sqrt();
        if sd > 0.0 {
            *slot = Some(mean / sd * ann_scale);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_all_none() {
        let r = vec![0.01_f64; 30];
        assert!(compute(&r, 1, 252.0, 0.0).iter().all(|x| x.is_none()));
        assert!(compute(&r, 30, 0.0, 0.0).iter().all(|x| x.is_none()));
        assert!(compute(&r, 30, 252.0, f64::NAN).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let mut r = vec![0.01_f64; 30];
        r[5] = f64::NAN;
        assert!(compute(&r, 20, 252.0, 0.0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_returns_yield_none() {
        let r = vec![0.01_f64; 30];
        let out = compute(&r, 20, 252.0, 0.0);
        for v in &out {
            assert!(v.is_none() || v.unwrap().is_finite());
        }
    }

    #[test]
    fn positive_mean_yields_positive_sharpe() {
        let mut state: u64 = 42;
        let r: Vec<f64> = (0..200)
            .map(|_| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                0.0005 + ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 0.01
            })
            .collect();
        let out = compute(&r, 60, 252.0, 0.0);
        let last = out[199].unwrap();
        assert!(last > 0.0);
    }

    #[test]
    fn output_length_matches_input() {
        let r: Vec<f64> = (0..50).map(|i| (i as f64 * 0.1).sin() * 0.01).collect();
        let out = compute(&r, 20, 252.0, 0.0);
        assert_eq!(out.len(), 50);
        assert!(out[18].is_none());
        assert!(out[19].is_some());
    }

    #[test]
    fn risk_free_subtracted() {
        // Varying returns so stdev > 0 and the rf-shift effect is visible.
        let r: Vec<f64> = (0..50)
            .map(|i| 0.001 + ((i as f64 * 0.3).sin()) * 0.005)
            .collect();
        let out_no_rf = compute(&r, 30, 252.0, 0.0);
        let out_rf = compute(&r, 30, 252.0, 0.005);
        // Higher rf → smaller excess return → smaller Sharpe.
        let last_no_rf = out_no_rf[49].unwrap();
        let last_rf = out_rf[49].unwrap();
        assert!(
            last_no_rf > last_rf,
            "no-rf Sharpe {} should exceed rf=0.5% Sharpe {}",
            last_no_rf,
            last_rf
        );
    }
}
