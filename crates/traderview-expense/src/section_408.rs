//! IRC § 408 — Individual Retirement Accounts. Natural
//! companion to section_408a (Roth IRA shipped iter 430).
//! Where § 408A houses POST-TAX Roth contributions
//! with TAX-FREE qualified distributions, § 408 governs
//! PRE-TAX traditional IRA contributions with DEDUCTIBLE
//! contributions (within § 219 limits) + TAX-DEFERRED
//! growth + ORDINARY-INCOME distributions taxed at
//! marginal rates.
//!
//! Companion to section_408a (Roth IRA), section_72t (10%
//! early-withdrawal penalty), section_67g (TCJA misc
//! deduction suspension), section_1411 (NIIT 3.8% surtax
//! — traditional IRA distributions exempt as not
//! investment income; but § 1411 applies to non-IRA
//! investment income), section_475 (trader mark-to-market
//! election + § 408 self-directed IRA trader-status
//! considerations), section_4975 (prohibited transaction
//! penalties).
//!
//! Trader-critical fact patterns:
//! - **Self-directed IRA trader** — intra-account
//!   trading gains/losses no current-year tax; § 1411
//!   NIIT exempt; but prohibited-transaction rules
//!   under § 4975 apply rigorously.
//! - **Trader using § 408(d)(3) 60-day rollover** —
//!   indirect rollover requires return to same or
//!   different IRA within 60 days; one-rollover-per-year
//!   rule (per Bobrow v. Commissioner T.C. Memo 2014-21,
//!   IRS Announcement 2014-15).
//! - **Trader age 73+ subject to RMD** — § 408(d)(6) +
//!   § 401(a)(9) RMD rules; SECURE Act 2.0 raised RMD
//!   age to 73 (rising to 75 for those born 1960+).
//! - **Trader using § 408(d)(8) QCD strategy** — age 70½+
//!   may direct up to **$111,000** (2026) directly from
//!   IRA to qualified charity, excluded from gross
//!   income.
//! - **Trader using § 408(p) SIMPLE IRA** — small-business
//!   trader (≤ 100 employees) — 2026 employee deferral
//!   $17,000 + catch-up $4,000 (50+).
//!
//! **§ 408(a) IRA definition** — trust for exclusive
//! benefit of an individual or beneficiaries; meets
//! requirements:
//! 1. § 408(a)(1) — except in case of rollover, no
//!    contributions accepted in excess of the dollar
//!    amount under § 219(b)(1)(A) ($7,500 for 2026);
//! 2. § 408(a)(2) — trustee must be a bank or other
//!    person who demonstrates to Secretary it can
//!    administer the trust;
//! 3. § 408(a)(3) — no part may be invested in life
//!    insurance contracts;
//! 4. § 408(a)(4) — interest of individual in balance is
//!    nonforfeitable;
//! 5. § 408(a)(5) — assets may not be commingled with
//!    other property except in common trust fund or
//!    common investment fund;
//! 6. § 408(a)(6) — distributions to satisfy § 401(a)(9)
//!    minimum distribution requirements.
//!
//! **§ 408(b) Individual retirement annuity** — annuity
//! contract issued by insurance company meeting
//! parallel requirements.
//!
//! **2026 contribution limits (IRS Notice 2025-77)**:
//! 1. Under age 50: **$7,500** (aggregate with § 408A
//!    Roth IRA);
//! 2. Age 50+ catch-up: additional **$1,100** (total
//!    $8,600).
//!
//! **§ 219(g) Deduction phase-out — active plan
//! participant**:
//! 2026 thresholds for taxpayers ACTIVE in an
//! employer-sponsored plan:
//! 1. Single + HOH: **$81,000-$91,000** MAGI phase-out;
//! 2. Married filing jointly (covered): **$129,000-
//!    $149,000** MAGI;
//! 3. Married filing jointly (spouse covered but
//!    taxpayer not): **$242,000-$252,000** MAGI;
//! 4. Married filing separately (covered): **$0-$10,000**
//!    MAGI.
//!
//! **§ 408(d)(1) Distribution taxation** — distributions
//! from traditional IRA included in gross income as
//! ORDINARY INCOME at marginal rates.
//!
//! **§ 408(d)(2) Pro-rata rule** — distribution from any
//! IRA computed on AGGREGATE BASIS across all
//! taxpayer's IRAs; non-deductible contribution basis
//! recovered pro-rata per Form 8606. Critical for
//! backdoor Roth analysis.
//!
//! **§ 408(d)(3) Rollover rules — 60-day window**:
//! 1. § 408(d)(3)(A) — distribution rolled over within
//!    60 DAYS not includible in gross income;
//! 2. **One-rollover-per-year rule** (Bobrow v.
//!    Commissioner, T.C. Memo 2014-21; IRS Announcement
//!    2014-15) — only ONE 60-day INDIRECT rollover
//!    per 12-month period across ALL IRAs;
//! 3. Direct trustee-to-trustee transfers NOT subject to
//!    one-per-year limit.
//!
//! **§ 408(d)(6) Required minimum distributions** —
//! cross-reference to § 401(a)(9); SECURE Act 2.0 raised
//! RMD age to **73** (rising to **75** for those born
//! 1960 or later).
//!
//! **§ 408(d)(8) Qualified Charitable Distribution
//! (QCD)** — age 70½+ may direct distribution UP TO
//! ANNUAL LIMIT directly from IRA to qualified charity,
//! EXCLUDED from gross income:
//! 2026 limit: **$111,000** (increased from $108,000 for
//! 2025); inflation-indexed under SECURE Act 2.0.
//!
//! § 408(d)(8)(F) — ONE-TIME $50,000 distribution to
//! charitable gift annuity OR charitable remainder
//! trust through "split-interest entity" (SECURE Act 2.0
//! addition).
//!
//! **§ 408(k) SEP IRA** — Simplified Employee Pension —
//! employer-sponsored IRA. 2026 limit: lesser of 25% of
//! employee compensation OR $70,000 § 415(c) limit.
//!
//! **§ 408(m) Collectibles prohibition**:
//! 1. § 408(m)(1) — IRA investment in collectibles is
//!    treated as DISTRIBUTION at cost basis (immediate
//!    taxation + § 72(t) penalty if under 59½);
//! 2. § 408(m)(2) — "collectibles" defined: any work of
//!    art, rug, antique, metal/gem, stamp/coin, alcoholic
//!    beverage, OR any other tangible personal property
//!    specified by Secretary;
//! 3. § 408(m)(3) — EXCEPTION for gold/silver coins +
//!    bullion meeting fineness standards (American Eagle,
//!    Canadian Maple Leaf, etc.).
//!
//! **§ 408(p) SIMPLE IRA** — Savings Incentive Match Plan
//! for Employees of Small Employers:
//! 1. Available to businesses with **≤ 100 EMPLOYEES**;
//! 2. 2026 employee deferral: **$17,000** base;
//! 3. 2026 catch-up (50+): additional **$4,000** (total
//!    $21,000);
//! 4. Employer match: 1:1 up to 3% of compensation OR
//!    2% non-elective contribution.
//!
//! **§ 408(q) Deemed IRA** — qualified employer plan
//! that adopts deemed-IRA feature; allows IRA
//! contributions within qualified plan.
//!
//! Citations: 26 USC § 408(a)-(q); 26 USC § 408A; 26 USC
//! § 219; 26 USC § 72(t); 26 USC § 401(a)(9); 26 USC
//! § 415(c); 26 USC § 1411; 26 USC § 4975; Employee
//! Retirement Income Security Act of 1974 (ERISA, Pub. L.
//! 93-406, September 2, 1974); SECURE Act of 2019 (Pub.
//! L. 116-94); SECURE Act 2.0 of 2022 (Pub. L. 117-328);
//! IRS Notice 2025-77 (2026 inflation-adjusted limits);
//! Bobrow v. Commissioner, T.C. Memo 2014-21; IRS
//! Announcement 2014-15; Treas. Reg. § 1.408-1 through
//! § 1.408-11; Form 8606 (Nondeductible IRAs).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    /// § 408(a) traditional IRA.
    TraditionalIra,
    /// § 408(b) individual retirement annuity.
    IraAnnuity,
    /// § 408(k) SEP IRA.
    SepIra,
    /// § 408(p) SIMPLE IRA.
    SimpleIra,
    /// § 408(q) deemed IRA (qualified-plan-adopted).
    DeemedIra,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Investment {
    /// Standard securities / cash / mutual fund.
    Securities,
    /// § 408(m) prohibited collectible (artwork, antique,
    /// gem, stamp, coin, alcohol, etc.).
    Collectible,
    /// § 408(m)(3) excepted gold/silver coin or bullion
    /// (American Eagle, etc.).
    PreciousMetalBullion,
    /// § 408(a)(3) life insurance contract (PROHIBITED).
    LifeInsuranceContract,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section408Input {
    pub account_type: AccountType,
    pub investment: Investment,
    /// Tax year of determination.
    pub tax_year: u32,
    /// Taxpayer age at end of year.
    pub age: u32,
    /// Contribution amount in cents.
    pub contribution_cents: u64,
    /// Modified AGI in cents (for § 219(g) phase-out).
    pub modified_agi_cents: u64,
    /// Whether taxpayer is ACTIVE participant in
    /// employer-sponsored retirement plan (§ 219(g)
    /// phase-out trigger).
    pub active_participant_in_employer_plan: bool,
    /// Distribution amount in cents (for § 408(d)
    /// taxation).
    pub distribution_cents: u64,
    /// Whether distribution is qualified charitable
    /// distribution (§ 408(d)(8) QCD).
    pub qualified_charitable_distribution: bool,
    /// Cumulative QCD amount this year in cents (tracks
    /// against annual limit).
    pub cumulative_qcd_this_year_cents: u64,
    /// Whether 60-day rollover (§ 408(d)(3)) — affects
    /// one-rollover-per-year tracking.
    pub indirect_60_day_rollover: bool,
    /// Whether another 60-day indirect rollover already
    /// occurred in trailing 12 months (Bobrow violation
    /// trigger).
    pub indirect_rollover_prior_12_months: bool,
    /// SIMPLE IRA employer employee count (§ 408(p)
    /// 100-employee threshold).
    pub simple_ira_employer_employee_count: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section408Result {
    pub account_type: AccountType,
    pub annual_contribution_limit_cents: u64,
    pub deductible_contribution_cents: u64,
    pub deduction_phase_out_low_cents: u64,
    pub deduction_phase_out_high_cents: u64,
    pub catch_up_eligible: bool,
    pub investment_permitted: bool,
    pub qcd_annual_limit_cents: u64,
    pub qcd_excluded_from_gross_income_cents: u64,
    pub rmd_age_threshold: u32,
    pub one_rollover_per_year_violated: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section408Input) -> Section408Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let catch_up_eligible = input.age >= 50;

    let annual_contribution_limit_cents = match input.account_type {
        AccountType::TraditionalIra | AccountType::IraAnnuity | AccountType::DeemedIra => {
            if catch_up_eligible {
                860_000
            } else {
                750_000
            }
        }
        AccountType::SepIra => 7_000_000,
        AccountType::SimpleIra => {
            if catch_up_eligible {
                2_100_000
            } else {
                1_700_000
            }
        }
    };

    let (phase_out_low_cents, phase_out_high_cents): (u64, u64) =
        if input.active_participant_in_employer_plan {
            (8_100_000, 9_100_000)
        } else {
            (24_200_000, 25_200_000)
        };

    let deductible_contribution_cents = if !input.active_participant_in_employer_plan
        || input.modified_agi_cents <= phase_out_low_cents
    {
        input.contribution_cents.min(annual_contribution_limit_cents)
    } else if input.modified_agi_cents >= phase_out_high_cents {
        0
    } else {
        let phase_range = phase_out_high_cents.saturating_sub(phase_out_low_cents);
        let in_phase = input.modified_agi_cents.saturating_sub(phase_out_low_cents);
        let reduction =
            annual_contribution_limit_cents.saturating_mul(in_phase) / phase_range;
        annual_contribution_limit_cents
            .saturating_sub(reduction)
            .min(input.contribution_cents)
    };

    let investment_permitted = !matches!(
        input.investment,
        Investment::Collectible | Investment::LifeInsuranceContract
    );

    if matches!(input.investment, Investment::Collectible) {
        failure_reasons.push(
            "26 USC § 408(m)(1) — IRA investment in COLLECTIBLES treated as DISTRIBUTION at cost basis (immediate taxation as ordinary income + § 72(t) 10% penalty if under age 59½); § 408(m)(2) collectibles defined: any work of art, rug, antique, metal/gem, stamp/coin, alcoholic beverage, OR any other tangible personal property specified by Secretary".to_string(),
        );
    }

    if matches!(input.investment, Investment::LifeInsuranceContract) {
        failure_reasons.push(
            "26 USC § 408(a)(3) — no part of an IRA may be invested in LIFE INSURANCE CONTRACTS; investment is prohibited and creates plan disqualification risk".to_string(),
        );
    }

    let qcd_annual_limit_cents: u64 = if input.tax_year >= 2026 {
        11_100_000
    } else if input.tax_year == 2025 {
        10_800_000
    } else {
        10_500_000
    };

    let qcd_excluded_from_gross_income_cents = if input.qualified_charitable_distribution
        && input.age >= 71
    {
        let remaining_allowance = qcd_annual_limit_cents
            .saturating_sub(input.cumulative_qcd_this_year_cents);
        input.distribution_cents.min(remaining_allowance)
    } else {
        0
    };

    if input.qualified_charitable_distribution && input.age < 71 {
        failure_reasons.push(
            "26 USC § 408(d)(8) — Qualified Charitable Distribution available only to taxpayers age 70½ OR OLDER; QCD exclusion does NOT apply for younger taxpayers".to_string(),
        );
    }

    let rmd_age_threshold = if input.tax_year >= 2033 {
        75
    } else {
        73
    };

    let one_rollover_per_year_violated =
        input.indirect_60_day_rollover && input.indirect_rollover_prior_12_months;
    if one_rollover_per_year_violated {
        failure_reasons.push(
            "26 USC § 408(d)(3) + Bobrow v. Commissioner, T.C. Memo 2014-21 + IRS Announcement 2014-15 — ONE-ROLLOVER-PER-YEAR rule applies across ALL IRAs (aggregate); second 60-day indirect rollover within 12 months is a TAXABLE DISTRIBUTION + 10% § 72(t) penalty if under 59½; direct trustee-to-trustee transfers NOT subject to one-per-year limit".to_string(),
        );
    }

    if matches!(input.account_type, AccountType::SimpleIra)
        && input.simple_ira_employer_employee_count > 100
    {
        failure_reasons.push(format!(
            "26 USC § 408(p)(2) — SIMPLE IRA available only to employers with 100 OR FEWER employees; employer has {} employees, exceeds threshold",
            input.simple_ira_employer_employee_count
        ));
    }

    if input.contribution_cents > annual_contribution_limit_cents {
        failure_reasons.push(format!(
            "26 USC § 408(a)(1) + § 219(b)(1)(A) — contribution {} cents EXCEEDS annual limit {} cents; excess contribution subject to § 4973 6% excise tax until withdrawn",
            input.contribution_cents, annual_contribution_limit_cents
        ));
    }

    let notes: Vec<String> = vec![
        "26 USC § 408(a) IRA defined as a trust for exclusive benefit of individual; six requirements: (1) contributions within § 219 limits; (2) trustee bank or qualified person; (3) NO life insurance investment; (4) interest is nonforfeitable; (5) assets not commingled; (6) RMD distributions under § 401(a)(9)".to_string(),
        "26 USC § 408(b) Individual Retirement Annuity — annuity contract issued by insurance company meeting parallel requirements to IRA trust".to_string(),
        "2026 IRA contribution limits (IRS Notice 2025-77): under 50 = $7,500 base; age 50+ catch-up = additional $1,100 (total $8,600); aggregate with § 408A Roth IRA".to_string(),
        "26 USC § 219(g) Deduction phase-out for ACTIVE PARTICIPANTS in employer-sponsored retirement plan — 2026 thresholds: Single/HOH $81,000-$91,000; MFJ (covered) $129,000-$149,000; MFJ (spouse covered, taxpayer not) $242,000-$252,000; MFS (covered) $0-$10,000".to_string(),
        "26 USC § 408(d)(1) Distribution taxation — distributions from traditional IRA included in gross income as ORDINARY INCOME at marginal rates; § 72(t) 10% penalty for early withdrawal under 59½ (with statutory exceptions)".to_string(),
        "26 USC § 408(d)(2) PRO-RATA RULE — distribution from any IRA computed on AGGREGATE BASIS across ALL taxpayer's IRAs; non-deductible contribution basis recovered pro-rata per Form 8606; CRITICAL for backdoor Roth analysis (large pre-tax balance dilutes conversion benefit)".to_string(),
        "26 USC § 408(d)(3) ROLLOVER RULES: (1) distribution rolled over within 60 DAYS not includible; (2) ONE-ROLLOVER-PER-YEAR rule per Bobrow v. Commissioner, T.C. Memo 2014-21 + IRS Announcement 2014-15 — only ONE 60-day INDIRECT rollover per 12-month period across ALL IRAs; (3) direct trustee-to-trustee transfers NOT subject to one-per-year limit".to_string(),
        "26 USC § 408(d)(6) + § 401(a)(9) Required Minimum Distributions — SECURE Act 2.0 raised RMD age to 73 (for those born 1951-1959) and 75 (for those born 1960+); RMD calculation = prior-year-end balance ÷ life-expectancy factor per Treas. Reg. § 1.401(a)(9)-9 Uniform Lifetime Table".to_string(),
        "26 USC § 408(d)(8) Qualified Charitable Distribution (QCD) — age 70½+ may direct IRA distribution UP TO $111,000 (2026; $108,000 for 2025) directly to qualified charity; EXCLUDED from gross income; inflation-indexed under SECURE Act 2.0; § 408(d)(8)(F) one-time $50,000 split-interest entity addition".to_string(),
        "26 USC § 408(k) SEP IRA — Simplified Employee Pension; 2026 limit = lesser of 25% of employee compensation OR $70,000 § 415(c) overall defined-contribution limit".to_string(),
        "26 USC § 408(m) COLLECTIBLES PROHIBITION — investment in collectibles treated as DISTRIBUTION at cost basis (immediate taxation + § 72(t) penalty); § 408(m)(2) defines collectibles to include art, rug, antique, metal/gem, stamp/coin, alcoholic beverage, other tangible personal property; § 408(m)(3) EXCEPTION for gold/silver coins + bullion meeting fineness standards (American Eagle, Canadian Maple Leaf, etc.)".to_string(),
        "26 USC § 408(p) SIMPLE IRA — Savings Incentive Match Plan for Employees of Small Employers; available to employers with 100 OR FEWER employees; 2026 employee deferral $17,000 + catch-up (50+) $4,000 (total $21,000); employer match 1:1 up to 3% OR 2% non-elective".to_string(),
        "26 USC § 408(q) Deemed IRA — qualified employer plan adopting deemed-IRA feature; allows IRA contributions within qualified plan with separate accounting".to_string(),
        "Trader-critical fact patterns: (1) Self-directed IRA trader — intra-account trading gains/losses no current-year tax; § 1411 NIIT exempt; § 4975 prohibited-transaction rules apply rigorously; (2) Trader using § 408(d)(3) 60-day rollover — one-rollover-per-year applies aggregate across ALL IRAs per Bobrow; (3) Trader age 73+ subject to RMD per SECURE Act 2.0; (4) Trader using § 408(d)(8) QCD strategy at 70½+ ($111,000 for 2026); (5) Trader using § 408(p) SIMPLE IRA as small-business owner (≤ 100 employees)".to_string(),
        "Created by ERISA of 1974 (Pub. L. 93-406, September 2, 1974); modified by SECURE Act of 2019 (Pub. L. 116-94) + SECURE Act 2.0 of 2022 (Pub. L. 117-328) which raised RMD age + indexed QCD + added § 408(d)(8)(F) split-interest entity QCD".to_string(),
    ];

    Section408Result {
        account_type: input.account_type,
        annual_contribution_limit_cents,
        deductible_contribution_cents,
        deduction_phase_out_low_cents: phase_out_low_cents,
        deduction_phase_out_high_cents: phase_out_high_cents,
        catch_up_eligible,
        investment_permitted,
        qcd_annual_limit_cents,
        qcd_excluded_from_gross_income_cents,
        rmd_age_threshold,
        one_rollover_per_year_violated,
        failure_reasons,
        citation: "26 USC § 408(a)-(q); 26 USC § 408A; 26 USC § 219; 26 USC § 72(t); 26 USC § 401(a)(9); 26 USC § 415(c); 26 USC § 1411; 26 USC § 4975; 26 USC § 4973; ERISA (Pub. L. 93-406, September 2, 1974); SECURE Act of 2019 (Pub. L. 116-94); SECURE Act 2.0 of 2022 (Pub. L. 117-328); IRS Notice 2025-77; Bobrow v. Commissioner, T.C. Memo 2014-21; IRS Announcement 2014-15; Treas. Reg. § 1.408-1 through § 1.408-11; Form 8606",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn under_50_traditional_ira() -> Section408Input {
        Section408Input {
            account_type: AccountType::TraditionalIra,
            investment: Investment::Securities,
            tax_year: 2026,
            age: 35,
            contribution_cents: 750_000,
            modified_agi_cents: 5_000_000,
            active_participant_in_employer_plan: false,
            distribution_cents: 0,
            qualified_charitable_distribution: false,
            cumulative_qcd_this_year_cents: 0,
            indirect_60_day_rollover: false,
            indirect_rollover_prior_12_months: false,
            simple_ira_employer_employee_count: 0,
        }
    }

    #[test]
    fn under_50_traditional_ira_base_limit_7500() {
        let r = check(&under_50_traditional_ira());
        assert_eq!(r.annual_contribution_limit_cents, 750_000);
        assert!(!r.catch_up_eligible);
    }

    #[test]
    fn age_50_catch_up_8600() {
        let mut i = under_50_traditional_ira();
        i.age = 50;
        let r = check(&i);
        assert_eq!(r.annual_contribution_limit_cents, 860_000);
        assert!(r.catch_up_eligible);
    }

    #[test]
    fn sep_ira_limit_70000() {
        let mut i = under_50_traditional_ira();
        i.account_type = AccountType::SepIra;
        let r = check(&i);
        assert_eq!(r.annual_contribution_limit_cents, 7_000_000);
    }

    #[test]
    fn simple_ira_under_50_deferral_17000() {
        let mut i = under_50_traditional_ira();
        i.account_type = AccountType::SimpleIra;
        let r = check(&i);
        assert_eq!(r.annual_contribution_limit_cents, 1_700_000);
    }

    #[test]
    fn simple_ira_age_50_catch_up_21000() {
        let mut i = under_50_traditional_ira();
        i.account_type = AccountType::SimpleIra;
        i.age = 50;
        let r = check(&i);
        assert_eq!(r.annual_contribution_limit_cents, 2_100_000);
    }

    #[test]
    fn active_participant_2026_phase_out_thresholds() {
        let mut i = under_50_traditional_ira();
        i.active_participant_in_employer_plan = true;
        let r = check(&i);
        assert_eq!(r.deduction_phase_out_low_cents, 8_100_000);
        assert_eq!(r.deduction_phase_out_high_cents, 9_100_000);
    }

    #[test]
    fn non_active_participant_full_deduction() {
        let mut i = under_50_traditional_ira();
        i.modified_agi_cents = 30_000_000;
        let r = check(&i);
        assert_eq!(r.deductible_contribution_cents, 750_000);
    }

    #[test]
    fn active_participant_above_phase_out_zero_deduction() {
        let mut i = under_50_traditional_ira();
        i.active_participant_in_employer_plan = true;
        i.modified_agi_cents = 10_000_000;
        let r = check(&i);
        assert_eq!(r.deductible_contribution_cents, 0);
    }

    #[test]
    fn collectibles_treated_as_distribution() {
        let mut i = under_50_traditional_ira();
        i.investment = Investment::Collectible;
        let r = check(&i);
        assert!(!r.investment_permitted);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 408(m)(1)")
            && f.contains("COLLECTIBLES")
            && f.contains("DISTRIBUTION at cost basis")));
    }

    #[test]
    fn life_insurance_contract_prohibited() {
        let mut i = under_50_traditional_ira();
        i.investment = Investment::LifeInsuranceContract;
        let r = check(&i);
        assert!(!r.investment_permitted);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 408(a)(3)")
            && f.contains("LIFE INSURANCE CONTRACTS")));
    }

    #[test]
    fn precious_metal_bullion_408m3_exception_permitted() {
        let mut i = under_50_traditional_ira();
        i.investment = Investment::PreciousMetalBullion;
        let r = check(&i);
        assert!(r.investment_permitted);
    }

    #[test]
    fn qcd_2026_limit_111000() {
        let mut i = under_50_traditional_ira();
        i.tax_year = 2026;
        i.age = 75;
        i.qualified_charitable_distribution = true;
        i.distribution_cents = 5_000_000;
        let r = check(&i);
        assert_eq!(r.qcd_annual_limit_cents, 11_100_000);
        assert_eq!(r.qcd_excluded_from_gross_income_cents, 5_000_000);
    }

    #[test]
    fn qcd_2025_limit_108000() {
        let mut i = under_50_traditional_ira();
        i.tax_year = 2025;
        i.age = 75;
        i.qualified_charitable_distribution = true;
        let r = check(&i);
        assert_eq!(r.qcd_annual_limit_cents, 10_800_000);
    }

    #[test]
    fn qcd_under_70_5_no_exclusion() {
        let mut i = under_50_traditional_ira();
        i.age = 65;
        i.qualified_charitable_distribution = true;
        i.distribution_cents = 5_000_000;
        let r = check(&i);
        assert_eq!(r.qcd_excluded_from_gross_income_cents, 0);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 408(d)(8)")
            && f.contains("age 70½")));
    }

    #[test]
    fn qcd_distribution_capped_at_annual_limit() {
        let mut i = under_50_traditional_ira();
        i.age = 75;
        i.qualified_charitable_distribution = true;
        i.distribution_cents = 15_000_000;
        let r = check(&i);
        assert_eq!(r.qcd_excluded_from_gross_income_cents, 11_100_000);
    }

    #[test]
    fn qcd_cumulative_prior_reduces_remaining() {
        let mut i = under_50_traditional_ira();
        i.age = 75;
        i.qualified_charitable_distribution = true;
        i.distribution_cents = 5_000_000;
        i.cumulative_qcd_this_year_cents = 8_000_000;
        let r = check(&i);
        assert_eq!(r.qcd_excluded_from_gross_income_cents, 3_100_000);
    }

    #[test]
    fn rmd_age_73_pre_2033() {
        let r = check(&under_50_traditional_ira());
        assert_eq!(r.rmd_age_threshold, 73);
    }

    #[test]
    fn rmd_age_75_post_2033() {
        let mut i = under_50_traditional_ira();
        i.tax_year = 2033;
        let r = check(&i);
        assert_eq!(r.rmd_age_threshold, 75);
    }

    #[test]
    fn one_rollover_per_year_violated_bobrow() {
        let mut i = under_50_traditional_ira();
        i.indirect_60_day_rollover = true;
        i.indirect_rollover_prior_12_months = true;
        let r = check(&i);
        assert!(r.one_rollover_per_year_violated);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 408(d)(3)")
            && f.contains("Bobrow")
            && f.contains("ONE-ROLLOVER-PER-YEAR")));
    }

    #[test]
    fn one_rollover_first_no_violation() {
        let mut i = under_50_traditional_ira();
        i.indirect_60_day_rollover = true;
        i.indirect_rollover_prior_12_months = false;
        let r = check(&i);
        assert!(!r.one_rollover_per_year_violated);
    }

    #[test]
    fn simple_ira_100_employee_threshold_compliant() {
        let mut i = under_50_traditional_ira();
        i.account_type = AccountType::SimpleIra;
        i.simple_ira_employer_employee_count = 100;
        let r = check(&i);
        assert!(!r.failure_reasons.iter().any(|f| f.contains("§ 408(p)(2)")));
    }

    #[test]
    fn simple_ira_101_employee_threshold_violation() {
        let mut i = under_50_traditional_ira();
        i.account_type = AccountType::SimpleIra;
        i.simple_ira_employer_employee_count = 101;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 408(p)(2)")
            && f.contains("100 OR FEWER")));
    }

    #[test]
    fn excess_contribution_4973_excise_violation() {
        let mut i = under_50_traditional_ira();
        i.contribution_cents = 1_000_000;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 408(a)(1)")
            && f.contains("§ 219(b)(1)(A)")
            && f.contains("§ 4973")
            && f.contains("6% excise tax")));
    }

    #[test]
    fn account_type_truth_table_five_cells() {
        for acct in [
            AccountType::TraditionalIra,
            AccountType::IraAnnuity,
            AccountType::SepIra,
            AccountType::SimpleIra,
            AccountType::DeemedIra,
        ] {
            let mut i = under_50_traditional_ira();
            i.account_type = acct;
            let r = check(&i);
            assert_eq!(r.account_type, acct);
        }
    }

    #[test]
    fn investment_truth_table_four_cells() {
        for (inv, exp_permitted) in [
            (Investment::Securities, true),
            (Investment::Collectible, false),
            (Investment::PreciousMetalBullion, true),
            (Investment::LifeInsuranceContract, false),
        ] {
            let mut i = under_50_traditional_ira();
            i.investment = inv;
            let r = check(&i);
            assert_eq!(r.investment_permitted, exp_permitted, "inv={:?}", inv);
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&under_50_traditional_ira());
        assert!(r.citation.contains("§ 408(a)-(q)"));
        assert!(r.citation.contains("§ 408A"));
        assert!(r.citation.contains("§ 219"));
        assert!(r.citation.contains("§ 72(t)"));
        assert!(r.citation.contains("§ 401(a)(9)"));
        assert!(r.citation.contains("§ 415(c)"));
        assert!(r.citation.contains("§ 1411"));
        assert!(r.citation.contains("§ 4975"));
        assert!(r.citation.contains("§ 4973"));
        assert!(r.citation.contains("ERISA"));
        assert!(r.citation.contains("Pub. L. 93-406"));
        assert!(r.citation.contains("September 2, 1974"));
        assert!(r.citation.contains("SECURE Act of 2019"));
        assert!(r.citation.contains("SECURE Act 2.0 of 2022"));
        assert!(r.citation.contains("IRS Notice 2025-77"));
        assert!(r.citation.contains("Bobrow v. Commissioner, T.C. Memo 2014-21"));
        assert!(r.citation.contains("IRS Announcement 2014-15"));
        assert!(r.citation.contains("Treas. Reg. § 1.408-1 through § 1.408-11"));
        assert!(r.citation.contains("Form 8606"));
    }

    #[test]
    fn note_pins_subsection_a_six_requirements() {
        let r = check(&under_50_traditional_ira());
        assert!(r.notes.iter().any(|n| n.contains("§ 408(a)")
            && n.contains("six requirements")
            && n.contains("trustee bank")
            && n.contains("NO life insurance")
            && n.contains("nonforfeitable")));
    }

    #[test]
    fn note_pins_2026_contribution_limits() {
        let r = check(&under_50_traditional_ira());
        assert!(r.notes.iter().any(|n| n.contains("2026 IRA contribution limits")
            && n.contains("$7,500 base")
            && n.contains("$1,100")
            && n.contains("$8,600")
            && n.contains("§ 408A Roth IRA")));
    }

    #[test]
    fn note_pins_219g_phase_out_active_participant() {
        let r = check(&under_50_traditional_ira());
        assert!(r.notes.iter().any(|n| n.contains("§ 219(g)")
            && n.contains("ACTIVE PARTICIPANTS")
            && n.contains("Single/HOH $81,000-$91,000")
            && n.contains("MFJ (covered) $129,000-$149,000")));
    }

    #[test]
    fn note_pins_d2_pro_rata_rule_backdoor_critical() {
        let r = check(&under_50_traditional_ira());
        assert!(r.notes.iter().any(|n| n.contains("§ 408(d)(2)")
            && n.contains("PRO-RATA RULE")
            && n.contains("AGGREGATE BASIS")
            && n.contains("Form 8606")
            && n.contains("CRITICAL for backdoor Roth")));
    }

    #[test]
    fn note_pins_d3_one_rollover_per_year_bobrow() {
        let r = check(&under_50_traditional_ira());
        assert!(r.notes.iter().any(|n| n.contains("§ 408(d)(3)")
            && n.contains("ROLLOVER RULES")
            && n.contains("Bobrow")
            && n.contains("ONE 60-day INDIRECT rollover")
            && n.contains("direct trustee-to-trustee")));
    }

    #[test]
    fn note_pins_d6_rmd_secure_2_0_73_75() {
        let r = check(&under_50_traditional_ira());
        assert!(r.notes.iter().any(|n| n.contains("§ 408(d)(6)")
            && n.contains("§ 401(a)(9)")
            && n.contains("SECURE Act 2.0")
            && n.contains("RMD age to 73")
            && n.contains("75 (for those born 1960+)")));
    }

    #[test]
    fn note_pins_d8_qcd_111000_split_interest() {
        let r = check(&under_50_traditional_ira());
        assert!(r.notes.iter().any(|n| n.contains("§ 408(d)(8)")
            && n.contains("Qualified Charitable Distribution")
            && n.contains("$111,000 (2026")
            && n.contains("$50,000 split-interest entity")));
    }

    #[test]
    fn note_pins_k_sep_ira_70000_limit() {
        let r = check(&under_50_traditional_ira());
        assert!(r.notes.iter().any(|n| n.contains("§ 408(k)")
            && n.contains("SEP IRA")
            && n.contains("25% of employee compensation")
            && n.contains("$70,000")));
    }

    #[test]
    fn note_pins_m_collectibles_prohibition_408m3_exception() {
        let r = check(&under_50_traditional_ira());
        assert!(r.notes.iter().any(|n| n.contains("§ 408(m)")
            && n.contains("COLLECTIBLES PROHIBITION")
            && n.contains("§ 408(m)(3) EXCEPTION")
            && n.contains("American Eagle")));
    }

    #[test]
    fn note_pins_p_simple_ira_100_employees_17000() {
        let r = check(&under_50_traditional_ira());
        assert!(r.notes.iter().any(|n| n.contains("§ 408(p)")
            && n.contains("100 OR FEWER")
            && n.contains("$17,000")
            && n.contains("$4,000")
            && n.contains("$21,000")));
    }

    #[test]
    fn note_pins_q_deemed_ira() {
        let r = check(&under_50_traditional_ira());
        assert!(r.notes.iter().any(|n| n.contains("§ 408(q)")
            && n.contains("Deemed IRA")));
    }

    #[test]
    fn note_pins_trader_critical_fact_patterns_five() {
        let r = check(&under_50_traditional_ira());
        assert!(r.notes.iter().any(|n| n.contains("Trader-critical fact patterns")
            && n.contains("Self-directed IRA trader")
            && n.contains("§ 4975 prohibited-transaction")
            && n.contains("Bobrow")
            && n.contains("§ 408(d)(8) QCD strategy")
            && n.contains("§ 408(p) SIMPLE IRA")));
    }

    #[test]
    fn note_pins_erisa_1974_origin() {
        let r = check(&under_50_traditional_ira());
        assert!(r.notes.iter().any(|n| n.contains("ERISA")
            && n.contains("Pub. L. 93-406")
            && n.contains("September 2, 1974")
            && n.contains("SECURE Act 2.0")));
    }

    #[test]
    fn defensive_overflow_saturating() {
        let mut i = under_50_traditional_ira();
        i.modified_agi_cents = u64::MAX;
        let r = check(&i);
        let _ = r.deductible_contribution_cents;
    }
}
