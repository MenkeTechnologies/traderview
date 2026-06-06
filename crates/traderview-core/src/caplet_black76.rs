//! Caplet / floorlet pricing under Black-76.
//!
//! A caplet is a European call on a forward rate F_t:
//!   payoff_T = max(F − K, 0) · accrual · notional, paid at T_end
//!
//! Floorlet = European put on the forward rate.
//!
//! Black-76 formula (treats the forward rate F as if it were a futures
//! price, with the convexity-of-bond-vs-rate already folded into the
//! lognormal-rate assumption):
//!
//!   d1 = [ln(F/K) + 0.5·σ²·T_expiry] / (σ·√T_expiry)
//!   d2 = d1 − σ·√T_expiry
//!   caplet  = accrual · notional · e^{−r·T_end} · [F·N(d1) − K·N(d2)]
//!   floorlet = accrual · notional · e^{−r·T_end} · [K·N(−d2) − F·N(−d1)]
//!
//! where T_expiry is the cap-reset date (typically T_start) and T_end
//! is the payment date.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionKind {
    Caplet,
    Floorlet,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CapletReport {
    pub price: f64,
    pub d1: f64,
    pub d2: f64,
}

#[allow(clippy::too_many_arguments)]
pub fn price(
    forward_rate: f64,
    strike_rate: f64,
    sigma: f64,
    t_expiry: f64,
    t_end: f64,
    discount_factor_t_end: f64,
    accrual: f64,
    notional: f64,
    kind: OptionKind,
) -> Option<CapletReport> {
    if !forward_rate.is_finite()
        || forward_rate <= 0.0
        || !strike_rate.is_finite()
        || strike_rate <= 0.0
        || !sigma.is_finite()
        || sigma <= 0.0
        || !t_expiry.is_finite()
        || t_expiry <= 0.0
        || !t_end.is_finite()
        || t_end < t_expiry
        || !discount_factor_t_end.is_finite()
        || discount_factor_t_end <= 0.0
        || discount_factor_t_end > 1.0 + 1e-9
        || !accrual.is_finite()
        || accrual <= 0.0
        || !notional.is_finite()
        || notional == 0.0
    {
        return None;
    }
    let sqrt_t = t_expiry.sqrt();
    let d1 =
        ((forward_rate / strike_rate).ln() + 0.5 * sigma * sigma * t_expiry) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;
    let nd1 = norm_cdf(d1);
    let nd2 = norm_cdf(d2);
    let scale = accrual * notional * discount_factor_t_end;
    let p = match kind {
        OptionKind::Caplet => scale * (forward_rate * nd1 - strike_rate * nd2),
        OptionKind::Floorlet => scale * (strike_rate * (1.0 - nd2) - forward_rate * (1.0 - nd1)),
    };
    if !p.is_finite() {
        return None;
    }
    Some(CapletReport {
        price: p.max(0.0),
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

    fn args() -> (f64, f64, f64, f64, f64, f64, f64, f64) {
        // 6-month caplet expiring in 1y, on a 1M notional, F=5%, K=5%, σ=20%.
        let forward = 0.05;
        let strike = 0.05;
        let sigma = 0.20;
        let t_expiry = 1.0;
        let t_end = 1.5;
        let p_end = (-0.05_f64 * 1.5).exp(); // simple flat-rate discount
        let accrual = 0.5;
        let notional = 1_000_000.0;
        (
            forward, strike, sigma, t_expiry, t_end, p_end, accrual, notional,
        )
    }

    #[test]
    fn invalid_inputs_return_none() {
        for bad in [0.0, -1.0, f64::NAN] {
            let (f, k, s, te, tend, p, a, n) = args();
            assert!(price(bad, k, s, te, tend, p, a, n, OptionKind::Caplet).is_none());
            assert!(price(f, bad, s, te, tend, p, a, n, OptionKind::Caplet).is_none());
            assert!(price(f, k, bad, te, tend, p, a, n, OptionKind::Caplet).is_none());
            assert!(price(f, k, s, bad, tend, p, a, n, OptionKind::Caplet).is_none());
        }
    }

    #[test]
    fn atm_caplet_price_positive() {
        let (f, k, s, te, tend, p, a, n) = args();
        let r = price(f, k, s, te, tend, p, a, n, OptionKind::Caplet).unwrap();
        assert!(r.price > 0.0);
    }

    #[test]
    fn caplet_floorlet_parity_holds() {
        // c − p = accrual · notional · DF · (F − K)
        let (f, k, s, te, tend, p, a, n) = args();
        let cap = price(f, k, s, te, tend, p, a, n, OptionKind::Caplet).unwrap();
        let flr = price(f, k, s, te, tend, p, a, n, OptionKind::Floorlet).unwrap();
        let parity = a * n * p * (f - k);
        assert!((cap.price - flr.price - parity).abs() < 1e-6);
    }

    #[test]
    fn deep_itm_caplet_approaches_intrinsic() {
        // F = 8% vs K = 5% → intrinsic = 0.03 · 0.5 · 1M · DF ≈ 13.9k.
        let (_, k, s, te, tend, p, a, n) = args();
        let r = price(0.08, k, s, te, tend, p, a, n, OptionKind::Caplet).unwrap();
        let intrinsic = a * n * p * (0.08 - k);
        // For low vol and deep ITM, caplet ≈ intrinsic + small extrinsic.
        assert!(r.price >= intrinsic - 1e-6);
        assert!(r.price < intrinsic + 5_000.0);
    }

    #[test]
    fn higher_vol_inflates_atm_caplet() {
        let (f, k, _, te, tend, p, a, n) = args();
        let r_low = price(f, k, 0.05, te, tend, p, a, n, OptionKind::Caplet).unwrap();
        let r_high = price(f, k, 0.50, te, tend, p, a, n, OptionKind::Caplet).unwrap();
        assert!(r_high.price > r_low.price);
    }

    #[test]
    fn longer_expiry_inflates_atm_caplet() {
        let (f, k, s, _, _, _, a, n) = args();
        let r_short = price(
            f,
            k,
            s,
            0.5,
            1.0,
            (-0.05_f64 * 1.0).exp(),
            a,
            n,
            OptionKind::Caplet,
        )
        .unwrap();
        let r_long = price(
            f,
            k,
            s,
            2.0,
            2.5,
            (-0.05_f64 * 2.5).exp(),
            a,
            n,
            OptionKind::Caplet,
        )
        .unwrap();
        assert!(r_long.price > r_short.price);
    }

    #[test]
    fn deep_otm_caplet_approaches_zero() {
        let (_, k, s, te, tend, p, a, n) = args();
        let r = price(0.01, k, s, te, tend, p, a, n, OptionKind::Caplet).unwrap();
        // F = 1%, K = 5% — deep OTM. Caplet price tiny.
        assert!(r.price < 50.0);
    }
}
