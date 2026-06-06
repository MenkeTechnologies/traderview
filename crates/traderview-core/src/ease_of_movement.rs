//! Ease of Movement (EOM / EMV) — Richard Arms.
//!
//! Combines price range with volume into a single "how easily did the
//! market move today" reading:
//!
//!   midpoint_move = (h_t + l_t)/2 − (h_{t−1} + l_{t−1})/2
//!   box_ratio     = (volume / 100_000_000) / (h_t − l_t)
//!   EMV_raw       = midpoint_move / box_ratio
//!   EOM           = SMA(EMV_raw, period)
//!
//! Positive = price moves up easily on light volume (bullish setup).
//! Negative = price falls easily on light volume. Large absolute values
//! = strong directional bias. Standard period = 14.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub volume: f64,
}

const VOLUME_SCALE: f64 = 100_000_000.0;

pub fn compute(bars: &[Bar], period: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if period == 0 || n < period.saturating_add(1) {
        return out;
    }
    let mut raw = vec![None; n];
    for i in 1..n {
        let mid_today = (bars[i].high + bars[i].low) / 2.0;
        let mid_prev = (bars[i - 1].high + bars[i - 1].low) / 2.0;
        let range = bars[i].high - bars[i].low;
        if range <= 0.0 || !range.is_finite() || !bars[i].volume.is_finite() {
            continue;
        }
        let box_ratio = (bars[i].volume / VOLUME_SCALE) / range;
        if box_ratio == 0.0 || !box_ratio.is_finite() {
            continue;
        }
        let v = (mid_today - mid_prev) / box_ratio;
        if v.is_finite() {
            raw[i] = Some(v);
        }
    }
    // SMA over raw with Option-awareness.
    for i in (period - 1)..n {
        let window = &raw[i + 1 - period..=i];
        if let Some(sum) = window.iter().try_fold(0.0_f64, |s, x| x.map(|v| s + v)) {
            out[i] = Some(sum / period as f64);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, v: f64) -> Bar {
        Bar {
            high: h,
            low: l,
            volume: v,
        }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 14).is_empty());
    }

    #[test]
    fn period_zero_returns_all_none() {
        let bars = vec![b(101.0, 99.0, 1_000_000.0); 30];
        assert!(compute(&bars, 0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn rising_midpoint_yields_positive_eom() {
        // Midpoint rises by 1 per bar; range constant at 2; volume constant.
        let bars: Vec<Bar> = (0..30)
            .map(|i| {
                let mid = 100.0 + i as f64;
                b(mid + 1.0, mid - 1.0, 50_000_000.0)
            })
            .collect();
        let out = compute(&bars, 14);
        let last = out[29].expect("populated");
        assert!(
            last > 0.0,
            "rising midpoint should yield EOM > 0, got {last}"
        );
    }

    #[test]
    fn falling_midpoint_yields_negative_eom() {
        let bars: Vec<Bar> = (0..30)
            .map(|i| {
                let mid = 200.0 - i as f64;
                b(mid + 1.0, mid - 1.0, 50_000_000.0)
            })
            .collect();
        let out = compute(&bars, 14);
        let last = out[29].expect("populated");
        assert!(last < 0.0);
    }

    #[test]
    fn zero_range_bars_skipped_safely() {
        let bars = vec![b(100.0, 100.0, 1_000_000.0); 30];
        let out = compute(&bars, 14);
        // All raw values are None → SMA is None.
        for v in &out {
            assert!(v.is_none());
        }
    }

    #[test]
    fn huge_period_safe() {
        let bars = vec![b(101.0, 99.0, 1_000_000.0); 5];
        assert!(compute(&bars, usize::MAX).iter().all(|x| x.is_none()));
    }
}
