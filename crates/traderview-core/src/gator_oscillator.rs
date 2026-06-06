//! Gator Oscillator — Bill Williams.
//!
//! Visualizes the convergence/divergence of the three Alligator lines
//! (jaw 13-bar SMMA shifted 8, teeth 8-bar SMMA shifted 5, lips 5-bar
//! SMMA shifted 3) as two histograms about the zero line:
//!
//!   upper = | jaw - teeth |       (drawn above zero)
//!   lower = -| teeth - lips |     (drawn below zero)
//!
//! Bill Williams' four "feeding stages":
//!   - Sleeping  : both histograms shrinking → low-volatility range
//!   - Awakening : opposite-color bars on both sides (one up, one down)
//!   - Eating    : both bars same color, expanding → trend in progress
//!   - Sated     : both bars opposite colors after eating → trend exhaust
//!
//! This module returns the raw histograms; stage classification is
//! deferred to the consumer.
//!
//! Pure compute. Direct companion to `alligator`, `acceleration_deceleration`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatorOscillatorReport {
    /// |jaw - teeth| (always ≥ 0).
    pub upper: Vec<Option<f64>>,
    /// -|teeth - lips| (always ≤ 0).
    pub lower: Vec<Option<f64>>,
    pub jaw_period: usize,
    pub teeth_period: usize,
    pub lips_period: usize,
}

#[allow(clippy::too_many_arguments)]
pub fn compute(
    highs: &[f64],
    lows: &[f64],
    jaw_period: usize,
    jaw_shift: usize,
    teeth_period: usize,
    teeth_shift: usize,
    lips_period: usize,
    lips_shift: usize,
) -> GatorOscillatorReport {
    let n = highs.len();
    let mut report = GatorOscillatorReport {
        upper: vec![None; n],
        lower: vec![None; n],
        jaw_period,
        teeth_period,
        lips_period,
    };
    if highs.len() != lows.len() || n == 0 {
        return report;
    }
    if jaw_period < 2 || teeth_period < 2 || lips_period < 2 {
        return report;
    }
    if highs.iter().any(|x| !x.is_finite()) || lows.iter().any(|x| !x.is_finite()) {
        return report;
    }
    let median_price: Vec<f64> = highs
        .iter()
        .zip(lows.iter())
        .map(|(h, l)| (h + l) / 2.0)
        .collect();
    let jaw_raw = smma(&median_price, jaw_period);
    let teeth_raw = smma(&median_price, teeth_period);
    let lips_raw = smma(&median_price, lips_period);
    let jaw = shift_forward(&jaw_raw, jaw_shift);
    let teeth = shift_forward(&teeth_raw, teeth_shift);
    let lips = shift_forward(&lips_raw, lips_shift);
    for i in 0..n {
        if let (Some(j), Some(t), Some(l)) = (jaw[i], teeth[i], lips[i]) {
            report.upper[i] = Some((j - t).abs());
            report.lower[i] = Some(-(t - l).abs());
        }
    }
    report
}

fn smma(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let p_f = period as f64;
    let seed: f64 = series[..period].iter().sum::<f64>() / p_f;
    out[period - 1] = Some(seed);
    let mut cur = seed;
    for i in period..n {
        cur = (cur * (p_f - 1.0) + series[i]) / p_f;
        out[i] = Some(cur);
    }
    out
}

fn shift_forward(series: &[Option<f64>], shift: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    for i in 0..n {
        if i >= shift {
            out[i] = series[i - shift];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        let r = compute(&[], &[], 13, 8, 8, 5, 5, 3);
        assert!(r.upper.is_empty());
    }

    #[test]
    fn nan_returns_all_none() {
        let mut h = vec![101.0_f64; 50];
        let l = vec![99.0_f64; 50];
        h[5] = f64::NAN;
        let r = compute(&h, &l, 13, 8, 8, 5, 5, 3);
        assert!(r.upper.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_market_yields_zero_gator() {
        let h = vec![101.0_f64; 50];
        let l = vec![99.0_f64; 50];
        let r = compute(&h, &l, 13, 8, 8, 5, 5, 3);
        for v in r.upper.iter().skip(30).flatten() {
            assert!(v.abs() < 1e-9);
        }
        for v in r.lower.iter().skip(30).flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn upper_always_non_negative() {
        let h: Vec<f64> = (0..80)
            .map(|i| 100.0 + (i as f64 * 0.2).sin() * 5.0)
            .collect();
        let l: Vec<f64> = h.iter().map(|x| x - 2.0).collect();
        let r = compute(&h, &l, 13, 8, 8, 5, 5, 3);
        for v in r.upper.iter().flatten() {
            assert!(*v >= 0.0);
        }
    }

    #[test]
    fn lower_always_non_positive() {
        let h: Vec<f64> = (0..80)
            .map(|i| 100.0 + (i as f64 * 0.2).sin() * 5.0)
            .collect();
        let l: Vec<f64> = h.iter().map(|x| x - 2.0).collect();
        let r = compute(&h, &l, 13, 8, 8, 5, 5, 3);
        for v in r.lower.iter().flatten() {
            assert!(*v <= 0.0);
        }
    }

    #[test]
    fn trend_expands_gator() {
        // Strong uptrend → jaw lags lips → both histograms grow.
        let h: Vec<f64> = (0..80).map(|i| 100.0 + i as f64).collect();
        let l: Vec<f64> = (0..80).map(|i| 99.0 + i as f64).collect();
        let r = compute(&h, &l, 13, 8, 8, 5, 5, 3);
        let last_up = r.upper[79].unwrap();
        let last_lo = r.lower[79].unwrap();
        assert!(last_up > 1.0);
        assert!(last_lo < -0.5);
    }

    #[test]
    fn output_lengths_match_input() {
        let h = vec![101.0_f64; 50];
        let l = vec![99.0_f64; 50];
        let r = compute(&h, &l, 13, 8, 8, 5, 5, 3);
        assert_eq!(r.upper.len(), 50);
        assert_eq!(r.lower.len(), 50);
    }

    #[test]
    fn mismatched_lengths_return_empty_data() {
        let h = vec![101.0_f64; 50];
        let l = vec![99.0_f64; 49];
        let r = compute(&h, &l, 13, 8, 8, 5, 5, 3);
        assert!(r.upper.iter().all(|x| x.is_none()));
    }
}
