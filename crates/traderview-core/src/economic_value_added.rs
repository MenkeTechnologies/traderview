//! Economic Value Added (EVA) — the Stern Stewart measure of economic profit:
//! the after-tax operating profit a business earns above the cost of the capital
//! employed to earn it.
//!
//! ```text
//! NOPAT          = EBIT × (1 − tax rate)
//! capital charge = invested capital × WACC
//! EVA            = NOPAT − capital charge
//!                = (ROIC − WACC) × invested capital
//! ```
//!
//! Positive EVA means the business returns more than its capital costs (it
//! creates value); negative means it destroys value even if accounting profit is
//! positive. Distinct from `free-cash-flow` (cash generation), `dupont-roe`
//! (return on equity), and `wacc` (the discount rate this consumes).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct EvaInput {
    /// Earnings before interest and taxes.
    pub ebit_usd: f64,
    pub tax_rate_pct: f64,
    /// Invested capital (debt + equity employed in operations).
    pub invested_capital_usd: f64,
    /// Weighted average cost of capital, percent.
    pub wacc_pct: f64,
    /// Revenue, for the EVA margin. 0 → margin omitted.
    #[serde(default)]
    pub revenue_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct EvaResult {
    /// EBIT × (1 − tax rate).
    pub nopat_usd: f64,
    /// Invested capital × WACC.
    pub capital_charge_usd: f64,
    pub eva_usd: f64,
    /// NOPAT / invested capital, percent. None if no capital.
    pub roic_pct: Option<f64>,
    /// ROIC − WACC, the value spread. None if ROIC is undefined.
    pub eva_spread_pct: Option<f64>,
    /// EVA / revenue, percent. None if no revenue.
    pub eva_margin_pct: Option<f64>,
    pub creates_value: bool,
}

pub fn analyze(input: &EvaInput) -> EvaResult {
    let nopat = input.ebit_usd * (1.0 - input.tax_rate_pct / 100.0);
    let capital_charge = input.invested_capital_usd * input.wacc_pct / 100.0;
    let eva = nopat - capital_charge;

    let roic = if input.invested_capital_usd != 0.0 {
        Some(nopat / input.invested_capital_usd * 100.0)
    } else {
        None
    };

    EvaResult {
        nopat_usd: nopat,
        capital_charge_usd: capital_charge,
        eva_usd: eva,
        roic_pct: roic,
        eva_spread_pct: roic.map(|r| r - input.wacc_pct),
        eva_margin_pct: if input.revenue_usd != 0.0 {
            Some(eva / input.revenue_usd * 100.0)
        } else {
            None
        },
        creates_value: eva > 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-2
    }

    fn base() -> EvaInput {
        EvaInput {
            ebit_usd: 1_000_000.0,
            tax_rate_pct: 25.0,
            invested_capital_usd: 5_000_000.0,
            wacc_pct: 8.0,
            revenue_usd: 4_000_000.0,
        }
    }

    #[test]
    fn nopat_and_charge() {
        let r = analyze(&base());
        // 1,000,000 × 0.75 = 750,000; 5,000,000 × 0.08 = 400,000.
        assert!(close(r.nopat_usd, 750_000.0));
        assert!(close(r.capital_charge_usd, 400_000.0));
    }

    #[test]
    fn eva_positive_creates_value() {
        let r = analyze(&base());
        assert!(close(r.eva_usd, 350_000.0));
        assert!(r.creates_value);
    }

    #[test]
    fn roic_and_spread() {
        let r = analyze(&base());
        // 750,000 / 5,000,000 = 15%; spread 15 − 8 = 7%.
        assert!(close(r.roic_pct.unwrap(), 15.0));
        assert!(close(r.eva_spread_pct.unwrap(), 7.0));
    }

    #[test]
    fn eva_margin() {
        let r = analyze(&base());
        // 350,000 / 4,000,000 = 8.75%.
        assert!(close(r.eva_margin_pct.unwrap(), 8.75));
    }

    #[test]
    fn spread_identity() {
        let r = analyze(&base());
        // EVA = (ROIC − WACC)/100 × invested capital.
        let from_spread = r.eva_spread_pct.unwrap() / 100.0 * 5_000_000.0;
        assert!(close(r.eva_usd, from_spread));
    }

    #[test]
    fn negative_eva_destroys_value() {
        let r = analyze(&EvaInput {
            ebit_usd: 200_000.0,
            ..base()
        });
        // nopat 150,000 − charge 400,000 = −250,000.
        assert!(close(r.eva_usd, -250_000.0));
        assert!(!r.creates_value);
        assert!(close(r.eva_spread_pct.unwrap(), -5.0));
    }

    #[test]
    fn breakeven_when_roic_equals_wacc() {
        // EBIT so NOPAT = capital charge (400,000 / 0.75).
        let r = analyze(&EvaInput {
            ebit_usd: 533_333.333333,
            ..base()
        });
        assert!(close(r.eva_usd, 0.0));
        assert!(close(r.eva_spread_pct.unwrap(), 0.0));
        assert!(!r.creates_value);
    }

    #[test]
    fn zero_capital_no_roic() {
        let r = analyze(&EvaInput {
            invested_capital_usd: 0.0,
            ..base()
        });
        assert!(r.roic_pct.is_none());
        assert!(r.eva_spread_pct.is_none());
        // No capital charge, so EVA equals NOPAT.
        assert!(close(r.eva_usd, 750_000.0));
    }

    #[test]
    fn no_revenue_no_margin() {
        let r = analyze(&EvaInput {
            revenue_usd: 0.0,
            ..base()
        });
        assert!(r.eva_margin_pct.is_none());
    }
}
