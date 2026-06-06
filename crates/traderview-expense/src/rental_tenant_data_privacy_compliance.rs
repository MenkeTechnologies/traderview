//! Tenant data privacy compliance framework for residential rentals.
//!
//! Landlords collect, store, and use a substantial amount of tenant personal data:
//! application screening (SSN, credit + criminal + eviction history, income
//! verification), lease execution (signed lease, payment data), tenancy operation
//! (smart-lock access logs, IoT-thermostat usage, package-locker biometrics, surveillance
//! footage), and post-tenancy (rent-payment history reported to credit bureaus). State
//! and local data-privacy regimes impose disclosure, consent, security, and right-to-
//! delete obligations. The federal Fair Credit Reporting Act (FCRA) layers on
//! consumer-report-specific obligations.
//!
//! Federal floor + jurisdictional grid:
//!
//! - FAIR CREDIT REPORTING ACT (FCRA), 15 U.S.C. §§ 1681-1681x: landlord must
//!   (1) obtain written tenant consent before pulling consumer report, (2) provide
//!   adverse-action notice if denied based on report content, (3) provide copy of
//!   report and rights notice on tenant request. Civil penalty $100-$1,000 per
//!   violation (§ 1681n willful) + actual damages + attorney fees.
//! - CA CCPA + CPRA (Cal. Civ. Code §§ 1798.100-1798.199.100): notice at collection,
//!   right to know/access, right to delete, right to correct, right to opt out of
//!   data sales, right to limit use of sensitive personal information. Effective
//!   Jan 1, 2020 (CCPA) + Jan 1, 2023 (CPRA amendments). $2,500-$7,500 per
//!   violation civil penalty.
//! - IL BIPA (Biometric Information Privacy Act, 740 ILCS 14/1-99): biometric data
//!   (fingerprint, face geometry, iris scan, voiceprint) requires WRITTEN consent
//!   before collection + written retention/destruction policy. $1,000 per
//!   negligent violation + $5,000 per intentional violation per record. Class-
//!   action liability magnifies exposure dramatically.
//! - NYC TENANT DATA PRIVACY ACT (NYC Admin. Code §§ 26-3001 to 26-3007): "smart
//!   access" buildings (smart locks, cameras, IoT sensors) must provide privacy
//!   notice + obtain tenant consent + limit data use + implement security measures
//!   + delete after tenancy ends.
//! - NY SHIELD ACT (Gen. Bus. Law § 899-aa + § 899-bb): reasonable data-security
//!   safeguards required; breach notification mandated; private property owners
//!   covered.
//! - VA CDPA (Va. Code §§ 59.1-575 to 59.1-585): biometric data is SENSITIVE DATA
//!   requiring opt-in + data-protection-assessment requirement.
//! - TX (Tex. Bus. & Com. Code §§ 521.052 + 503.001 CUBI): biometric identifier
//!   collection requires consent + reasonable security + retention limits.
//! - CO CPA (Colorado Privacy Act, C.R.S. §§ 6-1-1301 to 6-1-1313): opt-in for
//!   sensitive data + universal opt-out signal.
//! - DEFAULT: no statewide data privacy + FCRA + common-law negligent disclosure
//!   + state UDAP fraud frameworks apply.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - oag.ca.gov/privacy/ccpa
//! - nyc.gov/site/hpd/services-and-information/tenant-data-privacy-law.page
//! - privacyrights.org/housing
//! - portabletenant.com/post/fcra-compliant-tenant-screening

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    FederalFcraOnly,
    California,
    Illinois,
    NewYork,
    Virginia,
    Texas,
    Colorado,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataCategory {
    /// FCRA consumer report (credit, criminal, eviction history).
    FcraConsumerReport,
    /// Biometric identifier (fingerprint, face, iris, voiceprint).
    BiometricIdentifier,
    /// Smart-access/IoT data (smart-lock logs, thermostat, package locker).
    SmartAccessOrIotUsageData,
    /// Surveillance camera footage.
    SurveillanceCameraFootage,
    /// Standard rental application + payment data (PII).
    StandardRentalApplicationPii,
    /// Rent-payment-history reporting to credit bureaus.
    RentReportingToCreditBureaus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentAndNoticeStatus {
    /// Written informed consent obtained + privacy notice provided at collection.
    WrittenConsentAndNoticeProvided,
    /// Consent obtained but notice missing or insufficient.
    ConsentObtainedButNoticeInsufficient,
    /// No consent obtained.
    NoConsentObtained,
    /// FCRA-specific adverse-action notice provided after denial.
    FcraAdverseActionNoticeProvided,
    /// FCRA-specific adverse-action notice not provided.
    FcraAdverseActionNoticeNotProvided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantConsentAndNoticeFramework,
    FcraConsumerReportConsentObtainedAdverseActionDuty,
    FcraAdverseActionNoticeMissingViolation,
    BipaBiometricCollectionWithoutWrittenConsentViolation,
    CcpaCpraConsumerRightsNotProvidedViolation,
    NycTenantDataPrivacyActViolation,
    NySchieldActDataSecurityNonCompliance,
    DefaultJurisdictionFcraAndCommonLawOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub data_category: DataCategory,
    pub consent_and_notice_status: ConsentAndNoticeStatus,
    pub estimated_records_collected: u32,
    pub tenant_actual_damages_cents: u64,
}

pub type RentalTenantDataPrivacyComplianceInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub estimated_landlord_exposure_cents: u64,
    pub note: String,
}

pub type RentalTenantDataPrivacyComplianceOutput = Output;
pub type RentalTenantDataPrivacyComplianceResult = Output;

const FCRA_CIVIL_PENALTY_PER_WILLFUL_VIOLATION_CENTS: u64 = 100_000;
const BIPA_NEGLIGENT_PER_RECORD_CENTS: u64 = 100_000;
const BIPA_INTENTIONAL_PER_RECORD_CENTS: u64 = 500_000;
const CCPA_CIVIL_PENALTY_PER_VIOLATION_CENTS: u64 = 250_000;
const CCPA_INTENTIONAL_VIOLATION_PENALTY_CENTS: u64 = 750_000;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(input.data_category, DataCategory::BiometricIdentifier)
        && matches!(input.jurisdiction, Jurisdiction::Illinois)
        && !matches!(
            input.consent_and_notice_status,
            ConsentAndNoticeStatus::WrittenConsentAndNoticeProvided
        )
    {
        let per_record_penalty =
            BIPA_INTENTIONAL_PER_RECORD_CENTS.min(BIPA_NEGLIGENT_PER_RECORD_CENTS * 5);
        let exposure = u64::from(input.estimated_records_collected)
            .saturating_mul(per_record_penalty)
            .saturating_add(input.tenant_actual_damages_cents);
        return Output {
            severity: Severity::BipaBiometricCollectionWithoutWrittenConsentViolation,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "IL BIPA VIOLATION (740 ILCS 14/15): biometric identifier (fingerprint, face \
                 geometry, iris scan, voiceprint) collected WITHOUT prior written consent + \
                 written retention/destruction policy. ${} per negligent violation; ${} per \
                 intentional violation per record. Class-action liability magnifies \
                 dramatically: {} records × $5,000 intentional = ${} statutory damages + \
                 ${} tenant actual damages + attorney fees. Rosenbach v. Six Flags 432 Ill. \
                 Dec. 654 (Ill. 2019) established no actual injury required for statutory \
                 standing.",
                BIPA_NEGLIGENT_PER_RECORD_CENTS / 100,
                BIPA_INTENTIONAL_PER_RECORD_CENTS / 100,
                input.estimated_records_collected,
                u64::from(input.estimated_records_collected)
                    .saturating_mul(BIPA_INTENTIONAL_PER_RECORD_CENTS)
                    / 100,
                input.tenant_actual_damages_cents / 100
            ),
        };
    }

    if matches!(input.data_category, DataCategory::FcraConsumerReport) {
        if matches!(
            input.consent_and_notice_status,
            ConsentAndNoticeStatus::FcraAdverseActionNoticeNotProvided
        ) {
            let exposure = FCRA_CIVIL_PENALTY_PER_WILLFUL_VIOLATION_CENTS
                .saturating_mul(10)
                .saturating_add(input.tenant_actual_damages_cents);
            return Output {
                severity: Severity::FcraAdverseActionNoticeMissingViolation,
                estimated_landlord_exposure_cents: exposure,
                note: format!(
                    "FCRA VIOLATION 15 U.S.C. § 1681m(a): adverse-action notice MISSING after \
                     consumer-report-based denial. Landlord must (1) notify applicant of \
                     adverse action, (2) identify consumer reporting agency that provided \
                     report, (3) provide § 1681g rights notice, (4) inform of free copy + \
                     right to dispute. § 1681n willful violation: $100-$1,000 per violation \
                     + actual damages + punitive damages. § 1681o negligent violation: \
                     actual damages + attorney fees. Estimated exposure ${} = willful \
                     statutory damages range + tenant actual damages (${}).",
                    exposure / 100,
                    input.tenant_actual_damages_cents / 100
                ),
            };
        }
        if matches!(
            input.consent_and_notice_status,
            ConsentAndNoticeStatus::WrittenConsentAndNoticeProvided
                | ConsentAndNoticeStatus::FcraAdverseActionNoticeProvided
        ) {
            return Output {
                severity: Severity::FcraConsumerReportConsentObtainedAdverseActionDuty,
                estimated_landlord_exposure_cents: 0,
                note: "FCRA compliant: written consent obtained + adverse-action notice \
                       provided (if applicable). 15 U.S.C. § 1681b(b)(2) permissible-purpose \
                       satisfied by tenant-screening context. § 1681m adverse-action notice \
                       framework satisfied. Retain consent records + screening reports + \
                       adverse-action notice copies for at least 2 years."
                    .to_string(),
            };
        }
    }

    if matches!(input.jurisdiction, Jurisdiction::California)
        && matches!(
            input.consent_and_notice_status,
            ConsentAndNoticeStatus::NoConsentObtained
                | ConsentAndNoticeStatus::ConsentObtainedButNoticeInsufficient
        )
    {
        let exposure = CCPA_INTENTIONAL_VIOLATION_PENALTY_CENTS
            .saturating_add(input.tenant_actual_damages_cents);
        return Output {
            severity: Severity::CcpaCpraConsumerRightsNotProvidedViolation,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "CA CCPA + CPRA violation (Cal. Civ. Code §§ 1798.100-1798.199.100). \
                 Required: notice at collection + right to know / access / delete / correct \
                 + opt-out of data sales + limit-use of sensitive personal information. \
                 ${} per intentional violation civil penalty (${} per non-intentional). \
                 CCPA effective Jan 1, 2020; CPRA amendments effective Jan 1, 2023. \
                 California Privacy Protection Agency (CPPA) enforces; private right of \
                 action under § 1798.150 for security breaches involving non-encrypted \
                 personal information. Estimated exposure ${} = civil penalty + tenant \
                 actual damages.",
                CCPA_INTENTIONAL_VIOLATION_PENALTY_CENTS / 100,
                CCPA_CIVIL_PENALTY_PER_VIOLATION_CENTS / 100,
                exposure / 100
            ),
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::NewYork)
        && matches!(
            input.data_category,
            DataCategory::SmartAccessOrIotUsageData | DataCategory::BiometricIdentifier
        )
        && !matches!(
            input.consent_and_notice_status,
            ConsentAndNoticeStatus::WrittenConsentAndNoticeProvided
        )
    {
        let exposure = CCPA_CIVIL_PENALTY_PER_VIOLATION_CENTS
            .saturating_mul(2)
            .saturating_add(input.tenant_actual_damages_cents);
        return Output {
            severity: Severity::NycTenantDataPrivacyActViolation,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "NYC TENANT DATA PRIVACY ACT violation (NYC Admin. Code §§ 26-3001 to \
                 26-3007). 'Smart access' buildings (smart locks + cameras + IoT sensors) \
                 must (1) provide privacy notice, (2) obtain tenant consent for data \
                 collection, (3) limit data use to specified purposes, (4) implement \
                 reasonable security measures, (5) delete data after tenancy ends. NY \
                 SHIELD Act Gen. Bus. Law § 899-aa + § 899-bb breach-notification + \
                 reasonable-data-security overlay applies statewide. Estimated exposure \
                 ${}.",
                exposure / 100
            ),
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::Default)
        || matches!(input.jurisdiction, Jurisdiction::FederalFcraOnly)
    {
        return Output {
            severity: Severity::DefaultJurisdictionFcraAndCommonLawOnly,
            estimated_landlord_exposure_cents: 0,
            note: "No statewide data-privacy regime identified beyond federal FCRA. Compliance \
                   floor: FCRA written consent + adverse-action notice + record-retention. \
                   Common-law negligent-disclosure + state UDAP frameworks remain available. \
                   Confirm jurisdiction's emerging data-privacy statute (VA CDPA, TX CUBI, \
                   CO CPA, CT data privacy law, IA, MT, OR, TN, IN all enacted state privacy \
                   laws 2022-2026)."
                .to_string(),
        };
    }

    Output {
        severity: Severity::CompliantConsentAndNoticeFramework,
        estimated_landlord_exposure_cents: 0,
        note: format!(
            "Compliant data-privacy framework. {} Retain consent + notice records for the \
             longer of (a) tenancy + 7 years or (b) state statute-of-limitations.",
            jurisdiction_compliance_label(input.jurisdiction)
        ),
    }
}

fn jurisdiction_compliance_label(jurisdiction: Jurisdiction) -> &'static str {
    match jurisdiction {
        Jurisdiction::California => {
            "CA CCPA + CPRA (Cal. Civ. Code §§ 1798.100-1798.199.100) consumer-rights \
             framework satisfied."
        }
        Jurisdiction::Illinois => {
            "IL BIPA (740 ILCS 14/15) written-consent-and-retention-policy framework \
             satisfied."
        }
        Jurisdiction::NewYork => {
            "NY SHIELD Act Gen. Bus. Law § 899-aa + § 899-bb reasonable-data-security + \
             NYC Tenant Data Privacy Act §§ 26-3001-3007 framework satisfied."
        }
        Jurisdiction::Virginia => {
            "VA CDPA Va. Code §§ 59.1-575 to 59.1-585 sensitive-data opt-in + DPA \
             framework satisfied."
        }
        Jurisdiction::Texas => {
            "TX CUBI Tex. Bus. & Com. Code §§ 521.052 + 503.001 biometric-identifier \
             framework satisfied."
        }
        Jurisdiction::Colorado => {
            "CO CPA C.R.S. §§ 6-1-1301 to 6-1-1313 opt-in + universal-opt-out-signal \
             framework satisfied."
        }
        Jurisdiction::FederalFcraOnly | Jurisdiction::Default => {
            "Federal FCRA framework satisfied; verify state law overlays."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_ca() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            data_category: DataCategory::StandardRentalApplicationPii,
            consent_and_notice_status: ConsentAndNoticeStatus::WrittenConsentAndNoticeProvided,
            estimated_records_collected: 100,
            tenant_actual_damages_cents: 5_000_00,
        }
    }

    #[test]
    fn california_compliant_with_consent_and_notice() {
        let input = base_ca();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantConsentAndNoticeFramework
        );
        assert!(output.note.contains("CCPA"));
        assert!(output.note.contains("CPRA"));
    }

    #[test]
    fn california_no_consent_ccpa_violation() {
        let mut input = base_ca();
        input.consent_and_notice_status = ConsentAndNoticeStatus::NoConsentObtained;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CcpaCpraConsumerRightsNotProvidedViolation
        );
        assert!(output.note.contains("§§ 1798.100"));
        assert!(output.note.contains("CPPA"));
        assert!(output.note.contains("§ 1798.150"));
    }

    #[test]
    fn illinois_bipa_biometric_without_consent_violation() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Illinois;
        input.data_category = DataCategory::BiometricIdentifier;
        input.consent_and_notice_status = ConsentAndNoticeStatus::NoConsentObtained;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::BipaBiometricCollectionWithoutWrittenConsentViolation
        );
        assert!(output.note.contains("740 ILCS 14/15"));
        assert!(output.note.contains("Rosenbach v. Six Flags"));
    }

    #[test]
    fn illinois_bipa_biometric_100_records_5000_intentional() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Illinois;
        input.data_category = DataCategory::BiometricIdentifier;
        input.consent_and_notice_status = ConsentAndNoticeStatus::NoConsentObtained;
        input.tenant_actual_damages_cents = 0;
        let output = check(&input);
        // 100 records × $5,000 = $500,000
        assert_eq!(output.estimated_landlord_exposure_cents, 500_000_00);
    }

    #[test]
    fn fcra_adverse_action_notice_missing_violation() {
        let mut input = base_ca();
        input.data_category = DataCategory::FcraConsumerReport;
        input.consent_and_notice_status =
            ConsentAndNoticeStatus::FcraAdverseActionNoticeNotProvided;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::FcraAdverseActionNoticeMissingViolation
        );
        assert!(output.note.contains("15 U.S.C. § 1681m(a)"));
        assert!(output.note.contains("§ 1681n"));
        assert!(output.note.contains("§ 1681o"));
    }

    #[test]
    fn fcra_consumer_report_consent_obtained_compliant() {
        let mut input = base_ca();
        input.data_category = DataCategory::FcraConsumerReport;
        input.consent_and_notice_status = ConsentAndNoticeStatus::FcraAdverseActionNoticeProvided;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::FcraConsumerReportConsentObtainedAdverseActionDuty
        );
        assert!(output.note.contains("§ 1681b(b)(2)"));
    }

    #[test]
    fn nyc_smart_access_without_consent_tenant_data_privacy_violation() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewYork;
        input.data_category = DataCategory::SmartAccessOrIotUsageData;
        input.consent_and_notice_status = ConsentAndNoticeStatus::NoConsentObtained;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NycTenantDataPrivacyActViolation);
        assert!(output.note.contains("§§ 26-3001 to 26-3007"));
        assert!(output.note.contains("SHIELD Act"));
    }

    #[test]
    fn default_jurisdiction_fcra_and_common_law_only() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Default;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::DefaultJurisdictionFcraAndCommonLawOnly
        );
        assert!(output.note.contains("CDPA"));
        assert!(output.note.contains("CUBI"));
        assert!(output.note.contains("CPA"));
    }

    #[test]
    fn virginia_compliant_framework() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Virginia;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantConsentAndNoticeFramework
        );
        assert!(output.note.contains("VA CDPA"));
        assert!(output.note.contains("§§ 59.1-575"));
    }

    #[test]
    fn texas_compliant_framework() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Texas;
        let output = check(&input);
        assert!(output.note.contains("TX CUBI"));
        assert!(output.note.contains("§§ 521.052"));
    }

    #[test]
    fn colorado_compliant_framework() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Colorado;
        let output = check(&input);
        assert!(output.note.contains("CO CPA"));
        assert!(output.note.contains("§§ 6-1-1301"));
    }

    #[test]
    fn bipa_negligent_per_record_constant_pins_1000() {
        assert_eq!(BIPA_NEGLIGENT_PER_RECORD_CENTS, 100_000);
    }

    #[test]
    fn bipa_intentional_per_record_constant_pins_5000() {
        assert_eq!(BIPA_INTENTIONAL_PER_RECORD_CENTS, 500_000);
    }

    #[test]
    fn ccpa_civil_penalty_constant_pins_2500() {
        assert_eq!(CCPA_CIVIL_PENALTY_PER_VIOLATION_CENTS, 250_000);
    }

    #[test]
    fn ccpa_intentional_constant_pins_7500() {
        assert_eq!(CCPA_INTENTIONAL_VIOLATION_PENALTY_CENTS, 750_000);
    }

    #[test]
    fn fcra_willful_per_violation_constant_pins_1000() {
        assert_eq!(FCRA_CIVIL_PENALTY_PER_WILLFUL_VIOLATION_CENTS, 100_000);
    }

    #[test]
    fn very_large_records_no_overflow_in_bipa_calc() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Illinois;
        input.data_category = DataCategory::BiometricIdentifier;
        input.consent_and_notice_status = ConsentAndNoticeStatus::NoConsentObtained;
        input.estimated_records_collected = u32::MAX;
        let output = check(&input);
        // saturating_mul defense
        assert!(output.estimated_landlord_exposure_cents > 0);
    }

    #[test]
    fn zero_records_zero_bipa_exposure_aside_from_damages() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Illinois;
        input.data_category = DataCategory::BiometricIdentifier;
        input.consent_and_notice_status = ConsentAndNoticeStatus::NoConsentObtained;
        input.estimated_records_collected = 0;
        let output = check(&input);
        // 0 records × $5K = $0; + $5K tenant damages = $5K
        assert_eq!(output.estimated_landlord_exposure_cents, 5_000_00);
    }

    #[test]
    fn note_pins_rosenbach_v_six_flags_no_actual_injury() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Illinois;
        input.data_category = DataCategory::BiometricIdentifier;
        input.consent_and_notice_status = ConsentAndNoticeStatus::NoConsentObtained;
        let output = check(&input);
        assert!(output.note.contains("no actual injury required"));
    }

    #[test]
    fn ny_iot_data_without_consent_tenant_data_privacy_act_violation() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewYork;
        input.data_category = DataCategory::SmartAccessOrIotUsageData;
        input.consent_and_notice_status =
            ConsentAndNoticeStatus::ConsentObtainedButNoticeInsufficient;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NycTenantDataPrivacyActViolation);
    }
}
