//! Active Share — Cremers & Petajisto (2009).
//!
//! Quantifies how different a portfolio's holdings are from its
//! benchmark index:
//!
//!   AS = ½ · Σ_i | w_portfolio_i − w_benchmark_i |
//!
//! Range [0, 1]:
//!   - 0   = portfolio identical to benchmark (closet indexer)
//!   - 1   = no overlap with benchmark holdings
//!   - 0.6+ typically counts as "active" management
//!
//! Pure compute. Caller supplies a single common universe with weight
//! 0 for names absent from either portfolio.
//!
//! Companion to `tracking_error`, `factor_models`, `up_down_capture`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightPair {
    pub symbol: String,
    pub portfolio_weight: f64,
    pub benchmark_weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActiveShareReport {
    pub active_share: f64,
    pub portfolio_weight_sum: f64,
    pub benchmark_weight_sum: f64,
    pub n_names: usize,
    pub n_overweights: usize,
    pub n_underweights: usize,
}

pub fn compute(weights: &[WeightPair]) -> Option<ActiveShareReport> {
    if weights.is_empty() { return None; }
    if weights.iter().any(|w| !w.portfolio_weight.is_finite()
        || !w.benchmark_weight.is_finite()
        || w.portfolio_weight < 0.0 || w.benchmark_weight < 0.0) {
        return None;
    }
    let p_sum: f64 = weights.iter().map(|w| w.portfolio_weight).sum();
    let b_sum: f64 = weights.iter().map(|w| w.benchmark_weight).sum();
    let abs_diff_sum: f64 = weights.iter().map(|w| {
        (w.portfolio_weight - w.benchmark_weight).abs()
    }).sum();
    let active = 0.5 * abs_diff_sum;
    let mut n_over = 0;
    let mut n_under = 0;
    for w in weights {
        let diff = w.portfolio_weight - w.benchmark_weight;
        if diff > 1e-12 { n_over += 1; }
        else if diff < -1e-12 { n_under += 1; }
    }
    Some(ActiveShareReport {
        active_share: active.clamp(0.0, 1.0),
        portfolio_weight_sum: p_sum,
        benchmark_weight_sum: b_sum,
        n_names: weights.len(),
        n_overweights: n_over,
        n_underweights: n_under,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(sym: &str, p: f64, b: f64) -> WeightPair {
        WeightPair { symbol: sym.into(), portfolio_weight: p, benchmark_weight: b }
    }

    #[test]
    fn empty_returns_none() { assert!(compute(&[]).is_none()); }

    #[test]
    fn nan_or_negative_returns_none() {
        assert!(compute(&[w("X", f64::NAN, 0.5)]).is_none());
        assert!(compute(&[w("X", -0.1, 0.5)]).is_none());
        assert!(compute(&[w("X", 0.5, -0.1)]).is_none());
    }

    #[test]
    fn identical_portfolio_yields_zero_active_share() {
        let weights = vec![
            w("A", 0.4, 0.4),
            w("B", 0.3, 0.3),
            w("C", 0.3, 0.3),
        ];
        let r = compute(&weights).unwrap();
        assert!(r.active_share.abs() < 1e-12);
        assert_eq!(r.n_overweights, 0);
        assert_eq!(r.n_underweights, 0);
    }

    #[test]
    fn disjoint_portfolios_yield_active_share_one() {
        // Portfolio in A only, benchmark in B only.
        let weights = vec![
            w("A", 1.0, 0.0),
            w("B", 0.0, 1.0),
        ];
        let r = compute(&weights).unwrap();
        assert!((r.active_share - 1.0).abs() < 1e-12);
    }

    #[test]
    fn cremers_petajisto_canonical_example() {
        // 50% overlap → active_share = 0.50.
        let weights = vec![
            w("A", 0.5, 0.5),
            w("B", 0.5, 0.0),
            w("C", 0.0, 0.5),
        ];
        let r = compute(&weights).unwrap();
        assert!((r.active_share - 0.50).abs() < 1e-12);
    }

    #[test]
    fn over_and_under_weight_counts_correct() {
        let weights = vec![
            w("A", 0.5, 0.3),    // over
            w("B", 0.2, 0.3),    // under
            w("C", 0.3, 0.3),    // equal
            w("D", 0.0, 0.1),    // under
        ];
        let r = compute(&weights).unwrap();
        assert_eq!(r.n_overweights, 1);
        assert_eq!(r.n_underweights, 2);
    }

    #[test]
    fn weight_sums_reported() {
        let weights = vec![
            w("A", 0.4, 0.5),
            w("B", 0.6, 0.5),
        ];
        let r = compute(&weights).unwrap();
        assert!((r.portfolio_weight_sum - 1.0).abs() < 1e-12);
        assert!((r.benchmark_weight_sum - 1.0).abs() < 1e-12);
    }

    #[test]
    fn active_share_in_unit_range() {
        let weights = vec![
            w("A", 0.7, 0.3),
            w("B", 0.3, 0.7),
        ];
        let r = compute(&weights).unwrap();
        assert!((0.0..=1.0).contains(&r.active_share));
    }
}
