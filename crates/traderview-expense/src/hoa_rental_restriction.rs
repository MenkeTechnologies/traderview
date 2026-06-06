//! HOA rental restriction enforceability — when may a homeowners
//! association enforce restrictions on a unit owner's right to
//! rent their property?
//!
//! Trader-critical for landlord-owners holding property inside
//! common-interest communities — a single HOA amendment can
//! destroy a building's rental cash flow. State law varies on
//! whether such amendments bind existing owners or require
//! grandfather protection.
//!
//! Florida — Fla. Stat. § 720.306(1)(h) (eff. July 1, 2021):
//! GRANDFATHER RULE. Any HOA amendment adopted after July 1, 2021
//! prohibiting or regulating rentals applies ONLY to (a) owners
//! who acquired title AFTER the amendment took effect; OR (b)
//! owners who AFFIRMATIVELY consented to the amendment. Silence
//! does not count as consent. Two NARROW EXCEPTIONS bind all
//! owners regardless of grandfather: (1) amendments prohibiting
//! or regulating rentals for terms of 6 months or less
//! (short-term rental restrictions); (2) amendments limiting
//! rentals to 3 or fewer times per calendar year. Grandfather
//! protection survives transfer to (a) heir acquiring on prior
//! owner's death, or (b) entity affiliated with prior owner —
//! but is LOST upon transfer to unrelated third party.
//!
//! Arizona — A.R.S. § 33-1806.01 (planned communities): The
//! declaration controls — HOA may prohibit rentals or impose
//! time-period restrictions IF such restrictions appear in the
//! declaration. Statute does NOT set a percentage cap. Owner has
//! statutory right to designate a third-party agent to handle
//! HOA matters (except voting + board service) by signed written
//! designation; HOA must accept once written notice received.
//!
//! Default — declaration controls. No statutory grandfather
//! protection. New HOA amendments bind existing owners under
//! general covenants law subject to declaration's amendment
//! procedure.
//!
//! Citations: Fla. Stat. § 720.306(1)(h) (post-July-2021 HOA
//! rental restriction grandfather rule); Fla. Stat. § 720.306(1)(h)1
//! (short-term-rental ≤6-month exception); Fla. Stat. § 720.306(1)(h)2
//! (≤3 rentals/year exception); Fla. Stat. § 720.306(1)(h)3
//! (heir/affiliate grandfather preservation); A.R.S. § 33-1806.01
//! (Arizona planned-community rental restrictions);
//! A.R.S. § 33-1806.01(A) (declaration-based restriction
//! authority); A.R.S. § 33-1806.01(B) (third-party agent
//! designation right). Sibling modules: `tenant_topa` (tenant
//! opportunity to purchase — distinct from HOA restrictions);
//! `str_regulation` (short-term rental regulation by local
//! government — distinct from HOA private restrictions);
//! `condominium_conversion_protection` (tenant protection during
//! condo conversions).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    /// Fla. Stat. § 720.306(1)(h) — grandfather rule for HOA
    /// rental restrictions adopted after July 1, 2021.
    Florida,
    /// A.R.S. § 33-1806.01 — declaration controls; statutory
    /// agent designation right.
    Arizona,
    /// Default — declaration controls; no statutory grandfather.
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RestrictionType {
    /// General rental prohibition or regulation (≥6-month leases).
    GeneralLongTermRestriction,
    /// Short-term rental restriction — leases ≤ 6 months.
    /// Florida exception: binds all owners.
    ShortTermRentalRestriction,
    /// Frequency restriction — ≤ 3 rentals per calendar year.
    /// Florida exception: binds all owners.
    FrequencyRestrictionThreeOrFewer,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    pub restriction_type: RestrictionType,
    /// True if the restriction was adopted as an HOA amendment
    /// after July 1, 2021. Florida-specific trigger for
    /// § 720.306(1)(h) grandfather rule.
    pub amendment_adopted_after_july_2021: bool,
    /// True if the owner acquired title BEFORE the amendment.
    pub owner_acquired_before_amendment: bool,
    /// True if the owner AFFIRMATIVELY consented to the
    /// amendment (silence does NOT count as consent).
    pub owner_consented_to_amendment: bool,
    /// True if current owner acquired title via heirship or
    /// transfer from an affiliated entity (preserves Florida
    /// grandfather protection).
    pub owner_acquired_via_heir_or_affiliate: bool,
    /// True if title has been transferred to an unrelated party
    /// after grandfather protection attached. Erodes Florida
    /// grandfather under § 720.306(1)(h)3.
    pub transferred_to_unrelated_party: bool,
    /// True if the HOA declaration affirmatively permits rentals
    /// (or is silent on rentals — declaration default permits).
    pub declaration_permits_rental: bool,
    /// Arizona-specific — true if owner has signed and
    /// delivered written designation of a third-party agent
    /// for HOA matters under A.R.S. § 33-1806.01(B).
    pub agent_designation_signed_and_delivered: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// True if the HOA may enforce the restriction against this
    /// particular owner.
    pub restriction_enforceable_against_owner: bool,
    /// True if Florida § 720.306(1)(h) grandfather protection
    /// is engaged.
    pub grandfather_protection_engaged: bool,
    /// True if a Florida grandfather EXCEPTION applies (short-
    /// term or frequency restriction binds all owners).
    pub florida_exception_binds_all_owners: bool,
    /// Arizona-specific — true if statutory agent designation
    /// right has been validly invoked.
    pub agent_designation_engaged: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Input) -> CheckResult {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let restriction_enforceable_against_owner;
    let mut grandfather_protection_engaged = false;
    let mut florida_exception_binds_all_owners = false;
    let mut agent_designation_engaged = false;

    match input.regime {
        Regime::Florida => {
            // § 720.306(1)(h) two narrow exceptions bind all owners
            // regardless of grandfather.
            let is_florida_exception = matches!(
                input.restriction_type,
                RestrictionType::ShortTermRentalRestriction
                    | RestrictionType::FrequencyRestrictionThreeOrFewer
            );

            if is_florida_exception {
                florida_exception_binds_all_owners = true;
                restriction_enforceable_against_owner = true;
                notes.push(format!(
                    "§ 720.306(1)(h) EXCEPTION engaged — {} amendments bind ALL owners \
                     regardless of acquisition date or consent. Two narrow exceptions: \
                     (1) leases ≤6 months under § 720.306(1)(h)1; (2) ≤3 rentals/year \
                     under § 720.306(1)(h)2.",
                    match input.restriction_type {
                        RestrictionType::ShortTermRentalRestriction =>
                            "short-term-rental (≤6 month)",
                        RestrictionType::FrequencyRestrictionThreeOrFewer => "≤3-rentals-per-year",
                        _ => "exception",
                    },
                ));
            } else if input.amendment_adopted_after_july_2021 {
                // Grandfather rule applies.
                let owner_in_grandfather_class = (input.owner_acquired_before_amendment
                    && !input.owner_consented_to_amendment)
                    || (input.owner_acquired_via_heir_or_affiliate
                        && !input.transferred_to_unrelated_party);

                if owner_in_grandfather_class && !input.transferred_to_unrelated_party {
                    grandfather_protection_engaged = true;
                    restriction_enforceable_against_owner = false;
                    notes.push(
                        "§ 720.306(1)(h) GRANDFATHER PROTECTION engaged — restriction \
                         not enforceable against this owner. Owner either acquired title \
                         before the amendment without affirmatively consenting OR acquired \
                         via heir/affiliate from a grandfathered prior owner. Restriction \
                         attaches only upon transfer to an unrelated third party."
                            .to_string(),
                    );
                } else {
                    restriction_enforceable_against_owner = true;
                    notes.push(
                        "§ 720.306(1)(h) — restriction enforceable: owner either \
                         acquired title AFTER amendment, affirmatively consented, OR \
                         transferred from grandfathered status to an unrelated party. \
                         Grandfather protection LOST upon transfer to unrelated third \
                         party under § 720.306(1)(h)3."
                            .to_string(),
                    );
                }
            } else {
                // Pre-July-2021 amendments — no Florida grandfather rule applies; declaration controls.
                restriction_enforceable_against_owner = !input.declaration_permits_rental;
                notes.push(
                    "Pre-July-2021 amendments and pre-amendment declarations are NOT \
                     subject to § 720.306(1)(h) grandfather rule. Declaration controls \
                     enforceability under general Florida common-interest community \
                     covenants law."
                        .to_string(),
                );
            }
        }
        Regime::Arizona => {
            // A.R.S. § 33-1806.01 — declaration controls.
            restriction_enforceable_against_owner = !input.declaration_permits_rental;

            if input.agent_designation_signed_and_delivered {
                agent_designation_engaged = true;
                notes.push(
                    "A.R.S. § 33-1806.01(B) — owner has validly designated a third-party \
                     agent. HOA must accept the agent for all association matters except \
                     voting in elections and serving on the board. Signed written \
                     designation + delivery to HOA is the statutory trigger."
                        .to_string(),
                );
            }

            notes.push(format!(
                "A.R.S. § 33-1806.01 — declaration controls rental restrictions. HOA may \
                 prohibit rentals or impose time-period restrictions if such restrictions \
                 appear in the declaration. Statute does NOT set a percentage cap on \
                 rentals. Declaration {} rentals; restriction is {} enforceable.",
                if input.declaration_permits_rental {
                    "PERMITS"
                } else {
                    "RESTRICTS"
                },
                if restriction_enforceable_against_owner {
                    ""
                } else {
                    "NOT"
                },
            ));
        }
        Regime::Default => {
            restriction_enforceable_against_owner = !input.declaration_permits_rental;
            notes.push(
                "Default — declaration controls. No statutory grandfather protection. \
                 HOA amendments bind existing owners under general covenants law subject \
                 to declaration's amendment procedure. Most states follow this default \
                 absent specific legislation."
                    .to_string(),
            );
        }
    }

    // Compliance — restriction enforceability flags compliance from
    // the OWNER'S perspective (compliant = restriction is enforceable,
    // i.e., owner must comply). When grandfather engages, owner is
    // EXEMPT and treated as non-violating.
    if !restriction_enforceable_against_owner
        && !grandfather_protection_engaged
        && !input.declaration_permits_rental
    {
        violations.push(
            "Restriction not enforceable against owner under applicable regime, but \
             declaration does not permit rentals — anomalous state requiring legal \
             review."
                .to_string(),
        );
    }

    notes.push(
        "Sibling modules: `tenant_topa` (tenant opportunity to purchase — distinct from \
         HOA restrictions); `str_regulation` (short-term-rental regulation by local \
         government — distinct from HOA private restrictions); `condominium_conversion_\
         protection` (tenant protection during condo conversions). HOA rental \
         restrictions are PRIVATE COVENANTS enforced by the association; municipal STR \
         laws are PUBLIC REGULATIONS enforced by the local government. A property may \
         simultaneously face both."
            .to_string(),
    );

    CheckResult {
        restriction_enforceable_against_owner,
        grandfather_protection_engaged,
        florida_exception_binds_all_owners,
        agent_designation_engaged,
        compliant: violations.is_empty(),
        violations,
        citation: "Fla. Stat. § 720.306(1)(h) (post-July-2021 HOA rental restriction \
                   grandfather rule); Fla. Stat. § 720.306(1)(h)1 (≤6-month short-term \
                   rental exception — binds all owners); Fla. Stat. § 720.306(1)(h)2 \
                   (≤3-rentals-per-year exception — binds all owners); Fla. Stat. \
                   § 720.306(1)(h)3 (heir/affiliate grandfather preservation; transfer \
                   to unrelated party loses grandfather); A.R.S. § 33-1806.01 (Arizona \
                   planned-community rental restrictions); A.R.S. § 33-1806.01(A) \
                   (declaration-based restriction authority); A.R.S. § 33-1806.01(B) \
                   (third-party agent designation right)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(regime: Regime, restriction_type: RestrictionType) -> Input {
        Input {
            regime,
            restriction_type,
            amendment_adopted_after_july_2021: true,
            owner_acquired_before_amendment: true,
            owner_consented_to_amendment: false,
            owner_acquired_via_heir_or_affiliate: false,
            transferred_to_unrelated_party: false,
            declaration_permits_rental: true,
            agent_designation_signed_and_delivered: false,
        }
    }

    // ── Florida § 720.306(1)(h) grandfather rule ──────────────

    #[test]
    fn florida_pre_amendment_owner_no_consent_grandfather_engaged() {
        let r = check(&input(
            Regime::Florida,
            RestrictionType::GeneralLongTermRestriction,
        ));
        assert!(r.grandfather_protection_engaged);
        assert!(!r.restriction_enforceable_against_owner);
        assert!(r.compliant);
    }

    #[test]
    fn florida_pre_amendment_owner_consented_no_grandfather() {
        let mut b = input(Regime::Florida, RestrictionType::GeneralLongTermRestriction);
        b.owner_consented_to_amendment = true;
        let r = check(&b);
        assert!(!r.grandfather_protection_engaged);
        assert!(r.restriction_enforceable_against_owner);
    }

    #[test]
    fn florida_post_amendment_owner_no_grandfather() {
        let mut b = input(Regime::Florida, RestrictionType::GeneralLongTermRestriction);
        b.owner_acquired_before_amendment = false;
        let r = check(&b);
        assert!(!r.grandfather_protection_engaged);
        assert!(r.restriction_enforceable_against_owner);
    }

    #[test]
    fn florida_heir_acquires_grandfather_preserved() {
        let mut b = input(Regime::Florida, RestrictionType::GeneralLongTermRestriction);
        b.owner_acquired_before_amendment = false;
        b.owner_acquired_via_heir_or_affiliate = true;
        b.transferred_to_unrelated_party = false;
        let r = check(&b);
        assert!(r.grandfather_protection_engaged);
        assert!(!r.restriction_enforceable_against_owner);
    }

    #[test]
    fn florida_transfer_to_unrelated_party_erodes_grandfather() {
        let mut b = input(Regime::Florida, RestrictionType::GeneralLongTermRestriction);
        b.owner_acquired_via_heir_or_affiliate = true;
        b.transferred_to_unrelated_party = true;
        let r = check(&b);
        assert!(!r.grandfather_protection_engaged);
        assert!(r.restriction_enforceable_against_owner);
    }

    // ── Florida exceptions (short-term + frequency) ───────────

    #[test]
    fn florida_short_term_rental_restriction_binds_all_owners() {
        let r = check(&input(
            Regime::Florida,
            RestrictionType::ShortTermRentalRestriction,
        ));
        assert!(r.florida_exception_binds_all_owners);
        assert!(r.restriction_enforceable_against_owner);
        assert!(!r.grandfather_protection_engaged);
    }

    #[test]
    fn florida_three_or_fewer_rentals_restriction_binds_all_owners() {
        let r = check(&input(
            Regime::Florida,
            RestrictionType::FrequencyRestrictionThreeOrFewer,
        ));
        assert!(r.florida_exception_binds_all_owners);
        assert!(r.restriction_enforceable_against_owner);
    }

    #[test]
    fn florida_pre_july_2021_amendment_not_subject_to_grandfather() {
        let mut b = input(Regime::Florida, RestrictionType::GeneralLongTermRestriction);
        b.amendment_adopted_after_july_2021 = false;
        b.declaration_permits_rental = false;
        let r = check(&b);
        // Declaration controls — restriction enforceable.
        assert!(!r.grandfather_protection_engaged);
        assert!(r.restriction_enforceable_against_owner);
    }

    // ── Arizona § 33-1806.01 ───────────────────────────────────

    #[test]
    fn arizona_declaration_permits_no_enforceable_restriction() {
        let r = check(&input(
            Regime::Arizona,
            RestrictionType::GeneralLongTermRestriction,
        ));
        assert!(!r.restriction_enforceable_against_owner);
    }

    #[test]
    fn arizona_declaration_restricts_enforceable() {
        let mut b = input(Regime::Arizona, RestrictionType::GeneralLongTermRestriction);
        b.declaration_permits_rental = false;
        let r = check(&b);
        assert!(r.restriction_enforceable_against_owner);
    }

    #[test]
    fn arizona_agent_designation_validly_invoked() {
        let mut b = input(Regime::Arizona, RestrictionType::GeneralLongTermRestriction);
        b.agent_designation_signed_and_delivered = true;
        let r = check(&b);
        assert!(r.agent_designation_engaged);
    }

    #[test]
    fn arizona_no_agent_designation_no_engagement() {
        let r = check(&input(
            Regime::Arizona,
            RestrictionType::GeneralLongTermRestriction,
        ));
        assert!(!r.agent_designation_engaged);
    }

    // ── Default — declaration controls ─────────────────────────

    #[test]
    fn default_declaration_permits_no_enforceable_restriction() {
        let r = check(&input(
            Regime::Default,
            RestrictionType::GeneralLongTermRestriction,
        ));
        assert!(!r.restriction_enforceable_against_owner);
    }

    #[test]
    fn default_declaration_restricts_enforceable() {
        let mut b = input(Regime::Default, RestrictionType::GeneralLongTermRestriction);
        b.declaration_permits_rental = false;
        let r = check(&b);
        assert!(r.restriction_enforceable_against_owner);
        // No grandfather engaged in Default regime.
        assert!(!r.grandfather_protection_engaged);
    }

    // ── Multi-regime invariants ───────────────────────────────

    #[test]
    fn only_florida_offers_grandfather_protection_invariant() {
        // 3-regime sweep — grandfather protection engaged only in FL.
        for regime in [Regime::Florida, Regime::Arizona, Regime::Default] {
            let mut b = input(regime, RestrictionType::GeneralLongTermRestriction);
            b.declaration_permits_rental = false;
            let r = check(&b);
            let expected = matches!(regime, Regime::Florida);
            assert_eq!(r.grandfather_protection_engaged, expected, "{:?}", regime);
        }
    }

    #[test]
    fn only_arizona_has_statutory_agent_designation_invariant() {
        // 3-regime sweep — agent designation only engages in AZ.
        for regime in [Regime::Florida, Regime::Arizona, Regime::Default] {
            let mut b = input(regime, RestrictionType::GeneralLongTermRestriction);
            b.agent_designation_signed_and_delivered = true;
            let r = check(&b);
            let expected = matches!(regime, Regime::Arizona);
            assert_eq!(r.agent_designation_engaged, expected, "{:?}", regime);
        }
    }

    #[test]
    fn florida_grandfather_truth_table() {
        // 4-cell sweep: pre/post amendment × consented/not.
        // Grandfather engaged only when pre-amendment + not-consented.
        let cells = [
            (true, false, true),   // pre + no-consent → grandfather
            (true, true, false),   // pre + consent → no grandfather
            (false, false, false), // post → no grandfather
            (false, true, false),  // post + consent → no grandfather
        ];
        for (pre, consented, expected_grandfather) in cells.iter() {
            let mut b = input(Regime::Florida, RestrictionType::GeneralLongTermRestriction);
            b.owner_acquired_before_amendment = *pre;
            b.owner_consented_to_amendment = *consented;
            let r = check(&b);
            assert_eq!(
                r.grandfather_protection_engaged, *expected_grandfather,
                "pre={} consented={}",
                pre, consented
            );
        }
    }

    #[test]
    fn florida_exception_overrides_grandfather_invariant() {
        // Both Florida exceptions bind all owners regardless of grandfather facts.
        for restriction in [
            RestrictionType::ShortTermRentalRestriction,
            RestrictionType::FrequencyRestrictionThreeOrFewer,
        ] {
            let b = input(Regime::Florida, restriction);
            // Even with all grandfather-favoring facts.
            assert!(b.owner_acquired_before_amendment);
            assert!(!b.owner_consented_to_amendment);
            let r = check(&b);
            assert!(r.florida_exception_binds_all_owners);
            assert!(r.restriction_enforceable_against_owner);
            assert!(!r.grandfather_protection_engaged);
        }
    }

    #[test]
    fn citation_pins_all_regime_authorities() {
        let r = check(&input(
            Regime::Florida,
            RestrictionType::GeneralLongTermRestriction,
        ));
        assert!(r.citation.contains("§ 720.306(1)(h)"));
        assert!(r.citation.contains("§ 720.306(1)(h)1"));
        assert!(r.citation.contains("§ 720.306(1)(h)2"));
        assert!(r.citation.contains("§ 720.306(1)(h)3"));
        assert!(r.citation.contains("A.R.S. § 33-1806.01"));
        assert!(r.citation.contains("§ 33-1806.01(A)"));
        assert!(r.citation.contains("§ 33-1806.01(B)"));
    }

    #[test]
    fn sibling_distinction_note_present() {
        let r = check(&input(
            Regime::Florida,
            RestrictionType::GeneralLongTermRestriction,
        ));
        assert!(
            r.notes.iter().any(|n| n.contains("tenant_topa")
                && n.contains("str_regulation")
                && n.contains("condominium_conversion_protection")
                && n.contains("PRIVATE COVENANTS")
                && n.contains("PUBLIC REGULATIONS")),
            "sibling distinction note must distinguish HOA private covenants vs. municipal STR regulation"
        );
    }

    #[test]
    fn florida_grandfather_eroded_by_transfer_truth_table() {
        // Owner-acquired-via-heir × transferred-to-unrelated truth table.
        // Transfer to unrelated party ALWAYS erodes grandfather under
        // § 720.306(1)(h)3 regardless of how the current owner originally
        // qualified.
        let cells = [
            (true, false, true),  // heir + not-transferred → grandfather preserved
            (true, true, false),  // heir + transferred → grandfather lost
            (false, false, true), // pre-amendment-direct + not-transferred → grandfather
            (false, true, false), // pre-amendment-direct + transferred → grandfather lost
        ];
        for (via_heir, transferred, expected_grandfather) in cells.iter() {
            let mut b = input(Regime::Florida, RestrictionType::GeneralLongTermRestriction);
            b.owner_acquired_before_amendment = !via_heir;
            b.owner_acquired_via_heir_or_affiliate = *via_heir;
            b.transferred_to_unrelated_party = *transferred;
            let r = check(&b);
            assert_eq!(
                r.grandfather_protection_engaged, *expected_grandfather,
                "via_heir={} transferred={}",
                via_heir, transferred
            );
        }
    }
}
