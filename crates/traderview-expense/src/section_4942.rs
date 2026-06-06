//! IRC § 4942 — Taxes on failure to distribute income.
//! Private foundation minimum distribution requirement.
//! Direct PF Chapter 42 companion to section_4940 (PF NII
//! excise — iter 470), section_4941 (PF self-dealing —
//! iter 468), section_4958 (intermediate sanctions for
//! public charities — iter 466), section_4960 (ATEO
//! executive comp 21% — iter 464). Originally enacted by
//! Tax Reform Act of 1969, Pub. L. 91-172.
//!
//! TWO-TIER excise tax structure:
//! - § 4942(a) TIER 1: 30% of undistributed income for
//!   each year or partial year the deficiency remains
//!   uncorrected
//! - § 4942(b) TIER 2: additional 100% of undistributed
//!   income if PF fails to make up deficient distribution
//!   within 90 days of receiving notice from IRS of
//!   failure to make minimum distribution
//!
//! Distributable amount per § 4942(d): the MINIMUM
//! INVESTMENT RETURN reduced by § 4940 excise tax and UBI
//! tax. Must be distributed as QUALIFYING DISTRIBUTIONS
//! by the end of the IMMEDIATELY FOLLOWING TAXABLE YEAR.
//!
//! Minimum investment return per § 4942(e): 5% of the
//! AGGREGATE FAIR MARKET VALUE of all assets of the PF
//! other than assets used or held for use directly in
//! carrying out the exempt purpose, reduced by acquisition
//! indebtedness with respect to such assets.
//!
//! Qualifying distributions per § 4942(g)(1):
//! 1. § 4942(g)(1)(A) — any amount (including
//!    administrative expenses that are reasonable and
//!    necessary) PAID to accomplish one or more purposes
//!    described in § 170(c)(2)(B): RELIGIOUS, CHARITABLE,
//!    SCIENTIFIC, LITERARY, EDUCATIONAL, or other PUBLIC
//!    PURPOSES, or to foster amateur sports competition,
//!    or to prevent cruelty to children or animals
//! 2. § 4942(g)(1)(B) — any amount paid to acquire an
//!    asset used (or held for use) directly in carrying
//!    out one or more exempt purposes
//!
//! Set-asides per § 4942(g)(2): amounts set aside for a
//! specific project may be treated as qualifying
//! distributions if (a) at the time of the set-aside the
//! PF establishes that the amount will be paid for the
//! project within 60 months AND (b) either the suitability
//! test (project requires more than one year of accumulated
//! funds) OR the cash distribution test is satisfied.
//!
//! Carryover of excess qualifying distributions per
//! § 4942(i): if qualifying distributions in a year EXCEED
//! the distributable amount for that year, the excess may
//! be carried forward FIVE YEARS to offset future
//! distribution shortfalls (but excess cannot be used
//! against the year in which generated).
//!
//! Treatment of qualifying distributions per § 4942(h):
//! distributions are treated as made first out of
//! undistributed income of the immediately preceding year
//! and then current year, unless PF makes an election
//! under § 4942(h)(2) to treat as made out of corpus.
//!
//! Exceptions:
//! - § 4942(a)(2)(A): operating foundations described in
//!   § 4942(j)(3) are EXEMPT from § 4942
//! - § 4942(a)(2)(B): conduit foundations described in
//!   § 170(b)(1)(F)(ii) treat redistribution by donee as
//!   PF distribution
//! - § 4942(j)(5): certain PFs organized before May 27,
//!   1969 EXEMPT under § 4942(j)(5) conditions
//!
//! Trader-foundation critical because (1) the 5%
//! minimum distribution is the MOST DEMANDING annual
//! compliance obligation for private foundations — most
//! PFs target 5%-5.5% to build buffer; (2) Tier-2 100%
//! tax effectively confiscates the deficient amount and
//! is non-deductible; (3) family foundations often
//! struggle with the 5% requirement during market
//! downturns when minimum investment return rises while
//! liquid assets fall; (4) administrative expenses count
//! toward qualifying distributions if reasonable and
//! necessary — but cannot exceed § 4945 reasonable
//! standard; (5) set-asides under § 4942(g)(2) require
//! IRS advance approval in many situations; (6) carryover
//! of 5-year excess distributions is a critical planning
//! tool but expires.
//!
//! Distinction from § 4940 (iter 470): § 4940 is the
//! ANNUAL EXCISE TAX on NII; § 4942 is the ANNUAL
//! MINIMUM-DISTRIBUTION REQUIREMENT backed by 30% + 100%
//! penalty. Both apply concurrently; PFs pay § 4940 NII
//! tax AND must satisfy § 4942 distribution requirement.
//!
//! Authority: 26 U.S.C. § 4942; § 4942(a); § 4942(a)(2)(A);
//! § 4942(a)(2)(B); § 4942(b); § 4942(c); § 4942(d);
//! § 4942(e); § 4942(f); § 4942(g)(1)(A); § 4942(g)(1)(B);
//! § 4942(g)(2); § 4942(h); § 4942(h)(2); § 4942(i);
//! § 4942(j)(3); § 4942(j)(5); § 170(c)(2)(B);
//! § 170(b)(1)(F)(ii); 26 C.F.R. Part 53 Subpart C
//! (§ 53.4942(a)-1 through § 53.4942(b)-1); Tax Reform
//! Act of 1969, Pub. L. 91-172 (Dec. 30, 1969) —
//! original § 4942 enactment.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FoundationStatus {
    PrivateNonOperatingFoundation,
    OperatingFoundationSection4942j3,
    ConduitFoundationSection170b1Fii,
    GrandfatheredPreMay27_1969Section4942j5,
    PublicCharity,
    NonExempt,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub foundation_status: FoundationStatus,
    pub non_charitable_use_assets_fmv_cents: u64,
    pub acquisition_indebtedness_cents: u64,
    pub section_4940_excise_tax_cents: u64,
    pub ubi_tax_cents: u64,
    pub qualifying_distributions_cents: u64,
    pub prior_year_excess_carryover_cents: u64,
    pub corrected_within_90_day_notice_period: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    ExemptFoundation,
    DistributionSatisfied,
    Tier1TaxOwed,
    Tier2TaxOwed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub minimum_investment_return_cents: u64,
    pub distributable_amount_cents: u64,
    pub undistributed_income_cents: u64,
    pub tier1_tax_cents: u64,
    pub tier2_tax_cents: u64,
    pub total_tax_cents: u64,
    pub current_year_excess_carryover_cents: u64,
    pub notes: Vec<String>,
}

pub const MINIMUM_INVESTMENT_RETURN_PCT: u64 = 5;
pub const TIER1_RATE_PCT: u64 = 30;
pub const TIER2_RATE_PCT: u64 = 100;
pub const CARRYOVER_WINDOW_YEARS: u64 = 5;

pub type Section4942Input = Input;
pub type Section4942Result = Output;

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "§ 4942(a) imposes 30% TIER-1 excise tax on undistributed income for each year or partial year deficiency remains uncorrected; § 4942(b) imposes additional 100% TIER-2 excise tax if PF fails to make up deficient distribution within 90 days of IRS notice. Distributable amount per § 4942(d) = minimum investment return (5% of non-charitable-use FMV under § 4942(e)) reduced by § 4940 excise tax and UBI tax. Must be paid as qualifying distributions by end of IMMEDIATELY FOLLOWING TAXABLE YEAR.".to_string(),
        "Qualifying distributions per § 4942(g)(1): (A) any amount including reasonable and necessary administrative expenses paid to accomplish § 170(c)(2)(B) RELIGIOUS, CHARITABLE, SCIENTIFIC, LITERARY, EDUCATIONAL, or other PUBLIC PURPOSES (including foster amateur sports competition or prevent cruelty to children or animals); (B) amount paid to acquire asset used directly in carrying out exempt purpose. § 4942(g)(2) set-asides for specific project payable within 60 months may qualify if suitability test or cash distribution test satisfied.".to_string(),
        "Carryover per § 4942(i): excess qualifying distributions over distributable amount may carry forward FIVE YEARS to offset future shortfalls (excess cannot offset year generated). Treatment per § 4942(h): distributions deemed first out of prior-year undistributed income, then current year, unless § 4942(h)(2) corpus election made.".to_string(),
        "Exceptions: § 4942(a)(2)(A) operating foundations § 4942(j)(3) EXEMPT; § 4942(a)(2)(B) conduit foundations § 170(b)(1)(F)(ii) special rule; § 4942(j)(5) grandfathered PFs organized before May 27, 1969 EXEMPT under specific conditions.".to_string(),
        "Distinction from § 4940 (iter 470): § 4940 is ANNUAL EXCISE TAX on NII; § 4942 is ANNUAL MINIMUM-DISTRIBUTION REQUIREMENT backed by 30% + 100% penalty. Both apply concurrently; PFs pay § 4940 NII tax AND must satisfy § 4942 distribution. Distinction from § 4941 (iter 468): § 4942 is annual distribution failure; § 4941 is per-act self-dealing punitive.".to_string(),
        "Companion: section_4940 (iter 470), section_4941 (iter 468), section_4958 (iter 466), section_4960 (iter 464), section_4973 (iter 442), section_4974 (iter 436), section_4975 (iter 434), section_4980 (iter 460).".to_string(),
    ];

    match input.foundation_status {
        FoundationStatus::PublicCharity | FoundationStatus::NonExempt => {
            let mut n = notes;
            n.push("Organization is not a private foundation — § 4942 does not apply.".to_string());
            return Output {
                severity: Severity::NotApplicable,
                minimum_investment_return_cents: 0,
                distributable_amount_cents: 0,
                undistributed_income_cents: 0,
                tier1_tax_cents: 0,
                tier2_tax_cents: 0,
                total_tax_cents: 0,
                current_year_excess_carryover_cents: 0,
                notes: n,
            };
        }
        FoundationStatus::OperatingFoundationSection4942j3 => {
            let mut n = notes;
            n.push("Operating foundation under § 4942(j)(3) — EXEMPT from § 4942 minimum-distribution requirement per § 4942(a)(2)(A).".to_string());
            return Output {
                severity: Severity::ExemptFoundation,
                minimum_investment_return_cents: 0,
                distributable_amount_cents: 0,
                undistributed_income_cents: 0,
                tier1_tax_cents: 0,
                tier2_tax_cents: 0,
                total_tax_cents: 0,
                current_year_excess_carryover_cents: 0,
                notes: n,
            };
        }
        FoundationStatus::GrandfatheredPreMay27_1969Section4942j5 => {
            let mut n = notes;
            n.push("Grandfathered private foundation organized before May 27, 1969 — EXEMPT from § 4942 under § 4942(j)(5) conditions.".to_string());
            return Output {
                severity: Severity::ExemptFoundation,
                minimum_investment_return_cents: 0,
                distributable_amount_cents: 0,
                undistributed_income_cents: 0,
                tier1_tax_cents: 0,
                tier2_tax_cents: 0,
                total_tax_cents: 0,
                current_year_excess_carryover_cents: 0,
                notes: n,
            };
        }
        FoundationStatus::PrivateNonOperatingFoundation
        | FoundationStatus::ConduitFoundationSection170b1Fii => {}
    }

    let net_non_charitable_assets = input
        .non_charitable_use_assets_fmv_cents
        .saturating_sub(input.acquisition_indebtedness_cents);
    let minimum_investment_return = net_non_charitable_assets
        .saturating_mul(MINIMUM_INVESTMENT_RETURN_PCT)
        .checked_div(100)
        .unwrap_or(0);

    let distributable_amount = minimum_investment_return
        .saturating_sub(input.section_4940_excise_tax_cents)
        .saturating_sub(input.ubi_tax_cents);

    let total_qualifying_with_carryover = input
        .qualifying_distributions_cents
        .saturating_add(input.prior_year_excess_carryover_cents);

    let undistributed_income = distributable_amount.saturating_sub(total_qualifying_with_carryover);
    let current_year_excess = total_qualifying_with_carryover.saturating_sub(distributable_amount);

    let mut n = notes;

    if undistributed_income == 0 {
        n.push(format!(
            "Distribution requirement satisfied. Current-year excess carryover: ${}.{:02} (FIVE-year window per § 4942(i)).",
            current_year_excess / 100,
            current_year_excess % 100
        ));
        return Output {
            severity: Severity::DistributionSatisfied,
            minimum_investment_return_cents: minimum_investment_return,
            distributable_amount_cents: distributable_amount,
            undistributed_income_cents: 0,
            tier1_tax_cents: 0,
            tier2_tax_cents: 0,
            total_tax_cents: 0,
            current_year_excess_carryover_cents: current_year_excess,
            notes: n,
        };
    }

    let tier1_tax = undistributed_income
        .saturating_mul(TIER1_RATE_PCT)
        .checked_div(100)
        .unwrap_or(0);

    let tier2_tax = if input.corrected_within_90_day_notice_period {
        0
    } else {
        undistributed_income
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

    n.push(format!(
        "Undistributed income: ${}.{:02}; Tier-1 30% tax: ${}.{:02}; Tier-2 100% tax: ${}.{:02}; Total: ${}.{:02}.",
        undistributed_income / 100,
        undistributed_income % 100,
        tier1_tax / 100,
        tier1_tax % 100,
        tier2_tax / 100,
        tier2_tax % 100,
        total_tax / 100,
        total_tax % 100
    ));

    Output {
        severity,
        minimum_investment_return_cents: minimum_investment_return,
        distributable_amount_cents: distributable_amount,
        undistributed_income_cents: undistributed_income,
        tier1_tax_cents: tier1_tax,
        tier2_tax_cents: tier2_tax,
        total_tax_cents: total_tax,
        current_year_excess_carryover_cents: 0,
        notes: n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        // PF with $10M assets, $50K § 4940 tax, $0 UBI tax, $480K distributed, no carryover
        // MIR = 5% × $10M = $500K
        // Distributable = $500K - $50K - $0 = $450K
        // Distributions $480K vs distributable $450K → distribution satisfied, $30K excess
        Input {
            foundation_status: FoundationStatus::PrivateNonOperatingFoundation,
            non_charitable_use_assets_fmv_cents: 10_000_000_00,
            acquisition_indebtedness_cents: 0,
            section_4940_excise_tax_cents: 50_000_00,
            ubi_tax_cents: 0,
            qualifying_distributions_cents: 480_000_00,
            prior_year_excess_carryover_cents: 0,
            corrected_within_90_day_notice_period: false,
        }
    }

    #[test]
    fn public_charity_not_subject_to_4942() {
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
    fn operating_foundation_4942j3_exempt() {
        let mut i = baseline();
        i.foundation_status = FoundationStatus::OperatingFoundationSection4942j3;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptFoundation);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4942(j)(3)"));
        assert!(joined.contains("§ 4942(a)(2)(A)"));
    }

    #[test]
    fn grandfathered_pre_may_1969_exempt() {
        let mut i = baseline();
        i.foundation_status = FoundationStatus::GrandfatheredPreMay27_1969Section4942j5;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptFoundation);
        let joined = out.notes.join(" ");
        assert!(joined.contains("May 27, 1969"));
        assert!(joined.contains("§ 4942(j)(5)"));
    }

    #[test]
    fn baseline_distribution_satisfied() {
        let i = baseline();
        let out = check(&i);
        assert_eq!(out.severity, Severity::DistributionSatisfied);
        assert_eq!(out.minimum_investment_return_cents, 500_000_00);
        assert_eq!(out.distributable_amount_cents, 450_000_00);
        assert_eq!(out.undistributed_income_cents, 0);
        assert_eq!(out.current_year_excess_carryover_cents, 30_000_00);
    }

    #[test]
    fn shortfall_tier_1_30_pct_uncorrected_tier_2_100_pct() {
        let mut i = baseline();
        i.qualifying_distributions_cents = 400_000_00; // $50K short of $450K
        let out = check(&i);
        // Tier-1: 30% × $50K = $15K
        // Tier-2: 100% × $50K = $50K (uncorrected)
        assert_eq!(out.undistributed_income_cents, 50_000_00);
        assert_eq!(out.tier1_tax_cents, 15_000_00);
        assert_eq!(out.tier2_tax_cents, 50_000_00);
        assert_eq!(out.total_tax_cents, 65_000_00);
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn shortfall_corrected_within_90_day_notice_no_tier_2() {
        let mut i = baseline();
        i.qualifying_distributions_cents = 400_000_00;
        i.corrected_within_90_day_notice_period = true;
        let out = check(&i);
        // Tier-1: $15K, Tier-2: $0
        assert_eq!(out.tier1_tax_cents, 15_000_00);
        assert_eq!(out.tier2_tax_cents, 0);
        assert_eq!(out.severity, Severity::Tier1TaxOwed);
    }

    #[test]
    fn acquisition_indebtedness_reduces_mir() {
        let mut i = baseline();
        i.acquisition_indebtedness_cents = 2_000_000_00; // $2M debt
                                                         // Net = $8M × 5% = $400K MIR
                                                         // Distributable = $400K - $50K - $0 = $350K
                                                         // Distributions $480K >> $350K → satisfied, $130K excess
        let out = check(&i);
        assert_eq!(out.minimum_investment_return_cents, 400_000_00);
        assert_eq!(out.distributable_amount_cents, 350_000_00);
        assert_eq!(out.current_year_excess_carryover_cents, 130_000_00);
    }

    #[test]
    fn section_4940_excise_reduces_distributable_amount() {
        let mut i = baseline();
        i.section_4940_excise_tax_cents = 100_000_00;
        // MIR $500K - $100K § 4940 - $0 UBI = $400K distributable
        let out = check(&i);
        assert_eq!(out.distributable_amount_cents, 400_000_00);
    }

    #[test]
    fn ubi_tax_reduces_distributable_amount() {
        let mut i = baseline();
        i.ubi_tax_cents = 75_000_00;
        // MIR $500K - $50K - $75K = $375K
        let out = check(&i);
        assert_eq!(out.distributable_amount_cents, 375_000_00);
    }

    #[test]
    fn prior_year_carryover_satisfies_current_shortfall() {
        let mut i = baseline();
        i.qualifying_distributions_cents = 400_000_00; // $50K short
        i.prior_year_excess_carryover_cents = 100_000_00; // ample carryover
        let out = check(&i);
        // $400K + $100K carryover = $500K >= $450K distributable
        assert_eq!(out.severity, Severity::DistributionSatisfied);
        assert_eq!(out.undistributed_income_cents, 0);
    }

    #[test]
    fn prior_year_carryover_partial_offset_still_shortfall() {
        let mut i = baseline();
        i.qualifying_distributions_cents = 400_000_00;
        i.prior_year_excess_carryover_cents = 30_000_00;
        // $400K + $30K = $430K < $450K → $20K shortfall
        let out = check(&i);
        assert_eq!(out.undistributed_income_cents, 20_000_00);
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn five_percent_minimum_investment_return_per_4942e() {
        let mut i = baseline();
        i.non_charitable_use_assets_fmv_cents = 20_000_000_00; // $20M
        i.section_4940_excise_tax_cents = 0;
        i.ubi_tax_cents = 0;
        // MIR = 5% × $20M = $1M
        let out = check(&i);
        assert_eq!(out.minimum_investment_return_cents, 1_000_000_00);
        assert_eq!(out.distributable_amount_cents, 1_000_000_00);
    }

    #[test]
    fn zero_assets_no_distribution_required() {
        let mut i = baseline();
        i.non_charitable_use_assets_fmv_cents = 0;
        i.section_4940_excise_tax_cents = 0;
        i.qualifying_distributions_cents = 0;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DistributionSatisfied);
        assert_eq!(out.minimum_investment_return_cents, 0);
        assert_eq!(out.distributable_amount_cents, 0);
    }

    #[test]
    fn excess_distributions_create_carryover() {
        let mut i = baseline();
        i.qualifying_distributions_cents = 600_000_00; // $150K excess
        let out = check(&i);
        assert_eq!(out.severity, Severity::DistributionSatisfied);
        // $600K - $450K = $150K excess for 5-year carryover
        assert_eq!(out.current_year_excess_carryover_cents, 150_000_00);
    }

    #[test]
    fn realistic_10m_family_foundation() {
        // $10M assets × 5% = $500K MIR
        // § 4940 1.39% × NII = small reduction
        // $500K distributable with ~$15K § 4940 reduction = $485K
        let i = Input {
            foundation_status: FoundationStatus::PrivateNonOperatingFoundation,
            non_charitable_use_assets_fmv_cents: 10_000_000_00,
            acquisition_indebtedness_cents: 0,
            section_4940_excise_tax_cents: 15_000_00,
            ubi_tax_cents: 0,
            qualifying_distributions_cents: 525_000_00,
            prior_year_excess_carryover_cents: 0,
            corrected_within_90_day_notice_period: false,
        };
        let out = check(&i);
        assert_eq!(out.minimum_investment_return_cents, 500_000_00);
        assert_eq!(out.distributable_amount_cents, 485_000_00);
        assert_eq!(out.severity, Severity::DistributionSatisfied);
        assert_eq!(out.current_year_excess_carryover_cents, 40_000_00);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4942(a)"));
        assert!(joined.contains("§ 4942(b)"));
        assert!(joined.contains("§ 4942(d)"));
        assert!(joined.contains("§ 4942(e)"));
        assert!(joined.contains("§ 4942(g)(1)"));
        assert!(joined.contains("§ 4942(g)(2)"));
        assert!(joined.contains("§ 4942(h)"));
        assert!(joined.contains("§ 4942(h)(2)"));
        assert!(joined.contains("§ 4942(i)"));
        assert!(joined.contains("§ 4942(j)(3)"));
        assert!(joined.contains("§ 4942(j)(5)"));
        assert!(joined.contains("§ 170(c)(2)(B)"));
        assert!(joined.contains("§ 170(b)(1)(F)(ii)"));
        assert!(joined.contains("§ 4940 (iter 470)"));
    }

    #[test]
    fn note_pins_two_tier_30_100_structure() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("30%"));
        assert!(joined.contains("100%"));
        assert!(joined.contains("90 days"));
    }

    #[test]
    fn note_pins_5_percent_mir() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("5%"));
        assert!(joined.contains("non-charitable-use FMV"));
    }

    #[test]
    fn note_pins_qualifying_distribution_categories() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("RELIGIOUS"));
        assert!(joined.contains("CHARITABLE"));
        assert!(joined.contains("SCIENTIFIC"));
        assert!(joined.contains("LITERARY"));
        assert!(joined.contains("EDUCATIONAL"));
        assert!(joined.contains("amateur sports"));
        assert!(joined.contains("cruelty to children or animals"));
    }

    #[test]
    fn note_pins_60_month_setaside() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("60 months"));
        assert!(joined.contains("suitability test"));
        assert!(joined.contains("cash distribution test"));
    }

    #[test]
    fn note_pins_5_year_carryover() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("FIVE YEARS"));
        assert!(joined.contains("§ 4942(i)"));
    }

    #[test]
    fn note_pins_three_exceptions() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4942(a)(2)(A)"));
        assert!(joined.contains("operating foundations"));
        assert!(joined.contains("§ 4942(a)(2)(B)"));
        assert!(joined.contains("conduit"));
        assert!(joined.contains("§ 4942(j)(5)"));
    }

    #[test]
    fn note_pins_4940_4941_distinctions() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4940 (iter 470)"));
        assert!(joined.contains("§ 4941 (iter 468)"));
        assert!(joined.contains("ANNUAL"));
        assert!(joined.contains("per-act"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("section_4940"));
        assert!(joined.contains("section_4941"));
        assert!(joined.contains("section_4958"));
        assert!(joined.contains("section_4960"));
    }

    #[test]
    fn truth_table_five_status_cells() {
        // Public charity = NotApplicable
        let c1 = check(&Input {
            foundation_status: FoundationStatus::PublicCharity,
            ..baseline()
        });
        assert_eq!(c1.severity, Severity::NotApplicable);

        // Non-exempt = NotApplicable
        let c2 = check(&Input {
            foundation_status: FoundationStatus::NonExempt,
            ..baseline()
        });
        assert_eq!(c2.severity, Severity::NotApplicable);

        // Operating foundation = ExemptFoundation
        let c3 = check(&Input {
            foundation_status: FoundationStatus::OperatingFoundationSection4942j3,
            ..baseline()
        });
        assert_eq!(c3.severity, Severity::ExemptFoundation);

        // Grandfathered = ExemptFoundation
        let c4 = check(&Input {
            foundation_status: FoundationStatus::GrandfatheredPreMay27_1969Section4942j5,
            ..baseline()
        });
        assert_eq!(c4.severity, Severity::ExemptFoundation);

        // Private non-operating baseline = DistributionSatisfied
        let c5 = check(&baseline());
        assert_eq!(c5.severity, Severity::DistributionSatisfied);
    }

    #[test]
    fn saturating_arithmetic_overflow_defense() {
        let i = Input {
            foundation_status: FoundationStatus::PrivateNonOperatingFoundation,
            non_charitable_use_assets_fmv_cents: u64::MAX,
            acquisition_indebtedness_cents: 0,
            section_4940_excise_tax_cents: 0,
            ubi_tax_cents: 0,
            qualifying_distributions_cents: 0,
            prior_year_excess_carryover_cents: 0,
            corrected_within_90_day_notice_period: false,
        };
        let out = check(&i);
        // No panic
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn boundary_distribution_exactly_equals_requirement() {
        let mut i = baseline();
        i.qualifying_distributions_cents = 450_000_00; // exactly equal to distributable
        let out = check(&i);
        assert_eq!(out.severity, Severity::DistributionSatisfied);
        assert_eq!(out.undistributed_income_cents, 0);
        assert_eq!(out.current_year_excess_carryover_cents, 0);
    }

    #[test]
    fn boundary_one_cent_short() {
        let mut i = baseline();
        i.qualifying_distributions_cents = 449_999_99; // 1 cent short
        let out = check(&i);
        assert_eq!(out.undistributed_income_cents, 1);
        // 30% of 1 cent = 0
        assert_eq!(out.tier1_tax_cents, 0);
        // 100% of 1 cent = 1
        assert_eq!(out.tier2_tax_cents, 1);
    }

    #[test]
    fn excise_tax_exceeds_mir_distributable_clamps_to_zero() {
        let mut i = baseline();
        i.section_4940_excise_tax_cents = 1_000_000_00; // huge
        let out = check(&i);
        assert_eq!(out.distributable_amount_cents, 0);
        assert_eq!(out.severity, Severity::DistributionSatisfied);
    }

    #[test]
    fn debt_exceeding_assets_zero_mir() {
        let mut i = baseline();
        i.acquisition_indebtedness_cents = 20_000_000_00; // exceeds $10M assets
        let out = check(&i);
        assert_eq!(out.minimum_investment_return_cents, 0);
        assert_eq!(out.distributable_amount_cents, 0);
    }

    #[test]
    fn note_pins_distributable_amount_calculation() {
        let mut i = baseline();
        i.qualifying_distributions_cents = 400_000_00;
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("Undistributed income"));
        assert!(joined.contains("Tier-1 30% tax"));
        assert!(joined.contains("Tier-2 100% tax"));
    }
}
