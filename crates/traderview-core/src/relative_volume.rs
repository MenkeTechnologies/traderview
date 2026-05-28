//! Relative Volume (RVOL) — current volume / average past volume.
//!
//!   rvol_t = volume_t / SMA(volume, period, ending at t−1)
//!
//! 1.0 = average. Day-trader convention: RVOL ≥ 2.0 confirms volume
//! breakouts; ≥ 5.0 = "in play" (news, earnings, halt-resume). Many
//! gap-and-go strategies require RVOL ≥ 5.0 at the open before triggering.
//!
//! Pure compute. Past-volume SMA EXCLUDES the current bar (lookback
//! avoids look-ahead bias).

pub fn compute(volumes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = volumes.len();
    let mut out = vec![None; n];
    if period == 0 || n <= period {
        return out;
    }
    // SMA(volume, period) ending at i-1.
    let mut sum = 0.0;
    let mut valid = 0;
    for v in volumes.iter().take(period) {
        if v.is_finite() && *v >= 0.0 {
            sum += *v;
            valid += 1;
        }
    }
    for i in period..n {
        if valid == period && volumes[i].is_finite() && volumes[i] >= 0.0 {
            let avg = sum / period as f64;
            if avg > 0.0 {
                let r = volumes[i] / avg;
                if r.is_finite() {
                    out[i] = Some(r);
                }
            }
        }
        // Slide window forward.
        if volumes[i - period].is_finite() && volumes[i - period] >= 0.0 {
            sum -= volumes[i - period];
            valid = valid.saturating_sub(1);
        }
        if volumes[i].is_finite() && volumes[i] >= 0.0 {
            sum += volumes[i];
            valid += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 14).is_empty());
    }

    #[test]
    fn period_zero_returns_all_none() {
        let v = vec![1000.0; 50];
        assert!(compute(&v, 0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn period_too_large_returns_all_none() {
        let v = vec![1000.0; 10];
        assert!(compute(&v, 20).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_volumes_yield_rvol_one() {
        let v = vec![1000.0; 50];
        let out = compute(&v, 14);
        for x in out.iter().flatten() {
            assert!((x - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn double_volume_bar_yields_rvol_two() {
        let mut v = vec![1000.0; 30];
        v[20] = 2000.0;
        let out = compute(&v, 14);
        let r = out[20].expect("populated");
        assert!((r - 2.0).abs() < 1e-9);
    }

    #[test]
    fn zero_past_average_returns_none() {
        // All zero past → avg = 0 → division-by-zero guard returns None.
        let mut v = vec![0.0; 30];
        v[20] = 100.0;
        let out = compute(&v, 14);
        assert!(out[20].is_none());
    }

    #[test]
    fn nan_and_negative_volumes_filtered_safely() {
        let mut v = vec![1000.0; 30];
        v[5] = f64::NAN;
        v[6] = -100.0;
        v[20] = 2000.0;
        let out = compute(&v, 14);
        // Non-finite/negative bars don't crash; subsequent values still populated.
        assert!(out.iter().any(|x| x.is_some()));
    }

    #[test]
    fn huge_period_no_panic() {
        let v = vec![1000.0; 5];
        assert!(compute(&v, usize::MAX).iter().all(|x| x.is_none()));
    }
}
