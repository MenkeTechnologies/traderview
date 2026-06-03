//! Rental attached-garage carbon monoxide (CO) detector disclosure
//! framework — covers landlord obligations under state CO detector laws
//! when rental unit shares wall, floor, or ceiling with attached garage
//! (vehicle-exhaust CO migration pathway), regardless of presence of gas
//! appliances within the unit.
//!
//! Distinct from sibling `rental_natural_gas_leak_response` (gas-line
//! leak protocol — distinct from CO detection), [[rental_pellet_stove_
//! disclosure]] (iter 499 — solid-fuel appliance with separate CO concern),
//! `rental_chimney_fireplace_inspection_disclosure` (chimney-vented
//! appliance CO), `rental_oil_tank_replacement_disclosure` (iter 493 —
//! oil-heat fuel-storage framework), [[rental_in_unit_laundry_appliance_
//! provision]] (iter 501 — gas dryer CO concern subset), [[rental_radiator_
//! steam_heat_safety]] (iter 517 — distinct steam-heat framework).
//!
//! Trader-landlord critical because (1) **CDC reports approximately 450 US
//! deaths annually from non-fire carbon monoxide poisoning plus 20,000
//! nonfatal injuries** — CO is the LEADING cause of poison-related death
//! in the US; (2) **California SB 183 (Carbon Monoxide Poisoning
//! Prevention Act of 2010)** mandates CO alarms in all rental units with
//! gas appliances, fossil-fuel-burning appliances, fireplaces, OR
//! ATTACHED GARAGES — attached garage triggers mandate regardless of
//! in-unit gas appliance presence; (3) **UL 2034 Standard for Single and
//! Multiple Station Carbon Monoxide Alarms** — required listing for
//! state-compliant CO detector; alarm must report before CO reaches
//! levels causing loss of ability to react; (4) **Washington RCW 19.27.530**
//! required CO alarms in all newly constructed residential buildings as
//! of January 1, 2011 + all other residential occupancies by January 1,
//! 2013; (5) **NY Amanda's Law (Public Health Law § 1399-bbb-1)** named
//! after Amanda Hansen who died age 16 from CO poisoning at sleepover —
//! requires CO detector adjacent to bedroom in all residential structures
//! with attached garage or fossil-fuel-burning appliance, effective Feb
//! 22, 2010; (6) attached-garage CO migration to unit can reach lethal
//! 400 ppm within 5 minutes of vehicle idling per CDC field studies;
//! (7) HPD penalties + tenant-injury litigation routinely exceeds $1M-
//! $5M for permanent neurological injury or wrongful death.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    /// California SB 183 + Health and Safety Code § 17926-17926.2.
    California,
    /// New York Amanda's Law (PHL § 1399-bbb-1).
    NewYork,
    /// Washington RCW 19.27.530.
    Washington,
    /// Connecticut C.G.S. § 29-292(b).
    Connecticut,
    /// Massachusetts Nicole's Law (M.G.L. ch. 148 § 26F).
    Massachusetts,
    /// Default — common-law habitability + state-specific.
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoExposureRisk {
    /// Attached garage shares wall, floor, or ceiling with rental unit —
    /// vehicle-exhaust CO migration pathway.
    AttachedGarageSharedBoundary,
    /// Detached garage with breezeway / shared HVAC return path.
    DetachedGarageWithSharedHvacReturn,
    /// In-unit gas / oil / wood fossil-fuel-burning appliance only.
    InUnitFossilFuelAppliance,
    /// No attached garage and no fossil-fuel appliances — minimal CO risk.
    MinimalNoFossilFuelOrGarage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetectorStatus {
    /// UL 2034-listed CO detector installed adjacent to sleeping area +
    /// tested + within 7-year manufacturer-listed end-of-life window.
    UL2034InstalledAndCurrent,
    /// CO detector installed but past UL-2034-listed end-of-life
    /// (typically 5-7 years from manufacture).
    InstalledButPastEndOfLifeWindow,
    /// CO detector installed but not UL 2034 listed.
    InstalledButNotUL2034Listed,
    /// No CO detector installed despite attached garage or fossil-fuel
    /// appliance.
    NotInstalledViolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantUL2034InstalledAndDisclosed,
    DetectorNotInstalledStatutoryViolation,
    DetectorPastEndOfLifeReplacementRequired,
    DetectorNotUL2034ListedNonCompliant,
    DisclosureRequiredAtLeaseSigning,
    CoExposureInjuryHabitabilityBreach,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub exposure_risk: CoExposureRisk,
    pub detector_status: DetectorStatus,
    pub disclosure_provided_at_lease_signing: bool,
    pub co_exposure_injury_occurred: bool,
    pub years_since_detector_manufacture: u32,
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

pub const CDC_ANNUAL_NON_FIRE_CO_DEATHS: u32 = 450;
pub const CDC_ANNUAL_CO_NONFATAL_INJURIES: u32 = 20_000;
pub const UL_2034_END_OF_LIFE_YEARS: u32 = 7;
pub const CA_SB_183_EFFECTIVE_YEAR: i32 = 2011;
pub const NY_AMANDAS_LAW_EFFECTIVE_DATE: &str = "2010-02-22";
pub const WA_RCW_19_27_530_RESIDENTIAL_DEADLINE: &str = "2013-01-01";

pub fn check(input: &Input) -> Output {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let severity: Severity;

    if matches!(input.exposure_risk, CoExposureRisk::MinimalNoFossilFuelOrGarage) {
        notes.push(
            "No attached garage AND no fossil-fuel-burning appliance — CO detector \
             requirement not statutorily triggered absent state law catch-all (some states \
             require CO detector in ALL residential units). Confirm state-specific rule \
             before forgoing detector; voluntary installation is best-practice baseline."
                .to_string(),
        );
        return Output {
            severity: Severity::NotApplicable,
            recommended_actions: actions,
            annual_rent_at_risk_cents: 0,
            citation: "n/a (no CO exposure pathway)",
            notes,
        };
    }

    if input.co_exposure_injury_occurred {
        severity = Severity::CoExposureInjuryHabitabilityBreach;
        actions.push(format!(
            "CO exposure injury to tenant occurred — immediate habitability breach plus \
             premises-liability exposure. Attached-garage vehicle-exhaust CO can reach \
             lethal 400 ppm within 5 minutes per CDC field studies. CDC reports approximately \
             {} US deaths annually from non-fire CO poisoning plus {} nonfatal injuries. \
             Document injury via medical records preserving carboxyhemoglobin level test \
             results; preserve scene with detector + battery + ventilation evidence; notify \
             general liability + umbrella policy carriers within 24 hours; engage premises-\
             liability counsel. Tenant settlement exposure routinely $1M-$5M for permanent \
             neurological injury or wrongful death. Install UL 2034-listed CO detector \
             adjacent to sleeping area within 48 hours; consider whole-house ventilation \
             improvement.",
            CDC_ANNUAL_NON_FIRE_CO_DEATHS, CDC_ANNUAL_CO_NONFATAL_INJURIES
        ));
    } else if matches!(input.detector_status, DetectorStatus::NotInstalledViolation) {
        severity = Severity::DetectorNotInstalledStatutoryViolation;
        actions.push(
            "Statutory CO detector requirement violated — install UL 2034-listed detector \
             adjacent to each sleeping area + on each level of unit + within 15 feet of any \
             fossil-fuel-burning appliance + within unit-to-attached-garage shared boundary \
             area. CA SB 183 + NY Amanda's Law + WA RCW 19.27.530 + MA Nicole's Law + CT \
             C.G.S. § 29-292(b) all impose state-level mandates. HPD / state fire marshal \
             may impose civil penalty plus tenant rent withholding plus repair-and-deduct \
             remedies. Install within 7 days; document with installation invoice plus \
             tenant acknowledgment receipt."
                .to_string(),
        );
    } else if matches!(
        input.detector_status,
        DetectorStatus::InstalledButPastEndOfLifeWindow
    ) || input.years_since_detector_manufacture > UL_2034_END_OF_LIFE_YEARS
    {
        severity = Severity::DetectorPastEndOfLifeReplacementRequired;
        actions.push(format!(
            "CO detector past UL 2034 manufacturer-listed end-of-life window ({} years) — \
             electrochemical sensor degrades over time and may fail to detect dangerous CO \
             levels. Replace with new UL 2034-listed detector within 7 days; document \
             replacement with serial number + manufacture date + tenant acknowledgment + \
             10-year sealed-battery model recommended for elimination of battery-replacement \
             failure mode.",
            UL_2034_END_OF_LIFE_YEARS
        ));
    } else if matches!(
        input.detector_status,
        DetectorStatus::InstalledButNotUL2034Listed
    ) {
        severity = Severity::DetectorNotUL2034ListedNonCompliant;
        actions.push(
            "CO detector installed but NOT UL 2034 listed — non-compliant with state \
             statutory requirements (CA SB 183 + NY Amanda's Law + WA RCW 19.27.530 + MA \
             Nicole's Law + CT C.G.S. § 29-292(b) all require UL 2034 listing or equivalent). \
             Imported / unlisted detectors lack tested-performance assurance. Replace with \
             UL 2034-listed unit; preserve invoice plus UL listing-mark photo documentation."
                .to_string(),
        );
    } else if !input.disclosure_provided_at_lease_signing {
        severity = Severity::DisclosureRequiredAtLeaseSigning;
        actions.push(
            "Provide written CO disclosure at lease signing: (1) attached-garage / fossil-\
             fuel-appliance CO exposure pathway, (2) UL 2034-listed detector model + \
             location + manufacture date + replacement schedule, (3) tenant testing-\
             responsibility allocation (monthly test recommended), (4) emergency-evacuation \
             protocol on CO alarm trigger (immediate evacuation + 911 + utility shutoff), \
             (5) tenant-acknowledgment signature. Reference state-specific disclosure form \
             (CA Cal. Health & Safety Code § 17926.1; MA Nicole's Law form)."
                .to_string(),
        );
    } else {
        severity = Severity::CompliantUL2034InstalledAndDisclosed;
        actions.push(format!(
            "Compliant: UL 2034-listed CO detector installed within {}-year end-of-life \
             window + disclosure provided at lease signing. Maintain biennial inspection \
             plus battery-replacement cadence (or 10-year sealed-battery model); train \
             property-management staff on CO-alarm emergency response protocol; track \
             manufacture-date replacement calendar. CDC public-health surveillance reports \
             {} annual US non-fire CO deaths plus {} nonfatal injuries — compliance \
             materially reduces tenant injury exposure.",
            UL_2034_END_OF_LIFE_YEARS, CDC_ANNUAL_NON_FIRE_CO_DEATHS, CDC_ANNUAL_CO_NONFATAL_INJURIES
        ));
    }

    match input.jurisdiction {
        Jurisdiction::California => {
            notes.push(format!(
                "California Senate Bill 183 (Carbon Monoxide Poisoning Prevention Act of \
                 2010) plus Cal. Health & Safety Code § 17926-17926.2 require CO alarm in \
                 ALL dwelling units with fossil-fuel-burning appliances + fireplaces + \
                 attached garages; effective January 1, {} for single-family + January 1, \
                 2013 for multifamily. UL 2034-listing required. Cal. Civ. Code § 1947.13 \
                 limits rent collection if non-compliant.",
                CA_SB_183_EFFECTIVE_YEAR
            ));
        }
        Jurisdiction::NewYork => {
            notes.push(format!(
                "New York Amanda's Law (NY Public Health Law § 1399-bbb-1) named after \
                 Amanda Hansen who died age 16 from CO poisoning at a sleepover, effective \
                 {} — requires UL 2034-listed CO detector adjacent to bedroom in all \
                 residential structures with attached garage OR fossil-fuel-burning \
                 appliance. NYC Admin Code § 27-2046.2 + HPD Local Law 7 of 2004 + 1 RCNY \
                 § 12-12 implement at city level. HPD civil penalty up to $25 per detector \
                 per day per violation.",
                NY_AMANDAS_LAW_EFFECTIVE_DATE
            ));
        }
        Jurisdiction::Washington => {
            notes.push(format!(
                "RCW 19.27.530 required CO alarms in all newly constructed residential \
                 buildings as of January 1, 2011, plus all other residential occupancies by \
                 {}. UL 2034-listing required + adjacent to each sleeping area + on each \
                 level. Washington Department of Health overlay.",
                WA_RCW_19_27_530_RESIDENTIAL_DEADLINE
            ));
        }
        Jurisdiction::Connecticut => {
            notes.push(
                "Connecticut C.G.S. § 29-292(b) plus § 47a-7 require CO detector in all \
                 rental units with attached garage or fossil-fuel-burning appliance; UL \
                 2034-listing required + landlord must provide working detector at lease \
                 commencement and replace as needed throughout tenancy. CT State Fire \
                 Marshal Office enforcement."
                    .to_string(),
            );
        }
        Jurisdiction::Massachusetts => {
            notes.push(
                "Massachusetts Nicole's Law M.G.L. ch. 148 § 26F (named after Nicole \
                 Garofalo who died age 7 from CO poisoning) effective March 31, 2006 \
                 requires UL 2034-listed CO alarm on each level of all residential \
                 dwellings with fossil-fuel-burning appliance OR attached garage. 527 \
                 CMR 1.00 Comprehensive Fire Safety Code implements. MA Office of Public \
                 Safety and Inspections enforcement."
                    .to_string(),
            );
        }
        Jurisdiction::Default => {
            notes.push(format!(
                "Federal CDC public-health surveillance reports approximately {} US deaths \
                 annually from non-fire carbon monoxide poisoning plus {} nonfatal \
                 injuries. CO is the LEADING cause of poison-related death in the US. \
                 State CO detector requirements vary; majority of states (30+) require CO \
                 detector in residential units with attached garage OR fossil-fuel-burning \
                 appliance. UL 2034 listing is the federal de facto safety standard.",
                CDC_ANNUAL_NON_FIRE_CO_DEATHS, CDC_ANNUAL_CO_NONFATAL_INJURIES
            ));
        }
    }

    notes.push(
        "Coordination with [[rental_natural_gas_leak_response]] (gas-line leak protocol — \
         distinct exposure pathway), [[rental_pellet_stove_disclosure]] (iter 499 — solid-\
         fuel appliance CO subset), [[rental_chimney_fireplace_inspection_disclosure]] \
         (chimney-vented appliance CO), [[rental_oil_tank_replacement_disclosure]] (iter \
         493 — oil-heat fuel-storage framework), [[rental_in_unit_laundry_appliance_\
         provision]] (iter 501 — gas dryer CO concern subset), [[rental_radiator_steam_\
         heat_safety]] (iter 517 — distinct steam-heat framework), [[mid_tenancy_temporary_\
         relocation]] (when unit unsafe pending detector installation or remediation), \
         [[tenant_emotional_distress_damages]] (IIED claim for CO-poisoning psychological \
         injury)."
            .to_string(),
    );

    let annual_rent_at_risk: u64 = match severity {
        Severity::CoExposureInjuryHabitabilityBreach
        | Severity::DetectorNotInstalledStatutoryViolation => input.annual_rent_cents,
        Severity::DetectorPastEndOfLifeReplacementRequired
        | Severity::DetectorNotUL2034ListedNonCompliant
        | Severity::DisclosureRequiredAtLeaseSigning => {
            input.annual_rent_cents.saturating_div(2)
        }
        _ => 0,
    };

    Output {
        severity,
        recommended_actions: actions,
        annual_rent_at_risk_cents: annual_rent_at_risk,
        citation: match input.jurisdiction {
            Jurisdiction::California => {
                "Cal. SB 183 + Cal. Health & Safety Code § 17926-17926.2 + Cal. Civ. Code § 1947.13"
            }
            Jurisdiction::NewYork => "NY PHL § 1399-bbb-1 Amanda's Law + NYC Admin § 27-2046.2 + HPD LL 7 of 2004",
            Jurisdiction::Washington => "RCW 19.27.530 + WA DOH",
            Jurisdiction::Connecticut => "C.G.S. § 29-292(b) + § 47a-7 + CT SFMO",
            Jurisdiction::Massachusetts => "M.G.L. ch. 148 § 26F Nicole's Law + 527 CMR 1.00",
            Jurisdiction::Default => "CDC public-health surveillance + UL 2034 federal standard + 30+ state mandates",
        },
        notes,
    }
}

pub type RentalAttachedGarageCarbonMonoxideDisclosureInput = Input;
pub type RentalAttachedGarageCarbonMonoxideDisclosureResult = Output;

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            exposure_risk: CoExposureRisk::AttachedGarageSharedBoundary,
            detector_status: DetectorStatus::UL2034InstalledAndCurrent,
            disclosure_provided_at_lease_signing: true,
            co_exposure_injury_occurred: false,
            years_since_detector_manufacture: 2,
            annual_rent_cents: 36_000_00,
        }
    }

    #[test]
    fn no_exposure_risk_not_applicable() {
        let mut i = baseline();
        i.exposure_risk = CoExposureRisk::MinimalNoFossilFuelOrGarage;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn compliant_ul_2034_with_disclosure() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CompliantUL2034InstalledAndDisclosed
        ));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
        assert!(r.recommended_actions.iter().any(|a| a.contains("CDC")));
    }

    #[test]
    fn detector_not_installed_statutory_violation_full_rent() {
        let mut i = baseline();
        i.detector_status = DetectorStatus::NotInstalledViolation;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::DetectorNotInstalledStatutoryViolation
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("CA SB 183")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("Amanda's Law")));
    }

    #[test]
    fn detector_past_end_of_life_replacement_required() {
        let mut i = baseline();
        i.detector_status = DetectorStatus::InstalledButPastEndOfLifeWindow;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::DetectorPastEndOfLifeReplacementRequired
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
        assert!(r.recommended_actions.iter().any(|a| a.contains("10-year sealed-battery")));
    }

    #[test]
    fn years_since_manufacture_over_7_triggers_replacement() {
        let mut i = baseline();
        i.years_since_detector_manufacture = 8;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::DetectorPastEndOfLifeReplacementRequired
        ));
    }

    #[test]
    fn detector_not_ul_2034_listed_non_compliant() {
        let mut i = baseline();
        i.detector_status = DetectorStatus::InstalledButNotUL2034Listed;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::DetectorNotUL2034ListedNonCompliant
        ));
        assert!(r.recommended_actions.iter().any(|a| a.contains("UL 2034")));
    }

    #[test]
    fn missing_disclosure_at_lease_signing() {
        let mut i = baseline();
        i.disclosure_provided_at_lease_signing = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::DisclosureRequiredAtLeaseSigning));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 17926.1")));
    }

    #[test]
    fn co_injury_habitability_breach_full_rent() {
        let mut i = baseline();
        i.co_exposure_injury_occurred = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CoExposureInjuryHabitabilityBreach
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("400 ppm")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("$1M-$5M")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("carboxyhemoglobin")));
    }

    #[test]
    fn ca_jurisdiction_pins_sb_183() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Senate Bill 183")));
        assert!(r.notes.iter().any(|n| n.contains("§ 17926")));
        assert!(r.notes.iter().any(|n| n.contains("January 1, 2011")));
    }

    #[test]
    fn ny_jurisdiction_pins_amandas_law_phl_1399_bbb_1() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Amanda's Law")));
        assert!(r.notes.iter().any(|n| n.contains("Amanda Hansen")));
        assert!(r.notes.iter().any(|n| n.contains("§ 1399-bbb-1")));
        assert!(r.notes.iter().any(|n| n.contains("2010-02-22")));
    }

    #[test]
    fn wa_jurisdiction_pins_rcw_19_27_530() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Washington;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("RCW 19.27.530")));
        assert!(r.notes.iter().any(|n| n.contains("2013-01-01")));
    }

    #[test]
    fn ct_jurisdiction_pins_29_292_b() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Connecticut;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("C.G.S. § 29-292(b)")));
        assert!(r.notes.iter().any(|n| n.contains("§ 47a-7")));
    }

    #[test]
    fn ma_jurisdiction_pins_nicoles_law_ch_148_26f() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Massachusetts;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Nicole's Law")));
        assert!(r.notes.iter().any(|n| n.contains("M.G.L. ch. 148 § 26F")));
        assert!(r.notes.iter().any(|n| n.contains("Nicole Garofalo")));
        assert!(r.notes.iter().any(|n| n.contains("March 31, 2006")));
    }

    #[test]
    fn default_jurisdiction_pins_cdc_and_ul_2034() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("CDC")));
        assert!(r.notes.iter().any(|n| n.contains("UL 2034")));
        assert!(r.notes.iter().any(|n| n.contains("(30+)")));
    }

    #[test]
    fn coordination_note_references_siblings() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("rental_natural_gas_leak_response")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_oil_tank_replacement_disclosure")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_in_unit_laundry_appliance_provision")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_radiator_steam_heat_safety")));
    }

    #[test]
    fn cdc_annual_co_deaths_pins_450() {
        assert_eq!(CDC_ANNUAL_NON_FIRE_CO_DEATHS, 450);
    }

    #[test]
    fn cdc_annual_co_injuries_pins_20000() {
        assert_eq!(CDC_ANNUAL_CO_NONFATAL_INJURIES, 20_000);
    }

    #[test]
    fn ul_2034_end_of_life_pins_7_years() {
        assert_eq!(UL_2034_END_OF_LIFE_YEARS, 7);
    }

    #[test]
    fn ca_sb_183_effective_year_pins_2011() {
        assert_eq!(CA_SB_183_EFFECTIVE_YEAR, 2011);
    }

    #[test]
    fn ny_amandas_law_effective_pins_2010_02_22() {
        assert_eq!(NY_AMANDAS_LAW_EFFECTIVE_DATE, "2010-02-22");
    }

    #[test]
    fn wa_rcw_residential_deadline_pins_2013_01_01() {
        assert_eq!(WA_RCW_19_27_530_RESIDENTIAL_DEADLINE, "2013-01-01");
    }

    #[test]
    fn citation_branch_for_each_jurisdiction() {
        let ca = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::California; i });
        let ny = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::NewYork; i });
        let wa = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Washington; i });
        let ct = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Connecticut; i });
        let ma = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Massachusetts; i });
        let de = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Default; i });
        assert!(ca.citation.contains("Cal. SB 183"));
        assert!(ny.citation.contains("Amanda's Law"));
        assert!(wa.citation.contains("RCW 19.27.530"));
        assert!(ct.citation.contains("C.G.S. § 29-292(b)"));
        assert!(ma.citation.contains("Nicole's Law"));
        assert!(de.citation.contains("CDC"));
    }

    #[test]
    fn severity_priority_injury_overrides_compliant_status() {
        let mut i = baseline();
        i.co_exposure_injury_occurred = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::CoExposureInjuryHabitabilityBreach
        ));
    }

    #[test]
    fn detached_garage_with_shared_hvac_triggers_disclosure() {
        let mut i = baseline();
        i.exposure_risk = CoExposureRisk::DetachedGarageWithSharedHvacReturn;
        i.disclosure_provided_at_lease_signing = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::DisclosureRequiredAtLeaseSigning));
    }

    #[test]
    fn zero_rent_zero_risk_does_not_panic() {
        let mut i = baseline();
        i.annual_rent_cents = 0;
        i.detector_status = DetectorStatus::NotInstalledViolation;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }
}
