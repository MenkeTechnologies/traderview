//! Chop Zone — color-zone trend qualifier (TASC, derived from Elder's
//! "Coppock vs MA slope" school).
//!
//! Classifies each bar into one of 9 zones (1..9) based on the angle
//! (slope, in degrees) of an exponential moving average over the
//! typical price:
//!
//!   tp_t = (high + low + close) / 3
//!   ema_t = EMA(tp, period)
//!   slope_t = ema_t - ema_{t-1}        (per bar)
//!   angle_t = atan2(slope, range_scale) · 180 / π
//!
//! Where `range_scale` normalizes the slope against typical bar range
//! (Wilder ATR over period) so the angle is comparable across
//! instruments. Zone bins per Elder:
//!
//!   angle > 45  → 9 (strong up)
//!   30..45      → 8
//!   15..30      → 7
//!   5..15       → 6
//!   -5..5       → 5 (chop)
//!   -15..-5     → 4
//!   -30..-15    → 3
//!   -45..-30    → 2
//!   angle < -45 → 1 (strong down)
//!
//! Pure compute. Default period = 30.
//! Companion to `efficiency_ratio`, `chande_trend_index`, `ehlers_correlation_trend_indicator`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChopZoneReport {
    pub zone: Vec<Option<u8>>,
    pub angle_degrees: Vec<Option<f64>>,
    pub ema_typical: Vec<Option<f64>>,
    pub period: usize,
}

pub fn compute(bars: &[Bar], period: usize) -> ChopZoneReport {
    let n = bars.len();
    let mut report = ChopZoneReport {
        zone: vec![None; n],
        angle_degrees: vec![None; n],
        ema_typical: vec![None; n],
        period,
    };
    if period < 2 || n < period + 1 { return report; }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    let tp: Vec<f64> = bars.iter().map(|b| (b.high + b.low + b.close) / 3.0).collect();
    report.ema_typical = ema(&tp, period);
    // ATR for normalization scale.
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
    for (i, a_opt) in atr.iter().enumerate().skip(1) {
        let (Some(et), Some(ep), Some(a)) =
            (report.ema_typical[i], report.ema_typical[i - 1], *a_opt) else { continue };
        if a <= 0.0 { continue; }
        let slope = et - ep;
        let angle = (slope / a).atan().to_degrees();
        report.angle_degrees[i] = Some(angle);
        report.zone[i] = Some(classify(angle));
    }
    report
}

fn classify(angle: f64) -> u8 {
    if angle > 45.0 { 9 }
    else if angle > 30.0 { 8 }
    else if angle > 15.0 { 7 }
    else if angle > 5.0 { 6 }
    else if angle > -5.0 { 5 }
    else if angle > -15.0 { 4 }
    else if angle > -30.0 { 3 }
    else if angle > -45.0 { 2 }
    else { 1 }
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
    for i in period..n {
        cur = series[i] * k + cur * (1.0 - k);
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
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        let r = compute(&bars, 1);
        assert!(r.zone.iter().all(|x| x.is_none()));
        let r2 = compute(&bars[..10], 30);
        assert!(r2.zone.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 50];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        let r = compute(&bars, 30);
        assert!(r.zone.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_zone_five() {
        let bars = vec![b(101.0, 99.0, 100.0); 80];
        let r = compute(&bars, 30);
        for v in r.zone.iter().skip(50).flatten() {
            assert_eq!(*v, 5);    // chop
        }
    }

    #[test]
    fn strong_uptrend_zones_above_six() {
        let bars: Vec<_> = (0..100).map(|i| {
            let m = 100.0 + i as f64;
            b(m + 0.5, m - 0.5, m)
        }).collect();
        let r = compute(&bars, 30);
        let last = r.zone[99].unwrap();
        assert!(last >= 6, "expected zone ≥ 6, got {last}");
    }

    #[test]
    fn strong_downtrend_zones_below_four() {
        let bars: Vec<_> = (0..100).map(|i| {
            let m = 200.0 - i as f64;
            b(m + 0.5, m - 0.5, m)
        }).collect();
        let r = compute(&bars, 30);
        let last = r.zone[99].unwrap();
        assert!(last <= 4);
    }

    #[test]
    fn classify_branches() {
        assert_eq!(classify(50.0), 9);
        assert_eq!(classify(35.0), 8);
        assert_eq!(classify(20.0), 7);
        assert_eq!(classify(10.0), 6);
        assert_eq!(classify(0.0), 5);
        assert_eq!(classify(-10.0), 4);
        assert_eq!(classify(-20.0), 3);
        assert_eq!(classify(-35.0), 2);
        assert_eq!(classify(-50.0), 1);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        let r = compute(&bars, 30);
        assert_eq!(r.zone.len(), 50);
        assert_eq!(r.angle_degrees.len(), 50);
    }
}
