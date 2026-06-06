//! Elder Force Index — Dr. Alexander Elder.
//!
//!   raw_force_t = (close_t − close_{t−1}) · volume_t
//!   force_index = EMA(raw_force, period)
//!
//! Combines direction (close change) with conviction (volume). Crosses
//! through zero are textbook entries: positive force = buyers in
//! control, negative = sellers. Standard period = 13 (Elder).
//!
//! Pure compute. Distinct from the existing `force_index` module which
//! returns the raw (non-smoothed) series.

pub fn compute(closes: &[f64], volumes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period == 0 || closes.len() != volumes.len() || n < 2 {
        return out;
    }
    let mut raw = vec![0.0_f64; n];
    let mut have = vec![false; n];
    for i in 1..n {
        if closes[i].is_finite() && closes[i - 1].is_finite() && volumes[i].is_finite() {
            raw[i] = (closes[i] - closes[i - 1]) * volumes[i];
            have[i] = true;
        }
    }
    // Wilder-style EMA: simple `alpha = 2/(period+1)` smoothing seeded by
    // the first SMA window over valid points.
    if n < period.saturating_add(1) {
        return out;
    }
    let alpha = 2.0 / (period as f64 + 1.0);
    // Find seed: SMA of first `period` valid raw values starting at index 1.
    let mut count = 0;
    let mut sum = 0.0;
    let mut seed_idx: Option<usize> = None;
    for i in 1..n {
        if have[i] {
            sum += raw[i];
            count += 1;
            if count == period {
                seed_idx = Some(i);
                break;
            }
        }
    }
    let Some(start) = seed_idx else { return out };
    let mut ema = sum / period as f64;
    if !ema.is_finite() {
        return out;
    }
    out[start] = Some(ema);
    for i in (start + 1)..n {
        if !have[i] {
            continue;
        }
        let v = alpha * raw[i] + (1.0 - alpha) * ema;
        if v.is_finite() {
            ema = v;
            out[i] = Some(ema);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], &[], 13).is_empty());
    }

    #[test]
    fn length_mismatch_returns_all_none() {
        let c = vec![100.0; 30];
        let v = vec![1_000.0; 15];
        assert!(compute(&c, &v, 13).iter().all(|x| x.is_none()));
    }

    #[test]
    fn period_zero_returns_all_none() {
        let c = vec![100.0; 30];
        let v = vec![1_000.0; 30];
        assert!(compute(&c, &v, 0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_closes_yield_zero_force() {
        let c = vec![100.0; 30];
        let v = vec![1_000.0; 30];
        let out = compute(&c, &v, 13);
        // All raw = 0 → EMA = 0 from seed onward.
        for x in out.iter().flatten() {
            assert!(x.abs() < 1e-9);
        }
    }

    #[test]
    fn rising_closes_yield_positive_force() {
        let c: Vec<f64> = (0..40).map(|i| 100.0 + i as f64).collect();
        let v: Vec<f64> = vec![1_000.0; 40];
        let out = compute(&c, &v, 13);
        let last = out[39].expect("populated");
        assert!(last > 0.0);
    }

    #[test]
    fn falling_closes_yield_negative_force() {
        let c: Vec<f64> = (0..40).map(|i| 200.0 - i as f64).collect();
        let v: Vec<f64> = vec![1_000.0; 40];
        let out = compute(&c, &v, 13);
        let last = out[39].expect("populated");
        assert!(last < 0.0);
    }

    #[test]
    fn nan_input_skipped() {
        let mut c = vec![100.0; 40];
        c[5] = f64::NAN;
        let v = vec![1_000.0; 40];
        let out = compute(&c, &v, 13);
        // Result still populated (skipped bar doesn't crash).
        assert!(out.iter().any(|x| x.is_some()));
    }

    #[test]
    fn huge_period_no_panic() {
        let c = vec![100.0; 5];
        let v = vec![1_000.0; 5];
        assert!(compute(&c, &v, usize::MAX).iter().all(|x| x.is_none()));
    }
}
