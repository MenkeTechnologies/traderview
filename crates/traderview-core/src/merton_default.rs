//! Merton structural credit model — treats a firm's equity as a call option on
//! its assets and derives the distance to default and the probability of default.
//! With asset value V, asset volatility σ, debt face D maturing in T years, and
//! rate r, the distance to default is `DD = (ln(V/D) + (r − ½σ²)T) / (σ√T)` and the
//! (risk-neutral) default probability is `PD = N(−DD)`. Reports DD, PD, and the
//! leverage. Pure compute; a structural approximation, not a rating. Not advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct MertonInput {
    /// Market value of the firm's assets.
    pub asset_value_usd: f64,
    /// Face value of debt due at the horizon.
    pub debt_face_usd: f64,
    /// Asset volatility (annual, decimal, e.g. 0.30).
    pub asset_volatility: f64,
    /// Risk-free rate (decimal).
    #[serde(default)]
    pub risk_free_rate: f64,
    /// Horizon in years.
    #[serde(default = "default_horizon")]
    pub horizon_years: f64,
}

fn default_horizon() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct MertonReport {
    /// Distance to default, in standard deviations.
    pub distance_to_default: f64,
    /// Probability of default over the horizon, percent.
    pub probability_of_default_pct: f64,
    /// Debt ÷ assets, percent.
    pub leverage_pct: f64,
    pub valid: bool,
}

fn norm_cdf(x: f64) -> f64 {
    0.5 * erfc(-x / std::f64::consts::SQRT_2)
}

/// Complementary error function (Abramowitz & Stegun 7.1.26, max err ~1.5e-7).
fn erfc(x: f64) -> f64 {
    let z = x.abs();
    let t = 1.0 / (1.0 + 0.5 * z);
    let tau = t
        * (-z * z - 1.26551223
            + t * (1.00002368
                + t * (0.37409196
                    + t * (0.09678418
                        + t * (-0.18628806
                            + t * (0.27886807
                                + t * (-1.13520398
                                    + t * (1.48851587
                                        + t * (-0.82215223 + t * 0.17087277)))))))))
        .exp();
    if x >= 0.0 {
        tau
    } else {
        2.0 - tau
    }
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

pub fn generate(i: &MertonInput) -> MertonReport {
    if i.asset_value_usd <= 0.0
        || i.debt_face_usd <= 0.0
        || i.asset_volatility <= 0.0
        || i.horizon_years <= 0.0
    {
        return MertonReport::default();
    }
    let sig = i.asset_volatility;
    let t = i.horizon_years;
    let dd = ((i.asset_value_usd / i.debt_face_usd).ln()
        + (i.risk_free_rate - 0.5 * sig * sig) * t)
        / (sig * t.sqrt());
    let pd = norm_cdf(-dd);
    MertonReport {
        distance_to_default: round4(dd),
        probability_of_default_pct: round4(pd * 100.0),
        leverage_pct: round4(i.debt_face_usd / i.asset_value_usd * 100.0),
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.02
    }

    fn base() -> MertonInput {
        MertonInput {
            asset_value_usd: 100.0,
            debt_face_usd: 80.0,
            asset_volatility: 0.30,
            risk_free_rate: 0.05,
            horizon_years: 1.0,
        }
    }

    #[test]
    fn distance_and_pd() {
        let d = generate(&base());
        assert!(d.valid);
        assert!(close(d.distance_to_default, 0.7605));
        assert!(close(d.probability_of_default_pct, 22.35));
        assert!(close(d.leverage_pct, 80.0));
    }

    #[test]
    fn lower_leverage_lower_pd() {
        let d = generate(&MertonInput { debt_face_usd: 50.0, ..base() });
        assert!(d.distance_to_default > 0.7605);
        assert!(d.probability_of_default_pct < 22.35);
    }

    #[test]
    fn higher_vol_higher_pd() {
        let d = generate(&MertonInput { asset_volatility: 0.60, ..base() });
        assert!(d.probability_of_default_pct > 22.35);
    }

    #[test]
    fn invalid_inputs() {
        assert!(!generate(&MertonInput { asset_volatility: 0.0, ..base() }).valid);
        assert!(!generate(&MertonInput { debt_face_usd: 0.0, ..base() }).valid);
    }
}
