//! VIDYA — Variable Index Dynamic Average (Chande, 1992).
//!
//! Volatility-adaptive EMA where the smoothing constant scales with the
//! ratio of short-term to long-term standard deviation:
//!
//!   k = 2 / (period + 1)
//!   vi_t = |short_stdev_t / long_stdev_t|     ("volatility index")
//!   alpha_t = k · vi_t
//!   VIDYA_t = alpha_t · price_t + (1 − alpha_t) · VIDYA_{t-1}
//!
//! Reacts faster in volatile regimes (vi > 1 widens alpha) and slower
//! in quiet regimes (vi < 1 dampens alpha). Cleaner than EMA on
//! intermittent-volatility series like crypto.
//!
//! Pure compute.

pub fn compute(
    closes: &[f64],
    period: usize,
    short_stdev_period: usize,
    long_stdev_period: usize,
) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period == 0
        || short_stdev_period == 0
        || long_stdev_period == 0
        || short_stdev_period > long_stdev_period
        || n < long_stdev_period
    {
        return out;
    }
    let k = 2.0 / (period as f64 + 1.0);
    let short_sd = rolling_stdev(closes, short_stdev_period);
    let long_sd = rolling_stdev(closes, long_stdev_period);
    // Seed at first index where both stdevs are populated.
    let mut prev: Option<f64> = None;
    for i in 0..n {
        let (ss, ls) = match (short_sd[i], long_sd[i]) {
            (Some(s), Some(l)) => (s, l),
            _ => continue,
        };
        // Flat long-term stdev → alpha 0 (no update). DO NOT `continue`
        // before the seed branch runs — earlier code skipped seeding on
        // flat inputs, leaving the entire series None.
        let alpha = if ls <= 0.0 {
            0.0
        } else {
            (k * (ss / ls).abs()).clamp(0.0, 1.0)
        };
        match prev {
            None => {
                // Seed with the close itself.
                prev = Some(closes[i]);
                out[i] = prev;
            }
            Some(p) => {
                let new = alpha * closes[i] + (1.0 - alpha) * p;
                if new.is_finite() {
                    prev = Some(new);
                    out[i] = prev;
                } else {
                    out[i] = Some(p);
                }
            }
        }
    }
    out
}

fn rolling_stdev(values: &[f64], window: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if window == 0 || n < window {
        return out;
    }
    for i in (window - 1)..n {
        let slice = &values[(i + 1 - window)..=i];
        let m = slice.iter().sum::<f64>() / window as f64;
        let var = slice.iter().map(|v| (v - m).powi(2)).sum::<f64>() / window as f64;
        out[i] = Some(var.sqrt());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 9, 5, 20).is_empty());
    }

    #[test]
    fn invalid_params_return_all_none() {
        let v = vec![100.0; 50];
        for (p, s, l) in [(0, 5, 20), (9, 0, 20), (9, 5, 0), (9, 20, 5)] {
            assert!(
                compute(&v, p, s, l).iter().all(|x| x.is_none()),
                "({p},{s},{l})"
            );
        }
    }

    #[test]
    fn flat_series_vidya_holds_value() {
        let v = vec![100.0; 80];
        let out = compute(&v, 9, 5, 20);
        let last = out.last().copied().flatten().expect("populated");
        // Flat → ss = ls = 0 → carry seed; output stays at the seed value.
        assert!((last - 100.0).abs() < 1e-9);
    }

    #[test]
    fn high_volatility_vidya_tracks_price() {
        // Step function: 20 flat bars then a huge jump → short stdev spikes.
        let mut v = vec![100.0; 30];
        v.extend((0..30).map(|i| 100.0 + (i as f64).sin() * 20.0));
        let out = compute(&v, 9, 5, 20);
        let last = out.last().copied().flatten().expect("populated");
        assert!(last.is_finite());
    }

    #[test]
    fn rising_series_vidya_tracks_upward() {
        let v: Vec<f64> = (0..80).map(|i| 100.0 + i as f64).collect();
        let out = compute(&v, 9, 5, 20);
        let last = out[79].expect("populated");
        // On a clean linear rise short_stdev ≈ long_stdev, so vi ≈ 1,
        // alpha ≈ k, and VIDYA behaves like a standard EMA — lagging
        // upward.
        assert!(last < v[79] && last > v[60]);
    }

    #[test]
    fn huge_period_no_panic() {
        let v = vec![100.0; 5];
        assert!(compute(&v, usize::MAX, 5, 20).iter().all(|x| x.is_none()));
    }
}
