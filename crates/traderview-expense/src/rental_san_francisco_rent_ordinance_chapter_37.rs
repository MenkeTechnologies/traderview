//! San Francisco Residential Rent Stabilization and
//! Arbitration Ordinance (SF Admin Code Chapter 37)
//! Compliance Module — one of the strictest and most-
//! litigated rent-control regimes in the United States,
//! enacted in 1979 and continuously amended ever since.
//!
//! Pure-compute check for landlord compliance with the
//! San Francisco Residential Rent Stabilization and
//! Arbitration Ordinance codified at SF Administrative
//! Code Chapter 37 (Ordinance 276-79 enacted 1979 by the
//! San Francisco Board of Supervisors after Proposition R).
//! The ordinance combines (a) an annual rent-increase cap
//! tied to the SF Consumer Price Index AND (b) a 16-ground
//! just-cause eviction regime under § 37.9(a), administered
//! by the San Francisco Residential Rent Stabilization and
//! Arbitration Board (the "Rent Board"). Coverage is
//! shaped by the **certificate-of-occupancy cutoff of
//! June 13, 1979** and by the **Costa-Hawkins Rental
//! Housing Act of 1995** (California state-level vacancy
//! decontrol overlay).
//!
//! Web research (verified 2026-06-03):
//! - **Enactment**: SF Board of Supervisors Ordinance
//!   **276-79** enacted in **1979** following Proposition
//!   R; codified at SF Administrative Code Chapter 37
//!   ("Residential Rent Stabilization and Arbitration
//!   Ordinance") ([Foundations of Law and Society — Rent
//!   Control: Chapter 37 of the San Francisco
//!   Administrative Code](https://foundationsoflawandsociety.wordpress.com/2016/12/09/rent-control-chapter-37-of-the-san-francisco-administrative-code/);
//!   [Tenant Law Group — San Francisco Rent Ordinance](https://tenantlawgroupsf.com/san-francisco-rent-ordinance/);
//!   [Costa-Hawkins.com — San Francisco Rent Stabilization
//!   and Arbitration Ordinance](https://costa-hawkins.com/san-francisco-rent-stabilization-and-arbitration-ordinance/);
//!   [San Francisco Tenants Union — Rent Control](https://sftu.org/rent-control/)).
//! - **Certificate-of-Occupancy Cutoff (June 13, 1979)**:
//!   units in buildings receiving their first certificate
//!   of occupancy **AFTER JUNE 13, 1979** are EXEMPT from
//!   rent-price control under SF Admin Code § 37.2(r),
//!   except for limited categories under § 37.3(d) and
//!   foreclosed units under § 37.9D ([Costa-Hawkins.com —
//!   SF Rent Stabilization](https://costa-hawkins.com/san-francisco-rent-stabilization-and-arbitration-ordinance/)).
//!   These post-1979 buildings remain subject to the
//!   16-ground just-cause eviction requirement under
//!   § 37.9(a).
//! - **Costa-Hawkins Rental Housing Act (1995 California
//!   State Law)**: state-level vacancy decontrol overlay
//!   that EXEMPTS (1) single-family homes; (2) condominium
//!   units; AND (3) units with first certificate of
//!   occupancy after February 1, 1995 (or the SF post-
//!   June-13-1979 trigger, whichever is later) from local
//!   rent-price control on TENANCY INITIATION. SF Admin
//!   Code § 37.3(d) and § 37.3(g) implement Costa-Hawkins
//!   carve-outs at the local level.
//! - **Annual Allowable Rent Increase (§ 37.3(a))**:
//!   landlords may impose annual rent increases of **60 %
//!   of the published Consumer Price Index increase** for
//!   the San Francisco-Oakland-San Jose CPI-W (Bureau of
//!   Labor Statistics), subject to an **ABSOLUTE CEILING
//!   of 7 PERCENT of base rent**. The annual cap is
//!   published each year by the SF Rent Board ([Tenant Law
//!   Group — How Does Rent Control Work for Tenants in
//!   San Francisco?](https://tenantlawgroupsf.com/how-does-rent-control-work-for-tenants-in-san-francisco/)).
//! - **16 Just Cause Grounds under § 37.9(a)** ([San
//!   Francisco Tenants Union — Just Causes for Eviction
//!   Under the SF Rent Ordinance](https://sftu.org/justcauses/);
//!   [SF.gov — Sec. 37.9 - Evictions](https://www.sf.gov/information--sec-379-evictions);
//!   [SF Admin Code § 37.9](https://codelibrary.amlegal.com/codes/san_francisco/latest/sf_admin/0-0-0-16273)):
//!   1. Non-payment of rent, habitual late payment, or
//!      bounced checks (§ 37.9(a)(1));
//!   2. Breach of rental agreement terms not corrected
//!      after written notice (§ 37.9(a)(2));
//!   3. Nuisance, substantial damage, or interference with
//!      comfort/safety (§ 37.9(a)(3));
//!   4. Illegal unit use (§ 37.9(a)(4));
//!   5. Tenant refusal to execute lease extension on
//!      materially identical terms (§ 37.9(a)(5));
//!   6. Tenant denial of landlord access after written
//!      notice (§ 37.9(a)(6));
//!   7. Unapproved subtenant remaining as sole occupant
//!      (§ 37.9(a)(7));
//!   8. Owner move-in or close relative occupancy
//!      (§ 37.9(a)(8));
//!   9. Condo conversion sale with senior/disabled tenant
//!      protections (§ 37.9(a)(9));
//!  10. Demolition or removal from housing use
//!      (§ 37.9(a)(10));
//!  11. Capital improvement / rehabilitation with permits
//!      (§ 37.9(a)(11));
//!  12. Substantial building rehabilitation when
//!      uninhabitable (§ 37.9(a)(12));
//!  13. Ellis Act withdrawal from rental housing
//!      (§ 37.9(a)(13));
//!  14. Lead abatement requiring temporary unit removal
//!      (§ 37.9(a)(14));
//!  15. Demolition under development agreement per SF
//!      Admin Code Chapter 56 (§ 37.9(a)(15));
//!  16. Development project demolition under Planning
//!      Code Section 317 (§ 37.9(a)(16)).
//! - **Notice-of-Termination Filing Requirement
//!   (§ 37.9(c))**: landlord must file a copy of any
//!   termination notice with the Rent Board within 10 days
//!   of service on the tenant; failure to file invalidates
//!   the termination notice.
//! - **§ 37.9F Circumvention of Tenant Protections**:
//!   prohibits landlord acts that have the effect of
//!   circumventing the just-cause eviction regime
//!   (harassment, constructive eviction, threats, refusal
//!   to accept rent, etc.); separate civil cause of
//!   action with attorney fees and treble damages for
//!   willful conduct ([SF Admin Code § 37.9F](https://codelibrary.amlegal.com/codes/san_francisco/latest/sf_admin/0-0-0-64313)).
//! - **Capital Improvement Passthrough Cap (§ 37.7)**:
//!   landlords may pass through capital improvement costs
//!   ranging from **50 % to 100 %** of cost depending on
//!   property size, subject to **ANNUAL CAPS ranging from
//!   5 % to 10 % of base rent** (or $30 minimum); requires
//!   Rent Board petition + approval.
//! - **Enforcement**: SF Rent Board investigates ordinance
//!   complaints and adjudicates rent-increase petitions,
//!   tenant petitions for decreased housing services, and
//!   capital improvement passthrough requests; civil
//!   private right of action under § 37.10B with statutory
//!   damages, treble damages for willful violations, and
//!   reasonable attorney's fees.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SF_RENT_ORDINANCE_ENACTMENT_YEAR: u32 = 1979;
pub const SF_RENT_ORDINANCE_ENACTMENT_ORDINANCE_NUMBER: u32 = 276;
pub const SF_RENT_ORDINANCE_COSTA_HAWKINS_ENACTMENT_YEAR: u32 = 1995;
pub const SF_RENT_ORDINANCE_CERTIFICATE_OCCUPANCY_CUTOFF_YEAR: u32 = 1979;
pub const SF_RENT_ORDINANCE_CERTIFICATE_OCCUPANCY_CUTOFF_MONTH: u32 = 6;
pub const SF_RENT_ORDINANCE_CERTIFICATE_OCCUPANCY_CUTOFF_DAY: u32 = 13;
pub const SF_RENT_ORDINANCE_CPI_PERCENTAGE_BASIS_POINTS: u64 = 6_000;
pub const SF_RENT_ORDINANCE_ANNUAL_INCREASE_CEILING_BASIS_POINTS: u64 = 700;
pub const SF_RENT_ORDINANCE_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const SF_RENT_ORDINANCE_NUMBER_OF_JUST_CAUSE_GROUNDS: u32 = 16;
pub const SF_RENT_ORDINANCE_CAPITAL_IMPROVEMENT_PASSTHROUGH_ANNUAL_CAP_HIGH_BASIS_POINTS: u64 = 1_000;
pub const SF_RENT_ORDINANCE_CAPITAL_IMPROVEMENT_PASSTHROUGH_ANNUAL_CAP_LOW_BASIS_POINTS: u64 = 500;
pub const SF_RENT_ORDINANCE_NOTICE_FILING_DAYS: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyJurisdiction {
    WithinSanFranciscoCityAndCounty,
    OutsideSanFranciscoCityAndCounty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CertificateOfOccupancyDateStatus {
    IssuedOnOrBeforeJune13_1979CoveredByRentPriceControl,
    IssuedAfterJune13_1979ExemptFromRentPriceControlButCoveredByJustCause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitType {
    StandardRentControlledApartment,
    SingleFamilyHomeCostaHawkinsApplies,
    CondominiumUnitCostaHawkinsApplies,
    NonResidentialUnitExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    AnnualRentIncrease,
    JustCauseEviction,
    OwnerMoveInUnderSection379A8,
    EllisActWithdrawalUnderSection379A13,
    CapitalImprovementPassthroughUnderSection377,
    NoticeOfTerminationFilingWithRentBoard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JustCauseGroundAsserted {
    NonPaymentOfRentOrHabitualLatePaymentSection379A1,
    BreachOfRentalAgreementSection379A2,
    NuisanceOrSubstantialDamageSection379A3,
    IllegalUseSection379A4,
    TenantRefusedLeaseExtensionSection379A5,
    TenantDeniedLandlordAccessSection379A6,
    UnapprovedSubtenantSoleOccupantSection379A7,
    OwnerOrRelativeMoveInSection379A8,
    CondoConversionSaleSection379A9,
    DemolitionOrRemovalFromHousingUseSection379A10,
    CapitalImprovementRehabilitationWithPermitsSection379A11,
    SubstantialRehabilitationWhenUninhabitableSection379A12,
    EllisActWithdrawalSection379A13,
    LeadAbatementTemporaryRemovalSection379A14,
    DemolitionUnderChapter56DevelopmentAgreementSection379A15,
    DevelopmentProjectDemolitionPlanningCode317Section379A16,
    NoJustCauseAsserted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SfRentOrdinanceMode {
    NotApplicablePropertyOutsideSanFrancisco,
    NotApplicableNonResidentialUnit,
    NotApplicablePostJune13_1979CertificateOfOccupancyExemptFromRentPriceControl,
    NotApplicableCostaHawkinsSingleFamilyOrCondoVacancyDecontrolApplies,
    CompliantAnnualRentIncreaseAtOrBelow60PctOfCpiCapped7Pct,
    CompliantJustCauseEvictionUnderOneOfSixteenSection379aGrounds,
    CompliantOwnerMoveInUnderSection379A8WithProperNoticeAndFilingWithRentBoard,
    CompliantEllisActWithdrawalUnderSection379A13,
    CompliantCapitalImprovementPassthroughWithinAnnualCap,
    CompliantNoticeOfTerminationFiledWithRentBoardWithin10Days,
    ViolationAnnualRentIncreaseExceeds60PctOfCpiOr7PctAbsoluteCeiling,
    ViolationEvictionWithoutOneOfSixteenSection379aJustCauseGrounds,
    ViolationOwnerMoveInWithoutRentBoardNoticeFilingUnderSection379C,
    ViolationCapitalImprovementPassthroughExceedsAnnualCap,
    ViolationNoticeOfTerminationNotFiledWithRentBoardWithin10Days,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_jurisdiction: PropertyJurisdiction,
    pub certificate_of_occupancy_date_status: CertificateOfOccupancyDateStatus,
    pub unit_type: UnitType,
    pub compliance_aspect: ComplianceAspect,
    pub proposed_annual_rent_increase_basis_points: u64,
    pub published_san_francisco_cpi_increase_basis_points: u64,
    pub just_cause_ground_asserted: JustCauseGroundAsserted,
    pub owner_move_in_notice_filed_with_rent_board: bool,
    pub capital_improvement_passthrough_basis_points: u64,
    pub notice_of_termination_filing_days_after_service: u32,
    pub notice_of_termination_filed_with_rent_board: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: SfRentOrdinanceMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub statutory_annual_rent_increase_cap_basis_points: u64,
    pub statutory_capital_improvement_passthrough_cap_basis_points: u64,
}

pub type RentalSanFranciscoRentOrdinanceChapter37Input = Input;
pub type RentalSanFranciscoRentOrdinanceChapter37Output = Output;
pub type RentalSanFranciscoRentOrdinanceChapter37Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "San Francisco Residential Rent Stabilization and Arbitration Ordinance — Ordinance 276-79 enacted by SF Board of Supervisors in 1979 following Proposition R; codified at SF Administrative Code Chapter 37".to_string(),
        "Certificate-of-Occupancy Cutoff (§ 37.2(r)) — units in buildings receiving their first certificate of occupancy AFTER JUNE 13, 1979 are EXEMPT from rent-price control, except for limited categories under § 37.3(d) and foreclosed units under § 37.9D; these post-1979 buildings remain subject to the 16-ground just-cause eviction requirement under § 37.9(a)".to_string(),
        "Costa-Hawkins Rental Housing Act of 1995 (California state law) — vacancy decontrol overlay exempting (1) single-family homes, (2) condominium units, and (3) units with first certificate of occupancy after February 1, 1995 (or the SF post-June-13-1979 trigger, whichever is later) from local rent-price control on tenancy initiation; SF Admin Code §§ 37.3(d) and 37.3(g) implement Costa-Hawkins carve-outs locally".to_string(),
        "Annual Allowable Rent Increase (§ 37.3(a)) — landlords may impose annual rent increases of 60 % of the published Consumer Price Index increase for San Francisco-Oakland-San Jose CPI-W, subject to an ABSOLUTE CEILING of 7 PERCENT of base rent; annual cap published by SF Rent Board".to_string(),
        "16 Just Cause Grounds under § 37.9(a) — (1) non-payment/habitual late payment; (2) breach not cured; (3) nuisance/damage; (4) illegal use; (5) refused lease extension; (6) denied landlord access; (7) unapproved subtenant; (8) owner/relative move-in; (9) condo conversion sale; (10) demolition/removal; (11) capital improvement/rehab with permits; (12) substantial rehab when uninhabitable; (13) Ellis Act withdrawal; (14) lead abatement temporary removal; (15) demolition under Chapter 56 development agreement; (16) Planning Code § 317 development project demolition".to_string(),
        "Notice-of-Termination Filing Requirement (§ 37.9(c)) — landlord must file copy of any termination notice with Rent Board within 10 DAYS of service on tenant; failure to file invalidates the termination notice".to_string(),
        "§ 37.9F Circumvention of Tenant Protections — prohibits landlord acts that have the effect of circumventing just-cause eviction regime (harassment, constructive eviction, threats, refusal to accept rent, etc.); separate civil cause of action with attorney fees and treble damages for willful conduct".to_string(),
        "Capital Improvement Passthrough Cap (§ 37.7) — landlords may pass through capital improvement costs ranging from 50 % to 100 % of cost depending on property size, subject to ANNUAL CAPS of 5 % to 10 % of base rent (or $30 minimum); requires Rent Board petition + approval".to_string(),
        "Enforcement — SF Residential Rent Stabilization and Arbitration Board (the Rent Board) investigates complaints and adjudicates rent-increase petitions, tenant petitions for decreased housing services, and capital improvement passthrough requests; civil private right of action under § 37.10B with statutory damages, treble damages for willful violations, and reasonable attorney's fees".to_string(),
        "SF Administrative Code Chapter 37 — primary statutory text".to_string(),
        "San Francisco Tenants Union — Just Causes for Eviction Under the SF Rent Ordinance".to_string(),
        "Costa-Hawkins.com — San Francisco Rent Stabilization and Arbitration Ordinance practitioner guide".to_string(),
        "Tenant Law Group — San Francisco Rent Ordinance".to_string(),
        "SF.gov — Sec. 37.9 Evictions program page".to_string(),
        "Foundations of Law and Society — Rent Control: Chapter 37 of the San Francisco Administrative Code historical commentary".to_string(),
    ];

    if input.property_jurisdiction == PropertyJurisdiction::OutsideSanFranciscoCityAndCounty {
        return Output {
            mode: SfRentOrdinanceMode::NotApplicablePropertyOutsideSanFrancisco,
            statutory_basis: "Property outside SF city and county; SF Admin Code Chapter 37 inapplicable".to_string(),
            notes: "NOT APPLICABLE: property outside San Francisco city and county; SF Residential Rent Stabilization and Arbitration Ordinance (SF Admin Code Chapter 37) inapplicable.".to_string(),
            citations,
            statutory_annual_rent_increase_cap_basis_points: 0,
            statutory_capital_improvement_passthrough_cap_basis_points: 0,
        };
    }

    if input.unit_type == UnitType::NonResidentialUnitExempt {
        return Output {
            mode: SfRentOrdinanceMode::NotApplicableNonResidentialUnit,
            statutory_basis: "SF Admin Code Chapter 37 applies only to residential units; non-residential units exempt".to_string(),
            notes: "NOT APPLICABLE: unit is non-residential; SF Admin Code Chapter 37 applies only to residential rental units.".to_string(),
            citations,
            statutory_annual_rent_increase_cap_basis_points: 0,
            statutory_capital_improvement_passthrough_cap_basis_points: 0,
        };
    }

    let rent_increase_cap_from_cpi = u128::from(input.published_san_francisco_cpi_increase_basis_points)
        .saturating_mul(u128::from(SF_RENT_ORDINANCE_CPI_PERCENTAGE_BASIS_POINTS))
        .checked_div(u128::from(SF_RENT_ORDINANCE_BASIS_POINT_DENOMINATOR))
        .unwrap_or(0)
        .min(u128::from(u64::MAX)) as u64;
    let annual_rent_increase_cap_basis_points =
        rent_increase_cap_from_cpi.min(SF_RENT_ORDINANCE_ANNUAL_INCREASE_CEILING_BASIS_POINTS);

    if input.compliance_aspect == ComplianceAspect::AnnualRentIncrease {
        let costa_hawkins_applies = matches!(
            input.unit_type,
            UnitType::SingleFamilyHomeCostaHawkinsApplies | UnitType::CondominiumUnitCostaHawkinsApplies
        );
        if costa_hawkins_applies {
            return Output {
                mode: SfRentOrdinanceMode::NotApplicableCostaHawkinsSingleFamilyOrCondoVacancyDecontrolApplies,
                statutory_basis: "Costa-Hawkins Rental Housing Act of 1995 (CA state law) + SF Admin Code §§ 37.3(d) and 37.3(g) — single-family homes and condominiums exempt from local rent-price control on tenancy initiation".to_string(),
                notes: "NOT APPLICABLE: unit is a single-family home or condominium subject to Costa-Hawkins vacancy decontrol; SF rent-price control does not apply to rent increases on a new tenancy (just-cause eviction protection under SF Admin Code Chapter 37 may still apply within an ongoing tenancy).".to_string(),
                citations,
                statutory_annual_rent_increase_cap_basis_points: annual_rent_increase_cap_basis_points,
                statutory_capital_improvement_passthrough_cap_basis_points: 0,
            };
        }

        if input.certificate_of_occupancy_date_status
            == CertificateOfOccupancyDateStatus::IssuedAfterJune13_1979ExemptFromRentPriceControlButCoveredByJustCause
        {
            return Output {
                mode: SfRentOrdinanceMode::NotApplicablePostJune13_1979CertificateOfOccupancyExemptFromRentPriceControl,
                statutory_basis: "SF Admin Code § 37.2(r) — certificate of occupancy issued after June 13, 1979 exempt from rent-price control".to_string(),
                notes: "NOT APPLICABLE: certificate of occupancy issued after June 13, 1979; building exempt from rent-price control under § 37.2(r); 16-ground just-cause eviction requirement under § 37.9(a) continues to apply.".to_string(),
                citations,
                statutory_annual_rent_increase_cap_basis_points: 0,
                statutory_capital_improvement_passthrough_cap_basis_points: 0,
            };
        }

        if input.proposed_annual_rent_increase_basis_points > annual_rent_increase_cap_basis_points {
            return Output {
                mode: SfRentOrdinanceMode::ViolationAnnualRentIncreaseExceeds60PctOfCpiOr7PctAbsoluteCeiling,
                statutory_basis: "SF Admin Code § 37.3(a) — annual rent increase capped at 60 % of CPI subject to 7 % absolute ceiling".to_string(),
                notes: format!(
                    "VIOLATION: proposed annual rent increase of {} basis points exceeds the statutory cap of {} basis points (= LESSER of 60 % × CPI ({} bps × 60% = {} bps) OR 7 % absolute ceiling ({} bps)); tenant may petition Rent Board for rollback.",
                    input.proposed_annual_rent_increase_basis_points,
                    annual_rent_increase_cap_basis_points,
                    input.published_san_francisco_cpi_increase_basis_points,
                    rent_increase_cap_from_cpi,
                    SF_RENT_ORDINANCE_ANNUAL_INCREASE_CEILING_BASIS_POINTS
                ),
                citations,
                statutory_annual_rent_increase_cap_basis_points: annual_rent_increase_cap_basis_points,
                statutory_capital_improvement_passthrough_cap_basis_points: 0,
            };
        }

        return Output {
            mode: SfRentOrdinanceMode::CompliantAnnualRentIncreaseAtOrBelow60PctOfCpiCapped7Pct,
            statutory_basis: "SF Admin Code § 37.3(a) — annual rent increase within 60 % × CPI / 7 % absolute ceiling".to_string(),
            notes: format!(
                "COMPLIANT: proposed annual rent increase of {} basis points is within the SF Rent Board's annual cap of {} basis points (LESSER of 60 % × CPI ({} bps × 60% = {} bps) OR 7 % absolute ceiling ({} bps)).",
                input.proposed_annual_rent_increase_basis_points,
                annual_rent_increase_cap_basis_points,
                input.published_san_francisco_cpi_increase_basis_points,
                rent_increase_cap_from_cpi,
                SF_RENT_ORDINANCE_ANNUAL_INCREASE_CEILING_BASIS_POINTS
            ),
            citations,
            statutory_annual_rent_increase_cap_basis_points: annual_rent_increase_cap_basis_points,
            statutory_capital_improvement_passthrough_cap_basis_points: 0,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::JustCauseEviction => {
            if input.just_cause_ground_asserted == JustCauseGroundAsserted::NoJustCauseAsserted {
                Output {
                    mode: SfRentOrdinanceMode::ViolationEvictionWithoutOneOfSixteenSection379aJustCauseGrounds,
                    statutory_basis: "SF Admin Code § 37.9(a) — eviction prohibited without one of the 16 enumerated just-cause grounds".to_string(),
                    notes: "VIOLATION: landlord served termination notice without asserting any of the 16 enumerated just-cause grounds under SF Admin Code § 37.9(a); termination notice unenforceable; tenant may assert as affirmative defense in unlawful detainer.".to_string(),
                    citations,
                    statutory_annual_rent_increase_cap_basis_points: annual_rent_increase_cap_basis_points,
                    statutory_capital_improvement_passthrough_cap_basis_points: 0,
                }
            } else {
                Output {
                    mode: SfRentOrdinanceMode::CompliantJustCauseEvictionUnderOneOfSixteenSection379aGrounds,
                    statutory_basis: "SF Admin Code § 37.9(a) — eviction under one of the 16 enumerated just-cause grounds".to_string(),
                    notes: format!(
                        "COMPLIANT: just-cause eviction asserted under one of the 16 enumerated grounds in SF Admin Code § 37.9(a) ({:?}); termination notice procedurally valid (separate § 37.9(c) Rent Board filing requirement must also be satisfied).",
                        input.just_cause_ground_asserted
                    ),
                    citations,
                    statutory_annual_rent_increase_cap_basis_points: annual_rent_increase_cap_basis_points,
                    statutory_capital_improvement_passthrough_cap_basis_points: 0,
                }
            }
        }
        ComplianceAspect::OwnerMoveInUnderSection379A8 => {
            if !input.owner_move_in_notice_filed_with_rent_board {
                Output {
                    mode: SfRentOrdinanceMode::ViolationOwnerMoveInWithoutRentBoardNoticeFilingUnderSection379C,
                    statutory_basis: "SF Admin Code § 37.9(a)(8) + § 37.9(c) — owner move-in eviction requires Rent Board notice filing within 10 days of service on tenant".to_string(),
                    notes: "VIOLATION: owner move-in eviction under § 37.9(a)(8) but landlord failed to file the termination notice with the Rent Board within 10 days of service on the tenant; failure to file invalidates the termination notice under § 37.9(c).".to_string(),
                    citations,
                    statutory_annual_rent_increase_cap_basis_points: annual_rent_increase_cap_basis_points,
                    statutory_capital_improvement_passthrough_cap_basis_points: 0,
                }
            } else {
                Output {
                    mode: SfRentOrdinanceMode::CompliantOwnerMoveInUnderSection379A8WithProperNoticeAndFilingWithRentBoard,
                    statutory_basis: "SF Admin Code § 37.9(a)(8) — owner / relative move-in eviction with Rent Board notice filing".to_string(),
                    notes: "COMPLIANT: owner / relative move-in eviction under § 37.9(a)(8) with Rent Board termination notice filing complete; additional substantive owner-move-in requirements (good-faith intent, continuous occupancy, relocation payments) apply separately.".to_string(),
                    citations,
                    statutory_annual_rent_increase_cap_basis_points: annual_rent_increase_cap_basis_points,
                    statutory_capital_improvement_passthrough_cap_basis_points: 0,
                }
            }
        }
        ComplianceAspect::EllisActWithdrawalUnderSection379A13 => Output {
            mode: SfRentOrdinanceMode::CompliantEllisActWithdrawalUnderSection379A13,
            statutory_basis: "SF Admin Code § 37.9(a)(13) — Ellis Act withdrawal from rental housing".to_string(),
            notes: "COMPLIANT: Ellis Act withdrawal asserted under § 37.9(a)(13); separate state-law procedural and timing requirements under California Government Code § 7060 et seq. (Ellis Act) apply, including 120-day or 1-year tenant notice depending on tenant status (senior/disabled tenants entitled to extended notice).".to_string(),
            citations,
            statutory_annual_rent_increase_cap_basis_points: annual_rent_increase_cap_basis_points,
            statutory_capital_improvement_passthrough_cap_basis_points: 0,
        },
        ComplianceAspect::CapitalImprovementPassthroughUnderSection377 => {
            if input.capital_improvement_passthrough_basis_points
                > SF_RENT_ORDINANCE_CAPITAL_IMPROVEMENT_PASSTHROUGH_ANNUAL_CAP_HIGH_BASIS_POINTS
            {
                Output {
                    mode: SfRentOrdinanceMode::ViolationCapitalImprovementPassthroughExceedsAnnualCap,
                    statutory_basis: "SF Admin Code § 37.7 — capital improvement passthrough annual cap of 5 % to 10 % of base rent depending on property size".to_string(),
                    notes: format!(
                        "VIOLATION: capital improvement passthrough of {} basis points exceeds the statutory upper annual cap of {} basis points (10 % of base rent); tenant may petition Rent Board for rollback.",
                        input.capital_improvement_passthrough_basis_points,
                        SF_RENT_ORDINANCE_CAPITAL_IMPROVEMENT_PASSTHROUGH_ANNUAL_CAP_HIGH_BASIS_POINTS
                    ),
                    citations,
                    statutory_annual_rent_increase_cap_basis_points: annual_rent_increase_cap_basis_points,
                    statutory_capital_improvement_passthrough_cap_basis_points:
                        SF_RENT_ORDINANCE_CAPITAL_IMPROVEMENT_PASSTHROUGH_ANNUAL_CAP_HIGH_BASIS_POINTS,
                }
            } else {
                Output {
                    mode: SfRentOrdinanceMode::CompliantCapitalImprovementPassthroughWithinAnnualCap,
                    statutory_basis: "SF Admin Code § 37.7 — capital improvement passthrough within annual cap".to_string(),
                    notes: format!(
                        "COMPLIANT: capital improvement passthrough of {} basis points is within the SF Rent Board's annual cap range (5 % to 10 % of base rent depending on property size); Rent Board petition + approval required.",
                        input.capital_improvement_passthrough_basis_points
                    ),
                    citations,
                    statutory_annual_rent_increase_cap_basis_points: annual_rent_increase_cap_basis_points,
                    statutory_capital_improvement_passthrough_cap_basis_points:
                        SF_RENT_ORDINANCE_CAPITAL_IMPROVEMENT_PASSTHROUGH_ANNUAL_CAP_HIGH_BASIS_POINTS,
                }
            }
        }
        ComplianceAspect::NoticeOfTerminationFilingWithRentBoard => {
            if !input.notice_of_termination_filed_with_rent_board
                || input.notice_of_termination_filing_days_after_service
                    > SF_RENT_ORDINANCE_NOTICE_FILING_DAYS
            {
                Output {
                    mode: SfRentOrdinanceMode::ViolationNoticeOfTerminationNotFiledWithRentBoardWithin10Days,
                    statutory_basis: "SF Admin Code § 37.9(c) — termination notice must be filed with Rent Board within 10 days of service on tenant".to_string(),
                    notes: format!(
                        "VIOLATION: termination notice filing status (filed = {}, days after service = {}) does not satisfy § 37.9(c) requirement of filing with Rent Board within 10 days of service; failure invalidates the termination notice.",
                        input.notice_of_termination_filed_with_rent_board,
                        input.notice_of_termination_filing_days_after_service
                    ),
                    citations,
                    statutory_annual_rent_increase_cap_basis_points: annual_rent_increase_cap_basis_points,
                    statutory_capital_improvement_passthrough_cap_basis_points: 0,
                }
            } else {
                Output {
                    mode: SfRentOrdinanceMode::CompliantNoticeOfTerminationFiledWithRentBoardWithin10Days,
                    statutory_basis: "SF Admin Code § 37.9(c) — termination notice filed with Rent Board within 10 days".to_string(),
                    notes: format!(
                        "COMPLIANT: termination notice filed with Rent Board {} days after service on tenant (≤ 10-day statutory window).",
                        input.notice_of_termination_filing_days_after_service
                    ),
                    citations,
                    statutory_annual_rent_increase_cap_basis_points: annual_rent_increase_cap_basis_points,
                    statutory_capital_improvement_passthrough_cap_basis_points: 0,
                }
            }
        }
        ComplianceAspect::AnnualRentIncrease => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_rent_increase_input() -> Input {
        Input {
            property_jurisdiction: PropertyJurisdiction::WithinSanFranciscoCityAndCounty,
            certificate_of_occupancy_date_status:
                CertificateOfOccupancyDateStatus::IssuedOnOrBeforeJune13_1979CoveredByRentPriceControl,
            unit_type: UnitType::StandardRentControlledApartment,
            compliance_aspect: ComplianceAspect::AnnualRentIncrease,
            proposed_annual_rent_increase_basis_points: 150,
            published_san_francisco_cpi_increase_basis_points: 300,
            just_cause_ground_asserted: JustCauseGroundAsserted::NoJustCauseAsserted,
            owner_move_in_notice_filed_with_rent_board: false,
            capital_improvement_passthrough_basis_points: 0,
            notice_of_termination_filing_days_after_service: 0,
            notice_of_termination_filed_with_rent_board: false,
        }
    }

    #[test]
    fn property_outside_sf_not_applicable() {
        let mut input = baseline_rent_increase_input();
        input.property_jurisdiction = PropertyJurisdiction::OutsideSanFranciscoCityAndCounty;
        let output = check(&input);
        assert_eq!(output.mode, SfRentOrdinanceMode::NotApplicablePropertyOutsideSanFrancisco);
    }

    #[test]
    fn non_residential_unit_not_applicable() {
        let mut input = baseline_rent_increase_input();
        input.unit_type = UnitType::NonResidentialUnitExempt;
        let output = check(&input);
        assert_eq!(output.mode, SfRentOrdinanceMode::NotApplicableNonResidentialUnit);
    }

    #[test]
    fn post_june_13_1979_certificate_of_occupancy_exempt() {
        let mut input = baseline_rent_increase_input();
        input.certificate_of_occupancy_date_status =
            CertificateOfOccupancyDateStatus::IssuedAfterJune13_1979ExemptFromRentPriceControlButCoveredByJustCause;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SfRentOrdinanceMode::NotApplicablePostJune13_1979CertificateOfOccupancyExemptFromRentPriceControl
        );
    }

    #[test]
    fn single_family_home_costa_hawkins_applies() {
        let mut input = baseline_rent_increase_input();
        input.unit_type = UnitType::SingleFamilyHomeCostaHawkinsApplies;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SfRentOrdinanceMode::NotApplicableCostaHawkinsSingleFamilyOrCondoVacancyDecontrolApplies
        );
    }

    #[test]
    fn condominium_costa_hawkins_applies() {
        let mut input = baseline_rent_increase_input();
        input.unit_type = UnitType::CondominiumUnitCostaHawkinsApplies;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SfRentOrdinanceMode::NotApplicableCostaHawkinsSingleFamilyOrCondoVacancyDecontrolApplies
        );
    }

    #[test]
    fn rent_increase_within_60pct_cpi_cap_compliant() {
        // CPI 3% (300 bps) × 60% = 180 bps; 7% absolute = 700 bps; cap = lesser = 180 bps
        let mut input = baseline_rent_increase_input();
        input.proposed_annual_rent_increase_basis_points = 150;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SfRentOrdinanceMode::CompliantAnnualRentIncreaseAtOrBelow60PctOfCpiCapped7Pct
        );
        assert_eq!(output.statutory_annual_rent_increase_cap_basis_points, 180);
    }

    #[test]
    fn rent_increase_at_exactly_cpi_cap_compliant() {
        let mut input = baseline_rent_increase_input();
        input.proposed_annual_rent_increase_basis_points = 180; // = 60% × 300
        let output = check(&input);
        assert_eq!(
            output.mode,
            SfRentOrdinanceMode::CompliantAnnualRentIncreaseAtOrBelow60PctOfCpiCapped7Pct
        );
    }

    #[test]
    fn rent_increase_above_60pct_cpi_violation() {
        let mut input = baseline_rent_increase_input();
        input.proposed_annual_rent_increase_basis_points = 250; // > 180 bps cap
        let output = check(&input);
        assert_eq!(
            output.mode,
            SfRentOrdinanceMode::ViolationAnnualRentIncreaseExceeds60PctOfCpiOr7PctAbsoluteCeiling
        );
    }

    #[test]
    fn rent_increase_at_7pct_absolute_ceiling_compliant_with_high_cpi() {
        // CPI 15% (1500 bps) × 60% = 900 bps; absolute ceiling = 700 bps; cap = lesser = 700 bps
        let mut input = baseline_rent_increase_input();
        input.published_san_francisco_cpi_increase_basis_points = 1_500;
        input.proposed_annual_rent_increase_basis_points = 700;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SfRentOrdinanceMode::CompliantAnnualRentIncreaseAtOrBelow60PctOfCpiCapped7Pct
        );
        assert_eq!(output.statutory_annual_rent_increase_cap_basis_points, 700);
    }

    #[test]
    fn rent_increase_above_7pct_absolute_ceiling_violation() {
        let mut input = baseline_rent_increase_input();
        input.published_san_francisco_cpi_increase_basis_points = 1_500;
        input.proposed_annual_rent_increase_basis_points = 800; // > 700 bps ceiling
        let output = check(&input);
        assert_eq!(
            output.mode,
            SfRentOrdinanceMode::ViolationAnnualRentIncreaseExceeds60PctOfCpiOr7PctAbsoluteCeiling
        );
    }

    #[test]
    fn eviction_with_just_cause_compliant() {
        let mut input = baseline_rent_increase_input();
        input.compliance_aspect = ComplianceAspect::JustCauseEviction;
        input.just_cause_ground_asserted =
            JustCauseGroundAsserted::NonPaymentOfRentOrHabitualLatePaymentSection379A1;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SfRentOrdinanceMode::CompliantJustCauseEvictionUnderOneOfSixteenSection379aGrounds
        );
    }

    #[test]
    fn eviction_without_just_cause_violation() {
        let mut input = baseline_rent_increase_input();
        input.compliance_aspect = ComplianceAspect::JustCauseEviction;
        input.just_cause_ground_asserted = JustCauseGroundAsserted::NoJustCauseAsserted;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SfRentOrdinanceMode::ViolationEvictionWithoutOneOfSixteenSection379aJustCauseGrounds
        );
    }

    #[test]
    fn owner_move_in_with_rent_board_filing_compliant() {
        let mut input = baseline_rent_increase_input();
        input.compliance_aspect = ComplianceAspect::OwnerMoveInUnderSection379A8;
        input.owner_move_in_notice_filed_with_rent_board = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SfRentOrdinanceMode::CompliantOwnerMoveInUnderSection379A8WithProperNoticeAndFilingWithRentBoard
        );
    }

    #[test]
    fn owner_move_in_without_rent_board_filing_violation() {
        let mut input = baseline_rent_increase_input();
        input.compliance_aspect = ComplianceAspect::OwnerMoveInUnderSection379A8;
        input.owner_move_in_notice_filed_with_rent_board = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SfRentOrdinanceMode::ViolationOwnerMoveInWithoutRentBoardNoticeFilingUnderSection379C
        );
    }

    #[test]
    fn ellis_act_withdrawal_compliant() {
        let mut input = baseline_rent_increase_input();
        input.compliance_aspect = ComplianceAspect::EllisActWithdrawalUnderSection379A13;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SfRentOrdinanceMode::CompliantEllisActWithdrawalUnderSection379A13
        );
    }

    #[test]
    fn capital_improvement_within_cap_compliant() {
        let mut input = baseline_rent_increase_input();
        input.compliance_aspect = ComplianceAspect::CapitalImprovementPassthroughUnderSection377;
        input.capital_improvement_passthrough_basis_points = 500; // 5% within range
        let output = check(&input);
        assert_eq!(
            output.mode,
            SfRentOrdinanceMode::CompliantCapitalImprovementPassthroughWithinAnnualCap
        );
    }

    #[test]
    fn capital_improvement_exceeds_cap_violation() {
        let mut input = baseline_rent_increase_input();
        input.compliance_aspect = ComplianceAspect::CapitalImprovementPassthroughUnderSection377;
        input.capital_improvement_passthrough_basis_points = 1_200; // > 1000 bps cap
        let output = check(&input);
        assert_eq!(
            output.mode,
            SfRentOrdinanceMode::ViolationCapitalImprovementPassthroughExceedsAnnualCap
        );
    }

    #[test]
    fn termination_notice_filed_within_10_days_compliant() {
        let mut input = baseline_rent_increase_input();
        input.compliance_aspect = ComplianceAspect::NoticeOfTerminationFilingWithRentBoard;
        input.notice_of_termination_filed_with_rent_board = true;
        input.notice_of_termination_filing_days_after_service = 7;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SfRentOrdinanceMode::CompliantNoticeOfTerminationFiledWithRentBoardWithin10Days
        );
    }

    #[test]
    fn termination_notice_at_exactly_10_days_compliant_boundary() {
        let mut input = baseline_rent_increase_input();
        input.compliance_aspect = ComplianceAspect::NoticeOfTerminationFilingWithRentBoard;
        input.notice_of_termination_filed_with_rent_board = true;
        input.notice_of_termination_filing_days_after_service = 10;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SfRentOrdinanceMode::CompliantNoticeOfTerminationFiledWithRentBoardWithin10Days
        );
    }

    #[test]
    fn termination_notice_at_11_days_violation_boundary() {
        let mut input = baseline_rent_increase_input();
        input.compliance_aspect = ComplianceAspect::NoticeOfTerminationFilingWithRentBoard;
        input.notice_of_termination_filed_with_rent_board = true;
        input.notice_of_termination_filing_days_after_service = 11;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SfRentOrdinanceMode::ViolationNoticeOfTerminationNotFiledWithRentBoardWithin10Days
        );
    }

    #[test]
    fn termination_notice_not_filed_at_all_violation() {
        let mut input = baseline_rent_increase_input();
        input.compliance_aspect = ComplianceAspect::NoticeOfTerminationFilingWithRentBoard;
        input.notice_of_termination_filed_with_rent_board = false;
        input.notice_of_termination_filing_days_after_service = 0;
        let output = check(&input);
        assert_eq!(
            output.mode,
            SfRentOrdinanceMode::ViolationNoticeOfTerminationNotFiledWithRentBoardWithin10Days
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(SF_RENT_ORDINANCE_ENACTMENT_YEAR, 1979);
        assert_eq!(SF_RENT_ORDINANCE_ENACTMENT_ORDINANCE_NUMBER, 276);
        assert_eq!(SF_RENT_ORDINANCE_COSTA_HAWKINS_ENACTMENT_YEAR, 1995);
        assert_eq!(SF_RENT_ORDINANCE_CERTIFICATE_OCCUPANCY_CUTOFF_YEAR, 1979);
        assert_eq!(SF_RENT_ORDINANCE_CERTIFICATE_OCCUPANCY_CUTOFF_MONTH, 6);
        assert_eq!(SF_RENT_ORDINANCE_CERTIFICATE_OCCUPANCY_CUTOFF_DAY, 13);
        assert_eq!(SF_RENT_ORDINANCE_CPI_PERCENTAGE_BASIS_POINTS, 6_000);
        assert_eq!(SF_RENT_ORDINANCE_ANNUAL_INCREASE_CEILING_BASIS_POINTS, 700);
        assert_eq!(SF_RENT_ORDINANCE_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(SF_RENT_ORDINANCE_NUMBER_OF_JUST_CAUSE_GROUNDS, 16);
        assert_eq!(SF_RENT_ORDINANCE_CAPITAL_IMPROVEMENT_PASSTHROUGH_ANNUAL_CAP_HIGH_BASIS_POINTS, 1_000);
        assert_eq!(SF_RENT_ORDINANCE_CAPITAL_IMPROVEMENT_PASSTHROUGH_ANNUAL_CAP_LOW_BASIS_POINTS, 500);
        assert_eq!(SF_RENT_ORDINANCE_NOTICE_FILING_DAYS, 10);
    }

    #[test]
    fn citation_contains_landmarks() {
        let output = check(&baseline_rent_increase_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("Chapter 37"));
        assert!(joined.contains("1979"));
        assert!(joined.contains("JUNE 13, 1979"));
        assert!(joined.contains("Costa-Hawkins"));
        assert!(joined.contains("60 %"));
        assert!(joined.contains("7 PERCENT"));
        assert!(joined.contains("§ 37.9(a)"));
        assert!(joined.contains("§ 37.9(c)"));
        assert!(joined.contains("§ 37.9F"));
        assert!(joined.contains("Ellis Act"));
        assert!(joined.contains("§ 37.7"));
    }
}
