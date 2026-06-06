//! Kelly Criterion — optimal fractional position sizing (Kelly 1956).
//!
//! Two formulations:
//!
//! 1. **Discrete (win/loss bet)**:
//!
//!    f* = (p · b − q) / b
//!
//!    where p = win probability, q = 1 − p, b = win/loss payoff ratio
//!    (avg_win / avg_loss).
//!
//! 2. **Continuous (return distribution)** for normally-distributed
//!    excess returns:
//!
//!    f* = (μ − r_f) / σ²
//!
//! Companion outputs:
//!   - **half_kelly** = 0.5 · f* (conservative practitioner's adjustment)
//!   - **quarter_kelly** = 0.25 · f*
//!   - **expected_growth_rate** = f* · (μ − r_f) − ½·f*²·σ²
//!
//! Pure compute. Companion to `expectancy_per_trade`, `vol_targeting_sizer`,
//! `risk_adjusted_ratios`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KellyReport {
    pub full_kelly_fraction: f64,
    pub half_kelly: f64,
    pub quarter_kelly: f64,
    /// `None` when the growth rate is undefined — e.g. p = 1 (all wins)
    /// pushes f_full to 1 and the log term becomes 0 · log(0) → NaN.
    /// serde_json refuses non-finite floats, so we surface `None` instead.
    pub expected_growth_rate: Option<f64>,
}

/// Discrete win/loss Kelly:  f* = (p·b − q) / b.
pub fn discrete(win_probability: f64, win_loss_payoff_ratio: f64) -> Option<KellyReport> {
    if !win_probability.is_finite()
        || !(0.0..=1.0).contains(&win_probability)
        || !win_loss_payoff_ratio.is_finite()
        || win_loss_payoff_ratio <= 0.0
    {
        return None;
    }
    let p = win_probability;
    let q = 1.0 - p;
    let b = win_loss_payoff_ratio;
    let f_full = (p * b - q) / b;
    // Expected geometric growth rate at full Kelly. Two NaN traps to dodge:
    //   1) p == 1 → f_full == 1 → ln(1 - f_full) = ln(0) = -∞,
    //      then q * -∞ = 0 * -∞ = NaN in IEEE.
    //   2) f_full * b < -1 → ln(1 + f_full * b) of a non-positive
    //      → NaN. Can't happen here while f_full > 0 (guarded), but
    //      defensive .is_finite() catches future widening of the branch.
    let expected_growth = if f_full > 0.0 && f_full < 1.0 {
        let v = p * (1.0 + f_full * b).ln() + q * (1.0 - f_full).ln();
        v.is_finite().then_some(v)
    } else if f_full > 0.0 {
        // Edge case p == 1: log term contributes 0 by convention since
        // q = 0; the surviving term is p · ln(1 + b) which is finite.
        let v = p * (1.0 + f_full * b).ln();
        v.is_finite().then_some(v)
    } else {
        Some(0.0)
    };
    Some(KellyReport {
        full_kelly_fraction: f_full,
        half_kelly: 0.5 * f_full,
        quarter_kelly: 0.25 * f_full,
        expected_growth_rate: expected_growth,
    })
}

/// Continuous Kelly:  f* = (μ − r_f) / σ²  for normal returns.
pub fn continuous(
    expected_return: f64,
    return_volatility: f64,
    risk_free_rate: f64,
) -> Option<KellyReport> {
    if !expected_return.is_finite()
        || !return_volatility.is_finite()
        || return_volatility <= 0.0
        || !risk_free_rate.is_finite()
    {
        return None;
    }
    let excess = expected_return - risk_free_rate;
    let variance = return_volatility * return_volatility;
    let f_full = excess / variance;
    let expected_growth = f_full * excess - 0.5 * f_full * f_full * variance;
    Some(KellyReport {
        full_kelly_fraction: f_full,
        half_kelly: 0.5 * f_full,
        quarter_kelly: 0.25 * f_full,
        expected_growth_rate: expected_growth.is_finite().then_some(expected_growth),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discrete_invalid_inputs_return_none() {
        assert!(discrete(-0.1, 2.0).is_none());
        assert!(discrete(1.1, 2.0).is_none());
        assert!(discrete(0.5, 0.0).is_none());
        assert!(discrete(0.5, -1.0).is_none());
        assert!(discrete(f64::NAN, 2.0).is_none());
    }

    #[test]
    fn classic_discrete_example() {
        // p = 0.6, b = 1.0 → f* = (0.6·1 − 0.4)/1 = 0.20.
        let r = discrete(0.6, 1.0).unwrap();
        assert!((r.full_kelly_fraction - 0.20).abs() < 1e-12);
        assert!((r.half_kelly - 0.10).abs() < 1e-12);
        assert!((r.quarter_kelly - 0.05).abs() < 1e-12);
    }

    #[test]
    fn negative_edge_yields_negative_kelly() {
        // p = 0.4, b = 1.0 → f* = (0.4 − 0.6)/1 = -0.20.
        let r = discrete(0.4, 1.0).unwrap();
        assert!(r.full_kelly_fraction < 0.0);
    }

    #[test]
    fn continuous_invalid_inputs_return_none() {
        assert!(continuous(0.05, 0.0, 0.0).is_none());
        assert!(continuous(0.05, -0.1, 0.0).is_none());
        assert!(continuous(f64::NAN, 0.10, 0.0).is_none());
        assert!(continuous(0.05, 0.10, f64::NAN).is_none());
    }

    #[test]
    fn continuous_formula_matches_excess_over_variance() {
        // μ = 0.10, r_f = 0.03, σ = 0.20 → f* = 0.07 / 0.04 = 1.75.
        let r = continuous(0.10, 0.20, 0.03).unwrap();
        assert!((r.full_kelly_fraction - 1.75).abs() < 1e-12);
    }

    #[test]
    fn expected_growth_non_negative_under_positive_edge() {
        let r = discrete(0.6, 2.0).unwrap();
        assert!(r.expected_growth_rate.unwrap() > 0.0);
    }

    #[test]
    fn all_wins_yields_finite_growth_rate_not_nan() {
        // Regression: p=1.0 used to compute q * ln(1 - f_full) = 0 * ln(0)
        // = 0 * -∞ = NaN, which broke JSON serialization. With the fix,
        // we either get a finite growth rate or `None`.
        let r = discrete(1.0, 2.0).unwrap();
        if let Some(g) = r.expected_growth_rate {
            assert!(g.is_finite(), "p=1 must not produce NaN; got {g}");
        }
        let json = serde_json::to_string(&r).expect("must serialize");
        assert!(json.contains("expected_growth_rate"));
    }

    #[test]
    fn fractional_kelly_proportional_to_full() {
        let r = continuous(0.10, 0.20, 0.03).unwrap();
        assert!((r.half_kelly * 2.0 - r.full_kelly_fraction).abs() < 1e-12);
        assert!((r.quarter_kelly * 4.0 - r.full_kelly_fraction).abs() < 1e-12);
    }
}
