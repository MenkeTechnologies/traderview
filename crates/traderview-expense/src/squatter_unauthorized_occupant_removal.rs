//! Squatter / unauthorized occupant removal procedures — state-by-
//! state pathway for evicting a person occupying property without
//! any lease right. Critical for absentee trader-landlords whose
//! property is occupied during periods of non-occupancy or after
//! a tenancy ends with a holdover that lacks any colorable claim.
//!
//! Distinct from `adverse_possession_claim` (statutory title
//! acquisition via 5-25 year continuous occupation),
//! `eviction_notices` (formal eviction process for actual
//! tenants), `holdover_tenant_damages` (damages against tenants
//! who hold over past lease term), and `landlord_harassment`
//! (anti-retaliation framework). This module addresses ONLY the
//! REMOVAL PROCEDURE PATHWAY for a person with NO LEASE RIGHT.
//!
//! Five regimes:
//!
//! **Florida — Fla. Stat. § 82.036 (HB 621, signed March 27, 2024,
//! effective July 1, 2024)**. EXPEDITED 24-HOUR SHERIFF REMOVAL.
//! Property owner submits verified affidavit and valid ID to the
//! sheriff; sheriff posts a 24-hour notice to vacate; if squatter
//! refuses, law enforcement may physically remove them WITHOUT a
//! court order. Eligibility requires: (a) individual unlawfully
//! entered and remains; (b) owner directed individual to leave;
//! (c) individual is neither current/former tenant in legal
//! dispute nor family member; (d) no pending litigation regarding
//! occupancy. § 82.036 ALSO creates criminal felony penalties for
//! false legal authority claims, fraudulent documents (forged
//! leases), and property damage during unauthorized occupancy.
//! Strongest pro-owner squatter law in the US.
//!
//! **New York — N.Y. RPAPL § 711(1) + § 713 (amended April 22,
//! 2024 by FY2025 budget)**. POST-AMENDMENT — RPAPL § 711(1) now
//! defines "tenant" to EXCLUDE squatters; a squatter is "a person
//! who enters or intrudes upon real property without the
//! permission of the person entitled to possession, and continues
//! to occupy the property without title, right or permission."
//! § 713 SPECIAL HOLDOVER PROCEEDING still available with 10-day
//! notice to quit for licensees and squatters. Pre-amendment 30-
//! day occupancy threshold ABOLISHED. Police may arrest squatters
//! as trespassers without eviction order; landlord retains § 713
//! summary holdover option for additional procedural certainty.
//!
//! **California — Cal. Civ. Proc. § 1161**. UNLAWFUL DETAINER
//! action required. CA has not enacted a specific expedited
//! squatter removal law; squatter must be served with a 3-day
//! notice to quit (no cure period for unauthorized occupancy)
//! followed by a UD complaint, summons, judgment, and writ of
//! possession executed by the sheriff. Typical timeline 3-6 weeks
//! when uncontested.
//!
//! **Texas — Tex. Prop. Code § 24.005 + § 24.002**. FORCIBLE
//! ENTRY AND DETAINER (FED) with 3-DAY NOTICE TO VACATE for
//! squatters / non-tenants. Justice of the Peace court handles
//! FED actions; timeline typically faster than CA UD (2-4 weeks
//! uncontested). § 24.005(c) explicitly permits 3-day notice for
//! tenant-at-sufferance / squatter.
//!
//! **Default — common-law ejectment**. Most states require formal
//! ejectment action (common-law remedy) or state-specific
//! statutory summary procedure. Self-help removal is uniformly
//! prohibited and exposes the landlord to lockout damages under
//! `lockout_penalties` framework.
//!
//! Citations: Fla. Stat. § 82.036 (FL HB 621 squatter removal,
//! eff. July 1, 2024); Fla. Stat. § 82.036(8) (criminal penalty
//! for fraudulent documents); N.Y. RPAPL § 711(1) (post-April 22,
//! 2024 amendment defining squatter); N.Y. RPAPL § 713 (special
//! 10-day-notice holdover); Cal. Civ. Proc. § 1161 (unlawful
//! detainer general); Tex. Prop. Code § 24.005 (forcible entry
//! and detainer notice); Tex. Prop. Code § 24.002 (FED action);
//! common-law ejectment.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Florida,
    NewYork,
    California,
    Texas,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RemovalPathway {
    /// Fla. Stat. § 82.036 — sheriff posts 24-hour notice to
    /// vacate without court order on verified owner affidavit.
    Sheriff24HourRemoval,
    /// N.Y. RPAPL § 713 — 10-day notice to quit + summary
    /// holdover proceeding; or police trespass arrest post-2024.
    NySection713SummaryHoldover,
    /// Cal. Civ. Proc. § 1161 — 3-day notice to quit + unlawful
    /// detainer complaint + judgment + writ of possession.
    CaliforniaUnlawfulDetainer,
    /// Tex. Prop. Code § 24.005 + § 24.002 — 3-day notice to
    /// vacate + JP court forcible entry and detainer action.
    TexasForcibleEntryDetainer,
    /// Common-law ejectment action in court of general
    /// jurisdiction.
    CommonLawEjectment,
    /// No removal pathway available (eligibility prerequisites
    /// not met).
    NoPathwayAvailable,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SquatterRemovalInput {
    pub regime: Regime,
    /// Whether the individual entered the property unlawfully
    /// (never had permission from the owner / authorized
    /// possessor).
    pub unauthorized_entry: bool,
    /// Whether the owner has directed the individual to leave
    /// the property.
    pub owner_directed_to_leave: bool,
    /// Whether the individual is a current/former tenant in a
    /// legal dispute OR a family member of the property owner
    /// (excludes them from FL HB 621 expedited removal).
    pub current_former_tenant_or_family_member: bool,
    /// Whether there is pending litigation regarding occupancy
    /// (excludes FL HB 621 expedited removal).
    pub pending_occupancy_litigation: bool,
    /// FL-only: whether the owner submitted a verified affidavit
    /// and valid ID to the sheriff under § 82.036.
    pub fl_verified_affidavit_submitted: bool,
    /// FL-only: whether the squatter has made a false legal
    /// authority claim, presented fraudulent documents (forged
    /// lease), or damaged the property during unauthorized
    /// occupancy (triggers § 82.036(8) felony charges).
    pub fl_fraudulent_documents_or_damage: bool,
    /// NY-only: whether the relevant events occurred after the
    /// April 22, 2024 RPAPL § 711(1) amendment.
    pub ny_post_april_22_2024: bool,
    /// NY-only: whether the landlord served the 10-day notice to
    /// quit under RPAPL § 713.
    pub ny_10_day_notice_served: bool,
    /// CA-only: whether the landlord served the 3-day notice to
    /// quit under Cal. Civ. Proc. § 1161.
    pub ca_3_day_notice_served: bool,
    /// TX-only: whether the landlord served the 3-day notice to
    /// vacate under Tex. Prop. Code § 24.005.
    pub tx_3_day_notice_served: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SquatterRemovalResult {
    pub removal_pathway: RemovalPathway,
    pub pathway_available: bool,
    pub criminal_exposure_for_squatter: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &SquatterRemovalInput) -> SquatterRemovalResult {
    let mut notes: Vec<String> = Vec::new();

    if !input.unauthorized_entry {
        notes.push(
            "individual did NOT enter unlawfully — squatter removal pathway does not engage; landlord-tenant eviction procedures apply if a tenancy exists"
                .to_string(),
        );
        return SquatterRemovalResult {
            removal_pathway: RemovalPathway::NoPathwayAvailable,
            pathway_available: false,
            criminal_exposure_for_squatter: false,
            citation: citation_for(input.regime),
            notes,
        };
    }

    match input.regime {
        Regime::Florida => check_florida(input, &mut notes),
        Regime::NewYork => check_new_york(input, &mut notes),
        Regime::California => check_california(input, &mut notes),
        Regime::Texas => check_texas(input, &mut notes),
        Regime::Default => check_default(input, &mut notes),
    }
}

fn check_florida(input: &SquatterRemovalInput, notes: &mut Vec<String>) -> SquatterRemovalResult {
    let eligible = input.owner_directed_to_leave
        && !input.current_former_tenant_or_family_member
        && !input.pending_occupancy_litigation;

    if !eligible {
        if !input.owner_directed_to_leave {
            notes.push(
                "Fla. Stat. § 82.036 eligibility — owner has NOT directed individual to leave; expedited 24-hour sheriff removal unavailable"
                    .to_string(),
            );
        }
        if input.current_former_tenant_or_family_member {
            notes.push(
                "Fla. Stat. § 82.036 eligibility — individual is current/former tenant in dispute or family member of owner; expedited removal unavailable; standard FL eviction procedures apply"
                    .to_string(),
            );
        }
        if input.pending_occupancy_litigation {
            notes.push(
                "Fla. Stat. § 82.036 eligibility — pending occupancy litigation; expedited removal unavailable while court action pending"
                    .to_string(),
            );
        }
        return SquatterRemovalResult {
            removal_pathway: RemovalPathway::NoPathwayAvailable,
            pathway_available: false,
            criminal_exposure_for_squatter: input.fl_fraudulent_documents_or_damage,
            citation: citation_for(Regime::Florida),
            notes: notes.clone(),
        };
    }

    if !input.fl_verified_affidavit_submitted {
        notes.push(
            "Fla. Stat. § 82.036 — owner must submit verified affidavit and valid ID to sheriff to trigger 24-hour notice to vacate"
                .to_string(),
        );
        return SquatterRemovalResult {
            removal_pathway: RemovalPathway::NoPathwayAvailable,
            pathway_available: false,
            criminal_exposure_for_squatter: input.fl_fraudulent_documents_or_damage,
            citation: citation_for(Regime::Florida),
            notes: notes.clone(),
        };
    }

    notes.push(
        "Fla. Stat. § 82.036 (HB 621, eff. July 1, 2024) — sheriff posts 24-hour notice to vacate on verified owner affidavit; squatter physically removed without court order if non-compliant"
            .to_string(),
    );

    if input.fl_fraudulent_documents_or_damage {
        notes.push(
            "Fla. Stat. § 82.036(8) — felony charges for false legal authority claims, fraudulent documents (forged lease), or property damage during unauthorized occupancy"
                .to_string(),
        );
    }

    SquatterRemovalResult {
        removal_pathway: RemovalPathway::Sheriff24HourRemoval,
        pathway_available: true,
        criminal_exposure_for_squatter: input.fl_fraudulent_documents_or_damage,
        citation: citation_for(Regime::Florida),
        notes: notes.clone(),
    }
}

fn check_new_york(input: &SquatterRemovalInput, notes: &mut Vec<String>) -> SquatterRemovalResult {
    if !input.ny_post_april_22_2024 {
        notes.push(
            "pre-April 22, 2024 events governed by prior NY law where squatters acquired tenant-like protections after 30 days of occupancy; consult NY counsel"
                .to_string(),
        );
    } else {
        notes.push(
            "N.Y. RPAPL § 711(1) (post-April 22, 2024 amendment) — \"tenant\" defined to EXCLUDE squatters; police may arrest squatters as trespassers without eviction order; 30-day-occupancy threshold abolished"
                .to_string(),
        );
    }

    if !input.ny_10_day_notice_served {
        notes.push(
            "N.Y. RPAPL § 713 — 10-day notice to quit prerequisite to summary holdover proceeding"
                .to_string(),
        );
        return SquatterRemovalResult {
            removal_pathway: RemovalPathway::NoPathwayAvailable,
            pathway_available: false,
            criminal_exposure_for_squatter: false,
            citation: citation_for(Regime::NewYork),
            notes: notes.clone(),
        };
    }

    notes.push(
        "N.Y. RPAPL § 713 — summary holdover proceeding available with 10-day notice to quit; faster than common-law ejectment"
            .to_string(),
    );

    SquatterRemovalResult {
        removal_pathway: RemovalPathway::NySection713SummaryHoldover,
        pathway_available: true,
        criminal_exposure_for_squatter: false,
        citation: citation_for(Regime::NewYork),
        notes: notes.clone(),
    }
}

fn check_california(
    input: &SquatterRemovalInput,
    notes: &mut Vec<String>,
) -> SquatterRemovalResult {
    if !input.ca_3_day_notice_served {
        notes.push(
            "Cal. Civ. Proc. § 1161 — 3-day notice to quit prerequisite to unlawful detainer (UD) complaint"
                .to_string(),
        );
        return SquatterRemovalResult {
            removal_pathway: RemovalPathway::NoPathwayAvailable,
            pathway_available: false,
            criminal_exposure_for_squatter: false,
            citation: citation_for(Regime::California),
            notes: notes.clone(),
        };
    }

    notes.push(
        "Cal. Civ. Proc. § 1161 — unlawful detainer process: 3-day notice + UD complaint + summons + judgment + sheriff writ of possession; typical 3-6 week timeline uncontested"
            .to_string(),
    );
    notes.push(
        "CA has NOT enacted expedited squatter removal — must follow full UD process; self-help removal exposes landlord to lockout damages"
            .to_string(),
    );

    SquatterRemovalResult {
        removal_pathway: RemovalPathway::CaliforniaUnlawfulDetainer,
        pathway_available: true,
        criminal_exposure_for_squatter: false,
        citation: citation_for(Regime::California),
        notes: notes.clone(),
    }
}

fn check_texas(input: &SquatterRemovalInput, notes: &mut Vec<String>) -> SquatterRemovalResult {
    if !input.tx_3_day_notice_served {
        notes.push(
            "Tex. Prop. Code § 24.005 — 3-day notice to vacate prerequisite to forcible entry and detainer (FED) action"
                .to_string(),
        );
        return SquatterRemovalResult {
            removal_pathway: RemovalPathway::NoPathwayAvailable,
            pathway_available: false,
            criminal_exposure_for_squatter: false,
            citation: citation_for(Regime::Texas),
            notes: notes.clone(),
        };
    }

    notes.push(
        "Tex. Prop. Code §§ 24.005, 24.002 — FED action in Justice of the Peace court; 3-day notice + complaint + JP judgment; typical 2-4 week timeline uncontested"
            .to_string(),
    );
    notes.push(
        "§ 24.005(c) — 3-day notice expressly permitted for tenant-at-sufferance / squatter"
            .to_string(),
    );

    SquatterRemovalResult {
        removal_pathway: RemovalPathway::TexasForcibleEntryDetainer,
        pathway_available: true,
        criminal_exposure_for_squatter: false,
        citation: citation_for(Regime::Texas),
        notes: notes.clone(),
    }
}

fn check_default(_input: &SquatterRemovalInput, notes: &mut Vec<String>) -> SquatterRemovalResult {
    notes.push(
        "default rule — common-law ejectment action OR state-specific summary procedure; self-help removal universally prohibited (exposes landlord to lockout damages)"
            .to_string(),
    );
    SquatterRemovalResult {
        removal_pathway: RemovalPathway::CommonLawEjectment,
        pathway_available: true,
        criminal_exposure_for_squatter: false,
        citation: citation_for(Regime::Default),
        notes: notes.clone(),
    }
}

fn citation_for(regime: Regime) -> &'static str {
    match regime {
        Regime::Florida => "Fla. Stat. § 82.036 (HB 621, eff. July 1, 2024); § 82.036(8)",
        Regime::NewYork => "N.Y. RPAPL §§ 711(1), 713 (April 22, 2024 amendment)",
        Regime::California => "Cal. Civ. Proc. § 1161",
        Regime::Texas => "Tex. Prop. Code §§ 24.005, 24.005(c), 24.002",
        Regime::Default => "common-law ejectment + state-specific summary procedure",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fl_base() -> SquatterRemovalInput {
        SquatterRemovalInput {
            regime: Regime::Florida,
            unauthorized_entry: true,
            owner_directed_to_leave: true,
            current_former_tenant_or_family_member: false,
            pending_occupancy_litigation: false,
            fl_verified_affidavit_submitted: true,
            fl_fraudulent_documents_or_damage: false,
            ny_post_april_22_2024: false,
            ny_10_day_notice_served: false,
            ca_3_day_notice_served: false,
            tx_3_day_notice_served: false,
        }
    }

    fn ny_base() -> SquatterRemovalInput {
        SquatterRemovalInput {
            regime: Regime::NewYork,
            unauthorized_entry: true,
            owner_directed_to_leave: true,
            current_former_tenant_or_family_member: false,
            pending_occupancy_litigation: false,
            fl_verified_affidavit_submitted: false,
            fl_fraudulent_documents_or_damage: false,
            ny_post_april_22_2024: true,
            ny_10_day_notice_served: true,
            ca_3_day_notice_served: false,
            tx_3_day_notice_served: false,
        }
    }

    fn ca_base() -> SquatterRemovalInput {
        SquatterRemovalInput {
            regime: Regime::California,
            unauthorized_entry: true,
            owner_directed_to_leave: true,
            current_former_tenant_or_family_member: false,
            pending_occupancy_litigation: false,
            fl_verified_affidavit_submitted: false,
            fl_fraudulent_documents_or_damage: false,
            ny_post_april_22_2024: false,
            ny_10_day_notice_served: false,
            ca_3_day_notice_served: true,
            tx_3_day_notice_served: false,
        }
    }

    fn tx_base() -> SquatterRemovalInput {
        SquatterRemovalInput {
            regime: Regime::Texas,
            unauthorized_entry: true,
            owner_directed_to_leave: true,
            current_former_tenant_or_family_member: false,
            pending_occupancy_litigation: false,
            fl_verified_affidavit_submitted: false,
            fl_fraudulent_documents_or_damage: false,
            ny_post_april_22_2024: false,
            ny_10_day_notice_served: false,
            ca_3_day_notice_served: false,
            tx_3_day_notice_served: true,
        }
    }

    fn default_base() -> SquatterRemovalInput {
        SquatterRemovalInput {
            regime: Regime::Default,
            unauthorized_entry: true,
            owner_directed_to_leave: true,
            current_former_tenant_or_family_member: false,
            pending_occupancy_litigation: false,
            fl_verified_affidavit_submitted: false,
            fl_fraudulent_documents_or_damage: false,
            ny_post_april_22_2024: false,
            ny_10_day_notice_served: false,
            ca_3_day_notice_served: false,
            tx_3_day_notice_served: false,
        }
    }

    #[test]
    fn fl_full_eligibility_yields_24_hour_sheriff_removal() {
        let r = check(&fl_base());
        assert_eq!(r.removal_pathway, RemovalPathway::Sheriff24HourRemoval);
        assert!(r.pathway_available);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("HB 621") && n.contains("24-hour")));
    }

    #[test]
    fn fl_current_former_tenant_disqualifies_expedited_removal() {
        let mut i = fl_base();
        i.current_former_tenant_or_family_member = true;
        let r = check(&i);
        assert_eq!(r.removal_pathway, RemovalPathway::NoPathwayAvailable);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("tenant in dispute or family member")));
    }

    #[test]
    fn fl_pending_litigation_disqualifies() {
        let mut i = fl_base();
        i.pending_occupancy_litigation = true;
        let r = check(&i);
        assert_eq!(r.removal_pathway, RemovalPathway::NoPathwayAvailable);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("pending occupancy litigation")));
    }

    #[test]
    fn fl_owner_must_direct_to_leave() {
        let mut i = fl_base();
        i.owner_directed_to_leave = false;
        let r = check(&i);
        assert_eq!(r.removal_pathway, RemovalPathway::NoPathwayAvailable);
        assert!(r.notes.iter().any(|n| n.contains("has NOT directed")));
    }

    #[test]
    fn fl_no_verified_affidavit_no_pathway() {
        let mut i = fl_base();
        i.fl_verified_affidavit_submitted = false;
        let r = check(&i);
        assert_eq!(r.removal_pathway, RemovalPathway::NoPathwayAvailable);
        assert!(r.notes.iter().any(|n| n.contains("verified affidavit")));
    }

    #[test]
    fn fl_fraudulent_documents_trigger_felony_charges() {
        let mut i = fl_base();
        i.fl_fraudulent_documents_or_damage = true;
        let r = check(&i);
        assert_eq!(r.removal_pathway, RemovalPathway::Sheriff24HourRemoval);
        assert!(r.criminal_exposure_for_squatter);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 82.036(8)") && n.contains("felony")));
    }

    #[test]
    fn ny_post_april_2024_with_notice_yields_section_713_holdover() {
        let r = check(&ny_base());
        assert_eq!(
            r.removal_pathway,
            RemovalPathway::NySection713SummaryHoldover
        );
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 711(1)") && n.contains("EXCLUDE squatters")));
    }

    #[test]
    fn ny_pre_april_2024_warning_note_present() {
        let mut i = ny_base();
        i.ny_post_april_22_2024 = false;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("pre-April 22, 2024") && n.contains("30 days")));
    }

    #[test]
    fn ny_no_10_day_notice_no_pathway() {
        let mut i = ny_base();
        i.ny_10_day_notice_served = false;
        let r = check(&i);
        assert_eq!(r.removal_pathway, RemovalPathway::NoPathwayAvailable);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 713") && n.contains("10-day notice to quit")));
    }

    #[test]
    fn ca_with_3_day_notice_yields_unlawful_detainer() {
        let r = check(&ca_base());
        assert_eq!(
            r.removal_pathway,
            RemovalPathway::CaliforniaUnlawfulDetainer
        );
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1161") && n.contains("3-6 week")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("has NOT enacted expedited squatter removal")));
    }

    #[test]
    fn ca_no_3_day_notice_no_pathway() {
        let mut i = ca_base();
        i.ca_3_day_notice_served = false;
        let r = check(&i);
        assert_eq!(r.removal_pathway, RemovalPathway::NoPathwayAvailable);
    }

    #[test]
    fn tx_with_3_day_notice_yields_fed_action() {
        let r = check(&tx_base());
        assert_eq!(
            r.removal_pathway,
            RemovalPathway::TexasForcibleEntryDetainer
        );
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§§ 24.005, 24.002") && n.contains("Justice of the Peace")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 24.005(c)") && n.contains("tenant-at-sufferance")));
    }

    #[test]
    fn tx_no_3_day_notice_no_pathway() {
        let mut i = tx_base();
        i.tx_3_day_notice_served = false;
        let r = check(&i);
        assert_eq!(r.removal_pathway, RemovalPathway::NoPathwayAvailable);
    }

    #[test]
    fn default_yields_common_law_ejectment() {
        let r = check(&default_base());
        assert_eq!(r.removal_pathway, RemovalPathway::CommonLawEjectment);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("self-help removal universally prohibited")));
    }

    #[test]
    fn no_unauthorized_entry_no_pathway_all_regimes() {
        for regime in [
            Regime::Florida,
            Regime::NewYork,
            Regime::California,
            Regime::Texas,
            Regime::Default,
        ] {
            let mut i = fl_base();
            i.regime = regime;
            i.unauthorized_entry = false;
            let r = check(&i);
            assert_eq!(r.removal_pathway, RemovalPathway::NoPathwayAvailable);
            assert!(r
                .notes
                .iter()
                .any(|n| n.contains("did NOT enter unlawfully")));
        }
    }

    #[test]
    fn fl_unique_24_hour_pathway_invariant() {
        let r_fl = check(&fl_base());
        assert_eq!(r_fl.removal_pathway, RemovalPathway::Sheriff24HourRemoval);

        for regime in [
            Regime::NewYork,
            Regime::California,
            Regime::Texas,
            Regime::Default,
        ] {
            let mut i = fl_base();
            i.regime = regime;
            i.ny_10_day_notice_served = true;
            i.ny_post_april_22_2024 = true;
            i.ca_3_day_notice_served = true;
            i.tx_3_day_notice_served = true;
            let r = check(&i);
            assert_ne!(
                r.removal_pathway,
                RemovalPathway::Sheriff24HourRemoval,
                "only FL has 24-hour sheriff removal"
            );
        }
    }

    #[test]
    fn citation_florida_pins_hb_621_and_subsections() {
        let r = check(&fl_base());
        assert!(r.citation.contains("§ 82.036"));
        assert!(r.citation.contains("HB 621"));
        assert!(r.citation.contains("July 1, 2024"));
        assert!(r.citation.contains("§ 82.036(8)"));
    }

    #[test]
    fn citation_newyork_pins_711_713_april_2024() {
        let r = check(&ny_base());
        assert!(r.citation.contains("§§ 711(1), 713"));
        assert!(r.citation.contains("April 22, 2024"));
    }

    #[test]
    fn citation_california_pins_1161() {
        let r = check(&ca_base());
        assert!(r.citation.contains("§ 1161"));
    }

    #[test]
    fn citation_texas_pins_24_005_and_24_002() {
        let r = check(&tx_base());
        assert!(r.citation.contains("§§ 24.005, 24.005(c), 24.002"));
    }

    #[test]
    fn citation_default_pins_common_law() {
        let r = check(&default_base());
        assert!(r.citation.contains("common-law ejectment"));
    }

    #[test]
    fn fl_criminal_exposure_persists_even_when_eligibility_fails() {
        let mut i = fl_base();
        i.current_former_tenant_or_family_member = true;
        i.fl_fraudulent_documents_or_damage = true;
        let r = check(&i);
        assert!(r.criminal_exposure_for_squatter);
    }

    #[test]
    fn ny_post_amendment_note_describes_police_arrest_authority() {
        let r = check(&ny_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("police may arrest squatters as trespassers")));
    }

    #[test]
    fn ca_self_help_removal_warning_in_notes() {
        let r = check(&ca_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("self-help removal exposes landlord")));
    }

    #[test]
    fn default_self_help_warning_in_notes() {
        let r = check(&default_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("self-help removal universally prohibited")));
    }

    #[test]
    fn fl_eligibility_fails_when_owner_did_not_direct_to_leave() {
        let mut i = fl_base();
        i.owner_directed_to_leave = false;
        let r = check(&i);
        assert!(!r.pathway_available);
    }

    #[test]
    fn fl_eligibility_combined_failures_yields_multiple_violations() {
        let mut i = fl_base();
        i.owner_directed_to_leave = false;
        i.current_former_tenant_or_family_member = true;
        i.pending_occupancy_litigation = true;
        let r = check(&i);
        let eligibility_notes = r
            .notes
            .iter()
            .filter(|n| n.contains("§ 82.036 eligibility"))
            .count();
        assert_eq!(eligibility_notes, 3);
    }

    #[test]
    fn ny_amendment_abolishes_30_day_threshold_note() {
        let r = check(&ny_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("30-day-occupancy threshold abolished")));
    }

    #[test]
    fn florida_uniquely_carries_criminal_exposure_invariant() {
        let mut i = fl_base();
        i.fl_fraudulent_documents_or_damage = true;
        let r_fl = check(&i);
        assert!(r_fl.criminal_exposure_for_squatter);

        for regime in [
            Regime::NewYork,
            Regime::California,
            Regime::Texas,
            Regime::Default,
        ] {
            let mut i2 = fl_base();
            i2.regime = regime;
            i2.fl_fraudulent_documents_or_damage = true;
            i2.ny_post_april_22_2024 = true;
            i2.ny_10_day_notice_served = true;
            i2.ca_3_day_notice_served = true;
            i2.tx_3_day_notice_served = true;
            let r = check(&i2);
            assert!(
                !r.criminal_exposure_for_squatter,
                "regime {:?} does not surface criminal exposure flag",
                regime
            );
        }
    }
}
