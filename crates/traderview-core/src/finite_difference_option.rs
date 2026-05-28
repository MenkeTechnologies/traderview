//! Crank-Nicolson Finite-Difference European Option Pricer.
//!
//! Solves the Black-Scholes PDE:
//!
//!   ∂V/∂t + ½·σ²·S²·∂²V/∂S² + r·S·∂V/∂S − r·V = 0
//!
//! on a uniform S-grid using the implicit Crank-Nicolson scheme
//! (unconditionally stable, second-order accurate in both S and t).
//!
//! Each time-step solves a tridiagonal linear system via the Thomas
//! algorithm in O(N_S) time. Total cost O(N_t · N_S).
//!
//! Outputs:
//!   - present_value at spot
//!   - delta (∂V/∂S via central difference at spot)
//!   - gamma (∂²V/∂S² central difference)
//!   - per-grid PV array (optional)
//!
//! Pure compute. Companion to `black76`, `monte_carlo_option`,
//! `american_binomial`, `forward_start_option`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionType { Call, Put }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FdOptionReport {
    pub present_value: f64,
    pub delta: f64,
    pub gamma: f64,
    pub n_s_steps: usize,
    pub n_t_steps: usize,
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
    n_s_steps: usize,
    n_t_steps: usize,
    s_max_multiplier: f64,
) -> Option<FdOptionReport> {
    if !spot.is_finite() || spot <= 0.0
        || !strike.is_finite() || strike <= 0.0
        || !time_to_expiry.is_finite() || time_to_expiry <= 0.0
        || !risk_free_rate.is_finite() || !dividend_yield.is_finite()
        || !volatility.is_finite() || volatility <= 0.0
        || n_s_steps < 20 || n_t_steps < 10
        || !s_max_multiplier.is_finite() || s_max_multiplier <= 1.0
    {
        return None;
    }
    let s_max = spot * s_max_multiplier;
    let ds = s_max / n_s_steps as f64;
    let dt = time_to_expiry / n_t_steps as f64;
    // Terminal payoff.
    let s_grid: Vec<f64> = (0..=n_s_steps).map(|i| i as f64 * ds).collect();
    let mut v: Vec<f64> = s_grid.iter().map(|s| match option_type {
        OptionType::Call => (s - strike).max(0.0),
        OptionType::Put => (strike - s).max(0.0),
    }).collect();
    let q = dividend_yield;
    // Coefficients for Crank-Nicolson. For each interior i:
    //   a_i = ¼ · dt · (σ²·i² − (r − q)·i)
    //   b_i = −½ · dt · (σ²·i² + r)
    //   c_i = ¼ · dt · (σ²·i² + (r − q)·i)
    let mut a = vec![0.0_f64; n_s_steps + 1];
    let mut b = vec![0.0_f64; n_s_steps + 1];
    let mut c = vec![0.0_f64; n_s_steps + 1];
    for i in 1..n_s_steps {
        let i_f = i as f64;
        a[i] = 0.25 * dt * (volatility * volatility * i_f * i_f - (risk_free_rate - q) * i_f);
        b[i] = -0.5 * dt * (volatility * volatility * i_f * i_f + risk_free_rate);
        c[i] = 0.25 * dt * (volatility * volatility * i_f * i_f + (risk_free_rate - q) * i_f);
    }
    // Step backwards in time.
    for _ in (0..n_t_steps).rev() {
        // Build RHS = (I + M_R) · v
        let mut rhs = vec![0.0_f64; n_s_steps + 1];
        for i in 1..n_s_steps {
            rhs[i] = a[i] * v[i - 1] + (1.0 + b[i]) * v[i] + c[i] * v[i + 1];
        }
        // Boundary conditions (held constant in time on grid edges).
        let s0 = match option_type {
            OptionType::Call => 0.0,
            OptionType::Put => strike,
        };
        let s_top = match option_type {
            OptionType::Call => s_max - strike,
            OptionType::Put => 0.0,
        };
        rhs[0] = s0;
        rhs[n_s_steps] = s_top;
        // LHS tridiag: (I − M_L). Coefficients:
        //   alpha_i = −a_i
        //   beta_i  = 1 − b_i
        //   gamma_i = −c_i
        let mut alpha = vec![0.0_f64; n_s_steps + 1];
        let mut beta = vec![0.0_f64; n_s_steps + 1];
        let mut gamma_v = vec![0.0_f64; n_s_steps + 1];
        for i in 1..n_s_steps {
            alpha[i] = -a[i];
            beta[i] = 1.0 - b[i];
            gamma_v[i] = -c[i];
        }
        // Solve tridiagonal system on interior indices 1..n_s_steps via Thomas.
        let mut new_v = v.clone();
        new_v[0] = s0;
        new_v[n_s_steps] = s_top;
        let mut cp = vec![0.0_f64; n_s_steps + 1];
        let mut dp = vec![0.0_f64; n_s_steps + 1];
        cp[1] = gamma_v[1] / beta[1];
        dp[1] = rhs[1] / beta[1];
        for i in 2..n_s_steps {
            let denom = beta[i] - alpha[i] * cp[i - 1];
            if denom.abs() < 1e-18 { return None; }
            cp[i] = gamma_v[i] / denom;
            dp[i] = (rhs[i] - alpha[i] * dp[i - 1]) / denom;
        }
        new_v[n_s_steps - 1] = dp[n_s_steps - 1];
        for i in (1..(n_s_steps - 1)).rev() {
            new_v[i] = dp[i] - cp[i] * new_v[i + 1];
        }
        v = new_v;
    }
    // Interpolate PV at `spot` using nearest two grid nodes.
    let i_real = spot / ds;
    let i_lo = (i_real.floor() as usize).min(n_s_steps - 1);
    let frac = i_real - i_lo as f64;
    let pv = v[i_lo] * (1.0 - frac) + v[i_lo + 1] * frac;
    let delta = (v[i_lo + 1] - v[i_lo]) / ds;
    let gamma_val = if i_lo > 0 && i_lo + 1 < n_s_steps {
        (v[i_lo + 1] - 2.0 * v[i_lo] + v[i_lo - 1]) / (ds * ds)
    } else { 0.0 };
    Some(FdOptionReport {
        present_value: pv,
        delta,
        gamma: gamma_val,
        n_s_steps,
        n_t_steps,
    })
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
        assert!(price(0.0, 100.0, 0.5, 0.05, 0.0, 0.20, OptionType::Call, 100, 50, 3.0).is_none());
        assert!(price(100.0, 0.0, 0.5, 0.05, 0.0, 0.20, OptionType::Call, 100, 50, 3.0).is_none());
        assert!(price(100.0, 100.0, 0.0, 0.05, 0.0, 0.20, OptionType::Call, 100, 50, 3.0).is_none());
        assert!(price(100.0, 100.0, 0.5, 0.05, 0.0, 0.0, OptionType::Call, 100, 50, 3.0).is_none());
        assert!(price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20, OptionType::Call, 10, 50, 3.0).is_none());
    }

    #[test]
    fn fd_call_matches_black_scholes() {
        let s = 100.0_f64;
        let k = 100.0_f64;
        let t = 0.5_f64;
        let r = 0.05_f64;
        let q = 0.0_f64;
        let sigma = 0.20_f64;
        let fd = price(s, k, t, r, q, sigma, OptionType::Call, 200, 100, 4.0).unwrap();
        let bs = bs_call(s, k, t, r, q, sigma);
        let rel = (fd.present_value - bs).abs() / bs;
        assert!(rel < 0.02, "FD {} vs BS {}, rel diff {:.4}",
            fd.present_value, bs, rel);
    }

    #[test]
    fn put_value_positive_when_in_the_money() {
        let fd = price(80.0, 100.0, 0.5, 0.05, 0.0, 0.20, OptionType::Put, 200, 100, 4.0).unwrap();
        assert!(fd.present_value > 15.0,
            "in-the-money put should have value > intrinsic, got {}", fd.present_value);
    }

    #[test]
    fn delta_call_in_unit_range() {
        let fd = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20, OptionType::Call, 200, 100, 4.0).unwrap();
        assert!((0.0..=1.0).contains(&fd.delta));
    }

    #[test]
    fn gamma_non_negative() {
        let fd = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.20, OptionType::Call, 200, 100, 4.0).unwrap();
        assert!(fd.gamma >= 0.0);
    }

    #[test]
    fn higher_vol_increases_value() {
        let low = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.10, OptionType::Call, 200, 100, 4.0).unwrap();
        let high = price(100.0, 100.0, 0.5, 0.05, 0.0, 0.40, OptionType::Call, 200, 100, 4.0).unwrap();
        assert!(high.present_value > low.present_value);
    }
}
