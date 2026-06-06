//! Absorption Detector — bars where heavy volume produced minimal
//! price progress, indicating one side absorbed the other's flow.
//!
//! Per bar i, compares range-to-volume against a recent baseline:
//!
//!   range_t       = high - low
//!   range_per_vol = range / volume
//!   baseline_t    = SMA(range_per_vol, period)
//!   absorption    = range_per_vol_t < baseline · threshold
//!     AND volume_t > SMA(volume, period) · vol_multiplier
//!
//! Direction inferred from close vs midpoint:
//!   bullish: close > midpoint and absorption AND close > prior close
//!     (buyers absorbed sellers)
//!   bearish: close < midpoint and absorption AND close < prior close
//!     (sellers absorbed buyers)
//!
//! Pure compute. Defaults: period = 20, threshold = 0.5, vol_multiplier = 1.5.
//! Companion to `liquidity_grab`, `volume_burst`, `weiss_wave`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AbsorptionReport {
    pub bullish: Vec<bool>,
    pub bearish: Vec<bool>,
    pub period: usize,
    pub threshold: f64,
    pub vol_multiplier: f64,
}

pub fn compute(
    bars: &[Bar],
    period: usize,
    threshold: f64,
    vol_multiplier: f64,
) -> AbsorptionReport {
    let n = bars.len();
    let mut report = AbsorptionReport {
        bullish: vec![false; n],
        bearish: vec![false; n],
        period,
        threshold,
        vol_multiplier,
    };
    if period < 2
        || !threshold.is_finite()
        || threshold <= 0.0
        || !vol_multiplier.is_finite()
        || vol_multiplier <= 0.0
        || n < period + 1
    {
        return report;
    }
    if bars.iter().any(|b| {
        !b.high.is_finite()
            || !b.low.is_finite()
            || !b.close.is_finite()
            || !b.volume.is_finite()
            || b.volume <= 0.0
    }) {
        return report;
    }
    let rpv: Vec<f64> = bars.iter().map(|b| (b.high - b.low) / b.volume).collect();
    let p_f = period as f64;
    for i in period..n {
        let rpv_avg: f64 = rpv[i - period..i].iter().sum::<f64>() / p_f;
        let vol_avg: f64 = bars[i - period..i].iter().map(|b| b.volume).sum::<f64>() / p_f;
        let cur = bars[i];
        let range = cur.high - cur.low;
        if range <= 0.0 {
            continue;
        }
        let mid = cur.low + range / 2.0;
        let prev_close = bars[i - 1].close;
        let absorb = rpv[i] < rpv_avg * threshold && cur.volume > vol_avg * vol_multiplier;
        if absorb && cur.close > mid && cur.close > prev_close {
            report.bullish[i] = true;
        }
        if absorb && cur.close < mid && cur.close < prev_close {
            report.bearish[i] = true;
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64, v: f64) -> Bar {
        Bar {
            high: h,
            low: l,
            close: c,
            volume: v,
        }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 30];
        let r = compute(&bars, 1, 0.5, 1.5);
        assert!(!r.bullish.iter().any(|x| *x));
        let r2 = compute(&bars, 20, 0.0, 1.5);
        assert!(!r2.bullish.iter().any(|x| *x));
    }

    #[test]
    fn nan_or_zero_volume_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0, 1000.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0, 1000.0);
        let r = compute(&bars, 20, 0.5, 1.5);
        assert!(!r.bullish.iter().any(|x| *x));
        let mut bars2 = vec![b(101.0, 99.0, 100.0, 1000.0); 30];
        bars2[5] = b(101.0, 99.0, 100.0, 0.0);
        let r2 = compute(&bars2, 20, 0.5, 1.5);
        assert!(!r2.bullish.iter().any(|x| *x));
    }

    #[test]
    fn flat_market_no_signal() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 30];
        let r = compute(&bars, 20, 0.5, 1.5);
        assert!(!r.bullish.iter().any(|x| *x));
        assert!(!r.bearish.iter().any(|x| *x));
    }

    #[test]
    fn bullish_absorption_detected() {
        // Quiet bars then huge volume + tight range + close at top.
        let mut bars = vec![b(101.0, 99.0, 100.0, 1000.0); 25];
        // Absorption bar: range=1 (tight), volume=10000 (huge), close=100.9 (top).
        bars.push(b(100.9, 99.9, 100.9, 10000.0));
        let r = compute(&bars, 20, 0.5, 1.5);
        assert!(r.bullish[25]);
    }

    #[test]
    fn bearish_absorption_detected() {
        let mut bars = vec![b(101.0, 99.0, 100.0, 1000.0); 25];
        bars.push(b(100.1, 99.1, 99.1, 10000.0));
        let r = compute(&bars, 20, 0.5, 1.5);
        assert!(r.bearish[25]);
    }

    #[test]
    fn no_absorption_when_normal_volume() {
        let mut bars = vec![b(101.0, 99.0, 100.0, 1000.0); 25];
        // Tight range but normal volume → not absorption.
        bars.push(b(100.9, 99.9, 100.9, 1000.0));
        let r = compute(&bars, 20, 0.5, 1.5);
        assert!(!r.bullish[25]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 30];
        let r = compute(&bars, 20, 0.5, 1.5);
        assert_eq!(r.bullish.len(), 30);
        assert_eq!(r.bearish.len(), 30);
    }
}
