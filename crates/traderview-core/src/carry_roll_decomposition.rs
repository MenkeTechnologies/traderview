//! Fixed-Income Carry + Roll-Down Return Decomposition.
//!
//! For a bond held over horizon Δt under a static yield curve:
//!
//!   Carry     = coupon yield earned over Δt
//!             = (coupon_rate / freq) · Δt_in_years   (approx for short Δt)
//!
//!   Roll-Down = capital appreciation as the bond rolls down a
//!               (typically positively-sloped) curve toward shorter
//!               maturities with lower yields:
//!             ≈ −Modified_Duration · (yield_at_T_minus_Δt − yield_at_T)
//!
//!   Total carry-and-roll = Carry + Roll-Down
//!
//! Annualized form: divide by Δt.
//!
//! Used by relative-value desks to identify forward-implied "free
//! returns" assuming the yield curve doesn't move (forward unbiased).
//!
//! Pure compute. Companion to `nelson_siegel`, `macaulay_duration`,
//! `key_rate_duration`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CarryRollReport {
    pub carry: f64,
    pub roll_down: f64,
    pub total_return: f64,
    pub annualized_total_return: f64,
    pub modified_duration: f64,
    pub yield_now: f64,
    pub yield_at_horizon_maturity: f64,
}

pub fn compute(
    coupon_rate_annual: f64,
    modified_duration_years: f64,
    yield_now_at_maturity_t: f64,
    yield_at_shorter_maturity_t_minus_horizon: f64,
    horizon_years: f64,
) -> Option<CarryRollReport> {
    if !coupon_rate_annual.is_finite()
        || !modified_duration_years.is_finite()
        || !yield_now_at_maturity_t.is_finite()
        || !yield_at_shorter_maturity_t_minus_horizon.is_finite()
        || !horizon_years.is_finite()
        || horizon_years <= 0.0
        || modified_duration_years < 0.0
    {
        return None;
    }
    let carry = coupon_rate_annual * horizon_years;
    // Yield curve change as the bond "rolls" to shorter maturity.
    let dy = yield_at_shorter_maturity_t_minus_horizon - yield_now_at_maturity_t;
    let roll_down = -modified_duration_years * dy;
    let total = carry + roll_down;
    let annualized = total / horizon_years;
    Some(CarryRollReport {
        carry,
        roll_down,
        total_return: total,
        annualized_total_return: annualized,
        modified_duration: modified_duration_years,
        yield_now: yield_now_at_maturity_t,
        yield_at_horizon_maturity: yield_at_shorter_maturity_t_minus_horizon,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nan_or_invalid_returns_none() {
        assert!(compute(f64::NAN, 5.0, 0.04, 0.038, 1.0).is_none());
        assert!(compute(0.05, 5.0, 0.04, 0.038, 0.0).is_none());
        assert!(compute(0.05, -1.0, 0.04, 0.038, 1.0).is_none());
    }

    #[test]
    fn flat_curve_yields_zero_roll_down() {
        // Same yield at maturity now and at shorter maturity → roll = 0.
        let r = compute(0.05, 5.0, 0.04, 0.04, 1.0).unwrap();
        assert!(r.roll_down.abs() < 1e-12);
        // Carry = coupon · horizon.
        assert!((r.carry - 0.05).abs() < 1e-12);
        assert!((r.total_return - 0.05).abs() < 1e-12);
    }

    #[test]
    fn positively_sloped_curve_yields_positive_roll_down() {
        // 10y yield = 4%, 9y yield = 3.8% → bond rolls down 20bps over 1y.
        // Roll-down = -duration · (3.8% - 4.0%) = -5 · -0.002 = +0.01.
        let r = compute(0.05, 5.0, 0.04, 0.038, 1.0).unwrap();
        assert!(r.roll_down > 0.0);
        assert!((r.roll_down - 0.01).abs() < 1e-12);
    }

    #[test]
    fn inverted_curve_yields_negative_roll_down() {
        // 10y = 4%, 9y = 4.2% → bond rolls up to higher yield.
        let r = compute(0.05, 5.0, 0.04, 0.042, 1.0).unwrap();
        assert!(r.roll_down < 0.0);
    }

    #[test]
    fn carry_scales_with_horizon() {
        let r6m = compute(0.05, 5.0, 0.04, 0.04, 0.5).unwrap();
        let r1y = compute(0.05, 5.0, 0.04, 0.04, 1.0).unwrap();
        assert!((r1y.carry - 2.0 * r6m.carry).abs() < 1e-12);
    }

    #[test]
    fn annualized_total_return_correctly_computed() {
        let r = compute(0.05, 5.0, 0.04, 0.038, 0.5).unwrap();
        // Annualized = total / 0.5.
        assert!((r.annualized_total_return - 2.0 * r.total_return).abs() < 1e-12);
    }

    #[test]
    fn higher_duration_amplifies_roll_down() {
        let short = compute(0.05, 2.0, 0.04, 0.038, 1.0).unwrap();
        let long = compute(0.05, 10.0, 0.04, 0.038, 1.0).unwrap();
        assert!(long.roll_down.abs() > short.roll_down.abs());
    }
}
