//! Williams Alligator — Bill Williams (chaos theory series).
//!
//! Three SMMAs (smoothed MAs, equivalent to RMA / Wilder smoothing) of
//! the median price, with progressive lengths + shifts forward:
//!   Jaw:   13-period SMMA, shift +8
//!   Teeth:  8-period SMMA, shift +5
//!   Lips:   5-period SMMA, shift +3
//!
//! When all three lines are intertwined → alligator sleeping (no trade).
//! When they fan out → alligator hunting (trend in progress).
//! Pure compute — caller applies the forward shifts at display.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct AlligatorPoint {
    pub jaw: f64,
    pub teeth: f64,
    pub lips: f64,
    pub sleeping: bool,
}

fn smma(values: &[f64], period: usize) -> Vec<f64> {
    let n = values.len();
    let mut out = vec![0.0; n];
    if n < period || period == 0 {
        return out;
    }
    let mut prev: f64 = values[..period].iter().sum::<f64>() / period as f64;
    out[period - 1] = prev;
    for i in period..n {
        let s = (prev * (period as f64 - 1.0) + values[i]) / period as f64;
        out[i] = s;
        prev = s;
    }
    out
}

pub fn compute(bars: &[Bar]) -> Vec<AlligatorPoint> {
    let n = bars.len();
    let medians: Vec<f64> = bars.iter().map(|b| (b.high + b.low) / 2.0).collect();
    let jaw = smma(&medians, 13);
    let teeth = smma(&medians, 8);
    let lips = smma(&medians, 5);
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let j = jaw[i];
        let t = teeth[i];
        let l = lips[i];
        // Sleeping = all three close to each other (within 0.5% range).
        let max = j.max(t).max(l);
        let min = j.min(t).min(l);
        let sleeping = if max > 0.0 {
            (max - min) / max < 0.005
        } else {
            false
        };
        out.push(AlligatorPoint {
            jaw: j,
            teeth: t,
            lips: l,
            sleeping,
        });
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlligatorBias {
    Up,
    Down,
    Sleeping,
}

pub fn classify(point: AlligatorPoint) -> AlligatorBias {
    if point.sleeping {
        return AlligatorBias::Sleeping;
    }
    if point.lips > point.teeth && point.teeth > point.jaw {
        AlligatorBias::Up
    } else if point.lips < point.teeth && point.teeth < point.jaw {
        AlligatorBias::Down
    } else {
        AlligatorBias::Sleeping
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> Bar {
        Bar { high: h, low: l }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[]).is_empty());
    }

    #[test]
    fn flat_series_alligator_sleeping() {
        let bars = vec![b(10.0, 9.0); 30];
        let out = compute(&bars);
        assert!(out[29].sleeping);
    }

    #[test]
    fn uptrend_lips_above_teeth_above_jaw() {
        let bars: Vec<Bar> = (1..=40)
            .map(|i| {
                let p = 100.0 + i as f64;
                b(p, p - 1.0)
            })
            .collect();
        let out = compute(&bars);
        let last = out[39];
        // Lips (5) fastest → tracks closest to current price → highest in uptrend.
        assert!(last.lips > last.teeth);
        assert!(last.teeth > last.jaw);
        assert_eq!(classify(last), AlligatorBias::Up);
    }

    #[test]
    fn downtrend_lips_below_teeth_below_jaw() {
        let bars: Vec<Bar> = (1..=40)
            .map(|i| {
                let p = 200.0 - i as f64;
                b(p, p - 1.0)
            })
            .collect();
        let out = compute(&bars);
        let last = out[39];
        assert!(last.lips < last.teeth);
        assert!(last.teeth < last.jaw);
        assert_eq!(classify(last), AlligatorBias::Down);
    }

    #[test]
    fn classify_returns_sleeping_when_intertwined() {
        let p = AlligatorPoint {
            jaw: 100.0,
            teeth: 100.0,
            lips: 100.0,
            sleeping: true,
        };
        assert_eq!(classify(p), AlligatorBias::Sleeping);
    }

    #[test]
    fn classify_returns_sleeping_when_lines_cross_disorderly() {
        // Not in pure ascending OR descending order → ambiguous → sleeping.
        let p = AlligatorPoint {
            jaw: 100.0,
            teeth: 105.0,
            lips: 95.0,
            sleeping: false,
        };
        assert_eq!(classify(p), AlligatorBias::Sleeping);
    }

    #[test]
    fn under_smma_warmup_zero_values() {
        let bars = vec![b(10.0, 9.0); 5];
        let out = compute(&bars);
        // jaw needs 13 — at index 4 jaw should be 0.
        assert_eq!(out[4].jaw, 0.0);
    }
}
