//! Volatility Swap — fair-strike formula with convexity adjustment.
//!
//! A volatility swap pays:
//!
//!   payoff = N · (realized_vol − K_vol)
//!
//! where realized_vol = √(realized_variance / T) annualized.
//!
//! Unlike a variance swap (linear in variance), a vol swap is linear
//! in volatility, so its fair strike is NOT simply √(K_var). The
//! standard approximation (Demeterfi-Derman-Kamal-Zou 1999):
//!
//!   K_vol ≈ √K_var · (1 − (1/8)·(vol_of_vol·T)²)
//!
//! For zero vol-of-vol the strike collapses to √K_var. The convexity
//! adjustment accounts for the fact that vol is a concave function of
//! variance (E[√X] ≤ √E`[X]` by Jensen).
//!
//! Pure compute. Caller supplies the fair variance-swap strike (from
//! `variance_swap_strike`) and a vol-of-vol estimate. Companion to
//! `variance_swap`, `variance_swap_strike`, `cliquet_option`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VolatilitySwapReport {
    pub fair_vol_strike: f64,
    pub variance_strike_used: f64,
    pub naive_vol_strike: f64,
    pub convexity_adjustment_bps: f64,
}

pub fn fair_strike(
    variance_strike: f64,
    vol_of_vol_annualized: f64,
    time_to_expiry_years: f64,
) -> Option<VolatilitySwapReport> {
    if !variance_strike.is_finite() || variance_strike < 0.0
        || !vol_of_vol_annualized.is_finite() || vol_of_vol_annualized < 0.0
        || !time_to_expiry_years.is_finite() || time_to_expiry_years <= 0.0 {
        return None;
    }
    let naive_vol = variance_strike.sqrt();
    // Demeterfi-Derman-Kamal-Zou first-order convexity adjustment:
    //   K_vol ≈ √K_var · (1 − ⅛·(ξ·T)²)
    // where ξ is annualized vol-of-vol.
    let xi_t = vol_of_vol_annualized * time_to_expiry_years;
    let adjustment = 1.0 - 0.125 * xi_t * xi_t;
    let fair = naive_vol * adjustment;
    let adj_bps = (fair - naive_vol) * 10_000.0;
    Some(VolatilitySwapReport {
        fair_vol_strike: fair.max(0.0),
        variance_strike_used: variance_strike,
        naive_vol_strike: naive_vol,
        convexity_adjustment_bps: adj_bps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(fair_strike(-0.01, 0.5, 0.25).is_none());
        assert!(fair_strike(0.04, -0.1, 0.25).is_none());
        assert!(fair_strike(0.04, 0.5, 0.0).is_none());
        assert!(fair_strike(f64::NAN, 0.5, 0.25).is_none());
    }

    #[test]
    fn zero_vol_of_vol_yields_naive_strike() {
        let r = fair_strike(0.04, 0.0, 0.25).unwrap();
        assert!((r.fair_vol_strike - r.naive_vol_strike).abs() < 1e-12);
        assert!(r.convexity_adjustment_bps.abs() < 1e-9);
    }

    #[test]
    fn nonzero_vol_of_vol_reduces_strike() {
        let r = fair_strike(0.04, 0.5, 0.25).unwrap();
        // Strike below naive √0.04 = 0.20.
        assert!(r.fair_vol_strike < r.naive_vol_strike);
        assert!(r.convexity_adjustment_bps < 0.0);
    }

    #[test]
    fn higher_vol_of_vol_widens_adjustment() {
        let low = fair_strike(0.04, 0.3, 0.5).unwrap();
        let high = fair_strike(0.04, 0.8, 0.5).unwrap();
        assert!(high.fair_vol_strike < low.fair_vol_strike);
    }

    #[test]
    fn longer_horizon_widens_adjustment() {
        let short_t = fair_strike(0.04, 0.5, 0.1).unwrap();
        let long_t = fair_strike(0.04, 0.5, 1.0).unwrap();
        assert!(long_t.fair_vol_strike < short_t.fair_vol_strike);
    }

    #[test]
    fn fair_strike_non_negative() {
        // Extreme vol-of-vol could overflow adjustment negative; floored at 0.
        let r = fair_strike(0.04, 2.0, 5.0).unwrap();
        assert!(r.fair_vol_strike >= 0.0);
    }

    #[test]
    fn fields_passed_through() {
        let r = fair_strike(0.0625, 0.4, 0.25).unwrap();
        assert!((r.variance_strike_used - 0.0625).abs() < 1e-12);
        assert!((r.naive_vol_strike - 0.25).abs() < 1e-12);
    }
}
