//! Years to financial independence — the savings-rate-to-timeline math.
//!
//! Financial independence is when your portfolio can fund your spending
//! indefinitely at a safe withdrawal rate: the **FI number** = annual
//! expenses / SWR (4% → 25× expenses). Starting from current savings and
//! adding the gap between income and expenses each year (grown at an
//! expected return), the years-to-FI is when the balance reaches that
//! number. The dominant lever is the **savings rate** — what fraction of
//! income you keep — far more than the return.
//!
//! Pure compute (year-by-year, end-of-year contributions, capped at 100y).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct YearsToFiInput {
    pub current_savings_usd: f64,
    pub annual_income_usd: f64,
    pub annual_expenses_usd: f64,
    pub annual_return_pct: f64,
    /// Safe withdrawal rate defining the FI number (default 4%).
    pub safe_withdrawal_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct YearsToFiResult {
    /// Income − expenses: what's saved each year.
    pub annual_savings_usd: f64,
    pub savings_rate_pct: f64,
    /// Annual expenses / SWR — the portfolio needed to retire.
    pub fi_number_usd: f64,
    /// Years until the balance reaches the FI number; `None` if never (≤100y).
    pub years_to_fi: Option<u32>,
    /// True when current savings already meet the FI number.
    pub already_fi: bool,
}

const HORIZON: u32 = 100;

pub fn analyze(i: &YearsToFiInput) -> YearsToFiResult {
    let r = i.annual_return_pct / 100.0;
    let swr = if i.safe_withdrawal_rate_pct > 0.0 { i.safe_withdrawal_rate_pct } else { 4.0 };
    let annual_savings = i.annual_income_usd - i.annual_expenses_usd;
    let savings_rate = if i.annual_income_usd > 0.0 {
        annual_savings / i.annual_income_usd * 100.0
    } else {
        0.0
    };
    let fi_number = i.annual_expenses_usd / (swr / 100.0);

    let mut balance = i.current_savings_usd;
    let already_fi = balance >= fi_number;
    let mut years = if already_fi { Some(0u32) } else { None };

    if years.is_none() {
        for y in 1..=HORIZON {
            balance = balance * (1.0 + r) + annual_savings;
            if balance >= fi_number {
                years = Some(y);
                break;
            }
        }
    }

    YearsToFiResult {
        annual_savings_usd: annual_savings,
        savings_rate_pct: savings_rate,
        fi_number_usd: fi_number,
        years_to_fi: years,
        already_fi,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inp(savings: f64, income: f64, expenses: f64, ret: f64) -> YearsToFiInput {
        YearsToFiInput {
            current_savings_usd: savings,
            annual_income_usd: income,
            annual_expenses_usd: expenses,
            annual_return_pct: ret,
            safe_withdrawal_rate_pct: 4.0,
        }
    }

    #[test]
    fn fi_number_is_expenses_over_swr() {
        // 40k / 4% = 1,000,000.
        let r = analyze(&inp(0.0, 80_000.0, 40_000.0, 0.0));
        assert!((r.fi_number_usd - 1_000_000.0).abs() < 1e-6);
    }

    #[test]
    fn savings_rate_and_annual_savings() {
        let r = analyze(&inp(0.0, 80_000.0, 40_000.0, 0.0));
        assert!((r.annual_savings_usd - 40_000.0).abs() < 1e-6);
        assert!((r.savings_rate_pct - 50.0).abs() < 1e-9);
    }

    #[test]
    fn already_fi_is_zero_years() {
        // 1M saved, FI number 1M → already there.
        let r = analyze(&inp(1_000_000.0, 80_000.0, 40_000.0, 5.0));
        assert!(r.already_fi);
        assert_eq!(r.years_to_fi, Some(0));
    }

    #[test]
    fn zero_return_is_linear() {
        // FI 750k, save 20k/yr, no return → 38 years (37×20k=740k<750k, 38×20k=760k).
        let r = analyze(&inp(0.0, 50_000.0, 30_000.0, 0.0));
        assert!((r.fi_number_usd - 750_000.0).abs() < 1e-6);
        assert_eq!(r.years_to_fi, Some(38));
    }

    #[test]
    fn higher_savings_rate_fewer_years() {
        let lean = analyze(&inp(0.0, 80_000.0, 60_000.0, 5.0)); // save 20k
        let aggressive = analyze(&inp(0.0, 80_000.0, 30_000.0, 5.0)); // save 50k
        assert!(aggressive.years_to_fi.unwrap() < lean.years_to_fi.unwrap());
    }

    #[test]
    fn higher_return_fewer_years() {
        let low = analyze(&inp(0.0, 80_000.0, 40_000.0, 3.0));
        let high = analyze(&inp(0.0, 80_000.0, 40_000.0, 8.0));
        assert!(high.years_to_fi.unwrap() < low.years_to_fi.unwrap());
    }

    #[test]
    fn expenses_at_or_above_income_never_via_saving() {
        // No savings, no return → never reaches FI within the horizon.
        let r = analyze(&inp(0.0, 50_000.0, 50_000.0, 0.0));
        assert!(r.annual_savings_usd.abs() < 1e-9);
        assert_eq!(r.years_to_fi, None);
    }

    #[test]
    fn return_compounds_toward_fi() {
        // With a return, FI arrives sooner than the linear no-return case.
        let flat = analyze(&inp(0.0, 80_000.0, 40_000.0, 0.0));
        let growth = analyze(&inp(0.0, 80_000.0, 40_000.0, 7.0));
        assert!(growth.years_to_fi.unwrap() < flat.years_to_fi.unwrap());
    }
}
