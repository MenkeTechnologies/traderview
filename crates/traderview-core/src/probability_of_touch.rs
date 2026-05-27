//! Probability-of-Touch for option strikes.
//:
//! For a Black-Scholes-modeled underlying, the probability that price
//! touches a barrier before expiration is approximately TWICE the
//! probability of expiring beyond it (for OTM levels), per the
//! reflection-principle approximation.
//!
//! Specifically:
//!   P(touch_K_at_or_before_T) ≈ 2 × P(S_T ≥ K)    for K > S_0 (above)
//!   P(touch_K_at_or_before_T) ≈ 2 × P(S_T ≤ K)    for K < S_0 (below)
//!
//! Used to size credit spreads — if the short strike has a 30%
//! probability of touch (vs 15% of expiring beyond), that's the realistic
//! likelihood the position needs management.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PotReport {
    pub spot: f64,
    pub strike: f64,
    pub p_expire_beyond: f64,
    /// Capped at 1.0 since 2× a tail prob can exceed 1.
    pub p_touch: f64,
    pub d2: f64,
}

pub fn compute(
    spot: f64,
    strike: f64,
    sigma: f64,
    days_to_expiry: f64,
    r: f64,
    q: f64,
) -> PotReport {
    if spot <= 0.0 || strike <= 0.0 || days_to_expiry <= 0.0 || sigma <= 0.0 {
        return PotReport {
            spot,
            strike,
            ..Default::default()
        };
    }
    let t = days_to_expiry / 365.0;
    let denom = sigma * t.sqrt();
    let d2 = ((spot / strike).ln() + (r - q - 0.5 * sigma * sigma) * t) / denom;
    let above = strike > spot;
    let p_expire = if above {
        // P(S_T ≥ K) = N(d2).
        norm_cdf(d2)
    } else {
        // P(S_T ≤ K) = N(-d2).
        norm_cdf(-d2)
    };
    let p_touch = (2.0 * p_expire).min(1.0);
    PotReport {
        spot,
        strike,
        p_expire_beyond: p_expire,
        p_touch,
        d2,
    }
}

/// Standard normal CDF via Abramowitz & Stegun approximation 7.1.26.
fn norm_cdf(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let xa = x.abs() / std::f64::consts::SQRT_2;
    // erf approximation
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;
    let t = 1.0 / (1.0 + p * xa);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-xa * xa).exp();
    0.5 * (1.0 + sign * y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn degenerate_inputs_return_default() {
        let r = compute(0.0, 100.0, 0.30, 30.0, 0.05, 0.0);
        assert_eq!(r.p_touch, 0.0);
    }

    #[test]
    fn atm_strike_high_touch_probability() {
        // Spot 100, strike 100 (ATM), 30d, 30% IV. d2 close to 0 → N(0)=0.5 → POT ≈ 1.0 (capped).
        let r = compute(100.0, 100.0, 0.30, 30.0, 0.05, 0.0);
        assert!(r.p_touch >= 0.9);
    }

    #[test]
    fn far_otm_strike_low_touch_probability() {
        // Strike 130, spot 100, 30d, 30% IV → very unlikely to touch.
        let r = compute(100.0, 130.0, 0.30, 30.0, 0.05, 0.0);
        assert!(r.p_touch < 0.1);
    }

    #[test]
    fn p_touch_approximately_twice_p_expire() {
        // For an OTM strike, p_touch ≈ 2 × p_expire (unless capped).
        let r = compute(100.0, 110.0, 0.30, 30.0, 0.05, 0.0);
        if r.p_touch < 1.0 {
            assert!((r.p_touch - 2.0 * r.p_expire_beyond).abs() < 1e-9);
        }
    }

    #[test]
    fn p_touch_capped_at_one_when_2x_p_expire_exceeds_one() {
        // ATM, long expiry → 2 × ~0.5 = 1.0 cap.
        let r = compute(100.0, 100.0, 0.50, 365.0, 0.05, 0.0);
        assert!(r.p_touch <= 1.0);
    }

    #[test]
    fn touch_probability_higher_for_higher_iv() {
        let low_iv = compute(100.0, 110.0, 0.10, 30.0, 0.05, 0.0);
        let high_iv = compute(100.0, 110.0, 0.50, 30.0, 0.05, 0.0);
        assert!(high_iv.p_touch > low_iv.p_touch);
    }

    #[test]
    fn touch_probability_higher_for_longer_expiry() {
        let short = compute(100.0, 110.0, 0.30, 7.0, 0.05, 0.0);
        let long = compute(100.0, 110.0, 0.30, 60.0, 0.05, 0.0);
        assert!(long.p_touch > short.p_touch);
    }

    #[test]
    fn put_side_strike_below_spot_uses_negative_tail() {
        // Strike 90, spot 100 → ITM-on-the-other-side calc.
        let r = compute(100.0, 90.0, 0.30, 30.0, 0.05, 0.0);
        assert!(r.p_touch > 0.0 && r.p_touch <= 1.0);
    }

    #[test]
    fn norm_cdf_symmetric_around_zero() {
        let zero = norm_cdf(0.0);
        assert!((zero - 0.5).abs() < 1e-3);
        let plus = norm_cdf(1.0);
        let minus = norm_cdf(-1.0);
        assert!((plus + minus - 1.0).abs() < 1e-3);
    }

    #[test]
    fn norm_cdf_tails() {
        assert!(norm_cdf(-5.0) < 0.001);
        assert!(norm_cdf(5.0) > 0.999);
    }
}
