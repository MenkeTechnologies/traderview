//! Linear Regression Curve (LRC).
//!
//! Per-bar endpoint of an OLS line fit to the last `period` closes —
//! the full series of values that `linear_regression_channel` reports
//! as its `regression` field, but as a standalone module for clients
//! that only need the regression line (no slope, no R², no bands).
//!
//! Same math as `linear_regression_slope` for the slope, plus an
//! intercept term:
//!
//!   slope_t     = Σ (x_k - x̄) · (y_k - ȳ) / Σ (x_k - x̄)²
//!   intercept_t = ȳ - slope · x̄
//!   LRC_t       = intercept_t + slope_t · (period - 1)
//!
//! Pure compute. Default period = 14. Companion to
//! `linear_regression_channel`, `linear_regression_slope`,
//! `time_series_forecast` (if shipped).

pub fn compute(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period < 3 || n < period {
        return out;
    }
    if closes.iter().any(|x| !x.is_finite()) {
        return out;
    }
    let p_f = period as f64;
    let x_mean = (p_f - 1.0) / 2.0;
    let x_var: f64 = (0..period)
        .map(|i| {
            let dx = i as f64 - x_mean;
            dx * dx
        })
        .sum();
    if x_var <= 0.0 {
        return out;
    }
    for (i, slot) in out.iter_mut().enumerate().skip(period - 1) {
        let win = &closes[i + 1 - period..=i];
        let y_mean: f64 = win.iter().sum::<f64>() / p_f;
        let mut sxy = 0.0_f64;
        for (k, &y) in win.iter().enumerate() {
            sxy += (k as f64 - x_mean) * (y - y_mean);
        }
        let slope = sxy / x_var;
        let intercept = y_mean - slope * x_mean;
        *slot = Some(intercept + slope * (p_f - 1.0));
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
    fn flat_market_lrc_equals_input() {
        let c = vec![100.0_f64; 50];
        let r = compute(&c, 14);
        for v in r.iter().flatten() {
            assert!((v - 100.0).abs() < 1e-9);
        }
    }

    #[test]
    fn perfect_uptrend_lrc_equals_current_close() {
        // Perfect linear fit → LRC at the window's last bar = closes[i].
        let c: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let r = compute(&c, 14);
        for i in 13..50 {
            assert!(
                (r[i].unwrap() - c[i]).abs() < 1e-9,
                "at i={i}: lrc {} != close {}",
                r[i].unwrap(),
                c[i]
            );
        }
    }

    #[test]
    fn noisy_uptrend_lrc_near_input_range() {
        let mut state: u64 = 42;
        let c: Vec<f64> = (0..100)
            .map(|i| {
                state = state
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let r = (state >> 32) as u32 as f64 / u32::MAX as f64;
                100.0 + i as f64 * 0.5 + (r - 0.5) * 4.0
            })
            .collect();
        let r = compute(&c, 14);
        let mn = c.iter().cloned().fold(f64::INFINITY, f64::min);
        let mx = c.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        for v in r.iter().flatten() {
            assert!(*v >= mn - 5.0 && *v <= mx + 5.0);
        }
    }

    #[test]
    fn output_length_matches_input() {
        let c = vec![100.0_f64; 50];
        assert_eq!(compute(&c, 14).len(), 50);
    }
}
