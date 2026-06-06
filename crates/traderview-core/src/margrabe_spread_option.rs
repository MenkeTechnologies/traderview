//! Margrabe (1978) spread option — closed-form European exchange option.
//!
//! Right to exchange asset 2 for asset 1 at expiry: payoff = max(S1_T − S2_T, 0).
//! Formula:
//!
//!   σ = √(σ₁² + σ₂² − 2·ρ·σ₁·σ₂)
//!   d1 = [ln(S1·e^{−q1·T} / (S2·e^{−q2·T})) + 0.5·σ²·T] / (σ·√T)
//!   d2 = d1 − σ·√T
//!   price = S1·e^{−q1·T}·N(d1) − S2·e^{−q2·T}·N(d2)
//!
//! Note: discount rate cancels out because both legs are risky assets;
//! only the dividend yields appear. Generalizes Black-Scholes (let S2
//! be the strike with σ₂ = 0 and the formula collapses to a vanilla
//! call — but use the dedicated `iv_solver` / `greeks` for that).
//!
//! Pure compute. Common use: crack-spread (gasoline vs crude), spark
//! spread (electricity vs gas), best-of basket / outperformance options.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MargrabeReport {
    pub price: f64,
    pub combined_vol: f64,
    pub d1: f64,
    pub d2: f64,
    pub delta_s1: f64,
    pub delta_s2: f64,
}

#[allow(clippy::too_many_arguments)] // Margrabe formula signature is canonical; struct-bundling adds noise
                                     // without changing what the caller has to compute.
pub fn price(
    s1: f64,
    s2: f64,
    sigma1: f64,
    sigma2: f64,
    correlation: f64,
    q1: f64,
    q2: f64,
    time_to_expiry: f64,
) -> Option<MargrabeReport> {
    if !s1.is_finite()
        || s1 <= 0.0
        || !s2.is_finite()
        || s2 <= 0.0
        || !sigma1.is_finite()
        || sigma1 < 0.0
        || !sigma2.is_finite()
        || sigma2 < 0.0
        || !correlation.is_finite()
        || !(-1.0..=1.0).contains(&correlation)
        || !q1.is_finite()
        || !q2.is_finite()
        || !time_to_expiry.is_finite()
        || time_to_expiry <= 0.0
    {
        return None;
    }
    let sigma2_combined = sigma1.powi(2) + sigma2.powi(2) - 2.0 * correlation * sigma1 * sigma2;
    if sigma2_combined < 0.0 {
        return None;
    }
    let sigma = sigma2_combined.sqrt();
    let sqrt_t = time_to_expiry.sqrt();
    let dq1 = (-q1 * time_to_expiry).exp();
    let dq2 = (-q2 * time_to_expiry).exp();
    if sigma == 0.0 || sqrt_t == 0.0 {
        // Degenerate (assets perfectly correlated + same vol or zero-T):
        // payoff is deterministic.
        let intrinsic = (s1 * dq1 - s2 * dq2).max(0.0);
        return Some(MargrabeReport {
            price: intrinsic,
            combined_vol: sigma,
            d1: f64::INFINITY,
            d2: f64::INFINITY,
            delta_s1: if s1 * dq1 > s2 * dq2 { dq1 } else { 0.0 },
            delta_s2: if s1 * dq1 > s2 * dq2 { -dq2 } else { 0.0 },
        });
    }
    let s1_pv = s1 * dq1;
    let s2_pv = s2 * dq2;
    let d1 = ((s1_pv / s2_pv).ln() + 0.5 * sigma2_combined * time_to_expiry) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;
    let nd1 = norm_cdf(d1);
    let nd2 = norm_cdf(d2);
    let price = s1_pv * nd1 - s2_pv * nd2;
    Some(MargrabeReport {
        price,
        combined_vol: sigma,
        d1,
        d2,
        delta_s1: dq1 * nd1,
        delta_s2: -dq2 * nd2,
    })
}

fn norm_cdf(x: f64) -> f64 {
    let a1 = 0.254829592_f64;
    let a2 = -0.284496736_f64;
    let a3 = 1.421413741_f64;
    let a4 = -1.453152027_f64;
    let a5 = 1.061405429_f64;
    let p = 0.3275911_f64;
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let xa = x.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + p * xa);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-xa * xa).exp();
    0.5 * (1.0 + sign * y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        for bad in [0.0, -1.0, f64::NAN] {
            assert!(price(bad, 100.0, 0.2, 0.25, 0.3, 0.0, 0.0, 0.5).is_none());
            assert!(price(100.0, bad, 0.2, 0.25, 0.3, 0.0, 0.0, 0.5).is_none());
            assert!(price(100.0, 100.0, 0.2, 0.25, 0.3, 0.0, 0.0, bad).is_none());
        }
        assert!(price(100.0, 100.0, -0.2, 0.25, 0.3, 0.0, 0.0, 0.5).is_none());
        assert!(price(100.0, 100.0, 0.2, 0.25, 1.5, 0.0, 0.0, 0.5).is_none());
        assert!(price(100.0, 100.0, 0.2, 0.25, -1.5, 0.0, 0.0, 0.5).is_none());
    }

    #[test]
    fn at_the_money_equal_vol_equal_correlation_price_positive() {
        let r = price(100.0, 100.0, 0.20, 0.25, 0.5, 0.0, 0.0, 0.5).unwrap();
        assert!(r.price > 0.0);
        assert!(r.combined_vol > 0.0);
    }

    #[test]
    fn perfect_positive_correlation_same_vol_yields_zero_volatility() {
        // σ_combined = √(σ₁² + σ₂² - 2σ₁σ₂) = 0 when σ₁=σ₂ and ρ=1.
        let r = price(100.0, 100.0, 0.20, 0.20, 1.0, 0.0, 0.0, 0.5).unwrap();
        assert!(r.combined_vol.abs() < 1e-12);
        // Spread is deterministic and equal → payoff = 0 (S1·dq1 == S2·dq2).
        assert_eq!(r.price, 0.0);
    }

    #[test]
    fn perfect_negative_correlation_inflates_combined_vol() {
        // Compare two scenarios that differ ONLY in correlation. Holding
        // q1/q2 + T fixed eliminates asymmetric-discount confounding.
        let r_neg = price(100.0, 100.0, 0.20, 0.20, -1.0, 0.0, 0.0, 0.5).unwrap();
        let r_zero = price(100.0, 100.0, 0.20, 0.20, 0.0, 0.0, 0.0, 0.5).unwrap();
        assert!(r_neg.combined_vol > r_zero.combined_vol);
        assert!(r_neg.price > r_zero.price);
    }

    #[test]
    fn higher_s1_yields_higher_price() {
        let r_low = price(95.0, 100.0, 0.20, 0.25, 0.5, 0.0, 0.0, 0.5).unwrap();
        let r_high = price(110.0, 100.0, 0.20, 0.25, 0.5, 0.0, 0.0, 0.5).unwrap();
        assert!(r_high.price > r_low.price);
    }

    #[test]
    fn delta_s1_positive_delta_s2_negative_for_otm_or_atm_spread() {
        let r = price(100.0, 100.0, 0.20, 0.25, 0.3, 0.02, 0.01, 1.0).unwrap();
        assert!(r.delta_s1 > 0.0);
        assert!(r.delta_s2 < 0.0);
        // Magnitudes between 0 and the dividend discounts.
        assert!(r.delta_s1.abs() < 1.0);
        assert!(r.delta_s2.abs() < 1.0);
    }

    #[test]
    fn dividend_yields_reduce_pv_legs() {
        let r_q0 = price(100.0, 100.0, 0.20, 0.25, 0.3, 0.0, 0.0, 1.0).unwrap();
        let r_q = price(100.0, 100.0, 0.20, 0.25, 0.3, 0.05, 0.01, 1.0).unwrap();
        // Higher q1 strictly lowers the S1 leg → typically lower price.
        assert!(r_q.price < r_q0.price);
    }
}
