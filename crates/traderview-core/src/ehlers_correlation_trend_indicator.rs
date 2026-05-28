//! Correlation Trend Indicator (CTI) — John Ehlers (TASC, May 2020).
//!
//! Pearson correlation of the closes over `period` bars against a
//! linear ramp 0..period-1. Equivalent to a normalized linear-regression
//! slope sign with stationary [-1, +1] range:
//!
//!   CTI_t = corr(closes_{t-period+1..t}, linear_ramp)
//!         = (N · Σ x_k · y_k - Σ x_k · Σ y_k)
//!           / sqrt((N · Σ x_k² - (Σ x_k)²) · (N · Σ y_k² - (Σ y_k)²))
//!
//! Range strictly within [-1, +1]:
//!   CTI ≈ +1 → near-perfect linear uptrend (trend)
//!   CTI ≈ -1 → near-perfect linear downtrend (trend)
//!   CTI ≈ 0  → choppy / non-linear movement (range)
//!
//! Used as a regime gauge: signals taken only when |CTI| > 0.7.
//!
//! Pure compute. Default period = 20. Companion to `chande_trend_index`,
//! `efficiency_ratio`, `hurst_exponent`.

pub fn compute(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period < 3 || n < period { return out; }
    if closes.iter().any(|x| !x.is_finite()) { return out; }
    let p_f = period as f64;
    // Precomputed x statistics.
    let sx: f64 = (0..period).map(|k| k as f64).sum();
    let sxx: f64 = (0..period).map(|k| (k as f64).powi(2)).sum();
    let denom_x = (p_f * sxx - sx * sx).sqrt();
    for (i, slot) in out.iter_mut().enumerate().skip(period - 1) {
        let win = &closes[i + 1 - period..=i];
        let sy: f64 = win.iter().sum();
        let syy: f64 = win.iter().map(|y| y * y).sum();
        let sxy: f64 = win.iter().enumerate().map(|(k, y)| k as f64 * y).sum();
        let var_y = p_f * syy - sy * sy;
        if var_y <= 0.0 { *slot = Some(0.0); continue; }
        let denom = denom_x * var_y.sqrt();
        if denom <= 0.0 { *slot = Some(0.0); continue; }
        *slot = Some((p_f * sxy - sx * sy) / denom);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let c = vec![100.0_f64; 30];
        assert!(compute(&c, 2).iter().all(|x| x.is_none()));
        assert!(compute(&c[..5], 20).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 30];
        c[5] = f64::NAN;
        assert!(compute(&c, 20).iter().all(|x| x.is_none()));
    }

    #[test]
    fn output_in_signed_unit_range() {
        let c: Vec<f64> = (0..200).map(|i| {
            100.0 + (i as f64 * 0.1).sin() * 5.0
        }).collect();
        let r = compute(&c, 20);
        for v in r.iter().flatten() {
            assert!((-1.0..=1.0).contains(v));
        }
    }

    #[test]
    fn perfect_uptrend_yields_plus_one() {
        let c: Vec<f64> = (0..50).map(|i| 100.0 + i as f64).collect();
        let r = compute(&c, 20);
        assert!((r[49].unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn perfect_downtrend_yields_minus_one() {
        let c: Vec<f64> = (0..50).map(|i| 200.0 - i as f64).collect();
        let r = compute(&c, 20);
        assert!((r[49].unwrap() + 1.0).abs() < 1e-9);
    }

    #[test]
    fn flat_market_yields_zero() {
        let c = vec![100.0_f64; 50];
        let r = compute(&c, 20);
        for v in r.iter().flatten() { assert!(v.abs() < 1e-9); }
    }

    #[test]
    fn output_length_matches_input() {
        let c = vec![100.0_f64; 30];
        assert_eq!(compute(&c, 20).len(), 30);
    }
}
