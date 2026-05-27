//! Portfolio-level Greeks aggregator.
//!
//! Sums delta / gamma / vega / theta / rho across an options portfolio
//! to surface aggregate exposures. Critical for sizing — a "delta-
//! neutral" trader can still be massively short vega or gamma without
//! noticing if they only look at individual positions.
//!
//! Multiplies per-share Greeks by contract size (default 100 for
//! equity options) and signed contract qty. Outputs absolute exposure
//! (dollar delta = delta × price × 100 × contracts) for risk sizing.
//!
//! Pure compute. Caller pre-computes per-contract Greeks via
//! crate::greeks and passes them in.

use crate::greeks::Greeks;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionPosition {
    pub symbol: String,
    pub greeks: Greeks,
    /// Signed contract count: positive = long, negative = short.
    pub contracts: i64,
    pub underlying_price: f64,
    pub multiplier: f64, // typically 100 for equity options
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PortfolioGreeks {
    pub total_delta: f64,
    pub total_gamma: f64,
    pub total_vega: f64,
    pub total_theta: f64,
    pub total_rho: f64,
    /// Sum of (delta × underlying × multiplier × contracts) — the
    /// equivalent dollar exposure to the underlyings.
    pub dollar_delta: f64,
    /// Position count for reference.
    pub position_count: usize,
}

pub fn aggregate(positions: &[OptionPosition]) -> PortfolioGreeks {
    let mut report = PortfolioGreeks::default();
    for p in positions {
        let qty = p.contracts as f64 * p.multiplier;
        report.total_delta += p.greeks.delta * qty;
        report.total_gamma += p.greeks.gamma * qty;
        report.total_vega += p.greeks.vega * qty;
        report.total_theta += p.greeks.theta * qty;
        report.total_rho += p.greeks.rho * qty;
        report.dollar_delta += p.greeks.delta * p.underlying_price * qty;
        report.position_count += 1;
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(
        sym: &str,
        delta: f64,
        gamma: f64,
        vega: f64,
        theta: f64,
        contracts: i64,
        underlying: f64,
    ) -> OptionPosition {
        OptionPosition {
            symbol: sym.into(),
            greeks: Greeks {
                price: 0.0,
                delta,
                gamma,
                vega,
                theta,
                rho: 0.0,
                d1: 0.0,
                d2: 0.0,
            },
            contracts,
            underlying_price: underlying,
            multiplier: 100.0,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = aggregate(&[]);
        assert_eq!(r.total_delta, 0.0);
        assert_eq!(r.position_count, 0);
    }

    #[test]
    fn single_long_call_total_delta_positive() {
        let positions = vec![pos("AAPL250620C200", 0.50, 0.02, 0.30, -0.05, 1, 200.0)];
        let r = aggregate(&positions);
        // delta × 100 × 1 = 50.
        assert_eq!(r.total_delta, 50.0);
        // dollar_delta = 0.50 × 200 × 100 × 1 = 10,000.
        assert_eq!(r.dollar_delta, 10_000.0);
    }

    #[test]
    fn short_position_negates_signs() {
        let long = vec![pos("X", 0.50, 0.02, 0.30, -0.05, 1, 200.0)];
        let short = vec![pos("X", 0.50, 0.02, 0.30, -0.05, -1, 200.0)];
        let rl = aggregate(&long);
        let rs = aggregate(&short);
        assert_eq!(rs.total_delta, -rl.total_delta);
        assert_eq!(rs.total_gamma, -rl.total_gamma);
        assert_eq!(rs.total_vega, -rl.total_vega);
        assert_eq!(rs.total_theta, -rl.total_theta);
        assert_eq!(rs.dollar_delta, -rl.dollar_delta);
    }

    #[test]
    fn delta_neutral_combo_aggregates_to_zero_delta_but_positive_gamma() {
        // Long 1 call (+0.5 delta) and short 1 call further OTM (-0.5 delta).
        let positions = vec![
            pos("CALL_ATM", 0.50, 0.02, 0.30, -0.05, 1, 200.0),
            pos("CALL_OTM", 0.50, 0.01, 0.20, -0.03, -1, 200.0),
        ];
        let r = aggregate(&positions);
        assert_eq!(r.total_delta, 0.0);
        // Gamma: 0.02 × 100 - 0.01 × 100 = 1.0. Net long gamma.
        assert!((r.total_gamma - 1.0).abs() < 1e-9);
    }

    #[test]
    fn theta_aggregates_as_signed_per_day_decay() {
        // Long options always have negative theta.
        let positions = vec![
            pos("A", 0.5, 0.02, 0.3, -0.05, 1, 100.0),
            pos("B", 0.5, 0.02, 0.3, -0.04, 1, 100.0),
        ];
        let r = aggregate(&positions);
        // (-0.05 + -0.04) × 100 = -9.
        assert!((r.total_theta + 9.0).abs() < 1e-9);
    }

    #[test]
    fn vega_scales_with_contracts() {
        let one_lot = aggregate(&[pos("X", 0.5, 0.02, 0.30, -0.05, 1, 100.0)]);
        let ten_lot = aggregate(&[pos("X", 0.5, 0.02, 0.30, -0.05, 10, 100.0)]);
        assert!((ten_lot.total_vega - one_lot.total_vega * 10.0).abs() < 1e-9);
    }

    #[test]
    fn position_count_tracked() {
        let positions: Vec<_> = (0..5)
            .map(|i| pos(&format!("S{i}"), 0.5, 0.02, 0.30, -0.05, 1, 100.0))
            .collect();
        let r = aggregate(&positions);
        assert_eq!(r.position_count, 5);
    }

    #[test]
    fn dollar_delta_uses_per_position_underlying() {
        // Different underlying prices per position.
        let positions = vec![
            pos("AAPL250620C200", 0.50, 0.02, 0.3, -0.05, 1, 200.0), // dd = 10000
            pos("TSLA250620C300", 0.40, 0.02, 0.3, -0.05, 1, 300.0), // dd = 12000
        ];
        let r = aggregate(&positions);
        assert_eq!(r.dollar_delta, 22_000.0);
    }
}
