//! Shared Black-Scholes-Merton vanilla pricer — the inline copy
//! calendar_spread carried, promoted to a module so new code stops
//! duplicating it. Same A&S 26.2.17 normal CDF (max err 7.5e-8), so
//! callers that migrated keep identical numerics.

/// Abramowitz & Stegun 26.2.17, max err 7.5e-8.
pub fn norm_cdf(x: f64) -> f64 {
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

/// European call with continuous dividend yield `q`.
pub fn call(s: f64, k: f64, t: f64, r: f64, q: f64, sigma: f64) -> f64 {
    let sqrt_t = t.sqrt();
    let d1 = ((s / k).ln() + (r - q + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;
    s * (-q * t).exp() * norm_cdf(d1) - k * (-r * t).exp() * norm_cdf(d2)
}

/// European put with continuous dividend yield `q`.
pub fn put(s: f64, k: f64, t: f64, r: f64, q: f64, sigma: f64) -> f64 {
    let sqrt_t = t.sqrt();
    let d1 = ((s / k).ln() + (r - q + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;
    k * (-r * t).exp() * (1.0 - norm_cdf(d2)) - s * (-q * t).exp() * (1.0 - norm_cdf(d1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_call_parity_holds() {
        let (s, k, t, r, q, sig) = (100.0, 95.0, 0.5, 0.04, 0.01, 0.3);
        let lhs = call(s, k, t, r, q, sig) - put(s, k, t, r, q, sig);
        let rhs = s * (-q * t).exp() - k * (-r * t).exp();
        assert!((lhs - rhs).abs() < 1e-12, "{lhs} vs {rhs}");
    }

    #[test]
    fn deep_itm_call_approaches_forward_intrinsic() {
        let c = call(100.0, 1.0, 0.25, 0.05, 0.0, 0.2);
        let want = 100.0 - 1.0 * (-0.05_f64 * 0.25).exp();
        assert!((c - want).abs() < 1e-9);
    }

    #[test]
    fn atm_call_matches_reference_value() {
        // S=K=100, T=1, r=5%, q=0, σ=20%: canonical BS value 10.4506.
        let c = call(100.0, 100.0, 1.0, 0.05, 0.0, 0.2);
        assert!((c - 10.4506).abs() < 1e-3, "{c}");
    }
}
