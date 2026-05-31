//! IRC §401(a)(9) — Required Minimum Distributions (RMDs).
//!
//! Every trader with a traditional IRA or 401(k) reaching the RMD age
//! must begin taking distributions or face a §4974 excise tax. SECURE
//! Act of 2019 raised the RMD age from 70½ to 72; SECURE 2.0 Act of
//! 2022 raised it again to 73 (and to 75 for those born in 1960+).
//! SECURE 2.0 also reduced the §4974 penalty from 50% to 25%, with a
//! further reduction to 10% if the shortfall is corrected within a
//! two-year correction window.
//!
//! **RMD age by birth year** (effective for ages reached in 2024+):
//!
//! | Birth year      | RMD age |
//! |-----------------|---------|
//! | 1949 or earlier | 70½     |
//! | 1950            | 72      |
//! | 1951-1959       | 73      |
//! | 1960+           | 75      |
//!
//! **Required Beginning Date (RBD)**: April 1 of the year AFTER the
//! account owner reaches the RMD age. Subsequent RMDs are due
//! December 31 of each year. The first-year-RMD has an extended
//! deadline; missing it pushes both into the same year and stacks two
//! distributions in the second tax year.
//!
//! **Roth account carve-outs**:
//!   - Roth IRA: never subject to RMD during owner's lifetime
//!   - Roth 401(k) **pre-2024**: was subject to RMD
//!   - Roth 401(k) **post-2024**: SECURE 2.0 removes the RMD requirement
//!
//! **Uniform Lifetime Table** factors are from IRS Pub 590-B Appendix B,
//! reflecting the November 2020 update effective 2022+. The RMD amount
//! is `prior_year_end_balance / lifetime_factor`. This module uses the
//! Uniform Lifetime Table (most common); the Joint Life Expectancy Table
//! (for spouse > 10 years younger) and Single Life Expectancy Table
//! (inherited IRAs) are caller-side determinations.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    TraditionalIra,
    Traditional401k,
    /// Never subject to RMD during owner's lifetime.
    RothIra,
    /// Roth 401(k) for tax years before 2024 — was subject to RMD.
    Roth401kPre2024,
    /// Roth 401(k) for tax years 2024+ — SECURE 2.0 removed RMD.
    Roth401kPost2024,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section401a9Input {
    pub account_type: AccountType,
    pub account_owner_birth_year: i32,
    pub current_tax_year: i32,
    pub prior_year_end_balance: Decimal,
    pub actual_distribution_taken: Decimal,
    /// True if the account owner timely corrected any RMD shortfall by
    /// taking the missed amount and filing Form 5329 within the SECURE
    /// 2.0 two-year correction window — qualifies for the reduced 10%
    /// penalty rate.
    pub timely_corrected_within_2_years: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section401a9Result {
    pub rmd_applicable: bool,
    /// RMD age determined by SECURE 2.0 birth-year cohort.
    pub rmd_age: u32,
    /// Account owner's age in the current tax year.
    pub current_age: u32,
    /// Uniform Lifetime Table factor at current_age. Zero when RMD not
    /// applicable.
    pub uniform_lifetime_factor: Decimal,
    pub required_rmd_amount: Decimal,
    pub shortfall_amount: Decimal,
    /// §4974 excise tax rate in basis points: 2500 = 25% (default
    /// post-SECURE 2.0), 1000 = 10% (with timely correction).
    pub penalty_rate_basis_points: u32,
    pub penalty_amount: Decimal,
    /// True if this is the first year RMD is required (the year owner
    /// reaches RMD age). RBD-extended deadline of April 1 of following
    /// year applies.
    pub is_first_rmd_year: bool,
    pub note: String,
}

/// Birth-year → RMD-age mapping under SECURE 2.0.
fn rmd_age_for_birth_year(birth_year: i32) -> u32 {
    if birth_year <= 1949 {
        70 // (actually 70½ but integer model uses 70)
    } else if birth_year == 1950 {
        72
    } else if (1951..=1959).contains(&birth_year) {
        73
    } else {
        75
    }
}

/// IRS Pub 590-B Uniform Lifetime Table (Appendix B), 2022+ factors.
/// Returns factor as a Decimal. Ages outside the table return zero.
fn uniform_lifetime_factor(age: u32) -> Decimal {
    use rust_decimal::Decimal as D;
    let factor_str: Option<&str> = match age {
        72 => Some("27.4"),
        73 => Some("26.5"),
        74 => Some("25.5"),
        75 => Some("24.6"),
        76 => Some("23.7"),
        77 => Some("22.9"),
        78 => Some("22.0"),
        79 => Some("21.1"),
        80 => Some("20.2"),
        81 => Some("19.4"),
        82 => Some("18.5"),
        83 => Some("17.7"),
        84 => Some("16.8"),
        85 => Some("16.0"),
        86 => Some("15.2"),
        87 => Some("14.4"),
        88 => Some("13.7"),
        89 => Some("12.9"),
        90 => Some("12.2"),
        91 => Some("11.5"),
        92 => Some("10.8"),
        93 => Some("10.1"),
        94 => Some("9.5"),
        95 => Some("8.9"),
        96 => Some("8.4"),
        97 => Some("7.8"),
        98 => Some("7.3"),
        99 => Some("6.8"),
        100 => Some("6.4"),
        _ => None,
    };
    factor_str
        .and_then(|s| s.parse::<D>().ok())
        .unwrap_or(Decimal::ZERO)
}

pub fn compute(input: &Section401a9Input) -> Section401a9Result {
    let current_age = (input.current_tax_year - input.account_owner_birth_year).max(0) as u32;
    let rmd_age = rmd_age_for_birth_year(input.account_owner_birth_year);

    // Roth account carve-outs.
    let roth_exempt = matches!(
        input.account_type,
        AccountType::RothIra | AccountType::Roth401kPost2024
    );

    if roth_exempt {
        return Section401a9Result {
            rmd_applicable: false,
            rmd_age,
            current_age,
            uniform_lifetime_factor: Decimal::ZERO,
            required_rmd_amount: Decimal::ZERO,
            shortfall_amount: Decimal::ZERO,
            penalty_rate_basis_points: 0,
            penalty_amount: Decimal::ZERO,
            is_first_rmd_year: false,
            note: format!(
                "Roth account ({:?}) — no RMD required during owner's lifetime under SECURE 2.0",
                input.account_type
            ),
        };
    }

    if current_age < rmd_age {
        return Section401a9Result {
            rmd_applicable: false,
            rmd_age,
            current_age,
            uniform_lifetime_factor: Decimal::ZERO,
            required_rmd_amount: Decimal::ZERO,
            shortfall_amount: Decimal::ZERO,
            penalty_rate_basis_points: 0,
            penalty_amount: Decimal::ZERO,
            is_first_rmd_year: false,
            note: format!(
                "owner age {} below RMD age {} (SECURE 2.0 cohort for birth year {}); no RMD required yet",
                current_age, rmd_age, input.account_owner_birth_year
            ),
        };
    }

    let factor = uniform_lifetime_factor(current_age);
    let required = if factor > Decimal::ZERO {
        input.prior_year_end_balance / factor
    } else {
        Decimal::ZERO
    };
    let shortfall = (required - input.actual_distribution_taken).max(Decimal::ZERO);

    let penalty_bp = if shortfall > Decimal::ZERO {
        if input.timely_corrected_within_2_years {
            1_000 // 10%
        } else {
            2_500 // 25%
        }
    } else {
        0
    };
    let penalty = shortfall * Decimal::from(penalty_bp) / Decimal::from(10_000);

    Section401a9Result {
        rmd_applicable: true,
        rmd_age,
        current_age,
        uniform_lifetime_factor: factor,
        required_rmd_amount: required,
        shortfall_amount: shortfall,
        penalty_rate_basis_points: penalty_bp,
        penalty_amount: penalty,
        is_first_rmd_year: current_age == rmd_age,
        note: if shortfall > Decimal::ZERO {
            format!(
                "RMD ${} required (balance ${} / factor {}); ${} taken; ${} shortfall; §4974 penalty {}% = ${}",
                required.round_dp(2),
                input.prior_year_end_balance.round_dp(2),
                factor,
                input.actual_distribution_taken.round_dp(2),
                shortfall.round_dp(2),
                penalty_bp / 100,
                penalty.round_dp(2)
            )
        } else {
            format!(
                "RMD ${} required (balance ${} / factor {}); ${} taken — satisfies §401(a)(9)",
                required.round_dp(2),
                input.prior_year_end_balance.round_dp(2),
                factor,
                input.actual_distribution_taken.round_dp(2)
            )
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section401a9Input {
        Section401a9Input {
            account_type: AccountType::TraditionalIra,
            account_owner_birth_year: 1953, // turns 73 in 2026
            current_tax_year: 2026,
            prior_year_end_balance: dec!(1_000_000),
            actual_distribution_taken: dec!(40_000),
            timely_corrected_within_2_years: false,
        }
    }

    #[test]
    fn roth_ira_no_rmd_regardless_of_age() {
        let mut i = base();
        i.account_type = AccountType::RothIra;
        i.account_owner_birth_year = 1940; // age 86
        let r = compute(&i);
        assert!(!r.rmd_applicable);
        assert_eq!(r.required_rmd_amount, Decimal::ZERO);
    }

    #[test]
    fn roth_401k_post_2024_no_rmd() {
        // SECURE 2.0 removed RMD for Roth 401(k) for tax years 2024+.
        let mut i = base();
        i.account_type = AccountType::Roth401kPost2024;
        let r = compute(&i);
        assert!(!r.rmd_applicable);
    }

    #[test]
    fn roth_401k_pre_2024_did_have_rmd() {
        let mut i = base();
        i.account_type = AccountType::Roth401kPre2024;
        let r = compute(&i);
        assert!(r.rmd_applicable);
    }

    #[test]
    fn born_1953_rmd_age_73() {
        // Born 1951-1959 cohort → RMD age 73.
        let r = compute(&base());
        assert_eq!(r.rmd_age, 73);
        assert_eq!(r.current_age, 73);
        assert!(r.is_first_rmd_year);
    }

    #[test]
    fn born_1960_rmd_age_75() {
        // Born 1960+ cohort → RMD age 75.
        let mut i = base();
        i.account_owner_birth_year = 1960;
        i.current_tax_year = 2035; // age 75
        let r = compute(&i);
        assert_eq!(r.rmd_age, 75);
        assert_eq!(r.current_age, 75);
        assert!(r.is_first_rmd_year);
    }

    #[test]
    fn born_1949_rmd_age_70() {
        // Pre-1950 cohort → legacy 70½ (modeled as 70).
        let mut i = base();
        i.account_owner_birth_year = 1949;
        i.current_tax_year = 2019;
        let r = compute(&i);
        assert_eq!(r.rmd_age, 70);
    }

    #[test]
    fn born_1950_rmd_age_72() {
        // 1950 cohort → SECURE 1.0 age 72.
        let mut i = base();
        i.account_owner_birth_year = 1950;
        i.current_tax_year = 2022;
        let r = compute(&i);
        assert_eq!(r.rmd_age, 72);
    }

    #[test]
    fn born_1951_rmd_age_73() {
        // 1951 = first year of SECURE 2.0 73 cohort.
        let mut i = base();
        i.account_owner_birth_year = 1951;
        let r = compute(&i);
        assert_eq!(r.rmd_age, 73);
    }

    #[test]
    fn born_1959_rmd_age_73_last_year() {
        // 1959 = last year of SECURE 2.0 73 cohort.
        let mut i = base();
        i.account_owner_birth_year = 1959;
        let r = compute(&i);
        assert_eq!(r.rmd_age, 73);
    }

    #[test]
    fn under_rmd_age_no_rmd_required() {
        let mut i = base();
        i.account_owner_birth_year = 1970; // young
        let r = compute(&i);
        assert!(!r.rmd_applicable);
        assert!(r.note.contains("below RMD age"));
    }

    #[test]
    fn age_73_uniform_lifetime_factor_26_5() {
        // Most-used factor: age 73 = 26.5. RMD = $1M / 26.5 ≈ $37,735.85.
        let r = compute(&base());
        assert_eq!(r.uniform_lifetime_factor, dec!(26.5));
        // 1000000 / 26.5 → expected ≈ 37735.84905...
        // Use full precision compare:
        let expected = dec!(1_000_000) / dec!(26.5);
        assert_eq!(r.required_rmd_amount, expected);
    }

    #[test]
    fn rmd_met_no_penalty() {
        // $40k distributed > $37,735.85 RMD → no shortfall, no penalty.
        let r = compute(&base());
        assert_eq!(r.shortfall_amount, Decimal::ZERO);
        assert_eq!(r.penalty_amount, Decimal::ZERO);
        assert_eq!(r.penalty_rate_basis_points, 0);
    }

    #[test]
    fn rmd_shortfall_25_percent_penalty() {
        // No distribution at all → full RMD as shortfall, 25% penalty.
        let mut i = base();
        i.actual_distribution_taken = Decimal::ZERO;
        let r = compute(&i);
        let expected_rmd = dec!(1_000_000) / dec!(26.5);
        assert_eq!(r.shortfall_amount, expected_rmd);
        assert_eq!(r.penalty_rate_basis_points, 2_500);
        let expected_penalty = expected_rmd * dec!(0.25);
        assert_eq!(r.penalty_amount, expected_penalty);
    }

    #[test]
    fn timely_correction_reduces_penalty_to_10_percent() {
        let mut i = base();
        i.actual_distribution_taken = Decimal::ZERO;
        i.timely_corrected_within_2_years = true;
        let r = compute(&i);
        assert_eq!(r.penalty_rate_basis_points, 1_000);
        let expected_rmd = dec!(1_000_000) / dec!(26.5);
        let expected_penalty = expected_rmd * dec!(0.10);
        assert_eq!(r.penalty_amount, expected_penalty);
    }

    #[test]
    fn partial_shortfall_proportional_penalty() {
        // RMD ≈ $37,736. Distributed $20k → shortfall ≈ $17,736.
        // Penalty 25% × shortfall.
        let mut i = base();
        i.actual_distribution_taken = dec!(20_000);
        let r = compute(&i);
        let expected_rmd = dec!(1_000_000) / dec!(26.5);
        let expected_shortfall = expected_rmd - dec!(20_000);
        assert_eq!(r.shortfall_amount, expected_shortfall);
        assert_eq!(r.penalty_amount, expected_shortfall * dec!(0.25));
    }

    #[test]
    fn uniform_lifetime_factor_age_75() {
        let mut i = base();
        i.account_owner_birth_year = 1951;
        i.current_tax_year = 2026;
        // age 75
        i.account_owner_birth_year = 1951; // turns 75 in 2026
        let r = compute(&Section401a9Input {
            account_owner_birth_year: 1951,
            current_tax_year: 2026,
            ..i
        });
        let _ = r;
        // Pin age 75 factor directly.
        let mut i2 = base();
        i2.account_owner_birth_year = 1951;
        i2.current_tax_year = 2026;
        let r2 = compute(&i2);
        assert_eq!(r2.current_age, 75);
        assert_eq!(r2.uniform_lifetime_factor, dec!(24.6));
    }

    #[test]
    fn uniform_lifetime_factor_age_85() {
        let mut i = base();
        i.account_owner_birth_year = 1941;
        i.current_tax_year = 2026; // age 85
        let r = compute(&i);
        assert_eq!(r.current_age, 85);
        assert_eq!(r.uniform_lifetime_factor, dec!(16.0));
    }

    #[test]
    fn uniform_lifetime_factor_age_100() {
        let mut i = base();
        i.account_owner_birth_year = 1926;
        i.current_tax_year = 2026; // age 100
        let r = compute(&i);
        assert_eq!(r.uniform_lifetime_factor, dec!(6.4));
    }

    #[test]
    fn is_first_rmd_year_only_when_current_age_equals_rmd_age() {
        // First RMD year flag set when current_age == rmd_age (RBD year).
        let first = compute(&base()); // age 73 = rmd_age
        assert!(first.is_first_rmd_year);

        let mut i = base();
        i.current_tax_year = 2027; // age 74
        let later = compute(&i);
        assert!(!later.is_first_rmd_year);
    }

    #[test]
    fn high_balance_rmd_no_precision_loss() {
        // $50M IRA at age 75: $50M / 24.6 = $2,032,520.33...
        let mut i = base();
        i.prior_year_end_balance = dec!(50_000_000);
        i.account_owner_birth_year = 1951;
        let r = compute(&i);
        let expected = dec!(50_000_000) / dec!(24.6);
        assert_eq!(r.required_rmd_amount, expected);
    }

    #[test]
    fn current_age_clamps_at_zero_for_future_birth_year() {
        // Pathological: birth year > current year → current_age = 0.
        let mut i = base();
        i.account_owner_birth_year = 2030;
        i.current_tax_year = 2026;
        let r = compute(&i);
        assert_eq!(r.current_age, 0);
        assert!(!r.rmd_applicable);
    }

    #[test]
    fn traditional_ira_subject_to_rmd() {
        let r = compute(&base());
        assert!(r.rmd_applicable);
    }

    #[test]
    fn traditional_401k_subject_to_rmd() {
        let mut i = base();
        i.account_type = AccountType::Traditional401k;
        let r = compute(&i);
        assert!(r.rmd_applicable);
    }

    #[test]
    fn boundary_age_birth_1959_2032_age_73_first_rmd() {
        // Born 1959 + tax year 2032 = age 73 = first RMD year.
        let mut i = base();
        i.account_owner_birth_year = 1959;
        i.current_tax_year = 2032;
        let r = compute(&i);
        assert_eq!(r.rmd_age, 73);
        assert_eq!(r.current_age, 73);
        assert!(r.is_first_rmd_year);
    }

    #[test]
    fn boundary_age_birth_1960_2035_age_75_first_rmd() {
        // Born 1960 + tax year 2035 = age 75 = first RMD year.
        // 1960 cohort uses age 75 not 73.
        let mut i = base();
        i.account_owner_birth_year = 1960;
        i.current_tax_year = 2035;
        let r = compute(&i);
        assert_eq!(r.rmd_age, 75);
        assert_eq!(r.current_age, 75);
        assert!(r.is_first_rmd_year);
    }

    #[test]
    fn note_describes_shortfall_with_penalty_amount() {
        let mut i = base();
        i.actual_distribution_taken = Decimal::ZERO;
        let r = compute(&i);
        assert!(r.note.contains("§4974"));
        assert!(r.note.contains("25%"));
    }

    #[test]
    fn note_describes_correction_with_10_percent() {
        let mut i = base();
        i.actual_distribution_taken = Decimal::ZERO;
        i.timely_corrected_within_2_years = true;
        let r = compute(&i);
        assert!(r.note.contains("10%"));
    }
}
