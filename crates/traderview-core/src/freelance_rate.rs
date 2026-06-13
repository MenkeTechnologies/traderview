//! Freelance / contractor billing rate — the hourly rate a 1099 worker must
//! charge to take home a target, after self-employment tax, income tax,
//! business expenses, and self-funded benefits, over their billable hours.
//!
//! ```text
//! pre-tax profit needed = take-home / (1 − SE rate − income rate)
//! revenue needed        = pre-tax profit + expenses + benefits
//! hourly rate           = revenue / billable hours
//! ```
//!
//! Billable hours are well under a full year — unbillable admin, sales, and
//! time off — which is why the rate must run far above an employee's wage.

use serde::{Deserialize, Serialize};

fn d_se() -> f64 {
    15.3
}

#[derive(Debug, Clone, Deserialize)]
pub struct FreelanceInput {
    pub desired_annual_take_home_usd: f64,
    pub billable_hours_per_year: f64,
    #[serde(default)]
    pub annual_business_expenses_usd: f64,
    /// Self-funded benefits (health insurance, retirement), annual.
    #[serde(default)]
    pub annual_benefits_usd: f64,
    #[serde(default = "d_se")]
    pub self_employment_tax_rate_pct: f64,
    #[serde(default)]
    pub income_tax_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FreelanceResult {
    /// Pre-tax profit the contractor must clear to net the target.
    pub pretax_profit_needed_usd: f64,
    /// Total revenue to bill (profit + expenses + benefits).
    pub revenue_needed_usd: f64,
    /// Required billing rate per hour.
    pub required_hourly_rate_usd: f64,
    /// Taxes (SE + income) on the pre-tax profit.
    pub total_tax_usd: f64,
    /// Combined SE + income rate, percent.
    pub combined_tax_rate_pct: f64,
}

pub fn analyze(input: &FreelanceInput) -> FreelanceResult {
    let combined = (input.self_employment_tax_rate_pct + input.income_tax_rate_pct) / 100.0;

    let pretax = if combined < 1.0 {
        input.desired_annual_take_home_usd / (1.0 - combined)
    } else {
        0.0
    };
    let revenue = pretax + input.annual_business_expenses_usd + input.annual_benefits_usd;
    let hourly = if input.billable_hours_per_year > 0.0 {
        revenue / input.billable_hours_per_year
    } else {
        0.0
    };

    FreelanceResult {
        pretax_profit_needed_usd: pretax,
        revenue_needed_usd: revenue,
        required_hourly_rate_usd: hourly,
        total_tax_usd: pretax - input.desired_annual_take_home_usd,
        combined_tax_rate_pct: combined * 100.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-2
    }

    fn base() -> FreelanceInput {
        FreelanceInput {
            desired_annual_take_home_usd: 80_000.0,
            billable_hours_per_year: 1_500.0,
            annual_business_expenses_usd: 10_000.0,
            annual_benefits_usd: 12_000.0,
            self_employment_tax_rate_pct: 15.3,
            income_tax_rate_pct: 22.0,
        }
    }

    #[test]
    fn pretax_profit() {
        // 80,000 / (1 − 0.373) = 127,591.71.
        assert!(close(analyze(&base()).pretax_profit_needed_usd, 127_591.7065));
    }

    #[test]
    fn revenue_needed() {
        assert!(close(analyze(&base()).revenue_needed_usd, 149_591.7065));
    }

    #[test]
    fn required_hourly_rate() {
        // 149,591.71 / 1,500 = 99.73.
        assert!(close(analyze(&base()).required_hourly_rate_usd, 99.7278));
    }

    #[test]
    fn total_tax_and_combined_rate() {
        let r = analyze(&base());
        assert!(close(r.combined_tax_rate_pct, 37.3));
        assert!(close(r.total_tax_usd, r.pretax_profit_needed_usd - 80_000.0));
    }

    #[test]
    fn fewer_billable_hours_raise_rate() {
        let many = analyze(&base());
        let few = analyze(&FreelanceInput {
            billable_hours_per_year: 1_000.0,
            ..base()
        });
        assert!(few.required_hourly_rate_usd > many.required_hourly_rate_usd);
    }

    #[test]
    fn expenses_raise_revenue() {
        let r = analyze(&FreelanceInput {
            annual_business_expenses_usd: 30_000.0,
            ..base()
        });
        assert!(r.revenue_needed_usd > analyze(&base()).revenue_needed_usd);
    }

    #[test]
    fn higher_taxes_raise_rate() {
        let low = analyze(&base());
        let high = analyze(&FreelanceInput {
            income_tax_rate_pct: 32.0,
            ..base()
        });
        assert!(high.required_hourly_rate_usd > low.required_hourly_rate_usd);
    }

    #[test]
    fn no_tax_no_costs_rate_covers_take_home() {
        let r = analyze(&FreelanceInput {
            annual_business_expenses_usd: 0.0,
            annual_benefits_usd: 0.0,
            self_employment_tax_rate_pct: 0.0,
            income_tax_rate_pct: 0.0,
            ..base()
        });
        // 80,000 / 1,500 = 53.33.
        assert!(close(r.required_hourly_rate_usd, 80_000.0 / 1_500.0));
    }
}
