//! Earnings Revision Scanner — flags symbols where the analyst consensus
//! EPS estimate has shifted materially over a recent window.
//!
//! For each symbol, compute:
//!   pct_change = (current_estimate − prior_estimate) / |prior_estimate|
//!   up_count   = number of analysts who raised
//!   down_count = number of analysts who lowered
//!   net_revisions = up_count − down_count
//!
//! Filter rules (configurable):
//!   - |pct_change| ≥ min_pct_change
//!   - distinct_analysts ≥ min_analysts
//!
//! Direction:
//!   - "upgrade" if pct_change > 0 AND net_revisions > 0
//!   - "downgrade" if pct_change < 0 AND net_revisions < 0
//!   - "mixed" otherwise
//!
//! Sort output by abs(pct_change) descending so the biggest revisions
//! surface first.
//!
//! Pure compute. Caller supplies the analyst data (Refinitiv I/B/E/S or
//! similar). Companion to `insider_buying_scanner`, `momentum_12_1`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolRevisions {
    pub symbol: String,
    pub current_consensus_eps: f64,
    pub prior_consensus_eps: f64,
    pub analysts_raised: usize,
    pub analysts_lowered: usize,
    pub analysts_unchanged: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevisionDirection { Upgrade, Downgrade, Mixed }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionHit {
    pub symbol: String,
    pub current_consensus_eps: f64,
    pub prior_consensus_eps: f64,
    pub pct_change: f64,
    pub net_revisions: i64,
    pub distinct_analysts: usize,
    pub direction: RevisionDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub min_abs_pct_change: f64,
    pub min_distinct_analysts: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self { min_abs_pct_change: 0.02, min_distinct_analysts: 3 }
    }
}

pub fn scan(symbols: &[SymbolRevisions], cfg: &Config) -> Vec<RevisionHit> {
    let mut hits: Vec<RevisionHit> = symbols.iter().filter_map(|s| {
        if !s.current_consensus_eps.is_finite()
            || !s.prior_consensus_eps.is_finite()
            || s.prior_consensus_eps.abs() < 1e-12 { return None; }
        let distinct = s.analysts_raised + s.analysts_lowered + s.analysts_unchanged;
        if distinct < cfg.min_distinct_analysts { return None; }
        let pct = (s.current_consensus_eps - s.prior_consensus_eps) / s.prior_consensus_eps.abs();
        if pct.abs() < cfg.min_abs_pct_change { return None; }
        let net = s.analysts_raised as i64 - s.analysts_lowered as i64;
        let direction = if pct > 0.0 && net > 0 { RevisionDirection::Upgrade }
            else if pct < 0.0 && net < 0 { RevisionDirection::Downgrade }
            else { RevisionDirection::Mixed };
        Some(RevisionHit {
            symbol: s.symbol.clone(),
            current_consensus_eps: s.current_consensus_eps,
            prior_consensus_eps: s.prior_consensus_eps,
            pct_change: pct,
            net_revisions: net,
            distinct_analysts: distinct,
            direction,
        })
    }).collect();
    hits.sort_by(|a, b| b.pct_change.abs().partial_cmp(&a.pct_change.abs())
        .unwrap_or(std::cmp::Ordering::Equal));
    hits
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(sym: &str, cur: f64, prior: f64, raised: usize, lowered: usize, unchanged: usize)
        -> SymbolRevisions {
        SymbolRevisions {
            symbol: sym.into(),
            current_consensus_eps: cur,
            prior_consensus_eps: prior,
            analysts_raised: raised,
            analysts_lowered: lowered,
            analysts_unchanged: unchanged,
        }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(scan(&[], &Config::default()).is_empty());
    }

    #[test]
    fn too_few_analysts_filtered() {
        // 2 analysts < min 3.
        let symbols = vec![s("AAA", 2.10, 2.00, 1, 0, 1)];
        assert!(scan(&symbols, &Config::default()).is_empty());
    }

    #[test]
    fn small_pct_change_filtered() {
        // 0.5% change < min 2%.
        let symbols = vec![s("AAA", 2.01, 2.00, 3, 1, 1)];
        assert!(scan(&symbols, &Config::default()).is_empty());
    }

    #[test]
    fn zero_prior_returns_none() {
        let symbols = vec![s("AAA", 1.0, 0.0, 3, 1, 1)];
        assert!(scan(&symbols, &Config::default()).is_empty());
    }

    #[test]
    fn nan_consensus_filtered() {
        let symbols = vec![s("AAA", f64::NAN, 2.00, 3, 1, 1)];
        assert!(scan(&symbols, &Config::default()).is_empty());
    }

    #[test]
    fn clear_upgrade_classified() {
        let symbols = vec![s("AAA", 2.20, 2.00, 5, 0, 1)];
        let hits = scan(&symbols, &Config::default());
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].direction, RevisionDirection::Upgrade);
        assert!((hits[0].pct_change - 0.10).abs() < 1e-9);
    }

    #[test]
    fn clear_downgrade_classified() {
        let symbols = vec![s("AAA", 1.80, 2.00, 0, 5, 1)];
        let hits = scan(&symbols, &Config::default());
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].direction, RevisionDirection::Downgrade);
        assert!((hits[0].pct_change + 0.10).abs() < 1e-9);
    }

    #[test]
    fn mixed_when_pct_and_net_disagree() {
        // Consensus rose 5% but more analysts cut than raised
        // (some big upward revisions from a few analysts driving the mean).
        let symbols = vec![s("AAA", 2.10, 2.00, 1, 4, 1)];
        let hits = scan(&symbols, &Config::default());
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].direction, RevisionDirection::Mixed);
    }

    #[test]
    fn negative_prior_eps_handled() {
        // Prior -$0.50, current -$0.40 → 20% improvement (less loss).
        let symbols = vec![s("AAA", -0.40, -0.50, 3, 1, 1)];
        let hits = scan(&symbols, &Config::default());
        assert_eq!(hits.len(), 1);
        // pct = (cur - prior) / |prior| = 0.10 / 0.50 = +0.20.
        assert!((hits[0].pct_change - 0.20).abs() < 1e-9);
    }

    #[test]
    fn sorted_by_abs_pct_change_descending() {
        let symbols = vec![
            s("AAA", 2.05, 2.00, 3, 1, 1),    // 2.5%
            s("BBB", 2.40, 2.00, 5, 0, 1),    // 20%
            s("CCC", 1.90, 2.00, 0, 4, 1),    // -5%
        ];
        let hits = scan(&symbols, &Config::default());
        assert_eq!(hits.len(), 3);
        assert_eq!(hits[0].symbol, "BBB");
        assert_eq!(hits[1].symbol, "CCC");
        assert_eq!(hits[2].symbol, "AAA");
    }
}
