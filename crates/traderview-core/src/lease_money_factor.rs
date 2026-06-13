//! Car lease payment — money factor, depreciation fee, and finance fee.
//!
//! A lease payment has two parts plus tax:
//!
//!   * **Depreciation fee** = (capitalized cost − residual value) / term —
//!     you pay for the value the car loses while you drive it.
//!   * **Finance (rent) fee** = (cap cost + residual) × money factor — the
//!     interest. The money factor is a disguised rate: **APR = money factor
//!     × 2400**.
//!   * **Tax** is charged on the monthly payment in most states.
//!
//! Reports the payment breakdown, the equivalent APR, and the total cost of
//! the lease. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LeaseInput {
    /// Capitalized cost — negotiated price + fees − any cap-cost reduction.
    pub cap_cost_usd: f64,
    /// Residual value at lease end.
    pub residual_value_usd: f64,
    pub term_months: f64,
    /// Money factor (e.g. 0.00125 ≈ 3% APR).
    pub money_factor: f64,
    /// Sales tax applied to the monthly payment.
    #[serde(default)]
    pub sales_tax_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LeaseResult {
    pub monthly_depreciation_usd: f64,
    pub monthly_finance_usd: f64,
    /// Depreciation + finance, before tax.
    pub monthly_pre_tax_usd: f64,
    pub monthly_tax_usd: f64,
    /// Full monthly payment including tax.
    pub monthly_payment_usd: f64,
    /// Money factor × 2400 — the equivalent APR.
    pub equivalent_apr_pct: f64,
    /// Monthly payment × term.
    pub total_lease_cost_usd: f64,
}

pub fn analyze(i: &LeaseInput) -> LeaseResult {
    let term = i.term_months.max(0.0);
    let depreciation = i.cap_cost_usd - i.residual_value_usd;
    let monthly_dep = if term > 0.0 { depreciation / term } else { 0.0 };
    let monthly_fin = (i.cap_cost_usd + i.residual_value_usd) * i.money_factor;
    let pre_tax = monthly_dep + monthly_fin;
    let tax = pre_tax * i.sales_tax_pct / 100.0;
    let payment = pre_tax + tax;

    LeaseResult {
        monthly_depreciation_usd: monthly_dep,
        monthly_finance_usd: monthly_fin,
        monthly_pre_tax_usd: pre_tax,
        monthly_tax_usd: tax,
        monthly_payment_usd: payment,
        equivalent_apr_pct: i.money_factor * 2400.0,
        total_lease_cost_usd: payment * term,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> LeaseInput {
        LeaseInput {
            cap_cost_usd: 35_000.0,
            residual_value_usd: 21_000.0,
            term_months: 36.0,
            money_factor: 0.00125,
            sales_tax_pct: 0.0,
        }
    }

    #[test]
    fn monthly_depreciation() {
        // (35k − 21k) / 36 = 388.89.
        let r = analyze(&base());
        assert!((r.monthly_depreciation_usd - 14_000.0 / 36.0).abs() < 1e-6);
    }

    #[test]
    fn finance_fee_uses_sum_times_money_factor() {
        // (35k + 21k) × 0.00125 = 70.
        let r = analyze(&base());
        assert!((r.monthly_finance_usd - 70.0).abs() < 1e-6);
    }

    #[test]
    fn pre_tax_is_dep_plus_finance() {
        let r = analyze(&base());
        assert!((r.monthly_pre_tax_usd - (r.monthly_depreciation_usd + r.monthly_finance_usd)).abs() < 1e-9);
    }

    #[test]
    fn tax_on_payment_and_total() {
        let r = analyze(&LeaseInput { sales_tax_pct: 8.0, ..base() });
        assert!((r.monthly_tax_usd - r.monthly_pre_tax_usd * 0.08).abs() < 1e-9);
        assert!((r.monthly_payment_usd - (r.monthly_pre_tax_usd + r.monthly_tax_usd)).abs() < 1e-9);
    }

    #[test]
    fn equivalent_apr_is_mf_times_2400() {
        // 0.00125 × 2400 = 3%.
        let r = analyze(&base());
        assert!((r.equivalent_apr_pct - 3.0).abs() < 1e-9);
    }

    #[test]
    fn total_cost_is_payment_times_term() {
        let r = analyze(&base());
        assert!((r.total_lease_cost_usd - r.monthly_payment_usd * 36.0).abs() < 1e-6);
    }

    #[test]
    fn higher_money_factor_raises_payment() {
        let low = analyze(&base());
        let high = analyze(&LeaseInput { money_factor: 0.0025, ..base() });
        assert!(high.monthly_payment_usd > low.monthly_payment_usd);
        assert!((high.equivalent_apr_pct - 6.0).abs() < 1e-9);
    }

    #[test]
    fn higher_residual_lowers_payment() {
        // Less depreciation dominates the small finance-fee increase.
        let low_resid = analyze(&LeaseInput { residual_value_usd: 18_000.0, ..base() });
        let high_resid = analyze(&LeaseInput { residual_value_usd: 24_000.0, ..base() });
        assert!(high_resid.monthly_payment_usd < low_resid.monthly_payment_usd);
    }
}
