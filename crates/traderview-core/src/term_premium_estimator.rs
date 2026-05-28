//! Term Premium Estimator — extract the term premium from a long-tenor
//! yield using a simplified Adrian-Crump-Moench (ACM) approach.
//!
//! The term premium is the long-yield minus the expected average of
//! short-rates over the same horizon:
//!
//!   term_premium = long_yield - E[avg short rates over tenor]
//!
//! Here E[avg short rates] is approximated by the AVERAGE short rate
//! over the last `lookback` periods (consistent with the expectations-
//! hypothesis approximation). A more rigorous ACM model fits an
//! affine no-arbitrage 5-factor regression; this module is the simple
//! linear-decomposition version sufficient for chart overlays.
//!
//! Returns the per-bar term-premium estimate in basis points.
//!
//! Pure compute. Default lookback = 60.
//! Companion to `term_spread`, `yield_curve_bootstrap`,
//! `nelson_siegel`, `breakeven_inflation`.

pub fn compute(
    long_yield_pct: &[f64],
    short_yield_pct: &[f64],
    lookback: usize,
) -> Vec<Option<f64>> {
    let n = long_yield_pct.len();
    let mut out = vec![None; n];
    if lookback < 2 || n < lookback || short_yield_pct.len() != n { return out; }
    if long_yield_pct.iter().chain(short_yield_pct.iter())
        .any(|x| !x.is_finite()) {
        return out;
    }
    let p_f = lookback as f64;
    for (i, slot) in out.iter_mut().enumerate().skip(lookback - 1) {
        let win = &short_yield_pct[i + 1 - lookback..=i];
        let expected_avg_short: f64 = win.iter().sum::<f64>() / p_f;
        let term_premium = (long_yield_pct[i] - expected_avg_short) * 100.0;
        *slot = Some(term_premium);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let l = vec![3.0_f64; 100];
        let s = vec![2.0_f64; 100];
        assert!(compute(&l, &s, 1).iter().all(|x| x.is_none()));
        assert!(compute(&l[..10], &s[..10], 60).iter().all(|x| x.is_none()));
        assert!(compute(&l, &s[..50], 60).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut l = vec![3.0_f64; 100];
        l[5] = f64::NAN;
        let s = vec![2.0_f64; 100];
        assert!(compute(&l, &s, 60).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_curves_yield_constant_term_premium() {
        // Long = 3.0%, short = 2.0% → term premium = 1.0pp = 100 bps.
        let l = vec![3.0_f64; 100];
        let s = vec![2.0_f64; 100];
        let r = compute(&l, &s, 60);
        for v in r.iter().flatten() {
            assert!((v - 100.0).abs() < 1e-9);
        }
    }

    #[test]
    fn rising_short_rates_shrink_term_premium() {
        let l = vec![5.0_f64; 100];
        let s: Vec<f64> = (0..100).map(|i| 1.0 + i as f64 * 0.05).collect();
        let r = compute(&l, &s, 60);
        let early = r[60].unwrap();
        let late = r[99].unwrap();
        // Later short rates higher → expected avg higher → premium lower.
        assert!(late < early);
    }

    #[test]
    fn long_yield_below_avg_short_yields_negative_premium() {
        let l = vec![1.0_f64; 100];
        let s = vec![3.0_f64; 100];
        let r = compute(&l, &s, 60);
        for v in r.iter().flatten() {
            assert!(*v < 0.0);
        }
    }

    #[test]
    fn output_length_matches_input() {
        let l = vec![3.0_f64; 100];
        let s = vec![2.0_f64; 100];
        assert_eq!(compute(&l, &s, 60).len(), 100);
    }
}
