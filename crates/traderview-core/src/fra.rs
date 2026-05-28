//! Forward Rate Agreement (FRA) pricing.
//!
//! An FRA contracts a forward rate F over an upcoming period
//! [T_start, T_end]. Given the discount-factor curve P(0, T):
//!
//!   forward_rate = (P(0, T_start) / P(0, T_end) − 1) / (T_end − T_start)
//!
//! The PV of paying fixed rate K vs receiving floating over a unit
//! notional is:
//!
//!   PV_per_unit = (forward_rate − K) · accrual · P(0, T_end)
//!
//! where accrual = T_end − T_start (year-fraction). Positive PV =
//! receiver-FRA buyer wins (floating > fixed).
//!
//! Pure compute. Standard money-market discounting.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FraReport {
    pub forward_rate: f64,
    pub pv_per_unit_notional: f64,
    pub accrual_period: f64,
    pub discount_factor_end: f64,
}

#[allow(clippy::too_many_arguments)]
pub fn analyze(
    discount_start: f64,
    discount_end: f64,
    t_start: f64,
    t_end: f64,
    contract_rate: f64,
    notional: f64,
) -> Option<FraReport> {
    if !discount_start.is_finite() || discount_start <= 0.0 || discount_start > 1.0 + 1e-9
        || !discount_end.is_finite() || discount_end <= 0.0 || discount_end > 1.0 + 1e-9
        || !t_start.is_finite() || t_start < 0.0
        || !t_end.is_finite() || t_end <= t_start
        || !contract_rate.is_finite()
        || !notional.is_finite() || notional == 0.0
    {
        return None;
    }
    let accrual = t_end - t_start;
    let forward_rate = (discount_start / discount_end - 1.0) / accrual;
    if !forward_rate.is_finite() { return None; }
    let pv_per_unit = (forward_rate - contract_rate) * accrual * discount_end;
    Some(FraReport {
        forward_rate,
        pv_per_unit_notional: pv_per_unit * notional,
        accrual_period: accrual,
        discount_factor_end: discount_end,
    })
}

/// Compute the par FRA rate (the rate that makes PV = 0). For a single-
/// period FRA this equals the forward rate.
pub fn par_rate(discount_start: f64, discount_end: f64, t_start: f64, t_end: f64) -> Option<f64> {
    let r = analyze(discount_start, discount_end, t_start, t_end, 0.0, 1.0)?;
    Some(r.forward_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        // Non-positive discounts.
        assert!(analyze(0.0, 0.95, 1.0, 2.0, 0.05, 1.0).is_none());
        assert!(analyze(0.97, 0.0, 1.0, 2.0, 0.05, 1.0).is_none());
        // Discount > 1 (impossible under positive rates).
        assert!(analyze(1.5, 0.95, 1.0, 2.0, 0.05, 1.0).is_none());
        // End <= start.
        assert!(analyze(0.97, 0.95, 2.0, 1.0, 0.05, 1.0).is_none());
        assert!(analyze(0.97, 0.95, 1.0, 1.0, 0.05, 1.0).is_none());
        // NaN.
        assert!(analyze(f64::NAN, 0.95, 1.0, 2.0, 0.05, 1.0).is_none());
        // Zero notional.
        assert!(analyze(0.97, 0.95, 1.0, 2.0, 0.05, 0.0).is_none());
    }

    #[test]
    fn par_rate_makes_pv_zero() {
        let par = par_rate(0.97, 0.92, 1.0, 2.0).unwrap();
        let r = analyze(0.97, 0.92, 1.0, 2.0, par, 1_000_000.0).unwrap();
        assert!(r.pv_per_unit_notional.abs() < 1e-6);
    }

    #[test]
    fn forward_rate_consistent_with_discount_factors() {
        // P(0, 1) = 0.97, P(0, 2) = 0.92, accrual = 1
        // F = (0.97 / 0.92 − 1) / 1 ≈ 0.05435
        let r = analyze(0.97, 0.92, 1.0, 2.0, 0.0, 1.0).unwrap();
        let expected = (0.97 / 0.92 - 1.0) / 1.0;
        assert!((r.forward_rate - expected).abs() < 1e-12);
    }

    #[test]
    fn positive_pv_when_floating_exceeds_fixed() {
        // Forward = ~5.4%, contract at 4% → positive PV for receiver of floating.
        let r = analyze(0.97, 0.92, 1.0, 2.0, 0.04, 1_000_000.0).unwrap();
        assert!(r.pv_per_unit_notional > 0.0);
    }

    #[test]
    fn negative_pv_when_fixed_exceeds_floating() {
        let r = analyze(0.97, 0.92, 1.0, 2.0, 0.08, 1_000_000.0).unwrap();
        assert!(r.pv_per_unit_notional < 0.0);
    }

    #[test]
    fn longer_accrual_inflates_pv_magnitude() {
        let r_short = analyze(0.97, 0.95, 1.0, 1.25, 0.04, 1_000_000.0).unwrap();
        let r_long  = analyze(0.97, 0.92, 1.0, 2.00, 0.04, 1_000_000.0).unwrap();
        assert!(r_long.pv_per_unit_notional.abs() > r_short.pv_per_unit_notional.abs());
    }

    #[test]
    fn negative_notional_inverts_pv_sign() {
        let r_pos = analyze(0.97, 0.92, 1.0, 2.0, 0.04, 1_000_000.0).unwrap();
        let r_neg = analyze(0.97, 0.92, 1.0, 2.0, 0.04, -1_000_000.0).unwrap();
        assert!((r_pos.pv_per_unit_notional + r_neg.pv_per_unit_notional).abs() < 1e-9);
    }
}
