//! Rental net operating income (NOI) — the property income statement that
//! feeds cap rate, DSCR, and debt yield.
//!
//! ```text
//! potential gross income = rent + other income
//! effective gross income = potential − vacancy loss
//! management fee          = management % × EGI
//! operating expenses      = taxes + insurance + maintenance + management
//!                           + utilities + repairs + HOA + other
//! NOI                     = EGI − operating expenses
//! ```
//!
//! NOI excludes debt service and capital improvements by definition.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RentalNoiInput {
    /// Annual gross rental income at full occupancy.
    pub gross_rental_income_usd: f64,
    /// Other income (laundry, parking, fees).
    #[serde(default)]
    pub other_income_usd: f64,
    /// Vacancy + credit loss as a percent of gross rent.
    #[serde(default)]
    pub vacancy_pct: f64,
    #[serde(default)]
    pub property_taxes_usd: f64,
    #[serde(default)]
    pub insurance_usd: f64,
    #[serde(default)]
    pub maintenance_usd: f64,
    /// Property management fee as a percent of effective gross income.
    #[serde(default)]
    pub management_pct: f64,
    #[serde(default)]
    pub utilities_usd: f64,
    #[serde(default)]
    pub repairs_usd: f64,
    #[serde(default)]
    pub hoa_usd: f64,
    #[serde(default)]
    pub other_expenses_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RentalNoiResult {
    /// Rent + other income at full occupancy.
    pub potential_gross_income_usd: f64,
    /// Vacancy/credit loss on the rent.
    pub vacancy_loss_usd: f64,
    /// Potential gross − vacancy loss.
    pub effective_gross_income_usd: f64,
    /// Management fee (% of EGI).
    pub management_fee_usd: f64,
    /// All operating expenses.
    pub total_operating_expenses_usd: f64,
    /// Net operating income.
    pub noi_usd: f64,
    /// Operating expenses as a percent of EGI.
    pub operating_expense_ratio_pct: Option<f64>,
}

pub fn analyze(input: &RentalNoiInput) -> RentalNoiResult {
    let potential = input.gross_rental_income_usd + input.other_income_usd;
    let vacancy_loss = input.vacancy_pct / 100.0 * input.gross_rental_income_usd;
    let egi = potential - vacancy_loss;
    let management_fee = input.management_pct / 100.0 * egi;

    let opex = input.property_taxes_usd
        + input.insurance_usd
        + input.maintenance_usd
        + management_fee
        + input.utilities_usd
        + input.repairs_usd
        + input.hoa_usd
        + input.other_expenses_usd;

    RentalNoiResult {
        potential_gross_income_usd: potential,
        vacancy_loss_usd: vacancy_loss,
        effective_gross_income_usd: egi,
        management_fee_usd: management_fee,
        total_operating_expenses_usd: opex,
        noi_usd: egi - opex,
        operating_expense_ratio_pct: if egi > 0.0 {
            Some(opex / egi * 100.0)
        } else {
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn base() -> RentalNoiInput {
        RentalNoiInput {
            gross_rental_income_usd: 60_000.0,
            other_income_usd: 2_000.0,
            vacancy_pct: 5.0,
            property_taxes_usd: 7_000.0,
            insurance_usd: 2_000.0,
            maintenance_usd: 3_000.0,
            management_pct: 8.0,
            utilities_usd: 1_500.0,
            repairs_usd: 1_000.0,
            hoa_usd: 0.0,
            other_expenses_usd: 500.0,
        }
    }

    #[test]
    fn potential_gross() {
        assert!(close(analyze(&base()).potential_gross_income_usd, 62_000.0));
    }

    #[test]
    fn vacancy_loss() {
        // 5% × 60,000 = 3,000.
        assert!(close(analyze(&base()).vacancy_loss_usd, 3_000.0));
    }

    #[test]
    fn effective_gross_income() {
        assert!(close(analyze(&base()).effective_gross_income_usd, 59_000.0));
    }

    #[test]
    fn management_fee_on_egi() {
        // 8% × 59,000 = 4,720.
        assert!(close(analyze(&base()).management_fee_usd, 4_720.0));
    }

    #[test]
    fn total_operating_expenses() {
        // 7000+2000+3000+4720+1500+1000+0+500 = 19,720.
        assert!(close(analyze(&base()).total_operating_expenses_usd, 19_720.0));
    }

    #[test]
    fn noi() {
        // 59,000 − 19,720 = 39,280.
        assert!(close(analyze(&base()).noi_usd, 39_280.0));
    }

    #[test]
    fn operating_expense_ratio() {
        // 19,720 / 59,000 = 33.4237%.
        assert!(close(analyze(&base()).operating_expense_ratio_pct.unwrap(), 33.423729));
    }

    #[test]
    fn higher_vacancy_lowers_noi() {
        let low = analyze(&base());
        let high = analyze(&RentalNoiInput {
            vacancy_pct: 20.0,
            ..base()
        });
        assert!(high.noi_usd < low.noi_usd);
    }
}
