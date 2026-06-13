//! Real raise — whether a pay raise actually beats inflation, and the raise
//! needed just to keep pace.
//!
//! A nominal raise only buys more if it outpaces inflation. The real change
//! uses the Fisher relation:
//!
//! ```text
//! inflation-adjusted salary = current × (1 + inflation)   (break-even pay)
//! real raise = (1 + raise) / (1 + inflation) − 1
//! ```
//!
//! A raise equal to inflation is a wash; below it, a real pay cut.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RealRaiseInput {
    pub current_salary_usd: f64,
    pub inflation_pct: f64,
    pub raise_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RealRaiseResult {
    /// Raise needed just to keep pace with inflation (= inflation rate).
    pub raise_to_keep_pace_pct: f64,
    /// Salary that would merely maintain purchasing power.
    pub inflation_adjusted_salary_usd: f64,
    pub new_salary_usd: f64,
    /// New − current (nominal dollars).
    pub nominal_change_usd: f64,
    /// Inflation-adjusted change, percent (the real raise).
    pub real_raise_pct: f64,
    /// New salary − inflation-adjusted salary (purchasing-power gain/loss).
    pub real_change_usd: f64,
    /// Whether the raise beats inflation.
    pub is_real_raise: bool,
}

pub fn analyze(input: &RealRaiseInput) -> RealRaiseResult {
    let infl = input.inflation_pct / 100.0;
    let raise = input.raise_pct / 100.0;

    let inflation_adjusted = input.current_salary_usd * (1.0 + infl);
    let new_salary = input.current_salary_usd * (1.0 + raise);
    let real_raise = if (1.0 + infl) != 0.0 {
        ((1.0 + raise) / (1.0 + infl) - 1.0) * 100.0
    } else {
        0.0
    };

    RealRaiseResult {
        raise_to_keep_pace_pct: input.inflation_pct,
        inflation_adjusted_salary_usd: inflation_adjusted,
        new_salary_usd: new_salary,
        nominal_change_usd: new_salary - input.current_salary_usd,
        real_raise_pct: real_raise,
        real_change_usd: new_salary - inflation_adjusted,
        is_real_raise: input.raise_pct > input.inflation_pct,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn run(salary: f64, infl: f64, raise: f64) -> RealRaiseResult {
        analyze(&RealRaiseInput {
            current_salary_usd: salary,
            inflation_pct: infl,
            raise_pct: raise,
        })
    }

    #[test]
    fn inflation_adjusted_salary() {
        // 80,000 × 1.03 = 82,400.
        assert!(close(run(80_000.0, 3.0, 5.0).inflation_adjusted_salary_usd, 82_400.0));
    }

    #[test]
    fn new_salary_and_nominal_change() {
        let r = run(80_000.0, 3.0, 5.0);
        assert!(close(r.new_salary_usd, 84_000.0));
        assert!(close(r.nominal_change_usd, 4_000.0));
    }

    #[test]
    fn real_raise_pct() {
        // (1.05/1.03 − 1) × 100 = 1.941748%.
        assert!(close(run(80_000.0, 3.0, 5.0).real_raise_pct, 1.941748));
    }

    #[test]
    fn real_change_usd() {
        // 84,000 − 82,400 = 1,600.
        assert!(close(run(80_000.0, 3.0, 5.0).real_change_usd, 1_600.0));
    }

    #[test]
    fn beats_inflation() {
        assert!(run(80_000.0, 3.0, 5.0).is_real_raise);
    }

    #[test]
    fn real_cut_below_inflation() {
        let r = run(80_000.0, 3.0, 2.0);
        assert!(!r.is_real_raise);
        assert!(r.real_raise_pct < 0.0);
        assert!(r.real_change_usd < 0.0);
    }

    #[test]
    fn raise_equals_inflation_is_a_wash() {
        let r = run(80_000.0, 3.0, 3.0);
        assert!(close(r.real_raise_pct, 0.0));
        assert!(close(r.real_change_usd, 0.0));
        assert!(!r.is_real_raise);
    }

    #[test]
    fn keep_pace_equals_inflation() {
        assert!(close(run(80_000.0, 3.0, 5.0).raise_to_keep_pace_pct, 3.0));
    }
}
