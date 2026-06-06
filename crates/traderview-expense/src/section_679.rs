//! IRC § 679 — Foreign Trusts Having One or More United States
//! Beneficiaries / Grantor Trust Anti-Deferral Module.
//!
//! Pure-compute check for grantor trust status under Internal
//! Revenue Code § 679 ("Foreign trusts having one or more
//! United States beneficiaries"). § 679 is the LAST substantive
//! grantor-trust trigger in the §§ 671-679 statutory progression
//! after § 673 reversionary (built iter 648), § 674 beneficial
//! enjoyment, § 675 administrative powers (iter 644), § 676
//! power to revoke (iter 646), § 677 income for benefit of
//! grantor (iter 642), § 678 person other than grantor (iter
//! 640). § 679 is the anti-deferral rule applicable to **US
//! persons who directly or indirectly transfer property to a
//! foreign trust** with one or more US beneficiaries — converts
//! the foreign trust to grantor trust status with respect to
//! the transferor.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 679(a)(1) General Rule**: a US person who directly
//!   or indirectly transfers property to a foreign trust (other
//!   than a trust described in § 6048(a)(3)(B)(ii)) shall be
//!   treated as the **OWNER** for his taxable year of the
//!   portion of such trust attributable to such property if for
//!   such year there is a **US BENEFICIARY** of any portion of
//!   such trust ([Cornell LII 26 USC § 679](https://www.law.cornell.edu/uscode/text/26/679);
//!   [IRS — Foreign Trust Reporting Requirements and Tax
//!   Consequences](https://www.irs.gov/businesses/international-businesses/foreign-trust-reporting-requirements-and-tax-consequences)).
//! - **IRC § 679(a)(2) US Beneficiary Presumption**: the
//!   Secretary may treat any foreign trust as having a US
//!   beneficiary unless the transferor rebuts the presumption
//!   by submitting information demonstrating that no part of
//!   the income or corpus may benefit a US person.
//! - **IRC § 679(a)(3) Transfer-at-Death Exception**: transfers
//!   by reason of the death of the transferor are EXCEPTED from
//!   § 679 application. Upon death of US settlor of foreign
//!   grantor trust, grantor status terminates.
//! - **IRC § 679(a)(4) Outbound Trust Migration**: if an
//!   individual who is a citizen or resident of the United
//!   States transfers property to a trust which was NOT a
//!   foreign trust, and such trust **becomes a foreign trust
//!   while such individual is alive**, this section and § 6048
//!   shall be applied as if such individual transferred to such
//!   trust on the date such trust becomes a foreign trust an
//!   amount equal to the portion attributable to the property
//!   previously transferred ([IRS Streamlined Procedures —
//!   Outbound Migration of Domestic Trusts](https://www.irsstreamlinedprocedures.com/outbound-migration-of-domestic-trusts-tax-and-reporting/)).
//! - **IRC § 679(a)(5) 5-Year Pre-Immigration Transfer
//!   Lookback**: if a nonresident alien individual has a
//!   **residency starting date within 5 YEARS** after directly
//!   or indirectly transferring property to a foreign trust,
//!   this section and § 6048 shall be applied as if such
//!   individual transferred to such trust on the residency
//!   starting date an amount equal to the portion of such trust
//!   attributable to the property transferred by such
//!   individual to such trust in such transfer ([Florida Bar —
//!   Pre-Immigration Planning With the Foreign Trust](https://www.floridabar.org/the-florida-bar-journal/pre-immigration-planning-with-the-foreign-trust-the-intersection-of-income-and-estate-tax/)).
//! - **IRC § 679(c) 5-Year Beneficiary Lookback**: a
//!   beneficiary shall NOT be treated as a US person in applying
//!   this section with respect to any transfer of property to
//!   foreign trust if such beneficiary first became a US person
//!   **more than 5 YEARS after the date of such transfer**.
//! - **Form 3520-A Reporting** (Annual Information Return of
//!   Foreign Trust With a United States Owner): foreign trust
//!   with US owner must file Form 3520-A by **15th day of 3rd
//!   month after end of trust's taxable year**; US owner must
//!   receive Foreign Grantor Trust Owner Statement (pages 3-4
//!   of Form 3520-A) for income reporting on grantor's Form 1040.
//! - **Form 3520 Reporting**: US transferor / owner must file
//!   Form 3520 (Annual Return To Report Transactions With
//!   Foreign Trusts and Receipt of Certain Foreign Gifts).
//! - **IRC § 6048 Reporting Penalty**: greater of **$10,000 OR
//!   35 % of gross reportable amount** for failure to file Form
//!   3520 or 3520-A.
//! - **Small Business Job Protection Act of 1996** (Public Law
//!   104-188): significant amendments to § 679 + § 6048 to
//!   tighten foreign trust anti-deferral rules.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_679_PRE_IMMIGRATION_LOOKBACK_YEARS: u32 = 5;
pub const IRC_679_C_BENEFICIARY_LOOKBACK_YEARS: u32 = 5;
pub const FORM_3520_A_DUE_MONTH_AFTER_TAX_YEAR_END: u32 = 3;
pub const FORM_3520_A_DUE_DAY_OF_THIRD_MONTH: u32 = 15;
pub const IRC_6048_REPORTING_PENALTY_FLOOR_DOLLARS: u64 = 10_000;
pub const IRC_6048_REPORTING_PENALTY_PCT_BASIS_POINTS: u64 = 3_500;
pub const IRC_6048_PENALTY_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const SMALL_BUSINESS_JOB_PROTECTION_ACT_1996_YEAR: u32 = 1996;
pub const SMALL_BUSINESS_JOB_PROTECTION_ACT_PL_NUMBER: u32 = 104_188;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferorStatus {
    UsCitizenOrResidentAtTimeOfTransfer,
    NonresidentAlienWithResidencyStartingDateWithin5YearsAfterTransfer,
    NonresidentAlienWithResidencyStartingDateOver5YearsAfterTransfer,
    NotAUsPersonAtAnyRelevantTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustClassification {
    AlreadyForeignTrustAtTimeOfTransfer,
    DomesticTrustWhichLaterBecameForeignDuringTransferorLife,
    DomesticTrustRemainsDomestic,
    ForeignTrustWithNoBeneficiaryDistribution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UsBeneficiaryStatus {
    UsBeneficiaryPresent,
    UsBeneficiaryPresumedUnlessRebutted,
    PresumptionRebuttedByDocumentation,
    BeneficiaryBecameUsPersonWithin5YearsOfTransfer,
    BeneficiaryBecameUsPersonOver5YearsAfterTransfer,
    NoUsBeneficiaryAndNoPresumption,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferCircumstance {
    InterVivosTransferDuringTransferorLife,
    TransferByReasonOfDeathOfTransferor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section679Mode {
    NotApplicableNoTrustOrNotAForeignTrust,
    NotApplicableTransferorNotAUsPerson,
    NotApplicableTransferByReasonOfDeathExceptionUnderSection679A3,
    NotApplicableUsBeneficiaryPresumptionRebutted,
    NotApplicableBeneficiaryBecameUsPersonOver5YearsAfterTransferUnderSection679C,
    CompliantSection679A1ForeignTrustWithUsBeneficiaryActiveGrantorTrust,
    CompliantSection679A2PresumedUsBeneficiaryActiveGrantorTrust,
    CompliantSection679A4OutboundMigrationDeemedTransferActiveGrantorTrust,
    CompliantSection679A5PreImmigrationLookbackDeemedTransferActiveGrantorTrust,
    CompliantForm3520AndForm3520aFiledTimely,
    ViolationSection679ActiveButGrantorTrustIncomeNotReportedOnForm1040,
    ViolationForm3520ReportingFailurePenaltyAccrues,
    ViolationForm3520aReportingFailurePenaltyAccrues,
    ViolationBothForm3520AndForm3520aReportingFailures,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub trust_classification: TrustClassification,
    pub transferor_status: TransferorStatus,
    pub us_beneficiary_status: UsBeneficiaryStatus,
    pub transfer_circumstance: TransferCircumstance,
    pub form_3520_filed_timely: bool,
    pub form_3520_a_filed_timely: bool,
    pub grantor_trust_income_reported_on_form_1040: bool,
    pub gross_reportable_amount_dollars: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section679Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub section_6048_penalty_dollars: u64,
}

pub type Section679Input = Input;
pub type Section679Output = Output;
pub type Section679Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 679(a)(1) — US person who directly or indirectly transfers property to foreign trust treated as OWNER of portion of trust attributable to property if any US beneficiary exists in that taxable year".to_string(),
        "IRC § 679(a)(2) — US BENEFICIARY PRESUMPTION; Secretary may treat any foreign trust as having US beneficiary unless transferor rebuts by demonstrating no part of income or corpus may benefit US person".to_string(),
        "IRC § 679(a)(3) — TRANSFER-AT-DEATH EXCEPTION; transfers by reason of death of transferor excepted from § 679 application; foreign grantor trust status terminates at US settlor death".to_string(),
        "IRC § 679(a)(4) — OUTBOUND TRUST MIGRATION; domestic trust that becomes foreign during transferor's life treated as if transferor transferred to foreign trust on migration date".to_string(),
        "IRC § 679(a)(5) — 5-YEAR PRE-IMMIGRATION LOOKBACK; nonresident alien with residency starting date within 5 years after transfer deemed to have transferred property on residency starting date".to_string(),
        "IRC § 679(c) — 5-YEAR BENEFICIARY LOOKBACK; beneficiary not treated as US person if first became US person more than 5 years after date of transfer".to_string(),
        "IRC § 6048 — Form 3520-A foreign trust annual information return + Form 3520 transactions/receipts return; penalty greater of $10,000 OR 35 % of gross reportable amount for failure to file".to_string(),
        "Form 3520-A (Annual Information Return of Foreign Trust With a United States Owner) — due 15th day of 3rd month after end of trust's taxable year; Foreign Grantor Trust Owner Statement (pages 3-4) flows to grantor's Form 1040 via § 671 attribution".to_string(),
        "Form 3520 (Annual Return To Report Transactions With Foreign Trusts and Receipt of Certain Foreign Gifts) — required of US transferor / owner".to_string(),
        "Small Business Job Protection Act of 1996 (Public Law 104-188) — significant amendments tightening § 679 + § 6048 anti-deferral rules; enacted in response to expanding foreign trust abuse".to_string(),
        "IRC § 671 — Subpart E general attribution; § 679-triggered grantor trust status flows income, deductions, credits to US grantor's Form 1040; foreign trust files Form 3520-A".to_string(),
        "Treasury Proposed Regulations (May 8, 2024; 89 FR 39440) — Transactions With Foreign Trusts and Information Reporting on Transactions With Foreign Trusts and Large Foreign Gifts; modernizes § 679 + § 6048 implementing rules".to_string(),
        "Cornell LII 26 USC § 679 — primary statutory text".to_string(),
        "IRS — Foreign Trust Reporting Requirements and Tax Consequences (practitioner guide)".to_string(),
        "IRS — Foreign Grantor Trust Determination Part I § 679 (FATCA International Practice Unit)".to_string(),
        "Florida Bar — Pre-Immigration Planning with the Foreign Trust: The Intersection of Income and Estate Tax".to_string(),
        "Meadows Collier — Foreign Grantor Trusts & U.S. Taxes: Understanding Section 679".to_string(),
    ];

    if matches!(
        input.trust_classification,
        TrustClassification::DomesticTrustRemainsDomestic
    ) {
        return Output {
            mode: Section679Mode::NotApplicableNoTrustOrNotAForeignTrust,
            statutory_basis: "IRC § 679 — applies only to foreign trusts; domestic trust outside scope".to_string(),
            notes: "NOT APPLICABLE: trust remains a domestic trust; § 679 anti-deferral rule does not apply to domestic trusts.".to_string(),
            citations,
            section_6048_penalty_dollars: 0,
        };
    }

    if input.transfer_circumstance == TransferCircumstance::TransferByReasonOfDeathOfTransferor {
        return Output {
            mode: Section679Mode::NotApplicableTransferByReasonOfDeathExceptionUnderSection679A3,
            statutory_basis: "IRC § 679(a)(3) — transfer-at-death exception".to_string(),
            notes: "NOT APPLICABLE: transfer by reason of death of transferor; § 679(a)(3) statutory exception applies; foreign grantor trust status terminates at US settlor death.".to_string(),
            citations,
            section_6048_penalty_dollars: 0,
        };
    }

    if input.transferor_status == TransferorStatus::NotAUsPersonAtAnyRelevantTime {
        return Output {
            mode: Section679Mode::NotApplicableTransferorNotAUsPerson,
            statutory_basis: "IRC § 679(a)(1) — applies only to transfers by US persons".to_string(),
            notes: "NOT APPLICABLE: transferor is not a US person at any relevant time; § 679 applies only to US-person transferors.".to_string(),
            citations,
            section_6048_penalty_dollars: 0,
        };
    }

    if input.transferor_status
        == TransferorStatus::NonresidentAlienWithResidencyStartingDateOver5YearsAfterTransfer
    {
        return Output {
            mode: Section679Mode::NotApplicableTransferorNotAUsPerson,
            statutory_basis: "IRC § 679(a)(5) — 5-year pre-immigration lookback window not satisfied".to_string(),
            notes: "NOT APPLICABLE: nonresident alien transferor's residency starting date more than 5 years after transfer; § 679(a)(5) pre-immigration lookback inapplicable; transferor not treated as US person for § 679 purposes.".to_string(),
            citations,
            section_6048_penalty_dollars: 0,
        };
    }

    if input.us_beneficiary_status == UsBeneficiaryStatus::PresumptionRebuttedByDocumentation {
        return Output {
            mode: Section679Mode::NotApplicableUsBeneficiaryPresumptionRebutted,
            statutory_basis: "IRC § 679(a)(2) — US beneficiary presumption rebutted by documentation".to_string(),
            notes: "NOT APPLICABLE: transferor submitted documentation demonstrating no part of trust income or corpus may benefit US person; § 679(a)(2) presumption rebutted; § 679 not triggered.".to_string(),
            citations,
            section_6048_penalty_dollars: 0,
        };
    }

    if input.us_beneficiary_status
        == UsBeneficiaryStatus::BeneficiaryBecameUsPersonOver5YearsAfterTransfer
    {
        return Output {
            mode: Section679Mode::NotApplicableBeneficiaryBecameUsPersonOver5YearsAfterTransferUnderSection679C,
            statutory_basis: "IRC § 679(c) — 5-year beneficiary lookback".to_string(),
            notes: "NOT APPLICABLE: beneficiary first became US person more than 5 years after transfer; § 679(c) statutory lookback bars treatment as US beneficiary.".to_string(),
            citations,
            section_6048_penalty_dollars: 0,
        };
    }

    if input.us_beneficiary_status == UsBeneficiaryStatus::NoUsBeneficiaryAndNoPresumption {
        return Output {
            mode: Section679Mode::NotApplicableUsBeneficiaryPresumptionRebutted,
            statutory_basis: "IRC § 679(a)(1) — requires US beneficiary".to_string(),
            notes: "NOT APPLICABLE: no US beneficiary and no § 679(a)(2) presumption asserted; § 679 not triggered.".to_string(),
            citations,
            section_6048_penalty_dollars: 0,
        };
    }

    let pct_penalty_dollars = input
        .gross_reportable_amount_dollars
        .saturating_mul(IRC_6048_REPORTING_PENALTY_PCT_BASIS_POINTS)
        / IRC_6048_PENALTY_BASIS_POINT_DENOMINATOR;
    let section_6048_penalty_dollars =
        IRC_6048_REPORTING_PENALTY_FLOOR_DOLLARS.max(pct_penalty_dollars);

    if !input.form_3520_filed_timely && !input.form_3520_a_filed_timely {
        return Output {
            mode: Section679Mode::ViolationBothForm3520AndForm3520aReportingFailures,
            statutory_basis: "IRC § 6048 + § 679 — Form 3520 + Form 3520-A both not filed".to_string(),
            notes: format!(
                "VIOLATION: both Form 3520 and Form 3520-A reporting failures; § 6048 penalty = greater of $10,000 OR 35 % of gross reportable amount = ${} (gross reportable ${}).",
                section_6048_penalty_dollars, input.gross_reportable_amount_dollars
            ),
            citations,
            section_6048_penalty_dollars,
        };
    }

    if !input.form_3520_filed_timely {
        return Output {
            mode: Section679Mode::ViolationForm3520ReportingFailurePenaltyAccrues,
            statutory_basis: "IRC § 6048 — Form 3520 not filed".to_string(),
            notes: format!(
                "VIOLATION: Form 3520 (Annual Return To Report Transactions With Foreign Trusts) not filed timely; § 6048 penalty = greater of $10,000 OR 35 % of gross reportable amount = ${}.",
                section_6048_penalty_dollars
            ),
            citations,
            section_6048_penalty_dollars,
        };
    }

    if !input.form_3520_a_filed_timely {
        return Output {
            mode: Section679Mode::ViolationForm3520aReportingFailurePenaltyAccrues,
            statutory_basis: "IRC § 6048 — Form 3520-A not filed".to_string(),
            notes: format!(
                "VIOLATION: Form 3520-A (Annual Information Return of Foreign Trust With a US Owner) not filed timely; § 6048 penalty = greater of $10,000 OR 35 % of gross reportable amount = ${}.",
                section_6048_penalty_dollars
            ),
            citations,
            section_6048_penalty_dollars,
        };
    }

    if !input.grantor_trust_income_reported_on_form_1040 {
        return Output {
            mode: Section679Mode::ViolationSection679ActiveButGrantorTrustIncomeNotReportedOnForm1040,
            statutory_basis: "IRC § 679 + § 671 — § 679-triggered grantor trust requires Form 1040 reporting".to_string(),
            notes: "VIOLATION: § 679 grantor trust status active but grantor trust income not reported on Form 1040 via § 671 attribution.".to_string(),
            citations,
            section_6048_penalty_dollars: 0,
        };
    }

    let mode = match input.trust_classification {
        TrustClassification::DomesticTrustWhichLaterBecameForeignDuringTransferorLife => {
            Section679Mode::CompliantSection679A4OutboundMigrationDeemedTransferActiveGrantorTrust
        }
        _ => match input.transferor_status {
            TransferorStatus::NonresidentAlienWithResidencyStartingDateWithin5YearsAfterTransfer => {
                Section679Mode::CompliantSection679A5PreImmigrationLookbackDeemedTransferActiveGrantorTrust
            }
            _ => match input.us_beneficiary_status {
                UsBeneficiaryStatus::UsBeneficiaryPresumedUnlessRebutted => {
                    Section679Mode::CompliantSection679A2PresumedUsBeneficiaryActiveGrantorTrust
                }
                _ => Section679Mode::CompliantSection679A1ForeignTrustWithUsBeneficiaryActiveGrantorTrust,
            },
        },
    };

    Output {
        mode,
        statutory_basis: "IRC § 679 — foreign trust with US beneficiary; grantor trust status active; Forms 3520 + 3520-A timely filed; Form 1040 reporting compliant".to_string(),
        notes: "COMPLIANT: § 679 grantor trust status active; foreign trust transferred property by US person with US beneficiary; both Form 3520 and Form 3520-A filed timely; income reported on Form 1040 via § 671 attribution.".to_string(),
        citations,
        section_6048_penalty_dollars: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_section_679_compliant() -> Input {
        Input {
            trust_classification: TrustClassification::AlreadyForeignTrustAtTimeOfTransfer,
            transferor_status: TransferorStatus::UsCitizenOrResidentAtTimeOfTransfer,
            us_beneficiary_status: UsBeneficiaryStatus::UsBeneficiaryPresent,
            transfer_circumstance: TransferCircumstance::InterVivosTransferDuringTransferorLife,
            form_3520_filed_timely: true,
            form_3520_a_filed_timely: true,
            grantor_trust_income_reported_on_form_1040: true,
            gross_reportable_amount_dollars: 100_000,
        }
    }

    #[test]
    fn domestic_trust_not_applicable() {
        let input = Input {
            trust_classification: TrustClassification::DomesticTrustRemainsDomestic,
            ..baseline_section_679_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section679Mode::NotApplicableNoTrustOrNotAForeignTrust
        );
    }

    #[test]
    fn transferor_not_us_person_not_applicable() {
        let input = Input {
            transferor_status: TransferorStatus::NotAUsPersonAtAnyRelevantTime,
            ..baseline_section_679_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section679Mode::NotApplicableTransferorNotAUsPerson
        );
    }

    #[test]
    fn transfer_by_reason_of_death_excepted() {
        let input = Input {
            transfer_circumstance: TransferCircumstance::TransferByReasonOfDeathOfTransferor,
            ..baseline_section_679_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section679Mode::NotApplicableTransferByReasonOfDeathExceptionUnderSection679A3
        );
    }

    #[test]
    fn us_beneficiary_presumption_rebutted_not_applicable() {
        let input = Input {
            us_beneficiary_status: UsBeneficiaryStatus::PresumptionRebuttedByDocumentation,
            ..baseline_section_679_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section679Mode::NotApplicableUsBeneficiaryPresumptionRebutted
        );
    }

    #[test]
    fn beneficiary_became_us_person_over_5_years_after_transfer_not_applicable() {
        let input = Input {
            us_beneficiary_status:
                UsBeneficiaryStatus::BeneficiaryBecameUsPersonOver5YearsAfterTransfer,
            ..baseline_section_679_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section679Mode::NotApplicableBeneficiaryBecameUsPersonOver5YearsAfterTransferUnderSection679C
        );
    }

    #[test]
    fn nonresident_alien_over_5_years_lookback_not_applicable() {
        let input = Input {
            transferor_status:
                TransferorStatus::NonresidentAlienWithResidencyStartingDateOver5YearsAfterTransfer,
            ..baseline_section_679_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section679Mode::NotApplicableTransferorNotAUsPerson
        );
    }

    #[test]
    fn section_679_a1_compliant_baseline() {
        let result = check(&baseline_section_679_compliant());
        assert_eq!(
            result.mode,
            Section679Mode::CompliantSection679A1ForeignTrustWithUsBeneficiaryActiveGrantorTrust
        );
    }

    #[test]
    fn section_679_a2_presumption_compliant() {
        let input = Input {
            us_beneficiary_status: UsBeneficiaryStatus::UsBeneficiaryPresumedUnlessRebutted,
            ..baseline_section_679_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section679Mode::CompliantSection679A2PresumedUsBeneficiaryActiveGrantorTrust
        );
    }

    #[test]
    fn section_679_a4_outbound_migration_compliant() {
        let input = Input {
            trust_classification:
                TrustClassification::DomesticTrustWhichLaterBecameForeignDuringTransferorLife,
            ..baseline_section_679_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section679Mode::CompliantSection679A4OutboundMigrationDeemedTransferActiveGrantorTrust
        );
    }

    #[test]
    fn section_679_a5_pre_immigration_5_year_lookback_compliant() {
        let input = Input {
            transferor_status:
                TransferorStatus::NonresidentAlienWithResidencyStartingDateWithin5YearsAfterTransfer,
            ..baseline_section_679_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section679Mode::CompliantSection679A5PreImmigrationLookbackDeemedTransferActiveGrantorTrust
        );
    }

    #[test]
    fn beneficiary_became_us_person_within_5_years_section_679_active() {
        let input = Input {
            us_beneficiary_status:
                UsBeneficiaryStatus::BeneficiaryBecameUsPersonWithin5YearsOfTransfer,
            ..baseline_section_679_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section679Mode::CompliantSection679A1ForeignTrustWithUsBeneficiaryActiveGrantorTrust
        );
    }

    #[test]
    fn form_3520_failure_violation() {
        let input = Input {
            form_3520_filed_timely: false,
            gross_reportable_amount_dollars: 50_000,
            ..baseline_section_679_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section679Mode::ViolationForm3520ReportingFailurePenaltyAccrues
        );
        assert_eq!(result.section_6048_penalty_dollars, 17_500);
    }

    #[test]
    fn form_3520_failure_below_floor_uses_10000_penalty() {
        let input = Input {
            form_3520_filed_timely: false,
            gross_reportable_amount_dollars: 20_000,
            ..baseline_section_679_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section679Mode::ViolationForm3520ReportingFailurePenaltyAccrues
        );
        assert_eq!(result.section_6048_penalty_dollars, 10_000);
    }

    #[test]
    fn form_3520a_failure_violation() {
        let input = Input {
            form_3520_a_filed_timely: false,
            gross_reportable_amount_dollars: 200_000,
            ..baseline_section_679_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section679Mode::ViolationForm3520aReportingFailurePenaltyAccrues
        );
        assert_eq!(result.section_6048_penalty_dollars, 70_000);
    }

    #[test]
    fn both_form_3520_and_3520a_failures_violation() {
        let input = Input {
            form_3520_filed_timely: false,
            form_3520_a_filed_timely: false,
            gross_reportable_amount_dollars: 1_000_000,
            ..baseline_section_679_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section679Mode::ViolationBothForm3520AndForm3520aReportingFailures
        );
        assert_eq!(result.section_6048_penalty_dollars, 350_000);
    }

    #[test]
    fn section_679_active_form_1040_omitted_violation() {
        let input = Input {
            grantor_trust_income_reported_on_form_1040: false,
            ..baseline_section_679_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section679Mode::ViolationSection679ActiveButGrantorTrustIncomeNotReportedOnForm1040
        );
    }

    #[test]
    fn citations_pin_section_679_subsections_and_forms() {
        let result = check(&baseline_section_679_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("IRC § 679(a)(1)"));
        assert!(joined.contains("IRC § 679(a)(2)"));
        assert!(joined.contains("IRC § 679(a)(3)"));
        assert!(joined.contains("IRC § 679(a)(4)"));
        assert!(joined.contains("IRC § 679(a)(5)"));
        assert!(joined.contains("IRC § 679(c)"));
        assert!(joined.contains("US BENEFICIARY PRESUMPTION"));
        assert!(joined.contains("TRANSFER-AT-DEATH"));
        assert!(joined.contains("OUTBOUND TRUST MIGRATION"));
        assert!(joined.contains("5-YEAR PRE-IMMIGRATION LOOKBACK"));
        assert!(joined.contains("5-YEAR BENEFICIARY LOOKBACK"));
        assert!(joined.contains("Form 3520-A"));
        assert!(joined.contains("Form 3520"));
        assert!(joined.contains("IRC § 6048"));
        assert!(joined.contains("$10,000"));
        assert!(joined.contains("35 %"));
        assert!(joined.contains("Small Business Job Protection Act of 1996"));
        assert!(joined.contains("Public Law 104-188"));
        assert!(joined.contains("IRC § 671"));
        assert!(joined.contains("Florida Bar"));
        assert!(joined.contains("Meadows Collier"));
    }

    #[test]
    fn constant_pin_lookback_years_and_penalty() {
        assert_eq!(IRC_679_PRE_IMMIGRATION_LOOKBACK_YEARS, 5);
        assert_eq!(IRC_679_C_BENEFICIARY_LOOKBACK_YEARS, 5);
        assert_eq!(FORM_3520_A_DUE_MONTH_AFTER_TAX_YEAR_END, 3);
        assert_eq!(FORM_3520_A_DUE_DAY_OF_THIRD_MONTH, 15);
        assert_eq!(IRC_6048_REPORTING_PENALTY_FLOOR_DOLLARS, 10_000);
        assert_eq!(IRC_6048_REPORTING_PENALTY_PCT_BASIS_POINTS, 3_500);
        assert_eq!(IRC_6048_PENALTY_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(SMALL_BUSINESS_JOB_PROTECTION_ACT_1996_YEAR, 1996);
        assert_eq!(SMALL_BUSINESS_JOB_PROTECTION_ACT_PL_NUMBER, 104_188);
    }

    #[test]
    fn saturating_overflow_defense_extreme_reportable_amount() {
        let input = Input {
            form_3520_filed_timely: false,
            gross_reportable_amount_dollars: u64::MAX,
            ..baseline_section_679_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section679Mode::ViolationForm3520ReportingFailurePenaltyAccrues
        );
        assert!(result.section_6048_penalty_dollars >= 10_000);
    }
}
