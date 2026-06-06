//! HUD Section 504 of the Rehabilitation Act of 1973 /
//! 24 CFR Part 8 Compliance Module — federal nondiscrimination
//! on the basis of disability in federally assisted housing
//! programs and activities.
//!
//! Pure-compute check for landlord compliance with Section
//! 504 of the Rehabilitation Act of 1973 (29 USC § 794) and
//! its HUD implementing regulations at 24 CFR Part 8
//! (Nondiscrimination Based on Handicap in Federally
//! Assisted Programs and Activities of the Department of
//! Housing and Urban Development). § 504 prohibits
//! discrimination on the basis of disability by federally
//! assisted housing recipients (Section 8 Housing Choice
//! Voucher operators, Section 8 project-based rental
//! assistance owners, Public Housing Authorities, HOPWA,
//! Section 202 Supportive Housing for the Elderly, Section
//! 811 Supportive Housing for Persons with Disabilities,
//! and certain LIHTC properties with HUD-administered
//! financing layers). The regulations were substantially
//! updated by the HUD final rule published in the Federal
//! Register on April 25, 2023.
//!
//! Web research (verified 2026-06-03):
//! - **Statutory Basis**: Section 504 of the Rehabilitation Act of 1973 (Public Law 93-112), codified at **29 USC § 794**; implementing regulations at **24 CFR Part 8** (HUD nondiscrimination based on handicap in federally assisted programs) ([HUD Section 504 program page](https://www.hud.gov/504); [eCFR 24 CFR Part 8](https://www.ecfr.gov/current/title-24/subtitle-A/part-8); [HUD CPD-N-05-09 Accessibility Notice](https://www.hud.gov/sites/documents/05-09cpdn.doc); [HUD HSGN-12-27 Housing Notice](https://www.hud.gov/sites/documents/12-27hsgn.pdf); [Buchanan Ingersoll & Rooney — HUD Seeks Input on Section 504 Regulation Changes](https://www.bipc.com/hud-seeks-input-on-changes-to-implementing-regulations-for-section-504-nondiscrimination-on-the-basis-of-disability); [Federal Register — Nondiscrimination on the Basis of Disability: Updates to HUD's Section 504 Regulations (April 25, 2023)](https://www.federalregister.gov/documents/2023/04/25/2023-08464/nondiscrimination-on-the-basis-of-disability-updates-to-huds-section-504-regulations); [Corada — 24 CFR Part 8 Use of Alt Accessibility Standard](https://www.corada.com/documents/24-cfr-part-8-use-of-alt-accessibility-standard/24-cfr-part-8-nondiscrimination-on-the-basis-of-disability-in-federally-assisted-programs-and-activities-notice-instructions-for-use-of-alternative-accessibility-standard)).
//! - **New Construction Accessibility (24 CFR § 8.22)**: new federally assisted multifamily housing projects must provide a minimum of **5 PERCENT of total units (or at least one unit, whichever is greater) READILY ACCESSIBLE to persons with MOBILITY DISABILITIES**, plus an **ADDITIONAL MINIMUM 2 PERCENT of total units (or at least one unit, whichever is greater) READILY ACCESSIBLE to persons with HEARING and VISION (SENSORY) DISABILITIES**. The cumulative threshold therefore represents 7 % of units distributed across mobility (5 %) and sensory (2 %) categories.
//! - **Substantial Alteration / Rehabilitation (24 CFR § 8.23)**: substantial alterations to existing federally assisted housing must comply with the new-construction accessibility standards under § 8.22 (i.e., to the maximum extent feasible, the altered portion must meet the 5 % mobility + 2 % sensory thresholds).
//! - **Existing Facilities (24 CFR § 8.24)**: providers of existing assisted housing must operate the housing so that, **WHEN VIEWED IN ITS ENTIRETY**, the program or activity is **READILY ACCESSIBLE TO AND USABLE BY** individuals with disabilities. This is a program-level (not unit-level) accessibility requirement — the recipient may comply through methods such as accessible building entrances, accessible common areas, accessible administrative offices, and reasonable accommodations to specific tenant requests.
//! - **Accessibility Standards (24 CFR § 8.32)**: the **UNIFORM FEDERAL ACCESSIBILITY STANDARDS (UFAS)** is HUD's default Section 504 federal accessibility standard. As of **March 2011** under U.S. Department of Justice guidance, the **2010 ADA Standards for Accessible Design** are permitted as an **ACCEPTABLE ALTERNATIVE** to UFAS for federally assisted housing.
//! - **Reasonable Accommodation Requirement (24 CFR § 8.20)**: recipients must make reasonable accommodations — both structural modifications to facilities AND adjustments or exceptions to policies and practices — necessary for an individual with a disability to equally participate in or benefit from federally assisted programs without discrimination.
//! - **Small Recipient Exception**: a recipient with FEWER THAN 15 EMPLOYEES is generally exempt from the affirmative procedural obligations (designated responsible employee, self-evaluation, grievance procedures), but the substantive nondiscrimination obligations continue to apply regardless of recipient size.
//! - **Designated Responsible Employee (24 CFR § 8.53)**: recipients with 15 or more employees must designate at least one employee to coordinate compliance with § 504 obligations; the designation must include the responsible employee's name, address, and telephone number in the recipient's notice of nondiscrimination.
//! - **Self-Evaluation (24 CFR § 8.51)**: recipients with 15 or more employees must conduct a self-evaluation of current policies and practices to identify and modify any that do not comply with § 504; results must be made available for public inspection for at least 3 years.
//! - **Grievance Procedures (24 CFR § 8.53)**: recipients with 15 or more employees must adopt grievance procedures providing for prompt and equitable resolution of complaints alleging actions prohibited by § 504.
//! - **Notice of Nondiscrimination (24 CFR § 8.54)**: recipients must take continuing steps to notify participants, beneficiaries, applicants, employees, and unions or professional organizations of the recipient's nondiscrimination policy; the notice must include the name and contact information of the designated responsible employee (if applicable).
//! - **April 25, 2023 HUD Final Rule Update**: HUD published a comprehensive update to 24 CFR Part 8 on April 25, 2023, incorporating modern accessibility technology guidance, updated 2010 ADA Standards references, expanded reasonable accommodation framework, and enhanced enforcement procedures.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const HUD_SECTION_504_ENACTMENT_YEAR: u32 = 1973;
pub const HUD_SECTION_504_REGULATION_TITLE: u32 = 24;
pub const HUD_SECTION_504_REGULATION_PART: u32 = 8;
pub const HUD_SECTION_504_NEW_CONSTRUCTION_MOBILITY_ACCESSIBILITY_PERCENTAGE_BASIS_POINTS: u64 =
    500;
pub const HUD_SECTION_504_NEW_CONSTRUCTION_SENSORY_ACCESSIBILITY_PERCENTAGE_BASIS_POINTS: u64 = 200;
pub const HUD_SECTION_504_LARGE_RECIPIENT_EMPLOYEE_THRESHOLD: u32 = 15;
pub const HUD_SECTION_504_2010_ADA_STANDARDS_AVAILABLE_AS_ALTERNATIVE_SINCE_YEAR: u32 = 2011;
pub const HUD_SECTION_504_2023_REGULATION_UPDATE_YEAR: u32 = 2023;
pub const HUD_SECTION_504_2023_REGULATION_UPDATE_MONTH: u32 = 4;
pub const HUD_SECTION_504_2023_REGULATION_UPDATE_DAY: u32 = 25;
pub const HUD_SECTION_504_SELF_EVALUATION_RETENTION_YEARS: u32 = 3;
pub const HUD_SECTION_504_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FederallyAssistedStatus {
    FederallyAssistedHousingRecipient,
    NotFederallyAssistedHousingExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    NewConstructionUnderSection822,
    SubstantialAlterationOrRehabilitationUnderSection823,
    ExistingFacilityUnderSection824,
    NotApplicableProjectStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessibilityStandard {
    UniformFederalAccessibilityStandardsUfas,
    AdaStandards2010AlternativeUnderDojMarch2011Guidance,
    NeitherUfasNor2010AdaStandards,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReasonableAccommodationStatus {
    AccommodationProvided,
    AccommodationDeniedWithoutLegitimateBasis,
    NoAccommodationRequested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    NewConstructionAccessibilityThresholdsUnderSection822,
    SubstantialAlterationAccessibilityUnderSection823,
    ExistingFacilityReadilyAccessibleWhenViewedInItsEntiretyUnderSection824,
    ReasonableAccommodationUnderSection820,
    DesignatedResponsibleEmployeeRequirementUnderSection853,
    SelfEvaluationRequirementUnderSection851,
    GrievanceProceduresRequirementUnderSection853,
    NoticeOfNondiscriminationRequirementUnderSection854,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HudSection504Mode {
    NotApplicableNotFederallyAssistedHousing,
    NotApplicableSmallRecipientUnderFifteenEmployees,
    CompliantNewConstruction5PctMobilityAnd2PctSensoryAccessibleUnitsProvided,
    CompliantSubstantialAlterationMeetsAccessibilityStandards,
    CompliantExistingFacilityReadilyAccessibleWhenViewedInItsEntirety,
    CompliantReasonableAccommodationProvidedUnderSection820,
    CompliantDesignatedResponsibleEmployeePresentUnderSection853,
    CompliantSelfEvaluationCompletedUnderSection851,
    CompliantGrievanceProceduresMaintainedUnderSection853,
    CompliantNoticeOfNondiscriminationPostedUnderSection854,
    CompliantUsingUfasAccessibilityStandard,
    Compliant2010AdaStandardsAlternativeAccepted,
    ViolationNewConstructionBelow5PctMobilityAccessibleUnits,
    ViolationNewConstructionBelow2PctSensoryAccessibleUnits,
    ViolationSubstantialAlterationNotMeetingAccessibilityStandards,
    ViolationExistingFacilityNotReadilyAccessibleWhenViewedInEntirety,
    ViolationReasonableAccommodationDeniedWithoutLegitimateBasis,
    ViolationLargeRecipientNoDesignatedResponsibleEmployee,
    ViolationLargeRecipientNoSelfEvaluation,
    ViolationLargeRecipientNoGrievanceProcedures,
    ViolationNoticeOfNondiscriminationNotPosted,
    ViolationNeitherUfasNor2010AdaStandardsApplied,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub federally_assisted_status: FederallyAssistedStatus,
    pub project_status: ProjectStatus,
    pub compliance_aspect: ComplianceAspect,
    pub total_units_at_property: u32,
    pub mobility_accessible_units_count: u32,
    pub sensory_accessible_units_count: u32,
    pub accessibility_standard: AccessibilityStandard,
    pub reasonable_accommodation_status: ReasonableAccommodationStatus,
    pub recipient_employee_count: u32,
    pub designated_responsible_employee_present: bool,
    pub self_evaluation_completed: bool,
    pub grievance_procedures_maintained: bool,
    pub notice_of_nondiscrimination_posted: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: HudSection504Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub required_mobility_accessible_units: u32,
    pub required_sensory_accessible_units: u32,
}

pub type RentalHudSection504RehabilitationAct24CfrPart8Input = Input;
pub type RentalHudSection504RehabilitationAct24CfrPart8Output = Output;
pub type RentalHudSection504RehabilitationAct24CfrPart8Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Section 504 of the Rehabilitation Act of 1973 (Public Law 93-112) — codified at 29 USC § 794; prohibits discrimination on basis of disability by federally assisted housing recipients (Section 8 Housing Choice Voucher operators, Section 8 PBRA owners, PHAs, HOPWA, Section 202 elderly housing, Section 811 disabled housing, and certain LIHTC properties with HUD financing layers)".to_string(),
        "24 CFR Part 8 — HUD implementing regulations (Nondiscrimination Based on Handicap in Federally Assisted Programs and Activities of the Department of Housing and Urban Development)".to_string(),
        "24 CFR § 8.22 New Construction Accessibility — new federally assisted multifamily housing projects must provide minimum 5 PERCENT of total units (or at least one unit, whichever is greater) READILY ACCESSIBLE to persons with MOBILITY DISABILITIES + ADDITIONAL minimum 2 PERCENT of total units (or at least one unit, whichever is greater) READILY ACCESSIBLE to persons with HEARING and VISION (SENSORY) DISABILITIES; cumulative 7 % accessibility threshold".to_string(),
        "24 CFR § 8.23 Substantial Alteration — substantial alterations to existing federally assisted housing must comply with new-construction accessibility standards under § 8.22 (to maximum extent feasible)".to_string(),
        "24 CFR § 8.24 Existing Facilities — providers of existing assisted housing must operate housing so that, WHEN VIEWED IN ITS ENTIRETY, program or activity is READILY ACCESSIBLE TO AND USABLE BY individuals with disabilities; program-level (not unit-level) accessibility requirement".to_string(),
        "24 CFR § 8.32 Accessibility Standards — Uniform Federal Accessibility Standards (UFAS) is HUD's default Section 504 federal accessibility standard; as of March 2011 under U.S. Department of Justice guidance, 2010 ADA Standards for Accessible Design permitted as ACCEPTABLE ALTERNATIVE to UFAS".to_string(),
        "24 CFR § 8.20 Reasonable Accommodation Requirement — recipients must make reasonable accommodations including structural modifications to facilities AND adjustments or exceptions to policies and practices necessary for individual with disability to equally participate in or benefit from federally assisted programs".to_string(),
        "Small Recipient Exception — recipient with FEWER THAN 15 EMPLOYEES generally exempt from affirmative procedural obligations (designated responsible employee, self-evaluation, grievance procedures), BUT substantive nondiscrimination obligations continue to apply regardless of size".to_string(),
        "24 CFR § 8.53 Designated Responsible Employee — recipients with 15 or more employees must designate at least one employee to coordinate § 504 compliance; designation must include name, address, telephone number in recipient's notice of nondiscrimination".to_string(),
        "24 CFR § 8.51 Self-Evaluation — recipients with 15 or more employees must conduct self-evaluation of current policies and practices to identify and modify any that do not comply with § 504; results made available for public inspection for at least 3 years".to_string(),
        "24 CFR § 8.53 Grievance Procedures — recipients with 15 or more employees must adopt grievance procedures providing for prompt and equitable resolution of complaints alleging actions prohibited by § 504".to_string(),
        "24 CFR § 8.54 Notice of Nondiscrimination — recipients must take continuing steps to notify participants, beneficiaries, applicants, employees, and unions of recipient's nondiscrimination policy; notice must include name and contact information of designated responsible employee".to_string(),
        "April 25, 2023 HUD Final Rule Update — comprehensive update to 24 CFR Part 8 incorporating modern accessibility technology guidance, updated 2010 ADA Standards references, expanded reasonable accommodation framework, enhanced enforcement procedures".to_string(),
        "HUD Section 504 program page — primary HUD compliance guidance".to_string(),
        "HUD CPD-N-05-09 Accessibility Notice — HUD CPD program guidance on Section 504".to_string(),
        "HUD HSGN-12-27 Housing Notice — HUD multifamily housing notice on Section 504".to_string(),
        "Federal Register — Nondiscrimination on the Basis of Disability: Updates to HUD's Section 504 Regulations (April 25, 2023)".to_string(),
        "Buchanan Ingersoll & Rooney — HUD Seeks Input on Changes to Implementing Regulations for Section 504 practitioner guide".to_string(),
        "Corada — 24 CFR Part 8 Use of Alt Accessibility Standard guidance".to_string(),
    ];

    if input.federally_assisted_status == FederallyAssistedStatus::NotFederallyAssistedHousingExempt
    {
        return Output {
            mode: HudSection504Mode::NotApplicableNotFederallyAssistedHousing,
            statutory_basis: "29 USC § 794 + 24 CFR Part 8 — Section 504 applies only to federally assisted housing recipients".to_string(),
            notes: "NOT APPLICABLE: housing is not federally assisted; Section 504 of Rehabilitation Act of 1973 and 24 CFR Part 8 do not apply; Fair Housing Act § 504-analog disability protections under 42 USC § 3604(f) continue to apply to non-federally-assisted housing.".to_string(),
            citations,
            required_mobility_accessible_units: 0,
            required_sensory_accessible_units: 0,
        };
    }

    let required_mobility_accessible = compute_required_accessible_units(
        input.total_units_at_property,
        HUD_SECTION_504_NEW_CONSTRUCTION_MOBILITY_ACCESSIBILITY_PERCENTAGE_BASIS_POINTS,
    );
    let required_sensory_accessible = compute_required_accessible_units(
        input.total_units_at_property,
        HUD_SECTION_504_NEW_CONSTRUCTION_SENSORY_ACCESSIBILITY_PERCENTAGE_BASIS_POINTS,
    );

    let is_small_recipient =
        input.recipient_employee_count < HUD_SECTION_504_LARGE_RECIPIENT_EMPLOYEE_THRESHOLD;

    match input.compliance_aspect {
        ComplianceAspect::NewConstructionAccessibilityThresholdsUnderSection822 => {
            if input.project_status != ProjectStatus::NewConstructionUnderSection822 {
                return Output {
                    mode: HudSection504Mode::CompliantExistingFacilityReadilyAccessibleWhenViewedInItsEntirety,
                    statutory_basis: "24 CFR § 8.22 — new-construction accessibility thresholds apply only to new construction projects".to_string(),
                    notes: "NOT APPLICABLE TO NEW CONSTRUCTION ASPECT: project status is not new construction; § 8.22 thresholds do not apply.".to_string(),
                    citations,
                    required_mobility_accessible_units: required_mobility_accessible,
                    required_sensory_accessible_units: required_sensory_accessible,
                };
            }
            if input.mobility_accessible_units_count < required_mobility_accessible {
                return Output {
                    mode: HudSection504Mode::ViolationNewConstructionBelow5PctMobilityAccessibleUnits,
                    statutory_basis: "24 CFR § 8.22 — new construction must provide 5 % of units (or at least one) mobility-accessible".to_string(),
                    notes: format!(
                        "VIOLATION: new federally assisted construction with {} total units provides only {} mobility-accessible units; § 8.22 requires {} units (greater of 5 % or one unit).",
                        input.total_units_at_property,
                        input.mobility_accessible_units_count,
                        required_mobility_accessible
                    ),
                    citations,
                    required_mobility_accessible_units: required_mobility_accessible,
                    required_sensory_accessible_units: required_sensory_accessible,
                };
            }
            if input.sensory_accessible_units_count < required_sensory_accessible {
                return Output {
                    mode: HudSection504Mode::ViolationNewConstructionBelow2PctSensoryAccessibleUnits,
                    statutory_basis: "24 CFR § 8.22 — new construction must provide additional 2 % of units (or at least one) sensory-accessible".to_string(),
                    notes: format!(
                        "VIOLATION: new federally assisted construction with {} total units provides only {} sensory-accessible units; § 8.22 requires {} additional units (greater of 2 % or one unit) on top of mobility-accessible requirement.",
                        input.total_units_at_property,
                        input.sensory_accessible_units_count,
                        required_sensory_accessible
                    ),
                    citations,
                    required_mobility_accessible_units: required_mobility_accessible,
                    required_sensory_accessible_units: required_sensory_accessible,
                };
            }
            if input.accessibility_standard == AccessibilityStandard::NeitherUfasNor2010AdaStandards {
                return Output {
                    mode: HudSection504Mode::ViolationNeitherUfasNor2010AdaStandardsApplied,
                    statutory_basis: "24 CFR § 8.32 — accessibility must conform to UFAS or 2010 ADA Standards (alternative)".to_string(),
                    notes: "VIOLATION: neither UFAS nor 2010 ADA Standards applied to design; § 8.32 requires conformance with one of the two accepted standards.".to_string(),
                    citations,
                    required_mobility_accessible_units: required_mobility_accessible,
                    required_sensory_accessible_units: required_sensory_accessible,
                };
            }
            Output {
                mode: HudSection504Mode::CompliantNewConstruction5PctMobilityAnd2PctSensoryAccessibleUnitsProvided,
                statutory_basis: "24 CFR § 8.22 + § 8.32 — new construction with 5 % mobility + 2 % sensory accessibility under UFAS or 2010 ADA Standards".to_string(),
                notes: format!(
                    "COMPLIANT: new federally assisted construction with {} total units provides {} mobility-accessible (≥ {} required) + {} sensory-accessible (≥ {} required) units under {:?}.",
                    input.total_units_at_property,
                    input.mobility_accessible_units_count,
                    required_mobility_accessible,
                    input.sensory_accessible_units_count,
                    required_sensory_accessible,
                    input.accessibility_standard
                ),
                citations,
                required_mobility_accessible_units: required_mobility_accessible,
                required_sensory_accessible_units: required_sensory_accessible,
            }
        }
        ComplianceAspect::SubstantialAlterationAccessibilityUnderSection823 => {
            if input.mobility_accessible_units_count >= required_mobility_accessible
                && input.sensory_accessible_units_count >= required_sensory_accessible
            {
                Output {
                    mode: HudSection504Mode::CompliantSubstantialAlterationMeetsAccessibilityStandards,
                    statutory_basis: "24 CFR § 8.23 — substantial alteration meets new-construction accessibility standards under § 8.22".to_string(),
                    notes: "COMPLIANT: substantial alteration to federally assisted housing achieves 5 % mobility + 2 % sensory accessibility thresholds under § 8.22 standards (to maximum extent feasible).".to_string(),
                    citations,
                    required_mobility_accessible_units: required_mobility_accessible,
                    required_sensory_accessible_units: required_sensory_accessible,
                }
            } else {
                Output {
                    mode: HudSection504Mode::ViolationSubstantialAlterationNotMeetingAccessibilityStandards,
                    statutory_basis: "24 CFR § 8.23 — substantial alteration must meet § 8.22 accessibility standards".to_string(),
                    notes: "VIOLATION: substantial alteration to federally assisted housing does not meet § 8.22 5 % mobility + 2 % sensory accessibility thresholds; § 8.23 requires substantial alterations to bring altered portions into compliance with new-construction standards to maximum extent feasible.".to_string(),
                    citations,
                    required_mobility_accessible_units: required_mobility_accessible,
                    required_sensory_accessible_units: required_sensory_accessible,
                }
            }
        }
        ComplianceAspect::ExistingFacilityReadilyAccessibleWhenViewedInItsEntiretyUnderSection824 => {
            // Existing facility program-level accessibility — accept the input designation
            if input.mobility_accessible_units_count > 0 || input.sensory_accessible_units_count > 0
            {
                Output {
                    mode: HudSection504Mode::CompliantExistingFacilityReadilyAccessibleWhenViewedInItsEntirety,
                    statutory_basis: "24 CFR § 8.24 — existing facility readily accessible when viewed in its entirety".to_string(),
                    notes: "COMPLIANT: existing federally assisted facility operated so that, when viewed in its entirety, program or activity is readily accessible to and usable by individuals with disabilities (program-level accessibility under § 8.24).".to_string(),
                    citations,
                    required_mobility_accessible_units: required_mobility_accessible,
                    required_sensory_accessible_units: required_sensory_accessible,
                }
            } else {
                Output {
                    mode: HudSection504Mode::ViolationExistingFacilityNotReadilyAccessibleWhenViewedInEntirety,
                    statutory_basis: "24 CFR § 8.24 — existing facility must be readily accessible when viewed in its entirety".to_string(),
                    notes: "VIOLATION: existing federally assisted facility has no mobility-accessible or sensory-accessible units; § 8.24 requires program-level accessibility through methods such as accessible entrances, common areas, administrative offices, and reasonable accommodations.".to_string(),
                    citations,
                    required_mobility_accessible_units: required_mobility_accessible,
                    required_sensory_accessible_units: required_sensory_accessible,
                }
            }
        }
        ComplianceAspect::ReasonableAccommodationUnderSection820 => match input.reasonable_accommodation_status {
            ReasonableAccommodationStatus::AccommodationProvided => Output {
                mode: HudSection504Mode::CompliantReasonableAccommodationProvidedUnderSection820,
                statutory_basis: "24 CFR § 8.20 — reasonable accommodation provided".to_string(),
                notes: "COMPLIANT: recipient provided requested reasonable accommodation (structural modification, policy adjustment, or exception) necessary for individual with disability to equally participate in or benefit from federally assisted program.".to_string(),
                citations,
                required_mobility_accessible_units: required_mobility_accessible,
                required_sensory_accessible_units: required_sensory_accessible,
            },
            ReasonableAccommodationStatus::AccommodationDeniedWithoutLegitimateBasis => Output {
                mode: HudSection504Mode::ViolationReasonableAccommodationDeniedWithoutLegitimateBasis,
                statutory_basis: "24 CFR § 8.20 — reasonable accommodation must be provided absent fundamental alteration or undue financial / administrative burden".to_string(),
                notes: "VIOLATION: recipient denied requested reasonable accommodation without demonstrating fundamental program alteration or undue financial / administrative burden; § 8.20 requires provision of reasonable accommodation.".to_string(),
                citations,
                required_mobility_accessible_units: required_mobility_accessible,
                required_sensory_accessible_units: required_sensory_accessible,
            },
            ReasonableAccommodationStatus::NoAccommodationRequested => Output {
                mode: HudSection504Mode::CompliantReasonableAccommodationProvidedUnderSection820,
                statutory_basis: "24 CFR § 8.20 — reasonable accommodation triggered by request".to_string(),
                notes: "NOT TRIGGERED: no reasonable accommodation requested; § 8.20 reasonable-accommodation duty is request-triggered.".to_string(),
                citations,
                required_mobility_accessible_units: required_mobility_accessible,
                required_sensory_accessible_units: required_sensory_accessible,
            },
        },
        ComplianceAspect::DesignatedResponsibleEmployeeRequirementUnderSection853 => {
            if is_small_recipient {
                Output {
                    mode: HudSection504Mode::NotApplicableSmallRecipientUnderFifteenEmployees,
                    statutory_basis: "24 CFR § 8.53 — designated responsible employee requirement applies only to recipients with 15 or more employees".to_string(),
                    notes: format!(
                        "NOT APPLICABLE: recipient has {} employees (< 15-employee threshold); § 8.53 designated-responsible-employee requirement does not apply; substantive nondiscrimination obligations continue to apply.",
                        input.recipient_employee_count
                    ),
                    citations,
                    required_mobility_accessible_units: required_mobility_accessible,
                    required_sensory_accessible_units: required_sensory_accessible,
                }
            } else if input.designated_responsible_employee_present {
                Output {
                    mode: HudSection504Mode::CompliantDesignatedResponsibleEmployeePresentUnderSection853,
                    statutory_basis: "24 CFR § 8.53 — designated responsible employee present".to_string(),
                    notes: "COMPLIANT: recipient with 15 or more employees has designated at least one employee to coordinate § 504 compliance; designation includes name, address, telephone number in notice of nondiscrimination.".to_string(),
                    citations,
                    required_mobility_accessible_units: required_mobility_accessible,
                    required_sensory_accessible_units: required_sensory_accessible,
                }
            } else {
                Output {
                    mode: HudSection504Mode::ViolationLargeRecipientNoDesignatedResponsibleEmployee,
                    statutory_basis: "24 CFR § 8.53 — recipient with 15 or more employees must designate responsible employee".to_string(),
                    notes: "VIOLATION: recipient has 15 or more employees but has not designated a responsible employee to coordinate § 504 compliance.".to_string(),
                    citations,
                    required_mobility_accessible_units: required_mobility_accessible,
                    required_sensory_accessible_units: required_sensory_accessible,
                }
            }
        }
        ComplianceAspect::SelfEvaluationRequirementUnderSection851 => {
            if is_small_recipient {
                Output {
                    mode: HudSection504Mode::NotApplicableSmallRecipientUnderFifteenEmployees,
                    statutory_basis: "24 CFR § 8.51 — self-evaluation requirement applies only to recipients with 15 or more employees".to_string(),
                    notes: format!(
                        "NOT APPLICABLE: recipient has {} employees (< 15-employee threshold); § 8.51 self-evaluation requirement does not apply.",
                        input.recipient_employee_count
                    ),
                    citations,
                    required_mobility_accessible_units: required_mobility_accessible,
                    required_sensory_accessible_units: required_sensory_accessible,
                }
            } else if input.self_evaluation_completed {
                Output {
                    mode: HudSection504Mode::CompliantSelfEvaluationCompletedUnderSection851,
                    statutory_basis: "24 CFR § 8.51 — self-evaluation completed".to_string(),
                    notes: "COMPLIANT: recipient with 15 or more employees has conducted self-evaluation of current policies and practices to identify and modify any that do not comply with § 504; results available for public inspection for at least 3 years.".to_string(),
                    citations,
                    required_mobility_accessible_units: required_mobility_accessible,
                    required_sensory_accessible_units: required_sensory_accessible,
                }
            } else {
                Output {
                    mode: HudSection504Mode::ViolationLargeRecipientNoSelfEvaluation,
                    statutory_basis: "24 CFR § 8.51 — recipient with 15 or more employees must conduct self-evaluation".to_string(),
                    notes: "VIOLATION: recipient with 15 or more employees has not conducted required § 8.51 self-evaluation of policies and practices.".to_string(),
                    citations,
                    required_mobility_accessible_units: required_mobility_accessible,
                    required_sensory_accessible_units: required_sensory_accessible,
                }
            }
        }
        ComplianceAspect::GrievanceProceduresRequirementUnderSection853 => {
            if is_small_recipient {
                Output {
                    mode: HudSection504Mode::NotApplicableSmallRecipientUnderFifteenEmployees,
                    statutory_basis: "24 CFR § 8.53 — grievance procedures requirement applies only to recipients with 15 or more employees".to_string(),
                    notes: format!(
                        "NOT APPLICABLE: recipient has {} employees (< 15-employee threshold); § 8.53 grievance-procedures requirement does not apply.",
                        input.recipient_employee_count
                    ),
                    citations,
                    required_mobility_accessible_units: required_mobility_accessible,
                    required_sensory_accessible_units: required_sensory_accessible,
                }
            } else if input.grievance_procedures_maintained {
                Output {
                    mode: HudSection504Mode::CompliantGrievanceProceduresMaintainedUnderSection853,
                    statutory_basis: "24 CFR § 8.53 — grievance procedures maintained".to_string(),
                    notes: "COMPLIANT: recipient with 15 or more employees has adopted and maintains grievance procedures providing for prompt and equitable resolution of complaints alleging actions prohibited by § 504.".to_string(),
                    citations,
                    required_mobility_accessible_units: required_mobility_accessible,
                    required_sensory_accessible_units: required_sensory_accessible,
                }
            } else {
                Output {
                    mode: HudSection504Mode::ViolationLargeRecipientNoGrievanceProcedures,
                    statutory_basis: "24 CFR § 8.53 — recipient with 15 or more employees must adopt grievance procedures".to_string(),
                    notes: "VIOLATION: recipient with 15 or more employees has not adopted required § 8.53 grievance procedures.".to_string(),
                    citations,
                    required_mobility_accessible_units: required_mobility_accessible,
                    required_sensory_accessible_units: required_sensory_accessible,
                }
            }
        }
        ComplianceAspect::NoticeOfNondiscriminationRequirementUnderSection854 => {
            if input.notice_of_nondiscrimination_posted {
                Output {
                    mode: HudSection504Mode::CompliantNoticeOfNondiscriminationPostedUnderSection854,
                    statutory_basis: "24 CFR § 8.54 — notice of nondiscrimination posted".to_string(),
                    notes: "COMPLIANT: recipient has taken continuing steps to notify participants, beneficiaries, applicants, employees, and unions of nondiscrimination policy; notice includes contact information of designated responsible employee.".to_string(),
                    citations,
                    required_mobility_accessible_units: required_mobility_accessible,
                    required_sensory_accessible_units: required_sensory_accessible,
                }
            } else {
                Output {
                    mode: HudSection504Mode::ViolationNoticeOfNondiscriminationNotPosted,
                    statutory_basis: "24 CFR § 8.54 — recipient must post notice of nondiscrimination".to_string(),
                    notes: "VIOLATION: recipient has not posted required § 8.54 notice of nondiscrimination; affects all participants, beneficiaries, applicants, employees, and unions.".to_string(),
                    citations,
                    required_mobility_accessible_units: required_mobility_accessible,
                    required_sensory_accessible_units: required_sensory_accessible,
                }
            }
        }
    }
}

fn compute_required_accessible_units(total_units: u32, percentage_basis_points: u64) -> u32 {
    let percentage_units = u128::from(total_units)
        .saturating_mul(u128::from(percentage_basis_points))
        .checked_div(u128::from(HUD_SECTION_504_BASIS_POINT_DENOMINATOR))
        .unwrap_or(0)
        .min(u128::from(u32::MAX)) as u32;
    percentage_units.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_new_construction_input() -> Input {
        Input {
            federally_assisted_status: FederallyAssistedStatus::FederallyAssistedHousingRecipient,
            project_status: ProjectStatus::NewConstructionUnderSection822,
            compliance_aspect:
                ComplianceAspect::NewConstructionAccessibilityThresholdsUnderSection822,
            total_units_at_property: 100,
            mobility_accessible_units_count: 5,
            sensory_accessible_units_count: 2,
            accessibility_standard: AccessibilityStandard::UniformFederalAccessibilityStandardsUfas,
            reasonable_accommodation_status:
                ReasonableAccommodationStatus::NoAccommodationRequested,
            recipient_employee_count: 50,
            designated_responsible_employee_present: true,
            self_evaluation_completed: true,
            grievance_procedures_maintained: true,
            notice_of_nondiscrimination_posted: true,
        }
    }

    #[test]
    fn not_federally_assisted_housing_not_applicable() {
        let mut input = baseline_new_construction_input();
        input.federally_assisted_status =
            FederallyAssistedStatus::NotFederallyAssistedHousingExempt;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HudSection504Mode::NotApplicableNotFederallyAssistedHousing
        );
    }

    #[test]
    fn new_construction_5pct_mobility_2pct_sensory_compliant() {
        let output = check(&baseline_new_construction_input());
        assert_eq!(
            output.mode,
            HudSection504Mode::CompliantNewConstruction5PctMobilityAnd2PctSensoryAccessibleUnitsProvided
        );
        assert_eq!(output.required_mobility_accessible_units, 5);
        assert_eq!(output.required_sensory_accessible_units, 2);
    }

    #[test]
    fn new_construction_below_5pct_mobility_violation() {
        let mut input = baseline_new_construction_input();
        input.mobility_accessible_units_count = 3;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HudSection504Mode::ViolationNewConstructionBelow5PctMobilityAccessibleUnits
        );
    }

    #[test]
    fn new_construction_below_2pct_sensory_violation() {
        let mut input = baseline_new_construction_input();
        input.sensory_accessible_units_count = 1;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HudSection504Mode::ViolationNewConstructionBelow2PctSensoryAccessibleUnits
        );
    }

    #[test]
    fn new_construction_small_project_one_unit_minimum_mobility() {
        let mut input = baseline_new_construction_input();
        input.total_units_at_property = 10;
        input.mobility_accessible_units_count = 1;
        input.sensory_accessible_units_count = 1;
        let output = check(&input);
        // 5% of 10 = 0; or-at-least-one rule → 1 mobility-accessible; 2% of 10 = 0; or-at-least-one rule → 1 sensory-accessible
        assert_eq!(output.required_mobility_accessible_units, 1);
        assert_eq!(output.required_sensory_accessible_units, 1);
        assert_eq!(
            output.mode,
            HudSection504Mode::CompliantNewConstruction5PctMobilityAnd2PctSensoryAccessibleUnitsProvided
        );
    }

    #[test]
    fn new_construction_2010_ada_standards_alternative_compliant() {
        let mut input = baseline_new_construction_input();
        input.accessibility_standard =
            AccessibilityStandard::AdaStandards2010AlternativeUnderDojMarch2011Guidance;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HudSection504Mode::CompliantNewConstruction5PctMobilityAnd2PctSensoryAccessibleUnitsProvided
        );
    }

    #[test]
    fn new_construction_neither_ufas_nor_2010_ada_standards_violation() {
        let mut input = baseline_new_construction_input();
        input.accessibility_standard = AccessibilityStandard::NeitherUfasNor2010AdaStandards;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HudSection504Mode::ViolationNeitherUfasNor2010AdaStandardsApplied
        );
    }

    #[test]
    fn substantial_alteration_compliant() {
        let mut input = baseline_new_construction_input();
        input.project_status = ProjectStatus::SubstantialAlterationOrRehabilitationUnderSection823;
        input.compliance_aspect =
            ComplianceAspect::SubstantialAlterationAccessibilityUnderSection823;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HudSection504Mode::CompliantSubstantialAlterationMeetsAccessibilityStandards
        );
    }

    #[test]
    fn substantial_alteration_below_standards_violation() {
        let mut input = baseline_new_construction_input();
        input.project_status = ProjectStatus::SubstantialAlterationOrRehabilitationUnderSection823;
        input.compliance_aspect =
            ComplianceAspect::SubstantialAlterationAccessibilityUnderSection823;
        input.mobility_accessible_units_count = 2;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HudSection504Mode::ViolationSubstantialAlterationNotMeetingAccessibilityStandards
        );
    }

    #[test]
    fn existing_facility_readily_accessible_compliant() {
        let mut input = baseline_new_construction_input();
        input.project_status = ProjectStatus::ExistingFacilityUnderSection824;
        input.compliance_aspect =
            ComplianceAspect::ExistingFacilityReadilyAccessibleWhenViewedInItsEntiretyUnderSection824;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HudSection504Mode::CompliantExistingFacilityReadilyAccessibleWhenViewedInItsEntirety
        );
    }

    #[test]
    fn existing_facility_no_accessible_units_violation() {
        let mut input = baseline_new_construction_input();
        input.project_status = ProjectStatus::ExistingFacilityUnderSection824;
        input.compliance_aspect =
            ComplianceAspect::ExistingFacilityReadilyAccessibleWhenViewedInItsEntiretyUnderSection824;
        input.mobility_accessible_units_count = 0;
        input.sensory_accessible_units_count = 0;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HudSection504Mode::ViolationExistingFacilityNotReadilyAccessibleWhenViewedInEntirety
        );
    }

    #[test]
    fn reasonable_accommodation_provided_compliant() {
        let mut input = baseline_new_construction_input();
        input.compliance_aspect = ComplianceAspect::ReasonableAccommodationUnderSection820;
        input.reasonable_accommodation_status =
            ReasonableAccommodationStatus::AccommodationProvided;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HudSection504Mode::CompliantReasonableAccommodationProvidedUnderSection820
        );
    }

    #[test]
    fn reasonable_accommodation_denied_violation() {
        let mut input = baseline_new_construction_input();
        input.compliance_aspect = ComplianceAspect::ReasonableAccommodationUnderSection820;
        input.reasonable_accommodation_status =
            ReasonableAccommodationStatus::AccommodationDeniedWithoutLegitimateBasis;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HudSection504Mode::ViolationReasonableAccommodationDeniedWithoutLegitimateBasis
        );
    }

    #[test]
    fn small_recipient_under_15_employees_no_designated_employee_required() {
        let mut input = baseline_new_construction_input();
        input.compliance_aspect =
            ComplianceAspect::DesignatedResponsibleEmployeeRequirementUnderSection853;
        input.recipient_employee_count = 10;
        input.designated_responsible_employee_present = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HudSection504Mode::NotApplicableSmallRecipientUnderFifteenEmployees
        );
    }

    #[test]
    fn large_recipient_15_employees_at_threshold_designated_employee_required() {
        let mut input = baseline_new_construction_input();
        input.compliance_aspect =
            ComplianceAspect::DesignatedResponsibleEmployeeRequirementUnderSection853;
        input.recipient_employee_count = 15;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HudSection504Mode::CompliantDesignatedResponsibleEmployeePresentUnderSection853
        );
    }

    #[test]
    fn large_recipient_no_designated_employee_violation() {
        let mut input = baseline_new_construction_input();
        input.compliance_aspect =
            ComplianceAspect::DesignatedResponsibleEmployeeRequirementUnderSection853;
        input.designated_responsible_employee_present = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HudSection504Mode::ViolationLargeRecipientNoDesignatedResponsibleEmployee
        );
    }

    #[test]
    fn large_recipient_self_evaluation_completed_compliant() {
        let mut input = baseline_new_construction_input();
        input.compliance_aspect = ComplianceAspect::SelfEvaluationRequirementUnderSection851;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HudSection504Mode::CompliantSelfEvaluationCompletedUnderSection851
        );
    }

    #[test]
    fn large_recipient_no_self_evaluation_violation() {
        let mut input = baseline_new_construction_input();
        input.compliance_aspect = ComplianceAspect::SelfEvaluationRequirementUnderSection851;
        input.self_evaluation_completed = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HudSection504Mode::ViolationLargeRecipientNoSelfEvaluation
        );
    }

    #[test]
    fn large_recipient_no_grievance_procedures_violation() {
        let mut input = baseline_new_construction_input();
        input.compliance_aspect = ComplianceAspect::GrievanceProceduresRequirementUnderSection853;
        input.grievance_procedures_maintained = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HudSection504Mode::ViolationLargeRecipientNoGrievanceProcedures
        );
    }

    #[test]
    fn notice_of_nondiscrimination_posted_compliant() {
        let mut input = baseline_new_construction_input();
        input.compliance_aspect =
            ComplianceAspect::NoticeOfNondiscriminationRequirementUnderSection854;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HudSection504Mode::CompliantNoticeOfNondiscriminationPostedUnderSection854
        );
    }

    #[test]
    fn notice_of_nondiscrimination_not_posted_violation() {
        let mut input = baseline_new_construction_input();
        input.compliance_aspect =
            ComplianceAspect::NoticeOfNondiscriminationRequirementUnderSection854;
        input.notice_of_nondiscrimination_posted = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HudSection504Mode::ViolationNoticeOfNondiscriminationNotPosted
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(HUD_SECTION_504_ENACTMENT_YEAR, 1973);
        assert_eq!(HUD_SECTION_504_REGULATION_TITLE, 24);
        assert_eq!(HUD_SECTION_504_REGULATION_PART, 8);
        assert_eq!(
            HUD_SECTION_504_NEW_CONSTRUCTION_MOBILITY_ACCESSIBILITY_PERCENTAGE_BASIS_POINTS,
            500
        );
        assert_eq!(
            HUD_SECTION_504_NEW_CONSTRUCTION_SENSORY_ACCESSIBILITY_PERCENTAGE_BASIS_POINTS,
            200
        );
        assert_eq!(HUD_SECTION_504_LARGE_RECIPIENT_EMPLOYEE_THRESHOLD, 15);
        assert_eq!(
            HUD_SECTION_504_2010_ADA_STANDARDS_AVAILABLE_AS_ALTERNATIVE_SINCE_YEAR,
            2011
        );
        assert_eq!(HUD_SECTION_504_2023_REGULATION_UPDATE_YEAR, 2023);
        assert_eq!(HUD_SECTION_504_2023_REGULATION_UPDATE_MONTH, 4);
        assert_eq!(HUD_SECTION_504_2023_REGULATION_UPDATE_DAY, 25);
        assert_eq!(HUD_SECTION_504_SELF_EVALUATION_RETENTION_YEARS, 3);
        assert_eq!(HUD_SECTION_504_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citation_contains_landmarks() {
        let output = check(&baseline_new_construction_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("Section 504 of the Rehabilitation Act of 1973"));
        assert!(joined.contains("Public Law 93-112"));
        assert!(joined.contains("29 USC § 794"));
        assert!(joined.contains("24 CFR Part 8"));
        assert!(joined.contains("24 CFR § 8.22"));
        assert!(joined.contains("24 CFR § 8.23"));
        assert!(joined.contains("24 CFR § 8.24"));
        assert!(joined.contains("24 CFR § 8.32"));
        assert!(joined.contains("24 CFR § 8.20"));
        assert!(joined.contains("24 CFR § 8.53"));
        assert!(joined.contains("24 CFR § 8.51"));
        assert!(joined.contains("24 CFR § 8.54"));
        assert!(joined.contains("5 PERCENT"));
        assert!(joined.contains("2 PERCENT"));
        assert!(joined.contains("UFAS"));
        assert!(joined.contains("2010 ADA Standards"));
        assert!(joined.contains("April 25, 2023"));
        assert!(joined.contains("15 EMPLOYEES"));
    }
}
