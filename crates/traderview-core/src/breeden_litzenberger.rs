//! Breeden-Litzenberger Risk-Neutral Density Extraction (1978).
//!
//! Given the spot price S, time-to-expiry T, risk-free rate r, and a
//! strip of call prices C(K) across strikes K, the risk-neutral
//! density of the underlying at expiry is:
//!
//!   f_RN(K) = e^{r·T} · ∂²C(K) / ∂K²
//!
//! Numerical second-derivative via central differences on a sorted
//! strike strip:
//!
//!   f_RN(K_i) ≈ e^{r·T} · (C(K_{i+1}) − 2·C(K_i) + C(K_{i-1})) / (h_i)²
//!
//! where h_i = (K_{i+1} − K_{i-1}) / 2 is the centered half-spacing.
//!
//! Outputs:
//!   - per-strike density (normalized to integrate to 1)
//!   - implied mean (forward)
//!   - implied variance / volatility
//!   - implied skewness / kurtosis
//!
//! Use cases:
//!   - Compare market-implied distribution to model-assumed log-normal
//!   - Estimate tail probabilities of large moves directly from option chain
//!   - Track shifts in market expectations during news / events
//!
//! Pure compute. Companion to `variance_swap_strike`, `iv_term_structure`,
//! `iv_skew_scanner`, `gaussian_copula`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StrikeCall {
    pub strike: f64,
    pub call_price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrikeDensity {
    pub strike: f64,
    pub density: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BreedenLitzenbergerReport {
    pub per_strike_density: Vec<StrikeDensity>,
    pub implied_mean: f64,
    pub implied_variance: f64,
    pub implied_volatility: f64,
    pub implied_skewness: f64,
    pub implied_excess_kurtosis: f64,
    pub n_strikes_used: usize,
}

pub fn extract(
    call_strip: &[StrikeCall],
    risk_free_rate: f64,
    time_to_expiry_years: f64,
) -> Option<BreedenLitzenbergerReport> {
    if call_strip.len() < 5
        || !risk_free_rate.is_finite()
        || !time_to_expiry_years.is_finite()
        || time_to_expiry_years <= 0.0
    {
        return None;
    }
    if call_strip.iter().any(|s| {
        !s.strike.is_finite() || s.strike <= 0.0 || !s.call_price.is_finite() || s.call_price < 0.0
    }) {
        return None;
    }
    let mut sorted = call_strip.to_vec();
    sorted.sort_by(|a, b| {
        a.strike
            .partial_cmp(&b.strike)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let n = sorted.len();
    let discount = (risk_free_rate * time_to_expiry_years).exp();
    // Second-derivative density at each interior strike.
    let mut density = Vec::with_capacity(n - 2);
    let mut raw_strikes = Vec::with_capacity(n - 2);
    for i in 1..(n - 1) {
        let h_l = sorted[i].strike - sorted[i - 1].strike;
        let h_r = sorted[i + 1].strike - sorted[i].strike;
        if h_l <= 0.0 || h_r <= 0.0 {
            continue;
        }
        let avg_h = 0.5 * (h_l + h_r);
        // Central second difference scaled to uniform spacing:
        let d2 = (sorted[i + 1].call_price - 2.0 * sorted[i].call_price + sorted[i - 1].call_price)
            / (avg_h * avg_h);
        let f = discount * d2;
        if f >= 0.0 && f.is_finite() {
            density.push(f);
            raw_strikes.push(sorted[i].strike);
        }
    }
    if density.len() < 3 {
        return None;
    }
    // Trapezoidal integrate to normalize.
    let mut integral = 0.0_f64;
    for i in 0..(density.len() - 1) {
        let dk = raw_strikes[i + 1] - raw_strikes[i];
        integral += 0.5 * (density[i] + density[i + 1]) * dk;
    }
    if integral <= 0.0 {
        return None;
    }
    let normalized: Vec<f64> = density.iter().map(|f| f / integral).collect();
    // Moments via trapezoidal integration on normalized density.
    let mut mean = 0.0_f64;
    for i in 0..(normalized.len() - 1) {
        let dk = raw_strikes[i + 1] - raw_strikes[i];
        let avg = 0.5 * (raw_strikes[i] * normalized[i] + raw_strikes[i + 1] * normalized[i + 1]);
        mean += avg * dk;
    }
    let mut var = 0.0_f64;
    for i in 0..(normalized.len() - 1) {
        let dk = raw_strikes[i + 1] - raw_strikes[i];
        let f_l = (raw_strikes[i] - mean).powi(2) * normalized[i];
        let f_r = (raw_strikes[i + 1] - mean).powi(2) * normalized[i + 1];
        var += 0.5 * (f_l + f_r) * dk;
    }
    let vol = var.max(0.0).sqrt();
    let mut m3 = 0.0_f64;
    let mut m4 = 0.0_f64;
    for i in 0..(normalized.len() - 1) {
        let dk = raw_strikes[i + 1] - raw_strikes[i];
        let f3_l = (raw_strikes[i] - mean).powi(3) * normalized[i];
        let f3_r = (raw_strikes[i + 1] - mean).powi(3) * normalized[i + 1];
        m3 += 0.5 * (f3_l + f3_r) * dk;
        let f4_l = (raw_strikes[i] - mean).powi(4) * normalized[i];
        let f4_r = (raw_strikes[i + 1] - mean).powi(4) * normalized[i + 1];
        m4 += 0.5 * (f4_l + f4_r) * dk;
    }
    let skew = if vol > 0.0 { m3 / vol.powi(3) } else { 0.0 };
    let excess_kurt = if var > 0.0 {
        m4 / (var * var) - 3.0
    } else {
        0.0
    };
    let per_strike: Vec<StrikeDensity> = raw_strikes
        .iter()
        .zip(normalized.iter())
        .map(|(k, d)| StrikeDensity {
            strike: *k,
            density: *d,
        })
        .collect();
    Some(BreedenLitzenbergerReport {
        per_strike_density: per_strike,
        implied_mean: mean,
        implied_variance: var,
        implied_volatility: vol,
        implied_skewness: skew,
        implied_excess_kurtosis: excess_kurt,
        n_strikes_used: density.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(strike: f64, call: f64) -> StrikeCall {
        StrikeCall {
            strike,
            call_price: call,
        }
    }

    /// Constructs a synthetic Black-Scholes call price at strike K for
    /// a forward of `fwd`, vol σ, and time T. Used for test fixtures.
    fn bs_call(fwd: f64, strike: f64, vol: f64, t: f64) -> f64 {
        let sigma_sqrt_t = vol * t.sqrt();
        if sigma_sqrt_t <= 0.0 {
            return (fwd - strike).max(0.0);
        }
        let d1 = ((fwd / strike).ln() + 0.5 * sigma_sqrt_t * sigma_sqrt_t) / sigma_sqrt_t;
        let d2 = d1 - sigma_sqrt_t;
        fwd * norm_cdf(d1) - strike * norm_cdf(d2)
    }

    fn norm_cdf(z: f64) -> f64 {
        0.5 * (1.0 + erf(z / std::f64::consts::SQRT_2))
    }
    fn erf(x: f64) -> f64 {
        let sign = if x < 0.0 { -1.0 } else { 1.0 };
        let x = x.abs();
        let t = 1.0 / (1.0 + 0.327_591_1 * x);
        let y = 1.0
            - (((((1.061_405_429 * t - 1.453_152_027) * t) + 1.421_413_741) * t - 0.284_496_736)
                * t
                + 0.254_829_592)
                * t
                * (-x * x).exp();
        sign * y
    }

    #[test]
    fn too_few_strikes_returns_none() {
        let strikes = vec![k(95.0, 6.0), k(100.0, 3.0), k(105.0, 1.0)];
        assert!(extract(&strikes, 0.0, 0.5).is_none());
    }

    #[test]
    fn invalid_inputs_return_none() {
        let s = vec![k(95.0, 6.0); 5];
        assert!(extract(&s, 0.0, 0.0).is_none());
        assert!(extract(&s, f64::NAN, 0.5).is_none());
        let bad_strike = vec![
            k(0.0, 1.0),
            k(100.0, 3.0),
            k(110.0, 1.0),
            k(120.0, 0.5),
            k(130.0, 0.1),
        ];
        assert!(extract(&bad_strike, 0.05, 0.5).is_none());
    }

    #[test]
    fn black_scholes_chain_recovers_known_mean() {
        // Forward = 100, vol = 20%, T = 0.5. Generate calls across many strikes.
        let fwd = 100.0_f64;
        let vol = 0.20_f64;
        let t = 0.5_f64;
        let strikes: Vec<StrikeCall> = (60..=140)
            .step_by(2)
            .map(|kf| {
                let strike = kf as f64;
                k(strike, bs_call(fwd, strike, vol, t))
            })
            .collect();
        let r = extract(&strikes, 0.0, t).unwrap();
        // Implied mean should be ≈ forward.
        assert!(
            (r.implied_mean - fwd).abs() < 2.0,
            "mean = {}, expected ≈ {}",
            r.implied_mean,
            fwd
        );
    }

    #[test]
    fn black_scholes_chain_recovers_log_normal_variance() {
        // For log-normal with forward 100, vol 20%, T=0.5:
        //   E[S_T] = 100, Var[S_T] = 100² · (e^{vol²·T} - 1) ≈ 200.
        let fwd = 100.0_f64;
        let vol = 0.20_f64;
        let t = 0.5_f64;
        let strikes: Vec<StrikeCall> = (60..=140)
            .step_by(2)
            .map(|kf| {
                let strike = kf as f64;
                k(strike, bs_call(fwd, strike, vol, t))
            })
            .collect();
        let r = extract(&strikes, 0.0, t).unwrap();
        let expected_var = fwd * fwd * ((vol * vol * t).exp() - 1.0);
        let rel = (r.implied_variance - expected_var).abs() / expected_var;
        assert!(
            rel < 0.30,
            "variance = {}, expected ≈ {}, rel diff {:.2}",
            r.implied_variance,
            expected_var,
            rel
        );
    }

    #[test]
    fn density_normalized_to_one() {
        let fwd = 100.0_f64;
        let vol = 0.20_f64;
        let t = 0.5_f64;
        let strikes: Vec<StrikeCall> = (60..=140)
            .step_by(2)
            .map(|kf| {
                let strike = kf as f64;
                k(strike, bs_call(fwd, strike, vol, t))
            })
            .collect();
        let r = extract(&strikes, 0.0, t).unwrap();
        let mut integral = 0.0_f64;
        for i in 0..(r.per_strike_density.len() - 1) {
            let dk = r.per_strike_density[i + 1].strike - r.per_strike_density[i].strike;
            integral +=
                0.5 * (r.per_strike_density[i].density + r.per_strike_density[i + 1].density) * dk;
        }
        assert!((integral - 1.0).abs() < 1e-9);
    }

    #[test]
    fn skewness_near_zero_for_at_the_money_lognormal() {
        let fwd = 100.0_f64;
        let vol = 0.10_f64;
        let t = 0.5_f64;
        let strikes: Vec<StrikeCall> = (60..=140)
            .step_by(2)
            .map(|kf| {
                let strike = kf as f64;
                k(strike, bs_call(fwd, strike, vol, t))
            })
            .collect();
        let r = extract(&strikes, 0.0, t).unwrap();
        // Lognormal skew is small at low vol.
        assert!(
            r.implied_skewness.abs() < 0.5,
            "low-vol lognormal: skew {} should be small",
            r.implied_skewness
        );
    }

    #[test]
    fn n_strikes_reported() {
        let fwd = 100.0_f64;
        let vol = 0.20_f64;
        let t = 0.5_f64;
        let strikes: Vec<StrikeCall> = (80..=120)
            .step_by(2)
            .map(|kf| {
                let strike = kf as f64;
                k(strike, bs_call(fwd, strike, vol, t))
            })
            .collect();
        let n_input = strikes.len();
        let r = extract(&strikes, 0.0, t).unwrap();
        // 2 boundary strikes dropped → interior count.
        assert_eq!(r.n_strikes_used, n_input - 2);
    }
}
