//! IRC § 4943 — Taxes on excess business holdings of
//! private foundations. Limits combined holdings of a
//! private foundation and its disqualified persons in a
//! single business enterprise to prevent PF use to
//! finance family business. Direct PF Chapter 42 companion
//! to section_4940 (PF NII excise — iter 470), section_4941
//! (PF self-dealing — iter 468), section_4942 (PF minimum
//! distribution — iter 472), section_4958 (intermediate
//! sanctions for public charities — iter 466), section_4960
//! (ATEO executive comp 21% — iter 464). Originally
//! enacted by Tax Reform Act of 1969, Pub. L. 91-172.
//!
//! TWO-TIER excise tax structure:
//! - § 4943(a)(1) TIER 1: 10% of value of excess business
//!   holdings determined as of date of greatest excess
//!   during taxable year
//! - § 4943(b) TIER 2: 200% of value of excess business
//!   holdings remaining at end of taxable period (PF must
//!   dispose within IRS-notice correction window)
//!
//! Combined holding limits under § 4943(c)(2):
//! - § 4943(c)(2)(A) DEFAULT 20% LIMIT: combined holdings
//!   of PF and ALL disqualified persons (§ 4946) may not
//!   exceed 20% of voting stock of corporation (or
//!   equivalent profits interest in partnership / joint
//!   venture / unincorporated enterprise)
//! - § 4943(c)(2)(B) 35% LIMIT WITH EFFECTIVE CONTROL: if
//!   PF establishes to IRS satisfaction that effective
//!   control of business is in one or more NON-disqualified
//!   persons, 20% combined limit raised to 35%
//! - § 4943(c)(2)(C) 2% DE MINIMIS: regardless of combined
//!   holdings, PF may hold up to 2% of voting stock (or
//!   equivalent) of any single business enterprise
//!
//! Non-voting stock rule per § 4943(c)(3)(B): PF may hold
//! ALL non-voting stock if all DPs combined hold no more
//! than the 20% (or 35%) voting limit. Non-voting holdings
//! count separately from voting-stock combined-limit
//! calculation.
//!
//! Business enterprise per § 4943(d)(3) EXCLUDES:
//! 1. § 4943(d)(3)(A) — "functionally-related business"
//!    that is substantially related to PF exempt purpose
//! 2. § 4943(d)(3)(B) — "95% passive income test" trade
//!    or business in which at least 95% of gross income is
//!    from PASSIVE SOURCES (interest, dividends, rents,
//!    royalties, capital gains)
//! 3. Program-related investments under § 4944(c)
//!
//! § 4943(c)(6) FIVE-YEAR DISPOSITION PERIOD: where PF
//! acquires excess business holdings by GIFT, BEQUEST, or
//! DEVISE, PF has 5 years from date of acquisition to
//! dispose of excess. § 4943(c)(7) authorizes IRS to grant
//! ADDITIONAL 5-YEAR extension for "complex" or "unusual"
//! estate plans where diligent efforts to dispose have been
//! made.
//!
//! § 4943(g) FAMILY BUSINESS EXCEPTION (added by Tax Cuts
//! and Jobs Act of 2017, Pub. L. 115-97, Dec. 22, 2017):
//! permits 100% PF ownership of "philanthropic business
//! holding" if ALL THREE conditions met:
//! 1. PF owns ALL voting stock at all times during the
//!    taxable year
//! 2. PF received the voting stock by means OTHER THAN
//!    PURCHASE (e.g., by gift, devise, or bequest)
//! 3. Operating requirements: all net operating income
//!    (after taxes + reserves) is distributed to PF
//!    annually + no DP serves as director/officer/employee
//!
//! Trader-foundation critical because (1) family-owned
//! business that is partially gifted to PF must be
//! carefully structured to avoid combined 20% cap with
//! family-DP holdings; (2) Tier-2 200% tax is effectively
//! CONFISCATORY — PF must dispose during taxable period
//! or face penalty exceeding asset value; (3) 5-year
//! disposition window for inherited interest is a
//! critical estate planning constraint; (4) § 4943(g)
//! TCJA 2017 family business exception allows PFs to hold
//! 100% of family business if all three conditions
//! satisfied — most valuable when family wants to
//! perpetuate business through PF; (5) 35% effective-
//! control exception requires PF to demonstrate
//! independent control which is rare in family foundation
//! contexts.
//!
//! Distinction from § 4942 (iter 472): § 4942 is annual
//! minimum-DISTRIBUTION requirement; § 4943 is a CAPITAL
//! HOLDINGS limit on PF business enterprise concentration.
//!
//! Authority: 26 U.S.C. § 4943; § 4943(a)(1); § 4943(b);
//! § 4943(c)(1); § 4943(c)(2)(A); § 4943(c)(2)(B);
//! § 4943(c)(2)(C); § 4943(c)(3)(A); § 4943(c)(3)(B);
//! § 4943(c)(4); § 4943(c)(5); § 4943(c)(6); § 4943(c)(7);
//! § 4943(d)(1); § 4943(d)(2); § 4943(d)(3); § 4943(d)(4);
//! § 4943(e); § 4943(f); § 4943(g) (Tax Cuts and Jobs Act
//! 2017 family business exception); § 4944(c); § 4946;
//! 26 C.F.R. § 53.4943-1 through § 53.4943-10; Tax Reform
//! Act of 1969, Pub. L. 91-172 (Dec. 30, 1969) — original
//! § 4943 enactment; Tax Cuts and Jobs Act of 2017,
//! Pub. L. 115-97 (Dec. 22, 2017) — § 4943(g) family
//! business exception.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FoundationStatus {
    PrivateFoundation,
    PublicCharity,
    NonExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BusinessEnterpriseExclusion {
    None,
    FunctionallyRelatedBusiness,
    PassiveIncome95Percent,
    ProgramRelatedInvestment,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub foundation_status: FoundationStatus,
    pub enterprise_exclusion: BusinessEnterpriseExclusion,
    pub pf_voting_percentage_basis_points: u32,
    pub combined_pf_and_dp_voting_percentage_basis_points: u32,
    pub effective_control_in_non_dp_satisfied: bool,
    pub section_4943g_family_business_exception_applies: bool,
    pub holdings_acquired_by_gift_bequest_devise: bool,
    pub months_since_gift_or_bequest_acquisition: u32,
    pub irs_4943c7_extension_granted: bool,
    pub corrected_within_taxable_period: bool,
    pub excess_holdings_value_cents: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    ExcludedEnterprise,
    FamilyBusinessExceptionApplies,
    WithinFiveYearDispositionWindow,
    Compliant,
    Tier1TaxOwed,
    Tier2TaxOwed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub applicable_combined_limit_basis_points: u32,
    pub tier1_tax_cents: u64,
    pub tier2_tax_cents: u64,
    pub total_tax_cents: u64,
    pub notes: Vec<String>,
}

pub const DEFAULT_LIMIT_BPS: u32 = 2000; // 20%
pub const EFFECTIVE_CONTROL_LIMIT_BPS: u32 = 3500; // 35%
pub const DE_MINIMIS_BPS: u32 = 200; // 2%
pub const TIER1_RATE_PCT: u64 = 10;
pub const TIER2_RATE_PCT: u64 = 200;
pub const FIVE_YEAR_DISPOSITION_MAX_MONTHS: u32 = 60;
pub const TEN_YEAR_EXTENSION_MAX_MONTHS: u32 = 120;

pub type Section4943Input = Input;
pub type Section4943Result = Output;

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "§ 4943(a)(1) TIER-1 10% excise tax on value of excess business holdings as of date of greatest excess during taxable year; § 4943(b) TIER-2 200% excise tax on remaining excess holdings at end of taxable period (after IRS-notice correction window).".to_string(),
        "Combined holding limits under § 4943(c)(2): § 4943(c)(2)(A) default 20% limit on combined PF + all DPs (§ 4946) voting stock of corporation (or equivalent profits interest); § 4943(c)(2)(B) raised to 35% if PF establishes effective control of business is in non-DPs; § 4943(c)(2)(C) 2% de minimis — PF alone may hold up to 2% of voting stock regardless of combined holdings.".to_string(),
        "Non-voting stock per § 4943(c)(3)(B): PF may hold ALL non-voting stock if combined DP voting holdings do not exceed 20% (or 35%) limit. Non-voting holdings separate from voting-limit calculation.".to_string(),
        "Business enterprise per § 4943(d)(3) EXCLUDES: § 4943(d)(3)(A) functionally-related business substantially related to PF exempt purpose; § 4943(d)(3)(B) 95% passive income test trade or business with at least 95% gross income from interest + dividends + rents + royalties + capital gains; § 4944(c) program-related investments.".to_string(),
        "§ 4943(c)(6) FIVE-YEAR DISPOSITION PERIOD for holdings acquired by GIFT, BEQUEST, or DEVISE; § 4943(c)(7) IRS authorized to grant ADDITIONAL 5-YEAR (10-year total) extension for complex or unusual estate plans where diligent disposition efforts made.".to_string(),
        "§ 4943(g) FAMILY BUSINESS EXCEPTION (added by Tax Cuts and Jobs Act 2017, Pub. L. 115-97, Dec. 22, 2017): permits 100% PF ownership of philanthropic business holding if ALL THREE: (1) PF owns ALL voting stock at all times during the year; (2) PF received voting stock by means OTHER THAN PURCHASE (gift/devise/bequest); (3) all net operating income (after taxes + reserves) distributed annually to PF + no DP serves as director/officer/employee.".to_string(),
        "Distinction from § 4942 (iter 472): § 4942 is annual minimum-DISTRIBUTION requirement; § 4943 is CAPITAL HOLDINGS limit on PF business enterprise concentration.".to_string(),
        "Companion: section_4940 (iter 470), section_4941 (iter 468), section_4942 (iter 472), section_4958 (iter 466), section_4960 (iter 464), section_4973 (iter 442), section_4974 (iter 436), section_4975 (iter 434), section_4980 (iter 460).".to_string(),
    ];

    if !matches!(input.foundation_status, FoundationStatus::PrivateFoundation) {
        let mut n = notes;
        n.push("Organization is not a private foundation — § 4943 does not apply.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            applicable_combined_limit_basis_points: 0,
            tier1_tax_cents: 0,
            tier2_tax_cents: 0,
            total_tax_cents: 0,
            notes: n,
        };
    }

    if !matches!(
        input.enterprise_exclusion,
        BusinessEnterpriseExclusion::None
    ) {
        let mut n = notes;
        let citation = match input.enterprise_exclusion {
            BusinessEnterpriseExclusion::FunctionallyRelatedBusiness => {
                "§ 4943(d)(3)(A) functionally-related business"
            }
            BusinessEnterpriseExclusion::PassiveIncome95Percent => {
                "§ 4943(d)(3)(B) 95% passive income test"
            }
            BusinessEnterpriseExclusion::ProgramRelatedInvestment => {
                "§ 4944(c) program-related investment"
            }
            BusinessEnterpriseExclusion::None => unreachable!(),
        };
        n.push(format!(
            "Business enterprise excluded from § 4943: {} — § 4943 does not apply to this holding.",
            citation
        ));
        return Output {
            severity: Severity::ExcludedEnterprise,
            applicable_combined_limit_basis_points: 0,
            tier1_tax_cents: 0,
            tier2_tax_cents: 0,
            total_tax_cents: 0,
            notes: n,
        };
    }

    if input.section_4943g_family_business_exception_applies {
        let mut n = notes;
        n.push("§ 4943(g) family business exception applies — 100% PF ownership permitted; § 4943 does not impose tax on this holding.".to_string());
        return Output {
            severity: Severity::FamilyBusinessExceptionApplies,
            applicable_combined_limit_basis_points: 10_000,
            tier1_tax_cents: 0,
            tier2_tax_cents: 0,
            total_tax_cents: 0,
            notes: n,
        };
    }

    let max_disposition_months = if input.irs_4943c7_extension_granted {
        TEN_YEAR_EXTENSION_MAX_MONTHS
    } else {
        FIVE_YEAR_DISPOSITION_MAX_MONTHS
    };

    if input.holdings_acquired_by_gift_bequest_devise
        && input.months_since_gift_or_bequest_acquisition <= max_disposition_months
    {
        let mut n = notes;
        n.push(format!(
            "§ 4943(c)(6) {}-year disposition window applies: {} months since gift/bequest acquisition; PF has until month {} to dispose of excess.",
            max_disposition_months / 12,
            input.months_since_gift_or_bequest_acquisition,
            max_disposition_months
        ));
        return Output {
            severity: Severity::WithinFiveYearDispositionWindow,
            applicable_combined_limit_basis_points: 0,
            tier1_tax_cents: 0,
            tier2_tax_cents: 0,
            total_tax_cents: 0,
            notes: n,
        };
    }

    let applicable_limit = if input.effective_control_in_non_dp_satisfied {
        EFFECTIVE_CONTROL_LIMIT_BPS
    } else {
        DEFAULT_LIMIT_BPS
    };

    let combined_over_limit =
        input.combined_pf_and_dp_voting_percentage_basis_points > applicable_limit;
    let pf_over_de_minimis = input.pf_voting_percentage_basis_points > DE_MINIMIS_BPS;

    if !combined_over_limit || !pf_over_de_minimis {
        let mut n = notes;
        if !combined_over_limit {
            n.push(format!(
                "Combined PF + DP voting holdings {} bps within applicable {} bps limit — no excess business holdings.",
                input.combined_pf_and_dp_voting_percentage_basis_points, applicable_limit
            ));
        } else {
            n.push("PF holdings within 2% de minimis under § 4943(c)(2)(C) — no excess business holdings regardless of combined DP holdings.".to_string());
        }
        return Output {
            severity: Severity::Compliant,
            applicable_combined_limit_basis_points: applicable_limit,
            tier1_tax_cents: 0,
            tier2_tax_cents: 0,
            total_tax_cents: 0,
            notes: n,
        };
    }

    let tier1_tax = input
        .excess_holdings_value_cents
        .saturating_mul(TIER1_RATE_PCT)
        .checked_div(100)
        .unwrap_or(0);

    let tier2_tax = if input.corrected_within_taxable_period {
        0
    } else {
        input
            .excess_holdings_value_cents
            .saturating_mul(TIER2_RATE_PCT)
            .checked_div(100)
            .unwrap_or(0)
    };

    let total_tax = tier1_tax.saturating_add(tier2_tax);

    let severity = if tier2_tax > 0 {
        Severity::Tier2TaxOwed
    } else {
        Severity::Tier1TaxOwed
    };

    let mut n = notes;
    n.push(format!(
        "Excess business holdings § 4943 tax: Tier-1 10%: ${}.{:02}; Tier-2 200%: ${}.{:02}; Total: ${}.{:02} (applicable limit {} bps).",
        tier1_tax / 100,
        tier1_tax % 100,
        tier2_tax / 100,
        tier2_tax % 100,
        total_tax / 100,
        total_tax % 100,
        applicable_limit
    ));

    Output {
        severity,
        applicable_combined_limit_basis_points: applicable_limit,
        tier1_tax_cents: tier1_tax,
        tier2_tax_cents: tier2_tax,
        total_tax_cents: total_tax,
        notes: n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            foundation_status: FoundationStatus::PrivateFoundation,
            enterprise_exclusion: BusinessEnterpriseExclusion::None,
            pf_voting_percentage_basis_points: 1500, // 15%
            combined_pf_and_dp_voting_percentage_basis_points: 2500, // 25% — over 20% limit
            effective_control_in_non_dp_satisfied: false,
            section_4943g_family_business_exception_applies: false,
            holdings_acquired_by_gift_bequest_devise: false,
            months_since_gift_or_bequest_acquisition: 0,
            irs_4943c7_extension_granted: false,
            corrected_within_taxable_period: false,
            excess_holdings_value_cents: 500_000_00, // $500K excess
        }
    }

    #[test]
    fn public_charity_not_subject_to_4943() {
        let mut i = baseline();
        i.foundation_status = FoundationStatus::PublicCharity;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn non_exempt_not_applicable() {
        let mut i = baseline();
        i.foundation_status = FoundationStatus::NonExempt;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn functionally_related_business_excluded() {
        let mut i = baseline();
        i.enterprise_exclusion = BusinessEnterpriseExclusion::FunctionallyRelatedBusiness;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExcludedEnterprise);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4943(d)(3)(A)"));
    }

    #[test]
    fn passive_income_95_percent_excluded() {
        let mut i = baseline();
        i.enterprise_exclusion = BusinessEnterpriseExclusion::PassiveIncome95Percent;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExcludedEnterprise);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4943(d)(3)(B)"));
        assert!(joined.contains("95% passive"));
    }

    #[test]
    fn program_related_investment_excluded() {
        let mut i = baseline();
        i.enterprise_exclusion = BusinessEnterpriseExclusion::ProgramRelatedInvestment;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExcludedEnterprise);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4944(c)"));
    }

    #[test]
    fn section_4943g_family_business_exception() {
        let mut i = baseline();
        i.section_4943g_family_business_exception_applies = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::FamilyBusinessExceptionApplies);
        assert_eq!(out.applicable_combined_limit_basis_points, 10_000);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4943(g)"));
        assert!(joined.contains("100% PF ownership"));
    }

    #[test]
    fn five_year_disposition_window_within_60_months() {
        let mut i = baseline();
        i.holdings_acquired_by_gift_bequest_devise = true;
        i.months_since_gift_or_bequest_acquisition = 36;
        let out = check(&i);
        assert_eq!(out.severity, Severity::WithinFiveYearDispositionWindow);
        let joined = out.notes.join(" ");
        assert!(joined.contains("5-year disposition"));
        assert!(joined.contains("36 months"));
    }

    #[test]
    fn five_year_disposition_window_exactly_60_months() {
        let mut i = baseline();
        i.holdings_acquired_by_gift_bequest_devise = true;
        i.months_since_gift_or_bequest_acquisition = 60;
        let out = check(&i);
        assert_eq!(out.severity, Severity::WithinFiveYearDispositionWindow);
    }

    #[test]
    fn five_year_disposition_window_61_months_no_relief() {
        let mut i = baseline();
        i.holdings_acquired_by_gift_bequest_devise = true;
        i.months_since_gift_or_bequest_acquisition = 61;
        let out = check(&i);
        // Past 5 years, tier-2 applies
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn ten_year_irs_extension_window() {
        let mut i = baseline();
        i.holdings_acquired_by_gift_bequest_devise = true;
        i.months_since_gift_or_bequest_acquisition = 96; // 8 years
        i.irs_4943c7_extension_granted = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::WithinFiveYearDispositionWindow);
    }

    #[test]
    fn ten_year_extension_exactly_120_months() {
        let mut i = baseline();
        i.holdings_acquired_by_gift_bequest_devise = true;
        i.months_since_gift_or_bequest_acquisition = 120;
        i.irs_4943c7_extension_granted = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::WithinFiveYearDispositionWindow);
    }

    #[test]
    fn ten_year_extension_121_months_no_relief() {
        let mut i = baseline();
        i.holdings_acquired_by_gift_bequest_devise = true;
        i.months_since_gift_or_bequest_acquisition = 121;
        i.irs_4943c7_extension_granted = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn combined_at_20_percent_compliant() {
        let mut i = baseline();
        i.combined_pf_and_dp_voting_percentage_basis_points = 2000; // exactly 20%
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
        assert_eq!(
            out.applicable_combined_limit_basis_points,
            DEFAULT_LIMIT_BPS
        );
    }

    #[test]
    fn combined_at_20_point_01_percent_over_limit() {
        let mut i = baseline();
        i.combined_pf_and_dp_voting_percentage_basis_points = 2001; // just over 20%
        let out = check(&i);
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn pf_under_2_percent_de_minimis_safe_harbor() {
        let mut i = baseline();
        i.pf_voting_percentage_basis_points = 200; // exactly 2%
        i.combined_pf_and_dp_voting_percentage_basis_points = 4000; // 40% combined
        let out = check(&i);
        // PF at 2% de minimis — compliant under § 4943(c)(2)(C) regardless of DPs
        assert_eq!(out.severity, Severity::Compliant);
        let joined = out.notes.join(" ");
        assert!(joined.contains("2% de minimis"));
        assert!(joined.contains("§ 4943(c)(2)(C)"));
    }

    #[test]
    fn pf_at_2_point_01_percent_above_de_minimis() {
        let mut i = baseline();
        i.pf_voting_percentage_basis_points = 201;
        i.combined_pf_and_dp_voting_percentage_basis_points = 4000;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn effective_control_35_percent_limit_applies() {
        let mut i = baseline();
        i.effective_control_in_non_dp_satisfied = true;
        i.combined_pf_and_dp_voting_percentage_basis_points = 3000; // 30%
        let out = check(&i);
        // 30% < 35% — compliant under § 4943(c)(2)(B)
        assert_eq!(out.severity, Severity::Compliant);
        assert_eq!(out.applicable_combined_limit_basis_points, 3500);
    }

    #[test]
    fn effective_control_over_35_percent_violates() {
        let mut i = baseline();
        i.effective_control_in_non_dp_satisfied = true;
        i.combined_pf_and_dp_voting_percentage_basis_points = 3600; // over 35%
        let out = check(&i);
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
        assert_eq!(out.applicable_combined_limit_basis_points, 3500);
    }

    #[test]
    fn tier_1_only_when_corrected_within_taxable_period() {
        let mut i = baseline();
        i.corrected_within_taxable_period = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Tier1TaxOwed);
        // Tier-1 = 10% × $500K = $50K
        assert_eq!(out.tier1_tax_cents, 50_000_00);
        assert_eq!(out.tier2_tax_cents, 0);
    }

    #[test]
    fn tier_2_when_uncorrected() {
        let i = baseline();
        let out = check(&i);
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
        // Tier-1 $50K + Tier-2 $1M = $1.05M
        assert_eq!(out.tier1_tax_cents, 50_000_00);
        assert_eq!(out.tier2_tax_cents, 1_000_000_00);
        assert_eq!(out.total_tax_cents, 1_050_000_00);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4943(a)(1)"));
        assert!(joined.contains("§ 4943(b)"));
        assert!(joined.contains("§ 4943(c)(2)"));
        assert!(joined.contains("§ 4943(c)(2)(A)"));
        assert!(joined.contains("§ 4943(c)(2)(B)"));
        assert!(joined.contains("§ 4943(c)(2)(C)"));
        assert!(joined.contains("§ 4943(c)(3)(B)"));
        assert!(joined.contains("§ 4943(c)(6)"));
        assert!(joined.contains("§ 4943(c)(7)"));
        assert!(joined.contains("§ 4943(d)(3)(A)"));
        assert!(joined.contains("§ 4943(d)(3)(B)"));
        assert!(joined.contains("§ 4943(g)"));
        assert!(joined.contains("§ 4944(c)"));
        assert!(joined.contains("§ 4946"));
        assert!(joined.contains("§ 4942 (iter 472)"));
        assert!(joined.contains("Tax Cuts and Jobs Act 2017"));
        assert!(joined.contains("Pub. L. 115-97"));
    }

    #[test]
    fn note_pins_two_tier_10_200_structure() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("10%"));
        assert!(joined.contains("200%"));
    }

    #[test]
    fn note_pins_three_limit_categories() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("default 20%"));
        assert!(joined.contains("raised to 35%"));
        assert!(joined.contains("2% de minimis"));
    }

    #[test]
    fn note_pins_three_business_enterprise_exclusions() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("functionally-related business"));
        assert!(joined.contains("95% passive income"));
        assert!(joined.contains("program-related investments"));
    }

    #[test]
    fn note_pins_5_year_disposition_window() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("FIVE-YEAR DISPOSITION"));
        assert!(joined.contains("GIFT, BEQUEST, or DEVISE"));
        assert!(joined.contains("10-year total"));
    }

    #[test]
    fn note_pins_4943g_family_business_three_conditions() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("FAMILY BUSINESS EXCEPTION"));
        assert!(joined.contains("100% PF ownership"));
        assert!(joined.contains("ALL voting stock"));
        assert!(joined.contains("OTHER THAN PURCHASE"));
        assert!(joined.contains("net operating income"));
    }

    #[test]
    fn note_pins_4942_distinction() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4942 (iter 472)"));
        assert!(joined.contains("minimum-DISTRIBUTION"));
        assert!(joined.contains("CAPITAL HOLDINGS"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("section_4940"));
        assert!(joined.contains("section_4941"));
        assert!(joined.contains("section_4942"));
        assert!(joined.contains("section_4958"));
        assert!(joined.contains("section_4960"));
    }

    #[test]
    fn severity_truth_table_six_cells() {
        // Public charity → NotApplicable
        let c1 = check(&Input {
            foundation_status: FoundationStatus::PublicCharity,
            ..baseline()
        });
        assert_eq!(c1.severity, Severity::NotApplicable);

        // PF + excluded enterprise → ExcludedEnterprise
        let c2 = check(&Input {
            enterprise_exclusion: BusinessEnterpriseExclusion::PassiveIncome95Percent,
            ..baseline()
        });
        assert_eq!(c2.severity, Severity::ExcludedEnterprise);

        // § 4943(g) family business → FamilyBusinessExceptionApplies
        let c3 = check(&Input {
            section_4943g_family_business_exception_applies: true,
            ..baseline()
        });
        assert_eq!(c3.severity, Severity::FamilyBusinessExceptionApplies);

        // Within 5-year window → WithinFiveYearDispositionWindow
        let c4 = check(&Input {
            holdings_acquired_by_gift_bequest_devise: true,
            months_since_gift_or_bequest_acquisition: 24,
            ..baseline()
        });
        assert_eq!(c4.severity, Severity::WithinFiveYearDispositionWindow);

        // Combined under 20% → Compliant
        let c5 = check(&Input {
            combined_pf_and_dp_voting_percentage_basis_points: 1500,
            ..baseline()
        });
        assert_eq!(c5.severity, Severity::Compliant);

        // Combined over 20% uncorrected → Tier2TaxOwed
        let c6 = check(&baseline());
        assert_eq!(c6.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn saturating_arithmetic_overflow_defense() {
        let mut i = baseline();
        i.excess_holdings_value_cents = u64::MAX;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn boundary_zero_excess_holdings_no_tax() {
        let mut i = baseline();
        i.excess_holdings_value_cents = 0;
        let out = check(&i);
        // 10% × $0 = $0
        assert_eq!(out.tier1_tax_cents, 0);
        assert_eq!(out.tier2_tax_cents, 0);
    }

    #[test]
    fn realistic_family_pf_with_15_percent_holding() {
        // PF holds 15% voting stock; family DPs hold 30% — combined 45% over 20%
        let i = Input {
            foundation_status: FoundationStatus::PrivateFoundation,
            enterprise_exclusion: BusinessEnterpriseExclusion::None,
            pf_voting_percentage_basis_points: 1500,
            combined_pf_and_dp_voting_percentage_basis_points: 4500,
            effective_control_in_non_dp_satisfied: false,
            section_4943g_family_business_exception_applies: false,
            holdings_acquired_by_gift_bequest_devise: false,
            months_since_gift_or_bequest_acquisition: 0,
            irs_4943c7_extension_granted: false,
            corrected_within_taxable_period: false,
            excess_holdings_value_cents: 2_000_000_00, // $2M excess holdings value
        };
        let out = check(&i);
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
        // Tier-1 10% × $2M = $200K + Tier-2 200% × $2M = $4M = $4.2M
        assert_eq!(out.tier1_tax_cents, 200_000_00);
        assert_eq!(out.tier2_tax_cents, 4_000_000_00);
        assert_eq!(out.total_tax_cents, 4_200_000_00);
    }
}
