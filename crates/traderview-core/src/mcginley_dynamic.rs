//! McGinley Dynamic — John McGinley (1990).
//!
//! Self-adjusting moving average that speeds up in downtrends and slows
//! down in uptrends, dampening whipsaws compared to fixed-α EMA:
//!
//!   MD_t = MD_{t-1} + (close_t − MD_{t-1}) / (k · period · (close_t / MD_{t-1})^4)
//!
//! The `(close / MD)^4` factor is the adaptive piece — when price is
//! far above the MA, the denominator grows and the MA catches up
//! slowly; when price is below the MA, the denominator shrinks and the
//! MA catches up fast (protecting against sell-side overshoot).
//!
//! Standard `k` = 0.6 (the McGinley default). Pure compute.

pub fn compute(closes: &[f64], period: usize, k: f64) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period == 0 || !k.is_finite() || k <= 0.0 || n < period {
        return out;
    }
    // Seed at index period-1 with the SMA of the first `period` closes.
    let seed: f64 = closes[..period].iter().sum::<f64>() / period as f64;
    if !seed.is_finite() || seed <= 0.0 {
        return out;
    }
    let mut prev = seed;
    out[period - 1] = Some(prev);
    let kp = k * period as f64;
    for i in period..n {
        let c = closes[i];
        if !c.is_finite() || !prev.is_finite() || prev <= 0.0 {
            // Recover: keep prior MD; do not write a corrupt value.
            out[i] = Some(prev);
            continue;
        }
        let ratio = c / prev;
        let denom = kp * ratio.powi(4);
        if !denom.is_finite() || denom.abs() < f64::EPSILON {
            out[i] = Some(prev);
            continue;
        }
        let new = prev + (c - prev) / denom;
        if new.is_finite() {
            prev = new;
            out[i] = Some(prev);
        } else {
            out[i] = Some(prev);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 14, 0.6).is_empty());
    }

    #[test]
    fn invalid_params_return_all_none() {
        let v = vec![100.0; 50];
        for (p, k) in [(0, 0.6), (14, 0.0), (14, -1.0), (14, f64::NAN)] {
            assert!(compute(&v, p, k).iter().all(|x| x.is_none()), "({p}, {k})");
        }
    }

    #[test]
    fn flat_series_md_equals_constant() {
        let v = vec![100.0; 50];
        let out = compute(&v, 14, 0.6);
        let last = out[49].expect("populated");
        assert!((last - 100.0).abs() < 1e-9);
    }

    #[test]
    fn rising_series_md_below_price() {
        let v: Vec<f64> = (1..=60).map(|i| 100.0 + i as f64).collect();
        let out = compute(&v, 14, 0.6);
        let last = out[59].expect("populated");
        // MD lags price upward; on a clean rise the MA is BELOW current price.
        assert!(last < v[59], "MD={last} vs price={}", v[59]);
        // But ABOVE the start of the lookback window.
        assert!(last > v[59 - 14]);
    }

    #[test]
    fn falling_series_md_above_price() {
        let v: Vec<f64> = (1..=60).map(|i| 200.0 - i as f64).collect();
        let out = compute(&v, 14, 0.6);
        let last = out[59].expect("populated");
        // McGinley speeds up in downtrends but still lags — MD > price.
        assert!(last > v[59]);
    }

    #[test]
    fn nonfinite_close_keeps_prior_md() {
        let mut v = vec![100.0; 30];
        v.push(f64::NAN);
        v.extend(vec![100.0; 5]);
        let out = compute(&v, 14, 0.6);
        // Index 30 (the NaN) should carry the prior MD, not corrupt it.
        let at_nan = out[30].expect("populated");
        assert!(at_nan.is_finite());
        assert!((at_nan - 100.0).abs() < 1.0);
    }

    #[test]
    fn huge_period_no_panic() {
        let v = vec![100.0; 5];
        assert!(compute(&v, usize::MAX, 0.6).iter().all(|x| x.is_none()));
    }
}
