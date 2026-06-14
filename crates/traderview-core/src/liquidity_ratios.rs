//! Business liquidity ratios — the short-term solvency measures read off a
//! company balance sheet: net working capital (current assets − current
//! liabilities), the current ratio (current assets ÷ current liabilities), the
//! quick / acid-test ratio (current assets less inventory ÷ current liabilities),
//! and the cash ratio (cash & equivalents ÷ current liabilities). Distinct from
//! the personal-finance ratios module (DTI, emergency fund) — these are
//! business-entity ratios. Pure compute, not financial advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LiquidityInput {
    pub company_label: String,
    pub current_assets_usd: f64,
    #[serde(default)]
    pub inventory_usd: f64,
    #[serde(default)]
    pub cash_and_equivalents_usd: f64,
    pub current_liabilities_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct LiquidityReport {
    /// Current assets − current liabilities.
    pub net_working_capital_usd: f64,
    /// Current assets ÷ current liabilities.
    pub current_ratio: f64,
    /// (Current assets − inventory) ÷ current liabilities.
    pub quick_ratio: f64,
    /// Cash & equivalents ÷ current liabilities.
    pub cash_ratio: f64,
    /// True when the current ratio is at least 1 (can cover near-term obligations).
    pub solvent_short_term: bool,
    pub valid: bool,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

pub fn generate(i: &LiquidityInput) -> LiquidityReport {
    if i.current_liabilities_usd <= 0.0 {
        return LiquidityReport::default();
    }
    let cl = i.current_liabilities_usd;
    let current = i.current_assets_usd / cl;
    LiquidityReport {
        net_working_capital_usd: cents(i.current_assets_usd - cl),
        current_ratio: round4(current),
        quick_ratio: round4((i.current_assets_usd - i.inventory_usd) / cl),
        cash_ratio: round4(i.cash_and_equivalents_usd / cl),
        solvent_short_term: current >= 1.0,
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> LiquidityInput {
        LiquidityInput {
            company_label: "Acme".into(),
            current_assets_usd: 200_000.0,
            inventory_usd: 80_000.0,
            cash_and_equivalents_usd: 50_000.0,
            current_liabilities_usd: 100_000.0,
        }
    }

    #[test]
    fn standard_ratios() {
        let d = generate(&base());
        assert!(close(d.net_working_capital_usd, 100_000.0));
        assert!(close(d.current_ratio, 2.0));
        assert!(close(d.quick_ratio, 1.2));
        assert!(close(d.cash_ratio, 0.5));
        assert!(d.solvent_short_term);
    }

    #[test]
    fn quick_excludes_inventory() {
        let d = generate(&base());
        let no_inv = generate(&LiquidityInput { inventory_usd: 0.0, ..base() });
        assert!(d.quick_ratio < no_inv.quick_ratio);
        // With no inventory, quick equals current.
        assert!(close(no_inv.quick_ratio, no_inv.current_ratio));
    }

    #[test]
    fn insolvent_when_current_below_one() {
        let d = generate(&LiquidityInput { current_assets_usd: 80_000.0, ..base() });
        assert!(close(d.current_ratio, 0.8));
        assert!(close(d.net_working_capital_usd, -20_000.0));
        assert!(!d.solvent_short_term);
    }

    #[test]
    fn ordering_cash_le_quick_le_current() {
        let d = generate(&base());
        assert!(d.cash_ratio <= d.quick_ratio);
        assert!(d.quick_ratio <= d.current_ratio);
    }

    #[test]
    fn zero_liabilities_invalid() {
        let d = generate(&LiquidityInput { current_liabilities_usd: 0.0, ..base() });
        assert!(!d.valid);
    }
}
