//! DuPont analysis — decomposes return on equity into the operating, asset-use,
//! and leverage levers that drive it. The five-step form:
//!
//! ```text
//! ROE = tax burden × interest burden × operating margin
//!       × asset turnover × equity multiplier
//!     = (NI/EBT) × (EBT/EBIT) × (EBIT/Rev) × (Rev/Assets) × (Assets/Equity)
//!     = NI / Equity
//! ```
//!
//! The first three factors collapse to net profit margin (NI/Rev), so the
//! three-step form is margin × turnover × leverage. Seeing the factors apart
//! shows whether ROE comes from profitability, efficiency, or debt.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct DupontInput {
    pub net_income_usd: f64,
    /// Earnings before tax (pre-tax income).
    pub pretax_income_usd: f64,
    /// Earnings before interest and taxes.
    pub ebit_usd: f64,
    pub revenue_usd: f64,
    pub total_assets_usd: f64,
    pub total_equity_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DupontResult {
    /// NI / EBT — fraction of pre-tax income kept after tax.
    pub tax_burden: Option<f64>,
    /// EBT / EBIT — fraction of operating income left after interest.
    pub interest_burden: Option<f64>,
    /// EBIT / revenue, percent.
    pub operating_margin_pct: Option<f64>,
    /// Revenue / assets.
    pub asset_turnover: Option<f64>,
    /// Assets / equity — the leverage lever.
    pub equity_multiplier: Option<f64>,
    /// NI / revenue, percent (the three-step margin factor).
    pub net_profit_margin_pct: Option<f64>,
    /// NI / equity, percent.
    pub roe_pct: Option<f64>,
}

fn ratio(num: f64, den: f64) -> Option<f64> {
    if den != 0.0 {
        Some(num / den)
    } else {
        None
    }
}

pub fn analyze(input: &DupontInput) -> DupontResult {
    DupontResult {
        tax_burden: ratio(input.net_income_usd, input.pretax_income_usd),
        interest_burden: ratio(input.pretax_income_usd, input.ebit_usd),
        operating_margin_pct: ratio(input.ebit_usd, input.revenue_usd).map(|r| r * 100.0),
        asset_turnover: ratio(input.revenue_usd, input.total_assets_usd),
        equity_multiplier: ratio(input.total_assets_usd, input.total_equity_usd),
        net_profit_margin_pct: ratio(input.net_income_usd, input.revenue_usd).map(|r| r * 100.0),
        roe_pct: ratio(input.net_income_usd, input.total_equity_usd).map(|r| r * 100.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn base() -> DupontInput {
        DupontInput {
            net_income_usd: 120.0,
            pretax_income_usd: 150.0,
            ebit_usd: 200.0,
            revenue_usd: 1000.0,
            total_assets_usd: 800.0,
            total_equity_usd: 400.0,
        }
    }

    #[test]
    fn tax_burden() {
        assert!(close(analyze(&base()).tax_burden.unwrap(), 0.8));
    }

    #[test]
    fn interest_burden() {
        assert!(close(analyze(&base()).interest_burden.unwrap(), 0.75));
    }

    #[test]
    fn operating_margin() {
        assert!(close(analyze(&base()).operating_margin_pct.unwrap(), 20.0));
    }

    #[test]
    fn asset_turnover() {
        assert!(close(analyze(&base()).asset_turnover.unwrap(), 1.25));
    }

    #[test]
    fn equity_multiplier() {
        assert!(close(analyze(&base()).equity_multiplier.unwrap(), 2.0));
    }

    #[test]
    fn five_factors_multiply_to_roe() {
        let r = analyze(&base());
        let product = r.tax_burden.unwrap()
            * r.interest_burden.unwrap()
            * (r.operating_margin_pct.unwrap() / 100.0)
            * r.asset_turnover.unwrap()
            * r.equity_multiplier.unwrap();
        // 0.8 × 0.75 × 0.2 × 1.25 × 2.0 = 0.30 → ROE 30%.
        assert!(close(product * 100.0, 30.0));
        assert!(close(r.roe_pct.unwrap(), 30.0));
    }

    #[test]
    fn first_three_factors_are_net_margin() {
        let r = analyze(&base());
        let three = r.tax_burden.unwrap()
            * r.interest_burden.unwrap()
            * (r.operating_margin_pct.unwrap() / 100.0);
        assert!(close(three * 100.0, r.net_profit_margin_pct.unwrap()));
        assert!(close(r.net_profit_margin_pct.unwrap(), 12.0));
    }

    #[test]
    fn zero_denominators_guard() {
        let r = analyze(&DupontInput {
            net_income_usd: 120.0,
            pretax_income_usd: 0.0,
            ebit_usd: 0.0,
            revenue_usd: 0.0,
            total_assets_usd: 0.0,
            total_equity_usd: 0.0,
        });
        assert!(r.tax_burden.is_none());
        assert!(r.operating_margin_pct.is_none());
        assert!(r.roe_pct.is_none());
    }
}
