//! Federal FCC Over-the-Air Reception Devices (OTARD) rule
//! compliance check for tenant antenna / satellite-dish / fixed-
//! wireless-receiver installation.
//!
//! 47 CFR § 1.4000 preempts state and local laws AND landlord /
//! HOA restrictions that impair a tenant's ability to install,
//! maintain, or use a covered antenna in an area within the
//! tenant's exclusive use or control. Critical for traders who
//! depend on satellite-TV market data feeds, broadcast-TV business
//! news, or fixed-wireless broadband from a roof-mounted relay.
//!
//! Five protected antenna types under § 1.4000(a):
//!
//!   - **DBS satellite dish** (direct broadcast satellite service)
//!     — ≤ 1 meter in diameter (no size limit in Alaska).
//!   - **MMDS antenna** (multipoint distribution service / wireless
//!     cable) — ≤ 1 meter in diameter or diagonal measurement.
//!   - **Broadcast TV antenna** — any size; receives over-the-air
//!     television.
//!   - **Mast supporting** any of the above antenna types.
//!   - **Fixed wireless hub / relay antenna** (added by the 2021
//!     FCC Report and Order, eff. 2021-03-29) — serves broadband-
//!     only fixed-wireless service to one or more on-premises
//!     customer locations.
//!
//! Tenant-exclusive-use scope: the rule protects installations in
//! areas where the antenna user has direct or indirect ownership
//! or leasehold interest AND exclusive use or control. Balconies,
//! patios, single-tenant rooftops qualify. Common areas (shared
//! rooftops, exterior walls of apartment buildings, hallways) are
//! OUTSIDE the rule's scope — landlord retains restriction
//! authority on those.
//!
//! Three permissible-restriction categories under § 1.4000(b):
//!
//!   1. **Safety** — restriction necessary to accomplish a clearly
//!      defined, legitimate safety objective; must be applied non-
//!      discriminatorily to comparable devices.
//!   2. **Historic preservation** — restriction necessary to
//!      preserve a property on the National Register of Historic
//!      Places; cannot be more restrictive than restrictions on
//!      modern appurtenances.
//!   3. **No-impairment standard** — any restriction must NOT
//!      (i) unreasonably delay or prevent installation,
//!      (ii) unreasonably increase the cost, OR
//!      (iii) preclude reception or transmission of an acceptable-
//!      quality signal.
//!
//! Burden of proof rests on the party seeking to impose or
//! maintain the restriction (§ 1.4000(c)). Enforcement is stayed
//! pending FCC or court review.
//!
//! Aesthetic restrictions, blanket prohibitions, "no antennas of
//! any kind" lease clauses, and pre-installation approval
//! requirements that unreasonably delay are all NOT permissible.
//!
//! Citations: 47 CFR § 1.4000(a)(1) (protected antenna types);
//! § 1.4000(a)(2)(i)–(iii) (size limits + tenant-exclusive-use
//! scope); § 1.4000(b)(1) (safety exception); § 1.4000(b)(2)
//! (historic-preservation exception); § 1.4000(a)(3)–(b)(3)
//! (no-impairment standard); § 1.4000(c) (burden of proof on
//! restricting party); FCC 21-10 (2021 Report and Order — fixed
//! wireless hub/relay expansion, effective 2021-03-29); Section
//! 207 of the Telecommunications Act of 1996, Pub. L. 104-104.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AntennaType {
    /// Direct broadcast satellite dish (DBS) — ≤ 1 meter in
    /// diameter (no size limit in Alaska). Examples: DirecTV, Dish
    /// Network.
    DBS,
    /// Multipoint distribution service antenna (MMDS / wireless
    /// cable) — ≤ 1 meter in diameter or diagonal measurement.
    MMDS,
    /// Television broadcast antenna — any size; receives over-the-
    /// air TV signals.
    BroadcastTV,
    /// Fixed-wireless hub or relay antenna (2021 OTARD expansion)
    /// — serves broadband-only fixed-wireless service to on-
    /// premises customer locations.
    FixedWirelessHubRelay2021,
    /// Antenna outside OTARD scope (oversized dish > 1 m, amateur
    /// radio antenna, AM/FM broadcast receiver, etc.).
    OutsideOTARDScope,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InstallationLocation {
    /// Tenant's exclusive use or control (patio, balcony,
    /// single-tenant rooftop, owned condo unit interior facing
    /// exterior). § 1.4000(a)(2) covers.
    TenantExclusiveUse,
    /// Common area (shared rooftop, exterior wall of apartment
    /// building, hallway, shared yard). OUTSIDE § 1.4000 scope.
    CommonArea,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RestrictionType {
    /// No restriction imposed.
    None,
    /// § 1.4000(b)(1) safety restriction — potentially permissible.
    Safety,
    /// § 1.4000(b)(2) historic-preservation restriction —
    /// potentially permissible.
    HistoricPreservation,
    /// Aesthetic restriction — NOT permissible under OTARD.
    Aesthetic,
    /// Blanket prohibition / "no antennas of any kind" —
    /// NOT permissible.
    BlanketProhibition,
    /// Pre-installation approval requirement that unreasonably
    /// delays — NOT permissible.
    PreApprovalDelay,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub antenna_type: AntennaType,
    pub installation_location: InstallationLocation,
    pub restriction_type: RestrictionType,
    /// Whether the restriction unreasonably DELAYS or prevents
    /// installation — § 1.4000(a)(3)(i).
    pub restriction_unreasonably_delays: bool,
    /// Whether the restriction unreasonably INCREASES the cost of
    /// installation, maintenance, or use — § 1.4000(a)(3)(ii).
    pub restriction_unreasonably_increases_cost: bool,
    /// Whether the restriction PRECLUDES reception or transmission
    /// of an acceptable-quality signal — § 1.4000(a)(3)(iii).
    pub restriction_precludes_acceptable_signal: bool,
    /// 2021 fixed-wireless expansion threshold — antenna must
    /// serve a customer on whose premises it is located.
    pub fixed_wireless_serves_on_premises_customer: bool,
    /// 2021 fixed-wireless expansion threshold — service must be
    /// broadband-only.
    pub fixed_wireless_broadband_only: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// True if the antenna installation falls within OTARD
    /// protection (covered type + tenant-exclusive-use location +
    /// any 2021-expansion requirements satisfied).
    pub otard_protected: bool,
    /// True if the landlord/HOA restriction is permissible under
    /// OTARD (i.e., satisfies safety / historic / no-impairment
    /// standards). When `otard_protected` is false, restrictions
    /// are not OTARD-evaluated.
    pub restriction_permissible: bool,
    /// True where OTARD assigns the burden of proof on the
    /// restricting party (§ 1.4000(c)). False where OTARD does
    /// not engage.
    pub burden_of_proof_on_restricting_party: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Input) -> CheckResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    // Antenna type must be protected.
    let covered_antenna_type = match input.antenna_type {
        AntennaType::DBS | AntennaType::MMDS | AntennaType::BroadcastTV => true,
        AntennaType::FixedWirelessHubRelay2021 => {
            // 2021 expansion has two extra requirements: serves
            // on-premises customer + broadband-only.
            if !input.fixed_wireless_serves_on_premises_customer {
                notes.push(
                    "2021 OTARD expansion — fixed-wireless hub/relay antenna must serve a \
                     customer on whose premises the antenna is located; the on-premises-\
                     customer requirement is not satisfied. Antenna falls outside the 2021 \
                     expansion."
                        .to_string(),
                );
                false
            } else if !input.fixed_wireless_broadband_only {
                notes.push(
                    "2021 OTARD expansion — fixed-wireless hub/relay antenna service must be \
                     broadband-only; the broadband-only requirement is not satisfied. Antenna \
                     falls outside the 2021 expansion."
                        .to_string(),
                );
                false
            } else {
                true
            }
        }
        AntennaType::OutsideOTARDScope => {
            notes.push(
                "Antenna type is outside the OTARD rule scope (oversized dish > 1m, amateur \
                 radio, AM/FM broadcast receiver, etc.). 47 CFR § 1.4000 does not preempt \
                 landlord / HOA restrictions on this antenna type."
                    .to_string(),
            );
            false
        }
    };

    // Location must be tenant-exclusive use.
    let covered_location = matches!(input.installation_location, InstallationLocation::TenantExclusiveUse);
    if !covered_location && covered_antenna_type {
        notes.push(
            "Installation location is in a common area (shared rooftop, exterior wall, \
             hallway, shared yard) — OUTSIDE § 1.4000 tenant-exclusive-use scope. OTARD \
             does not preempt landlord restrictions on common areas."
                .to_string(),
        );
    }

    let otard_protected = covered_antenna_type && covered_location;

    // Restriction permissibility under § 1.4000(b).
    let (restriction_permissible, citation) = if !otard_protected {
        // OTARD does not engage. Restriction is governed by other
        // law (lease, state, or local). We report restriction_
        // permissible as true (i.e., not invalid under OTARD).
        let cite = match input.antenna_type {
            AntennaType::OutsideOTARDScope => {
                "47 CFR § 1.4000(a)(1) (antenna type outside protected categories — DBS ≤ 1m, \
                 MMDS ≤ 1m, broadcast TV any size, fixed-wireless hub/relay 2021 expansion)"
            }
            AntennaType::FixedWirelessHubRelay2021 => {
                "47 CFR § 1.4000(a)(1) + FCC 21-10 (2021 Report and Order — fixed-wireless \
                 hub/relay must serve on-premises customer AND be broadband-only); preemption \
                 not engaged because expansion requirements not satisfied"
            }
            _ => {
                "47 CFR § 1.4000(a)(2)(i) (tenant-exclusive-use-or-control requirement) — \
                 installation in common area outside § 1.4000 scope"
            }
        };
        (true, cite)
    } else {
        // OTARD applies. Evaluate the restriction.
        let permissible = match input.restriction_type {
            RestrictionType::None => true,
            RestrictionType::Safety | RestrictionType::HistoricPreservation => {
                // Even safety / historic restrictions must satisfy
                // the no-impairment standard.
                if input.restriction_unreasonably_delays
                    || input.restriction_unreasonably_increases_cost
                    || input.restriction_precludes_acceptable_signal
                {
                    violations.push(
                        "§ 1.4000(a)(3) no-impairment standard violated — safety or historic \
                         preservation restriction must NOT (i) unreasonably delay or prevent \
                         installation, (ii) unreasonably increase the cost, or (iii) preclude \
                         reception or transmission of an acceptable-quality signal."
                            .to_string(),
                    );
                    false
                } else {
                    notes.push(
                        match input.restriction_type {
                            RestrictionType::Safety => {
                                "§ 1.4000(b)(1) safety restriction — restriction must be \
                                 necessary to accomplish a clearly defined legitimate safety \
                                 objective and applied non-discriminatorily to comparable \
                                 devices. Burden of proof on landlord per § 1.4000(c)."
                            }
                            RestrictionType::HistoricPreservation => {
                                "§ 1.4000(b)(2) historic-preservation restriction — restriction \
                                 must be necessary to preserve a property on the National \
                                 Register and cannot be more restrictive than restrictions on \
                                 modern appurtenances. Burden of proof on landlord per \
                                 § 1.4000(c)."
                            }
                            _ => unreachable!(),
                        }
                        .to_string(),
                    );
                    true
                }
            }
            RestrictionType::Aesthetic => {
                violations.push(
                    "§ 1.4000 — aesthetic restrictions are NOT permissible categories under \
                     OTARD. Landlord/HOA may not prohibit installation on aesthetic grounds \
                     alone."
                        .to_string(),
                );
                false
            }
            RestrictionType::BlanketProhibition => {
                violations.push(
                    "§ 1.4000 — blanket prohibitions (\"no antennas of any kind\" lease \
                     clauses) are NOT permissible under OTARD."
                        .to_string(),
                );
                false
            }
            RestrictionType::PreApprovalDelay => {
                violations.push(
                    "§ 1.4000(a)(3)(i) — pre-installation approval requirements that \
                     unreasonably delay are NOT permissible under OTARD."
                        .to_string(),
                );
                false
            }
        };
        let cite = match input.antenna_type {
            AntennaType::FixedWirelessHubRelay2021 => {
                "47 CFR § 1.4000(a)(1) (protected antenna type — fixed-wireless hub/relay per \
                 2021 expansion); § 1.4000(a)(2)(i) (tenant-exclusive-use scope); \
                 § 1.4000(b)(1) (safety); § 1.4000(b)(2) (historic preservation); \
                 § 1.4000(a)(3) (no-impairment standard); § 1.4000(c) (burden of proof on \
                 restricting party); FCC 21-10 (2021 Report and Order)"
            }
            _ => {
                "47 CFR § 1.4000(a)(1) (protected antenna types: DBS ≤ 1m, MMDS ≤ 1m, \
                 broadcast TV any size); § 1.4000(a)(2)(i) (tenant-exclusive-use scope); \
                 § 1.4000(b)(1) (safety); § 1.4000(b)(2) (historic preservation); \
                 § 1.4000(a)(3) (no-impairment standard); § 1.4000(c) (burden of proof on \
                 restricting party)"
            }
        };
        (permissible, cite)
    };

    notes.push(
        "47 CFR § 1.4000 PREEMPTS state, local, HOA, and lease restrictions that conflict \
         with the OTARD rule. Enforcement of conflicting restrictions is stayed pending FCC \
         or court review."
            .to_string(),
    );

    CheckResult {
        otard_protected,
        restriction_permissible,
        burden_of_proof_on_restricting_party: otard_protected,
        violations,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(antenna: AntennaType, location: InstallationLocation) -> Input {
        Input {
            antenna_type: antenna,
            installation_location: location,
            restriction_type: RestrictionType::None,
            restriction_unreasonably_delays: false,
            restriction_unreasonably_increases_cost: false,
            restriction_precludes_acceptable_signal: false,
            fixed_wireless_serves_on_premises_customer: true,
            fixed_wireless_broadband_only: true,
        }
    }

    // ── § 1.4000(a)(1) protected antenna types ──────────────────

    #[test]
    fn dbs_satellite_dish_on_patio_protected() {
        let r = check(&base(AntennaType::DBS, InstallationLocation::TenantExclusiveUse));
        assert!(r.otard_protected);
        assert!(r.restriction_permissible);
        assert!(r.burden_of_proof_on_restricting_party);
        assert!(r.citation.contains("§ 1.4000(a)(1)"));
    }

    #[test]
    fn mmds_antenna_on_balcony_protected() {
        let r = check(&base(AntennaType::MMDS, InstallationLocation::TenantExclusiveUse));
        assert!(r.otard_protected);
    }

    #[test]
    fn broadcast_tv_antenna_any_size_protected() {
        let r = check(&base(
            AntennaType::BroadcastTV,
            InstallationLocation::TenantExclusiveUse,
        ));
        assert!(r.otard_protected);
    }

    #[test]
    fn antenna_outside_otard_scope_not_protected() {
        let r = check(&base(
            AntennaType::OutsideOTARDScope,
            InstallationLocation::TenantExclusiveUse,
        ));
        assert!(!r.otard_protected);
        assert!(!r.burden_of_proof_on_restricting_party);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("outside the OTARD rule scope"))
        );
    }

    // ── 2021 fixed-wireless expansion ───────────────────────────

    #[test]
    fn fixed_wireless_2021_with_both_requirements_protected() {
        let r = check(&base(
            AntennaType::FixedWirelessHubRelay2021,
            InstallationLocation::TenantExclusiveUse,
        ));
        assert!(r.otard_protected);
        assert!(r.citation.contains("FCC 21-10"));
    }

    #[test]
    fn fixed_wireless_2021_without_on_premises_customer_not_protected() {
        let mut i = base(
            AntennaType::FixedWirelessHubRelay2021,
            InstallationLocation::TenantExclusiveUse,
        );
        i.fixed_wireless_serves_on_premises_customer = false;
        let r = check(&i);
        assert!(!r.otard_protected);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("on-premises-customer requirement is not satisfied"))
        );
    }

    #[test]
    fn fixed_wireless_2021_without_broadband_only_not_protected() {
        let mut i = base(
            AntennaType::FixedWirelessHubRelay2021,
            InstallationLocation::TenantExclusiveUse,
        );
        i.fixed_wireless_broadband_only = false;
        let r = check(&i);
        assert!(!r.otard_protected);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("broadband-only requirement is not satisfied"))
        );
    }

    // ── § 1.4000(a)(2) tenant-exclusive-use scope ───────────────

    #[test]
    fn common_area_installation_not_protected() {
        let r = check(&base(AntennaType::DBS, InstallationLocation::CommonArea));
        assert!(!r.otard_protected);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("common area") && n.contains("OUTSIDE"))
        );
    }

    #[test]
    fn common_area_with_protected_antenna_type_still_unprotected() {
        for at in [
            AntennaType::DBS,
            AntennaType::MMDS,
            AntennaType::BroadcastTV,
            AntennaType::FixedWirelessHubRelay2021,
        ] {
            let r = check(&base(at, InstallationLocation::CommonArea));
            assert!(
                !r.otard_protected,
                "{:?}: common-area location must override antenna-type protection",
                at,
            );
        }
    }

    // ── § 1.4000(b) permissible restrictions ────────────────────

    #[test]
    fn safety_restriction_without_impairment_permissible() {
        let mut i = base(AntennaType::DBS, InstallationLocation::TenantExclusiveUse);
        i.restriction_type = RestrictionType::Safety;
        let r = check(&i);
        assert!(r.otard_protected);
        assert!(r.restriction_permissible);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1.4000(b)(1) safety"))
        );
    }

    #[test]
    fn safety_restriction_with_unreasonable_delay_not_permissible() {
        let mut i = base(AntennaType::DBS, InstallationLocation::TenantExclusiveUse);
        i.restriction_type = RestrictionType::Safety;
        i.restriction_unreasonably_delays = true;
        let r = check(&i);
        assert!(!r.restriction_permissible);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("no-impairment standard") && v.contains("delay"))
        );
    }

    #[test]
    fn safety_restriction_with_cost_increase_not_permissible() {
        let mut i = base(AntennaType::DBS, InstallationLocation::TenantExclusiveUse);
        i.restriction_type = RestrictionType::Safety;
        i.restriction_unreasonably_increases_cost = true;
        let r = check(&i);
        assert!(!r.restriction_permissible);
    }

    #[test]
    fn safety_restriction_with_signal_preclusion_not_permissible() {
        let mut i = base(AntennaType::DBS, InstallationLocation::TenantExclusiveUse);
        i.restriction_type = RestrictionType::Safety;
        i.restriction_precludes_acceptable_signal = true;
        let r = check(&i);
        assert!(!r.restriction_permissible);
    }

    #[test]
    fn historic_preservation_without_impairment_permissible() {
        let mut i = base(AntennaType::DBS, InstallationLocation::TenantExclusiveUse);
        i.restriction_type = RestrictionType::HistoricPreservation;
        let r = check(&i);
        assert!(r.restriction_permissible);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1.4000(b)(2) historic-preservation"))
        );
    }

    #[test]
    fn historic_preservation_with_impairment_not_permissible() {
        let mut i = base(AntennaType::DBS, InstallationLocation::TenantExclusiveUse);
        i.restriction_type = RestrictionType::HistoricPreservation;
        i.restriction_precludes_acceptable_signal = true;
        let r = check(&i);
        assert!(!r.restriction_permissible);
    }

    #[test]
    fn aesthetic_restriction_never_permissible() {
        let mut i = base(AntennaType::DBS, InstallationLocation::TenantExclusiveUse);
        i.restriction_type = RestrictionType::Aesthetic;
        let r = check(&i);
        assert!(!r.restriction_permissible);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("aesthetic") && v.contains("NOT permissible"))
        );
    }

    #[test]
    fn blanket_prohibition_never_permissible() {
        let mut i = base(AntennaType::DBS, InstallationLocation::TenantExclusiveUse);
        i.restriction_type = RestrictionType::BlanketProhibition;
        let r = check(&i);
        assert!(!r.restriction_permissible);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("blanket prohibitions") && v.contains("NOT permissible"))
        );
    }

    #[test]
    fn pre_approval_delay_not_permissible() {
        let mut i = base(AntennaType::DBS, InstallationLocation::TenantExclusiveUse);
        i.restriction_type = RestrictionType::PreApprovalDelay;
        let r = check(&i);
        assert!(!r.restriction_permissible);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("pre-installation approval") && v.contains("delay"))
        );
    }

    // ── § 1.4000(c) burden of proof ─────────────────────────────

    #[test]
    fn burden_on_restricting_party_when_otard_protected() {
        let r = check(&base(AntennaType::DBS, InstallationLocation::TenantExclusiveUse));
        assert!(r.burden_of_proof_on_restricting_party);
    }

    #[test]
    fn no_burden_when_otard_not_protected() {
        let r = check(&base(
            AntennaType::OutsideOTARDScope,
            InstallationLocation::TenantExclusiveUse,
        ));
        assert!(!r.burden_of_proof_on_restricting_party);
    }

    // ── Regression invariants ───────────────────────────────────

    #[test]
    fn aesthetic_blanket_pre_approval_invalidate_regardless_of_antenna_type() {
        for at in [
            AntennaType::DBS,
            AntennaType::MMDS,
            AntennaType::BroadcastTV,
            AntennaType::FixedWirelessHubRelay2021,
        ] {
            for rt in [
                RestrictionType::Aesthetic,
                RestrictionType::BlanketProhibition,
                RestrictionType::PreApprovalDelay,
            ] {
                let mut i = base(at, InstallationLocation::TenantExclusiveUse);
                i.restriction_type = rt;
                let r = check(&i);
                assert!(
                    !r.restriction_permissible,
                    "{:?} + {:?}: must not be permissible",
                    at,
                    rt,
                );
            }
        }
    }

    #[test]
    fn safety_and_historic_subject_to_no_impairment_standard_invariant() {
        for rt in [
            RestrictionType::Safety,
            RestrictionType::HistoricPreservation,
        ] {
            // No impairment → permissible.
            let mut clean = base(AntennaType::DBS, InstallationLocation::TenantExclusiveUse);
            clean.restriction_type = rt;
            assert!(check(&clean).restriction_permissible);
            // Any of the three impairment flags → not permissible.
            for (delay, cost, signal) in [
                (true, false, false),
                (false, true, false),
                (false, false, true),
            ] {
                let mut i = base(AntennaType::DBS, InstallationLocation::TenantExclusiveUse);
                i.restriction_type = rt;
                i.restriction_unreasonably_delays = delay;
                i.restriction_unreasonably_increases_cost = cost;
                i.restriction_precludes_acceptable_signal = signal;
                assert!(
                    !check(&i).restriction_permissible,
                    "{:?} with impairment (delay={delay} cost={cost} signal={signal}) must \
                     not be permissible",
                    rt,
                );
            }
        }
    }

    #[test]
    fn citation_pins_2021_expansion_for_fixed_wireless() {
        let r = check(&base(
            AntennaType::FixedWirelessHubRelay2021,
            InstallationLocation::TenantExclusiveUse,
        ));
        assert!(r.citation.contains("FCC 21-10"));
    }

    #[test]
    fn preemption_note_present_across_all_paths() {
        for at in [
            AntennaType::DBS,
            AntennaType::MMDS,
            AntennaType::BroadcastTV,
            AntennaType::FixedWirelessHubRelay2021,
            AntennaType::OutsideOTARDScope,
        ] {
            for loc in [
                InstallationLocation::TenantExclusiveUse,
                InstallationLocation::CommonArea,
            ] {
                let r = check(&base(at, loc));
                assert!(
                    r.notes
                        .iter()
                        .any(|n| n.contains("PREEMPTS") && n.contains("OTARD")),
                    "{:?} + {:?}: preemption note must be present",
                    at,
                    loc,
                );
            }
        }
    }

    #[test]
    fn fixed_wireless_2021_expansion_thresholds_required_invariant() {
        // Both on-premises-customer AND broadband-only must be
        // satisfied to invoke 2021 expansion.
        let mut both = base(
            AntennaType::FixedWirelessHubRelay2021,
            InstallationLocation::TenantExclusiveUse,
        );
        both.fixed_wireless_serves_on_premises_customer = true;
        both.fixed_wireless_broadband_only = true;
        assert!(check(&both).otard_protected);

        for (on_prem, broadband) in [(false, true), (true, false), (false, false)] {
            let mut i = base(
                AntennaType::FixedWirelessHubRelay2021,
                InstallationLocation::TenantExclusiveUse,
            );
            i.fixed_wireless_serves_on_premises_customer = on_prem;
            i.fixed_wireless_broadband_only = broadband;
            assert!(
                !check(&i).otard_protected,
                "fixed-wireless 2021: must require both on_premises ({on_prem}) and \
                 broadband_only ({broadband})",
            );
        }
    }

    #[test]
    fn no_restriction_path_compliant_when_protected() {
        let r = check(&base(AntennaType::DBS, InstallationLocation::TenantExclusiveUse));
        assert!(r.otard_protected);
        assert!(r.restriction_permissible);
        assert!(r.violations.is_empty());
    }
}
