//! Age-based asset allocation — the "rule of N" equity glidepath.
//!
//! A simple heuristic for how much of a portfolio to hold in stocks: equity %
//! = N − age. The classic N is 100; longer lifespans pushed it to 110 or 120
//! for more growth. Bonds take the rest. As you age the equity share falls,
//! shifting toward capital preservation — a built-in glidepath.
//!
//!   * equity % = clamp(N − age, 0, 100)
//!   * bond %   = 100 − equity %
//!
//! Reports the split today, the dollar allocation, and the glidepath at
//! 10-year steps. A rule of thumb, not advice. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct AllocationInput {
    pub age: f64,
    /// The rule constant N (100/110/120; defaults to 110 if 0).
    #[serde(default)]
    pub rule_n: f64,
    pub portfolio_value_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct GlidePoint {
    pub age: f64,
    pub equity_pct: f64,
    pub bond_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AllocationResult {
    pub rule_n: f64,
    pub equity_pct: f64,
    pub bond_pct: f64,
    pub equity_usd: f64,
    pub bond_usd: f64,
    /// Equity/bond split at the current age and 10/20/30 years out.
    pub glidepath: Vec<GlidePoint>,
}

fn equity_pct_at(rule_n: f64, age: f64) -> f64 {
    (rule_n - age).clamp(0.0, 100.0)
}

pub fn analyze(i: &AllocationInput) -> AllocationResult {
    let rule_n = if i.rule_n > 0.0 { i.rule_n } else { 110.0 };
    let portfolio = i.portfolio_value_usd.max(0.0);

    let equity_pct = equity_pct_at(rule_n, i.age);
    let bond_pct = 100.0 - equity_pct;

    let glidepath = [0.0, 10.0, 20.0, 30.0]
        .iter()
        .map(|&d| {
            let age = i.age + d;
            let eq = equity_pct_at(rule_n, age);
            GlidePoint { age, equity_pct: eq, bond_pct: 100.0 - eq }
        })
        .collect();

    AllocationResult {
        rule_n,
        equity_pct,
        bond_pct,
        equity_usd: portfolio * equity_pct / 100.0,
        bond_usd: portfolio * bond_pct / 100.0,
        glidepath,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inp(age: f64, n: f64, pv: f64) -> AllocationInput {
        AllocationInput { age, rule_n: n, portfolio_value_usd: pv }
    }

    #[test]
    fn rule_110_age_30() {
        let r = analyze(&inp(30.0, 110.0, 100_000.0));
        assert!((r.equity_pct - 80.0).abs() < 1e-9); // 110 − 30
        assert!((r.bond_pct - 20.0).abs() < 1e-9);
    }

    #[test]
    fn rule_100_age_40() {
        let r = analyze(&inp(40.0, 100.0, 100_000.0));
        assert!((r.equity_pct - 60.0).abs() < 1e-9);
    }

    #[test]
    fn dollar_allocation() {
        let r = analyze(&inp(30.0, 110.0, 100_000.0));
        assert!((r.equity_usd - 80_000.0).abs() < 1e-6);
        assert!((r.bond_usd - 20_000.0).abs() < 1e-6);
    }

    #[test]
    fn equity_clamps_at_100_for_young() {
        // Rule 120, age 10 → 110, clamps to 100.
        let r = analyze(&inp(10.0, 120.0, 100_000.0));
        assert!((r.equity_pct - 100.0).abs() < 1e-9);
        assert!(r.bond_pct.abs() < 1e-9);
    }

    #[test]
    fn equity_clamps_at_0_for_old() {
        // Rule 100, age 110 → −10, clamps to 0.
        let r = analyze(&inp(110.0, 100.0, 100_000.0));
        assert!(r.equity_pct.abs() < 1e-9);
        assert!((r.bond_pct - 100.0).abs() < 1e-9);
    }

    #[test]
    fn glidepath_decreases_equity_with_age() {
        let r = analyze(&inp(40.0, 110.0, 100_000.0));
        assert_eq!(r.glidepath.len(), 4);
        // 70 (age40), 60 (50), 50 (60), 40 (70).
        assert!((r.glidepath[0].equity_pct - 70.0).abs() < 1e-9);
        assert!((r.glidepath[3].equity_pct - 40.0).abs() < 1e-9);
        assert!(r.glidepath[0].equity_pct > r.glidepath[3].equity_pct);
    }

    #[test]
    fn default_rule_is_110() {
        let r = analyze(&inp(50.0, 0.0, 100_000.0));
        assert!((r.rule_n - 110.0).abs() < 1e-9);
        assert!((r.equity_pct - 60.0).abs() < 1e-9); // 110 − 50
    }

    #[test]
    fn glidepath_clamps_at_zero_at_old_ages() {
        // Rule 100, age 80 → 20/10/0/0 across the decades.
        let r = analyze(&inp(80.0, 100.0, 100_000.0));
        assert!((r.glidepath[0].equity_pct - 20.0).abs() < 1e-9);
        assert!(r.glidepath[2].equity_pct.abs() < 1e-9); // age 100 → 0
        assert!(r.glidepath[3].equity_pct.abs() < 1e-9); // age 110 → clamped 0
    }
}
