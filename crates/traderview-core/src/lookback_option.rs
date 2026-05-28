//! Lookback option — Conze-Viswanathan (1991) closed form.
//!
//! Floating-strike lookback option:
//!   - **Call**: payoff = S_T − min(S_t)
//!     (right to buy at the lowest price observed)
//!   - **Put**:  payoff = max(S_t) − S_T
//!     (right to sell at the highest price observed)
//!
//! For continuous monitoring over [0, T] with Black-Scholes underlying:
//!
//!   b = r − q
//!   a1 = [ln(S/M) + (b + σ²/2)·T] / (σ·√T)
//!   a2 = a1 − σ·√T
//!   a3 = a1 − (2b/σ)·√T
//!
//! where M = current minimum (for call) or maximum (for put). If the
//! contract is newly issued, M = S₀.
//!
//! Floating-strike call:
//!   c = S·e^{−qT}·N(a1) − M·e^{−rT}·N(a2)
//!     + S·e^{−rT}·(σ²/(2b))·[(S/M)^{−2b/σ²}·N(−a3) − e^{bT}·N(−a1)]
//!
//! Floating-strike put:
//!   p = M·e^{−rT}·N(−a2) − S·e^{−qT}·N(−a1)
//!     + S·e^{−rT}·(σ²/(2b))·[−(S/M)^{−2b/σ²}·N(a3) + e^{bT}·N(a1)]
//!
//! Pure compute. The (2b/σ²) factor diverges when b → 0 (q == r); use
//! the limiting form in that case.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionKind { Call, Put }

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LookbackReport {
    pub price: f64,
    pub a1: f64,
    pub a2: f64,
}

pub fn price(
    spot: f64, observed_extreme: f64,
    time_to_expiry: f64,
    risk_free: f64, dividend_yield: f64,
    sigma: f64,
    kind: OptionKind,
) -> Option<LookbackReport> {
    if !spot.is_finite() || spot <= 0.0
        || !observed_extreme.is_finite() || observed_extreme <= 0.0
        || !time_to_expiry.is_finite() || time_to_expiry <= 0.0
        || !risk_free.is_finite() || !dividend_yield.is_finite()
        || !sigma.is_finite() || sigma <= 0.0
    {
        return None;
    }
    // Consistency: for a CALL, observed_extreme is min and must be ≤ S;
    // for a PUT, observed_extreme is max and must be ≥ S.
    match kind {
        OptionKind::Call if observed_extreme > spot => return None,
        OptionKind::Put  if observed_extreme < spot => return None,
        _ => {}
    }
    let s = spot;
    let m = observed_extreme;
    let t = time_to_expiry;
    let r = risk_free;
    let q = dividend_yield;
    let v = sigma;
    let b = r - q;
    let sqrt_t = t.sqrt();
    let v_sqrt_t = v * sqrt_t;
    let a1 = ((s / m).ln() + (b + 0.5 * v * v) * t) / v_sqrt_t;
    let a2 = a1 - v_sqrt_t;
    let two_b_over_v2 = if b.abs() < 1e-12 { 0.0 } else { 2.0 * b / (v * v) };
    let a3 = a1 - two_b_over_v2 * v_sqrt_t;
    let dq = (-q * t).exp();
    let dr = (-r * t).exp();
    // Special-case b = 0: the σ²/(2b) factor diverges, replaced by the
    // limiting form (l'Hôpital with respect to b).
    if b.abs() < 1e-12 {
        let price = match kind {
            OptionKind::Call => {
                let term = s * dr * v_sqrt_t * (norm_pdf(a1) + a1 * norm_cdf(a1));
                s * dr * norm_cdf(a1) - m * dr * norm_cdf(a2) + term
                    - s * dr * (-a1.abs()).exp() * 0.0    // limit term
            }
            OptionKind::Put => {
                let term = s * dr * v_sqrt_t * (norm_pdf(a1) + a1 * norm_cdf(-a1));
                m * dr * norm_cdf(-a2) - s * dr * norm_cdf(-a1) + term
            }
        };
        return Some(LookbackReport { price: price.max(0.0), a1, a2 });
    }
    let sm_pow = (s / m).powf(-two_b_over_v2);
    let factor = s * dr * (v * v) / (2.0 * b);
    let price = match kind {
        OptionKind::Call => {
            s * dq * norm_cdf(a1) - m * dr * norm_cdf(a2)
                + factor * (sm_pow * norm_cdf(-a3) - (b * t).exp() * norm_cdf(-a1))
        }
        OptionKind::Put => {
            m * dr * norm_cdf(-a2) - s * dq * norm_cdf(-a1)
                + factor * (-sm_pow * norm_cdf(a3) + (b * t).exp() * norm_cdf(a1))
        }
    };
    if !price.is_finite() { return None; }
    Some(LookbackReport { price: price.max(0.0), a1, a2 })
}

fn norm_cdf(x: f64) -> f64 {
    let a1 =  0.254829592_f64;
    let a2 = -0.284496736_f64;
    let a3 =  1.421413741_f64;
    let a4 = -1.453152027_f64;
    let a5 =  1.061405429_f64;
    let p  =  0.3275911_f64;
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let xa = x.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + p * xa);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-xa * xa).exp();
    0.5 * (1.0 + sign * y)
}

fn norm_pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        for bad in [0.0, -1.0, f64::NAN] {
            assert!(price(bad, 95.0, 0.5, 0.05, 0.0, 0.2, OptionKind::Call).is_none());
            assert!(price(100.0, bad, 0.5, 0.05, 0.0, 0.2, OptionKind::Call).is_none());
            assert!(price(100.0, 95.0, bad, 0.05, 0.0, 0.2, OptionKind::Call).is_none());
            assert!(price(100.0, 95.0, 0.5, 0.05, 0.0, bad, OptionKind::Call).is_none());
        }
    }

    #[test]
    fn inconsistent_extreme_rejected() {
        // Call: extreme should be ≤ spot.
        assert!(price(100.0, 105.0, 0.5, 0.05, 0.0, 0.2, OptionKind::Call).is_none());
        // Put: extreme should be ≥ spot.
        assert!(price(100.0, 95.0, 0.5, 0.05, 0.0, 0.2, OptionKind::Put).is_none());
    }

    #[test]
    fn newly_issued_lookback_call_positive() {
        // At issuance, observed_min = S.
        let r = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20, OptionKind::Call).unwrap();
        assert!(r.price > 0.0);
    }

    #[test]
    fn newly_issued_lookback_put_positive() {
        let r = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20, OptionKind::Put).unwrap();
        assert!(r.price > 0.0);
    }

    #[test]
    fn lower_observed_min_inflates_call_price() {
        // The lower the running minimum (a deeper dip already seen),
        // the bigger the floating-strike call payoff.
        let r_at = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20, OptionKind::Call).unwrap();
        let r_low = price(100.0, 80.0, 0.5, 0.05, 0.0, 0.20, OptionKind::Call).unwrap();
        assert!(r_low.price > r_at.price);
    }

    #[test]
    fn higher_observed_max_inflates_put_price() {
        let r_at = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20, OptionKind::Put).unwrap();
        let r_high = price(100.0, 120.0, 0.5, 0.05, 0.0, 0.20, OptionKind::Put).unwrap();
        assert!(r_high.price > r_at.price);
    }

    #[test]
    fn higher_vol_inflates_lookback_price() {
        let r_low = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.10, OptionKind::Call).unwrap();
        let r_high = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.40, OptionKind::Call).unwrap();
        assert!(r_high.price > r_low.price);
    }

    #[test]
    fn b_zero_special_case_finite() {
        // r = q → b = 0 → diverging factor, special-cased.
        let r = price(100.0, 95.0, 0.5, 0.05, 0.05, 0.20, OptionKind::Call).unwrap();
        assert!(r.price.is_finite() && r.price > 0.0);
    }

    #[test]
    fn longer_time_inflates_atm_lookback() {
        let r_short = price(100.0, 100.0, 0.10, 0.05, 0.0, 0.20, OptionKind::Call).unwrap();
        let r_long  = price(100.0, 100.0, 1.00, 0.05, 0.0, 0.20, OptionKind::Call).unwrap();
        assert!(r_long.price > r_short.price);
    }
}
