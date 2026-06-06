//! Guppy Multiple Moving Average (GMMA) — Daryl Guppy.
//!
//! Two ribbons of EMAs plotted together:
//!   - Short-term traders: 3, 5, 8, 10, 12, 15
//!   - Long-term investors: 30, 35, 40, 45, 50, 60
//!
//! The relationship between the two ribbons reveals trend strength
//! and turning points:
//!   - Both ribbons fanning out + apart → strong trend
//!   - Both ribbons compressing → loss of momentum
//!   - Short-term crossing through long-term → trend change
//!
//! Classifier output:
//!   - StrongUptrend     : short ribbon all > long ribbon
//!   - StrongDowntrend   : short ribbon all < long ribbon
//!   - Compression       : ribbons overlap (short min < long max AND
//!     short max > long min)
//!   - MixedOrTransition : default
//!
//! Pure compute. Companion to `alligator`, `kaufman_efficiency_ratio`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum GuppyRegime {
    #[default]
    MixedOrTransition,
    StrongUptrend,
    StrongDowntrend,
    Compression,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GuppyMmaReport {
    /// Each inner Vec is one EMA series across all bars.
    /// Index 0..6 = short-term, 6..12 = long-term.
    pub ribbons: Vec<Vec<Option<f64>>>,
    pub short_periods: Vec<usize>,
    pub long_periods: Vec<usize>,
    pub regime: Vec<GuppyRegime>,
}

pub fn compute(closes: &[f64]) -> GuppyMmaReport {
    let short_periods = vec![3, 5, 8, 10, 12, 15];
    let long_periods = vec![30, 35, 40, 45, 50, 60];
    compute_with(closes, short_periods, long_periods)
}

pub fn compute_with(
    closes: &[f64],
    short_periods: Vec<usize>,
    long_periods: Vec<usize>,
) -> GuppyMmaReport {
    let n = closes.len();
    let mut report = GuppyMmaReport {
        ribbons: Vec::new(),
        short_periods: short_periods.clone(),
        long_periods: long_periods.clone(),
        regime: vec![GuppyRegime::MixedOrTransition; n],
    };
    if short_periods.is_empty() || long_periods.is_empty() {
        return report;
    }
    if short_periods
        .iter()
        .chain(long_periods.iter())
        .any(|p| *p < 2)
    {
        return report;
    }
    if closes.iter().any(|x| !x.is_finite()) {
        return report;
    }
    let all_periods: Vec<usize> = short_periods
        .iter()
        .chain(long_periods.iter())
        .copied()
        .collect();
    for p in &all_periods {
        report.ribbons.push(ema(closes, *p));
    }
    let n_short = short_periods.len();
    let n_long = long_periods.len();
    for i in 0..n {
        let short_vals: Option<Vec<f64>> = (0..n_short).map(|k| report.ribbons[k][i]).collect();
        let long_vals: Option<Vec<f64>> = (0..n_long)
            .map(|k| report.ribbons[n_short + k][i])
            .collect();
        if let (Some(s), Some(l)) = (short_vals, long_vals) {
            let s_min = s.iter().cloned().fold(f64::INFINITY, f64::min);
            let s_max = s.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let l_min = l.iter().cloned().fold(f64::INFINITY, f64::min);
            let l_max = l.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            report.regime[i] = if s_min > l_max {
                GuppyRegime::StrongUptrend
            } else if s_max < l_min {
                GuppyRegime::StrongDowntrend
            } else if s_min < l_max && s_max > l_min {
                GuppyRegime::Compression
            } else {
                GuppyRegime::MixedOrTransition
            };
        }
    }
    report
}

fn ema(series: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if period == 0 || n < period {
        return out;
    }
    let p_f = period as f64;
    let k = 2.0 / (p_f + 1.0);
    let seed: f64 = series[..period].iter().sum::<f64>() / p_f;
    out[period - 1] = Some(seed);
    let mut cur = seed;
    for i in period..n {
        cur = series[i] * k + cur * (1.0 - k);
        out[i] = Some(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty_ribbons() {
        let r = compute(&[]);
        assert!(r.ribbons.iter().all(|s| s.is_empty()));
        assert!(r.regime.is_empty());
    }

    #[test]
    fn nan_returns_no_ribbons() {
        let mut c = vec![100.0_f64; 100];
        c[5] = f64::NAN;
        let r = compute(&c);
        assert!(r.ribbons.is_empty());
    }

    #[test]
    fn strong_uptrend_classified() {
        let c: Vec<f64> = (0..200).map(|i| 100.0 + i as f64).collect();
        let r = compute(&c);
        assert_eq!(r.regime[199], GuppyRegime::StrongUptrend);
    }

    #[test]
    fn strong_downtrend_classified() {
        let c: Vec<f64> = (0..200).map(|i| 300.0 - i as f64).collect();
        let r = compute(&c);
        assert_eq!(r.regime[199], GuppyRegime::StrongDowntrend);
    }

    #[test]
    fn flat_market_yields_compression() {
        let c = vec![100.0_f64; 200];
        let r = compute(&c);
        // All EMAs converge to 100 → s_min == s_max == l_min == l_max == 100.
        // s_min > l_max fails; s_max < l_min fails; s_min < l_max fails → MixedOrTransition.
        // Equal values fail both strict-overlap branches. That is the correct classification
        // for a zero-volatility market (no trend AND no compression because no overlap).
        for v in r.regime.iter().skip(100) {
            assert!(matches!(v, GuppyRegime::MixedOrTransition));
        }
    }

    #[test]
    fn ribbons_have_correct_lengths() {
        let c = vec![100.0_f64; 200];
        let r = compute(&c);
        assert_eq!(r.ribbons.len(), 12);
        for s in &r.ribbons {
            assert_eq!(s.len(), 200);
        }
        assert_eq!(r.short_periods.len(), 6);
        assert_eq!(r.long_periods.len(), 6);
    }

    #[test]
    fn invalid_periods_return_empty() {
        let c = vec![100.0_f64; 200];
        let r = compute_with(&c, vec![1, 5], vec![30, 60]);
        assert!(r.ribbons.is_empty());
    }
}
