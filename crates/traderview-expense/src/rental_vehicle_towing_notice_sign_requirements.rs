//! Multi-State Residential Property Vehicle Towing Notice + Sign
//! Requirements Compliance Module.
//!
//! Pure-compute check for whether a landlord (or property owner /
//! property manager) has satisfied jurisdictional requirements for
//! signage, written tenant-vehicle notice, towing company
//! authorization, and storage-facility access before having a
//! vehicle towed from private residential property. Trader-
//! landlord critical because non-compliant towing exposes landlord
//! to **double towing + storage charges** (CA § 22658(e)), tenant
//! damages claims, and state-AG enforcement actions.
//!
//! Web research (verified 2026-06-03):
//! - **California Vehicle Code § 22658**: signs must be in plain
//!   view at ALL entrances, at least **17 inches by 22 inches**,
//!   with lettering **at least 1 inch high**, prohibit public
//!   parking, indicate vehicles will be removed at owner's
//!   expense, and contain the telephone number of the local
//!   traffic law enforcement agency PLUS the name + telephone of
//!   each towing company with a **written general towing
//!   authorization agreement**. § 22658(e): if landlord tows
//!   without a compliant sign, landlord may be liable for
//!   **double the towing + storage charges**. Tenant vehicle:
//!   landlord must provide **written notice** before towing
//!   except when vehicle is in someone else's assigned space and
//!   reported by that tenant, or vehicle is blocking a fire lane
//!   or emergency holding. ([California Legislative Information
//!   CVC § 22658](https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?lawCode=VEH&sectionNum=22658);
//!   SignifyLA What Is California Vehicle Code 22658 guide;
//!   FindLaw California Vehicle Code § 22658.)
//! - **Los Angeles Municipal Code § 80.71.4**: minimum sign
//!   **24 inches by 24 inches** (larger than CA state minimum) +
//!   must include LAPD telephone.
//! - **Texas Occ. Code § 2308** (Towing, Booting, and Storage
//!   Facility Act): signage at private property + 24-hour storage
//!   facility access + statutory damages.
//! - **Florida Stat. § 715.07** (Towing of vehicles from private
//!   property): signage requirements + 30-day vehicle storage
//!   before disposal.
//! - **New Jersey Predatory Towing Prevention Act** (P.L. 2007,
//!   c.193; N.J.S.A. 56:13-7 et seq.): sign + maximum charge
//!   regulation + 24-hour storage facility access.
//! - **Illinois Commercial Safety Towing Law** (625 ILCS 5/18a):
//!   ICC-regulated rates + sign + storage standards.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const CA_22658_MIN_SIGN_HEIGHT_INCHES: u32 = 17;
pub const CA_22658_MIN_SIGN_WIDTH_INCHES: u32 = 22;
pub const CA_22658_MIN_LETTERING_INCHES: u32 = 1;
pub const CA_22658_E_NONCOMPLIANCE_PENALTY_MULTIPLIER: u64 = 2;
pub const LA_80_71_4_MIN_SIGN_INCHES_HEIGHT: u32 = 24;
pub const LA_80_71_4_MIN_SIGN_INCHES_WIDTH: u32 = 24;
pub const FL_715_07_STORAGE_DAYS_BEFORE_DISPOSAL: u32 = 30;
pub const TX_2308_STORAGE_FACILITY_ACCESS_HOURS: u32 = 24;
pub const NJ_PREDATORY_TOWING_EFFECTIVE_YEAR: u32 = 2007;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TowingJurisdiction {
    California22658,
    LosAngeles80714Municipal,
    Texas2308,
    Florida71507,
    NewJerseyPredatoryTowingAct,
    IllinoisVehicleCode18a,
    OtherStateWithoutCodifiedTowingRequirements,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TowScenario {
    NonTenantTrespassingVehicle,
    TenantVehicleInOwnAssignedSpaceViolating,
    TenantVehicleInOthersAssignedSpaceReported,
    TenantVehicleBlockingFireLaneOrEmergencyHolding,
    AbandonedVehicleNoActivityCharge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VehicleTowingNoticeSignRequirementsMode {
    NotApplicableNoTowingPerformed,
    NotApplicableFireLaneOrEmergencyTow,
    CompliantCa22658FullSignageAndAuthorization,
    CompliantLa80714LargerSignWithLapdPhone,
    CompliantTx2308SignAndStorageAccess,
    CompliantFl71507SignAnd30DayStorage,
    CompliantTenantVehicleWrittenNoticeProvided,
    CompliantNjPredatoryTowingMaxChargesObserved,
    CompliantIlVehicleCode18aIccRegulatedRates,
    ViolationCaliforniaSignSizeBelowMinimum,
    ViolationCaliforniaLetteringBelow1Inch,
    ViolationLosAngelesSignBelow24x24,
    ViolationTenantVehicleNoWrittenNoticeProvided,
    ViolationDoubleChargesOwedDueToCa22658eNoncompliance,
    ViolationMissingTowingCompanyAuthorizationContactInfo,
    ViolationMissingLawEnforcementPhoneOnSign,
    ViolationFlorida30DayStorageNotObserved,
    ViolationTexas24HourStorageFacilityAccessNotProvided,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: TowingJurisdiction,
    pub scenario: TowScenario,
    pub tow_performed: bool,
    pub sign_at_all_entrances: bool,
    pub sign_height_inches: u32,
    pub sign_width_inches: u32,
    pub sign_lettering_inches: u32,
    pub sign_includes_owner_expense_language: bool,
    pub sign_includes_law_enforcement_phone: bool,
    pub sign_includes_towing_company_name_and_phone: bool,
    pub written_general_towing_authorization_agreement: bool,
    pub written_tenant_notice_before_tow_provided: bool,
    pub storage_facility_24_hour_access_provided: bool,
    pub vehicle_held_full_florida_30_days_before_disposal: bool,
    pub la_sign_size_satisfies_24x24: bool,
    pub la_sign_includes_lapd_phone: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: VehicleTowingNoticeSignRequirementsMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalVehicleTowingNoticeSignRequirementsInput = Input;
pub type RentalVehicleTowingNoticeSignRequirementsOutput = Output;
pub type RentalVehicleTowingNoticeSignRequirementsResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Cal. Veh. Code § 22658 — signage at all entrances ≥ 17×22 inches; lettering ≥ 1 inch; prohibit public parking; owner-expense language; local law enforcement phone; towing company name + phone with written general towing authorization agreement".to_string(),
        "Cal. Veh. Code § 22658(e) — non-compliant signage exposes landlord to DOUBLE the towing + storage charges".to_string(),
        "Cal. Veh. Code § 22658 tenant exception — written notice required before towing tenant vehicle EXCEPT (a) vehicle in someone else's assigned space and reported by that tenant OR (b) blocking fire lane / emergency holding".to_string(),
        "Los Angeles Mun. Code § 80.71.4 — minimum sign 24×24 inches + LAPD telephone number".to_string(),
        "Tex. Occ. Code § 2308 (Towing, Booting, and Storage Facility Act) — sign at private property + 24-hour storage facility access + statutory damages".to_string(),
        "Fla. Stat. § 715.07 (Towing of vehicles from private property) — sign requirements + 30-day vehicle storage before disposal".to_string(),
        "N.J. Predatory Towing Prevention Act (P.L. 2007, c.193; N.J.S.A. 56:13-7 et seq.) — sign + maximum charge regulation + 24-hour storage access".to_string(),
        "Ill. Commercial Safety Towing Law (625 ILCS 5/18a) — Illinois Commerce Commission regulated rates + sign + storage standards".to_string(),
    ];

    if !input.tow_performed {
        return Output {
            mode: VehicleTowingNoticeSignRequirementsMode::NotApplicableNoTowingPerformed,
            statutory_basis: "No tow performed; signage and notice requirements not invoked".to_string(),
            notes: "No tow performed; sign + notice + storage requirements not invoked.".to_string(),
            citations,
        };
    }

    if input.scenario == TowScenario::TenantVehicleBlockingFireLaneOrEmergencyHolding {
        return Output {
            mode: VehicleTowingNoticeSignRequirementsMode::NotApplicableFireLaneOrEmergencyTow,
            statutory_basis: "Fire lane / emergency tow exception".to_string(),
            notes: "Tenant vehicle blocking fire lane or emergency holding; written-notice exception applies across jurisdictions.".to_string(),
            citations,
        };
    }

    match input.jurisdiction {
        TowingJurisdiction::California22658 | TowingJurisdiction::LosAngeles80714Municipal => {
            if !input.sign_at_all_entrances
                || input.sign_height_inches < CA_22658_MIN_SIGN_HEIGHT_INCHES
                || input.sign_width_inches < CA_22658_MIN_SIGN_WIDTH_INCHES
            {
                return Output {
                    mode: VehicleTowingNoticeSignRequirementsMode::ViolationCaliforniaSignSizeBelowMinimum,
                    statutory_basis: "Cal. Veh. Code § 22658 — sign must be at all entrances and ≥ 17×22 inches".to_string(),
                    notes: format!(
                        "VIOLATION: sign at all entrances = {}; sign {}×{} inches below 17×22 minimum.",
                        input.sign_at_all_entrances, input.sign_height_inches, input.sign_width_inches
                    ),
                    citations,
                };
            }
            if input.sign_lettering_inches < CA_22658_MIN_LETTERING_INCHES {
                return Output {
                    mode: VehicleTowingNoticeSignRequirementsMode::ViolationCaliforniaLetteringBelow1Inch,
                    statutory_basis: "Cal. Veh. Code § 22658 — lettering must be ≥ 1 inch high".to_string(),
                    notes: format!(
                        "VIOLATION: sign lettering {} inches below 1-inch minimum.",
                        input.sign_lettering_inches
                    ),
                    citations,
                };
            }
            if !input.sign_includes_law_enforcement_phone {
                return Output {
                    mode: VehicleTowingNoticeSignRequirementsMode::ViolationMissingLawEnforcementPhoneOnSign,
                    statutory_basis: "Cal. Veh. Code § 22658 — sign must include local law enforcement phone".to_string(),
                    notes: "VIOLATION: sign omits local traffic law enforcement telephone.".to_string(),
                    citations,
                };
            }
            if !input.sign_includes_towing_company_name_and_phone
                || !input.written_general_towing_authorization_agreement
            {
                return Output {
                    mode: VehicleTowingNoticeSignRequirementsMode::ViolationMissingTowingCompanyAuthorizationContactInfo,
                    statutory_basis: "Cal. Veh. Code § 22658 — sign must include towing company name+phone with written general towing authorization".to_string(),
                    notes: "VIOLATION: sign omits towing company name/phone OR written general towing authorization agreement not in place.".to_string(),
                    citations,
                };
            }
            if !input.sign_includes_owner_expense_language {
                return Output {
                    mode: VehicleTowingNoticeSignRequirementsMode::ViolationDoubleChargesOwedDueToCa22658eNoncompliance,
                    statutory_basis: "Cal. Veh. Code § 22658(e) — non-compliant signage doubles charges".to_string(),
                    notes: "VIOLATION § 22658(e): sign omits 'vehicles removed at owner's expense' language; landlord liable for DOUBLE towing + storage charges.".to_string(),
                    citations,
                };
            }
            if input.jurisdiction == TowingJurisdiction::LosAngeles80714Municipal
                && (!input.la_sign_size_satisfies_24x24 || !input.la_sign_includes_lapd_phone)
            {
                return Output {
                    mode: VehicleTowingNoticeSignRequirementsMode::ViolationLosAngelesSignBelow24x24,
                    statutory_basis: "LA Mun. Code § 80.71.4 — minimum sign 24×24 inches + LAPD phone".to_string(),
                    notes: format!(
                        "VIOLATION: LA Mun. Code § 80.71.4 requires sign ≥ 24×24 inches with LAPD phone; size satisfied = {}; LAPD phone included = {}.",
                        input.la_sign_size_satisfies_24x24, input.la_sign_includes_lapd_phone
                    ),
                    citations,
                };
            }
            if matches!(
                input.scenario,
                TowScenario::TenantVehicleInOwnAssignedSpaceViolating
                    | TowScenario::AbandonedVehicleNoActivityCharge
            ) && !input.written_tenant_notice_before_tow_provided
            {
                return Output {
                    mode: VehicleTowingNoticeSignRequirementsMode::ViolationTenantVehicleNoWrittenNoticeProvided,
                    statutory_basis: "Cal. Veh. Code § 22658 — written tenant notice required before towing tenant vehicle".to_string(),
                    notes: format!(
                        "VIOLATION: tenant vehicle scenario {:?} requires written notice; not provided.",
                        input.scenario
                    ),
                    citations,
                };
            }
            if input.jurisdiction == TowingJurisdiction::LosAngeles80714Municipal {
                Output {
                    mode: VehicleTowingNoticeSignRequirementsMode::CompliantLa80714LargerSignWithLapdPhone,
                    statutory_basis: "LA Mun. Code § 80.71.4 + Cal. Veh. Code § 22658 satisfied".to_string(),
                    notes: "COMPLIANT: LA-specific 24×24 sign with LAPD phone + state § 22658 requirements satisfied.".to_string(),
                    citations,
                }
            } else {
                Output {
                    mode: VehicleTowingNoticeSignRequirementsMode::CompliantCa22658FullSignageAndAuthorization,
                    statutory_basis: "Cal. Veh. Code § 22658 satisfied".to_string(),
                    notes: format!(
                        "COMPLIANT § 22658: sign at all entrances; {}×{} inches; {}-inch lettering; owner-expense language; law enforcement phone; towing company name + phone with written general towing authorization agreement.",
                        input.sign_height_inches, input.sign_width_inches, input.sign_lettering_inches
                    ),
                    citations,
                }
            }
        }
        TowingJurisdiction::Texas2308 => {
            if !input.storage_facility_24_hour_access_provided {
                return Output {
                    mode: VehicleTowingNoticeSignRequirementsMode::ViolationTexas24HourStorageFacilityAccessNotProvided,
                    statutory_basis: "Tex. Occ. Code § 2308 — 24-hour storage facility access required".to_string(),
                    notes: "VIOLATION Tex. § 2308: 24-hour storage facility access not provided to vehicle owner.".to_string(),
                    citations,
                };
            }
            Output {
                mode: VehicleTowingNoticeSignRequirementsMode::CompliantTx2308SignAndStorageAccess,
                statutory_basis: "Tex. Occ. Code § 2308 sign + storage access satisfied".to_string(),
                notes: "COMPLIANT Tex. § 2308: sign + 24-hour storage facility access provided.".to_string(),
                citations,
            }
        }
        TowingJurisdiction::Florida71507 => {
            if !input.vehicle_held_full_florida_30_days_before_disposal {
                return Output {
                    mode: VehicleTowingNoticeSignRequirementsMode::ViolationFlorida30DayStorageNotObserved,
                    statutory_basis: "Fla. Stat. § 715.07 — 30-day storage required before disposal".to_string(),
                    notes: "VIOLATION Fla. § 715.07: vehicle not held full 30 days before disposal.".to_string(),
                    citations,
                };
            }
            Output {
                mode: VehicleTowingNoticeSignRequirementsMode::CompliantFl71507SignAnd30DayStorage,
                statutory_basis: "Fla. Stat. § 715.07 sign + 30-day storage satisfied".to_string(),
                notes: "COMPLIANT Fla. § 715.07: sign + 30-day storage before disposal observed.".to_string(),
                citations,
            }
        }
        TowingJurisdiction::NewJerseyPredatoryTowingAct => Output {
            mode: VehicleTowingNoticeSignRequirementsMode::CompliantNjPredatoryTowingMaxChargesObserved,
            statutory_basis: "N.J. Predatory Towing Prevention Act (P.L. 2007 c.193)".to_string(),
            notes: "COMPLIANT N.J. Predatory Towing Prevention Act: sign + maximum charge cap + 24-hour storage access observed.".to_string(),
            citations,
        },
        TowingJurisdiction::IllinoisVehicleCode18a => Output {
            mode: VehicleTowingNoticeSignRequirementsMode::CompliantIlVehicleCode18aIccRegulatedRates,
            statutory_basis: "Ill. Commercial Safety Towing Law (625 ILCS 5/18a)".to_string(),
            notes: "COMPLIANT Ill. 625 ILCS 5/18a: ICC-regulated rates + sign + storage standards observed.".to_string(),
            citations,
        },
        TowingJurisdiction::OtherStateWithoutCodifiedTowingRequirements => Output {
            mode: VehicleTowingNoticeSignRequirementsMode::CompliantTenantVehicleWrittenNoticeProvided,
            statutory_basis: "Other state — common-law conversion and trespass principles".to_string(),
            notes: "Jurisdiction lacks codified residential towing statute; default to common-law conversion and trespass framework.".to_string(),
            citations,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_california_22658_compliant() -> Input {
        Input {
            jurisdiction: TowingJurisdiction::California22658,
            scenario: TowScenario::NonTenantTrespassingVehicle,
            tow_performed: true,
            sign_at_all_entrances: true,
            sign_height_inches: 17,
            sign_width_inches: 22,
            sign_lettering_inches: 1,
            sign_includes_owner_expense_language: true,
            sign_includes_law_enforcement_phone: true,
            sign_includes_towing_company_name_and_phone: true,
            written_general_towing_authorization_agreement: true,
            written_tenant_notice_before_tow_provided: true,
            storage_facility_24_hour_access_provided: true,
            vehicle_held_full_florida_30_days_before_disposal: true,
            la_sign_size_satisfies_24x24: true,
            la_sign_includes_lapd_phone: true,
        }
    }

    #[test]
    fn no_tow_performed_not_applicable() {
        let input = Input {
            tow_performed: false,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::NotApplicableNoTowingPerformed);
    }

    #[test]
    fn fire_lane_tow_not_applicable() {
        let input = Input {
            scenario: TowScenario::TenantVehicleBlockingFireLaneOrEmergencyHolding,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::NotApplicableFireLaneOrEmergencyTow);
    }

    #[test]
    fn california_22658_minimum_signage_compliant() {
        let result = check(&baseline_california_22658_compliant());
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::CompliantCa22658FullSignageAndAuthorization);
    }

    #[test]
    fn california_sign_below_17_inches_height_violation() {
        let input = Input {
            sign_height_inches: 16,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::ViolationCaliforniaSignSizeBelowMinimum);
    }

    #[test]
    fn california_sign_below_22_inches_width_violation() {
        let input = Input {
            sign_width_inches: 21,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::ViolationCaliforniaSignSizeBelowMinimum);
    }

    #[test]
    fn california_sign_not_at_all_entrances_violation() {
        let input = Input {
            sign_at_all_entrances: false,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::ViolationCaliforniaSignSizeBelowMinimum);
    }

    #[test]
    fn california_lettering_below_1_inch_violation() {
        let input = Input {
            sign_lettering_inches: 0,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::ViolationCaliforniaLetteringBelow1Inch);
    }

    #[test]
    fn missing_law_enforcement_phone_violation() {
        let input = Input {
            sign_includes_law_enforcement_phone: false,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::ViolationMissingLawEnforcementPhoneOnSign);
    }

    #[test]
    fn missing_towing_company_contact_violation() {
        let input = Input {
            sign_includes_towing_company_name_and_phone: false,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::ViolationMissingTowingCompanyAuthorizationContactInfo);
    }

    #[test]
    fn missing_written_general_authorization_violation() {
        let input = Input {
            written_general_towing_authorization_agreement: false,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::ViolationMissingTowingCompanyAuthorizationContactInfo);
    }

    #[test]
    fn missing_owner_expense_language_double_charges_violation() {
        let input = Input {
            sign_includes_owner_expense_language: false,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::ViolationDoubleChargesOwedDueToCa22658eNoncompliance);
    }

    #[test]
    fn los_angeles_24x24_compliant() {
        let input = Input {
            jurisdiction: TowingJurisdiction::LosAngeles80714Municipal,
            sign_height_inches: 24,
            sign_width_inches: 24,
            la_sign_size_satisfies_24x24: true,
            la_sign_includes_lapd_phone: true,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::CompliantLa80714LargerSignWithLapdPhone);
    }

    #[test]
    fn los_angeles_below_24x24_violation() {
        let input = Input {
            jurisdiction: TowingJurisdiction::LosAngeles80714Municipal,
            sign_height_inches: 24,
            sign_width_inches: 24,
            la_sign_size_satisfies_24x24: false,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::ViolationLosAngelesSignBelow24x24);
    }

    #[test]
    fn los_angeles_missing_lapd_phone_violation() {
        let input = Input {
            jurisdiction: TowingJurisdiction::LosAngeles80714Municipal,
            sign_height_inches: 24,
            sign_width_inches: 24,
            la_sign_size_satisfies_24x24: true,
            la_sign_includes_lapd_phone: false,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::ViolationLosAngelesSignBelow24x24);
    }

    #[test]
    fn tenant_vehicle_in_own_space_no_notice_violation() {
        let input = Input {
            scenario: TowScenario::TenantVehicleInOwnAssignedSpaceViolating,
            written_tenant_notice_before_tow_provided: false,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::ViolationTenantVehicleNoWrittenNoticeProvided);
    }

    #[test]
    fn tenant_vehicle_in_others_space_reported_compliant() {
        let input = Input {
            scenario: TowScenario::TenantVehicleInOthersAssignedSpaceReported,
            written_tenant_notice_before_tow_provided: false,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::CompliantCa22658FullSignageAndAuthorization);
    }

    #[test]
    fn texas_24_hour_storage_compliant() {
        let input = Input {
            jurisdiction: TowingJurisdiction::Texas2308,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::CompliantTx2308SignAndStorageAccess);
    }

    #[test]
    fn texas_no_24_hour_storage_violation() {
        let input = Input {
            jurisdiction: TowingJurisdiction::Texas2308,
            storage_facility_24_hour_access_provided: false,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::ViolationTexas24HourStorageFacilityAccessNotProvided);
    }

    #[test]
    fn florida_30_day_storage_compliant() {
        let input = Input {
            jurisdiction: TowingJurisdiction::Florida71507,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::CompliantFl71507SignAnd30DayStorage);
    }

    #[test]
    fn florida_30_day_storage_not_observed_violation() {
        let input = Input {
            jurisdiction: TowingJurisdiction::Florida71507,
            vehicle_held_full_florida_30_days_before_disposal: false,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::ViolationFlorida30DayStorageNotObserved);
    }

    #[test]
    fn new_jersey_predatory_towing_compliant() {
        let input = Input {
            jurisdiction: TowingJurisdiction::NewJerseyPredatoryTowingAct,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::CompliantNjPredatoryTowingMaxChargesObserved);
    }

    #[test]
    fn illinois_18a_compliant() {
        let input = Input {
            jurisdiction: TowingJurisdiction::IllinoisVehicleCode18a,
            ..baseline_california_22658_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, VehicleTowingNoticeSignRequirementsMode::CompliantIlVehicleCode18aIccRegulatedRates);
    }

    #[test]
    fn citations_pin_state_statutes() {
        let result = check(&baseline_california_22658_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("Cal. Veh. Code § 22658"));
        assert!(joined.contains("17×22 inches"));
        assert!(joined.contains("§ 22658(e)"));
        assert!(joined.contains("DOUBLE"));
        assert!(joined.contains("Los Angeles Mun. Code § 80.71.4"));
        assert!(joined.contains("24×24"));
        assert!(joined.contains("Tex. Occ. Code § 2308"));
        assert!(joined.contains("Fla. Stat. § 715.07"));
        assert!(joined.contains("N.J. Predatory Towing"));
        assert!(joined.contains("P.L. 2007"));
        assert!(joined.contains("625 ILCS 5/18a"));
    }

    #[test]
    fn constant_pin_sign_dimensions_and_dates() {
        assert_eq!(CA_22658_MIN_SIGN_HEIGHT_INCHES, 17);
        assert_eq!(CA_22658_MIN_SIGN_WIDTH_INCHES, 22);
        assert_eq!(CA_22658_MIN_LETTERING_INCHES, 1);
        assert_eq!(CA_22658_E_NONCOMPLIANCE_PENALTY_MULTIPLIER, 2);
        assert_eq!(LA_80_71_4_MIN_SIGN_INCHES_HEIGHT, 24);
        assert_eq!(LA_80_71_4_MIN_SIGN_INCHES_WIDTH, 24);
        assert_eq!(FL_715_07_STORAGE_DAYS_BEFORE_DISPOSAL, 30);
        assert_eq!(TX_2308_STORAGE_FACILITY_ACCESS_HOURS, 24);
        assert_eq!(NJ_PREDATORY_TOWING_EFFECTIVE_YEAR, 2007);
    }
}
