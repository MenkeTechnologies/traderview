//! Cash flow statement (indirect method) — reconciles net income to operating
//! cash flow by adding back non-cash charges and adjusting for changes in working
//! capital (an increase in a current asset uses cash; an increase in a current
//! liability provides cash), then adds the investing and financing sections to
//! reach the net change in cash and the ending balance. Distinct from the
//! free-cash-flow module, which derives FCF from a given operating cash flow;
//! this derives operating cash flow itself. Drafting aid, not accounting advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CashFlowInput {
    pub company_name: String,
    pub period_label: String,
    pub net_income_usd: f64,
    /// Depreciation and amortization (non-cash add-back).
    #[serde(default)]
    pub depreciation_amortization_usd: f64,
    /// Increase (+) or decrease (−) in accounts receivable.
    #[serde(default)]
    pub change_in_ar_usd: f64,
    /// Increase (+) or decrease (−) in inventory.
    #[serde(default)]
    pub change_in_inventory_usd: f64,
    /// Increase (+) or decrease (−) in accounts payable.
    #[serde(default)]
    pub change_in_ap_usd: f64,
    /// Capital expenditures (cash outflow).
    #[serde(default)]
    pub capex_usd: f64,
    /// Proceeds from selling assets.
    #[serde(default)]
    pub asset_sales_usd: f64,
    /// Debt raised.
    #[serde(default)]
    pub debt_issued_usd: f64,
    /// Debt repaid.
    #[serde(default)]
    pub debt_repaid_usd: f64,
    /// Equity raised.
    #[serde(default)]
    pub equity_issued_usd: f64,
    /// Dividends paid.
    #[serde(default)]
    pub dividends_usd: f64,
    /// Beginning cash balance.
    #[serde(default)]
    pub beginning_cash_usd: f64,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DocClause {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CashFlowStatement {
    pub title: String,
    /// Cash flow from operating activities.
    pub operating_cash_flow_usd: f64,
    /// Cash flow from investing activities.
    pub investing_cash_flow_usd: f64,
    /// Cash flow from financing activities.
    pub financing_cash_flow_usd: f64,
    /// CFO + CFI + CFF.
    pub net_change_usd: f64,
    pub ending_cash_usd: f64,
    pub clauses: Vec<DocClause>,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

/// Format a signed working-capital effect: an increase in an asset is shown as a
/// use of cash (negative).
fn signed(v: f64) -> String {
    if v < 0.0 {
        format!("({})", money(v.abs()))
    } else {
        money(v)
    }
}

pub fn generate(i: &CashFlowInput) -> CashFlowStatement {
    // Increase in a current asset uses cash; increase in a current liability provides cash.
    let cfo = cents(
        i.net_income_usd + i.depreciation_amortization_usd - i.change_in_ar_usd
            - i.change_in_inventory_usd
            + i.change_in_ap_usd,
    );
    let cfi = cents(-i.capex_usd + i.asset_sales_usd);
    let cff = cents(
        i.debt_issued_usd - i.debt_repaid_usd + i.equity_issued_usd - i.dividends_usd,
    );
    let net = cents(cfo + cfi + cff);
    let ending = cents(i.beginning_cash_usd + net);

    let operating_body = format!(
        "Net income {}\n  + Depreciation & amortization {}\n  − Increase in AR {}\n  − Increase in inventory {}\n  + Increase in AP {}\n  = Operating cash flow {}",
        money(i.net_income_usd),
        money(i.depreciation_amortization_usd),
        signed(-i.change_in_ar_usd),
        signed(-i.change_in_inventory_usd),
        signed(i.change_in_ap_usd),
        money(cfo)
    );

    let investing_body = format!(
        "Capital expenditures {}\n  + Asset sales {}\n  = Investing cash flow {}",
        signed(-i.capex_usd),
        money(i.asset_sales_usd),
        money(cfi)
    );

    let financing_body = format!(
        "Debt issued {}\n  − Debt repaid {}\n  + Equity issued {}\n  − Dividends {}\n  = Financing cash flow {}",
        money(i.debt_issued_usd),
        signed(-i.debt_repaid_usd),
        money(i.equity_issued_usd),
        signed(-i.dividends_usd),
        money(cff)
    );

    let clauses = vec![
        DocClause {
            heading: "Statement".into(),
            body: format!("Company: {}\nPeriod: {}\nDate: {}", i.company_name, i.period_label, i.date),
        },
        DocClause { heading: "Operating Activities".into(), body: operating_body },
        DocClause { heading: "Investing Activities".into(), body: investing_body },
        DocClause { heading: "Financing Activities".into(), body: financing_body },
        DocClause {
            heading: "Net Change in Cash".into(),
            body: format!(
                "Net change {} = operating {} + investing {} + financing {}. Beginning cash {} → ending cash {}.",
                money(net), money(cfo), money(cfi), money(cff), money(i.beginning_cash_usd), money(ending)
            ),
        },
    ];

    CashFlowStatement {
        title: "Cash Flow Statement (Indirect Method)".into(),
        operating_cash_flow_usd: cfo,
        investing_cash_flow_usd: cfi,
        financing_cash_flow_usd: cff,
        net_change_usd: net,
        ending_cash_usd: ending,
        clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> CashFlowInput {
        CashFlowInput {
            company_name: "Acme Co".into(),
            period_label: "FY2025".into(),
            net_income_usd: 100_000.0,
            depreciation_amortization_usd: 20_000.0,
            change_in_ar_usd: 15_000.0,
            change_in_inventory_usd: 10_000.0,
            change_in_ap_usd: 5_000.0,
            capex_usd: 30_000.0,
            asset_sales_usd: 0.0,
            debt_issued_usd: 0.0,
            debt_repaid_usd: 10_000.0,
            equity_issued_usd: 0.0,
            dividends_usd: 8_000.0,
            beginning_cash_usd: 40_000.0,
            date: "2026-01-31".into(),
        }
    }

    #[test]
    fn full_statement() {
        let d = generate(&base());
        assert!(close(d.operating_cash_flow_usd, 100_000.0));
        assert!(close(d.investing_cash_flow_usd, -30_000.0));
        assert!(close(d.financing_cash_flow_usd, -18_000.0));
        assert!(close(d.net_change_usd, 52_000.0));
        assert!(close(d.ending_cash_usd, 92_000.0));
    }

    #[test]
    fn ar_decrease_is_a_source() {
        let d = generate(&CashFlowInput {
            net_income_usd: 50_000.0,
            depreciation_amortization_usd: 5_000.0,
            change_in_ar_usd: -3_000.0, // AR fell → cash collected
            change_in_inventory_usd: 0.0,
            change_in_ap_usd: 0.0,
            capex_usd: 0.0,
            debt_repaid_usd: 0.0,
            dividends_usd: 0.0,
            beginning_cash_usd: 10_000.0,
            ..base()
        });
        assert!(close(d.operating_cash_flow_usd, 58_000.0));
        assert!(close(d.ending_cash_usd, 68_000.0));
    }

    #[test]
    fn sections_sum_to_net_change() {
        let d = generate(&base());
        assert!(close(
            d.operating_cash_flow_usd + d.investing_cash_flow_usd + d.financing_cash_flow_usd,
            d.net_change_usd
        ));
    }

    #[test]
    fn ending_equals_beginning_plus_change() {
        let d = generate(&base());
        assert!(close(d.ending_cash_usd, 40_000.0 + d.net_change_usd));
    }

    #[test]
    fn all_three_sections_present() {
        let d = generate(&base());
        assert!(d.clauses.iter().any(|c| c.heading == "Operating Activities"));
        assert!(d.clauses.iter().any(|c| c.heading == "Investing Activities"));
        assert!(d.clauses.iter().any(|c| c.heading == "Financing Activities"));
    }
}
