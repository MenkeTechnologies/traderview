//! Vortex Indicator — Etienne Botes, Douglas Siepman (2010).
//!
//!   tr_t = max(high_t - low_t, |high_t - close_{t-1}|, |low_t - close_{t-1}|)
//!   vm_plus_t  = |high_t - low_{t-1}|
//!   vm_minus_t = |low_t - high_{t-1}|
//!
//!   sum_tr = N-period sum of TR
//!   vi_plus = N-period sum(vm_plus) / sum_tr
//!   vi_minus = N-period sum(vm_minus) / sum_tr
//!
//! VI+ above VI- = uptrend; crossover = trend change.
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct VortexPoint {
    pub vi_plus: f64,
    pub vi_minus: f64,
}

pub fn compute(bars: &[Bar], period: usize) -> Vec<VortexPoint> {
    let n = bars.len();
    let mut out = vec![VortexPoint::default(); n];
    // saturating_add against `period = usize::MAX` overflow that would
    // otherwise bypass the guard. Without it `n < 0` is false (because
    // period+1 wraps to 0) and the function returns junk results.
    if period == 0 || n < period.saturating_add(1) {
        return out;
    }
    let mut tr = vec![0.0; n];
    let mut vm_plus = vec![0.0; n];
    let mut vm_minus = vec![0.0; n];
    for i in 1..n {
        let h = bars[i].high;
        let l = bars[i].low;
        let prev_c = bars[i - 1].close;
        let prev_h = bars[i - 1].high;
        let prev_l = bars[i - 1].low;
        tr[i] = (h - l).max((h - prev_c).abs()).max((l - prev_c).abs());
        vm_plus[i] = (h - prev_l).abs();
        vm_minus[i] = (l - prev_h).abs();
    }
    for i in period..n {
        let sum_tr: f64 = tr[(i + 1 - period)..=i].iter().sum();
        if sum_tr <= 0.0 {
            continue;
        }
        let sum_plus: f64 = vm_plus[(i + 1 - period)..=i].iter().sum();
        let sum_minus: f64 = vm_minus[(i + 1 - period)..=i].iter().sum();
        out[i] = VortexPoint {
            vi_plus: sum_plus / sum_tr,
            vi_minus: sum_minus / sum_tr,
        };
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VortexTrend {
    Up,
    Down,
    Crossover,
    Neutral,
}

pub fn classify(prev: VortexPoint, cur: VortexPoint) -> VortexTrend {
    let was_up = prev.vi_plus > prev.vi_minus;
    let is_up = cur.vi_plus > cur.vi_minus;
    if was_up != is_up {
        return VortexTrend::Crossover;
    }
    if is_up {
        VortexTrend::Up
    } else {
        VortexTrend::Down
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar {
        Bar {
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 14).is_empty());
    }

    #[test]
    fn under_period_zeros() {
        let bars = vec![b(10.0, 9.0, 9.5); 5];
        let out = compute(&bars, 14);
        for p in &out {
            assert_eq!(p.vi_plus, 0.0);
        }
    }

    #[test]
    fn strong_uptrend_vi_plus_dominant() {
        let bars: Vec<Bar> = (1..=20)
            .map(|i| {
                let c = 100.0 + i as f64;
                b(c + 1.0, c - 1.0, c)
            })
            .collect();
        let out = compute(&bars, 14);
        assert!(out[19].vi_plus > out[19].vi_minus);
    }

    #[test]
    fn strong_downtrend_vi_minus_dominant() {
        let bars: Vec<Bar> = (1..=20)
            .map(|i| {
                let c = 200.0 - i as f64;
                b(c + 1.0, c - 1.0, c)
            })
            .collect();
        let out = compute(&bars, 14);
        assert!(out[19].vi_minus > out[19].vi_plus);
    }

    // ─── classify ──────────────────────────────────────────────────────

    #[test]
    fn crossover_detected() {
        let prev = VortexPoint {
            vi_plus: 0.6,
            vi_minus: 0.8,
        };
        let cur = VortexPoint {
            vi_plus: 0.9,
            vi_minus: 0.7,
        };
        assert_eq!(classify(prev, cur), VortexTrend::Crossover);
    }

    #[test]
    fn up_when_vi_plus_stays_above_vi_minus() {
        let prev = VortexPoint {
            vi_plus: 0.9,
            vi_minus: 0.7,
        };
        let cur = VortexPoint {
            vi_plus: 1.0,
            vi_minus: 0.6,
        };
        assert_eq!(classify(prev, cur), VortexTrend::Up);
    }

    #[test]
    fn down_when_vi_minus_stays_above_vi_plus() {
        let prev = VortexPoint {
            vi_plus: 0.5,
            vi_minus: 0.9,
        };
        let cur = VortexPoint {
            vi_plus: 0.4,
            vi_minus: 1.0,
        };
        assert_eq!(classify(prev, cur), VortexTrend::Down);
    }
}
