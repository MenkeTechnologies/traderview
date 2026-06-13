//! Free cash flow and the ratios value investors read off it.
//!
//! ```text
//! FCF            = operating cash flow − capital expenditures
//! FCF margin     = FCF / revenue
//! FCF yield      = FCF / market cap
//! FCF / net income = cash-conversion quality (>1 = earnings backed by cash)
//! ```
//!
//! FCF is the cash a business throws off after maintaining and growing its
//! asset base — what's actually available for dividends, buybacks, and debt
//! paydown.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct FcfInput {
    pub operating_cash_flow_usd: f64,
    pub capital_expenditures_usd: f64,
    #[serde(default)]
    pub revenue_usd: f64,
    #[serde(default)]
    pub market_cap_usd: f64,
    #[serde(default)]
    pub net_income_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FcfResult {
    /// operating cash flow − capex.
    pub free_cash_flow_usd: f64,
    /// FCF / revenue, percent; `None` if revenue ≤ 0.
    pub fcf_margin_pct: Option<f64>,
    /// FCF / market cap, percent; `None` if market cap ≤ 0.
    pub fcf_yield_pct: Option<f64>,
    /// FCF / net income; `None` if net income ≤ 0. Above 1 means cash exceeds
    /// reported earnings (high quality).
    pub fcf_to_net_income: Option<f64>,
}

pub fn analyze(input: &FcfInput) -> FcfResult {
    let fcf = input.operating_cash_flow_usd - input.capital_expenditures_usd;

    FcfResult {
        free_cash_flow_usd: fcf,
        fcf_margin_pct: if input.revenue_usd > 0.0 {
            Some(fcf / input.revenue_usd * 100.0)
        } else {
            None
        },
        fcf_yield_pct: if input.market_cap_usd > 0.0 {
            Some(fcf / input.market_cap_usd * 100.0)
        } else {
            None
        },
        fcf_to_net_income: if input.net_income_usd > 0.0 {
            Some(fcf / input.net_income_usd)
        } else {
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn base() -> FcfInput {
        FcfInput {
            operating_cash_flow_usd: 500.0,
            capital_expenditures_usd: 150.0,
            revenue_usd: 2000.0,
            market_cap_usd: 5000.0,
            net_income_usd: 400.0,
        }
    }

    #[test]
    fn fcf_value() {
        assert!(close(analyze(&base()).free_cash_flow_usd, 350.0));
    }

    #[test]
    fn fcf_margin() {
        assert!(close(analyze(&base()).fcf_margin_pct.unwrap(), 17.5));
    }

    #[test]
    fn fcf_yield() {
        assert!(close(analyze(&base()).fcf_yield_pct.unwrap(), 7.0));
    }

    #[test]
    fn fcf_to_net_income() {
        assert!(close(analyze(&base()).fcf_to_net_income.unwrap(), 0.875));
    }

    #[test]
    fn negative_fcf_when_capex_exceeds_ocf() {
        let mut i = base();
        i.capital_expenditures_usd = 700.0;
        assert!(analyze(&i).free_cash_flow_usd < 0.0);
    }

    #[test]
    fn zero_revenue_guards_margin() {
        let mut i = base();
        i.revenue_usd = 0.0;
        assert!(analyze(&i).fcf_margin_pct.is_none());
    }

    #[test]
    fn zero_market_cap_guards_yield() {
        let mut i = base();
        i.market_cap_usd = 0.0;
        assert!(analyze(&i).fcf_yield_pct.is_none());
    }

    #[test]
    fn high_quality_when_fcf_exceeds_earnings() {
        let mut i = base();
        i.operating_cash_flow_usd = 600.0; // FCF 450 > NI 400
        assert!(analyze(&i).fcf_to_net_income.unwrap() > 1.0);
    }
}
