//! Rolling Sortino Ratio.
//!
//! Like rolling Sharpe but penalizes only DOWNSIDE volatility:
//!
//!   sortino_t = (mean(r) − target) / downside_std(r) · √periods_per_year
//!   downside_std = √(mean(min(0, r − target)²))
//!
//! Sortino is preferred when returns are asymmetric (skewed) since
//! Sharpe penalizes upside vol equally with downside vol.
//!
//! Pure compute. Companion to `rolling_sharpe`, `rolling_beta`,
//! `lower_partial_moments`, `risk_adjusted_ratios`.

pub fn compute(
    returns: &[f64],
    window: usize,
    periods_per_year: f64,
    minimum_acceptable_return: f64,
) -> Vec<Option<f64>> {
    let n = returns.len();
    let mut out = vec![None; n];
    if window < 2
        || n < window
        || !periods_per_year.is_finite()
        || periods_per_year <= 0.0
        || !minimum_acceptable_return.is_finite()
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
        let mean: f64 = win.iter().sum::<f64>() / n_f;
        let excess = mean - minimum_acceptable_return;
        let downside_var: f64 = win
            .iter()
            .map(|r| {
                let d = (r - minimum_acceptable_return).min(0.0);
                d * d
            })
            .sum::<f64>()
            / n_f;
        let downside_sd = downside_var.max(0.0).sqrt();
        if downside_sd > 0.0 {
            *slot = Some(excess / downside_sd * ann_scale);
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
    fn all_upside_yields_none_sortino() {
        // All returns ≥ MAR → downside_std = 0 → None.
        let r = vec![0.01_f64; 30];
        let out = compute(&r, 20, 252.0, 0.0);
        for v in out.iter().skip(19) {
            assert!(v.is_none(), "all-upside should yield None, got {v:?}");
        }
    }

    #[test]
    fn mixed_returns_yield_finite_sortino() {
        let r: Vec<f64> = (0..50)
            .map(|i| if i % 3 == 0 { -0.02 } else { 0.01 })
            .collect();
        let out = compute(&r, 20, 252.0, 0.0);
        let last = out[49].unwrap();
        assert!(last.is_finite());
    }

    #[test]
    fn output_length_matches_input() {
        let r: Vec<f64> = (0..50).map(|i| (i as f64 * 0.3).sin() * 0.01).collect();
        let out = compute(&r, 20, 252.0, 0.0);
        assert_eq!(out.len(), 50);
        assert!(out[18].is_none());
    }
}
