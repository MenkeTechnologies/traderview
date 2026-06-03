//! Fair Housing Amendments Act of 1988 (FHAA) Compliance
//! Module — federal civil rights statute that added
//! HANDICAP (DISABILITY) and FAMILIAL STATUS as protected
//! classes under the Fair Housing Act and imposed design
//! and construction requirements on covered multifamily
//! dwellings first occupied after March 13, 1991.
//!
//! Pure-compute check for landlord compliance with the Fair
//! Housing Amendments Act of 1988 (Public Law 100-430),
//! signed into law by President Ronald Reagan on **September
//! 13, 1988** and made generally effective on **March 12,
//! 1989** (180 days after enactment). The FHAA added two new
//! protected classes — HANDICAP (renamed DISABILITY in 1992
//! amendments) and FAMILIAL STATUS — to the seven-class
//! Fair Housing Act framework codified at 42 USC §§
//! 3601-3619 (Title VIII of the Civil Rights Act of 1968 as
//! amended). The FHAA also imposed comprehensive design and
//! construction accessibility requirements on covered
//! multifamily dwellings for first occupancy after **March
//! 13, 1991** (30 months after enactment).
//!
//! Web research (verified 2026-06-03):
//! - **Enactment**: Fair Housing Amendments Act of 1988 (Public Law 100-430); signed by President Ronald Reagan on **SEPTEMBER 13, 1988**; generally effective 180 days after enactment (**MARCH 12, 1989**) ([HUD-DOJ Joint Statement on Reasonable Accommodations](https://www.hud.gov/sites/documents/huddojstatement.pdf); [Cornell LII 42 USC § 3604](https://www.law.cornell.edu/uscode/text/42/3604); [House Office of Law Revision Counsel — 42 USC Chapter 45 Fair Housing](https://uscode.house.gov/view.xhtml?path=%2Fprelim%40title42%2Fchapter45&edition=prelim); [GovInfo — Fair Housing Amendments Act of 1988 (USCODE-2009)](https://www.govinfo.gov/content/pkg/USCODE-2009-title42/html/USCODE-2009-title42-chap45-subchapI.htm); [Federal Register — Implementation of the Fair Housing Act's Discriminatory Effects Standard](https://www.federalregister.gov/documents/2013/02/15/2013-03375/implementation-of-the-fair-housing-acts-discriminatory-effects-standard); [DC Office of Human Rights Fair Housing Act PDF](https://ohr.dc.gov/sites/default/files/dc/sites/ohr/publication/attachments/FairHousingAct.pdf); [Animal Legal & Historical Center — Fair Housing Definitions § 3602](https://www.animallaw.info/statute/us-housing-fair-housing-subchapter-i-generally-section-3602-definitions); [Behavioral Health News — Using the Fair Housing Act to Obtain Housing for People with Disabilities](https://behavioralhealthnews.org/using-the-fair-housing-act-to-obtain-housing-for-people-with-disabilities/)).
//! - **Statutory Basis**: codified at **42 USC §§ 3601-3619** (Title VIII of the Civil Rights Act of 1968 as amended); definitions at 42 USC § 3602; prohibited practices at 42 USC § 3604.
//! - **Seven Protected Classes Under FHA After FHAA**: (1) **RACE**; (2) **COLOR**; (3) **RELIGION**; (4) **SEX**; (5) **NATIONAL ORIGIN** (these five were in the original 1968 Fair Housing Act); (6) **HANDICAP / DISABILITY** (added by FHAA 1988); (7) **FAMILIAL STATUS** (added by FHAA 1988).
//! - **Handicap / Disability Definition (42 USC § 3602(h))**: a physical or mental impairment which substantially limits one or more of a person's major life activities; or a record of having such an impairment; or being regarded as having such an impairment. Excludes current illegal use of a controlled substance.
//! - **Familial Status Definition (42 USC § 3602(k))**: one or more individuals who have not attained the age of **18** YEARS being domiciled with (a) a parent or another person having legal custody of such individual or individuals; or (b) the designee of such parent or other person having such custody, with the written permission of such parent or other person. Also extends to any person who is pregnant or in the process of securing legal custody of any individual who has not attained age 18.
//! - **Reasonable Accommodation (42 USC § 3604(f)(3)(B))**: it is unlawful to refuse to make REASONABLE ACCOMMODATIONS in rules, policies, practices, or services when such accommodations may be necessary to afford a handicapped person equal opportunity to use and enjoy a dwelling.
//! - **Reasonable Modification (42 USC § 3604(f)(3)(A))**: it is unlawful to refuse to permit, at the expense of the handicapped person, REASONABLE MODIFICATIONS of existing premises occupied or to be occupied by such person if such modifications may be necessary to afford such person full enjoyment of the premises. (In the case of a rental, the landlord may, where it is reasonable to do so, condition permission for a modification on the renter agreeing to restore the interior of the premises to the condition that existed before the modification.)
//! - **Design and Construction Requirements (42 USC § 3604(f)(3)(C))**: for covered multifamily dwellings for first occupancy AFTER **MARCH 13, 1991** (30 months following enactment), the following design and construction requirements apply: **(i)** public and common use portions of such dwellings are READILY ACCESSIBLE TO and USABLE BY handicapped persons; **(ii)** all doors designed to allow passage into and within all premises within such dwellings are sufficiently wide to allow passage by handicapped persons in WHEELCHAIRS (typically 32-inch clear width minimum); **(iii)** all premises within such dwellings contain the following features of adaptive design: **(I)** an accessible route into and through the dwelling; **(II)** light switches, electrical outlets, thermostats, and other environmental controls in accessible locations; **(III)** reinforcements in bathroom walls to allow later installation of grab bars; and **(IV)** usable kitchens and bathrooms such that an individual in a wheelchair can maneuver about the space.
//! - **Covered Multifamily Dwelling Definition (42 USC § 3604(f)(7))**: covered multifamily dwellings means (A) buildings consisting of **4 OR MORE UNITS** if such buildings have ONE OR MORE ELEVATORS (all units covered); and (B) GROUND FLOOR UNITS in other buildings consisting of 4 or more units (only ground floor units covered).
//! - **Senior Housing Exemption Under HOPA**: § 3607 provides that nothing in the FHA limits the applicability of any reasonable local, state, or federal restrictions regarding the maximum number of occupants permitted to occupy a dwelling. The Housing for Older Persons Act of 1995 (HOPA — built iter 675) carved out three categories of senior housing from the FHAA familial-status protections.
//! - **Single-Family Dwelling Exemption (42 USC § 3603(b)(1))**: the FHA generally does not apply to single-family houses sold or rented by their owner provided (a) the owner does not own more than 3 single-family houses at any one time; (b) such house was not sold without the use of a real estate broker; and (c) the owner has not engaged in more than one sale within 24 months. Note: this exemption does NOT apply to advertising or to actions involving direct or indirect discriminatory intent.
//! - **Mrs. Murphy Exemption (42 USC § 3603(b)(2))**: the FHA generally does not apply to rooms or units in dwellings containing living quarters occupied by NO MORE THAN 4 FAMILIES (commonly called the "Mrs. Murphy" exemption) if the owner actually maintains and occupies one of such living quarters as his residence.
//! - **Enforcement**: HUD Office of Fair Housing and Equal Opportunity (FHEO) administrative complaint process under 24 CFR Part 103; private right of action in federal district court under 42 USC § 3613 (actual damages + punitive damages + reasonable attorney's fees + injunctive relief); DOJ enforcement of pattern-or-practice violations under 42 USC § 3614 with civil penalties up to **$100,000** per pattern-or-practice violation.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const FHAA_ENACTMENT_YEAR: u32 = 1988;
pub const FHAA_ENACTMENT_MONTH: u32 = 9;
pub const FHAA_ENACTMENT_DAY: u32 = 13;
pub const FHAA_GENERAL_EFFECTIVE_DATE_YEAR: u32 = 1989;
pub const FHAA_GENERAL_EFFECTIVE_DATE_MONTH: u32 = 3;
pub const FHAA_GENERAL_EFFECTIVE_DATE_DAY: u32 = 12;
pub const FHAA_DESIGN_CONSTRUCTION_EFFECTIVE_DATE_YEAR: u32 = 1991;
pub const FHAA_DESIGN_CONSTRUCTION_EFFECTIVE_DATE_MONTH: u32 = 3;
pub const FHAA_DESIGN_CONSTRUCTION_EFFECTIVE_DATE_DAY: u32 = 13;
pub const FHAA_COVERED_MULTIFAMILY_MIN_UNITS: u32 = 4;
pub const FHAA_DESIGN_CONSTRUCTION_LAG_MONTHS: u32 = 30;
pub const FHAA_FAMILIAL_STATUS_AGE_THRESHOLD_YEARS: u32 = 18;
pub const FHAA_SINGLE_FAMILY_EXEMPTION_MAX_HOMES: u32 = 3;
pub const FHAA_SINGLE_FAMILY_EXEMPTION_MAX_SALES_PER_24_MONTHS: u32 = 1;
pub const FHAA_MRS_MURPHY_EXEMPTION_MAX_FAMILIES: u32 = 4;
pub const FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS: u64 = 100_000;
pub const FHAA_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    CoveredHousingUnderFha,
    SingleFamilyDwellingExemptUnderSection3603B1,
    MrsMurphyExemptUnderSection3603B2,
    SeniorHousingExemptUnderHopaSection3607,
    NotHousingOutsideFhaScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedClassAtIssue {
    Race,
    Color,
    Religion,
    Sex,
    NationalOrigin,
    HandicapDisabilityAddedByFhaa,
    FamilialStatusAddedByFhaa,
    NoProtectedClassDiscriminationAtIssue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CoveredMultifamilyStatus {
    CoveredMultifamilyDwellingFirstOccupancyAfterMarch13_1991,
    CoveredMultifamilyDwellingFirstOccupancyOnOrBeforeMarch13_1991PreEffectiveDate,
    NotCoveredMultifamilyDwellingExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    NondiscriminationOnAnyProtectedClassUnderSection3604,
    ReasonableAccommodationUnderSection3604F3B,
    ReasonableModificationUnderSection3604F3A,
    DesignAndConstructionRequirementsUnderSection3604F3C,
    FamilialStatusProtectionsForFamiliesWithChildrenUnder18,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscriminatoryActionStatus {
    NoDiscriminatoryActionTaken,
    DiscriminatoryActionTakenAgainstProtectedClass,
    ReasonableAccommodationDeniedWithoutLegitimateBasis,
    ReasonableModificationDeniedWithoutLegitimateBasis,
    DesignAndConstructionRequirementsNotMet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FhaaMode {
    NotApplicableNotHousingOutsideFhaScope,
    NotApplicablePreEffectiveDateMarch12_1989,
    NotApplicableSingleFamilyDwellingExemptUnderSection3603B1,
    NotApplicableMrsMurphyExemptUnderSection3603B2,
    NotApplicableSeniorHousingExemptUnderHopaSection3607,
    CompliantNoDiscriminationOnAnyProtectedClass,
    CompliantDesignAndConstructionRequirementsForCoveredMultifamilyDwelling,
    CompliantReasonableAccommodationProvided,
    CompliantReasonableModificationPermitted,
    CompliantFamilialStatusProtectionsApplied,
    ViolationDiscriminationOnBasisOfRace,
    ViolationDiscriminationOnBasisOfColor,
    ViolationDiscriminationOnBasisOfReligion,
    ViolationDiscriminationOnBasisOfSex,
    ViolationDiscriminationOnBasisOfNationalOrigin,
    ViolationDiscriminationOnBasisOfHandicapDisability,
    ViolationDiscriminationOnBasisOfFamilialStatus,
    ViolationFailureToProvideReasonableAccommodation,
    ViolationFailureToPermitReasonableModification,
    ViolationDesignAndConstructionRequirementsNotMet,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_type: PropertyType,
    pub action_date_satisfies_general_effective_date: bool,
    pub compliance_aspect: ComplianceAspect,
    pub protected_class_at_issue: ProtectedClassAtIssue,
    pub discriminatory_action_status: DiscriminatoryActionStatus,
    pub covered_multifamily_status: CoveredMultifamilyStatus,
    pub design_and_construction_requirements_met: bool,
    pub reasonable_accommodation_provided: bool,
    pub reasonable_modification_permitted: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: FhaaMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub maximum_pattern_or_practice_civil_penalty_dollars: u64,
}

pub type RentalFairHousingAmendmentsActOf1988FhaaInput = Input;
pub type RentalFairHousingAmendmentsActOf1988FhaaOutput = Output;
pub type RentalFairHousingAmendmentsActOf1988FhaaResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Fair Housing Amendments Act of 1988 (Public Law 100-430) — signed by President Ronald Reagan on September 13, 1988; generally effective March 12, 1989 (180 days after enactment); design and construction requirements effective March 13, 1991 (30 months after enactment); codified at 42 USC §§ 3601-3619".to_string(),
        "Seven Protected Classes Under Fair Housing Act After FHAA — RACE, COLOR, RELIGION, SEX, NATIONAL ORIGIN (original 1968 Civil Rights Act / Title VIII); HANDICAP / DISABILITY (added by FHAA 1988); FAMILIAL STATUS (added by FHAA 1988)".to_string(),
        "42 USC § 3602(h) Handicap / Disability Definition — physical or mental impairment which substantially limits one or more of a person's major life activities; record of having such impairment; or being regarded as having such impairment; excludes current illegal use of controlled substance".to_string(),
        "42 USC § 3602(k) Familial Status Definition — one or more individuals under age 18 domiciled with (a) parent or another person having legal custody; or (b) designee of such parent / custodian with written permission; extends to pregnant persons and those in process of securing legal custody".to_string(),
        "42 USC § 3604(f)(3)(B) Reasonable Accommodation — unlawful to refuse to make REASONABLE ACCOMMODATIONS in rules, policies, practices, or services when such accommodations may be necessary to afford handicapped person equal opportunity to use and enjoy dwelling".to_string(),
        "42 USC § 3604(f)(3)(A) Reasonable Modification — unlawful to refuse to permit, at expense of handicapped person, REASONABLE MODIFICATIONS of existing premises if such modifications may be necessary to afford full enjoyment; landlord may condition modification permission on renter restoring premises to pre-modification condition where reasonable".to_string(),
        "42 USC § 3604(f)(3)(C) Design and Construction Requirements — for covered multifamily dwellings for first occupancy AFTER MARCH 13, 1991: (i) public and common use portions readily accessible to and usable by handicapped persons; (ii) doors sufficiently wide for wheelchair passage (typically 32-inch clear width); (iii) accessible route into and through dwelling + accessible environmental controls (light switches, outlets, thermostats) + bathroom wall reinforcements for later grab bar installation + usable kitchens and bathrooms with wheelchair maneuvering space".to_string(),
        "42 USC § 3604(f)(7) Covered Multifamily Dwelling Definition — buildings of 4 OR MORE UNITS with one or more ELEVATORS (ALL units covered); GROUND FLOOR units in other buildings of 4 or more units (only ground floor covered)".to_string(),
        "42 USC § 3603(b)(1) Single-Family Dwelling Exemption — generally exempts single-family houses sold or rented by owner if (a) owner does not own more than 3 single-family houses at any one time; (b) sold without real estate broker; (c) owner has not engaged in more than one sale within 24 months; exemption does NOT apply to advertising or direct/indirect discriminatory intent".to_string(),
        "42 USC § 3603(b)(2) Mrs. Murphy Exemption — generally exempts rooms or units in dwellings with NO MORE THAN 4 FAMILIES if owner maintains and occupies one of the units as residence".to_string(),
        "42 USC § 3607 + Housing for Older Persons Act of 1995 (HOPA — built iter 675) — senior housing exemptions from FHAA familial-status protections; three HOPA exemption categories (62+ 100% / 55+ with 80% occupancy + verification + intent / state or federally funded elderly housing)".to_string(),
        "Enforcement — HUD Office of Fair Housing and Equal Opportunity (FHEO) administrative complaint under 24 CFR Part 103; private right of action under 42 USC § 3613 (actual damages + punitive damages + reasonable attorney's fees + injunctive relief); DOJ pattern-or-practice enforcement under 42 USC § 3614 with civil penalties up to $100,000 per violation".to_string(),
        "Cornell LII 42 USC § 3604 — primary statutory text on prohibited practices".to_string(),
        "HUD-DOJ Joint Statement on Reasonable Accommodations — practitioner guidance".to_string(),
        "HUD Office of Fair Housing and Equal Opportunity (FHEO) program page".to_string(),
        "Federal Register — Implementation of Fair Housing Act's Discriminatory Effects Standard (2013)".to_string(),
    ];

    if input.property_type == PropertyType::NotHousingOutsideFhaScope {
        return Output {
            mode: FhaaMode::NotApplicableNotHousingOutsideFhaScope,
            statutory_basis: "42 USC §§ 3601-3619 — FHA applies only to dwellings within statutory scope".to_string(),
            notes: "NOT APPLICABLE: property is not a dwelling subject to FHA / FHAA; outside Title VIII statutory scope.".to_string(),
            citations,
            maximum_pattern_or_practice_civil_penalty_dollars: 0,
        };
    }

    if !input.action_date_satisfies_general_effective_date {
        return Output {
            mode: FhaaMode::NotApplicablePreEffectiveDateMarch12_1989,
            statutory_basis: "Public Law 100-430 effective date — FHAA generally effective March 12, 1989 (180 days after September 13, 1988 enactment)".to_string(),
            notes: "NOT APPLICABLE: action occurred before the FHAA general effective date of March 12, 1989; pre-effective-date action; original 1968 FHA without FHAA handicap and familial status protections governs.".to_string(),
            citations,
            maximum_pattern_or_practice_civil_penalty_dollars: 0,
        };
    }

    match input.property_type {
        PropertyType::SingleFamilyDwellingExemptUnderSection3603B1 => {
            return Output {
                mode: FhaaMode::NotApplicableSingleFamilyDwellingExemptUnderSection3603B1,
                statutory_basis: "42 USC § 3603(b)(1) — single-family dwelling exemption when owner owns ≤ 3 homes, no broker, ≤ 1 sale per 24 months".to_string(),
                notes: "NOT APPLICABLE: single-family dwelling sold or rented by owner satisfying all three § 3603(b)(1) exemption prongs (owner owns ≤ 3 homes; no real estate broker; ≤ 1 sale per 24 months); FHA generally inapplicable; advertising and direct/indirect discriminatory intent provisions still apply.".to_string(),
                citations,
                maximum_pattern_or_practice_civil_penalty_dollars: 0,
            };
        }
        PropertyType::MrsMurphyExemptUnderSection3603B2 => {
            return Output {
                mode: FhaaMode::NotApplicableMrsMurphyExemptUnderSection3603B2,
                statutory_basis: "42 USC § 3603(b)(2) — Mrs. Murphy exemption for owner-occupied dwellings with no more than 4 families".to_string(),
                notes: "NOT APPLICABLE: owner-occupied dwelling with no more than 4 families ('Mrs. Murphy' exemption); FHA generally inapplicable.".to_string(),
                citations,
                maximum_pattern_or_practice_civil_penalty_dollars: 0,
            };
        }
        PropertyType::SeniorHousingExemptUnderHopaSection3607 => {
            return Output {
                mode: FhaaMode::NotApplicableSeniorHousingExemptUnderHopaSection3607,
                statutory_basis: "42 USC § 3607 + HOPA 1995 — senior housing exemptions from familial status protections (62+ / 55+ with 80%+ verification / state-or-federally-funded elderly housing)".to_string(),
                notes: "NOT APPLICABLE TO FAMILIAL STATUS: property qualifies for senior housing exemption under § 3607 and HOPA 1995; familial status protections do not bar exclusion of families with children; other FHAA protected classes (race, color, religion, sex, national origin, handicap) continue to apply.".to_string(),
                citations,
                maximum_pattern_or_practice_civil_penalty_dollars: FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS,
            };
        }
        PropertyType::CoveredHousingUnderFha => {}
        PropertyType::NotHousingOutsideFhaScope => unreachable!(),
    }

    match input.compliance_aspect {
        ComplianceAspect::NondiscriminationOnAnyProtectedClassUnderSection3604 => match input.protected_class_at_issue {
            ProtectedClassAtIssue::NoProtectedClassDiscriminationAtIssue => Output {
                mode: FhaaMode::CompliantNoDiscriminationOnAnyProtectedClass,
                statutory_basis: "42 USC § 3604 — no discriminatory action against any protected class".to_string(),
                notes: "COMPLIANT: no discriminatory action taken against any of the seven protected classes (race, color, religion, sex, national origin, handicap, familial status).".to_string(),
                citations,
                maximum_pattern_or_practice_civil_penalty_dollars: FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS,
            },
            ProtectedClassAtIssue::Race => Output {
                mode: FhaaMode::ViolationDiscriminationOnBasisOfRace,
                statutory_basis: "42 USC § 3604 — race discrimination prohibited".to_string(),
                notes: "VIOLATION: discriminatory action taken against applicant or tenant on basis of RACE; violates Civil Rights Act of 1968 Title VIII as amended.".to_string(),
                citations,
                maximum_pattern_or_practice_civil_penalty_dollars: FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS,
            },
            ProtectedClassAtIssue::Color => Output {
                mode: FhaaMode::ViolationDiscriminationOnBasisOfColor,
                statutory_basis: "42 USC § 3604 — color discrimination prohibited".to_string(),
                notes: "VIOLATION: discriminatory action taken against applicant or tenant on basis of COLOR.".to_string(),
                citations,
                maximum_pattern_or_practice_civil_penalty_dollars: FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS,
            },
            ProtectedClassAtIssue::Religion => Output {
                mode: FhaaMode::ViolationDiscriminationOnBasisOfReligion,
                statutory_basis: "42 USC § 3604 — religion discrimination prohibited".to_string(),
                notes: "VIOLATION: discriminatory action taken against applicant or tenant on basis of RELIGION.".to_string(),
                citations,
                maximum_pattern_or_practice_civil_penalty_dollars: FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS,
            },
            ProtectedClassAtIssue::Sex => Output {
                mode: FhaaMode::ViolationDiscriminationOnBasisOfSex,
                statutory_basis: "42 USC § 3604 — sex discrimination prohibited".to_string(),
                notes: "VIOLATION: discriminatory action taken against applicant or tenant on basis of SEX.".to_string(),
                citations,
                maximum_pattern_or_practice_civil_penalty_dollars: FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS,
            },
            ProtectedClassAtIssue::NationalOrigin => Output {
                mode: FhaaMode::ViolationDiscriminationOnBasisOfNationalOrigin,
                statutory_basis: "42 USC § 3604 — national origin discrimination prohibited".to_string(),
                notes: "VIOLATION: discriminatory action taken against applicant or tenant on basis of NATIONAL ORIGIN.".to_string(),
                citations,
                maximum_pattern_or_practice_civil_penalty_dollars: FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS,
            },
            ProtectedClassAtIssue::HandicapDisabilityAddedByFhaa => Output {
                mode: FhaaMode::ViolationDiscriminationOnBasisOfHandicapDisability,
                statutory_basis: "42 USC § 3604(f) — handicap / disability discrimination prohibited (added by FHAA 1988)".to_string(),
                notes: "VIOLATION: discriminatory action taken against applicant or tenant on basis of HANDICAP / DISABILITY; protected class added by FHAA 1988; tenant may pursue HUD administrative complaint or private right of action under 42 USC § 3613.".to_string(),
                citations,
                maximum_pattern_or_practice_civil_penalty_dollars: FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS,
            },
            ProtectedClassAtIssue::FamilialStatusAddedByFhaa => Output {
                mode: FhaaMode::ViolationDiscriminationOnBasisOfFamilialStatus,
                statutory_basis: "42 USC § 3604(a)-(c) — familial status discrimination prohibited (added by FHAA 1988)".to_string(),
                notes: "VIOLATION: discriminatory action taken against applicant or tenant on basis of FAMILIAL STATUS (presence of children under 18 in household; pregnant person; person securing legal custody); protected class added by FHAA 1988; HOPA exemption not applicable.".to_string(),
                citations,
                maximum_pattern_or_practice_civil_penalty_dollars: FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS,
            },
        },
        ComplianceAspect::ReasonableAccommodationUnderSection3604F3B => {
            if input.reasonable_accommodation_provided {
                Output {
                    mode: FhaaMode::CompliantReasonableAccommodationProvided,
                    statutory_basis: "42 USC § 3604(f)(3)(B) — reasonable accommodation provided".to_string(),
                    notes: "COMPLIANT: landlord provided requested reasonable accommodation in rules, policies, practices, or services necessary to afford handicapped person equal opportunity to use and enjoy dwelling.".to_string(),
                    citations,
                    maximum_pattern_or_practice_civil_penalty_dollars: FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS,
                }
            } else {
                Output {
                    mode: FhaaMode::ViolationFailureToProvideReasonableAccommodation,
                    statutory_basis: "42 USC § 3604(f)(3)(B) — failure to provide reasonable accommodation".to_string(),
                    notes: "VIOLATION: landlord refused to make reasonable accommodation in rules, policies, practices, or services without demonstrating fundamental program alteration or undue financial / administrative burden; § 3604(f)(3)(B) requires accommodation.".to_string(),
                    citations,
                    maximum_pattern_or_practice_civil_penalty_dollars: FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS,
                }
            }
        }
        ComplianceAspect::ReasonableModificationUnderSection3604F3A => {
            if input.reasonable_modification_permitted {
                Output {
                    mode: FhaaMode::CompliantReasonableModificationPermitted,
                    statutory_basis: "42 USC § 3604(f)(3)(A) — reasonable modification permitted at handicapped person's expense".to_string(),
                    notes: "COMPLIANT: landlord permitted requested reasonable modification of premises at the handicapped person's expense; landlord may condition modification on renter's agreement to restore interior to pre-modification condition where reasonable.".to_string(),
                    citations,
                    maximum_pattern_or_practice_civil_penalty_dollars: FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS,
                }
            } else {
                Output {
                    mode: FhaaMode::ViolationFailureToPermitReasonableModification,
                    statutory_basis: "42 USC § 3604(f)(3)(A) — failure to permit reasonable modification".to_string(),
                    notes: "VIOLATION: landlord refused to permit reasonable modification of premises at handicapped person's expense; § 3604(f)(3)(A) requires permission absent legitimate basis.".to_string(),
                    citations,
                    maximum_pattern_or_practice_civil_penalty_dollars: FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS,
                }
            }
        }
        ComplianceAspect::DesignAndConstructionRequirementsUnderSection3604F3C => match input.covered_multifamily_status {
            CoveredMultifamilyStatus::CoveredMultifamilyDwellingFirstOccupancyAfterMarch13_1991 => {
                if input.design_and_construction_requirements_met {
                    Output {
                        mode: FhaaMode::CompliantDesignAndConstructionRequirementsForCoveredMultifamilyDwelling,
                        statutory_basis: "42 USC § 3604(f)(3)(C) — covered multifamily dwelling design and construction requirements met".to_string(),
                        notes: "COMPLIANT: covered multifamily dwelling first occupied after March 13, 1991 satisfies all § 3604(f)(3)(C) design and construction requirements (accessible public and common use areas; wheelchair-passable doors; accessible route; accessible environmental controls; bathroom wall reinforcements for grab bars; usable kitchens and bathrooms with wheelchair maneuvering space).".to_string(),
                        citations,
                        maximum_pattern_or_practice_civil_penalty_dollars: FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS,
                    }
                } else {
                    Output {
                        mode: FhaaMode::ViolationDesignAndConstructionRequirementsNotMet,
                        statutory_basis: "42 USC § 3604(f)(3)(C) — covered multifamily dwelling design and construction requirements not met".to_string(),
                        notes: "VIOLATION: covered multifamily dwelling first occupied after March 13, 1991 fails to satisfy § 3604(f)(3)(C) design and construction requirements; retrofit may be required + civil penalties up to $100,000 per pattern-or-practice violation under § 3614.".to_string(),
                        citations,
                        maximum_pattern_or_practice_civil_penalty_dollars: FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS,
                    }
                }
            }
            CoveredMultifamilyStatus::CoveredMultifamilyDwellingFirstOccupancyOnOrBeforeMarch13_1991PreEffectiveDate => Output {
                mode: FhaaMode::NotApplicablePreEffectiveDateMarch12_1989,
                statutory_basis: "42 USC § 3604(f)(3)(C) — design and construction requirements apply only to dwellings first occupied after March 13, 1991".to_string(),
                notes: "NOT APPLICABLE: covered multifamily dwelling first occupied on or before March 13, 1991; pre-effective-date dwelling; design and construction requirements do not apply (reasonable accommodation and reasonable modification under § 3604(f)(3)(A) and (B) continue to apply).".to_string(),
                citations,
                maximum_pattern_or_practice_civil_penalty_dollars: FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS,
            },
            CoveredMultifamilyStatus::NotCoveredMultifamilyDwellingExempt => Output {
                mode: FhaaMode::CompliantNoDiscriminationOnAnyProtectedClass,
                statutory_basis: "42 USC § 3604(f)(7) — dwelling not within covered multifamily definition".to_string(),
                notes: "NOT TRIGGERED: dwelling is not covered multifamily (fewer than 4 units, or no elevator and not ground floor); § 3604(f)(3)(C) design and construction requirements do not apply.".to_string(),
                citations,
                maximum_pattern_or_practice_civil_penalty_dollars: FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS,
            },
        },
        ComplianceAspect::FamilialStatusProtectionsForFamiliesWithChildrenUnder18 => match input.discriminatory_action_status {
            DiscriminatoryActionStatus::NoDiscriminatoryActionTaken => Output {
                mode: FhaaMode::CompliantFamilialStatusProtectionsApplied,
                statutory_basis: "42 USC § 3604(a)-(c) + § 3602(k) — familial status protections applied".to_string(),
                notes: "COMPLIANT: no familial status discrimination against family with children under 18; pregnant person; or person securing legal custody.".to_string(),
                citations,
                maximum_pattern_or_practice_civil_penalty_dollars: FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS,
            },
            _ => Output {
                mode: FhaaMode::ViolationDiscriminationOnBasisOfFamilialStatus,
                statutory_basis: "42 USC § 3604(a)-(c) — familial status discrimination prohibited".to_string(),
                notes: "VIOLATION: discriminatory action taken against family with children under 18; familial status protection added by FHAA 1988.".to_string(),
                citations,
                maximum_pattern_or_practice_civil_penalty_dollars: FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS,
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            property_type: PropertyType::CoveredHousingUnderFha,
            action_date_satisfies_general_effective_date: true,
            compliance_aspect: ComplianceAspect::NondiscriminationOnAnyProtectedClassUnderSection3604,
            protected_class_at_issue: ProtectedClassAtIssue::NoProtectedClassDiscriminationAtIssue,
            discriminatory_action_status: DiscriminatoryActionStatus::NoDiscriminatoryActionTaken,
            covered_multifamily_status: CoveredMultifamilyStatus::NotCoveredMultifamilyDwellingExempt,
            design_and_construction_requirements_met: true,
            reasonable_accommodation_provided: true,
            reasonable_modification_permitted: true,
        }
    }

    #[test]
    fn not_housing_not_applicable() {
        let mut input = baseline_input();
        input.property_type = PropertyType::NotHousingOutsideFhaScope;
        let output = check(&input);
        assert_eq!(output.mode, FhaaMode::NotApplicableNotHousingOutsideFhaScope);
    }

    #[test]
    fn pre_effective_date_march_12_1989_not_applicable() {
        let mut input = baseline_input();
        input.action_date_satisfies_general_effective_date = false;
        let output = check(&input);
        assert_eq!(output.mode, FhaaMode::NotApplicablePreEffectiveDateMarch12_1989);
    }

    #[test]
    fn single_family_dwelling_exemption() {
        let mut input = baseline_input();
        input.property_type = PropertyType::SingleFamilyDwellingExemptUnderSection3603B1;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FhaaMode::NotApplicableSingleFamilyDwellingExemptUnderSection3603B1
        );
    }

    #[test]
    fn mrs_murphy_exemption() {
        let mut input = baseline_input();
        input.property_type = PropertyType::MrsMurphyExemptUnderSection3603B2;
        let output = check(&input);
        assert_eq!(output.mode, FhaaMode::NotApplicableMrsMurphyExemptUnderSection3603B2);
    }

    #[test]
    fn senior_housing_hopa_exemption() {
        let mut input = baseline_input();
        input.property_type = PropertyType::SeniorHousingExemptUnderHopaSection3607;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FhaaMode::NotApplicableSeniorHousingExemptUnderHopaSection3607
        );
    }

    #[test]
    fn no_discrimination_compliant() {
        let output = check(&baseline_input());
        assert_eq!(output.mode, FhaaMode::CompliantNoDiscriminationOnAnyProtectedClass);
    }

    #[test]
    fn race_discrimination_violation() {
        let mut input = baseline_input();
        input.protected_class_at_issue = ProtectedClassAtIssue::Race;
        let output = check(&input);
        assert_eq!(output.mode, FhaaMode::ViolationDiscriminationOnBasisOfRace);
    }

    #[test]
    fn handicap_discrimination_violation() {
        let mut input = baseline_input();
        input.protected_class_at_issue = ProtectedClassAtIssue::HandicapDisabilityAddedByFhaa;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FhaaMode::ViolationDiscriminationOnBasisOfHandicapDisability
        );
    }

    #[test]
    fn familial_status_discrimination_violation() {
        let mut input = baseline_input();
        input.protected_class_at_issue = ProtectedClassAtIssue::FamilialStatusAddedByFhaa;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FhaaMode::ViolationDiscriminationOnBasisOfFamilialStatus
        );
    }

    #[test]
    fn reasonable_accommodation_provided_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ReasonableAccommodationUnderSection3604F3B;
        let output = check(&input);
        assert_eq!(output.mode, FhaaMode::CompliantReasonableAccommodationProvided);
    }

    #[test]
    fn reasonable_accommodation_denied_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ReasonableAccommodationUnderSection3604F3B;
        input.reasonable_accommodation_provided = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FhaaMode::ViolationFailureToProvideReasonableAccommodation
        );
    }

    #[test]
    fn reasonable_modification_permitted_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ReasonableModificationUnderSection3604F3A;
        let output = check(&input);
        assert_eq!(output.mode, FhaaMode::CompliantReasonableModificationPermitted);
    }

    #[test]
    fn reasonable_modification_denied_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ReasonableModificationUnderSection3604F3A;
        input.reasonable_modification_permitted = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FhaaMode::ViolationFailureToPermitReasonableModification
        );
    }

    #[test]
    fn design_and_construction_post_march_13_1991_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DesignAndConstructionRequirementsUnderSection3604F3C;
        input.covered_multifamily_status =
            CoveredMultifamilyStatus::CoveredMultifamilyDwellingFirstOccupancyAfterMarch13_1991;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FhaaMode::CompliantDesignAndConstructionRequirementsForCoveredMultifamilyDwelling
        );
    }

    #[test]
    fn design_and_construction_post_march_13_1991_not_met_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DesignAndConstructionRequirementsUnderSection3604F3C;
        input.covered_multifamily_status =
            CoveredMultifamilyStatus::CoveredMultifamilyDwellingFirstOccupancyAfterMarch13_1991;
        input.design_and_construction_requirements_met = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FhaaMode::ViolationDesignAndConstructionRequirementsNotMet
        );
    }

    #[test]
    fn design_and_construction_pre_march_13_1991_not_applicable() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::DesignAndConstructionRequirementsUnderSection3604F3C;
        input.covered_multifamily_status =
            CoveredMultifamilyStatus::CoveredMultifamilyDwellingFirstOccupancyOnOrBeforeMarch13_1991PreEffectiveDate;
        let output = check(&input);
        assert_eq!(output.mode, FhaaMode::NotApplicablePreEffectiveDateMarch12_1989);
    }

    #[test]
    fn familial_status_protections_applied_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FamilialStatusProtectionsForFamiliesWithChildrenUnder18;
        let output = check(&input);
        assert_eq!(output.mode, FhaaMode::CompliantFamilialStatusProtectionsApplied);
    }

    #[test]
    fn familial_status_discrimination_under_compliance_aspect_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::FamilialStatusProtectionsForFamiliesWithChildrenUnder18;
        input.discriminatory_action_status =
            DiscriminatoryActionStatus::DiscriminatoryActionTakenAgainstProtectedClass;
        let output = check(&input);
        assert_eq!(
            output.mode,
            FhaaMode::ViolationDiscriminationOnBasisOfFamilialStatus
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(FHAA_ENACTMENT_YEAR, 1988);
        assert_eq!(FHAA_ENACTMENT_MONTH, 9);
        assert_eq!(FHAA_ENACTMENT_DAY, 13);
        assert_eq!(FHAA_GENERAL_EFFECTIVE_DATE_YEAR, 1989);
        assert_eq!(FHAA_GENERAL_EFFECTIVE_DATE_MONTH, 3);
        assert_eq!(FHAA_GENERAL_EFFECTIVE_DATE_DAY, 12);
        assert_eq!(FHAA_DESIGN_CONSTRUCTION_EFFECTIVE_DATE_YEAR, 1991);
        assert_eq!(FHAA_DESIGN_CONSTRUCTION_EFFECTIVE_DATE_MONTH, 3);
        assert_eq!(FHAA_DESIGN_CONSTRUCTION_EFFECTIVE_DATE_DAY, 13);
        assert_eq!(FHAA_COVERED_MULTIFAMILY_MIN_UNITS, 4);
        assert_eq!(FHAA_DESIGN_CONSTRUCTION_LAG_MONTHS, 30);
        assert_eq!(FHAA_FAMILIAL_STATUS_AGE_THRESHOLD_YEARS, 18);
        assert_eq!(FHAA_SINGLE_FAMILY_EXEMPTION_MAX_HOMES, 3);
        assert_eq!(FHAA_SINGLE_FAMILY_EXEMPTION_MAX_SALES_PER_24_MONTHS, 1);
        assert_eq!(FHAA_MRS_MURPHY_EXEMPTION_MAX_FAMILIES, 4);
        assert_eq!(FHAA_PATTERN_OR_PRACTICE_CIVIL_PENALTY_DOLLARS, 100_000);
        assert_eq!(FHAA_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citation_contains_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("Fair Housing Amendments Act of 1988"));
        assert!(joined.contains("Public Law 100-430"));
        assert!(joined.contains("September 13, 1988"));
        assert!(joined.contains("March 12, 1989"));
        assert!(joined.contains("March 13, 1991"));
        assert!(joined.contains("42 USC §§ 3601-3619"));
        assert!(joined.contains("42 USC § 3604"));
        assert!(joined.contains("42 USC § 3602"));
        assert!(joined.contains("§ 3604(f)(3)(A)"));
        assert!(joined.contains("§ 3604(f)(3)(B)"));
        assert!(joined.contains("§ 3604(f)(3)(C)"));
        assert!(joined.contains("§ 3604(f)(7)"));
        assert!(joined.contains("§ 3603(b)(1)"));
        assert!(joined.contains("§ 3603(b)(2)"));
        assert!(joined.contains("HANDICAP"));
        assert!(joined.contains("FAMILIAL STATUS"));
        assert!(joined.contains("$100,000"));
    }
}
