//! STARC Bands — Stoller Average Range Channel (Manning Stoller).
//!
//! Like Keltner Channels but uses an SMA midline (Keltner uses EMA)
//! and explicit ATR multiplier on each side:
//!
//!   middle = SMA(close, sma_period)
//!   atr    = Wilder ATR over atr_period
//!   upper  = middle + multiplier · atr
//!   lower  = middle - multiplier · atr
//!
//! Common defaults: sma_period = 5, atr_period = 15, multiplier = 2.
//! STARC bands are tighter than Bollinger because volatility is measured
//! by ATR rather than σ, and they don't expand on absolute-price drift.
//!
//! Pure compute. Companion to `keltner_squeeze`, `bollinger_band_width`,
//! `donchian_channels`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StarcBandsReport {
    pub middle: Vec<Option<f64>>,
    pub upper: Vec<Option<f64>>,
    pub lower: Vec<Option<f64>>,
    pub sma_period: usize,
    pub atr_period: usize,
    pub multiplier: f64,
}

pub fn compute(
    bars: &[Bar],
    sma_period: usize,
    atr_period: usize,
    multiplier: f64,
) -> StarcBandsReport {
    let n = bars.len();
    let mut report = StarcBandsReport {
        middle: vec![None; n],
        upper: vec![None; n],
        lower: vec![None; n],
        sma_period,
        atr_period,
        multiplier,
    };
    if sma_period < 2 || atr_period < 2
        || !multiplier.is_finite() || multiplier <= 0.0
        || n < sma_period.max(atr_period) + 1 { return report; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    // SMA of closes.
    let s_f = sma_period as f64;
    let mut sum: f64 = bars[..sma_period].iter().map(|b| b.close).sum();
    report.middle[sma_period - 1] = Some(sum / s_f);
    for i in sma_period..n {
        sum += bars[i].close - bars[i - sma_period].close;
        report.middle[i] = Some(sum / s_f);
    }
    // Wilder ATR.
    let mut tr = vec![0.0_f64; n];
    tr[0] = bars[0].high - bars[0].low;
    for i in 1..n {
        let prev_close = bars[i - 1].close;
        tr[i] = (bars[i].high - bars[i].low)
            .max((bars[i].high - prev_close).abs())
            .max((bars[i].low - prev_close).abs());
    }
    let a_f = atr_period as f64;
    let mut atr = vec![None; n];
    let seed: f64 = tr[1..=atr_period].iter().sum::<f64>() / a_f;
    atr[atr_period] = Some(seed);
    let mut cur = seed;
    for i in (atr_period + 1)..n {
        cur = (cur * (a_f - 1.0) + tr[i]) / a_f;
        atr[i] = Some(cur);
    }
    for (i, a_opt) in atr.iter().enumerate() {
        if let (Some(m), Some(a)) = (report.middle[i], *a_opt) {
            report.upper[i] = Some(m + multiplier * a);
            report.lower[i] = Some(m - multiplier * a);
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

    #[test]
    fn invalid_params_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 1, 15, 2.0);
        assert!(r.middle.iter().all(|x| x.is_none()));
        let r2 = compute(&bars, 5, 15, 0.0);
        assert!(r2.upper.iter().all(|x| x.is_none()));
        let r3 = compute(&bars, 5, 15, f64::NAN);
        assert!(r3.upper.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        let r = compute(&bars, 5, 15, 2.0);
        assert!(r.middle.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_bands_collapse_on_middle() {
        let bars = vec![b(101.0, 99.0, 100.0); 40];
        let r = compute(&bars, 5, 15, 2.0);
        // ATR settles to 2.0 (HL range), bands ±2·2.0 = ±4.0.
        let last = 39;
        assert!((r.middle[last].unwrap() - 100.0).abs() < 1e-9);
        let upper = r.upper[last].unwrap();
        let lower = r.lower[last].unwrap();
        assert!((upper - 104.0).abs() < 0.1);
        assert!((lower - 96.0).abs() < 0.1);
    }

    #[test]
    fn upper_above_lower_always() {
        let bars: Vec<_> = (0..80).map(|i| {
            let m = 100.0 + (i as f64 * 0.3).sin() * 5.0;
            b(m + 1.5, m - 1.5, m)
        }).collect();
        let r = compute(&bars, 5, 15, 2.0);
        for i in 0..80 {
            if let (Some(u), Some(l)) = (r.upper[i], r.lower[i]) {
                assert!(u > l);
            }
        }
    }

    #[test]
    fn higher_multiplier_widens_bands() {
        let bars: Vec<_> = (0..80).map(|i| {
            let m = 100.0 + (i as f64 * 0.3).sin() * 5.0;
            b(m + 1.5, m - 1.5, m)
        }).collect();
        let r1 = compute(&bars, 5, 15, 1.0);
        let r2 = compute(&bars, 5, 15, 3.0);
        let last = 79;
        let w1 = r1.upper[last].unwrap() - r1.lower[last].unwrap();
        let w2 = r2.upper[last].unwrap() - r2.lower[last].unwrap();
        assert!(w2 > w1);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 5, 15, 2.0);
        assert_eq!(r.middle.len(), 30);
        assert_eq!(r.upper.len(), 30);
        assert_eq!(r.lower.len(), 30);
    }
}
