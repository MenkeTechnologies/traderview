//! Piotroski F-Score — Joseph Piotroski's 9-point test of a firm's financial
//! strength, scoring one point for each criterion passed across profitability,
//! leverage/liquidity, and operating efficiency. A score of 8–9 is strong,
//! 0–2 is weak; it's most often applied to cheap (high book-to-market) stocks
//! to separate the healthy from the value traps.
//!
//! The nine tests (`_t` = this year, `_ty` = prior year):
//!  1. ROA positive (net income > 0)
//!  2. Operating cash flow positive
//!  3. ROA rising year over year
//!  4. Accruals: operating cash flow exceeds net income
//!  5. Leverage falling: long-term-debt/assets lower than last year
//!  6. Liquidity rising: current ratio higher than last year
//!  7. No dilution: shares outstanding not increased
//!  8. Gross margin rising
//!  9. Asset turnover rising

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PiotroskiInput {
    pub net_income_t: f64,
    pub net_income_ty: f64,
    pub total_assets_t: f64,
    pub total_assets_ty: f64,
    pub operating_cash_flow_t: f64,
    pub long_term_debt_t: f64,
    pub long_term_debt_ty: f64,
    pub current_assets_t: f64,
    pub current_liabilities_t: f64,
    pub current_assets_ty: f64,
    pub current_liabilities_ty: f64,
    pub shares_outstanding_t: f64,
    pub shares_outstanding_ty: f64,
    pub gross_profit_t: f64,
    pub gross_profit_ty: f64,
    pub revenue_t: f64,
    pub revenue_ty: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PiotroskiResult {
    pub roa_positive: bool,
    pub cfo_positive: bool,
    pub roa_rising: bool,
    pub accruals_quality: bool,
    pub leverage_falling: bool,
    pub liquidity_rising: bool,
    pub no_dilution: bool,
    pub margin_rising: bool,
    pub turnover_rising: bool,
    /// 0–9.
    pub f_score: u8,
    /// "strong" (≥7), "moderate" (4–6), or "weak" (≤3).
    pub rating: String,
}

fn ratio(num: f64, den: f64) -> Option<f64> {
    if den != 0.0 {
        Some(num / den)
    } else {
        None
    }
}

pub fn analyze(input: &PiotroskiInput) -> PiotroskiResult {
    // Profitability.
    let roa_positive = input.net_income_t > 0.0;
    let cfo_positive = input.operating_cash_flow_t > 0.0;
    let roa_t = ratio(input.net_income_t, input.total_assets_t);
    let roa_ty = ratio(input.net_income_ty, input.total_assets_ty);
    let roa_rising = matches!((roa_t, roa_ty), (Some(a), Some(b)) if a > b);
    let accruals_quality = input.operating_cash_flow_t > input.net_income_t;

    // Leverage, liquidity, dilution.
    let ltd_t = ratio(input.long_term_debt_t, input.total_assets_t);
    let ltd_ty = ratio(input.long_term_debt_ty, input.total_assets_ty);
    let leverage_falling = matches!((ltd_t, ltd_ty), (Some(a), Some(b)) if a < b);
    let cr_t = ratio(input.current_assets_t, input.current_liabilities_t);
    let cr_ty = ratio(input.current_assets_ty, input.current_liabilities_ty);
    let liquidity_rising = matches!((cr_t, cr_ty), (Some(a), Some(b)) if a > b);
    let no_dilution = input.shares_outstanding_t <= input.shares_outstanding_ty;

    // Operating efficiency.
    let margin_t = ratio(input.gross_profit_t, input.revenue_t);
    let margin_ty = ratio(input.gross_profit_ty, input.revenue_ty);
    let margin_rising = matches!((margin_t, margin_ty), (Some(a), Some(b)) if a > b);
    let turn_t = ratio(input.revenue_t, input.total_assets_t);
    let turn_ty = ratio(input.revenue_ty, input.total_assets_ty);
    let turnover_rising = matches!((turn_t, turn_ty), (Some(a), Some(b)) if a > b);

    let f_score = [
        roa_positive,
        cfo_positive,
        roa_rising,
        accruals_quality,
        leverage_falling,
        liquidity_rising,
        no_dilution,
        margin_rising,
        turnover_rising,
    ]
    .iter()
    .filter(|&&b| b)
    .count() as u8;

    let rating = if f_score >= 7 {
        "strong"
    } else if f_score >= 4 {
        "moderate"
    } else {
        "weak"
    }
    .to_string();

    PiotroskiResult {
        roa_positive,
        cfo_positive,
        roa_rising,
        accruals_quality,
        leverage_falling,
        liquidity_rising,
        no_dilution,
        margin_rising,
        turnover_rising,
        f_score,
        rating,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn perfect() -> PiotroskiInput {
        PiotroskiInput {
            net_income_t: 100.0,
            net_income_ty: 50.0,
            total_assets_t: 1000.0,
            total_assets_ty: 1000.0,
            operating_cash_flow_t: 150.0,
            long_term_debt_t: 100.0,
            long_term_debt_ty: 200.0,
            current_assets_t: 400.0,
            current_liabilities_t: 200.0,
            current_assets_ty: 300.0,
            current_liabilities_ty: 200.0,
            shares_outstanding_t: 1000.0,
            shares_outstanding_ty: 1000.0,
            gross_profit_t: 600.0,
            gross_profit_ty: 400.0,
            revenue_t: 1000.0,
            revenue_ty: 800.0,
        }
    }

    #[test]
    fn perfect_nine() {
        let r = analyze(&perfect());
        assert_eq!(r.f_score, 9);
        assert!(r.roa_positive && r.cfo_positive && r.roa_rising && r.accruals_quality);
        assert!(r.leverage_falling && r.liquidity_rising && r.no_dilution);
        assert!(r.margin_rising && r.turnover_rising);
        assert_eq!(r.rating, "strong");
    }

    #[test]
    fn worst_zero() {
        let r = analyze(&PiotroskiInput {
            net_income_t: -10.0,
            net_income_ty: 50.0,
            total_assets_t: 1000.0,
            total_assets_ty: 1000.0,
            operating_cash_flow_t: -20.0,
            long_term_debt_t: 300.0,
            long_term_debt_ty: 100.0,
            current_assets_t: 200.0,
            current_liabilities_t: 200.0,
            current_assets_ty: 300.0,
            current_liabilities_ty: 200.0,
            shares_outstanding_t: 1100.0,
            shares_outstanding_ty: 1000.0,
            gross_profit_t: 300.0,
            gross_profit_ty: 400.0,
            revenue_t: 700.0,
            revenue_ty: 800.0,
        });
        assert_eq!(r.f_score, 0);
        assert_eq!(r.rating, "weak");
    }

    #[test]
    fn accruals_flips_when_cfo_below_ni() {
        let mut i = perfect();
        i.operating_cash_flow_t = 80.0; // below net income 100
        let r = analyze(&i);
        assert!(!r.accruals_quality);
        assert_eq!(r.f_score, 8);
    }

    #[test]
    fn dilution_fails_criterion() {
        let mut i = perfect();
        i.shares_outstanding_t = 1100.0;
        let r = analyze(&i);
        assert!(!r.no_dilution);
        assert_eq!(r.f_score, 8);
    }

    #[test]
    fn equal_shares_count_as_no_dilution() {
        let r = analyze(&perfect()); // shares equal both years
        assert!(r.no_dilution);
    }

    #[test]
    fn leverage_falling_detected() {
        let r = analyze(&perfect());
        // LTD/TA 0.1 < 0.2 → leverage fell.
        assert!(r.leverage_falling);
        let mut i = perfect();
        i.long_term_debt_t = 250.0; // 0.25 > 0.2
        assert!(!analyze(&i).leverage_falling);
    }

    #[test]
    fn moderate_rating() {
        // Flip four criteria off the perfect nine → 5 → moderate.
        let mut i = perfect();
        i.operating_cash_flow_t = -1.0; // crit2 off (and crit4: -1 > 100 false)
        i.shares_outstanding_t = 2000.0; // crit7 off
        i.gross_profit_t = 300.0; // margin 0.3 < 0.4 → crit8 off
        let r = analyze(&i);
        assert_eq!(r.f_score, 5);
        assert_eq!(r.rating, "moderate");
    }

    #[test]
    fn zero_revenue_guards_efficiency() {
        let mut i = perfect();
        i.revenue_t = 0.0;
        let r = analyze(&i);
        // No revenue → margin and turnover comparisons are false, not a panic.
        assert!(!r.margin_rising);
        assert!(!r.turnover_rising);
    }
}
