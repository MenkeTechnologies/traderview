//! Multi-jurisdictional rental property SEPTIC SYSTEM
//! disclosure + inspection compliance framework. When a
//! landlord rents a property served by a private on-site
//! sewage treatment and disposal system (OSTDS or "septic
//! system") rather than connection to municipal sewer, what
//! disclosure rules apply, what inspection certifications
//! are required, and what failure-mode liabilities expose
//! landlord?
//!
//! Distinct from sibling modules: rental_underground_
//! storage_tank_disclosure (UST + LUST), rental_water_
//! submetering_disclosure (water billing), rental_basement_
//! water_intrusion_disclosure (water intrusion), rental_
//! sinkhole_disclosure (sinkhole), rental_flood_hazard_
//! disclosure (flood plain), rental_well_water_disclosure
//! (private drinking water — companion not yet shipped).
//!
//! Four-jurisdiction framework:
//!
//! 1. MASSACHUSETTS (most stringent) — 310 C.M.R. 15.000
//!    "Title 5" of the State Environmental Code, originally
//!    promulgated 1995, MOST RECENTLY AMENDED EFFECTIVE
//!    July 7, 2023. Title 5 requires a pass-or-fail system
//!    inspection by a Mass. Department of Environmental
//!    Protection (MassDEP) approved system inspector at
//!    TIME OF TITLE TRANSFER or within 2 YEARS prior to
//!    transfer (extended to 3 years if pumped annually).
//!    Refinanced/Conveyed/RENTED-FOR-TRANSFER inspection
//!    triggers under 310 CMR 15.301. Nitrogen-sensitive
//!    watershed (Cape Cod + Nantucket + Buzzards Bay)
//!    Title 5 + nitrogen-aggregation loading 2023
//!    amendments tightened in I/A (innovative/alternative)
//!    nitrogen-reducing technology requirements.
//! 2. FLORIDA — Fla. Stat. § 381.0065 (Onsite Sewage
//!    Treatment and Disposal Systems); Fla. Stat.
//!    § 381.00655 (mandatory connection to central sewer
//!    when available); Fla. Admin. Code Ch. 64E-6; 2020
//!    Clean Waterways Act (Senate Bill 712, signed June 30,
//!    2020) imposed strengthened inspection + permit
//!    requirements for OSTDS located within Basin
//!    Management Action Plans (BMAPs). § 381.0065(2)(a)
//!    requires installation permit; § 381.0065(4) requires
//!    operating permit for performance-based systems.
//!    Voluntary inspection program under § 381.0065(4)(g):
//!    inspector MUST inform person having ownership of the
//!    inspection standards + person's authority to request
//!    inspection.
//! 3. TEXAS — Tex. Health & Safety Code § 366.011 et seq.
//!    plus 30 Tex. Admin. Code Ch. 285 (TCEQ regulations).
//!    OSSF (On-Site Sewage Facility) registration with
//!    local Authorized Agent (typically county). § 366.051
//!    permit required pre-construction; § 366.071
//!    authorization to operate. TCEQ Form 20021
//!    notification. Landlord renting property served by
//!    OSSF must use Texas Property Code § 92.354 lead-based
//!    paint analog disclosure framework where lease has
//!    OSSF mention or owner is required to disclose by
//!    federal law.
//! 4. CALIFORNIA / DEFAULT — California Water Boards
//!    Onsite Wastewater Treatment Systems Policy (adopted
//!    June 19, 2012, amended); Cal. Water Code § 13290;
//!    Cal. Health & Safety Code § 18046 (mobile home park
//!    septic). Most states lack specific rental disclosure
//!    statute; landlord falls back on common-law
//!    HABITABILITY warranty (Green v. Superior Court, 10
//!    Cal. 3d 616 (1974)) + statutory implied warranty
//!    (Cal. Civ. Code § 1941.1 — sanitary facilities) +
//!    federal Clean Water Act § 502(14) does NOT regulate
//!    septic but state water-quality enforcement under
//!    state EPA + delegated NPDES exists. RCRA Subtitle D
//!    solid-waste regulation may apply.
//!
//! Universal failure-mode liability framework:
//! 1. Sewage backup into unit → habitability breach +
//!    constructive eviction (Hilder v. St. Peter, 478
//!    A.2d 202 (Vt. 1984)) + tenant remedies escrow / rent
//!    abatement (see rent_abatement_construction_nuisance).
//! 2. Groundwater contamination of private well (if dual
//!    well + septic) → CERCLA potential under 42 U.S.C.
//!    § 9607(a)(1) owner/operator strict liability + state
//!    cleanup liability.
//! 3. Pump-out frequency neglect → Title 5 violation in
//!    MA; § 381.0065(4) operating permit violation in FL.
//! 4. Drainfield failure on hill / slope → soil percolation
//!    re-engineering cost $15K-$45K typical.
//! 5. Tenant misuse (flushing wipes, grease, harsh
//!    chemicals) → tenant liability under lease; landlord
//!    cannot disclaim system condition entirely.
//!
//! Trader-landlord critical because (1) MA Title 5
//! inspection failure at title transfer can require
//! $30,000+ in system replacement before closing; (2) FL
//! OSTDS in BMAP basin triggers 5-year inspection cycle
//! cost $400-$800 per inspection; (3) sewage backup
//! incidents are leading source of habitability litigation
//! in rural rental properties; (4) inherited septic
//! systems in distressed-property purchases frequently
//! discovered non-permitted ("wildcat") at the time of
//! resale — Texas TCEQ § 366.051 backflow.
//!
//! Authority: 310 C.M.R. 15.000; 310 C.M.R. 15.301; 310
//! C.M.R. 15.305; Title 5 July 7, 2023 amendments; Fla.
//! Stat. § 381.0065; Fla. Stat. § 381.0065(2)(a); Fla. Stat.
//! § 381.0065(4); Fla. Stat. § 381.0065(4)(g); Fla. Stat.
//! § 381.00655; Fla. Admin. Code Ch. 64E-6; Florida Senate
//! Bill 712 (2020 Clean Waterways Act, signed June 30,
//! 2020); Tex. Health & Safety Code § 366.011; Tex. Health
//! & Safety Code § 366.051; Tex. Health & Safety Code
//! § 366.071; 30 Tex. Admin. Code Ch. 285; California Water
//! Boards OWTS Policy June 19, 2012; Cal. Water Code
//! § 13290; Cal. Health & Safety Code § 18046; Cal. Civ.
//! Code § 1941.1; Green v. Superior Court, 10 Cal. 3d 616
//! (1974); Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984);
//! CERCLA 42 U.S.C. § 9607(a); RCRA Subtitle D.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    Massachusetts,
    Florida,
    Texas,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemType {
    ConventionalTank,
    AerobicTreatmentUnit,
    InnovativeAlternativeNitrogenReducing,
    PerformanceBased,
    Cesspool,
    Wildcat,
    None,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub system_type: SystemType,
    pub property_served_by_septic: bool,
    pub permit_on_file: bool,
    pub operating_permit_on_file: bool,
    pub last_pump_out_months_ago: u32,
    pub last_inspection_months_ago: u32,
    pub in_nitrogen_sensitive_watershed: bool,
    pub in_florida_bmap_basin: bool,
    pub disclosure_provided_in_lease: bool,
    pub sewage_backup_event_reported: bool,
    pub estimated_replacement_cost_cents: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Compliant,
    DisclosureRequired,
    InspectionOverdue,
    PermitMissing,
    SystemFailure,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub jurisdiction_specific_actions: Vec<String>,
    pub notes: Vec<String>,
}

pub type RentalSepticSystemDisclosureInput = Input;
pub type RentalSepticSystemDisclosureResult = Output;

pub const MA_TITLE5_INSPECTION_MAX_MONTHS: u32 = 24;
pub const MA_TITLE5_PUMPED_ANNUALLY_MAX_MONTHS: u32 = 36;
pub const FL_BMAP_INSPECTION_CYCLE_MONTHS: u32 = 60;
pub const TX_PUMP_OUT_RECOMMENDED_MONTHS: u32 = 60;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let mut actions: Vec<String> = Vec::new();

    notes.push("Four-jurisdiction framework: Massachusetts (most stringent — 310 C.M.R. 15.000 Title 5 + July 7, 2023 amendments), Florida (Fla. Stat. § 381.0065 + 2020 Clean Waterways Act SB 712 for BMAP basins), Texas (Tex. Health & Safety Code § 366.011 + 30 TAC Ch. 285 OSSF + TCEQ Authorized Agent), Default (common-law habitability + state implied warranty + CERCLA owner/operator liability).".to_string());
    notes.push("Massachusetts Title 5 requires pass-or-fail inspection at title transfer or within 24 months prior (36 months if pumped annually) per 310 C.M.R. 15.301; nitrogen-sensitive watersheds (Cape Cod + Nantucket + Buzzards Bay) require innovative/alternative (I/A) nitrogen-reducing technology under 2023 amendments.".to_string());
    notes.push("Florida § 381.0065(2)(a) requires installation permit; § 381.0065(4) requires operating permit for performance-based systems; § 381.0065(4)(g) voluntary inspection program — inspector MUST inform owner of inspection standards and authority to request; SB 712 (2020 Clean Waterways Act) imposed 5-year inspection cycle for OSTDS in Basin Management Action Plan (BMAP) basins.".to_string());
    notes.push("Texas Tex. Health & Safety Code § 366.011 et seq. + 30 TAC Ch. 285: OSSF registration with local Authorized Agent (county); § 366.051 installation permit required pre-construction; § 366.071 authorization to operate required for performance-based systems.".to_string());
    notes.push("California / Default: most states lack specific rental statutory disclosure; landlord falls back on common-law habitability (Green v. Superior Court, 10 Cal. 3d 616 (1974)) + statutory implied warranty (Cal. Civ. Code § 1941.1 sanitary facilities); federal Clean Water Act § 502(14) does NOT regulate septic but CERCLA 42 U.S.C. § 9607(a) owner/operator strict liability if groundwater contamination triggered.".to_string());
    notes.push("Five universal failure-mode liabilities: (1) sewage backup → habitability breach + constructive eviction per Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984); (2) groundwater contamination → CERCLA strict liability; (3) pump-out frequency neglect → Title 5 / § 381.0065(4) violation; (4) drainfield failure $15K-$45K re-engineering; (5) tenant misuse limited landlord disclaimer.".to_string());
    notes.push("Companion modules: rental_underground_storage_tank_disclosure (UST + LUST), rental_basement_water_intrusion_disclosure, rental_sinkhole_disclosure, rental_flood_hazard_disclosure, rent_abatement_construction_nuisance.".to_string());

    if !input.property_served_by_septic {
        notes.push("Property is on municipal sewer — septic disclosure not applicable.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            jurisdiction_specific_actions: actions,
            notes,
        };
    }

    if matches!(input.system_type, SystemType::Wildcat) || !input.permit_on_file {
        actions.push(match input.jurisdiction {
            Jurisdiction::Massachusetts => "Massachusetts: wildcat (non-permitted) septic violates 310 C.M.R. 15.000; immediate notification to local Board of Health required. MassDEP enforcement.".to_string(),
            Jurisdiction::Florida => "Florida: wildcat septic violates § 381.0065(2)(a) installation permit requirement. Fla. Admin. Code Ch. 64E-6 enforcement.".to_string(),
            Jurisdiction::Texas => "Texas: wildcat OSSF violates Tex. Health & Safety Code § 366.051 + 30 TAC Ch. 285 + TCEQ Authorized Agent jurisdiction.".to_string(),
            Jurisdiction::Default => "State permit required; CERCLA 42 U.S.C. § 9607(a) owner/operator strict liability exposure if groundwater contamination triggers.".to_string(),
        });
    }

    let mut needs_inspection = false;
    let mut needs_pump_out = false;

    match input.jurisdiction {
        Jurisdiction::Massachusetts => {
            let cap = if input.last_pump_out_months_ago <= 12 {
                MA_TITLE5_PUMPED_ANNUALLY_MAX_MONTHS
            } else {
                MA_TITLE5_INSPECTION_MAX_MONTHS
            };
            if input.last_inspection_months_ago > cap {
                needs_inspection = true;
                actions.push(format!(
                    "Massachusetts Title 5: inspection {} months old exceeds {} month cap per 310 C.M.R. 15.301. Engage MassDEP-approved system inspector.",
                    input.last_inspection_months_ago, cap
                ));
            }
            if input.in_nitrogen_sensitive_watershed
                && !matches!(
                    input.system_type,
                    SystemType::InnovativeAlternativeNitrogenReducing
                )
            {
                actions.push("Massachusetts nitrogen-sensitive watershed (Cape Cod / Nantucket / Buzzards Bay): innovative/alternative (I/A) nitrogen-reducing technology required under 310 C.M.R. 15.000 July 7, 2023 amendments.".to_string());
            }
        }
        Jurisdiction::Florida => {
            if input.in_florida_bmap_basin
                && input.last_inspection_months_ago > FL_BMAP_INSPECTION_CYCLE_MONTHS
            {
                needs_inspection = true;
                actions.push(format!(
                    "Florida BMAP basin: inspection {} months old exceeds 60-month cycle per 2020 SB 712 Clean Waterways Act. Engage authorized inspector.",
                    input.last_inspection_months_ago
                ));
            }
            if matches!(input.system_type, SystemType::PerformanceBased)
                && !input.operating_permit_on_file
            {
                actions.push("Florida performance-based system: operating permit required under Fla. Stat. § 381.0065(4).".to_string());
            }
        }
        Jurisdiction::Texas => {
            if matches!(input.system_type, SystemType::PerformanceBased)
                && !input.operating_permit_on_file
            {
                actions.push("Texas performance-based OSSF: authorization to operate required under Tex. Health & Safety Code § 366.071 + TCEQ Form 20021.".to_string());
            }
            if input.last_pump_out_months_ago > TX_PUMP_OUT_RECOMMENDED_MONTHS {
                needs_pump_out = true;
                actions.push(format!(
                    "Texas: pump-out {} months old exceeds typical 60-month service cycle; engage TCEQ-licensed installer / maintenance provider.",
                    input.last_pump_out_months_ago
                ));
            }
        }
        Jurisdiction::Default => {
            actions.push("Default jurisdiction: no statutory rental disclosure regime; common-law habitability + Cal. Civ. Code § 1941.1 implied warranty of sanitary facilities applies; pump-out every 3-5 years industry standard.".to_string());
        }
    }

    if !input.disclosure_provided_in_lease {
        actions.push(
            "Provide written septic-system disclosure in lease addendum: system type + permit number + last inspection date + last pump-out date + tenant flushing/maintenance prohibitions + emergency contact for backup events."
                .to_string(),
        );
    }

    if input.sewage_backup_event_reported {
        actions.push("Sewage backup event reported: habitability breach + constructive eviction risk per Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984); engage licensed plumber + remediation contractor; offer rent abatement (see rent_abatement_construction_nuisance) + temporary relocation (see mid_tenancy_temporary_relocation).".to_string());
    }

    let severity = if input.sewage_backup_event_reported {
        Severity::SystemFailure
    } else if matches!(input.system_type, SystemType::Wildcat) || !input.permit_on_file {
        Severity::PermitMissing
    } else if needs_inspection || needs_pump_out {
        Severity::InspectionOverdue
    } else if !input.disclosure_provided_in_lease {
        Severity::DisclosureRequired
    } else {
        Severity::Compliant
    };

    Output {
        severity,
        jurisdiction_specific_actions: actions,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::Massachusetts,
            system_type: SystemType::ConventionalTank,
            property_served_by_septic: true,
            permit_on_file: true,
            operating_permit_on_file: true,
            last_pump_out_months_ago: 6,
            last_inspection_months_ago: 12,
            in_nitrogen_sensitive_watershed: false,
            in_florida_bmap_basin: false,
            disclosure_provided_in_lease: true,
            sewage_backup_event_reported: false,
            estimated_replacement_cost_cents: 0,
        }
    }

    #[test]
    fn property_on_municipal_sewer_not_applicable() {
        let mut i = baseline();
        i.property_served_by_septic = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn massachusetts_compliant_baseline() {
        let i = baseline();
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn massachusetts_title5_inspection_overdue_default_24_months() {
        let mut i = baseline();
        i.last_pump_out_months_ago = 18; // not pumped annually
        i.last_inspection_months_ago = 30;
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionOverdue);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("310 C.M.R. 15.301"));
        assert!(joined.contains("24 month cap"));
    }

    #[test]
    fn massachusetts_title5_pumped_annually_36_month_cap_applies() {
        let mut i = baseline();
        i.last_pump_out_months_ago = 11; // pumped within 12 months
        i.last_inspection_months_ago = 30; // under 36 month cap
        let out = check(&i);
        // 30 < 36 cap, compliant on inspection
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn massachusetts_title5_pumped_annually_over_36_months_overdue() {
        let mut i = baseline();
        i.last_pump_out_months_ago = 11;
        i.last_inspection_months_ago = 40;
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionOverdue);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("36 month cap"));
    }

    #[test]
    fn massachusetts_nitrogen_sensitive_requires_ia_technology() {
        let mut i = baseline();
        i.in_nitrogen_sensitive_watershed = true;
        i.system_type = SystemType::ConventionalTank;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("innovative/alternative"));
        assert!(joined.contains("Cape Cod"));
        assert!(joined.contains("July 7, 2023"));
    }

    #[test]
    fn massachusetts_nitrogen_sensitive_with_ia_technology_no_warning() {
        let mut i = baseline();
        i.in_nitrogen_sensitive_watershed = true;
        i.system_type = SystemType::InnovativeAlternativeNitrogenReducing;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(!joined.contains("innovative/alternative"));
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn florida_bmap_basin_inspection_overdue_60_months() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Florida;
        i.in_florida_bmap_basin = true;
        i.last_inspection_months_ago = 72; // over 60-month cycle
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionOverdue);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("BMAP"));
        assert!(joined.contains("SB 712"));
        assert!(joined.contains("Clean Waterways Act"));
    }

    #[test]
    fn florida_non_bmap_no_5year_cycle_compliant() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Florida;
        i.in_florida_bmap_basin = false;
        i.last_inspection_months_ago = 72;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn florida_performance_based_requires_operating_permit() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Florida;
        i.system_type = SystemType::PerformanceBased;
        i.operating_permit_on_file = false;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("§ 381.0065(4)"));
        assert!(joined.contains("operating permit"));
    }

    #[test]
    fn texas_performance_based_requires_authorization_to_operate() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Texas;
        i.system_type = SystemType::PerformanceBased;
        i.operating_permit_on_file = false;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("§ 366.071"));
        assert!(joined.contains("TCEQ"));
        assert!(joined.contains("Form 20021"));
    }

    #[test]
    fn texas_pump_out_over_60_months_warning() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Texas;
        i.last_pump_out_months_ago = 84;
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionOverdue);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("TCEQ-licensed"));
    }

    #[test]
    fn default_jurisdiction_uses_common_law_habitability() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("common-law habitability"));
        assert!(joined.contains("§ 1941.1"));
    }

    #[test]
    fn wildcat_system_permit_missing_severity() {
        let mut i = baseline();
        i.system_type = SystemType::Wildcat;
        i.permit_on_file = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::PermitMissing);
    }

    #[test]
    fn wildcat_texas_cites_tceq_authorized_agent() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Texas;
        i.system_type = SystemType::Wildcat;
        i.permit_on_file = false;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("§ 366.051"));
        assert!(joined.contains("TCEQ Authorized Agent"));
    }

    #[test]
    fn wildcat_florida_cites_64e_6() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Florida;
        i.system_type = SystemType::Wildcat;
        i.permit_on_file = false;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("§ 381.0065(2)(a)"));
        assert!(joined.contains("64E-6"));
    }

    #[test]
    fn wildcat_default_cercla_strict_liability_warning() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        i.system_type = SystemType::Wildcat;
        i.permit_on_file = false;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("CERCLA"));
        assert!(joined.contains("§ 9607(a)"));
    }

    #[test]
    fn sewage_backup_event_system_failure_severity() {
        let mut i = baseline();
        i.sewage_backup_event_reported = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::SystemFailure);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("478 A.2d 202"));
        assert!(joined.contains("constructive eviction"));
    }

    #[test]
    fn missing_lease_disclosure_severity() {
        let mut i = baseline();
        i.disclosure_provided_in_lease = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DisclosureRequired);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("written septic-system disclosure"));
        assert!(joined.contains("emergency contact"));
    }

    #[test]
    fn severity_priority_failure_above_permit_above_inspection_above_disclosure() {
        // Stack everything wrong: sewage backup wins (SystemFailure)
        let mut i = baseline();
        i.sewage_backup_event_reported = true;
        i.system_type = SystemType::Wildcat;
        i.permit_on_file = false;
        i.last_inspection_months_ago = 60;
        i.disclosure_provided_in_lease = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::SystemFailure);
    }

    #[test]
    fn severity_permit_above_inspection_when_no_backup() {
        let mut i = baseline();
        i.system_type = SystemType::Wildcat;
        i.permit_on_file = false;
        i.last_inspection_months_ago = 60;
        let out = check(&i);
        assert_eq!(out.severity, Severity::PermitMissing);
    }

    #[test]
    fn severity_inspection_above_disclosure_when_permitted() {
        let mut i = baseline();
        i.last_pump_out_months_ago = 18;
        i.last_inspection_months_ago = 30;
        i.disclosure_provided_in_lease = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionOverdue);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("310 C.M.R. 15.000"));
        assert!(joined.contains("310 C.M.R. 15.301"));
        assert!(joined.contains("July 7, 2023"));
        assert!(joined.contains("§ 381.0065"));
        assert!(joined.contains("§ 381.0065(2)(a)"));
        assert!(joined.contains("§ 381.0065(4)"));
        assert!(joined.contains("§ 381.0065(4)(g)"));
        assert!(joined.contains("SB 712"));
        assert!(joined.contains("Clean Waterways Act"));
        assert!(joined.contains("§ 366.011"));
        assert!(joined.contains("30 TAC Ch. 285"));
        assert!(joined.contains("§ 1941.1"));
        assert!(joined.contains("Green v. Superior Court"));
        assert!(joined.contains("10 Cal. 3d 616"));
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("478 A.2d 202"));
        assert!(joined.contains("§ 9607(a)"));
    }

    #[test]
    fn note_pins_four_jurisdiction_framework() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("Massachusetts (most stringent"));
        assert!(joined.contains("Florida"));
        assert!(joined.contains("Texas"));
        assert!(joined.contains("Default"));
    }

    #[test]
    fn note_pins_five_failure_mode_liabilities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("sewage backup"));
        assert!(joined.contains("groundwater contamination"));
        assert!(joined.contains("pump-out frequency neglect"));
        assert!(joined.contains("drainfield failure"));
        assert!(joined.contains("tenant misuse"));
    }

    #[test]
    fn note_pins_ma_title5_inspection_caps() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("24 months"));
        assert!(joined.contains("36 months if pumped annually"));
    }

    #[test]
    fn note_pins_fl_bmap_60_month_cycle() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("5-year inspection cycle"));
        assert!(joined.contains("BMAP"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("rental_underground_storage_tank_disclosure"));
        assert!(joined.contains("rental_basement_water_intrusion_disclosure"));
        assert!(joined.contains("rental_sinkhole_disclosure"));
        assert!(joined.contains("rental_flood_hazard_disclosure"));
    }

    #[test]
    fn jurisdiction_truth_table_four_cells() {
        // MA stringent: 24 month inspection cap triggers
        let ma = check(&Input {
            jurisdiction: Jurisdiction::Massachusetts,
            last_inspection_months_ago: 30,
            last_pump_out_months_ago: 18,
            ..baseline()
        });
        assert_eq!(ma.severity, Severity::InspectionOverdue);

        // FL non-BMAP: no 5-year cycle
        let fl = check(&Input {
            jurisdiction: Jurisdiction::Florida,
            in_florida_bmap_basin: false,
            last_inspection_months_ago: 30,
            ..baseline()
        });
        assert_eq!(fl.severity, Severity::Compliant);

        // TX no specific inspection cycle; only pump-out cycle
        let tx = check(&Input {
            jurisdiction: Jurisdiction::Texas,
            last_inspection_months_ago: 30,
            last_pump_out_months_ago: 30,
            ..baseline()
        });
        assert_eq!(tx.severity, Severity::Compliant);

        // Default: common-law only
        let de = check(&Input {
            jurisdiction: Jurisdiction::Default,
            last_inspection_months_ago: 30,
            ..baseline()
        });
        // No statutory cap → compliant; only common-law warning
        assert_eq!(de.severity, Severity::Compliant);
    }

    #[test]
    fn ma_uniquely_strictest_inspection_invariant() {
        // Same fact pattern: 30-month-old inspection
        let ma = check(&Input {
            jurisdiction: Jurisdiction::Massachusetts,
            last_inspection_months_ago: 30,
            last_pump_out_months_ago: 18,
            ..baseline()
        });
        let fl = check(&Input {
            jurisdiction: Jurisdiction::Florida,
            in_florida_bmap_basin: false,
            last_inspection_months_ago: 30,
            ..baseline()
        });
        let tx = check(&Input {
            jurisdiction: Jurisdiction::Texas,
            last_inspection_months_ago: 30,
            last_pump_out_months_ago: 30,
            ..baseline()
        });
        // MA triggers; FL & TX don't (in same fact pattern)
        assert_eq!(ma.severity, Severity::InspectionOverdue);
        assert_eq!(fl.severity, Severity::Compliant);
        assert_eq!(tx.severity, Severity::Compliant);
    }

    #[test]
    fn fl_uniquely_bmap_triggers_60_month_cycle() {
        let bmap = check(&Input {
            jurisdiction: Jurisdiction::Florida,
            in_florida_bmap_basin: true,
            last_inspection_months_ago: 72,
            ..baseline()
        });
        let non_bmap = check(&Input {
            jurisdiction: Jurisdiction::Florida,
            in_florida_bmap_basin: false,
            last_inspection_months_ago: 72,
            ..baseline()
        });
        assert_eq!(bmap.severity, Severity::InspectionOverdue);
        assert_eq!(non_bmap.severity, Severity::Compliant);
    }

    #[test]
    fn cesspool_treatment_via_wildcat_path_when_no_permit() {
        // Cesspools (illegal in most jurisdictions) — modeled as
        // conventional treatment; permit-on-file flag drives outcome.
        let i = Input {
            system_type: SystemType::Cesspool,
            permit_on_file: false,
            ..baseline()
        };
        let out = check(&i);
        // Without permit, severity at least PermitMissing
        assert!(matches!(
            out.severity,
            Severity::PermitMissing | Severity::SystemFailure
        ));
    }

    #[test]
    fn aerobic_treatment_unit_no_permit_violation() {
        let i = Input {
            system_type: SystemType::AerobicTreatmentUnit,
            permit_on_file: true,
            operating_permit_on_file: true,
            ..baseline()
        };
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn texas_jurisdiction_with_florida_bmap_flag_ignored() {
        // FL BMAP flag is FL-specific; TX inspection cycle ignores it
        let i = Input {
            jurisdiction: Jurisdiction::Texas,
            in_florida_bmap_basin: true,
            last_inspection_months_ago: 72,
            last_pump_out_months_ago: 30,
            ..baseline()
        };
        let out = check(&i);
        // TX doesn't have a statutory inspection cycle → compliant
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn massachusetts_pumped_annually_boundary_12_months_exactly() {
        // pump_out=12 → uses 36 month cap (≤12 qualifies)
        let i = Input {
            jurisdiction: Jurisdiction::Massachusetts,
            last_pump_out_months_ago: 12,
            last_inspection_months_ago: 35,
            ..baseline()
        };
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn massachusetts_pumped_13_months_back_to_24_month_cap() {
        let i = Input {
            jurisdiction: Jurisdiction::Massachusetts,
            last_pump_out_months_ago: 13,
            last_inspection_months_ago: 25,
            ..baseline()
        };
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionOverdue);
    }
}
