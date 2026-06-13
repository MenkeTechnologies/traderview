//! Real (inflation-adjusted) return — what your money actually buys.
//!
//! A nominal return overstates your gain: inflation erodes purchasing power
//! and taxes take a cut. The Fisher equation gives the exact real return:
//!
//!   * real = (1 + nominal) / (1 + inflation) − 1   (exact)
//!   * real ≈ nominal − inflation                    (the common shortcut)
//!   * after-tax real = (1 + nominal×(1−tax)) / (1 + inflation) − 1
//!
//! The shortcut overstates the real return slightly; the gap grows with the
//! rates. Over a horizon, the real future value shows the principal's
//! purchasing power in today's dollars. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RealReturnInput {
    pub nominal_return_pct: f64,
    pub inflation_pct: f64,
    /// Tax rate on the return (0 ⇒ pre-tax).
    #[serde(default)]
    pub tax_rate_pct: f64,
    /// Principal and horizon for the purchasing-power projection (optional).
    #[serde(default)]
    pub principal_usd: f64,
    #[serde(default)]
    pub years: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RealReturnResult {
    /// Exact Fisher real return.
    pub real_return_pct: f64,
    /// nominal − inflation (the shortcut).
    pub approx_real_return_pct: f64,
    /// After-tax nominal return.
    pub after_tax_nominal_pct: f64,
    /// After-tax real return.
    pub after_tax_real_pct: f64,
    /// Principal grown at the real return for `years` (today's dollars).
    pub real_future_value_usd: f64,
}

pub fn analyze(i: &RealReturnInput) -> RealReturnResult {
    let n = i.nominal_return_pct / 100.0;
    let infl = i.inflation_pct / 100.0;
    let tax = i.tax_rate_pct / 100.0;

    let real = (1.0 + n) / (1.0 + infl) - 1.0;
    let after_tax_nominal = n * (1.0 - tax);
    let after_tax_real = (1.0 + after_tax_nominal) / (1.0 + infl) - 1.0;

    let real_fv = i.principal_usd * (1.0 + real).powf(i.years.max(0.0));

    RealReturnResult {
        real_return_pct: real * 100.0,
        approx_real_return_pct: i.nominal_return_pct - i.inflation_pct,
        after_tax_nominal_pct: after_tax_nominal * 100.0,
        after_tax_real_pct: after_tax_real * 100.0,
        real_future_value_usd: real_fv,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inp(nom: f64, infl: f64) -> RealReturnInput {
        RealReturnInput {
            nominal_return_pct: nom,
            inflation_pct: infl,
            tax_rate_pct: 0.0,
            principal_usd: 0.0,
            years: 0.0,
        }
    }

    #[test]
    fn fisher_exact_real_return() {
        // (1.07/1.03) − 1 = 3.8835%.
        let r = analyze(&inp(7.0, 3.0));
        assert!((r.real_return_pct - ((1.07 / 1.03 - 1.0) * 100.0)).abs() < 1e-9);
        assert!((r.real_return_pct - 3.8835).abs() < 1e-3);
    }

    #[test]
    fn approx_is_simple_difference() {
        let r = analyze(&inp(7.0, 3.0));
        assert!((r.approx_real_return_pct - 4.0).abs() < 1e-9);
    }

    #[test]
    fn approx_overstates_exact() {
        let r = analyze(&inp(7.0, 3.0));
        assert!(r.approx_real_return_pct > r.real_return_pct);
    }

    #[test]
    fn after_tax_nominal_applies_tax() {
        let r = analyze(&RealReturnInput { tax_rate_pct: 15.0, ..inp(7.0, 3.0) });
        assert!((r.after_tax_nominal_pct - 7.0 * 0.85).abs() < 1e-9);
    }

    #[test]
    fn after_tax_real_below_pretax_real() {
        let r = analyze(&RealReturnInput { tax_rate_pct: 15.0, ..inp(7.0, 3.0) });
        assert!(r.after_tax_real_pct < r.real_return_pct);
    }

    #[test]
    fn zero_inflation_real_equals_nominal() {
        let r = analyze(&inp(7.0, 0.0));
        assert!((r.real_return_pct - 7.0).abs() < 1e-9);
    }

    #[test]
    fn negative_real_when_inflation_exceeds_nominal() {
        // 2% nominal, 5% inflation → losing purchasing power.
        let r = analyze(&inp(2.0, 5.0));
        assert!(r.real_return_pct < 0.0);
    }

    #[test]
    fn real_future_value_uses_real_return() {
        let r = analyze(&RealReturnInput { principal_usd: 100_000.0, years: 20.0, ..inp(7.0, 3.0) });
        let real: f64 = 1.07 / 1.03 - 1.0;
        assert!((r.real_future_value_usd - 100_000.0 * (1.0 + real).powi(20)).abs() < 1e-2);
    }
}
