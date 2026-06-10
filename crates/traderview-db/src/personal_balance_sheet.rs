//! Personal balance sheet — GAAP-style current vs non-current split.
//!
//! Unlike `net_worth_tracker` (a simple list-of-line-items snapshot),
//! this organises assets and liabilities into the standard accounting
//! categorisation so the user can compute liquidity ratios that
//! depend on the < 12-month bucket:
//!
//!   - current assets       — cash, savings, money market, CDs maturing
//!                            in < 12m, receivables
//!   - non-current assets   — retirement accounts (locked until 59½),
//!                            home equity, vehicles, collectibles
//!   - current liabilities  — credit cards, bills due, loan payments
//!                            due within 12m
//!   - long-term liabilities— mortgage principal beyond 12m, student
//!                            loans, auto-loan principal beyond 12m
//!
//! Outputs:
//!   - equity_usd = total_assets − total_liabilities (same as net worth)
//!   - current_ratio = current_assets / current_liabilities
//!   - quick_ratio   = (current_assets − illiquid_in_current) /
//!                     current_liabilities — but for personal finance the
//!                     classical quick = (cash + receivables) / curr_liab
//!     We surface both ratios. Quick-ratio inputs come from a separate
//!     `liquid_only` flag on each current-asset line.
//!   - working_capital = current_assets − current_liabilities
//!   - net_worth_usd   = equity (alias for legibility)
//!
//! Pure compute — no DB I/O.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct AssetItem {
    pub name: String,
    pub value_usd: f64,
    /// If true, asset is convertible to cash within 12 months
    /// (cash, MMF, treasury bills, brokerage cash, etc.).
    #[serde(default)]
    pub is_current: bool,
    /// If true, asset is cash or near-cash (for quick ratio).
    /// Implies is_current.
    #[serde(default)]
    pub is_liquid: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LiabilityItem {
    pub name: String,
    pub value_usd: f64,
    /// If true, liability is due within 12 months
    /// (credit card balance, current-year property tax, etc.).
    #[serde(default)]
    pub is_current: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BalanceSheetInput {
    #[serde(default)]
    pub assets: Vec<AssetItem>,
    #[serde(default)]
    pub liabilities: Vec<LiabilityItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BalanceSheetReport {
    pub current_assets_usd: f64,
    pub non_current_assets_usd: f64,
    pub total_assets_usd: f64,
    pub current_liabilities_usd: f64,
    pub long_term_liabilities_usd: f64,
    pub total_liabilities_usd: f64,
    pub equity_usd: f64,
    pub net_worth_usd: f64,
    pub working_capital_usd: f64,
    pub current_ratio: Option<f64>,
    pub quick_ratio: Option<f64>,
    pub debt_to_equity: Option<f64>,
    pub status: String,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn split_assets(assets: &[AssetItem]) -> (f64, f64, f64) {
    let mut current = 0.0;
    let mut non_current = 0.0;
    let mut liquid = 0.0;
    for a in assets {
        if a.is_current {
            current += a.value_usd;
        } else {
            non_current += a.value_usd;
        }
        if a.is_liquid {
            liquid += a.value_usd;
        }
    }
    (current, non_current, liquid)
}

pub fn split_liabilities(liab: &[LiabilityItem]) -> (f64, f64) {
    let mut current = 0.0;
    let mut long_term = 0.0;
    for l in liab {
        if l.is_current {
            current += l.value_usd;
        } else {
            long_term += l.value_usd;
        }
    }
    (current, long_term)
}

pub fn ratio_or_none(num: f64, denom: f64) -> Option<f64> {
    if denom.abs() < 1e-9 {
        None
    } else {
        Some(num / denom)
    }
}

pub fn compute(input: &BalanceSheetInput) -> BalanceSheetReport {
    let (current_a, non_current_a, liquid_a) = split_assets(&input.assets);
    let (current_l, long_term_l) = split_liabilities(&input.liabilities);
    let total_a = current_a + non_current_a;
    let total_l = current_l + long_term_l;
    let equity = total_a - total_l;
    let working_capital = current_a - current_l;
    let current_ratio = ratio_or_none(current_a, current_l);
    let quick_ratio = ratio_or_none(liquid_a, current_l);
    let debt_to_equity = ratio_or_none(total_l, equity);
    let status = if equity > 0.0 && working_capital > 0.0 {
        "solvent"
    } else if equity > 0.0 {
        "illiquid"
    } else {
        "insolvent"
    }
    .to_string();
    BalanceSheetReport {
        current_assets_usd: current_a,
        non_current_assets_usd: non_current_a,
        total_assets_usd: total_a,
        current_liabilities_usd: current_l,
        long_term_liabilities_usd: long_term_l,
        total_liabilities_usd: total_l,
        equity_usd: equity,
        net_worth_usd: equity,
        working_capital_usd: working_capital,
        current_ratio,
        quick_ratio,
        debt_to_equity,
        status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(name: &str, v: f64, cur: bool, liq: bool) -> AssetItem {
        AssetItem { name: name.into(), value_usd: v, is_current: cur, is_liquid: liq }
    }

    fn l(name: &str, v: f64, cur: bool) -> LiabilityItem {
        LiabilityItem { name: name.into(), value_usd: v, is_current: cur }
    }

    #[test]
    fn split_assets_empty() {
        let (c, nc, liq) = split_assets(&[]);
        assert_eq!((c, nc, liq), (0.0, 0.0, 0.0));
    }

    #[test]
    fn split_assets_basic() {
        let xs = vec![
            a("cash",   5_000.0, true, true),
            a("ira",  100_000.0, false, false),
            a("brok",  50_000.0, true, false),
        ];
        let (c, nc, liq) = split_assets(&xs);
        assert_eq!(c, 55_000.0);
        assert_eq!(nc, 100_000.0);
        assert_eq!(liq, 5_000.0);
    }

    #[test]
    fn split_liabilities_basic() {
        let xs = vec![l("cc", 3_000.0, true), l("mortgage", 250_000.0, false)];
        let (c, lt) = split_liabilities(&xs);
        assert_eq!(c, 3_000.0);
        assert_eq!(lt, 250_000.0);
    }

    #[test]
    fn ratio_or_none_zero_denom() {
        assert!(ratio_or_none(100.0, 0.0).is_none());
    }

    #[test]
    fn ratio_or_none_basic() {
        assert_eq!(ratio_or_none(200.0, 100.0), Some(2.0));
    }

    #[test]
    fn compute_solvent_household() {
        let r = compute(&BalanceSheetInput {
            assets: vec![
                a("cash",  10_000.0, true, true),
                a("ira",  100_000.0, false, false),
                a("home", 400_000.0, false, false),
            ],
            liabilities: vec![
                l("cc",          1_000.0, true),
                l("mortgage",  250_000.0, false),
            ],
        });
        assert_eq!(r.current_assets_usd, 10_000.0);
        assert_eq!(r.non_current_assets_usd, 500_000.0);
        assert_eq!(r.total_assets_usd, 510_000.0);
        assert_eq!(r.current_liabilities_usd, 1_000.0);
        assert_eq!(r.long_term_liabilities_usd, 250_000.0);
        assert_eq!(r.equity_usd, 259_000.0);
        assert_eq!(r.working_capital_usd, 9_000.0);
        assert_eq!(r.current_ratio, Some(10.0));
        assert_eq!(r.quick_ratio, Some(10.0));
        assert!((r.debt_to_equity.unwrap() - 251_000.0 / 259_000.0).abs() < 1e-9);
        assert_eq!(r.status, "solvent");
    }

    #[test]
    fn compute_illiquid_positive_equity_no_current_assets() {
        let r = compute(&BalanceSheetInput {
            assets: vec![a("home", 500_000.0, false, false)],
            liabilities: vec![l("cc", 1_000.0, true)],
        });
        assert!(r.equity_usd > 0.0);
        assert!(r.working_capital_usd < 0.0);
        assert_eq!(r.status, "illiquid");
        assert_eq!(r.current_ratio, Some(0.0));
    }

    #[test]
    fn compute_insolvent_negative_equity() {
        let r = compute(&BalanceSheetInput {
            assets: vec![a("car", 5_000.0, false, false)],
            liabilities: vec![l("loan", 20_000.0, false)],
        });
        assert_eq!(r.equity_usd, -15_000.0);
        assert_eq!(r.status, "insolvent");
    }

    #[test]
    fn compute_zero_liab_no_ratios() {
        let r = compute(&BalanceSheetInput {
            assets: vec![a("cash", 1_000.0, true, true)],
            liabilities: vec![],
        });
        assert!(r.current_ratio.is_none());
        assert!(r.quick_ratio.is_none());
    }

    #[test]
    fn compute_equity_alias_net_worth() {
        let r = compute(&BalanceSheetInput {
            assets: vec![a("cash", 1_000.0, true, true)],
            liabilities: vec![l("cc", 200.0, true)],
        });
        assert_eq!(r.equity_usd, r.net_worth_usd);
    }

    #[test]
    fn compute_quick_ratio_excludes_non_liquid_current() {
        let r = compute(&BalanceSheetInput {
            assets: vec![
                a("brokerage", 10_000.0, true, false),
                a("cash",       1_000.0, true, true),
            ],
            liabilities: vec![l("cc", 2_000.0, true)],
        });
        assert_eq!(r.current_ratio, Some(11_000.0 / 2_000.0));
        assert_eq!(r.quick_ratio, Some(0.5));
    }

    #[test]
    fn compute_working_capital_basic() {
        let r = compute(&BalanceSheetInput {
            assets: vec![a("cash", 15_000.0, true, true)],
            liabilities: vec![l("cc", 5_000.0, true)],
        });
        assert_eq!(r.working_capital_usd, 10_000.0);
    }
}
