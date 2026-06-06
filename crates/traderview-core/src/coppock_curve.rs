//! Coppock Curve — Edwin Sedgwick Coppock ("Barron's", October 1962).
//!
//! Long-term momentum indicator originally designed for monthly equity
//! index data. Sums two rate-of-change values, then applies a
//! weighted-moving-average smoother:
//!
//!   ROC_long_t  = 100 · (close_t / close_{t − long_period} − 1)
//!   ROC_short_t = 100 · (close_t / close_{t − short_period} − 1)
//!   Coppock_t   = WMA(ROC_long_t + ROC_short_t, wma_period)
//!
//! Defaults (Coppock's original):
//!   long_period  = 14
//!   short_period = 11
//!   wma_period   = 10
//!
//! Classic signal: zero-line cross from below = long-term buy. Coppock
//! famously avoided sell signals; the indicator is asymmetric by
//! design.
//!
//! Distinct from `coppock_rsi` (which is unrelated despite the shared name
//! token — that one is an RSI variant by a different author).
//!
//! Pure compute.

pub fn compute(
    closes: &[f64],
    long_period: usize,
    short_period: usize,
    wma_period: usize,
) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if long_period < 2 || short_period < 2 || wma_period < 2 || n <= long_period + wma_period {
        return out;
    }
    if closes.iter().any(|x| !x.is_finite() || *x <= 0.0) {
        return out;
    }
    // Compute the ROC sum series.
    let mut roc_sum: Vec<Option<f64>> = vec![None; n];
    for (i, slot) in roc_sum.iter_mut().enumerate().skip(long_period) {
        if i < short_period {
            continue;
        }
        let roc_long = 100.0 * (closes[i] / closes[i - long_period] - 1.0);
        let roc_short = 100.0 * (closes[i] / closes[i - short_period] - 1.0);
        *slot = Some(roc_long + roc_short);
    }
    // WMA over `wma_period`; require contiguous Some inputs.
    let weight_sum = (wma_period * (wma_period + 1) / 2) as f64;
    for (i, slot) in out.iter_mut().enumerate().skip(wma_period - 1) {
        let mut acc = 0.0_f64;
        let mut ok = true;
        for k in 0..wma_period {
            let idx = i + 1 - wma_period + k;
            match roc_sum[idx] {
                Some(v) => acc += v * (k + 1) as f64,
                None => {
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            *slot = Some(acc / weight_sum);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 14, 11, 10).is_empty());
    }

    #[test]
    fn invalid_params_return_all_none() {
        let closes: Vec<f64> = (1..=100).map(|i| 100.0 + i as f64).collect();
        assert!(compute(&closes, 1, 11, 10).iter().all(|x| x.is_none()));
        assert!(compute(&closes, 14, 1, 10).iter().all(|x| x.is_none()));
        assert!(compute(&closes, 14, 11, 1).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nonpositive_close_returns_all_none() {
        let mut closes: Vec<f64> = (1..=100).map(|i| 100.0 + i as f64).collect();
        closes[50] = 0.0;
        assert!(compute(&closes, 14, 11, 10).iter().all(|x| x.is_none()));
        closes[50] = -1.0;
        assert!(compute(&closes, 14, 11, 10).iter().all(|x| x.is_none()));
    }

    #[test]
    fn too_short_returns_all_none() {
        let closes = vec![100.0_f64; 20];
        // Need n > long + wma = 24.
        assert!(compute(&closes, 14, 11, 10).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_yields_zero_coppock() {
        let closes = vec![100.0_f64; 60];
        let out = compute(&closes, 14, 11, 10);
        // ROC = 0 always → Coppock = 0.
        for v in out.iter().skip(23) {
            assert!(v.unwrap().abs() < 1e-12, "got {v:?}");
        }
    }

    #[test]
    fn sustained_uptrend_yields_positive_coppock() {
        let closes: Vec<f64> = (1..=100).map(|i| 100.0 + i as f64).collect();
        let out = compute(&closes, 14, 11, 10);
        let last = out[99].unwrap();
        assert!(last > 0.0, "uptrend Coppock should be positive, got {last}");
    }

    #[test]
    fn sustained_downtrend_yields_negative_coppock() {
        let closes: Vec<f64> = (0..100).map(|i| 200.0 - i as f64).collect();
        let out = compute(&closes, 14, 11, 10);
        let last = out[99].unwrap();
        assert!(
            last < 0.0,
            "downtrend Coppock should be negative, got {last}"
        );
    }

    #[test]
    fn output_length_matches_input() {
        let closes: Vec<f64> = (1..=100)
            .map(|i| 100.0 + (i as f64 * 0.1).sin() * 5.0)
            .collect();
        let out = compute(&closes, 14, 11, 10);
        assert_eq!(out.len(), 100);
    }
}
