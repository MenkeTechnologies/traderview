//! WMA — Weighted Moving Average (linear weights).
//!
//! Most-recent bar gets the largest weight (= period), oldest gets 1.
//! Weight sum = `period · (period + 1) / 2`.
//!
//!   WMA_t = (period·p_t + (period−1)·p_{t−1} + ... + 1·p_{t−period+1})
//!           / (period·(period+1)/2)
//!
//! Less laggy than SMA, smoother than EMA on short windows. Used as the
//! building block for HMA and other linear-weight smoothers.
//!
//! Pure compute.

pub fn compute(values: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let wsum: f64 = (1..=period).map(|k| k as f64).sum();
    for i in (period - 1)..n {
        let window = &values[i + 1 - period..=i];
        let mut num = 0.0;
        // window[0] is the OLDEST (weight 1), window[period-1] is the NEWEST (weight = period).
        for (k, &v) in window.iter().enumerate() {
            num += v * (k + 1) as f64;
        }
        let val = num / wsum;
        if val.is_finite() {
            out[i] = Some(val);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 9).is_empty());
    }

    #[test]
    fn period_zero_returns_all_none() {
        let v = vec![1.0; 20];
        assert!(compute(&v, 0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_wma_equals_constant() {
        let v = vec![100.0; 30];
        let out = compute(&v, 9);
        let last = out[29].expect("populated");
        assert!((last - 100.0).abs() < 1e-9);
    }

    #[test]
    fn wma_weights_most_recent_heavier_than_sma() {
        // 1, 2, 3, 4, 5 — period 5.
        // WMA = (1·1 + 2·2 + 3·3 + 4·4 + 5·5) / 15 = 55/15 ≈ 3.667.
        // SMA = 3.0. WMA pulled toward the most recent value.
        let v = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let out = compute(&v, 5);
        let last = out[4].expect("populated");
        assert!((last - 11.0 / 3.0).abs() < 1e-9, "WMA={last}, expected {}", 11.0 / 3.0);
    }

    #[test]
    fn rising_series_wma_above_window_midpoint() {
        let v: Vec<f64> = (1..=10).map(|i| i as f64).collect();
        let out = compute(&v, 5);
        // Final window [6,7,8,9,10] — midpoint 8, WMA pulled up by larger weights on 10.
        let last = out[9].expect("populated");
        assert!(last > 8.0, "WMA should be above midpoint on rising series, got {last}");
    }

    #[test]
    fn huge_period_no_panic() {
        let v = vec![1.0; 5];
        assert!(compute(&v, usize::MAX).iter().all(|x| x.is_none()));
    }
}
