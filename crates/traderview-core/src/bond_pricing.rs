//! Bond pricing — the present value of a coupon bond given its yield to
//! maturity. Solves price from yield (the inverse of `yield_to_call`, which
//! solves yield from price).
//!
//! ```text
//! price = coupon · (1 − (1+y)^−n)/y + face · (1+y)^−n
//!   coupon = face · rate / freq,  y = ytm / freq,  n = years · freq
//! ```
//!
//! Above par when the coupon beats the yield (premium), below when the yield
//! beats the coupon (discount).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct BondPriceInput {
    pub face_value_usd: f64,
    /// Annual coupon rate, percent.
    pub coupon_rate_pct: f64,
    /// Yield to maturity, percent (annual).
    pub ytm_pct: f64,
    pub years_to_maturity: f64,
    /// Coupon payments per year (1 annual, 2 semiannual).
    pub frequency: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BondPriceResult {
    /// Clean price (present value of coupons + redemption).
    pub price_usd: f64,
    /// Annual coupon in dollars.
    pub annual_coupon_usd: f64,
    /// Annual coupon / price, percent.
    pub current_yield_pct: f64,
    /// "premium", "discount", or "par".
    pub premium_discount: String,
    /// face − price (gain pulled in to maturity; negative for a premium bond).
    pub capital_gain_at_maturity_usd: f64,
}

pub fn analyze(input: &BondPriceInput) -> BondPriceResult {
    let freq = if input.frequency > 0.0 { input.frequency } else { 1.0 };
    let n = input.years_to_maturity * freq;
    let coupon = input.face_value_usd * input.coupon_rate_pct / 100.0 / freq;
    let y = input.ytm_pct / 100.0 / freq;

    let price = if y.abs() < 1e-12 {
        coupon * n + input.face_value_usd
    } else {
        let v = (1.0 + y).powf(-n);
        coupon * (1.0 - v) / y + input.face_value_usd * v
    };

    let annual_coupon = input.face_value_usd * input.coupon_rate_pct / 100.0;
    let current_yield = if price > 0.0 {
        annual_coupon / price * 100.0
    } else {
        0.0
    };

    let pd = if (price - input.face_value_usd).abs() < 1e-6 {
        "par"
    } else if price > input.face_value_usd {
        "premium"
    } else {
        "discount"
    };

    BondPriceResult {
        price_usd: price,
        annual_coupon_usd: annual_coupon,
        current_yield_pct: current_yield,
        premium_discount: pd.to_string(),
        capital_gain_at_maturity_usd: input.face_value_usd - price,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn run(coupon: f64, ytm: f64) -> BondPriceResult {
        analyze(&BondPriceInput {
            face_value_usd: 1000.0,
            coupon_rate_pct: coupon,
            ytm_pct: ytm,
            years_to_maturity: 10.0,
            frequency: 2.0,
        })
    }

    #[test]
    fn price_from_yield() {
        // 5% coupon, 6% YTM, 10yr semiannual → 925.6126.
        let r = run(5.0, 6.0);
        assert!(close(r.price_usd, 925.612626));
    }

    #[test]
    fn discount_when_yield_above_coupon() {
        let r = run(5.0, 6.0);
        assert_eq!(r.premium_discount, "discount");
        assert!(r.price_usd < 1000.0);
    }

    #[test]
    fn premium_when_yield_below_coupon() {
        let r = run(6.0, 5.0);
        assert_eq!(r.premium_discount, "premium");
        assert!(r.price_usd > 1000.0);
    }

    #[test]
    fn par_when_yield_equals_coupon() {
        let r = run(5.0, 5.0);
        assert!(close(r.price_usd, 1000.0));
        assert_eq!(r.premium_discount, "par");
    }

    #[test]
    fn annual_coupon_and_current_yield() {
        let r = run(5.0, 6.0);
        assert!(close(r.annual_coupon_usd, 50.0));
        assert!(close(r.current_yield_pct, 5.401828));
    }

    #[test]
    fn capital_gain_for_discount_bond() {
        let r = run(5.0, 6.0);
        // Discount bond pulls in to par → positive gain.
        assert!(close(r.capital_gain_at_maturity_usd, 1000.0 - r.price_usd));
        assert!(r.capital_gain_at_maturity_usd > 0.0);
    }

    #[test]
    fn premium_bond_has_capital_loss() {
        let r = run(6.0, 5.0);
        assert!(r.capital_gain_at_maturity_usd < 0.0);
    }

    #[test]
    fn zero_yield_sums_cash_flows() {
        // 0% YTM: price = coupons + face = 20×25 + 1000 = 1500.
        let r = run(5.0, 0.0);
        assert!(close(r.price_usd, 1500.0));
    }
}
