//! State succession rights — surviving family member's right to
//! continue a lease after the named tenant dies or permanently
//! vacates.
//!
//! Distinct from `tenant_death_termination` (which covers the
//! estate's right to TERMINATE the lease). This module captures
//! the COMPETING right of a family member living with the
//! deceased tenant to ASSUME or RENEW the lease in their own
//! name. Only one state has a comprehensive succession framework
//! (NY for rent-regulated units); NJ provides limited
//! anti-eviction protections; the rest leave succession to the
//! lease, which typically extinguishes on the named tenant's
//! death.
//!
//! Three regimes:
//!
//! `NewYorkRentRegulatedSuccession`: NY only. NYC Rent
//! Stabilization Code (RSC) § 2523.5(b)(1) (DHCR, 1987) +
//! analogous rent-control provisions. A "family member" who
//! resided with the named tenant as a primary resident for at
//! least **2 years** before death / permanent vacancy is entitled
//! to a rent-stabilized renewal lease in their name or eviction
//! protection in rent-controlled units. **1 year** suffices for
//! senior citizens (age 62+) or disabled persons.
//!
//! Family-member definition is BROAD: traditional family
//! (spouse, children, parents, siblings, grandparents, in-laws,
//! step-relations) plus non-traditional family (aunts, uncles,
//! nieces, nephews, cousins, unmarried couples, LGBQT couples)
//! demonstrating an "emotionally and financially committed and
//! interdependent relationship."
//!
//! `NewJerseyAntiEvictionImmediateFamily`: NJ only. N.J.S.A.
//! 2A:18-61.1 et seq. (Anti-Eviction Act) bars no-fault eviction
//! of tenants on enumerated grounds; immediate family members
//! who held the unit jointly or under a §61.3 family-trust
//! arrangement may continue occupancy. NJ does not provide a
//! comprehensive succession framework like NY's RSC, but the
//! Anti-Eviction Act's just-cause requirement effectively
//! protects co-resident family from eviction following the
//! named tenant's death.
//!
//! `DefaultLeaseGovernsNoSuccession`: 48 other states + DC. Lease
//! is a contract that extinguishes on the named tenant's death.
//! Family members residing with the deceased tenant have no
//! statutory right to assume the lease; they are subject to the
//! landlord's discretion or the estate's choice to continue
//! paying rent through the lease term.
//!
//! Sources:
//! [NYC RSC § 2523.5(b)(1) — Botway AI Succession Compliance Playbook](https://www.botway.ai/docs/playbooks/landlord/succession-rights-in-rent-stabilized-apartments-rsc-2523-5-compliance),
//! [N.Y. State HCR — Succession Rights Overview](https://hcr.ny.gov/succession),
//! [NYC Rent Guidelines Board — Succession Rights FAQs](https://rentguidelinesboard.cityofnewyork.us/resources/faqs/succession-rights/),
//! [Met Council on Housing — NY Succession Rights](https://www.metcouncilonhousing.org/help-answers/succession-rights-in-rent-stabilized-and-rent-controlled-apartments/),
//! [N.J.S.A. 2A:18-61.1 — Justia](https://law.justia.com/codes/new-jersey/title-2a/section-2a-18-61-1/).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseSuccessionRegime {
    NewYorkRentRegulatedSuccession,
    NewJerseyAntiEvictionImmediateFamily,
    DefaultLeaseGovernsNoSuccession,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FamilyMemberCategory {
    /// Spouse, child, parent, sibling, grandparent, in-law,
    /// step-relation (traditional family).
    TraditionalFamily,
    /// Aunt, uncle, niece, nephew, cousin, unmarried couple,
    /// LGBQT couple with emotionally/financially committed
    /// interdependent relationship (non-traditional family under
    /// NY's broad definition).
    NonTraditionalFamily,
    /// Not a family member.
    NotFamily,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: LeaseSuccessionRegime,
    /// Years of co-residency required for succession (NY = 2 for
    /// adults; 1 for senior/disabled).
    pub adult_co_residency_years_required: u32,
    /// Reduced co-residency requirement for senior (age 62+) or
    /// disabled successors.
    pub senior_or_disabled_co_residency_years_required: u32,
    /// True if statute requires the unit to be rent-regulated for
    /// succession to apply (NY).
    pub requires_rent_regulated_unit: bool,
    /// True if the regime extends to non-traditional family
    /// members (NY broad definition).
    pub non_traditional_family_eligible: bool,
    pub citation: &'static str,
}

const fn rule(
    regime: LeaseSuccessionRegime,
    adult_co_residency_years_required: u32,
    senior_or_disabled_co_residency_years_required: u32,
    requires_rent_regulated_unit: bool,
    non_traditional_family_eligible: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        adult_co_residency_years_required,
        senior_or_disabled_co_residency_years_required,
        requires_rent_regulated_unit,
        non_traditional_family_eligible,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use LeaseSuccessionRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "NY",
        rule(
            NewYorkRentRegulatedSuccession,
            2,
            1,
            true,
            true,
            "NYC Rent Stabilization Code § 2523.5(b)(1) (DHCR, 1987) + analogous rent-control provisions — family member who resided with named tenant as primary resident for at least 2 years before death/permanent vacancy entitled to rent-stabilized renewal lease in own name or eviction protection in rent-controlled unit; 1 year suffices for senior citizens (age 62+) or disabled persons; broad family-member definition includes traditional family + non-traditional family with emotionally and financially committed interdependent relationship",
        ),
    );

    m.insert(
        "NJ",
        rule(
            NewJerseyAntiEvictionImmediateFamily,
            0,
            0,
            false,
            false,
            "N.J.S.A. 2A:18-61.1 et seq. (Anti-Eviction Act) — bars no-fault eviction of tenants on enumerated grounds; immediate family co-resident members protected from eviction following named tenant's death; § 2A:18-61.3 governs family-trust arrangements; no comprehensive succession framework like NY's RSC § 2523.5",
        ),
    );

    // DefaultLeaseGovernsNoSuccession — 48 other states + DC.
    let default_states = [
        "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DC",
        "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA",
        "KS", "KY", "LA", "ME", "MD", "MA", "MI", "MN",
        "MS", "MO", "MT", "NE", "NV", "NH", "NM", "NC",
        "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD",
        "TN", "TX", "UT", "VT", "VA", "WA", "WV", "WI",
        "WY",
    ];
    for code in default_states {
        m.insert(
            code,
            rule(
                DefaultLeaseGovernsNoSuccession,
                0,
                0,
                false,
                false,
                "Lease is a contract that extinguishes on named tenant's death; family members residing with deceased tenant have no statutory succession right; subject to landlord discretion or estate's choice to continue paying rent through lease term",
            ),
        );
    }

    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseSuccessionInput {
    pub state_code: String,
    /// True if the unit is rent-regulated (rent-stabilized or
    /// rent-controlled). Required for NY succession.
    pub unit_is_rent_regulated: bool,
    pub family_member_category: FamilyMemberCategory,
    /// Years the successor resided with the named tenant as
    /// primary resident before death / permanent vacancy.
    pub co_residency_years_with_named_tenant: u32,
    /// True if the successor is a senior citizen (age 62+) or
    /// disabled person.
    pub successor_is_senior_or_disabled: bool,
    /// True if the successor has been the primary resident of the
    /// unit (not just a co-occupant).
    pub successor_was_primary_resident: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseSuccessionResult {
    pub regime: LeaseSuccessionRegime,
    pub successor_eligible_for_succession: bool,
    pub effective_required_residency_years: u32,
    pub residency_satisfied: bool,
    pub family_eligibility_satisfied: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &LeaseSuccessionInput) -> LeaseSuccessionResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: LeaseSuccessionRegime::DefaultLeaseGovernsNoSuccession,
        adult_co_residency_years_required: 0,
        senior_or_disabled_co_residency_years_required: 0,
        requires_rent_regulated_unit: false,
        non_traditional_family_eligible: false,
        citation: "Unknown state code; lease governs default assumed",
    });

    // Effective residency requirement.
    let required_years = if input.successor_is_senior_or_disabled {
        rule.senior_or_disabled_co_residency_years_required
    } else {
        rule.adult_co_residency_years_required
    };

    // Family eligibility check.
    let family_eligible = match input.family_member_category {
        FamilyMemberCategory::TraditionalFamily => true,
        FamilyMemberCategory::NonTraditionalFamily => rule.non_traditional_family_eligible,
        FamilyMemberCategory::NotFamily => false,
    };

    // Residency check.
    let residency_ok = input.co_residency_years_with_named_tenant >= required_years;

    // Rent-regulated gate (NY).
    let regulated_gate_ok = !rule.requires_rent_regulated_unit || input.unit_is_rent_regulated;

    let eligible = match rule.regime {
        LeaseSuccessionRegime::NewYorkRentRegulatedSuccession => {
            regulated_gate_ok
                && family_eligible
                && residency_ok
                && input.successor_was_primary_resident
        }
        LeaseSuccessionRegime::NewJerseyAntiEvictionImmediateFamily => {
            family_eligible && matches!(input.family_member_category, FamilyMemberCategory::TraditionalFamily)
        }
        LeaseSuccessionRegime::DefaultLeaseGovernsNoSuccession => false,
    };

    let regime_label = match rule.regime {
        LeaseSuccessionRegime::NewYorkRentRegulatedSuccession => {
            "New York RSC § 2523.5 rent-regulated succession"
        }
        LeaseSuccessionRegime::NewJerseyAntiEvictionImmediateFamily => {
            "New Jersey Anti-Eviction Act immediate-family protection"
        }
        LeaseSuccessionRegime::DefaultLeaseGovernsNoSuccession => {
            "default lease-governs no succession"
        }
    };

    let note = if eligible {
        format!(
            "State applies {} regime; successor IS eligible for succession on these facts.",
            regime_label,
        )
    } else {
        let mut reasons = vec![];
        if matches!(rule.regime, LeaseSuccessionRegime::DefaultLeaseGovernsNoSuccession) {
            reasons.push("no statutory succession right in this state".to_string());
        } else {
            if !regulated_gate_ok {
                reasons.push("unit not rent-regulated".to_string());
            }
            if !family_eligible {
                reasons.push("successor not within statutory family definition".to_string());
            }
            if !residency_ok {
                reasons.push(format!(
                    "{} years co-residency required, {} provided",
                    required_years, input.co_residency_years_with_named_tenant
                ));
            }
            if matches!(rule.regime, LeaseSuccessionRegime::NewYorkRentRegulatedSuccession)
                && !input.successor_was_primary_resident
            {
                reasons.push("successor was not primary resident".to_string());
            }
        }
        format!(
            "State applies {} regime; successor NOT eligible: {}.",
            regime_label,
            reasons.join("; "),
        )
    };

    LeaseSuccessionResult {
        regime: rule.regime,
        successor_eligible_for_succession: eligible,
        effective_required_residency_years: required_years,
        residency_satisfied: residency_ok,
        family_eligibility_satisfied: family_eligible,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline(state: &str) -> LeaseSuccessionInput {
        LeaseSuccessionInput {
            state_code: state.to_string(),
            unit_is_rent_regulated: true,
            family_member_category: FamilyMemberCategory::TraditionalFamily,
            co_residency_years_with_named_tenant: 2,
            successor_is_senior_or_disabled: false,
            successor_was_primary_resident: true,
        }
    }

    // ── Regime classification ───────────────────────────────────────

    #[test]
    fn ny_rent_regulated_succession_regime() {
        let r = check(&baseline("NY"));
        assert_eq!(
            r.regime,
            LeaseSuccessionRegime::NewYorkRentRegulatedSuccession
        );
    }

    #[test]
    fn nj_anti_eviction_regime() {
        let r = check(&baseline("NJ"));
        assert_eq!(
            r.regime,
            LeaseSuccessionRegime::NewJerseyAntiEvictionImmediateFamily
        );
    }

    #[test]
    fn default_state_no_succession_regime() {
        for s in ["AL", "CA", "FL", "TX", "WA", "DC", "WY", "MA"] {
            let r = check(&baseline(s));
            assert_eq!(
                r.regime,
                LeaseSuccessionRegime::DefaultLeaseGovernsNoSuccession,
                "expected {s} default regime"
            );
        }
    }

    // ── NY rent-regulated succession ────────────────────────────────

    #[test]
    fn ny_2_year_residency_traditional_family_eligible() {
        let r = check(&baseline("NY"));
        assert!(r.successor_eligible_for_succession);
        assert_eq!(r.effective_required_residency_years, 2);
    }

    #[test]
    fn ny_1_year_residency_traditional_family_not_eligible() {
        let mut i = baseline("NY");
        i.co_residency_years_with_named_tenant = 1;
        let r = check(&i);
        assert!(!r.residency_satisfied);
        assert!(!r.successor_eligible_for_succession);
    }

    #[test]
    fn ny_senior_1_year_residency_eligible() {
        let mut i = baseline("NY");
        i.co_residency_years_with_named_tenant = 1;
        i.successor_is_senior_or_disabled = true;
        let r = check(&i);
        assert_eq!(r.effective_required_residency_years, 1);
        assert!(r.residency_satisfied);
        assert!(r.successor_eligible_for_succession);
    }

    #[test]
    fn ny_non_traditional_family_eligible() {
        let mut i = baseline("NY");
        i.family_member_category = FamilyMemberCategory::NonTraditionalFamily;
        let r = check(&i);
        assert!(r.family_eligibility_satisfied);
        assert!(r.successor_eligible_for_succession);
    }

    #[test]
    fn ny_not_family_member_not_eligible() {
        let mut i = baseline("NY");
        i.family_member_category = FamilyMemberCategory::NotFamily;
        let r = check(&i);
        assert!(!r.family_eligibility_satisfied);
        assert!(!r.successor_eligible_for_succession);
    }

    #[test]
    fn ny_non_rent_regulated_unit_not_eligible() {
        let mut i = baseline("NY");
        i.unit_is_rent_regulated = false;
        let r = check(&i);
        assert!(!r.successor_eligible_for_succession);
        assert!(r.note.contains("unit not rent-regulated"));
    }

    #[test]
    fn ny_not_primary_resident_not_eligible() {
        let mut i = baseline("NY");
        i.successor_was_primary_resident = false;
        let r = check(&i);
        assert!(!r.successor_eligible_for_succession);
    }

    #[test]
    fn ny_disabled_1_year_residency_eligible() {
        // Disabled successor also gets 1-year reduced threshold.
        let mut i = baseline("NY");
        i.co_residency_years_with_named_tenant = 1;
        i.successor_is_senior_or_disabled = true;
        let r = check(&i);
        assert!(r.successor_eligible_for_succession);
    }

    // ── NJ anti-eviction ───────────────────────────────────────────

    #[test]
    fn nj_traditional_family_protected() {
        let r = check(&baseline("NJ"));
        assert!(r.successor_eligible_for_succession);
    }

    #[test]
    fn nj_non_traditional_family_not_protected() {
        let mut i = baseline("NJ");
        i.family_member_category = FamilyMemberCategory::NonTraditionalFamily;
        let r = check(&i);
        assert!(
            !r.family_eligibility_satisfied,
            "NJ does NOT extend to non-traditional family (regression — distinguishes from NY)"
        );
        assert!(!r.successor_eligible_for_succession);
    }

    #[test]
    fn nj_no_residency_requirement() {
        let mut i = baseline("NJ");
        i.co_residency_years_with_named_tenant = 0;
        let r = check(&i);
        // NJ has no residency requirement under §2A:18-61.1.
        assert!(r.successor_eligible_for_succession);
    }

    // ── Default state ──────────────────────────────────────────────

    #[test]
    fn default_state_no_succession_right() {
        let r = check(&baseline("CA"));
        assert!(!r.successor_eligible_for_succession);
        assert!(r.note.contains("no statutory succession right"));
    }

    #[test]
    fn default_state_traditional_family_still_no_right() {
        let mut i = baseline("TX");
        i.co_residency_years_with_named_tenant = 10;
        let r = check(&i);
        assert!(!r.successor_eligible_for_succession);
    }

    // ── Citations ──────────────────────────────────────────────────

    #[test]
    fn ny_citation_mentions_2523_5_and_2_years() {
        let r = check(&baseline("NY"));
        assert!(r.citation.contains("§ 2523.5(b)(1)"));
        assert!(r.citation.contains("2 years"));
        assert!(r.citation.contains("1 year"));
        assert!(r.citation.contains("non-traditional family"));
    }

    #[test]
    fn nj_citation_mentions_2a_18_61_1_anti_eviction() {
        let r = check(&baseline("NJ"));
        assert!(r.citation.contains("2A:18-61.1"));
        assert!(r.citation.contains("Anti-Eviction Act"));
    }

    // ── Coverage / single-state-uniqueness ─────────────────────────

    #[test]
    fn coverage_is_all_50_states_plus_dc() {
        assert_eq!(RULES.len(), 51);
    }

    #[test]
    fn citations_never_empty() {
        for (code, rule) in RULES.iter() {
            assert!(!rule.citation.is_empty(), "{code} empty citation");
        }
    }

    #[test]
    fn ny_only_rent_regulated_succession_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    LeaseSuccessionRegime::NewYorkRentRegulatedSuccession
                )
            })
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn nj_only_anti_eviction_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    LeaseSuccessionRegime::NewJerseyAntiEvictionImmediateFamily
                )
            })
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn ny_only_non_traditional_family_eligible_state() {
        let count = RULES.iter().filter(|(_, r)| r.non_traditional_family_eligible).count();
        assert_eq!(count, 1, "only NY extends to non-traditional family");
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn ny_eligible_note_describes_regime() {
        let r = check(&baseline("NY"));
        assert!(r.note.contains("New York RSC § 2523.5"));
    }

    #[test]
    fn nj_non_traditional_note_describes_reason() {
        let mut i = baseline("NJ");
        i.family_member_category = FamilyMemberCategory::NonTraditionalFamily;
        let r = check(&i);
        assert!(r.note.contains("not within statutory family definition"));
    }

    // ── Normalization ──────────────────────────────────────────────

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&baseline("ny"));
        assert_eq!(
            r.regime,
            LeaseSuccessionRegime::NewYorkRentRegulatedSuccession
        );
    }
}
