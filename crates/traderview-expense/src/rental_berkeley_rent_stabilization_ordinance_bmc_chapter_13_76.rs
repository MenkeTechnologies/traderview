//! Berkeley Rent Stabilization and Good Cause for Eviction
//! Ordinance (BMC Chapter 13.76) Compliance Module — second
//! oldest comprehensive California municipal rent-control
//! regime, enacted by voters of Berkeley in June 1980 as
//! Measure I (one year after SF Chapter 37, enacted 1979).
//!
//! Pure-compute check for landlord compliance with the
//! Berkeley Rent Stabilization and Good Cause for Eviction
//! Ordinance, codified at Berkeley Municipal Code (BMC)
//! Chapter 13.76, administered by the Berkeley Rent
//! Stabilization Board (the "Rent Board"). The ordinance
//! has FOUR PRIMARY COMPONENTS: (1) mandatory registration
//! of all covered rental units with the Rent Board; (2) rent
//! control via Annual General Adjustment (AGA) tied to CPI
//! and capped at 5 PERCENT under Measure BB; (3) eviction
//! protection through good-cause requirements under BMC
//! 13.76.130; AND (4) tenant's right to annual interest on
//! security deposits.
//!
//! Web research (verified 2026-06-03):
//! - **Enactment**: Berkeley voters approved the Rent
//!   Stabilization and Good Cause for Eviction Ordinance in
//!   **JUNE 1980** as Measure I; codified at BMC Chapter
//!   13.76; administered by the Berkeley Rent Stabilization
//!   Board (the Rent Board) ([Berkeley Rent Board — Rent
//!   Ordinance & Rent Board Regulations](https://rentboard.berkeleyca.gov/laws-regulations/rent-ordinance-rent-board-regulations);
//!   [Berkeley Property Owners Association — Laws & Ordinances](https://www.bpoa.org/laws-and-ordinances);
//!   [Berkeley Rent Board — Just Cause & Other Local
//!   Requirements](https://rentboard.berkeleyca.gov/rights-responsibilities/evictions/just-cause-other-local-requirements);
//!   [Berkeley Rent Stabilization Ordinance (BMC Chapter
//!   13.76) — full ordinance PDF (October 2021)](https://berkeleyca.gov/sites/default/files/2022-01/Rent%20Stabilization%20Ordinance_Oct%202021_0.pdf);
//!   [Berkeley Rent Board Ordinance No. 7950-NS — Rent
//!   Stabilization Ordinance text](https://rentboard.berkeleyca.gov/sites/default/files/documents/Rent%20Stabilization%20Ordinance_BMC%20Chapter%2013.76.pdf)).
//! - **Base Rent Ceiling**: upon adoption of BMC Chapter
//!   13.76, no landlord could charge rent in excess of the
//!   lawful rent actually due and payable on, or last
//!   preceding, **MAY 31, 1980**, in accordance with the
//!   Temporary Rent Stabilization Ordinance, No. 5212-N.S.,
//!   except as permitted by the Rent Board.
//! - **Annual General Adjustment (AGA)**: BMC § 13.76.110
//!   authorizes the Rent Board to set an annual rent
//!   increase tied to CPI. **Measure BB (2020) capped the
//!   AGA at 5 PERCENT** to prevent extraordinary single-year
//!   increases during high-inflation periods.
//! - **Just Cause for Eviction Grounds (BMC § 13.76.130)**:
//!   following a December 2024 amendment that removed one
//!   ground, the ordinance now provides **11 ENUMERATED
//!   GROUNDS** (previously 12). The grounds include
//!   non-payment of rent following a 3-day notice; material
//!   lease violations with written cure notice; substantial
//!   property damage with written repair demand; peace and
//!   quiet disturbance after written request to cease;
//!   denial of landlord entry after written notice; major
//!   repairs requiring temporary vacancy; demolition with
//!   valid permit obtained; owner or qualified-relative
//!   move-in (requires 90-day vacancy search in landlord's
//!   other Berkeley properties); owner / lessor reoccupancy
//!   as specified in rental agreement; refusal to vacate
//!   temporary housing after repairs completed; and Ellis
//!   Act withdrawal of all rental units from the housing
//!   market (cross-referenced from California Government
//!   Code § 7060 et seq.).
//! - **Owner Move-In 90-Day Vacancy Search Requirement**:
//!   before serving an OMI termination notice, the landlord
//!   must conduct a 90-day vacancy search among the
//!   landlord's other Berkeley rental properties to ensure
//!   no comparable vacant unit is available for the intended
//!   occupant; failure to perform this search invalidates
//!   the OMI termination.
//! - **Notice of Termination Filing Requirement**: landlord
//!   must file a copy of any termination notice with the
//!   Berkeley Rent Board within **3 BUSINESS DAYS** of
//!   service on the tenant; the notice must state the just
//!   cause, reference Rent Board counseling services, AND
//!   allege landlord's compliance with registration and
//!   habitability standards.
//! - **Non-Qualifying Grounds**: explicitly NOT valid bases
//!   for eviction: property sales, lease expiration, Section
//!   8 status changes, and foreclosure (the foreclosure
//!   carve-out is one of the strongest tenant protections in
//!   California municipal rent-control regimes).
//! - **Mandatory Registration of Rental Units**: BMC
//!   § 13.76.080 requires landlords of covered rental units
//!   to register every unit with the Rent Board annually
//!   and pay the per-unit registration fee; failure to
//!   register prevents the landlord from collecting rent
//!   increases authorized by the AGA.
//! - **Security Deposit Interest**: BMC § 13.76.070
//!   requires landlords to pay tenants annual interest on
//!   security deposits at a rate set by the Rent Board each
//!   year (typically tied to the average rate paid on bank
//!   passbook savings accounts).
//! - **Costa-Hawkins Rental Housing Act of 1995** (California
//!   state law): vacancy decontrol overlay that exempts (1)
//!   single-family homes, (2) condominium units, AND (3)
//!   units with first certificate of occupancy after
//!   February 1, 1995 from local rent-price control on
//!   tenancy initiation. Just-cause eviction protection
//!   continues to apply during the tenancy under BMC
//!   § 13.76.130.
//! - **Enforcement**: Berkeley Rent Board investigates
//!   complaints and adjudicates rent-increase petitions,
//!   tenant petitions for decreased housing services, and
//!   capital improvement passthrough requests; civil
//!   private right of action under BMC § 13.76.150 with
//!   statutory damages, treble damages for willful
//!   violations, and reasonable attorney's fees.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const BERKELEY_RENT_ORDINANCE_ENACTMENT_YEAR: u32 = 1980;
pub const BERKELEY_RENT_ORDINANCE_ENACTMENT_MONTH: u32 = 6;
pub const BERKELEY_RENT_ORDINANCE_BASE_RENT_CEILING_REFERENCE_YEAR: u32 = 1980;
pub const BERKELEY_RENT_ORDINANCE_BASE_RENT_CEILING_REFERENCE_MONTH: u32 = 5;
pub const BERKELEY_RENT_ORDINANCE_BASE_RENT_CEILING_REFERENCE_DAY: u32 = 31;
pub const BERKELEY_RENT_ORDINANCE_AGA_CAP_BASIS_POINTS: u64 = 500;
pub const BERKELEY_RENT_ORDINANCE_OMI_VACANCY_SEARCH_DAYS: u32 = 90;
pub const BERKELEY_RENT_ORDINANCE_NOTICE_OF_TERMINATION_FILING_BUSINESS_DAYS: u32 = 3;
pub const BERKELEY_RENT_ORDINANCE_NUMBER_OF_JUST_CAUSE_GROUNDS_POST_2024_AMENDMENT: u32 = 11;
pub const BERKELEY_RENT_ORDINANCE_NUMBER_OF_JUST_CAUSE_GROUNDS_PRE_2024_AMENDMENT: u32 = 12;
pub const BERKELEY_RENT_ORDINANCE_NON_PAYMENT_NOTICE_DAYS: u32 = 3;
pub const BERKELEY_RENT_ORDINANCE_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const BERKELEY_RENT_ORDINANCE_COSTA_HAWKINS_VACANCY_DECONTROL_YEAR: u32 = 1995;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyJurisdiction {
    WithinBerkeleyCityLimits,
    OutsideBerkeleyCityLimits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnitType {
    StandardRentControlledApartment,
    SingleFamilyHomeCostaHawkinsApplies,
    CondominiumUnitCostaHawkinsApplies,
    PostFebruary1_1995CertificateOfOccupancyCostaHawkinsApplies,
    NonResidentialUnitExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    AnnualGeneralAdjustmentRentIncrease,
    JustCauseEviction,
    OwnerMoveInUnderSection137613,
    NoticeOfTerminationFilingWithRentBoard,
    MandatoryRegistrationOfRentalUnits,
    SecurityDepositAnnualInterest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JustCauseGroundAsserted {
    NonPaymentOfRentAfter3DayNotice,
    MaterialLeaseViolationWithCureNotice,
    SubstantialPropertyDamageWithRepairDemand,
    PeaceAndQuietDisturbanceAfterWrittenRequest,
    DenialOfLandlordEntryAfterWrittenNotice,
    MajorRepairsRequiringTemporaryVacancy,
    DemolitionWithValidPermit,
    OwnerOrQualifiedRelativeMoveInWith90DayVacancySearch,
    OwnerOrLessorReoccupancyPerRentalAgreement,
    RefusalToVacateTemporaryHousingAfterRepairsCompleted,
    EllisActWithdrawalUnderCaliforniaGovCode7060,
    NoJustCauseAsserted,
    NonQualifyingGroundPropertySaleOrLeaseExpirationOrSection8OrForeclosure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BerkeleyRentOrdinanceMode {
    NotApplicablePropertyOutsideBerkeley,
    NotApplicableNonResidentialUnit,
    NotApplicableCostaHawkinsSingleFamilyOrCondoOrPost1995VacancyDecontrolApplies,
    CompliantAnnualGeneralAdjustmentAtOrBelowMeasureBb5PctCap,
    CompliantJustCauseEvictionUnderOneOfElevenSection137613Grounds,
    CompliantOwnerMoveInWith90DayVacancySearchAndRentBoardFiling,
    CompliantNoticeOfTerminationFiledWithRentBoardWithin3BusinessDays,
    CompliantRentalUnitRegisteredAndRegistrationFeePaid,
    CompliantSecurityDepositAnnualInterestPaidAtRentBoardSetRate,
    ViolationAnnualGeneralAdjustmentExceedsMeasureBb5PctCap,
    ViolationEvictionWithoutOneOfElevenJustCauseGrounds,
    ViolationEvictionAssertingNonQualifyingGroundPropertySaleOrLeaseExpirationOrSection8OrForeclosure,
    ViolationOwnerMoveInWithout90DayVacancySearchOfBerkeleyProperties,
    ViolationNoticeOfTerminationNotFiledWithRentBoardWithin3BusinessDays,
    ViolationRentalUnitNotRegisteredCannotCollectAga,
    ViolationSecurityDepositAnnualInterestNotPaid,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_jurisdiction: PropertyJurisdiction,
    pub unit_type: UnitType,
    pub compliance_aspect: ComplianceAspect,
    pub proposed_annual_general_adjustment_basis_points: u64,
    pub just_cause_ground_asserted: JustCauseGroundAsserted,
    pub owner_move_in_90_day_vacancy_search_performed: bool,
    pub notice_of_termination_filing_business_days_after_service: u32,
    pub notice_of_termination_filed_with_rent_board: bool,
    pub rental_unit_registered_and_fee_paid: bool,
    pub security_deposit_annual_interest_paid: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: BerkeleyRentOrdinanceMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub statutory_annual_general_adjustment_cap_basis_points: u64,
}

pub type RentalBerkeleyRentStabilizationOrdinanceBmcChapter1376Input = Input;
pub type RentalBerkeleyRentStabilizationOrdinanceBmcChapter1376Output = Output;
pub type RentalBerkeleyRentStabilizationOrdinanceBmcChapter1376Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Berkeley Rent Stabilization and Good Cause for Eviction Ordinance — adopted by voters of Berkeley in June 1980 as Measure I; codified at Berkeley Municipal Code Chapter 13.76; administered by the Berkeley Rent Stabilization Board (the Rent Board)".to_string(),
        "Base Rent Ceiling — upon adoption, no landlord could charge rent in excess of lawful rent actually due and payable on or last preceding MAY 31, 1980 under the Temporary Rent Stabilization Ordinance No. 5212-N.S., except as permitted by the Rent Board".to_string(),
        "Annual General Adjustment (AGA) under BMC § 13.76.110 — Rent Board sets annual rent increase tied to CPI; Measure BB (2020) capped AGA at 5 PERCENT to prevent extraordinary single-year increases during high-inflation periods".to_string(),
        "Just Cause for Eviction Grounds under BMC § 13.76.130 — 11 enumerated grounds after December 2024 amendment removed one ground (previously 12); includes non-payment after 3-day notice; material lease violation with cure notice; substantial damage with repair demand; peace/quiet disturbance after written request; denial of landlord entry after written notice; major repairs requiring temporary vacancy; demolition with valid permit; owner / qualified-relative move-in with 90-day Berkeley vacancy search; owner / lessor reoccupancy per rental agreement; refusal to vacate temporary housing after repairs completed; Ellis Act withdrawal under California Government Code § 7060".to_string(),
        "Owner Move-In 90-Day Vacancy Search Requirement — before serving OMI termination notice, landlord must conduct 90-day vacancy search among landlord's other Berkeley rental properties to ensure no comparable vacant unit is available for the intended occupant; failure invalidates the OMI termination".to_string(),
        "Notice-of-Termination Filing Requirement — landlord must file copy of any termination notice with Berkeley Rent Board within 3 BUSINESS DAYS of service on tenant; notice must state just cause, reference Rent Board counseling services, AND allege landlord's compliance with registration and habitability standards".to_string(),
        "Non-Qualifying Grounds — explicitly NOT valid bases for eviction: property sales, lease expiration, Section 8 status changes, and foreclosure; foreclosure carve-out is one of strongest tenant protections in CA municipal rent-control regimes".to_string(),
        "Mandatory Registration of Rental Units under BMC § 13.76.080 — landlords of covered rental units must register every unit with Rent Board annually and pay per-unit registration fee; failure to register prevents landlord from collecting AGA-authorized rent increases".to_string(),
        "Security Deposit Annual Interest under BMC § 13.76.070 — landlord must pay tenant annual interest on security deposit at rate set by Rent Board each year (typically tied to average rate paid on bank passbook savings accounts)".to_string(),
        "Costa-Hawkins Rental Housing Act of 1995 (California state law) — vacancy decontrol overlay exempts (1) single-family homes, (2) condominium units, AND (3) units with first certificate of occupancy after February 1, 1995 from local rent-price control on tenancy initiation; just-cause eviction protection continues to apply during tenancy under BMC § 13.76.130".to_string(),
        "Enforcement — Berkeley Rent Board investigates complaints and adjudicates rent-increase petitions, tenant petitions for decreased housing services, and capital improvement passthrough requests; civil private right of action under BMC § 13.76.150 with statutory damages, treble damages for willful violations, and reasonable attorney's fees".to_string(),
        "Berkeley Rent Board — Rent Ordinance & Rent Board Regulations program page".to_string(),
        "Berkeley Rent Board — Just Cause & Other Local Requirements".to_string(),
        "Berkeley Property Owners Association — Laws & Ordinances practitioner guide".to_string(),
        "Berkeley Rent Stabilization Ordinance (BMC Chapter 13.76) — full ordinance text".to_string(),
    ];

    if input.property_jurisdiction == PropertyJurisdiction::OutsideBerkeleyCityLimits {
        return Output {
            mode: BerkeleyRentOrdinanceMode::NotApplicablePropertyOutsideBerkeley,
            statutory_basis: "Property outside Berkeley city limits; BMC Chapter 13.76 inapplicable".to_string(),
            notes: "NOT APPLICABLE: property outside Berkeley city limits; Berkeley Rent Stabilization and Good Cause for Eviction Ordinance (BMC Chapter 13.76) inapplicable.".to_string(),
            citations,
            statutory_annual_general_adjustment_cap_basis_points: 0,
        };
    }

    if input.unit_type == UnitType::NonResidentialUnitExempt {
        return Output {
            mode: BerkeleyRentOrdinanceMode::NotApplicableNonResidentialUnit,
            statutory_basis: "BMC Chapter 13.76 applies only to residential units; non-residential exempt".to_string(),
            notes: "NOT APPLICABLE: unit is non-residential; BMC Chapter 13.76 applies only to residential rental units.".to_string(),
            citations,
            statutory_annual_general_adjustment_cap_basis_points: 0,
        };
    }

    let costa_hawkins_applies = matches!(
        input.unit_type,
        UnitType::SingleFamilyHomeCostaHawkinsApplies
            | UnitType::CondominiumUnitCostaHawkinsApplies
            | UnitType::PostFebruary1_1995CertificateOfOccupancyCostaHawkinsApplies
    );

    if costa_hawkins_applies
        && input.compliance_aspect == ComplianceAspect::AnnualGeneralAdjustmentRentIncrease
    {
        return Output {
            mode: BerkeleyRentOrdinanceMode::NotApplicableCostaHawkinsSingleFamilyOrCondoOrPost1995VacancyDecontrolApplies,
            statutory_basis: "Costa-Hawkins Rental Housing Act of 1995 — single-family home, condominium, or post-Feb-1-1995 certificate of occupancy exempt from local rent-price control on tenancy initiation".to_string(),
            notes: "NOT APPLICABLE: unit is a single-family home, condominium, or building with first certificate of occupancy after February 1, 1995; Costa-Hawkins vacancy decontrol applies; Berkeley AGA does not apply to rent-increase setting on a new tenancy (just-cause eviction protection under BMC § 13.76.130 continues to apply during the ongoing tenancy).".to_string(),
            citations,
            statutory_annual_general_adjustment_cap_basis_points: BERKELEY_RENT_ORDINANCE_AGA_CAP_BASIS_POINTS,
        };
    }

    match input.compliance_aspect {
        ComplianceAspect::AnnualGeneralAdjustmentRentIncrease => {
            if input.proposed_annual_general_adjustment_basis_points
                > BERKELEY_RENT_ORDINANCE_AGA_CAP_BASIS_POINTS
            {
                Output {
                    mode: BerkeleyRentOrdinanceMode::ViolationAnnualGeneralAdjustmentExceedsMeasureBb5PctCap,
                    statutory_basis: "BMC § 13.76.110 + Measure BB (2020) — AGA capped at 5 percent".to_string(),
                    notes: format!(
                        "VIOLATION: proposed AGA of {} basis points exceeds the Measure BB statutory cap of {} basis points (5 percent); tenant may petition Rent Board for rollback.",
                        input.proposed_annual_general_adjustment_basis_points,
                        BERKELEY_RENT_ORDINANCE_AGA_CAP_BASIS_POINTS
                    ),
                    citations,
                    statutory_annual_general_adjustment_cap_basis_points: BERKELEY_RENT_ORDINANCE_AGA_CAP_BASIS_POINTS,
                }
            } else {
                Output {
                    mode: BerkeleyRentOrdinanceMode::CompliantAnnualGeneralAdjustmentAtOrBelowMeasureBb5PctCap,
                    statutory_basis: "BMC § 13.76.110 + Measure BB (2020) — AGA at or below 5 percent cap".to_string(),
                    notes: format!(
                        "COMPLIANT: proposed AGA of {} basis points is at or below the Measure BB statutory cap of {} basis points (5 percent).",
                        input.proposed_annual_general_adjustment_basis_points,
                        BERKELEY_RENT_ORDINANCE_AGA_CAP_BASIS_POINTS
                    ),
                    citations,
                    statutory_annual_general_adjustment_cap_basis_points: BERKELEY_RENT_ORDINANCE_AGA_CAP_BASIS_POINTS,
                }
            }
        }
        ComplianceAspect::JustCauseEviction => match input.just_cause_ground_asserted {
            JustCauseGroundAsserted::NoJustCauseAsserted => Output {
                mode: BerkeleyRentOrdinanceMode::ViolationEvictionWithoutOneOfElevenJustCauseGrounds,
                statutory_basis: "BMC § 13.76.130 — eviction prohibited without one of the 11 enumerated just-cause grounds".to_string(),
                notes: "VIOLATION: landlord served termination notice without asserting any of the 11 enumerated just-cause grounds under BMC § 13.76.130; termination notice unenforceable; tenant may assert as affirmative defense in unlawful detainer.".to_string(),
                citations,
                statutory_annual_general_adjustment_cap_basis_points: BERKELEY_RENT_ORDINANCE_AGA_CAP_BASIS_POINTS,
            },
            JustCauseGroundAsserted::NonQualifyingGroundPropertySaleOrLeaseExpirationOrSection8OrForeclosure => Output {
                mode: BerkeleyRentOrdinanceMode::ViolationEvictionAssertingNonQualifyingGroundPropertySaleOrLeaseExpirationOrSection8OrForeclosure,
                statutory_basis: "BMC § 13.76.130 — property sales, lease expiration, Section 8 status changes, and foreclosure are NOT valid bases for eviction".to_string(),
                notes: "VIOLATION: landlord asserted a non-qualifying ground (property sale, lease expiration, Section 8 status change, or foreclosure) as the basis for eviction; BMC § 13.76.130 explicitly excludes these as valid bases; termination notice unenforceable.".to_string(),
                citations,
                statutory_annual_general_adjustment_cap_basis_points: BERKELEY_RENT_ORDINANCE_AGA_CAP_BASIS_POINTS,
            },
            _ => Output {
                mode: BerkeleyRentOrdinanceMode::CompliantJustCauseEvictionUnderOneOfElevenSection137613Grounds,
                statutory_basis: "BMC § 13.76.130 — eviction under one of the 11 enumerated just-cause grounds".to_string(),
                notes: format!(
                    "COMPLIANT: just-cause eviction asserted under one of the 11 enumerated grounds in BMC § 13.76.130 ({:?}); separate § 13.76.130 3-business-day Rent Board filing requirement must also be satisfied.",
                    input.just_cause_ground_asserted
                ),
                citations,
                statutory_annual_general_adjustment_cap_basis_points: BERKELEY_RENT_ORDINANCE_AGA_CAP_BASIS_POINTS,
            },
        },
        ComplianceAspect::OwnerMoveInUnderSection137613 => {
            if !input.owner_move_in_90_day_vacancy_search_performed {
                Output {
                    mode: BerkeleyRentOrdinanceMode::ViolationOwnerMoveInWithout90DayVacancySearchOfBerkeleyProperties,
                    statutory_basis: "BMC § 13.76.130 — owner move-in requires 90-day vacancy search of landlord's other Berkeley rental properties".to_string(),
                    notes: "VIOLATION: owner / qualified-relative move-in termination served without 90-day vacancy search of landlord's other Berkeley rental properties; § 13.76.130 requires search to ensure no comparable vacant unit is available for the intended occupant; failure invalidates the OMI termination.".to_string(),
                    citations,
                    statutory_annual_general_adjustment_cap_basis_points: BERKELEY_RENT_ORDINANCE_AGA_CAP_BASIS_POINTS,
                }
            } else {
                Output {
                    mode: BerkeleyRentOrdinanceMode::CompliantOwnerMoveInWith90DayVacancySearchAndRentBoardFiling,
                    statutory_basis: "BMC § 13.76.130 — owner / qualified-relative move-in with 90-day Berkeley vacancy search".to_string(),
                    notes: "COMPLIANT: owner / qualified-relative move-in with 90-day vacancy search of landlord's other Berkeley rental properties completed; separate Rent Board notice filing within 3 business days must also be satisfied.".to_string(),
                    citations,
                    statutory_annual_general_adjustment_cap_basis_points: BERKELEY_RENT_ORDINANCE_AGA_CAP_BASIS_POINTS,
                }
            }
        }
        ComplianceAspect::NoticeOfTerminationFilingWithRentBoard => {
            if !input.notice_of_termination_filed_with_rent_board
                || input.notice_of_termination_filing_business_days_after_service
                    > BERKELEY_RENT_ORDINANCE_NOTICE_OF_TERMINATION_FILING_BUSINESS_DAYS
            {
                Output {
                    mode: BerkeleyRentOrdinanceMode::ViolationNoticeOfTerminationNotFiledWithRentBoardWithin3BusinessDays,
                    statutory_basis: "BMC § 13.76.130 — termination notice must be filed with Rent Board within 3 business days of service on tenant".to_string(),
                    notes: format!(
                        "VIOLATION: termination notice filing status (filed = {}, business days after service = {}) does not satisfy BMC § 13.76.130 3-business-day Rent Board filing requirement; failure invalidates the termination notice.",
                        input.notice_of_termination_filed_with_rent_board,
                        input.notice_of_termination_filing_business_days_after_service
                    ),
                    citations,
                    statutory_annual_general_adjustment_cap_basis_points: BERKELEY_RENT_ORDINANCE_AGA_CAP_BASIS_POINTS,
                }
            } else {
                Output {
                    mode: BerkeleyRentOrdinanceMode::CompliantNoticeOfTerminationFiledWithRentBoardWithin3BusinessDays,
                    statutory_basis: "BMC § 13.76.130 — termination notice filed with Rent Board within 3 business days".to_string(),
                    notes: format!(
                        "COMPLIANT: termination notice filed with Berkeley Rent Board {} business days after service on tenant (≤ 3-business-day statutory window).",
                        input.notice_of_termination_filing_business_days_after_service
                    ),
                    citations,
                    statutory_annual_general_adjustment_cap_basis_points: BERKELEY_RENT_ORDINANCE_AGA_CAP_BASIS_POINTS,
                }
            }
        }
        ComplianceAspect::MandatoryRegistrationOfRentalUnits => {
            if !input.rental_unit_registered_and_fee_paid {
                Output {
                    mode: BerkeleyRentOrdinanceMode::ViolationRentalUnitNotRegisteredCannotCollectAga,
                    statutory_basis: "BMC § 13.76.080 — rental unit registration with Rent Board and annual fee required".to_string(),
                    notes: "VIOLATION: rental unit not registered with Berkeley Rent Board OR annual per-unit registration fee not paid; landlord cannot collect AGA-authorized rent increases until registration is current.".to_string(),
                    citations,
                    statutory_annual_general_adjustment_cap_basis_points: BERKELEY_RENT_ORDINANCE_AGA_CAP_BASIS_POINTS,
                }
            } else {
                Output {
                    mode: BerkeleyRentOrdinanceMode::CompliantRentalUnitRegisteredAndRegistrationFeePaid,
                    statutory_basis: "BMC § 13.76.080 — rental unit registered and annual fee paid".to_string(),
                    notes: "COMPLIANT: rental unit registered with Berkeley Rent Board and annual per-unit registration fee paid; landlord eligible to collect AGA-authorized rent increases.".to_string(),
                    citations,
                    statutory_annual_general_adjustment_cap_basis_points: BERKELEY_RENT_ORDINANCE_AGA_CAP_BASIS_POINTS,
                }
            }
        }
        ComplianceAspect::SecurityDepositAnnualInterest => {
            if !input.security_deposit_annual_interest_paid {
                Output {
                    mode: BerkeleyRentOrdinanceMode::ViolationSecurityDepositAnnualInterestNotPaid,
                    statutory_basis: "BMC § 13.76.070 — landlord must pay annual interest on security deposits at Rent Board-set rate".to_string(),
                    notes: "VIOLATION: landlord failed to pay tenant annual interest on security deposit; BMC § 13.76.070 requires annual interest payment at rate set by Rent Board each year (typically tied to average rate paid on bank passbook savings accounts).".to_string(),
                    citations,
                    statutory_annual_general_adjustment_cap_basis_points: BERKELEY_RENT_ORDINANCE_AGA_CAP_BASIS_POINTS,
                }
            } else {
                Output {
                    mode: BerkeleyRentOrdinanceMode::CompliantSecurityDepositAnnualInterestPaidAtRentBoardSetRate,
                    statutory_basis: "BMC § 13.76.070 — annual interest on security deposits paid at Rent Board-set rate".to_string(),
                    notes: "COMPLIANT: landlord paid tenant annual interest on security deposit at Rent Board-set rate.".to_string(),
                    citations,
                    statutory_annual_general_adjustment_cap_basis_points: BERKELEY_RENT_ORDINANCE_AGA_CAP_BASIS_POINTS,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_aga_input() -> Input {
        Input {
            property_jurisdiction: PropertyJurisdiction::WithinBerkeleyCityLimits,
            unit_type: UnitType::StandardRentControlledApartment,
            compliance_aspect: ComplianceAspect::AnnualGeneralAdjustmentRentIncrease,
            proposed_annual_general_adjustment_basis_points: 300,
            just_cause_ground_asserted: JustCauseGroundAsserted::NoJustCauseAsserted,
            owner_move_in_90_day_vacancy_search_performed: false,
            notice_of_termination_filing_business_days_after_service: 0,
            notice_of_termination_filed_with_rent_board: false,
            rental_unit_registered_and_fee_paid: true,
            security_deposit_annual_interest_paid: true,
        }
    }

    #[test]
    fn property_outside_berkeley_not_applicable() {
        let mut input = baseline_aga_input();
        input.property_jurisdiction = PropertyJurisdiction::OutsideBerkeleyCityLimits;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::NotApplicablePropertyOutsideBerkeley
        );
    }

    #[test]
    fn non_residential_unit_not_applicable() {
        let mut input = baseline_aga_input();
        input.unit_type = UnitType::NonResidentialUnitExempt;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::NotApplicableNonResidentialUnit
        );
    }

    #[test]
    fn single_family_costa_hawkins_aga_not_applicable() {
        let mut input = baseline_aga_input();
        input.unit_type = UnitType::SingleFamilyHomeCostaHawkinsApplies;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::NotApplicableCostaHawkinsSingleFamilyOrCondoOrPost1995VacancyDecontrolApplies
        );
    }

    #[test]
    fn condominium_costa_hawkins_aga_not_applicable() {
        let mut input = baseline_aga_input();
        input.unit_type = UnitType::CondominiumUnitCostaHawkinsApplies;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::NotApplicableCostaHawkinsSingleFamilyOrCondoOrPost1995VacancyDecontrolApplies
        );
    }

    #[test]
    fn post_february_1_1995_costa_hawkins_aga_not_applicable() {
        let mut input = baseline_aga_input();
        input.unit_type = UnitType::PostFebruary1_1995CertificateOfOccupancyCostaHawkinsApplies;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::NotApplicableCostaHawkinsSingleFamilyOrCondoOrPost1995VacancyDecontrolApplies
        );
    }

    #[test]
    fn aga_at_5_pct_cap_exactly_compliant() {
        let mut input = baseline_aga_input();
        input.proposed_annual_general_adjustment_basis_points = 500;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::CompliantAnnualGeneralAdjustmentAtOrBelowMeasureBb5PctCap
        );
    }

    #[test]
    fn aga_below_5_pct_cap_compliant() {
        let output = check(&baseline_aga_input());
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::CompliantAnnualGeneralAdjustmentAtOrBelowMeasureBb5PctCap
        );
        assert_eq!(
            output.statutory_annual_general_adjustment_cap_basis_points,
            500
        );
    }

    #[test]
    fn aga_above_5_pct_cap_violation() {
        let mut input = baseline_aga_input();
        input.proposed_annual_general_adjustment_basis_points = 600;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::ViolationAnnualGeneralAdjustmentExceedsMeasureBb5PctCap
        );
    }

    #[test]
    fn just_cause_non_payment_compliant() {
        let mut input = baseline_aga_input();
        input.compliance_aspect = ComplianceAspect::JustCauseEviction;
        input.just_cause_ground_asserted = JustCauseGroundAsserted::NonPaymentOfRentAfter3DayNotice;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::CompliantJustCauseEvictionUnderOneOfElevenSection137613Grounds
        );
    }

    #[test]
    fn just_cause_ellis_act_withdrawal_compliant() {
        let mut input = baseline_aga_input();
        input.compliance_aspect = ComplianceAspect::JustCauseEviction;
        input.just_cause_ground_asserted =
            JustCauseGroundAsserted::EllisActWithdrawalUnderCaliforniaGovCode7060;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::CompliantJustCauseEvictionUnderOneOfElevenSection137613Grounds
        );
    }

    #[test]
    fn eviction_without_just_cause_violation() {
        let mut input = baseline_aga_input();
        input.compliance_aspect = ComplianceAspect::JustCauseEviction;
        input.just_cause_ground_asserted = JustCauseGroundAsserted::NoJustCauseAsserted;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::ViolationEvictionWithoutOneOfElevenJustCauseGrounds
        );
    }

    #[test]
    fn eviction_for_foreclosure_or_sale_or_section_8_violation() {
        let mut input = baseline_aga_input();
        input.compliance_aspect = ComplianceAspect::JustCauseEviction;
        input.just_cause_ground_asserted =
            JustCauseGroundAsserted::NonQualifyingGroundPropertySaleOrLeaseExpirationOrSection8OrForeclosure;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::ViolationEvictionAssertingNonQualifyingGroundPropertySaleOrLeaseExpirationOrSection8OrForeclosure
        );
    }

    #[test]
    fn owner_move_in_with_90_day_search_compliant() {
        let mut input = baseline_aga_input();
        input.compliance_aspect = ComplianceAspect::OwnerMoveInUnderSection137613;
        input.owner_move_in_90_day_vacancy_search_performed = true;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::CompliantOwnerMoveInWith90DayVacancySearchAndRentBoardFiling
        );
    }

    #[test]
    fn owner_move_in_without_90_day_search_violation() {
        let mut input = baseline_aga_input();
        input.compliance_aspect = ComplianceAspect::OwnerMoveInUnderSection137613;
        input.owner_move_in_90_day_vacancy_search_performed = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::ViolationOwnerMoveInWithout90DayVacancySearchOfBerkeleyProperties
        );
    }

    #[test]
    fn termination_notice_filed_within_3_business_days_compliant() {
        let mut input = baseline_aga_input();
        input.compliance_aspect = ComplianceAspect::NoticeOfTerminationFilingWithRentBoard;
        input.notice_of_termination_filed_with_rent_board = true;
        input.notice_of_termination_filing_business_days_after_service = 2;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::CompliantNoticeOfTerminationFiledWithRentBoardWithin3BusinessDays
        );
    }

    #[test]
    fn termination_notice_at_exactly_3_business_days_compliant_boundary() {
        let mut input = baseline_aga_input();
        input.compliance_aspect = ComplianceAspect::NoticeOfTerminationFilingWithRentBoard;
        input.notice_of_termination_filed_with_rent_board = true;
        input.notice_of_termination_filing_business_days_after_service = 3;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::CompliantNoticeOfTerminationFiledWithRentBoardWithin3BusinessDays
        );
    }

    #[test]
    fn termination_notice_at_4_business_days_violation_boundary() {
        let mut input = baseline_aga_input();
        input.compliance_aspect = ComplianceAspect::NoticeOfTerminationFilingWithRentBoard;
        input.notice_of_termination_filed_with_rent_board = true;
        input.notice_of_termination_filing_business_days_after_service = 4;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::ViolationNoticeOfTerminationNotFiledWithRentBoardWithin3BusinessDays
        );
    }

    #[test]
    fn termination_notice_not_filed_violation() {
        let mut input = baseline_aga_input();
        input.compliance_aspect = ComplianceAspect::NoticeOfTerminationFilingWithRentBoard;
        input.notice_of_termination_filed_with_rent_board = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::ViolationNoticeOfTerminationNotFiledWithRentBoardWithin3BusinessDays
        );
    }

    #[test]
    fn rental_unit_registered_compliant() {
        let mut input = baseline_aga_input();
        input.compliance_aspect = ComplianceAspect::MandatoryRegistrationOfRentalUnits;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::CompliantRentalUnitRegisteredAndRegistrationFeePaid
        );
    }

    #[test]
    fn rental_unit_not_registered_violation() {
        let mut input = baseline_aga_input();
        input.compliance_aspect = ComplianceAspect::MandatoryRegistrationOfRentalUnits;
        input.rental_unit_registered_and_fee_paid = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::ViolationRentalUnitNotRegisteredCannotCollectAga
        );
    }

    #[test]
    fn security_deposit_interest_paid_compliant() {
        let mut input = baseline_aga_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositAnnualInterest;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::CompliantSecurityDepositAnnualInterestPaidAtRentBoardSetRate
        );
    }

    #[test]
    fn security_deposit_interest_not_paid_violation() {
        let mut input = baseline_aga_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositAnnualInterest;
        input.security_deposit_annual_interest_paid = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            BerkeleyRentOrdinanceMode::ViolationSecurityDepositAnnualInterestNotPaid
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(BERKELEY_RENT_ORDINANCE_ENACTMENT_YEAR, 1980);
        assert_eq!(BERKELEY_RENT_ORDINANCE_ENACTMENT_MONTH, 6);
        assert_eq!(
            BERKELEY_RENT_ORDINANCE_BASE_RENT_CEILING_REFERENCE_YEAR,
            1980
        );
        assert_eq!(BERKELEY_RENT_ORDINANCE_BASE_RENT_CEILING_REFERENCE_MONTH, 5);
        assert_eq!(BERKELEY_RENT_ORDINANCE_BASE_RENT_CEILING_REFERENCE_DAY, 31);
        assert_eq!(BERKELEY_RENT_ORDINANCE_AGA_CAP_BASIS_POINTS, 500);
        assert_eq!(BERKELEY_RENT_ORDINANCE_OMI_VACANCY_SEARCH_DAYS, 90);
        assert_eq!(
            BERKELEY_RENT_ORDINANCE_NOTICE_OF_TERMINATION_FILING_BUSINESS_DAYS,
            3
        );
        assert_eq!(
            BERKELEY_RENT_ORDINANCE_NUMBER_OF_JUST_CAUSE_GROUNDS_POST_2024_AMENDMENT,
            11
        );
        assert_eq!(
            BERKELEY_RENT_ORDINANCE_NUMBER_OF_JUST_CAUSE_GROUNDS_PRE_2024_AMENDMENT,
            12
        );
        assert_eq!(BERKELEY_RENT_ORDINANCE_NON_PAYMENT_NOTICE_DAYS, 3);
        assert_eq!(BERKELEY_RENT_ORDINANCE_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(
            BERKELEY_RENT_ORDINANCE_COSTA_HAWKINS_VACANCY_DECONTROL_YEAR,
            1995
        );
    }

    #[test]
    fn citation_contains_landmarks() {
        let output = check(&baseline_aga_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("Chapter 13.76"));
        assert!(joined.contains("1980"));
        assert!(joined.contains("MAY 31, 1980"));
        assert!(joined.contains("Measure BB"));
        assert!(joined.contains("5 PERCENT"));
        assert!(joined.contains("§ 13.76.130"));
        assert!(joined.contains("§ 13.76.070"));
        assert!(joined.contains("§ 13.76.080"));
        assert!(joined.contains("3 BUSINESS DAYS"));
        assert!(joined.contains("Costa-Hawkins"));
        assert!(joined.contains("Ellis Act"));
        assert!(joined.contains("foreclosure"));
    }
}
