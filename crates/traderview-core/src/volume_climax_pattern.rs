//! Volume Climax Pattern — David Weis / Wyckoff-style climactic
//! volume detection.
//!
//! A volume climax is an exceptional volume spike accompanied by a
//! wide-range bar at a swing extreme — often marks the peak of a
//! buying/selling wave (subsequent reversal common).
//!
//!   bar i is a climax if:
//!     volume_t > avg_volume(period) · vol_multiplier
//!     AND range_t > avg_range(period) · range_multiplier
//!     AND it's the highest high (buying climax) or lowest low
//!         (selling climax) of the last `lookback` bars
//!
//! Pure compute. Defaults: period = 20, lookback = 20, vol_multiplier = 2.0,
//! range_multiplier = 1.5.
//! Companion to `weiss_wave`, `volume_burst`, `absorption_detector`,
//! `vsa`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64, pub volume: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VolumeClimaxReport {
    pub buying_climax: Vec<bool>,
    pub selling_climax: Vec<bool>,
    pub period: usize,
    pub lookback: usize,
    pub vol_multiplier: f64,
    pub range_multiplier: f64,
}

pub fn compute(
    bars: &[Bar],
    period: usize,
    lookback: usize,
    vol_multiplier: f64,
    range_multiplier: f64,
) -> VolumeClimaxReport {
    let n = bars.len();
    let mut report = VolumeClimaxReport {
        buying_climax: vec![false; n],
        selling_climax: vec![false; n],
        period,
        lookback,
        vol_multiplier,
        range_multiplier,
    };
    if period < 2 || lookback < 2
        || !vol_multiplier.is_finite() || vol_multiplier <= 0.0
        || !range_multiplier.is_finite() || range_multiplier <= 0.0
        || n < period.max(lookback) + 1 { return report; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite()
        || !b.close.is_finite() || !b.volume.is_finite() || b.volume < 0.0) {
        return report;
    }
    let p_f = period as f64;
    for i in period.max(lookback)..n {
        let win_p = &bars[i - period..i];
        let avg_vol: f64 = win_p.iter().map(|b| b.volume).sum::<f64>() / p_f;
        let avg_range: f64 = win_p.iter().map(|b| b.high - b.low).sum::<f64>() / p_f;
        let cur = bars[i];
        let cur_range = cur.high - cur.low;
        let big_vol = cur.volume > avg_vol * vol_multiplier;
        let big_range = cur_range > avg_range * range_multiplier;
        if !big_vol || !big_range { continue; }
        let win_l = &bars[i - lookback..i];
        let win_high = win_l.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
        let win_low = win_l.iter().fold(f64::INFINITY, |a, b| a.min(b.low));
        if cur.high > win_high { report.buying_climax[i] = true; }
        if cur.low < win_low { report.selling_climax[i] = true; }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64, v: f64) -> Bar {
        Bar { high: h, low: l, close: c, volume: v }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 50];
        let r = compute(&bars, 1, 20, 2.0, 1.5);
        assert!(!r.buying_climax.iter().any(|x| *x));
        let r2 = compute(&bars, 20, 20, 0.0, 1.5);
        assert!(!r2.buying_climax.iter().any(|x| *x));
    }

    #[test]
    fn nan_or_negative_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0, 1000.0); 50];
        bars[5] = b(f64::NAN, 99.0, 100.0, 1000.0);
        let r = compute(&bars, 20, 20, 2.0, 1.5);
        assert!(!r.buying_climax.iter().any(|x| *x));
    }

    #[test]
    fn flat_market_no_climax() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 50];
        let r = compute(&bars, 20, 20, 2.0, 1.5);
        assert!(!r.buying_climax.iter().any(|x| *x));
        assert!(!r.selling_climax.iter().any(|x| *x));
    }

    #[test]
    fn buying_climax_detected() {
        // 20 quiet bars, then a huge volume + wide range bar at new high.
        let mut bars = vec![b(101.0, 99.0, 100.0, 1000.0); 25];
        bars.push(b(115.0, 99.0, 114.0, 5000.0));
        let r = compute(&bars, 20, 20, 2.0, 1.5);
        assert!(r.buying_climax[25]);
    }

    #[test]
    fn selling_climax_detected() {
        let mut bars = vec![b(101.0, 99.0, 100.0, 1000.0); 25];
        bars.push(b(101.0, 85.0, 86.0, 5000.0));
        let r = compute(&bars, 20, 20, 2.0, 1.5);
        assert!(r.selling_climax[25]);
    }

    #[test]
    fn small_volume_rejects() {
        let mut bars = vec![b(101.0, 99.0, 100.0, 1000.0); 25];
        bars.push(b(115.0, 99.0, 114.0, 1500.0));    // wide range but normal volume
        let r = compute(&bars, 20, 20, 2.0, 1.5);
        assert!(!r.buying_climax[25]);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0); 50];
        let r = compute(&bars, 20, 20, 2.0, 1.5);
        assert_eq!(r.buying_climax.len(), 50);
        assert_eq!(r.selling_climax.len(), 50);
    }
}
