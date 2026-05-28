//! Monte Carlo European Option Pricer (geometric Brownian motion).
//!
//! Simulates n_paths terminal prices under risk-neutral GBM:
//!
//!   S_T = S_0 · exp((r − q − ½·σ²)·T + σ·√T·z)    z ~ N(0,1)
//!
//! Payoff:
//!   Call: max(S_T − K, 0)
//!   Put:  max(K − S_T, 0)
//!
//! Present value = exp(−r·T) · mean(payoff).
//!
//! Antithetic-variate variance reduction available via `use_antithetic = true`
//! (pair each z draw with −z, halving variance approximately for free).
//!
//! Pure compute. Companion to `finite_difference_option`,
//! `american_binomial`, `forward_start_option`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionType { Call, Put }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McOptionReport {
    pub present_value: f64,
    pub standard_error: f64,
    pub ci_95_lower: f64,
    pub ci_95_upper: f64,
    pub n_paths: usize,
}

#[allow(clippy::too_many_arguments)]
pub fn price(
    spot: f64,
    strike: f64,
    time_to_expiry: f64,
    risk_free_rate: f64,
    dividend_yield: f64,
    volatility: f64,
    option_type: OptionType,
    n_paths: usize,
    seed: u64,
    use_antithetic: bool,
) -> Option<McOptionReport> {
    if !spot.is_finite() || spot <= 0.0
        || !strike.is_finite() || strike <= 0.0
        || !time_to_expiry.is_finite() || time_to_expiry <= 0.0
        || !risk_free_rate.is_finite() || !dividend_yield.is_finite()
        || !volatility.is_finite() || volatility <= 0.0
        || n_paths < 100 {
        return None;
    }
    let drift = (risk_free_rate - dividend_yield - 0.5 * volatility * volatility) * time_to_expiry;
    let vol_sqrt_t = volatility * time_to_expiry.sqrt();
    let discount = (-risk_free_rate * time_to_expiry).exp();
    let mut state = seed;
    let mut payoffs = Vec::with_capacity(n_paths);
    let n_draws = if use_antithetic { n_paths / 2 } else { n_paths };
    for _ in 0..n_draws {
        let z = standard_normal(&mut state);
        let s_pos = spot * (drift + vol_sqrt_t * z).exp();
        let p_pos = match option_type {
            OptionType::Call => (s_pos - strike).max(0.0),
            OptionType::Put => (strike - s_pos).max(0.0),
        };
        payoffs.push(p_pos);
        if use_antithetic {
            let s_neg = spot * (drift - vol_sqrt_t * z).exp();
            let p_neg = match option_type {
                OptionType::Call => (s_neg - strike).max(0.0),
                OptionType::Put => (strike - s_neg).max(0.0),
            };
            payoffs.push(p_neg);
        }
    }
    let n_f = payoffs.len() as f64;
    let mean: f64 = payoffs.iter().sum::<f64>() / n_f;
    let var: f64 = payoffs.iter().map(|p| (p - mean).powi(2)).sum::<f64>() / (n_f - 1.0);
    let se = (var / n_f).sqrt() * discount;
    let pv = mean * discount;
    Some(McOptionReport {
        present_value: pv,
        standard_error: se,
        ci_95_lower: pv - 1.96 * se,
        ci_95_upper: pv + 1.96 * se,
        n_paths: payoffs.len(),
    })
}

fn standard_normal(state: &mut u64) -> f64 {
    *state = state.wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let u1 = ((*state >> 32) as f64 / u32::MAX as f64).max(1e-12);
    *state = state.wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let u2 = (*state >> 32) as f64 / u32::MAX as f64;
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bs_call(s: f64, k: f64, t: f64, r: f64, q: f64, sigma: f64) -> f64 {
        let sigma_sqrt_t = sigma * t.sqrt();
        let d1 = ((s / k).ln() + (r - q + 0.5 * sigma * sigma) * t) / sigma_sqrt_t;
        let d2 = d1 - sigma_sqrt_t;
        s * (-q * t).exp() * norm_cdf(d1) - k * (-r * t).exp() * norm_cdf(d2)
    }
    fn norm_cdf(z: f64) -> f64 {
        0.5 * (1.0 + erf(z / std::f64::consts::SQRT_2))
    }
    fn erf(x: f64) -> f64 {
        let sign = if x < 0.0 { -1.0 } else { 1.0 };
        let x = x.abs();
        let t = 1.0 / (1.0 + 0.327_591_1 * x);
        let y = 1.0 - (((((1.061_405_429 * t - 1.453_152_027) * t)
            + 1.421_413_741) * t - 0.284_496_736) * t + 0.254_829_592) * t * (-x * x).exp();
        sign * y
    }

    #[test]
    fn invalid_inputs_return_none() {
        assert!(price(0.0, 100.0, 0.5, 0.05, 0.0, 0.20,
            OptionType::Call, 1000, 42, false).is_none());
        assert!(price(100.0, 100.0, 0.0, 0.05, 0.0, 0.20,
            OptionType::Call, 1000, 42, false).is_none());
        assert!(price(100.0, 100.0, 0.5, 0.05, 0.0, 0.0,
            OptionType::Call, 1000, 42, false).is_none());
        assert!(price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20,
            OptionType::Call, 50, 42, false).is_none());
    }

    #[test]
    fn mc_call_matches_black_scholes() {
        let s = 100.0_f64;
        let k = 100.0_f64;
        let t = 0.5_f64;
        let r = 0.05_f64;
        let sigma = 0.20_f64;
        let mc = price(s, k, t, r, 0.0, sigma,
            OptionType::Call, 200_000, 42, true).unwrap();
        let bs = bs_call(s, k, t, r, 0.0, sigma);
        let rel = (mc.present_value - bs).abs() / bs;
        assert!(rel < 0.02, "MC {} vs BS {}, rel diff {:.4}",
            mc.present_value, bs, rel);
    }

    #[test]
    fn antithetic_reduces_standard_error() {
        let s = 100.0_f64;
        let plain = price(s, 100.0, 0.5, 0.05, 0.0, 0.20,
            OptionType::Call, 20_000, 42, false).unwrap();
        let anti = price(s, 100.0, 0.5, 0.05, 0.0, 0.20,
            OptionType::Call, 20_000, 42, true).unwrap();
        assert!(anti.standard_error < plain.standard_error,
            "antithetic SE {} should be lower than plain {}",
            anti.standard_error, plain.standard_error);
    }

    #[test]
    fn deterministic_for_fixed_seed() {
        let p1 = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20,
            OptionType::Call, 1000, 42, false).unwrap();
        let p2 = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20,
            OptionType::Call, 1000, 42, false).unwrap();
        assert_eq!(p1.present_value, p2.present_value);
    }

    #[test]
    fn ci_brackets_pv() {
        let p = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20,
            OptionType::Call, 10_000, 42, true).unwrap();
        assert!(p.ci_95_lower <= p.present_value);
        assert!(p.ci_95_upper >= p.present_value);
    }

    #[test]
    fn put_value_positive_when_in_the_money() {
        let p = price(80.0, 100.0, 0.5, 0.05, 0.0, 0.20,
            OptionType::Put, 50_000, 42, true).unwrap();
        assert!(p.present_value > 15.0);
    }

    #[test]
    fn n_paths_reported() {
        let p = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20,
            OptionType::Call, 1000, 42, false).unwrap();
        assert_eq!(p.n_paths, 1000);
    }
}
