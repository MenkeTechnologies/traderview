//! Almgren-Chriss (2000) optimal execution schedule.
//!
//! For a parent order of `total_shares` X liquidated over a horizon T
//! in `n_intervals` equal time slices τ = T/n, the cost-of-execution
//! IS = market impact + risk penalty:
//!
//!   cost = Σ τ·η·(v_k/τ)² + 0.5·γ·X² + 0.5·λ·σ²·Σ τ·x_k²
//!
//! where:
//!   v_k = shares traded in slice k
//!   x_k = remaining inventory at end of slice k
//!   η   = temporary price-impact coefficient
//!   γ   = permanent price-impact coefficient
//!   λ   = risk-aversion coefficient
//!   σ   = price volatility
//!
//! Closed-form optimal trajectory (continuous-time limit, discretized):
//!
//!   κ̃² = λ·σ² / η     (or equivalently λσ²/(η + 0.5γτ))
//!   κ  = √(κ̃² / τ²)  (for small τ)  — we use the standard form
//!   x_k = X · sinh(κ(T − t_k)) / sinh(κT)        (positions over time)
//!   v_k = x_{k−1} − x_k                            (trade sizes per slice)
//!
//! Limits:
//!   λ → 0:  uniform liquidation (TWAP) — minimize impact, ignore risk.
//!   λ → ∞: front-load aggressively — minimize timing risk, accept impact.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AlmgrenChrissParams {
    pub total_shares: f64,
    pub horizon_seconds: f64,
    pub n_intervals: usize,
    /// Temporary impact coefficient (cost per (share/second)²).
    pub eta: f64,
    /// Permanent impact coefficient (linear in shares).
    pub gamma: f64,
    /// Risk aversion (higher = more impatient).
    pub lambda: f64,
    /// Price volatility (per √second).
    pub sigma: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AlmgrenChrissReport {
    pub trade_schedule: Vec<f64>,
    pub inventory_path: Vec<f64>,
    pub expected_impact_cost: f64,
    pub risk_variance: f64,
    pub kappa: f64,
}

pub fn solve(p: &AlmgrenChrissParams) -> Option<AlmgrenChrissReport> {
    if !p.total_shares.is_finite() || p.total_shares == 0.0
        || !p.horizon_seconds.is_finite() || p.horizon_seconds <= 0.0
        || p.n_intervals == 0
        || !p.eta.is_finite() || p.eta <= 0.0
        || !p.gamma.is_finite() || p.gamma < 0.0
        || !p.lambda.is_finite() || p.lambda < 0.0
        || !p.sigma.is_finite() || p.sigma < 0.0
    {
        return None;
    }
    let n = p.n_intervals;
    let tau = p.horizon_seconds / n as f64;
    // κ̃² = λ σ² / η; κ = √(κ̃²) approximating the continuous form.
    // Use the discrete-form denominator η − 0.5·γ·τ if it stays positive,
    // else fall back to η.
    let denom_kappa = (p.eta - 0.5 * p.gamma * tau).max(p.eta * 1e-6);
    let kappa_sq = p.lambda * p.sigma * p.sigma / denom_kappa;
    let kappa = kappa_sq.max(0.0).sqrt();
    let mut inventory = Vec::with_capacity(n + 1);
    let mut schedule = Vec::with_capacity(n);
    let sign = p.total_shares.signum();
    let abs_x = p.total_shares.abs();
    // Edge case: λ=0 or σ=0 → κ=0 → uniform liquidation (TWAP).
    if kappa.abs() < 1e-12 {
        inventory.push(p.total_shares);
        for k in 1..=n {
            let frac = 1.0 - (k as f64) / (n as f64);
            inventory.push(p.total_shares * frac);
        }
    } else {
        let denom_sinh = (kappa * p.horizon_seconds).sinh();
        if denom_sinh.abs() < 1e-30 {
            // Numerically degenerate; fall back to TWAP.
            inventory.push(p.total_shares);
            for k in 1..=n {
                let frac = 1.0 - (k as f64) / (n as f64);
                inventory.push(p.total_shares * frac);
            }
        } else {
            for k in 0..=n {
                let t_k = (k as f64) * tau;
                let x_k = sign * abs_x * (kappa * (p.horizon_seconds - t_k)).sinh() / denom_sinh;
                inventory.push(x_k);
            }
        }
    }
    for k in 0..n {
        schedule.push(inventory[k] - inventory[k + 1]);
    }
    // Expected cost = 0.5·γ·X² + η/τ · Σv_k²  (permanent + temporary impact).
    let permanent = 0.5 * p.gamma * p.total_shares * p.total_shares;
    let temporary: f64 = schedule.iter().map(|v| v * v).sum::<f64>() * p.eta / tau;
    let expected_impact_cost = permanent + temporary;
    // Variance of execution P&L = σ² · Σ τ · x_k² (k=1..n, post-trade).
    let risk_variance: f64 = inventory.iter().skip(1)
        .map(|x| p.sigma * p.sigma * tau * x * x)
        .sum();
    Some(AlmgrenChrissReport {
        trade_schedule: schedule,
        inventory_path: inventory,
        expected_impact_cost,
        risk_variance,
        kappa,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> AlmgrenChrissParams {
        // Canonical Almgren-Chriss demo parameters (Almgren-Chriss 2000
        // and the QuantLib test suite both normalize time to T=1 day):
        // dollar-vol = 95 cents/day, 30 slices of ~13 min each, 1M-share
        // parent order. Small η/γ → keeps κ·T well inside the sinh-finite
        // band (κ ≈ 0.6 → sinh(κT) ≈ 0.64).
        AlmgrenChrissParams {
            total_shares: 1_000_000.0,
            horizon_seconds: 1.0,         // normalized: 1 day = 1 unit
            n_intervals: 30,
            eta: 2.5e-6,
            gamma: 2.5e-7,
            lambda: 1e-6,
            sigma: 0.95,                  // dollar-vol per √day
        }
    }

    #[test]
    fn invalid_inputs_return_none() {
        let mut bad = p(); bad.total_shares = 0.0;
        assert!(solve(&bad).is_none());
        let mut bad = p(); bad.horizon_seconds = 0.0;
        assert!(solve(&bad).is_none());
        let mut bad = p(); bad.n_intervals = 0;
        assert!(solve(&bad).is_none());
        let mut bad = p(); bad.eta = 0.0;
        assert!(solve(&bad).is_none());
        let mut bad = p(); bad.eta = f64::NAN;
        assert!(solve(&bad).is_none());
    }

    #[test]
    fn inventory_starts_at_total_and_ends_at_zero() {
        let r = solve(&p()).unwrap();
        assert!((r.inventory_path[0] - 1_000_000.0).abs() < 1e-6);
        assert!(r.inventory_path.last().unwrap().abs() < 1e-3);
    }

    #[test]
    fn trade_schedule_sums_to_total_shares() {
        let r = solve(&p()).unwrap();
        let sum: f64 = r.trade_schedule.iter().sum();
        assert!((sum - 1_000_000.0).abs() < 1e-3);
    }

    #[test]
    fn risk_neutral_yields_uniform_liquidation() {
        // λ = 0 → TWAP — equal slices.
        let mut params = p();
        params.lambda = 0.0;
        let r = solve(&params).unwrap();
        let target = 1_000_000.0 / params.n_intervals as f64;
        for v in &r.trade_schedule {
            assert!((v - target).abs() < 1e-3,
                "expected uniform slice = {target}, got {v}");
        }
    }

    #[test]
    fn risk_averse_front_loads_trading() {
        // Higher λ → faster initial liquidation than uniform.
        let mut low = p();
        low.lambda = 1e-7;
        let mut high = p();
        high.lambda = 1e-3;
        let r_low = solve(&low).unwrap();
        let r_high = solve(&high).unwrap();
        // First-slice fraction should be larger for high-λ.
        let frac_low = r_low.trade_schedule[0] / 1_000_000.0;
        let frac_high = r_high.trade_schedule[0] / 1_000_000.0;
        assert!(frac_high > frac_low,
            "high-λ should front-load: low={frac_low} high={frac_high}");
    }

    #[test]
    fn negative_total_shares_yields_buy_program() {
        // Negative shares = need to BUY — schedule should be negative (buys).
        let mut params = p();
        params.total_shares = -1_000_000.0;
        let r = solve(&params).unwrap();
        let sum: f64 = r.trade_schedule.iter().sum();
        assert!((sum - (-1_000_000.0)).abs() < 1e-3);
        assert!(r.inventory_path[0] < 0.0);
    }

    #[test]
    fn zero_volatility_yields_uniform_liquidation() {
        let mut params = p();
        params.sigma = 0.0;
        let r = solve(&params).unwrap();
        let target = 1_000_000.0 / params.n_intervals as f64;
        for v in &r.trade_schedule {
            assert!((v - target).abs() < 1e-3);
        }
    }

    #[test]
    fn expected_impact_cost_positive() {
        let r = solve(&p()).unwrap();
        assert!(r.expected_impact_cost > 0.0);
    }

    #[test]
    fn risk_variance_decreases_as_horizon_shrinks() {
        // Faster execution → less timing risk.
        let mut slow = p();
        slow.horizon_seconds = 1.0;
        let mut fast = p();
        fast.horizon_seconds = 0.1;
        let r_slow = solve(&slow).unwrap();
        let r_fast = solve(&fast).unwrap();
        assert!(r_fast.risk_variance < r_slow.risk_variance);
    }
}
