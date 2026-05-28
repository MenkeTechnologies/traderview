//! Negative Volume Index (NVI) + Positive Volume Index (PVI) +
//! Price-Volume Trend (PVT).
//!
//! Norman Fosback's NVI/PVI insight: smart money is more active on
//! low-volume days (NVI) while retail crowd noise dominates high-volume
//! days (PVI). PVT is a cumulative variant of OBV that scales by daily
//! percent change.
//!
//! Each series is a cumulative line seeded at 1000 (NVI/PVI) or 0 (PVT).
//! Standard reading: NVI above its 255-day EMA = smart money is bullish.
//!
//! Pure compute. Caller pre-computes (close, volume) pairs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PvBar {
    pub close: f64,
    pub volume: f64,
}

const SEED: f64 = 1000.0;

/// NVI: updates only on bars where volume FELL versus the prior bar.
pub fn nvi(bars: &[PvBar]) -> Vec<f64> {
    let n = bars.len();
    let mut out = vec![0.0; n];
    if n == 0 {
        return out;
    }
    out[0] = SEED;
    for i in 1..n {
        let prev = out[i - 1];
        if bars[i].volume < bars[i - 1].volume && bars[i - 1].close > 0.0 {
            let pct = (bars[i].close - bars[i - 1].close) / bars[i - 1].close;
            let new = prev * (1.0 + pct);
            out[i] = if new.is_finite() { new } else { prev };
        } else {
            out[i] = prev;
        }
    }
    out
}

/// PVI: updates only on bars where volume ROSE versus the prior bar.
pub fn pvi(bars: &[PvBar]) -> Vec<f64> {
    let n = bars.len();
    let mut out = vec![0.0; n];
    if n == 0 {
        return out;
    }
    out[0] = SEED;
    for i in 1..n {
        let prev = out[i - 1];
        if bars[i].volume > bars[i - 1].volume && bars[i - 1].close > 0.0 {
            let pct = (bars[i].close - bars[i - 1].close) / bars[i - 1].close;
            let new = prev * (1.0 + pct);
            out[i] = if new.is_finite() { new } else { prev };
        } else {
            out[i] = prev;
        }
    }
    out
}

/// Price Volume Trend — Joe Granville–style cumulative line:
/// `PVT_t = PVT_{t−1} + volume_t × pct_change_t`.
pub fn pvt(bars: &[PvBar]) -> Vec<f64> {
    let n = bars.len();
    let mut out = vec![0.0; n];
    if n == 0 {
        return out;
    }
    out[0] = 0.0;
    for i in 1..n {
        let prev = out[i - 1];
        if bars[i - 1].close > 0.0 && bars[i].volume.is_finite() {
            let pct = (bars[i].close - bars[i - 1].close) / bars[i - 1].close;
            let delta = bars[i].volume * pct;
            let new = prev + delta;
            out[i] = if new.is_finite() { new } else { prev };
        } else {
            out[i] = prev;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(c: f64, v: f64) -> PvBar {
        PvBar { close: c, volume: v }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(nvi(&[]).is_empty());
        assert!(pvi(&[]).is_empty());
        assert!(pvt(&[]).is_empty());
    }

    #[test]
    fn nvi_starts_at_seed_and_only_moves_on_volume_drops() {
        let bars = vec![
            b(100.0, 1000.0),
            b(110.0, 2000.0),    // volume UP → NVI flat
            b(120.0, 1500.0),    // volume DOWN → NVI moves +9.09%
        ];
        let out = nvi(&bars);
        assert_eq!(out[0], SEED);
        assert_eq!(out[1], SEED);                  // volume rose → no update
        assert!((out[2] - SEED * 1.0_f64.mul_add(120.0 / 110.0, 0.0)).abs() < 1.0);
    }

    #[test]
    fn pvi_only_moves_on_volume_rises() {
        let bars = vec![
            b(100.0, 1000.0),
            b(110.0, 2000.0),    // volume UP → PVI updates +10%
            b(105.0, 500.0),     // volume DOWN → PVI flat
        ];
        let out = pvi(&bars);
        assert_eq!(out[0], SEED);
        assert!((out[1] - SEED * 1.10).abs() < 0.01);
        assert_eq!(out[2], out[1]);
    }

    #[test]
    fn pvt_accumulates_volume_weighted_pct_change() {
        let bars = vec![
            b(100.0, 1000.0),
            b(110.0, 2000.0),     // +10% × 2000 = +200
            b(99.0, 1000.0),      // -10% × 1000 = -100
        ];
        let out = pvt(&bars);
        assert_eq!(out[0], 0.0);
        assert!((out[1] - 200.0).abs() < 1e-9);
        assert!((out[2] - 100.0).abs() < 1e-9);
    }

    #[test]
    fn zero_or_negative_prior_close_skipped() {
        let bars = vec![
            b(100.0, 1000.0),
            b(0.0, 2000.0),       // close=0, but prior_close=100 ✓ valid
            b(50.0, 500.0),       // prior_close=0 → skip
        ];
        // NVI at index 2: volume dropped 2000→500, but prior_close=0 → no update.
        let out = nvi(&bars);
        assert_eq!(out[2], out[1]);
    }

    #[test]
    fn no_panic_on_huge_inputs() {
        let bars: Vec<PvBar> = (0..1000).map(|i| b(100.0 + i as f64 * 0.1, 1e15)).collect();
        let n = nvi(&bars);
        let p = pvi(&bars);
        let t = pvt(&bars);
        assert!(n.iter().all(|x| x.is_finite()));
        assert!(p.iter().all(|x| x.is_finite()));
        assert!(t.iter().all(|x| x.is_finite()));
    }
}
