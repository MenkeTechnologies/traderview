//! APR ↔ APY — nominal vs effective annual rate.
//!
//! A nominal rate (APR) quoted "12% compounded monthly" isn't what you
//! actually earn or pay over a year — compounding makes the **effective**
//! annual rate (APY / EAR) higher. The more frequent the compounding, the
//! bigger the gap, up to a ceiling at continuous compounding:
//!
//!   * APY = (1 + APR/n)^n − 1
//!   * APR = n × ((1 + APY)^(1/n) − 1)
//!   * continuous-compounding APY = e^APR − 1
//!
//! Convert either direction given the compounding periods per year. Pure
//! compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    /// Input is the nominal APR; compute the effective APY.
    AprToApy,
    /// Input is the effective APY; back out the nominal APR.
    ApyToApr,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AprApyInput {
    pub direction: Direction,
    /// The input rate (percent) — interpreted per `direction`.
    pub rate_pct: f64,
    /// Compounding periods per year (1 annual, 12 monthly, 365 daily, …).
    pub periods_per_year: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AprApyResult {
    pub nominal_apr_pct: f64,
    pub effective_apy_pct: f64,
    /// APY − APR: the compounding boost.
    pub spread_pct: f64,
    /// Effective rate if compounded continuously (e^APR − 1).
    pub continuous_apy_pct: f64,
}

pub fn analyze(i: &AprApyInput) -> AprApyResult {
    let n = if i.periods_per_year > 0.0 { i.periods_per_year } else { 1.0 };
    let rate = i.rate_pct / 100.0;

    let (apr, apy) = match i.direction {
        Direction::AprToApy => {
            let apy = (1.0 + rate / n).powf(n) - 1.0;
            (rate, apy)
        }
        Direction::ApyToApr => {
            let apr = n * ((1.0 + rate).powf(1.0 / n) - 1.0);
            (apr, rate)
        }
    };

    let continuous_apy = apr.exp() - 1.0;

    AprApyResult {
        nominal_apr_pct: apr * 100.0,
        effective_apy_pct: apy * 100.0,
        spread_pct: (apy - apr) * 100.0,
        continuous_apy_pct: continuous_apy * 100.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apr_to(rate: f64, n: f64) -> AprApyResult {
        analyze(&AprApyInput { direction: Direction::AprToApy, rate_pct: rate, periods_per_year: n })
    }

    #[test]
    fn monthly_compounding_apy() {
        // 12% APR monthly → (1.01)^12 − 1 = 12.6825%.
        let r = apr_to(12.0, 12.0);
        assert!((r.effective_apy_pct - 12.682503).abs() < 1e-4);
    }

    #[test]
    fn daily_compounding_apy_higher_than_monthly() {
        let monthly = apr_to(12.0, 12.0);
        let daily = apr_to(12.0, 365.0);
        assert!(daily.effective_apy_pct > monthly.effective_apy_pct);
        assert!((daily.effective_apy_pct - 12.7474).abs() < 1e-3);
    }

    #[test]
    fn annual_compounding_apy_equals_apr() {
        let r = apr_to(12.0, 1.0);
        assert!((r.effective_apy_pct - 12.0).abs() < 1e-9);
        assert!(r.spread_pct.abs() < 1e-9);
    }

    #[test]
    fn apy_to_apr_inverts_monthly() {
        // APY 12.682503% monthly → APR 12%.
        let r = analyze(&AprApyInput {
            direction: Direction::ApyToApr,
            rate_pct: 12.682503,
            periods_per_year: 12.0,
        });
        assert!((r.nominal_apr_pct - 12.0).abs() < 1e-3);
    }

    #[test]
    fn roundtrip_apr_apy_apr() {
        let fwd = apr_to(8.0, 12.0);
        let back = analyze(&AprApyInput {
            direction: Direction::ApyToApr,
            rate_pct: fwd.effective_apy_pct,
            periods_per_year: 12.0,
        });
        assert!((back.nominal_apr_pct - 8.0).abs() < 1e-6);
    }

    #[test]
    fn continuous_apy_is_exp_minus_one() {
        // e^0.12 − 1 = 12.7497%.
        let r = apr_to(12.0, 12.0);
        assert!((r.continuous_apy_pct - 12.749685).abs() < 1e-4);
    }

    #[test]
    fn spread_is_apy_minus_apr() {
        let r = apr_to(12.0, 12.0);
        assert!((r.spread_pct - (r.effective_apy_pct - r.nominal_apr_pct)).abs() < 1e-9);
        assert!(r.spread_pct > 0.0);
    }

    #[test]
    fn zero_rate_zero_everything() {
        let r = apr_to(0.0, 12.0);
        assert!(r.effective_apy_pct.abs() < 1e-9);
        assert!(r.continuous_apy_pct.abs() < 1e-9);
    }
}
