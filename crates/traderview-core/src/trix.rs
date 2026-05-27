//! TRIX — Jack Hutson's triple-smoothed momentum oscillator.
//!
//! Per bar:
//!   ema1 = EMA(close, period)
//!   ema2 = EMA(ema1, period)
//!   ema3 = EMA(ema2, period)
//!   TRIX = (ema3_t - ema3_{t-1}) / ema3_{t-1} × 100
//!
//! Triple smoothing filters out short-term noise → very clean trend
//! direction signal. Crossing zero is the canonical entry. Signal
//! line = SMA of TRIX (typically 9 periods) for crossover signals.
//!
//! Pure compute.

fn ema(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    if n == 0 || period == 0 { return vec![]; }
    let k = 2.0 / (period as f64 + 1.0);
    let mut out = Vec::with_capacity(n);
    let mut prev = values[0];
    out.push(prev);
    for i in 1..n {
        let e = k * values[i] + (1.0 - k) * prev;
        out.push(e);
        prev = e;
    }
    out
}

pub fn compute(closes: &[f64], period: usize) -> Vec<f64> {
    let n = closes.len();
    let mut out = vec![0.0; n];
    if n < 2 || period == 0 { return out; }
    let e1 = ema(closes, period);
    let e2 = ema(&e1, period);
    let e3 = ema(&e2, period);
    for i in 1..n {
        let prev = e3[i - 1];
        if prev > 0.0 {
            out[i] = (e3[i] - prev) / prev * 100.0;
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
    fn single_close_returns_zero() {
        let out = compute(&[100.0], 14);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0], 0.0);
    }

    #[test]
    fn strong_uptrend_trix_positive() {
        let closes: Vec<f64> = (1..=50).map(|i| 100.0 + i as f64).collect();
        let out = compute(&closes, 14);
        assert!(out[49] > 0.0);
    }

    #[test]
    fn strong_downtrend_trix_negative() {
        let closes: Vec<f64> = (1..=50).map(|i| 200.0 - i as f64).collect();
        let out = compute(&closes, 14);
        assert!(out[49] < 0.0);
    }

    #[test]
    fn flat_series_trix_zero_after_warmup() {
        let closes = vec![100.0; 30];
        let out = compute(&closes, 14);
        // Triple EMA of constant = constant → zero ROC.
        assert!(out[29].abs() < 1e-9);
    }

    #[test]
    fn zero_prior_ema_returns_zero() {
        let closes = vec![0.0, 0.0, 100.0, 200.0];
        let out = compute(&closes, 2);
        // First few EMAs are 0 → TRIX = 0.
        assert_eq!(out[0], 0.0);
    }

    #[test]
    fn trix_responds_to_trend_change() {
        // Uptrend then flat.
        let mut closes: Vec<f64> = (1..=30).map(|i| 100.0 + i as f64).collect();
        closes.extend(vec![130.0; 30]);
        let out = compute(&closes, 5);
        // Mid-trend strongly positive, after-flat decays toward zero.
        assert!(out[29] > 0.0);
        assert!(out[59].abs() < out[29]);
    }
}
