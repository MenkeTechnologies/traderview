//! Mold disclosure + remediation compliance framework for residential rentals.
//!
//! Indoor mold poses a serious health hazard (asthma triggers, allergic reactions,
//! respiratory infection) and is subject to disclosure and remediation requirements
//! that vary sharply by jurisdiction. Some states impose explicit statutory disclosure
//! at lease execution; others rely on the implied warranty of habitability or
//! deceptive-trade-practices doctrine. Failure to disclose known visible mold or
//! promptly remediate exposes the landlord to: actual damages (medical bills, lost
//! belongings, relocation costs), statutory civil penalty, lease rescission +
//! constructive eviction, and personal-injury tort liability.
//!
//! Jurisdictional grid:
//!
//! - CA Health & Safety Code §§ 26100-26156 (Toxic Mold Protection Act of 2001) +
//!   CA Civ. Code § 1102.17: written disclosure required if landlord knows or has
//!   reasonable cause to believe mold is present that exceeds permissible exposure
//!   limits OR poses a health threat. CA Civ. Code § 1941.7 (eff. Jan 1, 2022)
//!   requires CDPH mold booklet to all prospective tenants before lease signing.
//! - NYC Local Law 55 of 2018 ("Asthma-Free Housing Act"): NYC HMC § 27-2017.1.
//!   Multi-dwelling property owners must investigate AND remove indoor mold;
//!   licensed-professional remediation required for mold > 10 square feet; annual
//!   inspections; tenant informational materials. 24-hour response window for
//!   reported mold per HPD enforcement guidance.
//! - WA RCW 59.18.060(13): landlord must notify tenants of health hazards associated
//!   with indoor mold exposure + provide WA Department of Health mold-prevention
//!   information at lease execution.
//! - TX Prop. Code § 92.052: diligent-effort-to-repair duty; no explicit mold
//!   disclosure statute but Texas Deceptive Trade Practices Act (DTPA) requires
//!   disclosure of facts that could influence the rental decision (including known
//!   mold infestation).
//! - VA Code § 55.1-1215 + § 55.1-1226 + § 8.01-226.12: landlord must disclose any
//!   visible mold in accessible areas via itemized inspection report within 5 days
//!   of move-in; § 55.1-1220(A)(5) requires landlord to take steps to prevent
//!   moisture accumulation and mold growth.
//! - FL Stat. § 83.51: landlord duty to maintain; no explicit mold statute.
//! - IL: no statewide mold statute; common-law implied warranty of habitability.
//! - DEFAULT: common-law implied warranty of habitability requires landlord to
//!   maintain habitable premises; known mold creates exposure regardless of
//!   statutory framework.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - law.lis.virginia.gov/vacode/title55.1/chapter12/section55.1-1215/
//! - nyc.gov/assets/doh/downloads/pdf/asthma/local-law-55.pdf
//! - app.leg.wa.gov/rcw/default.aspx?cite=59.18.060
//! - moldcompass.com/states/california

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    NewYorkCityLocalLaw55,
    Washington,
    Texas,
    Virginia,
    Florida,
    Illinois,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MoldKnowledgeStatus {
    /// Landlord has actual knowledge of visible mold.
    LandlordActualKnowledgeOfVisibleMold,
    /// Landlord has reasonable cause to believe mold present (water damage history,
    /// musty smell, tenant complaint, prior remediation).
    LandlordConstructiveKnowledgeReasonableCause,
    /// No knowledge identified; mold not present.
    NoKnowledgeOrMoldNotPresent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisclosureStatus {
    /// Written disclosure provided at lease execution per statute.
    WrittenDisclosureProvidedAtLeaseExecution,
    /// CDPH mold booklet provided per CA Civ. Code § 1941.7.
    CaCdphBookletProvidedSection1941_7,
    /// Itemized inspection report with visible-mold disclosure per VA § 55.1-1215.
    VaItemizedInspectionReportProvidedWithinFiveDays,
    /// No disclosure provided despite knowledge.
    NoDisclosureProvidedDespiteKnowledge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemediationStatus {
    /// Mold remediated by licensed professional per NYC LL 55 (over 10 sq ft) or
    /// state professional-licensing standard.
    RemediatedByLicensedProfessional,
    /// Tenant or landlord remediated using consumer products (acceptable under 10
    /// sq ft per NYC LL 55).
    SmallAreaRemediatedByTenantOrLandlordUnderTenSqFt,
    /// Remediation not yet completed; mold ongoing.
    RemediationNotCompletedMoldOngoing,
    /// Remediation attempted but failed; mold returned.
    RemediationAttemptedButFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NoKnowledgeOfMoldNoDisclosureObligation,
    CompliantWrittenDisclosureAndRemediation,
    DisclosureProvidedRemediationOngoing,
    NoDisclosureViolatesStateMoldStatute,
    NycLocalLaw55LicensedProfessionalRequiredViolation,
    UnremediatedMoldImpliedWarrantyHabitabilityBreach,
    PersonalInjuryToxicTortExposureRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub mold_knowledge_status: MoldKnowledgeStatus,
    pub disclosure_status: DisclosureStatus,
    pub remediation_status: RemediationStatus,
    pub estimated_affected_area_square_feet: u32,
    pub tenant_actual_damages_cents: u64,
}

pub type RentalMoldDisclosureRemediationInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub estimated_landlord_exposure_cents: u64,
    pub licensed_remediation_required: bool,
    pub note: String,
}

pub type RentalMoldDisclosureRemediationOutput = Output;
pub type RentalMoldDisclosureRemediationResult = Output;

const NYC_LICENSED_REMEDIATION_THRESHOLD_SQUARE_FEET: u32 = 10;
const TYPICAL_TOXIC_MOLD_TORT_AVERAGE_CENTS: u64 = 5_000_000;
const TYPICAL_STATUTORY_CIVIL_PENALTY_CENTS: u64 = 500_000;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.mold_knowledge_status,
        MoldKnowledgeStatus::NoKnowledgeOrMoldNotPresent
    ) {
        return Output {
            severity: Severity::NoKnowledgeOfMoldNoDisclosureObligation,
            estimated_landlord_exposure_cents: 0,
            licensed_remediation_required: false,
            note: "No actual or constructive knowledge of mold reported — disclosure obligation \
                   not triggered. Landlord should still maintain visual inspections at unit \
                   turnover + after water-damage events; constructive knowledge attaches once \
                   reasonable-cause-to-believe standard is met (water damage history, musty \
                   smell, tenant complaint, prior remediation in same unit)."
                .to_string(),
        };
    }

    let licensed_required = matches!(input.jurisdiction, Jurisdiction::NewYorkCityLocalLaw55)
        && input.estimated_affected_area_square_feet
            > NYC_LICENSED_REMEDIATION_THRESHOLD_SQUARE_FEET;

    if matches!(input.jurisdiction, Jurisdiction::NewYorkCityLocalLaw55)
        && licensed_required
        && !matches!(
            input.remediation_status,
            RemediationStatus::RemediatedByLicensedProfessional
        )
    {
        let exposure = input
            .tenant_actual_damages_cents
            .saturating_add(TYPICAL_STATUTORY_CIVIL_PENALTY_CENTS);
        return Output {
            severity: Severity::NycLocalLaw55LicensedProfessionalRequiredViolation,
            estimated_landlord_exposure_cents: exposure,
            licensed_remediation_required: true,
            note: format!(
                "NYC Local Law 55 VIOLATION. Mold area ({} sq ft) exceeds the 10-square-foot \
                 licensed-professional remediation threshold per NYC HMC § 27-2017.1; \
                 unlicensed remediation prohibited. Estimated exposure ${} = tenant actual \
                 damages (${}) + typical HPD civil penalty (${}) + attorney fees + injunctive \
                 relief. HPD class-C immediately-hazardous violation triggers escalated \
                 enforcement timeline.",
                input.estimated_affected_area_square_feet,
                exposure / 100,
                input.tenant_actual_damages_cents / 100,
                TYPICAL_STATUTORY_CIVIL_PENALTY_CENTS / 100
            ),
        };
    }

    if matches!(
        input.disclosure_status,
        DisclosureStatus::NoDisclosureProvidedDespiteKnowledge
    ) {
        let exposure = input
            .tenant_actual_damages_cents
            .saturating_add(TYPICAL_STATUTORY_CIVIL_PENALTY_CENTS);
        return Output {
            severity: Severity::NoDisclosureViolatesStateMoldStatute,
            estimated_landlord_exposure_cents: exposure,
            licensed_remediation_required: licensed_required,
            note: format!(
                "{} Failure to provide written mold disclosure despite actual or constructive \
                 knowledge is a statutory violation. Estimated exposure ${} = tenant actual \
                 damages (${}) + typical civil penalty (${}) + attorney fees + lease- \
                 rescission remedy + potential personal-injury tort exposure (toxic mold \
                 settlements average ${} per claim).",
                statute_citation(input.jurisdiction),
                exposure / 100,
                input.tenant_actual_damages_cents / 100,
                TYPICAL_STATUTORY_CIVIL_PENALTY_CENTS / 100,
                TYPICAL_TOXIC_MOLD_TORT_AVERAGE_CENTS / 100
            ),
        };
    }

    if matches!(
        input.remediation_status,
        RemediationStatus::RemediationNotCompletedMoldOngoing
            | RemediationStatus::RemediationAttemptedButFailed
    ) {
        let exposure = input
            .tenant_actual_damages_cents
            .saturating_add(TYPICAL_TOXIC_MOLD_TORT_AVERAGE_CENTS / 10);
        return Output {
            severity: Severity::UnremediatedMoldImpliedWarrantyHabitabilityBreach,
            estimated_landlord_exposure_cents: exposure,
            licensed_remediation_required: licensed_required,
            note: format!(
                "Implied warranty of habitability BREACHED. Disclosure provided but \
                 remediation not completed or failed; mold persists. Tenant may invoke \
                 constructive-eviction doctrine + repair-and-deduct + rent-withhold + \
                 lease rescission remedies depending on jurisdiction. Estimated exposure \
                 ${} = tenant actual damages (${}) + partial toxic-mold tort exposure \
                 baseline. Ongoing mold creates personal-injury tort risk; toxic-mold tort \
                 settlements range $50K-$5M+ depending on health impact and proof of \
                 causation.",
                exposure / 100,
                input.tenant_actual_damages_cents / 100
            ),
        };
    }

    Output {
        severity: Severity::CompliantWrittenDisclosureAndRemediation,
        estimated_landlord_exposure_cents: 0,
        licensed_remediation_required: licensed_required,
        note: format!(
            "Compliant: written disclosure + remediation completed. {} Document the \
             disclosure delivery (USPS receipt, tenant signature) and remediation completion \
             (licensed-contractor invoice, post-remediation air-quality test results, \
             warranty). Maintain records for the longer of (a) lease term + statute of \
             limitations or (b) 6 years.",
            statute_citation(input.jurisdiction)
        ),
    }
}

fn statute_citation(jurisdiction: Jurisdiction) -> &'static str {
    match jurisdiction {
        Jurisdiction::California => {
            "California Health & Safety Code §§ 26100-26156 (Toxic Mold Protection Act of \
             2001) + CA Civ. Code § 1102.17 + CA Civ. Code § 1941.7 (CDPH mold booklet)."
        }
        Jurisdiction::NewYorkCityLocalLaw55 => {
            "NYC Local Law 55 of 2018 (Asthma-Free Housing Act) codified at NYC HMC \
             § 27-2017.1; HPD class-C immediately-hazardous violation framework."
        }
        Jurisdiction::Washington => {
            "Washington RCW 59.18.060(13) requires landlord to notify tenants of health \
             hazards associated with indoor mold + provide WA Department of Health \
             mold-prevention information at lease execution."
        }
        Jurisdiction::Texas => {
            "Texas Prop. Code § 92.052 diligent-effort-to-repair duty + Texas Deceptive \
             Trade Practices Act (DTPA) requires disclosure of known mold infestation."
        }
        Jurisdiction::Virginia => {
            "Virginia Code § 55.1-1215 (lease-execution disclosure) + § 55.1-1226 \
             (itemized inspection report within 5 days of move-in) + § 8.01-226.12 \
             (visible-mold duty) + § 55.1-1220(A)(5) (moisture-prevention duty)."
        }
        Jurisdiction::Florida => {
            "Florida Stat. § 83.51 landlord duty to maintain — no explicit mold statute."
        }
        Jurisdiction::Illinois => {
            "Illinois — no statewide mold statute; common-law implied warranty of \
             habitability + Jack Spring v. Little doctrine."
        }
        Jurisdiction::Default => {
            "Common-law implied warranty of habitability requires landlord to maintain \
             habitable premises; known mold creates exposure regardless of statutory \
             framework."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_ca() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            mold_knowledge_status: MoldKnowledgeStatus::LandlordActualKnowledgeOfVisibleMold,
            disclosure_status: DisclosureStatus::WrittenDisclosureProvidedAtLeaseExecution,
            remediation_status: RemediationStatus::RemediatedByLicensedProfessional,
            estimated_affected_area_square_feet: 5,
            tenant_actual_damages_cents: 5_000_00,
        }
    }

    #[test]
    fn no_knowledge_of_mold_no_disclosure_obligation() {
        let mut input = base_ca();
        input.mold_knowledge_status = MoldKnowledgeStatus::NoKnowledgeOrMoldNotPresent;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoKnowledgeOfMoldNoDisclosureObligation
        );
        assert_eq!(output.estimated_landlord_exposure_cents, 0);
    }

    #[test]
    fn california_compliant_written_disclosure_and_remediation() {
        let input = base_ca();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantWrittenDisclosureAndRemediation
        );
        assert!(output.note.contains("Toxic Mold Protection Act"));
        assert!(output.note.contains("§ 1941.7"));
    }

    #[test]
    fn california_no_disclosure_violates_statute() {
        let mut input = base_ca();
        input.disclosure_status = DisclosureStatus::NoDisclosureProvidedDespiteKnowledge;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoDisclosureViolatesStateMoldStatute
        );
        assert!(output.note.contains("Toxic Mold Protection Act"));
    }

    #[test]
    fn nyc_local_law_55_under_10_sq_ft_no_licensed_required() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewYorkCityLocalLaw55;
        input.estimated_affected_area_square_feet = 5;
        let output = check(&input);
        assert!(!output.licensed_remediation_required);
        assert!(output.note.contains("Asthma-Free Housing Act"));
    }

    #[test]
    fn nyc_local_law_55_over_10_sq_ft_licensed_required() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewYorkCityLocalLaw55;
        input.estimated_affected_area_square_feet = 20;
        input.remediation_status = RemediationStatus::RemediatedByLicensedProfessional;
        let output = check(&input);
        assert!(output.licensed_remediation_required);
        assert_eq!(
            output.severity,
            Severity::CompliantWrittenDisclosureAndRemediation
        );
    }

    #[test]
    fn nyc_unlicensed_remediation_over_10_sq_ft_violation() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewYorkCityLocalLaw55;
        input.estimated_affected_area_square_feet = 20;
        input.remediation_status =
            RemediationStatus::SmallAreaRemediatedByTenantOrLandlordUnderTenSqFt;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NycLocalLaw55LicensedProfessionalRequiredViolation
        );
        assert!(output.licensed_remediation_required);
        assert!(output.note.contains("§ 27-2017.1"));
        assert!(output.note.contains("HPD"));
    }

    #[test]
    fn washington_compliant_disclosure() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Washington;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantWrittenDisclosureAndRemediation
        );
        assert!(output.note.contains("RCW 59.18.060(13)"));
    }

    #[test]
    fn texas_no_disclosure_dtpa_violation() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Texas;
        input.disclosure_status = DisclosureStatus::NoDisclosureProvidedDespiteKnowledge;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoDisclosureViolatesStateMoldStatute
        );
        assert!(output.note.contains("DTPA"));
        assert!(output.note.contains("§ 92.052"));
    }

    #[test]
    fn virginia_no_disclosure_violation_pins_55_1_1215() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Virginia;
        input.disclosure_status = DisclosureStatus::NoDisclosureProvidedDespiteKnowledge;
        let output = check(&input);
        assert!(output.note.contains("§ 55.1-1215"));
        assert!(output.note.contains("§ 55.1-1226"));
    }

    #[test]
    fn unremediated_mold_implied_warranty_breach() {
        let mut input = base_ca();
        input.remediation_status = RemediationStatus::RemediationNotCompletedMoldOngoing;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::UnremediatedMoldImpliedWarrantyHabitabilityBreach
        );
        assert!(output.note.contains("constructive-eviction"));
        assert!(output.note.contains("repair-and-deduct"));
    }

    #[test]
    fn failed_remediation_implied_warranty_breach() {
        let mut input = base_ca();
        input.remediation_status = RemediationStatus::RemediationAttemptedButFailed;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::UnremediatedMoldImpliedWarrantyHabitabilityBreach
        );
    }

    #[test]
    fn constructive_knowledge_triggers_disclosure_obligation() {
        let mut input = base_ca();
        input.mold_knowledge_status =
            MoldKnowledgeStatus::LandlordConstructiveKnowledgeReasonableCause;
        input.disclosure_status = DisclosureStatus::NoDisclosureProvidedDespiteKnowledge;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoDisclosureViolatesStateMoldStatute
        );
    }

    #[test]
    fn illinois_default_common_law_warranty() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Illinois;
        let output = check(&input);
        assert!(output.note.contains("Jack Spring"));
    }

    #[test]
    fn florida_default_section_83_51_landlord_duty() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Florida;
        let output = check(&input);
        assert!(output.note.contains("§ 83.51"));
    }

    #[test]
    fn default_jurisdiction_common_law_warranty() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Default;
        let output = check(&input);
        assert!(output.note.contains("implied warranty"));
    }

    #[test]
    fn nyc_licensed_threshold_constant_pins_10_sq_ft() {
        assert_eq!(NYC_LICENSED_REMEDIATION_THRESHOLD_SQUARE_FEET, 10);
    }

    #[test]
    fn typical_toxic_mold_tort_average_constant_pins_50000() {
        assert_eq!(TYPICAL_TOXIC_MOLD_TORT_AVERAGE_CENTS, 5_000_000);
    }

    #[test]
    fn typical_civil_penalty_constant_pins_5000() {
        assert_eq!(TYPICAL_STATUTORY_CIVIL_PENALTY_CENTS, 500_000);
    }

    #[test]
    fn very_large_damages_no_overflow() {
        let mut input = base_ca();
        input.disclosure_status = DisclosureStatus::NoDisclosureProvidedDespiteKnowledge;
        input.tenant_actual_damages_cents = u64::MAX;
        let output = check(&input);
        // saturating_add prevents overflow
        assert_eq!(output.estimated_landlord_exposure_cents, u64::MAX);
    }

    #[test]
    fn zero_damages_uses_baseline_civil_penalty() {
        let mut input = base_ca();
        input.disclosure_status = DisclosureStatus::NoDisclosureProvidedDespiteKnowledge;
        input.tenant_actual_damages_cents = 0;
        let output = check(&input);
        // Baseline = $5K civil penalty
        assert_eq!(output.estimated_landlord_exposure_cents, 500_000);
    }

    #[test]
    fn nyc_boundary_exactly_10_sq_ft_no_licensed_required() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewYorkCityLocalLaw55;
        input.estimated_affected_area_square_feet = 10;
        let output = check(&input);
        // > 10 triggers licensed; exactly 10 does not
        assert!(!output.licensed_remediation_required);
    }

    #[test]
    fn nyc_boundary_11_sq_ft_licensed_required() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewYorkCityLocalLaw55;
        input.estimated_affected_area_square_feet = 11;
        input.remediation_status = RemediationStatus::RemediatedByLicensedProfessional;
        let output = check(&input);
        assert!(output.licensed_remediation_required);
    }
}
