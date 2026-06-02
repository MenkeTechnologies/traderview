//! Mandatory landlord-paid post-entry notice after emergency
//! entry. When landlord enters tenant's unit without prior
//! notice due to true emergency (fire, water leak, gas leak),
//! what post-entry written notice obligation attaches? Distinct
//! from `entry_notice` (general 24-hour pre-entry notice),
//! `pesticide_application_notice` (pesticide treatment), and
//! `landlord_harassment` (unauthorized entry as harassment).
//!
//! Trader-landlord operational concern when landlord must enter
//! unit due to flood / fire / gas leak / similar emergency.
//! Failure to provide proper post-entry notice exposes landlord
//! to trespass / harassment / breach of quiet enjoyment claims
//! even when underlying entry was emergency-justified.
//!
//! **Four regimes**:
//!
//! **California — Cal. Civ. Code § 1954(e)**. Landlord may enter
//! without prior notice "in case of emergency." Post-entry, the
//! landlord must (1) leave WRITTEN NOTICE describing the date,
//! time, and purpose of entry on the premises, AND (2) provide
//! such notice within a REASONABLE TIME. Aggressive or
//! pretextual emergency entries actionable under § 1940.2
//! prohibited harassing acts.
//!
//! **Texas — Tex. Prop. Code § 92.0081**. Emergency entry
//! permitted without prior notice. Texas does NOT impose
//! specific post-entry written-notice obligation but landlord
//! remains liable for actual damages + civil penalty under
//! § 92.0081 for unauthorized entry.
//!
//! **New York — N.Y. Mult. Dwell. Law § 78 + common-law quiet
//! enjoyment**. Emergency entry permitted but landlord must
//! provide post-entry notification to tenant + leave premises
//! secured. Failure to secure exposes landlord to trespass +
//! conversion claims.
//!
//! **Default — common-law quiet enjoyment**. Most states permit
//! emergency entry under common-law necessity doctrine. Most
//! impose reasonable post-entry notice obligation under quiet
//! enjoyment covenant. Failure constitutes actionable trespass +
//! breach of quiet enjoyment.
//!
//! Citations: Cal. Civ. Code §§ 1954(e), 1940.2 (CA emergency
//! entry + harassment); Tex. Prop. Code § 92.0081 (TX
//! unauthorized entry civil penalty); N.Y. Mult. Dwell. Law
//! § 78 (NY emergency entry + securing); common-law necessity
//! doctrine + quiet enjoyment covenant + trespass.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Texas,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LandlordEmergencyEntryInput {
    pub regime: Regime,
    /// Whether the landlord entered without prior notice due to
    /// claimed emergency.
    pub emergency_entry_made_without_prior_notice: bool,
    /// Whether the emergency was ACTUAL (true emergency) vs
    /// pretextual.
    pub emergency_was_actual: bool,
    /// Whether the landlord left written notice on premises
    /// describing date, time, and purpose of entry.
    pub written_notice_describing_date_time_purpose_left: bool,
    /// Whether the landlord provided post-entry notification
    /// within a REASONABLE TIME (CA + most states).
    pub post_entry_notice_provided_within_reasonable_time: bool,
    /// Whether the landlord secured the premises after entry
    /// (NY requirement).
    pub premises_secured_after_entry: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LandlordEmergencyEntryResult {
    pub compliant: bool,
    pub emergency_exception_engaged: bool,
    pub harassment_or_pretext_engaged: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &LandlordEmergencyEntryInput) -> LandlordEmergencyEntryResult {
    match input.regime {
        Regime::California => check_california(input),
        Regime::Texas => check_texas(input),
        Regime::NewYork => check_new_york(input),
        Regime::Default => check_default(input),
    }
}

fn check_california(
    input: &LandlordEmergencyEntryInput,
) -> LandlordEmergencyEntryResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Civ. Code § 1954(e) — landlord may enter without prior notice in case of emergency; post-entry landlord MUST (1) leave WRITTEN NOTICE describing date + time + purpose of entry on premises AND (2) provide such notice within REASONABLE TIME"
            .to_string(),
        "Cal. Civ. Code § 1940.2 — aggressive or pretextual emergency entries actionable as prohibited harassing acts with $2,000 per-violation civil penalty"
            .to_string(),
    ];

    if input.emergency_entry_made_without_prior_notice {
        if !input.emergency_was_actual {
            violations.push(
                "Cal. Civ. Code § 1940.2 — pretextual 'emergency' entry constitutes prohibited harassing act with $2,000 per-violation civil penalty"
                    .to_string(),
            );
        }
        if !input.written_notice_describing_date_time_purpose_left {
            violations.push(
                "Cal. Civ. Code § 1954(e) — landlord MUST leave written notice describing date + time + purpose of emergency entry on the premises"
                    .to_string(),
            );
        }
        if !input.post_entry_notice_provided_within_reasonable_time {
            violations.push(
                "Cal. Civ. Code § 1954(e) — landlord MUST provide post-entry notice to tenant within a REASONABLE TIME"
                    .to_string(),
            );
        }
    }

    let compliant = violations.is_empty();
    LandlordEmergencyEntryResult {
        compliant,
        emergency_exception_engaged: input.emergency_entry_made_without_prior_notice
            && input.emergency_was_actual,
        harassment_or_pretext_engaged: input.emergency_entry_made_without_prior_notice
            && !input.emergency_was_actual,
        violations,
        citation: "Cal. Civ. Code §§ 1954(e), 1940.2",
        notes,
    }
}

fn check_texas(input: &LandlordEmergencyEntryInput) -> LandlordEmergencyEntryResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Tex. Prop. Code § 92.0081 — emergency entry permitted without prior notice; Texas does NOT impose specific post-entry written-notice obligation but landlord remains liable for actual damages + civil penalty for unauthorized entry"
            .to_string(),
        "Tex. Prop. Code § 92.0081 — unauthorized entry civil penalty: actual damages + one month's rent + $1,000 + reasonable attorney fees"
            .to_string(),
    ];

    if input.emergency_entry_made_without_prior_notice && !input.emergency_was_actual {
        violations.push(
            "Tex. Prop. Code § 92.0081 — pretextual 'emergency' entry constitutes unauthorized entry with civil penalty: actual damages + one month's rent + $1,000 + attorney fees"
                .to_string(),
        );
    }

    let compliant = violations.is_empty();
    LandlordEmergencyEntryResult {
        compliant,
        emergency_exception_engaged: input.emergency_entry_made_without_prior_notice
            && input.emergency_was_actual,
        harassment_or_pretext_engaged: input.emergency_entry_made_without_prior_notice
            && !input.emergency_was_actual,
        violations,
        citation: "Tex. Prop. Code § 92.0081",
        notes,
    }
}

fn check_new_york(input: &LandlordEmergencyEntryInput) -> LandlordEmergencyEntryResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "N.Y. Mult. Dwell. Law § 78 + common-law quiet enjoyment — emergency entry permitted but landlord must provide post-entry notification to tenant + leave premises SECURED; failure to secure exposes landlord to trespass + conversion claims"
            .to_string(),
        "NY common-law quiet enjoyment — pretextual emergency entries actionable as breach of covenant of quiet enjoyment + trespass + harassment"
            .to_string(),
    ];

    if input.emergency_entry_made_without_prior_notice {
        if !input.emergency_was_actual {
            violations.push(
                "NY common-law — pretextual 'emergency' entry constitutes breach of covenant of quiet enjoyment + trespass + actionable harassment"
                    .to_string(),
            );
        }
        if !input.post_entry_notice_provided_within_reasonable_time {
            violations.push(
                "N.Y. Mult. Dwell. Law § 78 + common-law — landlord must provide post-entry notification to tenant within reasonable time"
                    .to_string(),
            );
        }
        if !input.premises_secured_after_entry {
            violations.push(
                "N.Y. Mult. Dwell. Law § 78 — landlord must leave premises SECURED after emergency entry; failure exposes to trespass + conversion claims"
                    .to_string(),
            );
        }
    }

    let compliant = violations.is_empty();
    LandlordEmergencyEntryResult {
        compliant,
        emergency_exception_engaged: input.emergency_entry_made_without_prior_notice
            && input.emergency_was_actual,
        harassment_or_pretext_engaged: input.emergency_entry_made_without_prior_notice
            && !input.emergency_was_actual,
        violations,
        citation: "N.Y. Mult. Dwell. Law § 78; common-law quiet enjoyment",
        notes,
    }
}

fn check_default(input: &LandlordEmergencyEntryInput) -> LandlordEmergencyEntryResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "default rule — common-law necessity doctrine permits emergency entry; most states impose reasonable post-entry notice obligation under quiet enjoyment covenant"
            .to_string(),
        "default rule — failure to provide post-entry notice constitutes actionable trespass + breach of quiet enjoyment + common-law harassment"
            .to_string(),
    ];

    if input.emergency_entry_made_without_prior_notice
        && !input.emergency_was_actual
    {
        violations.push(
            "common-law — pretextual 'emergency' entry constitutes trespass + breach of quiet enjoyment + common-law harassment"
                .to_string(),
        );
    }

    if input.emergency_entry_made_without_prior_notice
        && !input.post_entry_notice_provided_within_reasonable_time
    {
        violations.push(
            "common-law quiet enjoyment — landlord must provide reasonable post-entry notice to tenant; failure constitutes actionable breach"
                .to_string(),
        );
    }

    let compliant = violations.is_empty();
    LandlordEmergencyEntryResult {
        compliant,
        emergency_exception_engaged: input.emergency_entry_made_without_prior_notice
            && input.emergency_was_actual,
        harassment_or_pretext_engaged: input.emergency_entry_made_without_prior_notice
            && !input.emergency_was_actual,
        violations,
        citation: "common-law necessity doctrine + quiet enjoyment covenant + trespass",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_compliant() -> LandlordEmergencyEntryInput {
        LandlordEmergencyEntryInput {
            regime: Regime::California,
            emergency_entry_made_without_prior_notice: true,
            emergency_was_actual: true,
            written_notice_describing_date_time_purpose_left: true,
            post_entry_notice_provided_within_reasonable_time: true,
            premises_secured_after_entry: true,
        }
    }

    fn tx_compliant() -> LandlordEmergencyEntryInput {
        let mut i = ca_compliant();
        i.regime = Regime::Texas;
        i
    }

    fn ny_compliant() -> LandlordEmergencyEntryInput {
        let mut i = ca_compliant();
        i.regime = Regime::NewYork;
        i
    }

    fn default_base() -> LandlordEmergencyEntryInput {
        let mut i = ca_compliant();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn ca_compliant_passes() {
        let r = check(&ca_compliant());
        assert!(r.compliant);
        assert!(r.emergency_exception_engaged);
    }

    #[test]
    fn ca_pretext_engages_harassment() {
        let mut i = ca_compliant();
        i.emergency_was_actual = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.harassment_or_pretext_engaged);
        assert!(r.violations.iter().any(|v| v.contains("§ 1940.2") && v.contains("$2,000")));
    }

    #[test]
    fn ca_missing_written_notice_violates() {
        let mut i = ca_compliant();
        i.written_notice_describing_date_time_purpose_left = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 1954(e)") && v.contains("date + time + purpose")));
    }

    #[test]
    fn ca_missing_reasonable_time_notice_violates() {
        let mut i = ca_compliant();
        i.post_entry_notice_provided_within_reasonable_time = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 1954(e)") && v.contains("REASONABLE TIME")));
    }

    #[test]
    fn ca_no_emergency_entry_no_violation() {
        let mut i = ca_compliant();
        i.emergency_entry_made_without_prior_notice = false;
        i.written_notice_describing_date_time_purpose_left = false;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn ca_citation_pins_subsections() {
        let r = check(&ca_compliant());
        assert!(r.citation.contains("§§ 1954(e), 1940.2"));
    }

    #[test]
    fn tx_compliant_passes() {
        let r = check(&tx_compliant());
        assert!(r.compliant);
    }

    #[test]
    fn tx_pretext_engages_civil_penalty() {
        let mut i = tx_compliant();
        i.emergency_was_actual = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 92.0081") && v.contains("$1,000")));
    }

    #[test]
    fn tx_no_specific_post_entry_notice_obligation_note() {
        let r = check(&tx_compliant());
        assert!(r.notes.iter().any(|n| n.contains("does NOT impose specific post-entry written-notice obligation")));
    }

    #[test]
    fn tx_citation_pins_92_0081() {
        let r = check(&tx_compliant());
        assert!(r.citation.contains("§ 92.0081"));
    }

    #[test]
    fn ny_compliant_passes() {
        let r = check(&ny_compliant());
        assert!(r.compliant);
    }

    #[test]
    fn ny_missing_secured_premises_violates() {
        let mut i = ny_compliant();
        i.premises_secured_after_entry = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 78") && v.contains("SECURED")));
    }

    #[test]
    fn ny_missing_post_entry_notice_violates() {
        let mut i = ny_compliant();
        i.post_entry_notice_provided_within_reasonable_time = false;
        let r = check(&i);
        assert!(!r.compliant);
    }

    #[test]
    fn ny_pretext_engages_breach_of_quiet_enjoyment() {
        let mut i = ny_compliant();
        i.emergency_was_actual = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("quiet enjoyment") && v.contains("trespass")));
    }

    #[test]
    fn ny_citation_pins_mdl_78() {
        let r = check(&ny_compliant());
        assert!(r.citation.contains("Mult. Dwell. Law § 78"));
    }

    #[test]
    fn default_compliant_when_actual_emergency_with_notice() {
        let r = check(&default_base());
        assert!(r.compliant);
    }

    #[test]
    fn default_pretext_violates_common_law() {
        let mut i = default_base();
        i.emergency_was_actual = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("trespass") && v.contains("quiet enjoyment")));
    }

    #[test]
    fn default_missing_post_entry_notice_violates() {
        let mut i = default_base();
        i.post_entry_notice_provided_within_reasonable_time = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("common-law quiet enjoyment")));
    }

    #[test]
    fn ny_unique_premises_secured_requirement_invariant() {
        let mut i_ny = ny_compliant();
        i_ny.premises_secured_after_entry = false;
        let r_ny = check(&i_ny);
        assert!(!r_ny.compliant);

        let mut i_ca = ca_compliant();
        i_ca.premises_secured_after_entry = false;
        let r_ca = check(&i_ca);
        assert!(r_ca.compliant);
    }

    #[test]
    fn ca_uniquely_requires_specific_written_notice_invariant() {
        let mut i_ca = ca_compliant();
        i_ca.written_notice_describing_date_time_purpose_left = false;
        let r_ca = check(&i_ca);
        assert!(!r_ca.compliant);

        let mut i_default = default_base();
        i_default.written_notice_describing_date_time_purpose_left = false;
        let r_default = check(&i_default);
        assert!(r_default.compliant);
    }

    #[test]
    fn pretext_engages_across_all_regimes() {
        for regime in [Regime::California, Regime::Texas, Regime::NewYork, Regime::Default] {
            let mut i = ca_compliant();
            i.regime = regime;
            i.emergency_was_actual = false;
            let r = check(&i);
            assert!(!r.compliant);
            assert!(r.harassment_or_pretext_engaged);
        }
    }

    #[test]
    fn emergency_exception_engaged_when_actual_emergency() {
        let r = check(&ca_compliant());
        assert!(r.emergency_exception_engaged);
    }

    #[test]
    fn four_regimes_routed_correctly() {
        for regime in [Regime::California, Regime::Texas, Regime::NewYork, Regime::Default] {
            let mut i = ca_compliant();
            i.regime = regime;
            let r = check(&i);
            let _ = r.compliant;
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn ca_clean_no_violations() {
        let r = check(&ca_compliant());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn ny_multiple_violations_simultaneous() {
        let mut i = ny_compliant();
        i.emergency_was_actual = false;
        i.post_entry_notice_provided_within_reasonable_time = false;
        i.premises_secured_after_entry = false;
        let r = check(&i);
        assert_eq!(r.violations.len(), 3);
    }
}
