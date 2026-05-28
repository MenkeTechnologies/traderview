//! Woodies CCI — Ken Wood's variant of the Commodity Channel Index.
//!
//! Two CCI lines computed at different periods:
//!   - Short-period CCI (default 6 bars): "Turbo CCI"
//!   - Long-period CCI (default 14 bars): standard CCI
//!
//! Plus the "TLB" (Trend Line Break): the 25-period SMA of standard
//! CCI, used as a slow trend line that the CCI crosses to signal
//! changes.
//!
//! Signals (per the Woodies CCI Trading Club playbook):
//!   - Zero-line reject: CCI fails to cross 0 → trend continues
//!   - Trend Line Break: 14-CCI crosses TLB → trend change
//!   - "Famir": 14-CCI bounces off ±100 in trend direction
//!
//! Pure compute. Companion to the standard `cci` if present, plus
//! `chande_trend_index` and `aroon_indicator`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WoodiesCciReport {
    pub turbo_cci: Vec<Option<f64>>,
    pub standard_cci: Vec<Option<f64>>,
    pub trend_line_break: Vec<Option<f64>>,
    pub turbo_period: usize,
    pub standard_period: usize,
    pub tlb_period: usize,
}

pub fn compute(
    bars: &[Bar],
    turbo_period: usize,
    standard_period: usize,
    tlb_period: usize,
) -> WoodiesCciReport {
    let n = bars.len();
    let mut turbo = vec![None; n];
    let mut standard = vec![None; n];
    let mut tlb = vec![None; n];
    if turbo_period < 2 || standard_period < 2 || tlb_period < 2
        || n < standard_period.max(turbo_period).max(tlb_period) {
        return WoodiesCciReport {
            turbo_cci: turbo, standard_cci: standard, trend_line_break: tlb,
            turbo_period, standard_period, tlb_period,
        };
    }
    if bars.iter().any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()) {
        return WoodiesCciReport {
            turbo_cci: turbo, standard_cci: standard, trend_line_break: tlb,
            turbo_period, standard_period, tlb_period,
        };
    }
    let typical: Vec<f64> = bars.iter().map(|b| (b.high + b.low + b.close) / 3.0).collect();
    turbo = cci_series(&typical, turbo_period);
    standard = cci_series(&typical, standard_period);
    // TLB = SMA of standard CCI over tlb_period.
    for i in (tlb_period - 1)..n {
        let win = &standard[i + 1 - tlb_period..=i];
        if win.iter().any(|x| x.is_none()) { continue; }
        let sum: f64 = win.iter().filter_map(|x| *x).sum();
        tlb[i] = Some(sum / tlb_period as f64);
    }
    WoodiesCciReport {
        turbo_cci: turbo,
        standard_cci: standard,
        trend_line_break: tlb,
        turbo_period,
        standard_period,
        tlb_period,
    }
}

fn cci_series(typical: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = typical.len();
    let mut out = vec![None; n];
    if period == 0 || n < period { return out; }
    let p_f = period as f64;
    for (i, slot) in out.iter_mut().enumerate().skip(period - 1) {
        let win = &typical[i + 1 - period..=i];
        let sma: f64 = win.iter().sum::<f64>() / p_f;
        let mean_dev: f64 = win.iter().map(|x| (x - sma).abs()).sum::<f64>() / p_f;
        *slot = if mean_dev > 0.0 {
            Some((typical[i] - sma) / (0.015 * mean_dev))
        } else {
            Some(0.0)
        };
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar { Bar { high: h, low: l, close: c } }

    #[test]
    fn empty_or_invalid_returns_all_none() {
        let r = compute(&[], 6, 14, 25);
        assert!(r.turbo_cci.is_empty());
        let bars = vec![b(101.0, 99.0, 100.0); 30];
        let r2 = compute(&bars, 1, 14, 25);
        assert!(r2.turbo_cci.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_all_none() {
        let mut bars = vec![b(101.0, 99.0, 100.0); 30];
        bars[5] = b(f64::NAN, 99.0, 100.0);
        let r = compute(&bars, 6, 14, 25);
        assert!(r.standard_cci.iter().all(|x| x.is_none()));
    }

    #[test]
    fn uptrend_yields_positive_cci() {
        let bars: Vec<_> = (0..50).map(|i| {
            let mid = 100.0 + i as f64;
            b(mid + 0.5, mid - 0.5, mid)
        }).collect();
        let r = compute(&bars, 6, 14, 25);
        // CCI is bounded; uptrend yields large positive values.
        assert!(r.standard_cci[49].unwrap() > 0.0);
        assert!(r.turbo_cci[49].unwrap() > 0.0);
    }

    #[test]
    fn downtrend_yields_negative_cci() {
        let bars: Vec<_> = (0..50).map(|i| {
            let mid = 200.0 - i as f64;
            b(mid + 0.5, mid - 0.5, mid)
        }).collect();
        let r = compute(&bars, 6, 14, 25);
        assert!(r.standard_cci[49].unwrap() < 0.0);
    }

    #[test]
    fn flat_market_yields_zero_cci() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        let r = compute(&bars, 6, 14, 25);
        assert!(r.standard_cci[49].unwrap().abs() < 1e-9);
    }

    #[test]
    fn tlb_present_after_full_warmup() {
        let bars: Vec<_> = (0..80).map(|i| {
            let mid = 100.0 + (i as f64 * 0.1).sin();
            b(mid + 1.0, mid - 1.0, mid)
        }).collect();
        let r = compute(&bars, 6, 14, 25);
        // TLB needs standard_period + tlb_period bars of warmup.
        assert!(r.trend_line_break[79].is_some());
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(101.0, 99.0, 100.0); 50];
        let r = compute(&bars, 6, 14, 25);
        assert_eq!(r.turbo_cci.len(), 50);
        assert_eq!(r.standard_cci.len(), 50);
        assert_eq!(r.trend_line_break.len(), 50);
    }
}
