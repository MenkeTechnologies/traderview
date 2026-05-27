//! Chaikin Volatility (CHV) — Marc Chaikin.
//!
//! Per bar:
//!   ema_range = EMA(high - low, period)
//!   chv = (ema_range_t - ema_range_{t - lookback}) / ema_range_{t - lookback} × 100
//!
//! Rising CHV = volatility expanding (markup or markdown phase).
//! Falling CHV = volatility contracting (accumulation/distribution).
//!
//! Distinct from CHX (Chaikin Oscillator, which is a volume indicator).
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
}

pub fn compute(bars: &[Bar], ema_period: usize, change_lookback: usize) -> Vec<f64> {
    let n = bars.len();
    let mut out = vec![0.0; n];
    if n == 0 || ema_period == 0 {
        return out;
    }
    let mut ema_range = vec![0.0; n];
    let k = 2.0 / (ema_period as f64 + 1.0);
    let mut prev_ema = bars[0].high - bars[0].low;
    ema_range[0] = prev_ema;
    for i in 1..n {
        let range = bars[i].high - bars[i].low;
        let ema = k * range + (1.0 - k) * prev_ema;
        ema_range[i] = ema;
        prev_ema = ema;
    }
    for i in change_lookback..n {
        let prior = ema_range[i - change_lookback];
        if prior > 0.0 {
            out[i] = (ema_range[i] - prior) / prior * 100.0;
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VolatilityRegime {
    Expanding,
    Contracting,
    Stable,
}

pub fn classify(chv: f64) -> VolatilityRegime {
    if chv > 20.0 {
        VolatilityRegime::Expanding
    } else if chv < -20.0 {
        VolatilityRegime::Contracting
    } else {
        VolatilityRegime::Stable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> Bar {
        Bar { high: h, low: l }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 10, 10).is_empty());
    }

    #[test]
    fn under_lookback_returns_zeros() {
        let bars = vec![b(10.0, 9.0); 5];
        let out = compute(&bars, 10, 10);
        for v in &out {
            assert_eq!(*v, 0.0);
        }
    }

    #[test]
    fn flat_range_chv_zero() {
        let bars = vec![b(10.0, 9.0); 30];
        let out = compute(&bars, 10, 10);
        assert!(out[29].abs() < 1e-9);
    }

    #[test]
    fn expanding_range_positive_chv() {
        // First 10 bars range 1, next 20 bars range 5 → expanding.
        let mut bars: Vec<Bar> = (0..10).map(|_| b(10.0, 9.0)).collect();
        bars.extend((0..20).map(|_| b(15.0, 10.0)));
        let out = compute(&bars, 5, 10);
        assert!(out[29] > 0.0, "expanding range → positive CHV");
    }

    #[test]
    fn contracting_range_negative_chv() {
        let mut bars: Vec<Bar> = (0..10).map(|_| b(15.0, 10.0)).collect();
        bars.extend((0..20).map(|_| b(10.0, 9.0)));
        let out = compute(&bars, 5, 10);
        assert!(out[29] < 0.0);
    }

    // ─── classify ──────────────────────────────────────────────────────

    #[test]
    fn classify_above_20pct_expanding() {
        assert_eq!(classify(30.0), VolatilityRegime::Expanding);
    }

    #[test]
    fn classify_below_minus_20pct_contracting() {
        assert_eq!(classify(-30.0), VolatilityRegime::Contracting);
    }

    #[test]
    fn classify_within_20pct_stable() {
        assert_eq!(classify(10.0), VolatilityRegime::Stable);
        assert_eq!(classify(-10.0), VolatilityRegime::Stable);
        assert_eq!(classify(0.0), VolatilityRegime::Stable);
    }

    #[test]
    fn classify_exactly_20_or_minus_20_stable() {
        // Strict > and < — boundaries themselves are stable.
        assert_eq!(classify(20.0), VolatilityRegime::Stable);
        assert_eq!(classify(-20.0), VolatilityRegime::Stable);
    }
}
