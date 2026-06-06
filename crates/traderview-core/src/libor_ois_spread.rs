//! LIBOR–OIS Spread (TED-style bank funding stress indicator).
//!
//! The unsecured interbank funding rate (historically 3M LIBOR, now
//! its successor SOFR-term or BSBY) minus the overnight-indexed swap
//! rate of the same maturity. Captures bank credit + liquidity
//! premium:
//!
//!   spread_bps = (libor_or_term_rate − ois_rate) · 10_000
//!
//! Stress thresholds (typical equity-quant practice):
//!   - <  25 bps → benign
//!   - 25–50 bps → elevated
//!   - 50–100 bps → stress
//!   - > 100 bps → crisis (2008-style)
//!
//! Time-series report aggregates min/max/mean and flags days exceeding
//! a configurable stress threshold.
//!
//! Pure compute. Companion to `cross_currency_basis`, `breakeven_inflation`,
//! `swap_valuation`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DailyRate {
    pub libor_or_term_rate: f64,
    pub ois_rate: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StressLevel {
    #[default]
    Benign,
    Elevated,
    Stress,
    Crisis,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LiborOisReport {
    pub per_day_spread_bps: Vec<f64>,
    pub stress_levels: Vec<StressLevel>,
    pub mean_spread_bps: f64,
    pub max_spread_bps: f64,
    pub min_spread_bps: f64,
    pub days_in_stress: usize,
    pub days_in_crisis: usize,
    pub n_days: usize,
}

pub fn compute(daily_rates: &[DailyRate]) -> Option<LiborOisReport> {
    if daily_rates.is_empty() {
        return None;
    }
    if daily_rates
        .iter()
        .any(|r| !r.libor_or_term_rate.is_finite() || !r.ois_rate.is_finite())
    {
        return None;
    }
    let spreads_bps: Vec<f64> = daily_rates
        .iter()
        .map(|r| (r.libor_or_term_rate - r.ois_rate) * 10_000.0)
        .collect();
    let levels: Vec<StressLevel> = spreads_bps
        .iter()
        .map(|s| {
            if *s < 25.0 {
                StressLevel::Benign
            } else if *s < 50.0 {
                StressLevel::Elevated
            } else if *s < 100.0 {
                StressLevel::Stress
            } else {
                StressLevel::Crisis
            }
        })
        .collect();
    let n = spreads_bps.len();
    let n_f = n as f64;
    let mean: f64 = spreads_bps.iter().sum::<f64>() / n_f;
    let max = spreads_bps
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let min = spreads_bps.iter().cloned().fold(f64::INFINITY, f64::min);
    let stress = levels
        .iter()
        .filter(|l| matches!(l, StressLevel::Stress | StressLevel::Crisis))
        .count();
    let crisis = levels
        .iter()
        .filter(|l| matches!(l, StressLevel::Crisis))
        .count();
    Some(LiborOisReport {
        per_day_spread_bps: spreads_bps,
        stress_levels: levels,
        mean_spread_bps: mean,
        max_spread_bps: max,
        min_spread_bps: min,
        days_in_stress: stress,
        days_in_crisis: crisis,
        n_days: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(libor: f64, ois: f64) -> DailyRate {
        DailyRate {
            libor_or_term_rate: libor,
            ois_rate: ois,
        }
    }

    #[test]
    fn empty_returns_none() {
        assert!(compute(&[]).is_none());
    }

    #[test]
    fn nan_returns_none() {
        assert!(compute(&[d(0.05, f64::NAN)]).is_none());
    }

    #[test]
    fn benign_spread_classified() {
        // 10 bps spread → Benign.
        let rates = vec![d(0.051, 0.050); 10];
        let r = compute(&rates).unwrap();
        assert!(r.per_day_spread_bps.iter().all(|s| (s - 10.0).abs() < 1e-9));
        assert!(r
            .stress_levels
            .iter()
            .all(|l| matches!(l, StressLevel::Benign)));
        assert_eq!(r.days_in_stress, 0);
    }

    #[test]
    fn crisis_spread_flagged() {
        // 150 bps spread → Crisis.
        let rates = vec![d(0.065, 0.050); 5];
        let r = compute(&rates).unwrap();
        assert!(r
            .stress_levels
            .iter()
            .all(|l| matches!(l, StressLevel::Crisis)));
        assert_eq!(r.days_in_crisis, 5);
    }

    #[test]
    fn mixed_regime_aggregated() {
        let rates = vec![
            d(0.0505, 0.050), // 5 bps benign
            d(0.0530, 0.050), // 30 bps elevated
            d(0.0570, 0.050), // 70 bps stress
            d(0.0660, 0.050), // 160 bps crisis
        ];
        let r = compute(&rates).unwrap();
        assert_eq!(r.days_in_stress, 2); // stress + crisis count as stress
        assert_eq!(r.days_in_crisis, 1);
        assert!(r.max_spread_bps > 150.0);
        assert!(r.min_spread_bps < 10.0);
    }

    #[test]
    fn mean_spread_correct() {
        let rates = vec![
            d(0.0510, 0.0500), // 10 bps
            d(0.0520, 0.0500), // 20 bps
        ];
        let r = compute(&rates).unwrap();
        assert!((r.mean_spread_bps - 15.0).abs() < 1e-9);
    }

    #[test]
    fn n_days_reported() {
        let rates = vec![d(0.0510, 0.0500); 25];
        let r = compute(&rates).unwrap();
        assert_eq!(r.n_days, 25);
    }
}
