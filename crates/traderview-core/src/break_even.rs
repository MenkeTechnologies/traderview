//! Break-even / cost-volume-profit (CVP) analysis.
//!
//! The most fundamental small-business question: how many units must I sell
//! to cover my costs? Each unit sold contributes `price − variable cost`
//! toward the fixed costs (the **contribution margin**); break-even is the
//! point where accumulated contribution exactly covers fixed costs.
//!
//!   * **Break-even units** = fixed costs / contribution margin per unit
//!   * **Break-even revenue** = fixed costs / contribution-margin ratio
//!   * **Target-profit units** = (fixed costs + target profit) / CM per unit
//!   * **Margin of safety** = how far expected sales sit above break-even
//!
//! When the contribution margin is zero or negative (each unit loses money
//! before fixed costs), there is no break-even at any volume — selling more
//! only deepens the loss. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct BreakEvenInput {
    /// Total fixed costs for the period (rent, salaries, insurance, …).
    pub fixed_costs_usd: f64,
    pub price_per_unit_usd: f64,
    pub variable_cost_per_unit_usd: f64,
    /// Desired profit on top of covering fixed costs (default 0 = break-even).
    #[serde(default)]
    pub target_profit_usd: f64,
    /// Units you actually expect to sell, for the margin-of-safety figures
    /// (0 ⇒ skip those outputs).
    #[serde(default)]
    pub expected_units: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BreakEvenResult {
    /// Price − variable cost: what each unit contributes to fixed costs.
    pub contribution_margin_usd: f64,
    /// Contribution margin as a fraction of price (0..1), or `None` if price ≤ 0.
    pub contribution_margin_ratio: Option<f64>,
    /// Units needed to break even; `None` when CM ≤ 0 (no break-even exists).
    pub break_even_units: Option<f64>,
    /// Revenue at the break-even point; `None` when CM ≤ 0.
    pub break_even_revenue_usd: Option<f64>,
    /// Units needed to reach the target profit; `None` when CM ≤ 0.
    pub units_for_target_profit: Option<f64>,
    /// Revenue needed to reach the target profit; `None` when CM ≤ 0.
    pub revenue_for_target_profit_usd: Option<f64>,
    /// Expected units − break-even units; `None` if no expected units / no CM.
    pub margin_of_safety_units: Option<f64>,
    /// Margin of safety as a percent of expected units; `None` likewise.
    pub margin_of_safety_pct: Option<f64>,
    /// Operating profit at the expected volume; `None` if no expected units.
    pub profit_at_expected_usd: Option<f64>,
    /// True when CM ≤ 0 — no volume ever breaks even.
    pub no_break_even: bool,
}

pub fn analyze(i: &BreakEvenInput) -> BreakEvenResult {
    let fixed = i.fixed_costs_usd.max(0.0);
    let price = i.price_per_unit_usd;
    let cm = price - i.variable_cost_per_unit_usd;
    let has_be = cm > 0.0;

    let cm_ratio = if price > 0.0 { Some(cm / price) } else { None };

    let be_units = if has_be { Some(fixed / cm) } else { None };
    let be_revenue = be_units.map(|u| u * price);

    let target_units = if has_be { Some((fixed + i.target_profit_usd) / cm) } else { None };
    let target_revenue = target_units.map(|u| u * price);

    let (mos_units, mos_pct, profit_at_expected) = if i.expected_units > 0.0 {
        let profit = i.expected_units * cm - fixed;
        let (mu, mp) = match be_units {
            Some(be) => {
                let mu = i.expected_units - be;
                let mp = if i.expected_units > 0.0 { mu / i.expected_units * 100.0 } else { 0.0 };
                (Some(mu), Some(mp))
            }
            None => (None, None),
        };
        (mu, mp, Some(profit))
    } else {
        (None, None, None)
    };

    BreakEvenResult {
        contribution_margin_usd: cm,
        contribution_margin_ratio: cm_ratio,
        break_even_units: be_units,
        break_even_revenue_usd: be_revenue,
        units_for_target_profit: target_units,
        revenue_for_target_profit_usd: target_revenue,
        margin_of_safety_units: mos_units,
        margin_of_safety_pct: mos_pct,
        profit_at_expected_usd: profit_at_expected,
        no_break_even: !has_be,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> BreakEvenInput {
        BreakEvenInput {
            fixed_costs_usd: 10_000.0,
            price_per_unit_usd: 50.0,
            variable_cost_per_unit_usd: 30.0,
            target_profit_usd: 0.0,
            expected_units: 0.0,
        }
    }

    #[test]
    fn basic_break_even_units_and_revenue() {
        // CM = 20; 10,000 / 20 = 500 units; 500 × $50 = $25,000 revenue.
        let r = analyze(&base());
        assert!((r.contribution_margin_usd - 20.0).abs() < 1e-9);
        assert!((r.break_even_units.unwrap() - 500.0).abs() < 1e-9);
        assert!((r.break_even_revenue_usd.unwrap() - 25_000.0).abs() < 1e-9);
        assert!(!r.no_break_even);
    }

    #[test]
    fn contribution_margin_ratio_and_revenue_consistency() {
        // CM ratio = 20/50 = 0.40; fixed / ratio = 10,000 / 0.40 = $25,000.
        let r = analyze(&base());
        assert!((r.contribution_margin_ratio.unwrap() - 0.40).abs() < 1e-9);
        let via_ratio = 10_000.0 / r.contribution_margin_ratio.unwrap();
        assert!((r.break_even_revenue_usd.unwrap() - via_ratio).abs() < 1e-6);
    }

    #[test]
    fn target_profit_adds_to_fixed_costs() {
        // (10,000 + 5,000) / 20 = 750 units.
        let r = analyze(&BreakEvenInput { target_profit_usd: 5_000.0, ..base() });
        assert!((r.units_for_target_profit.unwrap() - 750.0).abs() < 1e-9);
        assert!((r.revenue_for_target_profit_usd.unwrap() - 37_500.0).abs() < 1e-9);
    }

    #[test]
    fn margin_of_safety_against_expected_sales() {
        // Expected 800; BE 500 → 300 units of safety = 37.5%; profit = 800×20−10,000 = 6,000.
        let r = analyze(&BreakEvenInput { expected_units: 800.0, ..base() });
        assert!((r.margin_of_safety_units.unwrap() - 300.0).abs() < 1e-9);
        assert!((r.margin_of_safety_pct.unwrap() - 37.5).abs() < 1e-9);
        assert!((r.profit_at_expected_usd.unwrap() - 6_000.0).abs() < 1e-9);
    }

    #[test]
    fn negative_contribution_has_no_break_even() {
        // Price below variable cost: every unit loses money.
        let r = analyze(&BreakEvenInput {
            price_per_unit_usd: 30.0,
            variable_cost_per_unit_usd: 40.0,
            ..base()
        });
        assert!(r.no_break_even);
        assert!(r.break_even_units.is_none());
        assert!(r.units_for_target_profit.is_none());
        assert!((r.contribution_margin_usd - (-10.0)).abs() < 1e-9);
    }

    #[test]
    fn zero_fixed_costs_break_even_at_zero() {
        let r = analyze(&BreakEvenInput { fixed_costs_usd: 0.0, ..base() });
        assert!(r.break_even_units.unwrap().abs() < 1e-9);
        assert!(r.break_even_revenue_usd.unwrap().abs() < 1e-9);
    }

    #[test]
    fn expected_below_break_even_is_negative_safety_and_loss() {
        // Expected 400 < BE 500 → negative safety; profit = 400×20−10,000 = −2,000.
        let r = analyze(&BreakEvenInput { expected_units: 400.0, ..base() });
        assert!((r.margin_of_safety_units.unwrap() - (-100.0)).abs() < 1e-9);
        assert!(r.margin_of_safety_pct.unwrap() < 0.0);
        assert!((r.profit_at_expected_usd.unwrap() - (-2_000.0)).abs() < 1e-9);
    }
}
