//! Personal cash-flow statement — GAAP-style operating / investing /
//! financing split.
//!
//! Mirrors the standard accounting cash-flow statement structure
//! applied to a household:
//!
//!   - operating   — net salary, side-hustle income, interest, dividends,
//!                   taxes, mortgage interest (NOT principal), groceries,
//!                   utilities, insurance premiums, every recurring spend
//!   - investing   — 401k / IRA / brokerage contributions (outflow),
//!                   sales of investments (inflow), home down-payment,
//!                   property purchase (outflow)
//!   - financing   — new debt issued (inflow), principal paydowns on
//!                   mortgage / auto / student loans / credit cards
//!                   (outflow), gifts received, large transfers
//!
//! Each input row is `{ name, category ∈ {operating, investing, financing},
//! direction ∈ {inflow, outflow}, amount_usd }`. Compute returns:
//!
//!   - operating_inflows / operating_outflows / operating_net
//!   - investing_inflows / investing_outflows / investing_net
//!   - financing_inflows / financing_outflows / financing_net
//!   - net_change_in_cash = sum of the three nets
//!   - total_inflows / total_outflows
//!   - savings_rate_pct = (operating_inflows + investing_inflows
//!                        − operating_outflows) / total_inflows × 100
//!     (Bogleheads-style: % of gross income that ends up as net positive
//!     after recurring operating outflows. Investing inflows from sales
//!     are excluded from the numerator since they're not new wealth —
//!     only their NET treatment over time matters.)
//!   - status = "surplus" (net_change > 0) | "balanced" (= 0) |
//!              "deficit" (< 0)
//!
//! Pure compute — no DB I/O.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CashFlowRow {
    pub name: String,
    pub category: String,
    pub direction: String,
    pub amount_usd: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CashFlowInput {
    #[serde(default)]
    pub rows: Vec<CashFlowRow>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct SectionTotals {
    pub inflows_usd: f64,
    pub outflows_usd: f64,
    pub net_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CashFlowReport {
    pub operating: SectionTotals,
    pub investing: SectionTotals,
    pub financing: SectionTotals,
    pub total_inflows_usd: f64,
    pub total_outflows_usd: f64,
    pub net_change_in_cash_usd: f64,
    pub savings_rate_pct: Option<f64>,
    pub status: String,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn section_totals(rows: &[CashFlowRow], category: &str) -> SectionTotals {
    let mut s = SectionTotals::default();
    for r in rows {
        if r.category != category {
            continue;
        }
        match r.direction.as_str() {
            "inflow" => s.inflows_usd += r.amount_usd,
            "outflow" => s.outflows_usd += r.amount_usd,
            _ => {}
        }
    }
    s.net_usd = s.inflows_usd - s.outflows_usd;
    s
}

pub fn savings_rate(operating_inflows: f64, operating_outflows: f64) -> Option<f64> {
    if operating_inflows <= 0.0 {
        return None;
    }
    Some((operating_inflows - operating_outflows) / operating_inflows * 100.0)
}

pub fn compute(input: &CashFlowInput) -> CashFlowReport {
    let operating = section_totals(&input.rows, "operating");
    let investing = section_totals(&input.rows, "investing");
    let financing = section_totals(&input.rows, "financing");
    let total_inflows =
        operating.inflows_usd + investing.inflows_usd + financing.inflows_usd;
    let total_outflows =
        operating.outflows_usd + investing.outflows_usd + financing.outflows_usd;
    let net = operating.net_usd + investing.net_usd + financing.net_usd;
    let rate = savings_rate(operating.inflows_usd, operating.outflows_usd);
    let status = if net > 0.0 {
        "surplus"
    } else if net < 0.0 {
        "deficit"
    } else {
        "balanced"
    }
    .to_string();
    CashFlowReport {
        operating,
        investing,
        financing,
        total_inflows_usd: total_inflows,
        total_outflows_usd: total_outflows,
        net_change_in_cash_usd: net,
        savings_rate_pct: rate,
        status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(name: &str, cat: &str, dir: &str, amt: f64) -> CashFlowRow {
        CashFlowRow {
            name: name.into(),
            category: cat.into(),
            direction: dir.into(),
            amount_usd: amt,
        }
    }

    #[test]
    fn section_totals_empty() {
        let s = section_totals(&[], "operating");
        assert_eq!(s.inflows_usd, 0.0);
        assert_eq!(s.outflows_usd, 0.0);
        assert_eq!(s.net_usd, 0.0);
    }

    #[test]
    fn section_totals_basic_operating() {
        let rs = vec![
            row("salary", "operating", "inflow", 5000.0),
            row("rent",   "operating", "outflow", 1500.0),
            row("groc",   "operating", "outflow", 500.0),
            row("brok",   "investing", "outflow", 1000.0),  // ignored
        ];
        let s = section_totals(&rs, "operating");
        assert_eq!(s.inflows_usd, 5000.0);
        assert_eq!(s.outflows_usd, 2000.0);
        assert_eq!(s.net_usd, 3000.0);
    }

    #[test]
    fn section_totals_unknown_direction_ignored() {
        let rs = vec![row("x", "operating", "bogus", 999.0)];
        let s = section_totals(&rs, "operating");
        assert_eq!(s.inflows_usd, 0.0);
        assert_eq!(s.outflows_usd, 0.0);
    }

    #[test]
    fn savings_rate_zero_inflows_is_none() {
        assert!(savings_rate(0.0, 100.0).is_none());
    }

    #[test]
    fn savings_rate_basic_20_pct() {
        // $5000 in, $4000 out = 20% saved.
        let r = savings_rate(5000.0, 4000.0).unwrap();
        assert!((r - 20.0).abs() < 1e-9);
    }

    #[test]
    fn savings_rate_negative_when_overspending() {
        let r = savings_rate(1000.0, 1500.0).unwrap();
        assert!((r + 50.0).abs() < 1e-9);
    }

    #[test]
    fn compute_surplus_basic() {
        let r = compute(&CashFlowInput {
            rows: vec![
                row("salary",     "operating", "inflow",  6000.0),
                row("groceries",  "operating", "outflow",  600.0),
                row("rent",       "operating", "outflow", 1500.0),
                row("utilities",  "operating", "outflow",  200.0),
                row("401k",       "investing", "outflow",  500.0),
                row("mort_prin",  "financing", "outflow",  800.0),
            ],
        });
        assert_eq!(r.operating.inflows_usd, 6000.0);
        assert_eq!(r.operating.outflows_usd, 2300.0);
        assert_eq!(r.operating.net_usd, 3700.0);
        assert_eq!(r.investing.net_usd, -500.0);
        assert_eq!(r.financing.net_usd, -800.0);
        assert_eq!(r.net_change_in_cash_usd, 2400.0);
        assert_eq!(r.status, "surplus");
        assert!((r.savings_rate_pct.unwrap() - 3700.0 / 6000.0 * 100.0).abs() < 1e-9);
    }

    #[test]
    fn compute_deficit_negative_net() {
        let r = compute(&CashFlowInput {
            rows: vec![
                row("salary", "operating", "inflow",  3000.0),
                row("rent",   "operating", "outflow", 2000.0),
                row("food",   "operating", "outflow", 1500.0),
                row("brok",   "investing", "outflow",  500.0),
            ],
        });
        assert_eq!(r.net_change_in_cash_usd, -1000.0);
        assert_eq!(r.status, "deficit");
    }

    #[test]
    fn compute_balanced_exactly_zero() {
        let r = compute(&CashFlowInput {
            rows: vec![
                row("salary",   "operating", "inflow",  1000.0),
                row("spending", "operating", "outflow", 1000.0),
            ],
        });
        assert_eq!(r.net_change_in_cash_usd, 0.0);
        assert_eq!(r.status, "balanced");
    }

    #[test]
    fn compute_total_inflows_outflows_aggregate_all_sections() {
        let r = compute(&CashFlowInput {
            rows: vec![
                row("sal",  "operating", "inflow",  5000.0),
                row("div",  "investing", "inflow",   200.0),
                row("gift", "financing", "inflow",  1000.0),
                row("food", "operating", "outflow",  600.0),
                row("ira",  "investing", "outflow",  500.0),
                row("loan", "financing", "outflow",  300.0),
            ],
        });
        assert_eq!(r.total_inflows_usd, 6200.0);
        assert_eq!(r.total_outflows_usd, 1400.0);
        assert_eq!(r.net_change_in_cash_usd, 4800.0);
    }

    #[test]
    fn compute_savings_rate_none_with_zero_operating_inflows() {
        let r = compute(&CashFlowInput {
            rows: vec![row("brok", "investing", "outflow", 500.0)],
        });
        assert!(r.savings_rate_pct.is_none());
    }
}
