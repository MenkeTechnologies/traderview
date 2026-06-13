//! Weighted Average Cost of Capital (WACC) — the blended after-tax rate a firm
//! pays its capital providers, and the standard discount rate for a DCF.
//!
//! ```text
//! WACC = (E/V)·Re + (D/V)·Rd·(1 − tax)
//!   E, D = market value of equity, debt;  V = E + D
//!   Re   = cost of equity (supplied, or via CAPM)
//!   Rd   = pre-tax cost of debt; interest is tax-deductible, hence (1 − tax)
//! ```
//!
//! The cost of equity can be given directly or derived from CAPM:
//! `Re = risk_free + beta·(market_return − risk_free)`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct WaccInput {
    pub market_value_equity_usd: f64,
    pub market_value_debt_usd: f64,
    /// Cost of equity, percent (used when `use_capm` is false).
    #[serde(default)]
    pub cost_of_equity_pct: f64,
    /// Pre-tax cost of debt, percent.
    pub cost_of_debt_pct: f64,
    /// Marginal tax rate, percent (debt's tax shield).
    pub tax_rate_pct: f64,
    /// Derive the cost of equity from CAPM instead of `cost_of_equity_pct`.
    #[serde(default)]
    pub use_capm: bool,
    #[serde(default)]
    pub risk_free_pct: f64,
    #[serde(default)]
    pub beta: f64,
    #[serde(default)]
    pub market_return_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WaccResult {
    /// Cost of equity actually used (CAPM result or the supplied value).
    pub cost_of_equity_used_pct: f64,
    /// Equity weight E/V, percent.
    pub weight_equity_pct: f64,
    /// Debt weight D/V, percent.
    pub weight_debt_pct: f64,
    /// Rd·(1 − tax), percent.
    pub after_tax_cost_of_debt_pct: f64,
    /// The blended WACC, percent.
    pub wacc_pct: f64,
}

pub fn analyze(input: &WaccInput) -> WaccResult {
    let re = if input.use_capm {
        input.risk_free_pct + input.beta * (input.market_return_pct - input.risk_free_pct)
    } else {
        input.cost_of_equity_pct
    };

    let v = input.market_value_equity_usd + input.market_value_debt_usd;
    let (we, wd) = if v > 0.0 {
        (
            input.market_value_equity_usd / v,
            input.market_value_debt_usd / v,
        )
    } else {
        (0.0, 0.0)
    };

    let after_tax_rd = input.cost_of_debt_pct * (1.0 - input.tax_rate_pct / 100.0);
    let wacc = we * re + wd * after_tax_rd;

    WaccResult {
        cost_of_equity_used_pct: re,
        weight_equity_pct: we * 100.0,
        weight_debt_pct: wd * 100.0,
        after_tax_cost_of_debt_pct: after_tax_rd,
        wacc_pct: wacc,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn direct(e: f64, d: f64, re: f64, rd: f64, tax: f64) -> WaccResult {
        analyze(&WaccInput {
            market_value_equity_usd: e,
            market_value_debt_usd: d,
            cost_of_equity_pct: re,
            cost_of_debt_pct: rd,
            tax_rate_pct: tax,
            use_capm: false,
            risk_free_pct: 0.0,
            beta: 0.0,
            market_return_pct: 0.0,
        })
    }

    #[test]
    fn weights() {
        let r = direct(600.0, 400.0, 10.0, 5.0, 21.0);
        assert!(close(r.weight_equity_pct, 60.0));
        assert!(close(r.weight_debt_pct, 40.0));
    }

    #[test]
    fn after_tax_cost_of_debt() {
        // 5% × (1 − 0.21) = 3.95%.
        let r = direct(600.0, 400.0, 10.0, 5.0, 21.0);
        assert!(close(r.after_tax_cost_of_debt_pct, 3.95));
    }

    #[test]
    fn wacc_direct() {
        // 0.6×10 + 0.4×3.95 = 7.58%.
        let r = direct(600.0, 400.0, 10.0, 5.0, 21.0);
        assert!(close(r.wacc_pct, 7.58));
        assert!(close(r.cost_of_equity_used_pct, 10.0));
    }

    #[test]
    fn capm_cost_of_equity() {
        // 4 + 1.2×(10 − 4) = 11.2%.
        let r = analyze(&WaccInput {
            market_value_equity_usd: 600.0,
            market_value_debt_usd: 400.0,
            cost_of_equity_pct: 0.0,
            cost_of_debt_pct: 5.0,
            tax_rate_pct: 21.0,
            use_capm: true,
            risk_free_pct: 4.0,
            beta: 1.2,
            market_return_pct: 10.0,
        });
        assert!(close(r.cost_of_equity_used_pct, 11.2));
        // 0.6×11.2 + 0.4×3.95 = 8.30%.
        assert!(close(r.wacc_pct, 8.30));
    }

    #[test]
    fn all_equity_wacc_is_cost_of_equity() {
        let r = direct(1000.0, 0.0, 9.0, 5.0, 21.0);
        assert!(close(r.wacc_pct, 9.0));
        assert!(close(r.weight_equity_pct, 100.0));
    }

    #[test]
    fn higher_tax_lowers_wacc() {
        let low = direct(600.0, 400.0, 10.0, 5.0, 10.0);
        let high = direct(600.0, 400.0, 10.0, 5.0, 35.0);
        assert!(high.wacc_pct < low.wacc_pct);
    }

    #[test]
    fn all_debt_uses_after_tax_rd() {
        let r = direct(0.0, 1000.0, 10.0, 5.0, 21.0);
        assert!(close(r.wacc_pct, 3.95));
        assert!(close(r.weight_debt_pct, 100.0));
    }

    #[test]
    fn zero_value_guards() {
        let r = direct(0.0, 0.0, 10.0, 5.0, 21.0);
        assert!(close(r.weight_equity_pct, 0.0));
        assert!(close(r.wacc_pct, 0.0));
    }
}
