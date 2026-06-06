//! IRC § 408A — Roth IRAs. Trader-critical for retirement
//! planning + after-tax growth — Roth IRA contributions
//! are made with AFTER-TAX dollars but qualified
//! distributions of EARNINGS AND PRINCIPAL are entirely
//! TAX-FREE; no required minimum distributions during
//! owner's lifetime; FREE-FROM § 1411 NIIT on
//! distributions (Roth withdrawals are not investment
//! income).
//!
//! Companion to section_408 (traditional IRA rules),
//! section_72t (10% early-withdrawal penalty + 72(t)
//! substantially equal periodic payments exception),
//! section_67g (TCJA misc deduction suspension —
//! reinforces value of tax-advantaged accounts),
//! section_1411 (NIIT 3.8% surtax — Roth distributions
//! exempt as not investment income), section_475 (trader
//! mark-to-market election — Roth account trader can
//! qualify for trader-status MTM treatment of
//! intra-account trading gains/losses).
//!
//! Trader-critical fact patterns:
//! - **High-income trader** subject to phase-out — must
//!   use backdoor Roth conversion to access Roth tax-free
//!   growth.
//! - **Active trader using self-directed Roth IRA** —
//!   intra-account trading gains/losses do not generate
//!   current-year tax; § 1411 NIIT does not apply.
//! - **Mega backdoor Roth** — after-tax 401(k)
//!   contributions converted to Roth (requires plan
//!   design + § 415 limits + non-discrimination).
//! - **§ 72(t) substantially-equal-periodic-payments
//!   exception** — pre-59½ early withdrawal without 10%
//!   penalty under fixed amortization / amortization-
//!   life-expectancy / RMD method.
//!
//! **§ 408A(c)(1) Contribution limit — aggregate with
//! § 408 traditional IRA**:
//! 2026 limits:
//! 1. **Under age 50**: $7,500 base annual limit
//!    (aggregate with § 408 traditional IRA).
//! 2. **Age 50 or older**: additional **$1,100** catch-up
//!    contribution (total $8,600).
//!
//! Limits cost-of-living-adjusted under § 408A(c)(2) + §
//! 219(b)(5)(C).
//!
//! **§ 408A(c)(3) Income-based phase-out**:
//! 2026 thresholds:
//! 1. **Single + Head of Household**: $153,000 - $168,000
//!    MAGI phase-out range.
//! 2. **Married Filing Jointly**: $242,000 - $252,000
//!    MAGI phase-out range.
//! 3. **Married Filing Separately** (NOT cost-of-living
//!    adjusted): $0 - $10,000 phase-out range.
//!
//! Modified AGI under § 408A(c)(3)(B) = AGI under
//! § 219(g)(3)(A) but DISREGARDING any Roth conversion
//! income.
//!
//! **§ 408A(d) Qualified distributions — tax-free
//! treatment**:
//! 1. § 408A(d)(2)(A) — distribution made after **5-YEAR
//!    HOLDING PERIOD** starting January 1 of the first
//!    year a Roth contribution was made; AND
//! 2. § 408A(d)(2)(B) — meets ONE of:
//!    - Made on or after age **59½**;
//!    - Made on account of disability per § 72(m)(7);
//!    - Made to beneficiary after death of owner;
//!    - First-time home purchase up to $10,000 lifetime
//!      cap per § 72(t)(2)(F).
//!
//! **§ 408A(d)(3) Ordering rules** — non-qualified
//! distributions ordered:
//! 1. Contributions (always tax-free + penalty-free);
//! 2. Conversions (5-year holding per conversion for
//!    penalty-free principal);
//! 3. Earnings (taxable + 10% penalty under § 72(t)
//!    unless exception).
//!
//! **§ 408A(d)(3)(A) Conversion 5-year rule** — separate
//! 5-year holding period applies to EACH conversion for
//! purposes of avoiding 10% penalty on conversion
//! principal withdrawn before age 59½. Multiple
//! conversions create stacked 5-year clocks.
//!
//! **§ 408A(e) Backdoor Roth conversion** — taxpayer
//! exceeding income phase-out may make NON-DEDUCTIBLE
//! TRADITIONAL IRA contribution (§ 408) then CONVERT to
//! Roth IRA (§ 408A(d)(3)(C)). Conversion is taxable to
//! the extent of pre-tax balance in IRA (pro-rata rule
//! per § 408(d)(2)) — backdoor Roth optimal when
//! taxpayer has NO pre-tax traditional IRA balance.
//!
//! **§ 408A(c)(5) No RMD during owner's lifetime** —
//! unlike traditional IRA, Roth IRA owner is NOT subject
//! to § 401(a)(9) required minimum distributions during
//! life. Beneficiaries subject to post-death RMD rules
//! under SECURE Act § 401(a)(9)(H).
//!
//! Citations: 26 USC § 408A(a)-(f); 26 USC § 408; 26 USC
//! § 219; 26 USC § 72; 26 USC § 72(t); 26 USC § 401(a)(9);
//! 26 USC § 1411; 26 USC § 415; Taxpayer Relief Act of
//! 1997 § 302 (Pub. L. 105-34, enacted August 5, 1997 —
//! created Roth IRA); SECURE Act of 2019 § 401 (Pub. L.
//! 116-94); SECURE Act 2.0 of 2022 (Pub. L. 117-328); IRS
//! Notice 2025-77 (2026 inflation-adjusted limits); Treas.
//! Reg. § 1.408A-1 through § 1.408A-10; Form 5498
//! (IRA Contribution Information).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    HeadOfHousehold,
    MarriedFilingJointly,
    MarriedFilingSeparately,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DistributionPath {
    /// Contribution principal withdrawn — always tax-free
    /// + penalty-free per § 408A(d)(3) ordering.
    ContributionPrincipal,
    /// Conversion principal withdrawn — 5-year holding
    /// per conversion to avoid 10% penalty under § 72(t).
    ConversionPrincipal,
    /// Earnings withdrawn — qualified distribution
    /// requires 5-year + age 59½/disability/death/first
    /// home.
    Earnings,
    /// No distribution this year.
    NoDistribution,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section408aInput {
    pub filing_status: FilingStatus,
    /// Modified AGI in cents (under § 408A(c)(3)(B) =
    /// § 219(g)(3)(A) AGI minus Roth conversion income).
    pub modified_agi_cents: u64,
    /// Tax year for determination (2026 inflation
    /// thresholds).
    pub tax_year: u32,
    /// Taxpayer age at end of tax year (50+ catch-up;
    /// 59½ qualified distribution).
    pub age: u32,
    /// Roth contribution amount in cents.
    pub roth_contribution_cents: u64,
    /// Years since first Roth contribution (5-year rule).
    pub years_since_first_roth_contribution: u32,
    /// Distribution path.
    pub distribution_path: DistributionPath,
    /// Distribution amount in cents.
    pub distribution_amount_cents: u64,
    /// Years since the specific conversion being withdrawn
    /// (5-year per-conversion rule).
    pub years_since_conversion: u32,
    /// Whether disability per § 72(m)(7) engages.
    pub disability_engaged: bool,
    /// Whether distribution is to beneficiary after death
    /// of owner.
    pub post_death_beneficiary_distribution: bool,
    /// Whether first-time home purchase (up to $10,000
    /// lifetime).
    pub first_time_home_purchase: bool,
    /// Cumulative prior § 72(t)(2)(F) first-home-purchase
    /// withdrawals in cents (lifetime $10K cap tracking).
    pub cumulative_first_home_prior_withdrawals_cents: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section408aResult {
    pub annual_contribution_limit_cents: u64,
    pub phase_out_low_cents: u64,
    pub phase_out_high_cents: u64,
    pub allowed_contribution_cents: u64,
    pub catch_up_eligible: bool,
    pub backdoor_roth_required: bool,
    pub distribution_qualified: bool,
    pub distribution_taxable_amount_cents: u64,
    pub distribution_penalty_amount_cents: u64,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section408aInput) -> Section408aResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let base_limit = 750_000_u64;
    let catch_up_amount = 110_000_u64;
    let catch_up_eligible = input.age >= 50;
    let annual_contribution_limit_cents = if catch_up_eligible {
        base_limit.saturating_add(catch_up_amount)
    } else {
        base_limit
    };

    let (phase_out_low, phase_out_high) = match input.filing_status {
        FilingStatus::Single | FilingStatus::HeadOfHousehold => (15_300_000, 16_800_000),
        FilingStatus::MarriedFilingJointly => (24_200_000, 25_200_000),
        FilingStatus::MarriedFilingSeparately => (0, 1_000_000),
    };

    let allowed_contribution_cents = if input.modified_agi_cents >= phase_out_high {
        0
    } else if input.modified_agi_cents <= phase_out_low {
        annual_contribution_limit_cents
    } else {
        let phase_range = phase_out_high.saturating_sub(phase_out_low);
        let in_phase = input.modified_agi_cents.saturating_sub(phase_out_low);
        let reduction = annual_contribution_limit_cents.saturating_mul(in_phase) / phase_range;
        annual_contribution_limit_cents.saturating_sub(reduction)
    };

    let backdoor_roth_required = input.modified_agi_cents >= phase_out_high;

    if backdoor_roth_required {
        failure_reasons.push(
            "26 USC § 408A(c)(3) — taxpayer's MAGI exceeds phase-out ceiling; direct Roth contribution NOT ALLOWED; § 408A(e) BACKDOOR ROTH conversion available via non-deductible traditional IRA contribution + Roth conversion (pro-rata rule under § 408(d)(2) limits benefit when pre-tax IRA balance exists)".to_string(),
        );
    }

    if input.roth_contribution_cents > allowed_contribution_cents && allowed_contribution_cents > 0
    {
        failure_reasons.push(format!(
            "26 USC § 408A(c)(1) — Roth contribution {} cents EXCEEDS allowed phase-out-reduced limit {} cents; excess contribution subject to § 4973 6% excise tax until withdrawn",
            input.roth_contribution_cents, allowed_contribution_cents
        ));
    }

    let five_year_holding_satisfied = input.years_since_first_roth_contribution >= 5;
    let age_or_event_satisfied = input.age >= 60
        || input.disability_engaged
        || input.post_death_beneficiary_distribution
        || (input.first_time_home_purchase
            && input.cumulative_first_home_prior_withdrawals_cents < 1_000_000);

    let distribution_qualified = matches!(input.distribution_path, DistributionPath::Earnings)
        && five_year_holding_satisfied
        && age_or_event_satisfied;

    let (distribution_taxable_amount_cents, distribution_penalty_amount_cents) =
        match input.distribution_path {
            DistributionPath::ContributionPrincipal => (0, 0),
            DistributionPath::ConversionPrincipal => {
                if input.years_since_conversion < 5 && input.age < 60 {
                    (0, input.distribution_amount_cents / 10)
                } else {
                    (0, 0)
                }
            }
            DistributionPath::Earnings => {
                if distribution_qualified {
                    (0, 0)
                } else {
                    let penalty = if input.age < 60 && !input.disability_engaged {
                        input.distribution_amount_cents / 10
                    } else {
                        0
                    };
                    (input.distribution_amount_cents, penalty)
                }
            }
            DistributionPath::NoDistribution => (0, 0),
        };

    if matches!(input.distribution_path, DistributionPath::Earnings) && !distribution_qualified {
        failure_reasons.push(
            "26 USC § 408A(d)(2) — Earnings distribution NOT QUALIFIED: requires (1) 5-YEAR HOLDING PERIOD starting January 1 of first Roth contribution year AND (2) ONE of (a) age 59½+; (b) disability per § 72(m)(7); (c) death of owner; (d) first-time home purchase up to $10,000 lifetime per § 72(t)(2)(F); 10% early withdrawal penalty applies under § 72(t)".to_string(),
        );
    }

    if matches!(
        input.distribution_path,
        DistributionPath::ConversionPrincipal
    ) && input.years_since_conversion < 5
        && input.age < 60
    {
        failure_reasons.push(format!(
            "26 USC § 408A(d)(3)(A) — Conversion principal withdrawn within 5-YEAR per-conversion holding period subject to 10% early withdrawal penalty under § 72(t); only {} years since conversion (need 5)",
            input.years_since_conversion
        ));
    }

    if input.first_time_home_purchase
        && input.cumulative_first_home_prior_withdrawals_cents >= 1_000_000
    {
        failure_reasons.push(
            "26 USC § 72(t)(2)(F) — first-time home purchase exception capped at $10,000 LIFETIME; cumulative prior withdrawals already at or above cap; cannot stack further first-home exceptions".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "26 USC § 408A(c)(1) Contribution limit aggregate with § 408 traditional IRA — 2026 limits: under age 50 = $7,500 base + age 50 catch-up = additional $1,100 (total $8,600); cost-of-living-adjusted under § 408A(c)(2) + § 219(b)(5)(C)".to_string(),
        "26 USC § 408A(c)(3) Income phase-out — 2026 MAGI thresholds: Single/HOH $153K-$168K; MFJ $242K-$252K; MFS $0-$10K (NOT cost-of-living adjusted)".to_string(),
        "26 USC § 408A(c)(3)(B) Modified AGI = AGI under § 219(g)(3)(A) DISREGARDING any Roth conversion income (large conversion does not push you into higher phase-out tier for same year's contribution)".to_string(),
        "26 USC § 408A(d)(2) Qualified distribution two-prong test: (1) 5-YEAR HOLDING PERIOD starting January 1 of first Roth contribution year AND (2) ONE of (a) age 59½+; (b) disability per § 72(m)(7); (c) death of owner; (d) first-time home purchase up to $10,000 lifetime per § 72(t)(2)(F)".to_string(),
        "26 USC § 408A(d)(3) Non-qualified distribution ORDERING rules: (1) Contributions (always tax-free + penalty-free); (2) Conversions (5-year per-conversion holding for penalty-free principal); (3) Earnings (taxable + 10% penalty under § 72(t) unless exception)".to_string(),
        "26 USC § 408A(d)(3)(A) Conversion 5-year rule — separate 5-year holding period applies to EACH conversion for purposes of avoiding 10% penalty on conversion principal withdrawn before age 59½; multiple conversions create stacked 5-year clocks".to_string(),
        "26 USC § 408A(e) BACKDOOR ROTH conversion — taxpayer exceeding income phase-out may make NON-DEDUCTIBLE TRADITIONAL IRA contribution then CONVERT to Roth IRA per § 408A(d)(3)(C); conversion is taxable to extent of pre-tax balance in IRA (pro-rata rule under § 408(d)(2)) — BACKDOOR ROTH OPTIMAL when taxpayer has NO pre-tax traditional IRA balance".to_string(),
        "26 USC § 408A(c)(5) No RMD during owner's lifetime — unlike traditional IRA, Roth IRA owner is NOT subject to § 401(a)(9) required minimum distributions during life; beneficiaries subject to post-death RMD rules under SECURE Act § 401(a)(9)(H)".to_string(),
        "Roth IRA distributions are EXEMPT from § 1411 NIIT 3.8% surtax — Roth withdrawals are not investment income; trader-relevant for high-income retirees".to_string(),
        "Trader-critical fact patterns: (1) high-income trader subject to phase-out must use backdoor Roth conversion; (2) active trader using self-directed Roth IRA — intra-account trading gains/losses do not generate current-year tax; § 1411 NIIT exempt; (3) mega backdoor Roth — after-tax 401(k) contributions converted to Roth (requires plan design + § 415 limits + non-discrimination); (4) § 72(t) substantially-equal-periodic-payments exception — pre-59½ early withdrawal without 10% penalty under fixed amortization / amortization-life-expectancy / RMD method".to_string(),
        "Created by Taxpayer Relief Act of 1997 § 302 (Pub. L. 105-34, enacted August 5, 1997); modified by SECURE Act of 2019 § 401 (Pub. L. 116-94) + SECURE Act 2.0 of 2022 (Pub. L. 117-328); 2026 limits per IRS Notice 2025-77 (October 2025 cost-of-living adjustment release)".to_string(),
        "26 USC § 4973 — 6% excise tax on excess contributions to IRAs (including Roth); applies until excess is withdrawn or absorbed in subsequent year's contribution allowance".to_string(),
    ];

    Section408aResult {
        annual_contribution_limit_cents,
        phase_out_low_cents: phase_out_low,
        phase_out_high_cents: phase_out_high,
        allowed_contribution_cents,
        catch_up_eligible,
        backdoor_roth_required,
        distribution_qualified,
        distribution_taxable_amount_cents,
        distribution_penalty_amount_cents,
        failure_reasons,
        citation: "26 USC § 408A(a)-(f); 26 USC § 408; 26 USC § 219; 26 USC § 72; 26 USC § 72(t); 26 USC § 401(a)(9); 26 USC § 1411; 26 USC § 415; 26 USC § 4973; Taxpayer Relief Act of 1997 § 302 (Pub. L. 105-34, August 5, 1997); SECURE Act of 2019 § 401 (Pub. L. 116-94); SECURE Act 2.0 of 2022 (Pub. L. 117-328); IRS Notice 2025-77 (2026 inflation-adjusted limits); Treas. Reg. § 1.408A-1 through § 1.408A-10; Form 5498",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn under_50_single_low_income() -> Section408aInput {
        Section408aInput {
            filing_status: FilingStatus::Single,
            modified_agi_cents: 5_000_000,
            tax_year: 2026,
            age: 35,
            roth_contribution_cents: 750_000,
            years_since_first_roth_contribution: 0,
            distribution_path: DistributionPath::NoDistribution,
            distribution_amount_cents: 0,
            years_since_conversion: 0,
            disability_engaged: false,
            post_death_beneficiary_distribution: false,
            first_time_home_purchase: false,
            cumulative_first_home_prior_withdrawals_cents: 0,
        }
    }

    #[test]
    fn under_50_2026_base_limit_7500() {
        let r = check(&under_50_single_low_income());
        assert_eq!(r.annual_contribution_limit_cents, 750_000);
        assert!(!r.catch_up_eligible);
    }

    #[test]
    fn age_50_catch_up_8600() {
        let mut i = under_50_single_low_income();
        i.age = 50;
        let r = check(&i);
        assert_eq!(r.annual_contribution_limit_cents, 860_000);
        assert!(r.catch_up_eligible);
    }

    #[test]
    fn single_phase_out_thresholds_2026() {
        let r = check(&under_50_single_low_income());
        assert_eq!(r.phase_out_low_cents, 15_300_000);
        assert_eq!(r.phase_out_high_cents, 16_800_000);
    }

    #[test]
    fn mfj_phase_out_thresholds_2026() {
        let mut i = under_50_single_low_income();
        i.filing_status = FilingStatus::MarriedFilingJointly;
        let r = check(&i);
        assert_eq!(r.phase_out_low_cents, 24_200_000);
        assert_eq!(r.phase_out_high_cents, 25_200_000);
    }

    #[test]
    fn mfs_phase_out_thresholds_zero_to_10k() {
        let mut i = under_50_single_low_income();
        i.filing_status = FilingStatus::MarriedFilingSeparately;
        let r = check(&i);
        assert_eq!(r.phase_out_low_cents, 0);
        assert_eq!(r.phase_out_high_cents, 1_000_000);
    }

    #[test]
    fn below_phase_out_full_contribution_allowed() {
        let r = check(&under_50_single_low_income());
        assert_eq!(r.allowed_contribution_cents, 750_000);
        assert!(!r.backdoor_roth_required);
    }

    #[test]
    fn above_phase_out_zero_contribution_backdoor_required() {
        let mut i = under_50_single_low_income();
        i.modified_agi_cents = 17_000_000;
        let r = check(&i);
        assert_eq!(r.allowed_contribution_cents, 0);
        assert!(r.backdoor_roth_required);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 408A(c)(3)")
            && f.contains("BACKDOOR ROTH")
            && f.contains("§ 408A(e)")));
    }

    #[test]
    fn in_phase_out_partial_contribution_allowed() {
        let mut i = under_50_single_low_income();
        i.modified_agi_cents = 16_050_000;
        let r = check(&i);
        assert!(r.allowed_contribution_cents < 750_000);
        assert!(r.allowed_contribution_cents > 0);
    }

    #[test]
    fn excess_contribution_violation() {
        let mut i = under_50_single_low_income();
        i.modified_agi_cents = 16_050_000;
        i.roth_contribution_cents = 750_000;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 408A(c)(1)")
            && f.contains("§ 4973")
            && f.contains("6% excise tax")));
    }

    #[test]
    fn qualified_distribution_5_year_age_60() {
        let mut i = under_50_single_low_income();
        i.age = 60;
        i.years_since_first_roth_contribution = 5;
        i.distribution_path = DistributionPath::Earnings;
        i.distribution_amount_cents = 1_000_000;
        let r = check(&i);
        assert!(r.distribution_qualified);
        assert_eq!(r.distribution_taxable_amount_cents, 0);
        assert_eq!(r.distribution_penalty_amount_cents, 0);
    }

    #[test]
    fn non_qualified_earnings_under_5_year_taxable_plus_penalty() {
        let mut i = under_50_single_low_income();
        i.age = 60;
        i.years_since_first_roth_contribution = 4;
        i.distribution_path = DistributionPath::Earnings;
        i.distribution_amount_cents = 1_000_000;
        let r = check(&i);
        assert!(!r.distribution_qualified);
        assert_eq!(r.distribution_taxable_amount_cents, 1_000_000);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 408A(d)(2)") && f.contains("5-YEAR HOLDING PERIOD")));
    }

    #[test]
    fn earnings_pre_59_5_taxable_plus_10_percent_penalty() {
        let mut i = under_50_single_low_income();
        i.age = 40;
        i.years_since_first_roth_contribution = 10;
        i.distribution_path = DistributionPath::Earnings;
        i.distribution_amount_cents = 1_000_000;
        let r = check(&i);
        assert!(!r.distribution_qualified);
        assert_eq!(r.distribution_taxable_amount_cents, 1_000_000);
        assert_eq!(r.distribution_penalty_amount_cents, 100_000);
    }

    #[test]
    fn disability_qualifies_distribution_under_age() {
        let mut i = under_50_single_low_income();
        i.age = 40;
        i.years_since_first_roth_contribution = 5;
        i.disability_engaged = true;
        i.distribution_path = DistributionPath::Earnings;
        i.distribution_amount_cents = 1_000_000;
        let r = check(&i);
        assert!(r.distribution_qualified);
    }

    #[test]
    fn post_death_beneficiary_qualifies_distribution() {
        let mut i = under_50_single_low_income();
        i.age = 30;
        i.years_since_first_roth_contribution = 5;
        i.post_death_beneficiary_distribution = true;
        i.distribution_path = DistributionPath::Earnings;
        i.distribution_amount_cents = 1_000_000;
        let r = check(&i);
        assert!(r.distribution_qualified);
    }

    #[test]
    fn first_home_purchase_qualifies_under_10k_cap() {
        let mut i = under_50_single_low_income();
        i.age = 35;
        i.years_since_first_roth_contribution = 5;
        i.first_time_home_purchase = true;
        i.cumulative_first_home_prior_withdrawals_cents = 0;
        i.distribution_path = DistributionPath::Earnings;
        i.distribution_amount_cents = 500_000;
        let r = check(&i);
        assert!(r.distribution_qualified);
    }

    #[test]
    fn first_home_cumulative_10k_exhausts_lifetime_cap() {
        let mut i = under_50_single_low_income();
        i.age = 35;
        i.first_time_home_purchase = true;
        i.cumulative_first_home_prior_withdrawals_cents = 1_000_000;
        i.distribution_path = DistributionPath::Earnings;
        i.distribution_amount_cents = 500_000;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 72(t)(2)(F)") && f.contains("$10,000 LIFETIME")));
    }

    #[test]
    fn conversion_principal_within_5_year_10_percent_penalty() {
        let mut i = under_50_single_low_income();
        i.age = 40;
        i.distribution_path = DistributionPath::ConversionPrincipal;
        i.distribution_amount_cents = 1_000_000;
        i.years_since_conversion = 3;
        let r = check(&i);
        assert_eq!(r.distribution_taxable_amount_cents, 0);
        assert_eq!(r.distribution_penalty_amount_cents, 100_000);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 408A(d)(3)(A)") && f.contains("5-YEAR per-conversion")));
    }

    #[test]
    fn conversion_principal_after_5_year_no_penalty() {
        let mut i = under_50_single_low_income();
        i.age = 40;
        i.distribution_path = DistributionPath::ConversionPrincipal;
        i.distribution_amount_cents = 1_000_000;
        i.years_since_conversion = 5;
        let r = check(&i);
        assert_eq!(r.distribution_penalty_amount_cents, 0);
    }

    #[test]
    fn contribution_principal_always_tax_free_penalty_free() {
        let mut i = under_50_single_low_income();
        i.age = 25;
        i.distribution_path = DistributionPath::ContributionPrincipal;
        i.distribution_amount_cents = 500_000;
        let r = check(&i);
        assert_eq!(r.distribution_taxable_amount_cents, 0);
        assert_eq!(r.distribution_penalty_amount_cents, 0);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&under_50_single_low_income());
        assert!(r.citation.contains("§ 408A(a)-(f)"));
        assert!(r.citation.contains("§ 408"));
        assert!(r.citation.contains("§ 219"));
        assert!(r.citation.contains("§ 72(t)"));
        assert!(r.citation.contains("§ 401(a)(9)"));
        assert!(r.citation.contains("§ 1411"));
        assert!(r.citation.contains("§ 415"));
        assert!(r.citation.contains("§ 4973"));
        assert!(r.citation.contains("Taxpayer Relief Act of 1997 § 302"));
        assert!(r.citation.contains("Pub. L. 105-34"));
        assert!(r.citation.contains("August 5, 1997"));
        assert!(r.citation.contains("SECURE Act of 2019 § 401"));
        assert!(r.citation.contains("SECURE Act 2.0 of 2022"));
        assert!(r.citation.contains("IRS Notice 2025-77"));
        assert!(r
            .citation
            .contains("Treas. Reg. § 1.408A-1 through § 1.408A-10"));
        assert!(r.citation.contains("Form 5498"));
    }

    #[test]
    fn note_pins_2026_limits_7500_1100_catchup() {
        let r = check(&under_50_single_low_income());
        assert!(r.notes.iter().any(|n| n.contains("§ 408A(c)(1)")
            && n.contains("$7,500 base")
            && n.contains("$1,100")
            && n.contains("$8,600")));
    }

    #[test]
    fn note_pins_2026_phase_out_thresholds() {
        let r = check(&under_50_single_low_income());
        assert!(r.notes.iter().any(|n| n.contains("§ 408A(c)(3)")
            && n.contains("$153K-$168K")
            && n.contains("$242K-$252K")
            && n.contains("$0-$10K")));
    }

    #[test]
    fn note_pins_magi_disregards_conversion_income() {
        let r = check(&under_50_single_low_income());
        assert!(r.notes.iter().any(
            |n| n.contains("§ 408A(c)(3)(B)") && n.contains("DISREGARDING any Roth conversion")
        ));
    }

    #[test]
    fn note_pins_qualified_distribution_two_prong() {
        let r = check(&under_50_single_low_income());
        assert!(r.notes.iter().any(|n| n.contains("§ 408A(d)(2)")
            && n.contains("5-YEAR HOLDING PERIOD")
            && n.contains("age 59½+")
            && n.contains("$10,000 lifetime")));
    }

    #[test]
    fn note_pins_d3_ordering_rules() {
        let r = check(&under_50_single_low_income());
        assert!(r.notes.iter().any(|n| n.contains("§ 408A(d)(3)")
            && n.contains("Contributions")
            && n.contains("Conversions")
            && n.contains("Earnings")));
    }

    #[test]
    fn note_pins_d3a_conversion_5_year_per_conversion() {
        let r = check(&under_50_single_low_income());
        assert!(r.notes.iter().any(|n| n.contains("§ 408A(d)(3)(A)")
            && n.contains("5-year holding period applies to EACH conversion")
            && n.contains("stacked 5-year clocks")));
    }

    #[test]
    fn note_pins_e_backdoor_roth_pro_rata() {
        let r = check(&under_50_single_low_income());
        assert!(r.notes.iter().any(|n| n.contains("§ 408A(e)")
            && n.contains("BACKDOOR ROTH")
            && n.contains("§ 408(d)(2)")
            && n.contains("pro-rata rule")));
    }

    #[test]
    fn note_pins_c5_no_rmd_during_lifetime() {
        let r = check(&under_50_single_low_income());
        assert!(r.notes.iter().any(|n| n.contains("§ 408A(c)(5)")
            && n.contains("NOT subject to § 401(a)(9)")
            && n.contains("SECURE Act § 401(a)(9)(H)")));
    }

    #[test]
    fn note_pins_1411_niit_exempt() {
        let r = check(&under_50_single_low_income());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1411 NIIT 3.8%") && n.contains("not investment income")));
    }

    #[test]
    fn note_pins_trader_critical_fact_patterns_four() {
        let r = check(&under_50_single_low_income());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Trader-critical fact patterns")
                && n.contains("backdoor Roth")
                && n.contains("self-directed Roth IRA")
                && n.contains("mega backdoor Roth")
                && n.contains("§ 72(t) substantially-equal-periodic-payments")));
    }

    #[test]
    fn note_pins_taxpayer_relief_act_1997_origin() {
        let r = check(&under_50_single_low_income());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Taxpayer Relief Act of 1997 § 302")
                && n.contains("Pub. L. 105-34")
                && n.contains("August 5, 1997")
                && n.contains("SECURE Act of 2019")));
    }

    #[test]
    fn note_pins_4973_excess_contribution_excise_tax() {
        let r = check(&under_50_single_low_income());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 4973") && n.contains("6% excise tax")));
    }

    #[test]
    fn filing_status_truth_table_four_cells() {
        for status in [
            FilingStatus::Single,
            FilingStatus::HeadOfHousehold,
            FilingStatus::MarriedFilingJointly,
            FilingStatus::MarriedFilingSeparately,
        ] {
            let mut i = under_50_single_low_income();
            i.filing_status = status;
            let r = check(&i);
            let _ = r.phase_out_low_cents;
        }
    }

    #[test]
    fn mfj_uniquely_highest_phase_out_invariant() {
        let mut mfj = under_50_single_low_income();
        mfj.filing_status = FilingStatus::MarriedFilingJointly;
        let r_mfj = check(&mfj);

        for status in [FilingStatus::Single, FilingStatus::HeadOfHousehold] {
            let mut i = under_50_single_low_income();
            i.filing_status = status;
            let r = check(&i);
            assert!(r_mfj.phase_out_high_cents > r.phase_out_high_cents);
        }
    }

    #[test]
    fn mfs_uniquely_zero_low_threshold_invariant() {
        let mut mfs = under_50_single_low_income();
        mfs.filing_status = FilingStatus::MarriedFilingSeparately;
        let r_mfs = check(&mfs);
        assert_eq!(r_mfs.phase_out_low_cents, 0);

        for status in [
            FilingStatus::Single,
            FilingStatus::HeadOfHousehold,
            FilingStatus::MarriedFilingJointly,
        ] {
            let mut i = under_50_single_low_income();
            i.filing_status = status;
            let r = check(&i);
            assert!(r.phase_out_low_cents > 0);
        }
    }

    #[test]
    fn defensive_overflow_saturating() {
        let mut i = under_50_single_low_income();
        i.modified_agi_cents = u64::MAX;
        let r = check(&i);
        assert_eq!(r.allowed_contribution_cents, 0);
    }
}
