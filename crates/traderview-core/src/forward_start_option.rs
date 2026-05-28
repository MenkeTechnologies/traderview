//! Forward-Start European Option Pricer (Rubinstein 1991).
//!
//! A forward-start option's strike is set at a future date t₁ as a
//! function of the then-prevailing spot:
//!
//!   K_{t₁} = α · S_{t₁}                (α = "moneyness factor")
//!
//! For α = 1 the strike is set ATM at t₁. Time-to-expiry t₂ runs from
//! the strike-set date.
//!
//! Closed-form (Rubinstein 1991 / Haug):
//!
//!   C_0 = S_0 · e^{−q·t₁} · BS(1, α, t₂, r, q, σ)
//!
//! i.e. the discounted spot multiplied by a Black-Scholes call on a
//! unit underlying with strike α and time t₂. The same formula
//! applies to puts.
//!
//! Used in:
//!   - Cliquet / ratchet options (chain of forward-start)
//!   - Employee stock options struck at future grant date
//!   - Volatility-trading strategies sensitive to forward IV
//!
//! Pure compute. Companion to `cliquet_option`, `chooser_option`,
//! `finite_difference_option`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionType { Call, Put }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ForwardStartReport {
    pub present_value: f64,
    pub forward_atm_value_per_dollar: f64,
    pub moneyness_factor_alpha: f64,
    pub time_to_strike_set: f64,
    pub time_strike_to_expiry: f64,
}

#[allow(clippy::too_many_arguments)]
pub fn price(
    spot: f64,
    moneyness_factor_alpha: f64,
    time_to_strike_set_years: f64,
    time_strike_to_expiry_years: f64,
    risk_free_rate: f64,
    dividend_yield: f64,
    volatility: f64,
    option_type: OptionType,
) -> Option<ForwardStartReport> {
    if !spot.is_finite() || spot <= 0.0
        || !moneyness_factor_alpha.is_finite() || moneyness_factor_alpha <= 0.0
        || !time_to_strike_set_years.is_finite() || time_to_strike_set_years < 0.0
        || !time_strike_to_expiry_years.is_finite() || time_strike_to_expiry_years <= 0.0
        || !risk_free_rate.is_finite() || !dividend_yield.is_finite()
        || !volatility.is_finite() || volatility <= 0.0 {
        return None;
    }
    // BS price for spot=1, strike=α, time=t₂, rates r/q, vol σ.
    let bs_unit = black_scholes(1.0, moneyness_factor_alpha,
        time_strike_to_expiry_years, risk_free_rate, dividend_yield, volatility, option_type);
    let discount_to_strike_set = (-dividend_yield * time_to_strike_set_years).exp();
    let pv = spot * discount_to_strike_set * bs_unit;
    Some(ForwardStartReport {
        present_value: pv,
        forward_atm_value_per_dollar: bs_unit,
        moneyness_factor_alpha,
        time_to_strike_set: time_to_strike_set_years,
        time_strike_to_expiry: time_strike_to_expiry_years,
    })
}

fn black_scholes(
    s: f64, k: f64, t: f64, r: f64, q: f64, sigma: f64, opt: OptionType,
) -> f64 {
    let sigma_sqrt_t = sigma * t.sqrt();
    if sigma_sqrt_t <= 0.0 {
        return match opt {
            OptionType::Call => (s - k).max(0.0),
            OptionType::Put => (k - s).max(0.0),
        };
    }
    let d1 = ((s / k).ln() + (r - q + 0.5 * sigma * sigma) * t) / sigma_sqrt_t;
    let d2 = d1 - sigma_sqrt_t;
    match opt {
        OptionType::Call => s * (-q * t).exp() * norm_cdf(d1)
            - k * (-r * t).exp() * norm_cdf(d2),
        OptionType::Put => k * (-r * t).exp() * norm_cdf(-d2)
            - s * (-q * t).exp() * norm_cdf(-d1),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(price(0.0, 1.0, 0.25, 0.5, 0.05, 0.0, 0.20, OptionType::Call).is_none());
        assert!(price(100.0, 0.0, 0.25, 0.5, 0.05, 0.0, 0.20, OptionType::Call).is_none());
        assert!(price(100.0, 1.0, -0.1, 0.5, 0.05, 0.0, 0.20, OptionType::Call).is_none());
        assert!(price(100.0, 1.0, 0.25, 0.0, 0.05, 0.0, 0.20, OptionType::Call).is_none());
        assert!(price(100.0, 1.0, 0.25, 0.5, 0.05, 0.0, 0.0, OptionType::Call).is_none());
    }

    #[test]
    fn at_strike_set_time_zero_collapses_to_european() {
        // t₁ = 0 → no discount; PV should match a standard call with
        // K = α·S₀, T = t₂.
        let s = 100.0_f64;
        let alpha = 1.0_f64;
        let r = price(s, alpha, 0.0, 0.5, 0.05, 0.0, 0.20, OptionType::Call).unwrap();
        let bs = black_scholes(s, alpha * s, 0.5, 0.05, 0.0, 0.20, OptionType::Call);
        assert!((r.present_value - bs).abs() < 1e-9);
    }

    #[test]
    fn higher_vol_increases_value() {
        let low = price(100.0, 1.0, 0.25, 0.5, 0.05, 0.0, 0.10, OptionType::Call).unwrap();
        let high = price(100.0, 1.0, 0.25, 0.5, 0.05, 0.0, 0.40, OptionType::Call).unwrap();
        assert!(high.present_value > low.present_value);
    }

    #[test]
    fn dividend_yield_reduces_forward_start_value() {
        let no_div = price(100.0, 1.0, 0.25, 0.5, 0.05, 0.0, 0.20, OptionType::Call).unwrap();
        let div = price(100.0, 1.0, 0.25, 0.5, 0.05, 0.03, 0.20, OptionType::Call).unwrap();
        assert!(div.present_value < no_div.present_value);
    }

    #[test]
    fn higher_alpha_reduces_call_value() {
        // Higher strike-as-multiple-of-spot → less in-the-money → less valuable.
        let atm = price(100.0, 1.0, 0.25, 0.5, 0.05, 0.0, 0.20, OptionType::Call).unwrap();
        let otm = price(100.0, 1.10, 0.25, 0.5, 0.05, 0.0, 0.20, OptionType::Call).unwrap();
        assert!(otm.present_value < atm.present_value);
    }

    #[test]
    fn put_call_asymmetric_for_non_unit_alpha() {
        let call = price(100.0, 1.10, 0.25, 0.5, 0.05, 0.0, 0.20, OptionType::Call).unwrap();
        let put = price(100.0, 1.10, 0.25, 0.5, 0.05, 0.0, 0.20, OptionType::Put).unwrap();
        // At α > 1, ITM put > OTM call (put pays K − S_T).
        assert!(put.present_value > call.present_value);
    }

    #[test]
    fn fields_passed_through_to_report() {
        let r = price(100.0, 0.95, 0.25, 0.5, 0.05, 0.02, 0.20, OptionType::Call).unwrap();
        assert!((r.moneyness_factor_alpha - 0.95).abs() < 1e-12);
        assert!((r.time_to_strike_set - 0.25).abs() < 1e-12);
        assert!((r.time_strike_to_expiry - 0.50).abs() < 1e-12);
    }
}
