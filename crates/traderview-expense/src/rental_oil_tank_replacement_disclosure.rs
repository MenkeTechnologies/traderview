//! Rental home heating oil tank disclosure and replacement liability framework.
//!
//! Northeast trader-landlords (NJ, NY, MA, CT, RI, ME, NH, VT, PA) inherit
//! buried legacy USTs (underground storage tanks) from oil-heat era 1940-1980s.
//! NJDEP UHOT (Unregulated Heating Oil Tank) Program governs residential tanks
//! 2000 gal or less per N.J.A.C. 7:14B-1.6(a)(3) tank-size scope exemption from
//! federal 40 C.F.R. 280 UST rules. Massachusetts 527 CMR 9.00 (Board of Fire
//! Prevention) plus 310 CMR 80.00 (MassDEP UST regulations eff. 2015-01-02)
//! govern installation, leak detection, and closure. Maine 38 M.R.S. 568-A
//! requires age plus location disclosure on rental and sale. Federal CERCLA
//! 42 U.S.C. 9607(a)(1) imposes strict joint-and-several owner-operator
//! liability for petroleum-product UST leaks regardless of fault, knowledge, or
//! pre-purchase contamination. Tank replacement triggers: (1) age greater than
//! 30 years bare-steel UST presumptive leak; (2) failed pressure or precision
//! test; (3) visible exterior corrosion; (4) heating-oil odor; (5) soil
//! discoloration. Disclosure failure to incoming tenant equals fraud-in-the-
//! inducement plus rescission per Restatement (Second) of Contracts section
//! 164(1) plus state landlord-tenant statutes.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    NewJersey,
    Massachusetts,
    NewYork,
    Connecticut,
    RhodeIsland,
    Maine,
    NewHampshire,
    Vermont,
    Pennsylvania,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TankLocation {
    AbovegroundBasement,
    AbovegroundOutsideExterior,
    UndergroundBuried,
    NoOilTank,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TankMaterial {
    BareSteelUnprotected,
    SteelCathodicallyProtected,
    DoubleWalledSteel,
    FiberglassReinforcedPlastic,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantDisclosed,
    DisclosureRequiredNotProvided,
    ReplacementRequiredAgeFailure,
    ReplacementRequiredLeakDetected,
    CerclaStrictLiabilityExposure,
    RescissionRiskFraudInInducement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub tank_location: TankLocation,
    pub tank_material: TankMaterial,
    pub tank_age_years: u32,
    pub tank_capacity_gallons: u32,
    pub disclosure_provided_at_lease_signing: bool,
    pub disclosure_includes_age_and_location: bool,
    pub disclosure_includes_known_releases: bool,
    pub precision_test_passed_within_3_years: bool,
    pub visible_corrosion_or_oil_odor: bool,
    pub known_prior_release_or_remediation: bool,
    pub remediation_estimate_cents: u64,
    pub annual_rent_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub recommended_actions: Vec<String>,
    pub estimated_remediation_cost_cents: u64,
    pub annual_rent_at_risk_cents: u64,
    pub notes: Vec<String>,
}

pub const PRESUMPTIVE_BARE_STEEL_FAILURE_AGE_YEARS: u32 = 30;
pub const NJ_UHOT_RESIDENTIAL_SCOPE_MAX_GALLONS: u32 = 2_000;
pub const FEDERAL_UST_THRESHOLD_GALLONS: u32 = 110;

pub fn check(input: &Input) -> Output {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let severity: Severity;

    if matches!(input.tank_location, TankLocation::NoOilTank) {
        notes.push(
            "No oil tank on premises; gas/electric/heat-pump heating not in scope of this \
             framework. If conversion happened post-occupancy, confirm decommissioning per \
             N.J.A.C. 7:14B-9 closure-in-place or removal protocol (NJ) or 527 CMR 9.06 \
             (MA)."
                .to_string(),
        );
        return Output {
            severity: Severity::NotApplicable,
            recommended_actions: actions,
            estimated_remediation_cost_cents: 0,
            annual_rent_at_risk_cents: 0,
            notes,
        };
    }

    let is_ust = matches!(input.tank_location, TankLocation::UndergroundBuried);
    let bare_steel_age_failure = matches!(input.tank_material, TankMaterial::BareSteelUnprotected)
        && input.tank_age_years >= PRESUMPTIVE_BARE_STEEL_FAILURE_AGE_YEARS;

    if input.known_prior_release_or_remediation && is_ust {
        severity = Severity::CerclaStrictLiabilityExposure;
        actions.push(
            "Disclose prior release plus full N.J.A.C. 7:26E remediation history (NJDEP Case \
             Number plus Final RAW or Soil RAO) to incoming tenant in writing before lease \
             execution; failure equals fraud-in-the-inducement supporting tenant rescission."
                .to_string(),
        );
        actions.push(
            "Maintain CERCLA 42 U.S.C. 9607(a) strict-liability reserve: minimum $50,000 \
             escrow for residential UST or $100,000 for commercial multi-unit; petroleum \
             exclusion 42 U.S.C. 9601(14) does NOT bar state-law cost recovery."
                .to_string(),
        );
    } else if input.visible_corrosion_or_oil_odor {
        severity = Severity::ReplacementRequiredLeakDetected;
        actions.push(
            "Immediate precision-test plus tightness-test per 527 CMR 9.07 (MA) or N.J.A.C. \
             7:14B-6.2 (NJ); if test fails, notify NJDEP within 24 hours per N.J.A.C. \
             7:14B-7.2 or MassDEP within 24 hours per 310 CMR 80.31 plus initiate immediate \
             response action."
                .to_string(),
        );
        actions.push(
            "Suspend new tenancies and disclose suspected release status to existing \
             tenants in writing; failure to disclose triggers fraud-in-the-inducement \
             rescission risk equal to full annual rent."
                .to_string(),
        );
    } else if bare_steel_age_failure {
        severity = Severity::ReplacementRequiredAgeFailure;
        actions.push(format!(
            "Bare-steel tank age {} years exceeds presumptive {}-year failure threshold; \
             schedule replacement with double-walled fiberglass or cathodically-protected \
             steel within 90 days. Document with NJDEP UHOT Notification per N.J.A.C. \
             7:14B-9.4 or MA 527 CMR 9.06 closure permit before tenant turnover.",
            input.tank_age_years, PRESUMPTIVE_BARE_STEEL_FAILURE_AGE_YEARS
        ));
    } else if !input.disclosure_provided_at_lease_signing
        || !input.disclosure_includes_age_and_location
    {
        severity = Severity::DisclosureRequiredNotProvided;
        actions.push(
            "Provide written tank disclosure to all incoming tenants at lease signing: \
             include tank location (basement aboveground vs underground), age in years, \
             capacity in gallons, material (bare steel vs FRP vs cathodically protected), \
             last precision-test date, plus known prior releases. Pin to lease addendum."
                .to_string(),
        );
    } else if !input.disclosure_includes_known_releases && input.known_prior_release_or_remediation
    {
        severity = Severity::RescissionRiskFraudInInducement;
        actions.push(
            "Disclosure incomplete: prior release/remediation known but NOT disclosed to \
             tenant; this equals fraud-in-the-inducement per Restatement (Second) of \
             Contracts section 164(1) plus state landlord-tenant statutes. Tenant may \
             rescind lease plus recover full annual rent paid plus relocation costs."
                .to_string(),
        );
    } else {
        severity = Severity::CompliantDisclosed;
        actions.push(
            "Tank-disclosure framework compliant; maintain precision-test cadence per \
             jurisdiction (3-year for FRP, annual for bare-steel pre-replacement) plus \
             retain test certificates in landlord file for full statute-of-limitations \
             window (NJDEP 6-year per N.J.A.C. 7:14B-7.4; MassDEP 6-year per 310 CMR 80.32)."
                .to_string(),
        );
    }

    match input.jurisdiction {
        Jurisdiction::NewJersey => {
            notes.push(format!(
                "NJDEP UHOT (Unregulated Heating Oil Tank) Program governs residential \
                 heating-oil tanks; N.J.A.C. 7:14B-1.6(a)(3) scope exemption from full UST \
                 rules applies to tanks {} gallons or less serving residence on premises. \
                 Closure requires N.J.A.C. 7:14B-9.4 notification within 30 days plus \
                 N.J.A.C. 7:26E site investigation if release suspected. NJ Spill \
                 Compensation Act N.J.S.A. 58:10-23.11 imposes strict joint-and-several \
                 liability irrespective of CERCLA petroleum exclusion.",
                NJ_UHOT_RESIDENTIAL_SCOPE_MAX_GALLONS
            ));
        }
        Jurisdiction::Massachusetts => {
            notes.push(
                "527 CMR 9.00 (Mass. Board of Fire Prevention) governs installation plus \
                 removal of underground storage tanks for fuel oil; permit required from \
                 local fire department prior to any tank work. 310 CMR 80.00 (MassDEP) \
                 effective 2015-01-02 governs registration, leak detection, plus closure of \
                 USTs greater than 110 gallons used for petroleum-fuel storage. M.G.L. ch. \
                 21E imposes strict joint-and-several owner liability for oil-release \
                 cleanup including pre-existing contamination assumed at purchase."
                    .to_string(),
            );
        }
        Jurisdiction::NewYork => {
            notes.push(
                "6 NYCRR 613 (NY DEC PBS Petroleum Bulk Storage rules) governs USTs of any \
                 size at non-residential plus residential facilities greater than 1100 gal \
                 aggregate; residential tanks 1100 gal or less exempt from registration but \
                 NOT from N.Y. Navigation Law Article 12 (Oil Spill Prevention) strict \
                 liability for releases. NYC Local Law 152 (no oil-tank specific but \
                 inspections for fuel-burning equipment). Landlord disclosure not statute-\
                 specific but breach of habitability per Real Property Law section 235-b."
                    .to_string(),
            );
        }
        Jurisdiction::Connecticut => {
            notes.push(
                "Conn. Agencies Regs. section 22a-449(d)-1 governs CT DEEP UST regulations; \
                 residential heating-oil tanks 2100 gal or less exempt from UST registration \
                 but NOT from C.G.S. section 22a-452 strict liability for petroleum spill \
                 cleanup. Landlord disclosure governed by C.G.S. section 47a-7 (habitability) \
                 plus common-law fraud rule."
                    .to_string(),
            );
        }
        Jurisdiction::RhodeIsland => {
            notes.push(
                "RI DEM Regulation 250-RICR-140-25-1 (UST Regs) covers tanks greater than \
                 110 gal storing petroleum; residential heating-oil tanks excluded from \
                 registration but subject to RI Oil Pollution Control Act R.I.G.L. 46-12.5.1 \
                 strict liability for releases."
                    .to_string(),
            );
        }
        Jurisdiction::Maine => {
            notes.push(
                "38 M.R.S. section 568-A requires written disclosure to rental tenant of \
                 (1) heating-oil tank location, (2) age, (3) capacity, (4) last inspection \
                 date prior to lease signing; failure equals unfair trade practice under 5 \
                 M.R.S. ch. 10 plus tenant remedy of lease rescission plus actual damages. \
                 Maine Ground Water Oil Clean-up Fund 38 M.R.S. ch. 13-C provides limited \
                 cost-reimbursement for innocent landowner remediation."
                    .to_string(),
            );
        }
        Jurisdiction::NewHampshire => {
            notes.push(
                "Env-Or 411 (NH DES UST Rules) governs petroleum tanks greater than 110 gal; \
                 residential heating-oil tanks 1100 gal or less exempt from registration but \
                 NOT from N.H. R.S.A. 146-A (Oil Discharge and Disposal Cleanup Fund) strict \
                 liability."
                    .to_string(),
            );
        }
        Jurisdiction::Vermont => {
            notes.push(
                "VT Petroleum Cleanup Fund 10 V.S.A. ch. 159 reimburses qualified \
                 landowner cleanup costs but requires tank registration plus annual fee for \
                 USTs greater than 1100 gal; residential heating-oil tanks below threshold \
                 still subject to 10 V.S.A. section 6615 strict-liability releases."
                    .to_string(),
            );
        }
        Jurisdiction::Pennsylvania => {
            notes.push(
                "25 Pa. Code ch. 245 (PADEP Storage Tank Rules) registers USTs greater than \
                 1100 gal; residential heating-oil tanks below threshold exempt from \
                 registration but subject to PA Storage Tank and Spill Prevention Act 35 \
                 P.S. section 6021.1101 strict liability for releases. PA Underground \
                 Storage Tank Indemnification Fund (USTIF) covers third-party damages for \
                 registered tanks only."
                    .to_string(),
            );
        }
        Jurisdiction::Default => {
            notes.push(
                "Federal 40 C.F.R. Part 280 UST regulations exempt residential heating-oil \
                 tanks storing fuel for consumption on premises; CERCLA 42 U.S.C. \
                 9601(14) petroleum exclusion bars federal Superfund cost recovery but \
                 NOT state oil-spill-fund recovery actions. Consult state-specific UST \
                 program plus landlord-tenant disclosure statute."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Coordinate with [[rental_propane_tank_lease_disclosure]] (propane-tank lease \
         analog), [[rental_lead_pipe_disclosure]] (legacy infrastructure disclosure pattern), \
         [[rental_pesticide_application_notification]] (chemical-exposure disclosure pattern) \
         when oil tank coexists with other legacy hazards on the same property."
            .to_string(),
    );

    let annual_rent_at_risk: u64 = match severity {
        Severity::RescissionRiskFraudInInducement
        | Severity::CerclaStrictLiabilityExposure
        | Severity::ReplacementRequiredLeakDetected => input.annual_rent_cents,
        Severity::DisclosureRequiredNotProvided | Severity::ReplacementRequiredAgeFailure => {
            input.annual_rent_cents.saturating_div(2)
        }
        _ => 0,
    };

    Output {
        severity,
        recommended_actions: actions,
        estimated_remediation_cost_cents: input.remediation_estimate_cents,
        annual_rent_at_risk_cents: annual_rent_at_risk,
        notes,
    }
}

pub type RentalOilTankReplacementDisclosureInput = Input;
pub type RentalOilTankReplacementDisclosureResult = Output;

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::NewJersey,
            tank_location: TankLocation::AbovegroundBasement,
            tank_material: TankMaterial::SteelCathodicallyProtected,
            tank_age_years: 12,
            tank_capacity_gallons: 275,
            disclosure_provided_at_lease_signing: true,
            disclosure_includes_age_and_location: true,
            disclosure_includes_known_releases: true,
            precision_test_passed_within_3_years: true,
            visible_corrosion_or_oil_odor: false,
            known_prior_release_or_remediation: false,
            remediation_estimate_cents: 0,
            annual_rent_cents: 36_000_00,
        }
    }

    #[test]
    fn no_oil_tank_returns_not_applicable() {
        let mut i = baseline();
        i.tank_location = TankLocation::NoOilTank;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
        assert!(r.notes.iter().any(|n| n.contains("No oil tank")));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn ust_with_known_prior_release_triggers_cercla_exposure() {
        let mut i = baseline();
        i.tank_location = TankLocation::UndergroundBuried;
        i.known_prior_release_or_remediation = true;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CerclaStrictLiabilityExposure));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("strict-liability reserve")));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
    }

    #[test]
    fn ust_known_release_action_pins_njdep_case_number() {
        let mut i = baseline();
        i.tank_location = TankLocation::UndergroundBuried;
        i.known_prior_release_or_remediation = true;
        let r = check(&i);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("NJDEP Case Number")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Final RAW or Soil RAO")));
    }

    #[test]
    fn cercla_petroleum_exclusion_pinned_in_action() {
        let mut i = baseline();
        i.tank_location = TankLocation::UndergroundBuried;
        i.known_prior_release_or_remediation = true;
        let r = check(&i);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("petroleum exclusion")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("42 U.S.C. 9601(14)")));
    }

    #[test]
    fn visible_corrosion_triggers_immediate_replacement() {
        let mut i = baseline();
        i.visible_corrosion_or_oil_odor = true;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::ReplacementRequiredLeakDetected));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("precision-test")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("24 hours")));
    }

    #[test]
    fn visible_corrosion_action_pins_njdep_notification_window() {
        let mut i = baseline();
        i.visible_corrosion_or_oil_odor = true;
        let r = check(&i);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("N.J.A.C. 7:14B-7.2")));
    }

    #[test]
    fn visible_corrosion_action_pins_massdep_notification_window() {
        let mut i = baseline();
        i.visible_corrosion_or_oil_odor = true;
        let r = check(&i);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("310 CMR 80.31")));
    }

    #[test]
    fn bare_steel_age_30_triggers_replacement() {
        let mut i = baseline();
        i.tank_material = TankMaterial::BareSteelUnprotected;
        i.tank_age_years = 30;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::ReplacementRequiredAgeFailure));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("90 days")));
    }

    #[test]
    fn bare_steel_age_29_compliant_with_disclosure() {
        let mut i = baseline();
        i.tank_material = TankMaterial::BareSteelUnprotected;
        i.tank_age_years = 29;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantDisclosed));
    }

    #[test]
    fn frp_tank_age_45_compliant_with_disclosure() {
        let mut i = baseline();
        i.tank_material = TankMaterial::FiberglassReinforcedPlastic;
        i.tank_age_years = 45;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantDisclosed));
    }

    #[test]
    fn missing_disclosure_at_lease_signing_triggers_required_status() {
        let mut i = baseline();
        i.disclosure_provided_at_lease_signing = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::DisclosureRequiredNotProvided));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("lease addendum")));
    }

    #[test]
    fn disclosure_missing_age_or_location_triggers_required_status() {
        let mut i = baseline();
        i.disclosure_includes_age_and_location = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::DisclosureRequiredNotProvided));
    }

    #[test]
    fn known_release_undisclosed_triggers_rescission_risk() {
        let mut i = baseline();
        i.tank_location = TankLocation::AbovegroundBasement;
        i.known_prior_release_or_remediation = true;
        i.disclosure_includes_known_releases = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::RescissionRiskFraudInInducement));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Restatement (Second) of Contracts section 164(1)")));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
    }

    #[test]
    fn nj_jurisdiction_pins_uhot_scope() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("NJDEP UHOT")));
        assert!(r.notes.iter().any(|n| n.contains("N.J.A.C. 7:14B-1.6(a)(3)")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains(&NJ_UHOT_RESIDENTIAL_SCOPE_MAX_GALLONS.to_string())));
    }

    #[test]
    fn nj_jurisdiction_pins_spill_act_strict_liability() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("N.J.S.A. 58:10-23.11")));
    }

    #[test]
    fn ma_jurisdiction_pins_527_cmr_and_310_cmr() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Massachusetts;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("527 CMR 9.00")));
        assert!(r.notes.iter().any(|n| n.contains("310 CMR 80.00")));
        assert!(r.notes.iter().any(|n| n.contains("M.G.L. ch. 21E")));
    }

    #[test]
    fn ny_jurisdiction_pins_navigation_law_article_12() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("N.Y. Navigation Law Article 12")));
        assert!(r.notes.iter().any(|n| n.contains("6 NYCRR 613")));
    }

    #[test]
    fn ct_jurisdiction_pins_22a_449() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Connecticut;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Conn. Agencies Regs. section 22a-449(d)-1")));
        assert!(r.notes.iter().any(|n| n.contains("C.G.S. section 47a-7")));
    }

    #[test]
    fn me_jurisdiction_pins_38_mrs_568_a() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Maine;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("38 M.R.S. section 568-A")));
        assert!(r.notes.iter().any(|n| n.contains("5 M.R.S. ch. 10")));
    }

    #[test]
    fn ri_jurisdiction_pins_dem_regulation() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::RhodeIsland;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("250-RICR-140-25-1")));
        assert!(r.notes.iter().any(|n| n.contains("R.I.G.L. 46-12.5.1")));
    }

    #[test]
    fn nh_jurisdiction_pins_env_or_411() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewHampshire;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Env-Or 411")));
        assert!(r.notes.iter().any(|n| n.contains("N.H. R.S.A. 146-A")));
    }

    #[test]
    fn vt_jurisdiction_pins_petroleum_cleanup_fund() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Vermont;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("10 V.S.A. ch. 159")));
        assert!(r.notes.iter().any(|n| n.contains("10 V.S.A. section 6615")));
    }

    #[test]
    fn pa_jurisdiction_pins_25_pa_code_245() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Pennsylvania;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("25 Pa. Code ch. 245")));
        assert!(r.notes.iter().any(|n| n.contains("USTIF")));
    }

    #[test]
    fn default_jurisdiction_pins_federal_40_cfr_280() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("40 C.F.R. Part 280")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("CERCLA 42 U.S.C. 9601(14) petroleum exclusion")));
    }

    #[test]
    fn coordination_note_references_propane_lease_and_lead_pipe() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_propane_tank_lease_disclosure")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_lead_pipe_disclosure")));
    }

    #[test]
    fn compliant_disclosure_zero_rent_at_risk() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantDisclosed));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("precision-test cadence")));
    }

    #[test]
    fn half_rent_at_risk_for_disclosure_required_severity() {
        let mut i = baseline();
        i.disclosure_provided_at_lease_signing = false;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
    }

    #[test]
    fn half_rent_at_risk_for_replacement_required_age_failure() {
        let mut i = baseline();
        i.tank_material = TankMaterial::BareSteelUnprotected;
        i.tank_age_years = 35;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::ReplacementRequiredAgeFailure));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
    }

    #[test]
    fn saturating_div_does_not_panic_on_zero_annual_rent() {
        let mut i = baseline();
        i.annual_rent_cents = 0;
        i.disclosure_provided_at_lease_signing = false;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn remediation_estimate_passed_through_unchanged() {
        let mut i = baseline();
        i.tank_location = TankLocation::UndergroundBuried;
        i.known_prior_release_or_remediation = true;
        i.remediation_estimate_cents = 75_000_00;
        let r = check(&i);
        assert_eq!(r.estimated_remediation_cost_cents, 75_000_00);
    }

    #[test]
    fn coordination_note_pinned_in_all_jurisdictions() {
        for jurisdiction in [
            Jurisdiction::NewJersey,
            Jurisdiction::Massachusetts,
            Jurisdiction::NewYork,
            Jurisdiction::Connecticut,
            Jurisdiction::RhodeIsland,
            Jurisdiction::Maine,
            Jurisdiction::NewHampshire,
            Jurisdiction::Vermont,
            Jurisdiction::Pennsylvania,
            Jurisdiction::Default,
        ] {
            let mut i = baseline();
            i.jurisdiction = jurisdiction;
            let r = check(&i);
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("rental_propane_tank_lease_disclosure")),
                "coordination note missing for {jurisdiction:?}"
            );
        }
    }
}
