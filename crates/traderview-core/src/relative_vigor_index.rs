//! Relative Vigor Index (RVI) — John Ehlers.
//!
//!   numerator   = SMA( (close − open) +
//!                      2·(close_{t-1} − open_{t-1}) +
//!                      2·(close_{t-2} − open_{t-2}) +
//!                      (close_{t-3} − open_{t-3}), 4) / 6
//!   denominator = SMA( (high − low)   +
//!                      2·(high_{t-1} − low_{t-1}) +
//!                      2·(high_{t-2} − low_{t-2}) +
//!                      (high_{t-3} − low_{t-3}), 4) / 6
//!   RVI = SMA(numerator, period) / SMA(denominator, period)
//!   Signal = SMA(RVI, 4) — but Ehlers uses the 1-2-2-1 weighted sum:
//!            (RVI_t + 2·RVI_{t-1} + 2·RVI_{t-2} + RVI_{t-3}) / 6
//!
//! Reading: positive = bullish vigor; negative = bearish. RVI crossing
//! its signal line is the textbook entry.
//!
//! Pure compute. Standard period = 10.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RviReport {
    pub line: Vec<Option<f64>>,
    pub signal: Vec<Option<f64>>,
}

pub fn compute(bars: &[Bar], period: usize) -> RviReport {
    let n = bars.len();
    let mut report = RviReport {
        line: vec![None; n],
        signal: vec![None; n],
    };
    if period == 0 || n < period.saturating_add(3) {
        return report;
    }
    // Per-bar 1-2-2-1 weighted (close−open) and (high−low).
    let mut weighted_co = vec![0.0_f64; n];
    let mut weighted_hl = vec![0.0_f64; n];
    for i in 3..n {
        let co_now = bars[i].close - bars[i].open;
        let co_1 = bars[i - 1].close - bars[i - 1].open;
        let co_2 = bars[i - 2].close - bars[i - 2].open;
        let co_3 = bars[i - 3].close - bars[i - 3].open;
        weighted_co[i] = (co_now + 2.0 * co_1 + 2.0 * co_2 + co_3) / 6.0;
        let hl_now = bars[i].high - bars[i].low;
        let hl_1 = bars[i - 1].high - bars[i - 1].low;
        let hl_2 = bars[i - 2].high - bars[i - 2].low;
        let hl_3 = bars[i - 3].high - bars[i - 3].low;
        weighted_hl[i] = (hl_now + 2.0 * hl_1 + 2.0 * hl_2 + hl_3) / 6.0;
    }
    // SMA of each over `period`.
    let sma_num = sma(&weighted_co, period);
    let sma_den = sma(&weighted_hl, period);
    for i in 0..n {
        if let (Some(num), Some(den)) = (sma_num[i], sma_den[i]) {
            if den.abs() > f64::EPSILON {
                let v = num / den;
                if v.is_finite() {
                    report.line[i] = Some(v);
                }
            }
        }
    }
    // Signal = 1-2-2-1 weighted average of last 4 RVI values.
    for i in 3..n {
        if let (Some(a), Some(b), Some(c), Some(d)) = (
            report.line[i],
            report.line[i - 1],
            report.line[i - 2],
            report.line[i - 3],
        ) {
            report.signal[i] = Some((a + 2.0 * b + 2.0 * c + d) / 6.0);
        }
    }
    report
}

fn sma(values: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if period == 0 || period > n {
        return out;
    }
    let mut sum = 0.0;
    for i in 0..n {
        sum += values[i];
        if i >= period {
            sum -= values[i - period];
        }
        if i + 1 >= period {
            out[i] = Some(sum / period as f64);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(o: f64, h: f64, l: f64, c: f64) -> Bar {
        Bar {
            open: o,
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = compute(&[], 10);
        assert!(r.line.is_empty());
    }

    #[test]
    fn period_zero_returns_all_none() {
        let bars = vec![b(100.0, 101.0, 99.0, 100.5); 30];
        let r = compute(&bars, 0);
        assert!(r.line.iter().all(|x| x.is_none()));
    }

    #[test]
    fn rising_close_above_open_yields_positive_rvi() {
        // Every bar closes above open → numerator > 0.
        let bars: Vec<Bar> = (0..50).map(|_| b(100.0, 102.0, 99.0, 101.0)).collect();
        let r = compute(&bars, 10);
        let last = r.line.last().copied().flatten().expect("populated");
        assert!(last > 0.0);
    }

    #[test]
    fn falling_close_below_open_yields_negative_rvi() {
        let bars: Vec<Bar> = (0..50).map(|_| b(101.0, 102.0, 99.0, 100.0)).collect();
        let r = compute(&bars, 10);
        let last = r.line.last().copied().flatten().expect("populated");
        assert!(last < 0.0);
    }

    #[test]
    fn signal_lags_rvi_by_construction() {
        // Verify signal[i] uses RVI[i] and RVI[i-1..=i-3].
        let bars: Vec<Bar> = (0..50)
            .map(|i| {
                let o = 100.0 + i as f64;
                b(o, o + 1.0, o - 1.0, o + 0.5)
            })
            .collect();
        let r = compute(&bars, 10);
        for i in 13..r.line.len() {
            if let (Some(s), Some(a), Some(b_), Some(c), Some(d)) = (
                r.signal[i],
                r.line[i],
                r.line[i - 1],
                r.line[i - 2],
                r.line[i - 3],
            ) {
                let expected = (a + 2.0 * b_ + 2.0 * c + d) / 6.0;
                assert!((s - expected).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn huge_period_no_panic() {
        let bars = vec![b(100.0, 101.0, 99.0, 100.0); 5];
        let r = compute(&bars, usize::MAX);
        assert!(r.line.iter().all(|x| x.is_none()));
    }
}
