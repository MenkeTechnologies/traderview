//! State tenant "right to dry" — right to install and use a
//! clothesline or drying rack in leased premises.
//!
//! The broader "right to dry" movement has produced statutes in
//! 19 states voiding HOA / condo clothesline bans, but most such
//! statutes do NOT extend to landlord-tenant rentals. California
//! is the only state that has explicitly extended the right to
//! tenants — Cal. Civ. Code § 1940.20 (added by AB 1448, 2015)
//! lets tenants use a clothesline or drying rack in the tenant's
//! private leased area subject to safety and maintenance
//! conditions. Trader-tenants in expensive HCOL markets get real
//! savings on dryer energy costs.
//!
//! Two regimes:
//!
//! `CaliforniaTenantClotheslineRight`: CA only. Cal. Civ. Code
//! § 1940.20 (added by AB 1448, eff. 2015-09-08). Lessor / landlord
//! shall permit a tenant to utilize a clothesline or drying rack
//! in the tenant's private leased area, subject to certain
//! conditions:
//!   - The clothesline or drying rack does not interfere with
//!     maintenance of the premises.
//!   - The clothesline or drying rack does not create a health or
//!     safety hazard.
//!   - The clothesline or drying rack is compatible with applicable
//!     building codes and homeowner association rules (where
//!     applicable).
//!   - The leased area is private (balcony, patio, backyard
//!     dedicated to the tenant) — not common areas.
//!
//! `NoStatewideTenantClotheslineRight`: 49 other states + DC.
//! Most "right to dry" laws (FL § 163.04, CO § 38-30-168, HI
//! § 196-7.6, ME tit. 33 § 1521, MD Real Prop. § 2-119, VT
//! 27 V.S.A. § 544, AZ § 33-1816, OR ORS 105.480, NM § 47-16-3.1,
//! NV NRS 111.241, NC § 47C-2-119, etc.) apply only to deed
//! restrictions, HOA / condo covenants, and similar non-lease
//! restrictions. They do NOT extend to landlord-tenant rentals.
//! Tenants in these states must rely on lease negotiation.
//!
//! Sources:
//! [Cal. Civ. Code § 1940.20 — AB 1448 Bill Analysis](https://www.leginfo.ca.gov/pub/15-16/bill/asm/ab_1401-1450/ab_1448_cfa_20150511_130839_asm_comm.html),
//! [Sightline Institute — Clothesline Bans Void in 19 States](https://www.sightline.org/2012/02/21/clothesline-bans-void-in-19-states/),
//! [ABA Journal — 19 right to dry states outlaw clothesline bans](https://www.abajournal.com/news/article/20_right_to_dry_states_outlaw_clothesline_bans_is_yours_among_them),
//! [Landlord Talking — Tenant Right to Dry guide](https://www.landlordtalking.com/tips/landlords-must-honor-a-tenants-right-to-dry/),
//! [HOA Law Blog — California Civ. Code § 4750 + AB 1448](https://www.hoalawblog.com/clotheslines_and_california_ho_1/).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightToDryRegime {
    CaliforniaTenantClotheslineRight,
    NoStatewideTenantClotheslineRight,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: RightToDryRegime,
    /// True if statute affirmatively extends right to rental
    /// tenants (vs HOA/condo only).
    pub extends_to_rental_tenants: bool,
    /// True if statute restricts permitted use to the tenant's
    /// private leased area (balcony / patio / yard) — common areas
    /// excluded.
    pub private_leased_area_only: bool,
    pub citation: &'static str,
}

const fn rule(
    regime: RightToDryRegime,
    extends_to_rental_tenants: bool,
    private_leased_area_only: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        extends_to_rental_tenants,
        private_leased_area_only,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use RightToDryRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "CA",
        rule(
            CaliforniaTenantClotheslineRight,
            true,
            true,
            "Cal. Civ. Code § 1940.20 (added by AB 1448, eff. 2015-09-08) — lessor / landlord shall permit tenant to utilize clothesline or drying rack in tenant's PRIVATE leased area subject to no-interference-with-maintenance + no-health-or-safety-hazard + building-code-and-HOA compatibility; common areas EXCLUDED",
        ),
    );

    // NoStatewideTenantClotheslineRight default — 49 other states + DC.
    // Many states have "right to dry" laws but they apply to HOAs /
    // condos only and do NOT extend to landlord-tenant rentals.
    let no_state = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE",
        "FL", "GA", "HI", "ID", "IL", "IN", "IA", "KS",
        "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS",
        "MO", "MT", "NE", "NV", "NH", "NJ", "NM", "NY",
        "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC",
        "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV",
        "WI", "WY",
    ];
    for code in no_state {
        m.insert(
            code,
            rule(
                NoStatewideTenantClotheslineRight,
                false,
                false,
                "No statewide tenant clothesline / drying-rack right; broader 'right to dry' statutes in FL § 163.04 / CO § 38-30-168 / HI § 196-7.6 / ME tit. 33 § 1521 / MD Real Prop. § 2-119 / etc. apply to HOA / condo covenants only and do NOT extend to landlord-tenant rentals",
            ),
        );
    }

    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RightToDryInput {
    pub state_code: String,
    /// True if the tenant intends to use the clothesline / drying
    /// rack in their private leased area (balcony, patio, yard).
    pub installation_in_private_leased_area: bool,
    /// True if the clothesline / rack would interfere with the
    /// landlord's maintenance of the premises.
    pub interferes_with_maintenance: bool,
    /// True if the clothesline / rack would create a health or
    /// safety hazard.
    pub creates_health_or_safety_hazard: bool,
    /// True if the installation complies with applicable building
    /// codes and HOA rules.
    pub complies_with_building_code_and_hoa: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RightToDryResult {
    pub regime: RightToDryRegime,
    pub statute_extends_to_tenant: bool,
    pub installation_permitted: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &RightToDryInput) -> RightToDryResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: RightToDryRegime::NoStatewideTenantClotheslineRight,
        extends_to_rental_tenants: false,
        private_leased_area_only: false,
        citation: "Unknown state code; no statewide tenant clothesline right assumed",
    });

    let permitted = match rule.regime {
        RightToDryRegime::CaliforniaTenantClotheslineRight => {
            input.installation_in_private_leased_area
                && !input.interferes_with_maintenance
                && !input.creates_health_or_safety_hazard
                && input.complies_with_building_code_and_hoa
        }
        RightToDryRegime::NoStatewideTenantClotheslineRight => false,
    };

    let regime_label = match rule.regime {
        RightToDryRegime::CaliforniaTenantClotheslineRight => {
            "California Civ. Code § 1940.20 tenant clothesline right"
        }
        RightToDryRegime::NoStatewideTenantClotheslineRight => {
            "no statewide tenant clothesline right"
        }
    };

    let note = if !rule.extends_to_rental_tenants {
        format!(
            "State applies {} regime; tenant has no statutory right to install clothesline/drying rack (lease governs; HOA/condo laws don't extend to rentals).",
            regime_label,
        )
    } else if permitted {
        format!(
            "State applies {} regime; tenant installation permitted on these facts.",
            regime_label,
        )
    } else {
        let mut reasons = vec![];
        if !input.installation_in_private_leased_area {
            reasons.push("installation NOT in tenant's private leased area");
        }
        if input.interferes_with_maintenance {
            reasons.push("interferes with landlord's maintenance");
        }
        if input.creates_health_or_safety_hazard {
            reasons.push("creates health/safety hazard");
        }
        if !input.complies_with_building_code_and_hoa {
            reasons.push("does not comply with building code/HOA");
        }
        format!(
            "State applies {} regime; installation NOT permitted: {}.",
            regime_label,
            reasons.join("; "),
        )
    };

    RightToDryResult {
        regime: rule.regime,
        statute_extends_to_tenant: rule.extends_to_rental_tenants,
        installation_permitted: permitted,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline(state: &str) -> RightToDryInput {
        RightToDryInput {
            state_code: state.to_string(),
            installation_in_private_leased_area: true,
            interferes_with_maintenance: false,
            creates_health_or_safety_hazard: false,
            complies_with_building_code_and_hoa: true,
        }
    }

    // ── Regime classification ───────────────────────────────────────

    #[test]
    fn ca_tenant_clothesline_right_regime() {
        let r = check(&baseline("CA"));
        assert_eq!(
            r.regime,
            RightToDryRegime::CaliforniaTenantClotheslineRight
        );
    }

    #[test]
    fn default_state_no_statewide_right_regime() {
        for s in ["AL", "FL", "TX", "WA", "DC", "WY", "MA", "NJ", "NY"] {
            let r = check(&baseline(s));
            assert_eq!(
                r.regime,
                RightToDryRegime::NoStatewideTenantClotheslineRight,
                "expected {s} no-tenant-right regime"
            );
        }
    }

    // ── CA 4-prong test ────────────────────────────────────────────

    #[test]
    fn ca_all_4_prongs_met_installation_permitted() {
        let r = check(&baseline("CA"));
        assert!(r.statute_extends_to_tenant);
        assert!(r.installation_permitted);
    }

    #[test]
    fn ca_not_private_leased_area_not_permitted() {
        let mut i = baseline("CA");
        i.installation_in_private_leased_area = false;
        let r = check(&i);
        assert!(!r.installation_permitted);
        assert!(r.note.contains("NOT in tenant's private leased area"));
    }

    #[test]
    fn ca_interferes_with_maintenance_not_permitted() {
        let mut i = baseline("CA");
        i.interferes_with_maintenance = true;
        let r = check(&i);
        assert!(!r.installation_permitted);
        assert!(r.note.contains("interferes with landlord's maintenance"));
    }

    #[test]
    fn ca_health_safety_hazard_not_permitted() {
        let mut i = baseline("CA");
        i.creates_health_or_safety_hazard = true;
        let r = check(&i);
        assert!(!r.installation_permitted);
        assert!(r.note.contains("health/safety hazard"));
    }

    #[test]
    fn ca_non_compliant_building_code_not_permitted() {
        let mut i = baseline("CA");
        i.complies_with_building_code_and_hoa = false;
        let r = check(&i);
        assert!(!r.installation_permitted);
        assert!(r.note.contains("building code/HOA"));
    }

    // ── Default state ──────────────────────────────────────────────

    #[test]
    fn default_state_no_tenant_statutory_right() {
        // Florida has "right to dry" statute (§163.04) but it
        // doesn't extend to tenants — only HOA/condo restrictions.
        let r = check(&baseline("FL"));
        assert!(!r.statute_extends_to_tenant);
        assert!(!r.installation_permitted);
        assert!(r.note.contains("no statutory right to install"));
    }

    #[test]
    fn default_state_note_mentions_hoa_only() {
        let r = check(&baseline("FL"));
        assert!(r.note.contains("HOA/condo laws don't extend to rentals"));
    }

    // ── Citation contents ──────────────────────────────────────────

    #[test]
    fn ca_citation_mentions_1940_20_ab_1448_and_private_area() {
        let r = check(&baseline("CA"));
        assert!(r.citation.contains("§ 1940.20"));
        assert!(r.citation.contains("AB 1448"));
        assert!(r.citation.contains("2015-09-08"));
        assert!(r.citation.contains("PRIVATE leased area"));
        assert!(r.citation.contains("common areas EXCLUDED"));
    }

    #[test]
    fn default_citation_mentions_hoa_only_statutes() {
        let r = check(&baseline("FL"));
        assert!(r.citation.contains("FL § 163.04"));
        assert!(r.citation.contains("HOA / condo covenants only"));
        assert!(r.citation.contains("do NOT extend to landlord-tenant rentals"));
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
    fn ca_only_extends_to_tenants_state() {
        let count = RULES.iter().filter(|(_, r)| r.extends_to_rental_tenants).count();
        assert_eq!(count, 1, "only CA extends statutory right to tenants");
    }

    #[test]
    fn ca_only_clothesline_regime_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    RightToDryRegime::CaliforniaTenantClotheslineRight
                )
            })
            .count();
        assert_eq!(count, 1);
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn ca_permitted_note_describes_regime() {
        let r = check(&baseline("CA"));
        assert!(r.note.contains("California Civ. Code § 1940.20"));
    }

    #[test]
    fn ca_multiple_prong_failures_listed() {
        let mut i = baseline("CA");
        i.interferes_with_maintenance = true;
        i.creates_health_or_safety_hazard = true;
        let r = check(&i);
        assert!(r.note.contains("interferes with landlord's maintenance"));
        assert!(r.note.contains("health/safety hazard"));
    }

    // ── Normalization ──────────────────────────────────────────────

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&baseline("ca"));
        assert_eq!(
            r.regime,
            RightToDryRegime::CaliforniaTenantClotheslineRight
        );
    }
}
