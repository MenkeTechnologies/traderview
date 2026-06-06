//! Ehlers Instant Trendline (ITrend) — John Ehlers ("Rocket Science for
//! Traders", 2001).
//!
//! Near-zero-lag trend extractor designed to give the smoothing benefit
//! of a moving average without the standard period/2 lag. Uses the
//! Hilbert-style alpha = 2/(N+1) low-pass with a corrective term for
//! a sloping input:
//!
//!   alpha = 2 / (period + 1)
//!   ITrend_t = (alpha - alpha²/4)·x_t
//!              + 0.5·alpha²·x_{t-1}
//!              - (alpha - 3·alpha²/4)·x_{t-2}
//!              + 2·(1 - alpha)·ITrend_{t-1}
//!              - (1 - alpha)²·ITrend_{t-2}
//!
//! Also returns the "trigger" line = 2·ITrend - ITrend[2 bars ago],
//! which Ehlers uses as a faster signal that crosses ITrend to indicate
//! regime change.
//!
//! Pure compute. Companion to `ehlers_super_smoother`, `roofing_filter`,
//! `ehlers_decycler`, `kalman_filter_1d`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstantTrendlineReport {
    pub itrend: Vec<Option<f64>>,
    pub trigger: Vec<Option<f64>>,
    pub period: usize,
}

pub fn compute(series: &[f64], period: usize) -> InstantTrendlineReport {
    let n = series.len();
    let mut report = InstantTrendlineReport {
        itrend: vec![None; n],
        trigger: vec![None; n],
        period,
    };
    if period < 3 || n < 3 {
        return report;
    }
    if series.iter().any(|x| !x.is_finite()) {
        return report;
    }
    let alpha = 2.0 / (period as f64 + 1.0);
    let a2 = alpha * alpha;
    let c0 = alpha - a2 / 4.0;
    let c1 = 0.5 * a2;
    let c2 = alpha - 3.0 * a2 / 4.0;
    let d1 = 2.0 * (1.0 - alpha);
    let d2 = (1.0 - alpha).powi(2);
    let mut it = vec![0.0_f64; n];
    // Seed first two bars via simple average of first 3 inputs (Ehlers'
    // recommendation for warmup).
    let seed = (series[0]
        + 2.0 * series.get(1).copied().unwrap_or(series[0])
        + series.get(2).copied().unwrap_or(series[0]))
        / 4.0;
    for (i, slot) in it.iter_mut().enumerate().take(2.min(n)) {
        *slot = seed;
        report.itrend[i] = Some(seed);
    }
    for i in 2..n {
        it[i] = c0 * series[i] + c1 * series[i - 1] - c2 * series[i - 2] + d1 * it[i - 1]
            - d2 * it[i - 2];
        report.itrend[i] = Some(it[i]);
    }
    // Trigger = 2·itrend - itrend[i-2] (Ehlers' fast line).
    for i in 2..n {
        report.trigger[i] = Some(2.0 * it[i] - it[i - 2]);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let s = vec![100.0_f64; 50];
        let r = compute(&s, 2);
        assert!(r.itrend.iter().all(|x| x.is_none()));
        let r2 = compute(&s[..2], 10);
        assert!(r2.itrend.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut s = vec![100.0_f64; 50];
        s[5] = f64::NAN;
        let r = compute(&s, 10);
        assert!(r.itrend.iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_signal_settles_to_constant() {
        let s = vec![100.0_f64; 100];
        let r = compute(&s, 10);
        for v in r.itrend.iter().skip(20).flatten() {
            assert!((v - 100.0).abs() < 1e-6);
        }
    }

    #[test]
    fn linear_uptrend_tracks_input_at_steady_state() {
        let s: Vec<f64> = (0..200).map(|i| i as f64).collect();
        let r = compute(&s, 10);
        // Steady-state ITrend tracks linear input with bounded lag.
        let last = r.itrend[199].unwrap();
        // Allow lag up to half the period.
        assert!(
            (199.0 - last).abs() < 10.0,
            "itrend {last} should be within 10 of input 199"
        );
    }

    #[test]
    fn trigger_leads_itrend_in_uptrend() {
        let s: Vec<f64> = (0..200).map(|i| i as f64).collect();
        let r = compute(&s, 10);
        // Trigger = 2·itrend - itrend[i-2] ≈ itrend + 2 (constant slope = 1).
        let last = 199;
        let it_v = r.itrend[last].unwrap();
        let tr_v = r.trigger[last].unwrap();
        assert!(
            tr_v > it_v,
            "trigger {tr_v} should lead itrend {it_v} in steady uptrend"
        );
    }

    #[test]
    fn output_lengths_match_input() {
        let s = vec![100.0_f64; 50];
        let r = compute(&s, 10);
        assert_eq!(r.itrend.len(), 50);
        assert_eq!(r.trigger.len(), 50);
    }
}
