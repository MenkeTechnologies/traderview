//! Treasury-bill yield calculator.
//!
//! T-bills are quoted on a bank-discount basis, which understates the true
//! return (it divides the discount by face, not price, and uses a 360-day
//! year). This converts among the quotes the market actually uses, either
//! from a known price or from a known discount rate.
//!
//! Formulas (P = price per $100, r = days to maturity, y = days in year):
//!
//! * Bank discount rate:   d = (100 − P)/100 × 360/r
//! * Price from discount:  P = 100 × (1 − d × r/360)
//! * Money-market yield:   (100 − P)/P × 360/r   (CD-equivalent, 360-day)
//! * Investment rate (coupon-equivalent), per 31 CFR 356 Appendix B:
//!     - r ≤ y/2:  i = (100 − P)/P × y/r
//!     - r >  y/2:  quadratic a·i² + b·i + c = 0 with
//!         a = r/(2y) − 0.25,  b = r/y,  c = 1 − 100/P,
//!         i = (−b + √(b² − 4ac)) / (2a)
//! * Effective annual yield: (100/P)^(y/r) − 1

use serde::{Deserialize, Serialize};

fn default_year() -> f64 {
    365.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// `value` is the price per $100.
    FromPrice,
    /// `value` is the bank-discount rate in percent.
    FromDiscount,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TbillInput {
    pub mode: Mode,
    /// Price per $100 (FromPrice) or bank-discount rate % (FromDiscount).
    pub value: f64,
    /// Days to maturity.
    pub days_to_maturity: f64,
    /// Face value for the dollar figures (e.g. 1000.0 or 100.0).
    pub face_value: f64,
    /// Days in the year following issue — 365, or 366 if it spans Feb 29.
    #[serde(default = "default_year")]
    pub year_days: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TbillResult {
    /// Price per $100.
    pub price_per_100: f64,
    /// Dollar purchase price (face × P/100).
    pub purchase_price: f64,
    /// Dollar discount from face (face − purchase price).
    pub discount_usd: f64,
    /// Bank-discount rate, percent (the quoted rate).
    pub bank_discount_rate_pct: f64,
    /// Money-market / CD-equivalent yield, percent (360-day, on price).
    pub money_market_yield_pct: f64,
    /// Investment rate / coupon-equivalent yield, percent (Treasury method).
    pub investment_rate_pct: f64,
    /// Effective annual yield, percent (compounded).
    pub effective_annual_yield_pct: f64,
}

pub fn analyze(input: &TbillInput) -> TbillResult {
    let r = input.days_to_maturity;
    let y = if input.year_days > 0.0 {
        input.year_days
    } else {
        365.0
    };

    // Resolve price per $100 from whichever input was given.
    let p = match input.mode {
        Mode::FromPrice => input.value,
        Mode::FromDiscount => {
            let d = input.value / 100.0;
            100.0 * (1.0 - d * r / 360.0)
        }
    };

    // Degenerate guards: no time or no price means no yield.
    if r <= 0.0 || p <= 0.0 {
        let purchase = input.face_value * p / 100.0;
        return TbillResult {
            price_per_100: p,
            purchase_price: purchase,
            discount_usd: input.face_value - purchase,
            bank_discount_rate_pct: 0.0,
            money_market_yield_pct: 0.0,
            investment_rate_pct: 0.0,
            effective_annual_yield_pct: 0.0,
        };
    }

    let bank_discount = (100.0 - p) / 100.0 * 360.0 / r;
    let money_market = (100.0 - p) / p * 360.0 / r;

    // Coupon-equivalent (investment) rate.
    let investment = if r <= y / 2.0 {
        (100.0 - p) / p * y / r
    } else {
        let a = r / (2.0 * y) - 0.25;
        let b = r / y;
        let c = 1.0 - 100.0 / p;
        if a != 0.0 {
            (-b + (b * b - 4.0 * a * c).sqrt()) / (2.0 * a)
        } else {
            (100.0 - p) / p * y / r
        }
    };

    let effective = (100.0_f64 / p).powf(y / r) - 1.0;

    let purchase = input.face_value * p / 100.0;
    TbillResult {
        price_per_100: p,
        purchase_price: purchase,
        discount_usd: input.face_value - purchase,
        bank_discount_rate_pct: bank_discount * 100.0,
        money_market_yield_pct: money_market * 100.0,
        investment_rate_pct: investment * 100.0,
        effective_annual_yield_pct: effective * 100.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn from_discount(rate: f64, days: f64) -> TbillResult {
        analyze(&TbillInput {
            mode: Mode::FromDiscount,
            value: rate,
            days_to_maturity: days,
            face_value: 1000.0,
            year_days: 365.0,
        })
    }

    #[test]
    fn discount_to_price_91_day() {
        // 5% discount, 91 days: P = 100·(1 − 0.05·91/360) = 98.7361111.
        let r = from_discount(5.0, 91.0);
        assert!(close(r.price_per_100, 98.7361111));
        assert!(close(r.bank_discount_rate_pct, 5.0));
    }

    #[test]
    fn dollar_price_and_discount() {
        let r = from_discount(5.0, 91.0);
        // face 1000 → purchase = 987.361111, discount = 12.638889.
        assert!(close(r.purchase_price, 987.3611111));
        assert!(close(r.discount_usd, 12.6388889));
    }

    #[test]
    fn price_roundtrips_to_same_discount() {
        let p = from_discount(5.0, 91.0).price_per_100;
        let r = analyze(&TbillInput {
            mode: Mode::FromPrice,
            value: p,
            days_to_maturity: 91.0,
            face_value: 1000.0,
            year_days: 365.0,
        });
        assert!(close(r.bank_discount_rate_pct, 5.0));
    }

    #[test]
    fn investment_rate_short_bill_exact() {
        // 91-day: (100−P)/P · 365/91 = 5.134372%.
        let r = from_discount(5.0, 91.0);
        assert!(close(r.investment_rate_pct, 5.134372));
    }

    #[test]
    fn yields_are_ordered() {
        // Discount understates; MMY < investment < effective.
        let r = from_discount(5.0, 91.0);
        assert!(r.bank_discount_rate_pct < r.money_market_yield_pct);
        assert!(r.money_market_yield_pct < r.investment_rate_pct);
        assert!(r.investment_rate_pct < r.effective_annual_yield_pct);
    }

    #[test]
    fn long_bill_uses_quadratic() {
        // 364-day, 5% discount: quadratic gives 5.270135%, below the naive
        // simple-formula 5.339380% because of semiannual compounding.
        let r = from_discount(5.0, 364.0);
        assert!(close(r.investment_rate_pct, 5.270135));
        let p = r.price_per_100;
        let simple = (100.0 - p) / p * 365.0 / 364.0 * 100.0;
        assert!(r.investment_rate_pct < simple);
    }

    #[test]
    fn leap_year_uses_366() {
        let r = analyze(&TbillInput {
            mode: Mode::FromDiscount,
            value: 5.0,
            days_to_maturity: 91.0,
            face_value: 1000.0,
            year_days: 366.0,
        });
        let p = r.price_per_100;
        // 366/91 in the numerator → slightly higher than the 365-day case.
        assert!(close(r.investment_rate_pct, (100.0 - p) / p * 366.0 / 91.0 * 100.0));
        assert!(r.investment_rate_pct > from_discount(5.0, 91.0).investment_rate_pct);
    }

    #[test]
    fn zero_days_guard() {
        let r = from_discount(5.0, 0.0);
        assert!(close(r.bank_discount_rate_pct, 0.0));
        assert!(close(r.investment_rate_pct, 0.0));
    }
}
