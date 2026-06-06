//! Quanto option — foreign-asset, domestic-payoff European option.
//!
//! Pays max(S_T − K, 0) (call) in DOMESTIC currency at a *fixed* exchange
//! rate, even though S is denominated in FOREIGN currency. Standard
//! quanto Black-Scholes:
//!
//!   r_d  = domestic risk-free rate
//!   r_f  = foreign risk-free rate
//!   q    = foreign asset dividend yield
//!   σ_S  = foreign asset return vol
//!   σ_FX = exchange-rate return vol
//!   ρ    = correlation between asset and FX return
//!
//! Effective drift in the foreign measure adjusted to domestic:
//!   μ = r_f − q − ρ · σ_S · σ_FX
//!
//!   d1 = [ln(S/K) + (μ + 0.5·σ_S²)·T] / (σ_S·√T)
//!   d2 = d1 − σ_S·√T
//!   call = e^{−r_d · T} · [S · e^{μ·T} · N(d1) − K · N(d2)]
//!   put  = e^{−r_d · T} · [K · N(−d2) − S · e^{μ·T} · N(−d1)]
//!
//! Pure compute. ρ < 0 raises call price for a call buyer (the quanto
//! adjustment compensates them for negative comovement).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionKind {
    Call,
    Put,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct QuantoReport {
    pub price: f64,
    pub effective_drift: f64,
    pub d1: f64,
    pub d2: f64,
}

#[allow(clippy::too_many_arguments)] // Canonical signature
pub fn price(
    spot: f64,
    strike: f64,
    time_to_expiry: f64,
    rate_domestic: f64,
    rate_foreign: f64,
    dividend_yield: f64,
    sigma_asset: f64,
    sigma_fx: f64,
    correlation_asset_fx: f64,
    kind: OptionKind,
) -> Option<QuantoReport> {
    if !spot.is_finite()
        || spot <= 0.0
        || !strike.is_finite()
        || strike <= 0.0
        || !time_to_expiry.is_finite()
        || time_to_expiry <= 0.0
        || !rate_domestic.is_finite()
        || !rate_foreign.is_finite()
        || !dividend_yield.is_finite()
        || !sigma_asset.is_finite()
        || sigma_asset <= 0.0
        || !sigma_fx.is_finite()
        || sigma_fx < 0.0
        || !correlation_asset_fx.is_finite()
        || !(-1.0..=1.0).contains(&correlation_asset_fx)
    {
        return None;
    }
    let mu = rate_foreign - dividend_yield - correlation_asset_fx * sigma_asset * sigma_fx;
    let sqrt_t = time_to_expiry.sqrt();
    let d1 = ((spot / strike).ln() + (mu + 0.5 * sigma_asset * sigma_asset) * time_to_expiry)
        / (sigma_asset * sqrt_t);
    let d2 = d1 - sigma_asset * sqrt_t;
    let nd1 = norm_cdf(d1);
    let nd2 = norm_cdf(d2);
    let dr_d = (-rate_domestic * time_to_expiry).exp();
    let drift_factor = (mu * time_to_expiry).exp();
    let p = match kind {
        OptionKind::Call => dr_d * (spot * drift_factor * nd1 - strike * nd2),
        OptionKind::Put => dr_d * (strike * (1.0 - nd2) - spot * drift_factor * (1.0 - nd1)),
    };
    if !p.is_finite() {
        return None;
    }
    Some(QuantoReport {
        price: p.max(0.0),
        effective_drift: mu,
        d1,
        d2,
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
            assert!(price(
                bad,
                100.0,
                0.5,
                0.03,
                0.01,
                0.0,
                0.2,
                0.1,
                0.3,
                OptionKind::Call
            )
            .is_none());
            assert!(price(
                100.0,
                bad,
                0.5,
                0.03,
                0.01,
                0.0,
                0.2,
                0.1,
                0.3,
                OptionKind::Call
            )
            .is_none());
            assert!(price(
                100.0,
                100.0,
                bad,
                0.03,
                0.01,
                0.0,
                0.2,
                0.1,
                0.3,
                OptionKind::Call
            )
            .is_none());
            assert!(price(
                100.0,
                100.0,
                0.5,
                0.03,
                0.01,
                0.0,
                bad,
                0.1,
                0.3,
                OptionKind::Call
            )
            .is_none());
        }
        // Correlation out of [-1, 1].
        assert!(price(
            100.0,
            100.0,
            0.5,
            0.03,
            0.01,
            0.0,
            0.2,
            0.1,
            1.5,
            OptionKind::Call
        )
        .is_none());
        assert!(price(
            100.0,
            100.0,
            0.5,
            0.03,
            0.01,
            0.0,
            0.2,
            0.1,
            -1.5,
            OptionKind::Call
        )
        .is_none());
    }

    #[test]
    fn zero_correlation_collapses_to_drift_minus_rf() {
        // ρ = 0 → quanto-adjusted drift μ = r_f − q (no correlation penalty).
        let r = price(
            100.0,
            100.0,
            1.0,
            0.03,
            0.01,
            0.0,
            0.20,
            0.10,
            0.0,
            OptionKind::Call,
        )
        .unwrap();
        assert!((r.effective_drift - (0.01 - 0.0)).abs() < 1e-12);
    }

    #[test]
    fn negative_correlation_raises_call_price() {
        let common = (100.0, 100.0, 1.0, 0.03, 0.01, 0.0, 0.20, 0.10);
        let r_pos = price(
            common.0,
            common.1,
            common.2,
            common.3,
            common.4,
            common.5,
            common.6,
            common.7,
            0.5,
            OptionKind::Call,
        )
        .unwrap();
        let r_neg = price(
            common.0,
            common.1,
            common.2,
            common.3,
            common.4,
            common.5,
            common.6,
            common.7,
            -0.5,
            OptionKind::Call,
        )
        .unwrap();
        assert!(
            r_neg.price > r_pos.price,
            "neg corr → bigger μ → bigger call: pos={} neg={}",
            r_pos.price,
            r_neg.price
        );
    }

    #[test]
    fn at_the_money_call_price_positive() {
        let r = price(
            100.0,
            100.0,
            0.5,
            0.03,
            0.01,
            0.0,
            0.20,
            0.10,
            0.3,
            OptionKind::Call,
        )
        .unwrap();
        assert!(r.price > 0.0);
    }

    #[test]
    fn deep_itm_call_approaches_intrinsic_discounted() {
        let r_d = 0.03;
        let t = 0.25;
        let r = price(
            200.0,
            100.0,
            t,
            r_d,
            0.01,
            0.0,
            0.20,
            0.10,
            0.0,
            OptionKind::Call,
        )
        .unwrap();
        // Lower bound: discounted intrinsic ≈ 100 · e^{-r_d·T} = 99.25.
        assert!(r.price > 90.0);
    }

    #[test]
    fn quanto_put_call_parity_holds() {
        // c − p = e^{−r_d·T} · (S · e^{μ·T} − K).
        let s = 100.0;
        let k = 105.0;
        let t = 0.5;
        let r_d = 0.03;
        let r_f = 0.01;
        let q = 0.0;
        let s_a = 0.20;
        let s_fx = 0.10;
        let rho = 0.3;
        let c = price(s, k, t, r_d, r_f, q, s_a, s_fx, rho, OptionKind::Call).unwrap();
        let p = price(s, k, t, r_d, r_f, q, s_a, s_fx, rho, OptionKind::Put).unwrap();
        let mu = r_f - q - rho * s_a * s_fx;
        let parity = (-r_d * t).exp() * (s * (mu * t).exp() - k);
        assert!((c.price - p.price - parity).abs() < 1e-9);
    }

    #[test]
    fn higher_vol_asset_inflates_atm_call() {
        let r_low = price(
            100.0,
            100.0,
            0.5,
            0.03,
            0.01,
            0.0,
            0.10,
            0.10,
            0.3,
            OptionKind::Call,
        )
        .unwrap();
        let r_high = price(
            100.0,
            100.0,
            0.5,
            0.03,
            0.01,
            0.0,
            0.40,
            0.10,
            0.3,
            OptionKind::Call,
        )
        .unwrap();
        assert!(r_high.price > r_low.price);
    }
}
