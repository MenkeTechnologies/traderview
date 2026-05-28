//! Rolling Linear Regression R² — coefficient of determination over a
//! sliding window. Reflects how linearly the closes track over the
//! window (1 = perfect line, 0 = no linear relationship).
//!
//!   y_mean = mean(closes over period)
//!   slope, intercept = OLS fit
//!   sse = Σ (y - y_hat)²
//!   sst = Σ (y - y_mean)²
//!   r_squared = 1 - sse / sst   if sst > 0 else 1.0
//!
//! Used as a trend-quality filter: only act on signals when R² is
//! high (price actually trending in a line, not chopping). Common
//! threshold: R² > 0.7.
//!
//! Pure compute. Default period = 14. Companion to `linear_regression_slope`,
//! `linear_regression_channel`, `ehlers_correlation_trend_indicator`.

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
        let mut sst = 0.0_f64;
        for (k, &y) in win.iter().enumerate() {
            let dx = k as f64 - x_mean;
            let dy = y - y_mean;
            sxy += dx * dy;
            sst += dy * dy;
        }
        let slope = sxy / x_var;
        let intercept = y_mean - slope * x_mean;
        let mut sse = 0.0_f64;
        for (k, &y) in win.iter().enumerate() {
            let y_hat = intercept + slope * k as f64;
            sse += (y - y_hat).powi(2);
        }
        *slot = Some(if sst > 0.0 { 1.0 - sse / sst } else { 1.0 });
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
    fn perfect_linear_trend_yields_unity() {
        let c: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let r = compute(&c, 14);
        for v in r.iter().flatten() {
            assert!((v - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn flat_market_yields_unity() {
        // Zero variance → R² conventionally 1 (no error, no variance to explain).
        let c = vec![100.0_f64; 50];
        let r = compute(&c, 14);
        for v in r.iter().flatten() {
            assert!((v - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn noisy_data_yields_lower_r_squared() {
        let mut state: u64 = 42;
        let c: Vec<f64> = (0..200).map(|i| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let r = (state >> 32) as u32 as f64 / u32::MAX as f64;
            100.0 + i as f64 * 0.1 + (r - 0.5) * 20.0
        }).collect();
        let r = compute(&c, 20);
        let any_below_high = r.iter().flatten().any(|v| *v < 0.5);
        assert!(any_below_high,
            "noisy data should produce some R² < 0.5");
    }

    #[test]
    fn output_length_matches_input() {
        let c = vec![100.0_f64; 50];
        assert_eq!(compute(&c, 14).len(), 50);
    }
}
