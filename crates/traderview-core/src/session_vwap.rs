//! Session VWAP — intraday volume-weighted average price with reset at
//! every session boundary.
//!
//! For each bar in the input series, accumulates Σ(typical_price · volume)
//! and Σ(volume) from the start of the current trading session, then
//! divides. When `bar.is_session_start == true`, both running sums reset
//! to zero before adding the current bar.
//!
//! Also reports ±1σ and ±2σ bands using the running variance of
//! (typical_price - VWAP)² weighted by volume.
//!
//! Pure compute. Companion to `anchored_vwap`, `vwap_bands`, `vwap_slippage`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub is_session_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionVwapReport {
    pub vwap: Vec<Option<f64>>,
    pub upper_1: Vec<Option<f64>>,
    pub lower_1: Vec<Option<f64>>,
    pub upper_2: Vec<Option<f64>>,
    pub lower_2: Vec<Option<f64>>,
}

pub fn compute(bars: &[Bar]) -> SessionVwapReport {
    let n = bars.len();
    let mut report = SessionVwapReport {
        vwap: vec![None; n],
        upper_1: vec![None; n],
        lower_1: vec![None; n],
        upper_2: vec![None; n],
        lower_2: vec![None; n],
    };
    if bars.iter().any(|b| {
        !b.high.is_finite()
            || !b.low.is_finite()
            || !b.close.is_finite()
            || !b.volume.is_finite()
            || b.volume < 0.0
    }) {
        return report;
    }
    let mut sum_pv = 0.0_f64;
    let mut sum_v = 0.0_f64;
    let mut sum_p2v = 0.0_f64;
    for (i, bar) in bars.iter().enumerate() {
        if bar.is_session_start {
            sum_pv = 0.0;
            sum_v = 0.0;
            sum_p2v = 0.0;
        }
        let tp = (bar.high + bar.low + bar.close) / 3.0;
        sum_pv += tp * bar.volume;
        sum_v += bar.volume;
        sum_p2v += tp * tp * bar.volume;
        if sum_v > 0.0 {
            let vwap = sum_pv / sum_v;
            let var = (sum_p2v / sum_v - vwap * vwap).max(0.0);
            let std = var.sqrt();
            report.vwap[i] = Some(vwap);
            report.upper_1[i] = Some(vwap + std);
            report.lower_1[i] = Some(vwap - std);
            report.upper_2[i] = Some(vwap + 2.0 * std);
            report.lower_2[i] = Some(vwap - 2.0 * std);
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64, v: f64, ss: bool) -> Bar {
        Bar {
            high: h,
            low: l,
            close: c,
            volume: v,
            is_session_start: ss,
        }
    }

    #[test]
    fn empty_returns_empty() {
        let r = compute(&[]);
        assert!(r.vwap.is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let bars = vec![
            b(101.0, 99.0, 100.0, 1000.0, true),
            b(f64::NAN, 99.0, 100.0, 1000.0, false),
        ];
        let r = compute(&bars);
        assert!(r.vwap.iter().all(|x| x.is_none()));
    }

    #[test]
    fn single_bar_vwap_equals_typical_price() {
        let bars = vec![b(110.0, 100.0, 105.0, 1000.0, true)];
        let r = compute(&bars);
        let v = r.vwap[0].unwrap();
        assert!((v - 105.0).abs() < 1e-9);
    }

    #[test]
    fn session_start_resets_vwap() {
        // Bar 0: tp=100, v=1000 → VWAP=100.
        // Bar 1 (session start): tp=200, v=1000 → VWAP=200 (reset).
        let bars = vec![
            b(100.0, 100.0, 100.0, 1000.0, true),
            b(200.0, 200.0, 200.0, 1000.0, true),
        ];
        let r = compute(&bars);
        assert!((r.vwap[0].unwrap() - 100.0).abs() < 1e-9);
        assert!((r.vwap[1].unwrap() - 200.0).abs() < 1e-9);
    }

    #[test]
    fn vwap_volume_weights_correctly() {
        // Bar 1: tp=100, v=1000. Bar 2 (continued): tp=200, v=3000.
        // VWAP = (100·1000 + 200·3000) / 4000 = 700000/4000 = 175.
        let bars = vec![
            b(100.0, 100.0, 100.0, 1000.0, true),
            b(200.0, 200.0, 200.0, 3000.0, false),
        ];
        let r = compute(&bars);
        assert!((r.vwap[1].unwrap() - 175.0).abs() < 1e-9);
    }

    #[test]
    fn bands_centered_on_vwap() {
        let bars = vec![
            b(100.0, 100.0, 100.0, 1000.0, true),
            b(110.0, 110.0, 110.0, 1000.0, false),
            b(90.0, 90.0, 90.0, 1000.0, false),
        ];
        let r = compute(&bars);
        let v = r.vwap[2].unwrap();
        let u1 = r.upper_1[2].unwrap();
        let l1 = r.lower_1[2].unwrap();
        let u2 = r.upper_2[2].unwrap();
        let l2 = r.lower_2[2].unwrap();
        assert!((u1 - v - (v - l1)).abs() < 1e-9);
        assert!((u2 - v - (v - l2)).abs() < 1e-9);
        assert!(u2 > u1 && l2 < l1);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0, 1000.0, true); 30];
        let r = compute(&bars);
        assert_eq!(r.vwap.len(), 30);
        assert_eq!(r.upper_1.len(), 30);
        assert_eq!(r.upper_2.len(), 30);
    }
}
