//! Yield on cost — what a growing dividend yields on your original purchase.
//!
//! Current yield measures the dividend against today's price; **yield on
//! cost** (YOC) measures it against what *you* paid. For a dividend-growth
//! holding, YOC climbs every year the dividend is raised, so a position
//! bought cheaply years ago can yield double-digits on cost even while its
//! current yield looks modest.
//!
//!   * YOC = current annual dividend / cost basis
//!   * current yield = current annual dividend / current price
//!   * projected YOC = (dividend × (1+growth)^years) / cost basis
//!   * the dividend doubles in ln(2)/ln(1+growth) years
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct YieldOnCostInput {
    pub cost_basis_per_share_usd: f64,
    pub current_annual_dividend_usd: f64,
    /// Current share price (0 ⇒ skip the current-yield comparison).
    #[serde(default)]
    pub current_price_per_share_usd: f64,
    /// Expected annual dividend growth rate.
    #[serde(default)]
    pub dividend_growth_pct: f64,
    /// Projection horizon for the future YOC.
    #[serde(default)]
    pub years: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct YieldOnCostResult {
    pub yield_on_cost_pct: f64,
    /// Dividend / current price; `None` if no price supplied.
    pub current_yield_pct: Option<f64>,
    /// Dividend grown for `years` at the growth rate.
    pub projected_dividend_usd: f64,
    /// YOC after the projection.
    pub projected_yield_on_cost_pct: f64,
    /// Years for the dividend to double at the growth rate; `None` if growth ≤ 0.
    pub years_to_double_dividend: Option<f64>,
}

pub fn analyze(i: &YieldOnCostInput) -> YieldOnCostResult {
    let basis = i.cost_basis_per_share_usd;
    let div = i.current_annual_dividend_usd;
    let g = i.dividend_growth_pct / 100.0;

    let yoc = if basis > 0.0 { div / basis * 100.0 } else { 0.0 };
    let current_yield = if i.current_price_per_share_usd > 0.0 {
        Some(div / i.current_price_per_share_usd * 100.0)
    } else {
        None
    };

    let projected_div = div * (1.0 + g).powf(i.years.max(0.0));
    let projected_yoc = if basis > 0.0 { projected_div / basis * 100.0 } else { 0.0 };
    let doubling = if g > 0.0 { Some((2.0_f64).ln() / (1.0 + g).ln()) } else { None };

    YieldOnCostResult {
        yield_on_cost_pct: yoc,
        current_yield_pct: current_yield,
        projected_dividend_usd: projected_div,
        projected_yield_on_cost_pct: projected_yoc,
        years_to_double_dividend: doubling,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> YieldOnCostInput {
        YieldOnCostInput {
            cost_basis_per_share_usd: 40.0,
            current_annual_dividend_usd: 2.0,
            current_price_per_share_usd: 80.0,
            dividend_growth_pct: 7.0,
            years: 10.0,
        }
    }

    #[test]
    fn yield_on_cost_is_div_over_basis() {
        // 2 / 40 = 5%.
        let r = analyze(&base());
        assert!((r.yield_on_cost_pct - 5.0).abs() < 1e-9);
    }

    #[test]
    fn current_yield_is_div_over_price() {
        // 2 / 80 = 2.5%.
        let r = analyze(&base());
        assert!((r.current_yield_pct.unwrap() - 2.5).abs() < 1e-9);
    }

    #[test]
    fn yoc_exceeds_current_yield_when_appreciated() {
        // Bought at 40, now 80 → YOC 5% > current yield 2.5%.
        let r = analyze(&base());
        assert!(r.yield_on_cost_pct > r.current_yield_pct.unwrap());
    }

    #[test]
    fn projected_dividend_grows() {
        // 2 × 1.07^10.
        let r = analyze(&base());
        assert!((r.projected_dividend_usd - 2.0 * 1.07_f64.powi(10)).abs() < 1e-6);
    }

    #[test]
    fn projected_yoc_uses_grown_dividend() {
        let r = analyze(&base());
        assert!((r.projected_yield_on_cost_pct - r.projected_dividend_usd / 40.0 * 100.0).abs() < 1e-9);
        assert!(r.projected_yield_on_cost_pct > r.yield_on_cost_pct);
    }

    #[test]
    fn doubling_time_is_log_based() {
        // ln2 / ln(1.07) ≈ 10.24 years.
        let r = analyze(&base());
        let expected = 2.0_f64.ln() / 1.07_f64.ln();
        assert!((r.years_to_double_dividend.unwrap() - expected).abs() < 1e-9);
    }

    #[test]
    fn no_growth_projected_equals_current_and_no_doubling() {
        let r = analyze(&YieldOnCostInput { dividend_growth_pct: 0.0, ..base() });
        assert!((r.projected_dividend_usd - 2.0).abs() < 1e-9);
        assert!(r.years_to_double_dividend.is_none());
    }

    #[test]
    fn no_price_skips_current_yield() {
        let r = analyze(&YieldOnCostInput { current_price_per_share_usd: 0.0, ..base() });
        assert!(r.current_yield_pct.is_none());
        assert!((r.yield_on_cost_pct - 5.0).abs() < 1e-9); // YOC still computed
    }
}
