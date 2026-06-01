//! State landlord firearms-in-rental-unit tenant right compliance
//! check.
//!
//! Strong state-by-state variation. Federal floor: New York State
//! Rifle & Pistol Ass'n v. Bruen, 597 U.S. 1 (2022) confirms a
//! constitutional right to keep and bear arms in the home. Post-
//! Bruen federal courts have struck down public-housing handgun
//! bans (Cortland County 2024 permanent injunction). Private
//! landlords retain general contract-based ability to restrict
//! via lease in states without statutory tenant protection.
//!
//! Six regimes:
//!
//!   - **Minnesota** — Minn. Stat. § 504B.211 + Chapter 624 — a
//!     landlord cannot restrict the lawful possession, carry, or
//!     transportation of firearms by tenants or their guests in
//!     a rental unit. Strongest pro-tenant statutory protection
//!     in the matrix.
//!
//!   - **Virginia** — Va. Code § 55.1-1208(A)(15) — prohibits
//!     rental agreements in PUBLIC HOUSING from requiring tenant
//!     to agree to a prohibition or restriction of any LAWFUL
//!     possession of a firearm within individual dwelling units
//!     unless required by federal law or regulation. PRIVATE
//!     landlords may still restrict by lease.
//!
//!   - **Tennessee** — current law permits private landlords to
//!     prohibit firearms via lease clause (including for handgun-
//!     carry-permit holders). Tennessee SB0350 (2026 session)
//!     proposes to flip this to pro-tenant. Audit treats current
//!     state law: landlord restriction permitted.
//!
//!   - **Wisconsin** — Wis. Stat. § 175.60 (Concealed Carry
//!     Licensee Protections) — protects all occupants of a rented
//!     dwelling where a concealed-carry licensee lives from
//!     landlord restriction.
//!
//!   - **NewYork** — state statute silent on private landlord
//!     restriction. Federal court 2024 permanent injunction
//!     (Cortland County) struck down public-housing handgun ban
//!     under N.Y. State Rifle & Pistol Ass'n v. Bruen, 597 U.S. 1
//!     (2022). Public housing tenants protected; private landlords
//!     may still restrict via lease.
//!
//!   - **Default** — state silent; private landlord may restrict
//!     via lease per general contract law. Federal Bruen floor
//!     applies only to government action (public housing).
//!
//! Citations: Minn. Stat. § 504B.211 + Chapter 624 (Minnesota
//! tenant-firearms statutory protection); Va. Code § 55.1-1208(A)(15)
//! (Virginia public housing); Wis. Stat. § 175.60 (Wisconsin
//! Concealed Carry Licensee Protections); New York State Rifle &
//! Pistol Ass'n v. Bruen, 597 U.S. 1 (2022) (federal 2A floor);
//! Cortland County public-housing handgun ban (2024 federal court
//! permanent injunction); Tennessee SB0350 (2026 session — pending
//! pro-tenant statute).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Minnesota,
    Virginia,
    Tennessee,
    Wisconsin,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HousingType {
    /// Public housing — federal Section 8 voucher, public-housing
    /// authority, or other government-subsidized rental. Federal
    /// 2A Bruen floor applies as STATE ACTION.
    PublicHousing,
    /// Private rental — landlord retains general contract-based
    /// restriction authority absent state statute.
    PrivateRental,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    pub housing_type: HousingType,
    /// Whether the tenant holds a concealed-carry permit / license
    /// (relevant for Wisconsin § 175.60 protection).
    pub tenant_holds_concealed_carry_permit: bool,
    /// Whether the firearm possession is lawful under federal +
    /// state law (background check, FOID, no felony bar, etc.).
    /// Universal threshold across all regimes — illegal possession
    /// is never protected.
    pub firearm_possession_lawful: bool,
    /// Whether the lease contains a clause restricting or
    /// prohibiting firearms in the unit.
    pub lease_clause_restricts_firearms: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// True if the landlord's restriction (lease clause + policy)
    /// is permissible under the applicable regime.
    pub landlord_restriction_permissible: bool,
    /// True if the regime provides statutory tenant protection
    /// against landlord firearms restriction.
    pub statutory_tenant_protection: bool,
    /// True if the federal 2A Bruen floor applies (public housing
    /// = state action triggers Bruen).
    pub federal_2a_floor_applies: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Input) -> CheckResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    // Universal threshold — illegal possession is never protected.
    if !input.firearm_possession_lawful {
        notes.push(
            "Universal threshold — firearm possession must be lawful under federal + state law \
             (background check, FOID, no felony bar, etc.). Illegal possession is never \
             protected regardless of regime."
                .to_string(),
        );
    }

    // Federal Bruen floor — public housing = state action.
    let federal_2a_floor_applies = matches!(input.housing_type, HousingType::PublicHousing)
        && input.firearm_possession_lawful;

    let (statutory_tenant_protection, citation): (bool, &'static str) = match input.regime {
        Regime::Minnesota => (
            input.firearm_possession_lawful,
            "Minn. Stat. § 504B.211 + Minn. Stat. Chapter 624 (Minnesota landlord cannot \
             restrict lawful possession + carry + transportation of firearms by tenants or \
             guests in rental unit — strongest pro-tenant statutory protection in matrix)",
        ),
        Regime::Virginia => {
            let protection = matches!(input.housing_type, HousingType::PublicHousing)
                && input.firearm_possession_lawful;
            (
                protection,
                "Va. Code § 55.1-1208(A)(15) (Virginia public housing prohibition on rental \
                 agreement firearms restriction — applies only to PUBLIC HOUSING; private \
                 landlords may still restrict by lease)",
            )
        }
        Regime::Tennessee => (
            // Current TN law permits restriction; SB0350 pending.
            false,
            "Tennessee current state law (private landlord may prohibit firearms via lease \
             clause; Tennessee SB0350 in 2026 session proposes to flip to pro-tenant — audit \
             tracks current state law)",
        ),
        Regime::Wisconsin => (
            input.tenant_holds_concealed_carry_permit && input.firearm_possession_lawful,
            "Wis. Stat. § 175.60 (Wisconsin Concealed Carry Licensee Protections — protects \
             all occupants of a rented dwelling where a concealed-carry licensee lives from \
             landlord restriction)",
        ),
        Regime::NewYork => {
            let protection = matches!(input.housing_type, HousingType::PublicHousing)
                && input.firearm_possession_lawful;
            (
                protection,
                "New York state statute silent on private landlord restriction; federal court \
                 2024 permanent injunction (Cortland County public-housing handgun ban) under \
                 N.Y. State Rifle & Pistol Ass'n v. Bruen, 597 U.S. 1 (2022) protects public-\
                 housing tenants; private landlords may still restrict via lease",
            )
        }
        Regime::Default => (
            false,
            "Default — state statute silent; private landlord may restrict via lease per \
             general contract law; federal Bruen floor (N.Y. State Rifle & Pistol Ass'n v. \
             Bruen, 597 U.S. 1 (2022)) applies only to government action (public housing)",
        ),
    };

    let landlord_restriction_permissible = if statutory_tenant_protection
        && input.lease_clause_restricts_firearms
    {
        violations.push(format!(
            "Lease clause restricting lawful firearms possession violates {:?} regime's \
             statutory tenant protection.",
            input.regime,
        ));
        false
    } else if federal_2a_floor_applies && input.lease_clause_restricts_firearms {
        violations.push(
            "Lease clause restricting lawful firearms possession in PUBLIC HOUSING violates \
             federal 2A Bruen floor — public housing is state action subject to Second \
             Amendment scrutiny."
                .to_string(),
        );
        false
    } else if !input.firearm_possession_lawful {
        // Illegal possession — landlord restriction is always
        // permissible (and any prohibition is enforceable).
        true
    } else {
        true
    };

    // Wisconsin-specific note.
    if matches!(input.regime, Regime::Wisconsin) && !input.tenant_holds_concealed_carry_permit {
        notes.push(
            "Wis. Stat. § 175.60 — protection requires a concealed-carry licensee to live in \
             the rented dwelling. Tenant does not hold a concealed-carry permit; protection \
             does not engage."
                .to_string(),
        );
    }

    // Public housing federal 2A note.
    if matches!(input.housing_type, HousingType::PublicHousing)
        && input.firearm_possession_lawful
    {
        notes.push(
            "Public housing = STATE ACTION; federal 2A Bruen floor (597 U.S. 1 (2022)) applies. \
             Post-Bruen federal courts have struck down public-housing handgun bans (Cortland \
             County 2024 permanent injunction)."
                .to_string(),
        );
    }

    notes.push(
        "Companion to lease_disclosures + plain_language_lease (general lease-clause review). \
         This module addresses the specific tenant right to lawfully possess firearms in the \
         rental unit."
            .to_string(),
    );

    CheckResult {
        landlord_restriction_permissible,
        statutory_tenant_protection,
        federal_2a_floor_applies,
        violations,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(regime: Regime) -> Input {
        Input {
            regime,
            housing_type: HousingType::PrivateRental,
            tenant_holds_concealed_carry_permit: false,
            firearm_possession_lawful: true,
            lease_clause_restricts_firearms: false,
        }
    }

    // ── Minnesota § 504B.211 — strongest pro-tenant protection ──

    #[test]
    fn minnesota_lawful_possession_statutorily_protected() {
        let r = check(&base(Regime::Minnesota));
        assert!(r.statutory_tenant_protection);
        assert!(r.landlord_restriction_permissible);
        assert!(r.citation.contains("§ 504B.211"));
    }

    #[test]
    fn minnesota_lease_restriction_violates_statute() {
        let mut i = base(Regime::Minnesota);
        i.lease_clause_restricts_firearms = true;
        let r = check(&i);
        assert!(!r.landlord_restriction_permissible);
        assert!(r.violations.iter().any(|v| v.contains("Minnesota")));
    }

    #[test]
    fn minnesota_unlawful_possession_no_protection() {
        let mut i = base(Regime::Minnesota);
        i.firearm_possession_lawful = false;
        i.lease_clause_restricts_firearms = true;
        let r = check(&i);
        // Unlawful possession is never protected — landlord may restrict.
        assert!(!r.statutory_tenant_protection);
        assert!(r.landlord_restriction_permissible);
    }

    // ── Virginia § 55.1-1208(A)(15) — public-housing only ───────

    #[test]
    fn virginia_public_housing_protected() {
        let mut i = base(Regime::Virginia);
        i.housing_type = HousingType::PublicHousing;
        let r = check(&i);
        assert!(r.statutory_tenant_protection);
        assert!(r.federal_2a_floor_applies);
        assert!(r.citation.contains("§ 55.1-1208(A)(15)"));
    }

    #[test]
    fn virginia_private_rental_landlord_may_restrict() {
        let mut i = base(Regime::Virginia);
        i.housing_type = HousingType::PrivateRental;
        i.lease_clause_restricts_firearms = true;
        let r = check(&i);
        // Private rental — restriction permissible.
        assert!(!r.statutory_tenant_protection);
        assert!(r.landlord_restriction_permissible);
    }

    #[test]
    fn virginia_public_housing_lease_restriction_violation() {
        let mut i = base(Regime::Virginia);
        i.housing_type = HousingType::PublicHousing;
        i.lease_clause_restricts_firearms = true;
        let r = check(&i);
        assert!(!r.landlord_restriction_permissible);
    }

    // ── Tennessee — landlord may restrict (SB0350 pending) ──────

    #[test]
    fn tennessee_landlord_may_restrict_via_lease() {
        let mut i = base(Regime::Tennessee);
        i.lease_clause_restricts_firearms = true;
        let r = check(&i);
        // Current TN law permits restriction.
        assert!(!r.statutory_tenant_protection);
        assert!(r.landlord_restriction_permissible);
        assert!(r.citation.contains("SB0350"));
    }

    // ── Wisconsin § 175.60 — concealed-carry licensee protection ──

    #[test]
    fn wisconsin_concealed_carry_licensee_protected() {
        let mut i = base(Regime::Wisconsin);
        i.tenant_holds_concealed_carry_permit = true;
        let r = check(&i);
        assert!(r.statutory_tenant_protection);
        assert!(r.citation.contains("§ 175.60"));
    }

    #[test]
    fn wisconsin_no_concealed_carry_permit_no_protection() {
        let mut i = base(Regime::Wisconsin);
        i.tenant_holds_concealed_carry_permit = false;
        let r = check(&i);
        assert!(!r.statutory_tenant_protection);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 175.60") && n.contains("does not engage"))
        );
    }

    #[test]
    fn wisconsin_concealed_carry_licensee_lease_restriction_violation() {
        let mut i = base(Regime::Wisconsin);
        i.tenant_holds_concealed_carry_permit = true;
        i.lease_clause_restricts_firearms = true;
        let r = check(&i);
        assert!(!r.landlord_restriction_permissible);
    }

    // ── New York — Bruen + public-housing federal floor ────────

    #[test]
    fn new_york_public_housing_protected_via_bruen() {
        let mut i = base(Regime::NewYork);
        i.housing_type = HousingType::PublicHousing;
        let r = check(&i);
        assert!(r.statutory_tenant_protection);
        assert!(r.federal_2a_floor_applies);
        assert!(r.citation.contains("Cortland County"));
        assert!(r.citation.contains("Bruen"));
    }

    #[test]
    fn new_york_private_rental_landlord_may_restrict() {
        let mut i = base(Regime::NewYork);
        i.housing_type = HousingType::PrivateRental;
        i.lease_clause_restricts_firearms = true;
        let r = check(&i);
        assert!(!r.statutory_tenant_protection);
        assert!(r.landlord_restriction_permissible);
    }

    // ── Default ─────────────────────────────────────────────────

    #[test]
    fn default_landlord_may_restrict_via_lease() {
        let mut i = base(Regime::Default);
        i.lease_clause_restricts_firearms = true;
        let r = check(&i);
        assert!(!r.statutory_tenant_protection);
        assert!(r.landlord_restriction_permissible);
        assert!(r.citation.contains("state statute silent"));
    }

    #[test]
    fn default_public_housing_protected_via_bruen() {
        let mut i = base(Regime::Default);
        i.housing_type = HousingType::PublicHousing;
        i.lease_clause_restricts_firearms = true;
        let r = check(&i);
        // Default + public housing → federal 2A floor protects.
        assert!(r.federal_2a_floor_applies);
        assert!(!r.landlord_restriction_permissible);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("PUBLIC HOUSING") && v.contains("Bruen"))
        );
    }

    // ── Universal threshold: lawfulness ────────────────────────

    #[test]
    fn unlawful_possession_never_protected_across_all_regimes_invariant() {
        for &regime in &[
            Regime::Minnesota,
            Regime::Virginia,
            Regime::Tennessee,
            Regime::Wisconsin,
            Regime::NewYork,
            Regime::Default,
        ] {
            let mut i = base(regime);
            i.firearm_possession_lawful = false;
            i.lease_clause_restricts_firearms = true;
            i.tenant_holds_concealed_carry_permit = true;
            i.housing_type = HousingType::PublicHousing;
            let r = check(&i);
            assert!(
                !r.statutory_tenant_protection,
                "{:?}: unlawful possession must not be protected",
                regime,
            );
            assert!(
                r.landlord_restriction_permissible,
                "{:?}: landlord may restrict unlawful possession",
                regime,
            );
        }
    }

    // ── Regression-critical multi-regime invariants ────────────

    #[test]
    fn only_minnesota_protects_lawful_possession_in_private_rental_invariant() {
        // Minnesota is the only regime where statutory protection
        // extends to PRIVATE RENTAL without additional condition
        // (CCW permit, public housing).
        let mn = check(&base(Regime::Minnesota));
        assert!(mn.statutory_tenant_protection);

        for &regime in &[
            Regime::Virginia,
            Regime::Tennessee,
            Regime::NewYork,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(
                !r.statutory_tenant_protection,
                "{:?}: must NOT statutorily protect private rental without condition",
                regime,
            );
        }
        // Wisconsin requires CCW — without it, no protection.
        let wi_no_ccw = check(&base(Regime::Wisconsin));
        assert!(!wi_no_ccw.statutory_tenant_protection);
    }

    #[test]
    fn only_va_and_ny_have_public_housing_specific_protection_invariant() {
        for &regime in &[Regime::Virginia, Regime::NewYork] {
            let mut i = base(regime);
            i.housing_type = HousingType::PublicHousing;
            assert!(check(&i).statutory_tenant_protection);
        }
        // Tennessee + Default — even public housing not protected
        // by STATE statute (federal 2A Bruen floor still applies).
        for &regime in &[Regime::Tennessee, Regime::Default] {
            let mut i = base(regime);
            i.housing_type = HousingType::PublicHousing;
            let r = check(&i);
            assert!(
                !r.statutory_tenant_protection,
                "{:?}: must not have state statutory public-housing protection",
                regime,
            );
            // BUT federal 2A floor still applies.
            assert!(r.federal_2a_floor_applies);
        }
    }

    #[test]
    fn only_wisconsin_uses_ccw_permit_threshold_invariant() {
        // For Wisconsin, CCW flag changes protection status.
        let mut wi_no_ccw = base(Regime::Wisconsin);
        wi_no_ccw.tenant_holds_concealed_carry_permit = false;
        let mut wi_ccw = base(Regime::Wisconsin);
        wi_ccw.tenant_holds_concealed_carry_permit = true;
        assert_ne!(
            check(&wi_no_ccw).statutory_tenant_protection,
            check(&wi_ccw).statutory_tenant_protection,
        );

        // For other regimes, CCW flag is a no-op.
        for &regime in &[
            Regime::Minnesota,
            Regime::Virginia,
            Regime::Tennessee,
            Regime::NewYork,
            Regime::Default,
        ] {
            let mut a = base(regime);
            a.tenant_holds_concealed_carry_permit = false;
            let mut b = base(regime);
            b.tenant_holds_concealed_carry_permit = true;
            assert_eq!(
                check(&a).statutory_tenant_protection,
                check(&b).statutory_tenant_protection,
                "{:?}: CCW flag must be a no-op",
                regime,
            );
        }
    }

    #[test]
    fn federal_2a_floor_engages_for_public_housing_across_all_regimes_invariant() {
        for &regime in &[
            Regime::Minnesota,
            Regime::Virginia,
            Regime::Tennessee,
            Regime::Wisconsin,
            Regime::NewYork,
            Regime::Default,
        ] {
            let mut i = base(regime);
            i.housing_type = HousingType::PublicHousing;
            let r = check(&i);
            assert!(
                r.federal_2a_floor_applies,
                "{:?}: federal 2A floor must engage for public housing",
                regime,
            );
        }
        // Private rental — federal 2A floor does NOT engage.
        for &regime in &[
            Regime::Minnesota,
            Regime::Virginia,
            Regime::Tennessee,
            Regime::Wisconsin,
            Regime::NewYork,
            Regime::Default,
        ] {
            let mut i = base(regime);
            i.housing_type = HousingType::PrivateRental;
            let r = check(&i);
            assert!(
                !r.federal_2a_floor_applies,
                "{:?}: federal 2A floor must NOT engage for private rental",
                regime,
            );
        }
    }

    #[test]
    fn citation_pins_authority_per_regime() {
        assert!(
            check(&base(Regime::Minnesota))
                .citation
                .contains("§ 504B.211")
        );
        assert!(
            check(&base(Regime::Virginia))
                .citation
                .contains("§ 55.1-1208(A)(15)")
        );
        assert!(
            check(&base(Regime::Tennessee))
                .citation
                .contains("SB0350")
        );
        assert!(check(&base(Regime::Wisconsin)).citation.contains("§ 175.60"));
        assert!(check(&base(Regime::NewYork)).citation.contains("Bruen"));
        assert!(check(&base(Regime::Default)).citation.contains("Bruen"));
    }

    #[test]
    fn sibling_module_note_present_across_all_regimes() {
        for &regime in &[
            Regime::Minnesota,
            Regime::Virginia,
            Regime::Tennessee,
            Regime::Wisconsin,
            Regime::NewYork,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(
                r.notes.iter().any(|n| n.contains("lease_disclosures")
                    && n.contains("plain_language_lease")),
                "{:?}: sibling-module note must be present",
                regime,
            );
        }
    }
}
