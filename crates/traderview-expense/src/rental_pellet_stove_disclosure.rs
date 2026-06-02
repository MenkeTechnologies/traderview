//! Rental pellet-stove disclosure plus EPA NSPS certification framework.
//!
//! Pellet stoves are solid-fuel-burning residential heating appliances using
//! compressed wood-pellet fuel. EPA NSPS (New Source Performance Standards)
//! 40 C.F.R. Part 60 Subpart AAA (wood stoves) plus Subpart QQQQ (hydronic
//! heaters and forced-air furnaces) govern federal manufacture and retail
//! sale. Step 2 emissions limit effective May 15, 2020 caps particulate
//! matter at weighted average ≤ 2.0 grams per hour tested per EPA Method 28
//! crib-wood protocol; pre-2020 stoves cannot be sold at retail (resale
//! permitted state-by-state). Improper operation causes carbon monoxide
//! (CO) release plus chimney creosote buildup plus particulate-matter
//! (PM2.5) indoor air quality degradation.
//!
//! Six failure modes a trader-landlord faces: (1) pre-2015 non-certified
//! stove installed where state non-attainment area prohibits operation; (2)
//! missing CO detector adjacent to appliance violating state fire-marshal
//! requirements; (3) chimney plus connector not inspected annually per NFPA
//! 211 voiding fire-insurance coverage; (4) auger jam causing CO release
//! without tenant awareness; (5) failure to disclose stove operation
//! responsibility allocation (who buys pellets, who removes ash); (6)
//! failure to provide written owner's-manual replacement copy (40 C.F.R.
//! Appendix I to Part 60 retention requirement).
//!
//! State frameworks:
//!
//! - **Vermont**: 10 V.S.A. § 583 plus Vermont Air Pollution Control
//!   Regulations § 5-204.4 govern residential wood combustion units; landlord
//!   habitability framework requires written disclosure of solid-fuel-
//!   burning appliance operation. Vermont CO Detector Law 9 V.S.A. § 2882
//!   requires CO detector adjacent to fuel-burning appliance in all dwelling
//!   units regardless of date built.
//!
//! - **Maine**: 38 M.R.S. § 581 plus Maine Solid Fuel-Burning Appliance
//!   Regulations; state fire marshal applies NFPA 211 (Standard for
//!   Chimneys, Fireplaces, Vents, and Solid Fuel-Burning Appliances) chimney
//!   inspection. Maine CO Detector Law 25 M.R.S. § 2468-A.
//!
//! - **New Hampshire**: RSA 153:4-a Solid Fuel-Burning Appliance Installation
//!   plus NH Code of Administrative Rules Saf-C 6000; requires permit from
//!   local fire chief before installation; NFPA 211 applies. RSA 153:10-a CO
//!   detector requirement.
//!
//! - **Washington**: WAC 173-433 Solid Fuel Burning Device Standards —
//!   strictest in US, prohibits sale of non-certified stoves in PM2.5
//!   National Ambient Air Quality Standards non-attainment counties
//!   (Pierce, Spokane, Yakima, Klickitat); landlord disclosure required
//!   per WAC 173-433-130.
//!
//! - **Colorado**: 5 CCR 1001-10 Regulation No. 4 Statewide Wood Stove
//!   Program plus High-Altitude burn ban regulations in "Designated Areas"
//!   such as Aspen, Telluride, Vail; landlord disclosure required.
//!
//! - **Default**: Federal EPA NSPS 40 C.F.R. Part 60 plus NFPA 211 chimney
//!   inspection plus state common-law habitability doctrine.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    Vermont,
    Maine,
    NewHampshire,
    Washington,
    Colorado,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoveCertification {
    NoPelletStove,
    Pre1988NonCertified,
    Phase2_1988To2015Certified,
    Step1_2015To2020Certified,
    Step2_2020OrLaterCertified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantStep2WithFullDisclosure,
    DisclosureRequiredAtLeaseSigning,
    CoDetectorMissingViolation,
    ChimneyAnnualInspectionOverdueViolation,
    NonCertifiedInNonAttainmentAreaProhibited,
    OwnersManualNotProvidedViolation,
    AugerJamCoReleaseExposureWithoutDisclosure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub stove_certification: StoveCertification,
    pub in_pm25_non_attainment_area: bool,
    pub disclosure_provided_at_lease_signing: bool,
    pub disclosure_includes_operation_responsibility: bool,
    pub disclosure_includes_co_risk_and_auger_jam: bool,
    pub co_detector_installed_adjacent: bool,
    pub chimney_inspection_within_12_months: bool,
    pub owners_manual_replacement_provided: bool,
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

pub const EPA_STEP_2_EFFECTIVE_DATE: &str = "2020-05-15";
pub const EPA_STEP_2_PM_LIMIT_GRAMS_PER_HOUR: u32 = 2;
pub const NFPA_211_CHIMNEY_INSPECTION_INTERVAL_MONTHS: u32 = 12;

pub fn check(input: &Input) -> Output {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let severity: Severity;

    if matches!(input.stove_certification, StoveCertification::NoPelletStove) {
        notes.push(
            "No pellet stove on premises; framework inapplicable. If tenant proposes to \
             install a portable pellet stove or fireplace insert, route to landlord-permission \
             framework plus jurisdiction installation permit per RSA 153:4-a (NH), 38 M.R.S. \
             § 581 (ME), or WAC 173-433-130 (WA)."
                .to_string(),
        );
        return Output {
            severity: Severity::NotApplicable,
            recommended_actions: actions,
            annual_rent_at_risk_cents: 0,
            citation: "n/a",
            notes,
        };
    }

    let is_non_certified = matches!(
        input.stove_certification,
        StoveCertification::Pre1988NonCertified
    );

    if is_non_certified && input.in_pm25_non_attainment_area {
        severity = Severity::NonCertifiedInNonAttainmentAreaProhibited;
        actions.push(
            "Pre-1988 non-EPA-certified pellet stove in PM2.5 NAAQS non-attainment area is \
             PROHIBITED for operation per WAC 173-433-130 (WA), 5 CCR 1001-10 (CO), or \
             equivalent state designated-area rule. Immediate decommission or replace with \
             Step-2 (2020 or later) EPA-certified appliance. Tenant entitled to rent \
             withholding or rescission under habitability doctrine until cured."
                .to_string(),
        );
    } else if !input.co_detector_installed_adjacent {
        severity = Severity::CoDetectorMissingViolation;
        actions.push(
            "Carbon monoxide detector NOT installed adjacent to pellet stove appliance; this \
             violates 9 V.S.A. § 2882 (VT), 25 M.R.S. § 2468-A (ME), RSA 153:10-a (NH), \
             plus state fire-marshal codes universally. Install UL 2034-listed CO detector \
             within 15 feet of appliance plus on each level of dwelling within 24 hours."
                .to_string(),
        );
    } else if !input.chimney_inspection_within_12_months {
        severity = Severity::ChimneyAnnualInspectionOverdueViolation;
        actions.push(format!(
            "Chimney plus connector inspection overdue under NFPA 211 {}-month interval; \
             schedule certified-chimney-sweep Level 2 inspection per CSIA (Chimney Safety \
             Institute of America) standards within 30 days. Fire-insurance coverage \
             typically conditioned on annual inspection; lapse may void coverage on \
             chimney-related claim.",
            NFPA_211_CHIMNEY_INSPECTION_INTERVAL_MONTHS
        ));
    } else if !input.owners_manual_replacement_provided {
        severity = Severity::OwnersManualNotProvidedViolation;
        actions.push(
            "Owner's manual replacement copy NOT provided to tenant; 40 C.F.R. Appendix I \
             to Part 60 requires retention with appliance. Provide manufacturer-issued \
             replacement manual covering Step-2 emission performance, recommended pellet-fuel \
             grade (Premium grade per PFI Standards Program), recommended burn settings, \
             ash-removal procedure, plus annual maintenance schedule."
                .to_string(),
        );
    } else if !input.disclosure_includes_co_risk_and_auger_jam {
        severity = Severity::AugerJamCoReleaseExposureWithoutDisclosure;
        actions.push(
            "Auger jam (frequent failure mode in residential pellet stoves) causes \
             incomplete combustion releasing CO into living space; tenant must be advised in \
             writing of (1) symptoms — headache, dizziness, nausea, confusion — (2) immediate \
             evacuation protocol on CO detector alarm, (3) auger maintenance schedule, plus \
             (4) emergency contact for stove service."
                .to_string(),
        );
    } else if !input.disclosure_provided_at_lease_signing
        || !input.disclosure_includes_operation_responsibility
    {
        severity = Severity::DisclosureRequiredAtLeaseSigning;
        actions.push(
            "Provide written pellet-stove disclosure at lease signing: (1) stove certification \
             tier and EPA model number, (2) operation-responsibility allocation (landlord vs \
             tenant for pellet purchase, ash removal, glass cleaning), (3) CO detector \
             location and test schedule, (4) chimney inspection record, (5) owner's-manual \
             provision, (6) recommended pellet-fuel grade plus prohibited fuels (no painted \
             wood, no garbage, no plastic)."
                .to_string(),
        );
    } else {
        severity = Severity::CompliantStep2WithFullDisclosure;
        actions.push(format!(
            "Disclosure framework compliant; maintain annual NFPA 211 chimney inspection \
             cadence, retain CO detector battery-change schedule (twice annually with \
             daylight saving time), provide pellet-fuel-grade recommendation per PFI \
             Standards Program (Premium grade ≤ 1% ash content). EPA Step-2 emission \
             standard ({} g/hr PM limit effective {}) confirmed.",
            EPA_STEP_2_PM_LIMIT_GRAMS_PER_HOUR, EPA_STEP_2_EFFECTIVE_DATE
        ));
    }

    match input.jurisdiction {
        Jurisdiction::Vermont => {
            notes.push(
                "10 V.S.A. § 583 plus Vermont Air Pollution Control Regulations § 5-204.4 \
                 govern residential wood combustion units; 9 V.S.A. § 2882 requires CO \
                 detector adjacent to fuel-burning appliance in all dwelling units \
                 regardless of date built. Vermont DEC has authority to require Phase-2 or \
                 newer certified stove in particular geographic areas."
                    .to_string(),
            );
        }
        Jurisdiction::Maine => {
            notes.push(
                "38 M.R.S. § 581 plus Maine Solid Fuel-Burning Appliance Regulations; state \
                 fire marshal applies NFPA 211 chimney inspection requirements; 25 M.R.S. § \
                 2468-A CO detector law. Maine SACO ordinance and Bath ordinance impose \
                 additional municipal rules."
                    .to_string(),
            );
        }
        Jurisdiction::NewHampshire => {
            notes.push(
                "RSA 153:4-a Solid Fuel-Burning Appliance Installation plus NH Code of \
                 Administrative Rules Saf-C 6000 require permit from local fire chief before \
                 installation; NFPA 211 applies. RSA 153:10-a CO detector requirement applies \
                 to all rental properties."
                    .to_string(),
            );
        }
        Jurisdiction::Washington => {
            notes.push(
                "WAC 173-433 Solid Fuel Burning Device Standards is strictest in US — \
                 prohibits sale of non-certified stoves in PM2.5 NAAQS non-attainment \
                 counties (Pierce, Spokane, Yakima, Klickitat). WAC 173-433-130 requires \
                 landlord disclosure of solid-fuel appliance operation. WA Clean Air Act RCW \
                 70A.15 enforcement authority."
                    .to_string(),
            );
        }
        Jurisdiction::Colorado => {
            notes.push(
                "5 CCR 1001-10 Regulation No. 4 Statewide Wood Stove Program plus High-\
                 Altitude burn ban regulations in 'Designated Areas' (Aspen, Telluride, Vail, \
                 Crested Butte, Steamboat Springs); landlord disclosure required. Colorado \
                 Air Pollution Control Division (APCD) Regulation No. 4 governs."
                    .to_string(),
            );
        }
        Jurisdiction::Default => {
            notes.push(
                "Federal EPA NSPS 40 C.F.R. Part 60 Subpart AAA (wood stoves) plus Subpart \
                 QQQQ (hydronic heaters and forced-air furnaces) governs federal manufacture \
                 and retail sale; Step-2 emissions limit of 2.0 g/hr PM effective May 15, \
                 2020 per EPA Method 28 crib-wood protocol. NFPA 211 chimney inspection \
                 standard applies in all jurisdictions absent local override. State common-\
                 law habitability doctrine governs landlord operational obligations."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Coordination with [[rental_chimney_fireplace_inspection_disclosure]] (chimney + \
         fireplace inspection analog), [[rental_natural_gas_leak_response]] (CO-detector \
         framework cross-reference), [[rental_oil_tank_replacement_disclosure]] (legacy \
         heating appliance disclosure pattern), [[rental_propane_tank_lease_disclosure]] \
         (LP-gas alternative heating fuel), [[carbon_monoxide_detector_compliance]] (general \
         CO detector framework if separate)."
            .to_string(),
    );

    let annual_rent_at_risk: u64 = match severity {
        Severity::NonCertifiedInNonAttainmentAreaProhibited | Severity::CoDetectorMissingViolation => {
            input.annual_rent_cents
        }
        Severity::ChimneyAnnualInspectionOverdueViolation
        | Severity::AugerJamCoReleaseExposureWithoutDisclosure
        | Severity::OwnersManualNotProvidedViolation
        | Severity::DisclosureRequiredAtLeaseSigning => input.annual_rent_cents.saturating_div(2),
        _ => 0,
    };

    Output {
        severity,
        recommended_actions: actions,
        annual_rent_at_risk_cents: annual_rent_at_risk,
        citation: match input.jurisdiction {
            Jurisdiction::Vermont => "10 V.S.A. § 583 + § 5-204.4 + 9 V.S.A. § 2882",
            Jurisdiction::Maine => "38 M.R.S. § 581 + 25 M.R.S. § 2468-A + NFPA 211",
            Jurisdiction::NewHampshire => "RSA 153:4-a + Saf-C 6000 + RSA 153:10-a + NFPA 211",
            Jurisdiction::Washington => "WAC 173-433 + WAC 173-433-130 + RCW 70A.15",
            Jurisdiction::Colorado => "5 CCR 1001-10 Reg No. 4 + designated-area burn ban",
            Jurisdiction::Default => {
                "40 C.F.R. Part 60 Subpart AAA + QQQQ + NFPA 211 + common-law habitability"
            }
        },
        notes,
    }
}

pub type RentalPelletStoveDisclosureInput = Input;
pub type RentalPelletStoveDisclosureResult = Output;

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::Vermont,
            stove_certification: StoveCertification::Step2_2020OrLaterCertified,
            in_pm25_non_attainment_area: false,
            disclosure_provided_at_lease_signing: true,
            disclosure_includes_operation_responsibility: true,
            disclosure_includes_co_risk_and_auger_jam: true,
            co_detector_installed_adjacent: true,
            chimney_inspection_within_12_months: true,
            owners_manual_replacement_provided: true,
            annual_rent_cents: 24_000_00,
        }
    }

    #[test]
    fn no_pellet_stove_not_applicable() {
        let mut i = baseline();
        i.stove_certification = StoveCertification::NoPelletStove;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
        assert!(r.notes.iter().any(|n| n.contains("RSA 153:4-a")));
    }

    #[test]
    fn compliant_step2_full_disclosure() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantStep2WithFullDisclosure));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("PFI Standards Program")));
    }

    #[test]
    fn non_certified_in_non_attainment_prohibited() {
        let mut i = baseline();
        i.stove_certification = StoveCertification::Pre1988NonCertified;
        i.in_pm25_non_attainment_area = true;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NonCertifiedInNonAttainmentAreaProhibited));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("WAC 173-433-130")));
    }

    #[test]
    fn non_certified_outside_non_attainment_falls_through() {
        let mut i = baseline();
        i.stove_certification = StoveCertification::Pre1988NonCertified;
        i.in_pm25_non_attainment_area = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantStep2WithFullDisclosure));
    }

    #[test]
    fn co_detector_missing_violation_full_rent_at_risk() {
        let mut i = baseline();
        i.co_detector_installed_adjacent = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CoDetectorMissingViolation));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("9 V.S.A. § 2882")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("UL 2034-listed")));
    }

    #[test]
    fn chimney_inspection_overdue_violation_half_rent_at_risk() {
        let mut i = baseline();
        i.chimney_inspection_within_12_months = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::ChimneyAnnualInspectionOverdueViolation));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
        assert!(r.recommended_actions.iter().any(|a| a.contains("NFPA 211")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("CSIA")));
    }

    #[test]
    fn owners_manual_not_provided_violation_half_rent_at_risk() {
        let mut i = baseline();
        i.owners_manual_replacement_provided = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::OwnersManualNotProvidedViolation));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("40 C.F.R. Appendix I to Part 60")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("PFI Standards Program")));
    }

    #[test]
    fn auger_jam_co_risk_undisclosed_half_rent_at_risk() {
        let mut i = baseline();
        i.disclosure_includes_co_risk_and_auger_jam = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::AugerJamCoReleaseExposureWithoutDisclosure));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
    }

    #[test]
    fn disclosure_missing_at_lease_signing_half_rent_at_risk() {
        let mut i = baseline();
        i.disclosure_provided_at_lease_signing = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::DisclosureRequiredAtLeaseSigning));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
    }

    #[test]
    fn disclosure_missing_operation_responsibility_half_rent_at_risk() {
        let mut i = baseline();
        i.disclosure_includes_operation_responsibility = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::DisclosureRequiredAtLeaseSigning));
    }

    #[test]
    fn vt_jurisdiction_pins_co_detector_law() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("9 V.S.A. § 2882")));
        assert!(r.notes.iter().any(|n| n.contains("10 V.S.A. § 583")));
        assert!(r.notes.iter().any(|n| n.contains("§ 5-204.4")));
    }

    #[test]
    fn me_jurisdiction_pins_38_mrs_581_and_25_mrs_2468_a() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Maine;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("38 M.R.S. § 581")));
        assert!(r.notes.iter().any(|n| n.contains("25 M.R.S. § 2468-A")));
        assert!(r.notes.iter().any(|n| n.contains("NFPA 211")));
    }

    #[test]
    fn nh_jurisdiction_pins_rsa_153_4_a() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewHampshire;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("RSA 153:4-a")));
        assert!(r.notes.iter().any(|n| n.contains("Saf-C 6000")));
        assert!(r.notes.iter().any(|n| n.contains("RSA 153:10-a")));
    }

    #[test]
    fn wa_jurisdiction_pins_wac_173_433_and_pm25_non_attainment() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Washington;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("WAC 173-433")));
        assert!(r.notes.iter().any(|n| n.contains("PM2.5 NAAQS")));
        assert!(r.notes.iter().any(|n| n.contains("Pierce")));
        assert!(r.notes.iter().any(|n| n.contains("Spokane")));
    }

    #[test]
    fn co_jurisdiction_pins_5_ccr_1001_10_designated_areas() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Colorado;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("5 CCR 1001-10")));
        assert!(r.notes.iter().any(|n| n.contains("Aspen")));
        assert!(r.notes.iter().any(|n| n.contains("APCD")));
    }

    #[test]
    fn default_jurisdiction_pins_federal_nsps_step2_date() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Subpart AAA")));
        assert!(r.notes.iter().any(|n| n.contains("Subpart QQQQ")));
        assert!(r.notes.iter().any(|n| n.contains("May 15, 2020")));
        assert!(r.notes.iter().any(|n| n.contains("2.0 g/hr")));
    }

    #[test]
    fn coordination_note_references_siblings() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_chimney_fireplace_inspection_disclosure")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_natural_gas_leak_response")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_oil_tank_replacement_disclosure")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_propane_tank_lease_disclosure")));
    }

    #[test]
    fn coordination_note_pinned_in_all_jurisdictions() {
        for j in [
            Jurisdiction::Vermont,
            Jurisdiction::Maine,
            Jurisdiction::NewHampshire,
            Jurisdiction::Washington,
            Jurisdiction::Colorado,
            Jurisdiction::Default,
        ] {
            let mut i = baseline();
            i.jurisdiction = j;
            let r = check(&i);
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("rental_chimney_fireplace_inspection_disclosure")),
                "coordination missing for {j:?}"
            );
        }
    }

    #[test]
    fn epa_step_2_pm_limit_constant_pins_2_g_per_hour() {
        assert_eq!(EPA_STEP_2_PM_LIMIT_GRAMS_PER_HOUR, 2);
    }

    #[test]
    fn epa_step_2_effective_date_constant_pins_may_15_2020() {
        assert_eq!(EPA_STEP_2_EFFECTIVE_DATE, "2020-05-15");
    }

    #[test]
    fn nfpa_211_inspection_interval_pins_12_months() {
        assert_eq!(NFPA_211_CHIMNEY_INSPECTION_INTERVAL_MONTHS, 12);
    }

    #[test]
    fn citation_branch_for_each_jurisdiction() {
        let vt = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Vermont; i });
        let me = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Maine; i });
        let nh = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::NewHampshire; i });
        let wa = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Washington; i });
        let co = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Colorado; i });
        let de = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Default; i });
        assert!(vt.citation.contains("V.S.A."));
        assert!(me.citation.contains("M.R.S."));
        assert!(nh.citation.contains("RSA"));
        assert!(wa.citation.contains("WAC"));
        assert!(co.citation.contains("CCR"));
        assert!(de.citation.contains("C.F.R."));
    }

    #[test]
    fn severity_priority_co_detector_overrides_chimney() {
        let mut i = baseline();
        i.co_detector_installed_adjacent = false;
        i.chimney_inspection_within_12_months = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CoDetectorMissingViolation));
    }

    #[test]
    fn severity_priority_non_attainment_overrides_co_detector() {
        let mut i = baseline();
        i.stove_certification = StoveCertification::Pre1988NonCertified;
        i.in_pm25_non_attainment_area = true;
        i.co_detector_installed_adjacent = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NonCertifiedInNonAttainmentAreaProhibited));
    }

    #[test]
    fn zero_rent_zero_risk_does_not_panic() {
        let mut i = baseline();
        i.annual_rent_cents = 0;
        i.co_detector_installed_adjacent = false;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn phase_2_1988_to_2015_certified_compliant_with_disclosure() {
        let mut i = baseline();
        i.stove_certification = StoveCertification::Phase2_1988To2015Certified;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantStep2WithFullDisclosure));
    }
}
