//! Probability of profit (POP) — chance the underlying finishes in a
//! strategy's profitable zone at expiry under the lognormal model.
//!
//!   ln S_T ~ N( ln S + (μ − σ²/2)T, σ²T )
//!
//! Caller supplies the profitable price zone as optional lower/upper
//! breakevens (credit spreads profit BETWEEN them, long strangles
//! OUTSIDE — `profit_between` picks the side). Drift μ defaults to the
//! risk-neutral r − q; passing `zero_drift = true` uses μ = 0, the
//! convention most retail POP screens use.
//!
//! Pure compute. Companion to `probability_of_touch`, `heston`,
//! `monte_carlo`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PopInput {
    pub spot: f64,
    /// Annualized implied volatility (decimal, e.g. 0.25).
    pub iv: f64,
    pub time_to_expiry_years: f64,
    #[serde(default)]
    pub risk_free_rate: f64,
    #[serde(default)]
    pub dividend_yield: f64,
    /// μ = 0 instead of r − q (the common retail-screen convention).
    #[serde(default)]
    pub zero_drift: bool,
    /// Profit zone bounds; at least one must be present.
    pub lower_breakeven: Option<f64>,
    pub upper_breakeven: Option<f64>,
    /// true = profits between the bounds (credit spreads, condors);
    /// false = profits outside them (long straddles/strangles).
    pub profit_between: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PopReport {
    pub probability_of_profit: f64,
    /// P(S_T < lower) and P(S_T > upper) for the supplied bounds.
    pub prob_below_lower: Option<f64>,
    pub prob_above_upper: Option<f64>,
    /// Expected underlying at expiry under the chosen drift.
    pub expected_spot: f64,
}

fn norm_cdf(x: f64) -> f64 {
    // A&S 26.2.17, max err 7.5e-8 — same approximation the option
    // pricers in this crate use.
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

pub fn compute(inp: &PopInput) -> Option<PopReport> {
    if !inp.spot.is_finite()
        || inp.spot <= 0.0
        || !inp.iv.is_finite()
        || inp.iv <= 0.0
        || !inp.time_to_expiry_years.is_finite()
        || inp.time_to_expiry_years <= 0.0
        || !inp.risk_free_rate.is_finite()
        || !inp.dividend_yield.is_finite()
    {
        return None;
    }
    let (lo, hi) = (inp.lower_breakeven, inp.upper_breakeven);
    if lo.is_none() && hi.is_none() {
        return None;
    }
    if let Some(l) = lo {
        if !l.is_finite() || l <= 0.0 {
            return None;
        }
    }
    if let Some(h) = hi {
        if !h.is_finite() || h <= 0.0 {
            return None;
        }
    }
    if let (Some(l), Some(h)) = (lo, hi) {
        if h <= l {
            return None;
        }
    }
    let t = inp.time_to_expiry_years;
    let mu = if inp.zero_drift {
        0.0
    } else {
        inp.risk_free_rate - inp.dividend_yield
    };
    let sig_t = inp.iv * t.sqrt();
    let mean_ln = inp.spot.ln() + (mu - inp.iv * inp.iv / 2.0) * t;
    // P(S_T < x) = N((ln x − mean) / σ√T)
    let p_below = |x: f64| norm_cdf((x.ln() - mean_ln) / sig_t);
    let prob_below_lower = lo.map(p_below);
    let prob_above_upper = hi.map(|h| 1.0 - p_below(h));
    let inside = 1.0
        - prob_below_lower.unwrap_or(0.0)
        - prob_above_upper.unwrap_or(0.0);
    let pop = if inp.profit_between {
        inside
    } else {
        1.0 - inside
    };
    Some(PopReport {
        probability_of_profit: pop.clamp(0.0, 1.0),
        prob_below_lower,
        prob_above_upper,
        expected_spot: inp.spot * (mu * t).exp(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> PopInput {
        PopInput {
            spot: 100.0,
            iv: 0.25,
            time_to_expiry_years: 0.25,
            risk_free_rate: 0.0,
            dividend_yield: 0.0,
            zero_drift: true,
            lower_breakeven: None,
            upper_breakeven: None,
            profit_between: true,
        }
    }

    #[test]
    fn median_breakeven_splits_fifty_fifty() {
        // Lognormal median under zero drift = S·e^{−σ²T/2}. A single
        // upper breakeven at the median ⇒ exactly 50% below.
        let median = 100.0 * (-0.25_f64 * 0.25 * 0.25 / 2.0).exp();
        let r = compute(&PopInput {
            upper_breakeven: Some(median),
            ..base()
        })
        .unwrap();
        assert!((r.prob_above_upper.unwrap() - 0.5).abs() < 1e-6);
        assert!((r.probability_of_profit - 0.5).abs() < 1e-6);
    }

    #[test]
    fn between_and_outside_are_complements() {
        let between = compute(&PopInput {
            lower_breakeven: Some(90.0),
            upper_breakeven: Some(110.0),
            profit_between: true,
            ..base()
        })
        .unwrap();
        let outside = compute(&PopInput {
            lower_breakeven: Some(90.0),
            upper_breakeven: Some(110.0),
            profit_between: false,
            ..base()
        })
        .unwrap();
        assert!(
            (between.probability_of_profit + outside.probability_of_profit - 1.0).abs() < 1e-12
        );
        // A ±10% zone over one quarter at 25% vol captures the bulk.
        assert!(between.probability_of_profit > 0.5);
    }

    #[test]
    fn wider_zone_raises_between_pop() {
        let narrow = compute(&PopInput {
            lower_breakeven: Some(95.0),
            upper_breakeven: Some(105.0),
            ..base()
        })
        .unwrap();
        let wide = compute(&PopInput {
            lower_breakeven: Some(85.0),
            upper_breakeven: Some(115.0),
            ..base()
        })
        .unwrap();
        assert!(wide.probability_of_profit > narrow.probability_of_profit);
    }

    #[test]
    fn positive_drift_shifts_mass_upward() {
        let drift = compute(&PopInput {
            zero_drift: false,
            risk_free_rate: 0.10,
            upper_breakeven: Some(100.0),
            profit_between: false,
            ..base()
        })
        .unwrap();
        let flat = compute(&PopInput {
            upper_breakeven: Some(100.0),
            profit_between: false,
            ..base()
        })
        .unwrap();
        assert!(drift.prob_above_upper.unwrap() > flat.prob_above_upper.unwrap());
        assert!((drift.expected_spot - 100.0 * (0.10_f64 * 0.25).exp()).abs() < 1e-9);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&base()).is_none()); // no breakevens at all
        assert!(compute(&PopInput { spot: 0.0, upper_breakeven: Some(100.0), ..base() }).is_none());
        assert!(compute(&PopInput { iv: 0.0, upper_breakeven: Some(100.0), ..base() }).is_none());
        assert!(compute(&PopInput {
            lower_breakeven: Some(110.0),
            upper_breakeven: Some(90.0),
            ..base()
        })
        .is_none());
        assert!(compute(&PopInput {
            lower_breakeven: Some(-5.0),
            ..base()
        })
        .is_none());
    }
}
