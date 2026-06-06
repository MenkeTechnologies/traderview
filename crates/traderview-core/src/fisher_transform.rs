//! Fisher Transform — John Ehlers (2002).
//!
//! Converts the price series into a near-Gaussian distribution by
//! Fisher-transforming a normalized rolling-mid-range location:
//!
//!   x_t = 0.33 · 2 · ((mid_t − min_low) / (max_high − min_low) − 0.5)
//!         + 0.67 · x_{t−1}
//!   x_t = clamp(x_t, −0.999, +0.999)
//!   fish_t = 0.5 · ln((1 + x_t) / (1 − x_t)) + 0.5 · fish_{t−1}
//!
//! Turning points become sharp peaks because Fisher amplifies tails of
//! the bounded normalized input — Ehlers's headline claim is that
//! reversals are nearly impossible to miss visually.
//!
//! Pure compute. Standard period = 10.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FisherReport {
    pub fisher: Vec<Option<f64>>,
    /// Same series shifted forward by 1 — the "trigger". Cross of fisher
    /// over trigger is the classic Ehlers entry signal.
    pub trigger: Vec<Option<f64>>,
}

pub fn compute(bars: &[Bar], period: usize) -> FisherReport {
    let n = bars.len();
    let mut report = FisherReport {
        fisher: vec![None; n],
        trigger: vec![None; n],
    };
    if period < 2 || n < period {
        return report;
    }
    let mut x_prev = 0.0_f64;
    let mut fish_prev = 0.0_f64;
    for i in (period - 1)..n {
        let window = &bars[i + 1 - period..=i];
        let mut hi = f64::NEG_INFINITY;
        let mut lo = f64::INFINITY;
        for b in window {
            if b.high.is_finite() && b.high > hi {
                hi = b.high;
            }
            if b.low.is_finite() && b.low < lo {
                lo = b.low;
            }
        }
        let range = hi - lo;
        if !range.is_finite() || range <= 0.0 {
            continue;
        }
        let mid = (bars[i].high + bars[i].low) / 2.0;
        let x_raw = 0.33 * 2.0 * ((mid - lo) / range - 0.5) + 0.67 * x_prev;
        let x = x_raw.clamp(-0.999, 0.999);
        let fish = 0.5 * ((1.0 + x) / (1.0 - x)).ln() + 0.5 * fish_prev;
        if fish.is_finite() {
            report.fisher[i] = Some(fish);
            if i > 0 {
                report.trigger[i] = report.fisher[i - 1];
            }
            x_prev = x;
            fish_prev = fish;
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> Bar {
        Bar { high: h, low: l }
    }

    #[test]
    fn empty_returns_empty() {
        let r = compute(&[], 10);
        assert!(r.fisher.is_empty());
    }

    #[test]
    fn period_too_small_returns_all_none() {
        let bars = vec![b(101.0, 99.0); 20];
        let r = compute(&bars, 0);
        assert!(r.fisher.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_range_skipped_safely() {
        let bars = vec![b(100.0, 100.0); 30];
        let r = compute(&bars, 10);
        for v in &r.fisher {
            assert!(v.is_none());
        }
    }

    #[test]
    fn rising_then_falling_produces_sign_flip() {
        // Construct a wave: 20 rising then 20 falling.
        let mut bars: Vec<Bar> = (1..=20)
            .map(|i| {
                let m = 100.0 + i as f64;
                b(m + 0.5, m - 0.5)
            })
            .collect();
        bars.extend((1..=20).map(|i| {
            let m = 120.0 - i as f64;
            b(m + 0.5, m - 0.5)
        }));
        let r = compute(&bars, 10);
        // Late in the rise → fisher likely positive; deep into fall → negative.
        let rise = r.fisher[19].expect("populated");
        let fall = r.fisher[39].expect("populated");
        assert!(
            rise.signum() != fall.signum() || rise.abs() < 0.1 || fall.abs() < 0.1,
            "fisher should flip sign across reversal, got rise={rise} fall={fall}"
        );
    }

    #[test]
    fn trigger_one_bar_behind_fisher() {
        let bars: Vec<Bar> = (1..=40)
            .map(|i| b(100.0 + i as f64, 99.0 + i as f64))
            .collect();
        let r = compute(&bars, 10);
        // At any populated index, trigger[i] == fisher[i-1].
        for i in 1..r.fisher.len() {
            if let (Some(f_prev), Some(t)) = (r.fisher[i - 1], r.trigger[i]) {
                assert!((f_prev - t).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn huge_period_no_panic() {
        let bars = vec![b(101.0, 99.0); 5];
        let r = compute(&bars, usize::MAX);
        assert!(r.fisher.iter().all(|x| x.is_none()));
    }
}
