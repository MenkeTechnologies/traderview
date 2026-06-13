//! Two-asset portfolio risk and return — the Markowitz closed form, showing how
//! correlation below 1 lowers portfolio volatility below the weighted average
//! (the diversification benefit).
//!
//! ```text
//! return = wa·ra + wb·rb
//! var    = wa²σa² + wb²σb² + 2·wa·wb·σa·σb·ρ
//! vol    = √var
//! diversification benefit = (wa·σa + wb·σb) − vol
//! ```
//!
//! At ρ = 1 the portfolio vol equals the weighted average (no benefit); below
//! that it falls, reaching |wa·σa − wb·σb| at ρ = −1.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct TwoAssetInput {
    /// Weight of asset A, percent (B takes the remainder).
    pub weight_a_pct: f64,
    pub return_a_pct: f64,
    pub return_b_pct: f64,
    pub volatility_a_pct: f64,
    pub volatility_b_pct: f64,
    /// Correlation between the assets, −1..1.
    pub correlation: f64,
    /// Risk-free rate for the Sharpe ratio, percent.
    #[serde(default)]
    pub risk_free_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TwoAssetResult {
    pub weight_a_pct: f64,
    pub weight_b_pct: f64,
    /// Expected portfolio return, percent.
    pub portfolio_return_pct: f64,
    /// Portfolio volatility (standard deviation), percent.
    pub portfolio_volatility_pct: f64,
    /// Weight-averaged volatility — the no-diversification benchmark.
    pub weighted_avg_volatility_pct: f64,
    /// weighted_avg − portfolio vol; the volatility shaved off by diversifying.
    pub diversification_benefit_pct: f64,
    /// (return − risk-free) / volatility; `None` if volatility is 0.
    pub sharpe_ratio: Option<f64>,
}

pub fn analyze(input: &TwoAssetInput) -> TwoAssetResult {
    let wa = input.weight_a_pct / 100.0;
    let wb = 1.0 - wa;
    let sa = input.volatility_a_pct / 100.0;
    let sb = input.volatility_b_pct / 100.0;
    let rho = input.correlation.clamp(-1.0, 1.0);

    let ret = wa * input.return_a_pct + wb * input.return_b_pct;

    let var = wa * wa * sa * sa + wb * wb * sb * sb + 2.0 * wa * wb * sa * sb * rho;
    let vol = var.max(0.0).sqrt() * 100.0;
    let weighted_avg = (wa * sa + wb * sb) * 100.0;

    let sharpe = if vol > 0.0 {
        Some((ret - input.risk_free_pct) / vol)
    } else {
        None
    };

    TwoAssetResult {
        weight_a_pct: input.weight_a_pct,
        weight_b_pct: wb * 100.0,
        portfolio_return_pct: ret,
        portfolio_volatility_pct: vol,
        weighted_avg_volatility_pct: weighted_avg,
        diversification_benefit_pct: weighted_avg - vol,
        sharpe_ratio: sharpe,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn run(wa: f64, rho: f64) -> TwoAssetResult {
        analyze(&TwoAssetInput {
            weight_a_pct: wa,
            return_a_pct: 8.0,
            return_b_pct: 4.0,
            volatility_a_pct: 20.0,
            volatility_b_pct: 10.0,
            correlation: rho,
            risk_free_pct: 0.0,
        })
    }

    #[test]
    fn portfolio_return() {
        // 0.6×8 + 0.4×4 = 6.4%.
        assert!(close(run(60.0, 0.2).portfolio_return_pct, 6.4));
    }

    #[test]
    fn portfolio_volatility() {
        // var = 0.0144 + 0.0016 + 0.00192 = 0.01792 → vol 13.3866%.
        assert!(close(run(60.0, 0.2).portfolio_volatility_pct, 13.3866));
    }

    #[test]
    fn weighted_average_volatility() {
        assert!(close(run(60.0, 0.2).weighted_avg_volatility_pct, 16.0));
    }

    #[test]
    fn diversification_benefit() {
        // 16 − 13.3866 = 2.6134.
        assert!(close(run(60.0, 0.2).diversification_benefit_pct, 2.6134));
    }

    #[test]
    fn perfect_correlation_no_benefit() {
        let r = run(60.0, 1.0);
        assert!(close(r.portfolio_volatility_pct, 16.0));
        assert!(close(r.diversification_benefit_pct, 0.0));
    }

    #[test]
    fn negative_correlation_minimizes_vol() {
        // ρ = −1 → |0.6×20 − 0.4×10| = |12 − 4| = 8%.
        assert!(close(run(60.0, -1.0).portfolio_volatility_pct, 8.0));
    }

    #[test]
    fn sharpe_ratio() {
        let r = analyze(&TwoAssetInput {
            weight_a_pct: 60.0,
            return_a_pct: 8.0,
            return_b_pct: 4.0,
            volatility_a_pct: 20.0,
            volatility_b_pct: 10.0,
            correlation: 0.2,
            risk_free_pct: 2.0,
        });
        // (6.4 − 2) / 13.3866 = 0.328686.
        assert!(close(r.sharpe_ratio.unwrap(), 0.328686));
    }

    #[test]
    fn single_asset_collapses_to_asset_a() {
        let r = run(100.0, 0.2);
        assert!(close(r.portfolio_return_pct, 8.0));
        assert!(close(r.portfolio_volatility_pct, 20.0));
        assert!(close(r.diversification_benefit_pct, 0.0));
    }
}
