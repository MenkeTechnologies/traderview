//! Tenant smart-thermostat installation right and landlord-permission
//! framework. Parallel to `tenant_solar_installation` (right-to-install
//! rooftop solar), `tenant_ev_charging_installation_right` (right-to-
//! charge EV), `tenant_clothesline_drying_right` (utility-conservation
//! right), `tenant_window_air_conditioner_install_right` (cooling
//! provision and bracket safety).
//!
//! Smart thermostats (Nest, Ecobee, Honeywell Lyric, Wyse, Sensi) replace
//! mechanical or programmable thermostats and learn occupancy patterns to
//! reduce HVAC runtime by 10-23% (Nest claim, 2014 EPA validation study).
//! Modern residential leases generally REQUIRE landlord permission for any
//! alteration to building systems including thermostats; smart-thermostat
//! installation involves: (1) wiring change at HVAC junction (24V control
//! circuit C-wire required for power); (2) drilling into wall; (3) removing
//! existing thermostat; (4) potential warranty issues with HVAC equipment.
//!
//! State-level statutory tenant right-to-install smart thermostat is largely
//! absent (2025-2026). Framework rests on five regimes:
//!
//! 1. **California** — Cal. Civ. Code § 1947.6 (energy-efficiency
//!    improvement allowance — implied limited right for tenant-paid
//!    energy-saving upgrades) plus Title 24 Building Energy Efficiency
//!    Standards (effective January 1, 2023, requires thermostat replacement
//!    on HVAC alteration). Cal. Civ. Code § 1942.1 self-help repair right
//!    where landlord fails to maintain working thermostat after 30-day
//!    notice. SB 1136 (introduced February 14, 2024 — energy-efficiency
//!    rental disclosure expansion) PENDING.
//!
//! 2. **New York** — NYC Local Law 97 (Climate Mobilization Act, signed
//!    April 18, 2019, effective for building emissions starting calendar
//!    year 2024) caps building carbon emissions for properties greater than
//!    25,000 sq ft; landlords incentivized to permit tenant-driven smart
//!    thermostat adoption to reduce penalties. Real Property Law § 235-b
//!    implied warranty of habitability requires reasonable heat (68°F
//!    daytime / 62°F nighttime per NYC Admin Code § 27-2029 October-May).
//!
//! 3. **Massachusetts** — DOER Mass Save program offers $100 instant rebate
//!    per smart thermostat per residence per program year (Eversource,
//!    National Grid, Unitil). MA Stretch Code (520 CMR 13.00) adopted by 299
//!    municipalities as of June 2025 requires programmable thermostat in new
//!    construction. M.G.L. ch. 186 § 14 quiet-enjoyment doctrine — landlord
//!    refusal of reasonable energy-efficiency upgrade may breach.
//!
//! 4. **Federal Fair Housing Act / ADA** — 42 U.S.C. § 3604(f)(3)(B) plus 24
//!    C.F.R. § 100.203 require landlord to permit reasonable modifications
//!    necessary for disabled tenant to fully enjoy premises, including voice-
//!    controlled smart thermostat for mobility-impaired tenants. Tenant
//!    typically must pay for modification plus restoration at lease end
//!    unless modification is reasonable to leave in place.
//!
//! 5. **Default** — common-law habitability doctrine plus lease-specific
//!    alteration clause governs; landlord refusal must not be unreasonable
//!    if tenant pays installation cost plus restoration cost; refusal grounds
//!    typically limited to (a) HVAC warranty void, (b) wiring incompatibility
//!    (24V vs 120V baseboard heat), (c) historic-district restriction.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    NewYork,
    Massachusetts,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HvacWiringType {
    /// 24V low-voltage control with C-wire — compatible with all smart
    /// thermostats.
    LowVoltage24vWithCWire,
    /// 24V low-voltage WITHOUT C-wire — most modern smart thermostats
    /// incompatible without C-wire adapter.
    LowVoltage24vNoCWire,
    /// 120V baseboard heat or radiant — NOT compatible with standard
    /// smart thermostats; needs line-voltage smart thermostat (Mysa,
    /// Sinopé).
    HighVoltage120vBaseboard,
    /// Steam / hot-water radiator with knob valve — NO electronic
    /// thermostat compatibility.
    SteamOrHotWaterRadiator,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallationRequest {
    /// Tenant requests installation paying own cost.
    TenantPaysInstall,
    /// Tenant requests landlord install with cost amortized in rent.
    LandlordInstallsCostAmortized,
    /// Tenant requests as ADA / FHA reasonable accommodation.
    AdaFhaReasonableAccommodation,
    /// Tenant requests as energy-efficiency upgrade under state utility
    /// rebate program.
    UtilityRebateProgramUpgrade,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    InstallationPermittedRoutine,
    WiringIncompatibilityRefusalPermitted,
    HvacWarrantyVoidRefusalPermitted,
    HistoricDistrictRestrictionApplies,
    AdaFhaReasonableAccommodationRequired,
    EnergyEfficiencyRebateEligibleApprovalLikely,
    LandlordRefusalUnreasonableHabitabilityBreach,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub hvac_wiring: HvacWiringType,
    pub installation_request: InstallationRequest,
    pub lease_explicitly_prohibits_thermostat_change: bool,
    pub landlord_consent_obtained_in_writing: bool,
    pub tenant_disability_certified: bool,
    pub historic_district_landmark_property: bool,
    pub hvac_warranty_active_and_change_voids: bool,
    pub tenant_agrees_restore_on_lease_end: bool,
    pub state_utility_rebate_available_dollars_cents: u64,
    pub annual_energy_savings_estimate_cents: u64,
    pub annual_rent_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub recommended_actions: Vec<String>,
    pub utility_rebate_dollars_cents: u64,
    pub estimated_annual_energy_savings_cents: u64,
    pub annual_rent_at_risk_cents: u64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const NEST_ECOBEE_ESTIMATED_ANNUAL_SAVINGS_PCT_BPS: u32 = 1_200;
pub const MA_MASS_SAVE_STANDARD_REBATE_CENTS: u64 = 10_000;
pub const NYC_LOCAL_LAW_97_EFFECTIVE_YEAR: i32 = 2024;
pub const CA_TITLE_24_2022_EFFECTIVE_YEAR: i32 = 2023;

pub fn check(input: &Input) -> Output {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let severity: Severity;
    let mut rebate_cents: u64 = 0;
    let estimated_savings: u64 = input.annual_energy_savings_estimate_cents;

    if matches!(
        input.installation_request,
        InstallationRequest::AdaFhaReasonableAccommodation
    ) && input.tenant_disability_certified
    {
        severity = Severity::AdaFhaReasonableAccommodationRequired;
        actions.push(
            "Tenant disability certification triggers 42 U.S.C. § 3604(f)(3)(B) and 24 C.F.R. \
             § 100.203 reasonable-modification requirement; landlord MUST permit smart \
             thermostat installation absent undue financial or administrative burden. Tenant \
             pays installation cost (FHA) but landlord cannot refuse outright. Document \
             written request under 24 C.F.R. § 100.204."
                .to_string(),
        );
        notes.push(
            "Coordination with [[fair_housing_reasonable_modification]] (general FHA \
             modification framework) plus [[tenant_emotional_distress_damages]] (IIED claim \
             for wrongful refusal of disability accommodation)."
                .to_string(),
        );
    } else if matches!(input.hvac_wiring, HvacWiringType::SteamOrHotWaterRadiator) {
        severity = Severity::WiringIncompatibilityRefusalPermitted;
        actions.push(
            "Steam or hot-water radiator with mechanical knob valve has NO electronic \
             thermostat input; smart thermostat installation technically infeasible. Landlord \
             refusal categorically permitted. Tenant alternative: install smart radiator \
             valve (TRV) such as Honeywell Home HR92 or Drayton Wiser per-radiator — does \
             not require thermostat replacement and falls below alteration threshold in most \
             leases."
                .to_string(),
        );
    } else if matches!(input.hvac_wiring, HvacWiringType::HighVoltage120vBaseboard) {
        severity = Severity::WiringIncompatibilityRefusalPermitted;
        actions.push(
            "120V line-voltage baseboard or radiant heat requires line-voltage smart \
             thermostat (Mysa, Sinopé Technologies, Stelpro Maestro); standard 24V smart \
             thermostats (Nest, Ecobee, Honeywell T9) INCOMPATIBLE and installation creates \
             fire hazard plus voids manufacturer warranty. Landlord refusal of incompatible \
             24V product categorically permitted; refusal of compatible line-voltage product \
             must rest on separate ground."
                .to_string(),
        );
    } else if input.historic_district_landmark_property {
        severity = Severity::HistoricDistrictRestrictionApplies;
        actions.push(
            "Historic district or landmark designation may restrict thermostat replacement \
             where original thermostat is contributing feature; consult local landmarks \
             preservation commission before installation. NYC Landmarks Preservation \
             Commission permit required under NYC Admin Code § 25-303 if exterior visible. \
             National Register property may require Section 106 NHPA review for federally \
             funded properties only."
                .to_string(),
        );
    } else if input.hvac_warranty_active_and_change_voids {
        severity = Severity::HvacWarrantyVoidRefusalPermitted;
        actions.push(
            "Active HVAC equipment warranty conditioned on OEM thermostat compatibility; \
             aftermarket smart thermostat installation voids warranty. Landlord refusal \
             permitted unless tenant indemnifies for warranty-loss exposure equal to \
             replacement cost of HVAC equipment remaining warranty period."
                .to_string(),
        );
    } else if matches!(
        input.installation_request,
        InstallationRequest::UtilityRebateProgramUpgrade
    ) {
        severity = Severity::EnergyEfficiencyRebateEligibleApprovalLikely;
        rebate_cents = if input.state_utility_rebate_available_dollars_cents == 0
            && matches!(input.jurisdiction, Jurisdiction::Massachusetts)
        {
            MA_MASS_SAVE_STANDARD_REBATE_CENTS
        } else {
            input.state_utility_rebate_available_dollars_cents
        };
        actions.push(format!(
            "Utility-rebate-program installation typically pre-approved by program \
             administrator; rebate of {} cents directly reduces tenant cost; landlord refusal \
             grounded only on legitimate technical or contractual basis. Estimated annual \
             energy savings: {} cents at 12% Nest/Ecobee EPA-validated reduction.",
            rebate_cents, estimated_savings
        ));
    } else if input.lease_explicitly_prohibits_thermostat_change
        && !input.landlord_consent_obtained_in_writing
    {
        severity = Severity::LandlordRefusalUnreasonableHabitabilityBreach;
        actions.push(
            "Lease alteration clause prohibits thermostat change without landlord consent; \
             tenant request without written consent is breach. Tenant remedy: obtain written \
             consent OR demonstrate refusal is unreasonable given (1) tenant pays full cost, \
             (2) tenant agrees restoration on lease end, (3) no HVAC warranty void, (4) no \
             wiring incompatibility, (5) no historic restriction — at which point refusal \
             may breach M.G.L. ch. 186 § 14 quiet enjoyment (MA) or NY Real Property Law \
             § 235-b implied warranty."
                .to_string(),
        );
    } else if input.landlord_consent_obtained_in_writing && input.tenant_agrees_restore_on_lease_end
    {
        severity = Severity::InstallationPermittedRoutine;
        actions.push(
            "Written landlord consent obtained and tenant agrees restoration on lease end; \
             installation is routine. Document (1) make/model/serial of removed thermostat, \
             (2) photo evidence of original wiring and faceplate, (3) tenant retention of \
             removed thermostat for restoration, (4) lease addendum noting reversal \
             obligation."
                .to_string(),
        );
    } else {
        severity = Severity::InstallationPermittedRoutine;
        actions.push(
            "Standard installation request: tenant pays cost plus agrees restoration; \
             landlord consent should be granted unless one of the categorical refusal grounds \
             applies (wiring incompatibility, HVAC warranty void, historic restriction)."
                .to_string(),
        );
    }

    match input.jurisdiction {
        Jurisdiction::California => {
            notes.push(format!(
                "Cal. Civ. Code § 1947.6 energy-efficiency improvement allowance creates \
                 limited implied right for tenant-paid energy-saving upgrades; Title 24 \
                 Building Energy Efficiency Standards (effective January 1, {}) requires \
                 thermostat replacement at HVAC alteration; Cal. Civ. Code § 1942.1 \
                 self-help repair right after 30-day notice for landlord-broken thermostat. \
                 SB 1136 energy-efficiency rental disclosure expansion pending.",
                CA_TITLE_24_2022_EFFECTIVE_YEAR
            ));
        }
        Jurisdiction::NewYork => {
            notes.push(format!(
                "NYC Local Law 97 (Climate Mobilization Act, signed April 18, 2019) caps \
                 building emissions starting calendar year {} for buildings > 25,000 sq ft; \
                 landlords incentivized to permit tenant-driven smart-thermostat adoption to \
                 reduce penalties under NYC Admin Code § 28-320. Real Property Law § 235-b \
                 implied warranty of habitability requires reasonable heat (68°F day / 62°F \
                 night October-May per NYC Admin Code § 27-2029).",
                NYC_LOCAL_LAW_97_EFFECTIVE_YEAR
            ));
        }
        Jurisdiction::Massachusetts => {
            notes.push(format!(
                "MA DOER Mass Save program (Eversource, National Grid, Unitil) offers instant \
                 rebate of {} cents per smart thermostat per residence per program year; MA \
                 Stretch Code 520 CMR 13.00 adopted by 299 municipalities as of June 2025 \
                 requires programmable thermostat in new construction; M.G.L. ch. 186 § 14 \
                 quiet-enjoyment doctrine — unreasonable refusal of energy-efficiency upgrade \
                 may breach.",
                MA_MASS_SAVE_STANDARD_REBATE_CENTS
            ));
        }
        Jurisdiction::Default => {
            notes.push(
                "Federal Fair Housing Act 42 U.S.C. § 3604(f)(3)(B) plus 24 C.F.R. § 100.203 \
                 reasonable-modification requirement applies in all jurisdictions; tenant \
                 disability certification triggers landlord duty to permit unless undue \
                 burden. Common-law habitability doctrine plus lease alteration clause \
                 governs; landlord refusal grounds typically limited to wiring \
                 incompatibility, HVAC warranty void, historic-district restriction."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Coordination with [[tenant_solar_installation]] (parallel right-to-install \
         framework), [[tenant_ev_charging_installation_right]] (parallel right-to-charge), \
         [[tenant_clothesline_drying_right]] (utility-conservation right), [[rental_solar_\
         panel_disclosure]] (energy-infrastructure disclosure pattern), [[rental_gas_appliance_\
         ban]] (electrification mandate context), [[tenant_window_air_conditioner_install_\
         right]] (HVAC alteration analog)."
            .to_string(),
    );

    let annual_rent_at_risk: u64 = match severity {
        Severity::LandlordRefusalUnreasonableHabitabilityBreach => {
            input.annual_rent_cents.saturating_div(2)
        }
        Severity::AdaFhaReasonableAccommodationRequired => input.annual_rent_cents,
        _ => 0,
    };

    Output {
        severity,
        recommended_actions: actions,
        utility_rebate_dollars_cents: rebate_cents,
        estimated_annual_energy_savings_cents: estimated_savings,
        annual_rent_at_risk_cents: annual_rent_at_risk,
        citation: match input.jurisdiction {
            Jurisdiction::California => {
                "Cal. Civ. Code § 1947.6 + § 1942.1 + Title 24 + SB 1136 (pending)"
            }
            Jurisdiction::NewYork => {
                "NYC Local Law 97 + Real Property Law § 235-b + NYC Admin § 27-2029 + § 28-320"
            }
            Jurisdiction::Massachusetts => {
                "M.G.L. ch. 186 § 14 + 520 CMR 13.00 Stretch Code + Mass Save program"
            }
            Jurisdiction::Default => {
                "42 U.S.C. § 3604(f)(3)(B) + 24 C.F.R. § 100.203 FHA + common-law habitability"
            }
        },
        notes,
    }
}

pub type TenantSmartThermostatInstallRightInput = Input;
pub type TenantSmartThermostatInstallRightResult = Output;

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            hvac_wiring: HvacWiringType::LowVoltage24vWithCWire,
            installation_request: InstallationRequest::TenantPaysInstall,
            lease_explicitly_prohibits_thermostat_change: false,
            landlord_consent_obtained_in_writing: true,
            tenant_disability_certified: false,
            historic_district_landmark_property: false,
            hvac_warranty_active_and_change_voids: false,
            tenant_agrees_restore_on_lease_end: true,
            state_utility_rebate_available_dollars_cents: 0,
            annual_energy_savings_estimate_cents: 240_00,
            annual_rent_cents: 36_000_00,
        }
    }

    #[test]
    fn standard_request_with_consent_and_restoration_routine() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(r.severity, Severity::InstallationPermittedRoutine));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn ada_fha_reasonable_accommodation_required_with_disability() {
        let mut i = baseline();
        i.installation_request = InstallationRequest::AdaFhaReasonableAccommodation;
        i.tenant_disability_certified = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::AdaFhaReasonableAccommodationRequired
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("42 U.S.C. § 3604(f)(3)(B)")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("24 C.F.R. § 100.203")));
    }

    #[test]
    fn ada_request_without_certification_falls_to_default_path() {
        let mut i = baseline();
        i.installation_request = InstallationRequest::AdaFhaReasonableAccommodation;
        i.tenant_disability_certified = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::InstallationPermittedRoutine));
    }

    #[test]
    fn steam_radiator_incompatibility_refusal_permitted() {
        let mut i = baseline();
        i.hvac_wiring = HvacWiringType::SteamOrHotWaterRadiator;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::WiringIncompatibilityRefusalPermitted
        ));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("smart radiator valve")));
    }

    #[test]
    fn baseboard_120v_incompatibility_refusal_permitted() {
        let mut i = baseline();
        i.hvac_wiring = HvacWiringType::HighVoltage120vBaseboard;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::WiringIncompatibilityRefusalPermitted
        ));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Mysa")));
    }

    #[test]
    fn historic_district_restriction_applies() {
        let mut i = baseline();
        i.historic_district_landmark_property = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::HistoricDistrictRestrictionApplies
        ));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("NYC Admin Code § 25-303")));
    }

    #[test]
    fn hvac_warranty_void_refusal_permitted() {
        let mut i = baseline();
        i.hvac_warranty_active_and_change_voids = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::HvacWarrantyVoidRefusalPermitted
        ));
        assert!(r.recommended_actions.iter().any(|a| a.contains("indemnif")));
    }

    #[test]
    fn ma_utility_rebate_program_default_to_mass_save_100() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.installation_request = InstallationRequest::UtilityRebateProgramUpgrade;
        i.state_utility_rebate_available_dollars_cents = 0;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::EnergyEfficiencyRebateEligibleApprovalLikely
        ));
        assert_eq!(
            r.utility_rebate_dollars_cents,
            MA_MASS_SAVE_STANDARD_REBATE_CENTS
        );
    }

    #[test]
    fn ca_utility_rebate_with_explicit_amount_uses_input_value() {
        let mut i = baseline();
        i.installation_request = InstallationRequest::UtilityRebateProgramUpgrade;
        i.state_utility_rebate_available_dollars_cents = 75_00;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::EnergyEfficiencyRebateEligibleApprovalLikely
        ));
        assert_eq!(r.utility_rebate_dollars_cents, 75_00);
    }

    #[test]
    fn lease_prohibits_no_consent_unreasonable_breach() {
        let mut i = baseline();
        i.lease_explicitly_prohibits_thermostat_change = true;
        i.landlord_consent_obtained_in_writing = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::LandlordRefusalUnreasonableHabitabilityBreach
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
    }

    #[test]
    fn ca_jurisdiction_pins_section_1947_6_and_1942_1() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 1947.6")));
        assert!(r.notes.iter().any(|n| n.contains("§ 1942.1")));
        assert!(r.notes.iter().any(|n| n.contains("Title 24")));
        assert!(r.citation.contains("SB 1136"));
    }

    #[test]
    fn ny_jurisdiction_pins_local_law_97_and_real_property_235_b() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Local Law 97")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Real Property Law § 235-b")));
        assert!(r.notes.iter().any(|n| n.contains("April 18, 2019")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("NYC Admin Code § 27-2029")));
    }

    #[test]
    fn ma_jurisdiction_pins_mass_save_and_stretch_code() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Massachusetts;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Mass Save")));
        assert!(r.notes.iter().any(|n| n.contains("520 CMR 13.00")));
        assert!(r.notes.iter().any(|n| n.contains("299 municipalities")));
        assert!(r.notes.iter().any(|n| n.contains("M.G.L. ch. 186 § 14")));
    }

    #[test]
    fn default_jurisdiction_pins_42_usc_3604_fha() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("42 U.S.C. § 3604(f)(3)(B)")));
        assert!(r.notes.iter().any(|n| n.contains("24 C.F.R. § 100.203")));
    }

    #[test]
    fn coordination_note_references_solar_ev_clothesline_window_ac() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("tenant_solar_installation")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("tenant_ev_charging_installation_right")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("tenant_clothesline_drying_right")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_solar_panel_disclosure")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("tenant_window_air_conditioner_install_right")));
    }

    #[test]
    fn coordination_note_pinned_in_all_jurisdictions() {
        for j in [
            Jurisdiction::California,
            Jurisdiction::NewYork,
            Jurisdiction::Massachusetts,
            Jurisdiction::Default,
        ] {
            let mut i = baseline();
            i.jurisdiction = j;
            let r = check(&i);
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("tenant_solar_installation")),
                "coordination note missing for {j:?}"
            );
        }
    }

    #[test]
    fn nest_ecobee_savings_pct_constant_pins_12() {
        assert_eq!(NEST_ECOBEE_ESTIMATED_ANNUAL_SAVINGS_PCT_BPS, 1_200);
    }

    #[test]
    fn ma_mass_save_rebate_pins_100_dollars() {
        assert_eq!(MA_MASS_SAVE_STANDARD_REBATE_CENTS, 10_000);
    }

    #[test]
    fn nyc_local_law_97_effective_year_pins_2024() {
        assert_eq!(NYC_LOCAL_LAW_97_EFFECTIVE_YEAR, 2024);
    }

    #[test]
    fn ca_title_24_effective_year_pins_2023() {
        assert_eq!(CA_TITLE_24_2022_EFFECTIVE_YEAR, 2023);
    }

    #[test]
    fn estimated_savings_passed_through() {
        let mut i = baseline();
        i.annual_energy_savings_estimate_cents = 350_00;
        let r = check(&i);
        assert_eq!(r.estimated_annual_energy_savings_cents, 350_00);
    }

    #[test]
    fn full_annual_rent_at_risk_for_ada_violation() {
        let mut i = baseline();
        i.installation_request = InstallationRequest::AdaFhaReasonableAccommodation;
        i.tenant_disability_certified = true;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
    }

    #[test]
    fn half_annual_rent_at_risk_for_unreasonable_refusal() {
        let mut i = baseline();
        i.lease_explicitly_prohibits_thermostat_change = true;
        i.landlord_consent_obtained_in_writing = false;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
    }

    #[test]
    fn zero_rent_zero_risk_does_not_panic() {
        let mut i = baseline();
        i.annual_rent_cents = 0;
        i.lease_explicitly_prohibits_thermostat_change = true;
        i.landlord_consent_obtained_in_writing = false;
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
        let ny = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::NewYork;
            i
        });
        let ma = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::Massachusetts;
            i
        });
        let de = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::Default;
            i
        });
        assert!(ca.citation.contains("Cal. Civ. Code"));
        assert!(ny.citation.contains("Local Law 97"));
        assert!(ma.citation.contains("Stretch Code"));
        assert!(de.citation.contains("FHA"));
    }

    #[test]
    fn refusal_grounds_priority_wiring_overrides_warranty() {
        let mut i = baseline();
        i.hvac_wiring = HvacWiringType::SteamOrHotWaterRadiator;
        i.hvac_warranty_active_and_change_voids = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::WiringIncompatibilityRefusalPermitted
        ));
    }

    #[test]
    fn refusal_grounds_priority_historic_overrides_warranty() {
        let mut i = baseline();
        i.historic_district_landmark_property = true;
        i.hvac_warranty_active_and_change_voids = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::HistoricDistrictRestrictionApplies
        ));
    }
}
