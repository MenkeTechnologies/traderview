//! TED Spread — 3-month Treasury bill rate minus 3-month interbank
//! borrowing rate (historically LIBOR, now SOFR-term or similar).
//!
//!   ted_spread_bps = (interbank_rate − tbill_rate) · 10_000
//!
//! Classic counterparty-credit-risk stress indicator. Stress thresholds
//! per the 2008-era literature:
//!   - <  25 bps → benign
//!   - 25–50 bps → elevated
//!   - 50–100 bps → stress
//!   - > 100 bps → crisis (2008-style)
//!
//! Distinct from LIBOR-OIS (which compares interbank to OIS, isolating
//! pure bank-credit risk). TED conflates bank credit + flight-to-quality
//! to Treasuries (denominator).
//!
//! Pure compute. Companion to `libor_ois_spread`, `breakeven_inflation`,
//! `cross_currency_basis`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DailyRate {
    pub interbank_rate: f64,
    pub treasury_bill_rate: f64,
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
pub struct TedSpreadReport {
    pub per_day_spread_bps: Vec<f64>,
    pub stress_levels: Vec<StressLevel>,
    pub mean_spread_bps: f64,
    pub max_spread_bps: f64,
    pub min_spread_bps: f64,
    pub days_in_stress: usize,
    pub days_in_crisis: usize,
    pub n_days: usize,
}

pub fn compute(daily_rates: &[DailyRate]) -> Option<TedSpreadReport> {
    if daily_rates.is_empty() {
        return None;
    }
    if daily_rates
        .iter()
        .any(|r| !r.interbank_rate.is_finite() || !r.treasury_bill_rate.is_finite())
    {
        return None;
    }
    let spreads_bps: Vec<f64> = daily_rates
        .iter()
        .map(|r| (r.interbank_rate - r.treasury_bill_rate) * 10_000.0)
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
    Some(TedSpreadReport {
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

    fn d(ib: f64, tb: f64) -> DailyRate {
        DailyRate {
            interbank_rate: ib,
            treasury_bill_rate: tb,
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
    fn benign_regime_classified() {
        let rates = vec![d(0.0510, 0.0500); 5]; // 10 bps
        let r = compute(&rates).unwrap();
        assert!(r
            .stress_levels
            .iter()
            .all(|l| matches!(l, StressLevel::Benign)));
    }

    #[test]
    fn crisis_regime_flagged() {
        let rates = vec![d(0.0650, 0.0500); 5]; // 150 bps
        let r = compute(&rates).unwrap();
        assert!(r
            .stress_levels
            .iter()
            .all(|l| matches!(l, StressLevel::Crisis)));
        assert_eq!(r.days_in_crisis, 5);
    }

    #[test]
    fn mean_correctly_computed() {
        let rates = vec![d(0.0510, 0.0500), d(0.0530, 0.0500)];
        let r = compute(&rates).unwrap();
        assert!((r.mean_spread_bps - 20.0).abs() < 1e-9);
    }

    #[test]
    fn min_max_reported() {
        let rates = vec![
            d(0.0505, 0.0500), // 5 bps
            d(0.0660, 0.0500), // 160 bps
        ];
        let r = compute(&rates).unwrap();
        assert!((r.min_spread_bps - 5.0).abs() < 1e-9);
        assert!((r.max_spread_bps - 160.0).abs() < 1e-9);
    }

    #[test]
    fn n_days_reported() {
        let rates = vec![d(0.0510, 0.0500); 30];
        let r = compute(&rates).unwrap();
        assert_eq!(r.n_days, 30);
    }
}
