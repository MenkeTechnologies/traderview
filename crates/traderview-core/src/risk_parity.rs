//! Risk-Parity allocator.
//!
//! Standard "risk parity" target: each asset contributes the SAME amount
//! of portfolio variance. Differs from equal-weighting (which gives high-
//! vol assets too much risk) and Markowitz mean-variance (which needs
//! return forecasts).
//!
//! Iterative naive approximation: weight_i ∝ 1/σ_i, then normalize to 1.
//! For correlated assets a full Riccati solver is needed — out of scope;
//! this simple variant assumes independence.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetVol {
    pub symbol: String,
    /// Annualized volatility (or any period-consistent measure).
    pub vol: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allocation {
    pub symbol: String,
    pub weight: f64,
    pub risk_contribution: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RiskParityReport {
    pub allocations: Vec<Allocation>,
    pub total_weight: f64,
}

pub fn allocate(assets: &[AssetVol]) -> RiskParityReport {
    let mut report = RiskParityReport::default();
    if assets.is_empty() {
        return report;
    }
    let inv_vol: Vec<f64> = assets
        .iter()
        .map(|a| if a.vol > 0.0 { 1.0 / a.vol } else { 0.0 })
        .collect();
    let total: f64 = inv_vol.iter().sum();
    if total <= 0.0 {
        return report;
    }
    for (i, asset) in assets.iter().enumerate() {
        let weight = inv_vol[i] / total;
        let risk_contribution = weight * asset.vol;
        report.allocations.push(Allocation {
            symbol: asset.symbol.clone(),
            weight,
            risk_contribution,
        });
    }
    report.total_weight = report.allocations.iter().map(|a| a.weight).sum();
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(sym: &str, v: f64) -> AssetVol {
        AssetVol {
            symbol: sym.into(),
            vol: v,
        }
    }

    #[test]
    fn empty_returns_empty() {
        let r = allocate(&[]);
        assert!(r.allocations.is_empty());
    }

    #[test]
    fn equal_vol_assets_equal_weighted() {
        let r = allocate(&[a("A", 0.20), a("B", 0.20), a("C", 0.20)]);
        for w in &r.allocations {
            assert!((w.weight - 1.0 / 3.0).abs() < 1e-9);
        }
    }

    #[test]
    fn high_vol_asset_gets_smaller_weight() {
        let r = allocate(&[a("LOW", 0.10), a("HIGH", 0.40)]);
        let low = r.allocations.iter().find(|a| a.symbol == "LOW").unwrap();
        let high = r.allocations.iter().find(|a| a.symbol == "HIGH").unwrap();
        assert!(low.weight > high.weight);
        // Ratio: inv_vol_low / inv_vol_high = (1/0.10) / (1/0.40) = 4.
        assert!((low.weight / high.weight - 4.0).abs() < 1e-9);
    }

    #[test]
    fn weights_sum_to_one() {
        let r = allocate(&[a("A", 0.10), a("B", 0.20), a("C", 0.30)]);
        assert!((r.total_weight - 1.0).abs() < 1e-9);
    }

    #[test]
    fn risk_contribution_equal_across_assets() {
        // Each asset's risk_contribution = weight × vol = (1/vol)/total × vol = 1/total.
        // → all equal.
        let r = allocate(&[a("A", 0.10), a("B", 0.20), a("C", 0.30)]);
        let first_rc = r.allocations[0].risk_contribution;
        for w in &r.allocations[1..] {
            assert!(
                (w.risk_contribution - first_rc).abs() < 1e-9,
                "risk parity → all assets equal risk contribution"
            );
        }
    }

    #[test]
    fn zero_vol_asset_gets_zero_weight() {
        // 1/0 protection.
        let r = allocate(&[a("ZERO", 0.0), a("NORMAL", 0.20)]);
        let zero = r.allocations.iter().find(|a| a.symbol == "ZERO").unwrap();
        let normal = r.allocations.iter().find(|a| a.symbol == "NORMAL").unwrap();
        assert_eq!(zero.weight, 0.0);
        assert_eq!(normal.weight, 1.0);
    }

    #[test]
    fn single_asset_full_weight() {
        let r = allocate(&[a("ONLY", 0.25)]);
        assert_eq!(r.allocations[0].weight, 1.0);
    }
}
