//! HUD Housing Opportunity Through Modernization Act of 2016
//! (HOTMA) Income and Asset Compliance Module.
//!
//! Pure-compute check for landlord compliance with HUD's HOTMA
//! Sections 102 and 104 income calculation and asset limitation
//! rules for federally-assisted housing programs (public
//! housing, Section 8 Housing Choice Voucher, Section 8
//! Project-Based Rental Assistance, Multifamily). HOTMA Final
//! Rule (88 FR 9600; February 14, 2023) became generally
//! effective January 1, 2024; Section 102/104 mandatory
//! compliance dates have been extended multiple times — most
//! recently to **January 1, 2027** for Multifamily programs
//! under Notice H-2025-03.
//!
//! Web research (verified 2026-06-03):
//! - **Housing Opportunity Through Modernization Act of 2016**
//!   (**P.L. 114-201**; signed July 29, 2016) — major
//!   bipartisan federal housing reform amending the United
//!   States Housing Act of 1937.
//! - **HUD Final Rule**: HOTMA Income and Assets Final Rule
//!   published in Federal Register on **February 14, 2023**
//!   (**88 FR 9600**); general effective date **January 1,
//!   2024** ([HUD HOTMA Resources](https://www.hud.gov/hud-partners/hotma);
//!   [HUD HOTMA Income and Assets — Public and Indian Housing](https://www.hud.gov/program_offices/public_indian_housing/hotma_income_assets)).
//! - **HOTMA Section 102 — Income Definition** (amends 24
//!   CFR § 5.609(a)): all amounts received by adult household
//!   members plus unearned income by any household member
//!   under age 18 is income, unless excluded; **imputed
//!   returns on assets over $50,000** PLUS actual returns on
//!   assets that can be calculated ([HUD HOTMA Q&A — PIH](https://www.hud.gov/sites/dfiles/PIH/images/PIH%20HOTMA%20QA.pdf)).
//! - **HOTMA Section 103 — Adjusted Income Deductions**:
//!   standardized deductions for elderly families, disabled
//!   families, medical expenses, child care, and other
//!   adjustments under revised 24 CFR § 5.611.
//! - **HOTMA Section 104 — Asset Limitations** (NEW): two
//!   asset limits for public housing, tenant-based Section 8
//!   (Housing Choice Voucher), and project-based Section 8:
//!   (1) **$100,000 net household assets** ceiling, OR (2)
//!   ownership of **real property suitable for occupancy by
//!   household as residence** ([HUD HOTMA Asset Limits
//!   Detailed Guidance — NLIHC](https://nlihc.org/resource/hud-provides-detailed-guidance-regarding-hotmas-asset-limits-provision)).
//! - **Asset Threshold Raised**: imputed asset threshold raised
//!   from **$5,000 to $50,000**; households with ≤ $50,000 in
//!   net assets do NOT need imputed return calculation;
//!   households with > $50,000 in net assets get imputed
//!   return = greater of actual or HUD passbook rate.
//! - **Compliance Extensions**: Section 102/104 mandatory
//!   compliance dates extended multiple times:
//!     - Original: January 1, 2024 (general effective date)
//!     - First extension to **January 1, 2025** (Sept 29, 2023)
//!     - PIH September 18, 2024 announcement — further delay
//!     - HUD Notice **H-2025-03** extended Multifamily compliance
//!       to **January 1, 2026** ([NCSHA — HUD Notice H-2025-03](https://www.ncsha.org/resource/hud-notice-h-2025-03-extension-of-mandatory-compliance-date-sections-102-and-104-of-hotma/))
//!     - Further extension Federal Register **December 30, 2025**
//!       (90 FR) to **January 1, 2027** for full Multifamily
//!       compliance with revised income/asset documentation
//!       standards ([Federal Register — Further Extension](https://www.federalregister.gov/documents/2025/12/30/2025-23989/housing-opportunity-through-modernization-act-implementation-of-sections-102-and-104-further)).
//! - **Covered Programs**: Public Housing; Section 8 Housing
//!   Choice Voucher (tenant-based); Section 8 Project-Based
//!   Rental Assistance; Section 8 Moderate Rehabilitation;
//!   HOPWA (Housing Opportunities for Persons with AIDS); some
//!   Multifamily programs (Section 202, Section 811).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const HOTMA_ENACTMENT_DATE_YEAR: u32 = 2016;
pub const HOTMA_PUBLIC_LAW_NUMBER: u32 = 114_201;
pub const HOTMA_FINAL_RULE_FR_VOLUME: u32 = 88;
pub const HOTMA_FINAL_RULE_FR_PAGE: u32 = 9_600;
pub const HOTMA_GENERAL_EFFECTIVE_DATE_YEAR: u32 = 2024;
pub const HOTMA_GENERAL_EFFECTIVE_DATE_MONTH: u32 = 1;
pub const HOTMA_GENERAL_EFFECTIVE_DATE_DAY: u32 = 1;
pub const HOTMA_PRE_HOTMA_IMPUTED_ASSET_THRESHOLD_DOLLARS: u64 = 5_000;
pub const HOTMA_POST_HOTMA_IMPUTED_ASSET_THRESHOLD_DOLLARS: u64 = 50_000;
pub const HOTMA_SECTION_104_ASSET_LIMIT_DOLLARS: u64 = 100_000;
pub const HOTMA_MULTIFAMILY_FINAL_COMPLIANCE_DEADLINE_YEAR: u32 = 2027;
pub const HOTMA_MULTIFAMILY_FINAL_COMPLIANCE_DEADLINE_MONTH: u32 = 1;
pub const HOTMA_MULTIFAMILY_FINAL_COMPLIANCE_DEADLINE_DAY: u32 = 1;
pub const HOTMA_MINOR_AGE_THRESHOLD: u32 = 18;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HudProgram {
    PublicHousing,
    Section8HousingChoiceVoucher,
    Section8ProjectBasedRentalAssistance,
    Section8ModerateRehabilitation,
    HopwaSection,
    MultifamilySection202Or811,
    NonHudProgramNotCoveredByHotma,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetLimitException {
    NoExceptionApplies,
    ExistingTenantGrandfatheredFromAssetLimit,
    ElderlyOrDisabledHouseholdException,
    NonPurchasingTenantPriorToHotmaEffectiveDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImputedReturnApplied {
    NotRequiredAssetsAtOrBelow50K,
    GreaterOfActualOrPassbookRateAppliedCorrectly,
    NotAppliedDespiteAssetsAbove50K,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HotmaMode {
    NotApplicableNonHudProgram,
    NotApplicableMultifamilyComplianceDeadlineNotYetEffective,
    NotApplicableAssetLimitExceptionApplies,
    CompliantSection102IncomeCalculationCorrectAndSection104AssetsWithinLimit,
    CompliantSection102ImputedReturnAppliedAt50KThreshold,
    CompliantSection104NetAssetsBelow100KAndNoSuitableRealProperty,
    CompliantSection103AdjustedIncomeDeductionsApplied,
    ViolationSection104NetAssetsExceed100KAndNoExceptionApplies,
    ViolationSection104OwnershipOfSuitableRealPropertyAndNoException,
    ViolationSection102ImputedReturnNotAppliedDespiteAssetsAbove50K,
    ViolationSection102IncomeMisclassifiedOrUnderreported,
    ViolationSection103AdjustedIncomeDeductionsMisapplied,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub hud_program: HudProgram,
    pub asset_limit_exception: AssetLimitException,
    pub net_household_assets_dollars: u64,
    pub owns_real_property_suitable_for_residence: bool,
    pub imputed_return_applied: ImputedReturnApplied,
    pub income_calculation_correct: bool,
    pub adjusted_income_deductions_correctly_applied: bool,
    pub current_date_year: u32,
    pub current_date_month: u32,
    pub current_date_day: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: HotmaMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalHudHotmaIncomeAssetComplianceInput = Input;
pub type RentalHudHotmaIncomeAssetComplianceOutput = Output;
pub type RentalHudHotmaIncomeAssetComplianceResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Housing Opportunity Through Modernization Act of 2016 (P.L. 114-201; signed July 29, 2016) — major bipartisan federal housing reform amending the United States Housing Act of 1937".to_string(),
        "HUD HOTMA Income and Assets Final Rule (88 FR 9600; February 14, 2023) — general effective date January 1, 2024".to_string(),
        "HOTMA Section 102 — Income Definition; amends 24 CFR § 5.609(a) — all amounts received by adult household members plus unearned income by household member under age 18 is income unless excluded; imputed returns on assets over $50,000 plus actual returns on calculable assets".to_string(),
        "HOTMA Section 103 — Adjusted Income Deductions; standardized elderly/disabled/medical/child care deductions under revised 24 CFR § 5.611".to_string(),
        "HOTMA Section 104 — Asset Limitations; two asset limits for public housing, tenant-based Section 8 (Housing Choice Voucher), and project-based Section 8: (1) $100,000 net household assets ceiling, OR (2) ownership of real property suitable for occupancy by household as residence".to_string(),
        "Pre-HOTMA imputed asset threshold $5,000 raised to $50,000 — households with ≤ $50,000 in net assets do NOT need imputed return calculation; households > $50,000 get imputed return = greater of actual or HUD passbook rate".to_string(),
        "Compliance Extensions — original deadline January 1, 2024; first extension to January 1, 2025 (September 29, 2023); PIH September 18, 2024 announcement further delay; HUD Notice H-2025-03 extended Multifamily compliance to January 1, 2026; Federal Register December 30, 2025 (90 FR) further extension to January 1, 2027 for full Multifamily compliance".to_string(),
        "Covered Programs — Public Housing; Section 8 Housing Choice Voucher (tenant-based); Section 8 Project-Based Rental Assistance; Section 8 Moderate Rehabilitation; HOPWA (Housing Opportunities for Persons with AIDS); some Multifamily Section 202 / Section 811 programs".to_string(),
        "Existing Tenant Grandfathering — tenants in residence as of effective date generally grandfathered from Section 104 asset limit until next certification cycle or program change".to_string(),
        "Elderly / Disabled Household Exception — certain elderly (62+) and disabled household exceptions to Section 104 asset limit under HOTMA implementing regulations".to_string(),
        "HUD HOTMA Resources — primary HUD landing page with regulatory guidance and implementation notices".to_string(),
        "HUD HOTMA Q&A — Public and Indian Housing comprehensive PHA guidance".to_string(),
        "NLIHC HOTMA Asset Limits Detailed Guidance — practitioner analysis".to_string(),
        "NAHRO HUD Releases New HOTMA 102 and 104 Compliance Guidance — industry summary".to_string(),
        "NCSHA HUD Notice H-2025-03 — Extension of Mandatory Compliance Date".to_string(),
    ];

    if input.hud_program == HudProgram::NonHudProgramNotCoveredByHotma {
        return Output {
            mode: HotmaMode::NotApplicableNonHudProgram,
            statutory_basis: "HOTMA inapplicable — property not in HUD program covered by HOTMA".to_string(),
            notes: "NOT APPLICABLE: property not in HUD program covered by HOTMA; private market rental not subject to HOTMA income and asset rules.".to_string(),
            citations,
        };
    }

    if matches!(
        input.hud_program,
        HudProgram::MultifamilySection202Or811
    ) && (input.current_date_year < HOTMA_MULTIFAMILY_FINAL_COMPLIANCE_DEADLINE_YEAR
        || (input.current_date_year == HOTMA_MULTIFAMILY_FINAL_COMPLIANCE_DEADLINE_YEAR
            && input.current_date_month < HOTMA_MULTIFAMILY_FINAL_COMPLIANCE_DEADLINE_MONTH))
    {
        return Output {
            mode: HotmaMode::NotApplicableMultifamilyComplianceDeadlineNotYetEffective,
            statutory_basis: "HUD Notice H-2025-03 + Federal Register Dec 30, 2025 — Multifamily compliance deadline extended to January 1, 2027".to_string(),
            notes: format!(
                "NOT APPLICABLE: Multifamily Section 202/811 program HOTMA mandatory compliance deadline extended to January 1, 2027; current date {}/{}/{} is before deadline.",
                input.current_date_year, input.current_date_month, input.current_date_day
            ),
            citations,
        };
    }

    if !matches!(
        input.asset_limit_exception,
        AssetLimitException::NoExceptionApplies
    ) {
        return Output {
            mode: HotmaMode::NotApplicableAssetLimitExceptionApplies,
            statutory_basis: "HOTMA Section 104 — statutory exception applies".to_string(),
            notes: format!(
                "NOT APPLICABLE: Section 104 asset limit exception applies ({:?}); household exempt from $100,000 net asset limit + real property limit.",
                input.asset_limit_exception
            ),
            citations,
        };
    }

    if !input.income_calculation_correct {
        return Output {
            mode: HotmaMode::ViolationSection102IncomeMisclassifiedOrUnderreported,
            statutory_basis: "HOTMA Section 102 — 24 CFR § 5.609(a) income calculation requirements".to_string(),
            notes: "VIOLATION: Section 102 income calculation incorrect; all amounts received by adult household members plus unearned income by minors under age 18 must be classified as income unless statutorily excluded.".to_string(),
            citations,
        };
    }

    if !input.adjusted_income_deductions_correctly_applied {
        return Output {
            mode: HotmaMode::ViolationSection103AdjustedIncomeDeductionsMisapplied,
            statutory_basis: "HOTMA Section 103 — 24 CFR § 5.611 adjusted income deductions".to_string(),
            notes: "VIOLATION: Section 103 adjusted income deductions misapplied (elderly/disabled/medical/child care standardized deductions under revised 24 CFR § 5.611).".to_string(),
            citations,
        };
    }

    if input.net_household_assets_dollars > HOTMA_POST_HOTMA_IMPUTED_ASSET_THRESHOLD_DOLLARS
        && input.imputed_return_applied == ImputedReturnApplied::NotAppliedDespiteAssetsAbove50K
    {
        return Output {
            mode: HotmaMode::ViolationSection102ImputedReturnNotAppliedDespiteAssetsAbove50K,
            statutory_basis: "HOTMA Section 102 — imputed return required on assets over $50,000".to_string(),
            notes: format!(
                "VIOLATION: net household assets of ${} exceed $50,000 imputed-return threshold; HOTMA Section 102 requires imputed return = greater of actual or HUD passbook rate; landlord failed to apply.",
                input.net_household_assets_dollars
            ),
            citations,
        };
    }

    if input.owns_real_property_suitable_for_residence {
        return Output {
            mode: HotmaMode::ViolationSection104OwnershipOfSuitableRealPropertyAndNoException,
            statutory_basis: "HOTMA Section 104 — household may not own real property suitable for occupancy as residence".to_string(),
            notes: "VIOLATION: household owns real property suitable for occupancy by household as residence; HOTMA Section 104 disqualifies household from continued HUD assistance absent statutory exception.".to_string(),
            citations,
        };
    }

    if input.net_household_assets_dollars > HOTMA_SECTION_104_ASSET_LIMIT_DOLLARS {
        return Output {
            mode: HotmaMode::ViolationSection104NetAssetsExceed100KAndNoExceptionApplies,
            statutory_basis: "HOTMA Section 104 — $100,000 net household assets ceiling".to_string(),
            notes: format!(
                "VIOLATION: net household assets of ${} exceed $100,000 Section 104 statutory ceiling; no exception applies; household disqualified from continued HUD assistance.",
                input.net_household_assets_dollars
            ),
            citations,
        };
    }

    if input.net_household_assets_dollars > HOTMA_POST_HOTMA_IMPUTED_ASSET_THRESHOLD_DOLLARS {
        return Output {
            mode: HotmaMode::CompliantSection102ImputedReturnAppliedAt50KThreshold,
            statutory_basis: "HOTMA Section 102 — imputed return applied correctly above $50,000 threshold".to_string(),
            notes: format!(
                "COMPLIANT: net household assets of ${} between $50,000 imputed-return threshold and $100,000 Section 104 ceiling; imputed return = greater of actual or HUD passbook rate correctly applied; all other HOTMA requirements satisfied.",
                input.net_household_assets_dollars
            ),
            citations,
        };
    }

    Output {
        mode: HotmaMode::CompliantSection102IncomeCalculationCorrectAndSection104AssetsWithinLimit,
        statutory_basis: "HOTMA Sections 102 + 103 + 104 — all income/deduction/asset requirements satisfied".to_string(),
        notes: format!(
            "COMPLIANT: net household assets of ${} ≤ $50,000 imputed-return threshold (no imputed return required); Section 102 income calculation correct; Section 103 adjusted income deductions applied; Section 104 asset limit satisfied; no real property suitable for residence owned.",
            input.net_household_assets_dollars
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_compliant_section_8_voucher() -> Input {
        Input {
            hud_program: HudProgram::Section8HousingChoiceVoucher,
            asset_limit_exception: AssetLimitException::NoExceptionApplies,
            net_household_assets_dollars: 30_000,
            owns_real_property_suitable_for_residence: false,
            imputed_return_applied: ImputedReturnApplied::NotRequiredAssetsAtOrBelow50K,
            income_calculation_correct: true,
            adjusted_income_deductions_correctly_applied: true,
            current_date_year: 2026,
            current_date_month: 6,
            current_date_day: 3,
        }
    }

    #[test]
    fn non_hud_program_not_applicable() {
        let input = Input {
            hud_program: HudProgram::NonHudProgramNotCoveredByHotma,
            ..baseline_compliant_section_8_voucher()
        };
        let result = check(&input);
        assert_eq!(result.mode, HotmaMode::NotApplicableNonHudProgram);
    }

    #[test]
    fn multifamily_pre_2027_deadline_not_applicable() {
        let input = Input {
            hud_program: HudProgram::MultifamilySection202Or811,
            current_date_year: 2026,
            current_date_month: 12,
            ..baseline_compliant_section_8_voucher()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            HotmaMode::NotApplicableMultifamilyComplianceDeadlineNotYetEffective
        );
    }

    #[test]
    fn multifamily_at_2027_compliance_deadline_applicable() {
        let input = Input {
            hud_program: HudProgram::MultifamilySection202Or811,
            current_date_year: 2027,
            current_date_month: 1,
            current_date_day: 1,
            ..baseline_compliant_section_8_voucher()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            HotmaMode::CompliantSection102IncomeCalculationCorrectAndSection104AssetsWithinLimit
        );
    }

    #[test]
    fn asset_limit_exception_grandfathered_not_applicable() {
        let input = Input {
            asset_limit_exception: AssetLimitException::ExistingTenantGrandfatheredFromAssetLimit,
            net_household_assets_dollars: 500_000,
            ..baseline_compliant_section_8_voucher()
        };
        let result = check(&input);
        assert_eq!(result.mode, HotmaMode::NotApplicableAssetLimitExceptionApplies);
    }

    #[test]
    fn elderly_disabled_exception_not_applicable() {
        let input = Input {
            asset_limit_exception: AssetLimitException::ElderlyOrDisabledHouseholdException,
            ..baseline_compliant_section_8_voucher()
        };
        let result = check(&input);
        assert_eq!(result.mode, HotmaMode::NotApplicableAssetLimitExceptionApplies);
    }

    #[test]
    fn standard_compliant_section_8_voucher_at_50k_threshold() {
        let result = check(&baseline_compliant_section_8_voucher());
        assert_eq!(
            result.mode,
            HotmaMode::CompliantSection102IncomeCalculationCorrectAndSection104AssetsWithinLimit
        );
    }

    #[test]
    fn assets_at_exactly_50k_below_imputed_threshold_compliant() {
        let input = Input {
            net_household_assets_dollars: 50_000,
            imputed_return_applied: ImputedReturnApplied::NotRequiredAssetsAtOrBelow50K,
            ..baseline_compliant_section_8_voucher()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            HotmaMode::CompliantSection102IncomeCalculationCorrectAndSection104AssetsWithinLimit
        );
    }

    #[test]
    fn assets_at_50001_triggers_imputed_return_compliant() {
        let input = Input {
            net_household_assets_dollars: 50_001,
            imputed_return_applied: ImputedReturnApplied::GreaterOfActualOrPassbookRateAppliedCorrectly,
            ..baseline_compliant_section_8_voucher()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            HotmaMode::CompliantSection102ImputedReturnAppliedAt50KThreshold
        );
    }

    #[test]
    fn assets_above_50k_imputed_return_not_applied_violation() {
        let input = Input {
            net_household_assets_dollars: 75_000,
            imputed_return_applied: ImputedReturnApplied::NotAppliedDespiteAssetsAbove50K,
            ..baseline_compliant_section_8_voucher()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            HotmaMode::ViolationSection102ImputedReturnNotAppliedDespiteAssetsAbove50K
        );
    }

    #[test]
    fn assets_at_exactly_100k_compliant() {
        let input = Input {
            net_household_assets_dollars: 100_000,
            imputed_return_applied: ImputedReturnApplied::GreaterOfActualOrPassbookRateAppliedCorrectly,
            ..baseline_compliant_section_8_voucher()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            HotmaMode::CompliantSection102ImputedReturnAppliedAt50KThreshold
        );
    }

    #[test]
    fn assets_at_100001_section_104_violation() {
        let input = Input {
            net_household_assets_dollars: 100_001,
            imputed_return_applied: ImputedReturnApplied::GreaterOfActualOrPassbookRateAppliedCorrectly,
            ..baseline_compliant_section_8_voucher()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            HotmaMode::ViolationSection104NetAssetsExceed100KAndNoExceptionApplies
        );
    }

    #[test]
    fn real_property_suitable_for_residence_violation() {
        let input = Input {
            owns_real_property_suitable_for_residence: true,
            ..baseline_compliant_section_8_voucher()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            HotmaMode::ViolationSection104OwnershipOfSuitableRealPropertyAndNoException
        );
    }

    #[test]
    fn income_calculation_incorrect_violation() {
        let input = Input {
            income_calculation_correct: false,
            ..baseline_compliant_section_8_voucher()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            HotmaMode::ViolationSection102IncomeMisclassifiedOrUnderreported
        );
    }

    #[test]
    fn adjusted_income_deductions_misapplied_violation() {
        let input = Input {
            adjusted_income_deductions_correctly_applied: false,
            ..baseline_compliant_section_8_voucher()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            HotmaMode::ViolationSection103AdjustedIncomeDeductionsMisapplied
        );
    }

    #[test]
    fn public_housing_compliant() {
        let input = Input {
            hud_program: HudProgram::PublicHousing,
            ..baseline_compliant_section_8_voucher()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            HotmaMode::CompliantSection102IncomeCalculationCorrectAndSection104AssetsWithinLimit
        );
    }

    #[test]
    fn section_8_pbra_compliant() {
        let input = Input {
            hud_program: HudProgram::Section8ProjectBasedRentalAssistance,
            ..baseline_compliant_section_8_voucher()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            HotmaMode::CompliantSection102IncomeCalculationCorrectAndSection104AssetsWithinLimit
        );
    }

    #[test]
    fn citations_pin_hotma_sections_and_dates() {
        let result = check(&baseline_compliant_section_8_voucher());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("Housing Opportunity Through Modernization Act of 2016"));
        assert!(joined.contains("P.L. 114-201"));
        assert!(joined.contains("July 29, 2016"));
        assert!(joined.contains("88 FR 9600"));
        assert!(joined.contains("February 14, 2023"));
        assert!(joined.contains("January 1, 2024"));
        assert!(joined.contains("HOTMA Section 102"));
        assert!(joined.contains("HOTMA Section 103"));
        assert!(joined.contains("HOTMA Section 104"));
        assert!(joined.contains("24 CFR § 5.609(a)"));
        assert!(joined.contains("24 CFR § 5.611"));
        assert!(joined.contains("$50,000"));
        assert!(joined.contains("$100,000"));
        assert!(joined.contains("$5,000 raised to $50,000"));
        assert!(joined.contains("Public Housing"));
        assert!(joined.contains("Section 8 Housing Choice Voucher"));
        assert!(joined.contains("Project-Based Rental Assistance"));
        assert!(joined.contains("HOPWA"));
        assert!(joined.contains("Section 202"));
        assert!(joined.contains("Section 811"));
        assert!(joined.contains("Notice H-2025-03"));
        assert!(joined.contains("January 1, 2027"));
        assert!(joined.contains("NLIHC"));
        assert!(joined.contains("NAHRO"));
        assert!(joined.contains("NCSHA"));
    }

    #[test]
    fn constant_pin_dates_and_thresholds() {
        assert_eq!(HOTMA_ENACTMENT_DATE_YEAR, 2016);
        assert_eq!(HOTMA_PUBLIC_LAW_NUMBER, 114_201);
        assert_eq!(HOTMA_FINAL_RULE_FR_VOLUME, 88);
        assert_eq!(HOTMA_FINAL_RULE_FR_PAGE, 9_600);
        assert_eq!(HOTMA_GENERAL_EFFECTIVE_DATE_YEAR, 2024);
        assert_eq!(HOTMA_GENERAL_EFFECTIVE_DATE_MONTH, 1);
        assert_eq!(HOTMA_GENERAL_EFFECTIVE_DATE_DAY, 1);
        assert_eq!(HOTMA_PRE_HOTMA_IMPUTED_ASSET_THRESHOLD_DOLLARS, 5_000);
        assert_eq!(HOTMA_POST_HOTMA_IMPUTED_ASSET_THRESHOLD_DOLLARS, 50_000);
        assert_eq!(HOTMA_SECTION_104_ASSET_LIMIT_DOLLARS, 100_000);
        assert_eq!(HOTMA_MULTIFAMILY_FINAL_COMPLIANCE_DEADLINE_YEAR, 2027);
        assert_eq!(HOTMA_MULTIFAMILY_FINAL_COMPLIANCE_DEADLINE_MONTH, 1);
        assert_eq!(HOTMA_MULTIFAMILY_FINAL_COMPLIANCE_DEADLINE_DAY, 1);
        assert_eq!(HOTMA_MINOR_AGE_THRESHOLD, 18);
    }
}
