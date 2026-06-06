//! Anchored Momentum — ROC and weighted-MA smoothing anchored to a
//! specific reference bar.
//!
//! Standard ROC measures (close_t − close_{t−n}) / close_{t−n}.
//! Anchored momentum measures (close_t − close_anchor) / close_anchor,
//! where `anchor` is a user-chosen event bar (earnings, news, FOMC,
//! halt-resume). Pairs naturally with `anchored_obv`, `anchored_vwap`.
//!
//! Optional WMA smoothing over `smooth_period` bars after the anchor
//! using closed-form n(n+1)/2 weight sum (matches the WMA fix done
//! earlier in coppock_rsi).
//!
//! Pure compute.

pub fn compute(closes: &[f64], anchor: usize, smooth_period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if anchor >= n || smooth_period == 0 {
        return out;
    }
    let anchor_close = closes[anchor];
    if !anchor_close.is_finite() || anchor_close <= 0.0 {
        return out;
    }
    // Raw anchored momentum.
    let mut raw = vec![None::<f64>; n];
    for (i, item) in raw.iter_mut().enumerate().skip(anchor) {
        let c = closes[i];
        if !c.is_finite() {
            continue;
        }
        let v = (c - anchor_close) / anchor_close;
        if v.is_finite() {
            *item = Some(v);
        }
    }
    // If smooth_period == 1, output is raw.
    if smooth_period == 1 {
        return raw;
    }
    // Bound smooth_period to (anchor + n - anchor) bars effectively.
    let max_eligible = n - anchor;
    if smooth_period > max_eligible {
        return out;
    }
    let weight_sum = smooth_period as f64 * (smooth_period as f64 + 1.0) / 2.0;
    for (i, slot) in out.iter_mut().enumerate() {
        if i < anchor + smooth_period - 1 {
            continue;
        }
        let lo = i + 1 - smooth_period;
        if lo < anchor {
            continue;
        }
        let mut numer = 0.0_f64;
        let mut ok = true;
        for (k, j) in (lo..=i).enumerate() {
            match raw[j] {
                Some(v) => numer += v * (k + 1) as f64,
                None => {
                    ok = false;
                    break;
                }
            }
        }
        if ok && numer.is_finite() {
            *slot = Some(numer / weight_sum);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 0, 5).is_empty());
    }

    #[test]
    fn anchor_out_of_range_returns_all_none() {
        let c = vec![100.0; 10];
        assert!(compute(&c, 20, 5).iter().all(|x| x.is_none()));
    }

    #[test]
    fn zero_smooth_period_returns_all_none() {
        let c = vec![100.0; 10];
        assert!(compute(&c, 0, 0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn zero_or_nan_anchor_returns_all_none() {
        let mut c = vec![100.0; 10];
        c[0] = 0.0;
        assert!(compute(&c, 0, 1).iter().all(|x| x.is_none()));
        c[0] = f64::NAN;
        assert!(compute(&c, 0, 1).iter().all(|x| x.is_none()));
    }

    #[test]
    fn smooth_period_one_returns_raw_momentum() {
        let c = vec![100.0, 105.0, 110.0, 115.0, 120.0];
        let out = compute(&c, 0, 1);
        // Indices < anchor → None. anchor=0 means first slot included → 0%.
        assert!((out[0].unwrap() - 0.0).abs() < 1e-9);
        // 105/100 - 1 = 0.05
        assert!((out[1].unwrap() - 0.05).abs() < 1e-9);
        assert!((out[4].unwrap() - 0.20).abs() < 1e-9);
    }

    #[test]
    fn smooth_period_larger_than_available_returns_all_none() {
        let c = vec![100.0; 5];
        // Anchor at 3 → only 2 bars after → smooth_period=5 invalid.
        assert!(compute(&c, 3, 5).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_after_anchor_yields_zero_smoothed() {
        let c = vec![100.0; 20];
        let out = compute(&c, 5, 3);
        for x in out.iter().flatten() {
            assert_eq!(*x, 0.0);
        }
    }

    #[test]
    fn rising_series_yields_positive_smoothed_momentum() {
        let c: Vec<f64> = (0..20).map(|i| 100.0 + i as f64).collect();
        let out = compute(&c, 0, 5);
        // Last bar smoothed momentum: WMA over last 5 raw values, each rising.
        let last = out[19].expect("populated");
        assert!(last > 0.0);
    }

    #[test]
    fn falling_series_yields_negative_smoothed_momentum() {
        let c: Vec<f64> = (0..20).map(|i| 100.0 - i as f64 * 0.5).collect();
        let out = compute(&c, 0, 5);
        let last = out[19].expect("populated");
        assert!(last < 0.0);
    }

    #[test]
    fn nan_intermediate_close_blocks_smoothed_window_but_not_others() {
        let mut c = vec![100.0; 20];
        c[10] = f64::NAN;
        let out = compute(&c, 0, 3);
        // The 3-bar window straddling index 10 should be None.
        for (i, slot) in out.iter().enumerate().take(13).skip(10) {
            assert!(slot.is_none(), "index {i} should be None due to NaN");
        }
        // Windows fully before or after should still be populated.
        assert!(out[5].is_some());
        assert!(out[15].is_some());
    }
}
