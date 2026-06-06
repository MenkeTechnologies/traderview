//! NYC SCRIE / DRIE Rent Freeze Program Compliance Module.
//!
//! Pure-compute compliance check for the NYC Senior Citizen Rent
//! Increase Exemption (SCRIE) and Disability Rent Increase
//! Exemption (DRIE) programs. Both programs PERMANENTLY FREEZE
//! tenant rent at the prior level; every future Rent Guidelines
//! Board (RGB) increase is credited back to the landlord as a
//! property tax abatement rather than charged to the tenant.
//! Trader-landlord critical because SCRIE / DRIE-frozen units
//! create a NYC-only property-tax-abatement revenue stream that
//! must be properly applied and reported; failure to honor the
//! freeze creates statutory violations + clawback exposure.
//!
//! Web research (verified 2026-06-03):
//! - **NYC Admin Code § 26-509** (Local Law 6 of 1986) — SCRIE:
//!   tenant aged **62 or older**; legal tenant of record in a
//!   rent-regulated apartment (rent-stabilized, rent-controlled,
//!   or rent-regulated Mitchell-Lama / HDFC co-op); household
//!   income at or below **$50,000 per year** for 2025; paying
//!   **more than one-third (33.33 %)** of monthly income in rent.
//!   Future RGB increases credited back to landlord as property
//!   tax abatement. ([NYC Department of Finance Rent Freeze
//!   Qualifications](https://www.nyc.gov/site/rentfreeze/qualifications/qualifications.page);
//!   HelpNewYork — NYC Rent Freeze Program 2026.)
//! - **NYC Admin Code § 26-509.1** (Local Law 76 of 2005) — DRIE:
//!   tenant aged **18 or older** currently receiving qualifying
//!   federal disability benefit (SSI / SSDI / VA disability
//!   compensation) OR currently receiving Medicaid based on
//!   disability determination; same **$50,000 income cap** and
//!   **> 1/3 rent burden** as SCRIE. ([NYC Department of Finance
//!   DRIE overview; ACCESS NYC DRIE explainer](https://access.nyc.gov/programs/disability-rent-increase-exemption-drie/);
//!   LegalClarity DRIE NYC Disability Rent Freeze.)
//! - **Renewal**: required every **2 years**; NYC Department of
//!   Finance mails renewal notice approximately **60 days** before
//!   benefit expires.
//! - **Proposed 2026 income limit increase**: Governor Hochul's
//!   2026 State of the State proposal would raise SCRIE / DRIE
//!   income eligibility limit from **$50,000 to $75,000** in NYC
//!   (and to $50,000 statewide for analog programs). Bill tracking
//!   under NY Senate S01457 (2025-2026 session). ([NY Senate
//!   S01457; Retired Public Employees Association State of the
//!   State 2026 Proposals.)

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SCRIE_SENIOR_MIN_AGE: u32 = 62;
pub const DRIE_DISABILITY_MIN_AGE: u32 = 18;
pub const SCRIE_DRIE_INCOME_LIMIT_DOLLARS_2025: u64 = 50_000;
pub const SCRIE_DRIE_PROPOSED_INCOME_LIMIT_DOLLARS_2026: u64 = 75_000;
pub const SCRIE_DRIE_RENT_BURDEN_THRESHOLD_BASIS_POINTS: u64 = 3_333;
pub const SCRIE_DRIE_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const SCRIE_DRIE_RENEWAL_INTERVAL_MONTHS: u32 = 24;
pub const SCRIE_DRIE_RENEWAL_NOTICE_DAYS_BEFORE_EXPIRATION: u32 = 60;
pub const SCRIE_LOCAL_LAW_YEAR: u32 = 1986;
pub const SCRIE_LOCAL_LAW_NUMBER: u32 = 6;
pub const DRIE_LOCAL_LAW_YEAR: u32 = 2005;
pub const DRIE_LOCAL_LAW_NUMBER: u32 = 76;
pub const NY_SENATE_S01457_2026_PROPOSED_INCOME_INCREASE_BILL: u32 = 1457;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScrieDrieProgram {
    ScrieSeniorCitizen,
    DrieDisability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RegulatedUnitType {
    RentStabilized,
    RentControlled,
    MitchellLamaRentRegulated,
    HdfcCoopRentRegulated,
    MarketRateUnregulated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FederalDisabilityBenefit {
    Ssi,
    Ssdi,
    VaDisabilityCompensation,
    MedicaidBasedOnDisabilityDetermination,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScrieDrieMode {
    NotApplicableNotInNyc,
    NotApplicableNotInRentRegulatedUnit,
    CompliantScrieFrozenRentAndPropertyTaxAbatementApplied,
    CompliantDrieFrozenRentAndPropertyTaxAbatementApplied,
    CompliantRenewalSubmittedWithin2YearWindow,
    ViolationLandlordChargedTenantPostFreezeIncreases,
    ViolationLandlordFailedToReflectPropertyTaxAbatement,
    ViolationTenantNotTenantOfRecord,
    ViolationIncomeLimitExceeded50000,
    ViolationRentBurdenBelowOneThirdThreshold,
    ViolationRenewalNotSubmittedExpirationLapsed,
    ViolationTenantUnderAge62ForScrie,
    ViolationTenantUnderAge18ForDrie,
    ViolationTenantNotReceivingQualifyingFederalDisabilityForDrie,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_in_nyc: bool,
    pub regulated_unit_type: RegulatedUnitType,
    pub program: ScrieDrieProgram,
    pub tenant_of_record: bool,
    pub tenant_age_years: u32,
    pub household_annual_income_dollars: u64,
    pub monthly_rent_dollars: u64,
    pub monthly_income_dollars: u64,
    pub federal_disability_benefit: FederalDisabilityBenefit,
    pub landlord_charged_post_freeze_increases: bool,
    pub landlord_applied_property_tax_abatement: bool,
    pub renewal_application_submitted_within_window: bool,
    pub months_since_last_renewal: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: ScrieDrieMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalNycScrieDrieRentFreezeInput = Input;
pub type RentalNycScrieDrieRentFreezeOutput = Output;
pub type RentalNycScrieDrieRentFreezeResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

fn rent_burden_meets_one_third(monthly_rent: u64, monthly_income: u64) -> bool {
    if monthly_income == 0 {
        return false;
    }
    let burden_bp = (monthly_rent as u128)
        .saturating_mul(SCRIE_DRIE_BASIS_POINT_DENOMINATOR as u128)
        .checked_div(monthly_income as u128)
        .unwrap_or(0) as u64;
    burden_bp > SCRIE_DRIE_RENT_BURDEN_THRESHOLD_BASIS_POINTS
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "NYC Admin Code § 26-509 (Local Law 6 of 1986) — SCRIE: senior 62+ in rent-regulated unit; income ≤ $50,000 (2025); > 1/3 rent burden; RGB increases credited to landlord as property tax abatement".to_string(),
        "NYC Admin Code § 26-509.1 (Local Law 76 of 2005) — DRIE: disabled tenant 18+ with SSI/SSDI/VA disability/Medicaid-based-on-disability; same $50,000 income cap; same > 1/3 rent burden".to_string(),
        "NYC Department of Finance Rent Freeze Qualifications — eligibility + application + renewal procedure".to_string(),
        "Renewal required every 24 months; DOF mails renewal notice approximately 60 days before benefit expires".to_string(),
        "NY Senate S01457 (2025-2026 session) — proposed increase of SCRIE/DRIE income limit from $50,000 to $75,000 NYC".to_string(),
        "ACCESS NYC DRIE program page — qualifying federal disability benefits: SSI, SSDI, VA disability compensation, Medicaid based on disability determination".to_string(),
        "RGB (Rent Guidelines Board) annual increase order superseded for SCRIE/DRIE-frozen units; tenant pays prior rent; landlord recovers via property tax abatement on DOF Form RP-467-C".to_string(),
    ];

    if !input.property_in_nyc {
        return Output {
            mode: ScrieDrieMode::NotApplicableNotInNyc,
            statutory_basis: "Property outside NYC; SCRIE/DRIE inapplicable".to_string(),
            notes: "Property not located in NYC; SCRIE / DRIE local-law programs inapplicable."
                .to_string(),
            citations,
        };
    }

    if input.regulated_unit_type == RegulatedUnitType::MarketRateUnregulated {
        return Output {
            mode: ScrieDrieMode::NotApplicableNotInRentRegulatedUnit,
            statutory_basis: "SCRIE/DRIE requires rent-regulated unit".to_string(),
            notes: "Unit is market-rate unregulated; SCRIE/DRIE applies only to rent-stabilized, rent-controlled, or rent-regulated Mitchell-Lama / HDFC co-op units.".to_string(),
            citations,
        };
    }

    if !input.tenant_of_record {
        return Output {
            mode: ScrieDrieMode::ViolationTenantNotTenantOfRecord,
            statutory_basis: "SCRIE/DRIE requires applicant to be legal tenant of record".to_string(),
            notes: "VIOLATION: SCRIE/DRIE applicant must be the legal tenant of record on the lease; not satisfied.".to_string(),
            citations,
        };
    }

    if input.household_annual_income_dollars > SCRIE_DRIE_INCOME_LIMIT_DOLLARS_2025 {
        return Output {
            mode: ScrieDrieMode::ViolationIncomeLimitExceeded50000,
            statutory_basis: "SCRIE/DRIE 2025 income limit $50,000 exceeded".to_string(),
            notes: format!(
                "VIOLATION: household income ${} exceeds 2025 income limit of $50,000. (Note: NY Senate S01457 proposes raise to $75,000 for 2026.)",
                input.household_annual_income_dollars
            ),
            citations,
        };
    }

    if !rent_burden_meets_one_third(input.monthly_rent_dollars, input.monthly_income_dollars) {
        return Output {
            mode: ScrieDrieMode::ViolationRentBurdenBelowOneThirdThreshold,
            statutory_basis: "SCRIE/DRIE requires rent > 1/3 of monthly income".to_string(),
            notes: format!(
                "VIOLATION: monthly rent ${} is not more than 1/3 of monthly income ${}; required > 33.33 % rent burden not satisfied.",
                input.monthly_rent_dollars, input.monthly_income_dollars
            ),
            citations,
        };
    }

    match input.program {
        ScrieDrieProgram::ScrieSeniorCitizen => {
            if input.tenant_age_years < SCRIE_SENIOR_MIN_AGE {
                return Output {
                    mode: ScrieDrieMode::ViolationTenantUnderAge62ForScrie,
                    statutory_basis: "SCRIE requires tenant 62+ years old".to_string(),
                    notes: format!(
                        "VIOLATION SCRIE: tenant age {} below 62-year minimum.",
                        input.tenant_age_years
                    ),
                    citations,
                };
            }
        }
        ScrieDrieProgram::DrieDisability => {
            if input.tenant_age_years < DRIE_DISABILITY_MIN_AGE {
                return Output {
                    mode: ScrieDrieMode::ViolationTenantUnderAge18ForDrie,
                    statutory_basis: "DRIE requires tenant 18+ years old".to_string(),
                    notes: format!(
                        "VIOLATION DRIE: tenant age {} below 18-year minimum.",
                        input.tenant_age_years
                    ),
                    citations,
                };
            }
            if input.federal_disability_benefit == FederalDisabilityBenefit::None {
                return Output {
                    mode: ScrieDrieMode::ViolationTenantNotReceivingQualifyingFederalDisabilityForDrie,
                    statutory_basis: "DRIE requires SSI/SSDI/VA disability compensation/Medicaid-based-on-disability".to_string(),
                    notes: "VIOLATION DRIE: tenant not currently receiving qualifying federal disability benefit (SSI, SSDI, VA disability compensation, or Medicaid based on disability determination).".to_string(),
                    citations,
                };
            }
        }
    }

    if input.months_since_last_renewal > SCRIE_DRIE_RENEWAL_INTERVAL_MONTHS
        && !input.renewal_application_submitted_within_window
    {
        return Output {
            mode: ScrieDrieMode::ViolationRenewalNotSubmittedExpirationLapsed,
            statutory_basis: "SCRIE/DRIE renewal required every 24 months".to_string(),
            notes: format!(
                "VIOLATION: {} months have elapsed since last renewal; 24-month renewal not submitted; SCRIE/DRIE benefit lapsed.",
                input.months_since_last_renewal
            ),
            citations,
        };
    }

    if input.landlord_charged_post_freeze_increases {
        return Output {
            mode: ScrieDrieMode::ViolationLandlordChargedTenantPostFreezeIncreases,
            statutory_basis: "SCRIE/DRIE freeze prohibits landlord from charging tenant post-freeze RGB increases".to_string(),
            notes: "VIOLATION: landlord charged tenant RGB increases that should have been credited back to landlord as property tax abatement, not billed to tenant.".to_string(),
            citations,
        };
    }

    if !input.landlord_applied_property_tax_abatement {
        return Output {
            mode: ScrieDrieMode::ViolationLandlordFailedToReflectPropertyTaxAbatement,
            statutory_basis: "SCRIE/DRIE property tax abatement claim required by landlord".to_string(),
            notes: "VIOLATION: landlord failed to claim property tax abatement on DOF Form RP-467-C reflecting frozen-rent increases; landlord cannot recover the foregone rent without claiming the abatement.".to_string(),
            citations,
        };
    }

    if input.renewal_application_submitted_within_window {
        return Output {
            mode: ScrieDrieMode::CompliantRenewalSubmittedWithin2YearWindow,
            statutory_basis: "SCRIE/DRIE renewal application submitted within 24-month window".to_string(),
            notes: format!(
                "COMPLIANT: SCRIE/DRIE renewal application submitted within 24-month window ({} months since last renewal).",
                input.months_since_last_renewal
            ),
            citations,
        };
    }

    match input.program {
        ScrieDrieProgram::ScrieSeniorCitizen => Output {
            mode: ScrieDrieMode::CompliantScrieFrozenRentAndPropertyTaxAbatementApplied,
            statutory_basis: "NYC Admin Code § 26-509 SCRIE satisfied".to_string(),
            notes: format!(
                "COMPLIANT SCRIE: senior 62+ in rent-regulated unit; income ${} ≤ $50,000; rent burden > 1/3; landlord applied property tax abatement; tenant rent frozen at prior level.",
                input.household_annual_income_dollars
            ),
            citations,
        },
        ScrieDrieProgram::DrieDisability => Output {
            mode: ScrieDrieMode::CompliantDrieFrozenRentAndPropertyTaxAbatementApplied,
            statutory_basis: "NYC Admin Code § 26-509.1 DRIE satisfied".to_string(),
            notes: format!(
                "COMPLIANT DRIE: disabled tenant 18+ with {:?} benefit; income ${} ≤ $50,000; rent burden > 1/3; landlord applied property tax abatement; tenant rent frozen at prior level.",
                input.federal_disability_benefit, input.household_annual_income_dollars
            ),
            citations,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_scrie_compliant() -> Input {
        Input {
            property_in_nyc: true,
            regulated_unit_type: RegulatedUnitType::RentStabilized,
            program: ScrieDrieProgram::ScrieSeniorCitizen,
            tenant_of_record: true,
            tenant_age_years: 70,
            household_annual_income_dollars: 40_000,
            monthly_rent_dollars: 1_500,
            monthly_income_dollars: 3_333,
            federal_disability_benefit: FederalDisabilityBenefit::None,
            landlord_charged_post_freeze_increases: false,
            landlord_applied_property_tax_abatement: true,
            renewal_application_submitted_within_window: false,
            months_since_last_renewal: 12,
        }
    }

    #[test]
    fn property_not_in_nyc_not_applicable() {
        let input = Input {
            property_in_nyc: false,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, ScrieDrieMode::NotApplicableNotInNyc);
    }

    #[test]
    fn unregulated_unit_not_applicable() {
        let input = Input {
            regulated_unit_type: RegulatedUnitType::MarketRateUnregulated,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ScrieDrieMode::NotApplicableNotInRentRegulatedUnit
        );
    }

    #[test]
    fn scrie_compliant_baseline() {
        let result = check(&baseline_scrie_compliant());
        assert_eq!(
            result.mode,
            ScrieDrieMode::CompliantScrieFrozenRentAndPropertyTaxAbatementApplied
        );
    }

    #[test]
    fn drie_compliant_with_ssdi() {
        let input = Input {
            program: ScrieDrieProgram::DrieDisability,
            tenant_age_years: 30,
            federal_disability_benefit: FederalDisabilityBenefit::Ssdi,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ScrieDrieMode::CompliantDrieFrozenRentAndPropertyTaxAbatementApplied
        );
    }

    #[test]
    fn tenant_under_62_scrie_violation() {
        let input = Input {
            tenant_age_years: 61,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ScrieDrieMode::ViolationTenantUnderAge62ForScrie
        );
    }

    #[test]
    fn tenant_at_exactly_62_compliant() {
        let input = Input {
            tenant_age_years: 62,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ScrieDrieMode::CompliantScrieFrozenRentAndPropertyTaxAbatementApplied
        );
    }

    #[test]
    fn drie_no_qualifying_disability_violation() {
        let input = Input {
            program: ScrieDrieProgram::DrieDisability,
            tenant_age_years: 30,
            federal_disability_benefit: FederalDisabilityBenefit::None,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ScrieDrieMode::ViolationTenantNotReceivingQualifyingFederalDisabilityForDrie
        );
    }

    #[test]
    fn drie_under_18_violation() {
        let input = Input {
            program: ScrieDrieProgram::DrieDisability,
            tenant_age_years: 17,
            federal_disability_benefit: FederalDisabilityBenefit::Ssi,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, ScrieDrieMode::ViolationTenantUnderAge18ForDrie);
    }

    #[test]
    fn not_tenant_of_record_violation() {
        let input = Input {
            tenant_of_record: false,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, ScrieDrieMode::ViolationTenantNotTenantOfRecord);
    }

    #[test]
    fn income_over_50000_violation() {
        let input = Input {
            household_annual_income_dollars: 51_000,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ScrieDrieMode::ViolationIncomeLimitExceeded50000
        );
    }

    #[test]
    fn income_at_exactly_50000_compliant() {
        let input = Input {
            household_annual_income_dollars: 50_000,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ScrieDrieMode::CompliantScrieFrozenRentAndPropertyTaxAbatementApplied
        );
    }

    #[test]
    fn rent_burden_below_one_third_violation() {
        let input = Input {
            monthly_rent_dollars: 500,
            monthly_income_dollars: 3_000,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ScrieDrieMode::ViolationRentBurdenBelowOneThirdThreshold
        );
    }

    #[test]
    fn rent_burden_at_exactly_one_third_violation() {
        let input = Input {
            monthly_rent_dollars: 1_000,
            monthly_income_dollars: 3_000,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ScrieDrieMode::ViolationRentBurdenBelowOneThirdThreshold
        );
    }

    #[test]
    fn rent_burden_above_one_third_compliant() {
        let input = Input {
            monthly_rent_dollars: 1_100,
            monthly_income_dollars: 3_000,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ScrieDrieMode::CompliantScrieFrozenRentAndPropertyTaxAbatementApplied
        );
    }

    #[test]
    fn renewal_lapsed_violation() {
        let input = Input {
            months_since_last_renewal: 30,
            renewal_application_submitted_within_window: false,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ScrieDrieMode::ViolationRenewalNotSubmittedExpirationLapsed
        );
    }

    #[test]
    fn renewal_within_window_compliant() {
        let input = Input {
            months_since_last_renewal: 23,
            renewal_application_submitted_within_window: true,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ScrieDrieMode::CompliantRenewalSubmittedWithin2YearWindow
        );
    }

    #[test]
    fn landlord_charged_post_freeze_increases_violation() {
        let input = Input {
            landlord_charged_post_freeze_increases: true,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ScrieDrieMode::ViolationLandlordChargedTenantPostFreezeIncreases
        );
    }

    #[test]
    fn landlord_failed_to_apply_abatement_violation() {
        let input = Input {
            landlord_applied_property_tax_abatement: false,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ScrieDrieMode::ViolationLandlordFailedToReflectPropertyTaxAbatement
        );
    }

    #[test]
    fn drie_with_va_disability_compensation_compliant() {
        let input = Input {
            program: ScrieDrieProgram::DrieDisability,
            tenant_age_years: 40,
            federal_disability_benefit: FederalDisabilityBenefit::VaDisabilityCompensation,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ScrieDrieMode::CompliantDrieFrozenRentAndPropertyTaxAbatementApplied
        );
    }

    #[test]
    fn drie_with_medicaid_disability_compliant() {
        let input = Input {
            program: ScrieDrieProgram::DrieDisability,
            tenant_age_years: 45,
            federal_disability_benefit:
                FederalDisabilityBenefit::MedicaidBasedOnDisabilityDetermination,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ScrieDrieMode::CompliantDrieFrozenRentAndPropertyTaxAbatementApplied
        );
    }

    #[test]
    fn mitchell_lama_compliant() {
        let input = Input {
            regulated_unit_type: RegulatedUnitType::MitchellLamaRentRegulated,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ScrieDrieMode::CompliantScrieFrozenRentAndPropertyTaxAbatementApplied
        );
    }

    #[test]
    fn hdfc_coop_compliant() {
        let input = Input {
            regulated_unit_type: RegulatedUnitType::HdfcCoopRentRegulated,
            ..baseline_scrie_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            ScrieDrieMode::CompliantScrieFrozenRentAndPropertyTaxAbatementApplied
        );
    }

    #[test]
    fn citations_pin_local_laws_and_admin_code() {
        let result = check(&baseline_scrie_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("NYC Admin Code § 26-509"));
        assert!(joined.contains("Local Law 6 of 1986"));
        assert!(joined.contains("§ 26-509.1"));
        assert!(joined.contains("Local Law 76 of 2005"));
        assert!(joined.contains("$50,000"));
        assert!(joined.contains("$75,000"));
        assert!(joined.contains("S01457"));
        assert!(joined.contains("60 days"));
        assert!(joined.contains("Form RP-467-C"));
        assert!(joined.contains("ACCESS NYC"));
    }

    #[test]
    fn constant_pin_program_thresholds() {
        assert_eq!(SCRIE_SENIOR_MIN_AGE, 62);
        assert_eq!(DRIE_DISABILITY_MIN_AGE, 18);
        assert_eq!(SCRIE_DRIE_INCOME_LIMIT_DOLLARS_2025, 50_000);
        assert_eq!(SCRIE_DRIE_PROPOSED_INCOME_LIMIT_DOLLARS_2026, 75_000);
        assert_eq!(SCRIE_DRIE_RENT_BURDEN_THRESHOLD_BASIS_POINTS, 3_333);
        assert_eq!(SCRIE_DRIE_RENEWAL_INTERVAL_MONTHS, 24);
        assert_eq!(SCRIE_DRIE_RENEWAL_NOTICE_DAYS_BEFORE_EXPIRATION, 60);
        assert_eq!(SCRIE_LOCAL_LAW_YEAR, 1986);
        assert_eq!(SCRIE_LOCAL_LAW_NUMBER, 6);
        assert_eq!(DRIE_LOCAL_LAW_YEAR, 2005);
        assert_eq!(DRIE_LOCAL_LAW_NUMBER, 76);
        assert_eq!(NY_SENATE_S01457_2026_PROPOSED_INCOME_INCREASE_BILL, 1457);
    }
}
