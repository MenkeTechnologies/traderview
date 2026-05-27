//! Detrended Price Oscillator.
//!
//!   DPO_t = close_{t - (N/2 + 1)} - SMA(close, N)_t
//!
//! Strips out the longer-cycle trend so cyclic peaks and troughs in
//! the short term become visible. NOT a directional indicator on its
//! own — used to identify cycle length / amplitude.
//!
//! Pure compute.

pub fn compute(closes: &[f64], period: usize) -> Vec<f64> {
    let n = closes.len();
    let mut out = vec![0.0; n];
    if n < period || period == 0 {
        return out;
    }
    let shift = period / 2 + 1;
    for i in (period - 1)..n {
        let sma = closes[(i + 1 - period)..=i].iter().sum::<f64>() / period as f64;
        if i >= shift {
            out[i] = closes[i - shift] - sma;
        }
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
    fn under_period_returns_zeros() {
        let closes = vec![100.0; 5];
        let out = compute(&closes, 20);
        for v in &out {
            assert_eq!(*v, 0.0);
        }
    }

    #[test]
    fn flat_series_dpo_zero() {
        let closes = vec![100.0; 30];
        let out = compute(&closes, 10);
        // SMA = 100, shifted close also 100 → DPO = 0.
        for v in &out[10..] {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn trending_series_dpo_swings_around_zero() {
        // Pure trend should average to zero DPO (trend is REMOVED).
        let closes: Vec<f64> = (1..=50).map(|i| 100.0 + i as f64).collect();
        let out = compute(&closes, 10);
        // Sample some bars.
        let dpo_values: Vec<f64> = out.iter().skip(20).cloned().collect();
        let mean: f64 = dpo_values.iter().sum::<f64>() / dpo_values.len() as f64;
        // Mean should be small relative to absolute magnitudes — the
        // half-period shift means the offset isn't perfect but bounded.
        assert!(mean.abs() < 10.0);
    }

    #[test]
    fn sinusoidal_series_dpo_oscillates() {
        // Pure sin wave around 100. DPO should remove the trend (none here)
        // and emit the oscillation.
        use std::f64::consts::PI;
        let closes: Vec<f64> = (0..40)
            .map(|i| 100.0 + 10.0 * (2.0 * PI * (i as f64) / 20.0).sin())
            .collect();
        let out = compute(&closes, 10);
        // Some bars should be positive, some negative — confirms cycle.
        let positives = out.iter().skip(15).filter(|v| **v > 0.5).count();
        let negatives = out.iter().skip(15).filter(|v| **v < -0.5).count();
        assert!(
            positives > 0 && negatives > 0,
            "sinusoidal input should produce both positive and negative DPO"
        );
    }
}
