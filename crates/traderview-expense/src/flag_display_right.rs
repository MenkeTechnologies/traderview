//! State + federal tenant/owner right-to-display-flag compliance
//! check.
//!
//! Federal floor: Freedom to Display the American Flag Act of 2005
//! (4 U.S.C. § 5, Pub. L. 109-243). State extensions provide
//! additional protections — Florida § 720.304 is the strongest,
//! explicitly extending to renters.
//!
//! Sibling to `religious_display_doorpost` (mezuzah / religious-
//! item display), `firearms_in_rental_unit`, and
//! `otard_antenna_installation` — collectively the
//! "tenant-rights-in-rental-unit display + installation" cluster.
//!
//! Four regimes:
//!
//!   - **Federal** — 4 U.S.C. § 5 + Pub. L. 109-243 (Freedom to
//!     Display the American Flag Act of 2005). Applies to
//!     condominium associations, cooperative associations, and
//!     residential real estate management associations. Such
//!     associations may not adopt or enforce policy preventing a
//!     member from displaying the U.S. flag on residential
//!     property where member has separate ownership interest or
//!     right to exclusive possession or use. Reasonable time /
//!     place / manner restrictions permitted. CRITICAL LIMITATION:
//!     NO PRIVATE RIGHT OF ACTION — federal Act lacks an
//!     enforcement mechanism. Does NOT directly cover landlord-
//!     tenant rentals.
//!
//!   - **Florida** — Fla. Stat. § 720.304 (HOA) + § 718.113
//!     (condo) + § 723.054 (mobile home parks). HB 437 (2023)
//!     expanded protections to TWO flags. Renter coverage: any
//!     tenant may display one portable, removable U.S. flag
//!     (and other listed flags) regardless of any covenants,
//!     restrictions, bylaws, rules, or lease provisions to the
//!     contrary. STATE LAW EXPLICITLY TRUMPS LEASE. Military
//!     branch + POW-MIA flags limited to 4.5 × 6 feet.
//!
//!   - **Virginia** — Va. Code § 55.1-1820 (display of the flag
//!     of the United States — supporting structures + affirmative
//!     defense in HOA context).
//!
//!   - **Default** — federal Act applies to associations only
//!     (and lacks private right of action); no general landlord-
//!     tenant private statutory protection.
//!
//! Citations: 4 U.S.C. § 5 (codification of rules and customs);
//! Freedom to Display the American Flag Act of 2005, Pub. L.
//! 109-243; H.R. 42 (109th Congress); Fla. Stat. § 720.304(2)
//! (HOA flag display + renter extension); Fla. Stat. § 718.113
//! (condo flag display); Fla. HB 437 (2023 — two-flag expansion);
//! Va. Code § 55.1-1820 (flag display in common interest
//! communities).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Federal,
    Florida,
    Virginia,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PropertyOwnershipStatus {
    /// Owner with separate ownership interest OR right to
    /// exclusive possession / use — federal Act applies.
    OwnerWithExclusiveUse,
    /// Renter / tenant — federal Act does NOT directly apply;
    /// only state statutes that explicitly extend to renters.
    Renter,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FlagType {
    /// U.S. flag — federal Act + all state extensions cover.
    UnitedStatesFlag,
    /// State flag (e.g., Florida state flag) — Florida § 720.304
    /// extension; not covered by federal Act.
    StateFlag,
    /// Military branch flag (Army, Navy, Air Force, Marine Corps,
    /// Coast Guard, Space Force) — Florida § 720.304 covers
    /// (max 4.5 × 6 ft).
    MilitaryBranchFlag,
    /// POW-MIA flag — Florida § 720.304 covers (max 4.5 × 6 ft).
    PowMiaFlag,
    /// Other flag (political, decorative, etc.) — not covered by
    /// any statutory regime in this matrix.
    Other,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    pub ownership_status: PropertyOwnershipStatus,
    pub flag_type: FlagType,
    /// Whether the flag is portable and removable (Florida
    /// requires for renter protection).
    pub is_portable_removable: bool,
    /// Whether flag dimensions are within the 4.5 × 6 feet limit
    /// for military / POW-MIA flags (Florida).
    pub dimensions_within_florida_limit: bool,
    /// Whether the display is in a "respectful manner" as
    /// required by Florida § 720.304.
    pub respectful_manner_of_display: bool,
    /// Whether the display infringes on other residents' space.
    pub infringes_on_other_residents: bool,
    /// Whether a restriction has been imposed by the landlord,
    /// HOA, or association.
    pub restriction_imposed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// True if the regime provides any statutory protection for
    /// the display under the present facts.
    pub statutory_protection: bool,
    /// True if the federal Freedom to Display the American Flag
    /// Act of 2005 applies (owner with exclusive use + U.S. flag).
    pub federal_act_applies: bool,
    /// True if a state statute explicitly protects renter display.
    pub state_protection_for_renter: bool,
    /// True if the imposed restriction is permissible (i.e., not
    /// preempted by statutory protection).
    pub restriction_permissible: bool,
    /// True if the regime provides a private right of action for
    /// enforcement (federal Act does NOT).
    pub private_right_of_action: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Input) -> CheckResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    // Federal Act application: owner with exclusive use + U.S. flag.
    let federal_act_applies = matches!(
        input.ownership_status,
        PropertyOwnershipStatus::OwnerWithExclusiveUse
    ) && matches!(input.flag_type, FlagType::UnitedStatesFlag);

    // Florida § 720.304 renter coverage requirements:
    //   - U.S. flag (any size), state flag (any size), military/
    //     POW-MIA flag (≤ 4.5 × 6 ft)
    //   - Portable and removable
    //   - Respectful manner
    //   - Does not infringe on other residents
    let florida_renter_eligible_flag = matches!(
        input.flag_type,
        FlagType::UnitedStatesFlag
            | FlagType::StateFlag
            | FlagType::MilitaryBranchFlag
            | FlagType::PowMiaFlag
    );
    let florida_military_size_ok = match input.flag_type {
        FlagType::MilitaryBranchFlag | FlagType::PowMiaFlag => {
            input.dimensions_within_florida_limit
        }
        _ => true, // size limit applies only to military/POW-MIA
    };
    let florida_renter_protected = matches!(input.regime, Regime::Florida)
        && matches!(input.ownership_status, PropertyOwnershipStatus::Renter)
        && florida_renter_eligible_flag
        && input.is_portable_removable
        && input.respectful_manner_of_display
        && !input.infringes_on_other_residents
        && florida_military_size_ok;

    let florida_owner_protected = matches!(input.regime, Regime::Florida)
        && matches!(input.ownership_status, PropertyOwnershipStatus::OwnerWithExclusiveUse)
        && florida_renter_eligible_flag
        && input.respectful_manner_of_display
        && florida_military_size_ok;

    let virginia_protection = matches!(input.regime, Regime::Virginia)
        && matches!(input.ownership_status, PropertyOwnershipStatus::OwnerWithExclusiveUse)
        && matches!(input.flag_type, FlagType::UnitedStatesFlag);

    let federal_protection = matches!(input.regime, Regime::Federal) && federal_act_applies;

    let statutory_protection = federal_protection
        || florida_renter_protected
        || florida_owner_protected
        || virginia_protection;

    let state_protection_for_renter = florida_renter_protected;

    // Federal Act lacks private right of action.
    let private_right_of_action = !matches!(input.regime, Regime::Federal);

    let restriction_permissible = !(statutory_protection && input.restriction_imposed);

    if statutory_protection && input.restriction_imposed {
        violations.push(format!(
            "{:?} — restriction on flag display violates statutory protection.",
            input.regime,
        ));
    }

    // Federal Act private-right-of-action limitation note.
    if matches!(input.regime, Regime::Federal) {
        notes.push(
            "Freedom to Display the American Flag Act of 2005 (4 U.S.C. § 5, Pub. L. 109-243) \
             lacks a PRIVATE RIGHT OF ACTION — the statute prohibits association restrictions \
             but does not authorize private suit by member or tenant. Enforcement must go \
             through other channels (e.g., state attorney general, public-interest litigation)."
                .to_string(),
        );
    }

    // Florida renter-protection state-law-trumps-lease note.
    if matches!(input.regime, Regime::Florida) && state_protection_for_renter {
        notes.push(
            "Fla. Stat. § 720.304(2) — STATE LAW EXPLICITLY TRUMPS LEASE. Any tenant may \
             display one portable, removable U.S. flag (and other listed flags) regardless of \
             any covenants, restrictions, bylaws, rules, or lease provisions to the contrary."
                .to_string(),
        );
    }

    // Federal Act ownership-status note.
    if matches!(input.regime, Regime::Federal)
        && matches!(input.ownership_status, PropertyOwnershipStatus::Renter)
    {
        notes.push(
            "Federal Act applies only to OWNERS with separate ownership interest or right to \
             exclusive possession / use. Renters in associations without state-statute \
             extension must rely on lease enforcement or other channels."
                .to_string(),
        );
    }

    // Reasonable time / place / manner restriction note.
    if statutory_protection && !input.respectful_manner_of_display {
        notes.push(
            "Reasonable time / place / manner restrictions permitted under both federal Act \
             and state statutes. Display must be in respectful manner."
                .to_string(),
        );
    }

    notes.push(
        "Sibling to religious_display_doorpost (mezuzah/religious-item display + 8 regimes); \
         firearms_in_rental_unit (2A-protected possession + 6 regimes); \
         otard_antenna_installation (federal OTARD rule for satellite dish/antenna). \
         Collectively the 'tenant-rights-in-rental-unit display + installation' cluster."
            .to_string(),
    );

    let citation = match input.regime {
        Regime::Federal => {
            "4 U.S.C. § 5 (codification of rules and customs); Freedom to Display the American \
             Flag Act of 2005, Pub. L. 109-243; H.R. 42 (109th Congress); applies to \
             condominium + cooperative + residential real estate management associations; \
             NO private right of action"
        }
        Regime::Florida => {
            "Fla. Stat. § 720.304(2) (HOA flag display + renter extension); § 718.113 (condo \
             flag display); § 723.054 (mobile home parks); HB 437 (2023 — two-flag expansion); \
             STATE LAW EXPLICITLY TRUMPS LEASE for renter U.S./state/military/POW-MIA flag \
             display"
        }
        Regime::Virginia => {
            "Va. Code § 55.1-1820 (display of U.S. flag in common interest communities + \
             supporting structures + affirmative defense)"
        }
        Regime::Default => {
            "Federal Act applies to associations only (4 U.S.C. § 5 + Pub. L. 109-243); no \
             general landlord-tenant private statutory protection"
        }
    };

    CheckResult {
        statutory_protection,
        federal_act_applies,
        state_protection_for_renter,
        restriction_permissible,
        private_right_of_action,
        violations,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(
        regime: Regime,
        ownership: PropertyOwnershipStatus,
        flag: FlagType,
    ) -> Input {
        Input {
            regime,
            ownership_status: ownership,
            flag_type: flag,
            is_portable_removable: true,
            dimensions_within_florida_limit: true,
            respectful_manner_of_display: true,
            infringes_on_other_residents: false,
            restriction_imposed: false,
        }
    }

    // ── Federal Act ─────────────────────────────────────────────

    #[test]
    fn federal_act_owner_us_flag_protected() {
        let r = check(&base(
            Regime::Federal,
            PropertyOwnershipStatus::OwnerWithExclusiveUse,
            FlagType::UnitedStatesFlag,
        ));
        assert!(r.federal_act_applies);
        assert!(r.statutory_protection);
        assert!(!r.private_right_of_action);
        assert!(r.citation.contains("4 U.S.C. § 5"));
        assert!(r.citation.contains("NO private right of action"));
    }

    #[test]
    fn federal_act_renter_not_directly_covered() {
        let r = check(&base(
            Regime::Federal,
            PropertyOwnershipStatus::Renter,
            FlagType::UnitedStatesFlag,
        ));
        assert!(!r.federal_act_applies);
        assert!(!r.statutory_protection);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("applies only to OWNERS"))
        );
    }

    #[test]
    fn federal_act_state_flag_not_covered() {
        // Federal Act only covers U.S. flag.
        let r = check(&base(
            Regime::Federal,
            PropertyOwnershipStatus::OwnerWithExclusiveUse,
            FlagType::StateFlag,
        ));
        assert!(!r.federal_act_applies);
    }

    #[test]
    fn federal_act_no_private_right_of_action_note() {
        let r = check(&base(
            Regime::Federal,
            PropertyOwnershipStatus::OwnerWithExclusiveUse,
            FlagType::UnitedStatesFlag,
        ));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("PRIVATE RIGHT OF ACTION") && n.contains("Pub. L. 109-243"))
        );
    }

    // ── Florida § 720.304 + renter extension ────────────────────

    #[test]
    fn florida_renter_us_flag_protected_with_state_law_trumping_lease() {
        let r = check(&base(
            Regime::Florida,
            PropertyOwnershipStatus::Renter,
            FlagType::UnitedStatesFlag,
        ));
        assert!(r.statutory_protection);
        assert!(r.state_protection_for_renter);
        assert!(r.citation.contains("§ 720.304(2)"));
        assert!(r.citation.contains("TRUMPS LEASE"));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("STATE LAW EXPLICITLY TRUMPS LEASE"))
        );
    }

    #[test]
    fn florida_renter_state_flag_protected() {
        let r = check(&base(
            Regime::Florida,
            PropertyOwnershipStatus::Renter,
            FlagType::StateFlag,
        ));
        assert!(r.state_protection_for_renter);
    }

    #[test]
    fn florida_renter_military_flag_within_size_limit_protected() {
        let mut i = base(
            Regime::Florida,
            PropertyOwnershipStatus::Renter,
            FlagType::MilitaryBranchFlag,
        );
        i.dimensions_within_florida_limit = true;
        let r = check(&i);
        assert!(r.state_protection_for_renter);
    }

    #[test]
    fn florida_renter_military_flag_oversized_not_protected() {
        let mut i = base(
            Regime::Florida,
            PropertyOwnershipStatus::Renter,
            FlagType::MilitaryBranchFlag,
        );
        i.dimensions_within_florida_limit = false;
        let r = check(&i);
        assert!(!r.state_protection_for_renter);
    }

    #[test]
    fn florida_renter_pow_mia_within_size_limit_protected() {
        let r = check(&base(
            Regime::Florida,
            PropertyOwnershipStatus::Renter,
            FlagType::PowMiaFlag,
        ));
        assert!(r.state_protection_for_renter);
    }

    #[test]
    fn florida_renter_other_flag_not_protected() {
        let r = check(&base(
            Regime::Florida,
            PropertyOwnershipStatus::Renter,
            FlagType::Other,
        ));
        assert!(!r.state_protection_for_renter);
        assert!(!r.statutory_protection);
    }

    #[test]
    fn florida_renter_non_portable_not_protected() {
        let mut i = base(
            Regime::Florida,
            PropertyOwnershipStatus::Renter,
            FlagType::UnitedStatesFlag,
        );
        i.is_portable_removable = false;
        let r = check(&i);
        assert!(!r.state_protection_for_renter);
    }

    #[test]
    fn florida_renter_infringes_on_others_not_protected() {
        let mut i = base(
            Regime::Florida,
            PropertyOwnershipStatus::Renter,
            FlagType::UnitedStatesFlag,
        );
        i.infringes_on_other_residents = true;
        let r = check(&i);
        assert!(!r.state_protection_for_renter);
    }

    #[test]
    fn florida_owner_us_flag_protected() {
        let r = check(&base(
            Regime::Florida,
            PropertyOwnershipStatus::OwnerWithExclusiveUse,
            FlagType::UnitedStatesFlag,
        ));
        assert!(r.statutory_protection);
    }

    #[test]
    fn florida_restriction_imposed_violation() {
        let mut i = base(
            Regime::Florida,
            PropertyOwnershipStatus::Renter,
            FlagType::UnitedStatesFlag,
        );
        i.restriction_imposed = true;
        let r = check(&i);
        assert!(!r.restriction_permissible);
        assert!(r.violations.iter().any(|v| v.contains("Florida")));
    }

    // ── Virginia § 55.1-1820 ────────────────────────────────────

    #[test]
    fn virginia_owner_us_flag_protected() {
        let r = check(&base(
            Regime::Virginia,
            PropertyOwnershipStatus::OwnerWithExclusiveUse,
            FlagType::UnitedStatesFlag,
        ));
        assert!(r.statutory_protection);
        assert!(r.citation.contains("§ 55.1-1820"));
    }

    #[test]
    fn virginia_renter_not_covered() {
        let r = check(&base(
            Regime::Virginia,
            PropertyOwnershipStatus::Renter,
            FlagType::UnitedStatesFlag,
        ));
        assert!(!r.statutory_protection);
    }

    // ── Default ─────────────────────────────────────────────────

    #[test]
    fn default_no_statutory_protection() {
        let r = check(&base(
            Regime::Default,
            PropertyOwnershipStatus::OwnerWithExclusiveUse,
            FlagType::UnitedStatesFlag,
        ));
        assert!(!r.statutory_protection);
    }

    // ── Regression-critical invariants ──────────────────────────

    #[test]
    fn only_florida_protects_renters_invariant() {
        let mut fl_renter = base(
            Regime::Florida,
            PropertyOwnershipStatus::Renter,
            FlagType::UnitedStatesFlag,
        );
        fl_renter.is_portable_removable = true;
        fl_renter.respectful_manner_of_display = true;
        assert!(check(&fl_renter).state_protection_for_renter);

        for &regime in &[Regime::Federal, Regime::Virginia, Regime::Default] {
            let r = check(&base(
                regime,
                PropertyOwnershipStatus::Renter,
                FlagType::UnitedStatesFlag,
            ));
            assert!(
                !r.state_protection_for_renter,
                "{:?}: must NOT protect renters",
                regime,
            );
        }
    }

    #[test]
    fn only_federal_lacks_private_right_of_action_invariant() {
        assert!(!check(&base(
            Regime::Federal,
            PropertyOwnershipStatus::OwnerWithExclusiveUse,
            FlagType::UnitedStatesFlag,
        ))
        .private_right_of_action);

        for &regime in &[Regime::Florida, Regime::Virginia, Regime::Default] {
            let r = check(&base(
                regime,
                PropertyOwnershipStatus::OwnerWithExclusiveUse,
                FlagType::UnitedStatesFlag,
            ));
            assert!(
                r.private_right_of_action,
                "{:?}: must have private right of action",
                regime,
            );
        }
    }

    #[test]
    fn military_size_limit_applies_only_to_military_and_pow_mia_invariant() {
        // For non-military flags, the size limit input is a no-op.
        for flag in [
            FlagType::UnitedStatesFlag,
            FlagType::StateFlag,
        ] {
            let mut within = base(Regime::Florida, PropertyOwnershipStatus::Renter, flag);
            within.dimensions_within_florida_limit = true;
            let mut over = base(Regime::Florida, PropertyOwnershipStatus::Renter, flag);
            over.dimensions_within_florida_limit = false;
            assert_eq!(
                check(&within).state_protection_for_renter,
                check(&over).state_protection_for_renter,
                "{:?}: size flag should be no-op",
                flag,
            );
        }

        // For military and POW-MIA, size limit IS load-bearing.
        for flag in [FlagType::MilitaryBranchFlag, FlagType::PowMiaFlag] {
            let mut within = base(Regime::Florida, PropertyOwnershipStatus::Renter, flag);
            within.dimensions_within_florida_limit = true;
            let mut over = base(Regime::Florida, PropertyOwnershipStatus::Renter, flag);
            over.dimensions_within_florida_limit = false;
            assert!(check(&within).state_protection_for_renter);
            assert!(!check(&over).state_protection_for_renter);
        }
    }

    #[test]
    fn florida_renter_protection_requires_all_4_conditions_invariant() {
        // (portable + respectful + not-infringing + size-ok) — all
        // must hold. Each flipped to false should defeat protection.
        for flip_portable in [false, true] {
            for flip_respectful in [false, true] {
                for flip_infringes in [false, true] {
                    let mut i = base(
                        Regime::Florida,
                        PropertyOwnershipStatus::Renter,
                        FlagType::UnitedStatesFlag,
                    );
                    i.is_portable_removable = flip_portable;
                    i.respectful_manner_of_display = flip_respectful;
                    i.infringes_on_other_residents = !flip_infringes;
                    let r = check(&i);
                    let expected_protected =
                        flip_portable && flip_respectful && flip_infringes;
                    assert_eq!(
                        r.state_protection_for_renter, expected_protected,
                        "portable={} respectful={} not_infringes={} expected={}",
                        flip_portable, flip_respectful, flip_infringes, expected_protected,
                    );
                }
            }
        }
    }

    #[test]
    fn citation_pins_authority_per_regime() {
        assert!(
            check(&base(
                Regime::Federal,
                PropertyOwnershipStatus::OwnerWithExclusiveUse,
                FlagType::UnitedStatesFlag,
            ))
            .citation
            .contains("4 U.S.C. § 5")
        );
        assert!(
            check(&base(
                Regime::Florida,
                PropertyOwnershipStatus::Renter,
                FlagType::UnitedStatesFlag,
            ))
            .citation
            .contains("§ 720.304(2)")
        );
        assert!(
            check(&base(
                Regime::Virginia,
                PropertyOwnershipStatus::OwnerWithExclusiveUse,
                FlagType::UnitedStatesFlag,
            ))
            .citation
            .contains("§ 55.1-1820")
        );
        assert!(
            check(&base(
                Regime::Default,
                PropertyOwnershipStatus::Renter,
                FlagType::UnitedStatesFlag,
            ))
            .citation
            .contains("no general landlord-tenant")
        );
    }

    #[test]
    fn sibling_module_note_present_across_all_regimes() {
        for &regime in &[Regime::Federal, Regime::Florida, Regime::Virginia, Regime::Default] {
            for &ownership in &[
                PropertyOwnershipStatus::OwnerWithExclusiveUse,
                PropertyOwnershipStatus::Renter,
            ] {
                let r = check(&base(regime, ownership, FlagType::UnitedStatesFlag));
                assert!(
                    r.notes.iter().any(|n| n.contains("religious_display_doorpost")
                        && n.contains("firearms_in_rental_unit")
                        && n.contains("otard_antenna_installation")),
                    "{:?} {:?}: sibling-module note must be present",
                    regime,
                    ownership,
                );
            }
        }
    }
}
