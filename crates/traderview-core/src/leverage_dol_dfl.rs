//! Degree of operating, financial, and combined leverage.
//!
//! Leverage measures how a change in sales is amplified on the way down the
//! income statement — fixed operating costs amplify it into EBIT, fixed
//! interest amplifies it again into EPS.
//!
//! ```text
//! contribution margin = sales − variable costs
//! EBIT                = contribution margin − fixed costs
//! DOL = contribution margin / EBIT          (sales → EBIT amplification)
//! DFL = EBIT / (EBIT − interest)            (EBIT → EPS amplification)
//! DCL = DOL × DFL = contribution margin / (EBIT − interest)
//! ```
//!
//! A DOL of 2 means a 1% change in sales swings EBIT 2%; DCL chains both, so a
//! 1% sales change swings EPS by DCL%.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LeverageInput {
    pub sales_usd: f64,
    pub variable_costs_usd: f64,
    pub fixed_costs_usd: f64,
    #[serde(default)]
    pub interest_expense_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LeverageResult {
    /// sales − variable costs.
    pub contribution_margin_usd: f64,
    /// contribution margin − fixed costs.
    pub ebit_usd: f64,
    /// Pre-tax income: EBIT − interest.
    pub pretax_income_usd: f64,
    /// Degree of operating leverage; `None` if EBIT is 0.
    pub dol: Option<f64>,
    /// Degree of financial leverage; `None` if pre-tax income is 0.
    pub dfl: Option<f64>,
    /// Degree of combined leverage; `None` if pre-tax income is 0.
    pub dcl: Option<f64>,
}

pub fn analyze(input: &LeverageInput) -> LeverageResult {
    let cm = input.sales_usd - input.variable_costs_usd;
    let ebit = cm - input.fixed_costs_usd;
    let pretax = ebit - input.interest_expense_usd;

    let dol = if ebit != 0.0 { Some(cm / ebit) } else { None };
    let dfl = if pretax != 0.0 { Some(ebit / pretax) } else { None };
    let dcl = if pretax != 0.0 { Some(cm / pretax) } else { None };

    LeverageResult {
        contribution_margin_usd: cm,
        ebit_usd: ebit,
        pretax_income_usd: pretax,
        dol,
        dfl,
        dcl,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn run(sales: f64, var: f64, fixed: f64, interest: f64) -> LeverageResult {
        analyze(&LeverageInput {
            sales_usd: sales,
            variable_costs_usd: var,
            fixed_costs_usd: fixed,
            interest_expense_usd: interest,
        })
    }

    #[test]
    fn contribution_margin_and_ebit() {
        let r = run(1000.0, 400.0, 300.0, 100.0);
        assert!(close(r.contribution_margin_usd, 600.0));
        assert!(close(r.ebit_usd, 300.0));
        assert!(close(r.pretax_income_usd, 200.0));
    }

    #[test]
    fn dol() {
        // 600 / 300 = 2.0.
        assert!(close(run(1000.0, 400.0, 300.0, 100.0).dol.unwrap(), 2.0));
    }

    #[test]
    fn dfl() {
        // 300 / 200 = 1.5.
        assert!(close(run(1000.0, 400.0, 300.0, 100.0).dfl.unwrap(), 1.5));
    }

    #[test]
    fn dcl_is_product_and_cm_over_pretax() {
        let r = run(1000.0, 400.0, 300.0, 100.0);
        assert!(close(r.dcl.unwrap(), 3.0));
        assert!(close(r.dcl.unwrap(), r.dol.unwrap() * r.dfl.unwrap()));
        assert!(close(r.dcl.unwrap(), r.contribution_margin_usd / r.pretax_income_usd));
    }

    #[test]
    fn no_fixed_costs_no_interest_is_unit_leverage() {
        // All-variable, no fixed costs, no debt → no amplification.
        let r = run(1000.0, 400.0, 0.0, 0.0);
        assert!(close(r.dol.unwrap(), 1.0));
        assert!(close(r.dfl.unwrap(), 1.0));
        assert!(close(r.dcl.unwrap(), 1.0));
    }

    #[test]
    fn zero_ebit_guards_dol() {
        // CM exactly covers fixed costs → EBIT 0.
        let r = run(1000.0, 400.0, 600.0, 0.0);
        assert!(close(r.ebit_usd, 0.0));
        assert!(r.dol.is_none());
    }

    #[test]
    fn breakeven_pretax_guards_dfl_dcl() {
        // EBIT equals interest → pre-tax income 0.
        let r = run(1000.0, 400.0, 300.0, 300.0);
        assert!(close(r.pretax_income_usd, 0.0));
        assert!(r.dfl.is_none());
        assert!(r.dcl.is_none());
    }

    #[test]
    fn higher_fixed_costs_raise_dol() {
        let low = run(1000.0, 400.0, 100.0, 0.0);
        let high = run(1000.0, 400.0, 400.0, 0.0);
        assert!(high.dol.unwrap() > low.dol.unwrap());
    }
}
