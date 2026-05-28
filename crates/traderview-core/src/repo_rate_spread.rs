//! Repo Rate Spread — overnight repo rate minus target / IOR rate.
//!
//! Tracks funding-market stress. Spread > 0 = repo above target (cash
//! is tight, demand for short-term borrowing exceeds supply). Spread
//! >> 0 = stress regime (Sep 2019 repo spike, Mar 2020 dash-for-cash).
//!
//!   spread_bps = (repo_rate - target_rate) · 10000
//!
//! Also returns a 5-state classifier:
//!   Easy           : spread < -10 bps
//!   Normal         : -10 ≤ spread < 10
//!   MildlyTight    : 10 ≤ spread < 25
//!   Tight          : 25 ≤ spread < 75
//!   StressedSpike  : spread ≥ 75
//!
//! Pure compute. Companion to `libor_ois_spread`, `cross_currency_basis`,
//! `breakeven_inflation`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RepoRegime {
    #[default]
    Normal,
    Easy,
    MildlyTight,
    Tight,
    StressedSpike,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepoRateSpreadReport {
    pub spread_bps: Vec<f64>,
    pub regime: Vec<RepoRegime>,
    pub max_spread_bps: f64,
    pub days_in_stress: usize,
}

pub fn compute(repo_rate: &[f64], target_rate: &[f64]) -> RepoRateSpreadReport {
    let n = repo_rate.len();
    let mut report = RepoRateSpreadReport {
        spread_bps: vec![0.0; n],
        regime: vec![RepoRegime::Normal; n],
        max_spread_bps: 0.0,
        days_in_stress: 0,
    };
    if n == 0 || target_rate.len() != n { return report; }
    if repo_rate.iter().chain(target_rate.iter()).any(|x| !x.is_finite()) { return report; }
    let mut max_spread = f64::NEG_INFINITY;
    let mut stress_count = 0_usize;
    for i in 0..n {
        let spread = (repo_rate[i] - target_rate[i]) * 10000.0;
        report.spread_bps[i] = spread;
        report.regime[i] = classify(spread);
        if spread > max_spread { max_spread = spread; }
        if matches!(report.regime[i], RepoRegime::Tight | RepoRegime::StressedSpike) {
            stress_count += 1;
        }
    }
    report.max_spread_bps = max_spread;
    report.days_in_stress = stress_count;
    report
}

fn classify(spread_bps: f64) -> RepoRegime {
    if spread_bps >= 75.0 { RepoRegime::StressedSpike }
    else if spread_bps >= 25.0 { RepoRegime::Tight }
    else if spread_bps >= 10.0 { RepoRegime::MildlyTight }
    else if spread_bps < -10.0 { RepoRegime::Easy }
    else { RepoRegime::Normal }
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
        let r = compute(&[0.05; 5], &[0.04; 4]);
        assert!(r.spread_bps.iter().all(|s| *s == 0.0));
    }

    #[test]
    fn nan_returns_empty() {
        let r = compute(&[f64::NAN; 3], &[0.04; 3]);
        assert!(r.spread_bps.iter().all(|s| *s == 0.0));
    }

    #[test]
    fn spread_calculated_correctly() {
        // Repo 4.5%, target 4.25% → 25 bps spread.
        let r = compute(&[0.045; 5], &[0.0425; 5]);
        for &s in &r.spread_bps {
            assert!((s - 25.0).abs() < 1e-6);
        }
    }

    #[test]
    fn classify_branches() {
        assert_eq!(classify(-20.0), RepoRegime::Easy);
        assert_eq!(classify(0.0), RepoRegime::Normal);
        assert_eq!(classify(15.0), RepoRegime::MildlyTight);
        assert_eq!(classify(50.0), RepoRegime::Tight);
        assert_eq!(classify(150.0), RepoRegime::StressedSpike);
    }

    #[test]
    fn stress_count_aggregates() {
        let r = compute(
            &[0.045, 0.05, 0.055, 0.0425],
            &[0.04, 0.04, 0.04, 0.0425],
        );
        // Spreads: 50, 100, 150, 0 → 3 stress (Tight or above).
        assert_eq!(r.days_in_stress, 3);
        assert!((r.max_spread_bps - 150.0).abs() < 1e-6);
    }
}
