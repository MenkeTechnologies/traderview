//! Coppock Curve — E.S.C. Coppock (1962), Barron's.
//!
//!   coppock = 10-period WMA of (ROC_14 + ROC_11)
//!
//! Long-term momentum used originally to time bull-market entries
//! after major corrections. Crossing zero from below = buy signal;
//! crossing above 0 then turning down = sell. Periods typically
//! quoted for monthly bars but works on any timeframe.
//!
//! Pure compute.

pub fn compute(closes: &[f64], roc1: usize, roc2: usize, wma_period: usize) -> Vec<f64> {
    let n = closes.len();
    let mut out = vec![0.0; n];
    if n == 0 || roc1 == 0 || roc2 == 0 || wma_period == 0 {
        return out;
    }
    let max_roc = roc1.max(roc2);
    if n <= max_roc {
        return out;
    }
    // ROC series. ROC is undefined when the base price is zero or
    // negative — division by zero (or by negative, which flips the sign
    // of the rate-of-change) would emit Inf/NaN that poisons the WMA
    // downstream. Leave such bars at 0.0 (the pre-warmup default).
    let mut combined = vec![0.0; n];
    for i in max_roc..n {
        let base1 = closes[i - roc1];
        let base2 = closes[i - roc2];
        if base1 <= 0.0
            || base2 <= 0.0
            || !base1.is_finite()
            || !base2.is_finite()
            || !closes[i].is_finite()
        {
            continue;
        }
        let r1 = (closes[i] - base1) / base1 * 100.0;
        let r2 = (closes[i] - base2) / base2 * 100.0;
        combined[i] = r1 + r2;
    }
    // Weighted MA of combined ROC. Saturating math against hostile JSON
    // `wma_period == usize::MAX` which would otherwise panic in debug.
    let start = max_roc.saturating_add(wma_period).saturating_sub(1);
    for i in start..n {
        let mut sum = 0.0;
        let mut weight_total = 0.0;
        for k in 0..wma_period {
            let w = (wma_period - k) as f64;
            sum += combined[i - k] * w;
            weight_total += w;
        }
        out[i] = sum / weight_total;
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
    fn under_warmup_returns_zeros() {
        let closes = vec![100.0; 15];
        let out = compute(&closes, 14, 11, 10);
        for v in &out {
            assert_eq!(*v, 0.0);
        }
    }

    #[test]
    fn strong_uptrend_coppock_positive() {
        let closes: Vec<f64> = (1..=50).map(|i| 100.0 + i as f64).collect();
        let out = compute(&closes, 14, 11, 10);
        assert!(out[49] > 0.0);
    }

    #[test]
    fn strong_downtrend_coppock_negative() {
        let closes: Vec<f64> = (1..=50).map(|i| 200.0 - i as f64).collect();
        let out = compute(&closes, 14, 11, 10);
        assert!(out[49] < 0.0);
    }

    #[test]
    fn flat_series_coppock_zero_after_warmup() {
        let closes = vec![100.0; 50];
        let out = compute(&closes, 14, 11, 10);
        assert!(out[49].abs() < 1e-9);
    }
}
