//! Rolling Quantile — empirical quantile of any input series over a
//! sliding window.
//!
//! Per bar, sorts the window values and returns the linear-interpolation
//! (Type 7, numpy-default) quantile at `q ∈ [0, 1]`.
//!
//! Useful as a generic primitive for many indicators: rolling-min
//! (q=0), rolling-max (q=1), rolling-median (q=0.5), rolling 90th
//! percentile, etc.
//!
//! Pure compute. Companion to `rolling_zscore`, `quantile_regression`,
//! `empirical_distribution_function`.

pub fn compute(series: &[f64], period: usize, q: f64) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period < 1 || n < period || !q.is_finite() || !(0.0..=1.0).contains(&q) {
        return out;
    }
    if series.iter().any(|x| !x.is_finite()) {
        return out;
    }
    for (i, slot) in out.iter_mut().enumerate().skip(period - 1) {
        let win = &series[i + 1 - period..=i];
        let mut sorted: Vec<f64> = win.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        *slot = Some(quantile_type7(&sorted, q));
    }
    out
}

fn quantile_type7(sorted: &[f64], q: f64) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return f64::NAN;
    }
    if n == 1 {
        return sorted[0];
    }
    let h = q * (n as f64 - 1.0);
    let lo = h.floor() as usize;
    let hi = (lo + 1).min(n - 1);
    let frac = h - lo as f64;
    sorted[lo] * (1.0 - frac) + sorted[hi] * frac
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let s = vec![1.0_f64; 50];
        assert!(compute(&s, 0, 0.5).iter().all(|x| x.is_none()));
        assert!(compute(&s, 10, -0.1).iter().all(|x| x.is_none()));
        assert!(compute(&s, 10, 1.1).iter().all(|x| x.is_none()));
        assert!(compute(&s[..5], 10, 0.5).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut s = vec![1.0_f64; 50];
        s[5] = f64::NAN;
        assert!(compute(&s, 10, 0.5).iter().all(|x| x.is_none()));
    }

    #[test]
    fn q_zero_yields_rolling_min() {
        let s: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let r = compute(&s, 10, 0.0);
        // Bar 9: window 0..=9, min = 0.
        assert!((r[9].unwrap()).abs() < 1e-9);
        // Bar 19: window 10..=19, min = 10.
        assert!((r[19].unwrap() - 10.0).abs() < 1e-9);
    }

    #[test]
    fn q_one_yields_rolling_max() {
        let s: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let r = compute(&s, 10, 1.0);
        assert!((r[9].unwrap() - 9.0).abs() < 1e-9);
        assert!((r[19].unwrap() - 19.0).abs() < 1e-9);
    }

    #[test]
    fn q_half_yields_rolling_median() {
        // 0..10 median = 4.5.
        let s: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let r = compute(&s, 10, 0.5);
        assert!((r[9].unwrap() - 4.5).abs() < 1e-9);
    }

    #[test]
    fn constant_series_yields_constant_quantile() {
        let s = vec![42.0_f64; 30];
        let r = compute(&s, 10, 0.7);
        for v in r.iter().flatten() {
            assert!((v - 42.0).abs() < 1e-9);
        }
    }

    #[test]
    fn output_length_matches_input() {
        let s = vec![1.0_f64; 50];
        assert_eq!(compute(&s, 10, 0.5).len(), 50);
    }
}
