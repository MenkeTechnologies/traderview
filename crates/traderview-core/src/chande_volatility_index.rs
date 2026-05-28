//! Chande Volatility Index (CVI) — Tushar Chande.
//!
//! Percent change in the EMA of the high-low range over `roc_period`:
//!
//!   range_t   = high - low
//!   ema_t     = EMA(range, ema_period)
//!   cvi_t     = (ema_t - ema_{t-roc_period}) / ema_{t-roc_period} · 100
//!
//! Interpretation:
//!   - CVI > 0  → range is expanding (volatility rising)
//!   - CVI < 0  → range is contracting (volatility falling)
//!   - |CVI|>30 → notable shift; used as a regime filter
//!
//! Distinct from `volatility_quality_index` (Stridsman) and
//! `chaikin_volatility` (which uses a similar formula but with
//! different default parameters; CVI's defaults are 10/10 vs
//! Chaikin's 10/10 but Chande's source spec uses 10/10 with
//! exponentially smoothed range as the input).
//!
//! Pure compute. Defaults: ema_period = 10, roc_period = 10.
//! Companion to `chaikin_volatility`, `volatility_stop`,
//! `elder_thermometer`, `damiani_volatmeter`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64 }

pub fn compute(bars: &[Bar], ema_period: usize, roc_period: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if ema_period < 2 || roc_period < 1
        || n < ema_period + roc_period { return out; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite()) { return out; }
    let ranges: Vec<f64> = bars.iter().map(|b| b.high - b.low).collect();
    let p_f = ema_period as f64;
    let k = 2.0 / (p_f + 1.0);
    let seed: f64 = ranges[..ema_period].iter().sum::<f64>() / p_f;
    let mut ema = vec![None; n];
    ema[ema_period - 1] = Some(seed);
    let mut cur = seed;
    for (i, &v) in ranges.iter().enumerate().skip(ema_period) {
        cur = v * k + cur * (1.0 - k);
        ema[i] = Some(cur);
    }
    for (i, slot) in out.iter_mut().enumerate().skip(ema_period + roc_period - 1) {
        if let (Some(cur), Some(prev)) = (ema[i], ema[i - roc_period]) {
            if prev != 0.0 {
                *slot = Some((cur - prev) / prev * 100.0);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> Bar { Bar { high: h, low: l } }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0); 50];
        assert!(compute(&bars, 1, 10).iter().all(|x| x.is_none()));
        assert!(compute(&bars[..5], 10, 10).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0); 50];
        bars[5] = b(f64::NAN, 99.0);
        assert!(compute(&bars, 10, 10).iter().all(|x| x.is_none()));
    }

    #[test]
    fn constant_range_yields_zero_cvi() {
        let bars = vec![b(101.0, 99.0); 50];
        let r = compute(&bars, 10, 10);
        for v in r.iter().flatten() { assert!(v.abs() < 1e-9); }
    }

    #[test]
    fn expanding_range_yields_positive_cvi() {
        // 30 bars of tight range, then expanding range.
        let mut bars = vec![b(101.0, 99.0); 30];
        for i in 0..20 {
            let half = 1.0 + i as f64 * 0.5;
            bars.push(b(100.0 + half, 100.0 - half));
        }
        let r = compute(&bars, 10, 10);
        let last = bars.len() - 1;
        assert!(r[last].unwrap() > 0.0);
    }

    #[test]
    fn contracting_range_yields_negative_cvi() {
        let mut bars: Vec<_> = (0..30).map(|i| {
            let half = 10.0 - i as f64 * 0.2;
            b(100.0 + half, 100.0 - half)
        }).collect();
        bars.extend(vec![b(100.5, 99.5); 20]);
        let r = compute(&bars, 10, 10);
        let last = bars.len() - 1;
        assert!(r[last].unwrap() < 0.0);
    }

    #[test]
    fn output_length_matches_input() {
        let bars = vec![b(101.0, 99.0); 50];
        assert_eq!(compute(&bars, 10, 10).len(), 50);
    }
}
