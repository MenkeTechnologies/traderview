//! Waste collection + recycling + organic-composting compliance framework for
//! residential rentals.
//!
//! Multifamily landlords face increasingly comprehensive waste-diversion mandates
//! requiring separate streams for trash, recycling, and organic/compostable waste.
//! California pioneered statewide mandatory organic waste recycling with SB 1383;
//! Vermont, Massachusetts, and several Northeast states followed with universal-
//! recycling frameworks. NYC has the most complex municipal regime layering Local
//! Law 87 organic-waste collection + Local Law 199 commercial-waste zones +
//! organics-collection ordinance + Local Law 142 textile recycling.
//!
//! Jurisdictional grid:
//!
//! - CA SB 1383 (Short-Lived Climate Pollutants Reduction Strategy; Cal. Pub. Res.
//!   Code §§ 42652-42653): effective Jan 1, 2022 for collection; Jan 1, 2024 for
//!   penalties. ALL multifamily properties (3+ units; 5+ in some cities) MUST
//!   provide organics cart for tenant food waste + landscaping debris. Tenant
//!   education at move-in + at least 2 weeks pre-move-out + annually. Goal: 75%
//!   reduction in recyclable/compostable disposal by Jan 1, 2025.
//! - CA AB 939 (Integrated Waste Management Act of 1989): 50% diversion mandate
//!   foundational framework.
//! - VT Universal Recycling Act 2012 (Act 148): bans recyclables, leaf and yard
//!   debris, and food scraps from landfill. Food scrap diversion required at all
//!   residential generators (including multifamily) since Jul 1, 2020.
//! - MA Mass. Gen. Laws ch. 21A + 310 CMR 16.00 + commercial organic waste ban
//!   (eff. Oct 1, 2014, threshold lowered Nov 1, 2022 to 0.5 ton/week).
//! - NY LL 87/2009 + NYC RCNY § 16 organic-waste curbside collection +
//!   NYC Mandatory Curbside Composting Law (eff. 2024-2025 staged rollout by
//!   borough): all NYC residential buildings required to separate organic waste.
//! - WA HB 1799 (2023): expands organic-waste-diversion mandate.
//! - OR HB 2065 + CO HB 22-1355 + NJ A4416 (Food Waste Recycling Act): state-
//!   level food-waste-diversion requirements with phased thresholds.
//! - DEFAULT: no statewide organic-waste mandate; local ordinances vary; standard
//!   municipal trash-and-recycling collection only.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - recyclemore.com/business/mandatory-multi-family-organics-waste-and-recycle-law-sb-1383/
//! - recyclesmart.org/sb-1383
//! - wm.com/content/dam/wm/assets/sb1383/preparing-for-california-sb1383.pdf
//! - athensservices.com/sb-1383/

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    CaliforniaSb1383,
    NewYorkCity,
    Vermont,
    Massachusetts,
    Washington,
    Oregon,
    Colorado,
    NewJersey,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    /// Organic-waste bin + recycling bin + trash bin all provided + signage +
    /// tenant education protocol satisfied.
    AllStreamsProvidedWithSignageAndEducation,
    /// Organic-waste bin not provided (recycling + trash provided).
    OrganicWasteBinNotProvidedViolation,
    /// Recycling bin not provided (organics + trash provided).
    RecyclingBinNotProvidedViolation,
    /// Tenant education protocol (move-in + move-out + annual notice) not followed.
    TenantEducationProtocolNotFollowed,
    /// Bins provided but signage missing or contamination not monitored.
    BinsProvidedButSignageOrMonitoringMissing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildingSizeCoverage {
    /// Building covered by jurisdictional mandate (≥ unit threshold).
    CoveredByMandate,
    /// Building below threshold (e.g., 1-2 units in CA SB 1383 + smaller-than-
    /// 5-units in some city versions).
    BelowJurisdictionalThreshold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    BelowJurisdictionalThresholdNoMandate,
    CompliantAllStreamsAndEducationProvided,
    OrganicWasteBinNotProvidedSb1383Violation,
    RecyclingBinNotProvidedViolation,
    TenantEducationProtocolNotFollowed,
    SignageOrContaminationMonitoringMissing,
    DefaultJurisdictionStandardMunicipalCollectionOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub building_size_coverage: BuildingSizeCoverage,
    pub compliance_status: ComplianceStatus,
    pub days_since_violation_notice_received: u32,
}

pub type RentalWasteRecyclingCollectionMandateInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub estimated_civil_penalty_cents: u64,
    pub note: String,
}

pub type RentalWasteRecyclingCollectionMandateOutput = Output;
pub type RentalWasteRecyclingCollectionMandateResult = Output;

const SB_1383_DAILY_PENALTY_FIRST_TIER_CENTS: u64 = 5_000;
const SB_1383_DAILY_PENALTY_REPEAT_TIER_CENTS: u64 = 10_000;
const NYC_LL97_FINE_FIRST_OFFENSE_CENTS: u64 = 10_000;
const VT_ACT_148_PENALTY_BASE_CENTS: u64 = 50_000;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(input.jurisdiction, Jurisdiction::Default) {
        return Output {
            severity: Severity::DefaultJurisdictionStandardMunicipalCollectionOnly,
            estimated_civil_penalty_cents: 0,
            note: "Default jurisdiction without identified statewide waste-recycling-organic \
                   mandate. Standard municipal collection ordinances apply. Many \
                   municipalities (Seattle, Boston, Cambridge, Portland, Berkeley) have \
                   local organic-waste mandates that exceed state law. Confirm local \
                   ordinance for multifamily-property obligations + tenant-education \
                   requirements."
                .to_string(),
        };
    }

    if matches!(
        input.building_size_coverage,
        BuildingSizeCoverage::BelowJurisdictionalThreshold
    ) {
        return Output {
            severity: Severity::BelowJurisdictionalThresholdNoMandate,
            estimated_civil_penalty_cents: 0,
            note: format!(
                "Building below jurisdictional coverage threshold. {} Most mandates exempt \
                 1-2 unit residential property; some city-specific versions apply only at \
                 5+ units. Confirm specific unit-count threshold with local jurisdiction.",
                statute_citation(input.jurisdiction)
            ),
        };
    }

    if matches!(
        input.compliance_status,
        ComplianceStatus::AllStreamsProvidedWithSignageAndEducation
    ) {
        return Output {
            severity: Severity::CompliantAllStreamsAndEducationProvided,
            estimated_civil_penalty_cents: 0,
            note: format!(
                "Compliant: all waste streams (trash + recycling + organic-waste) provided \
                 with signage + tenant education protocol satisfied. {} Document compliance \
                 records: bin-provisioning + tenant-education-notice distribution \
                 (move-in + move-out + annual) + contamination-monitoring log + waste-\
                 hauler invoices.",
                statute_citation(input.jurisdiction)
            ),
        };
    }

    if matches!(
        input.compliance_status,
        ComplianceStatus::OrganicWasteBinNotProvidedViolation
    ) {
        let penalty = jurisdiction_organic_violation_penalty(
            input.jurisdiction,
            input.days_since_violation_notice_received,
        );
        return Output {
            severity: Severity::OrganicWasteBinNotProvidedSb1383Violation,
            estimated_civil_penalty_cents: penalty,
            note: format!(
                "ORGANIC-WASTE-BIN NOT PROVIDED VIOLATION. {} CA SB 1383 jurisdictions \
                 (CalRecycle + local enforcement) escalate from notice-of-violation to \
                 daily-penalty $50/day first tier + $100/day repeat tier. Estimated civil \
                 penalty ${}. Multifamily-3-plus-units coverage rule + tenant-education \
                 distribution (move-in + 2-weeks-pre-move-out + annual) required.",
                statute_citation(input.jurisdiction),
                penalty / 100
            ),
        };
    }

    if matches!(
        input.compliance_status,
        ComplianceStatus::RecyclingBinNotProvidedViolation
    ) {
        let penalty = NYC_LL97_FINE_FIRST_OFFENSE_CENTS;
        return Output {
            severity: Severity::RecyclingBinNotProvidedViolation,
            estimated_civil_penalty_cents: penalty,
            note: format!(
                "RECYCLING BIN NOT PROVIDED VIOLATION. {} Provide separately-labeled \
                 recycling bins for paper + metal + glass + plastic per local ordinance. \
                 Estimated civil penalty ${}.",
                statute_citation(input.jurisdiction),
                penalty / 100
            ),
        };
    }

    if matches!(
        input.compliance_status,
        ComplianceStatus::TenantEducationProtocolNotFollowed
    ) {
        let penalty = SB_1383_DAILY_PENALTY_FIRST_TIER_CENTS.saturating_mul(10);
        return Output {
            severity: Severity::TenantEducationProtocolNotFollowed,
            estimated_civil_penalty_cents: penalty,
            note: format!(
                "TENANT EDUCATION PROTOCOL NOT FOLLOWED. {} CA SB 1383 requires tenant \
                 education at move-in (no later than 2 weeks after) + at least 2 weeks \
                 before move-out + annually. Maintain dated distribution records. \
                 Estimated civil penalty ${}.",
                statute_citation(input.jurisdiction),
                penalty / 100
            ),
        };
    }

    Output {
        severity: Severity::SignageOrContaminationMonitoringMissing,
        estimated_civil_penalty_cents: SB_1383_DAILY_PENALTY_FIRST_TIER_CENTS,
        note: format!(
            "Bins provided but signage missing or contamination not monitored. {} Install \
             multilingual signage in waste-enclosure area + train property staff to monitor \
             contamination + log violations + provide tenant feedback. Estimated civil \
             penalty ${}.",
            statute_citation(input.jurisdiction),
            SB_1383_DAILY_PENALTY_FIRST_TIER_CENTS / 100
        ),
    }
}

fn jurisdiction_organic_violation_penalty(
    jurisdiction: Jurisdiction,
    days_since_violation: u32,
) -> u64 {
    match jurisdiction {
        Jurisdiction::CaliforniaSb1383 => {
            let first_tier = SB_1383_DAILY_PENALTY_FIRST_TIER_CENTS
                .saturating_mul(u64::from(days_since_violation.min(30)));
            let repeat_tier = SB_1383_DAILY_PENALTY_REPEAT_TIER_CENTS
                .saturating_mul(u64::from(days_since_violation.saturating_sub(30)));
            first_tier.saturating_add(repeat_tier)
        }
        Jurisdiction::NewYorkCity => NYC_LL97_FINE_FIRST_OFFENSE_CENTS,
        Jurisdiction::Vermont => VT_ACT_148_PENALTY_BASE_CENTS,
        _ => SB_1383_DAILY_PENALTY_FIRST_TIER_CENTS
            .saturating_mul(u64::from(days_since_violation)),
    }
}

fn statute_citation(jurisdiction: Jurisdiction) -> &'static str {
    match jurisdiction {
        Jurisdiction::CaliforniaSb1383 => {
            "CA SB 1383 (Cal. Pub. Res. Code §§ 42652-42653 + Short-Lived Climate Pollutants \
             Reduction Strategy) — collection mandate effective Jan 1, 2022; penalties \
             effective Jan 1, 2024. Multifamily-3-plus-units (5+ in some cities) coverage. \
             75% diversion goal by Jan 1, 2025. CA AB 939 (1989) foundational diversion \
             framework."
        }
        Jurisdiction::NewYorkCity => {
            "NYC LL 87/2009 + NYC RCNY § 16 + Mandatory Curbside Composting Law (2024-2025 \
             staged rollout by borough) — residential building organic-waste separation \
             requirement. NYC LL 142/2013 textile recycling. NYC LL 199/2017 commercial \
             waste zones."
        }
        Jurisdiction::Vermont => {
            "VT Universal Recycling Act 2012 (Act 148) — bans recyclables + leaf and yard \
             debris + food scraps from landfill. Food scrap diversion required at all \
             residential generators including multifamily since Jul 1, 2020."
        }
        Jurisdiction::Massachusetts => {
            "MA Mass. Gen. Laws ch. 21A + 310 CMR 16.00 + commercial organic waste ban \
             (eff. Oct 1, 2014; threshold lowered Nov 1, 2022 to 0.5 ton/week)."
        }
        Jurisdiction::Washington => {
            "WA HB 1799 (2023) — expands organic-waste-diversion mandate; phased \
             multifamily compliance schedule."
        }
        Jurisdiction::Oregon => {
            "OR HB 2065 — organic-waste diversion + state-level food-waste recycling \
             requirements."
        }
        Jurisdiction::Colorado => {
            "CO HB 22-1355 — Producer Responsibility Program for Statewide Recycling \
             effective 2024+; multifamily property obligations to follow."
        }
        Jurisdiction::NewJersey => {
            "NJ A4416 Food Waste Recycling Act + N.J.A.C. 7:26-2A.13 commercial generators \
             1 ton/week threshold; multifamily-property obligations vary by municipality."
        }
        Jurisdiction::Default => "No statewide waste-recycling-organic mandate identified.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_ca() -> Input {
        Input {
            jurisdiction: Jurisdiction::CaliforniaSb1383,
            building_size_coverage: BuildingSizeCoverage::CoveredByMandate,
            compliance_status:
                ComplianceStatus::AllStreamsProvidedWithSignageAndEducation,
            days_since_violation_notice_received: 0,
        }
    }

    #[test]
    fn default_jurisdiction_no_mandate() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Default;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::DefaultJurisdictionStandardMunicipalCollectionOnly
        );
        assert!(output.note.contains("Seattle"));
    }

    #[test]
    fn below_threshold_no_mandate() {
        let mut input = base_ca();
        input.building_size_coverage = BuildingSizeCoverage::BelowJurisdictionalThreshold;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::BelowJurisdictionalThresholdNoMandate
        );
        assert!(output.note.contains("1-2 unit"));
        assert!(output.note.contains("5+ units"));
    }

    #[test]
    fn ca_compliant_all_streams_with_signage_and_education() {
        let input = base_ca();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantAllStreamsAndEducationProvided
        );
        assert!(output.note.contains("SB 1383"));
        assert!(output.note.contains("§§ 42652-42653"));
    }

    #[test]
    fn ca_organic_waste_bin_not_provided_violation() {
        let mut input = base_ca();
        input.compliance_status =
            ComplianceStatus::OrganicWasteBinNotProvidedViolation;
        input.days_since_violation_notice_received = 10;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::OrganicWasteBinNotProvidedSb1383Violation
        );
        // 10 × $50 = $500
        assert_eq!(output.estimated_civil_penalty_cents, 500_00);
    }

    #[test]
    fn ca_organic_violation_repeat_tier_after_30_days() {
        let mut input = base_ca();
        input.compliance_status =
            ComplianceStatus::OrganicWasteBinNotProvidedViolation;
        input.days_since_violation_notice_received = 60;
        let output = check(&input);
        // First 30 × $50 = $1,500; next 30 × $100 = $3,000; total $4,500
        assert_eq!(output.estimated_civil_penalty_cents, 4_500_00);
    }

    #[test]
    fn ca_recycling_bin_not_provided_violation() {
        let mut input = base_ca();
        input.compliance_status =
            ComplianceStatus::RecyclingBinNotProvidedViolation;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::RecyclingBinNotProvidedViolation
        );
    }

    #[test]
    fn ca_tenant_education_protocol_not_followed() {
        let mut input = base_ca();
        input.compliance_status = ComplianceStatus::TenantEducationProtocolNotFollowed;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::TenantEducationProtocolNotFollowed
        );
        assert!(output.note.contains("move-in"));
        assert!(output.note.contains("2 weeks before move-out"));
    }

    #[test]
    fn ca_bins_provided_but_signage_or_monitoring_missing() {
        let mut input = base_ca();
        input.compliance_status =
            ComplianceStatus::BinsProvidedButSignageOrMonitoringMissing;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::SignageOrContaminationMonitoringMissing
        );
        assert!(output.note.contains("multilingual signage"));
    }

    #[test]
    fn nyc_local_law_87_compliance() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewYorkCity;
        let output = check(&input);
        assert!(output.note.contains("LL 87/2009"));
        assert!(output.note.contains("Mandatory Curbside Composting Law"));
    }

    #[test]
    fn vermont_act_148_compliance() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Vermont;
        let output = check(&input);
        assert!(output.note.contains("Act 148"));
        assert!(output.note.contains("Jul 1, 2020"));
    }

    #[test]
    fn massachusetts_310_cmr_16_compliance() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Massachusetts;
        let output = check(&input);
        assert!(output.note.contains("310 CMR 16.00"));
        assert!(output.note.contains("0.5 ton/week"));
    }

    #[test]
    fn washington_hb_1799_compliance() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Washington;
        let output = check(&input);
        assert!(output.note.contains("HB 1799"));
    }

    #[test]
    fn oregon_hb_2065_compliance() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Oregon;
        let output = check(&input);
        assert!(output.note.contains("HB 2065"));
    }

    #[test]
    fn colorado_hb_22_1355_compliance() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Colorado;
        let output = check(&input);
        assert!(output.note.contains("HB 22-1355"));
    }

    #[test]
    fn new_jersey_food_waste_act_compliance() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewJersey;
        let output = check(&input);
        assert!(output.note.contains("Food Waste Recycling Act"));
        assert!(output.note.contains("N.J.A.C. 7:26-2A.13"));
    }

    #[test]
    fn sb_1383_daily_penalty_first_tier_constant_pins_50() {
        assert_eq!(SB_1383_DAILY_PENALTY_FIRST_TIER_CENTS, 5_000);
    }

    #[test]
    fn sb_1383_daily_penalty_repeat_tier_constant_pins_100() {
        assert_eq!(SB_1383_DAILY_PENALTY_REPEAT_TIER_CENTS, 10_000);
    }

    #[test]
    fn nyc_ll97_first_offense_constant_pins_100() {
        assert_eq!(NYC_LL97_FINE_FIRST_OFFENSE_CENTS, 10_000);
    }

    #[test]
    fn vt_act_148_base_penalty_constant_pins_500() {
        assert_eq!(VT_ACT_148_PENALTY_BASE_CENTS, 50_000);
    }

    #[test]
    fn very_large_days_no_overflow_in_ca_penalty_calc() {
        let mut input = base_ca();
        input.compliance_status =
            ComplianceStatus::OrganicWasteBinNotProvidedViolation;
        input.days_since_violation_notice_received = u32::MAX;
        let output = check(&input);
        // saturating arithmetic prevents panic
        assert!(output.estimated_civil_penalty_cents > 0);
    }

    #[test]
    fn note_pins_75_pct_diversion_goal_2025() {
        let input = base_ca();
        let output = check(&input);
        assert!(output.note.contains("75%"));
        assert!(output.note.contains("Jan 1, 2025"));
    }
}
