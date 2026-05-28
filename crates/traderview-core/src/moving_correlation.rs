//! Moving Correlation — rolling Pearson correlation between two series
//! over a sliding window.
//!
//!   corr_t = Σ (x_k - x̄) · (y_k - ȳ)
//!            / sqrt(Σ (x_k - x̄)² · Σ (y_k - ȳ)²)
//!
//! Useful for tracking how the relationship between two assets evolves:
//!   - SPY vs XLF (sector beta)
//!   - SPY vs VIX (typically negative)
//!   - Gold vs USD (typically negative)
//!
//! Output in [-1, +1].
//!
//! Pure compute. Default period = 30. Companion to `realized_correlation`,
//! `spearman_correlation`, `distance_correlation`, `rolling_beta`.

pub fn compute(series_x: &[f64], series_y: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series_x.len();
    let mut out = vec![None; n];
    if period < 3 || n < period || series_y.len() != n { return out; }
    if series_x.iter().chain(series_y.iter()).any(|v| !v.is_finite()) { return out; }
    let p_f = period as f64;
    for (i, slot) in out.iter_mut().enumerate().skip(period - 1) {
        let x_win = &series_x[i + 1 - period..=i];
        let y_win = &series_y[i + 1 - period..=i];
        let x_mean: f64 = x_win.iter().sum::<f64>() / p_f;
        let y_mean: f64 = y_win.iter().sum::<f64>() / p_f;
        let mut sxy = 0.0_f64;
        let mut sxx = 0.0_f64;
        let mut syy = 0.0_f64;
        for k in 0..period {
            let dx = x_win[k] - x_mean;
            let dy = y_win[k] - y_mean;
            sxy += dx * dy;
            sxx += dx * dx;
            syy += dy * dy;
        }
        let denom = (sxx * syy).sqrt();
        if denom > 0.0 {
            *slot = Some(sxy / denom);
        } else {
            *slot = Some(0.0);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let x = vec![1.0_f64; 50];
        let y = vec![1.0_f64; 50];
        assert!(compute(&x, &y, 2).iter().all(|v| v.is_none()));
        assert!(compute(&x, &y[..10], 30).iter().all(|v| v.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut x = vec![1.0_f64; 50];
        x[5] = f64::NAN;
        let y = vec![1.0_f64; 50];
        assert!(compute(&x, &y, 30).iter().all(|v| v.is_none()));
    }

    #[test]
    fn perfect_positive_correlation_yields_unity() {
        let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let y: Vec<f64> = (0..50).map(|i| 2.0 * i as f64 + 5.0).collect();
        let r = compute(&x, &y, 30);
        let last = r[49].unwrap();
        assert!((last - 1.0).abs() < 1e-9);
    }

    #[test]
    fn perfect_negative_correlation_yields_minus_unity() {
        let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let y: Vec<f64> = (0..50).map(|i| -i as f64).collect();
        let r = compute(&x, &y, 30);
        assert!((r[49].unwrap() + 1.0).abs() < 1e-9);
    }

    #[test]
    fn flat_series_yields_zero() {
        // Zero variance → defined as 0.
        let x = vec![100.0_f64; 50];
        let y: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let r = compute(&x, &y, 30);
        for v in r.iter().flatten() { assert!(v.abs() < 1e-9); }
    }

    #[test]
    fn output_in_signed_unit_range() {
        let mut state: u64 = 42;
        let x: Vec<f64> = (0..200).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let r = (state >> 32) as u32 as f64 / u32::MAX as f64;
            (r - 0.5) * 10.0
        }).collect();
        let mut state2: u64 = 99;
        let y: Vec<f64> = (0..200).map(|_| {
            state2 = state2.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let r = (state2 >> 32) as u32 as f64 / u32::MAX as f64;
            (r - 0.5) * 10.0
        }).collect();
        let r = compute(&x, &y, 30);
        for v in r.iter().flatten() {
            assert!((-1.0..=1.0).contains(v));
        }
    }
}
