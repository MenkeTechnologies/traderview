//! Three-fund portfolio recommender (Boglehead).
//!
//! The canonical Boglehead three-fund portfolio: US stocks +
//! International stocks + Bonds. Allocation depends on age + risk
//! tolerance. Heuristics:
//!
//!   - Total stock allocation = 110 − age (aggressive), 100 − age
//!     (moderate), or 90 − age (conservative). Clamped to [10, 95].
//!   - Within stocks, 70/30 US/international (Bogle's recommendation;
//!     Vanguard target-date funds use 60/40).
//!   - Bonds = 100 − stocks.
//!
//! Compute returns recommended weights, target dollar amounts given
//! a portfolio total, current weights from user holdings, drift
//! (current − target), and suggested rebalance trades.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ThreeFundInput {
    pub age: u32,
    pub risk_tolerance: String, // "conservative" | "moderate" | "aggressive"
    pub current_us_stocks_usd: f64,
    pub current_intl_stocks_usd: f64,
    pub current_bonds_usd: f64,
    /// Within stocks, fraction allocated to US (rest to international).
    /// 0.70 = 70/30 US/intl (Bogle); 0.60 = Vanguard TDF default.
    #[serde(default = "default_us_share")]
    pub us_within_stocks_pct: f64,
}

fn default_us_share() -> f64 { 70.0 }

#[derive(Debug, Clone, Serialize)]
pub struct AssetReport {
    pub target_weight_pct: f64,
    pub target_dollar_usd: f64,
    pub current_dollar_usd: f64,
    pub current_weight_pct: f64,
    pub drift_pct: f64,
    pub rebalance_buy_sell_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThreeFundReport {
    pub total_portfolio_usd: f64,
    pub us_stocks: AssetReport,
    pub intl_stocks: AssetReport,
    pub bonds: AssetReport,
    pub total_stock_target_pct: f64,
    pub total_bond_target_pct: f64,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn total_stock_pct(age: u32, risk: &str) -> f64 {
    let raw = match risk {
        "aggressive" => 110.0_f64 - age as f64,
        "conservative" => 90.0 - age as f64,
        _ => 100.0 - age as f64, // moderate default
    };
    raw.clamp(10.0, 95.0)
}

pub fn compute(input: &ThreeFundInput) -> ThreeFundReport {
    let total = input.current_us_stocks_usd + input.current_intl_stocks_usd
        + input.current_bonds_usd;
    let stock_pct = total_stock_pct(input.age, &input.risk_tolerance);
    let bond_pct = 100.0 - stock_pct;
    let us_share = input.us_within_stocks_pct.clamp(0.0, 100.0);
    let us_target_pct = stock_pct * us_share / 100.0;
    let intl_target_pct = stock_pct * (100.0 - us_share) / 100.0;

    let mk = |target_pct: f64, current: f64| -> AssetReport {
        let target_dollar = total * target_pct / 100.0;
        let current_pct = if total > 0.0 { current / total * 100.0 } else { 0.0 };
        let drift = current_pct - target_pct;
        let buy_sell = target_dollar - current;
        AssetReport {
            target_weight_pct: target_pct,
            target_dollar_usd: target_dollar,
            current_dollar_usd: current,
            current_weight_pct: current_pct,
            drift_pct: drift,
            rebalance_buy_sell_usd: buy_sell,
        }
    };
    ThreeFundReport {
        total_portfolio_usd: total,
        us_stocks: mk(us_target_pct, input.current_us_stocks_usd),
        intl_stocks: mk(intl_target_pct, input.current_intl_stocks_usd),
        bonds: mk(bond_pct, input.current_bonds_usd),
        total_stock_target_pct: stock_pct,
        total_bond_target_pct: bond_pct,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> ThreeFundInput {
        ThreeFundInput {
            age: 40,
            risk_tolerance: "moderate".into(),
            current_us_stocks_usd: 100_000.0,
            current_intl_stocks_usd: 20_000.0,
            current_bonds_usd: 30_000.0,
            us_within_stocks_pct: 70.0,
        }
    }

    #[test]
    fn total_stock_pct_moderate_basic() {
        // age 40, moderate = 100 − 40 = 60%
        assert_eq!(total_stock_pct(40, "moderate"), 60.0);
    }

    #[test]
    fn total_stock_pct_aggressive_basic() {
        // age 40, aggressive = 110 − 40 = 70%
        assert_eq!(total_stock_pct(40, "aggressive"), 70.0);
    }

    #[test]
    fn total_stock_pct_conservative_basic() {
        // age 40, conservative = 90 − 40 = 50%
        assert_eq!(total_stock_pct(40, "conservative"), 50.0);
    }

    #[test]
    fn total_stock_pct_clamps_high() {
        // age 5 aggressive = 105 → clamped to 95
        assert_eq!(total_stock_pct(5, "aggressive"), 95.0);
    }

    #[test]
    fn total_stock_pct_clamps_low() {
        // age 90 conservative = 0 → clamped to 10
        assert_eq!(total_stock_pct(90, "conservative"), 10.0);
    }

    #[test]
    fn total_stock_pct_unknown_risk_defaults_moderate() {
        assert_eq!(total_stock_pct(40, "bogus"), 60.0);
    }

    #[test]
    fn compute_total_basic() {
        let r = compute(&input());
        assert_eq!(r.total_portfolio_usd, 150_000.0);
    }

    #[test]
    fn compute_target_weights_sum_to_100() {
        let r = compute(&input());
        let sum = r.us_stocks.target_weight_pct + r.intl_stocks.target_weight_pct
            + r.bonds.target_weight_pct;
        assert!((sum - 100.0).abs() < 1e-6);
    }

    #[test]
    fn compute_us_target_70pct_of_stocks() {
        let r = compute(&input());
        // 60% stocks × 70% US within = 42%
        assert!((r.us_stocks.target_weight_pct - 42.0).abs() < 1e-6);
        // 60% stocks × 30% intl within = 18%
        assert!((r.intl_stocks.target_weight_pct - 18.0).abs() < 1e-6);
        // 100 − 60 = 40% bonds
        assert!((r.bonds.target_weight_pct - 40.0).abs() < 1e-6);
    }

    #[test]
    fn compute_drift_basic() {
        let r = compute(&input());
        // Current US: 100k / 150k = 66.67%, target 42% → drift +24.67%
        let curr = 100_000.0 / 150_000.0 * 100.0;
        assert!((r.us_stocks.drift_pct - (curr - 42.0)).abs() < 1e-6);
    }

    #[test]
    fn compute_rebalance_buy_sell_signs() {
        let r = compute(&input());
        // US over-allocated → sell (negative buy_sell)
        assert!(r.us_stocks.rebalance_buy_sell_usd < 0.0);
        // Intl: 20k current, target 18% of 150k = 27k → buy 7k
        assert!(r.intl_stocks.rebalance_buy_sell_usd > 0.0);
        assert!((r.intl_stocks.rebalance_buy_sell_usd - 7_000.0).abs() < 1.0);
        // Bonds: 30k current, target 40% of 150k = 60k → buy 30k
        assert!((r.bonds.rebalance_buy_sell_usd - 30_000.0).abs() < 1.0);
    }

    #[test]
    fn compute_zero_portfolio_safe() {
        let r = compute(&ThreeFundInput {
            age: 40,
            risk_tolerance: "moderate".into(),
            current_us_stocks_usd: 0.0,
            current_intl_stocks_usd: 0.0,
            current_bonds_usd: 0.0,
            us_within_stocks_pct: 70.0,
        });
        assert_eq!(r.total_portfolio_usd, 0.0);
        assert_eq!(r.us_stocks.current_weight_pct, 0.0);
    }

    #[test]
    fn compute_old_age_high_bond_allocation() {
        let mut i = input();
        i.age = 75;
        // moderate, age 75 = 100 − 75 = 25% stocks → 75% bonds
        let r = compute(&i);
        assert!((r.total_bond_target_pct - 75.0).abs() < 1e-6);
    }
}
