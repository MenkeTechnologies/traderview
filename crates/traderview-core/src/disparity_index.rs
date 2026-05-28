//! Disparity Index — Steve Nison.
//!
//! Percent deviation of the current close from its moving average:
//!
//!   DI_t = (close_t - SMA(close, N)_t) / SMA(close, N)_t · 100
//!
//! Interpretation:
//!   - DI > 0  → price above its mean → uptrend
//!   - DI < 0  → price below its mean → downtrend
//!   - Extreme readings (±5% typical) → overbought/oversold
//!   - Convergence to zero from extremes → mean reversion signal
//!
//! Default period 14. Pure compute.
//!
//! Companion to `bollinger_band_width`, `keltner_squeeze`, `efficiency_ratio`.

pub fn compute(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period < 2 || n < period { return out; }
    if closes.iter().any(|x| !x.is_finite()) { return out; }
    let p_f = period as f64;
    let mut sum: f64 = closes[..period].iter().sum();
    let emit = |i: usize, sum: f64, out: &mut Vec<Option<f64>>| {
        let sma = sum / p_f;
        if sma != 0.0 { out[i] = Some((closes[i] - sma) / sma * 100.0); }
    };
    emit(period - 1, sum, &mut out);
    for i in period..n {
        sum += closes[i] - closes[i - period];
        emit(i, sum, &mut out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() { assert!(compute(&[], 14).is_empty()); }

    #[test]
    fn invalid_params_return_all_none() {
        let c = vec![100.0_f64; 30];
        assert!(compute(&c, 1).iter().all(|x| x.is_none()));
        assert!(compute(&c[..5], 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let mut c = vec![100.0_f64; 30];
        c[5] = f64::NAN;
        assert!(compute(&c, 14).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_di() {
        let c = vec![100.0_f64; 30];
        let r = compute(&c, 14);
        for v in r.iter().flatten() { assert!(v.abs() < 1e-9); }
    }

    #[test]
    fn price_above_mean_yields_positive_di() {
        // Constant 100 then jump to 110 → close > SMA.
        let mut c = vec![100.0_f64; 14];
        c.push(110.0);
        let r = compute(&c, 14);
        let last = r[14].unwrap();
        assert!(last > 0.0);
        // SMA of [100×13, 110] = 100.714... → DI ≈ (110-100.714)/100.714 × 100.
        let expected_sma = (100.0 * 13.0 + 110.0) / 14.0;
        let expected_di = (110.0 - expected_sma) / expected_sma * 100.0;
        assert!((last - expected_di).abs() < 1e-9);
    }

    #[test]
    fn price_below_mean_yields_negative_di() {
        let mut c = vec![100.0_f64; 14];
        c.push(90.0);
        let r = compute(&c, 14);
        assert!(r[14].unwrap() < 0.0);
    }

    #[test]
    fn output_length_matches_input() {
        let c = vec![100.0_f64; 30];
        assert_eq!(compute(&c, 14).len(), 30);
    }
}
