//! Amihud (2002) Illiquidity Ratio — `|r_t| / dollar_volume_t`.
//!
//! Measures price impact per dollar traded. Scaled per million dollars
//! to get reasonable magnitudes (canonical Amihud paper convention).
//!
//!   illiq_t = |return_t| / dollar_volume_t · 10^6
//!   amihud  = mean(illiq) over `period` bars
//!
//! Higher = less liquid. Cross-sectional ranking is the standard use:
//! a stock with Amihud illiquidity in the top decile commands a
//! significant illiquidity premium in long-run returns.
//!
//! Pure compute.

pub fn compute(returns: &[f64], dollar_volumes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = returns.len();
    let mut out = vec![None; n];
    if period == 0 || returns.len() != dollar_volumes.len() || n < period {
        return out;
    }
    // Per-bar illiq (None where dollar volume is 0 or non-finite).
    let mut per_bar = vec![None::<f64>; n];
    for i in 0..n {
        if !returns[i].is_finite() || !dollar_volumes[i].is_finite() || dollar_volumes[i] <= 0.0 {
            continue;
        }
        let v = returns[i].abs() / dollar_volumes[i] * 1_000_000.0;
        if v.is_finite() {
            per_bar[i] = Some(v);
        }
    }
    // Rolling mean over `period`, treating None as a skip (don't fail the window).
    for i in (period - 1)..n {
        let win = &per_bar[i + 1 - period..=i];
        let valid: Vec<f64> = win.iter().filter_map(|x| *x).collect();
        if !valid.is_empty() {
            let mean = valid.iter().sum::<f64>() / valid.len() as f64;
            if mean.is_finite() {
                out[i] = Some(mean);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], &[], 14).is_empty());
    }

    #[test]
    fn length_mismatch_returns_all_none() {
        let r = vec![0.01; 30];
        let v = vec![1_000_000.0; 15];
        assert!(compute(&r, &v, 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn period_zero_returns_all_none() {
        let r = vec![0.01; 30];
        let v = vec![1_000_000.0; 30];
        assert!(compute(&r, &v, 0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn high_volume_yields_low_illiquidity() {
        // |0.01| / 100M · 1M = 0.0001
        let r = vec![0.01; 30];
        let v = vec![100_000_000.0; 30];
        let out = compute(&r, &v, 14);
        let val = out[29].expect("populated");
        assert!((val - 0.0001).abs() < 1e-9);
    }

    #[test]
    fn low_volume_yields_high_illiquidity() {
        // |0.01| / 10k · 1M = 1.0
        let r = vec![0.01; 30];
        let v = vec![10_000.0; 30];
        let out = compute(&r, &v, 14);
        let val = out[29].expect("populated");
        assert!((val - 1.0).abs() < 1e-9);
    }

    #[test]
    fn zero_volume_skipped_safely() {
        let r = vec![0.01; 30];
        let mut v = vec![1_000_000.0; 30];
        v[5] = 0.0;
        let out = compute(&r, &v, 14);
        // Should still populate via other valid bars in the window.
        assert!(out[19].is_some());
    }

    #[test]
    fn nan_inputs_filtered_without_panic() {
        let mut r = vec![0.01; 30];
        r[10] = f64::NAN;
        let mut v = vec![1_000_000.0; 30];
        v[15] = f64::NAN;
        let out = compute(&r, &v, 14);
        // No panic; populated.
        assert!(out[29].is_some());
    }

    #[test]
    fn all_zero_volume_window_returns_none() {
        let r = vec![0.01; 30];
        let v = vec![0.0; 30];
        let out = compute(&r, &v, 14);
        assert!(out[29].is_none());
    }

    #[test]
    fn huge_period_no_panic() {
        let r = vec![0.01; 5];
        let v = vec![1_000_000.0; 5];
        assert!(compute(&r, &v, usize::MAX).iter().all(|x| x.is_none()));
    }
}
