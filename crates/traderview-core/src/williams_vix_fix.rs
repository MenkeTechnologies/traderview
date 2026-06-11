//! Williams VIX Fix — Larry Williams (2007), Active Trader.
//!
//!   WVF_t = (highest(close, period) − low_t) / highest(close, period) × 100
//!
//! Synthesizes a VIX-like fear gauge for any instrument from its own
//! bars: spikes when price drops hard below the recent close high.
//! High WVF = capitulation / bottom-fishing zone, the inverse of the
//! complacency readings near zero. Default period 22 (≈1 month).
//!
//! Pure compute. Output is aligned with the input bars; pre-warmup
//! positions (and bars with a non-positive rolling high) stay 0.0.

pub fn compute(closes: &[f64], lows: &[f64], period: usize) -> Vec<f64> {
    let n = closes.len().min(lows.len());
    let mut out = vec![0.0; n];
    if n == 0 || period == 0 || n < period {
        return out;
    }
    for i in period - 1..n {
        let hi = closes[i + 1 - period..=i]
            .iter()
            .copied()
            .filter(|c| c.is_finite())
            .fold(f64::MIN, f64::max);
        if hi <= 0.0 || !lows[i].is_finite() {
            continue;
        }
        out[i] = (hi - lows[i]) / hi * 100.0;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wvf_is_zero_when_low_touches_the_rolling_close_high() {
        // Flat tape: close == low == 100 ⇒ WVF = 0 after warmup.
        let closes = vec![100.0; 30];
        let lows = vec![100.0; 30];
        let v = compute(&closes, &lows, 22);
        assert!(v[29].abs() < 1e-12);
        // Pre-warmup positions stay 0.
        assert!(v[..21].iter().all(|x| *x == 0.0));
    }

    #[test]
    fn wvf_spikes_on_a_crash_bar() {
        // 25 flat bars at 100, then a bar whose low crashes to 80 with
        // the rolling close-high still 100 ⇒ WVF = (100−80)/100 = 20%.
        let mut closes = vec![100.0; 26];
        let mut lows = vec![100.0; 26];
        closes[25] = 85.0;
        lows[25] = 80.0;
        let v = compute(&closes, &lows, 22);
        assert!((v[25] - 20.0).abs() < 1e-12, "{}", v[25]);
        // The bar before the crash reads 0 (no fear).
        assert!(v[24].abs() < 1e-12);
    }

    #[test]
    fn wvf_uses_close_high_not_current_close() {
        // Rolling close-high includes the current close: rising tape
        // where low == prior close keeps a small positive reading.
        let closes: Vec<f64> = (0..30).map(|i| 100.0 + i as f64).collect();
        let lows: Vec<f64> = closes.iter().map(|c| c - 1.0).collect();
        let v = compute(&closes, &lows, 22);
        // hi = current close (the max of a rising window), low = c−1.
        let want = 1.0 / closes[29] * 100.0;
        assert!((v[29] - want).abs() < 1e-12);
    }

    #[test]
    fn wvf_survives_hostile_inputs() {
        assert!(compute(&[], &[], 22).is_empty());
        assert_eq!(compute(&[100.0; 5], &[99.0; 5], 22), vec![0.0; 5]);
        assert_eq!(compute(&[100.0; 5], &[99.0; 5], 0), vec![0.0; 5]);
        // NaN poisoning stays contained: finite bars still compute.
        let mut closes = vec![100.0; 30];
        closes[10] = f64::NAN;
        let lows = vec![95.0; 30];
        let v = compute(&closes, &lows, 22);
        assert!((v[29] - 5.0).abs() < 1e-12);
    }
}
