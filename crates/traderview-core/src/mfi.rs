//! Money Flow Index — volume-weighted RSI.
//!
//! Per-bar:
//!   typical = (H + L + C) / 3
//!   money_flow = typical × volume
//!   If typical_t > typical_{t-1} → positive money flow.
//!   If typical_t < typical_{t-1} → negative money flow.
//!   Equal → both zero.
//!
//! MFI = 100 - 100 / (1 + positive_sum / negative_sum)
//!     over the lookback period.
//!
//! Convention: >80 overbought, <20 oversold. Same scale as RSI but
//! incorporates volume — money-following beats price-following.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MfiPoint {
    pub typical: f64,
    pub money_flow: f64,
    pub mfi: f64,
}

pub fn compute(bars: &[Bar], period: usize) -> Vec<MfiPoint> {
    let n = bars.len();
    let mut out = vec![MfiPoint::default(); n];
    if n < 2 || period == 0 { return out; }
    let mut typical_series = vec![0.0; n];
    let mut mf_series = vec![0.0; n];
    for i in 0..n {
        typical_series[i] = (bars[i].high + bars[i].low + bars[i].close) / 3.0;
        mf_series[i] = typical_series[i] * bars[i].volume;
        out[i].typical = typical_series[i];
        out[i].money_flow = mf_series[i];
    }
    for (i, slot) in out.iter_mut().enumerate().take(n).skip(period) {
        let mut pos = 0.0;
        let mut neg = 0.0;
        for j in (i + 1 - period)..=i {
            if j == 0 { continue; }
            let tp = typical_series[j];
            let prev_tp = typical_series[j - 1];
            if tp > prev_tp { pos += mf_series[j]; }
            else if tp < prev_tp { neg += mf_series[j]; }
        }
        slot.mfi = if neg == 0.0 {
            if pos > 0.0 { 100.0 } else { 50.0 }
        } else {
            let ratio = pos / neg;
            100.0 - 100.0 / (1.0 + ratio)
        };
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MfiZone { Oversold, Neutral, Overbought }

pub fn classify(mfi: f64) -> MfiZone {
    if mfi > 80.0 { MfiZone::Overbought }
    else if mfi < 20.0 { MfiZone::Oversold }
    else { MfiZone::Neutral }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64, v: f64) -> Bar {
        Bar { high: h, low: l, close: c, volume: v }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 14).is_empty());
    }

    #[test]
    fn single_bar_no_mfi_computable() {
        let out = compute(&[b(10.0, 9.0, 9.5, 1000.0)], 14);
        assert_eq!(out[0].mfi, 0.0);
    }

    #[test]
    fn typical_price_is_hlc_average() {
        let out = compute(&[b(12.0, 9.0, 10.5, 1000.0), b(12.0, 9.0, 10.5, 1000.0)], 1);
        // typical = (12 + 9 + 10.5) / 3 = 10.5.
        assert_eq!(out[0].typical, 10.5);
    }

    #[test]
    fn all_positive_money_flow_yields_100_mfi() {
        // Every bar has higher typical than prior → no negative flow.
        let bars: Vec<Bar> = (1..=10).map(|i| {
            let c = 100.0 + i as f64;
            b(c + 0.5, c - 0.5, c, 1000.0)
        }).collect();
        let out = compute(&bars, 5);
        assert_eq!(out.last().unwrap().mfi, 100.0);
    }

    #[test]
    fn all_negative_money_flow_yields_zero_mfi() {
        let bars: Vec<Bar> = (1..=10).map(|i| {
            let c = 200.0 - i as f64;
            b(c + 0.5, c - 0.5, c, 1000.0)
        }).collect();
        let out = compute(&bars, 5);
        let last = out.last().unwrap();
        // pos = 0, neg > 0 → MFI = 100 - 100/(1+0) = 0.
        assert_eq!(last.mfi, 0.0);
    }

    #[test]
    fn balanced_flow_around_50_mfi() {
        // Alternating up/down typical → roughly equal pos/neg flows.
        let bars = vec![
            b(10.0, 9.0, 9.5, 1000.0),
            b(11.0, 10.0, 10.5, 1000.0),    // up
            b(10.0, 9.0, 9.5, 1000.0),       // down
            b(11.0, 10.0, 10.5, 1000.0),    // up
            b(10.0, 9.0, 9.5, 1000.0),       // down
        ];
        let out = compute(&bars, 4);
        let last = out.last().unwrap();
        // 2 up bars × ~10.5 × 1000 = 21000. 2 down bars × ~9.5 × 1000 = 19000.
        // pos/neg ≈ 1.105, MFI ≈ 100 - 100/2.105 ≈ 52.5.
        assert!(last.mfi > 40.0 && last.mfi < 60.0,
            "balanced flow should produce MFI near 50, got {}", last.mfi);
    }

    // ─── classify ────────────────────────────────────────────────────

    #[test]
    fn classify_over_80_overbought() {
        assert_eq!(classify(85.0), MfiZone::Overbought);
    }

    #[test]
    fn classify_under_20_oversold() {
        assert_eq!(classify(15.0), MfiZone::Oversold);
    }

    #[test]
    fn classify_middle_neutral() {
        assert_eq!(classify(50.0), MfiZone::Neutral);
    }

    #[test]
    fn classify_boundary_at_20_and_80_neutral() {
        assert_eq!(classify(20.0), MfiZone::Neutral);
        assert_eq!(classify(80.0), MfiZone::Neutral);
    }
}
