//! Rental post-construction lead-dust clearance testing framework — covers
//! the EPA Renovation, Repair, and Painting (RRP) rule plus HUD Lead Safe
//! Housing rule plus state-level dust-clearance regimes that govern when a
//! trader-landlord must conduct dust-wipe sampling before tenant re-occupancy
//! after renovation work on pre-1978 housing.
//!
//! Distinct from sibling [[rental_lead_pipe_disclosure]] (lead-in-drinking-
//! water disclosure) and [[rental_lead_paint_disclosure]] (initial Title X
//! § 1018 pre-lease lead-based paint disclosure if separate).
//!
//! Federal framework:
//!
//! - **EPA RRP rule** (40 C.F.R. Part 745 Subpart E) — applies to renovation,
//!   repair, or painting activities in pre-1978 target housing or child-
//!   occupied facilities that disturb more than 6 square feet interior or 20
//!   square feet exterior. Requires (1) EPA Lead-Safe Certified Firm
//!   certification, (2) certified renovator on-site, (3) pre-renovation
//!   tenant notification with "Renovate Right" EPA pamphlet, (4) work-area
//!   containment, (5) post-renovation cleaning verification using cleaning
//!   verification card. Cleaning VERIFICATION is mandatory; dust-wipe
//!   CLEARANCE testing is NOT mandatory under RRP alone but is required by
//!   contract, state law, or HUD if federally-assisted housing.
//!
//! - **HUD Lead Safe Housing Rule** (24 C.F.R. Part 35 Subparts B-R) —
//!   applies to federally-assisted pre-1978 target housing; MANDATES dust-
//!   wipe clearance testing by certified inspector / risk assessor / dust
//!   sampling technician before tenant re-occupancy. Clearance levels per
//!   24 C.F.R. § 35.1320.
//!
//! - **EPA TSCA § 402(c) Dust-Lead Hazard Standards** — pre-2020-01-06:
//!   floor 40 μg/ft², window sill 250 μg/ft², window trough 400 μg/ft².
//!   Post-2020-01-06: floor 10 μg/ft², window sill 100 μg/ft², window
//!   trough 400 μg/ft² per Federal Register June 21, 2019 final rule.
//!   Effective 2026-01-12: ANY reportable level of lead in dust constitutes
//!   a dust-lead hazard per October 24, 2024 EPA final rule (TSCA
//!   §§ 402 and 403) — zero-tolerance shift.
//!
//! State frameworks:
//!
//! - **Massachusetts** — 105 CMR 460 Lead Poisoning Prevention and Control
//!   regulations are MOST STRINGENT in US; apply to ALL rental property
//!   regardless of federal RRP scope; require Letter of Interim Control
//!   plus Letter of Compliance from MA Childhood Lead Poisoning Prevention
//!   Program (CLPPP) before re-occupancy.
//!
//! - **California** — Cal. Health & Safety Code § 17920.10 makes lead-
//!   based-paint hazard a substandard housing condition; Cal. Code Regs.
//!   Title 17 Div. 1 Ch. 8 CLPPB (Childhood Lead Poisoning Prevention
//!   Branch) governs.
//!
//! - **New York** — NYC Local Law 1 of 2004 (NYC Admin Code § 27-2056)
//!   plus DOH Lead Regulation Section 11-101 require dust-wipe clearance
//!   testing in all NYC pre-1960 housing with children under 6 PLUS HPD
//!   lead inspection cycle.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    Massachusetts,
    California,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HousingScope {
    /// Pre-1978 target housing — RRP applies.
    Pre1978TargetHousing,
    /// Pre-1960 housing — NYC LL 1 of 2004 trigger.
    Pre1960Housing,
    /// Post-1978 housing — RRP excluded.
    Post1978NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FederalAssistanceStatus {
    /// HUD-assisted (Section 8, PHA, project-based) — Lead Safe Housing Rule
    /// applies.
    HudAssistedLeadSafeHousingRuleApplies,
    /// Private market — RRP only unless state law or contract triggers HUD-
    /// style clearance.
    PrivateMarketRrpOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantClearancePassedAllLocations,
    UncertifiedFirmRrpViolation,
    NoCleaningVerificationViolation,
    NoTenantNotificationPreWorkViolation,
    DustWipeClearanceRequiredButNotPerformed,
    FloorDustExceedsClearanceStandard,
    WindowSillDustExceedsClearanceStandard,
    WindowTroughDustExceedsClearanceStandard,
    TenantReoccupancyBeforeClearancePass,
    ChildPregnancyExposureMaximumLiability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub housing_scope: HousingScope,
    pub federal_assistance: FederalAssistanceStatus,
    pub renovation_year: i32,
    pub interior_disturbed_square_feet: u32,
    pub exterior_disturbed_square_feet: u32,
    pub firm_epa_lead_safe_certified: bool,
    pub certified_renovator_on_site: bool,
    pub renovate_right_pamphlet_provided: bool,
    pub cleaning_verification_card_documented: bool,
    pub dust_wipe_clearance_required_by_state_or_contract: bool,
    pub dust_wipe_clearance_performed: bool,
    pub floor_dust_lead_micrograms_per_sq_ft: u32,
    pub window_sill_dust_lead_micrograms_per_sq_ft: u32,
    pub window_trough_dust_lead_micrograms_per_sq_ft: u32,
    pub child_under_6_or_pregnant_woman_in_unit: bool,
    pub tenant_reoccupied_before_clearance_results: bool,
    pub annual_rent_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub recommended_actions: Vec<String>,
    pub annual_rent_at_risk_cents: u64,
    pub epa_civil_penalty_per_violation_per_day_cents: u64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const RRP_INTERIOR_TRIGGER_SQ_FT: u32 = 6;
pub const RRP_EXTERIOR_TRIGGER_SQ_FT: u32 = 20;
pub const TSCA_PRE_2020_FLOOR_CLEARANCE_UG_PER_SQ_FT: u32 = 40;
pub const TSCA_POST_2020_FLOOR_CLEARANCE_UG_PER_SQ_FT: u32 = 10;
pub const TSCA_PRE_2020_SILL_CLEARANCE_UG_PER_SQ_FT: u32 = 250;
pub const TSCA_POST_2020_SILL_CLEARANCE_UG_PER_SQ_FT: u32 = 100;
pub const TSCA_TROUGH_CLEARANCE_UG_PER_SQ_FT: u32 = 400;
pub const TSCA_POST_2020_CLEARANCE_EFFECTIVE_YEAR: i32 = 2020;
pub const TSCA_2026_ANY_REPORTABLE_HAZARD_EFFECTIVE_YEAR: i32 = 2026;
pub const EPA_CIVIL_PENALTY_PER_VIOLATION_PER_DAY_CENTS: u64 = 5_179_600;

pub fn check(input: &Input) -> Output {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let severity: Severity;

    if matches!(input.housing_scope, HousingScope::Post1978NotApplicable) {
        notes.push(
            "Post-1978 housing — RRP rule and TSCA § 402 dust-lead hazard framework \
             inapplicable. Building constructed after federal residential lead-based paint \
             ban effective January 1, 1978 per 16 C.F.R. § 1303 Consumer Product Safety \
             Commission rule. Confirm via title deed or building permit records before \
             relying on this status."
                .to_string(),
        );
        return Output {
            severity: Severity::NotApplicable,
            recommended_actions: actions,
            annual_rent_at_risk_cents: 0,
            epa_civil_penalty_per_violation_per_day_cents: 0,
            citation: "16 C.F.R. § 1303 (1978 lead paint ban); n/a otherwise",
            notes,
        };
    }

    let triggers_rrp = input.interior_disturbed_square_feet > RRP_INTERIOR_TRIGGER_SQ_FT
        || input.exterior_disturbed_square_feet > RRP_EXTERIOR_TRIGGER_SQ_FT;
    if !triggers_rrp {
        notes.push(format!(
            "Renovation disturbed {} interior sq ft and {} exterior sq ft — below RRP \
             trigger thresholds of {} interior sq ft and {} exterior sq ft per 40 C.F.R. § \
             745.83. RRP exemption applies under 40 C.F.R. § 745.82(a)(2) minor repair / \
             maintenance carve-out. Tenant notification and certification not federally \
             required, but state law may impose lower threshold.",
            input.interior_disturbed_square_feet,
            input.exterior_disturbed_square_feet,
            RRP_INTERIOR_TRIGGER_SQ_FT,
            RRP_EXTERIOR_TRIGGER_SQ_FT
        ));
        return Output {
            severity: Severity::NotApplicable,
            recommended_actions: actions,
            annual_rent_at_risk_cents: 0,
            epa_civil_penalty_per_violation_per_day_cents: 0,
            citation: "40 C.F.R. § 745.82(a)(2) minor repair exemption",
            notes,
        };
    }

    let is_post_2020 =
        input.renovation_year >= TSCA_POST_2020_CLEARANCE_EFFECTIVE_YEAR;
    let floor_clearance_limit = if is_post_2020 {
        TSCA_POST_2020_FLOOR_CLEARANCE_UG_PER_SQ_FT
    } else {
        TSCA_PRE_2020_FLOOR_CLEARANCE_UG_PER_SQ_FT
    };
    let sill_clearance_limit = if is_post_2020 {
        TSCA_POST_2020_SILL_CLEARANCE_UG_PER_SQ_FT
    } else {
        TSCA_PRE_2020_SILL_CLEARANCE_UG_PER_SQ_FT
    };
    let trough_clearance_limit = TSCA_TROUGH_CLEARANCE_UG_PER_SQ_FT;

    let clearance_required = matches!(
        input.federal_assistance,
        FederalAssistanceStatus::HudAssistedLeadSafeHousingRuleApplies
    ) || input.dust_wipe_clearance_required_by_state_or_contract
        || matches!(input.jurisdiction, Jurisdiction::Massachusetts | Jurisdiction::NewYork);

    if input.child_under_6_or_pregnant_woman_in_unit
        && input.tenant_reoccupied_before_clearance_results
    {
        severity = Severity::ChildPregnancyExposureMaximumLiability;
        actions.push(
            "Child under 6 OR pregnant woman re-occupied unit BEFORE clearance results \
             confirmed — maximum landlord liability exposure. CDC reports blood lead level \
             greater than 3.5 μg/dL is reference value triggering case management; lead \
             poisoning litigation routinely exceeds $1M for permanent neurodevelopmental \
             injury. IMMEDIATE temporary relocation under [[mid_tenancy_temporary_\
             relocation]] sibling framework; conduct dust clearance plus blood-lead screening; \
             notify CDC Lead Poisoning Prevention Branch and state CLPPP."
                .to_string(),
        );
    } else if !input.firm_epa_lead_safe_certified {
        severity = Severity::UncertifiedFirmRrpViolation;
        actions.push(format!(
            "Renovation firm NOT EPA Lead-Safe Certified — direct RRP violation under 40 \
             C.F.R. § 745.81(a); civil penalty up to ${} per violation per day per 40 \
             C.F.R. § 19.4 schedule. STOP work immediately; engage EPA Lead-Safe Certified \
             Firm (search at epa.gov/lead/find-certified-lead-firms) before resumption; \
             coordinate post-hoc dust clearance testing.",
            EPA_CIVIL_PENALTY_PER_VIOLATION_PER_DAY_CENTS / 100
        ));
    } else if !input.certified_renovator_on_site {
        severity = Severity::UncertifiedFirmRrpViolation;
        actions.push(
            "Certified renovator NOT on-site — RRP violation under 40 C.F.R. § 745.85(a)(1). \
             Certified renovator must direct work, perform critical tasks (work-area \
             preparation, post-work cleaning verification), and document training of \
             non-certified workers."
                .to_string(),
        );
    } else if !input.renovate_right_pamphlet_provided {
        severity = Severity::NoTenantNotificationPreWorkViolation;
        actions.push(
            "EPA 'Renovate Right' (EPA-740-K-10-001) pamphlet NOT provided to tenant before \
             work commenced — direct violation of 40 C.F.R. § 745.84(a)(1) pre-renovation \
             education requirement. Tenant signature acknowledgment required; provide \
             pamphlet now plus tenant-acknowledgment form retroactively plus document tenant \
             notification timeline."
                .to_string(),
        );
    } else if !input.cleaning_verification_card_documented {
        severity = Severity::NoCleaningVerificationViolation;
        actions.push(
            "Post-renovation cleaning verification card NOT documented — violation of 40 \
             C.F.R. § 745.85(a)(2)(v) plus § 745.85(b). Certified renovator must compare \
             cleaning cloth color to verification card; re-clean and re-verify until clean. \
             Document with photograph plus written verification per § 745.85(b)(2)."
                .to_string(),
        );
    } else if clearance_required && !input.dust_wipe_clearance_performed {
        severity = Severity::DustWipeClearanceRequiredButNotPerformed;
        actions.push(
            "Dust-wipe clearance testing required by HUD Lead Safe Housing Rule (24 C.F.R. \
             Part 35), state law (105 CMR 460 MA, NYC Admin Code § 27-2056), or contract; \
             NOT performed. Engage certified inspector / risk assessor / dust sampling \
             technician per 40 C.F.R. § 745.227 to collect floor, window sill, and window \
             trough wipe samples; submit to NLLAP-recognized laboratory; await pass results \
             before tenant re-occupancy."
                .to_string(),
        );
    } else if input.floor_dust_lead_micrograms_per_sq_ft > floor_clearance_limit {
        severity = Severity::FloorDustExceedsClearanceStandard;
        actions.push(format!(
            "Floor dust-lead level {} μg/ft² exceeds {} μg/ft² clearance standard per 40 \
             C.F.R. § 745.227(e)(8)(viii) (post-{} EPA final rule). Re-clean work area; \
             re-sample after re-cleaning; submit to NLLAP-recognized lab. Effective {} per \
             October 24, 2024 EPA final rule under TSCA §§ 402 plus 403: ANY reportable \
             level constitutes a dust-lead hazard (zero-tolerance regime).",
            input.floor_dust_lead_micrograms_per_sq_ft,
            floor_clearance_limit,
            TSCA_POST_2020_CLEARANCE_EFFECTIVE_YEAR,
            TSCA_2026_ANY_REPORTABLE_HAZARD_EFFECTIVE_YEAR
        ));
    } else if input.window_sill_dust_lead_micrograms_per_sq_ft > sill_clearance_limit {
        severity = Severity::WindowSillDustExceedsClearanceStandard;
        actions.push(format!(
            "Window sill dust-lead level {} μg/ft² exceeds {} μg/ft² clearance standard. \
             Re-clean window sill (typically requires HEPA-vacuum followed by wet-wipe with \
             detergent solution); re-sample after cleaning.",
            input.window_sill_dust_lead_micrograms_per_sq_ft, sill_clearance_limit
        ));
    } else if input.window_trough_dust_lead_micrograms_per_sq_ft > trough_clearance_limit {
        severity = Severity::WindowTroughDustExceedsClearanceStandard;
        actions.push(format!(
            "Window trough dust-lead level {} μg/ft² exceeds {} μg/ft² clearance standard. \
             Window trough often has highest lead concentration in older housing due to \
             friction from window operation; thorough cleaning plus re-sampling required.",
            input.window_trough_dust_lead_micrograms_per_sq_ft, trough_clearance_limit
        ));
    } else if input.tenant_reoccupied_before_clearance_results {
        severity = Severity::TenantReoccupancyBeforeClearancePass;
        actions.push(
            "Tenant re-occupied unit BEFORE clearance pass results confirmed; HUD Lead Safe \
             Housing Rule 24 C.F.R. § 35.1340 PROHIBITS re-occupancy until clearance \
             achieved. Temporarily relocate tenant per [[mid_tenancy_temporary_relocation]] \
             sibling framework until clearance pass documented; document with sampling \
             report plus laboratory analysis plus dust sampling technician certification."
                .to_string(),
        );
    } else {
        severity = Severity::CompliantClearancePassedAllLocations;
        actions.push(format!(
            "Compliant: EPA Lead-Safe Certified Firm, certified renovator on-site, tenant \
             pre-work notification with EPA Renovate Right pamphlet, post-work cleaning \
             verification documented, dust wipe samples passed all three locations (floor \
             {} ≤ {} μg/ft², sill {} ≤ {} μg/ft², trough {} ≤ {} μg/ft²). Retain all \
             documentation for {}-year RRP recordkeeping period per 40 C.F.R. § 745.86(a).",
            input.floor_dust_lead_micrograms_per_sq_ft, floor_clearance_limit,
            input.window_sill_dust_lead_micrograms_per_sq_ft, sill_clearance_limit,
            input.window_trough_dust_lead_micrograms_per_sq_ft, trough_clearance_limit,
            3
        ));
    }

    match input.jurisdiction {
        Jurisdiction::Massachusetts => {
            notes.push(
                "MA 105 CMR 460 Lead Poisoning Prevention and Control is MOST STRINGENT in \
                 US; applies to ALL rental property regardless of federal RRP scope; require \
                 Letter of Interim Control plus Letter of Compliance from MA Childhood Lead \
                 Poisoning Prevention Program (CLPPP). M.G.L. ch. 111 §§ 190-199A statutory \
                 framework. CLPPP private right of action by tenant or parent for triple \
                 damages plus attorney's fees."
                    .to_string(),
            );
        }
        Jurisdiction::California => {
            notes.push(
                "Cal. Health & Safety Code § 17920.10 makes lead-based-paint hazard a \
                 substandard housing condition; Cal. Code Regs. Title 17 Div. 1 Ch. 8 CLPPB \
                 (Childhood Lead Poisoning Prevention Branch). § 1241 California Real \
                 Estate Transfer Disclosure Statement requires lead disclosure on sale; \
                 § 1102.6 commercial lead paint disclosure. CDPH Section 35001 governs."
                    .to_string(),
            );
        }
        Jurisdiction::NewYork => {
            notes.push(
                "NYC Local Law 1 of 2004 (NYC Admin Code § 27-2056) plus DOH Lead Regulation \
                 Section 11-101 require dust-wipe clearance testing in NYC pre-1960 housing \
                 with children under 6 plus annual HPD lead inspection cycle. Local Law 31 \
                 of 2020 requires XRF testing of all rental units within 5 years. NY State \
                 Public Health Law § 1373 + § 1370-a."
                    .to_string(),
            );
        }
        Jurisdiction::Default => {
            notes.push(
                "Federal RRP rule 40 C.F.R. Part 745 Subpart E governs renovation in pre-\
                 1978 target housing exceeding 6 sq ft interior / 20 sq ft exterior \
                 disturbance; HUD Lead Safe Housing Rule 24 C.F.R. Part 35 governs federally-\
                 assisted housing requiring mandatory dust-wipe clearance testing per § \
                 35.1340. TSCA § 402(c) dust-lead clearance levels updated effective \
                 January 6, 2020 (Federal Register June 21, 2019 final rule) AND further \
                 EFFECTIVE January 12, 2026 EPA final rule (October 24, 2024) ANY reportable \
                 level = hazard zero-tolerance regime."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Coordination with [[rental_lead_pipe_disclosure]] (lead-in-drinking-water — \
         different lead exposure pathway plus EPA Lead and Copper Rule), [[mid_tenancy_\
         temporary_relocation]] (when tenant must vacate during cleaning / clearance), \
         [[tenant_emotional_distress_damages]] (IIED claim for child lead exposure), \
         [[carbon_monoxide_detector_compliance]] (parallel children-and-pregnant-women \
         protection framework), [[rental_chimney_fireplace_inspection_disclosure]] (renovation \
         work overlap with chimney repointing pre-1978 lead-paint trim)."
            .to_string(),
    );

    let annual_rent_at_risk: u64 = match severity {
        Severity::ChildPregnancyExposureMaximumLiability => input.annual_rent_cents,
        Severity::UncertifiedFirmRrpViolation
        | Severity::FloorDustExceedsClearanceStandard
        | Severity::WindowSillDustExceedsClearanceStandard
        | Severity::WindowTroughDustExceedsClearanceStandard
        | Severity::TenantReoccupancyBeforeClearancePass => input.annual_rent_cents,
        Severity::DustWipeClearanceRequiredButNotPerformed
        | Severity::NoCleaningVerificationViolation
        | Severity::NoTenantNotificationPreWorkViolation => {
            input.annual_rent_cents.saturating_div(2)
        }
        _ => 0,
    };

    Output {
        severity,
        recommended_actions: actions,
        annual_rent_at_risk_cents: annual_rent_at_risk,
        epa_civil_penalty_per_violation_per_day_cents: EPA_CIVIL_PENALTY_PER_VIOLATION_PER_DAY_CENTS,
        citation: match input.jurisdiction {
            Jurisdiction::Massachusetts => {
                "105 CMR 460 + M.G.L. ch. 111 §§ 190-199A + CLPPP + 40 C.F.R. Part 745 Subpart E"
            }
            Jurisdiction::California => {
                "Cal. Health & Safety Code § 17920.10 + Title 17 Ch. 8 CLPPB + RRP"
            }
            Jurisdiction::NewYork => {
                "NYC Local Law 1 of 2004 + § 27-2056 + DOH § 11-101 + LL 31 of 2020 + PHL § 1373"
            }
            Jurisdiction::Default => {
                "40 C.F.R. Part 745 Subpart E RRP + 24 C.F.R. Part 35 HUD LSHR + TSCA §§ 402 + 403"
            }
        },
        notes,
    }
}

pub type RentalPostConstructionLeadDustClearanceInput = Input;
pub type RentalPostConstructionLeadDustClearanceResult = Output;

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::Massachusetts,
            housing_scope: HousingScope::Pre1978TargetHousing,
            federal_assistance: FederalAssistanceStatus::PrivateMarketRrpOnly,
            renovation_year: 2024,
            interior_disturbed_square_feet: 50,
            exterior_disturbed_square_feet: 0,
            firm_epa_lead_safe_certified: true,
            certified_renovator_on_site: true,
            renovate_right_pamphlet_provided: true,
            cleaning_verification_card_documented: true,
            dust_wipe_clearance_required_by_state_or_contract: true,
            dust_wipe_clearance_performed: true,
            floor_dust_lead_micrograms_per_sq_ft: 5,
            window_sill_dust_lead_micrograms_per_sq_ft: 50,
            window_trough_dust_lead_micrograms_per_sq_ft: 100,
            child_under_6_or_pregnant_woman_in_unit: false,
            tenant_reoccupied_before_clearance_results: false,
            annual_rent_cents: 36_000_00,
        }
    }

    #[test]
    fn post_1978_housing_not_applicable() {
        let mut i = baseline();
        i.housing_scope = HousingScope::Post1978NotApplicable;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
        assert!(r.notes.iter().any(|n| n.contains("16 C.F.R. § 1303")));
    }

    #[test]
    fn below_rrp_threshold_not_applicable() {
        let mut i = baseline();
        i.interior_disturbed_square_feet = 5;
        i.exterior_disturbed_square_feet = 10;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
        assert!(r.notes.iter().any(|n| n.contains("40 C.F.R. § 745.82(a)(2)")));
    }

    #[test]
    fn rrp_trigger_pins_6_interior_sq_ft() {
        assert_eq!(RRP_INTERIOR_TRIGGER_SQ_FT, 6);
    }

    #[test]
    fn rrp_trigger_pins_20_exterior_sq_ft() {
        assert_eq!(RRP_EXTERIOR_TRIGGER_SQ_FT, 20);
    }

    #[test]
    fn post_2020_floor_clearance_pins_10_ug() {
        assert_eq!(TSCA_POST_2020_FLOOR_CLEARANCE_UG_PER_SQ_FT, 10);
    }

    #[test]
    fn post_2020_sill_clearance_pins_100_ug() {
        assert_eq!(TSCA_POST_2020_SILL_CLEARANCE_UG_PER_SQ_FT, 100);
    }

    #[test]
    fn pre_2020_floor_clearance_pins_40_ug() {
        assert_eq!(TSCA_PRE_2020_FLOOR_CLEARANCE_UG_PER_SQ_FT, 40);
    }

    #[test]
    fn pre_2020_sill_clearance_pins_250_ug() {
        assert_eq!(TSCA_PRE_2020_SILL_CLEARANCE_UG_PER_SQ_FT, 250);
    }

    #[test]
    fn trough_clearance_pins_400_ug() {
        assert_eq!(TSCA_TROUGH_CLEARANCE_UG_PER_SQ_FT, 400);
    }

    #[test]
    fn epa_civil_penalty_pins_51796_per_day() {
        assert_eq!(EPA_CIVIL_PENALTY_PER_VIOLATION_PER_DAY_CENTS, 5_179_600);
    }

    #[test]
    fn compliant_clearance_passed_all_locations() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantClearancePassedAllLocations));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 745.86(a)")));
    }

    #[test]
    fn uncertified_firm_rrp_violation_full_rent() {
        let mut i = baseline();
        i.firm_epa_lead_safe_certified = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::UncertifiedFirmRrpViolation));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("40 C.F.R. § 745.81(a)")));
    }

    #[test]
    fn uncertified_renovator_on_site_violation() {
        let mut i = baseline();
        i.certified_renovator_on_site = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::UncertifiedFirmRrpViolation));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("40 C.F.R. § 745.85(a)(1)")));
    }

    #[test]
    fn no_renovate_right_pamphlet_violation_half_rent() {
        let mut i = baseline();
        i.renovate_right_pamphlet_provided = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NoTenantNotificationPreWorkViolation));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("EPA-740-K-10-001")));
    }

    #[test]
    fn no_cleaning_verification_violation_half_rent() {
        let mut i = baseline();
        i.cleaning_verification_card_documented = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NoCleaningVerificationViolation));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("§ 745.85(a)(2)(v)")));
    }

    #[test]
    fn hud_assisted_requires_clearance_violation_half_rent() {
        let mut i = baseline();
        i.federal_assistance = FederalAssistanceStatus::HudAssistedLeadSafeHousingRuleApplies;
        i.dust_wipe_clearance_performed = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::DustWipeClearanceRequiredButNotPerformed));
    }

    #[test]
    fn floor_dust_exceeds_post_2020_10_ug_violation() {
        let mut i = baseline();
        i.renovation_year = 2024;
        i.floor_dust_lead_micrograms_per_sq_ft = 15;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FloorDustExceedsClearanceStandard));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 745.227")));
    }

    #[test]
    fn floor_dust_at_10_ug_compliant() {
        let mut i = baseline();
        i.floor_dust_lead_micrograms_per_sq_ft = 10;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantClearancePassedAllLocations));
    }

    #[test]
    fn floor_dust_pre_2020_40_ug_compliant() {
        let mut i = baseline();
        i.renovation_year = 2019;
        i.floor_dust_lead_micrograms_per_sq_ft = 40;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::CompliantClearancePassedAllLocations));
    }

    #[test]
    fn floor_dust_pre_2020_at_41_ug_violation() {
        let mut i = baseline();
        i.renovation_year = 2019;
        i.floor_dust_lead_micrograms_per_sq_ft = 41;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::FloorDustExceedsClearanceStandard));
    }

    #[test]
    fn window_sill_exceeds_post_2020_100_ug_violation() {
        let mut i = baseline();
        i.window_sill_dust_lead_micrograms_per_sq_ft = 150;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::WindowSillDustExceedsClearanceStandard));
    }

    #[test]
    fn window_trough_exceeds_400_ug_violation() {
        let mut i = baseline();
        i.window_trough_dust_lead_micrograms_per_sq_ft = 450;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::WindowTroughDustExceedsClearanceStandard));
    }

    #[test]
    fn tenant_reoccupied_before_clearance_full_rent_at_risk() {
        let mut i = baseline();
        i.tenant_reoccupied_before_clearance_results = true;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::TenantReoccupancyBeforeClearancePass));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
    }

    #[test]
    fn child_pregnancy_exposure_maximum_liability_full_rent() {
        let mut i = baseline();
        i.child_under_6_or_pregnant_woman_in_unit = true;
        i.tenant_reoccupied_before_clearance_results = true;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::ChildPregnancyExposureMaximumLiability));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("3.5 μg/dL")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("mid_tenancy_temporary_relocation")));
    }

    #[test]
    fn ma_jurisdiction_pins_105_cmr_460_and_mgl_111() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("105 CMR 460")));
        assert!(r.notes.iter().any(|n| n.contains("M.G.L. ch. 111 §§ 190-199A")));
        assert!(r.notes.iter().any(|n| n.contains("CLPPP")));
        assert!(r.notes.iter().any(|n| n.contains("triple damages")));
    }

    #[test]
    fn ca_jurisdiction_pins_17920_10_and_clppb() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::California;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 17920.10")));
        assert!(r.notes.iter().any(|n| n.contains("Title 17 Div. 1 Ch. 8 CLPPB")));
    }

    #[test]
    fn ny_jurisdiction_pins_ll_1_2004_and_ll_31_2020() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Local Law 1 of 2004")));
        assert!(r.notes.iter().any(|n| n.contains("Local Law 31 of 2020")));
        assert!(r.notes.iter().any(|n| n.contains("XRF testing")));
    }

    #[test]
    fn default_jurisdiction_pins_2026_any_reportable_zero_tolerance() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("January 12, 2026")));
        assert!(r.notes.iter().any(|n| n.contains("zero-tolerance")));
        assert!(r.notes.iter().any(|n| n.contains("October 24, 2024")));
    }

    #[test]
    fn coordination_note_references_lead_pipe_relocation_iied() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("rental_lead_pipe_disclosure")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("mid_tenancy_temporary_relocation")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("tenant_emotional_distress_damages")));
    }

    #[test]
    fn coordination_note_pinned_in_all_jurisdictions() {
        for j in [
            Jurisdiction::Massachusetts,
            Jurisdiction::California,
            Jurisdiction::NewYork,
            Jurisdiction::Default,
        ] {
            let mut i = baseline();
            i.jurisdiction = j;
            let r = check(&i);
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("rental_lead_pipe_disclosure")),
                "coordination missing for {j:?}"
            );
        }
    }

    #[test]
    fn severity_priority_child_pregnancy_overrides_clearance_fail() {
        let mut i = baseline();
        i.child_under_6_or_pregnant_woman_in_unit = true;
        i.tenant_reoccupied_before_clearance_results = true;
        i.floor_dust_lead_micrograms_per_sq_ft = 5_000;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::ChildPregnancyExposureMaximumLiability));
    }

    #[test]
    fn severity_priority_uncertified_firm_overrides_pamphlet() {
        let mut i = baseline();
        i.firm_epa_lead_safe_certified = false;
        i.renovate_right_pamphlet_provided = false;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::UncertifiedFirmRrpViolation));
    }

    #[test]
    fn citation_branch_for_each_jurisdiction() {
        let ma = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Massachusetts; i });
        let ca = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::California; i });
        let ny = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::NewYork; i });
        let de = check(&{ let mut i = baseline(); i.jurisdiction = Jurisdiction::Default; i });
        assert!(ma.citation.contains("105 CMR 460"));
        assert!(ca.citation.contains("§ 17920.10"));
        assert!(ny.citation.contains("Local Law 1 of 2004"));
        assert!(de.citation.contains("40 C.F.R. Part 745"));
    }

    #[test]
    fn zero_rent_zero_risk_does_not_panic() {
        let mut i = baseline();
        i.annual_rent_cents = 0;
        i.firm_epa_lead_safe_certified = false;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn epa_civil_penalty_pinned_in_action() {
        let mut i = baseline();
        i.firm_epa_lead_safe_certified = false;
        let r = check(&i);
        assert!(r.recommended_actions.iter().any(|a| a.contains("51796")));
        assert_eq!(
            r.epa_civil_penalty_per_violation_per_day_cents,
            EPA_CIVIL_PENALTY_PER_VIOLATION_PER_DAY_CENTS
        );
    }
}
