//! Dealer Gamma Exposure (GEX) estimator.
//!
//! Net dealer gamma exposure across an option chain. Standard
//! convention: dealers are assumed SHORT calls (sold to retail) and
//! LONG puts (sold to retail as protection — but they're paid to be
//! short equity hedge). Net dealer gamma:
//!
//!   GEX = Σ_calls (gamma × OI × 100 × spot²) × (+1 for long, -1 for short)
//!       - Σ_puts  (gamma × OI × 100 × spot²)
//!
//! Following the canonical "SqueezeMetrics" methodology, dealer is
//! assumed LONG call gamma + SHORT put gamma (the inverse of customer
//! positioning). Sign convention here: positive GEX = dealer LONG gamma
//! → suppresses realized vol (mean-reverting market); negative GEX =
//! dealer SHORT gamma → amplifies realized vol (momentum / squeeze).
//!
//! Zero-gamma "flip" level is the strike where GEX changes sign — the
//! local boundary between vol-suppression and vol-amplification regimes.
//!
//! Pure compute. Caller supplies per-strike (call_gamma, call_oi,
//! put_gamma, put_oi) — typically pre-computed via crate::greeks.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StrikeGreeks {
    pub strike: f64,
    pub call_gamma: f64,
    pub call_oi: u64,
    pub put_gamma: f64,
    pub put_oi: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GexReport {
    pub spot: f64,
    /// Dealer-side total gamma exposure in dollars (gamma × OI × 100
    /// × spot² aggregated with sign convention above).
    pub total_gex: f64,
    /// Pure call-side dealer gamma (assumed LONG by convention).
    pub call_gex: f64,
    /// Pure put-side dealer gamma (assumed SHORT by convention).
    pub put_gex: f64,
    /// Sign convention helper: "positive" = vol-suppressive regime.
    pub regime: GexRegime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GexRegime {
    Positive,    // dealer long gamma — vol suppression
    Negative,    // dealer short gamma — vol amplification
    Neutral,     // ~ zero
}

impl Default for GexRegime {
    fn default() -> Self { GexRegime::Neutral }
}

pub fn compute(chain: &[StrikeGreeks], spot: f64) -> GexReport {
    let spot2 = spot * spot;
    let mut call_gex = 0.0;
    let mut put_gex = 0.0;
    for k in chain {
        call_gex += k.call_gamma * k.call_oi as f64 * 100.0 * spot2;
        put_gex  += k.put_gamma  * k.put_oi  as f64 * 100.0 * spot2;
    }
    let total = call_gex - put_gex;    // calls long, puts short (dealer side)
    let regime = if total.abs() < 1.0 { GexRegime::Neutral }
        else if total > 0.0 { GexRegime::Positive }
        else { GexRegime::Negative };
    GexReport {
        spot,
        total_gex: total,
        call_gex,
        put_gex,
        regime,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn k(strike: f64, cg: f64, coi: u64, pg: f64, poi: u64) -> StrikeGreeks {
        StrikeGreeks { strike, call_gamma: cg, call_oi: coi, put_gamma: pg, put_oi: poi }
    }

    #[test]
    fn empty_chain_returns_neutral() {
        let r = compute(&[], 100.0);
        assert_eq!(r.regime, GexRegime::Neutral);
        assert_eq!(r.total_gex, 0.0);
    }

    #[test]
    fn pure_call_dominance_positive_regime() {
        // Only call OI → dealer is long gamma → positive.
        let r = compute(&[k(100.0, 0.02, 10_000, 0.0, 0)], 100.0);
        assert!(r.total_gex > 0.0);
        assert_eq!(r.regime, GexRegime::Positive);
        // call_gex = 0.02 × 10000 × 100 × 100² = 200,000,000.
        assert_eq!(r.call_gex, 200_000_000.0);
        assert_eq!(r.put_gex, 0.0);
    }

    #[test]
    fn pure_put_dominance_negative_regime() {
        // Only put OI → dealer is short put gamma → negative.
        let r = compute(&[k(100.0, 0.0, 0, 0.02, 10_000)], 100.0);
        assert!(r.total_gex < 0.0);
        assert_eq!(r.regime, GexRegime::Negative);
    }

    #[test]
    fn balanced_chain_near_zero_neutral_regime() {
        // Equal call & put gamma + OI → net cancels.
        let r = compute(&[k(100.0, 0.02, 1000, 0.02, 1000)], 100.0);
        assert_eq!(r.total_gex, 0.0);
        assert_eq!(r.regime, GexRegime::Neutral);
    }

    #[test]
    fn gex_scales_with_spot_squared() {
        // Doubling spot → GEX should quadruple (spot²).
        let r100 = compute(&[k(100.0, 0.02, 10_000, 0.0, 0)], 100.0);
        let r200 = compute(&[k(100.0, 0.02, 10_000, 0.0, 0)], 200.0);
        assert!((r200.total_gex / r100.total_gex - 4.0).abs() < 1e-9);
    }

    #[test]
    fn gex_scales_linearly_with_oi() {
        let r1 = compute(&[k(100.0, 0.02, 1_000, 0.0, 0)], 100.0);
        let r10 = compute(&[k(100.0, 0.02, 10_000, 0.0, 0)], 100.0);
        assert!((r10.total_gex / r1.total_gex - 10.0).abs() < 1e-9);
    }

    #[test]
    fn sum_call_and_put_gex_equals_difference_in_total() {
        let chain = vec![
            k(100.0, 0.02, 5_000, 0.01, 3_000),
            k(105.0, 0.03, 4_000, 0.02, 2_000),
        ];
        let r = compute(&chain, 100.0);
        assert!((r.total_gex - (r.call_gex - r.put_gex)).abs() < 1e-6);
    }

    #[test]
    fn multi_strike_aggregates_correctly() {
        let chain = vec![
            k(95.0,  0.01, 1_000, 0.01, 1_000),
            k(100.0, 0.02, 2_000, 0.02, 2_000),
            k(105.0, 0.01, 1_000, 0.01, 1_000),
        ];
        let r = compute(&chain, 100.0);
        // Symmetric → net 0.
        assert_eq!(r.total_gex, 0.0);
    }
}
