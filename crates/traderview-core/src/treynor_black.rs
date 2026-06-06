//! Treynor–Black (1973) Active Portfolio Model.
//!
//! Given:
//!   - A set of active securities each with forecast alpha α_i and
//!     residual (idiosyncratic) variance σ²_ε,i
//!   - The market portfolio's excess return E[R_M − Rf] and variance σ²_M
//!
//! Optimal weights within the *active* sub-portfolio:
//!
//!   w_i^0 = (α_i / σ²_ε,i) / Σ_j (α_j / σ²_ε,j)
//!
//! Active portfolio aggregate alpha + residual variance:
//!
//!   α_A = Σ w_i α_i
//!   β_A = Σ w_i β_i
//!   σ²_ε,A = Σ w_i² σ²_ε,i
//!
//! Allocation between active and market portfolios:
//!
//!   w_A^0 = (α_A / σ²_ε,A) / (E[R_M − Rf] / σ²_M)
//!
//! adjusted for the active portfolio's beta:
//!
//!   w_A^* = w_A^0 / (1 + (1 − β_A) · w_A^0)
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSecurity {
    pub symbol: String,
    pub alpha: f64,
    pub beta: f64,
    pub residual_variance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAllocation {
    pub symbol: String,
    /// w_i within the active sub-portfolio.
    pub weight_active_subportfolio: f64,
    /// Final allocation = w_A^* · w_i_active.
    pub weight_overall: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TreynorBlackReport {
    pub active_portfolio_alpha: f64,
    pub active_portfolio_beta: f64,
    pub active_portfolio_residual_variance: f64,
    pub active_portfolio_weight_naive: f64,
    pub active_portfolio_weight_adjusted: f64,
    pub market_portfolio_weight: f64,
    pub allocations: Vec<SecurityAllocation>,
    pub expected_active_information_ratio: f64,
}

pub fn solve(
    securities: &[ActiveSecurity],
    market_excess_return: f64,
    market_variance: f64,
) -> Option<TreynorBlackReport> {
    if securities.is_empty()
        || !market_excess_return.is_finite()
        || !market_variance.is_finite()
        || market_variance <= 0.0
    {
        return None;
    }
    if securities.iter().any(|s| {
        !s.alpha.is_finite()
            || !s.beta.is_finite()
            || !s.residual_variance.is_finite()
            || s.residual_variance <= 0.0
    }) {
        return None;
    }
    let score_sum: f64 = securities
        .iter()
        .map(|s| s.alpha / s.residual_variance)
        .sum();
    if !score_sum.is_finite() || score_sum.abs() < 1e-18 {
        return None;
    }
    let mut active_weights = Vec::with_capacity(securities.len());
    for s in securities {
        active_weights.push((s.alpha / s.residual_variance) / score_sum);
    }
    let alpha_a: f64 = securities
        .iter()
        .zip(active_weights.iter())
        .map(|(s, w)| s.alpha * w)
        .sum();
    let beta_a: f64 = securities
        .iter()
        .zip(active_weights.iter())
        .map(|(s, w)| s.beta * w)
        .sum();
    let resid_var_a: f64 = securities
        .iter()
        .zip(active_weights.iter())
        .map(|(s, w)| w * w * s.residual_variance)
        .sum();
    if resid_var_a <= 0.0 {
        return None;
    }
    // Naive active weight: (α_A / σ²_ε,A) / (E[R_M − Rf] / σ²_M)
    let market_score = market_excess_return / market_variance;
    if market_score.abs() < 1e-18 {
        return None;
    }
    let w_a_naive = (alpha_a / resid_var_a) / market_score;
    // Beta adjustment: w_A^* = w_A^0 / (1 + (1 − β_A) · w_A^0)
    let adjust = 1.0 + (1.0 - beta_a) * w_a_naive;
    if adjust.abs() < 1e-18 {
        return None;
    }
    let w_a_adj = w_a_naive / adjust;
    let w_market = 1.0 - w_a_adj;
    // Information ratio = α_A / σ_ε,A (Sharpe ratio for active alpha).
    let info_ratio = alpha_a / resid_var_a.sqrt();
    let allocations = securities
        .iter()
        .zip(active_weights.iter())
        .map(|(s, w)| SecurityAllocation {
            symbol: s.symbol.clone(),
            weight_active_subportfolio: *w,
            weight_overall: w_a_adj * w,
        })
        .collect();
    Some(TreynorBlackReport {
        active_portfolio_alpha: alpha_a,
        active_portfolio_beta: beta_a,
        active_portfolio_residual_variance: resid_var_a,
        active_portfolio_weight_naive: w_a_naive,
        active_portfolio_weight_adjusted: w_a_adj,
        market_portfolio_weight: w_market,
        allocations,
        expected_active_information_ratio: info_ratio,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(sym: &str, alpha: f64, beta: f64, sv: f64) -> ActiveSecurity {
        ActiveSecurity {
            symbol: sym.into(),
            alpha,
            beta,
            residual_variance: sv,
        }
    }

    #[test]
    fn empty_or_invalid_returns_none() {
        assert!(solve(&[], 0.06, 0.04).is_none());
        let sec = vec![s("X", 0.02, 1.0, 0.01)];
        assert!(solve(&sec, 0.06, 0.0).is_none());
        assert!(solve(&sec, f64::NAN, 0.04).is_none());
        let bad_var = vec![s("X", 0.02, 1.0, -0.01)];
        assert!(solve(&bad_var, 0.06, 0.04).is_none());
    }

    #[test]
    fn zero_alpha_pool_returns_none() {
        let sec = vec![s("A", 0.0, 1.0, 0.01), s("B", 0.0, 0.8, 0.02)];
        assert!(solve(&sec, 0.06, 0.04).is_none());
    }

    #[test]
    fn higher_alpha_to_variance_ratio_dominates() {
        // A: α=0.03 / σ²=0.01 → ratio 3.0
        // B: α=0.01 / σ²=0.02 → ratio 0.5
        // A should dominate the active sub-portfolio.
        let sec = vec![s("A", 0.03, 1.0, 0.01), s("B", 0.01, 1.0, 0.02)];
        let r = solve(&sec, 0.06, 0.04).unwrap();
        let a_w = r
            .allocations
            .iter()
            .find(|x| x.symbol == "A")
            .unwrap()
            .weight_active_subportfolio;
        let b_w = r
            .allocations
            .iter()
            .find(|x| x.symbol == "B")
            .unwrap()
            .weight_active_subportfolio;
        assert!(a_w > b_w);
        assert!((a_w + b_w - 1.0).abs() < 1e-12);
    }

    #[test]
    fn negative_alpha_yields_negative_weight() {
        // Short the security with negative alpha.
        let sec = vec![s("LONG", 0.02, 1.0, 0.01), s("SHORT", -0.01, 1.0, 0.01)];
        let r = solve(&sec, 0.06, 0.04).unwrap();
        let short_w = r
            .allocations
            .iter()
            .find(|x| x.symbol == "SHORT")
            .unwrap()
            .weight_active_subportfolio;
        assert!(
            short_w < 0.0,
            "negative alpha should produce short weight, got {short_w}"
        );
    }

    #[test]
    fn positive_alpha_pool_yields_positive_active_weight() {
        let sec = vec![s("A", 0.02, 1.0, 0.01), s("B", 0.015, 0.9, 0.012)];
        let r = solve(&sec, 0.06, 0.04).unwrap();
        assert!(r.active_portfolio_weight_adjusted > 0.0);
        assert!(r.active_portfolio_alpha > 0.0);
    }

    #[test]
    fn overall_weights_sum_to_active_allocation() {
        let sec = vec![s("A", 0.02, 1.0, 0.01), s("B", 0.015, 0.9, 0.012)];
        let r = solve(&sec, 0.06, 0.04).unwrap();
        let total_active: f64 = r.allocations.iter().map(|a| a.weight_overall).sum();
        assert!((total_active - r.active_portfolio_weight_adjusted).abs() < 1e-9);
        assert!((total_active + r.market_portfolio_weight - 1.0).abs() < 1e-9);
    }

    #[test]
    fn beta_adjustment_for_low_beta_active_reduces_weight() {
        // Treynor-Black: w_A* = w_A^0 / (1 + (1 − β_A) · w_A^0).
        // For β_A < 1 with positive w_A^0, denominator > 1, so adjusted <
        // naive (less of a defensive active portfolio is needed because the
        // market portfolio already provides the missing market exposure).
        let sec = vec![s("DEFENSIVE", 0.02, 0.5, 0.01)];
        let r = solve(&sec, 0.06, 0.04).unwrap();
        assert!(r.active_portfolio_weight_naive > 0.0);
        assert!(r.active_portfolio_weight_adjusted < r.active_portfolio_weight_naive);
    }

    #[test]
    fn beta_adjustment_for_high_beta_active_increases_weight() {
        // Symmetric: β_A > 1 → denominator < 1 (positive w_A^0) → adjusted >
        // naive. The high-beta active portfolio "self-supplies" market
        // exposure so more of it can be held.
        let sec = vec![s("AGGRESSIVE", 0.02, 1.5, 0.01)];
        let r = solve(&sec, 0.06, 0.04).unwrap();
        assert!(r.active_portfolio_weight_naive > 0.0);
        assert!(r.active_portfolio_weight_adjusted > r.active_portfolio_weight_naive);
    }
}
