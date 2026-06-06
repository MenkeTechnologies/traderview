//! Rental in-unit laundry appliance provision plus dryer-vent fire safety
//! and mold liability framework.
//!
//! Most US states do NOT mandate landlord provision of washer / dryer per
//! statute. California AB 628 (effective January 1, 2026) requires only
//! refrigerator and stove. Once a landlord PROVIDES washer / dryer as part
//! of the rental, however, the landlord assumes a continuing duty to
//! maintain those appliances in safe operating condition under common-law
//! habitability doctrine; lapse triggers tenant remedies including rent
//! withholding, repair-and-deduct, and constructive-eviction claims.
//!
//! Five operational concerns: (1) landlord-provided washer / dryer in
//! disrepair triggers habitability breach; (2) improper dryer venting
//! into attic, crawlspace, or unconditioned space — most common safety
//! violation per InterNACHI surveys — causes mold growth and structural
//! damage; (3) flex transition duct exceeding 8 feet OR non-UL 2158A
//! plastic / foil construction is fire hazard; (4) dryer-vent lint
//! buildup uncleaned annually causes approximately 2,900 residential
//! dryer fires per year per US Fire Administration data resulting in $35M
//! annual property damage; (5) gas dryer without proper combustion-air
//! ventilation causes CO accumulation.
//!
//! Federal: 24 C.F.R. § 3280.708 HUD manufactured home standards plus IRC
//! Section M1502 (mechanical code dryer exhaust) plus NFPA 211 Standard
//! for Chimneys, Fireplaces, Vents, and Solid Fuel-Burning Appliances
//! (cross-referenced for dryer-vent fire prevention) plus IFC 504 (fuel-
//! gas appliance vent) plus UL 2158A Clothes Dryer Transition Duct.
//!
//! State frameworks:
//!
//! - **California** — AB 628 effective January 1, 2026 (refrigerator and
//!   stove only — no W/D mandate); Cal. Civ. Code § 1941.1 habitability
//!   includes mold per Health and Safety Code § 17920.3; Cal. Civ. Code §
//!   1942 self-help repair after 30-day notice.
//!
//! - **Massachusetts** — M.G.L. ch. 186 § 14 quiet enjoyment plus 105 CMR
//!   410 State Sanitary Code Chapter II (Minimum Standards of Fitness for
//!   Human Habitation) plus Mass.gov DFS Dryer Fire Safety guidance plus
//!   527 CMR 50.00 fire prevention regulations.
//!
//! - **New York** — Real Property Law § 235-b implied warranty of
//!   habitability; NYC Admin Code § 27-2017.3 mold remediation standards
//!   (Local Law 55 of 2018); MDL Multiple Dwelling Law § 78 maintenance
//!   of dwelling; NYC HPC inspections.
//!
//! - **Washington** — RCW 59.18.060 implied warranty of habitability plus
//!   RCW 59.18.060(11) mold disclosure (Disclosure of Mold and Indoor Air
//!   Quality at Tenant Move-In Act); Department of Health mold guidance.
//!
//! - **Default** — common-law implied warranty of habitability plus NFPA
//!   211 fire-prevention recommendations plus IRC Section M1502 mechanical
//!   code dryer-exhaust requirements.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    Massachusetts,
    NewYork,
    Washington,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplianceProvisionStatus {
    /// No washer or dryer in unit.
    NoLaundryAppliances,
    /// Tenant-owned and -installed appliances; landlord has NO maintenance
    /// duty.
    TenantOwnedTenantMaintained,
    /// Landlord-provided appliances with continuing maintenance duty.
    LandlordProvidedWithContinuingDuty,
    /// Shared coin-op laundry room (separate framework — distinct sibling).
    SharedCoinOpLaundryRoom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DryerVentTermination {
    /// Exterior wall termination with backdraft damper — code-compliant.
    ExteriorWallWithBackdraftDamper,
    /// Vents into attic — IRC M1502.2 prohibited; causes mold.
    AtticTerminationProhibited,
    /// Vents into crawlspace — IRC M1502.2 prohibited; causes mold.
    CrawlspaceTerminationProhibited,
    /// Vents into garage or unconditioned space — fire and CO risk.
    GarageOrUnconditionedSpace,
    /// No dryer present.
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantWithMaintenanceCadence,
    LandlordProvidedRepairOverdueHabitabilityBreach,
    ImproperVentingAtticOrCrawlspaceMoldRisk,
    NonCompliantTransitionDuctFireRisk,
    LintBuildupAnnualMaintenanceOverdueFireRisk,
    GasDryerCoExposureWithoutAdequateVentilation,
    DisclosureRequiredAtLeaseSigning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub provision_status: ApplianceProvisionStatus,
    pub dryer_vent_termination: DryerVentTermination,
    pub transition_duct_length_feet: u32,
    pub transition_duct_ul_2158a_certified: bool,
    pub transition_duct_is_metal: bool,
    pub annual_lint_cleaning_within_12_months: bool,
    pub dryer_fuel_is_gas: bool,
    pub gas_dryer_combustion_air_ventilation_compliant: bool,
    pub appliance_in_disrepair_with_overdue_tenant_notice: bool,
    pub disclosure_provided_at_lease_signing: bool,
    pub annual_rent_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub recommended_actions: Vec<String>,
    pub annual_rent_at_risk_cents: u64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const UL_2158A_TRANSITION_DUCT_MAX_LENGTH_FEET: u32 = 8;
pub const NFPA_211_LINT_CLEANING_INTERVAL_MONTHS: u32 = 12;
pub const US_DRYER_FIRES_ANNUALLY: u32 = 2_900;

pub fn check(input: &Input) -> Output {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let severity: Severity;

    if matches!(
        input.provision_status,
        ApplianceProvisionStatus::NoLaundryAppliances
    ) {
        notes.push(
            "No washer or dryer in unit. Note that California AB 628 effective January 1, \
             2026 requires only refrigerator and stove provision — no statewide W/D mandate. \
             If shared coin-op laundry exists on premises, route to separate framework."
                .to_string(),
        );
        return Output {
            severity: Severity::NotApplicable,
            recommended_actions: actions,
            annual_rent_at_risk_cents: 0,
            citation: "Cal. Civ. Code § 1941.1 + AB 628 (CA); common-law habitability",
            notes,
        };
    }

    if matches!(
        input.provision_status,
        ApplianceProvisionStatus::TenantOwnedTenantMaintained
    ) {
        notes.push(
            "Tenant-owned and -installed appliances; landlord has NO continuing maintenance \
             duty under common-law habitability doctrine. Landlord retains duty to maintain \
             utility connections (water supply, drain, gas line, electrical service, dryer \
             vent enclosure) but NOT the appliance itself. Tenant bears risk of failure plus \
             repair plus replacement."
                .to_string(),
        );
        return Output {
            severity: Severity::NotApplicable,
            recommended_actions: actions,
            annual_rent_at_risk_cents: 0,
            citation: "Common-law habitability; tenant-owned exception",
            notes,
        };
    }

    let is_landlord_provided = matches!(
        input.provision_status,
        ApplianceProvisionStatus::LandlordProvidedWithContinuingDuty
            | ApplianceProvisionStatus::SharedCoinOpLaundryRoom
    );

    if matches!(
        input.dryer_vent_termination,
        DryerVentTermination::AtticTerminationProhibited
            | DryerVentTermination::CrawlspaceTerminationProhibited
    ) {
        severity = Severity::ImproperVentingAtticOrCrawlspaceMoldRisk;
        actions.push(
            "Dryer venting into attic or crawlspace VIOLATES IRC Section M1502.2 (Duct \
             Termination) which requires termination to the OUTSIDE of the building with \
             backdraft damper. Moisture deposition causes mold growth, wood decay, ceiling \
             damage. Re-route exhaust through exterior wall within 7 days; restore visible \
             mold per Cal. Health & Safety Code § 17920.3 (CA) or NYC Local Law 55 of 2018 \
             remediation standards (NY). Tenant may invoke rent withholding plus self-help \
             remediation plus emotional-distress claim if exposure continues."
                .to_string(),
        );
    } else if !input.transition_duct_is_metal {
        severity = Severity::NonCompliantTransitionDuctFireRisk;
        actions.push(
            "Non-metal transition duct (plastic / foil flex) violates IRC Section M1502.4 \
             plus NFPA 211 which mandates rigid metal or semi-rigid aluminum smooth-interior \
             construction; plastic and foil retain lint accumulation and ignite at lower \
             temperature than UL 2158A-certified metal duct (430°F rating). Replace with UL \
             2158A-certified semi-rigid aluminum or rigid metal duct within 7 days."
                .to_string(),
        );
    } else if !input.transition_duct_ul_2158a_certified
        || input.transition_duct_length_feet > UL_2158A_TRANSITION_DUCT_MAX_LENGTH_FEET
    {
        severity = Severity::NonCompliantTransitionDuctFireRisk;
        actions.push(format!(
            "Transition duct exceeds UL 2158A 8-foot maximum single-length limitation OR is \
             not UL 2158A-certified. Current length: {} feet; max: {} feet. Replace with UL \
             2158A-certified semi-rigid aluminum duct rated to 430°F; shorten run to under \
             8 feet or extend rigid metal exhaust duct to within 8 feet of dryer.",
            input.transition_duct_length_feet, UL_2158A_TRANSITION_DUCT_MAX_LENGTH_FEET
        ));
    } else if input.dryer_fuel_is_gas && !input.gas_dryer_combustion_air_ventilation_compliant {
        severity = Severity::GasDryerCoExposureWithoutAdequateVentilation;
        actions.push(
            "Gas dryer without adequate combustion-air ventilation per IFC 504 plus IRC \
             Section G2407 causes incomplete combustion plus CO accumulation. Install \
             dedicated combustion-air supply opening sized per IRC G2407.5 (50 sq in per \
             1,000 BTU input) OR convert to direct-vent appliance. Verify CO detector \
             adjacent to gas appliance per state code (universal requirement)."
                .to_string(),
        );
    } else if is_landlord_provided && input.appliance_in_disrepair_with_overdue_tenant_notice {
        severity = Severity::LandlordProvidedRepairOverdueHabitabilityBreach;
        actions.push(
            "Landlord-provided washer or dryer in disrepair with overdue tenant repair \
             notice triggers continuing-duty breach under Cal. Civ. Code § 1941 (CA), M.G.L. \
             ch. 186 § 14 (MA), Real Property Law § 235-b (NY), or RCW 59.18.060 (WA). \
             Repair or replace within statutory time window (typically 30 days from written \
             notice); failure exposes landlord to tenant rent withholding plus repair-and-\
             deduct plus constructive-eviction claim."
                .to_string(),
        );
    } else if !input.annual_lint_cleaning_within_12_months {
        severity = Severity::LintBuildupAnnualMaintenanceOverdueFireRisk;
        actions.push(format!(
            "Annual dryer-vent lint cleaning overdue beyond NFPA 211 {}-month interval; \
             approximately {} US residential dryer fires occur annually per US Fire \
             Administration causing $35M property damage. Schedule certified dryer-vent \
             cleaning per IAQA Standard 3000 (Indoor Air Quality Association) within 30 \
             days; document with receipt retained in landlord file.",
            NFPA_211_LINT_CLEANING_INTERVAL_MONTHS, US_DRYER_FIRES_ANNUALLY
        ));
    } else if is_landlord_provided && !input.disclosure_provided_at_lease_signing {
        severity = Severity::DisclosureRequiredAtLeaseSigning;
        actions.push(
            "Landlord-provided laundry appliances exist but no written disclosure at lease \
             signing; create disclosure addendum covering (1) appliance make / model / \
             serial / age, (2) repair responsibility allocation, (3) lint-cleaning cadence \
             with last service date, (4) dryer-vent termination location, (5) gas vs \
             electric fuel type, (6) tenant notice procedure for repair requests, (7) \
             after-hours emergency contact for water-leak events."
                .to_string(),
        );
    } else {
        severity = Severity::CompliantWithMaintenanceCadence;
        actions.push(format!(
            "Compliant: appliances maintained, vent termination to exterior, UL 2158A \
             transition duct under {} feet, annual lint cleaning within {} months, no gas \
             dryer CO risk. Maintain documentation of (1) UL 2158A duct invoice, (2) annual \
             cleaning receipt, (3) appliance-warranty record, (4) tenant-acknowledged \
             operating manual.",
            UL_2158A_TRANSITION_DUCT_MAX_LENGTH_FEET, NFPA_211_LINT_CLEANING_INTERVAL_MONTHS
        ));
    }

    match input.jurisdiction {
        Jurisdiction::California => {
            notes.push(
                "Cal. Civ. Code § 1941.1 implied warranty of habitability includes mold per \
                 Cal. Health & Safety Code § 17920.3 substandard housing classification; \
                 Cal. Civ. Code § 1942 self-help repair after 30-day notice; AB 628 \
                 effective January 1, 2026 mandates refrigerator and stove only (no W/D). \
                 Landlord retains continuing duty once appliance is provided."
                    .to_string(),
            );
        }
        Jurisdiction::Massachusetts => {
            notes.push(
                "M.G.L. ch. 186 § 14 quiet enjoyment plus 105 CMR 410 State Sanitary Code \
                 Chapter II (Minimum Standards of Fitness for Human Habitation) plus 527 \
                 CMR 50.00 fire prevention regulations; Mass.gov DFS Dryer Fire Safety \
                 guidance recommends annual professional lint cleaning."
                    .to_string(),
            );
        }
        Jurisdiction::NewYork => {
            notes.push(
                "Real Property Law § 235-b implied warranty of habitability; NYC Admin Code \
                 § 27-2017.3 mold remediation standards under NYC Local Law 55 of 2018; MDL \
                 § 78 maintenance of dwelling; NYC HPC inspections enforce. NYC DOB \
                 enforces IRC M1502 dryer-vent termination requirements through Building \
                 Code Chapter 5."
                    .to_string(),
            );
        }
        Jurisdiction::Washington => {
            notes.push(
                "RCW 59.18.060 implied warranty of habitability; RCW 59.18.060(11) \
                 Disclosure of Mold and Indoor Air Quality at Tenant Move-In Act requires \
                 landlord to provide DOH mold disclosure before tenant occupies unit; \
                 Washington State Department of Health (WA DOH) mold guidance publication \
                 'Got Mold?' is statutorily-required attachment."
                    .to_string(),
            );
        }
        Jurisdiction::Default => {
            notes.push(
                "Common-law implied warranty of habitability plus NFPA 211 fire-prevention \
                 recommendations plus IRC Section M1502 mechanical code dryer-exhaust \
                 requirements (termination to outside, backdraft damper, smooth interior \
                 metal duct, 25-foot maximum total length minus 5 feet per 45-degree elbow \
                 minus 5 feet per 90-degree elbow). IFC 504 governs fuel-gas appliance \
                 vent."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Coordination with [[tenant_in_unit_appliance_repair_responsibility]] (general \
         landlord-provided appliance maintenance duty framework), [[rental_chimney_fireplace_\
         inspection_disclosure]] (NFPA 211 cross-reference for solid-fuel-burning chimney \
         analog), [[rental_natural_gas_leak_response]] (gas dryer CO + leak protocol), \
         [[rental_gas_appliance_ban]] (electrification mandate context — converting gas \
         dryer to electric), [[rental_pellet_stove_disclosure]] (iter 499 CO detector cross-\
         reference), [[rental_basement_water_intrusion_disclosure]] (laundry-room water \
         leak source)."
            .to_string(),
    );

    let annual_rent_at_risk: u64 = match severity {
        Severity::ImproperVentingAtticOrCrawlspaceMoldRisk
        | Severity::NonCompliantTransitionDuctFireRisk
        | Severity::GasDryerCoExposureWithoutAdequateVentilation
        | Severity::LandlordProvidedRepairOverdueHabitabilityBreach => input.annual_rent_cents,
        Severity::LintBuildupAnnualMaintenanceOverdueFireRisk
        | Severity::DisclosureRequiredAtLeaseSigning => input.annual_rent_cents.saturating_div(2),
        _ => 0,
    };

    Output {
        severity,
        recommended_actions: actions,
        annual_rent_at_risk_cents: annual_rent_at_risk,
        citation: match input.jurisdiction {
            Jurisdiction::California => "Cal. Civ. Code § 1941.1 + § 1942 + § 17920.3 + AB 628",
            Jurisdiction::Massachusetts => "M.G.L. ch. 186 § 14 + 105 CMR 410 + 527 CMR 50.00",
            Jurisdiction::NewYork => "Real Property Law § 235-b + NYC Admin § 27-2017.3 + MDL § 78",
            Jurisdiction::Washington => "RCW 59.18.060 + 59.18.060(11) + DOH mold guidance",
            Jurisdiction::Default => "IRC M1502 + NFPA 211 + IFC 504 + common-law habitability",
        },
        notes,
    }
}

pub type RentalInUnitLaundryApplianceProvisionInput = Input;
pub type RentalInUnitLaundryApplianceProvisionResult = Output;

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            provision_status: ApplianceProvisionStatus::LandlordProvidedWithContinuingDuty,
            dryer_vent_termination: DryerVentTermination::ExteriorWallWithBackdraftDamper,
            transition_duct_length_feet: 6,
            transition_duct_ul_2158a_certified: true,
            transition_duct_is_metal: true,
            annual_lint_cleaning_within_12_months: true,
            dryer_fuel_is_gas: false,
            gas_dryer_combustion_air_ventilation_compliant: true,
            appliance_in_disrepair_with_overdue_tenant_notice: false,
            disclosure_provided_at_lease_signing: true,
            annual_rent_cents: 36_000_00,
        }
    }

    #[test]
    fn no_laundry_appliances_not_applicable() {
        let mut i = baseline();
        i.provision_status = ApplianceProvisionStatus::NoLaundryAppliances;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
        assert!(r.notes.iter().any(|n| n.contains("AB 628")));
    }

    #[test]
    fn tenant_owned_no_landlord_duty() {
        let mut i = baseline();
        i.provision_status = ApplianceProvisionStatus::TenantOwnedTenantMaintained;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
        assert!(r.notes.iter().any(|n| n.contains("dryer vent enclosure")));
    }

    #[test]
    fn compliant_with_maintenance_cadence() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CompliantWithMaintenanceCadence
        ));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn attic_termination_mold_risk_full_rent() {
        let mut i = baseline();
        i.dryer_vent_termination = DryerVentTermination::AtticTerminationProhibited;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::ImproperVentingAtticOrCrawlspaceMoldRisk
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("IRC Section M1502.2")));
    }

    #[test]
    fn crawlspace_termination_mold_risk_full_rent() {
        let mut i = baseline();
        i.dryer_vent_termination = DryerVentTermination::CrawlspaceTerminationProhibited;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::ImproperVentingAtticOrCrawlspaceMoldRisk
        ));
    }

    #[test]
    fn plastic_foil_duct_non_metal_fire_risk() {
        let mut i = baseline();
        i.transition_duct_is_metal = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::NonCompliantTransitionDuctFireRisk
        ));
        assert!(r.recommended_actions.iter().any(|a| a.contains("430°F")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("UL 2158A")));
    }

    #[test]
    fn transition_duct_over_8_feet_fire_risk() {
        let mut i = baseline();
        i.transition_duct_length_feet = 12;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::NonCompliantTransitionDuctFireRisk
        ));
        assert!(r.recommended_actions.iter().any(|a| a.contains("8 feet")));
    }

    #[test]
    fn transition_duct_at_exactly_8_feet_compliant() {
        let mut i = baseline();
        i.transition_duct_length_feet = 8;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CompliantWithMaintenanceCadence
        ));
    }

    #[test]
    fn not_ul_2158a_certified_fire_risk() {
        let mut i = baseline();
        i.transition_duct_ul_2158a_certified = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::NonCompliantTransitionDuctFireRisk
        ));
    }

    #[test]
    fn gas_dryer_without_ventilation_co_risk() {
        let mut i = baseline();
        i.dryer_fuel_is_gas = true;
        i.gas_dryer_combustion_air_ventilation_compliant = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::GasDryerCoExposureWithoutAdequateVentilation
        ));
        assert!(r.recommended_actions.iter().any(|a| a.contains("IFC 504")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("IRC G2407.5")));
    }

    #[test]
    fn gas_dryer_with_compliant_ventilation_no_co_severity() {
        let mut i = baseline();
        i.dryer_fuel_is_gas = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CompliantWithMaintenanceCadence
        ));
    }

    #[test]
    fn landlord_provided_overdue_repair_habitability_breach() {
        let mut i = baseline();
        i.appliance_in_disrepair_with_overdue_tenant_notice = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::LandlordProvidedRepairOverdueHabitabilityBreach
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
    }

    #[test]
    fn lint_cleaning_overdue_half_rent_at_risk() {
        let mut i = baseline();
        i.annual_lint_cleaning_within_12_months = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::LintBuildupAnnualMaintenanceOverdueFireRisk
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains(&US_DRYER_FIRES_ANNUALLY.to_string())));
    }

    #[test]
    fn lint_action_pins_iaqa_standard_3000() {
        let mut i = baseline();
        i.annual_lint_cleaning_within_12_months = false;
        let r = check(&i);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("IAQA Standard 3000")));
    }

    #[test]
    fn disclosure_missing_at_lease_signing_half_rent() {
        let mut i = baseline();
        i.disclosure_provided_at_lease_signing = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::DisclosureRequiredAtLeaseSigning
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
    }

    #[test]
    fn ca_jurisdiction_pins_civ_code_1941_1_and_ab_628() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 1941.1")));
        assert!(r.notes.iter().any(|n| n.contains("AB 628")));
        assert!(r.notes.iter().any(|n| n.contains("§ 17920.3")));
    }

    #[test]
    fn ma_jurisdiction_pins_mgl_ch_186_14_and_105_cmr_410() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Massachusetts;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("M.G.L. ch. 186 § 14")));
        assert!(r.notes.iter().any(|n| n.contains("105 CMR 410")));
        assert!(r.notes.iter().any(|n| n.contains("527 CMR 50.00")));
    }

    #[test]
    fn ny_jurisdiction_pins_rpl_235_b_and_local_law_55() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Real Property Law § 235-b")));
        assert!(r.notes.iter().any(|n| n.contains("Local Law 55 of 2018")));
        assert!(r.notes.iter().any(|n| n.contains("MDL § 78")));
    }

    #[test]
    fn wa_jurisdiction_pins_rcw_59_18_060_and_doh_got_mold() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Washington;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("RCW 59.18.060")));
        assert!(r.notes.iter().any(|n| n.contains("Got Mold?")));
    }

    #[test]
    fn default_jurisdiction_pins_irc_m1502_and_nfpa_211() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("IRC Section M1502")));
        assert!(r.notes.iter().any(|n| n.contains("NFPA 211")));
        assert!(r.notes.iter().any(|n| n.contains("IFC 504")));
    }

    #[test]
    fn coordination_note_references_siblings() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("tenant_in_unit_appliance_repair_responsibility")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_natural_gas_leak_response")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_gas_appliance_ban")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_pellet_stove_disclosure")));
    }

    #[test]
    fn coordination_note_pinned_in_all_jurisdictions() {
        for j in [
            Jurisdiction::California,
            Jurisdiction::Massachusetts,
            Jurisdiction::NewYork,
            Jurisdiction::Washington,
            Jurisdiction::Default,
        ] {
            let mut i = baseline();
            i.jurisdiction = j;
            let r = check(&i);
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("tenant_in_unit_appliance_repair_responsibility")),
                "coordination missing for {j:?}"
            );
        }
    }

    #[test]
    fn ul_2158a_max_length_constant_pins_8_feet() {
        assert_eq!(UL_2158A_TRANSITION_DUCT_MAX_LENGTH_FEET, 8);
    }

    #[test]
    fn nfpa_211_lint_cleaning_interval_pins_12_months() {
        assert_eq!(NFPA_211_LINT_CLEANING_INTERVAL_MONTHS, 12);
    }

    #[test]
    fn us_dryer_fires_constant_pins_2900_annually() {
        assert_eq!(US_DRYER_FIRES_ANNUALLY, 2_900);
    }

    #[test]
    fn severity_priority_attic_overrides_lint() {
        let mut i = baseline();
        i.dryer_vent_termination = DryerVentTermination::AtticTerminationProhibited;
        i.annual_lint_cleaning_within_12_months = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::ImproperVentingAtticOrCrawlspaceMoldRisk
        ));
    }

    #[test]
    fn severity_priority_metal_check_overrides_length() {
        let mut i = baseline();
        i.transition_duct_is_metal = false;
        i.transition_duct_length_feet = 100;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::NonCompliantTransitionDuctFireRisk
        ));
    }

    #[test]
    fn zero_rent_zero_risk_does_not_panic() {
        let mut i = baseline();
        i.annual_rent_cents = 0;
        i.dryer_vent_termination = DryerVentTermination::AtticTerminationProhibited;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn citation_branch_for_each_jurisdiction() {
        let ca = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::California;
            i
        });
        let ma = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::Massachusetts;
            i
        });
        let ny = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::NewYork;
            i
        });
        let wa = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::Washington;
            i
        });
        let de = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::Default;
            i
        });
        assert!(ca.citation.contains("AB 628"));
        assert!(ma.citation.contains("105 CMR 410"));
        assert!(ny.citation.contains("MDL § 78"));
        assert!(wa.citation.contains("RCW 59.18.060"));
        assert!(de.citation.contains("IRC M1502"));
    }

    #[test]
    fn shared_coin_op_treated_as_landlord_provided() {
        let mut i = baseline();
        i.provision_status = ApplianceProvisionStatus::SharedCoinOpLaundryRoom;
        i.appliance_in_disrepair_with_overdue_tenant_notice = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::LandlordProvidedRepairOverdueHabitabilityBreach
        ));
    }
}
