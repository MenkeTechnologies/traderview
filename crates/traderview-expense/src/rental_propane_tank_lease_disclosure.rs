//! Multi-jurisdictional rental property PROPANE
//! (Liquefied Petroleum Gas / LP-Gas) TANK disclosure
//! and compliance framework. When a landlord rents a
//! property served by a propane tank (either leased from
//! supplier or owned outright), what disclosure must be
//! given to tenant, what NFPA 58 installation/clearance/
//! venting standards apply, and what failure-mode
//! liabilities expose landlord after a leak, fire, or
//! carbon monoxide event?
//!
//! Distinct from sibling modules: rental_gas_appliance_
//! ban (electrification policy), rental_chimney_fireplace_
//! inspection_disclosure (iter 471), rental_carbon_
//! monoxide_detector, rental_fire_extinguisher_requirement
//! (iter 473), rental_underground_storage_tank_disclosure
//! (UST/LUST petroleum), tenant_emotional_distress_damages
//! (IIED).
//!
//! Three-jurisdiction framework:
//!
//! 1. MASSACHUSETTS (most prescriptive amendments) — 248
//!    C.M.R. 8.00 amendments to NFPA 58 (Liquefied
//!    Petroleum Gas Code) imposed by Mass. Board of Fire
//!    Prevention Regulations; M.G.L. c. 148 § 9 gas-fitter
//!    licensing; M.G.L. c. 142 § 1 plumbing/gas authority.
//!    State amendments tighten clearances, venting, and
//!    installer-licensing beyond NFPA 58 minimums.
//! 2. NEW YORK — 19 NYCRR Department of State Uniform
//!    Code adopts NFPA 58 by reference; NY courts hold
//!    property owners and service providers responsible
//!    for damages from non-compliance — personal injury +
//!    property damage + environmental harm; insurance
//!    policies typically require compliance as condition
//!    of coverage.
//! 3. DEFAULT — NFPA 58 (Liquefied Petroleum Gas Code,
//!    2024 edition) is the universal national standard;
//!    DOT 49 C.F.R. Part 173 transport regulations + 49
//!    C.F.R. Part 192 pipeline integrity; Virginia 13
//!    VAC 5-52-580 IFC Chapter 61 LP gases; state PSC/PUC
//!    consumer-protection rules for leased-tank
//!    arrangements; common-law habitability per Hilder
//!    v. St. Peter, 478 A.2d 202 (Vt. 1984); Green v.
//!    Superior Court, 10 Cal. 3d 616 (1974); Cal. Civ.
//!    Code § 1941.1 implied warranty.
//!
//! Tank ownership framework — TWO MODELS:
//!
//! 1. LEASED TANK (most common for residential) — tank
//!    owned by propane supplier (Suburban Propane, AmeriGas,
//!    Ferrellgas, etc.); customer pays lease fee plus fuel;
//!    supplier retains right of refill access; CUSTOMER MAY
//!    BE LOCKED TO ORIGINAL SUPPLIER and prohibited from
//!    switching suppliers without paying tank removal/
//!    relocation fees; supplier typically responsible for
//!    NFPA 58 compliance + inspection
//! 2. OWNED TANK — customer purchased tank outright; full
//!    freedom to switch suppliers + comparison shop fuel;
//!    OWNER (landlord or tenant per lease) responsible for
//!    NFPA 58 compliance + inspection + maintenance +
//!    eventual replacement (typical tank lifespan 20-30
//!    years)
//!
//! NFPA 58 (2024 edition) clearance requirements (above-
//! ground horizontal tanks):
//! - 0-125 gallon residential tank: 0-foot minimum from
//!   building if tank meets ASME spec; 10-foot horizontal
//!   from window/door/source of ignition
//! - 125-500 gallon tank: 10-foot from building +
//!   25-foot from line of adjoining property
//! - 500-2000 gallon tank: 10-foot from building +
//!   50-foot from line of adjoining property
//! - Underground tank: 10-foot from building + cathodic
//!   protection required
//!
//! Universal failure-mode liability framework:
//! 1. Tank leak → fire/explosion + tort negligence + tenant
//!    emergency relocation duty (mid_tenancy_temporary_
//!    relocation)
//! 2. Improper venting/back-draft → CO poisoning + cross-
//!    reference rental_carbon_monoxide_detector + IIED
//!    parallel to tenant_emotional_distress_damages
//!    iter 453
//! 3. Frost-heave / settling dislocation → corrosion +
//!    leak risk; cathodic protection failure on buried
//!    tanks
//! 4. Tenant refill obstruction → supplier service
//!    interruption + tenant relocation duty
//! 5. Unauthorized supplier switch on leased tank →
//!    contract breach + tank removal costs ($500-$1500
//!    typical)
//!
//! Trader-landlord critical because (1) trader inheriting
//! rural property often inherits leased-tank arrangement
//! locked to specific supplier — disclosure to tenant of
//! supplier identity and lease status is essential at
//! lease execution; (2) NFPA 58 clearance violations from
//! later-built additions to home create insurer-denial
//! risk + tort negligence exposure; (3) trader cannot
//! unilaterally switch suppliers on leased tank without
//! paying removal/relocation fees; (4) tenant-paid fuel
//! vs landlord-paid fuel allocation must be specified in
//! lease (commonly tenant pays); (5) cathodic protection
//! failure on buried tank is leading cause of late-life
//! leak claims; (6) propane stove + furnace + water
//! heater combination concentrates CO/fire risk in
//! single rural-property tenancy.
//!
//! Authority: NFPA 58 (Liquefied Petroleum Gas Code, 2024
//! edition); DOT 49 C.F.R. Part 173 (transport); DOT 49
//! C.F.R. Part 192 (pipeline integrity); 248 C.M.R. 8.00
//! (Massachusetts amendments to NFPA 58); M.G.L. c. 148
//! § 9 (gas-fitter licensing); M.G.L. c. 142 § 1
//! (plumbing/gas authority); 19 NYCRR (New York
//! Department of State Uniform Code); 13 VAC 5-52-580
//! (Virginia IFC Chapter 61); Hilder v. St. Peter, 478
//! A.2d 202 (Vt. 1984); Green v. Superior Court, 10 Cal.
//! 3d 616 (1974); Cal. Civ. Code § 1941.1 implied
//! warranty.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    Massachusetts,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TankOwnership {
    LeasedFromSupplier,
    OwnedByLandlord,
    OwnedByTenant,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TankLocation {
    AboveGround,
    Underground,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub tank_ownership: TankOwnership,
    pub tank_location: TankLocation,
    pub tank_capacity_gallons: u32,
    pub clearance_from_building_feet: u32,
    pub clearance_from_window_or_ignition_feet: u32,
    pub clearance_from_property_line_feet: u32,
    pub cathodic_protection_installed: bool,
    pub supplier_identity_disclosed_in_lease: bool,
    pub supplier_lease_terms_disclosed: bool,
    pub tenant_fuel_payment_responsibility_disclosed: bool,
    pub last_supplier_inspection_months_ago: u32,
    pub leak_or_explosion_event_reported: bool,
    pub co_event_reported: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Compliant,
    DisclosureRequired,
    ClearanceViolation,
    InspectionOverdue,
    LeakOrCoEvent,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub jurisdiction_specific_actions: Vec<String>,
    pub notes: Vec<String>,
}

pub const ANNUAL_INSPECTION_MAX_MONTHS: u32 = 12;
pub const SMALL_TANK_BUILDING_CLEARANCE_FEET: u32 = 10;
pub const MID_TANK_PROPERTY_LINE_CLEARANCE_FEET: u32 = 25;
pub const LARGE_TANK_PROPERTY_LINE_CLEARANCE_FEET: u32 = 50;
pub const SMALL_TANK_CAPACITY_THRESHOLD: u32 = 125;
pub const MID_TANK_CAPACITY_THRESHOLD: u32 = 500;
pub const LARGE_TANK_CAPACITY_THRESHOLD: u32 = 2000;

pub type RentalPropaneTankLeaseDisclosureInput = Input;
pub type RentalPropaneTankLeaseDisclosureResult = Output;

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "Three-jurisdiction framework: Massachusetts (most prescriptive — 248 C.M.R. 8.00 amendments to NFPA 58 imposed by Mass. Board of Fire Prevention Regulations + M.G.L. c. 148 § 9 gas-fitter licensing + M.G.L. c. 142 § 1 plumbing/gas authority); New York (19 NYCRR Department of State Uniform Code adopting NFPA 58 by reference; NY courts hold owners/service providers responsible for personal injury + property damage + environmental harm); Default (NFPA 58 Liquefied Petroleum Gas Code 2024 edition + DOT 49 C.F.R. Part 173 transport + 49 C.F.R. Part 192 pipeline integrity + 13 VAC 5-52-580 IFC Chapter 61 + common-law habitability per Hilder v. St. Peter 478 A.2d 202 (Vt. 1984) + Cal. Civ. Code § 1941.1).".to_string(),
        "Tank ownership models: (1) LEASED TANK (most common residential) — tank owned by propane supplier (Suburban Propane, AmeriGas, Ferrellgas, etc.); customer pays lease fee plus fuel; supplier retains right of refill access; customer may be LOCKED TO ORIGINAL SUPPLIER and prohibited from switching without tank removal/relocation fees ($500-$1500 typical); supplier typically responsible for NFPA 58 compliance + inspection. (2) OWNED TANK — customer purchased outright; full freedom to switch suppliers + comparison shop; owner responsible for NFPA 58 compliance + inspection + maintenance + replacement (typical tank lifespan 20-30 years).".to_string(),
        "NFPA 58 (2024 edition) clearance requirements (above-ground horizontal tanks): 0-125 gallon residential = 0-foot minimum from building if ASME-spec compliant, 10-foot horizontal from window/door/ignition source; 125-500 gallon = 10-foot from building + 25-foot from adjoining property line; 500-2000 gallon = 10-foot from building + 50-foot from adjoining property line. Underground tank = 10-foot from building + cathodic protection required.".to_string(),
        "Five universal failure-mode liabilities: (1) tank leak → fire/explosion + tort negligence + tenant emergency relocation duty (mid_tenancy_temporary_relocation); (2) improper venting/back-draft → CO poisoning + cross-reference rental_carbon_monoxide_detector + IIED parallel to tenant_emotional_distress_damages iter 453; (3) frost-heave/settling dislocation → corrosion + leak risk + cathodic protection failure on buried tanks; (4) tenant refill obstruction → supplier service interruption + tenant relocation duty; (5) unauthorized supplier switch on leased tank → contract breach + tank removal costs.".to_string(),
        "Companion modules: rental_gas_appliance_ban (electrification), rental_chimney_fireplace_inspection_disclosure (iter 471), rental_carbon_monoxide_detector, rental_fire_extinguisher_requirement (iter 473), rental_underground_storage_tank_disclosure, mid_tenancy_temporary_relocation, tenant_emotional_distress_damages.".to_string(),
    ];
    let mut actions: Vec<String> = Vec::new();

    if matches!(input.tank_ownership, TankOwnership::None) {
        let mut n = notes;
        n.push("No propane tank present — NFPA 58 disclosure not applicable.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            jurisdiction_specific_actions: actions,
            notes: n,
        };
    }

    if input.leak_or_explosion_event_reported || input.co_event_reported {
        actions.push("Leak / explosion / CO event reported: engage emergency services + counsel; preserve evidence; tenant emergency relocation duty (mid_tenancy_temporary_relocation); tort negligence + wrongful death + IIED parallel to tenant_emotional_distress_damages iter 453; NFPA 58 + DOT investigation likely.".to_string());
    }

    let building_clearance_violation = matches!(input.tank_location, TankLocation::AboveGround)
        && input.tank_capacity_gallons > SMALL_TANK_CAPACITY_THRESHOLD
        && input.clearance_from_building_feet < SMALL_TANK_BUILDING_CLEARANCE_FEET;
    let ignition_clearance_violation = matches!(input.tank_location, TankLocation::AboveGround)
        && input.clearance_from_window_or_ignition_feet < SMALL_TANK_BUILDING_CLEARANCE_FEET;
    let property_line_violation = matches!(input.tank_location, TankLocation::AboveGround)
        && ((input.tank_capacity_gallons > SMALL_TANK_CAPACITY_THRESHOLD
            && input.tank_capacity_gallons <= MID_TANK_CAPACITY_THRESHOLD
            && input.clearance_from_property_line_feet < MID_TANK_PROPERTY_LINE_CLEARANCE_FEET)
            || (input.tank_capacity_gallons > MID_TANK_CAPACITY_THRESHOLD
                && input.tank_capacity_gallons <= LARGE_TANK_CAPACITY_THRESHOLD
                && input.clearance_from_property_line_feet
                    < LARGE_TANK_PROPERTY_LINE_CLEARANCE_FEET));
    let underground_cathodic_violation = matches!(input.tank_location, TankLocation::Underground)
        && !input.cathodic_protection_installed;

    if building_clearance_violation {
        actions.push(format!(
            "NFPA 58 clearance violation: tank {} gallons over 125-gallon threshold requires 10-foot building clearance; actual {} feet.",
            input.tank_capacity_gallons, input.clearance_from_building_feet
        ));
    }
    if ignition_clearance_violation {
        actions.push(format!(
            "NFPA 58 ignition-source clearance violation: tank must be 10 feet from any window/door/ignition source; actual {} feet.",
            input.clearance_from_window_or_ignition_feet
        ));
    }
    if property_line_violation {
        let required = if input.tank_capacity_gallons <= MID_TANK_CAPACITY_THRESHOLD {
            MID_TANK_PROPERTY_LINE_CLEARANCE_FEET
        } else {
            LARGE_TANK_PROPERTY_LINE_CLEARANCE_FEET
        };
        actions.push(format!(
            "NFPA 58 property-line clearance violation: tank {} gallons requires {}-foot clearance from adjoining property line; actual {} feet.",
            input.tank_capacity_gallons, required, input.clearance_from_property_line_feet
        ));
    }
    if underground_cathodic_violation {
        actions.push("NFPA 58 underground tank: cathodic protection REQUIRED to prevent corrosion-induced leak; install and test sacrificial-anode or impressed-current system.".to_string());
    }

    let inspection_overdue =
        input.last_supplier_inspection_months_ago > ANNUAL_INSPECTION_MAX_MONTHS;
    if inspection_overdue {
        actions.push(format!(
            "Supplier inspection overdue: {} months exceeds 12-month NFPA 58 maintenance interval; schedule supplier or independent gas-fitter inspection.",
            input.last_supplier_inspection_months_ago
        ));
    }

    let leased_disclosure_missing =
        matches!(input.tank_ownership, TankOwnership::LeasedFromSupplier)
            && (!input.supplier_identity_disclosed_in_lease
                || !input.supplier_lease_terms_disclosed);
    if leased_disclosure_missing {
        actions.push("Leased-tank disclosure missing: lease must identify propane supplier + disclose lease terms including supplier-switch restrictions + tank removal/relocation fees ($500-$1500 typical); essential for tenant fuel-cost transparency and switching feasibility.".to_string());
    }

    if !input.tenant_fuel_payment_responsibility_disclosed {
        actions.push("Fuel payment responsibility not disclosed in lease: specify whether tenant pays for fuel directly to supplier or whether landlord includes fuel in rent; common-area shared tanks require submetering or equitable allocation methodology.".to_string());
    }

    match input.jurisdiction {
        Jurisdiction::Massachusetts => {
            actions.push("Massachusetts: 248 C.M.R. 8.00 amendments to NFPA 58 tighten clearances and installer-licensing beyond NFPA 58 minimums; M.G.L. c. 148 § 9 requires licensed Massachusetts gas-fitter for installation/maintenance; M.G.L. c. 142 § 1 plumbing/gas authority.".to_string());
        }
        Jurisdiction::NewYork => {
            actions.push("New York: 19 NYCRR Department of State Uniform Code adopting NFPA 58 by reference; NY courts hold property owners and service providers responsible for damages from non-compliance including personal injury + property damage + environmental harm; insurance compliance condition.".to_string());
        }
        Jurisdiction::Default => {
            actions.push("Default jurisdiction: NFPA 58 Liquefied Petroleum Gas Code 2024 edition; DOT 49 C.F.R. Part 173 transport + 49 C.F.R. Part 192 pipeline integrity; 13 VAC 5-52-580 IFC Chapter 61 + common-law implied warranty of habitability per Hilder v. St. Peter, 478 A.2d 202 (Vt. 1984) + Cal. Civ. Code § 1941.1.".to_string());
        }
    }

    let clearance_violation = building_clearance_violation
        || ignition_clearance_violation
        || property_line_violation
        || underground_cathodic_violation;

    let severity = if input.leak_or_explosion_event_reported || input.co_event_reported {
        Severity::LeakOrCoEvent
    } else if clearance_violation {
        Severity::ClearanceViolation
    } else if inspection_overdue {
        Severity::InspectionOverdue
    } else if leased_disclosure_missing || !input.tenant_fuel_payment_responsibility_disclosed {
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
            tank_ownership: TankOwnership::LeasedFromSupplier,
            tank_location: TankLocation::AboveGround,
            tank_capacity_gallons: 250,
            clearance_from_building_feet: 12,
            clearance_from_window_or_ignition_feet: 12,
            clearance_from_property_line_feet: 30,
            cathodic_protection_installed: false,
            supplier_identity_disclosed_in_lease: true,
            supplier_lease_terms_disclosed: true,
            tenant_fuel_payment_responsibility_disclosed: true,
            last_supplier_inspection_months_ago: 6,
            leak_or_explosion_event_reported: false,
            co_event_reported: false,
        }
    }

    #[test]
    fn no_tank_not_applicable() {
        let mut i = baseline();
        i.tank_ownership = TankOwnership::None;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn ma_compliant_baseline() {
        let i = baseline();
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn leak_event_top_severity() {
        let mut i = baseline();
        i.leak_or_explosion_event_reported = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::LeakOrCoEvent);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Leak / explosion / CO event"));
        assert!(joined.contains("tenant_emotional_distress_damages"));
    }

    #[test]
    fn co_event_top_severity() {
        let mut i = baseline();
        i.co_event_reported = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::LeakOrCoEvent);
    }

    #[test]
    fn building_clearance_violation_under_10_feet() {
        let mut i = baseline();
        i.tank_capacity_gallons = 300;
        i.clearance_from_building_feet = 5;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ClearanceViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("10-foot building clearance"));
    }

    #[test]
    fn building_clearance_exactly_10_feet_compliant() {
        let mut i = baseline();
        i.tank_capacity_gallons = 300;
        i.clearance_from_building_feet = 10;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn small_tank_under_125_no_building_clearance_violation() {
        let mut i = baseline();
        i.tank_capacity_gallons = 100;
        i.clearance_from_building_feet = 0; // ASME-spec compliant 0-foot
        i.clearance_from_window_or_ignition_feet = 12; // still need 10ft from ignition
        let out = check(&i);
        // Building clearance only applies > 125 gallons
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn ignition_clearance_violation_under_10_feet() {
        let mut i = baseline();
        i.clearance_from_window_or_ignition_feet = 5;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ClearanceViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("ignition-source clearance"));
        assert!(joined.contains("10 feet from any window/door"));
    }

    #[test]
    fn property_line_violation_mid_tank_under_25_feet() {
        let mut i = baseline();
        i.tank_capacity_gallons = 400; // 125-500 range
        i.clearance_from_property_line_feet = 20;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ClearanceViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("25-foot"));
        assert!(joined.contains("property line"));
    }

    #[test]
    fn property_line_violation_large_tank_under_50_feet() {
        let mut i = baseline();
        i.tank_capacity_gallons = 1000;
        i.clearance_from_property_line_feet = 30;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ClearanceViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("50-foot"));
    }

    #[test]
    fn underground_tank_no_cathodic_protection_violation() {
        let mut i = baseline();
        i.tank_location = TankLocation::Underground;
        i.cathodic_protection_installed = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ClearanceViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("cathodic protection REQUIRED"));
        assert!(joined.contains("sacrificial-anode"));
    }

    #[test]
    fn underground_tank_with_cathodic_protection_compliant() {
        let mut i = baseline();
        i.tank_location = TankLocation::Underground;
        i.cathodic_protection_installed = true;
        // Property-line clearance doesn't apply underground in this model
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn supplier_inspection_overdue_over_12_months() {
        let mut i = baseline();
        i.last_supplier_inspection_months_ago = 18;
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionOverdue);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("12-month NFPA 58"));
    }

    #[test]
    fn supplier_inspection_exactly_12_months_compliant() {
        let mut i = baseline();
        i.last_supplier_inspection_months_ago = 12;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn leased_tank_supplier_not_disclosed_disclosure_required() {
        let mut i = baseline();
        i.supplier_identity_disclosed_in_lease = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DisclosureRequired);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Leased-tank disclosure"));
        assert!(joined.contains("$500-$1500"));
    }

    #[test]
    fn leased_tank_terms_not_disclosed_disclosure_required() {
        let mut i = baseline();
        i.supplier_lease_terms_disclosed = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DisclosureRequired);
    }

    #[test]
    fn owned_tank_no_supplier_disclosure_needed() {
        let mut i = baseline();
        i.tank_ownership = TankOwnership::OwnedByLandlord;
        i.supplier_identity_disclosed_in_lease = false;
        i.supplier_lease_terms_disclosed = false;
        let out = check(&i);
        // Owned tank — no leased-supplier disclosure required
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn fuel_payment_not_disclosed_disclosure_required() {
        let mut i = baseline();
        i.tenant_fuel_payment_responsibility_disclosed = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DisclosureRequired);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("Fuel payment responsibility"));
    }

    #[test]
    fn ma_jurisdiction_cites_248_cmr_and_148() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Massachusetts;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("248 C.M.R. 8.00"));
        assert!(joined.contains("M.G.L. c. 148 § 9"));
        assert!(joined.contains("M.G.L. c. 142 § 1"));
    }

    #[test]
    fn ny_jurisdiction_cites_19_nycrr() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("19 NYCRR"));
        assert!(joined.contains("personal injury"));
        assert!(joined.contains("insurance compliance"));
    }

    #[test]
    fn default_jurisdiction_common_law_habitability() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let out = check(&i);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("DOT 49 C.F.R. Part 173"));
        assert!(joined.contains("13 VAC 5-52-580"));
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("§ 1941.1"));
    }

    #[test]
    fn severity_priority_event_above_clearance_above_inspection_above_disclosure() {
        let mut i = baseline();
        i.leak_or_explosion_event_reported = true;
        i.tank_capacity_gallons = 300;
        i.clearance_from_building_feet = 5;
        i.last_supplier_inspection_months_ago = 24;
        i.supplier_identity_disclosed_in_lease = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::LeakOrCoEvent);
    }

    #[test]
    fn severity_clearance_above_inspection_above_disclosure() {
        let mut i = baseline();
        i.tank_capacity_gallons = 300;
        i.clearance_from_building_feet = 5;
        i.last_supplier_inspection_months_ago = 24;
        i.supplier_identity_disclosed_in_lease = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ClearanceViolation);
    }

    #[test]
    fn severity_inspection_above_disclosure() {
        let mut i = baseline();
        i.last_supplier_inspection_months_ago = 24;
        i.supplier_identity_disclosed_in_lease = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::InspectionOverdue);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("NFPA 58"));
        assert!(joined.contains("DOT 49 C.F.R. Part 173"));
        assert!(joined.contains("49 C.F.R. Part 192"));
        assert!(joined.contains("248 C.M.R. 8.00"));
        assert!(joined.contains("M.G.L. c. 148 § 9"));
        assert!(joined.contains("M.G.L. c. 142 § 1"));
        assert!(joined.contains("19 NYCRR"));
        assert!(joined.contains("13 VAC 5-52-580"));
        assert!(joined.contains("Hilder v. St. Peter"));
        assert!(joined.contains("478 A.2d 202"));
        assert!(joined.contains("§ 1941.1"));
    }

    #[test]
    fn note_pins_three_jurisdiction_framework() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("Massachusetts (most prescriptive"));
        assert!(joined.contains("New York"));
        assert!(joined.contains("Default"));
    }

    #[test]
    fn note_pins_two_ownership_models() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("LEASED TANK"));
        assert!(joined.contains("OWNED TANK"));
        assert!(joined.contains("Suburban Propane"));
        assert!(joined.contains("AmeriGas"));
        assert!(joined.contains("Ferrellgas"));
        assert!(joined.contains("20-30 years"));
    }

    #[test]
    fn note_pins_nfpa_58_clearance_table() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("0-125 gallon"));
        assert!(joined.contains("125-500 gallon"));
        assert!(joined.contains("500-2000 gallon"));
        assert!(joined.contains("cathodic protection"));
    }

    #[test]
    fn note_pins_five_failure_modes() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("tank leak"));
        assert!(joined.contains("improper venting"));
        assert!(joined.contains("frost-heave"));
        assert!(joined.contains("refill obstruction"));
        assert!(joined.contains("supplier switch"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("rental_gas_appliance_ban"));
        assert!(joined.contains("rental_chimney_fireplace_inspection_disclosure"));
        assert!(joined.contains("rental_carbon_monoxide_detector"));
        assert!(joined.contains("rental_fire_extinguisher_requirement"));
    }

    #[test]
    fn jurisdiction_truth_table_three_cells() {
        let ma = check(&Input {
            jurisdiction: Jurisdiction::Massachusetts,
            ..baseline()
        });
        let ny = check(&Input {
            jurisdiction: Jurisdiction::NewYork,
            ..baseline()
        });
        let de = check(&Input {
            jurisdiction: Jurisdiction::Default,
            ..baseline()
        });
        assert_eq!(ma.severity, Severity::Compliant);
        assert_eq!(ny.severity, Severity::Compliant);
        assert_eq!(de.severity, Severity::Compliant);
    }

    #[test]
    fn tank_ownership_truth_table_four_cells() {
        // Leased + full disclosure = compliant
        let c1 = check(&Input {
            tank_ownership: TankOwnership::LeasedFromSupplier,
            ..baseline()
        });
        assert_eq!(c1.severity, Severity::Compliant);

        // Owned by landlord + no supplier disclosure needed = compliant
        let c2 = check(&Input {
            tank_ownership: TankOwnership::OwnedByLandlord,
            supplier_identity_disclosed_in_lease: false,
            supplier_lease_terms_disclosed: false,
            ..baseline()
        });
        assert_eq!(c2.severity, Severity::Compliant);

        // Owned by tenant — landlord still required to manage fuel-payment disclosure
        let c3 = check(&Input {
            tank_ownership: TankOwnership::OwnedByTenant,
            ..baseline()
        });
        assert_eq!(c3.severity, Severity::Compliant);

        // None = not applicable
        let c4 = check(&Input {
            tank_ownership: TankOwnership::None,
            ..baseline()
        });
        assert_eq!(c4.severity, Severity::NotApplicable);
    }

    #[test]
    fn multiple_clearance_violations_stack() {
        let mut i = baseline();
        i.tank_capacity_gallons = 1000;
        i.clearance_from_building_feet = 5;
        i.clearance_from_window_or_ignition_feet = 6;
        i.clearance_from_property_line_feet = 20;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ClearanceViolation);
        let joined = out.jurisdiction_specific_actions.join(" ");
        assert!(joined.contains("10-foot building"));
        assert!(joined.contains("ignition-source"));
        assert!(joined.contains("50-foot"));
    }
}
