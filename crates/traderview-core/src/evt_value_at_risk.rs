//! Extreme Value Theory (EVT) Value-at-Risk and Expected Shortfall.
//!
//! Given a fitted GPD tail (ξ, β) at threshold u with empirical
//! exceedance probability p_u = n_exceedances / n_total, the
//! EVT VaR / ES at confidence level α (e.g. α = 0.99) are:
//!
//!   VaR_α = u + (β / ξ) · ((n / n_exceedances · (1 − α))^(−ξ) − 1)
//!   ES_α  = VaR_α / (1 − ξ) + (β − ξ · u) / (1 − ξ)        ξ < 1
//!
//! When ξ → 0:
//!   VaR_α → u + β · ln(n / (n_exceedances · (1 − α)))
//!   ES_α  → VaR_α + β
//!
//! Used when α exceeds the empirical maximum tail probability (e.g.
//! estimating 1-in-1000 day losses with only 250 days of data).
//!
//! Pure compute. Companion to `gpd_tail_fit`, `peaks_over_threshold`,
//! `conditional_var`, `monte_carlo_var`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EvtVarReport {
    pub confidence: f64,
    pub var: f64,
    pub expected_shortfall: f64,
    pub threshold_used: f64,
    pub exceedance_probability: f64,
    pub shape_xi: f64,
    pub scale_beta: f64,
}

pub fn compute(
    threshold: f64,
    n_exceedances: usize,
    n_total: usize,
    shape_xi: f64,
    scale_beta: f64,
    confidence: f64,
) -> Option<EvtVarReport> {
    if !threshold.is_finite()
        || n_exceedances == 0
        || n_total == 0
        || n_exceedances >= n_total
        || !shape_xi.is_finite()
        || !scale_beta.is_finite()
        || scale_beta <= 0.0
        || !confidence.is_finite()
        || !(0.5..1.0).contains(&confidence)
    {
        return None;
    }
    let n_f = n_total as f64;
    let n_u = n_exceedances as f64;
    let p_u = n_u / n_f;
    let alpha_excess = n_f / n_u * (1.0 - confidence);
    if alpha_excess <= 0.0 || alpha_excess > 1.0 {
        return None;
    }
    let (var, es) = if shape_xi.abs() < 1e-9 {
        let v = threshold + scale_beta * (1.0 / alpha_excess).ln();
        let e = v + scale_beta;
        (v, e)
    } else {
        let v = threshold + (scale_beta / shape_xi) * (alpha_excess.powf(-shape_xi) - 1.0);
        if shape_xi >= 1.0 {
            // ES not defined; just return VaR.
            (v, f64::INFINITY)
        } else {
            let e = v / (1.0 - shape_xi) + (scale_beta - shape_xi * threshold) / (1.0 - shape_xi);
            (v, e)
        }
    };
    Some(EvtVarReport {
        confidence,
        var,
        expected_shortfall: es,
        threshold_used: threshold,
        exceedance_probability: p_u,
        shape_xi,
        scale_beta,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(compute(1.0, 0, 100, 0.2, 1.0, 0.99).is_none());
        assert!(compute(1.0, 50, 50, 0.2, 1.0, 0.99).is_none());
        assert!(compute(1.0, 10, 100, 0.2, 0.0, 0.99).is_none());
        assert!(compute(1.0, 10, 100, 0.2, 1.0, 1.0).is_none());
        assert!(compute(1.0, 10, 100, f64::NAN, 1.0, 0.99).is_none());
    }

    #[test]
    fn confidence_below_empirical_returns_none() {
        // p_u = 0.10. Asking for VaR at 80% means alpha_excess > 1 → None.
        assert!(compute(1.0, 10, 100, 0.2, 1.0, 0.80).is_none());
    }

    #[test]
    fn var_above_threshold() {
        let r = compute(1.0, 10, 100, 0.2, 1.0, 0.99).unwrap();
        assert!(r.var > r.threshold_used);
    }

    #[test]
    fn es_at_least_as_large_as_var() {
        let r = compute(1.0, 10, 100, 0.2, 1.0, 0.99).unwrap();
        assert!(r.expected_shortfall >= r.var);
    }

    #[test]
    fn xi_zero_uses_exponential_form() {
        let r = compute(1.0, 10, 100, 0.0, 1.0, 0.99).unwrap();
        // ES = VaR + β.
        assert!((r.expected_shortfall - r.var - 1.0).abs() < 1e-9);
    }

    #[test]
    fn higher_confidence_yields_higher_var() {
        let r99 = compute(1.0, 10, 100, 0.2, 1.0, 0.99).unwrap();
        let r995 = compute(1.0, 10, 100, 0.2, 1.0, 0.995).unwrap();
        assert!(r995.var > r99.var);
    }

    #[test]
    fn higher_shape_inflates_var_more() {
        let light = compute(1.0, 10, 100, 0.0, 1.0, 0.99).unwrap();
        let heavy = compute(1.0, 10, 100, 0.5, 1.0, 0.99).unwrap();
        assert!(heavy.var > light.var);
    }

    #[test]
    fn xi_above_one_yields_infinite_es() {
        let r = compute(1.0, 10, 100, 1.5, 1.0, 0.99).unwrap();
        assert!(r.expected_shortfall.is_infinite());
    }
}
