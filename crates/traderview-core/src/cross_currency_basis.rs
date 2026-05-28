//! Cross-Currency Basis — implied basis spread from FX forward parity.
//!
//! Covered Interest-Rate Parity (CIP) says:
//!
//!   F / S = exp((r_d − r_f) · T)
//!
//! When CIP holds, the implied basis is zero. In practice (post-2008
//! USD funding stress, year-end balance-sheet effects, regulatory
//! distortions) the basis is non-zero — typically negative outside USD,
//! meaning non-USD holders pay extra to fund USD via FX swaps.
//!
//! Solving for the basis on the foreign leg (i.e. the residual to add
//! to r_f so CIP holds):
//!
//!   basis = (1/T) · ln(F / S) − (r_d − r_f)
//!
//! Convention: basis > 0 means foreign currency funding is cheaper
//! than CIP-implied; basis < 0 means foreign currency funding
//! premium (the typical post-2008 EUR/JPY/CHF cross-currency basis).
//!
//! All rates continuously compounded; T in years.
//!
//! Pure compute. Companion to `futures_roll` (basis-roll), `fra` (rate
//! forwards), `yield_curve_bootstrap`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BasisReport {
    pub implied_basis: f64,
    pub implied_forward_no_basis: f64,
    pub log_forward_premium: f64,
}

pub fn compute(
    spot: f64,
    forward: f64,
    domestic_rate: f64,
    foreign_rate: f64,
    time_years: f64,
) -> Option<BasisReport> {
    if !spot.is_finite() || spot <= 0.0
        || !forward.is_finite() || forward <= 0.0
        || !domestic_rate.is_finite() || !foreign_rate.is_finite()
        || !time_years.is_finite() || time_years <= 0.0 {
        return None;
    }
    let log_premium = (forward / spot).ln();
    let basis = log_premium / time_years - (domestic_rate - foreign_rate);
    let implied_fwd = spot * ((domestic_rate - foreign_rate) * time_years).exp();
    Some(BasisReport {
        implied_basis: basis,
        implied_forward_no_basis: implied_fwd,
        log_forward_premium: log_premium,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(0.0, 1.0, 0.05, 0.02, 1.0).is_none());
        assert!(compute(1.0, 0.0, 0.05, 0.02, 1.0).is_none());
        assert!(compute(1.0, 1.0, f64::NAN, 0.02, 1.0).is_none());
        assert!(compute(1.0, 1.0, 0.05, 0.02, 0.0).is_none());
        assert!(compute(1.0, 1.0, 0.05, 0.02, -1.0).is_none());
    }

    #[test]
    fn cip_consistent_inputs_yield_zero_basis() {
        // F = S · exp((rd - rf) · T) → basis = 0.
        let spot = 1.10_f64;
        let rd = 0.05_f64;
        let rf = 0.02_f64;
        let t = 0.25_f64;
        let forward = spot * ((rd - rf) * t).exp();
        let r = compute(spot, forward, rd, rf, t).unwrap();
        assert!(r.implied_basis.abs() < 1e-12,
            "CIP-consistent → basis 0, got {}", r.implied_basis);
    }

    #[test]
    fn forward_premium_above_cip_yields_positive_basis() {
        // Pretend the market trades F above the CIP-implied forward.
        // Then foreign funding looks "cheap" → positive basis.
        let spot = 1.10_f64;
        let rd = 0.05_f64;
        let rf = 0.02_f64;
        let t = 0.25_f64;
        let cip_fwd = spot * ((rd - rf) * t).exp();
        let r = compute(spot, cip_fwd * 1.01, rd, rf, t).unwrap();
        assert!(r.implied_basis > 0.0, "got {}", r.implied_basis);
    }

    #[test]
    fn forward_discount_below_cip_yields_negative_basis() {
        // Classic post-2008 EUR-USD pattern: forward trades below CIP-implied.
        let spot = 1.10_f64;
        let rd = 0.05_f64;
        let rf = 0.02_f64;
        let t = 0.25_f64;
        let cip_fwd = spot * ((rd - rf) * t).exp();
        let r = compute(spot, cip_fwd * 0.99, rd, rf, t).unwrap();
        assert!(r.implied_basis < 0.0, "got {}", r.implied_basis);
    }

    #[test]
    fn implied_forward_consistent_with_cip() {
        let spot = 1.10_f64;
        let rd = 0.05_f64;
        let rf = 0.02_f64;
        let t = 0.25_f64;
        let cip_fwd = spot * ((rd - rf) * t).exp();
        let r = compute(spot, cip_fwd, rd, rf, t).unwrap();
        assert!((r.implied_forward_no_basis - cip_fwd).abs() < 1e-12);
    }

    #[test]
    fn log_premium_zero_when_forward_equals_spot() {
        let r = compute(1.0, 1.0, 0.02, 0.02, 0.5).unwrap();
        assert!(r.log_forward_premium.abs() < 1e-12);
        // basis = -rd + rf = 0 when rates equal.
        assert!(r.implied_basis.abs() < 1e-12);
    }

    #[test]
    fn longer_tenor_amortizes_basis_linearly() {
        // Same forward dislocation → basis scales as 1/T (log_premium constant).
        let spot = 1.10_f64;
        let rd = 0.05_f64;
        let rf = 0.02_f64;
        // Construct a fixed log-premium gap of 0.005.
        let fwd = spot * (0.005_f64).exp();
        let r_short = compute(spot, fwd, rd, rf, 0.25).unwrap();
        let r_long = compute(spot, fwd, rd, rf, 1.0).unwrap();
        // The 1Y basis should be 1/4 of the 3M basis (in magnitude difference).
        let ratio = (r_short.implied_basis - (rf - rd))
            / (r_long.implied_basis - (rf - rd));
        assert!((ratio - 4.0).abs() < 1e-6,
            "ratio of 3M:1Y basis (above CIP) should be 4:1, got {ratio}");
    }
}
