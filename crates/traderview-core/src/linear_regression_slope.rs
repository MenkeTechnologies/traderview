//! Linear Regression Slope (LRS).
//!
//! Per-bar slope of the OLS line fit to the last `period` closes:
//!
//!   slope_t = Σ (x_k - x̄) · (y_k - ȳ) / Σ (x_k - x̄)²
//!     where x_k = 0..period-1 and y_k = closes[t-period+1..=t]
//!
//! Companion-only metric: slope > 0 = bull bias, slope < 0 = bear
//! bias. Magnitude is in units of price per bar; normalize via the
//! standard error for cross-instrument comparison (use
//! `linear_regression_channel` for that).
//!
//! Pure compute. Default period = 14. Companion to
//! `linear_regression_channel`, `standard_error_bands`,
//! `ehlers_correlation_trend_indicator`.

pub fn compute(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period < 3 || n < period { return out; }
    if closes.iter().any(|x| !x.is_finite()) { return out; }
    let p_f = period as f64;
    let x_mean = (p_f - 1.0) / 2.0;
    let x_var: f64 = (0..period).map(|i| {
        let dx = i as f64 - x_mean;
        dx * dx
    }).sum();
    if x_var <= 0.0 { return out; }
    for (i, slot) in out.iter_mut().enumerate().skip(period - 1) {
        let win = &closes[i + 1 - period..=i];
        let y_mean: f64 = win.iter().sum::<f64>() / p_f;
        let mut sxy = 0.0_f64;
        for (k, &y) in win.iter().enumerate() {
            sxy += (k as f64 - x_mean) * (y - y_mean);
        }
        *slot = Some(sxy / x_var);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let c = vec![100.0_f64; 50];
        assert!(compute(&c, 2).iter().all(|x| x.is_none()));
        assert!(compute(&c[..5], 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 50];
        c[5] = f64::NAN;
        assert!(compute(&c, 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_slope() {
        let c = vec![100.0_f64; 50];
        let r = compute(&c, 14);
        for v in r.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn perfect_uptrend_yields_unit_slope() {
        let c: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let r = compute(&c, 14);
        assert!((r[49].unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn downtrend_yields_negative_slope() {
        let c: Vec<f64> = (0..50).map(|i| 100.0 - i as f64).collect();
        let r = compute(&c, 14);
        assert!((r[49].unwrap() + 1.0).abs() < 1e-9);
    }

    #[test]
    fn scaled_trend_scales_slope() {
        let c: Vec<f64> = (0..50).map(|i| 2.0 * i as f64).collect();
        let r = compute(&c, 14);
        assert!((r[49].unwrap() - 2.0).abs() < 1e-9);
    }

    #[test]
    fn output_length_matches_input() {
        let c = vec![100.0_f64; 50];
        assert_eq!(compute(&c, 14).len(), 50);
    }
}
