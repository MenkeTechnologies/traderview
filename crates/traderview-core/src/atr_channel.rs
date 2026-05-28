//! ATR Channel — moving-average midline with ATR-based upper/lower
//! bands (volatility channel).
//!
//! Distinct from `keltner_squeeze` and `starc_bands`:
//!   - Keltner uses EMA midline with ATR offset
//!   - STARC uses SMA midline with ATR offset
//!   - This module defaults to EMA but takes `use_ema` parameter
//!     and is named for the simpler ATR-channel concept popular in
//!     "Encyclopedia of Trading Strategies".
//!
//!   middle = EMA(close, period) or SMA(close, period)
//!   atr    = Wilder ATR(period)
//!   upper  = middle + multiplier · atr
//!   lower  = middle - multiplier · atr
//!
//! Pure compute. Defaults: period = 20, multiplier = 2.0, use_ema = true.
//! Companion to `keltner_squeeze`, `starc_bands`, `bollinger_band_width`,
//! `chandelier_exit`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AtrChannelReport {
    pub middle: Vec<Option<f64>>,
    pub upper: Vec<Option<f64>>,
    pub lower: Vec<Option<f64>>,
    pub period: usize,
    pub multiplier: f64,
    pub use_ema: bool,
}

pub fn compute(
    bars: &[Bar],
    period: usize,
    multiplier: f64,
    use_ema: bool,
) -> AtrChannelReport {
    let n = bars.len();
    let mut report = AtrChannelReport {
        middle: vec![None; n],
        upper: vec![None; n],
        lower: vec![None; n],
        period,
        multiplier,
        use_ema,
    };
    if period < 2 || !multiplier.is_finite() || multiplier <= 0.0
        || n < period + 1 { return report; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    let closes: Vec<f64> = bars.iter().map(|b| b.close).collect();
    report.middle = if use_ema { ema(&closes, period) } else { sma(&closes, period) };
    // Wilder ATR.
    let mut tr = vec![0.0_f64; n];
    tr[0] = bars[0].high - bars[0].low;
    for i in 1..n {
        let pc = bars[i - 1].close;
        tr[i] = (bars[i].high - bars[i].low)
            .max((bars[i].high - pc).abs())
            .max((bars[i].low - pc).abs());
    }
    let p_f = period as f64;
    let seed: f64 = tr[1..=period].iter().sum::<f64>() / p_f;
    let mut atr = vec![None; n];
    atr[period] = Some(seed);
    let mut cur = seed;
    for i in (period + 1)..n {
        cur = (cur * (p_f - 1.0) + tr[i]) / p_f;
        atr[i] = Some(cur);
    }
    for (i, m_opt) in report.middle.iter().enumerate() {
        if let (Some(m), Some(a)) = (*m_opt, atr[i]) {
            report.upper[i] = Some(m + multiplier * a);
            report.lower[i] = Some(m - multiplier * a);
        }
    }
    report
}

fn sma(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period { return out; }
    let p_f = period as f64;
    let mut sum: f64 = series[..period].iter().sum();
    out[period - 1] = Some(sum / p_f);
    for i in period..n {
        sum += series[i] - series[i - period];
        out[i] = Some(sum / p_f);
    }
    out
}

fn ema(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period { return out; }
    let p_f = period as f64;
    let k = 2.0 / (p_f + 1.0);
    let seed: f64 = series[..period].iter().sum::<f64>() / p_f;
    out[period - 1] = Some(seed);
    let mut cur = seed;
    for (i, &v) in series.iter().enumerate().skip(period) {
        cur = v * k + cur * (1.0 - k);
        out[i] = Some(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 1, 2.0, true);
        assert!(r.middle.iter().all(|x| x.is_none()));
        let r2 = compute(&bars, 20, 0.0, true);
        assert!(r2.middle.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        let r = compute(&bars, 20, 2.0, true);
        assert!(r.middle.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_bands_at_constant_offset() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        let r = compute(&bars, 20, 2.0, true);
        // ATR ≈ 2, midline = 100 → upper ≈ 104, lower ≈ 96.
        let last = 49;
        assert!((r.middle[last].unwrap() - 100.0).abs() < 1e-9);
        let upper = r.upper[last].unwrap();
        let lower = r.lower[last].unwrap();
        assert!((upper - 104.0).abs() < 0.1);
        assert!((lower - 96.0).abs() < 0.1);
    }

    #[test]
    fn upper_above_lower_always() {
        let bars: Vec<_> = (0..50).map(|i| {
            let m = 100.0 + (i as f64 * 0.3).sin() * 5.0;
            b(m + 1.5, m - 1.5, m)
        }).collect();
        let r = compute(&bars, 20, 2.0, true);
        for i in 0..50 {
            if let (Some(u), Some(l)) = (r.upper[i], r.lower[i]) {
                assert!(u > l);
            }
        }
    }

    #[test]
    fn ema_vs_sma_differ_on_step_change() {
        let mut closes = vec![100.0_f64; 30];
        closes.extend(vec![200.0; 5]);
        let bars: Vec<_> = closes.iter()
            .map(|c| b(c + 1.0, c - 1.0, *c))
            .collect();
        let r_ema = compute(&bars, 20, 2.0, true);
        let r_sma = compute(&bars, 20, 2.0, false);
        let last = 34;
        assert!(r_ema.middle[last].unwrap() > r_sma.middle[last].unwrap());
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 20, 2.0, true);
        assert_eq!(r.middle.len(), 30);
        assert_eq!(r.upper.len(), 30);
        assert_eq!(r.lower.len(), 30);
    }
}
