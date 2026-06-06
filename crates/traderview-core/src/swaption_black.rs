//! Black (1976) swaption pricing — European payer / receiver swaption.
//!
//! A payer swaption is a CALL on the forward swap rate S; a receiver
//! swaption is a PUT. Black-76 prices the option's payoff times the
//! annuity factor (which captures the swap leg's PV):
//!
//!   payer    = annuity · [S · N(d1) − K · N(d2)]
//!   receiver = annuity · [K · N(−d2) − S · N(−d1)]
//!   d1 = [ln(S/K) + 0.5·σ²·T] / (σ·√T)
//!   d2 = d1 − σ·√T
//!
//! where:
//!   S        = forward swap rate
//!   K        = strike (fixed rate)
//!   σ        = lognormal volatility of the forward swap rate
//!   T        = option expiry (years)
//!   annuity  = Σ_i τ_i · P(0, T_i)   (sum of discount-factor × accrual
//!              over the swap's payment dates after expiry)
//!
//! Caller supplies `annuity_pv01` (the annuity = "PV01" of the swap leg)
//! computed externally from the discount curve.
//!
//! Pure compute. Distinct from `caplet_black76` — that one prices a
//! single cap/floorlet on a forward LIBOR rate; this prices an option
//! on the entire swap (multi-leg) underlying.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwaptionKind {
    Payer,
    Receiver,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SwaptionReport {
    pub price: f64,
    pub d1: f64,
    pub d2: f64,
    pub forward_rate: f64,
}

#[allow(clippy::too_many_arguments)]
pub fn price(
    forward_swap_rate: f64,
    strike_rate: f64,
    sigma: f64,
    time_to_expiry: f64,
    annuity_pv01: f64,
    notional: f64,
    kind: SwaptionKind,
) -> Option<SwaptionReport> {
    if !forward_swap_rate.is_finite()
        || forward_swap_rate <= 0.0
        || !strike_rate.is_finite()
        || strike_rate <= 0.0
        || !sigma.is_finite()
        || sigma <= 0.0
        || !time_to_expiry.is_finite()
        || time_to_expiry <= 0.0
        || !annuity_pv01.is_finite()
        || annuity_pv01 <= 0.0
        || !notional.is_finite()
        || notional == 0.0
    {
        return None;
    }
    let sqrt_t = time_to_expiry.sqrt();
    let d1 = ((forward_swap_rate / strike_rate).ln() + 0.5 * sigma * sigma * time_to_expiry)
        / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;
    let nd1 = norm_cdf(d1);
    let nd2 = norm_cdf(d2);
    let payoff_per_unit = match kind {
        SwaptionKind::Payer => forward_swap_rate * nd1 - strike_rate * nd2,
        SwaptionKind::Receiver => strike_rate * (1.0 - nd2) - forward_swap_rate * (1.0 - nd1),
    };
    let p = annuity_pv01 * notional * payoff_per_unit;
    if !p.is_finite() {
        return None;
    }
    Some(SwaptionReport {
        price: p.max(0.0),
        d1,
        d2,
        forward_rate: forward_swap_rate,
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

    fn args() -> (f64, f64, f64, f64, f64, f64) {
        // 5y10y payer swaption: forward = 4%, strike = 4%, σ = 25%,
        // expiry 5y, annuity PV01 ≈ 7.5 (10y swap), notional $10M.
        (0.04, 0.04, 0.25, 5.0, 7.5, 10_000_000.0)
    }

    #[test]
    fn invalid_inputs_return_none() {
        for bad in [0.0, -1.0, f64::NAN] {
            let (f, k, s, t, a, n) = args();
            assert!(price(bad, k, s, t, a, n, SwaptionKind::Payer).is_none());
            assert!(price(f, bad, s, t, a, n, SwaptionKind::Payer).is_none());
            assert!(price(f, k, bad, t, a, n, SwaptionKind::Payer).is_none());
            assert!(price(f, k, s, bad, a, n, SwaptionKind::Payer).is_none());
            assert!(price(f, k, s, t, bad, n, SwaptionKind::Payer).is_none());
        }
        assert!(price(0.04, 0.04, 0.25, 5.0, 7.5, 0.0, SwaptionKind::Payer).is_none());
    }

    #[test]
    fn atm_payer_swaption_price_positive() {
        let (f, k, s, t, a, n) = args();
        let r = price(f, k, s, t, a, n, SwaptionKind::Payer).unwrap();
        assert!(r.price > 0.0);
    }

    #[test]
    fn payer_receiver_parity_holds() {
        // Payer − Receiver = annuity · notional · (F − K)  (Black-76 parity).
        let (f, k, s, t, a, n) = args();
        let p = price(f, k, s, t, a, n, SwaptionKind::Payer).unwrap();
        let r = price(f, k, s, t, a, n, SwaptionKind::Receiver).unwrap();
        let parity = a * n * (f - k);
        assert!((p.price - r.price - parity).abs() < 1e-6);
    }

    #[test]
    fn deep_itm_payer_approaches_intrinsic() {
        // Forward 7% vs strike 4% → ITM payer.
        let r = price(
            0.07,
            0.04,
            0.25,
            5.0,
            7.5,
            10_000_000.0,
            SwaptionKind::Payer,
        )
        .unwrap();
        let intrinsic = 7.5 * 10_000_000.0 * (0.07 - 0.04); // ≈ 2.25M
                                                            // Some extrinsic value remains — price ≥ intrinsic.
        assert!(r.price >= intrinsic - 1.0);
    }

    #[test]
    fn deep_otm_payer_approaches_zero() {
        // Forward 1% vs strike 8% → deep OTM.
        let r = price(
            0.01,
            0.08,
            0.25,
            0.1,
            7.5,
            10_000_000.0,
            SwaptionKind::Payer,
        )
        .unwrap();
        assert!(r.price < 1_000.0);
    }

    #[test]
    fn higher_vol_inflates_atm_swaption() {
        let r_low = price(
            0.04,
            0.04,
            0.05,
            5.0,
            7.5,
            10_000_000.0,
            SwaptionKind::Payer,
        )
        .unwrap();
        let r_high = price(
            0.04,
            0.04,
            0.50,
            5.0,
            7.5,
            10_000_000.0,
            SwaptionKind::Payer,
        )
        .unwrap();
        assert!(r_high.price > r_low.price);
    }

    #[test]
    fn longer_expiry_inflates_atm_swaption() {
        let r_short = price(
            0.04,
            0.04,
            0.25,
            0.5,
            7.5,
            10_000_000.0,
            SwaptionKind::Payer,
        )
        .unwrap();
        let r_long = price(
            0.04,
            0.04,
            0.25,
            5.0,
            7.5,
            10_000_000.0,
            SwaptionKind::Payer,
        )
        .unwrap();
        assert!(r_long.price > r_short.price);
    }
}
