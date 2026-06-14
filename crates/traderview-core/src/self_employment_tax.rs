//! Self-employment tax — the Social Security and Medicare tax a self-employed
//! person owes (Schedule SE). Net self-employment earnings are 92.35% of net
//! profit; the Social Security portion (12.4%) applies up to the annual wage base
//! and the Medicare portion (2.9%) applies to all of it. One-half of the SE tax is
//! an above-the-line income-tax deduction. Wage base and rates are inputs so the
//! module stays correct across tax years. Distinct from the W-2 paycheck modules.
//! Not tax advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SeTaxInput {
    /// Net profit from self-employment (Schedule C).
    pub net_profit_usd: f64,
    /// Social Security wage base for the year (e.g. 168600 for 2024).
    #[serde(default = "default_wage_base")]
    pub ss_wage_base_usd: f64,
    /// Social Security rate, percent.
    #[serde(default = "default_ss_rate")]
    pub ss_rate_pct: f64,
    /// Medicare rate, percent.
    #[serde(default = "default_med_rate")]
    pub medicare_rate_pct: f64,
    /// Net-earnings factor, percent (92.35%).
    #[serde(default = "default_factor")]
    pub net_earnings_factor_pct: f64,
}

fn default_wage_base() -> f64 {
    168_600.0
}
fn default_ss_rate() -> f64 {
    12.4
}
fn default_med_rate() -> f64 {
    2.9
}
fn default_factor() -> f64 {
    92.35
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct SeTaxReport {
    /// Net profit × 92.35%.
    pub net_se_earnings_usd: f64,
    /// Social Security portion (capped at the wage base).
    pub social_security_usd: f64,
    pub medicare_usd: f64,
    /// SS + Medicare.
    pub se_tax_usd: f64,
    /// One-half of SE tax — the income-tax deduction.
    pub deductible_half_usd: f64,
    /// True when net SE earnings reached the Social Security wage base.
    pub ss_capped: bool,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

pub fn generate(i: &SeTaxInput) -> SeTaxReport {
    if i.net_profit_usd <= 0.0 {
        return SeTaxReport::default();
    }
    let nse = i.net_profit_usd * i.net_earnings_factor_pct / 100.0;
    let ss_base = nse.min(i.ss_wage_base_usd);
    let ss = ss_base * i.ss_rate_pct / 100.0;
    let med = nse * i.medicare_rate_pct / 100.0;
    let se = ss + med;
    SeTaxReport {
        net_se_earnings_usd: cents(nse),
        social_security_usd: cents(ss),
        medicare_usd: cents(med),
        se_tax_usd: cents(se),
        deductible_half_usd: cents(se * 0.5),
        ss_capped: nse >= i.ss_wage_base_usd,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> SeTaxInput {
        SeTaxInput {
            net_profit_usd: 100_000.0,
            ss_wage_base_usd: 168_600.0,
            ss_rate_pct: 12.4,
            medicare_rate_pct: 2.9,
            net_earnings_factor_pct: 92.35,
        }
    }

    #[test]
    fn standard_se_tax() {
        let d = generate(&base());
        assert!(close(d.net_se_earnings_usd, 92_350.0));
        assert!(close(d.social_security_usd, 11_451.40));
        assert!(close(d.medicare_usd, 2_678.15));
        assert!(close(d.se_tax_usd, 14_129.55));
        assert!(close(d.deductible_half_usd, 7_064.78));
        assert!(!d.ss_capped);
    }

    #[test]
    fn ss_capped_above_wage_base() {
        let d = generate(&SeTaxInput { net_profit_usd: 200_000.0, ..base() });
        // SS caps at the wage base; Medicare keeps rising.
        assert!(d.ss_capped);
        assert!(close(d.social_security_usd, 20_906.40));
        assert!(close(d.medicare_usd, 5_356.30));
        assert!(close(d.se_tax_usd, 26_262.70));
    }

    #[test]
    fn deductible_is_half() {
        let d = generate(&base());
        assert!(close(d.deductible_half_usd, d.se_tax_usd / 2.0));
    }

    #[test]
    fn no_profit_no_tax() {
        let d = generate(&SeTaxInput { net_profit_usd: 0.0, ..base() });
        assert!(close(d.se_tax_usd, 0.0));
    }
}
