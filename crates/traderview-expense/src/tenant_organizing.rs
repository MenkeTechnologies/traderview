//! State tenant right-to-organize / tenant association protection.
//!
//! Tenant unions and associations are organized groups of building
//! residents who collectively negotiate with landlords on rent
//! increases, habitability, services, eviction defense, and policy.
//! Four states + DC have specific statutes protecting tenants who
//! organize from landlord retaliation, with sharply different
//! remedies. The remaining 46 states either rely on general
//! anti-retaliation provisions (see `retaliation_windows`) or
//! provide no statewide protection.
//!
//! Five regimes:
//!
//! `NewYorkAffirmativeRoomAccess`: NY only. N.Y. RPL § 230. The
//! strongest landlord-side affirmative-obligation regime: landlord
//! MUST permit tenant organizations to meet, **at no cost**, in any
//! community or social room in the building, even if the room is
//! normally subject to a fee. Meetings must be at reasonable times
//! and in a peaceful manner that does not obstruct premises access.
//! Landlord cannot interfere with formation, joining, or
//! participation; cannot harass / penalize for tenant exercising
//! these rights.
//!
//! `DistrictColumbiaStrongCivilPenalty`: DC only. D.C. Code
//! § 42-3505.06. Strongest monetary regime: civil penalty up to
//! **$10,000 per violation** (inflation-adjusted annually) +
//! injunctive relief + damages + suspension/revocation of business
//! licenses + reasonable attorney's fees. Right to meet in tenant
//! unit, community room, lobbies, common areas, or any space
//! reasonably accessible under the lease.
//!
//! `NewJerseyOrganizerProtection`: NJ only. N.J.S.A. 2A:42-10.10
//! prohibits notice-to-quit or possession action as **reprisal**
//! against a tenant who is an organizer, member, or involved in
//! activities of any lawful organization. Remedy: civil action
//! for damages + injunctive/equitable relief.
//!
//! `CaliforniaRetaliatoryEvictionDefense`: CA only. Cal. Civ.
//! Code § 1942.5(d). Unlawful for lessor to increase rent, decrease
//! services, cause lessee to quit involuntarily, bring possession
//! action, or threaten any of the foregoing, for the purpose of
//! retaliating against tenant for lawfully organizing or
//! participating in a lessees' association. **180-day** protected
//! window from the protected act; tenant bears burden of producing
//! evidence of retaliatory motive.
//!
//! `NoStatewideTenantOrganizingProtection`: 46 other states + (none
//! at DC since DC has its own regime). General anti-retaliation
//! statutes (see `retaliation_windows`) may provide indirect
//! protection. No specific tenant-organizing statute confirmed.
//!
//! Sources:
//! [N.Y. RPL § 230 — NY State AG Tenants Rights Guide](https://ag.ny.gov/publications/residential-tenants-rights-guide),
//! [D.C. Code § 42-3505.06 — DC Law Library](https://code.dccouncil.gov/us/dc/council/code/sections/42-3505.06),
//! [N.J.S.A. 2A:42-10.10 — Justia](https://law.justia.com/codes/new-jersey/title-2a/section-2a-42-10-10/),
//! [Cal. Civ. Code § 1942.5(d) — California Legislative Information](https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?sectionNum=1942.5.&lawCode=CIV),
//! [CACI No. 4322 § 1942.5(d) affirmative defense — Justia](https://www.justia.com/trials-litigation/docs/caci/4300/4322/).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantOrganizingRegime {
    NewYorkAffirmativeRoomAccess,
    DistrictColumbiaStrongCivilPenalty,
    NewJerseyOrganizerProtection,
    CaliforniaRetaliatoryEvictionDefense,
    NoStatewideTenantOrganizingProtection,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: TenantOrganizingRegime,
    /// Maximum statutory civil penalty per violation in dollars.
    /// Zero where the regime uses damages-only or no monetary cap.
    pub max_civil_penalty_per_violation_dollars: i64,
    /// Days after the protected organizing act during which
    /// retaliation is presumed unlawful (CA 180-day window).
    pub protected_window_days: u32,
    /// True if landlord has an AFFIRMATIVE OBLIGATION to provide
    /// common-area meeting space at no cost (NY only).
    pub affirmative_common_room_access_required: bool,
    /// True if statute permits suspension or revocation of business
    /// licenses as a remedy (DC only).
    pub business_license_suspension_remedy: bool,
    pub citation: &'static str,
}

const fn rule(
    regime: TenantOrganizingRegime,
    max_civil_penalty_per_violation_dollars: i64,
    protected_window_days: u32,
    affirmative_common_room_access_required: bool,
    business_license_suspension_remedy: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        max_civil_penalty_per_violation_dollars,
        protected_window_days,
        affirmative_common_room_access_required,
        business_license_suspension_remedy,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use TenantOrganizingRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "NY",
        rule(
            NewYorkAffirmativeRoomAccess,
            0,
            0,
            true,
            false,
            "N.Y. RPL § 230 — no landlord shall interfere with right of tenant to form, join, or participate in lawful tenant organization; landlord MUST permit tenant organization meetings at NO COST in any community or social room in the building (even if room is normally fee-based); meetings at reasonable times in peaceful manner",
        ),
    );

    m.insert(
        "DC",
        rule(
            DistrictColumbiaStrongCivilPenalty,
            10_000,
            0,
            true,
            true,
            "D.C. Code § 42-3505.06 — owner / agent shall not interfere with tenant or tenant organizer activities including meetings in tenant unit, community room, lobbies, or other reasonably accessible spaces; civil penalty up to $10,000 per violation (inflation-adjusted) + injunctive relief + damages + suspension/revocation of business licenses + attorney's fees",
        ),
    );

    m.insert(
        "NJ",
        rule(
            NewJerseyOrganizerProtection,
            0,
            0,
            false,
            false,
            "N.J.S.A. 2A:42-10.10 — prohibits notice to quit or possession action as reprisal against tenant who is an organizer / member / involved in activities of any lawful organization; tenant remedy: civil action for damages + injunctive / equitable relief",
        ),
    );

    m.insert(
        "CA",
        rule(
            CaliforniaRetaliatoryEvictionDefense,
            0,
            180,
            false,
            false,
            "Cal. Civ. Code § 1942.5(d) — unlawful for lessor to increase rent / decrease services / cause involuntary quit / bring possession action / threaten any of these for the purpose of retaliating against lessee who lawfully organized or participated in lessees' association; 180-day protected window; tenant bears burden of producing evidence of retaliatory motive",
        ),
    );

    // NoStatewideTenantOrganizingProtection default — 46 other states.
    let no_state = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA", "KS",
        "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NM", "NC",
        "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV",
        "WI", "WY",
    ];
    for code in no_state {
        m.insert(
            code,
            rule(
                NoStatewideTenantOrganizingProtection,
                0,
                0,
                false,
                false,
                "No statewide tenant-organizing protection statute confirmed; general anti-retaliation laws (see retaliation_windows module) may provide indirect protection",
            ),
        );
    }

    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantOrganizingInput {
    pub state_code: String,
    /// True if the tenant engaged in protected organizing activity
    /// (formed / joined / participated in tenant association).
    pub tenant_engaged_in_organizing_activity: bool,
    /// True if the landlord took adverse action against the tenant
    /// (notice to quit, eviction, rent increase, decreased services,
    /// harassment, refusal of meeting room access).
    pub landlord_took_adverse_action: bool,
    /// Days elapsed between the protected organizing act and the
    /// landlord's adverse action. For CA's 180-day window analysis.
    pub days_between_organizing_and_adverse_action: u32,
    /// NY-only: true if landlord refused tenant meeting access to a
    /// common / social room in the building.
    pub landlord_refused_common_room_access: bool,
    /// NY-only: true if the landlord attempted to charge a fee for
    /// tenant association use of the meeting room.
    pub landlord_charged_fee_for_meeting_room: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantOrganizingResult {
    pub regime: TenantOrganizingRegime,
    pub statute_protects_organizing_on_facts: bool,
    pub landlord_compliant: bool,
    pub retaliation_presumed: bool,
    pub maximum_civil_penalty_available_dollars: i64,
    pub business_license_remedy_available: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &TenantOrganizingInput) -> TenantOrganizingResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: TenantOrganizingRegime::NoStatewideTenantOrganizingProtection,
        max_civil_penalty_per_violation_dollars: 0,
        protected_window_days: 0,
        affirmative_common_room_access_required: false,
        business_license_suspension_remedy: false,
        citation: "Unknown state code; no statewide tenant-organizing protection assumed",
    });

    let protected = input.tenant_engaged_in_organizing_activity
        && !matches!(
            rule.regime,
            TenantOrganizingRegime::NoStatewideTenantOrganizingProtection
        );

    // CA's 180-day window — adverse action outside window is not
    // presumed retaliatory.
    let within_protected_window = if rule.protected_window_days > 0 {
        input.days_between_organizing_and_adverse_action <= rule.protected_window_days
    } else {
        true
    };

    let retaliation_presumed =
        protected && input.landlord_took_adverse_action && within_protected_window;

    // NY common-room obligation is a separate compliance trigger.
    let ny_room_violation = matches!(
        rule.regime,
        TenantOrganizingRegime::NewYorkAffirmativeRoomAccess
    ) && (input.landlord_refused_common_room_access
        || input.landlord_charged_fee_for_meeting_room);

    let compliant = !retaliation_presumed && !ny_room_violation;

    let penalty = if !compliant {
        rule.max_civil_penalty_per_violation_dollars
    } else {
        0
    };

    let regime_label = match rule.regime {
        TenantOrganizingRegime::NewYorkAffirmativeRoomAccess => {
            "New York affirmative room-access protection"
        }
        TenantOrganizingRegime::DistrictColumbiaStrongCivilPenalty => {
            "District of Columbia strong civil-penalty regime"
        }
        TenantOrganizingRegime::NewJerseyOrganizerProtection => {
            "New Jersey reprisal-against-organizer protection"
        }
        TenantOrganizingRegime::CaliforniaRetaliatoryEvictionDefense => {
            "California 180-day retaliatory-eviction defense"
        }
        TenantOrganizingRegime::NoStatewideTenantOrganizingProtection => {
            "no statewide tenant-organizing protection"
        }
    };

    let note = if !protected {
        if matches!(
            rule.regime,
            TenantOrganizingRegime::NoStatewideTenantOrganizingProtection
        ) {
            format!(
                "State applies {} regime; tenant has no statewide statutory protection (consider local ordinances or general anti-retaliation).",
                regime_label,
            )
        } else {
            format!(
                "State applies {} regime; tenant has NOT engaged in protected organizing activity, so statute is dormant.",
                regime_label,
            )
        }
    } else if compliant {
        format!(
            "State applies {} regime; landlord compliant on these facts (no adverse action and no common-room violation).",
            regime_label,
        )
    } else {
        let mut parts = vec![format!(
            "State applies {} regime; landlord NON-COMPLIANT",
            regime_label
        )];
        if retaliation_presumed {
            parts.push(if rule.protected_window_days > 0 {
                format!(
                    "adverse action within {}-day protected window — retaliation presumed",
                    rule.protected_window_days
                )
            } else {
                "adverse action against protected tenant — retaliation presumed".to_string()
            });
        }
        if ny_room_violation {
            parts.push(
                "common-room access refused or fee charged in violation of RPL § 230".to_string(),
            );
        }
        if penalty > 0 {
            parts.push(format!(
                "civil penalty up to ${} per violation available",
                penalty
            ));
        }
        if rule.business_license_suspension_remedy {
            parts.push("business license suspension/revocation remedy available".to_string());
        }
        format!("{}.", parts.join("; "))
    };

    TenantOrganizingResult {
        regime: rule.regime,
        statute_protects_organizing_on_facts: protected,
        landlord_compliant: compliant,
        retaliation_presumed,
        maximum_civil_penalty_available_dollars: penalty,
        business_license_remedy_available: !compliant && rule.business_license_suspension_remedy,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline(state: &str) -> TenantOrganizingInput {
        TenantOrganizingInput {
            state_code: state.to_string(),
            tenant_engaged_in_organizing_activity: true,
            landlord_took_adverse_action: true,
            days_between_organizing_and_adverse_action: 30,
            landlord_refused_common_room_access: false,
            landlord_charged_fee_for_meeting_room: false,
        }
    }

    // ── Regime classification ───────────────────────────────────────

    #[test]
    fn ny_affirmative_room_access_regime() {
        let r = check(&baseline("NY"));
        assert_eq!(
            r.regime,
            TenantOrganizingRegime::NewYorkAffirmativeRoomAccess
        );
    }

    #[test]
    fn dc_strong_civil_penalty_regime() {
        let r = check(&baseline("DC"));
        assert_eq!(
            r.regime,
            TenantOrganizingRegime::DistrictColumbiaStrongCivilPenalty
        );
        assert_eq!(r.maximum_civil_penalty_available_dollars, 10_000);
    }

    #[test]
    fn nj_organizer_protection_regime() {
        let r = check(&baseline("NJ"));
        assert_eq!(
            r.regime,
            TenantOrganizingRegime::NewJerseyOrganizerProtection
        );
    }

    #[test]
    fn ca_retaliatory_eviction_defense_regime() {
        let r = check(&baseline("CA"));
        assert_eq!(
            r.regime,
            TenantOrganizingRegime::CaliforniaRetaliatoryEvictionDefense
        );
    }

    #[test]
    fn default_state_no_statewide_protection_regime() {
        for s in ["AL", "FL", "TX", "WA", "WY", "MA"] {
            let r = check(&baseline(s));
            assert_eq!(
                r.regime,
                TenantOrganizingRegime::NoStatewideTenantOrganizingProtection,
                "expected {s} no-protection regime"
            );
        }
    }

    // ── NY common-room obligation ──────────────────────────────────

    #[test]
    fn ny_refused_common_room_violation() {
        let mut i = baseline("NY");
        i.landlord_took_adverse_action = false;
        i.landlord_refused_common_room_access = true;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(r.note.contains("common-room access refused"));
    }

    #[test]
    fn ny_charged_fee_for_meeting_room_violation() {
        let mut i = baseline("NY");
        i.landlord_took_adverse_action = false;
        i.landlord_charged_fee_for_meeting_room = true;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(r.note.contains("fee charged"));
    }

    #[test]
    fn ny_room_access_granted_no_violation() {
        let mut i = baseline("NY");
        i.landlord_took_adverse_action = false;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    // ── DC civil penalty ───────────────────────────────────────────

    #[test]
    fn dc_adverse_action_triggers_10k_penalty() {
        let r = check(&baseline("DC"));
        assert!(r.retaliation_presumed);
        assert_eq!(r.maximum_civil_penalty_available_dollars, 10_000);
    }

    #[test]
    fn dc_business_license_remedy_available_on_violation() {
        let r = check(&baseline("DC"));
        assert!(r.business_license_remedy_available);
    }

    #[test]
    fn dc_no_violation_no_license_remedy() {
        let mut i = baseline("DC");
        i.landlord_took_adverse_action = false;
        let r = check(&i);
        assert!(!r.business_license_remedy_available);
    }

    // ── CA 180-day window ──────────────────────────────────────────

    #[test]
    fn ca_adverse_action_within_180_days_retaliation_presumed() {
        let mut i = baseline("CA");
        i.days_between_organizing_and_adverse_action = 90;
        let r = check(&i);
        assert!(r.retaliation_presumed);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn ca_day_180_boundary_within_protected_window() {
        let mut i = baseline("CA");
        i.days_between_organizing_and_adverse_action = 180;
        let r = check(&i);
        assert!(r.retaliation_presumed);
    }

    #[test]
    fn ca_day_181_outside_protected_window() {
        let mut i = baseline("CA");
        i.days_between_organizing_and_adverse_action = 181;
        let r = check(&i);
        assert!(
            !r.retaliation_presumed,
            "adverse action outside 180-day window is not presumed retaliatory"
        );
        assert!(r.landlord_compliant);
    }

    // ── NJ reprisal protection ─────────────────────────────────────

    #[test]
    fn nj_adverse_action_against_organizer_non_compliant() {
        let r = check(&baseline("NJ"));
        assert!(r.retaliation_presumed);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn nj_no_civil_penalty_dollar_amount() {
        // NJ uses damages-only remedy; no fixed civil penalty.
        let r = check(&baseline("NJ"));
        assert_eq!(r.maximum_civil_penalty_available_dollars, 0);
    }

    // ── Tenant not engaged in organizing ──────────────────────────

    #[test]
    fn no_organizing_activity_statute_dormant() {
        let mut i = baseline("NY");
        i.tenant_engaged_in_organizing_activity = false;
        let r = check(&i);
        assert!(!r.statute_protects_organizing_on_facts);
        assert!(r.landlord_compliant);
    }

    // ── Default state ──────────────────────────────────────────────

    #[test]
    fn default_state_no_protection() {
        let r = check(&baseline("TX"));
        assert!(!r.statute_protects_organizing_on_facts);
        assert!(r.landlord_compliant);
    }

    // ── Citation contents ──────────────────────────────────────────

    #[test]
    fn ny_citation_mentions_rpl_230_and_no_cost() {
        let r = check(&baseline("NY"));
        assert!(r.citation.contains("RPL § 230"));
        assert!(r.citation.contains("NO COST"));
    }

    #[test]
    fn dc_citation_mentions_42_3505_06_and_10_000() {
        let r = check(&baseline("DC"));
        assert!(r.citation.contains("§ 42-3505.06"));
        assert!(r.citation.contains("$10,000"));
        assert!(r.citation.contains("business licenses"));
    }

    #[test]
    fn nj_citation_mentions_2a_42_10_10() {
        let r = check(&baseline("NJ"));
        assert!(r.citation.contains("2A:42-10.10"));
        assert!(r.citation.contains("reprisal"));
    }

    #[test]
    fn ca_citation_mentions_1942_5_d_and_180() {
        let r = check(&baseline("CA"));
        assert!(r.citation.contains("§ 1942.5(d)"));
        assert!(r.citation.contains("180-day"));
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
    fn ny_only_affirmative_room_access_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    TenantOrganizingRegime::NewYorkAffirmativeRoomAccess
                )
            })
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn dc_only_strong_penalty_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    TenantOrganizingRegime::DistrictColumbiaStrongCivilPenalty
                )
            })
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn nj_only_organizer_protection_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    TenantOrganizingRegime::NewJerseyOrganizerProtection
                )
            })
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn ca_only_retaliatory_eviction_defense_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    TenantOrganizingRegime::CaliforniaRetaliatoryEvictionDefense
                )
            })
            .count();
        assert_eq!(count, 1);
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn dc_violation_note_mentions_business_license() {
        let r = check(&baseline("DC"));
        assert!(r.note.contains("business license"));
    }

    #[test]
    fn ny_violation_note_mentions_rpl_230() {
        let mut i = baseline("NY");
        i.landlord_took_adverse_action = false;
        i.landlord_refused_common_room_access = true;
        let r = check(&i);
        assert!(r.note.contains("RPL § 230"));
    }

    #[test]
    fn ca_within_window_note_mentions_180_day() {
        let mut i = baseline("CA");
        i.days_between_organizing_and_adverse_action = 90;
        let r = check(&i);
        assert!(r.note.contains("180-day protected window"));
    }

    // ── Normalization ──────────────────────────────────────────────

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&baseline("ny"));
        assert_eq!(
            r.regime,
            TenantOrganizingRegime::NewYorkAffirmativeRoomAccess
        );
    }
}
