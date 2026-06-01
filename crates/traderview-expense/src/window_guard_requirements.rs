//! State window-guard landlord compliance check.
//!
//! Two jurisdictions have statutory window-guard requirements for child
//! safety in multi-family housing. They differ on the trigger and the
//! cost-allocation model:
//!
//! **New York City (NYC Admin Code § 27-2043.1 + NYC Health Code §
//! 131.15)** — proactive / mandatory model. Buildings with **3+ units**
//! must install approved window guards in every unit where a child
//! **age 10 or under** resides AND in all public-hallway windows of the
//! building. Triggered automatically by the presence of a qualifying
//! child — no tenant request required. **Annual notice** must be sent to
//! every tenant between **Jan 1–15** asking whether a qualifying child
//! resides; if the tenant fails to return the form by Feb 15 the landlord
//! must inspect by Mar 1. Window-guard specs: ≥ 15 inches tall, ≤ 4.5"
//! horizontal bar spacing. **First-floor exception**: one window per
//! ground-floor unit may remain unguarded as an emergency exit. **Cost**
//! is borne entirely by the landlord — no pass-through to tenant.
//!
//! **New Jersey (N.J.S.A. 55:13A-7.13 + 7.14)** — reactive / on-request
//! model. Multiple-dwelling owners must install approved child-protection
//! window guards **upon the written request of a tenant** of a unit in
//! which a child ≤10 resides (or in public halls if any unit has such
//! child). **Lease notice** (N.J.S.A. 55:13A-7.14) — every multiple-
//! dwelling lease must contain conspicuous bold-face notice of the
//! availability of window guards. **Cost** — up to **$20 per window
//! guard** may be passed through to the requesting tenant.
//! **Biannual maintenance inspection** — at least twice per year the
//! landlord must inspect installed guards.
//!
//! **Default** — no statewide window-guard statute; common-law habit-
//! ability + landlord-liability rules apply.
//!
//! Citations: NYC Admin Code § 27-2043.1 (Local Law 31 of 1976); NYC
//! Health Code § 131.15 (annual notice + specs); N.J.S.A. 55:13A-7.13
//! (mandatory-on-request installation + maintenance); N.J.S.A.
//! 55:13A-7.14 (lease notice requirement); N.J. Admin. Code § 5:10-27.1
//! (NJ regulatory implementation).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    NewYorkCity,
    NewJersey,
    Default,
}

impl Regime {
    pub fn for_jurisdiction(state: &str, city: &str) -> Self {
        let st = state.trim().to_ascii_uppercase();
        let ct = city.trim().to_ascii_lowercase();
        match (st.as_str(), ct.as_str()) {
            ("NY", "new york") | ("NY", "nyc") => Self::NewYorkCity,
            ("NJ", _) => Self::NewJersey,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WindowGuardInput {
    pub regime: Regime,
    /// Multiple dwelling = 3+ apartments (NYC) or NJ-defined multiple
    /// dwelling under N.J.S.A. 55:13A-3(k).
    pub is_multiple_dwelling: bool,
    /// Whether a child age ≤ 10 resides in the unit.
    pub child_10_or_under_present: bool,
    /// NJ-only: written tenant request triggers the obligation.
    pub tenant_requested_in_writing: bool,
    /// NYC-only: annual Jan 1–15 notice form was sent to tenant.
    pub annual_notice_provided: bool,
    /// Whether the landlord has actually installed approved window guards.
    pub guards_installed: bool,
    /// NJ-only: lease contains the required N.J.S.A. 55:13A-7.14
    /// conspicuous bold-face notice of guard availability.
    pub lease_contains_required_notice: bool,
    /// NJ-only: dollars per guard passed through to tenant. § 7.13 caps
    /// at $20 per window guard.
    pub cost_passthrough_per_guard_dollars: u32,
    /// NJ-only: biannual maintenance inspection performed in the past
    /// year.
    pub biannual_maintenance_inspected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    None,
    InstallationNotPerformedWhenRequired,
    MissingAnnualNotice,
    MissingLeaseNotice,
    ExcessiveCostPassthrough,
    MissingBiannualInspection,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct WindowGuardResult {
    pub regime: Regime,
    pub installation_required: bool,
    pub annual_notice_required: bool,
    pub biannual_inspection_required: bool,
    pub max_cost_passthrough_per_guard_dollars: u32,
    pub violation: ViolationType,
    pub landlord_compliant: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn check(input: &WindowGuardInput) -> WindowGuardResult {
    match input.regime {
        Regime::NewYorkCity => nyc_check(input),
        Regime::NewJersey => nj_check(input),
        Regime::Default => default_check(input),
    }
}

fn nyc_check(input: &WindowGuardInput) -> WindowGuardResult {
    if !input.is_multiple_dwelling {
        return WindowGuardResult {
            regime: Regime::NewYorkCity,
            installation_required: false,
            annual_notice_required: false,
            biannual_inspection_required: false,
            max_cost_passthrough_per_guard_dollars: 0,
            violation: ViolationType::None,
            landlord_compliant: true,
            citation:
                "NYC Admin Code § 27-2043.1 — applies only to buildings with 3 or more apartments",
            note: "Building is not a multiple dwelling (< 3 units); NYC window-guard requirement does not apply.".to_string(),
        };
    }
    // NYC requires annual notice to ALL tenants (whether a child is
    // present or not). Missing notice is its own violation.
    if !input.annual_notice_provided {
        return WindowGuardResult {
            regime: Regime::NewYorkCity,
            installation_required: input.child_10_or_under_present,
            annual_notice_required: true,
            biannual_inspection_required: false,
            max_cost_passthrough_per_guard_dollars: 0,
            violation: ViolationType::MissingAnnualNotice,
            landlord_compliant: false,
            citation:
                "NYC Health Code § 131.15 — annual Jan 1-15 window-guard notice to all tenants required",
            note: "NYC requires annual Jan 1-15 notice to every tenant of every multiple dwelling, regardless of child presence. Notice was not provided.".to_string(),
        };
    }
    if input.child_10_or_under_present && !input.guards_installed {
        return WindowGuardResult {
            regime: Regime::NewYorkCity,
            installation_required: true,
            annual_notice_required: true,
            biannual_inspection_required: false,
            max_cost_passthrough_per_guard_dollars: 0,
            violation: ViolationType::InstallationNotPerformedWhenRequired,
            landlord_compliant: false,
            citation:
                "NYC Admin Code § 27-2043.1 — mandatory installation when child age 10 or under resides in unit",
            note: "Child age 10 or under resides in unit; landlord must install approved window guards (≥ 15-inch height, ≤ 4.5-inch bar spacing) on every window except one ground-floor emergency-exit window. Installation has not been performed.".to_string(),
        };
    }
    WindowGuardResult {
        regime: Regime::NewYorkCity,
        installation_required: input.child_10_or_under_present,
        annual_notice_required: true,
        biannual_inspection_required: false,
        max_cost_passthrough_per_guard_dollars: 0,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation:
            "NYC Admin Code § 27-2043.1 + NYC Health Code § 131.15 — proactive/mandatory model; landlord bears cost",
        note: "NYC window-guard compliance OK: annual notice provided and (if child present) guards installed.".to_string(),
    }
}

fn nj_check(input: &WindowGuardInput) -> WindowGuardResult {
    if !input.is_multiple_dwelling {
        return WindowGuardResult {
            regime: Regime::NewJersey,
            installation_required: false,
            annual_notice_required: false,
            biannual_inspection_required: false,
            max_cost_passthrough_per_guard_dollars: 20,
            violation: ViolationType::None,
            landlord_compliant: true,
            citation:
                "N.J.S.A. 55:13A-7.13 — applies only to multiple dwellings under N.J.S.A. 55:13A-3(k)",
            note: "Building is not a multiple dwelling; NJ window-guard requirement does not apply.".to_string(),
        };
    }
    // NJ § 7.14: lease must contain conspicuous bold-face notice of
    // window-guard availability — regardless of whether a tenant has
    // actually requested guards.
    if !input.lease_contains_required_notice {
        return WindowGuardResult {
            regime: Regime::NewJersey,
            installation_required: false,
            annual_notice_required: false,
            biannual_inspection_required: false,
            max_cost_passthrough_per_guard_dollars: 20,
            violation: ViolationType::MissingLeaseNotice,
            landlord_compliant: false,
            citation: "N.J.S.A. 55:13A-7.14 — lease must contain conspicuous bold-face notice advising tenants of window-guard availability",
            note: "Lease lacks the required N.J.S.A. 55:13A-7.14 conspicuous bold-face notice of window-guard availability.".to_string(),
        };
    }
    // § 7.13(b): $20-per-guard cap on cost pass-through.
    if input.cost_passthrough_per_guard_dollars > 20 {
        return WindowGuardResult {
            regime: Regime::NewJersey,
            installation_required: false,
            annual_notice_required: false,
            biannual_inspection_required: false,
            max_cost_passthrough_per_guard_dollars: 20,
            violation: ViolationType::ExcessiveCostPassthrough,
            landlord_compliant: false,
            citation: "N.J.S.A. 55:13A-7.13(b) — landlord may pass through at most $20 per window guard installed",
            note: format!(
                "Cost pass-through of ${} per window guard exceeds the $20 statutory cap.",
                input.cost_passthrough_per_guard_dollars
            ),
        };
    }
    let installation_required =
        input.tenant_requested_in_writing && input.child_10_or_under_present;
    if installation_required && !input.guards_installed {
        return WindowGuardResult {
            regime: Regime::NewJersey,
            installation_required: true,
            annual_notice_required: false,
            biannual_inspection_required: true,
            max_cost_passthrough_per_guard_dollars: 20,
            violation: ViolationType::InstallationNotPerformedWhenRequired,
            landlord_compliant: false,
            citation: "N.J.S.A. 55:13A-7.13(a) — installation mandatory upon written tenant request when child age 10 or under resides",
            note: "Written tenant request received and child age 10 or under resides in unit, but window guards have not been installed.".to_string(),
        };
    }
    // Biannual maintenance inspection required if guards are installed.
    if input.guards_installed && !input.biannual_maintenance_inspected {
        return WindowGuardResult {
            regime: Regime::NewJersey,
            installation_required,
            annual_notice_required: false,
            biannual_inspection_required: true,
            max_cost_passthrough_per_guard_dollars: 20,
            violation: ViolationType::MissingBiannualInspection,
            landlord_compliant: false,
            citation: "N.J.S.A. 55:13A-7.13 — landlord must inspect installed window guards at least twice annually",
            note: "Guards installed but biannual maintenance inspection not performed within the past year.".to_string(),
        };
    }
    WindowGuardResult {
        regime: Regime::NewJersey,
        installation_required,
        annual_notice_required: false,
        biannual_inspection_required: input.guards_installed,
        max_cost_passthrough_per_guard_dollars: 20,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: "N.J.S.A. 55:13A-7.13 + 7.14 — reactive/on-request model; up to $20/guard cost pass-through allowed",
        note: "NJ window-guard compliance OK: lease notice present; installation upon request satisfied; biannual inspection performed.".to_string(),
    }
}

fn default_check(_input: &WindowGuardInput) -> WindowGuardResult {
    WindowGuardResult {
        regime: Regime::Default,
        installation_required: false,
        annual_notice_required: false,
        biannual_inspection_required: false,
        max_cost_passthrough_per_guard_dollars: 0,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation:
            "No statewide window-guard statute — common-law habitability + landlord-liability rules apply",
        note: "Default regime: lease terms and common-law habitability govern. State has no statutory window-guard requirement.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        multi: bool,
        child: bool,
        requested: bool,
        annual_notice: bool,
        installed: bool,
        lease_notice: bool,
        cost: u32,
        biannual: bool,
    ) -> WindowGuardInput {
        WindowGuardInput {
            regime,
            is_multiple_dwelling: multi,
            child_10_or_under_present: child,
            tenant_requested_in_writing: requested,
            annual_notice_provided: annual_notice,
            guards_installed: installed,
            lease_contains_required_notice: lease_notice,
            cost_passthrough_per_guard_dollars: cost,
            biannual_maintenance_inspected: biannual,
        }
    }

    #[test]
    fn nyc_not_multiple_dwelling_no_obligation() {
        let r = check(&input(
            Regime::NewYorkCity,
            false,
            true,
            false,
            false,
            false,
            false,
            0,
            false,
        ));
        assert!(!r.installation_required);
        assert!(r.landlord_compliant);
        assert!(r.citation.contains("3 or more apartments"));
    }

    #[test]
    fn nyc_missing_annual_notice_violation() {
        let r = check(&input(
            Regime::NewYorkCity,
            true,
            false,
            false,
            false,
            false,
            false,
            0,
            false,
        ));
        assert_eq!(r.violation, ViolationType::MissingAnnualNotice);
        assert!(!r.landlord_compliant);
        assert!(r.citation.contains("Health Code § 131.15"));
        assert!(r.citation.contains("Jan 1-15"));
    }

    #[test]
    fn nyc_child_present_no_install_violation() {
        let r = check(&input(
            Regime::NewYorkCity,
            true,
            true,
            false,
            true,
            false,
            false,
            0,
            false,
        ));
        assert_eq!(r.violation, ViolationType::InstallationNotPerformedWhenRequired);
        assert!(r.installation_required);
        assert!(r.citation.contains("§ 27-2043.1"));
        assert!(r.note.contains("15-inch"));
        assert!(r.note.contains("4.5-inch"));
    }

    #[test]
    fn nyc_child_present_installed_with_notice_compliant() {
        let r = check(&input(
            Regime::NewYorkCity,
            true,
            true,
            false,
            true,
            true,
            false,
            0,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.landlord_compliant);
        assert!(r.installation_required);
    }

    #[test]
    fn nyc_no_child_notice_provided_compliant() {
        let r = check(&input(
            Regime::NewYorkCity,
            true,
            false,
            false,
            true,
            false,
            false,
            0,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(!r.installation_required);
    }

    #[test]
    fn nyc_landlord_bears_cost_zero_passthrough() {
        let r = check(&input(
            Regime::NewYorkCity,
            true,
            true,
            false,
            true,
            true,
            false,
            0,
            false,
        ));
        assert_eq!(r.max_cost_passthrough_per_guard_dollars, 0);
    }

    #[test]
    fn nj_missing_lease_notice_violation() {
        let r = check(&input(
            Regime::NewJersey,
            true,
            false,
            false,
            false,
            false,
            false,
            0,
            false,
        ));
        assert_eq!(r.violation, ViolationType::MissingLeaseNotice);
        assert!(r.citation.contains("55:13A-7.14"));
        assert!(r.citation.contains("bold-face"));
    }

    #[test]
    fn nj_excessive_cost_passthrough_violation() {
        let r = check(&input(
            Regime::NewJersey,
            true,
            false,
            false,
            false,
            false,
            true,
            25,
            false,
        ));
        assert_eq!(r.violation, ViolationType::ExcessiveCostPassthrough);
        assert!(r.note.contains("$25 per window guard exceeds the $20"));
    }

    #[test]
    fn nj_cost_at_20_boundary_compliant() {
        let r = check(&input(
            Regime::NewJersey,
            true,
            false,
            false,
            false,
            false,
            true,
            20,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn nj_request_but_no_install_violation() {
        let r = check(&input(
            Regime::NewJersey,
            true,
            true,
            true,
            false,
            false,
            true,
            0,
            false,
        ));
        assert_eq!(r.violation, ViolationType::InstallationNotPerformedWhenRequired);
        assert!(r.installation_required);
        assert!(r.citation.contains("written tenant request"));
    }

    #[test]
    fn nj_no_request_no_install_no_violation() {
        let r = check(&input(
            Regime::NewJersey,
            true,
            true,
            false,
            false,
            false,
            true,
            0,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(!r.installation_required);
    }

    #[test]
    fn nj_installed_no_biannual_inspection_violation() {
        let r = check(&input(
            Regime::NewJersey,
            true,
            true,
            true,
            false,
            true,
            true,
            0,
            false,
        ));
        assert_eq!(r.violation, ViolationType::MissingBiannualInspection);
        assert!(r.citation.contains("twice annually"));
    }

    #[test]
    fn nj_installed_with_inspection_compliant() {
        let r = check(&input(
            Regime::NewJersey,
            true,
            true,
            true,
            false,
            true,
            true,
            20,
            true,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.landlord_compliant);
        assert_eq!(r.max_cost_passthrough_per_guard_dollars, 20);
    }

    #[test]
    fn nj_passes_cost_through_to_tenant() {
        let r = check(&input(
            Regime::NewJersey,
            true,
            false,
            false,
            false,
            false,
            true,
            15,
            false,
        ));
        assert_eq!(r.max_cost_passthrough_per_guard_dollars, 20);
    }

    #[test]
    fn default_no_obligation() {
        let r = check(&input(
            Regime::Default,
            true,
            true,
            true,
            false,
            false,
            false,
            0,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn nyc_proactive_vs_nj_reactive_model() {
        // NYC: child present triggers automatic install obligation
        // (assuming annual notice was provided).
        let nyc = check(&input(
            Regime::NewYorkCity,
            true,
            true,
            false,
            true,
            false,
            false,
            0,
            false,
        ));
        assert!(nyc.installation_required);
        assert_eq!(
            nyc.violation,
            ViolationType::InstallationNotPerformedWhenRequired
        );

        // NJ: same child-present scenario WITHOUT written tenant request
        // → no installation required.
        let nj = check(&input(
            Regime::NewJersey,
            true,
            true,
            false,
            false,
            false,
            true,
            0,
            false,
        ));
        assert!(!nj.installation_required);
        assert_eq!(nj.violation, ViolationType::None);
    }

    #[test]
    fn jurisdiction_routing_nyc_nj_default() {
        assert_eq!(
            Regime::for_jurisdiction("NY", "New York"),
            Regime::NewYorkCity
        );
        assert_eq!(
            Regime::for_jurisdiction("NY", "NYC"),
            Regime::NewYorkCity
        );
        assert_eq!(
            Regime::for_jurisdiction("NY", "Buffalo"),
            Regime::Default
        );
        assert_eq!(
            Regime::for_jurisdiction("NJ", "Newark"),
            Regime::NewJersey
        );
        assert_eq!(
            Regime::for_jurisdiction("CA", "Los Angeles"),
            Regime::Default
        );
    }

    #[test]
    fn jurisdiction_routing_case_insensitive() {
        assert_eq!(
            Regime::for_jurisdiction("ny", "new york"),
            Regime::NewYorkCity
        );
        assert_eq!(
            Regime::for_jurisdiction("nj", "any"),
            Regime::NewJersey
        );
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let nyc = check(&input(
            Regime::NewYorkCity,
            true,
            true,
            false,
            true,
            true,
            false,
            0,
            false,
        ));
        assert!(nyc.citation.contains("§ 27-2043.1"));

        let nj = check(&input(
            Regime::NewJersey,
            true,
            true,
            true,
            false,
            true,
            true,
            20,
            true,
        ));
        assert!(nj.citation.contains("55:13A-7.13"));
        assert!(nj.citation.contains("7.14"));
    }

    #[test]
    fn nyc_annual_notice_required_nj_not() {
        let nyc = check(&input(
            Regime::NewYorkCity,
            true,
            true,
            false,
            true,
            true,
            false,
            0,
            false,
        ));
        let nj = check(&input(
            Regime::NewJersey,
            true,
            true,
            true,
            false,
            true,
            true,
            20,
            true,
        ));
        assert!(nyc.annual_notice_required);
        assert!(!nj.annual_notice_required);
    }

    #[test]
    fn nj_biannual_inspection_only_when_installed() {
        let nj_installed = check(&input(
            Regime::NewJersey,
            true,
            true,
            true,
            false,
            true,
            true,
            20,
            true,
        ));
        let nj_not_installed = check(&input(
            Regime::NewJersey,
            true,
            false,
            false,
            false,
            false,
            true,
            0,
            false,
        ));
        assert!(nj_installed.biannual_inspection_required);
        assert!(!nj_not_installed.biannual_inspection_required);
    }
}
