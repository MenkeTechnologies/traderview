//! Price-to-rent ratio — the buy-vs-rent screen for a market.
//!
//! A quick read on whether a metro favors buying or renting: divide a home's
//! price by the annual rent for a comparable home.
//!
//!   * price-to-rent = home price / annual rent
//!   * gross rental yield = annual rent / home price (the inverse, as a %)
//!
//! Rule of thumb: **< 15** generally favors buying, **15–20** is borderline,
//! **> 20** favors renting (the home is expensive relative to what it rents
//! for). A screen, not a full rent-vs-buy NPV. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PriceToRentInput {
    pub home_price_usd: f64,
    pub monthly_rent_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PriceToRentResult {
    pub annual_rent_usd: f64,
    pub price_to_rent_ratio: f64,
    /// Annual rent as a percent of price (the inverse of the ratio).
    pub gross_rental_yield_pct: f64,
    /// "favors buying" (<15), "borderline" (15–20), or "favors renting" (>20).
    pub verdict: String,
}

pub fn analyze(i: &PriceToRentInput) -> PriceToRentResult {
    let annual_rent = i.monthly_rent_usd.max(0.0) * 12.0;
    let ratio = if annual_rent > 0.0 { i.home_price_usd / annual_rent } else { 0.0 };
    let yield_pct = if i.home_price_usd > 0.0 { annual_rent / i.home_price_usd * 100.0 } else { 0.0 };

    let verdict = if annual_rent <= 0.0 || i.home_price_usd <= 0.0 {
        "n/a"
    } else if ratio < 15.0 {
        "favors buying"
    } else if ratio <= 20.0 {
        "borderline"
    } else {
        "favors renting"
    };

    PriceToRentResult {
        annual_rent_usd: annual_rent,
        price_to_rent_ratio: ratio,
        gross_rental_yield_pct: yield_pct,
        verdict: verdict.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inp(price: f64, rent: f64) -> PriceToRentInput {
        PriceToRentInput { home_price_usd: price, monthly_rent_usd: rent }
    }

    #[test]
    fn ratio_is_price_over_annual_rent() {
        // 400k / (2000×12 = 24k) = 16.67.
        let r = analyze(&inp(400_000.0, 2_000.0));
        assert!((r.annual_rent_usd - 24_000.0).abs() < 1e-9);
        assert!((r.price_to_rent_ratio - 400_000.0 / 24_000.0).abs() < 1e-9);
    }

    #[test]
    fn gross_yield_is_inverse() {
        let r = analyze(&inp(400_000.0, 2_000.0));
        // yield = 100 / ratio.
        assert!((r.gross_rental_yield_pct - 100.0 / r.price_to_rent_ratio).abs() < 1e-9);
    }

    #[test]
    fn favors_buying_under_15() {
        // 300k / 30k = 10 → buy.
        let r = analyze(&inp(300_000.0, 2_500.0));
        assert!(r.price_to_rent_ratio < 15.0);
        assert_eq!(r.verdict, "favors buying");
    }

    #[test]
    fn favors_renting_over_20() {
        // 600k / 24k = 25 → rent.
        let r = analyze(&inp(600_000.0, 2_000.0));
        assert!(r.price_to_rent_ratio > 20.0);
        assert_eq!(r.verdict, "favors renting");
    }

    #[test]
    fn borderline_15_to_20() {
        // 400k / 24k = 16.67 → borderline.
        let r = analyze(&inp(400_000.0, 2_000.0));
        assert_eq!(r.verdict, "borderline");
    }

    #[test]
    fn ratio_times_yield_is_100() {
        let r = analyze(&inp(500_000.0, 2_200.0));
        assert!((r.price_to_rent_ratio * r.gross_rental_yield_pct - 100.0).abs() < 1e-6);
    }

    #[test]
    fn boundary_at_15_is_borderline() {
        // 360k / 24k = 15 exactly → borderline (not "favors buying").
        let r = analyze(&inp(360_000.0, 2_000.0));
        assert!((r.price_to_rent_ratio - 15.0).abs() < 1e-9);
        assert_eq!(r.verdict, "borderline");
    }

    #[test]
    fn zero_rent_guards() {
        let r = analyze(&inp(400_000.0, 0.0));
        assert!(r.price_to_rent_ratio.abs() < 1e-9);
        assert_eq!(r.verdict, "n/a");
    }
}
