//! Efficient frontier (Markowitz) — traces the mean-variance frontier for a set
//! of assets and reports the minimum-variance and maximum-Sharpe (tangency)
//! portfolios plus the capital-market-line slope. It builds the covariance matrix
//! from each asset's volatility and a single constant pairwise correlation, then
//! reuses the closed-form solver in `min_variance_portfolio` for the two special
//! portfolios and sweeps their two-fund combinations to draw the frontier curve.
//! Short-selling is allowed (unconstrained MVO). Pure compute.

use crate::min_variance_portfolio;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Asset {
    pub name: String,
    /// Expected annual return, percent.
    pub expected_return_pct: f64,
    /// Annual volatility (standard deviation), percent.
    pub volatility_pct: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FrontierInput {
    pub assets: Vec<Asset>,
    /// Constant pairwise correlation across assets, percent (e.g. 30 = 0.30).
    #[serde(default)]
    pub correlation_pct: f64,
    /// Risk-free rate, percent.
    #[serde(default)]
    pub risk_free_pct: f64,
    /// Number of frontier points to trace.
    #[serde(default = "default_points")]
    pub points: u32,
}

fn default_points() -> u32 {
    25
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Weight {
    pub name: String,
    /// Portfolio weight, percent (can be negative when short-selling).
    pub weight_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Portfolio {
    pub weights: Vec<Weight>,
    pub expected_return_pct: f64,
    pub volatility_pct: f64,
    pub sharpe: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FrontierPoint {
    pub expected_return_pct: f64,
    pub volatility_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct FrontierReport {
    pub min_variance: Portfolio,
    pub tangency: Portfolio,
    /// Slope of the capital market line = tangency Sharpe ratio.
    pub cml_slope: f64,
    pub frontier: Vec<FrontierPoint>,
    pub ok: bool,
}

impl Default for Portfolio {
    fn default() -> Self {
        Portfolio { weights: Vec::new(), expected_return_pct: 0.0, volatility_pct: 0.0, sharpe: 0.0 }
    }
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

fn matvec(m: &[Vec<f64>], v: &[f64]) -> Vec<f64> {
    m.iter().map(|r| r.iter().zip(v).map(|(a, b)| a * b).sum()).collect()
}

fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

pub fn generate(i: &FrontierInput) -> FrontierReport {
    let n = i.assets.len();
    if n < 2 {
        return FrontierReport::default();
    }
    let rho = i.correlation_pct / 100.0;
    let rf = i.risk_free_pct / 100.0;
    // Decimal returns and volatilities.
    let rets: Vec<f64> = i.assets.iter().map(|a| a.expected_return_pct / 100.0).collect();
    let vols: Vec<f64> = i.assets.iter().map(|a| a.volatility_pct / 100.0).collect();

    // Covariance from vols and a constant pairwise correlation.
    let cov: Vec<Vec<f64>> = (0..n)
        .map(|r| {
            (0..n)
                .map(|c| if r == c { vols[r] * vols[r] } else { rho * vols[r] * vols[c] })
                .collect()
        })
        .collect();
    let excess: Vec<f64> = rets.iter().map(|r| r - rf).collect();

    let rep = match min_variance_portfolio::solve(&cov, &excess) {
        Some(r) => r,
        None => return FrontierReport::default(),
    };

    let names: Vec<String> = i.assets.iter().map(|a| a.name.clone()).collect();
    let portfolio = |w: &[f64]| -> Portfolio {
        let ret = dot(w, &rets);
        let var = dot(w, &matvec(&cov, w));
        let vol = var.max(0.0).sqrt();
        let sharpe = if vol > 0.0 { (ret - rf) / vol } else { 0.0 };
        Portfolio {
            weights: names
                .iter()
                .zip(w)
                .map(|(name, &wt)| Weight { name: name.clone(), weight_pct: round2(wt * 100.0) })
                .collect(),
            expected_return_pct: round2(ret * 100.0),
            volatility_pct: round2(vol * 100.0),
            sharpe: round4(sharpe),
        }
    };

    let min_var = portfolio(&rep.min_variance_weights);
    let tangency = portfolio(&rep.tangency_weights);

    // Two-fund separation: every frontier portfolio is an affine combination of
    // any two frontier portfolios. Sweep t·w_mv + (1−t)·w_tan.
    let pts = i.points.max(2);
    let (t_lo, t_hi) = (-0.6_f64, 1.4_f64);
    let frontier: Vec<FrontierPoint> = (0..pts)
        .map(|k| {
            let t = t_hi - (t_hi - t_lo) * (k as f64) / ((pts - 1) as f64);
            let w: Vec<f64> = rep
                .min_variance_weights
                .iter()
                .zip(&rep.tangency_weights)
                .map(|(mv, tan)| t * mv + (1.0 - t) * tan)
                .collect();
            let ret = dot(&w, &rets);
            let vol = dot(&w, &matvec(&cov, &w)).max(0.0).sqrt();
            FrontierPoint {
                expected_return_pct: round2(ret * 100.0),
                volatility_pct: round2(vol * 100.0),
            }
        })
        .collect();

    FrontierReport {
        cml_slope: tangency.sharpe,
        min_variance: min_var,
        tangency,
        frontier,
        ok: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.05
    }

    fn base() -> FrontierInput {
        FrontierInput {
            assets: vec![
                Asset { name: "Stocks".into(), expected_return_pct: 8.0, volatility_pct: 12.0 },
                Asset { name: "Bonds".into(), expected_return_pct: 12.0, volatility_pct: 18.0 },
                Asset { name: "REIT".into(), expected_return_pct: 15.0, volatility_pct: 25.0 },
            ],
            correlation_pct: 30.0,
            risk_free_pct: 3.0,
            points: 25,
        }
    }

    #[test]
    fn min_variance_and_tangency() {
        let d = generate(&base());
        assert!(d.ok);
        // Verified against an independent inverse-covariance computation.
        assert!(close(d.min_variance.volatility_pct, 11.15));
        assert!(close(d.min_variance.expected_return_pct, 9.19));
        assert!(close(d.tangency.volatility_pct, 12.89));
        assert!(close(d.tangency.expected_return_pct, 11.27));
        assert!(close(d.tangency.sharpe, 0.6417));
    }

    #[test]
    fn weights_sum_to_100() {
        let d = generate(&base());
        let mv: f64 = d.min_variance.weights.iter().map(|w| w.weight_pct).sum();
        let tan: f64 = d.tangency.weights.iter().map(|w| w.weight_pct).sum();
        assert!(close(mv, 100.0));
        assert!(close(tan, 100.0));
    }

    #[test]
    fn tangency_has_max_sharpe_on_frontier() {
        let d = generate(&base());
        // No frontier point should beat the tangency Sharpe (the CML slope).
        for p in &d.frontier {
            let sharpe = if p.volatility_pct > 0.0 {
                (p.expected_return_pct - 3.0) / p.volatility_pct
            } else {
                0.0
            };
            assert!(sharpe <= d.tangency.sharpe + 0.01);
        }
    }

    #[test]
    fn cml_slope_equals_tangency_sharpe() {
        let d = generate(&base());
        assert!(close(d.cml_slope, d.tangency.sharpe));
    }

    #[test]
    fn frontier_has_requested_points() {
        let d = generate(&FrontierInput { points: 10, ..base() });
        assert_eq!(d.frontier.len(), 10);
    }

    #[test]
    fn too_few_assets_not_ok() {
        let d = generate(&FrontierInput {
            assets: vec![Asset { name: "A".into(), expected_return_pct: 8.0, volatility_pct: 12.0 }],
            ..base()
        });
        assert!(!d.ok);
    }
}
