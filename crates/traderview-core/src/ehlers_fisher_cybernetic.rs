//! Cybernetic Fisher Transform — John Ehlers ("Cybernetic Analysis", 2004).
//!
//! Variant of the classic Fisher Transform with two-stage smoothing
//! that responds more sharply to extremes while filtering noise:
//!
//!   median_t = (high + low) / 2
//!   value_t = 0.66 · ((median - LL) / (HH - LL) - 0.5)
//!             + 0.67 · value_{t-1}
//!     (clamp to [-0.999, 0.999])
//!   fisher_t = 0.5 · ln((1 + value) / (1 - value)) + 0.5 · fisher_{t-1}
//!
//! Where HH/LL are rolling N-bar max/min of `median`. Output unbounded
//! but typically [-2, +2]; ±1.5 are considered overbought/oversold.
//! Trigger line = fisher[t-1] (one-bar lag); crossover of fisher with
//! trigger is the entry signal.
//!
//! Distinct from `fisher_transform` which uses simple Fisher math
//! without the Ehlers double-smoothing.
//!
//! Pure compute. Default period = 10.
//! Companion to `fisher_transform`, `ehlers_super_smoother`,
//! `ehlers_decycler`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CyberneticFisherReport {
    pub fisher: Vec<Option<f64>>,
    pub trigger: Vec<Option<f64>>,
    pub period: usize,
}

pub fn compute(bars: &[Bar], period: usize) -> CyberneticFisherReport {
    let n = bars.len();
    let mut report = CyberneticFisherReport {
        fisher: vec![None; n],
        trigger: vec![None; n],
        period,
    };
    if period < 2 || n < period {
        return report;
    }
    if bars
        .iter()
        .any(|b| !b.high.is_finite() || !b.low.is_finite())
    {
        return report;
    }
    let median: Vec<f64> = bars.iter().map(|b| (b.high + b.low) / 2.0).collect();
    let mut value = vec![0.0_f64; n];
    let mut fisher = vec![0.0_f64; n];
    for i in (period - 1)..n {
        let win = &median[i + 1 - period..=i];
        let hh = win.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let ll = win.iter().cloned().fold(f64::INFINITY, f64::min);
        let range = hh - ll;
        let normalized = if range > 0.0 {
            (median[i] - ll) / range - 0.5
        } else {
            0.0
        };
        value[i] = 0.66 * normalized + 0.67 * value[i.saturating_sub(1)];
        let v = value[i].clamp(-0.999, 0.999);
        fisher[i] = 0.5 * ((1.0 + v) / (1.0 - v)).ln() + 0.5 * fisher[i.saturating_sub(1)];
        report.fisher[i] = Some(fisher[i]);
        if i >= period {
            report.trigger[i] = Some(fisher[i - 1]);
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
    fn invalid_inputs_return_empty() {
        let bars = vec![b(101.0, 99.0); 30];
        let r = compute(&bars, 1);
        assert!(r.fisher.iter().all(|x| x.is_none()));
        let r2 = compute(&bars[..5], 10);
        assert!(r2.fisher.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![b(101.0, 99.0); 30];
        bars[5] = b(f64::NAN, 99.0);
        let r = compute(&bars, 10);
        assert!(r.fisher.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_fisher() {
        let bars = vec![b(101.0, 99.0); 50];
        let r = compute(&bars, 10);
        for v in r.fisher.iter().skip(20).flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn rising_median_yields_positive_fisher() {
        let bars: Vec<_> = (0..50)
            .map(|i| {
                let m = 100.0 + i as f64;
                b(m + 0.5, m - 0.5)
            })
            .collect();
        let r = compute(&bars, 10);
        let last = r.fisher[49].unwrap();
        assert!(last > 0.0);
    }

    #[test]
    fn falling_median_yields_negative_fisher() {
        let bars: Vec<_> = (0..50)
            .map(|i| {
                let m = 200.0 - i as f64;
                b(m + 0.5, m - 0.5)
            })
            .collect();
        let r = compute(&bars, 10);
        let last = r.fisher[49].unwrap();
        assert!(last < 0.0);
    }

    #[test]
    fn trigger_lags_fisher_by_one_bar() {
        let bars: Vec<_> = (0..50)
            .map(|i| {
                let m = 100.0 + (i as f64).sin() * 5.0;
                b(m + 0.5, m - 0.5)
            })
            .collect();
        let r = compute(&bars, 10);
        for i in 12..50 {
            if let (Some(f), Some(t)) = (r.fisher[i - 1], r.trigger[i]) {
                assert!((f - t).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0); 30];
        let r = compute(&bars, 10);
        assert_eq!(r.fisher.len(), 30);
        assert_eq!(r.trigger.len(), 30);
    }
}
