//! Multi-State Residential Asbestos Disclosure Compliance Module.
//!
//! Pure-compute check for landlord compliance with state-specific
//! and federal asbestos disclosure obligations for residential
//! rental property. Trader-landlord critical because pre-1981
//! housing stock commonly contains asbestos-containing
//! construction materials (ACMs) and California's **Connelly Act**
//! plus common-law landlord-disclosure duties impose substantial
//! exposure for non-disclosure of known asbestos hazards.
//!
//! Web research (verified 2026-06-03):
//! - **California Connelly Act** (AB 3713 of 1988; Cal. Health &
//!   Safety Code § 25915 et seq.): owner of any building
//!   constructed prior to **1979** who KNOWS the building contains
//!   asbestos-containing construction materials shall provide
//!   NOTICE to all employees of that owner working within the
//!   building (and to building occupants under AB 1992 of 1990
//!   extension). Notice must include: existence of any survey
//!   conducted to determine ACM existence and location; specific
//!   locations within the building where ACMs are present; general
//!   procedures and handling restrictions necessary to prevent
//!   disturbance, release, and exposure. Applies to landlords
//!   with **more than 10 employees**. ([Wilmes Risk Control —
//!   What is the Connelly Act AB 3713 of 1988](https://www.wilmes.co/connelly-act-ab-3713-of-1988-asbestos-notification-rule/);
//!   Justia California H&S Code §§ 25915-25919.7; California
//!   Asbestos Disclosure StateRecords.org.)
//! - **Federal AHERA** (Asbestos Hazard Emergency Response Act of
//!   1986; 15 U.S.C. §§ 2641-2656): applies only to **K-12 SCHOOLS**
//!   (and certain government buildings) — NOT residential.
//! - **Federal NESHAP** (40 CFR Part 61 Subpart M): demolition /
//!   renovation friable asbestos requirements — applies to
//!   commercial demolition primarily.
//! - **Federal OSHA 29 CFR 1910.1001** (general industry) and
//!   **29 CFR 1926.1101** (construction industry): worker
//!   protection during asbestos work.
//! - **Common law landlord disclosure duty**: in jurisdictions
//!   without specific asbestos disclosure statute, landlord with
//!   actual knowledge of material defect (including known
//!   asbestos hazard) must disclose to tenant under warranty of
//!   habitability + common law fraud/misrepresentation framework.
//!   ([Justia — Environmental Hazards on Rental Property; Nolo —
//!   Is the Landlord Responsible for Disclosing Asbestos in My
//!   Rental Unit](https://www.nolo.com/legal-encyclopedia/is-the-landlord-responsible-disclosing-asbestos-rental-unit-least-fixing.html).)
//! - **Massachusetts G.L. c. 149 § 6F + § 6F-1/2**: worker
//!   protection during asbestos work projects; no specific
//!   residential disclosure but common-law duty applies.
//! - **New York Industrial Code Rule 56**: asbestos work projects;
//!   no specific residential disclosure but warranty of
//!   habitability applies.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const CONNELLY_ACT_AB_3713_YEAR: u32 = 1988;
pub const CONNELLY_ACT_AB_1992_OCCUPANT_EXTENSION_YEAR: u32 = 1990;
pub const CONNELLY_ACT_CA_HSC_SECTION_BASE: u32 = 25_915;
pub const CONNELLY_ACT_CA_HSC_SECTION_LAST: u32 = 25_919;
pub const CONNELLY_ACT_BUILDING_AGE_THRESHOLD_YEAR: u32 = 1_979;
pub const CONNELLY_ACT_EMPLOYEE_THRESHOLD: u32 = 10;
pub const AHERA_YEAR: u32 = 1986;
pub const AHERA_USC_TITLE: u32 = 15;
pub const AHERA_USC_SECTION: u32 = 2_641;
pub const FEDERAL_OSHA_GENERAL_INDUSTRY_CFR_TITLE: u32 = 29;
pub const FEDERAL_OSHA_GENERAL_INDUSTRY_SECTION: u32 = 1_910;
pub const FEDERAL_OSHA_CONSTRUCTION_SECTION: u32 = 1_926;
pub const NESHAP_CFR_TITLE: u32 = 40;
pub const NESHAP_PART: u32 = 61;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AsbestosJurisdiction {
    CaliforniaConnellyAct,
    NewYorkIndustrialCodeRule56,
    MassachusettsChapter149,
    OtherStateCommonLawOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingConstructionYear {
    PreJanuary1_1979,
    Between1979And1981,
    PostJanuary1_1981,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerAsbestosKnowledge {
    KnownAsbestosPresent,
    SurveyConductedAsbestosFound,
    SurveyConductedNoAsbestos,
    NoSurveyNoActualKnowledge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AsbestosDisclosureMode {
    NotApplicableBuildingConstructedAfter1981,
    NotApplicableNoAsbestosKnownToOwner,
    NotApplicableConnellyActUnderEmployeeThreshold,
    CompliantCaliforniaConnellyActFullNoticeProvided,
    CompliantCommonLawDutyDisclosureToTenant,
    CompliantAheraSchoolOnlyNotApplicableToResidential,
    CompliantFederalOshaWorkerProtectionFollowed,
    ViolationCaliforniaConnellyActNoticeNotProvidedToOccupants,
    ViolationCaliforniaConnellyActNoticeMissingSurveyOrLocationOrProcedures,
    ViolationLandlordKnownAsbestosNotDisclosedCommonLaw,
    ViolationOshaWorkerProtectionViolatedDuringRenovation,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: AsbestosJurisdiction,
    pub building_construction_year: BuildingConstructionYear,
    pub owner_asbestos_knowledge: OwnerAsbestosKnowledge,
    pub owner_employee_count: u32,
    pub connelly_act_notice_includes_survey_disclosure: bool,
    pub connelly_act_notice_includes_specific_locations: bool,
    pub connelly_act_notice_includes_handling_procedures: bool,
    pub notice_provided_to_employees_and_occupants: bool,
    pub renovation_in_progress: bool,
    pub osha_worker_protection_followed: bool,
    pub landlord_disclosed_known_asbestos_to_tenant: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: AsbestosDisclosureMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalAsbestosDisclosureInput = Input;
pub type RentalAsbestosDisclosureOutput = Output;
pub type RentalAsbestosDisclosureResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "California Connelly Act (AB 3713 of 1988) — Cal. Health & Safety Code §§ 25915-25919.7: pre-1979 building owner who KNOWS of asbestos-containing materials must provide notice to employees + occupants (AB 1992 of 1990 extension); applies to owners with > 10 employees".to_string(),
        "Cal. Health & Safety Code § 25915 — notice content: existence of any ACM survey + specific locations + general procedures and handling restrictions".to_string(),
        "Federal AHERA (Asbestos Hazard Emergency Response Act of 1986; 15 U.S.C. §§ 2641-2656) — applies only to K-12 schools and certain government buildings; NOT residential".to_string(),
        "Federal NESHAP (40 CFR Part 61 Subpart M) — demolition / renovation friable asbestos requirements; commercial demolition primarily".to_string(),
        "Federal OSHA 29 CFR 1910.1001 (general industry) + 29 CFR 1926.1101 (construction) — worker protection during asbestos work".to_string(),
        "Massachusetts G.L. c. 149 § 6F + § 6F-1/2 — worker protection during asbestos work; no specific residential disclosure".to_string(),
        "New York Industrial Code Rule 56 — asbestos work projects; warranty of habitability requires landlord disclosure of known material defects".to_string(),
        "Common law landlord disclosure duty — landlord with actual knowledge of material defect (including known asbestos hazard) must disclose; warranty of habitability + fraud/misrepresentation framework".to_string(),
        "California Cal/OSHA Title 8 § 1529 — asbestos worker protection standard".to_string(),
    ];

    if input.building_construction_year == BuildingConstructionYear::PostJanuary1_1981 {
        return Output {
            mode: AsbestosDisclosureMode::NotApplicableBuildingConstructedAfter1981,
            statutory_basis: "Building constructed after pre-1979 / pre-1981 thresholds; ACM presumption negated".to_string(),
            notes: "Building constructed after January 1, 1981; asbestos-containing materials presumption negated; no specific disclosure obligation arises absent actual knowledge.".to_string(),
            citations,
        };
    }

    if matches!(
        input.owner_asbestos_knowledge,
        OwnerAsbestosKnowledge::SurveyConductedNoAsbestos
            | OwnerAsbestosKnowledge::NoSurveyNoActualKnowledge
    ) {
        return Output {
            mode: AsbestosDisclosureMode::NotApplicableNoAsbestosKnownToOwner,
            statutory_basis: "No actual knowledge of asbestos; Connelly Act requires KNOWLEDGE for disclosure obligation".to_string(),
            notes: "Owner has no actual knowledge of asbestos-containing materials; Connelly Act disclosure obligation not triggered.".to_string(),
            citations,
        };
    }

    match input.jurisdiction {
        AsbestosJurisdiction::CaliforniaConnellyAct => {
            if input.owner_employee_count <= CONNELLY_ACT_EMPLOYEE_THRESHOLD {
                return Output {
                    mode: AsbestosDisclosureMode::NotApplicableConnellyActUnderEmployeeThreshold,
                    statutory_basis: "Connelly Act applies to owners with > 10 employees".to_string(),
                    notes: format!(
                        "Owner employee count = {} ≤ 10; Connelly Act notice obligation does not apply.",
                        input.owner_employee_count
                    ),
                    citations,
                };
            }
            if !input.notice_provided_to_employees_and_occupants {
                return Output {
                    mode: AsbestosDisclosureMode::ViolationCaliforniaConnellyActNoticeNotProvidedToOccupants,
                    statutory_basis: "Cal. H&S Code § 25915 — notice to employees AND occupants required (AB 1992 of 1990 extension)".to_string(),
                    notes: "VIOLATION Cal. H&S Code § 25915: known asbestos in pre-1979 building; notice not provided to employees and/or building occupants.".to_string(),
                    citations,
                };
            }
            if !input.connelly_act_notice_includes_survey_disclosure
                || !input.connelly_act_notice_includes_specific_locations
                || !input.connelly_act_notice_includes_handling_procedures
            {
                return Output {
                    mode: AsbestosDisclosureMode::ViolationCaliforniaConnellyActNoticeMissingSurveyOrLocationOrProcedures,
                    statutory_basis: "Cal. H&S Code § 25915 — notice must include survey + locations + handling procedures".to_string(),
                    notes: format!(
                        "VIOLATION Cal. H&S Code § 25915: notice missing required elements — survey disclosure: {}; specific locations: {}; handling procedures: {}.",
                        input.connelly_act_notice_includes_survey_disclosure,
                        input.connelly_act_notice_includes_specific_locations,
                        input.connelly_act_notice_includes_handling_procedures
                    ),
                    citations,
                };
            }
            if input.renovation_in_progress && !input.osha_worker_protection_followed {
                return Output {
                    mode: AsbestosDisclosureMode::ViolationOshaWorkerProtectionViolatedDuringRenovation,
                    statutory_basis: "Cal/OSHA Title 8 § 1529 + Federal OSHA 29 CFR 1926.1101 — worker protection during asbestos work".to_string(),
                    notes: "VIOLATION: renovation in progress without Cal/OSHA Title 8 § 1529 worker protection compliance.".to_string(),
                    citations,
                };
            }
            Output {
                mode: AsbestosDisclosureMode::CompliantCaliforniaConnellyActFullNoticeProvided,
                statutory_basis: "Cal. H&S Code § 25915 — Connelly Act notice fully provided".to_string(),
                notes: "COMPLIANT Cal. H&S Code § 25915: pre-1979 building with known asbestos; notice provided to employees and occupants with survey disclosure + specific locations + handling procedures.".to_string(),
                citations,
            }
        }
        _ => {
            if !input.landlord_disclosed_known_asbestos_to_tenant {
                return Output {
                    mode: AsbestosDisclosureMode::ViolationLandlordKnownAsbestosNotDisclosedCommonLaw,
                    statutory_basis: "Common law landlord disclosure duty + warranty of habitability".to_string(),
                    notes: "VIOLATION: landlord has actual knowledge of asbestos in pre-1981 building but failed to disclose to tenant; warranty of habitability + common law fraud/misrepresentation framework.".to_string(),
                    citations,
                };
            }
            if input.renovation_in_progress && !input.osha_worker_protection_followed {
                return Output {
                    mode: AsbestosDisclosureMode::ViolationOshaWorkerProtectionViolatedDuringRenovation,
                    statutory_basis: "Federal OSHA 29 CFR 1910.1001 + 29 CFR 1926.1101 worker protection".to_string(),
                    notes: "VIOLATION: renovation in progress without federal OSHA asbestos worker protection compliance.".to_string(),
                    citations,
                };
            }
            Output {
                mode: AsbestosDisclosureMode::CompliantCommonLawDutyDisclosureToTenant,
                statutory_basis: "Common law landlord disclosure duty satisfied".to_string(),
                notes: "COMPLIANT: landlord disclosed known asbestos to tenant under common law landlord disclosure duty + warranty of habitability.".to_string(),
                citations,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_california_connelly_act_compliant() -> Input {
        Input {
            jurisdiction: AsbestosJurisdiction::CaliforniaConnellyAct,
            building_construction_year: BuildingConstructionYear::PreJanuary1_1979,
            owner_asbestos_knowledge: OwnerAsbestosKnowledge::KnownAsbestosPresent,
            owner_employee_count: 50,
            connelly_act_notice_includes_survey_disclosure: true,
            connelly_act_notice_includes_specific_locations: true,
            connelly_act_notice_includes_handling_procedures: true,
            notice_provided_to_employees_and_occupants: true,
            renovation_in_progress: false,
            osha_worker_protection_followed: true,
            landlord_disclosed_known_asbestos_to_tenant: true,
        }
    }

    #[test]
    fn post_1981_building_not_applicable() {
        let input = Input {
            building_construction_year: BuildingConstructionYear::PostJanuary1_1981,
            ..baseline_california_connelly_act_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            AsbestosDisclosureMode::NotApplicableBuildingConstructedAfter1981
        );
    }

    #[test]
    fn no_asbestos_knowledge_not_applicable() {
        let input = Input {
            owner_asbestos_knowledge: OwnerAsbestosKnowledge::NoSurveyNoActualKnowledge,
            ..baseline_california_connelly_act_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            AsbestosDisclosureMode::NotApplicableNoAsbestosKnownToOwner
        );
    }

    #[test]
    fn survey_no_asbestos_not_applicable() {
        let input = Input {
            owner_asbestos_knowledge: OwnerAsbestosKnowledge::SurveyConductedNoAsbestos,
            ..baseline_california_connelly_act_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            AsbestosDisclosureMode::NotApplicableNoAsbestosKnownToOwner
        );
    }

    #[test]
    fn connelly_under_10_employees_not_applicable() {
        let input = Input {
            owner_employee_count: 10,
            ..baseline_california_connelly_act_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            AsbestosDisclosureMode::NotApplicableConnellyActUnderEmployeeThreshold
        );
    }

    #[test]
    fn connelly_at_11_employees_triggered() {
        let input = Input {
            owner_employee_count: 11,
            ..baseline_california_connelly_act_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            AsbestosDisclosureMode::CompliantCaliforniaConnellyActFullNoticeProvided
        );
    }

    #[test]
    fn california_connelly_act_baseline_compliant() {
        let result = check(&baseline_california_connelly_act_compliant());
        assert_eq!(
            result.mode,
            AsbestosDisclosureMode::CompliantCaliforniaConnellyActFullNoticeProvided
        );
    }

    #[test]
    fn connelly_notice_not_provided_violation() {
        let input = Input {
            notice_provided_to_employees_and_occupants: false,
            ..baseline_california_connelly_act_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            AsbestosDisclosureMode::ViolationCaliforniaConnellyActNoticeNotProvidedToOccupants
        );
    }

    #[test]
    fn connelly_notice_missing_survey_violation() {
        let input = Input {
            connelly_act_notice_includes_survey_disclosure: false,
            ..baseline_california_connelly_act_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, AsbestosDisclosureMode::ViolationCaliforniaConnellyActNoticeMissingSurveyOrLocationOrProcedures);
    }

    #[test]
    fn connelly_notice_missing_locations_violation() {
        let input = Input {
            connelly_act_notice_includes_specific_locations: false,
            ..baseline_california_connelly_act_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, AsbestosDisclosureMode::ViolationCaliforniaConnellyActNoticeMissingSurveyOrLocationOrProcedures);
    }

    #[test]
    fn connelly_notice_missing_procedures_violation() {
        let input = Input {
            connelly_act_notice_includes_handling_procedures: false,
            ..baseline_california_connelly_act_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, AsbestosDisclosureMode::ViolationCaliforniaConnellyActNoticeMissingSurveyOrLocationOrProcedures);
    }

    #[test]
    fn connelly_renovation_without_osha_protection_violation() {
        let input = Input {
            renovation_in_progress: true,
            osha_worker_protection_followed: false,
            ..baseline_california_connelly_act_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            AsbestosDisclosureMode::ViolationOshaWorkerProtectionViolatedDuringRenovation
        );
    }

    #[test]
    fn new_york_common_law_disclosure_compliant() {
        let input = Input {
            jurisdiction: AsbestosJurisdiction::NewYorkIndustrialCodeRule56,
            landlord_disclosed_known_asbestos_to_tenant: true,
            ..baseline_california_connelly_act_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            AsbestosDisclosureMode::CompliantCommonLawDutyDisclosureToTenant
        );
    }

    #[test]
    fn new_york_known_asbestos_not_disclosed_violation() {
        let input = Input {
            jurisdiction: AsbestosJurisdiction::NewYorkIndustrialCodeRule56,
            landlord_disclosed_known_asbestos_to_tenant: false,
            ..baseline_california_connelly_act_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            AsbestosDisclosureMode::ViolationLandlordKnownAsbestosNotDisclosedCommonLaw
        );
    }

    #[test]
    fn massachusetts_common_law_disclosure_compliant() {
        let input = Input {
            jurisdiction: AsbestosJurisdiction::MassachusettsChapter149,
            landlord_disclosed_known_asbestos_to_tenant: true,
            ..baseline_california_connelly_act_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            AsbestosDisclosureMode::CompliantCommonLawDutyDisclosureToTenant
        );
    }

    #[test]
    fn other_state_common_law_violation() {
        let input = Input {
            jurisdiction: AsbestosJurisdiction::OtherStateCommonLawOnly,
            landlord_disclosed_known_asbestos_to_tenant: false,
            ..baseline_california_connelly_act_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            AsbestosDisclosureMode::ViolationLandlordKnownAsbestosNotDisclosedCommonLaw
        );
    }

    #[test]
    fn between_1979_1981_building_in_california_no_connelly_violation_if_disclosed() {
        let input = Input {
            building_construction_year: BuildingConstructionYear::Between1979And1981,
            jurisdiction: AsbestosJurisdiction::OtherStateCommonLawOnly,
            ..baseline_california_connelly_act_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            AsbestosDisclosureMode::CompliantCommonLawDutyDisclosureToTenant
        );
    }

    #[test]
    fn citations_pin_connelly_act_and_federal_regs() {
        let result = check(&baseline_california_connelly_act_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("California Connelly Act"));
        assert!(joined.contains("AB 3713 of 1988"));
        assert!(joined.contains("§§ 25915-25919.7"));
        assert!(joined.contains("pre-1979"));
        assert!(joined.contains("> 10 employees"));
        assert!(joined.contains("AB 1992 of 1990"));
        assert!(joined.contains("Cal. Health & Safety Code § 25915"));
        assert!(joined.contains("AHERA"));
        assert!(joined.contains("1986"));
        assert!(joined.contains("15 U.S.C. §§ 2641-2656"));
        assert!(joined.contains("K-12 schools"));
        assert!(joined.contains("40 CFR Part 61"));
        assert!(joined.contains("29 CFR 1910.1001"));
        assert!(joined.contains("29 CFR 1926.1101"));
        assert!(joined.contains("Massachusetts G.L. c. 149"));
        assert!(joined.contains("New York Industrial Code Rule 56"));
        assert!(joined.contains("warranty of habitability"));
        assert!(joined.contains("Cal/OSHA Title 8 § 1529"));
    }

    #[test]
    fn constant_pin_connelly_act_years_and_thresholds() {
        assert_eq!(CONNELLY_ACT_AB_3713_YEAR, 1988);
        assert_eq!(CONNELLY_ACT_AB_1992_OCCUPANT_EXTENSION_YEAR, 1990);
        assert_eq!(CONNELLY_ACT_CA_HSC_SECTION_BASE, 25_915);
        assert_eq!(CONNELLY_ACT_CA_HSC_SECTION_LAST, 25_919);
        assert_eq!(CONNELLY_ACT_BUILDING_AGE_THRESHOLD_YEAR, 1_979);
        assert_eq!(CONNELLY_ACT_EMPLOYEE_THRESHOLD, 10);
        assert_eq!(AHERA_YEAR, 1986);
        assert_eq!(AHERA_USC_TITLE, 15);
        assert_eq!(AHERA_USC_SECTION, 2_641);
        assert_eq!(FEDERAL_OSHA_GENERAL_INDUSTRY_CFR_TITLE, 29);
        assert_eq!(FEDERAL_OSHA_GENERAL_INDUSTRY_SECTION, 1_910);
        assert_eq!(FEDERAL_OSHA_CONSTRUCTION_SECTION, 1_926);
        assert_eq!(NESHAP_CFR_TITLE, 40);
        assert_eq!(NESHAP_PART, 61);
    }
}
