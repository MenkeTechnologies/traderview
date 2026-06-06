//! Hawkins Dynamic Zones — Leo Zamansky & David Stendahl
//! (Stocks & Commodities, August 1997).
//!
//! Builds two adaptive bands (overbought/oversold zones) around any
//! bounded oscillator (RSI, %K, %D, MFI, etc.). Instead of fixed
//! thresholds at 70/30 or 80/20, the bands are the rolling N-bar
//! percentile of the oscillator series itself:
//!
//!   upper_zone_t = empirical pct-th percentile of osc over period bars
//!   lower_zone_t = empirical (100 - pct)-th percentile
//!
//! When the oscillator crosses INTO its current dynamic band, that's
//! a regime-appropriate overbought / oversold signal that
//! self-calibrates to the underlying market's volatility.
//!
//! Caller supplies any bounded oscillator series (typically RSI,
//! %K, etc.). Pure compute.
//!
//! Defaults: period = 70, pct = 90. Companion to standard
//! overbought/oversold logic on `rsi`, `stochastic_rsi`,
//! `mfi`, `chande_dynamic_momentum_index`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HawkinsZonesReport {
    pub upper_zone: Vec<Option<f64>>,
    pub lower_zone: Vec<Option<f64>>,
    pub overbought: Vec<Option<bool>>,
    pub oversold: Vec<Option<bool>>,
    pub period: usize,
    pub pct: f64,
}

pub fn compute(oscillator: &[Option<f64>], period: usize, pct: f64) -> HawkinsZonesReport {
    let n = oscillator.len();
    let mut report = HawkinsZonesReport {
        upper_zone: vec![None; n],
        lower_zone: vec![None; n],
        overbought: vec![None; n],
        oversold: vec![None; n],
        period,
        pct,
    };
    if period < 2 || !pct.is_finite() || !(50.0..=99.9).contains(&pct) || n < period {
        return report;
    }
    let upper_q = pct / 100.0;
    let lower_q = 1.0 - upper_q;
    for i in (period - 1)..n {
        let win = &oscillator[i + 1 - period..=i];
        if win.iter().any(|x| x.is_none()) {
            continue;
        }
        let mut vals: Vec<f64> = win.iter().filter_map(|x| *x).collect();
        if vals.iter().any(|v| !v.is_finite()) {
            continue;
        }
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let upper = quantile(&vals, upper_q);
        let lower = quantile(&vals, lower_q);
        report.upper_zone[i] = Some(upper);
        report.lower_zone[i] = Some(lower);
        let cur = oscillator[i].unwrap();
        report.overbought[i] = Some(cur >= upper);
        report.oversold[i] = Some(cur <= lower);
    }
    report
}

/// Linear-interpolation quantile (Type 7, numpy default).
fn quantile(sorted: &[f64], q: f64) -> f64 {
    let n = sorted.len();
    if n == 0 {
        return f64::NAN;
    }
    if n == 1 {
        return sorted[0];
    }
    let h = q * (n as f64 - 1.0);
    let lo = h.floor() as usize;
    let hi = (lo + 1).min(n - 1);
    let frac = h - lo as f64;
    sorted[lo] * (1.0 - frac) + sorted[hi] * frac
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let osc = vec![Some(50.0); 100];
        let r = compute(&osc, 1, 90.0);
        assert!(r.upper_zone.iter().all(|x| x.is_none()));
        let r2 = compute(&osc, 70, 10.0); // pct out of range
        assert!(r2.upper_zone.iter().all(|x| x.is_none()));
        let r3 = compute(&osc[..10], 70, 90.0);
        assert!(r3.upper_zone.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_in_window_skips_bar() {
        let mut osc = vec![Some(50.0); 100];
        osc[80] = None;
        let r = compute(&osc, 70, 90.0);
        // Bars 80..149 (would be 80..n-1) where window includes index 80
        // get skipped.
        assert!(r.upper_zone[80].is_none());
    }

    #[test]
    fn constant_oscillator_yields_zone_at_value() {
        let osc = vec![Some(50.0); 100];
        let r = compute(&osc, 70, 90.0);
        // All values are 50 → upper = lower = 50.
        for v in r.upper_zone.iter().skip(70).flatten() {
            assert!((v - 50.0).abs() < 1e-9);
        }
        for v in r.lower_zone.iter().skip(70).flatten() {
            assert!((v - 50.0).abs() < 1e-9);
        }
    }

    #[test]
    fn extreme_high_flags_overbought() {
        let mut osc = vec![Some(50.0); 100];
        osc[99] = Some(100.0);
        let r = compute(&osc, 70, 90.0);
        assert!(r.overbought[99].unwrap());
    }

    #[test]
    fn extreme_low_flags_oversold() {
        let mut osc = vec![Some(50.0); 100];
        osc[99] = Some(0.0);
        let r = compute(&osc, 70, 90.0);
        assert!(r.oversold[99].unwrap());
    }

    #[test]
    fn output_lengths_match_input() {
        let osc = vec![Some(50.0); 100];
        let r = compute(&osc, 70, 90.0);
        assert_eq!(r.upper_zone.len(), 100);
        assert_eq!(r.lower_zone.len(), 100);
        assert_eq!(r.overbought.len(), 100);
        assert_eq!(r.oversold.len(), 100);
    }
}
