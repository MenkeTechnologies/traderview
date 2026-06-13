//! Enterprise value and the EV/EBITDA multiple.
//!
//! Enterprise value is what it would cost to buy the whole business — equity
//! plus the debt you assume, less the cash you get:
//!
//! ```text
//! EV = market cap + total debt + preferred equity + minority interest − cash
//! ```
//!
//! EV/EBITDA is the capital-structure-neutral valuation multiple (unlike P/E it
//! ignores how the firm is financed), and EV/Sales and the EBITDA margin round
//! out the picture when revenue is supplied.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct EvEbitdaInput {
    /// Equity market capitalization.
    pub market_cap_usd: f64,
    /// Total debt (short- and long-term).
    pub total_debt_usd: f64,
    /// Cash and cash equivalents.
    pub cash_usd: f64,
    /// EBITDA (earnings before interest, taxes, depreciation, amortization).
    pub ebitda_usd: f64,
    /// Preferred equity, if any.
    #[serde(default)]
    pub preferred_equity_usd: f64,
    /// Minority (non-controlling) interest, if any.
    #[serde(default)]
    pub minority_interest_usd: f64,
    /// Revenue, for the EV/Sales multiple and EBITDA margin. Optional.
    #[serde(default)]
    pub revenue_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct EvEbitdaResult {
    /// market cap + debt + preferred + minority − cash.
    pub enterprise_value_usd: f64,
    /// total debt − cash.
    pub net_debt_usd: f64,
    /// EV / EBITDA; `None` if EBITDA is not positive.
    pub ev_ebitda: Option<f64>,
    /// EV / revenue; `None` if revenue is not positive.
    pub ev_sales: Option<f64>,
    /// EBITDA / revenue, percent; `None` if revenue is not positive.
    pub ebitda_margin_pct: Option<f64>,
}

pub fn analyze(input: &EvEbitdaInput) -> EvEbitdaResult {
    let ev = input.market_cap_usd + input.total_debt_usd + input.preferred_equity_usd
        + input.minority_interest_usd
        - input.cash_usd;

    let ev_ebitda = if input.ebitda_usd > 0.0 {
        Some(ev / input.ebitda_usd)
    } else {
        None
    };
    let ev_sales = if input.revenue_usd > 0.0 {
        Some(ev / input.revenue_usd)
    } else {
        None
    };
    let ebitda_margin_pct = if input.revenue_usd > 0.0 {
        Some(input.ebitda_usd / input.revenue_usd * 100.0)
    } else {
        None
    };

    EvEbitdaResult {
        enterprise_value_usd: ev,
        net_debt_usd: input.total_debt_usd - input.cash_usd,
        ev_ebitda,
        ev_sales,
        ebitda_margin_pct,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn run(mcap: f64, debt: f64, cash: f64, ebitda: f64) -> EvEbitdaResult {
        analyze(&EvEbitdaInput {
            market_cap_usd: mcap,
            total_debt_usd: debt,
            cash_usd: cash,
            ebitda_usd: ebitda,
            preferred_equity_usd: 0.0,
            minority_interest_usd: 0.0,
            revenue_usd: 0.0,
        })
    }

    #[test]
    fn enterprise_value_basic() {
        // 1000 + 300 − 100 = 1200.
        let r = run(1000.0, 300.0, 100.0, 150.0);
        assert!(close(r.enterprise_value_usd, 1200.0));
    }

    #[test]
    fn enterprise_value_with_preferred_and_minority() {
        let r = analyze(&EvEbitdaInput {
            market_cap_usd: 1000.0,
            total_debt_usd: 300.0,
            cash_usd: 100.0,
            ebitda_usd: 150.0,
            preferred_equity_usd: 50.0,
            minority_interest_usd: 20.0,
            revenue_usd: 0.0,
        });
        assert!(close(r.enterprise_value_usd, 1270.0));
    }

    #[test]
    fn ev_ebitda_multiple() {
        // 1200 / 150 = 8.0.
        let r = run(1000.0, 300.0, 100.0, 150.0);
        assert!(close(r.ev_ebitda.unwrap(), 8.0));
    }

    #[test]
    fn net_debt() {
        let r = run(1000.0, 300.0, 100.0, 150.0);
        assert!(close(r.net_debt_usd, 200.0));
    }

    #[test]
    fn negative_net_debt_when_cash_rich() {
        let r = run(1000.0, 100.0, 400.0, 150.0);
        assert!(close(r.net_debt_usd, -300.0));
        // Cash-rich firm: EV below market cap.
        assert!(r.enterprise_value_usd < 1000.0);
    }

    #[test]
    fn negative_ebitda_has_no_multiple() {
        let r = run(1000.0, 300.0, 100.0, -50.0);
        assert!(r.ev_ebitda.is_none());
    }

    #[test]
    fn ev_sales_and_margin_with_revenue() {
        let r = analyze(&EvEbitdaInput {
            market_cap_usd: 1000.0,
            total_debt_usd: 300.0,
            cash_usd: 100.0,
            ebitda_usd: 150.0,
            preferred_equity_usd: 0.0,
            minority_interest_usd: 0.0,
            revenue_usd: 600.0,
        });
        // EV 1200 / 600 = 2.0; margin 150/600 = 25%.
        assert!(close(r.ev_sales.unwrap(), 2.0));
        assert!(close(r.ebitda_margin_pct.unwrap(), 25.0));
    }

    #[test]
    fn no_revenue_no_sales_metrics() {
        let r = run(1000.0, 300.0, 100.0, 150.0);
        assert!(r.ev_sales.is_none());
        assert!(r.ebitda_margin_pct.is_none());
    }
}
