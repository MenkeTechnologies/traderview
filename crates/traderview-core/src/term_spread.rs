//! Term Spread — yield-curve slope between two maturities.
//!
//!   spread_t = long_yield_t - short_yield_t   (basis points)
//!
//! Classic recession-prediction signal:
//!   spread > 0       → normal positive slope (growth regime)
//!   spread < 0       → INVERSION (recession signal — historically
//!                      precedes recessions by 6-18 months in U.S.
//!                      Treasury 10y-2y or 10y-3m)
//!   spread > 200 bps → steepening (post-recession recovery)
//!
//! Returns per-bar spread + a 5-state regime classifier and how long
//! the curve has been inverted (in consecutive bars).
//!
//! Pure compute. Companion to `yield_curve_bootstrap`,
//! `nelson_siegel`, `nelson_siegel_svensson`, `breakeven_inflation`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TermSpreadRegime {
    #[default]
    Normal,
    Flat,
    Inverted,
    SteepNormal,
    SteepInverted,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TermSpreadReport {
    pub spread_bps: Vec<f64>,
    pub regime: Vec<TermSpreadRegime>,
    pub inversion_run_bars: Vec<u32>,
}

pub fn compute(
    short_yield_pct: &[f64],
    long_yield_pct: &[f64],
) -> TermSpreadReport {
    let n = short_yield_pct.len();
    let mut report = TermSpreadReport {
        spread_bps: vec![0.0; n],
        regime: vec![TermSpreadRegime::Normal; n],
        inversion_run_bars: vec![0; n],
    };
    if n == 0 || long_yield_pct.len() != n { return report; }
    if short_yield_pct.iter().chain(long_yield_pct.iter()).any(|x| !x.is_finite()) {
        return report;
    }
    let mut run = 0_u32;
    for i in 0..n {
        let spread_bps = (long_yield_pct[i] - short_yield_pct[i]) * 100.0;
        report.spread_bps[i] = spread_bps;
        report.regime[i] = classify(spread_bps);
        run = if spread_bps < 0.0 { run + 1 } else { 0 };
        report.inversion_run_bars[i] = run;
    }
    report
}

fn classify(spread_bps: f64) -> TermSpreadRegime {
    if spread_bps >= 200.0 { TermSpreadRegime::SteepNormal }
    else if spread_bps >= 25.0 { TermSpreadRegime::Normal }
    else if spread_bps > -25.0 { TermSpreadRegime::Flat }
    else if spread_bps > -100.0 { TermSpreadRegime::Inverted }
    else { TermSpreadRegime::SteepInverted }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        let r = compute(&[], &[]);
        assert!(r.spread_bps.is_empty());
    }

    #[test]
    fn mismatched_lengths_return_empty() {
        let r = compute(&[1.0_f64; 5], &[2.0_f64; 4]);
        assert!(r.spread_bps.iter().all(|x| *x == 0.0));
    }

    #[test]
    fn nan_returns_empty() {
        let r = compute(&[f64::NAN; 5], &[2.0_f64; 5]);
        assert!(r.spread_bps.iter().all(|x| *x == 0.0));
    }

    #[test]
    fn positive_spread_calculated_correctly() {
        // long=4.0%, short=2.0% → spread = 2.0pp = 200 bps.
        let s = vec![2.0_f64; 5];
        let l = vec![4.0_f64; 5];
        let r = compute(&s, &l);
        for i in 0..5 {
            assert!((r.spread_bps[i] - 200.0).abs() < 1e-9);
        }
    }

    #[test]
    fn inversion_run_counts_consecutive_bars() {
        let s = vec![3.0, 3.0, 3.0, 3.0, 1.0];
        let l = vec![2.0, 2.0, 2.0, 2.0, 4.0];
        let r = compute(&s, &l);
        assert_eq!(r.inversion_run_bars[0], 1);
        assert_eq!(r.inversion_run_bars[1], 2);
        assert_eq!(r.inversion_run_bars[2], 3);
        assert_eq!(r.inversion_run_bars[3], 4);
        assert_eq!(r.inversion_run_bars[4], 0);    // spread positive
    }

    #[test]
    fn classify_branches() {
        assert_eq!(classify(250.0), TermSpreadRegime::SteepNormal);
        assert_eq!(classify(100.0), TermSpreadRegime::Normal);
        assert_eq!(classify(0.0), TermSpreadRegime::Flat);
        assert_eq!(classify(-50.0), TermSpreadRegime::Inverted);
        assert_eq!(classify(-150.0), TermSpreadRegime::SteepInverted);
    }

    #[test]
    fn output_lengths_match_input() {
        let s = vec![2.0_f64; 10];
        let l = vec![4.0_f64; 10];
        let r = compute(&s, &l);
        assert_eq!(r.spread_bps.len(), 10);
        assert_eq!(r.regime.len(), 10);
        assert_eq!(r.inversion_run_bars.len(), 10);
    }
}
