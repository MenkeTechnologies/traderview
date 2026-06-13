//! Break-even occupancy — the occupancy at which a rental's effective income
//! exactly covers operating expenses plus debt service. The lender's cushion
//! metric: the lower it is, the more vacancy a property can absorb.
//!
//! ```text
//! breakeven occupancy = (operating expenses + debt service) / gross potential rent
//! vacancy cushion     = 100% − breakeven occupancy
//! ```
//!
//! At an expected occupancy it also reports the resulting cash flow.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct BreakevenOccupancyInput {
    /// Gross potential rent at 100% occupancy (annual).
    pub gross_potential_rent_usd: f64,
    /// Annual operating expenses (excludes debt service).
    pub operating_expenses_usd: f64,
    /// Annual debt service (P&I).
    pub annual_debt_service_usd: f64,
    /// Expected occupancy for the cash-flow estimate, percent.
    #[serde(default)]
    pub expected_occupancy_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BreakevenOccupancyResult {
    /// Occupancy needed to cover expenses + debt service, percent.
    pub breakeven_occupancy_pct: Option<f64>,
    /// 100% − breakeven (vacancy the property can absorb), percent.
    pub vacancy_cushion_pct: Option<f64>,
    /// Effective gross income at the expected occupancy.
    pub effective_gross_income_usd: f64,
    /// Cash flow at the expected occupancy (EGI − opex − debt service).
    pub cash_flow_at_expected_usd: f64,
}

pub fn analyze(input: &BreakevenOccupancyInput) -> BreakevenOccupancyResult {
    let total_obligations = input.operating_expenses_usd + input.annual_debt_service_usd;

    let breakeven = if input.gross_potential_rent_usd > 0.0 {
        Some(total_obligations / input.gross_potential_rent_usd * 100.0)
    } else {
        None
    };
    let cushion = breakeven.map(|b| 100.0 - b);

    let egi = input.gross_potential_rent_usd * input.expected_occupancy_pct / 100.0;
    let cash_flow = egi - total_obligations;

    BreakevenOccupancyResult {
        breakeven_occupancy_pct: breakeven,
        vacancy_cushion_pct: cushion,
        effective_gross_income_usd: egi,
        cash_flow_at_expected_usd: cash_flow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn base() -> BreakevenOccupancyInput {
        BreakevenOccupancyInput {
            gross_potential_rent_usd: 100_000.0,
            operating_expenses_usd: 35_000.0,
            annual_debt_service_usd: 45_000.0,
            expected_occupancy_pct: 95.0,
        }
    }

    #[test]
    fn breakeven_occupancy() {
        // (35,000 + 45,000) / 100,000 = 80%.
        assert!(close(analyze(&base()).breakeven_occupancy_pct.unwrap(), 80.0));
    }

    #[test]
    fn vacancy_cushion() {
        assert!(close(analyze(&base()).vacancy_cushion_pct.unwrap(), 20.0));
    }

    #[test]
    fn egi_at_expected() {
        // 100,000 × 95% = 95,000.
        assert!(close(analyze(&base()).effective_gross_income_usd, 95_000.0));
    }

    #[test]
    fn cash_flow_at_expected() {
        // 95,000 − 35,000 − 45,000 = 15,000.
        assert!(close(analyze(&base()).cash_flow_at_expected_usd, 15_000.0));
    }

    #[test]
    fn cash_flow_zero_at_breakeven() {
        let r = analyze(&BreakevenOccupancyInput {
            expected_occupancy_pct: 80.0,
            ..base()
        });
        assert!(close(r.cash_flow_at_expected_usd, 0.0));
    }

    #[test]
    fn higher_obligations_raise_breakeven() {
        let low = analyze(&base());
        let high = analyze(&BreakevenOccupancyInput {
            operating_expenses_usd: 50_000.0,
            ..base()
        });
        assert!(high.breakeven_occupancy_pct.unwrap() > low.breakeven_occupancy_pct.unwrap());
    }

    #[test]
    fn over_100_when_obligations_exceed_rent() {
        let r = analyze(&BreakevenOccupancyInput {
            operating_expenses_usd: 70_000.0,
            annual_debt_service_usd: 45_000.0,
            ..base()
        });
        // 115,000 / 100,000 = 115% — can't break even even fully occupied.
        assert!(r.breakeven_occupancy_pct.unwrap() > 100.0);
        assert!(r.vacancy_cushion_pct.unwrap() < 0.0);
    }

    #[test]
    fn zero_rent_guards() {
        let r = analyze(&BreakevenOccupancyInput {
            gross_potential_rent_usd: 0.0,
            ..base()
        });
        assert!(r.breakeven_occupancy_pct.is_none());
    }
}
