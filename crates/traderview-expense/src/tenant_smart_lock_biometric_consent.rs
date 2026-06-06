//! Tenant right to refuse landlord-installed BIOMETRIC smart
//! lock (fingerprint, facial recognition, retinal scanner) —
//! biometric privacy framework. When landlord installs a smart
//! lock that collects biometric identifiers as condition of
//! entry, may the tenant refuse and demand traditional key
//! access? Distinct from `security_camera_disclosure` (which
//! addresses landlord-installed surveillance cameras),
//! `lock_change_between_tenancies` (between-tenancy lock
//! changes), `landlord_tenant_recording_consent` (audio
//! recording consent), and `tenant_data_privacy` (general
//! tenant data handling).
//!
//! Trader-landlord operational concern in IL, WA, TX (biometric
//! privacy states) + emerging CA/NY frameworks. Failure to
//! obtain written consent before collecting tenant biometrics
//! exposes landlord to per-violation statutory damages, treble
//! recovery on reckless or intentional violations, attorney
//! fees, and class action exposure under state biometric
//! privacy acts.
//!
//! **Five regimes**:
//!
//! **Illinois — Biometric Information Privacy Act (BIPA), 740
//! ILCS 14/1 et seq.** Most aggressive biometric privacy
//! framework. § 15(b) requires private entity to (1) inform
//! individual in writing that biometric information is being
//! collected; (2) identify in writing the SPECIFIC PURPOSE and
//! LENGTH OF TERM for collection, storage, and use; (3) receive
//! WRITTEN RELEASE from the individual before collection.
//! § 15(c) prohibits sale, lease, trade, or other profit from
//! biometric information. § 20 PRIVATE RIGHT OF ACTION with
//! statutory damages: **$1,000 per negligent violation; $5,000
//! per reckless or intentional violation** (or actual damages,
//! whichever greater) + reasonable attorney fees + costs +
//! injunctive relief. Cothron v. White Castle (Ill. 2023) — each
//! biometric scan a separate violation (potential
//! per-scan stacking).
//!
//! **Washington — Biometric Identifiers Act, RCW 19.375**. Enrollment
//! of biometric identifier in database for commercial purpose
//! requires NOTICE and CONSENT. Penalties limited to AG
//! enforcement; no statutory private right of action (unlike
//! BIPA).
//!
//! **Texas — Capture or Use of Biometric Identifier, Tex. Bus.
//! & Comm. Code § 503.001**. Capture of biometric identifier for
//! commercial purpose requires notice and consent. Penalties up
//! to $25,000 per violation enforced by Texas AG; no statutory
//! private right of action.
//!
//! **California — Cal. Civ. Code § 1798.80 et seq. + CCPA/CPRA
//! (Cal. Civ. Code § 1798.100 et seq.)**. No specific biometric
//! statute, but CCPA/CPRA classify biometric identifiers as
//! "sensitive personal information" subject to disclosure +
//! deletion rights. Private right of action limited to data
//! breaches.
//!
//! **Default — no specific biometric statute**. Federal FTC § 5
//! (15 U.S.C. § 45) UDAP doctrine applies generally. State
//! UDAP statutes may reach landlord-as-biometric-collector
//! arrangements. Common-law privacy torts available.
//!
//! **Anti-tying principle**: even where landlord may install a
//! biometric smart lock with proper consent, tenant retains
//! right to TRADITIONAL KEY ACCESS as alternative when tenant
//! refuses biometric enrollment. Landlord conditioning access
//! solely on biometric enrollment violates anti-tying and quiet
//! enjoyment.
//!
//! Citations: 740 ILCS 14/15(b) (IL BIPA written consent +
//! purpose disclosure); 740 ILCS 14/15(c) (BIPA no-profit);
//! 740 ILCS 14/20 (BIPA private right of action + $1,000 /
//! $5,000 statutory damages); Cothron v. White Castle System,
//! Inc., 2023 IL 128004 (per-scan violation accrual); RCW
//! 19.375 (WA Biometric Identifiers Act); Tex. Bus. & Comm.
//! Code § 503.001 (TX Capture or Use of Biometric Identifier);
//! Cal. Civ. Code §§ 1798.80, 1798.100 et seq. (CCPA/CPRA
//! biometric as sensitive personal information); 15 U.S.C. § 45
//! (FTC Act § 5 UDAP); common-law invasion of privacy.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Illinois,
    Washington,
    Texas,
    California,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViolationCharacterization {
    /// Negligent — statutory minimum damages.
    Negligent,
    /// Reckless or intentional — heightened statutory damages.
    RecklessOrIntentional,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantSmartLockBiometricInput {
    pub regime: Regime,
    /// Whether the landlord installed a smart lock that collects
    /// biometric identifiers (fingerprint, facial recognition,
    /// retinal scanner).
    pub landlord_installed_biometric_smart_lock: bool,
    /// Whether the landlord obtained the tenant's WRITTEN
    /// release / consent before collecting biometrics.
    pub written_consent_obtained: bool,
    /// Whether the landlord provided written disclosure of
    /// (1) the specific purpose and (2) the length of term for
    /// biometric collection, storage, and use.
    pub written_purpose_and_length_disclosure_provided: bool,
    /// Whether the landlord offers TRADITIONAL KEY ACCESS as
    /// an alternative when tenant refuses biometric enrollment.
    pub traditional_key_access_offered_as_alternative: bool,
    /// Whether the tenant has refused biometric consent.
    pub tenant_refused_biometric_consent: bool,
    /// Number of biometric scans collected from tenant (relevant
    /// to Cothron v. White Castle per-scan violation accrual in
    /// Illinois).
    pub biometric_scans_collected: u32,
    /// Whether the violation is characterized as negligent vs
    /// reckless/intentional (statutory damages tier).
    pub violation_characterization: ViolationCharacterization,
    /// Whether the landlord sold, leased, traded, or otherwise
    /// profited from tenant biometric information (BIPA § 15(c)
    /// violation).
    pub landlord_profited_from_biometric_information: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TenantSmartLockBiometricResult {
    pub tenant_refusal_right_engaged: bool,
    pub biometric_law_violated: bool,
    /// Statutory damages per violation in cents (Illinois BIPA
    /// $1,000 / $5,000).
    pub statutory_damages_per_violation_cents: i64,
    /// Total statutory damages exposure in cents (per-scan
    /// stacking in Illinois under Cothron).
    pub total_statutory_damages_exposure_cents: i64,
    pub private_right_of_action_available: bool,
    pub traditional_key_access_required: bool,
    pub attorney_fees_recoverable: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &TenantSmartLockBiometricInput) -> TenantSmartLockBiometricResult {
    match input.regime {
        Regime::Illinois => check_illinois(input),
        Regime::Washington => check_washington(input),
        Regime::Texas => check_texas(input),
        Regime::California => check_california(input),
        Regime::Default => check_default(input),
    }
}

fn check_illinois(input: &TenantSmartLockBiometricInput) -> TenantSmartLockBiometricResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    notes.push(
        "740 ILCS 14/15(b) — BIPA requires private entity to (1) inform individual in writing biometric information is being collected; (2) identify in writing specific PURPOSE and LENGTH OF TERM for collection, storage, use; (3) receive WRITTEN RELEASE from individual"
            .to_string(),
    );

    if input.landlord_installed_biometric_smart_lock && !input.written_consent_obtained {
        violations.push(
            "740 ILCS 14/15(b)(3) — landlord failed to obtain WRITTEN RELEASE from tenant before collecting biometric information; § 20 private right of action engaged"
                .to_string(),
        );
    }

    if input.landlord_installed_biometric_smart_lock
        && !input.written_purpose_and_length_disclosure_provided
    {
        violations.push(
            "740 ILCS 14/15(b)(2) — landlord failed to identify in writing the specific PURPOSE and LENGTH OF TERM for biometric collection, storage, and use; § 20 private right of action engaged"
                .to_string(),
        );
    }

    if input.landlord_profited_from_biometric_information {
        violations.push(
            "740 ILCS 14/15(c) — landlord may not sell, lease, trade, or otherwise profit from tenant biometric information"
                .to_string(),
        );
    }

    if input.landlord_installed_biometric_smart_lock
        && !input.traditional_key_access_offered_as_alternative
    {
        violations.push(
            "anti-tying + quiet enjoyment — landlord conditioning unit entry solely on biometric enrollment when tenant refuses violates tenant's reasonable use; traditional key access must be offered as alternative"
                .to_string(),
        );
    }

    let per_violation = match input.violation_characterization {
        ViolationCharacterization::Negligent => 100_000i64,
        ViolationCharacterization::RecklessOrIntentional => 500_000i64,
    };

    let scans = input.biometric_scans_collected.max(1) as i64;
    let total_exposure = if !violations.is_empty() {
        per_violation.saturating_mul(scans)
    } else {
        0
    };

    if total_exposure > 0 {
        notes.push(format!(
            "Cothron v. White Castle System, Inc., 2023 IL 128004 — each biometric scan is a SEPARATE BIPA violation; per-scan accrual: {} scans × ${} per violation = ${} total exposure",
            scans,
            per_violation / 100,
            total_exposure / 100
        ));
    }

    let refusal_engaged = input.tenant_refused_biometric_consent;
    let key_access_required = input.landlord_installed_biometric_smart_lock;
    let biometric_violated = !violations.is_empty();

    TenantSmartLockBiometricResult {
        tenant_refusal_right_engaged: refusal_engaged,
        biometric_law_violated: biometric_violated,
        statutory_damages_per_violation_cents: per_violation,
        total_statutory_damages_exposure_cents: total_exposure,
        private_right_of_action_available: true,
        traditional_key_access_required: key_access_required,
        attorney_fees_recoverable: biometric_violated,
        violations,
        citation: "740 ILCS 14/15(b), 14/15(c), 14/20; Cothron v. White Castle System, Inc., 2023 IL 128004",
        notes,
    }
}

fn check_washington(input: &TenantSmartLockBiometricInput) -> TenantSmartLockBiometricResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "RCW 19.375 — Washington Biometric Identifiers Act; enrollment of biometric identifier in database for COMMERCIAL PURPOSE requires NOTICE and CONSENT"
            .to_string(),
        "WA Biometric Identifiers Act — enforcement limited to Washington Attorney General; NO statutory private right of action (unlike Illinois BIPA)"
            .to_string(),
    ];

    if input.landlord_installed_biometric_smart_lock && !input.written_consent_obtained {
        violations.push(
            "RCW 19.375 — landlord failed to obtain notice + consent before enrolling tenant biometric identifier for commercial purpose"
                .to_string(),
        );
    }

    if input.landlord_installed_biometric_smart_lock
        && !input.traditional_key_access_offered_as_alternative
    {
        violations.push(
            "anti-tying + quiet enjoyment — landlord must offer traditional key access as alternative when tenant refuses biometric enrollment"
                .to_string(),
        );
    }

    let biometric_violated = !violations.is_empty();
    let refusal_engaged = input.tenant_refused_biometric_consent;
    let key_access_required = input.landlord_installed_biometric_smart_lock;

    TenantSmartLockBiometricResult {
        tenant_refusal_right_engaged: refusal_engaged,
        biometric_law_violated: biometric_violated,
        statutory_damages_per_violation_cents: 0,
        total_statutory_damages_exposure_cents: 0,
        private_right_of_action_available: false,
        traditional_key_access_required: key_access_required,
        attorney_fees_recoverable: false,
        violations,
        citation: "RCW 19.375 (Washington Biometric Identifiers Act)",
        notes,
    }
}

fn check_texas(input: &TenantSmartLockBiometricInput) -> TenantSmartLockBiometricResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Tex. Bus. & Comm. Code § 503.001 — Capture or Use of Biometric Identifier; capture of biometric identifier for COMMERCIAL PURPOSE requires NOTICE and CONSENT"
            .to_string(),
        "Tex. Bus. & Comm. Code § 503.001 — civil penalty up to $25,000 PER VIOLATION enforced by Texas Attorney General; NO statutory private right of action (unlike Illinois BIPA)"
            .to_string(),
    ];

    if input.landlord_installed_biometric_smart_lock && !input.written_consent_obtained {
        violations.push(
            "Tex. Bus. & Comm. Code § 503.001 — landlord failed to obtain notice + consent before capturing tenant biometric identifier for commercial purpose; $25,000 per violation AG civil penalty"
                .to_string(),
        );
    }

    if input.landlord_installed_biometric_smart_lock
        && !input.traditional_key_access_offered_as_alternative
    {
        violations.push(
            "anti-tying + quiet enjoyment — landlord must offer traditional key access as alternative when tenant refuses biometric enrollment"
                .to_string(),
        );
    }

    let biometric_violated = !violations.is_empty();
    let refusal_engaged = input.tenant_refused_biometric_consent;
    let key_access_required = input.landlord_installed_biometric_smart_lock;
    let ag_penalty_per_violation = 2_500_000i64;
    let total_exposure = if biometric_violated {
        ag_penalty_per_violation
    } else {
        0
    };

    TenantSmartLockBiometricResult {
        tenant_refusal_right_engaged: refusal_engaged,
        biometric_law_violated: biometric_violated,
        statutory_damages_per_violation_cents: ag_penalty_per_violation,
        total_statutory_damages_exposure_cents: total_exposure,
        private_right_of_action_available: false,
        traditional_key_access_required: key_access_required,
        attorney_fees_recoverable: false,
        violations,
        citation: "Tex. Bus. & Comm. Code § 503.001",
        notes,
    }
}

fn check_california(input: &TenantSmartLockBiometricInput) -> TenantSmartLockBiometricResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Civ. Code §§ 1798.80, 1798.100 et seq. (CCPA / CPRA) — biometric identifiers classified as SENSITIVE PERSONAL INFORMATION subject to disclosure + deletion rights; no California-specific biometric statute"
            .to_string(),
        "CCPA / CPRA — private right of action limited to data breach exposure; AG enforcement for general violations"
            .to_string(),
    ];

    if input.landlord_installed_biometric_smart_lock && !input.written_consent_obtained {
        violations.push(
            "Cal. Civ. Code § 1798.100 — landlord collecting biometric (sensitive personal information) without notice or consent may violate CCPA/CPRA disclosure obligations"
                .to_string(),
        );
    }

    if input.landlord_installed_biometric_smart_lock
        && !input.traditional_key_access_offered_as_alternative
    {
        violations.push(
            "anti-tying + quiet enjoyment + Cal. Civ. Code § 1940.2 (prohibited harassing acts including coercive conduct) — landlord must offer traditional key access as alternative when tenant refuses biometric enrollment"
                .to_string(),
        );
    }

    let biometric_violated = !violations.is_empty();
    let refusal_engaged = input.tenant_refused_biometric_consent;
    let key_access_required = input.landlord_installed_biometric_smart_lock;

    TenantSmartLockBiometricResult {
        tenant_refusal_right_engaged: refusal_engaged,
        biometric_law_violated: biometric_violated,
        statutory_damages_per_violation_cents: 0,
        total_statutory_damages_exposure_cents: 0,
        private_right_of_action_available: false,
        traditional_key_access_required: key_access_required,
        attorney_fees_recoverable: false,
        violations,
        citation:
            "Cal. Civ. Code §§ 1798.80, 1798.100 et seq. (CCPA/CPRA); Cal. Civ. Code § 1940.2",
        notes,
    }
}

fn check_default(input: &TenantSmartLockBiometricInput) -> TenantSmartLockBiometricResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "default rule — no state-specific biometric privacy statute; federal FTC Act § 5 (15 U.S.C. § 45) UDAP doctrine applies generally to landlord-as-biometric-collector arrangements"
            .to_string(),
        "default rule — state UDAP statutes (47 states + DC) and common-law invasion-of-privacy torts available; landlord may install smart lock subject to lease + common-law privacy framework"
            .to_string(),
    ];

    if input.landlord_installed_biometric_smart_lock
        && !input.traditional_key_access_offered_as_alternative
    {
        violations.push(
            "anti-tying + quiet enjoyment — landlord must offer traditional key access as alternative when tenant refuses biometric enrollment; common-law privacy + state UDAP framework applies"
                .to_string(),
        );
    }

    let biometric_violated = !violations.is_empty();
    let refusal_engaged = input.tenant_refused_biometric_consent;
    let key_access_required = input.landlord_installed_biometric_smart_lock;

    TenantSmartLockBiometricResult {
        tenant_refusal_right_engaged: refusal_engaged,
        biometric_law_violated: biometric_violated,
        statutory_damages_per_violation_cents: 0,
        total_statutory_damages_exposure_cents: 0,
        private_right_of_action_available: false,
        traditional_key_access_required: key_access_required,
        attorney_fees_recoverable: false,
        violations,
        citation: "15 U.S.C. § 45 (FTC Act § 5 UDAP); state-specific UDAP statutes + common-law invasion of privacy",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn il_violation() -> TenantSmartLockBiometricInput {
        TenantSmartLockBiometricInput {
            regime: Regime::Illinois,
            landlord_installed_biometric_smart_lock: true,
            written_consent_obtained: false,
            written_purpose_and_length_disclosure_provided: false,
            traditional_key_access_offered_as_alternative: false,
            tenant_refused_biometric_consent: true,
            biometric_scans_collected: 10,
            violation_characterization: ViolationCharacterization::Negligent,
            landlord_profited_from_biometric_information: false,
        }
    }

    fn il_compliant() -> TenantSmartLockBiometricInput {
        let mut i = il_violation();
        i.written_consent_obtained = true;
        i.written_purpose_and_length_disclosure_provided = true;
        i.traditional_key_access_offered_as_alternative = true;
        i.tenant_refused_biometric_consent = false;
        i.biometric_scans_collected = 0;
        i
    }

    fn wa_violation() -> TenantSmartLockBiometricInput {
        let mut i = il_violation();
        i.regime = Regime::Washington;
        i
    }

    fn tx_violation() -> TenantSmartLockBiometricInput {
        let mut i = il_violation();
        i.regime = Regime::Texas;
        i
    }

    fn ca_violation() -> TenantSmartLockBiometricInput {
        let mut i = il_violation();
        i.regime = Regime::California;
        i
    }

    fn default_violation() -> TenantSmartLockBiometricInput {
        let mut i = il_violation();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn il_full_compliance_no_violation() {
        let r = check(&il_compliant());
        assert!(!r.biometric_law_violated);
        assert!(r.violations.is_empty());
        assert_eq!(r.total_statutory_damages_exposure_cents, 0);
    }

    #[test]
    fn il_missing_consent_triggers_violation() {
        let r = check(&il_violation());
        assert!(r.biometric_law_violated);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("14/15(b)(3)") && v.contains("WRITTEN RELEASE")));
    }

    #[test]
    fn il_missing_purpose_disclosure_triggers_violation() {
        let mut i = il_violation();
        i.written_consent_obtained = true;
        i.written_purpose_and_length_disclosure_provided = false;
        let r = check(&i);
        assert!(r.biometric_law_violated);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("14/15(b)(2)") && v.contains("PURPOSE and LENGTH OF TERM")));
    }

    #[test]
    fn il_profit_from_biometric_triggers_15c_violation() {
        let mut i = il_compliant();
        i.landlord_profited_from_biometric_information = true;
        let r = check(&i);
        assert!(r.biometric_law_violated);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("14/15(c)") && v.contains("profit")));
    }

    #[test]
    fn il_negligent_damages_1000_per_violation() {
        let r = check(&il_violation());
        assert_eq!(r.statutory_damages_per_violation_cents, 100_000);
    }

    #[test]
    fn il_reckless_damages_5000_per_violation() {
        let mut i = il_violation();
        i.violation_characterization = ViolationCharacterization::RecklessOrIntentional;
        let r = check(&i);
        assert_eq!(r.statutory_damages_per_violation_cents, 500_000);
    }

    #[test]
    fn il_cothron_per_scan_stacking() {
        let mut i = il_violation();
        i.biometric_scans_collected = 50;
        let r = check(&i);
        assert_eq!(r.total_statutory_damages_exposure_cents, 50 * 100_000);
        assert!(r.notes.iter().any(
            |n| n.contains("Cothron v. White Castle") && n.contains("SEPARATE BIPA violation")
        ));
    }

    #[test]
    fn il_reckless_and_50_scans_per_scan_5000() {
        let mut i = il_violation();
        i.biometric_scans_collected = 50;
        i.violation_characterization = ViolationCharacterization::RecklessOrIntentional;
        let r = check(&i);
        assert_eq!(r.total_statutory_damages_exposure_cents, 50 * 500_000);
    }

    #[test]
    fn il_no_violations_no_exposure() {
        let r = check(&il_compliant());
        assert_eq!(r.total_statutory_damages_exposure_cents, 0);
    }

    #[test]
    fn il_private_right_of_action_always_available() {
        let r = check(&il_violation());
        assert!(r.private_right_of_action_available);
    }

    #[test]
    fn il_attorney_fees_recoverable_when_violation() {
        let r = check(&il_violation());
        assert!(r.attorney_fees_recoverable);
    }

    #[test]
    fn il_anti_tying_violation_when_no_key_alternative() {
        let r = check(&il_violation());
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("anti-tying") && v.contains("traditional key access")));
    }

    #[test]
    fn il_citation_pins_bipa_subsections_and_cothron() {
        let r = check(&il_violation());
        assert!(r.citation.contains("14/15(b)"));
        assert!(r.citation.contains("14/15(c)"));
        assert!(r.citation.contains("14/20"));
        assert!(r.citation.contains("Cothron v. White Castle"));
    }

    #[test]
    fn wa_missing_consent_violation() {
        let r = check(&wa_violation());
        assert!(r.biometric_law_violated);
        assert!(r.violations.iter().any(|v| v.contains("RCW 19.375")));
    }

    #[test]
    fn wa_no_private_right_of_action() {
        let r = check(&wa_violation());
        assert!(!r.private_right_of_action_available);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Washington Attorney General")
                && n.contains("NO statutory private right of action")));
    }

    #[test]
    fn wa_no_statutory_damages_per_violation() {
        let r = check(&wa_violation());
        assert_eq!(r.statutory_damages_per_violation_cents, 0);
    }

    #[test]
    fn tx_missing_consent_violation() {
        let r = check(&tx_violation());
        assert!(r.biometric_law_violated);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 503.001") && v.contains("$25,000 per violation")));
    }

    #[test]
    fn tx_civil_penalty_25000_per_violation() {
        let r = check(&tx_violation());
        assert_eq!(r.statutory_damages_per_violation_cents, 2_500_000);
    }

    #[test]
    fn tx_no_private_right_of_action() {
        let r = check(&tx_violation());
        assert!(!r.private_right_of_action_available);
    }

    #[test]
    fn ca_ccpa_classifies_biometric_as_sensitive_pi() {
        let r = check(&ca_violation());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("CCPA / CPRA") && n.contains("SENSITIVE PERSONAL INFORMATION")));
    }

    #[test]
    fn ca_missing_consent_violation() {
        let r = check(&ca_violation());
        assert!(r.biometric_law_violated);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1798.100") && v.contains("CCPA/CPRA")));
    }

    #[test]
    fn ca_no_state_specific_biometric_statute_note() {
        let r = check(&ca_violation());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("no California-specific biometric statute")));
    }

    #[test]
    fn default_no_state_specific_statute_note() {
        let r = check(&default_violation());
        assert!(r.notes.iter().any(
            |n| n.contains("no state-specific biometric privacy statute")
                && n.contains("FTC Act § 5")
        ));
    }

    #[test]
    fn default_anti_tying_violation_when_no_key_alternative() {
        let r = check(&default_violation());
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("anti-tying") && v.contains("common-law privacy")));
    }

    #[test]
    fn traditional_key_access_required_when_biometric_lock_installed() {
        for regime in [
            Regime::Illinois,
            Regime::Washington,
            Regime::Texas,
            Regime::California,
            Regime::Default,
        ] {
            let mut i = il_violation();
            i.regime = regime;
            let r = check(&i);
            assert!(r.traditional_key_access_required);
        }
    }

    #[test]
    fn no_lock_installed_no_key_access_required() {
        let mut i = il_compliant();
        i.landlord_installed_biometric_smart_lock = false;
        let r = check(&i);
        assert!(!r.traditional_key_access_required);
    }

    #[test]
    fn five_regimes_routed_correctly() {
        for regime in [
            Regime::Illinois,
            Regime::Washington,
            Regime::Texas,
            Regime::California,
            Regime::Default,
        ] {
            let mut i = il_violation();
            i.regime = regime;
            let r = check(&i);
            let _ = r.biometric_law_violated;
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn il_unique_private_right_of_action_invariant() {
        let r_il = check(&il_violation());
        assert!(r_il.private_right_of_action_available);

        for regime in [
            Regime::Washington,
            Regime::Texas,
            Regime::California,
            Regime::Default,
        ] {
            let mut i = il_violation();
            i.regime = regime;
            let r = check(&i);
            assert!(
                !r.private_right_of_action_available,
                "regime {:?} should not have private right of action",
                regime
            );
        }
    }

    #[test]
    fn il_unique_per_scan_stacking_invariant() {
        let mut i_il = il_violation();
        i_il.biometric_scans_collected = 100;
        let r_il = check(&i_il);
        assert_eq!(r_il.total_statutory_damages_exposure_cents, 100 * 100_000);

        let mut i_tx = il_violation();
        i_tx.regime = Regime::Texas;
        i_tx.biometric_scans_collected = 100;
        let r_tx = check(&i_tx);
        assert_eq!(r_tx.total_statutory_damages_exposure_cents, 2_500_000);
    }

    #[test]
    fn il_negligent_vs_reckless_damages_5x_ratio() {
        let mut i = il_violation();
        i.violation_characterization = ViolationCharacterization::Negligent;
        let r_neg = check(&i);

        i.violation_characterization = ViolationCharacterization::RecklessOrIntentional;
        let r_reck = check(&i);

        assert_eq!(
            r_reck.statutory_damages_per_violation_cents,
            r_neg.statutory_damages_per_violation_cents * 5
        );
    }

    #[test]
    fn il_saturating_mul_no_overflow_on_extreme_scans() {
        let mut i = il_violation();
        i.biometric_scans_collected = u32::MAX;
        i.violation_characterization = ViolationCharacterization::RecklessOrIntentional;
        let r = check(&i);
        assert!(r.total_statutory_damages_exposure_cents > 0);
    }

    #[test]
    fn il_zero_scans_floors_at_one_scan() {
        let mut i = il_violation();
        i.biometric_scans_collected = 0;
        let r = check(&i);
        assert_eq!(r.total_statutory_damages_exposure_cents, 100_000);
    }

    #[test]
    fn anti_tying_violation_across_all_regimes_when_no_key_alternative() {
        for regime in [
            Regime::Illinois,
            Regime::Washington,
            Regime::Texas,
            Regime::California,
            Regime::Default,
        ] {
            let mut i = il_violation();
            i.regime = regime;
            i.written_consent_obtained = true;
            i.written_purpose_and_length_disclosure_provided = true;
            i.traditional_key_access_offered_as_alternative = false;
            let r = check(&i);
            assert!(
                r.violations.iter().any(|v| v.contains("anti-tying")),
                "regime {:?} should engage anti-tying violation",
                regime
            );
        }
    }

    #[test]
    fn full_il_compliance_no_violations() {
        let r = check(&il_compliant());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn tenant_refusal_engaged_when_refused() {
        let r = check(&il_violation());
        assert!(r.tenant_refusal_right_engaged);
    }

    #[test]
    fn tenant_refusal_not_engaged_when_consented() {
        let r = check(&il_compliant());
        assert!(!r.tenant_refusal_right_engaged);
    }
}
