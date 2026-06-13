//! Gross rent multiplier & effective gross income — rental valuation screen.
//!
//! GRM is a fast way to compare rental prices: property price ÷ gross annual
//! rent. A lower GRM is cheaper relative to the rent it produces. But
//! scheduled rent overstates reality — vacancy and credit loss take a cut,
//! while laundry/parking/fees add a bit back. Netting those gives effective
//! gross income (EGI) and a truer "effective GRM":
//!
//!   * gross scheduled income = monthly rent × 12
//!   * GRM = price / gross scheduled income
//!   * EGI = gross scheduled − vacancy/credit loss + other income
//!   * effective GRM = price / EGI
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct GrmInput {
    pub property_price_usd: f64,
    /// Scheduled monthly rent with all units occupied.
    pub gross_monthly_rent_usd: f64,
    /// Vacancy + credit loss as a percent of scheduled rent.
    #[serde(default)]
    pub vacancy_rate_pct: f64,
    /// Other monthly income (laundry, parking, fees).
    #[serde(default)]
    pub other_income_monthly_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct GrmResult {
    pub gross_scheduled_income_usd: f64,
    pub gross_rent_multiplier: f64,
    pub vacancy_loss_usd: f64,
    pub other_income_usd: f64,
    /// Scheduled − vacancy/credit loss + other income.
    pub effective_gross_income_usd: f64,
    /// Price ÷ EGI (GRM on the income actually collected).
    pub effective_grm: f64,
}

pub fn analyze(i: &GrmInput) -> GrmResult {
    let gsi = i.gross_monthly_rent_usd.max(0.0) * 12.0;
    let grm = if gsi > 0.0 { i.property_price_usd / gsi } else { 0.0 };

    let vacancy_loss = gsi * i.vacancy_rate_pct / 100.0;
    let other_income = i.other_income_monthly_usd.max(0.0) * 12.0;
    let egi = gsi - vacancy_loss + other_income;
    let effective_grm = if egi > 0.0 { i.property_price_usd / egi } else { 0.0 };

    GrmResult {
        gross_scheduled_income_usd: gsi,
        gross_rent_multiplier: grm,
        vacancy_loss_usd: vacancy_loss,
        other_income_usd: other_income,
        effective_gross_income_usd: egi,
        effective_grm,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> GrmInput {
        GrmInput {
            property_price_usd: 600_000.0,
            gross_monthly_rent_usd: 5_000.0,
            vacancy_rate_pct: 5.0,
            other_income_monthly_usd: 0.0,
        }
    }

    #[test]
    fn grm_is_price_over_gross_annual() {
        // 600k / (5000×12 = 60k) = 10.
        let r = analyze(&base());
        assert!((r.gross_scheduled_income_usd - 60_000.0).abs() < 1e-6);
        assert!((r.gross_rent_multiplier - 10.0).abs() < 1e-9);
    }

    #[test]
    fn vacancy_loss_is_pct_of_scheduled() {
        // 5% of 60k = 3,000.
        let r = analyze(&base());
        assert!((r.vacancy_loss_usd - 3_000.0).abs() < 1e-6);
    }

    #[test]
    fn egi_nets_vacancy() {
        // 60k − 3k = 57k.
        let r = analyze(&base());
        assert!((r.effective_gross_income_usd - 57_000.0).abs() < 1e-6);
    }

    #[test]
    fn effective_grm_uses_egi() {
        // 600k / 57k ≈ 10.526.
        let r = analyze(&base());
        assert!((r.effective_grm - 600_000.0 / 57_000.0).abs() < 1e-6);
        assert!(r.effective_grm > r.gross_rent_multiplier); // EGI < scheduled
    }

    #[test]
    fn other_income_raises_egi() {
        // +200/mo = 2,400/yr → EGI 57k + 2.4k = 59.4k.
        let r = analyze(&GrmInput { other_income_monthly_usd: 200.0, ..base() });
        assert!((r.other_income_usd - 2_400.0).abs() < 1e-6);
        assert!((r.effective_gross_income_usd - 59_400.0).abs() < 1e-6);
    }

    #[test]
    fn lower_price_lower_grm() {
        let cheap = analyze(&GrmInput { property_price_usd: 480_000.0, ..base() });
        let dear = analyze(&base());
        assert!(cheap.gross_rent_multiplier < dear.gross_rent_multiplier);
    }

    #[test]
    fn no_vacancy_egi_equals_scheduled() {
        let r = analyze(&GrmInput { vacancy_rate_pct: 0.0, ..base() });
        assert!((r.effective_gross_income_usd - r.gross_scheduled_income_usd).abs() < 1e-6);
        assert!((r.effective_grm - r.gross_rent_multiplier).abs() < 1e-9);
    }

    #[test]
    fn zero_rent_guards() {
        let r = analyze(&GrmInput { gross_monthly_rent_usd: 0.0, ..base() });
        assert!(r.gross_rent_multiplier.abs() < 1e-9);
        assert!(r.effective_grm.abs() < 1e-9);
    }
}
