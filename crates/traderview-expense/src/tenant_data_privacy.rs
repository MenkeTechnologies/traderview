//! State tenant data privacy / records access compliance.
//!
//! Emerging area as landlord-traders adopt digital tenant screening,
//! smart-building tech (smart locks, doorbell cameras, biometric
//! entry), and tenant records databases. Comprehensive state privacy
//! laws have proliferated since CCPA (2018) — and Illinois's
//! Biometric Information Privacy Act (BIPA, 2008) uniquely applies to
//! ALL entities collecting biometric data without revenue threshold,
//! making it the dominant landlord risk for facial-recognition or
//! fingerprint building entry.
//!
//! Three regimes:
//!
//! `BiometricStrictWrittenConsent`: IL (Biometric Information Privacy
//! Act, 740 ILCS 14/) — applies to all entities collecting biometric
//! data with no revenue threshold; written informed consent required
//! BEFORE collection; private right of action with $1,000 per
//! negligent / $5,000 per intentional violation.
//!
//! `ComprehensivePrivacyLawRevenueThreshold`: CA (CCPA/CPRA), VA
//! (VCDPA), CO (CPA), CT (CTDPA), OR, DE, MD, MN, others. Applies
//! when landlord-business meets revenue / consumer-volume thresholds.
//! CA threshold: $25M+ revenue OR 100K+ CA consumers OR 50%+ revenue
//! from selling personal info. VA/CO/CT threshold: 100K+ residents OR
//! 25%+ revenue from sale of personal data of 25K+ residents. Tenant
//! data subject access requests must be responded to within 45 days
//! (most states).
//!
//! `NoStatePrivacyLaw`: most other states. No comprehensive privacy
//! law; common-law and contract terms govern.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyRegime {
    BiometricStrictWrittenConsent,
    ComprehensivePrivacyLawRevenueThreshold,
    NoStatePrivacyLaw,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: PrivacyRegime,
    /// Annual revenue threshold for comprehensive privacy law
    /// applicability (CA $25M); `None` if regime is not threshold-
    /// based or if state uses different metrics.
    pub annual_revenue_threshold_dollars: Option<i64>,
    /// Consumer/resident count threshold (CA 100k; VA/CO/CT 100k).
    pub consumer_count_threshold: Option<u64>,
    /// Maximum days to respond to a data subject access request.
    /// `None` if not applicable.
    pub max_days_to_respond_to_dsar: Option<u32>,
    /// True if state requires written informed consent BEFORE
    /// biometric collection (IL BIPA, plus many CCPA-equivalents
    /// for sensitive-data category).
    pub biometric_written_consent_required: bool,
    /// Per-violation statutory damages floor for biometric violations
    /// (IL BIPA $1k negligent / $5k intentional).
    pub biometric_negligent_damages_dollars: i64,
    pub biometric_intentional_damages_dollars: i64,
    pub citation: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    regime: PrivacyRegime,
    annual_revenue_threshold_dollars: Option<i64>,
    consumer_count_threshold: Option<u64>,
    max_days_to_respond_to_dsar: Option<u32>,
    biometric_written_consent_required: bool,
    biometric_negligent_damages_dollars: i64,
    biometric_intentional_damages_dollars: i64,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        annual_revenue_threshold_dollars,
        consumer_count_threshold,
        max_days_to_respond_to_dsar,
        biometric_written_consent_required,
        biometric_negligent_damages_dollars,
        biometric_intentional_damages_dollars,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use PrivacyRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "IL",
        rule(
            BiometricStrictWrittenConsent,
            None, None, None, true, 1_000, 5_000,
            "740 ILCS 14/ Biometric Information Privacy Act (BIPA, 2008) — all entities; written informed consent before biometric collection; $1k negligent / $5k intentional per violation; private right of action",
        ),
    );

    // ComprehensivePrivacyLawRevenueThreshold regime — 8 states.
    m.insert(
        "CA",
        rule(
            ComprehensivePrivacyLawRevenueThreshold,
            Some(25_000_000), Some(100_000), Some(45), true, 0, 0,
            "Cal. Civ. Code § 1798.100 (CCPA / CPRA) — $25M revenue OR 100k CA consumers OR 50% revenue from sales triggers; 45-day DSAR response; biometric is sensitive data requiring opt-in",
        ),
    );
    m.insert(
        "VA",
        rule(
            ComprehensivePrivacyLawRevenueThreshold,
            None, Some(100_000), Some(45), true, 0, 0,
            "Va. Code § 59.1-575 et seq. (VCDPA) — 100k VA residents OR 25k residents + 25% revenue from data sale; 45-day DSAR response",
        ),
    );
    m.insert(
        "CO",
        rule(
            ComprehensivePrivacyLawRevenueThreshold,
            None, Some(100_000), Some(45), true, 0, 0,
            "Colo. Rev. Stat. § 6-1-1301 (CPA) — 100k CO residents OR 25k + revenue from data sale; 45-day DSAR response; biometric sensitive data",
        ),
    );
    m.insert(
        "CT",
        rule(
            ComprehensivePrivacyLawRevenueThreshold,
            None, Some(100_000), Some(45), true, 0, 0,
            "Conn. Gen. Stat. § 42-515 (CTDPA) — 100k CT residents OR 25k + 25% revenue from data sale; 45-day DSAR response",
        ),
    );
    m.insert(
        "OR",
        rule(
            ComprehensivePrivacyLawRevenueThreshold,
            None, Some(100_000), Some(45), true, 0, 0,
            "Or. Consumer Privacy Act (eff. 2024-07-01) — 100k OR residents OR 25k + 25% revenue from data sale; biometric sensitive data",
        ),
    );
    m.insert(
        "DE",
        rule(
            ComprehensivePrivacyLawRevenueThreshold,
            None, Some(35_000), Some(45), true, 0, 0,
            "Del. Personal Data Privacy Act (eff. 2025-01-01) — 35k DE residents OR 10k + 20% revenue from data sale; lower than other states",
        ),
    );
    m.insert(
        "MD",
        rule(
            ComprehensivePrivacyLawRevenueThreshold,
            None, Some(35_000), Some(45), true, 0, 0,
            "Md. Online Data Privacy Act (eff. 2025-10-01) — 35k MD residents OR 10k + 20% revenue from data sale",
        ),
    );
    m.insert(
        "MN",
        rule(
            ComprehensivePrivacyLawRevenueThreshold,
            None, Some(100_000), Some(45), true, 0, 0,
            "Minn. Consumer Data Privacy Act (eff. 2025-07-31) — 100k MN residents OR 25k + 25% revenue from data sale",
        ),
    );

    // NoStatePrivacyLaw for remaining states + DC.
    let no_rule = [
        "AL", "AK", "AZ", "AR", "DC", "FL", "GA", "HI", "ID", "IN",
        "IA", "KS", "KY", "LA", "ME", "MA", "MI", "MS", "MO", "MT",
        "NE", "NV", "NH", "NJ", "NM", "NY", "NC", "ND", "OH", "OK",
        "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT", "WA", "WV",
        "WI", "WY",
    ];
    for code in no_rule {
        m.insert(
            code,
            rule(
                NoStatePrivacyLaw,
                None, None, None, false, 0, 0,
                "No comprehensive state privacy law; common-law and contract terms govern",
            ),
        );
    }
    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyInput {
    pub state_code: String,
    pub landlord_annual_revenue_dollars: i64,
    pub consumers_processed_per_year: u64,
    pub uses_biometric_building_entry: bool,
    pub biometric_written_consent_obtained: bool,
    pub privacy_notice_provided: bool,
    pub tenant_dsar_received: bool,
    pub days_since_dsar: u32,
    pub dsar_responded: bool,
    /// True if the biometric collection was intentional (vs.
    /// negligent) — IL BIPA distinguishes these for damages.
    pub biometric_violation_intentional: bool,
    pub number_of_affected_tenants: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyResult {
    pub regime: PrivacyRegime,
    pub subject_to_state_privacy_law: bool,
    pub biometric_consent_required: bool,
    pub biometric_consent_compliant: bool,
    pub biometric_violation_exposure_dollars: i64,
    pub dsar_required: bool,
    pub dsar_response_compliant: bool,
    pub overall_compliant: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &PrivacyInput) -> PrivacyResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: PrivacyRegime::NoStatePrivacyLaw,
        annual_revenue_threshold_dollars: None,
        consumer_count_threshold: None,
        max_days_to_respond_to_dsar: None,
        biometric_written_consent_required: false,
        biometric_negligent_damages_dollars: 0,
        biometric_intentional_damages_dollars: 0,
        citation: "Unknown state code; assuming no state privacy law",
    });

    // Subject test depends on regime.
    let subject = match rule.regime {
        PrivacyRegime::BiometricStrictWrittenConsent => true, // IL BIPA: no threshold
        PrivacyRegime::ComprehensivePrivacyLawRevenueThreshold => {
            let revenue_meets = rule
                .annual_revenue_threshold_dollars
                .is_some_and(|t| input.landlord_annual_revenue_dollars >= t);
            let consumers_meet = rule
                .consumer_count_threshold
                .is_some_and(|t| input.consumers_processed_per_year >= t);
            revenue_meets || consumers_meet
        }
        PrivacyRegime::NoStatePrivacyLaw => false,
    };

    // Biometric compliance.
    let biometric_required =
        rule.biometric_written_consent_required && input.uses_biometric_building_entry;
    let biometric_compliant =
        !biometric_required || input.biometric_written_consent_obtained;
    let biometric_exposure =
        if rule.regime == PrivacyRegime::BiometricStrictWrittenConsent
            && input.uses_biometric_building_entry
            && !input.biometric_written_consent_obtained
        {
            let per_violation = if input.biometric_violation_intentional {
                rule.biometric_intentional_damages_dollars
            } else {
                rule.biometric_negligent_damages_dollars
            };
            per_violation * input.number_of_affected_tenants as i64
        } else {
            0
        };

    // DSAR compliance.
    let dsar_required = subject && input.tenant_dsar_received;
    let dsar_compliant = if dsar_required {
        match rule.max_days_to_respond_to_dsar {
            Some(max) => input.dsar_responded && input.days_since_dsar <= max,
            None => true,
        }
    } else {
        true
    };

    let overall = (!biometric_required || biometric_compliant) && dsar_compliant;

    let note = match (rule.regime, subject) {
        (PrivacyRegime::NoStatePrivacyLaw, _) =>
            "NoStatePrivacyLaw: no comprehensive state privacy law; common-law / contract terms govern.".to_string(),
        (PrivacyRegime::BiometricStrictWrittenConsent, _) => {
            if !input.uses_biometric_building_entry {
                "IL BIPA: no biometric collection in use; no compliance check triggered.".to_string()
            } else if biometric_compliant {
                "IL BIPA: biometric written consent obtained; compliant.".to_string()
            } else {
                format!(
                    "IL BIPA VIOLATION: biometric collected without written informed consent; ${} statutory damages exposure for {} tenant(s) at ${}/violation ({:?}).",
                    biometric_exposure,
                    input.number_of_affected_tenants,
                    if input.biometric_violation_intentional { 5_000 } else { 1_000 },
                    if input.biometric_violation_intentional { "intentional" } else { "negligent" },
                )
            }
        }
        (PrivacyRegime::ComprehensivePrivacyLawRevenueThreshold, false) =>
            "ComprehensivePrivacyLawRevenueThreshold: landlord-business below revenue/consumer thresholds — state privacy law not applicable.".to_string(),
        (PrivacyRegime::ComprehensivePrivacyLawRevenueThreshold, true) => {
            let dsar_part = if dsar_required {
                if dsar_compliant {
                    format!(" DSAR responded within {} days.", rule.max_days_to_respond_to_dsar.unwrap_or(45))
                } else {
                    format!(" DSAR VIOLATION: {} days elapsed > {}-day window.", input.days_since_dsar, rule.max_days_to_respond_to_dsar.unwrap_or(45))
                }
            } else {
                String::new()
            };
            format!(
                "ComprehensivePrivacyLawRevenueThreshold: thresholds met; landlord subject to state privacy law.{}",
                dsar_part,
            )
        }
    };

    PrivacyResult {
        regime: rule.regime,
        subject_to_state_privacy_law: subject,
        biometric_consent_required: biometric_required,
        biometric_consent_compliant: biometric_compliant,
        biometric_violation_exposure_dollars: biometric_exposure,
        dsar_required,
        dsar_response_compliant: dsar_compliant,
        overall_compliant: overall,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str) -> PrivacyInput {
        PrivacyInput {
            state_code: state.to_string(),
            landlord_annual_revenue_dollars: 1_000_000,
            consumers_processed_per_year: 100,
            uses_biometric_building_entry: false,
            biometric_written_consent_obtained: false,
            privacy_notice_provided: false,
            tenant_dsar_received: false,
            days_since_dsar: 0,
            dsar_responded: false,
            biometric_violation_intentional: false,
            number_of_affected_tenants: 1,
        }
    }

    // IL BIPA — strict, no threshold.

    #[test]
    fn il_biometric_use_without_consent_violates() {
        let mut i = input("IL");
        i.uses_biometric_building_entry = true;
        i.biometric_written_consent_obtained = false;
        i.number_of_affected_tenants = 10;
        let r = check(&i);
        assert_eq!(r.regime, PrivacyRegime::BiometricStrictWrittenConsent);
        assert!(r.subject_to_state_privacy_law);
        assert!(!r.biometric_consent_compliant);
        assert_eq!(r.biometric_violation_exposure_dollars, 10_000); // $1k × 10
        assert!(!r.overall_compliant);
    }

    #[test]
    fn il_biometric_with_consent_complies() {
        let mut i = input("IL");
        i.uses_biometric_building_entry = true;
        i.biometric_written_consent_obtained = true;
        let r = check(&i);
        assert!(r.biometric_consent_compliant);
        assert!(r.overall_compliant);
    }

    #[test]
    fn il_intentional_violation_5x_damages() {
        let mut i = input("IL");
        i.uses_biometric_building_entry = true;
        i.biometric_violation_intentional = true;
        i.number_of_affected_tenants = 10;
        let r = check(&i);
        assert_eq!(r.biometric_violation_exposure_dollars, 50_000); // $5k × 10
    }

    #[test]
    fn il_no_biometric_no_violation() {
        let i = input("IL");
        let r = check(&i);
        assert_eq!(r.biometric_violation_exposure_dollars, 0);
        assert!(r.overall_compliant);
    }

    #[test]
    fn il_bipa_has_no_revenue_threshold() {
        // Even a $100k revenue landlord is subject.
        let mut i = input("IL");
        i.landlord_annual_revenue_dollars = 100_000;
        i.uses_biometric_building_entry = true;
        i.biometric_written_consent_obtained = false;
        let r = check(&i);
        assert!(r.subject_to_state_privacy_law);
        assert!(!r.overall_compliant);
    }

    // CA CCPA — revenue-threshold.

    #[test]
    fn ca_under_25m_revenue_under_100k_consumers_not_subject() {
        let mut i = input("CA");
        i.landlord_annual_revenue_dollars = 1_000_000;
        i.consumers_processed_per_year = 50;
        let r = check(&i);
        assert!(!r.subject_to_state_privacy_law);
        assert!(r.overall_compliant);
    }

    #[test]
    fn ca_over_25m_revenue_triggers_law() {
        let mut i = input("CA");
        i.landlord_annual_revenue_dollars = 30_000_000;
        let r = check(&i);
        assert!(r.subject_to_state_privacy_law);
    }

    #[test]
    fn ca_100k_consumers_triggers_law() {
        let mut i = input("CA");
        i.consumers_processed_per_year = 100_000;
        let r = check(&i);
        assert!(r.subject_to_state_privacy_law);
    }

    // DSAR window.

    #[test]
    fn ca_dsar_response_within_45_days_complies() {
        let mut i = input("CA");
        i.landlord_annual_revenue_dollars = 30_000_000;
        i.tenant_dsar_received = true;
        i.days_since_dsar = 30;
        i.dsar_responded = true;
        let r = check(&i);
        assert!(r.dsar_response_compliant);
        assert!(r.overall_compliant);
    }

    #[test]
    fn ca_dsar_response_46_days_violates() {
        let mut i = input("CA");
        i.landlord_annual_revenue_dollars = 30_000_000;
        i.tenant_dsar_received = true;
        i.days_since_dsar = 46;
        i.dsar_responded = true;
        let r = check(&i);
        assert!(!r.dsar_response_compliant);
        assert!(!r.overall_compliant);
    }

    #[test]
    fn ca_dsar_not_responded_violates() {
        let mut i = input("CA");
        i.landlord_annual_revenue_dollars = 30_000_000;
        i.tenant_dsar_received = true;
        i.days_since_dsar = 10;
        i.dsar_responded = false;
        let r = check(&i);
        assert!(!r.dsar_response_compliant);
    }

    // VA / CO / CT — same threshold pattern.

    #[test]
    fn va_100k_residents_triggers_vcdpa() {
        let mut i = input("VA");
        i.consumers_processed_per_year = 100_000;
        let r = check(&i);
        assert!(r.subject_to_state_privacy_law);
    }

    #[test]
    fn co_under_100k_residents_not_subject() {
        let mut i = input("CO");
        i.consumers_processed_per_year = 50_000;
        let r = check(&i);
        assert!(!r.subject_to_state_privacy_law);
    }

    // DE / MD lower thresholds (35k).

    #[test]
    fn de_35k_residents_triggers_law() {
        let mut i = input("DE");
        i.consumers_processed_per_year = 35_000;
        let r = check(&i);
        assert!(r.subject_to_state_privacy_law);
    }

    #[test]
    fn md_lower_threshold_than_ca() {
        // MD threshold 35k vs CA 100k.
        let mut i = input("MD");
        i.consumers_processed_per_year = 50_000;
        let r = check(&i);
        assert!(r.subject_to_state_privacy_law);
    }

    // No-rule states.

    #[test]
    fn tx_no_state_privacy_law_no_compliance_required() {
        let mut i = input("TX");
        i.uses_biometric_building_entry = true;
        let r = check(&i);
        assert_eq!(r.regime, PrivacyRegime::NoStatePrivacyLaw);
        assert!(r.overall_compliant);
    }

    #[test]
    fn ny_no_comprehensive_state_law() {
        let r = check(&input("NY"));
        assert_eq!(r.regime, PrivacyRegime::NoStatePrivacyLaw);
    }

    // Coverage / invariants.

    #[test]
    fn coverage_is_all_50_states_plus_dc() {
        let codes: Vec<&'static str> = RULES.keys().copied().collect();
        assert_eq!(codes.len(), 51, "expected 50 states + DC, got {}", codes.len());
    }

    #[test]
    fn citations_never_empty() {
        for (code, rule) in RULES.iter() {
            assert!(!rule.citation.is_empty(), "{code} missing citation");
        }
    }

    #[test]
    fn only_il_uses_bipa_regime() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == PrivacyRegime::BiometricStrictWrittenConsent {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected IL only with BIPA regime");
    }

    #[test]
    fn comprehensive_privacy_law_8_states() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == PrivacyRegime::ComprehensivePrivacyLawRevenueThreshold {
                count += 1;
            }
        }
        assert_eq!(count, 8, "expected 8 states with comprehensive privacy law");
    }

    #[test]
    fn only_il_has_bipa_damages() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.biometric_negligent_damages_dollars > 0
                || rule.biometric_intentional_damages_dollars > 0
            {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected IL only with BIPA per-violation damages");
    }

    #[test]
    fn unknown_state_falls_back_to_no_law() {
        let r = check(&input("XX"));
        assert_eq!(r.regime, PrivacyRegime::NoStatePrivacyLaw);
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let mut i = input("il");
        i.uses_biometric_building_entry = true;
        let r = check(&i);
        assert!(!r.overall_compliant);
    }

    // Notes.

    #[test]
    fn il_violation_note_mentions_bipa_and_dollar_amount() {
        let mut i = input("IL");
        i.uses_biometric_building_entry = true;
        i.number_of_affected_tenants = 5;
        let r = check(&i);
        assert!(r.note.contains("IL BIPA VIOLATION"));
        assert!(r.note.contains("$5000")); // 5 × $1k = $5k
    }

    #[test]
    fn ca_subject_note_describes_dsar() {
        let mut i = input("CA");
        i.landlord_annual_revenue_dollars = 30_000_000;
        i.tenant_dsar_received = true;
        i.days_since_dsar = 30;
        i.dsar_responded = true;
        let r = check(&i);
        assert!(r.note.contains("DSAR responded"));
    }
}
