//! SABR (Stochastic Alpha-Beta-Rho) implied-vol — Hagan, Kumar,
//! Lesniewski, Woodward (2002) asymptotic expansion.
//!
//!   dF_t = α · F_t^β · dW_t
//!   dα_t = ν · α_t · dZ_t
//!   d⟨W, Z⟩ = ρ · dt
//!
//! Hagan's lognormal-vol approximation σ_BLN(K, F, T) is the standard
//! market-fitting tool: given forward F, strike K, expiry T, and
//! (α, β, ρ, ν), it returns the Black-implied vol to plug into Black-76.
//!
//! Pure compute. Returns NaN for the ATM-limit edge case (special-cased
//! to the closed-form ATM expansion).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SabrParams {
    pub alpha: f64,    // current vol level
    pub beta: f64,     // skew exponent (0=normal, 1=lognormal)
    pub rho: f64,      // correlation
    pub nu: f64,       // vol-of-vol
}

pub fn implied_lognormal_vol(
    forward: f64, strike: f64, time_to_expiry: f64, params: &SabrParams,
) -> Option<f64> {
    if !forward.is_finite() || forward <= 0.0
        || !strike.is_finite() || strike <= 0.0
        || !time_to_expiry.is_finite() || time_to_expiry <= 0.0
        || !params.alpha.is_finite() || params.alpha <= 0.0
        || !params.beta.is_finite() || !(0.0..=1.0).contains(&params.beta)
        || !params.rho.is_finite() || !(-1.0..=1.0).contains(&params.rho)
        || !params.nu.is_finite() || params.nu < 0.0
    {
        return None;
    }
    let alpha = params.alpha;
    let beta = params.beta;
    let rho = params.rho;
    let nu = params.nu;
    let one_minus_beta = 1.0 - beta;
    let fk_avg = (forward * strike).powf(one_minus_beta / 2.0);
    let log_fk = (forward / strike).ln();
    // Hagan term 1: prefactor.
    let denominator = fk_avg * (
        1.0
        + (one_minus_beta * log_fk).powi(2) / 24.0
        + (one_minus_beta * log_fk).powi(4) / 1920.0
    );
    if denominator <= 0.0 || !denominator.is_finite() { return None; }
    let prefactor = alpha / denominator;
    // Hagan term 2: skew correction in z.
    let atm_threshold = 1e-9;
    let z_correction = if log_fk.abs() < atm_threshold {
        // ATM limit: z/x(z) → 1.
        1.0
    } else {
        let z = (nu / alpha) * fk_avg * log_fk;
        let x = ((1.0 - 2.0 * rho * z + z * z).sqrt() + z - rho) / (1.0 - rho);
        if x <= 0.0 || !x.is_finite() { return None; }
        z / x.ln()
    };
    // Hagan term 3: time-decay correction.
    let time_correction = 1.0 + (
        (one_minus_beta * alpha).powi(2) / (24.0 * fk_avg.powi(2))
        + rho * beta * nu * alpha / (4.0 * fk_avg)
        + (2.0 - 3.0 * rho * rho) * nu * nu / 24.0
    ) * time_to_expiry;
    let vol = prefactor * z_correction * time_correction;
    if !vol.is_finite() || vol <= 0.0 { return None; }
    Some(vol)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(alpha: f64, beta: f64, rho: f64, nu: f64) -> SabrParams {
        SabrParams { alpha, beta, rho, nu }
    }

    #[test]
    fn invalid_inputs_return_none() {
        let ok = p(0.20, 0.5, -0.3, 0.40);
        for bad in [0.0, -1.0, f64::NAN] {
            assert!(implied_lognormal_vol(bad, 100.0, 1.0, &ok).is_none());
            assert!(implied_lognormal_vol(100.0, bad, 1.0, &ok).is_none());
            assert!(implied_lognormal_vol(100.0, 100.0, bad, &ok).is_none());
        }
        assert!(implied_lognormal_vol(100.0, 100.0, 1.0, &p(0.0, 0.5, 0.0, 0.5)).is_none());
        assert!(implied_lognormal_vol(100.0, 100.0, 1.0, &p(0.2, -0.1, 0.0, 0.5)).is_none());
        assert!(implied_lognormal_vol(100.0, 100.0, 1.0, &p(0.2, 1.5, 0.0, 0.5)).is_none());
        assert!(implied_lognormal_vol(100.0, 100.0, 1.0, &p(0.2, 0.5, -1.5, 0.5)).is_none());
        assert!(implied_lognormal_vol(100.0, 100.0, 1.0, &p(0.2, 0.5, 1.5, 0.5)).is_none());
        assert!(implied_lognormal_vol(100.0, 100.0, 1.0, &p(0.2, 0.5, 0.0, -0.5)).is_none());
    }

    #[test]
    fn atm_lognormal_case_recovers_alpha() {
        // β = 1, ν = 0 → SABR reduces to lognormal: σ_BLN = α.
        let v = implied_lognormal_vol(100.0, 100.0, 1.0, &p(0.20, 1.0, 0.0, 0.0)).unwrap();
        assert!((v - 0.20).abs() < 1e-6);
    }

    #[test]
    fn atm_normal_beta_zero_yields_alpha_over_f() {
        // β = 0, ν = 0 → σ_BLN ≈ α / F  (the leading-order ATM normal
        // → lognormal conversion). Tolerance generous for asymptotics.
        let v = implied_lognormal_vol(100.0, 100.0, 0.5, &p(20.0, 0.0, 0.0, 0.0)).unwrap();
        let expected = 20.0 / 100.0;
        assert!((v - expected).abs() / expected < 0.05);
    }

    #[test]
    fn smile_curvature_increases_with_vol_of_vol() {
        // Higher ν inflates the wings of the smile (relative to ATM).
        let f = 100.0; let t = 1.0;
        let low_nu = p(0.20, 0.5, -0.3, 0.10);
        let high_nu = p(0.20, 0.5, -0.3, 0.80);
        let atm_low = implied_lognormal_vol(f, f, t, &low_nu).unwrap();
        let atm_high = implied_lognormal_vol(f, f, t, &high_nu).unwrap();
        let otm_low = implied_lognormal_vol(f, 130.0, t, &low_nu).unwrap();
        let otm_high = implied_lognormal_vol(f, 130.0, t, &high_nu).unwrap();
        // Smile width = |OTM − ATM| should grow with ν.
        let width_low = (otm_low - atm_low).abs();
        let width_high = (otm_high - atm_high).abs();
        assert!(width_high > width_low,
            "high-ν smile width ({width_high}) should exceed low-ν ({width_low})");
    }

    #[test]
    fn negative_rho_tilts_smile_left() {
        // ρ < 0 → put skew (left wing higher vol than right).
        let f = 100.0; let t = 1.0;
        let pp = p(0.20, 0.5, -0.5, 0.50);
        let v_lo_strike = implied_lognormal_vol(f, 80.0, t, &pp).unwrap();
        let v_hi_strike = implied_lognormal_vol(f, 120.0, t, &pp).unwrap();
        assert!(v_lo_strike > v_hi_strike, "negative ρ should produce put-skew");
    }

    #[test]
    fn positive_rho_tilts_smile_right() {
        let f = 100.0; let t = 1.0;
        let pp = p(0.20, 0.5, 0.5, 0.50);
        let v_lo_strike = implied_lognormal_vol(f, 80.0, t, &pp).unwrap();
        let v_hi_strike = implied_lognormal_vol(f, 120.0, t, &pp).unwrap();
        assert!(v_hi_strike > v_lo_strike, "positive ρ should produce call-skew");
    }

    #[test]
    fn vol_strictly_positive_across_wide_strikes() {
        let pp = p(0.25, 0.6, -0.3, 0.40);
        for k in [50.0, 70.0, 90.0, 100.0, 110.0, 130.0, 150.0] {
            let v = implied_lognormal_vol(100.0, k, 1.0, &pp);
            assert!(v.is_some_and(|v| v > 0.0), "vol should be > 0 at K={k}");
        }
    }
}
