//! State landlord religious-display / mezuzah-on-doorpost tenant
//! right compliance check.
//!
//! Post-2010 wave of state statutes protecting tenants' right to
//! affix religious items (most prominently the Jewish mezuzah, but
//! the statutes apply to any religiously-motivated display) on
//! the entry door or doorframe of a leased dwelling. Driven by
//! Bloch v. Frischholz, 587 F.3d 771 (7th Cir. 2009) (en banc)
//! which held that an HOA's removal of mezuzot violated the Fair
//! Housing Act.
//!
//! Six state regimes + federal FHA fallback:
//!
//!   - **California** — Cal. Civ. Code § 1940.45 (SB 652, the
//!     "Mezuzah Bill," signed 2024). Broadest scope in the
//!     matrix — explicitly covers landlord-tenant relationships
//!     and includes dormitory rooms AND apartments. Property
//!     owner SHALL NOT enforce or adopt restrictive covenant or
//!     other restriction that prohibits one or more religious
//!     items from being displayed or affixed on any entry door
//!     or doorframe.
//!
//!   - **Texas** — Tex. Prop. Code § 202.018. Property owners'
//!     association may not enforce or adopt a provision in a
//!     dedicatory instrument prohibiting display motivated by
//!     resident's sincere religious belief. Primarily HOA-focused
//!     but reaches landlord-tenant relationships through
//!     restrictive-covenant analysis.
//!
//!   - **Florida** — Fla. Stat. § 720.3045. HOA-focused with
//!     explicit tenant extension — "association may not restrict
//!     parcel owners or their tenants from installing, displaying,
//!     or storing any items which are not visible from the
//!     parcel's frontage or an adjacent parcel."
//!
//!   - **Illinois** — 765 ILCS 605/18.4 (the Illinois "Mezuzah
//!     Law"). Condominium-association-focused; prohibits Illinois
//!     condo boards from banning religious symbols on doorposts.
//!     Coverage to rentals via FHA enforcement.
//!
//!   - **Connecticut** — Conn. Gen. Stat. § 47-230a (Common
//!     Interest Ownership Act). Allows display of religiously-
//!     motivated objects on entry door or entry door frame
//!     subject to reasonable restrictions.
//!
//!   - **RhodeIsland** — parallel statute (R.I. Gen. Laws
//!     § 34-36.1-3.18 protects display in condominium common-
//!     interest communities).
//!
//!   - **NewYork** — N.Y. Real Prop. Law § 235-h NOT YET ENACTED
//!     at state level (S4466 proposed). Tenants rely on FHA
//!     religious-discrimination protection and case law (e.g.,
//!     2009 Cuomo settlement of Suffolk County HOA mezuzah
//!     discrimination complaint).
//!
//!   - **Default** — 42 U.S.C. § 3604(b) federal Fair Housing Act
//!     religious-discrimination protection. Bloch v. Frischholz
//!     established that mezuzah removal can constitute § 3604
//!     religious discrimination.
//!
//! Citations: Cal. Civ. Code § 1940.45 (SB 652); Tex. Prop. Code
//! § 202.018; Fla. Stat. § 720.3045; 765 ILCS 605/18.4; Conn.
//! Gen. Stat. § 47-230a (CIOA); R.I. Gen. Laws § 34-36.1-3.18;
//! Bloch v. Frischholz, 587 F.3d 771 (7th Cir. 2009) (en banc)
//! (mezuzah removal as § 3604 religious discrimination); 42
//! U.S.C. § 3604(b) (FHA religious-discrimination federal floor).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Texas,
    Florida,
    Illinois,
    Connecticut,
    RhodeIsland,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DwellingType {
    /// Standard apartment unit.
    Apartment,
    /// Dormitory room (CA § 1940.45 explicitly covers).
    Dormitory,
    /// Owned condominium / cooperative unit.
    Condominium,
    /// Single-family home rental.
    SingleFamily,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    pub dwelling_type: DwellingType,
    /// Whether the landlord / HOA has imposed a restriction
    /// prohibiting display.
    pub restriction_imposed: bool,
    /// Whether the display is motivated by a sincere religious
    /// belief (the universal threshold under every state statute).
    pub motivated_by_sincere_religious_belief: bool,
    /// Whether the item is displayed on an entry door or entry
    /// doorframe (statutes typically limit protection to that
    /// location).
    pub on_entry_door_or_doorframe: bool,
    /// Whether the restriction is a temporary removal-for-paint /
    /// removal-for-repair carve-out (universally permitted under
    /// each statute).
    pub temporary_removal_for_repair: bool,
    /// Item size in inches along longest dimension. Some HOA
    /// restrictions cap at 25 inches per door per statute design
    /// (TX + FL have similar reasonable-size language).
    pub item_size_inches: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// True if the regime statutorily protects the display.
    pub statutory_protection: bool,
    /// True if the restriction (if any) is permissible under the
    /// applicable regime.
    pub restriction_permissible: bool,
    /// Whether the regime covers landlord-tenant relationships
    /// (not just HOA / condominium associations).
    pub covers_landlord_tenant: bool,
    /// Whether the regime covers dormitory rooms (only CA
    /// § 1940.45 explicitly).
    pub covers_dormitory: bool,
    /// Whether failure to allow the display would constitute
    /// religious discrimination under federal FHA § 3604(b) or
    /// state parallel statute.
    pub fha_section_3604_religious_discrimination_risk: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Input) -> CheckResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    let (statutory_protection, covers_landlord_tenant, covers_dormitory, citation): (
        bool,
        bool,
        bool,
        &'static str,
    ) = match input.regime {
        Regime::California => (
            true,
            true,
            true,
            "Cal. Civ. Code § 1940.45 (SB 652 \"Mezuzah Bill,\" signed 2024 — property owner \
             SHALL NOT enforce or adopt restrictive covenant or other restriction prohibiting \
             religious-item display on any entry door or doorframe; covers landlord-tenant + \
             dormitory rooms + apartments)",
        ),
        Regime::Texas => (
            true,
            false,
            false,
            "Tex. Prop. Code § 202.018 (property owners' association may not enforce or adopt \
             provision prohibiting display motivated by sincere religious belief; HOA-focused \
             with rental coverage via restrictive-covenant analysis)",
        ),
        Regime::Florida => (
            true,
            true,
            false,
            "Fla. Stat. § 720.3045 (HOA-focused with explicit tenant extension — association \
             may not restrict parcel owners or their tenants from installing, displaying, or \
             storing religious items)",
        ),
        Regime::Illinois => (
            true,
            false,
            false,
            "765 ILCS 605/18.4 (Illinois Mezuzah Law — condominium-association-focused; \
             prohibits Illinois condo boards from banning religious symbols on doorposts; \
             rental coverage via FHA enforcement)",
        ),
        Regime::Connecticut => (
            true,
            false,
            false,
            "Conn. Gen. Stat. § 47-230a (Common Interest Ownership Act — allows display of \
             religiously-motivated objects on entry door or entry door frame subject to \
             reasonable restrictions)",
        ),
        Regime::RhodeIsland => (
            true,
            false,
            false,
            "R.I. Gen. Laws § 34-36.1-3.18 (Rhode Island parallel to CT — protects display in \
             condominium common-interest communities)",
        ),
        Regime::NewYork => (
            false,
            false,
            false,
            "N.Y. state — no enacted statute (S4466 proposed but not enacted at state level); \
             tenants rely on 42 U.S.C. § 3604(b) Fair Housing Act religious-discrimination \
             protection and Bloch v. Frischholz, 587 F.3d 771 (7th Cir. 2009) (en banc) case \
             law",
        ),
        Regime::Default => (
            false,
            false,
            false,
            "42 U.S.C. § 3604(b) Fair Housing Act religious-discrimination floor; Bloch v. \
             Frischholz, 587 F.3d 771 (7th Cir. 2009) (en banc) (mezuzah removal can \
             constitute § 3604 religious discrimination)",
        ),
    };

    // Threshold gates — every regime requires sincere religious
    // belief + display on entry door or doorframe.
    if !input.motivated_by_sincere_religious_belief {
        notes.push(
            "Display must be motivated by sincere religious belief (universal threshold under \
             every state statute + FHA § 3604(b))."
                .to_string(),
        );
    }
    if !input.on_entry_door_or_doorframe {
        notes.push(
            "Statutes generally limit protection to displays on entry door or entry doorframe; \
             interior-only displays may fall outside statutory scope."
                .to_string(),
        );
    }

    // Temporary removal carve-out (universally permitted).
    if input.temporary_removal_for_repair {
        notes.push(
            "Temporary removal for painting, repair, or maintenance is universally permitted \
             across the regimes."
                .to_string(),
        );
    }

    // CA § 1940.45 dormitory + apartment coverage.
    if matches!(input.regime, Regime::California)
        && matches!(input.dwelling_type, DwellingType::Dormitory)
    {
        notes.push(
            "Cal. Civ. Code § 1940.45 explicitly extends to dormitory rooms — the unique CA \
             expansion in this matrix."
                .to_string(),
        );
    }

    // Determine if the restriction violates the statute.
    let display_protected = statutory_protection
        && input.motivated_by_sincere_religious_belief
        && input.on_entry_door_or_doorframe
        && !input.temporary_removal_for_repair;

    let restriction_permissible = if input.restriction_imposed && display_protected {
        // Restriction violates the statute.
        violations.push(format!(
            "Restriction on religious-display motivated by sincere religious belief violates \
             {:?} regime's statutory protection.",
            input.regime,
        ));
        false
    } else {
        // No restriction OR no statutory protection (so restriction
        // is governed by other law).
        true
    };

    // FHA § 3604 religious-discrimination risk — applies even where
    // no state statute exists (Default + NY paths).
    let fha_risk = input.restriction_imposed
        && input.motivated_by_sincere_religious_belief
        && input.on_entry_door_or_doorframe
        && !input.temporary_removal_for_repair;

    if fha_risk && !statutory_protection {
        notes.push(
            "Bloch v. Frischholz (7th Cir. 2009 en banc) — restriction on religious display \
             may constitute religious discrimination under 42 U.S.C. § 3604(b) Fair Housing \
             Act regardless of state-statute coverage."
                .to_string(),
        );
    }

    notes.push(
        "Companion to service_animal (FHA reasonable accommodation for assistance animals) \
         and reasonable_accommodation_modification (FHA § 3604(f)(3)(A)-(B) accommodation + \
         modification). This module addresses RELIGIOUS-display rights specifically, distinct \
         from disability accommodation."
            .to_string(),
    );

    CheckResult {
        statutory_protection,
        restriction_permissible,
        covers_landlord_tenant,
        covers_dormitory,
        fha_section_3604_religious_discrimination_risk: fha_risk,
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
            dwelling_type: DwellingType::Apartment,
            restriction_imposed: false,
            motivated_by_sincere_religious_belief: true,
            on_entry_door_or_doorframe: true,
            temporary_removal_for_repair: false,
            item_size_inches: 5,
        }
    }

    // ── California § 1940.45 — broadest coverage ────────────────

    #[test]
    fn california_protects_apartment_display() {
        let r = check(&base(Regime::California));
        assert!(r.statutory_protection);
        assert!(r.covers_landlord_tenant);
        assert!(r.covers_dormitory);
        assert!(r.citation.contains("§ 1940.45"));
        assert!(r.citation.contains("SB 652"));
    }

    #[test]
    fn california_dormitory_explicitly_covered() {
        let mut i = base(Regime::California);
        i.dwelling_type = DwellingType::Dormitory;
        let r = check(&i);
        assert!(r.covers_dormitory);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("§ 1940.45") && n.contains("dormitory"))
        );
    }

    #[test]
    fn california_restriction_imposed_violation() {
        let mut i = base(Regime::California);
        i.restriction_imposed = true;
        let r = check(&i);
        assert!(!r.restriction_permissible);
        assert!(r.fha_section_3604_religious_discrimination_risk);
        assert!(r.violations.iter().any(|v| v.contains("California")));
    }

    // ── Texas § 202.018 ─────────────────────────────────────────

    #[test]
    fn texas_protects_display_motivated_by_religious_belief() {
        let r = check(&base(Regime::Texas));
        assert!(r.statutory_protection);
        assert!(!r.covers_landlord_tenant);
        assert!(!r.covers_dormitory);
        assert!(r.citation.contains("§ 202.018"));
    }

    #[test]
    fn texas_restriction_violation() {
        let mut i = base(Regime::Texas);
        i.restriction_imposed = true;
        let r = check(&i);
        assert!(!r.restriction_permissible);
    }

    // ── Florida § 720.3045 ──────────────────────────────────────

    #[test]
    fn florida_explicit_tenant_extension() {
        let r = check(&base(Regime::Florida));
        assert!(r.statutory_protection);
        assert!(r.covers_landlord_tenant);
        assert!(r.citation.contains("§ 720.3045"));
    }

    // ── Illinois 765 ILCS 605/18.4 ──────────────────────────────

    #[test]
    fn illinois_mezuzah_law_condominium_focused() {
        let r = check(&base(Regime::Illinois));
        assert!(r.statutory_protection);
        assert!(!r.covers_landlord_tenant);
        assert!(r.citation.contains("765 ILCS 605/18.4"));
        assert!(r.citation.contains("Mezuzah Law"));
    }

    // ── Connecticut § 47-230a ───────────────────────────────────

    #[test]
    fn connecticut_cioa_protection() {
        let r = check(&base(Regime::Connecticut));
        assert!(r.statutory_protection);
        assert!(r.citation.contains("§ 47-230a"));
        assert!(r.citation.contains("Common Interest Ownership Act"));
    }

    // ── Rhode Island § 34-36.1-3.18 ─────────────────────────────

    #[test]
    fn rhode_island_parallel_protection() {
        let r = check(&base(Regime::RhodeIsland));
        assert!(r.statutory_protection);
        assert!(r.citation.contains("R.I. Gen. Laws § 34-36.1-3.18"));
    }

    // ── New York — no enacted state statute ─────────────────────

    #[test]
    fn new_york_no_state_statute_fha_only() {
        let r = check(&base(Regime::NewYork));
        assert!(!r.statutory_protection);
        assert!(r.citation.contains("S4466 proposed"));
        assert!(r.citation.contains("§ 3604(b)"));
        assert!(r.citation.contains("Bloch v. Frischholz"));
    }

    #[test]
    fn new_york_restriction_still_triggers_fha_risk() {
        let mut i = base(Regime::NewYork);
        i.restriction_imposed = true;
        let r = check(&i);
        // No state statute → restriction not state-statutorily
        // impermissible, but FHA risk attaches.
        assert!(r.fha_section_3604_religious_discrimination_risk);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("Bloch v. Frischholz") && n.contains("§ 3604(b)"))
        );
    }

    // ── Default — FHA § 3604 fallback ───────────────────────────

    #[test]
    fn default_federal_fha_floor_only() {
        let r = check(&base(Regime::Default));
        assert!(!r.statutory_protection);
        assert!(r.citation.contains("42 U.S.C. § 3604(b)"));
        assert!(r.citation.contains("Bloch v. Frischholz"));
    }

    // ── Threshold gates: sincere religious belief + location ────

    #[test]
    fn no_religious_motivation_no_protection() {
        let mut i = base(Regime::California);
        i.motivated_by_sincere_religious_belief = false;
        i.restriction_imposed = true;
        let r = check(&i);
        // Without religious motivation, statute doesn't engage.
        assert!(r.restriction_permissible);
        assert!(!r.fha_section_3604_religious_discrimination_risk);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("sincere religious belief"))
        );
    }

    #[test]
    fn interior_only_display_outside_statutory_scope() {
        let mut i = base(Regime::California);
        i.on_entry_door_or_doorframe = false;
        i.restriction_imposed = true;
        let r = check(&i);
        // Statute protects only entry door / doorframe displays.
        assert!(r.restriction_permissible);
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("entry door or entry doorframe"))
        );
    }

    // ── Temporary removal carve-out ─────────────────────────────

    #[test]
    fn temporary_removal_for_repair_universally_permitted() {
        for &regime in &[
            Regime::California,
            Regime::Texas,
            Regime::Florida,
            Regime::Illinois,
            Regime::Connecticut,
            Regime::RhodeIsland,
            Regime::NewYork,
            Regime::Default,
        ] {
            let mut i = base(regime);
            i.restriction_imposed = true;
            i.temporary_removal_for_repair = true;
            let r = check(&i);
            assert!(
                r.restriction_permissible,
                "{:?}: temporary removal for repair must be permissible",
                regime,
            );
            assert!(
                !r.fha_section_3604_religious_discrimination_risk,
                "{:?}: temporary removal for repair must not trigger FHA risk",
                regime,
            );
        }
    }

    // ── Regression-critical multi-regime invariants ─────────────

    #[test]
    fn only_california_covers_dormitory_explicitly_invariant() {
        let ca = check(&base(Regime::California));
        assert!(ca.covers_dormitory);
        for &regime in &[
            Regime::Texas,
            Regime::Florida,
            Regime::Illinois,
            Regime::Connecticut,
            Regime::RhodeIsland,
            Regime::NewYork,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(
                !r.covers_dormitory,
                "{:?}: must NOT explicitly cover dormitory",
                regime,
            );
        }
    }

    #[test]
    fn only_ca_and_fl_explicitly_cover_landlord_tenant_invariant() {
        for &regime in &[Regime::California, Regime::Florida] {
            assert!(
                check(&base(regime)).covers_landlord_tenant,
                "{:?}: must cover landlord-tenant",
                regime,
            );
        }
        for &regime in &[
            Regime::Texas,
            Regime::Illinois,
            Regime::Connecticut,
            Regime::RhodeIsland,
            Regime::NewYork,
            Regime::Default,
        ] {
            assert!(
                !check(&base(regime)).covers_landlord_tenant,
                "{:?}: must NOT explicitly cover landlord-tenant",
                regime,
            );
        }
    }

    #[test]
    fn only_six_states_have_enacted_statute_invariant() {
        for &regime in &[
            Regime::California,
            Regime::Texas,
            Regime::Florida,
            Regime::Illinois,
            Regime::Connecticut,
            Regime::RhodeIsland,
        ] {
            assert!(
                check(&base(regime)).statutory_protection,
                "{:?}: must have enacted statute",
                regime,
            );
        }
        for &regime in &[Regime::NewYork, Regime::Default] {
            assert!(
                !check(&base(regime)).statutory_protection,
                "{:?}: must NOT have enacted statute",
                regime,
            );
        }
    }

    #[test]
    fn fha_risk_attaches_universally_when_restriction_meets_threshold_invariant() {
        // Restriction imposed + sincere belief + door location +
        // no temporary removal → FHA risk attaches regardless of
        // regime.
        for &regime in &[
            Regime::California,
            Regime::Texas,
            Regime::Florida,
            Regime::Illinois,
            Regime::Connecticut,
            Regime::RhodeIsland,
            Regime::NewYork,
            Regime::Default,
        ] {
            let mut i = base(regime);
            i.restriction_imposed = true;
            assert!(
                check(&i).fha_section_3604_religious_discrimination_risk,
                "{:?}: FHA risk must attach",
                regime,
            );
        }
    }

    #[test]
    fn citation_pins_authority_per_regime() {
        assert!(check(&base(Regime::California)).citation.contains("§ 1940.45"));
        assert!(check(&base(Regime::Texas)).citation.contains("§ 202.018"));
        assert!(check(&base(Regime::Florida)).citation.contains("§ 720.3045"));
        assert!(
            check(&base(Regime::Illinois))
                .citation
                .contains("765 ILCS 605/18.4")
        );
        assert!(check(&base(Regime::Connecticut)).citation.contains("§ 47-230a"));
        assert!(
            check(&base(Regime::RhodeIsland))
                .citation
                .contains("R.I. Gen. Laws § 34-36.1-3.18")
        );
        assert!(check(&base(Regime::NewYork)).citation.contains("S4466"));
        assert!(check(&base(Regime::Default)).citation.contains("§ 3604(b)"));
    }

    #[test]
    fn bloch_v_frischholz_cited_in_default_and_ny_paths_invariant() {
        for &regime in &[Regime::Default, Regime::NewYork] {
            assert!(
                check(&base(regime)).citation.contains("Bloch v. Frischholz"),
                "{:?}: must cite Bloch v. Frischholz",
                regime,
            );
        }
    }

    #[test]
    fn sibling_module_note_present_across_all_regimes() {
        for &regime in &[
            Regime::California,
            Regime::Texas,
            Regime::Florida,
            Regime::Illinois,
            Regime::Connecticut,
            Regime::RhodeIsland,
            Regime::NewYork,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(
                r.notes.iter().any(|n| n.contains("service_animal")
                    && n.contains("reasonable_accommodation_modification")),
                "{:?}: sibling-module note must be present",
                regime,
            );
        }
    }
}
