//! Rental marijuana / cannabis cultivation restriction framework — covers
//! tenant home-grow rights under state recreational and medical cannabis
//! laws, landlord prohibition rights via lease, federal Controlled
//! Substances Act preemption considerations, Fair Housing Act medical-
//! cannabis reasonable-accommodation doctrine, and mold + moisture damage
//! liability from indoor cultivation operations.
//!
//! Distinct from sibling `tenant_cannabis_use_protection` (smoking and
//! consumption protection framework — distinct from cultivation),
//! `rental_smoke_free_housing_disclosure` (smoke-free designation
//! framework), `rental_basement_water_intrusion_disclosure` (mold-
//! infiltration framework — coordinates with grow-room moisture),
//! `rental_in_unit_laundry_appliance_provision` (iter 501 high-humidity
//! appliance framework — distinct ventilation analog).
//!
//! Trader-landlord critical because (1) **as of 2025, at least 16 states +
//! DC allow adult home cultivation for recreational use** (CA + CO + MA +
//! NY + MI + ME + VT + DC + AZ + MT + NV + OR + RI + VA + NM + MO + MN);
//! state plant counts range from 2 (MD) to 12 (MI); (2) **federal
//! Controlled Substances Act (21 U.S.C. § 812) classifies marijuana as
//! Schedule I**; DEA scheduling proposal to move marijuana to Schedule III
//! pending as of June 2025; CSA conflict creates limited federal
//! preemption argument for landlords; (3) **HUD HOME / Section 8 federally-
//! assisted housing**: HUD Notice PIH 2011-25 prohibits federally-assisted
//! tenants from any cannabis use including state-legal medical (potential
//! 24 C.F.R. § 5.852 lease termination); (4) **FHA disability accommodation
//! for medical cannabis**: 42 U.S.C. § 3604(f)(3)(B) DOES NOT require
//! landlord accommodation per HUD Notice — federal law preempts state
//! medical-cannabis-as-disability framing; (5) cultivation damages include
//! mold from 60-80% humidity grow rooms + electrical-fire risk from
//! high-wattage grow lights + property odor migration + insurance policy
//! exclusion for cannabis-related losses; (6) Colorado tenant home-grow
//! up to 6 plants (max 3 mature) per Colo. Const. Art. XVIII § 16(3)(b);
//! California Prop 64 up to 6 plants per Cal. Health & Safety Code §
//! 11362.1(a)(3); NY MRTA up to 3 mature + 3 immature per Cannabis Law §
//! 222 (post-18-month delay).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    /// California Prop 64 (Adult Use of Marijuana Act) — 6 plants per
    /// residence.
    California,
    /// Colorado — 6 plants (3 mature) per Colo. Const. Art. XVIII § 16(3).
    Colorado,
    /// New York MRTA — 3 mature + 3 immature per Cannabis Law § 222.
    NewYork,
    /// Massachusetts — 6 plants (12 if multiple adults).
    Massachusetts,
    /// Illinois — recreational possession permitted, NO home cultivation.
    Illinois,
    /// State with legal cannabis but no home cultivation (e.g., WA + NJ).
    NoHomeCultivationState,
    /// State where cannabis remains illegal — full landlord prohibition.
    CannabisIllegalState,
    /// Default — federal CSA Schedule I + common-law lease enforcement.
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HousingProgram {
    /// HUD-assisted (Section 8, public housing, project-based) — federal
    /// CSA enforcement attaches.
    HudAssistedFederalCsaEnforcement,
    /// Private market rental — state law primary.
    PrivateMarketStateLawPrimary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CultivationStatus {
    /// No cultivation observed or claimed.
    NoCultivation,
    /// Recreational cultivation within state limit.
    RecreationalWithinStateLimit,
    /// Cultivation exceeding state plant-count limit.
    ExceedingStatePlantCountLimit,
    /// Medical cannabis cultivation under qualifying-patient status.
    MedicalCannabisQualifyingPatient,
    /// Black-market commercial cultivation.
    BlackMarketCommercialCultivation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantStateLegalCultivationWithLeaseConsent,
    LandlordProhibitionEnforceable,
    HudFederalCsaTerminationGround,
    FhaMedicalCannabisAccommodationDenied,
    ExceedsStatePlantCountViolation,
    BlackMarketCommercialEvictionGround,
    MoldOrElectricalFireDamageHabitabilityBreach,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub housing_program: HousingProgram,
    pub cultivation_status: CultivationStatus,
    pub plant_count_actual: u32,
    pub state_plant_count_limit: u32,
    pub lease_explicitly_prohibits_cultivation: bool,
    pub tenant_qualifying_medical_patient: bool,
    pub mold_or_fire_damage_observed: bool,
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

pub const CA_PROP_64_PLANT_LIMIT: u32 = 6;
pub const CO_PLANT_LIMIT: u32 = 6;
pub const CO_MATURE_PLANT_LIMIT: u32 = 3;
pub const NY_MRTA_MATURE_PLUS_IMMATURE_LIMIT: u32 = 6;
pub const MA_PLANT_LIMIT_PER_RESIDENCE: u32 = 12;
pub const CSA_SCHEDULE_ROW: u32 = 1;
pub const STATES_PERMITTING_HOME_GROW_2025: u32 = 16;

pub fn check(input: &Input) -> Output {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let severity: Severity;

    if matches!(input.cultivation_status, CultivationStatus::NoCultivation) {
        notes.push(
            "No tenant cannabis cultivation observed or claimed — framework inapplicable. \
             Recommend maintaining explicit cultivation-prohibition language in lease addendum \
             as default to mitigate future operational risk + insurance coverage exclusion."
                .to_string(),
        );
        return Output {
            severity: Severity::NotApplicable,
            recommended_actions: actions,
            annual_rent_at_risk_cents: 0,
            citation: "n/a (no cultivation)",
            notes,
        };
    }

    if input.mold_or_fire_damage_observed {
        severity = Severity::MoldOrElectricalFireDamageHabitabilityBreach;
        actions.push(
            "Mold or electrical-fire damage from indoor cultivation operation observed — \
             grow rooms operate at 60-80% relative humidity creating substantial \
             condensation + structural moisture + mold growth + drywall + flooring damage \
             + insurance policy fire-cause exclusion for cannabis-related grow-light \
             overload events. Document with photographs + insurance claim adjuster \
             inspection + air-quality assessment; pursue eviction under common-law nuisance \
             plus state-law waste-of-premises theory; refer to [[rental_basement_water_\
             intrusion_disclosure]] sibling for mold-remediation framework cross-reference."
                .to_string(),
        );
    } else if matches!(
        input.housing_program,
        HousingProgram::HudAssistedFederalCsaEnforcement
    ) {
        severity = Severity::HudFederalCsaTerminationGround;
        actions.push(
            "HUD-assisted housing (Section 8 + public housing + project-based) tenant \
             cultivation creates federal Controlled Substances Act 21 U.S.C. § 812 \
             enforcement ground regardless of state legality. HUD Notice PIH 2011-25 \
             prohibits federally-assisted tenants from any cannabis use including state-\
             legal medical. Initiate lease termination under 24 C.F.R. § 5.852; offer \
             tenant 30-day grace period for voluntary cessation before HUD-required \
             termination proceedings; document with HUD field office consultation."
                .to_string(),
        );
    } else if matches!(
        input.cultivation_status,
        CultivationStatus::BlackMarketCommercialCultivation
    ) {
        severity = Severity::BlackMarketCommercialEvictionGround;
        actions.push(
            "Black-market commercial cultivation operation on premises — substantial \
             lease breach via state-statute-violation clause + nuisance per se + commercial \
             use of residential property. Document plant count + grow-light wattage + \
             ventilation modification + product packaging. Notify local law enforcement \
             plus state cannabis-control agency. Pursue immediate eviction under expedited \
             unlawful detainer procedure for criminal-activity-on-premises ground. \
             Federal RICO civil-forfeiture exposure may attach if landlord knowing-\
             permission established."
                .to_string(),
        );
    } else if matches!(
        input.cultivation_status,
        CultivationStatus::ExceedingStatePlantCountLimit
    ) || input.plant_count_actual > input.state_plant_count_limit
    {
        severity = Severity::ExceedsStatePlantCountViolation;
        actions.push(format!(
            "Tenant cultivation of {} plants EXCEEDS state plant-count limit of {} per \
             residence. Issue 3-day Notice to Cure under state landlord-tenant procedure; \
             require reduction to legal limit + photographic verification. State cannabis-\
             control agency may impose civil penalty; landlord exposure if knowing-\
             permission established under state aiding-and-abetting framework.",
            input.plant_count_actual, input.state_plant_count_limit
        ));
    } else if matches!(
        input.cultivation_status,
        CultivationStatus::MedicalCannabisQualifyingPatient
    ) && input.tenant_qualifying_medical_patient
        && matches!(input.housing_program, HousingProgram::HudAssistedFederalCsaEnforcement)
    {
        severity = Severity::FhaMedicalCannabisAccommodationDenied;
        actions.push(
            "Fair Housing Act medical-cannabis reasonable-accommodation request: HUD \
             interpretation per HUD General Counsel memorandum holds FHA 42 U.S.C. § \
             3604(f)(3)(B) DOES NOT require landlord accommodation for state-legal medical \
             cannabis use — federal CSA Schedule I preempts state medical-cannabis-as-\
             disability framing. Landlord may DENY medical-cannabis accommodation request \
             in HUD-assisted housing without FHA violation. Document denial in writing \
             citing HUD General Counsel guidance; preserve tenant alternative reasonable-\
             accommodation rights for other disability conditions."
                .to_string(),
        );
    } else if input.lease_explicitly_prohibits_cultivation {
        severity = Severity::LandlordProhibitionEnforceable;
        actions.push(
            "Lease explicitly prohibits cannabis cultivation — landlord prohibition is \
             enforceable even in state-legal recreational jurisdiction. Most states \
             expressly permit landlord lease prohibition on cultivation per state cannabis \
             statute. Issue 3-day Notice to Cure plus subsequent unlawful detainer if \
             uncured. Document cultivation evidence (plants + grow lights + ventilation \
             modifications + smell + electrical consumption pattern)."
                .to_string(),
        );
    } else {
        severity = Severity::CompliantStateLegalCultivationWithLeaseConsent;
        actions.push(format!(
            "Tenant cultivation of {} plants within state limit of {} and no explicit \
             lease prohibition — compliant. Recommend lease amendment at next renewal to \
             specify (1) plant-count limit, (2) ventilation requirements (CFM minimum + \
             humidity control), (3) electrical-load notice for grow-light installation, \
             (4) odor-control protocol, (5) renter's-insurance coverage exclusion \
             acknowledgment. Verify lease coordinates with state cannabis-control \
             regulations.",
            input.plant_count_actual, input.state_plant_count_limit
        ));
    }

    match input.jurisdiction {
        Jurisdiction::California => {
            notes.push(format!(
                "California Adult Use of Marijuana Act (Prop 64, 2016) plus Cal. Health & \
                 Safety Code § 11362.1(a)(3) permits adult ≥21 to cultivate up to {} \
                 plants per residence. Local jurisdictions may impose additional indoor \
                 cultivation restrictions; some municipalities prohibit outdoor cultivation \
                 entirely. Landlord lease prohibition explicitly permitted per Cal. Health \
                 & Safety Code § 11362.45(h).",
                CA_PROP_64_PLANT_LIMIT
            ));
        }
        Jurisdiction::Colorado => {
            notes.push(format!(
                "Colo. Const. Art. XVIII § 16(3)(b) permits adult ≥21 to cultivate up to {} \
                 plants per residence with no more than {} mature flowering. Tenant may \
                 cultivate unless lease prohibits in writing per § 16(3)(b)(II). Colorado \
                 Marijuana Enforcement Division regulates.",
                CO_PLANT_LIMIT, CO_MATURE_PLANT_LIMIT
            ));
        }
        Jurisdiction::NewYork => {
            notes.push(format!(
                "NY Marijuana Regulation and Taxation Act (MRTA) Cannabis Law § 222 permits \
                 adult ≥21 to cultivate up to {} mature plus {} immature plants per \
                 residence (post-18-month-delay activation). NY Office of Cannabis \
                 Management (OCM) regulates. NYC Local Law 18 of 2021 disqualifies \
                 cannabis-cultivation premises from short-term rental registration.",
                NY_MRTA_MATURE_PLUS_IMMATURE_LIMIT / 2,
                NY_MRTA_MATURE_PLUS_IMMATURE_LIMIT / 2
            ));
        }
        Jurisdiction::Massachusetts => {
            notes.push(format!(
                "M.G.L. ch. 94G § 7 permits adult ≥21 to cultivate up to 6 plants per \
                 person + up to {} plants per residence regardless of number of \
                 occupants. MA Cannabis Control Commission (CCC) regulates. Landlord \
                 prohibition permitted by lease addendum.",
                MA_PLANT_LIMIT_PER_RESIDENCE
            ));
        }
        Jurisdiction::Illinois => {
            notes.push(
                "Illinois Cannabis Regulation and Tax Act (410 ILCS 705) permits adult \
                 recreational possession but PROHIBITS home cultivation for non-medical \
                 users; medical-cannabis patients may cultivate up to 5 plants per 410 \
                 ILCS 130. Recreational tenant cultivation is illegal regardless of \
                 lease."
                    .to_string(),
            );
        }
        Jurisdiction::NoHomeCultivationState => {
            notes.push(
                "State has legal recreational cannabis but does NOT permit home \
                 cultivation (Washington recreational; New Jersey; certain others). \
                 Tenant cultivation illegal regardless of lease silence; landlord may \
                 invoke state-statute-violation lease clause."
                    .to_string(),
            );
        }
        Jurisdiction::CannabisIllegalState => {
            notes.push(
                "State maintains cannabis illegality — full state-statute-violation lease \
                 termination ground; coordinate with state law enforcement on \
                 enforcement priorities."
                    .to_string(),
            );
        }
        Jurisdiction::Default => {
            notes.push(format!(
                "Federal Controlled Substances Act (21 U.S.C. § 812) classifies marijuana \
                 as Schedule {} (DEA scheduling proposal to Schedule III pending as of \
                 June 2025). State cannabis legalization status varies — as of 2025, at \
                 least {} states plus DC permit adult home cultivation. HUD Notice PIH \
                 2011-25 prohibits federally-assisted tenants from any cannabis use. \
                 Common-law lease enforcement governs absent state-specific tenant \
                 cultivation right.",
                CSA_SCHEDULE_ROW, STATES_PERMITTING_HOME_GROW_2025
            ));
        }
    }

    notes.push(
        "Coordination with [[tenant_cannabis_use_protection]] (consumption / smoking \
         framework — distinct from cultivation), [[rental_smoke_free_housing_disclosure]] \
         (smoke-free designation framework — may incorporate cannabis prohibition), \
         [[rental_basement_water_intrusion_disclosure]] (mold-infiltration coordination — \
         grow rooms 60-80% humidity drive condensation damage), [[rental_in_unit_laundry_\
         appliance_provision]] (iter 501 — high-humidity-appliance ventilation analog), \
         [[rental_short_term_subletting_airbnb_restriction]] (iter 513 — NYC LL18 \
         disqualifies cultivation premises from STR registration), [[mid_tenancy_temporary_\
         relocation]] (when tenant must vacate during mold remediation), [[tenant_\
         emotional_distress_damages]] (IIED claim for wrongful eviction in legal-\
         cultivation state)."
            .to_string(),
    );

    let annual_rent_at_risk: u64 = match severity {
        Severity::MoldOrElectricalFireDamageHabitabilityBreach
        | Severity::BlackMarketCommercialEvictionGround
        | Severity::HudFederalCsaTerminationGround => input.annual_rent_cents,
        Severity::ExceedsStatePlantCountViolation
        | Severity::FhaMedicalCannabisAccommodationDenied
        | Severity::LandlordProhibitionEnforceable => {
            input.annual_rent_cents.saturating_div(2)
        }
        _ => 0,
    };

    Output {
        severity,
        recommended_actions: actions,
        annual_rent_at_risk_cents: annual_rent_at_risk,
        citation: match input.jurisdiction {
            Jurisdiction::California => "Cal. Health & Safety Code § 11362.1(a)(3) + § 11362.45(h) + Prop 64",
            Jurisdiction::Colorado => "Colo. Const. Art. XVIII § 16(3)(b)(I)-(II)",
            Jurisdiction::NewYork => "NY Cannabis Law § 222 + MRTA + NY OCM",
            Jurisdiction::Massachusetts => "M.G.L. ch. 94G § 7 + MA CCC",
            Jurisdiction::Illinois => "410 ILCS 705 + 410 ILCS 130",
            Jurisdiction::NoHomeCultivationState => "State recreational + no-cultivation framework",
            Jurisdiction::CannabisIllegalState => "State cannabis prohibition + common-law lease",
            Jurisdiction::Default => "21 U.S.C. § 812 CSA Schedule I + HUD PIH 2011-25",
        },
        notes,
    }
}

pub type RentalMarijuanaCultivationRestrictionInput = Input;
pub type RentalMarijuanaCultivationRestrictionResult = Output;

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            housing_program: HousingProgram::PrivateMarketStateLawPrimary,
            cultivation_status: CultivationStatus::RecreationalWithinStateLimit,
            plant_count_actual: 4,
            state_plant_count_limit: 6,
            lease_explicitly_prohibits_cultivation: false,
            tenant_qualifying_medical_patient: false,
            mold_or_fire_damage_observed: false,
            annual_rent_cents: 36_000_00,
        }
    }

    #[test]
    fn no_cultivation_not_applicable() {
        let mut i = baseline();
        i.cultivation_status = CultivationStatus::NoCultivation;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn compliant_state_legal_with_lease_consent() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CompliantStateLegalCultivationWithLeaseConsent
        ));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn lease_prohibition_enforceable() {
        let mut i = baseline();
        i.lease_explicitly_prohibits_cultivation = true;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::LandlordProhibitionEnforceable));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
        assert!(r.recommended_actions.iter().any(|a| a.contains("3-day Notice to Cure")));
    }

    #[test]
    fn hud_assisted_federal_csa_termination() {
        let mut i = baseline();
        i.housing_program = HousingProgram::HudAssistedFederalCsaEnforcement;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::HudFederalCsaTerminationGround));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("HUD Notice PIH 2011-25")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("24 C.F.R. § 5.852")));
    }

    #[test]
    fn exceeds_state_plant_count_violation() {
        let mut i = baseline();
        i.cultivation_status = CultivationStatus::ExceedingStatePlantCountLimit;
        i.plant_count_actual = 12;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::ExceedsStatePlantCountViolation));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
    }

    #[test]
    fn plant_count_actual_over_limit_triggers_violation() {
        let mut i = baseline();
        i.plant_count_actual = 10;
        i.state_plant_count_limit = 6;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::ExceedsStatePlantCountViolation));
    }

    #[test]
    fn black_market_commercial_eviction_ground() {
        let mut i = baseline();
        i.cultivation_status = CultivationStatus::BlackMarketCommercialCultivation;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::BlackMarketCommercialEvictionGround));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("RICO")));
    }

    #[test]
    fn mold_or_fire_damage_habitability_breach() {
        let mut i = baseline();
        i.mold_or_fire_damage_observed = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::MoldOrElectricalFireDamageHabitabilityBreach
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("60-80% relative humidity")));
    }

    #[test]
    fn fha_medical_cannabis_accommodation_denied_in_hud() {
        let mut i = baseline();
        i.housing_program = HousingProgram::HudAssistedFederalCsaEnforcement;
        i.cultivation_status = CultivationStatus::MedicalCannabisQualifyingPatient;
        i.tenant_qualifying_medical_patient = true;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::HudFederalCsaTerminationGround));
    }

    #[test]
    fn ca_jurisdiction_pins_prop_64_and_11362_45_h() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Prop 64")));
        assert!(r.notes.iter().any(|n| n.contains("§ 11362.1(a)(3)")));
        assert!(r.notes.iter().any(|n| n.contains("§ 11362.45(h)")));
    }

    #[test]
    fn co_jurisdiction_pins_constitutional_art_xviii_16() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Colorado;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Colo. Const. Art. XVIII § 16(3)(b)")));
        assert!(r.notes.iter().any(|n| n.contains("Marijuana Enforcement Division")));
    }

    #[test]
    fn ny_jurisdiction_pins_mrta_and_ocm() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("MRTA")));
        assert!(r.notes.iter().any(|n| n.contains("Cannabis Law § 222")));
        assert!(r.notes.iter().any(|n| n.contains("OCM")));
        assert!(r.notes.iter().any(|n| n.contains("LL18")));
    }

    #[test]
    fn ma_jurisdiction_pins_ch_94g_7_and_ccc() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Massachusetts;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("M.G.L. ch. 94G § 7")));
        assert!(r.notes.iter().any(|n| n.contains("Cannabis Control Commission")));
    }

    #[test]
    fn il_jurisdiction_pins_410_ilcs_705_and_130() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Illinois;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("410 ILCS 705")));
        assert!(r.notes.iter().any(|n| n.contains("410 ILCS 130")));
        assert!(r.notes.iter().any(|n| n.contains("PROHIBITS home cultivation")));
    }

    #[test]
    fn default_jurisdiction_pins_csa_schedule_i_and_hud_pih() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Schedule 1")));
        assert!(r.notes.iter().any(|n| n.contains("21 U.S.C. § 812")));
        assert!(r.notes.iter().any(|n| n.contains("HUD Notice PIH 2011-25")));
        assert!(r.notes.iter().any(|n| n.contains("16")));
    }

    #[test]
    fn coordination_note_references_siblings() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("tenant_cannabis_use_protection")));
        assert!(r.notes.iter().any(|n| n.contains("rental_smoke_free_housing_disclosure")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_basement_water_intrusion_disclosure")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_short_term_subletting_airbnb_restriction")));
    }

    #[test]
    fn ca_prop_64_plant_limit_pins_6() {
        assert_eq!(CA_PROP_64_PLANT_LIMIT, 6);
    }

    #[test]
    fn co_plant_limit_pins_6_total_3_mature() {
        assert_eq!(CO_PLANT_LIMIT, 6);
        assert_eq!(CO_MATURE_PLANT_LIMIT, 3);
    }

    #[test]
    fn ny_mrta_total_limit_pins_6() {
        assert_eq!(NY_MRTA_MATURE_PLUS_IMMATURE_LIMIT, 6);
    }

    #[test]
    fn ma_plant_limit_per_residence_pins_12() {
        assert_eq!(MA_PLANT_LIMIT_PER_RESIDENCE, 12);
    }

    #[test]
    fn csa_schedule_pins_1() {
        assert_eq!(CSA_SCHEDULE_ROW, 1);
    }

    #[test]
    fn states_permitting_home_grow_2025_pins_16() {
        assert_eq!(STATES_PERMITTING_HOME_GROW_2025, 16);
    }

    #[test]
    fn citation_branch_for_each_jurisdiction() {
        let ca = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::California; i });
        let co = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Colorado; i });
        let ny = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::NewYork; i });
        let ma = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Massachusetts; i });
        let il = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Illinois; i });
        let de = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Default; i });
        assert!(ca.citation.contains("Prop 64"));
        assert!(co.citation.contains("Colo. Const. Art. XVIII"));
        assert!(ny.citation.contains("MRTA"));
        assert!(ma.citation.contains("M.G.L. ch. 94G"));
        assert!(il.citation.contains("410 ILCS"));
        assert!(de.citation.contains("HUD PIH 2011-25"));
    }

    #[test]
    fn severity_priority_mold_overrides_lease_prohibition() {
        let mut i = baseline();
        i.mold_or_fire_damage_observed = true;
        i.lease_explicitly_prohibits_cultivation = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::MoldOrElectricalFireDamageHabitabilityBreach
        ));
    }

    #[test]
    fn severity_priority_hud_overrides_lease_silence() {
        let mut i = baseline();
        i.housing_program = HousingProgram::HudAssistedFederalCsaEnforcement;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::HudFederalCsaTerminationGround));
    }

    #[test]
    fn zero_rent_zero_risk_does_not_panic() {
        let mut i = baseline();
        i.annual_rent_cents = 0;
        i.lease_explicitly_prohibits_cultivation = true;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }
}
