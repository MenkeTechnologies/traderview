//! Chaikin Money Flow (CMF) — Marc Chaikin.
//!
//!   MFM = ((close − low) − (high − close)) / (high − low)     ∈ [−1, +1]
//!   MFV = MFM · volume
//!   CMF = sum(MFV, N) / sum(volume, N)                         ∈ [−1, +1]
//!
//! Distinct from Chaikin Oscillator (which is the MACD of accumulation-
//! distribution). CMF directly answers "is money flowing in (positive)
//! or out (negative) on a per-bar weighted basis." Default period = 21.
//!
//! Convention: > +0.05 sustained = bullish accumulation; < -0.05 sustained
//! = distribution.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

pub fn compute(bars: &[Bar], period: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let mut mfv = vec![0.0_f64; n];
    let mut vol = vec![0.0_f64; n];
    for i in 0..n {
        let b = bars[i];
        let range = b.high - b.low;
        // Skip zero-range bars (and non-finite OHLC) — MFM is undefined.
        if !(range > 0.0
            && b.high.is_finite()
            && b.low.is_finite()
            && b.close.is_finite()
            && b.volume.is_finite())
        {
            // Leave mfv[i]=0 and vol[i]=0; the bar contributes nothing.
            continue;
        }
        let mfm = ((b.close - b.low) - (b.high - b.close)) / range;
        mfv[i] = mfm * b.volume;
        vol[i] = b.volume;
    }
    for i in (period - 1)..n {
        let win_mfv: f64 = mfv[i + 1 - period..=i].iter().sum();
        let win_vol: f64 = vol[i + 1 - period..=i].iter().sum();
        if win_vol > 0.0 {
            let v = win_mfv / win_vol;
            if v.is_finite() {
                out[i] = Some(v);
            }
        }
    }
    out
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
    fn empty_returns_empty() {
        assert!(compute(&[], 21).is_empty());
    }

    #[test]
    fn period_zero_returns_all_none() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 30];
        assert!(compute(&bars, 0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn closes_at_high_yields_positive_cmf() {
        // Closes pinned to highs → MFM = +1 → CMF = +1.
        let bars = vec![b(101.0, 99.0, 101.0, 1000.0); 30];
        let out = compute(&bars, 21);
        let last = out[29].expect("populated");
        assert!((last - 1.0).abs() < 1e-9, "got {last}");
    }

    #[test]
    fn closes_at_low_yields_negative_cmf() {
        let bars = vec![b(101.0, 99.0, 99.0, 1000.0); 30];
        let out = compute(&bars, 21);
        let last = out[29].expect("populated");
        assert!((last + 1.0).abs() < 1e-9, "got {last}");
    }

    #[test]
    fn closes_at_midpoint_yields_zero_cmf() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 30];
        let out = compute(&bars, 21);
        let last = out[29].expect("populated");
        assert!(last.abs() < 1e-9, "got {last}");
    }

    #[test]
    fn zero_range_bar_contributes_nothing() {
        // 21 normal bars then 21 doji (high==low). CMF still well-defined
        // from the non-doji bars in the trailing window.
        let mut bars = vec![b(101.0, 99.0, 101.0, 1000.0); 21];
        bars.extend(vec![b(100.0, 100.0, 100.0, 1000.0); 21]);
        let out = compute(&bars, 21);
        // Last 21 are all zero-range → win_vol = 0 → None.
        let last = out[41];
        assert!(last.is_none());
    }

    #[test]
    fn huge_period_no_panic() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 5];
        assert!(compute(&bars, usize::MAX).iter().all(|x| x.is_none()));
    }
}
