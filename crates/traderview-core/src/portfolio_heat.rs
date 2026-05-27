//! Portfolio heat tracker — correlated-position size budget.
//!
//! Naive position sizing treats every trade independently: 1% risk per trade,
//! 5 simultaneous trades, max loss = 5%. But if those 5 positions are all
//! tech longs, they're really a single 5% concentrated tech bet — they move
//! together, drawdown together, and lock you out of new exposure together.
//!
//! Portfolio heat fixes this by treating CORRELATED positions as a
//! single bundle. For each new candidate trade, compute the bundle this
//! trade would join (positions whose correlation to it exceeds the threshold),
//! sum the bundle's total dollar risk, and compare to the heat budget.
//!
//! Pure compute. Caller supplies the correlation matrix and the bundle
//! threshold. Returns an admit/reject decision + the heat figure.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRiskPosition {
    pub symbol: String,
    /// Dollar risk this position is taking (entry minus stop × qty × multiplier).
    pub dollar_risk: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateTrade {
    pub symbol: String,
    pub dollar_risk: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatConfig {
    /// Pairs of symbols above this absolute correlation are "bundled" together.
    pub bundle_threshold: f64,
    /// Maximum dollar risk allowed in any single bundle.
    pub bundle_budget: f64,
    /// Maximum total dollar risk across the whole portfolio.
    pub total_budget: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HeatDecision {
    #[default]
    Admit,
    /// Reject because the new trade would push the symbol's bundle past `bundle_budget`.
    BundleOverBudget,
    /// Reject because total portfolio risk would exceed `total_budget`.
    TotalOverBudget,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeatReport {
    pub decision: HeatDecision,
    /// Symbols already in the bundle the candidate would join (correlation
    /// above the threshold), including the candidate itself if admitted.
    pub bundle_members: Vec<String>,
    pub bundle_existing_heat: f64,
    pub bundle_projected_heat: f64,
    pub portfolio_existing_heat: f64,
    pub portfolio_projected_heat: f64,
    pub note: String,
}

/// Correlations come as a flat list of edges. Order of `a`/`b` is
/// irrelevant; the engine looks up both directions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrEdge {
    pub a: String,
    pub b: String,
    pub corr: f64,
}

pub fn evaluate(
    open_positions: &[OpenRiskPosition],
    correlations: &[CorrEdge],
    candidate: &CandidateTrade,
    cfg: &HeatConfig,
) -> HeatReport {
    // Build symmetric correlation map.
    let mut corr_map: BTreeMap<(String, String), f64> = BTreeMap::new();
    for e in correlations {
        corr_map.insert((e.a.clone(), e.b.clone()), e.corr);
        corr_map.insert((e.b.clone(), e.a.clone()), e.corr);
    }
    let portfolio_existing: f64 = open_positions.iter().map(|p| p.dollar_risk).sum();
    let portfolio_projected = portfolio_existing + candidate.dollar_risk;

    let mut bundle_members = vec![candidate.symbol.clone()];
    let mut bundle_existing = 0.0_f64;
    for p in open_positions {
        let key = (candidate.symbol.clone(), p.symbol.clone());
        // Self-correlation = 1.0; candidate doesn't need to be in open positions.
        let c = if p.symbol == candidate.symbol {
            1.0
        } else {
            corr_map.get(&key).copied().unwrap_or(0.0)
        };
        if c.abs() >= cfg.bundle_threshold {
            bundle_members.push(p.symbol.clone());
            bundle_existing += p.dollar_risk;
        }
    }
    let bundle_projected = bundle_existing + candidate.dollar_risk;

    let (decision, note) = if portfolio_projected > cfg.total_budget {
        (HeatDecision::TotalOverBudget,
         format!("portfolio projected ${} exceeds total budget ${}",
                 portfolio_projected as i64, cfg.total_budget as i64))
    } else if bundle_projected > cfg.bundle_budget {
        (HeatDecision::BundleOverBudget,
         format!("bundle of {} symbols projected ${} exceeds bundle budget ${}",
                 bundle_members.len(), bundle_projected as i64, cfg.bundle_budget as i64))
    } else {
        (HeatDecision::Admit, format!("admitted into {} symbol bundle", bundle_members.len()))
    };

    HeatReport {
        decision, bundle_members,
        bundle_existing_heat: bundle_existing,
        bundle_projected_heat: bundle_projected,
        portfolio_existing_heat: portfolio_existing,
        portfolio_projected_heat: portfolio_projected,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(sym: &str, risk: f64) -> OpenRiskPosition {
        OpenRiskPosition { symbol: sym.into(), dollar_risk: risk }
    }

    fn cand(sym: &str, risk: f64) -> CandidateTrade {
        CandidateTrade { symbol: sym.into(), dollar_risk: risk }
    }

    fn edge(a: &str, b: &str, c: f64) -> CorrEdge {
        CorrEdge { a: a.into(), b: b.into(), corr: c }
    }

    #[test]
    fn empty_portfolio_admits_when_under_budgets() {
        let cfg = HeatConfig { bundle_threshold: 0.7, bundle_budget: 500.0, total_budget: 2000.0 };
        let r = evaluate(&[], &[], &cand("AAPL", 100.0), &cfg);
        assert!(matches!(r.decision, HeatDecision::Admit));
        assert_eq!(r.bundle_members, vec!["AAPL".to_string()]);
        assert_eq!(r.portfolio_projected_heat, 100.0);
    }

    #[test]
    fn correlated_position_joins_bundle_and_blocks_when_over_budget() {
        // AAPL + MSFT correlated 0.9. Bundle budget $300. Adding another
        // correlated trade puts the bundle at $500 — over budget.
        let cfg = HeatConfig { bundle_threshold: 0.7, bundle_budget: 300.0, total_budget: 5000.0 };
        let open = vec![pos("AAPL", 200.0), pos("MSFT", 100.0)];
        let corrs = vec![edge("AAPL", "MSFT", 0.9), edge("AAPL", "NVDA", 0.85), edge("MSFT", "NVDA", 0.88)];
        let r = evaluate(&open, &corrs, &cand("NVDA", 200.0), &cfg);
        assert!(matches!(r.decision, HeatDecision::BundleOverBudget));
        // Bundle = NVDA + AAPL + MSFT — 3 symbols.
        assert_eq!(r.bundle_members.len(), 3);
        assert!((r.bundle_projected_heat - 500.0).abs() < 1e-9);
    }

    #[test]
    fn uncorrelated_position_doesnt_join_bundle() {
        // Gold (GLD) uncorrelated with tech — admits even if portfolio is hot.
        let cfg = HeatConfig { bundle_threshold: 0.7, bundle_budget: 300.0, total_budget: 5000.0 };
        let open = vec![pos("AAPL", 250.0), pos("MSFT", 250.0)];
        let corrs = vec![
            edge("AAPL", "MSFT", 0.9),
            edge("GLD",  "AAPL", 0.1),
            edge("GLD",  "MSFT", 0.1),
        ];
        let r = evaluate(&open, &corrs, &cand("GLD", 200.0), &cfg);
        assert!(matches!(r.decision, HeatDecision::Admit),
            "uncorrelated GLD should be admitted, got {:?}", r.decision);
        assert_eq!(r.bundle_members, vec!["GLD".to_string()]);
    }

    #[test]
    fn total_budget_blocks_even_uncorrelated_when_portfolio_is_already_at_cap() {
        // Total budget is the hard cap.
        let cfg = HeatConfig { bundle_threshold: 0.7, bundle_budget: 10_000.0, total_budget: 500.0 };
        let open = vec![pos("AAPL", 300.0), pos("GLD", 200.0)];
        let corrs = vec![edge("AAPL", "GLD", 0.1), edge("TLT", "AAPL", 0.0), edge("TLT", "GLD", 0.0)];
        let r = evaluate(&open, &corrs, &cand("TLT", 50.0), &cfg);
        assert!(matches!(r.decision, HeatDecision::TotalOverBudget),
            "portfolio cap should block, got {:?}", r.decision);
    }

    #[test]
    fn missing_correlation_defaults_to_zero() {
        // No corr edge between candidate and existing positions.
        let cfg = HeatConfig { bundle_threshold: 0.7, bundle_budget: 100.0, total_budget: 10_000.0 };
        let open = vec![pos("AAPL", 500.0)];
        let r = evaluate(&open, &[], &cand("XYZ", 50.0), &cfg);
        // No correlation → not bundled → admitted.
        assert!(matches!(r.decision, HeatDecision::Admit));
        assert_eq!(r.bundle_members, vec!["XYZ".to_string()]);
    }

    #[test]
    fn negative_correlation_above_threshold_still_bundles() {
        // -0.85 is a strong INVERSE relationship — those positions still move
        // together (in opposite directions), and that's still a concentration
        // risk if you're long one and short the other.
        let cfg = HeatConfig { bundle_threshold: 0.7, bundle_budget: 200.0, total_budget: 10_000.0 };
        let open = vec![pos("SPY", 150.0)];
        let corrs = vec![edge("SPY", "VIX", -0.85)];
        let r = evaluate(&open, &corrs, &cand("VIX", 100.0), &cfg);
        assert!(matches!(r.decision, HeatDecision::BundleOverBudget));
        assert_eq!(r.bundle_members.len(), 2);
    }
}
