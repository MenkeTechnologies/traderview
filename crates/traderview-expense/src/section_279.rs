//! IRC § 279 interest disallowance on corporate acquisition indebtedness.
//!
//! § 279 disallows the corporate interest deduction for amounts exceeding $5,000,000 per
//! tax year on "corporate acquisition indebtedness" — debt incurred to acquire stock or
//! assets of another corporation that satisfies all four statutory criteria. The provision
//! targets debt-financed leveraged-buyout (LBO) transactions and "junk-bond" subordinated
//! convertible takeover financings of the 1970s-1980s era. § 279 remains live but its
//! practical importance is reduced by post-1980s changes in capital-structure norms and
//! parallel limitations under § 163(j) (interest limit), § 385 (debt-equity classification),
//! and § 7874 (anti-inversion).
//!
//! § 279(a) DISALLOWANCE: interest in excess of $5,000,000 annual cap (reduced by
//! interest on pre-Oct-9-1969 acquisition obligations) is not deductible by the issuer
//! corporation. Disallowance is PERMANENT (not capitalized; no carryforward).
//!
//! § 279(b) DEFINITION — all four prongs must be satisfied for an obligation to be
//! "corporate acquisition indebtedness":
//!   (1) Issued after October 9, 1969 to provide consideration for acquisition of stock
//!       or assets of another corporation.
//!   (2) SUBORDINATED to claims of trade creditors generally OR expressly subordinated to
//!       a substantial amount of unsecured indebtedness.
//!   (3) CONVERTIBLE into stock of the issuer OR issued as part of an investment unit with
//!       stock-purchase warrants or options.
//!   (4) Issuer's debt:equity ratio EXCEEDS 2:1 OR issuer's projected EBITDA fails 3:1
//!       interest-coverage test for three-taxable-year averaging period.
//!
//! § 279(d) PRE-1969 EXEMPTION: obligations issued on or before October 9, 1969 are
//! grandfathered.
//!
//! § 279(g) SAFE HARBOR: applies only to issuer with more than $5,000,000 of total
//! interest on acquisition indebtedness; small-issuer interest fully deductible.
//!
//! § 279(h) STOCK ACQUIRED AS COMPENSATION: stock acquired as compensation (employee
//! stock-purchase plan, deferred-compensation funding) is NOT an "acquisition" for § 279
//! purposes.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - law.cornell.edu/uscode/text/26/279
//! - law.cornell.edu/cfr/text/26/1.279-2 (amount of disallowance)
//! - law.cornell.edu/cfr/text/26/1.279-3 (corporate acquisition indebtedness definition)
//! - ecfr.gov/current/title-26/chapter-I/subchapter-A/part-1/subject-group-ECFR210006225231fb0/section-1.279-2
//! - irc.bloombergtax.com/public/uscode/doc/irc/section_279

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssuanceDateBranch {
    /// On or before October 9, 1969 — grandfathered out of § 279.
    PreOctober9_1969Grandfathered,
    /// After October 9, 1969 — subject to § 279 analysis.
    PostOctober9_1969SubjectToSection279,
}

/// Whether the obligation is subordinated under § 279(b)(2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubordinationStatus {
    SubordinatedToTradeCreditorsOrUnsecuredIndebtedness,
    NotSubordinatedFailsSection279B2,
}

/// Whether the obligation is convertible/warrant-packaged under § 279(b)(3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConvertibilityStatus {
    ConvertibleIntoIssuerStock,
    IssuedAsInvestmentUnitWithWarrantsOrOptions,
    NeitherConvertibleNorWarrantPackagedFailsSection279B3,
}

/// Whether the issuer fails the § 279(b)(4) debt-equity OR interest-coverage tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DebtEquityAndCoverageStatus {
    /// Debt:equity > 2:1 — fails § 279(b)(4)(A) test, triggers § 279.
    DebtEquityRatioExceedsTwoToOne,
    /// Projected EBITDA fails 3:1 interest-coverage test — fails § 279(b)(4)(B).
    ProjectedEbitdaFailsThreeToOneInterestCoverage,
    /// Both tests passed — § 279(b)(4) safe harbor.
    DebtEquityUnderTwoToOneAndCoverageAboveThreeToOnePasses,
}

/// Stock-acquired-as-compensation safe harbor branch under § 279(h).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompensationStockBranch {
    NotInvoked,
    StockAcquiredAsCompensationSafeHarbor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    PreOctober1969GrandfatheredNoDisallowance,
    Section279HCompensationStockSafeHarborNoDisallowance,
    NotSubordinatedFailsSection279DefinitionNoDisallowance,
    NeitherConvertibleNorWarrantPackagedFailsSection279DefinitionNoDisallowance,
    DebtEquityAndCoverageSafeHarborPassesNoDisallowance,
    SmallIssuerUnderFiveMillionThresholdNoDisallowance,
    Section279AInterestDisallowanceApplied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub issuance_date_branch: IssuanceDateBranch,
    pub subordination_status: SubordinationStatus,
    pub convertibility_status: ConvertibilityStatus,
    pub debt_equity_and_coverage_status: DebtEquityAndCoverageStatus,
    pub compensation_stock_branch: CompensationStockBranch,
    pub total_acquisition_interest_expense_cents: u64,
    pub pre_oct_9_1969_acquisition_interest_cents: u64,
}

pub type Section279CorporateAcquisitionIndebtednessInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub disallowed_interest_cents: u64,
    pub allowed_interest_cents: u64,
    pub effective_threshold_cents: u64,
    pub note: String,
}

pub type Section279CorporateAcquisitionIndebtednessOutput = Output;
pub type Section279CorporateAcquisitionIndebtednessResult = Output;

const SECTION_279A_ANNUAL_THRESHOLD_CENTS: u64 = 500_000_000;
const SECTION_279B1_EFFECTIVE_DATE_LABEL: &str = "October 9, 1969";
const SECTION_279B4_DEBT_EQUITY_LIMIT_LABEL: &str = "2:1";
const SECTION_279B4_INTEREST_COVERAGE_LABEL: &str = "3:1";

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.issuance_date_branch,
        IssuanceDateBranch::PreOctober9_1969Grandfathered
    ) {
        return Output {
            severity: Severity::PreOctober1969GrandfatheredNoDisallowance,
            disallowed_interest_cents: 0,
            allowed_interest_cents: input.total_acquisition_interest_expense_cents,
            effective_threshold_cents: 0,
            note: format!(
                "Pre-{SECTION_279B1_EFFECTIVE_DATE_LABEL} grandfathered obligation. § 279 \
                 applies only to obligations issued AFTER Oct 9, 1969. Interest expense ${} \
                 fully deductible (subject to other limitations such as § 163(j) business \
                 interest limitation and § 385 debt-equity classification).",
                input.total_acquisition_interest_expense_cents / 100
            ),
        };
    }

    if matches!(
        input.compensation_stock_branch,
        CompensationStockBranch::StockAcquiredAsCompensationSafeHarbor
    ) {
        return Output {
            severity: Severity::Section279HCompensationStockSafeHarborNoDisallowance,
            disallowed_interest_cents: 0,
            allowed_interest_cents: input.total_acquisition_interest_expense_cents,
            effective_threshold_cents: 0,
            note: format!(
                "§ 279(h) safe harbor applies. Stock acquired as compensation (employee \
                 stock-purchase plan, deferred-compensation funding, § 83 transfer for \
                 services) is NOT an 'acquisition' for § 279 purposes. Interest expense ${} \
                 fully deductible (subject to § 163(j) and other limitations).",
                input.total_acquisition_interest_expense_cents / 100
            ),
        };
    }

    if matches!(
        input.subordination_status,
        SubordinationStatus::NotSubordinatedFailsSection279B2
    ) {
        return Output {
            severity: Severity::NotSubordinatedFailsSection279DefinitionNoDisallowance,
            disallowed_interest_cents: 0,
            allowed_interest_cents: input.total_acquisition_interest_expense_cents,
            effective_threshold_cents: 0,
            note: format!(
                "Obligation FAILS § 279(b)(2) subordination prong. Not subordinated to trade \
                 creditors generally or to a substantial amount of unsecured indebtedness, so \
                 the obligation does NOT meet the 'corporate acquisition indebtedness' \
                 definition. All four § 279(b) prongs must be satisfied; failing any one prong \
                 takes the obligation out of § 279 altogether. Interest expense ${} fully \
                 deductible under § 279 (subject to § 163(j), § 385, § 269).",
                input.total_acquisition_interest_expense_cents / 100
            ),
        };
    }

    if matches!(
        input.convertibility_status,
        ConvertibilityStatus::NeitherConvertibleNorWarrantPackagedFailsSection279B3
    ) {
        return Output {
            severity:
                Severity::NeitherConvertibleNorWarrantPackagedFailsSection279DefinitionNoDisallowance,
            disallowed_interest_cents: 0,
            allowed_interest_cents: input.total_acquisition_interest_expense_cents,
            effective_threshold_cents: 0,
            note: format!(
                "Obligation FAILS § 279(b)(3) convertibility prong. Neither convertible into \
                 issuer stock NOR issued as part of an investment unit with stock-purchase \
                 warrants or options. Straight non-convertible subordinated debt does NOT meet \
                 the 'corporate acquisition indebtedness' definition. Interest expense ${} \
                 fully deductible under § 279 (subject to § 163(j), § 385).",
                input.total_acquisition_interest_expense_cents / 100
            ),
        };
    }

    if matches!(
        input.debt_equity_and_coverage_status,
        DebtEquityAndCoverageStatus::DebtEquityUnderTwoToOneAndCoverageAboveThreeToOnePasses
    ) {
        return Output {
            severity: Severity::DebtEquityAndCoverageSafeHarborPassesNoDisallowance,
            disallowed_interest_cents: 0,
            allowed_interest_cents: input.total_acquisition_interest_expense_cents,
            effective_threshold_cents: 0,
            note: format!(
                "Obligation PASSES § 279(b)(4) safe harbor. Both debt:equity ratio under \
                 {SECTION_279B4_DEBT_EQUITY_LIMIT_LABEL} AND projected EBITDA interest-coverage \
                 above {SECTION_279B4_INTEREST_COVERAGE_LABEL} for three-taxable-year averaging \
                 period satisfied. Obligation does NOT meet 'corporate acquisition \
                 indebtedness' definition. Interest expense ${} fully deductible under § 279.",
                input.total_acquisition_interest_expense_cents / 100
            ),
        };
    }

    let effective_threshold = SECTION_279A_ANNUAL_THRESHOLD_CENTS
        .saturating_sub(input.pre_oct_9_1969_acquisition_interest_cents);

    if input.total_acquisition_interest_expense_cents <= effective_threshold {
        return Output {
            severity: Severity::SmallIssuerUnderFiveMillionThresholdNoDisallowance,
            disallowed_interest_cents: 0,
            allowed_interest_cents: input.total_acquisition_interest_expense_cents,
            effective_threshold_cents: effective_threshold,
            note: format!(
                "Total acquisition interest expense (${}) under § 279(a) effective threshold \
                 (${}). § 279 disallowance does not apply — interest fully deductible. \
                 Effective threshold = $5,000,000 annual cap reduced by pre-Oct-9-1969 \
                 acquisition interest (${}).",
                input.total_acquisition_interest_expense_cents / 100,
                effective_threshold / 100,
                input.pre_oct_9_1969_acquisition_interest_cents / 100
            ),
        };
    }

    let disallowed = input
        .total_acquisition_interest_expense_cents
        .saturating_sub(effective_threshold);
    Output {
        severity: Severity::Section279AInterestDisallowanceApplied,
        disallowed_interest_cents: disallowed,
        allowed_interest_cents: effective_threshold,
        effective_threshold_cents: effective_threshold,
        note: format!(
            "§ 279(a) disallowance applies. All four § 279(b) prongs satisfied: post-{} \
             issuance + subordination + convertibility/warrant-packaging + failing § 279(b)(4) \
             debt-equity OR interest-coverage test. Interest in excess of $5,000,000 annual \
             cap (reduced by pre-Oct-9-1969 acquisition interest) is PERMANENTLY DISALLOWED \
             (not capitalized, no carryforward). Total acquisition interest ${} minus effective \
             threshold ${} = ${} disallowed; ${} allowed. Coordinates with § 163(j) business \
             interest limit (separate cap on remaining interest), § 385 debt-equity \
             classification, § 269 acquisition-to-evade-tax disallowance.",
            SECTION_279B1_EFFECTIVE_DATE_LABEL,
            input.total_acquisition_interest_expense_cents / 100,
            effective_threshold / 100,
            disallowed / 100,
            effective_threshold / 100
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            issuance_date_branch: IssuanceDateBranch::PostOctober9_1969SubjectToSection279,
            subordination_status:
                SubordinationStatus::SubordinatedToTradeCreditorsOrUnsecuredIndebtedness,
            convertibility_status: ConvertibilityStatus::ConvertibleIntoIssuerStock,
            debt_equity_and_coverage_status:
                DebtEquityAndCoverageStatus::DebtEquityRatioExceedsTwoToOne,
            compensation_stock_branch: CompensationStockBranch::NotInvoked,
            total_acquisition_interest_expense_cents: 10_000_000_00,
            pre_oct_9_1969_acquisition_interest_cents: 0,
        }
    }

    #[test]
    fn pre_october_1969_grandfathered_no_disallowance() {
        let mut input = base();
        input.issuance_date_branch = IssuanceDateBranch::PreOctober9_1969Grandfathered;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::PreOctober1969GrandfatheredNoDisallowance
        );
        assert_eq!(output.disallowed_interest_cents, 0);
        assert_eq!(output.allowed_interest_cents, 10_000_000_00);
        assert!(output.note.contains("October 9, 1969"));
        assert!(output.note.contains("§ 163(j)"));
    }

    #[test]
    fn section_279h_compensation_stock_safe_harbor_no_disallowance() {
        let mut input = base();
        input.compensation_stock_branch =
            CompensationStockBranch::StockAcquiredAsCompensationSafeHarbor;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section279HCompensationStockSafeHarborNoDisallowance
        );
        assert_eq!(output.disallowed_interest_cents, 0);
        assert!(output.note.contains("§ 279(h)"));
        assert!(output.note.contains("§ 83"));
    }

    #[test]
    fn not_subordinated_fails_definition_no_disallowance() {
        let mut input = base();
        input.subordination_status = SubordinationStatus::NotSubordinatedFailsSection279B2;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NotSubordinatedFailsSection279DefinitionNoDisallowance
        );
        assert_eq!(output.disallowed_interest_cents, 0);
        assert!(output.note.contains("§ 279(b)(2)"));
        assert!(output.note.contains("All four"));
    }

    #[test]
    fn neither_convertible_nor_warrant_packaged_fails_definition() {
        let mut input = base();
        input.convertibility_status =
            ConvertibilityStatus::NeitherConvertibleNorWarrantPackagedFailsSection279B3;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NeitherConvertibleNorWarrantPackagedFailsSection279DefinitionNoDisallowance
        );
        assert_eq!(output.disallowed_interest_cents, 0);
        assert!(output.note.contains("§ 279(b)(3)"));
        assert!(output.note.contains("Straight non-convertible"));
    }

    #[test]
    fn debt_equity_and_coverage_safe_harbor_passes_no_disallowance() {
        let mut input = base();
        input.debt_equity_and_coverage_status =
            DebtEquityAndCoverageStatus::DebtEquityUnderTwoToOneAndCoverageAboveThreeToOnePasses;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::DebtEquityAndCoverageSafeHarborPassesNoDisallowance
        );
        assert!(output.note.contains("§ 279(b)(4)"));
        assert!(output.note.contains("2:1"));
        assert!(output.note.contains("3:1"));
    }

    #[test]
    fn small_issuer_under_5_million_threshold_no_disallowance() {
        let mut input = base();
        input.total_acquisition_interest_expense_cents = 4_000_000_00;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::SmallIssuerUnderFiveMillionThresholdNoDisallowance
        );
        assert_eq!(output.disallowed_interest_cents, 0);
        assert_eq!(output.allowed_interest_cents, 4_000_000_00);
        assert_eq!(output.effective_threshold_cents, 5_000_000_00);
    }

    #[test]
    fn debt_equity_failure_triggers_full_disallowance_above_5m() {
        let input = base();
        let output = check(&input);
        // $10M interest - $5M threshold = $5M disallowed
        assert_eq!(
            output.severity,
            Severity::Section279AInterestDisallowanceApplied
        );
        assert_eq!(output.disallowed_interest_cents, 5_000_000_00);
        assert_eq!(output.allowed_interest_cents, 5_000_000_00);
        assert_eq!(output.effective_threshold_cents, 5_000_000_00);
        assert!(output.note.contains("PERMANENTLY DISALLOWED"));
    }

    #[test]
    fn coverage_failure_triggers_full_disallowance_above_5m() {
        let mut input = base();
        input.debt_equity_and_coverage_status =
            DebtEquityAndCoverageStatus::ProjectedEbitdaFailsThreeToOneInterestCoverage;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section279AInterestDisallowanceApplied
        );
        assert_eq!(output.disallowed_interest_cents, 5_000_000_00);
    }

    #[test]
    fn pre_1969_interest_reduces_effective_threshold() {
        let mut input = base();
        input.pre_oct_9_1969_acquisition_interest_cents = 1_000_000_00;
        let output = check(&input);
        // Effective threshold = $5M - $1M = $4M
        // Disallowed = $10M - $4M = $6M
        assert_eq!(output.effective_threshold_cents, 4_000_000_00);
        assert_eq!(output.disallowed_interest_cents, 6_000_000_00);
        assert_eq!(output.allowed_interest_cents, 4_000_000_00);
    }

    #[test]
    fn pre_1969_interest_exceeds_5m_floors_threshold_at_zero() {
        let mut input = base();
        input.pre_oct_9_1969_acquisition_interest_cents = 6_000_000_00;
        let output = check(&input);
        // Effective threshold = $5M saturating_sub $6M = $0
        assert_eq!(output.effective_threshold_cents, 0);
        assert_eq!(output.disallowed_interest_cents, 10_000_000_00);
        assert_eq!(output.allowed_interest_cents, 0);
    }

    #[test]
    fn warrant_packaged_obligation_meets_section_279b3() {
        let mut input = base();
        input.convertibility_status =
            ConvertibilityStatus::IssuedAsInvestmentUnitWithWarrantsOrOptions;
        let output = check(&input);
        // Warrant-packaged satisfies § 279(b)(3) — disallowance still triggers
        assert_eq!(
            output.severity,
            Severity::Section279AInterestDisallowanceApplied
        );
    }

    #[test]
    fn section_279a_threshold_constant_pins_5_million() {
        assert_eq!(SECTION_279A_ANNUAL_THRESHOLD_CENTS, 500_000_000);
    }

    #[test]
    fn note_pins_section_385_debt_equity_classification() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 385"));
    }

    #[test]
    fn note_pins_section_269_acquisition_to_evade_tax() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 269"));
    }

    #[test]
    fn note_pins_section_163j_business_interest_limit() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 163(j)"));
    }

    #[test]
    fn boundary_exactly_5_million_no_disallowance() {
        let mut input = base();
        input.total_acquisition_interest_expense_cents = 5_000_000_00;
        let output = check(&input);
        // 5M = threshold; <= threshold returns no disallowance
        assert_eq!(
            output.severity,
            Severity::SmallIssuerUnderFiveMillionThresholdNoDisallowance
        );
        assert_eq!(output.disallowed_interest_cents, 0);
    }

    #[test]
    fn boundary_one_cent_above_5_million_triggers_disallowance() {
        let mut input = base();
        input.total_acquisition_interest_expense_cents = 500_000_001;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section279AInterestDisallowanceApplied
        );
        assert_eq!(output.disallowed_interest_cents, 1);
    }

    #[test]
    fn very_large_interest_no_overflow() {
        let mut input = base();
        input.total_acquisition_interest_expense_cents = u64::MAX;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section279AInterestDisallowanceApplied
        );
        // saturating_sub defense
        assert_eq!(
            output.disallowed_interest_cents,
            u64::MAX - SECTION_279A_ANNUAL_THRESHOLD_CENTS
        );
    }

    #[test]
    fn zero_interest_no_disallowance() {
        let mut input = base();
        input.total_acquisition_interest_expense_cents = 0;
        let output = check(&input);
        assert_eq!(output.disallowed_interest_cents, 0);
        assert_eq!(output.allowed_interest_cents, 0);
    }

    #[test]
    fn subordination_failure_takes_priority_over_debt_equity_failure() {
        let mut input = base();
        input.subordination_status = SubordinationStatus::NotSubordinatedFailsSection279B2;
        // Even with debt:equity failing, subordination failure dispositive
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NotSubordinatedFailsSection279DefinitionNoDisallowance
        );
    }

    #[test]
    fn compensation_safe_harbor_overrides_all_other_branches() {
        let mut input = base();
        input.compensation_stock_branch =
            CompensationStockBranch::StockAcquiredAsCompensationSafeHarbor;
        input.subordination_status = SubordinationStatus::NotSubordinatedFailsSection279B2;
        let output = check(&input);
        // Compensation safe harbor short-circuits subordination failure
        assert_eq!(
            output.severity,
            Severity::Section279HCompensationStockSafeHarborNoDisallowance
        );
    }

    #[test]
    fn grandfathering_takes_priority_over_compensation_safe_harbor() {
        let mut input = base();
        input.issuance_date_branch = IssuanceDateBranch::PreOctober9_1969Grandfathered;
        input.compensation_stock_branch =
            CompensationStockBranch::StockAcquiredAsCompensationSafeHarbor;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::PreOctober1969GrandfatheredNoDisallowance
        );
    }
}
