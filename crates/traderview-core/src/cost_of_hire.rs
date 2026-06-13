//! True cost of hire — fully-loaded W-2 employee vs 1099 contractor.
//!
//! A salary is only part of what an employee costs. The fully-loaded cost
//! adds the employer's payroll taxes (employer FICA ~7.65%, plus
//! unemployment and workers' comp), benefits (health, retirement match),
//! and overhead (equipment, software, office, training). The "burden" is how
//! much that loading adds over base pay — commonly 1.25×–1.4× salary.
//!
//! A 1099 contractor bills a rate and covers their own taxes, benefits, and
//! equipment, so their cost is just the contract spend — but usually at a
//! higher headline rate. This compares the two on total annual cost and on
//! an effective hourly basis (the employee's paid time off reduces their
//! productive hours, raising their true hourly cost). Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CostOfHireInput {
    pub base_salary_usd: f64,
    /// Employer payroll tax rate (employer FICA ~7.65%, + FUTA/SUTA).
    pub employer_payroll_tax_pct: f64,
    /// Annual benefits cost (health, dental, life, …).
    #[serde(default)]
    pub annual_benefits_usd: f64,
    /// Retirement match as a percent of base salary.
    #[serde(default)]
    pub retirement_match_pct: f64,
    /// Workers' comp insurance as a percent of base salary.
    #[serde(default)]
    pub workers_comp_pct: f64,
    /// Equipment, software, office, training, etc.
    #[serde(default)]
    pub other_overhead_usd: f64,
    /// Paid days off (reduce productive hours for the hourly comparison).
    #[serde(default)]
    pub pto_days: f64,
    /// Scheduled annual hours (2080 = 40h × 52w).
    pub annual_hours: f64,
    /// The 1099 contractor's total annual cost (rate × hours, or contract).
    pub contractor_annual_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CostOfHireResult {
    pub employer_payroll_tax_usd: f64,
    pub retirement_match_usd: f64,
    pub workers_comp_usd: f64,
    /// Base + payroll tax + benefits + match + workers' comp + overhead.
    pub total_w2_cost_usd: f64,
    /// How much the loading adds over base pay, as a percent.
    pub burden_pct: f64,
    /// W-2 cost ÷ productive hours (scheduled hours − PTO).
    pub w2_effective_hourly_usd: f64,
    pub contractor_effective_hourly_usd: f64,
    /// W-2 total − contractor total (positive ⇒ contractor is cheaper).
    pub w2_minus_contractor_usd: f64,
    /// True when the W-2 employee costs less than the contractor.
    pub w2_cheaper: bool,
}

pub fn analyze(i: &CostOfHireInput) -> CostOfHireResult {
    let base = i.base_salary_usd.max(0.0);
    let payroll_tax = base * i.employer_payroll_tax_pct / 100.0;
    let retirement = base * i.retirement_match_pct / 100.0;
    let workers_comp = base * i.workers_comp_pct / 100.0;

    let total_w2 =
        base + payroll_tax + i.annual_benefits_usd + retirement + workers_comp + i.other_overhead_usd;
    let burden_pct = if base > 0.0 { (total_w2 - base) / base * 100.0 } else { 0.0 };

    let productive_hours = (i.annual_hours - i.pto_days * 8.0).max(1.0);
    let w2_hourly = total_w2 / productive_hours;
    let contractor_hourly = if i.annual_hours > 0.0 {
        i.contractor_annual_usd / i.annual_hours
    } else {
        0.0
    };

    let diff = total_w2 - i.contractor_annual_usd;

    CostOfHireResult {
        employer_payroll_tax_usd: payroll_tax,
        retirement_match_usd: retirement,
        workers_comp_usd: workers_comp,
        total_w2_cost_usd: total_w2,
        burden_pct,
        w2_effective_hourly_usd: w2_hourly,
        contractor_effective_hourly_usd: contractor_hourly,
        w2_minus_contractor_usd: diff,
        w2_cheaper: diff < 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> CostOfHireInput {
        CostOfHireInput {
            base_salary_usd: 100_000.0,
            employer_payroll_tax_pct: 7.65,
            annual_benefits_usd: 12_000.0,
            retirement_match_pct: 4.0,
            workers_comp_pct: 1.0,
            other_overhead_usd: 5_000.0,
            pto_days: 15.0,
            annual_hours: 2_080.0,
            contractor_annual_usd: 150_000.0,
        }
    }

    #[test]
    fn employer_payroll_tax() {
        let r = analyze(&base());
        assert!((r.employer_payroll_tax_usd - 7_650.0).abs() < 1e-6); // 100k × 7.65%
    }

    #[test]
    fn total_w2_sums_all_components() {
        // 100k + 7.65k + 12k + 4k + 1k + 5k = 129.65k.
        let r = analyze(&base());
        assert!((r.retirement_match_usd - 4_000.0).abs() < 1e-6);
        assert!((r.workers_comp_usd - 1_000.0).abs() < 1e-6);
        assert!((r.total_w2_cost_usd - 129_650.0).abs() < 1e-6);
    }

    #[test]
    fn burden_is_loading_over_base() {
        // (129.65k − 100k)/100k = 29.65%.
        let r = analyze(&base());
        assert!((r.burden_pct - 29.65).abs() < 1e-9);
    }

    #[test]
    fn pto_reduces_productive_hours_raising_hourly() {
        // 15 PTO days × 8 = 120h; productive = 2080 − 120 = 1960.
        let r = analyze(&base());
        assert!((r.w2_effective_hourly_usd - 129_650.0 / 1_960.0).abs() < 1e-6);
        // No-PTO version has a lower hourly (more productive hours).
        let no_pto = analyze(&CostOfHireInput { pto_days: 0.0, ..base() });
        assert!(no_pto.w2_effective_hourly_usd < r.w2_effective_hourly_usd);
    }

    #[test]
    fn contractor_hourly_uses_scheduled_hours() {
        // 150k / 2080 ≈ 72.12.
        let r = analyze(&base());
        assert!((r.contractor_effective_hourly_usd - 150_000.0 / 2_080.0).abs() < 1e-6);
    }

    #[test]
    fn w2_cheaper_when_total_below_contractor() {
        // W-2 129.65k < contractor 150k → W-2 cheaper.
        let r = analyze(&base());
        assert!(r.w2_cheaper);
        assert!((r.w2_minus_contractor_usd - (-20_350.0)).abs() < 1e-6);
    }

    #[test]
    fn cheap_contractor_flips_decision() {
        let r = analyze(&CostOfHireInput { contractor_annual_usd: 90_000.0, ..base() });
        assert!(!r.w2_cheaper); // contractor 90k < W-2 129.65k
        assert!(r.w2_minus_contractor_usd > 0.0);
    }

    #[test]
    fn zero_base_guards_burden() {
        let r = analyze(&CostOfHireInput { base_salary_usd: 0.0, ..base() });
        assert!(r.burden_pct.abs() < 1e-9);
    }
}
