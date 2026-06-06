//! Rental property sex offender registry notice disclosure
//! compliance — when must a trader-landlord include a
//! statutory Megan's Law notice in residential rental
//! agreements, and what restrictions apply to the
//! landlord's use of registry information? Trader-landlord
//! operational concern: failure to include statutory
//! Megan's Law disclosure in lease creates per-violation
//! penalty exposure + tenant rescission claim; landlord USE
//! of registry information to deny tenancy or evict
//! triggers Fair Housing Act discrimination liability.
//! Distinct from siblings `fair_chance_housing` (criminal-
//! background screening), `rental_application_denial_
//! disclosure` (denial reason), `tenant_data_privacy`
//! (general data handling).
//!
//! **Three regimes**:
//!
//! **California — Cal. Civ. Code § 2079.10a + Cal. Pen.
//! Code § 290.46**:
//! - Every residential rental agreement must include
//!   **exact statutory Megan's Law notice** directing
//!   tenant to www.meganslaw.ca.gov.
//! - Disclosure required **before tenant signs lease**.
//! - Landlord **cannot use registry information to deny
//!   tenancy or evict** a registered sex offender (Fair
//!   Housing Act + Cal. Gov. Code § 12955 protections).
//! - Disclosure language must be in 10-point type minimum.
//! - Per-violation statutory damages + tenant rescission
//!   right.
//!
//! **New Jersey — N.J.S.A. 2C:7-21 + N.J.S.A. 2C:7-2 (NJ
//! Megan's Law) + NJ Attorney General Guidelines**:
//! - NJ does NOT require landlord disclosure in lease
//!   (registry is publicly accessible via NJ State Police
//!   Sex Offender Internet Registry).
//! - Landlord NOTIFICATION right — community notification
//!   tiers (Tier 1 / Tier 2 / Tier 3) determine notice
//!   scope.
//! - Landlords prohibited from using registry information
//!   to deny tenancy of Tier 1 / Tier 2 offenders (Tier 3
//!   notification permits broader public notice).
//! - NJ Law Against Discrimination (NJ LAD) protects
//!   non-violent registered offenders.
//!
//! **Default — federal Adam Walsh Act (SORNA, 42 USC §
//! 16901 et seq.) + state-specific public registry**:
//! - No federal mandate requiring landlord lease
//!   disclosure.
//! - Federal Fair Housing Act prohibits status-based
//!   discrimination but NOT criminal-history
//!   discrimination per se (HUD 2016 guidance applies
//!   disparate-impact analysis).
//! - State registries publicly accessible via state law
//!   enforcement.
//! - Many states (TX, FL, IL, NY) have no specific
//!   landlord disclosure mandate.
//!
//! Citations: Cal. Civ. Code § 2079.10a; Cal. Pen. Code §
//! 290.46; Cal. Gov. Code § 12955; N.J.S.A. 2C:7-21 + 2C:7-
//! 2; NJ LAD; Adam Walsh Child Protection and Safety Act
//! of 2006 (SORNA, 42 USC § 16901 et seq.); Fair Housing
//! Act (42 USC § 3604); HUD 2016 Guidance on Criminal
//! History.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    NewJersey,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NjOffenderTier {
    /// NJ Tier 1 — low risk; notification only to law
    /// enforcement.
    Tier1Low,
    /// NJ Tier 2 — moderate risk; notification to schools
    /// and community organizations.
    Tier2Moderate,
    /// NJ Tier 3 — high risk; broad public notification
    /// permitted.
    Tier3High,
    /// Not applicable / not in NJ.
    NotApplicable,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalSexOffenderRegistryNoticeInput {
    pub regime: Regime,
    /// Whether statutory Megan's Law notice was included in
    /// lease.
    pub statutory_notice_in_lease: bool,
    /// Whether notice was provided BEFORE tenant signed
    /// lease.
    pub disclosure_before_signing: bool,
    /// Whether notice was in at least 10-point type (CA).
    pub ten_point_type_minimum: bool,
    /// Whether notice directs tenant to www.meganslaw.ca.gov
    /// (CA-specific).
    pub directs_to_ca_meganslaw_website: bool,
    /// Whether landlord used registry information to deny
    /// tenancy or evict.
    pub used_registry_to_deny_or_evict: bool,
    /// NJ offender tier (for NJ regime application).
    pub nj_offender_tier: NjOffenderTier,
    /// Whether tenant has rescinded lease based on missing
    /// disclosure.
    pub tenant_rescinded_lease: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalSexOffenderRegistryNoticeResult {
    pub disclosure_compliant: bool,
    pub statutory_notice_required: bool,
    pub fair_housing_violation: bool,
    pub tenant_rescission_right_engaged: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &RentalSexOffenderRegistryNoticeInput,
) -> RentalSexOffenderRegistryNoticeResult {
    match input.regime {
        Regime::California => check_ca(input),
        Regime::NewJersey => check_nj(input),
        Regime::Default => check_default(input),
    }
}

fn check_ca(input: &RentalSexOffenderRegistryNoticeInput) -> RentalSexOffenderRegistryNoticeResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Civ. Code § 2079.10a — every residential rental agreement must include exact statutory Megan's Law notice directing tenant to www.meganslaw.ca.gov".to_string(),
        "Cal. Pen. Code § 290.46 — California Megan's Law public registry framework; CA Department of Justice maintains registry".to_string(),
        "Cal. Civ. Code § 2079.10a — disclosure required BEFORE tenant signs lease; notice must be in at least 10-point type".to_string(),
        "Cal. Gov. Code § 12955 + Fair Housing Act (42 USC § 3604) — landlord CANNOT use registry information to deny tenancy or evict a registered sex offender; doing so constitutes housing discrimination".to_string(),
        "Cal. Civ. Code § 2079.10a — per-violation statutory damages + tenant rescission right available when disclosure omitted".to_string(),
    ];

    if !input.statutory_notice_in_lease {
        violations.push(
            "Cal. Civ. Code § 2079.10a — every residential rental agreement must include exact statutory Megan's Law notice".to_string(),
        );
    }

    if input.statutory_notice_in_lease && !input.disclosure_before_signing {
        violations.push(
            "Cal. Civ. Code § 2079.10a — Megan's Law disclosure must be provided BEFORE tenant signs lease".to_string(),
        );
    }

    if input.statutory_notice_in_lease && !input.ten_point_type_minimum {
        violations.push(
            "Cal. Civ. Code § 2079.10a — disclosure must be in at least 10-point type".to_string(),
        );
    }

    if input.statutory_notice_in_lease && !input.directs_to_ca_meganslaw_website {
        violations.push(
            "Cal. Civ. Code § 2079.10a — disclosure must direct tenant to www.meganslaw.ca.gov per Cal. Pen. Code § 290.46".to_string(),
        );
    }

    let fair_housing_violation = input.used_registry_to_deny_or_evict;
    if fair_housing_violation {
        violations.push(
            "Cal. Gov. Code § 12955 + Fair Housing Act (42 USC § 3604) — landlord cannot use Megan's Law registry information to deny tenancy or evict a registered sex offender".to_string(),
        );
    }

    let rescission_engaged = !input.statutory_notice_in_lease && input.tenant_rescinded_lease;

    RentalSexOffenderRegistryNoticeResult {
        disclosure_compliant: violations.is_empty(),
        statutory_notice_required: true,
        fair_housing_violation,
        tenant_rescission_right_engaged: rescission_engaged,
        violations,
        citation: "Cal. Civ. Code § 2079.10a; Cal. Pen. Code § 290.46; Cal. Gov. Code § 12955; Fair Housing Act (42 USC § 3604)",
        notes,
    }
}

fn check_nj(input: &RentalSexOffenderRegistryNoticeInput) -> RentalSexOffenderRegistryNoticeResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "N.J.S.A. 2C:7-21 + N.J.S.A. 2C:7-2 (NJ Megan's Law) — NJ does NOT require landlord disclosure in lease; registry is publicly accessible via NJ State Police Sex Offender Internet Registry".to_string(),
        "NJ Attorney General Guidelines — community notification tiers determine notice scope (Tier 1 law enforcement only; Tier 2 schools/community organizations; Tier 3 broad public)".to_string(),
        "NJ Law Against Discrimination (NJ LAD) — landlords prohibited from using registry information to deny tenancy of Tier 1 / Tier 2 offenders; Tier 3 notification permits broader public notice".to_string(),
        "NJ regime distinct from CA — no statutory landlord disclosure mandate in lease".to_string(),
        "NJ Tier 3 (high risk) offenders subject to broad public notification under N.J.S.A. 2C:7-8 — landlord notification permitted".to_string(),
    ];

    let tier_restricts_use = matches!(
        input.nj_offender_tier,
        NjOffenderTier::Tier1Low | NjOffenderTier::Tier2Moderate
    );

    let fair_housing_violation = input.used_registry_to_deny_or_evict && tier_restricts_use;
    if fair_housing_violation {
        violations.push(
            "NJ Law Against Discrimination — landlord prohibited from using Tier 1 / Tier 2 registry information to deny tenancy or evict; Tier 3 high-risk offenders permit broader public notice".to_string(),
        );
    }

    RentalSexOffenderRegistryNoticeResult {
        disclosure_compliant: violations.is_empty(),
        statutory_notice_required: false,
        fair_housing_violation,
        tenant_rescission_right_engaged: false,
        violations,
        citation: "N.J.S.A. 2C:7-21 + N.J.S.A. 2C:7-2; NJ Attorney General Guidelines; NJ Law Against Discrimination",
        notes,
    }
}

fn check_default(
    input: &RentalSexOffenderRegistryNoticeInput,
) -> RentalSexOffenderRegistryNoticeResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Adam Walsh Child Protection and Safety Act of 2006 (SORNA, 42 USC § 16901 et seq.) — federal sex offender registration framework; state registries publicly accessible via state law enforcement".to_string(),
        "Default — no federal mandate requiring landlord lease disclosure".to_string(),
        "Fair Housing Act (42 USC § 3604) — prohibits status-based discrimination but NOT criminal-history discrimination per se; HUD 2016 Guidance applies disparate-impact analysis".to_string(),
        "Default — many states (TX, FL, IL, NY) have no specific landlord disclosure mandate; verify local jurisdiction".to_string(),
        "Default — HUD 2016 Guidance on Use of Criminal Records by Providers of Housing — landlord blanket bans on criminal history may have disparate-impact discriminatory effect".to_string(),
    ];

    let fair_housing_violation = input.used_registry_to_deny_or_evict;
    if fair_housing_violation {
        violations.push(
            "Fair Housing Act (42 USC § 3604) + HUD 2016 Guidance — landlord blanket use of sex offender registry to deny tenancy may have disparate-impact discriminatory effect; individualized assessment required".to_string(),
        );
    }

    RentalSexOffenderRegistryNoticeResult {
        disclosure_compliant: violations.is_empty(),
        statutory_notice_required: false,
        fair_housing_violation,
        tenant_rescission_right_engaged: false,
        violations,
        citation: "Adam Walsh Act (SORNA, 42 USC § 16901); Fair Housing Act (42 USC § 3604); HUD 2016 Guidance on Criminal History",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_compliant() -> RentalSexOffenderRegistryNoticeInput {
        RentalSexOffenderRegistryNoticeInput {
            regime: Regime::California,
            statutory_notice_in_lease: true,
            disclosure_before_signing: true,
            ten_point_type_minimum: true,
            directs_to_ca_meganslaw_website: true,
            used_registry_to_deny_or_evict: false,
            nj_offender_tier: NjOffenderTier::NotApplicable,
            tenant_rescinded_lease: false,
        }
    }

    fn nj_compliant() -> RentalSexOffenderRegistryNoticeInput {
        let mut i = ca_compliant();
        i.regime = Regime::NewJersey;
        i.nj_offender_tier = NjOffenderTier::Tier3High;
        i
    }

    fn default_compliant() -> RentalSexOffenderRegistryNoticeInput {
        let mut i = ca_compliant();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn ca_compliant_passes() {
        let r = check(&ca_compliant());
        assert!(r.disclosure_compliant);
        assert!(r.statutory_notice_required);
    }

    #[test]
    fn ca_missing_notice_violation() {
        let mut i = ca_compliant();
        i.statutory_notice_in_lease = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 2079.10a") && v.contains("exact statutory")));
    }

    #[test]
    fn ca_disclosure_after_signing_violation() {
        let mut i = ca_compliant();
        i.disclosure_before_signing = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("BEFORE tenant signs")));
    }

    #[test]
    fn ca_below_10_point_type_violation() {
        let mut i = ca_compliant();
        i.ten_point_type_minimum = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.violations.iter().any(|v| v.contains("10-point type")));
    }

    #[test]
    fn ca_missing_website_direction_violation() {
        let mut i = ca_compliant();
        i.directs_to_ca_meganslaw_website = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.violations.iter().any(|v| v.contains("meganslaw.ca.gov")));
    }

    #[test]
    fn ca_using_registry_to_deny_fair_housing_violation() {
        let mut i = ca_compliant();
        i.used_registry_to_deny_or_evict = true;
        let r = check(&i);
        assert!(r.fair_housing_violation);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 12955") && v.contains("Fair Housing Act")));
    }

    #[test]
    fn ca_tenant_rescission_right_engaged_when_notice_omitted() {
        let mut i = ca_compliant();
        i.statutory_notice_in_lease = false;
        i.tenant_rescinded_lease = true;
        let r = check(&i);
        assert!(r.tenant_rescission_right_engaged);
    }

    #[test]
    fn ca_tenant_rescission_not_engaged_when_notice_provided() {
        let mut i = ca_compliant();
        i.tenant_rescinded_lease = true;
        let r = check(&i);
        assert!(!r.tenant_rescission_right_engaged);
    }

    #[test]
    fn nj_no_disclosure_required_compliant() {
        let mut i = nj_compliant();
        i.statutory_notice_in_lease = false;
        i.disclosure_before_signing = false;
        let r = check(&i);
        assert!(r.disclosure_compliant);
        assert!(!r.statutory_notice_required);
    }

    #[test]
    fn nj_tier_1_use_to_deny_violation() {
        let mut i = nj_compliant();
        i.nj_offender_tier = NjOffenderTier::Tier1Low;
        i.used_registry_to_deny_or_evict = true;
        let r = check(&i);
        assert!(r.fair_housing_violation);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("NJ Law Against Discrimination") && v.contains("Tier 1")));
    }

    #[test]
    fn nj_tier_2_use_to_deny_violation() {
        let mut i = nj_compliant();
        i.nj_offender_tier = NjOffenderTier::Tier2Moderate;
        i.used_registry_to_deny_or_evict = true;
        let r = check(&i);
        assert!(r.fair_housing_violation);
    }

    #[test]
    fn nj_tier_3_use_to_deny_no_violation() {
        let mut i = nj_compliant();
        i.nj_offender_tier = NjOffenderTier::Tier3High;
        i.used_registry_to_deny_or_evict = true;
        let r = check(&i);
        assert!(!r.fair_housing_violation);
    }

    #[test]
    fn default_no_disclosure_required_compliant() {
        let mut i = default_compliant();
        i.statutory_notice_in_lease = false;
        let r = check(&i);
        assert!(r.disclosure_compliant);
        assert!(!r.statutory_notice_required);
    }

    #[test]
    fn default_blanket_use_disparate_impact_violation() {
        let mut i = default_compliant();
        i.used_registry_to_deny_or_evict = true;
        let r = check(&i);
        assert!(r.fair_housing_violation);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("HUD 2016 Guidance") && v.contains("disparate-impact")));
    }

    #[test]
    fn citation_pins_ca_authority() {
        let r = check(&ca_compliant());
        assert!(r.citation.contains("§ 2079.10a"));
        assert!(r.citation.contains("§ 290.46"));
        assert!(r.citation.contains("§ 12955"));
        assert!(r.citation.contains("§ 3604"));
    }

    #[test]
    fn citation_pins_nj_authority() {
        let r = check(&nj_compliant());
        assert!(r.citation.contains("2C:7-21"));
        assert!(r.citation.contains("2C:7-2"));
        assert!(r.citation.contains("NJ Law Against Discrimination"));
    }

    #[test]
    fn citation_pins_default_authority() {
        let r = check(&default_compliant());
        assert!(r.citation.contains("Adam Walsh Act"));
        assert!(r.citation.contains("SORNA"));
        assert!(r.citation.contains("§ 16901"));
        assert!(r.citation.contains("HUD 2016 Guidance"));
    }

    #[test]
    fn note_pins_ca_exact_statutory_notice_requirement() {
        let r = check(&ca_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("exact statutory") && n.contains("meganslaw.ca.gov")));
    }

    #[test]
    fn note_pins_ca_anti_discrimination_protection() {
        let r = check(&ca_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("CANNOT use registry") && n.contains("§ 12955")));
    }

    #[test]
    fn note_pins_nj_three_tier_framework() {
        let r = check(&nj_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Tier 1") && n.contains("Tier 2") && n.contains("Tier 3")));
    }

    #[test]
    fn note_pins_default_sorna_42_usc_16901() {
        let r = check(&default_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("SORNA") && n.contains("42 USC § 16901")));
    }

    #[test]
    fn note_pins_default_hud_2016_guidance() {
        let r = check(&default_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("HUD 2016 Guidance") && n.contains("disparate-impact")));
    }

    #[test]
    fn ca_uniquely_requires_lease_disclosure_invariant() {
        let r_ca = check(&ca_compliant());
        let r_nj = check(&nj_compliant());
        let r_default = check(&default_compliant());
        assert!(r_ca.statutory_notice_required);
        assert!(!r_nj.statutory_notice_required);
        assert!(!r_default.statutory_notice_required);
    }

    #[test]
    fn nj_tier_truth_table() {
        for (tier, exp_violation_when_deny) in [
            (NjOffenderTier::Tier1Low, true),
            (NjOffenderTier::Tier2Moderate, true),
            (NjOffenderTier::Tier3High, false),
        ] {
            let mut i = nj_compliant();
            i.nj_offender_tier = tier;
            i.used_registry_to_deny_or_evict = true;
            let r = check(&i);
            assert_eq!(
                r.fair_housing_violation, exp_violation_when_deny,
                "tier={:?} expected violation={}",
                tier, exp_violation_when_deny
            );
        }
    }

    #[test]
    fn multiple_ca_violations_stack() {
        let mut i = ca_compliant();
        i.statutory_notice_in_lease = false;
        i.used_registry_to_deny_or_evict = true;
        let r = check(&i);
        assert!(r.violations.len() >= 2);
    }

    #[test]
    fn fair_housing_violation_invariant_across_regimes() {
        let mut i_ca = ca_compliant();
        i_ca.used_registry_to_deny_or_evict = true;
        let r_ca = check(&i_ca);
        assert!(r_ca.fair_housing_violation);

        let mut i_default = default_compliant();
        i_default.used_registry_to_deny_or_evict = true;
        let r_default = check(&i_default);
        assert!(r_default.fair_housing_violation);
    }

    #[test]
    fn rescission_only_engages_when_notice_omitted_and_tenant_invokes_invariant() {
        let cases: [(bool, bool, bool); 4] = [
            (true, false, false),
            (true, true, false),
            (false, false, false),
            (false, true, true),
        ];
        for (notice, tenant_rescinded, exp_engaged) in cases {
            let mut i = ca_compliant();
            i.statutory_notice_in_lease = notice;
            i.tenant_rescinded_lease = tenant_rescinded;
            let r = check(&i);
            assert_eq!(
                r.tenant_rescission_right_engaged, exp_engaged,
                "notice={} tenant_rescinded={}",
                notice, tenant_rescinded
            );
        }
    }
}
