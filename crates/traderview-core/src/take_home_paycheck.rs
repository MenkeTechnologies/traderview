//! Take-home pay — gross paycheck to net, per period and per year.
//!
//! Income-tax withholding depends on the W-4 and bracket structure (the
//! annual `income_tax_estimator` covers that), so federal and state are taken
//! here as effective rates on taxable wages. FICA is computed precisely:
//!
//! * Social Security: 6.2% on wages up to the annual wage base ($184,500 for
//!   2026), nothing above.
//! * Medicare: 1.45% on all wages, plus an additional 0.9% above the filing
//!   threshold ($200k single / $250k joint / $125k separate).
//!
//! Pre-tax deductions (traditional 401(k), HSA) reduce the income-tax base but
//! NOT the FICA base — 401(k) deferrals are still subject to payroll tax — so
//! FICA is taken on gross wages. Post-tax deductions reduce take-home only.

use serde::{Deserialize, Serialize};

fn d_ss_rate() -> f64 {
    6.2
}
fn d_ss_base() -> f64 {
    184_500.0
}
fn d_medicare_rate() -> f64 {
    1.45
}
fn d_addl_rate() -> f64 {
    0.9
}
fn d_addl_threshold() -> f64 {
    200_000.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaycheckInput {
    /// Gross pay per period.
    pub gross_per_period: f64,
    /// Pay periods per year (52 weekly, 26 biweekly, 24 semimonthly, 12 monthly).
    pub periods_per_year: f64,
    /// Pre-tax deductions per period (401k, HSA, cafeteria) — cut income-tax base.
    #[serde(default)]
    pub pre_tax_per_period: f64,
    /// Post-tax deductions per period (Roth 401k, garnishments) — cut net only.
    #[serde(default)]
    pub post_tax_per_period: f64,
    /// Effective federal income-tax rate on taxable wages, percent.
    #[serde(default)]
    pub federal_rate_pct: f64,
    /// Effective state income-tax rate on taxable wages, percent.
    #[serde(default)]
    pub state_rate_pct: f64,
    #[serde(default = "d_ss_rate")]
    pub ss_rate_pct: f64,
    #[serde(default = "d_ss_base")]
    pub ss_wage_base: f64,
    #[serde(default = "d_medicare_rate")]
    pub medicare_rate_pct: f64,
    #[serde(default = "d_addl_rate")]
    pub addl_medicare_rate_pct: f64,
    #[serde(default = "d_addl_threshold")]
    pub addl_medicare_threshold: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PaycheckResult {
    pub gross_annual: f64,
    /// Wages subject to income tax (gross − pre-tax deductions).
    pub taxable_annual: f64,
    pub federal_annual: f64,
    pub state_annual: f64,
    pub social_security_annual: f64,
    pub medicare_annual: f64,
    pub pre_tax_annual: f64,
    pub post_tax_annual: f64,
    /// Federal + state + FICA.
    pub total_tax_annual: f64,
    pub take_home_annual: f64,
    pub take_home_per_period: f64,
    /// Total tax as a percent of gross.
    pub effective_tax_rate_pct: f64,
    /// Take-home as a percent of gross.
    pub take_home_pct: f64,
}

pub fn analyze(input: &PaycheckInput) -> PaycheckResult {
    let periods = input.periods_per_year.max(0.0);
    let gross = input.gross_per_period * periods;
    let pre_tax = input.pre_tax_per_period * periods;
    let post_tax = input.post_tax_per_period * periods;

    let taxable = (gross - pre_tax).max(0.0);
    let federal = taxable * input.federal_rate_pct / 100.0;
    let state = taxable * input.state_rate_pct / 100.0;

    // FICA is on gross wages — pre-tax 401(k) does not reduce it.
    let ss = input.ss_rate_pct / 100.0 * gross.min(input.ss_wage_base);
    let medicare = input.medicare_rate_pct / 100.0 * gross
        + input.addl_medicare_rate_pct / 100.0 * (gross - input.addl_medicare_threshold).max(0.0);

    let total_tax = federal + state + ss + medicare;
    let take_home = gross - pre_tax - total_tax - post_tax;

    PaycheckResult {
        gross_annual: gross,
        taxable_annual: taxable,
        federal_annual: federal,
        state_annual: state,
        social_security_annual: ss,
        medicare_annual: medicare,
        pre_tax_annual: pre_tax,
        post_tax_annual: post_tax,
        total_tax_annual: total_tax,
        take_home_annual: take_home,
        take_home_per_period: if periods > 0.0 {
            take_home / periods
        } else {
            0.0
        },
        effective_tax_rate_pct: if gross > 0.0 {
            total_tax / gross * 100.0
        } else {
            0.0
        },
        take_home_pct: if gross > 0.0 {
            take_home / gross * 100.0
        } else {
            0.0
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn base() -> PaycheckInput {
        PaycheckInput {
            gross_per_period: 5000.0,
            periods_per_year: 24.0,
            pre_tax_per_period: 500.0,
            post_tax_per_period: 0.0,
            federal_rate_pct: 12.0,
            state_rate_pct: 5.0,
            ss_rate_pct: 6.2,
            ss_wage_base: 184_500.0,
            medicare_rate_pct: 1.45,
            addl_medicare_rate_pct: 0.9,
            addl_medicare_threshold: 200_000.0,
        }
    }

    #[test]
    fn annual_gross_and_taxable() {
        let r = analyze(&base());
        assert!(close(r.gross_annual, 120_000.0));
        // 120k − 12k pre-tax = 108k taxable.
        assert!(close(r.taxable_annual, 108_000.0));
    }

    #[test]
    fn fica_on_full_gross_not_taxable() {
        let r = analyze(&base());
        // SS 6.2% × 120k = 7,440 (under base); Medicare 1.45% × 120k = 1,740.
        assert!(close(r.social_security_annual, 7_440.0));
        assert!(close(r.medicare_annual, 1_740.0));
    }

    #[test]
    fn income_tax_on_taxable() {
        let r = analyze(&base());
        // Federal 12% × 108k = 12,960; state 5% × 108k = 5,400.
        assert!(close(r.federal_annual, 12_960.0));
        assert!(close(r.state_annual, 5_400.0));
    }

    #[test]
    fn take_home_and_per_period() {
        let r = analyze(&base());
        // 120k − 12k − (12960+5400+7440+1740) = 80,460; /24 = 3,352.50.
        assert!(close(r.total_tax_annual, 27_540.0));
        assert!(close(r.take_home_annual, 80_460.0));
        assert!(close(r.take_home_per_period, 3_352.50));
    }

    #[test]
    fn ss_caps_at_wage_base() {
        let mut i = base();
        i.gross_per_period = 25_000.0; // 600k annual, over the 184,500 base.
        let r = analyze(&i);
        assert!(close(r.social_security_annual, 0.062 * 184_500.0));
    }

    #[test]
    fn additional_medicare_over_threshold() {
        let mut i = base();
        i.gross_per_period = 25_000.0; // 600k annual.
        let r = analyze(&i);
        // 1.45% × 600k + 0.9% × (600k − 200k) = 8,700 + 3,600 = 12,300.
        assert!(close(r.medicare_annual, 12_300.0));
    }

    #[test]
    fn post_tax_cuts_only_take_home() {
        let with = {
            let mut i = base();
            i.post_tax_per_period = 200.0;
            analyze(&i)
        };
        let without = analyze(&base());
        // Taxes identical; take-home lower by exactly the post-tax annual.
        assert!(close(with.total_tax_annual, without.total_tax_annual));
        assert!(close(
            with.take_home_annual,
            without.take_home_annual - 200.0 * 24.0
        ));
    }

    #[test]
    fn effective_rate_and_zero_periods() {
        let r = analyze(&base());
        assert!(close(r.effective_tax_rate_pct, 27_540.0 / 120_000.0 * 100.0));
        let mut i = base();
        i.periods_per_year = 0.0;
        let z = analyze(&i);
        assert!(close(z.take_home_per_period, 0.0));
        assert!(close(z.effective_tax_rate_pct, 0.0));
    }
}
