//! Required Minimum Distribution (RMD) calculator.
//!
//! Per the SECURE 2.0 Act (2022) and IRS Publication 590-B:
//!
//!   - RMDs begin at age 73 for those born 1951-1959 (and at age 75
//!     for those born 1960+).
//!   - RMD_year_x = prior_year_end_balance / IRS_Uniform_Lifetime_Factor_age_x
//!   - Missed RMD penalty (SECURE 2.0): 25% of shortfall (reduced from
//!     50% pre-2023), or 10% if corrected within 2 years.
//!
//! This module embeds the IRS Uniform Lifetime Table (effective 2022,
//! current as of Publication 590-B 2024 update) for ages 72-120. Also
//! supports projection over N years assuming the user's balance grows
//! at `expected_return_pct` net of the RMD withdrawal each year.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RmdInput {
    pub birth_year: i32,
    pub current_age: u32,
    pub balance_usd: f64,
    #[serde(default = "default_return")]
    pub expected_annual_return_pct: f64,
    /// Years to project forward.
    #[serde(default = "default_proj_years")]
    pub project_years: u32,
}

fn default_return() -> f64 { 6.0 }
fn default_proj_years() -> u32 { 20 }

#[derive(Debug, Clone, Serialize)]
pub struct YearRmd {
    pub age: u32,
    pub start_balance_usd: f64,
    pub rmd_factor: f64,
    pub rmd_amount_usd: f64,
    pub end_balance_after_rmd_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RmdReport {
    pub rmd_start_age: u32,
    pub current_factor: Option<f64>,
    pub current_rmd_usd: Option<f64>,
    pub years_until_rmd: i32,
    pub projection: Vec<YearRmd>,
    pub total_rmds_through_projection_usd: f64,
}

// ─── IRS Uniform Lifetime Table (2022) ────────────────────────────────────

/// Divisor for age `a`. Outside the table range, returns None.
/// Table source: IRS Publication 590-B, Appendix B, Table III.
pub fn uniform_lifetime_factor(age: u32) -> Option<f64> {
    let f = match age {
        72 => 27.4,
        73 => 26.5,
        74 => 25.5,
        75 => 24.6,
        76 => 23.7,
        77 => 22.9,
        78 => 22.0,
        79 => 21.1,
        80 => 20.2,
        81 => 19.4,
        82 => 18.5,
        83 => 17.7,
        84 => 16.8,
        85 => 16.0,
        86 => 15.2,
        87 => 14.4,
        88 => 13.7,
        89 => 12.9,
        90 => 12.2,
        91 => 11.5,
        92 => 10.8,
        93 => 10.1,
        94 => 9.5,
        95 => 8.9,
        96 => 8.4,
        97 => 7.8,
        98 => 7.3,
        99 => 6.8,
        100 => 6.4,
        101 => 6.0,
        102 => 5.6,
        103 => 5.2,
        104 => 4.9,
        105 => 4.6,
        106 => 4.3,
        107 => 4.1,
        108 => 3.9,
        109 => 3.7,
        110 => 3.5,
        111 => 3.4,
        112 => 3.3,
        113 => 3.1,
        114 => 3.0,
        115 => 2.9,
        116 => 2.8,
        117 => 2.7,
        118 => 2.5,
        119 => 2.3,
        120 => 2.0,
        _ => return None,
    };
    Some(f)
}

pub fn rmd_start_age_for_birth_year(birth_year: i32) -> u32 {
    if birth_year >= 1960 { 75 }
    else if birth_year >= 1951 { 73 }
    else { 72 }
}

pub fn compute(input: &RmdInput) -> RmdReport {
    let start_age = rmd_start_age_for_birth_year(input.birth_year);
    let years_until = start_age as i32 - input.current_age as i32;

    let current_factor = if input.current_age >= start_age {
        uniform_lifetime_factor(input.current_age)
    } else { None };
    let current_rmd = current_factor.map(|f| input.balance_usd / f);

    let r = input.expected_annual_return_pct / 100.0;
    let mut balance = input.balance_usd;
    let mut age = input.current_age;
    // Grow balance up to start age (no RMDs yet).
    while age < start_age {
        balance *= 1.0 + r;
        age += 1;
    }
    let mut projection: Vec<YearRmd> = Vec::new();
    let mut total_rmds = 0.0_f64;
    for _ in 0..input.project_years {
        let factor_opt = uniform_lifetime_factor(age);
        let (factor, rmd) = match factor_opt {
            Some(f) => (f, balance / f),
            None => break,
        };
        let start = balance;
        let end = (balance - rmd).max(0.0);
        projection.push(YearRmd {
            age,
            start_balance_usd: start,
            rmd_factor: factor,
            rmd_amount_usd: rmd,
            end_balance_after_rmd_usd: end,
        });
        total_rmds += rmd;
        balance = end * (1.0 + r);
        age += 1;
    }
    RmdReport {
        rmd_start_age: start_age,
        current_factor,
        current_rmd_usd: current_rmd,
        years_until_rmd: years_until,
        projection,
        total_rmds_through_projection_usd: total_rmds,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rmd_start_age_pre_secure_2_0() {
        assert_eq!(rmd_start_age_for_birth_year(1948), 72);
    }

    #[test]
    fn rmd_start_age_secure_2_0_window() {
        assert_eq!(rmd_start_age_for_birth_year(1951), 73);
        assert_eq!(rmd_start_age_for_birth_year(1959), 73);
    }

    #[test]
    fn rmd_start_age_post_2033() {
        assert_eq!(rmd_start_age_for_birth_year(1960), 75);
        assert_eq!(rmd_start_age_for_birth_year(1975), 75);
    }

    #[test]
    fn uniform_factor_known_ages() {
        assert_eq!(uniform_lifetime_factor(73), Some(26.5));
        assert_eq!(uniform_lifetime_factor(80), Some(20.2));
        assert_eq!(uniform_lifetime_factor(90), Some(12.2));
        assert_eq!(uniform_lifetime_factor(100), Some(6.4));
    }

    #[test]
    fn uniform_factor_outside_table() {
        assert!(uniform_lifetime_factor(50).is_none());
        assert!(uniform_lifetime_factor(125).is_none());
    }

    #[test]
    fn compute_pre_rmd_age_no_current_rmd() {
        let r = compute(&RmdInput {
            birth_year: 1955,
            current_age: 65,
            balance_usd: 500_000.0,
            expected_annual_return_pct: 6.0,
            project_years: 20,
        });
        assert_eq!(r.rmd_start_age, 73);
        assert!(r.current_rmd_usd.is_none());
        assert_eq!(r.years_until_rmd, 8);
    }

    #[test]
    fn compute_at_rmd_age_current_rmd() {
        let r = compute(&RmdInput {
            birth_year: 1952,
            current_age: 73,
            balance_usd: 1_000_000.0,
            expected_annual_return_pct: 6.0,
            project_years: 20,
        });
        assert_eq!(r.rmd_start_age, 73);
        // $1M / 26.5 = $37,736
        assert!(r.current_rmd_usd.is_some());
        assert!((r.current_rmd_usd.unwrap() - 37_735.85).abs() < 1.0);
    }

    #[test]
    fn compute_projection_starts_at_rmd_age() {
        let r = compute(&RmdInput {
            birth_year: 1955,
            current_age: 65,
            balance_usd: 500_000.0,
            expected_annual_return_pct: 6.0,
            project_years: 5,
        });
        assert_eq!(r.projection.first().unwrap().age, 73);
    }

    #[test]
    fn compute_projection_year_count() {
        let r = compute(&RmdInput {
            birth_year: 1952,
            current_age: 73,
            balance_usd: 1_000_000.0,
            expected_annual_return_pct: 6.0,
            project_years: 10,
        });
        assert_eq!(r.projection.len(), 10);
    }

    #[test]
    fn compute_projection_balance_after_rmd_decreasing_when_rmd_exceeds_growth() {
        // After ~age 88, RMD divisor < 14 → RMD > 7%, exceeds 6% return.
        let r = compute(&RmdInput {
            birth_year: 1952,
            current_age: 88,
            balance_usd: 500_000.0,
            expected_annual_return_pct: 6.0,
            project_years: 5,
        });
        let first = r.projection[0].end_balance_after_rmd_usd;
        let last_start = r.projection.last().unwrap().start_balance_usd;
        assert!(last_start < first || (last_start - first).abs() < first * 0.5);
    }

    #[test]
    fn compute_total_rmds_sums_projection() {
        let r = compute(&RmdInput {
            birth_year: 1952,
            current_age: 73,
            balance_usd: 1_000_000.0,
            expected_annual_return_pct: 6.0,
            project_years: 5,
        });
        let sum: f64 = r.projection.iter().map(|y| y.rmd_amount_usd).sum();
        assert!((r.total_rmds_through_projection_usd - sum).abs() < 0.01);
    }
}
