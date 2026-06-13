//! Seller financing / carryback note — when the seller is the bank.
//!
//! Instead of the buyer getting a bank loan, the seller "carries back" a
//! note: the buyer puts down a down payment and pays the seller directly on
//! the balance, at a rate, amortized over a schedule — often with a
//! **balloon** where the remaining balance is due in a few years.
//!
//!   * note amount = sale price − down payment
//!   * monthly payment amortizes the note over the amortization period
//!   * balloon balance = the loan balance still owed at the balloon date
//!   * seller interest income = total payments + balloon − note amount
//!
//! Pure compute (standard amortization, zero-rate straight-line guard).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SellerFinancingInput {
    pub sale_price_usd: f64,
    pub down_payment_usd: f64,
    pub annual_rate_pct: f64,
    /// Years the payment is amortized over (e.g. 30).
    pub amortization_years: f64,
    /// Years until the balloon (remaining balance due); 0 = fully amortizing.
    #[serde(default)]
    pub balloon_years: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SellerFinancingResult {
    pub note_amount_usd: f64,
    pub monthly_payment_usd: f64,
    /// Balance still owed at the balloon date (0 if fully amortizing).
    pub balloon_balance_usd: f64,
    pub has_balloon: bool,
    /// Payments made before the balloon (or over the full term).
    pub total_payments_usd: f64,
    /// Interest the seller earns over the life of the note.
    pub seller_interest_income_usd: f64,
}

fn monthly_payment(principal: f64, annual_rate_pct: f64, months: f64) -> f64 {
    if months <= 0.0 {
        return 0.0;
    }
    let r = annual_rate_pct / 100.0 / 12.0;
    if r.abs() < 1e-12 {
        principal / months
    } else {
        let f = (1.0 + r).powf(months);
        principal * r * f / (f - 1.0)
    }
}

/// Remaining balance after `k` payments of `pmt` on `principal` at monthly `r`.
fn balance_after(principal: f64, annual_rate_pct: f64, pmt: f64, k: f64) -> f64 {
    let r = annual_rate_pct / 100.0 / 12.0;
    if r.abs() < 1e-12 {
        (principal - pmt * k).max(0.0)
    } else {
        let f = (1.0 + r).powf(k);
        (principal * f - pmt * (f - 1.0) / r).max(0.0)
    }
}

pub fn analyze(i: &SellerFinancingInput) -> SellerFinancingResult {
    let note = (i.sale_price_usd - i.down_payment_usd).max(0.0);
    let amort_months = (i.amortization_years * 12.0).max(0.0);
    let pmt = monthly_payment(note, i.annual_rate_pct, amort_months);

    // The note runs to the balloon, or to full amortization if no balloon.
    let has_balloon = i.balloon_years > 0.0 && i.balloon_years < i.amortization_years;
    let pay_months = if has_balloon { i.balloon_years * 12.0 } else { amort_months };

    let balloon_balance = if has_balloon {
        balance_after(note, i.annual_rate_pct, pmt, pay_months)
    } else {
        0.0
    };

    let total_payments = pmt * pay_months;
    let seller_interest = total_payments + balloon_balance - note;

    SellerFinancingResult {
        note_amount_usd: note,
        monthly_payment_usd: pmt,
        balloon_balance_usd: balloon_balance,
        has_balloon,
        total_payments_usd: total_payments,
        seller_interest_income_usd: seller_interest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> SellerFinancingInput {
        SellerFinancingInput {
            sale_price_usd: 400_000.0,
            down_payment_usd: 80_000.0,
            annual_rate_pct: 7.0,
            amortization_years: 30.0,
            balloon_years: 5.0,
        }
    }

    #[test]
    fn note_is_price_minus_down() {
        let r = analyze(&base());
        assert!((r.note_amount_usd - 320_000.0).abs() < 1e-6);
    }

    #[test]
    fn monthly_payment_amortizes_note() {
        // 320k @ 7% / 30y ≈ $2,128.97.
        let r = analyze(&base());
        assert!((r.monthly_payment_usd - 2_128.97).abs() < 0.5);
    }

    #[test]
    fn balloon_balance_after_five_years() {
        // Most of a 30y note is still owed after 5y.
        let r = analyze(&base());
        assert!(r.has_balloon);
        assert!(r.balloon_balance_usd > 280_000.0 && r.balloon_balance_usd < 310_000.0);
    }

    #[test]
    fn no_balloon_fully_amortizes() {
        let r = analyze(&SellerFinancingInput { balloon_years: 0.0, ..base() });
        assert!(!r.has_balloon);
        assert!(r.balloon_balance_usd.abs() < 1e-6);
    }

    #[test]
    fn seller_interest_income_positive() {
        let r = analyze(&base());
        assert!(r.seller_interest_income_usd > 0.0);
        // total payments + balloon − note = interest.
        let expected = r.total_payments_usd + r.balloon_balance_usd - r.note_amount_usd;
        assert!((r.seller_interest_income_usd - expected).abs() < 1e-6);
    }

    #[test]
    fn higher_rate_raises_payment() {
        let low = analyze(&SellerFinancingInput { annual_rate_pct: 5.0, ..base() });
        let high = analyze(&SellerFinancingInput { annual_rate_pct: 9.0, ..base() });
        assert!(high.monthly_payment_usd > low.monthly_payment_usd);
    }

    #[test]
    fn balloon_at_or_after_amort_is_full_amortization() {
        let r = analyze(&SellerFinancingInput { balloon_years: 30.0, ..base() });
        assert!(!r.has_balloon); // not < amortization
        assert!(r.balloon_balance_usd.abs() < 1e-6);
    }

    #[test]
    fn zero_rate_straight_line_balance() {
        // 320k over 30y = 888.89/mo; after 5y (60 pmts) balance = 320k − 53,333 = 266,667.
        let r = analyze(&SellerFinancingInput { annual_rate_pct: 0.0, ..base() });
        assert!((r.monthly_payment_usd - 320_000.0 / 360.0).abs() < 1e-6);
        let expected = 320_000.0 - (320_000.0 / 360.0) * 60.0;
        assert!((r.balloon_balance_usd - expected).abs() < 1e-3);
    }
}
