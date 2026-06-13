//! Small-business valuation by Seller's Discretionary Earnings (SDE).
//!
//! For owner-operated businesses, buyers value the cash flow available to a
//! single working owner — net income with owner compensation and discretionary
//! items added back — times an industry multiple (typically ~2–4×).
//!
//! ```text
//! SDE = net income + owner comp + depreciation/amort + interest + add-backs
//! business value = SDE × multiple
//! ```
//!
//! This is the small-business analog of `ev-ebitda` for larger firms.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SdeInput {
    pub net_income_usd: f64,
    /// Owner's salary/compensation (added back — a buyer sets their own).
    pub owner_compensation_usd: f64,
    #[serde(default)]
    pub depreciation_amortization_usd: f64,
    #[serde(default)]
    pub interest_usd: f64,
    /// Discretionary add-backs (owner perks, personal expenses, one-time items).
    #[serde(default)]
    pub discretionary_addbacks_usd: f64,
    /// Industry SDE multiple.
    pub sde_multiple: f64,
    /// Revenue, for the SDE margin. Optional.
    #[serde(default)]
    pub revenue_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SdeResult {
    /// Seller's Discretionary Earnings.
    pub sde_usd: f64,
    /// SDE × multiple.
    pub business_value_usd: f64,
    /// SDE / revenue, percent; `None` if revenue ≤ 0.
    pub sde_margin_pct: Option<f64>,
    /// Total add-backs over net income (owner comp + D&A + interest + other).
    pub total_addbacks_usd: f64,
}

pub fn analyze(input: &SdeInput) -> SdeResult {
    let addbacks = input.owner_compensation_usd
        + input.depreciation_amortization_usd
        + input.interest_usd
        + input.discretionary_addbacks_usd;
    let sde = input.net_income_usd + addbacks;

    SdeResult {
        sde_usd: sde,
        business_value_usd: sde * input.sde_multiple,
        sde_margin_pct: if input.revenue_usd > 0.0 {
            Some(sde / input.revenue_usd * 100.0)
        } else {
            None
        },
        total_addbacks_usd: addbacks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn base() -> SdeInput {
        SdeInput {
            net_income_usd: 100_000.0,
            owner_compensation_usd: 80_000.0,
            depreciation_amortization_usd: 15_000.0,
            interest_usd: 5_000.0,
            discretionary_addbacks_usd: 10_000.0,
            sde_multiple: 2.5,
            revenue_usd: 600_000.0,
        }
    }

    #[test]
    fn sde() {
        // 100k + 80k + 15k + 5k + 10k = 210,000.
        assert!(close(analyze(&base()).sde_usd, 210_000.0));
    }

    #[test]
    fn business_value() {
        // 210,000 × 2.5 = 525,000.
        assert!(close(analyze(&base()).business_value_usd, 525_000.0));
    }

    #[test]
    fn total_addbacks() {
        assert!(close(analyze(&base()).total_addbacks_usd, 110_000.0));
    }

    #[test]
    fn sde_margin() {
        // 210,000 / 600,000 = 35%.
        assert!(close(analyze(&base()).sde_margin_pct.unwrap(), 35.0));
    }

    #[test]
    fn higher_multiple_higher_value() {
        let low = analyze(&base());
        let high = analyze(&SdeInput {
            sde_multiple: 4.0,
            ..base()
        });
        assert!(high.business_value_usd > low.business_value_usd);
    }

    #[test]
    fn owner_comp_drives_addbacks() {
        let r = analyze(&SdeInput {
            owner_compensation_usd: 120_000.0,
            ..base()
        });
        assert!(close(r.sde_usd, 250_000.0));
    }

    #[test]
    fn zero_revenue_no_margin() {
        let r = analyze(&SdeInput {
            revenue_usd: 0.0,
            ..base()
        });
        assert!(r.sde_margin_pct.is_none());
    }

    #[test]
    fn no_addbacks_sde_equals_net_income() {
        let r = analyze(&SdeInput {
            owner_compensation_usd: 0.0,
            depreciation_amortization_usd: 0.0,
            interest_usd: 0.0,
            discretionary_addbacks_usd: 0.0,
            ..base()
        });
        assert!(close(r.sde_usd, 100_000.0));
    }
}
