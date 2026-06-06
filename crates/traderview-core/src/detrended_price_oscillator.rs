//! Detrended Price Oscillator (DPO).
//!
//! Strips the longer-term trend from price by subtracting a SMA that
//! is shifted into the past:
//!
//!   DPO_t = close_{t − shift} − SMA(close, period)_t
//!
//! where shift = period/2 + 1 (standard convention).
//!
//! Because the SMA is forward-shifted, DPO is offset relative to
//! contemporaneous price — it is meant for cycle analysis (counting
//! peaks and troughs) and NOT for signaling. It does not lag in the
//! usual sense; it lines up the current price with a centered moving
//! average.
//!
//! Default period = 20.
//!
//! Pure compute.

pub fn compute(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period < 2 || n < period {
        return out;
    }
    if closes.iter().any(|x| !x.is_finite()) {
        return out;
    }
    let shift = period / 2 + 1;
    for (i, slot) in out.iter_mut().enumerate().skip(period - 1) {
        if i < shift {
            continue;
        }
        let sma: f64 = closes[i + 1 - period..=i].iter().sum::<f64>() / period as f64;
        *slot = Some(closes[i - shift] - sma);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 20).is_empty());
    }

    #[test]
    fn invalid_period_returns_all_none() {
        let closes = vec![100.0_f64; 30];
        assert!(compute(&closes, 1).iter().all(|x| x.is_none()));
    }

    #[test]
    fn shorter_than_period_returns_all_none() {
        let closes = vec![100.0_f64; 5];
        assert!(compute(&closes, 20).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_yields_zero_dpo() {
        let closes = vec![100.0_f64; 50];
        let out = compute(&closes, 20);
        for x in out.iter().skip(19).flatten() {
            assert!(x.abs() < 1e-12);
        }
    }

    #[test]
    fn linear_trend_yields_constant_dpo() {
        // Linear trend → SMA = midpoint of window; close_{t - shift} - SMA
        // is a constant offset by the linear slope * geometric offset.
        let closes: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let out = compute(&closes, 20);
        // After warmup, all DPO values should be identical (within float).
        let vals: Vec<f64> = out.iter().filter_map(|x| *x).collect();
        let first = vals[0];
        for v in &vals[1..] {
            assert!(
                (v - first).abs() < 1e-9,
                "DPO not constant on linear trend: {first} vs {v}"
            );
        }
    }

    #[test]
    fn cycle_input_dpo_oscillates_around_zero() {
        let closes: Vec<f64> = (0..200)
            .map(|i| 100.0 + (i as f64 * 0.1).sin() * 10.0)
            .collect();
        let out = compute(&closes, 20);
        let vals: Vec<f64> = out.iter().filter_map(|x| *x).collect();
        let mean = vals.iter().sum::<f64>() / vals.len() as f64;
        assert!(
            mean.abs() < 1.0,
            "DPO mean should be near zero on cycle data, got {mean}"
        );
        let max = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min = vals.iter().cloned().fold(f64::INFINITY, f64::min);
        assert!(
            max > 0.0 && min < 0.0,
            "DPO should cross zero on cycle data: [{min}, {max}]"
        );
    }

    #[test]
    fn output_length_matches_input() {
        let closes: Vec<f64> = (0..50).map(|i| 100.0 + i as f64).collect();
        let out = compute(&closes, 20);
        assert_eq!(out.len(), 50);
    }
}
