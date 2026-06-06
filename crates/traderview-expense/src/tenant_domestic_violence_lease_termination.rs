//! Tenant domestic violence (DV) early lease termination
//! compliance — when a tenant or household member is a
//! victim of domestic violence, sexual assault, stalking, or
//! human trafficking and seeks to terminate the rental
//! agreement early without penalty. Trader-landlord
//! operational concern: an improperly handled DV
//! termination request creates anti-discrimination liability
//! (federal VAWA), state-law statutory damages, and
//! per-incident retaliation claims. Distinct from siblings
//! `tenant_accessible_parking` (ADA), `rental_application_
//! denial_disclosure` (screening), `rental_bed_bug_
//! disclosure` (lease disclosure).
//!
//! **Four regimes**:
//!
//! **Federal — Violence Against Women Act Reauthorization
//! Act of 2022 (VAWA 2022, 34 USC § 12491 + 24 CFR § 5.2005)**:
//! - Applies to all HUD-covered housing programs (Section
//!   8, public housing, LIHTC under § 42, HOME, Section
//!   202/811).
//! - Landlord SHALL NOT terminate or refuse to renew a
//!   lease solely because tenant is DV victim.
//! - Form HUD-91066 self-certification accepted as
//!   documentation; landlord MAY request within 14 business
//!   days.
//! - Lease provisions purporting to terminate on police
//!   calls in DV situations are VOID.
//! - Lease provisions requiring waiver of VAWA rights are
//!   VOID.
//! - VAWA emergency transfer plan required.
//!
//! **California — Cal. Civ. Code § 1946.7**:
//! - Tenant may terminate lease 14 days after written
//!   notice.
//! - Acceptable documentation (must be issued within prior
//!   180 days): (1) restraining or protective order; (2)
//!   police report; (3) qualified third-party statement
//!   from doctor, nurse, therapist, counselor, or caseworker.
//! - Tenant liable for rent only up to 14 calendar days
//!   after notice.
//! - Documentation confidential — landlord may disclose
//!   only with tenant written consent OR if required by law
//!   or court order.
//! - Landlord retaliation prohibited.
//!
//! **Illinois — Safe Homes Act (765 ILCS 750)**:
//! - § 750/15(a)(1) — survivor may terminate if DV
//!   occurred at the leased premises.
//! - Written notice of termination required.
//! - Documentation: protective order OR qualified third-
//!   party statement.
//! - Eviction defense — landlord cannot evict based on
//!   tenant or household member being DV victim.
//! - Confidentiality of documentation.
//!
//! **Washington — RCW 59.18.575**:
//! - Termination request must be made within **90 days** of
//!   the reported DV act, event, or circumstance giving
//!   rise to a protective order or report to qualified
//!   third party.
//! - Tenant remains liable for rent for the month in which
//!   termination occurs.
//! - Discharged from rent for any period after the last
//!   day of the month of the quitting date.
//! - Documentation: protective order OR qualified third-
//!   party report.
//!
//! Citations: 34 USC § 12491 + 24 CFR § 5.2005 (VAWA 2022);
//! Form HUD-91066; Cal. Civ. Code § 1946.7; 765 ILCS 750
//! (IL Safe Homes Act); RCW 59.18.575 (Washington Victim
//! Protection).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    /// Federal VAWA Reauthorization Act of 2022 (HUD-covered
    /// housing only).
    FederalVawa,
    California,
    Illinois,
    Washington,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentationType {
    /// Restraining or protective order.
    ProtectiveOrder,
    /// Police report.
    PoliceReport,
    /// Qualified third-party statement (doctor, nurse,
    /// therapist, counselor, caseworker).
    QualifiedThirdPartyStatement,
    /// Form HUD-91066 self-certification (federal VAWA only).
    HudForm91066SelfCertification,
    /// No documentation provided.
    None,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantDomesticViolenceLeaseTerminationInput {
    pub regime: Regime,
    /// Whether tenant or household member is a victim of DV,
    /// sexual assault, stalking, or human trafficking.
    pub victim_status: bool,
    /// Whether written notice of termination was provided.
    pub written_notice_provided: bool,
    /// Days between written notice and proposed move-out
    /// date.
    pub days_between_notice_and_moveout: u32,
    /// Days between DV incident and termination request
    /// (Washington 90-day lookback).
    pub days_since_dv_incident: u32,
    /// Days since documentation was issued (California 180-
    /// day lookback).
    pub days_since_documentation_issued: u32,
    /// Documentation type provided.
    pub documentation_type: DocumentationType,
    /// Whether dwelling is HUD-covered housing (federal VAWA
    /// applicability).
    pub hud_covered_housing: bool,
    /// Whether lease contains void clauses (police-call
    /// termination, VAWA-rights waiver).
    pub lease_contains_void_clauses: bool,
    /// Whether landlord disclosed tenant documentation
    /// without consent.
    pub landlord_disclosed_documentation_without_consent: bool,
    /// Whether landlord retaliated (eviction, refusal to
    /// renew based on victim status).
    pub landlord_retaliated: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TenantDomesticViolenceLeaseTerminationResult {
    pub termination_request_compliant: bool,
    pub notice_period_compliant: bool,
    pub documentation_compliant: bool,
    pub documentation_lookback_compliant: bool,
    pub federal_vawa_engaged: bool,
    pub confidentiality_violated: bool,
    pub retaliation_violation: bool,
    pub void_clauses_engaged: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &TenantDomesticViolenceLeaseTerminationInput,
) -> TenantDomesticViolenceLeaseTerminationResult {
    match input.regime {
        Regime::FederalVawa => check_vawa(input),
        Regime::California => check_ca(input),
        Regime::Illinois => check_il(input),
        Regime::Washington => check_wa(input),
    }
}

fn check_vawa(
    input: &TenantDomesticViolenceLeaseTerminationInput,
) -> TenantDomesticViolenceLeaseTerminationResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "34 USC § 12491 (VAWA Reauthorization Act of 2022) + 24 CFR § 5.2005 — applies to all HUD-covered housing programs (Section 8, public housing, LIHTC § 42, HOME, Section 202/811)".to_string(),
        "VAWA 2022 — landlord SHALL NOT terminate or refuse to renew lease solely because tenant is DV victim".to_string(),
        "Form HUD-91066 self-certification accepted as documentation; landlord MAY request within 14 business days".to_string(),
        "VAWA 2022 — lease provisions terminating on police calls in DV situations are VOID".to_string(),
        "VAWA 2022 — lease provisions requiring waiver of VAWA rights are VOID; emergency transfer plan required".to_string(),
    ];

    if input.hud_covered_housing && input.lease_contains_void_clauses {
        violations.push(
            "34 USC § 12491 + 24 CFR § 5.2005 — lease provisions terminating on police calls in DV situations or requiring waiver of VAWA rights are VOID and unenforceable".to_string(),
        );
    }

    if input.victim_status && input.landlord_retaliated {
        violations.push(
            "34 USC § 12491 — landlord shall not terminate or refuse to renew lease solely because tenant is DV victim".to_string(),
        );
    }

    if input.hud_covered_housing
        && input.victim_status
        && matches!(input.documentation_type, DocumentationType::None)
        && !input.written_notice_provided
    {
        violations.push(
            "34 USC § 12491 — Form HUD-91066 self-certification or other acceptable documentation should be provided within 14 business days of landlord request".to_string(),
        );
    }

    TenantDomesticViolenceLeaseTerminationResult {
        termination_request_compliant: violations.is_empty(),
        notice_period_compliant: true,
        documentation_compliant: !matches!(input.documentation_type, DocumentationType::None)
            || !input.victim_status,
        documentation_lookback_compliant: true,
        federal_vawa_engaged: input.hud_covered_housing,
        confidentiality_violated: input.landlord_disclosed_documentation_without_consent,
        retaliation_violation: input.landlord_retaliated,
        void_clauses_engaged: input.lease_contains_void_clauses,
        violations,
        citation:
            "34 USC § 12491 (VAWA Reauthorization Act of 2022); 24 CFR § 5.2005; Form HUD-91066",
        notes,
    }
}

fn check_ca(
    input: &TenantDomesticViolenceLeaseTerminationInput,
) -> TenantDomesticViolenceLeaseTerminationResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Civ. Code § 1946.7 — tenant or household member victim of DV, sexual assault, stalking, human trafficking, elder abuse, or other serious crimes causing bodily injury may terminate lease 14 days after written notice".to_string(),
        "Cal. Civ. Code § 1946.7 — acceptable documentation issued within prior 180 days: (1) restraining or protective order; (2) police report; (3) qualified third-party statement from doctor, nurse, therapist, counselor, or caseworker".to_string(),
        "Cal. Civ. Code § 1946.7 — tenant liable for rent only up to 14 calendar days after notice; if landlord re-rents sooner, rent prorated".to_string(),
        "Cal. Civ. Code § 1946.7 — documentation confidential; landlord may disclose only with tenant written consent OR if required by law or court order".to_string(),
        "Cal. Civ. Code § 1946.7 — landlord retaliation prohibited; eviction based on tenant exercising § 1946.7 rights is unlawful retaliation".to_string(),
    ];

    if input.victim_status && !input.written_notice_provided {
        violations
            .push("Cal. Civ. Code § 1946.7 — written notice of termination required".to_string());
    }

    let notice_compliant = input.days_between_notice_and_moveout >= 14;
    if input.victim_status && input.written_notice_provided && !notice_compliant {
        violations.push(
            "Cal. Civ. Code § 1946.7 — tenant must select move-out date at least 14 days from delivery of notice".to_string(),
        );
    }

    let doc_acceptable = matches!(
        input.documentation_type,
        DocumentationType::ProtectiveOrder
            | DocumentationType::PoliceReport
            | DocumentationType::QualifiedThirdPartyStatement
    );

    if input.victim_status && !doc_acceptable {
        violations.push(
            "Cal. Civ. Code § 1946.7 — documentation required: restraining/protective order, police report, or qualified third-party statement".to_string(),
        );
    }

    let lookback_compliant = !doc_acceptable || input.days_since_documentation_issued <= 180;
    if input.victim_status && doc_acceptable && !lookback_compliant {
        violations.push(
            "Cal. Civ. Code § 1946.7 — documentation must be issued within prior 180 days"
                .to_string(),
        );
    }

    if input.landlord_disclosed_documentation_without_consent {
        violations.push(
            "Cal. Civ. Code § 1946.7 — documentation confidential; landlord disclosure without tenant consent or court order prohibited".to_string(),
        );
    }

    if input.landlord_retaliated {
        violations.push(
            "Cal. Civ. Code § 1946.7 — landlord retaliation for tenant exercise of § 1946.7 rights prohibited".to_string(),
        );
    }

    TenantDomesticViolenceLeaseTerminationResult {
        termination_request_compliant: violations.is_empty(),
        notice_period_compliant: notice_compliant,
        documentation_compliant: doc_acceptable || !input.victim_status,
        documentation_lookback_compliant: lookback_compliant,
        federal_vawa_engaged: false,
        confidentiality_violated: input.landlord_disclosed_documentation_without_consent,
        retaliation_violation: input.landlord_retaliated,
        void_clauses_engaged: false,
        violations,
        citation: "Cal. Civ. Code § 1946.7",
        notes,
    }
}

fn check_il(
    input: &TenantDomesticViolenceLeaseTerminationInput,
) -> TenantDomesticViolenceLeaseTerminationResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "765 ILCS 750 (Illinois Safe Homes Act) § 750/15(a)(1) — survivor may terminate lease if DV occurred at the leased premises; written notice required".to_string(),
        "765 ILCS 750/15 — acceptable documentation: protective order or qualified third-party statement".to_string(),
        "765 ILCS 750/30 — landlord cannot evict based on tenant or household member being DV victim; Safe Homes Act provides eviction defense".to_string(),
        "765 ILCS 750/25 — confidentiality of documentation; landlord may not disclose without tenant consent".to_string(),
        "Illinois Safe Homes Act named to protect survivors of domestic violence by allowing safe housing-relocation pathway".to_string(),
    ];

    if input.victim_status && !input.written_notice_provided {
        violations
            .push("765 ILCS 750/15(a)(1) — written notice of termination required".to_string());
    }

    let doc_acceptable = matches!(
        input.documentation_type,
        DocumentationType::ProtectiveOrder | DocumentationType::QualifiedThirdPartyStatement
    );

    if input.victim_status && !doc_acceptable {
        violations.push(
            "765 ILCS 750/15 — acceptable documentation: protective order or qualified third-party statement".to_string(),
        );
    }

    if input.landlord_disclosed_documentation_without_consent {
        violations.push(
            "765 ILCS 750/25 — documentation confidential; landlord disclosure without tenant consent prohibited".to_string(),
        );
    }

    if input.landlord_retaliated {
        violations.push(
            "765 ILCS 750/30 — landlord cannot evict or retaliate based on tenant or household member being DV victim".to_string(),
        );
    }

    TenantDomesticViolenceLeaseTerminationResult {
        termination_request_compliant: violations.is_empty(),
        notice_period_compliant: true,
        documentation_compliant: doc_acceptable || !input.victim_status,
        documentation_lookback_compliant: true,
        federal_vawa_engaged: false,
        confidentiality_violated: input.landlord_disclosed_documentation_without_consent,
        retaliation_violation: input.landlord_retaliated,
        void_clauses_engaged: false,
        violations,
        citation: "765 ILCS 750 (Illinois Safe Homes Act)",
        notes,
    }
}

fn check_wa(
    input: &TenantDomesticViolenceLeaseTerminationInput,
) -> TenantDomesticViolenceLeaseTerminationResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "RCW 59.18.575 (Washington Residential Landlord-Tenant Act, Victim Protection) — tenant may terminate rental agreement within 90 days of reported DV act, event, or circumstance giving rise to protective order or qualified third-party report".to_string(),
        "RCW 59.18.575 — tenant remains liable for rent for month in which termination occurs; discharged from rent for any period following last day of month of quitting date".to_string(),
        "RCW 59.18.575 — acceptable documentation: protective order or qualified third-party report".to_string(),
        "RCW 59.18.575 — written notice to landlord required".to_string(),
        "RCW 59.18.575 — confidentiality of documentation; landlord disclosure prohibited without consent".to_string(),
    ];

    if input.victim_status && !input.written_notice_provided {
        violations.push("RCW 59.18.575 — written notice to landlord required".to_string());
    }

    let within_90_days = input.days_since_dv_incident <= 90;
    if input.victim_status && !within_90_days {
        violations.push(
            "RCW 59.18.575 — termination request must be made within 90 days of reported DV act, event, or circumstance".to_string(),
        );
    }

    let doc_acceptable = matches!(
        input.documentation_type,
        DocumentationType::ProtectiveOrder | DocumentationType::QualifiedThirdPartyStatement
    );

    if input.victim_status && !doc_acceptable {
        violations.push(
            "RCW 59.18.575 — acceptable documentation: protective order or qualified third-party report".to_string(),
        );
    }

    if input.landlord_disclosed_documentation_without_consent {
        violations.push(
            "RCW 59.18.575 — documentation confidential; landlord disclosure without consent prohibited".to_string(),
        );
    }

    if input.landlord_retaliated {
        violations.push(
            "RCW 59.18.575 — landlord cannot retaliate against tenant exercising § 59.18.575 rights".to_string(),
        );
    }

    TenantDomesticViolenceLeaseTerminationResult {
        termination_request_compliant: violations.is_empty(),
        notice_period_compliant: true,
        documentation_compliant: doc_acceptable || !input.victim_status,
        documentation_lookback_compliant: within_90_days,
        federal_vawa_engaged: false,
        confidentiality_violated: input.landlord_disclosed_documentation_without_consent,
        retaliation_violation: input.landlord_retaliated,
        void_clauses_engaged: false,
        violations,
        citation: "RCW 59.18.575",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_clean() -> TenantDomesticViolenceLeaseTerminationInput {
        TenantDomesticViolenceLeaseTerminationInput {
            regime: Regime::California,
            victim_status: true,
            written_notice_provided: true,
            days_between_notice_and_moveout: 14,
            days_since_dv_incident: 30,
            days_since_documentation_issued: 60,
            documentation_type: DocumentationType::ProtectiveOrder,
            hud_covered_housing: false,
            lease_contains_void_clauses: false,
            landlord_disclosed_documentation_without_consent: false,
            landlord_retaliated: false,
        }
    }

    fn il_clean() -> TenantDomesticViolenceLeaseTerminationInput {
        let mut i = ca_clean();
        i.regime = Regime::Illinois;
        i
    }

    fn wa_clean() -> TenantDomesticViolenceLeaseTerminationInput {
        let mut i = ca_clean();
        i.regime = Regime::Washington;
        i
    }

    fn vawa_clean() -> TenantDomesticViolenceLeaseTerminationInput {
        let mut i = ca_clean();
        i.regime = Regime::FederalVawa;
        i.hud_covered_housing = true;
        i.documentation_type = DocumentationType::HudForm91066SelfCertification;
        i
    }

    #[test]
    fn ca_clean_termination_compliant() {
        let r = check(&ca_clean());
        assert!(r.termination_request_compliant);
        assert!(r.notice_period_compliant);
        assert!(r.documentation_compliant);
    }

    #[test]
    fn ca_no_written_notice_violation() {
        let mut i = ca_clean();
        i.written_notice_provided = false;
        let r = check(&i);
        assert!(!r.termination_request_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1946.7") && v.contains("written notice")));
    }

    #[test]
    fn ca_14_day_notice_boundary_compliant() {
        let mut i = ca_clean();
        i.days_between_notice_and_moveout = 14;
        let r = check(&i);
        assert!(r.notice_period_compliant);
    }

    #[test]
    fn ca_13_day_notice_violation() {
        let mut i = ca_clean();
        i.days_between_notice_and_moveout = 13;
        let r = check(&i);
        assert!(!r.notice_period_compliant);
        assert!(r.violations.iter().any(|v| v.contains("14 days")));
    }

    #[test]
    fn ca_180_day_documentation_boundary_compliant() {
        let mut i = ca_clean();
        i.days_since_documentation_issued = 180;
        let r = check(&i);
        assert!(r.documentation_lookback_compliant);
        assert!(r.termination_request_compliant);
    }

    #[test]
    fn ca_181_day_documentation_violation() {
        let mut i = ca_clean();
        i.days_since_documentation_issued = 181;
        let r = check(&i);
        assert!(!r.documentation_lookback_compliant);
        assert!(r.violations.iter().any(|v| v.contains("180 days")));
    }

    #[test]
    fn ca_police_report_acceptable() {
        let mut i = ca_clean();
        i.documentation_type = DocumentationType::PoliceReport;
        let r = check(&i);
        assert!(r.documentation_compliant);
    }

    #[test]
    fn ca_qualified_third_party_acceptable() {
        let mut i = ca_clean();
        i.documentation_type = DocumentationType::QualifiedThirdPartyStatement;
        let r = check(&i);
        assert!(r.documentation_compliant);
    }

    #[test]
    fn ca_no_documentation_violation() {
        let mut i = ca_clean();
        i.documentation_type = DocumentationType::None;
        let r = check(&i);
        assert!(!r.documentation_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("restraining/protective order") && v.contains("police report")));
    }

    #[test]
    fn ca_landlord_disclosure_without_consent_violation() {
        let mut i = ca_clean();
        i.landlord_disclosed_documentation_without_consent = true;
        let r = check(&i);
        assert!(r.confidentiality_violated);
        assert!(r.violations.iter().any(|v| v.contains("confidential")));
    }

    #[test]
    fn ca_landlord_retaliation_violation() {
        let mut i = ca_clean();
        i.landlord_retaliated = true;
        let r = check(&i);
        assert!(r.retaliation_violation);
        assert!(r.violations.iter().any(|v| v.contains("retaliation")));
    }

    #[test]
    fn il_clean_termination_compliant() {
        let r = check(&il_clean());
        assert!(r.termination_request_compliant);
    }

    #[test]
    fn il_no_documentation_violation() {
        let mut i = il_clean();
        i.documentation_type = DocumentationType::None;
        let r = check(&i);
        assert!(!r.documentation_compliant);
        assert!(r.violations.iter().any(|v| v.contains("765 ILCS 750/15")));
    }

    #[test]
    fn il_police_report_not_acceptable_documentation() {
        let mut i = il_clean();
        i.documentation_type = DocumentationType::PoliceReport;
        let r = check(&i);
        assert!(!r.documentation_compliant);
    }

    #[test]
    fn il_landlord_retaliation_violation() {
        let mut i = il_clean();
        i.landlord_retaliated = true;
        let r = check(&i);
        assert!(r.retaliation_violation);
        assert!(r.violations.iter().any(|v| v.contains("765 ILCS 750/30")));
    }

    #[test]
    fn wa_clean_termination_compliant() {
        let r = check(&wa_clean());
        assert!(r.termination_request_compliant);
    }

    #[test]
    fn wa_within_90_day_compliant() {
        let mut i = wa_clean();
        i.days_since_dv_incident = 90;
        let r = check(&i);
        assert!(r.documentation_lookback_compliant);
        assert!(r.termination_request_compliant);
    }

    #[test]
    fn wa_91_day_violation() {
        let mut i = wa_clean();
        i.days_since_dv_incident = 91;
        let r = check(&i);
        assert!(!r.documentation_lookback_compliant);
        assert!(r.violations.iter().any(|v| v.contains("90 days")));
    }

    #[test]
    fn vawa_clean_termination_compliant() {
        let r = check(&vawa_clean());
        assert!(r.termination_request_compliant);
        assert!(r.federal_vawa_engaged);
    }

    #[test]
    fn vawa_lease_void_clauses_violation() {
        let mut i = vawa_clean();
        i.lease_contains_void_clauses = true;
        let r = check(&i);
        assert!(r.void_clauses_engaged);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("VOID") && v.contains("police calls")));
    }

    #[test]
    fn vawa_landlord_retaliation_violation() {
        let mut i = vawa_clean();
        i.landlord_retaliated = true;
        let r = check(&i);
        assert!(r.retaliation_violation);
        assert!(r.violations.iter().any(|v| v.contains("34 USC § 12491")));
    }

    #[test]
    fn vawa_hud_form_91066_accepted() {
        let mut i = vawa_clean();
        i.documentation_type = DocumentationType::HudForm91066SelfCertification;
        let r = check(&i);
        assert!(r.documentation_compliant);
    }

    #[test]
    fn vawa_not_hud_covered_no_federal_engagement() {
        let mut i = vawa_clean();
        i.hud_covered_housing = false;
        let r = check(&i);
        assert!(!r.federal_vawa_engaged);
    }

    #[test]
    fn citation_pins_ca_authority() {
        let r = check(&ca_clean());
        assert!(r.citation.contains("§ 1946.7"));
    }

    #[test]
    fn citation_pins_il_authority() {
        let r = check(&il_clean());
        assert!(r.citation.contains("765 ILCS 750"));
        assert!(r.citation.contains("Safe Homes Act"));
    }

    #[test]
    fn citation_pins_wa_authority() {
        let r = check(&wa_clean());
        assert!(r.citation.contains("RCW 59.18.575"));
    }

    #[test]
    fn citation_pins_vawa_authority() {
        let r = check(&vawa_clean());
        assert!(r.citation.contains("34 USC § 12491"));
        assert!(r.citation.contains("VAWA Reauthorization Act of 2022"));
        assert!(r.citation.contains("Form HUD-91066"));
    }

    #[test]
    fn note_pins_ca_180_day_lookback() {
        let r = check(&ca_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("180 days") && n.contains("§ 1946.7")));
    }

    #[test]
    fn note_pins_il_safe_homes_act() {
        let r = check(&il_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Safe Homes Act") && n.contains("survivors")));
    }

    #[test]
    fn note_pins_wa_90_day_lookback() {
        let r = check(&wa_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("90 days") && n.contains("RCW 59.18.575")));
    }

    #[test]
    fn note_pins_vawa_void_clauses() {
        let r = check(&vawa_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("VOID") && n.contains("police calls")));
    }

    #[test]
    fn ca_strictest_180_day_vs_wa_90_day_invariant() {
        let mut i_ca = ca_clean();
        i_ca.days_since_documentation_issued = 150;
        let r_ca = check(&i_ca);
        assert!(r_ca.documentation_lookback_compliant);

        let mut i_wa = wa_clean();
        i_wa.days_since_dv_incident = 150;
        let r_wa = check(&i_wa);
        assert!(!r_wa.documentation_lookback_compliant);
    }

    #[test]
    fn vawa_uniquely_accepts_hud_91066_invariant() {
        let mut i_ca = ca_clean();
        i_ca.documentation_type = DocumentationType::HudForm91066SelfCertification;
        let r_ca = check(&i_ca);
        assert!(!r_ca.documentation_compliant);

        let r_vawa = check(&vawa_clean());
        assert!(r_vawa.documentation_compliant);
    }

    #[test]
    fn vawa_uniquely_engages_void_clause_doctrine_invariant() {
        let mut i_ca = ca_clean();
        i_ca.lease_contains_void_clauses = true;
        let r_ca = check(&i_ca);
        assert!(!r_ca.void_clauses_engaged);

        let mut i_vawa = vawa_clean();
        i_vawa.lease_contains_void_clauses = true;
        let r_vawa = check(&i_vawa);
        assert!(r_vawa.void_clauses_engaged);
    }

    #[test]
    fn documentation_type_truth_table_for_ca() {
        for (doc, exp_compliant) in [
            (DocumentationType::ProtectiveOrder, true),
            (DocumentationType::PoliceReport, true),
            (DocumentationType::QualifiedThirdPartyStatement, true),
            (DocumentationType::HudForm91066SelfCertification, false),
            (DocumentationType::None, false),
        ] {
            let mut i = ca_clean();
            i.documentation_type = doc;
            let r = check(&i);
            assert_eq!(
                r.documentation_compliant, exp_compliant,
                "doc={:?} expected compliant={}",
                doc, exp_compliant
            );
        }
    }

    #[test]
    fn multiple_ca_violations_stack() {
        let mut i = ca_clean();
        i.written_notice_provided = false;
        i.documentation_type = DocumentationType::None;
        i.landlord_disclosed_documentation_without_consent = true;
        i.landlord_retaliated = true;
        let r = check(&i);
        assert!(r.violations.len() >= 4);
    }
}
