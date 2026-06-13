//! BRRRR — Buy, Rehab, Rent, Refinance, Repeat.
//!
//! The real-estate capital-recycling play: buy a distressed property
//! cheap, rehab it to force appreciation, rent it, then refinance against
//! the higher after-repair value (ARV) and pull your cash back out to do
//! it again. The deal succeeds when the cash-out refinance recovers all
//! the cash you put in — what's left in the deal still earns rent, so the
//! cash-on-cash return goes infinite.
//!
//! This nets the refinance proceeds against the total cash invested to
//! report the cash left in the deal, the post-refi mortgage and cash
//! flow, the equity captured by the rehab, and the cash-on-cash return.
//!
//! Pure compute — the refinance P&I uses the shared
//! [`crate::mortgage_amortization::monthly_payment`].

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct BrrrrInput {
    pub purchase_price_usd: f64,
    pub rehab_cost_usd: f64,
    /// Acquisition closing costs (title, transfer, points on the buy).
    pub purchase_closing_usd: f64,
    /// After-repair value — the appraised value the refinance is based on.
    pub after_repair_value_usd: f64,
    /// Refinance loan-to-value, e.g. 75 for a 75% cash-out refi.
    pub refi_ltv_pct: f64,
    pub refi_apr_pct: f64,
    pub refi_term_months: u32,
    /// Refinance closing costs, netted out of the cash pulled.
    pub refi_closing_usd: f64,
    pub monthly_rent_usd: f64,
    /// All-in monthly operating cost EXCLUDING the mortgage (tax,
    /// insurance, maintenance, vacancy, management).
    pub monthly_operating_usd: f64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct BrrrrResult {
    /// Purchase + rehab + acquisition closing — the cash that goes in.
    pub total_cash_invested_usd: f64,
    /// ARV × refi LTV.
    pub refi_loan_usd: f64,
    /// Loan proceeds net of refinance closing costs — cash back to you.
    pub cash_out_usd: f64,
    /// Cash invested minus cash pulled. Negative = you pulled out more
    /// than you put in (and still own the asset).
    pub cash_left_in_deal_usd: f64,
    /// True when the refinance recovered all the invested cash.
    pub all_cash_recovered: bool,
    pub monthly_pi_usd: f64,
    /// Rent − operating − mortgage P&I.
    pub monthly_cash_flow_usd: f64,
    pub annual_cash_flow_usd: f64,
    /// ARV − refi loan: the equity you keep after pulling cash.
    pub equity_after_refi_usd: f64,
    /// Annual cash flow ÷ cash left in the deal. `None` when no cash is
    /// left (the return is infinite — the BRRRR ideal).
    pub cash_on_cash_pct: Option<f64>,
}

pub fn compute(i: &BrrrrInput) -> BrrrrResult {
    let total_cash_invested =
        i.purchase_price_usd + i.rehab_cost_usd + i.purchase_closing_usd;
    let refi_loan = i.after_repair_value_usd * (i.refi_ltv_pct / 100.0);
    let cash_out = refi_loan - i.refi_closing_usd;
    let cash_left = total_cash_invested - cash_out;

    let monthly_pi =
        crate::mortgage_amortization::monthly_payment(refi_loan, i.refi_apr_pct, i.refi_term_months);
    let monthly_cash_flow = i.monthly_rent_usd - i.monthly_operating_usd - monthly_pi;

    let cash_on_cash_pct = if cash_left > 0.0 {
        Some(monthly_cash_flow * 12.0 / cash_left * 100.0)
    } else {
        None
    };

    BrrrrResult {
        total_cash_invested_usd: total_cash_invested,
        refi_loan_usd: refi_loan,
        cash_out_usd: cash_out,
        cash_left_in_deal_usd: cash_left,
        all_cash_recovered: cash_left <= 0.0,
        monthly_pi_usd: monthly_pi,
        monthly_cash_flow_usd: monthly_cash_flow,
        annual_cash_flow_usd: monthly_cash_flow * 12.0,
        equity_after_refi_usd: i.after_repair_value_usd - refi_loan,
        cash_on_cash_pct,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Buy $100k + $30k rehab + $5k closing = $135k cash in.
    fn deal() -> BrrrrInput {
        BrrrrInput {
            purchase_price_usd: 100_000.0,
            rehab_cost_usd: 30_000.0,
            purchase_closing_usd: 5_000.0,
            after_repair_value_usd: 200_000.0,
            refi_ltv_pct: 75.0,
            refi_apr_pct: 7.0,
            refi_term_months: 360,
            refi_closing_usd: 4_000.0,
            monthly_rent_usd: 1_800.0,
            monthly_operating_usd: 600.0,
        }
    }

    #[test]
    fn successful_brrrr_recovers_all_cash() {
        // ARV $200k × 75% = $150k loan; net of $4k closing = $146k out
        // vs $135k in → all cash recovered, $11k pulled beyond basis.
        let r = compute(&deal());
        assert!((r.total_cash_invested_usd - 135_000.0).abs() < 1e-6);
        assert!((r.refi_loan_usd - 150_000.0).abs() < 1e-6);
        assert!((r.cash_out_usd - 146_000.0).abs() < 1e-6);
        assert!((r.cash_left_in_deal_usd - (-11_000.0)).abs() < 1e-6);
        assert!(r.all_cash_recovered);
        assert_eq!(r.cash_on_cash_pct, None); // infinite return
        assert!((r.equity_after_refi_usd - 50_000.0).abs() < 1e-6);
    }

    #[test]
    fn partial_brrrr_leaves_cash_and_has_finite_coc() {
        // Lower ARV → smaller loan → cash stays in the deal.
        let r = compute(&BrrrrInput { after_repair_value_usd: 160_000.0, ..deal() });
        // $160k × 75% = $120k loan; $116k out vs $135k in → $19k left.
        assert!((r.cash_left_in_deal_usd - 19_000.0).abs() < 1e-6);
        assert!(!r.all_cash_recovered);
        let coc = r.cash_on_cash_pct.expect("finite when cash is left");
        let expect = r.annual_cash_flow_usd / 19_000.0 * 100.0;
        assert!((coc - expect).abs() < 1e-9);
    }

    #[test]
    fn pi_and_cash_flow_use_shared_amortization() {
        let r = compute(&deal());
        let pi = crate::mortgage_amortization::monthly_payment(150_000.0, 7.0, 360);
        assert!((r.monthly_pi_usd - pi).abs() < 1e-9);
        assert!((r.monthly_cash_flow_usd - (1_800.0 - 600.0 - pi)).abs() < 1e-9);
        assert!((r.annual_cash_flow_usd - r.monthly_cash_flow_usd * 12.0).abs() < 1e-6);
    }

    #[test]
    fn equity_is_arv_minus_loan() {
        let r = compute(&BrrrrInput { refi_ltv_pct: 70.0, ..deal() });
        // $200k − $140k loan = $60k equity.
        assert!((r.equity_after_refi_usd - 60_000.0).abs() < 1e-6);
    }
}
