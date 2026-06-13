//! Fix-and-flip underwriting — the 70% rule + full deal P&L.
//!
//! Two questions every house flipper asks:
//!
//!   * **What's the most I can pay?** The **70% rule**: max allowable offer
//!     = after-repair value × 70% − repair costs. The 30% haircut is the
//!     flipper's margin for holding, financing, selling costs, and profit.
//!     (The percentage is configurable; 70 is the convention.)
//!   * **What do I actually make?** Net profit = sale proceeds (ARV less
//!     selling costs) − every cost: purchase, rehab, closing, the monthly
//!     carry over the hold, and the financing (points + interest on the
//!     loan). Cash-on-cash divides that profit by the cash you actually put
//!     in (all-in cost less the loan), then annualizes by the hold length.
//!
//! Profit is independent of how the deal is financed except for the
//! financing *costs* — the loan only changes how much of your own cash is
//! tied up, which is why a smaller down payment lifts cash-on-cash return.
//! Distinct from BRRRR (which refinances and holds as a rental); this sells.
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct FlipInput {
    /// After-repair value — what the rehabbed house sells for.
    pub arv_usd: f64,
    pub purchase_price_usd: f64,
    pub rehab_cost_usd: f64,
    /// The rule percentage applied to ARV (default convention 70).
    pub rule_pct: f64,
    /// Months held from purchase to sale.
    pub holding_months: f64,
    /// Monthly carry: taxes, insurance, utilities, lawn, etc.
    pub monthly_holding_cost_usd: f64,
    /// Closing costs paid at purchase.
    pub buying_closing_cost_usd: f64,
    /// Loan principal (financed portion); the rest of the all-in cost is cash.
    pub loan_amount_usd: f64,
    /// Loan points (origination), percent of the loan.
    pub financing_points_pct: f64,
    /// Annual interest rate on the loan (e.g. hard money).
    pub annual_interest_rate_pct: f64,
    /// Selling costs as a percent of ARV (agent commission + closing).
    pub selling_cost_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FlipResult {
    /// ARV × rule% − rehab: the most you should pay under the rule.
    pub max_allowable_offer_usd: f64,
    /// True when the purchase price is at or below the MAO.
    pub passes_rule: bool,
    pub financing_points_usd: f64,
    pub interest_cost_usd: f64,
    pub holding_cost_usd: f64,
    pub selling_cost_usd: f64,
    /// Every cost: purchase + rehab + closing + holding + points + interest.
    pub total_project_cost_usd: f64,
    /// ARV − selling costs.
    pub net_sale_proceeds_usd: f64,
    pub net_profit_usd: f64,
    /// Your own cash in the deal (all-in cost − loan).
    pub cash_invested_usd: f64,
    /// Net profit as a percent of cash invested.
    pub cash_on_cash_pct: f64,
    /// Cash-on-cash scaled to a 12-month basis; `None` when hold is 0.
    pub annualized_roi_pct: Option<f64>,
    /// Net profit as a percent of ARV.
    pub profit_margin_pct: f64,
}

pub fn analyze(i: &FlipInput) -> FlipResult {
    let arv = i.arv_usd.max(0.0);
    let loan = i.loan_amount_usd.max(0.0);

    let max_allowable_offer = arv * i.rule_pct / 100.0 - i.rehab_cost_usd;
    let passes_rule = i.purchase_price_usd <= max_allowable_offer;

    let points = loan * i.financing_points_pct / 100.0;
    let interest = loan * (i.annual_interest_rate_pct / 100.0) * (i.holding_months / 12.0);
    let holding = i.monthly_holding_cost_usd * i.holding_months;
    let selling = arv * i.selling_cost_pct / 100.0;

    let total_cost = i.purchase_price_usd
        + i.rehab_cost_usd
        + i.buying_closing_cost_usd
        + holding
        + points
        + interest;
    let net_proceeds = arv - selling;
    let net_profit = net_proceeds - total_cost;

    // Cash you front: everything the loan didn't cover.
    let cash_invested = (total_cost - loan).max(0.0);
    let cash_on_cash = if cash_invested > 0.0 {
        net_profit / cash_invested * 100.0
    } else {
        0.0
    };
    let annualized_roi = if i.holding_months > 0.0 {
        Some(cash_on_cash * 12.0 / i.holding_months)
    } else {
        None
    };
    let profit_margin = if arv > 0.0 { net_profit / arv * 100.0 } else { 0.0 };

    FlipResult {
        max_allowable_offer_usd: max_allowable_offer,
        passes_rule,
        financing_points_usd: points,
        interest_cost_usd: interest,
        holding_cost_usd: holding,
        selling_cost_usd: selling,
        total_project_cost_usd: total_cost,
        net_sale_proceeds_usd: net_proceeds,
        net_profit_usd: net_profit,
        cash_invested_usd: cash_invested,
        cash_on_cash_pct: cash_on_cash,
        annualized_roi_pct: annualized_roi,
        profit_margin_pct: profit_margin,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> FlipInput {
        FlipInput {
            arv_usd: 300_000.0,
            purchase_price_usd: 150_000.0,
            rehab_cost_usd: 50_000.0,
            rule_pct: 70.0,
            holding_months: 6.0,
            monthly_holding_cost_usd: 1_000.0,
            buying_closing_cost_usd: 5_000.0,
            loan_amount_usd: 0.0,
            financing_points_pct: 0.0,
            annual_interest_rate_pct: 0.0,
            selling_cost_pct: 6.0,
        }
    }

    #[test]
    fn seventy_percent_rule_max_allowable_offer() {
        // 300k × 70% − 50k rehab = 210k − 50k = 160k.
        let r = analyze(&base());
        assert!((r.max_allowable_offer_usd - 160_000.0).abs() < 1e-6);
    }

    #[test]
    fn passes_rule_at_or_below_mao() {
        assert!(analyze(&base()).passes_rule); // 150k <= 160k
        let over = analyze(&FlipInput { purchase_price_usd: 170_000.0, ..base() });
        assert!(!over.passes_rule);
        let at = analyze(&FlipInput { purchase_price_usd: 160_000.0, ..base() });
        assert!(at.passes_rule); // boundary inclusive
    }

    #[test]
    fn all_cash_deal_net_profit() {
        // all-in = 150+50+5+6 (holding) = 211k; proceeds = 300 − 18 (6%) = 282k;
        // profit = 282 − 211 = 71k.
        let r = analyze(&base());
        assert!((r.holding_cost_usd - 6_000.0).abs() < 1e-6);
        assert!((r.selling_cost_usd - 18_000.0).abs() < 1e-6);
        assert!((r.total_project_cost_usd - 211_000.0).abs() < 1e-6);
        assert!((r.net_profit_usd - 71_000.0).abs() < 1e-6);
    }

    #[test]
    fn all_cash_cash_on_cash_uses_full_cost() {
        // No loan → cash invested = full 211k; CoC = 71/211 = 33.65%.
        let r = analyze(&base());
        assert!((r.cash_invested_usd - 211_000.0).abs() < 1e-6);
        assert!((r.cash_on_cash_pct - (71_000.0 / 211_000.0 * 100.0)).abs() < 1e-9);
    }

    #[test]
    fn financing_costs_reduce_profit_and_cash() {
        // Loan 160k, 2 points = 3.2k, 10% × 6/12 = 8k interest.
        let r = analyze(&FlipInput {
            loan_amount_usd: 160_000.0,
            financing_points_pct: 2.0,
            annual_interest_rate_pct: 10.0,
            ..base()
        });
        assert!((r.financing_points_usd - 3_200.0).abs() < 1e-6);
        assert!((r.interest_cost_usd - 8_000.0).abs() < 1e-6);
        // total = 211k + 3.2k + 8k = 222.2k; profit = 282 − 222.2 = 59.8k.
        assert!((r.total_project_cost_usd - 222_200.0).abs() < 1e-6);
        assert!((r.net_profit_usd - 59_800.0).abs() < 1e-6);
        // cash = 222.2k − 160k loan = 62.2k.
        assert!((r.cash_invested_usd - 62_200.0).abs() < 1e-6);
    }

    #[test]
    fn annualized_roi_scales_by_hold_length() {
        // 6-month hold → annualized = CoC × 2.
        let r = analyze(&base());
        assert!((r.annualized_roi_pct.unwrap() - r.cash_on_cash_pct * 2.0).abs() < 1e-9);
    }

    #[test]
    fn zero_hold_has_no_annualized_roi() {
        let r = analyze(&FlipInput { holding_months: 0.0, ..base() });
        assert!(r.annualized_roi_pct.is_none());
        assert!(r.holding_cost_usd.abs() < 1e-9);
    }

    #[test]
    fn profit_margin_is_profit_over_arv() {
        let r = analyze(&base());
        assert!((r.profit_margin_pct - (71_000.0 / 300_000.0 * 100.0)).abs() < 1e-9);
    }
}
