//! State tenant roommate / additional-occupant authorization rules.
//!
//! Distinct from `sublet_consent` (assignment / re-leasing of the
//! entire unit) and from `occupancy_standards` (building-code
//! occupancy limits). This module captures the tenant's STATUTORY
//! right to have an unrelated adult roommate or additional
//! occupant move in despite contrary lease terms. Two states have
//! specific roommate statutes; the rest fall to the lease.
//!
//! Three regimes:
//!
//! `NewYorkStatutoryRoommateRight`: NY only. RPL § 235-f (the
//! "Roommate Law"). Tenant has STATUTORY right to share the
//! apartment with: immediate family of the tenant, ONE additional
//! adult occupant, and dependent children of that occupant. The
//! apartment must be the tenant's primary residence. Any lease
//! restriction prohibiting unrelated occupants is statutorily VOID
//! and unenforceable as against public policy (RPL § 235-f(7)).
//! Tenant must inform the landlord of the occupant's name within
//! 30 days of occupancy commencement OR 30 days following the
//! landlord's request, whichever later. Multi-tenant leases
//! preserve the right but with a unit cap: total tenants +
//! occupants ≤ number of tenants on the lease.
//!
//! `CaliforniaTwoPlusOneFormula`: CA only. State-law occupancy
//! formula limits occupants to 2 per bedroom PLUS 1 additional
//! occupant overall. E.g., 1-bedroom unit → max 3 occupants;
//! 2-bedroom unit → max 5 occupants. Lease may impose roommate-
//! consent terms BUT cannot reduce the occupancy limit below this
//! floor. Occupants residing continuously may acquire de-facto
//! tenant rights via implicit-consent doctrine.
//!
//! `DefaultLeaseGoverns`: 48 other states + DC. Lease terms govern
//! roommate / occupant authorization. State building codes set
//! maximum occupancy limits but do not create an affirmative
//! tenant right against the lease's restrictions. NJ recognizes
//! a de-facto co-tenancy doctrine; MA permits minor / elderly /
//! disabled relatives as guests. Tenants in default-regime states
//! must obtain landlord consent before moving in roommates.
//!
//! Sources:
//! [N.Y. RPL § 235-f — New York Senate](https://www.nysenate.gov/legislation/laws/RPP/235-F),
//! [N.Y. RPL § 235-f — FindLaw](https://codes.findlaw.com/ny/real-property-law/rpp-sect-235-f/),
//! [Met Council on Housing — Your Right To Have A Roommate](https://www.metcouncilonhousing.org/help-answers/your-right-to-have-a-roommate/),
//! [Law Soup CA — Roommates and Houseguests](https://cal.lawsoup.org/legal-guides/tenant-renter/roommates-and-houseguests/),
//! [California Tenants Guide (CA Courts)](https://www4.courts.ca.gov/documents/California-Tenants-Guide.pdf).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoommateAuthorizationRegime {
    NewYorkStatutoryRoommateRight,
    CaliforniaTwoPlusOneFormula,
    DefaultLeaseGoverns,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: RoommateAuthorizationRegime,
    /// Maximum number of additional unrelated adult occupants the
    /// statute permits per tenant. NY = 1; CA capped by 2-per-
    /// bedroom + 1 formula; default = 0 (lease governs).
    pub additional_unrelated_occupants_per_tenant: u32,
    /// True if lease restrictions prohibiting roommates are
    /// statutorily VOID and unenforceable.
    pub lease_restrictions_void: bool,
    /// Days the tenant must notify landlord of new occupant.
    pub notification_window_days: u32,
    pub citation: &'static str,
}

const fn rule(
    regime: RoommateAuthorizationRegime,
    additional_unrelated_occupants_per_tenant: u32,
    lease_restrictions_void: bool,
    notification_window_days: u32,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        additional_unrelated_occupants_per_tenant,
        lease_restrictions_void,
        notification_window_days,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use RoommateAuthorizationRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "NY",
        rule(
            NewYorkStatutoryRoommateRight,
            1,
            true,
            30,
            "N.Y. RPL § 235-f (Roommate Law) — tenant has statutory right to share apartment with immediate family + ONE additional adult occupant + dependent children of occupant when the apartment is tenant's primary residence; lease restrictions prohibiting unrelated occupants are VOID and unenforceable as against public policy under § 235-f(7); tenant must inform landlord of occupant name within 30 days of occupancy commencement or 30 days following landlord request",
        ),
    );

    m.insert(
        "CA",
        rule(
            CaliforniaTwoPlusOneFormula,
            0,
            false,
            0,
            "California occupancy formula — '2 plus 1' state-law standard limits occupants to 2 per bedroom + 1 additional overall (1-BR max 3; 2-BR max 5); lease may impose roommate-consent terms but cannot reduce the occupancy limit below this floor; occupants residing continuously may acquire de-facto tenant rights via implicit-consent doctrine",
        ),
    );

    // DefaultLeaseGoverns — 48 other states + DC.
    let default_states = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE",
        "FL", "GA", "HI", "ID", "IL", "IN", "IA", "KS",
        "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS",
        "MO", "MT", "NE", "NV", "NH", "NJ", "NM", "NC",
        "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD",
        "TN", "TX", "UT", "VT", "VA", "WA", "WV", "WI",
        "WY",
    ];
    for code in default_states {
        m.insert(
            code,
            rule(
                DefaultLeaseGoverns,
                0,
                false,
                0,
                "Lease terms govern roommate / occupant authorization; state building codes set maximum occupancy limits but do not create an affirmative tenant right against lease restrictions; NJ recognizes de-facto co-tenancy doctrine; MA permits minor/elderly/disabled relatives as guests; tenants must generally obtain landlord consent before moving in roommates",
            ),
        );
    }

    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoommateAuthorizationInput {
    pub state_code: String,
    /// Number of tenants currently signed on the lease.
    pub tenants_on_lease: u32,
    /// Number of additional unrelated adults the tenant wishes to
    /// have occupy the unit (excluding tenant + tenant's immediate
    /// family + dependent children).
    pub additional_unrelated_adults_proposed: u32,
    /// Number of bedrooms in the unit (for CA occupancy formula).
    pub number_of_bedrooms: u32,
    /// Total proposed occupant count including tenant, immediate
    /// family, occupant, and dependent children (for CA test).
    pub total_proposed_occupants: u32,
    /// Days since occupancy commenced (for NY notification window).
    pub days_since_occupancy_commenced: u32,
    /// True if tenant notified landlord of the new occupant.
    pub tenant_notified_landlord: bool,
    /// True if the apartment is the tenant's primary residence
    /// (NY requirement).
    pub tenant_primary_residence: bool,
    /// True if the lease contains terms restricting / prohibiting
    /// unrelated occupants.
    pub lease_contains_roommate_restriction: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoommateAuthorizationResult {
    pub regime: RoommateAuthorizationRegime,
    pub additional_occupant_permitted_by_statute: bool,
    pub lease_restriction_enforceable: bool,
    pub tenant_compliant_with_notification: bool,
    pub ca_two_plus_one_max_occupants: u32,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &RoommateAuthorizationInput) -> RoommateAuthorizationResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: RoommateAuthorizationRegime::DefaultLeaseGoverns,
        additional_unrelated_occupants_per_tenant: 0,
        lease_restrictions_void: false,
        notification_window_days: 0,
        citation: "Unknown state code; lease-governs default assumed",
    });

    let ca_max_occupants = 2 * input.number_of_bedrooms + 1;

    let (permitted, lease_enforceable, notification_compliant) = match rule.regime {
        RoommateAuthorizationRegime::NewYorkStatutoryRoommateRight => {
            // NY: permits 1 additional occupant per tenant + must be
            // tenant's primary residence. Multi-tenant leases cap
            // total tenants+occupants at lease tenant count.
            let per_tenant_cap = rule.additional_unrelated_occupants_per_tenant
                * input.tenants_on_lease;
            let multi_tenant_cap_ok = if input.tenants_on_lease > 1 {
                input.tenants_on_lease + input.additional_unrelated_adults_proposed
                    <= input.tenants_on_lease + per_tenant_cap
            } else {
                true
            };
            let permitted = input.tenant_primary_residence
                && input.additional_unrelated_adults_proposed <= per_tenant_cap
                && multi_tenant_cap_ok;
            // Lease restriction unenforceable when permitted by statute.
            let lease_enforceable = !(permitted && input.lease_contains_roommate_restriction);
            // Notification compliance: within 30 days of occupancy.
            let notification_compliant = input.tenant_notified_landlord
                || input.days_since_occupancy_commenced <= rule.notification_window_days;
            (permitted, lease_enforceable, notification_compliant)
        }
        RoommateAuthorizationRegime::CaliforniaTwoPlusOneFormula => {
            // CA: 2 per bedroom + 1 additional. Lease can restrict
            // above the floor but not reduce it.
            let permitted = input.total_proposed_occupants <= ca_max_occupants;
            // Lease can still impose roommate-consent terms — not
            // void. Lease enforceable above the statutory floor.
            let lease_enforceable = true;
            (permitted, lease_enforceable, true)
        }
        RoommateAuthorizationRegime::DefaultLeaseGoverns => {
            // Default: lease governs; no statutory entitlement to
            // bypass restrictions.
            let permitted = !input.lease_contains_roommate_restriction;
            let lease_enforceable = true;
            (permitted, lease_enforceable, true)
        }
    };

    let regime_label = match rule.regime {
        RoommateAuthorizationRegime::NewYorkStatutoryRoommateRight => {
            "New York RPL § 235-f statutory roommate right"
        }
        RoommateAuthorizationRegime::CaliforniaTwoPlusOneFormula => {
            "California '2 plus 1' occupancy formula"
        }
        RoommateAuthorizationRegime::DefaultLeaseGoverns => "default lease-governs",
    };

    let note = match rule.regime {
        RoommateAuthorizationRegime::NewYorkStatutoryRoommateRight => {
            format!(
                "State applies {} regime; tenant primary residence = {}; {} additional adult(s) proposed (cap = 1 per tenant × {} tenants); statutory permission = {}; lease restriction enforceable = {} (lease-restriction-VOID rule applies when statute permits); 30-day notification {} (window: {} days; tenant notified: {}).",
                regime_label,
                input.tenant_primary_residence,
                input.additional_unrelated_adults_proposed,
                input.tenants_on_lease,
                permitted,
                lease_enforceable,
                if notification_compliant { "SATISFIED" } else { "NOT YET satisfied" },
                input.days_since_occupancy_commenced,
                input.tenant_notified_landlord,
            )
        }
        RoommateAuthorizationRegime::CaliforniaTwoPlusOneFormula => {
            format!(
                "State applies {} regime; {} bedrooms × 2 + 1 = max {} occupants; {} proposed; permitted = {}.",
                regime_label,
                input.number_of_bedrooms,
                ca_max_occupants,
                input.total_proposed_occupants,
                permitted,
            )
        }
        RoommateAuthorizationRegime::DefaultLeaseGoverns => {
            format!(
                "State applies {} regime; lease contains roommate restriction = {}; additional occupant permitted on these facts = {}.",
                regime_label,
                input.lease_contains_roommate_restriction,
                permitted,
            )
        }
    };

    RoommateAuthorizationResult {
        regime: rule.regime,
        additional_occupant_permitted_by_statute: permitted,
        lease_restriction_enforceable: lease_enforceable,
        tenant_compliant_with_notification: notification_compliant,
        ca_two_plus_one_max_occupants: ca_max_occupants,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline(state: &str) -> RoommateAuthorizationInput {
        RoommateAuthorizationInput {
            state_code: state.to_string(),
            tenants_on_lease: 1,
            additional_unrelated_adults_proposed: 1,
            number_of_bedrooms: 1,
            total_proposed_occupants: 2,
            days_since_occupancy_commenced: 15,
            tenant_notified_landlord: true,
            tenant_primary_residence: true,
            lease_contains_roommate_restriction: false,
        }
    }

    // ── Regime classification ───────────────────────────────────────

    #[test]
    fn ny_statutory_roommate_right_regime() {
        let r = check(&baseline("NY"));
        assert_eq!(
            r.regime,
            RoommateAuthorizationRegime::NewYorkStatutoryRoommateRight
        );
    }

    #[test]
    fn ca_two_plus_one_regime() {
        let r = check(&baseline("CA"));
        assert_eq!(
            r.regime,
            RoommateAuthorizationRegime::CaliforniaTwoPlusOneFormula
        );
    }

    #[test]
    fn default_state_lease_governs_regime() {
        for s in ["AL", "FL", "TX", "WA", "DC", "WY", "MA", "NJ", "IL"] {
            let r = check(&baseline(s));
            assert_eq!(
                r.regime,
                RoommateAuthorizationRegime::DefaultLeaseGoverns,
                "expected {s} default regime"
            );
        }
    }

    // ── NY: 1 additional + primary-residence + 30-day notice ──────

    #[test]
    fn ny_single_tenant_one_additional_permitted() {
        let r = check(&baseline("NY"));
        assert!(r.additional_occupant_permitted_by_statute);
    }

    #[test]
    fn ny_two_additional_adults_exceeds_per_tenant_cap() {
        let mut i = baseline("NY");
        i.additional_unrelated_adults_proposed = 2;
        let r = check(&i);
        assert!(
            !r.additional_occupant_permitted_by_statute,
            "RPL § 235-f permits only 1 additional adult per tenant"
        );
    }

    #[test]
    fn ny_not_primary_residence_no_statutory_right() {
        let mut i = baseline("NY");
        i.tenant_primary_residence = false;
        let r = check(&i);
        assert!(!r.additional_occupant_permitted_by_statute);
    }

    #[test]
    fn ny_lease_restriction_void_when_statute_permits() {
        let mut i = baseline("NY");
        i.lease_contains_roommate_restriction = true;
        let r = check(&i);
        assert!(
            !r.lease_restriction_enforceable,
            "lease prohibition unenforceable when § 235-f permits the occupant"
        );
    }

    #[test]
    fn ny_30_day_notification_window_compliance() {
        let mut i = baseline("NY");
        i.tenant_notified_landlord = false;
        i.days_since_occupancy_commenced = 30;
        let r = check(&i);
        assert!(r.tenant_compliant_with_notification);
    }

    #[test]
    fn ny_31_day_no_notification_non_compliant() {
        let mut i = baseline("NY");
        i.tenant_notified_landlord = false;
        i.days_since_occupancy_commenced = 31;
        let r = check(&i);
        assert!(!r.tenant_compliant_with_notification);
    }

    #[test]
    fn ny_multi_tenant_lease_caps_total_at_lease_count() {
        // 2 tenants on lease + 2 additional = 4 total; cap = 2 + 2×1 = 4. OK.
        let mut i = baseline("NY");
        i.tenants_on_lease = 2;
        i.additional_unrelated_adults_proposed = 2;
        let r = check(&i);
        assert!(r.additional_occupant_permitted_by_statute);
    }

    #[test]
    fn ny_multi_tenant_3_additional_exceeds_per_tenant_cap() {
        // 2 tenants on lease + 3 additional = 5 total; cap is 2 tenants
        // × 1 additional each = 2 → 3 exceeds.
        let mut i = baseline("NY");
        i.tenants_on_lease = 2;
        i.additional_unrelated_adults_proposed = 3;
        let r = check(&i);
        assert!(!r.additional_occupant_permitted_by_statute);
    }

    // ── CA "2 plus 1" formula ──────────────────────────────────────

    #[test]
    fn ca_one_bedroom_max_3_occupants() {
        let mut i = baseline("CA");
        i.number_of_bedrooms = 1;
        i.total_proposed_occupants = 3;
        let r = check(&i);
        assert_eq!(r.ca_two_plus_one_max_occupants, 3);
        assert!(r.additional_occupant_permitted_by_statute);
    }

    #[test]
    fn ca_one_bedroom_4_occupants_exceeds_max() {
        let mut i = baseline("CA");
        i.number_of_bedrooms = 1;
        i.total_proposed_occupants = 4;
        let r = check(&i);
        assert!(!r.additional_occupant_permitted_by_statute);
    }

    #[test]
    fn ca_two_bedroom_max_5_occupants() {
        let mut i = baseline("CA");
        i.number_of_bedrooms = 2;
        i.total_proposed_occupants = 5;
        let r = check(&i);
        assert_eq!(r.ca_two_plus_one_max_occupants, 5);
        assert!(r.additional_occupant_permitted_by_statute);
    }

    #[test]
    fn ca_three_bedroom_max_7_occupants() {
        let mut i = baseline("CA");
        i.number_of_bedrooms = 3;
        i.total_proposed_occupants = 7;
        let r = check(&i);
        assert_eq!(r.ca_two_plus_one_max_occupants, 7);
        assert!(r.additional_occupant_permitted_by_statute);
    }

    #[test]
    fn ca_lease_remains_enforceable_above_floor() {
        // Lease can still impose roommate-consent terms.
        let r = check(&baseline("CA"));
        assert!(r.lease_restriction_enforceable);
    }

    // ── Default lease-governs ──────────────────────────────────────

    #[test]
    fn default_state_no_lease_restriction_permitted() {
        let r = check(&baseline("TX"));
        assert!(r.additional_occupant_permitted_by_statute);
    }

    #[test]
    fn default_state_lease_restriction_blocks_roommate() {
        let mut i = baseline("TX");
        i.lease_contains_roommate_restriction = true;
        let r = check(&i);
        assert!(
            !r.additional_occupant_permitted_by_statute,
            "default regime: lease restriction binds"
        );
        assert!(r.lease_restriction_enforceable);
    }

    // ── Citation contents ──────────────────────────────────────────

    #[test]
    fn ny_citation_mentions_235_f_and_void() {
        let r = check(&baseline("NY"));
        assert!(r.citation.contains("§ 235-f"));
        assert!(r.citation.contains("VOID"));
        assert!(r.citation.contains("30 days"));
    }

    #[test]
    fn ca_citation_mentions_2_plus_1_and_bedroom_formula() {
        let r = check(&baseline("CA"));
        assert!(r.citation.contains("2 plus 1"));
        assert!(r.citation.contains("2 per bedroom"));
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
    fn ny_only_statutory_roommate_right_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    RoommateAuthorizationRegime::NewYorkStatutoryRoommateRight
                )
            })
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn ca_only_two_plus_one_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    RoommateAuthorizationRegime::CaliforniaTwoPlusOneFormula
                )
            })
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn ny_only_lease_restrictions_void_state() {
        let count = RULES.iter().filter(|(_, r)| r.lease_restrictions_void).count();
        assert_eq!(count, 1, "only NY makes lease restrictions VOID");
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn ny_note_describes_per_tenant_cap_arithmetic() {
        let r = check(&baseline("NY"));
        assert!(r.note.contains("cap = 1 per tenant"));
    }

    #[test]
    fn ca_note_describes_bedroom_formula() {
        let r = check(&baseline("CA"));
        assert!(r.note.contains("max"));
        assert!(r.note.contains("bedrooms"));
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&baseline("ny"));
        assert_eq!(
            r.regime,
            RoommateAuthorizationRegime::NewYorkStatutoryRoommateRight
        );
    }
}
