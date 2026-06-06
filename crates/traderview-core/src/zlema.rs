//! ZLEMA — Zero-Lag Exponential Moving Average (John Ehlers).
//!
//! Idea: pre-shift the input by `lag = (period − 1)/2` bars BEFORE running
//! a standard EMA. The pre-shift compensates for the EMA's natural lag,
//! producing a smoother but less-laggy line.
//!
//!   lag = (period − 1) / 2
//!   x_t = 2·price_t − price_{t−lag}
//!   ZLEMA = EMA(x, period)
//!
//! Faster than EMA for the same smoothing factor; the cost is occasional
//! overshoot on sharp reversals (the pre-shift extrapolates forward).
//!
//! Pure compute.

pub fn compute(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period < 2 || n < period {
        return out;
    }
    let lag = (period - 1) / 2;
    let mut shifted = vec![0.0_f64; n];
    for i in lag..n {
        shifted[i] = 2.0 * closes[i] - closes[i - lag];
    }
    let alpha = 2.0 / (period as f64 + 1.0);
    // Seed at i=lag+period-1 with SMA of shifted[lag..lag+period].
    let seed_end = lag + period - 1;
    if seed_end >= n {
        return out;
    }
    let seed: f64 = shifted[lag..=seed_end].iter().sum::<f64>() / period as f64;
    out[seed_end] = Some(seed);
    let mut prev = seed;
    for i in (seed_end + 1)..n {
        prev = alpha * shifted[i] + (1.0 - alpha) * prev;
        if prev.is_finite() {
            out[i] = Some(prev);
        } else {
            // Recovery: reset to current shifted value so the recursion can heal.
            prev = shifted[i];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 10).is_empty());
    }

    #[test]
    fn period_too_small_returns_all_none() {
        let v = vec![100.0; 20];
        assert!(compute(&v, 0).iter().all(|x| x.is_none()));
        assert!(compute(&v, 1).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_zlema_equals_constant() {
        let v = vec![100.0; 50];
        let out = compute(&v, 10);
        let last = out[49].expect("populated");
        assert!((last - 100.0).abs() < 1e-9);
    }

    #[test]
    fn linear_uptrend_zlema_close_to_current_price() {
        let v: Vec<f64> = (1..=60).map(|i| 100.0 + i as f64).collect();
        let out = compute(&v, 10);
        let last = out[59].expect("populated");
        // ZLEMA on a linear trend ≈ current price (the pre-shift cancels the lag).
        assert!(
            (last - v[59]).abs() < 3.0,
            "ZLEMA={last} vs price={}",
            v[59]
        );
    }

    #[test]
    fn huge_period_no_panic() {
        let v = vec![100.0; 5];
        assert!(compute(&v, usize::MAX).iter().all(|x| x.is_none()));
    }
}
