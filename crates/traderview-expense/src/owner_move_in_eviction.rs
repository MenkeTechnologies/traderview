//! State owner-move-in (OMI) / no-fault eviction restrictions
//! compliance.
//!
//! When a landlord seeks to terminate a tenancy on the no-fault
//! ground that the owner (or a qualified family member) intends to
//! occupy the unit personally, five distinct statutory regimes
//! govern. OMI is the most-litigated no-fault eviction ground
//! nationally because of the obvious risk of pretextual abuse —
//! "I'll move in" → tenant vacates → owner re-rents at market.
//! Recent legislative waves (CA SB 567 2024; OR SB 608 2019)
//! tightened the requirements significantly.
//!
//! Five regimes:
//!
//! `CaliforniaSb567Strict`: CA only. Cal. Civ. Code § 1946.2 as
//! amended by SB 567 (eff. 2024-04-01). Owner or qualified family
//! member (spouse, domestic partner, parent, child, grandchild,
//! grandparent) must:
//!   - Move in within **90 days** of tenant vacating;
//!   - Reside as primary residence for at least **12 continuous
//!     months**;
//!   - Pay relocation assistance (typically 1 month's rent).
//!
//! Failure exposes the landlord to tenant remedies under
//! §§ 1946.2(g), 1947.12.
//!
//! `OregonSb608Combined`: OR only. ORS 90.427 as amended by SB 608
//! (eff. 2019-02-28). For month-to-month tenancies over one year,
//! 90-day written notice required AND landlord must pay 1 month's
//! rent in relocation assistance when terminating for qualifying
//! landlord-based reasons including OMI.
//!
//! `NewJerseyTripleDamagesGoodFaith`: NJ only. N.J.S.A. 2A:18-61.1(l)(3)
//! permits OMI removal only for owner of building of 3 residential
//! units or less. If owner gives OMI notice and "arbitrarily fails
//! to personally occupy the premises for at least 6 months," the
//! owner is liable under § 2A:18-61.6 for **THREE TIMES the
//! tenant's damages plus attorney fees and costs**.
//!
//! `NewYorkRentStabilizedOnlyOneUnit`: NY only (rent-stabilized
//! units). NYC Rent Stabilization Code § 2524.4. Only one of the
//! individual owners of a building may take possession of only one
//! dwelling unit for personal or immediate-family use. Requires
//! "immediate and compelling need" + use as principal residence for
//! **3 years** following recovery. If owner fails to maintain
//! personal use for 3 years, the owner may lose the right to rent
//! increases on other flats in the building for 3 years.
//!
//! `NoStateOwnerMoveInRestriction`: 46 other states + DC. No
//! statewide statutory restriction on OMI evictions confirmed; the
//! tenancy-termination rules under the applicable state landlord-
//! tenant act apply. Many cities maintain local OMI ordinances
//! (e.g., SF Rent Ordinance § 37.9(a)(8); DC Tenant Opportunity to
//! Purchase Act etc.).
//!
//! Sources:
//! [Cal. SB 567 (2024) — CA Apartment Association](https://caanet.org/governor-signs-bill-revising-states-no-fault-eviction-requirements/),
//! [ORS 90.427 / SB 608 Summary — Oregon Realtors](https://oregonrealtors.org/sites/default/files/Senate%20Bill%20608%20summary.pdf),
//! [N.J.S.A. 2A:18-61.1 — Justia](https://law.justia.com/codes/new-jersey/title-2a/section-2a-18-61-1/),
//! [N.J.S.A. 2A:18-61.6 owner wrongful eviction liability — Justia](https://law.justia.com/codes/new-jersey/title-2a/section-2a-18-61-6/),
//! [NYC Rent Stabilization Code overview — RGB](https://rentguidelinesboard.cityofnewyork.us/resources/faqs/rent-stabilization/).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerMoveInRegime {
    CaliforniaSb567Strict,
    OregonSb608Combined,
    NewJerseyTripleDamagesGoodFaith,
    NewYorkRentStabilizedOnlyOneUnit,
    NoStateOwnerMoveInRestriction,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: OwnerMoveInRegime,
    /// Days the owner must move in by after tenant vacates.
    pub move_in_deadline_days: u32,
    /// Months the owner must reside before failing the good-faith
    /// occupancy requirement.
    pub minimum_owner_residency_months: u32,
    /// True if statute affirmatively requires landlord to pay
    /// relocation assistance (typically 1 month's rent).
    pub relocation_assistance_required: bool,
    /// Multiplier on tenant damages for bad-faith failure to occupy
    /// (NJ has 3×; other regimes either have no multiplier or use
    /// alternative remedy mechanism).
    pub bad_faith_damages_multiplier: u32,
    pub citation: &'static str,
}

const fn rule(
    regime: OwnerMoveInRegime,
    move_in_deadline_days: u32,
    minimum_owner_residency_months: u32,
    relocation_assistance_required: bool,
    bad_faith_damages_multiplier: u32,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        move_in_deadline_days,
        minimum_owner_residency_months,
        relocation_assistance_required,
        bad_faith_damages_multiplier,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use OwnerMoveInRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "CA",
        rule(
            CaliforniaSb567Strict,
            90,
            12,
            true,
            0,
            "Cal. Civ. Code § 1946.2 as amended by SB 567 (eff. 2024-04-01) — owner / qualified family member (spouse, domestic partner, parent, child, grandchild, grandparent) must move in within 90 days + reside as primary residence ≥ 12 continuous months + pay relocation assistance (typically 1 month's rent)",
        ),
    );

    m.insert(
        "OR",
        rule(
            OregonSb608Combined,
            0,
            0,
            true,
            0,
            "ORS 90.427 as amended by SB 608 (eff. 2019-02-28) — month-to-month tenancies over 1 year: 90-day notice + 1 month rent relocation assistance required for qualifying landlord-based reasons including owner move-in",
        ),
    );

    m.insert(
        "NJ",
        rule(
            NewJerseyTripleDamagesGoodFaith,
            0,
            6,
            false,
            3,
            "N.J.S.A. 2A:18-61.1(l)(3) — owner of 3-or-fewer-unit building may evict for personal occupancy; § 2A:18-61.6: arbitrarily failing to personally occupy ≥ 6 months exposes owner to liability of 3× tenant damages plus attorney fees and costs",
        ),
    );

    m.insert(
        "NY",
        rule(
            NewYorkRentStabilizedOnlyOneUnit,
            0,
            36,
            false,
            0,
            "NYC Rent Stabilization Code § 2524.4 — only one individual owner per building may recover only one rent-stabilized unit for personal / immediate-family use; requires immediate and compelling need + 3-year principal-residence use; failure = loss of rent increases on other building units for 3 years",
        ),
    );

    // NoStateOwnerMoveInRestriction default — 46 other states + DC.
    let no_state = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA",
        "KS", "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NM",
        "NC", "ND", "OH", "OK", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV",
        "WI", "WY",
    ];
    for code in no_state {
        m.insert(
            code,
            rule(
                NoStateOwnerMoveInRestriction,
                0,
                0,
                false,
                0,
                "No statewide statutory owner-move-in restriction confirmed; applicable state landlord-tenant act governs; many cities maintain local OMI ordinances (SF Rent Ordinance § 37.9(a)(8), Seattle SMC 22.206, etc.)",
            ),
        );
    }

    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerMoveInInput {
    pub state_code: String,
    /// Whether the unit is rent-stabilized (NY regime only applies
    /// to rent-stabilized units).
    pub unit_is_rent_stabilized: bool,
    /// Number of dwelling units in the building (NJ restricts OMI
    /// to buildings of 3 units or fewer).
    pub building_unit_count: u32,
    /// True if the proposed occupant qualifies as owner or
    /// qualified family member under the state's regime (CA: 6
    /// family relations; NJ: owner only; NY: owner / immediate
    /// family).
    pub occupant_qualifies_as_owner_or_family: bool,
    /// Days elapsed since tenant vacated until owner move-in.
    pub days_until_owner_moves_in: u32,
    /// Months the owner has continuously resided in the unit after
    /// move-in.
    pub months_owner_resides_after_moving_in: u32,
    /// True if landlord paid the required relocation assistance.
    pub relocation_assistance_paid: bool,
    /// True if landlord provided the required notice period (90
    /// days under CA SB 567 + OR SB 608).
    pub written_notice_provided_with_required_days: bool,
    /// Tenant's actual damages for bad-faith failure to occupy
    /// (used for NJ 3× damages computation).
    pub tenant_actual_damages_dollars: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerMoveInResult {
    pub regime: OwnerMoveInRegime,
    pub owner_move_in_allowed_on_facts: bool,
    pub landlord_compliant: bool,
    /// True if tenant has a bad-faith / good-faith failure remedy
    /// available on these facts.
    pub tenant_remedy_available: bool,
    /// Multiplier × tenant_actual_damages (e.g., NJ 3× damages).
    pub tenant_damages_award_dollars: i64,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &OwnerMoveInInput) -> OwnerMoveInResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: OwnerMoveInRegime::NoStateOwnerMoveInRestriction,
        move_in_deadline_days: 0,
        minimum_owner_residency_months: 0,
        relocation_assistance_required: false,
        bad_faith_damages_multiplier: 0,
        citation: "Unknown state code; no statewide OMI restriction assumed",
    });

    let (allowed, compliant, remedy_available, damages) = match rule.regime {
        OwnerMoveInRegime::CaliforniaSb567Strict => {
            // Allowed if occupant qualifies. Compliant if all 4
            // requirements met: notice + 90-day move-in + 12-month
            // residency + relocation assistance.
            let allowed_now = input.occupant_qualifies_as_owner_or_family;
            let compliant_now = allowed_now
                && input.written_notice_provided_with_required_days
                && input.days_until_owner_moves_in <= rule.move_in_deadline_days
                && input.months_owner_resides_after_moving_in
                    >= rule.minimum_owner_residency_months
                && input.relocation_assistance_paid;
            let remedy = !compliant_now && allowed_now;
            // CA uses statutory § 1946.2(g) civil penalty + actual
            // damages — no statute-fixed multiplier, modeled as
            // actual damages here.
            let award = if remedy {
                input.tenant_actual_damages_dollars.max(0)
            } else {
                0
            };
            (allowed_now, compliant_now, remedy, award)
        }
        OwnerMoveInRegime::OregonSb608Combined => {
            // OR: 90-day notice + 1 month relocation = compliance.
            // Occupant qualification baseline assumed.
            let allowed_now = input.occupant_qualifies_as_owner_or_family;
            let compliant_now = allowed_now
                && input.written_notice_provided_with_required_days
                && input.relocation_assistance_paid;
            let remedy = !compliant_now && allowed_now;
            let award = if remedy {
                input.tenant_actual_damages_dollars.max(0)
            } else {
                0
            };
            (allowed_now, compliant_now, remedy, award)
        }
        OwnerMoveInRegime::NewJerseyTripleDamagesGoodFaith => {
            // NJ: must be ≤ 3-unit building + owner qualifies. 6
            // months residency triggers good-faith presumption.
            // Failure = 3× damages + attorney fees.
            let building_size_ok = input.building_unit_count <= 3;
            let allowed_now = building_size_ok && input.occupant_qualifies_as_owner_or_family;
            let residency_satisfied =
                input.months_owner_resides_after_moving_in >= rule.minimum_owner_residency_months;
            let compliant_now = allowed_now && residency_satisfied;
            let remedy = allowed_now && !residency_satisfied;
            let award = if remedy {
                input.tenant_actual_damages_dollars.max(0)
                    * (rule.bad_faith_damages_multiplier as i64)
            } else {
                0
            };
            (allowed_now, compliant_now, remedy, award)
        }
        OwnerMoveInRegime::NewYorkRentStabilizedOnlyOneUnit => {
            // NY: applies only to rent-stabilized units. Owner /
            // immediate family + 3-year residency. Non-compliance
            // = 3-year rent-increase lockout on other building
            // units (not modeled as $ damages here).
            if !input.unit_is_rent_stabilized {
                (true, true, false, 0)
            } else {
                let allowed_now = input.occupant_qualifies_as_owner_or_family;
                let residency_satisfied = input.months_owner_resides_after_moving_in
                    >= rule.minimum_owner_residency_months;
                let compliant_now = allowed_now && residency_satisfied;
                let remedy = allowed_now && !residency_satisfied;
                (allowed_now, compliant_now, remedy, 0)
            }
        }
        OwnerMoveInRegime::NoStateOwnerMoveInRestriction => (true, true, false, 0),
    };

    let regime_label = match rule.regime {
        OwnerMoveInRegime::CaliforniaSb567Strict => "California SB 567 strict OMI",
        OwnerMoveInRegime::OregonSb608Combined => "Oregon SB 608 combined notice + relocation",
        OwnerMoveInRegime::NewJerseyTripleDamagesGoodFaith => {
            "New Jersey 3× damages bad-faith remedy"
        }
        OwnerMoveInRegime::NewYorkRentStabilizedOnlyOneUnit => {
            "New York rent-stabilized 3-year residency"
        }
        OwnerMoveInRegime::NoStateOwnerMoveInRestriction => "no statewide OMI restriction",
    };

    let note = if !allowed {
        format!(
            "State applies {} regime; OMI NOT permitted on these facts (occupant does not qualify / building size disqualifies).",
            regime_label,
        )
    } else if compliant {
        format!(
            "State applies {} regime; landlord compliant on these facts.",
            regime_label,
        )
    } else {
        format!(
            "State applies {} regime; landlord NON-COMPLIANT — tenant remedy available{}.",
            regime_label,
            if damages > 0 {
                format!(" with ${} statutory damages award", damages)
            } else {
                String::new()
            },
        )
    };

    OwnerMoveInResult {
        regime: rule.regime,
        owner_move_in_allowed_on_facts: allowed,
        landlord_compliant: compliant,
        tenant_remedy_available: remedy_available,
        tenant_damages_award_dollars: damages,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline(state: &str) -> OwnerMoveInInput {
        OwnerMoveInInput {
            state_code: state.to_string(),
            unit_is_rent_stabilized: false,
            building_unit_count: 3,
            occupant_qualifies_as_owner_or_family: true,
            days_until_owner_moves_in: 45,
            months_owner_resides_after_moving_in: 24,
            relocation_assistance_paid: true,
            written_notice_provided_with_required_days: true,
            tenant_actual_damages_dollars: 10_000,
        }
    }

    // ── Regime classification ───────────────────────────────────────

    #[test]
    fn ca_sb567_regime() {
        let r = check(&baseline("CA"));
        assert_eq!(r.regime, OwnerMoveInRegime::CaliforniaSb567Strict);
    }

    #[test]
    fn or_sb608_regime() {
        let r = check(&baseline("OR"));
        assert_eq!(r.regime, OwnerMoveInRegime::OregonSb608Combined);
    }

    #[test]
    fn nj_triple_damages_regime() {
        let r = check(&baseline("NJ"));
        assert_eq!(r.regime, OwnerMoveInRegime::NewJerseyTripleDamagesGoodFaith);
    }

    #[test]
    fn ny_rent_stabilized_regime() {
        let r = check(&baseline("NY"));
        assert_eq!(
            r.regime,
            OwnerMoveInRegime::NewYorkRentStabilizedOnlyOneUnit
        );
    }

    #[test]
    fn default_state_no_restriction_regime() {
        for s in ["AL", "FL", "TX", "WA", "DC", "WY"] {
            let r = check(&baseline(s));
            assert_eq!(
                r.regime,
                OwnerMoveInRegime::NoStateOwnerMoveInRestriction,
                "expected {s} no-state regime"
            );
        }
    }

    // ── CA SB 567 — 4-prong compliance ─────────────────────────────

    #[test]
    fn ca_all_4_requirements_met_compliant() {
        let r = check(&baseline("CA"));
        assert!(r.owner_move_in_allowed_on_facts);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ca_late_move_in_91_days_non_compliant() {
        let mut i = baseline("CA");
        i.days_until_owner_moves_in = 91;
        let r = check(&i);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn ca_90_day_exact_move_in_compliant() {
        let mut i = baseline("CA");
        i.days_until_owner_moves_in = 90;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ca_residency_under_12_months_non_compliant() {
        let mut i = baseline("CA");
        i.months_owner_resides_after_moving_in = 11;
        let r = check(&i);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn ca_no_relocation_assistance_non_compliant() {
        let mut i = baseline("CA");
        i.relocation_assistance_paid = false;
        let r = check(&i);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn ca_non_qualifying_occupant_omi_not_permitted() {
        let mut i = baseline("CA");
        i.occupant_qualifies_as_owner_or_family = false;
        let r = check(&i);
        assert!(!r.owner_move_in_allowed_on_facts);
    }

    // ── OR SB 608 — notice + relocation ────────────────────────────

    #[test]
    fn or_notice_and_relocation_compliant() {
        let r = check(&baseline("OR"));
        assert!(r.landlord_compliant);
    }

    #[test]
    fn or_no_notice_non_compliant() {
        let mut i = baseline("OR");
        i.written_notice_provided_with_required_days = false;
        let r = check(&i);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn or_no_relocation_non_compliant() {
        let mut i = baseline("OR");
        i.relocation_assistance_paid = false;
        let r = check(&i);
        assert!(!r.landlord_compliant);
    }

    // ── NJ 3× damages bad-faith ────────────────────────────────────

    #[test]
    fn nj_3_unit_building_owner_qualifies_omi_permitted() {
        let mut i = baseline("NJ");
        i.building_unit_count = 3;
        let r = check(&i);
        assert!(r.owner_move_in_allowed_on_facts);
    }

    #[test]
    fn nj_4_unit_building_omi_not_permitted() {
        let mut i = baseline("NJ");
        i.building_unit_count = 4;
        let r = check(&i);
        assert!(
            !r.owner_move_in_allowed_on_facts,
            "NJ 2A:18-61.1(l)(3) restricts OMI to ≤ 3-unit buildings"
        );
    }

    #[test]
    fn nj_6_month_residency_satisfied_compliant() {
        let mut i = baseline("NJ");
        i.months_owner_resides_after_moving_in = 6;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn nj_5_month_residency_triggers_triple_damages() {
        let mut i = baseline("NJ");
        i.months_owner_resides_after_moving_in = 5;
        i.tenant_actual_damages_dollars = 10_000;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(r.tenant_remedy_available);
        // 3 × $10k = $30k.
        assert_eq!(r.tenant_damages_award_dollars, 30_000);
    }

    #[test]
    fn nj_large_actual_damages_3x_no_precision_loss() {
        let mut i = baseline("NJ");
        i.months_owner_resides_after_moving_in = 0;
        i.tenant_actual_damages_dollars = 100_000_000;
        let r = check(&i);
        assert_eq!(r.tenant_damages_award_dollars, 300_000_000);
    }

    // ── NY rent-stabilized 3-year residency ────────────────────────

    #[test]
    fn ny_non_rent_stabilized_no_restriction() {
        let mut i = baseline("NY");
        i.unit_is_rent_stabilized = false;
        let r = check(&i);
        assert!(r.owner_move_in_allowed_on_facts);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ny_rent_stabilized_36_month_residency_compliant() {
        let mut i = baseline("NY");
        i.unit_is_rent_stabilized = true;
        i.months_owner_resides_after_moving_in = 36;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ny_rent_stabilized_35_month_residency_non_compliant() {
        let mut i = baseline("NY");
        i.unit_is_rent_stabilized = true;
        i.months_owner_resides_after_moving_in = 35;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(r.tenant_remedy_available);
    }

    // ── Default state ──────────────────────────────────────────────

    #[test]
    fn default_state_omi_allowed_no_restriction() {
        let r = check(&baseline("FL"));
        assert!(r.owner_move_in_allowed_on_facts);
        assert!(r.landlord_compliant);
    }

    // ── Citation contents ──────────────────────────────────────────

    #[test]
    fn ca_citation_mentions_sb_567_and_90_days() {
        let r = check(&baseline("CA"));
        assert!(r.citation.contains("SB 567"));
        assert!(r.citation.contains("90 days"));
        assert!(r.citation.contains("12 continuous months"));
    }

    #[test]
    fn or_citation_mentions_sb_608_and_90_day() {
        let r = check(&baseline("OR"));
        assert!(r.citation.contains("SB 608"));
        assert!(r.citation.contains("90-day notice"));
    }

    #[test]
    fn nj_citation_mentions_2a_18_61_and_3x_damages() {
        let r = check(&baseline("NJ"));
        assert!(r.citation.contains("2A:18-61.1(l)(3)"));
        assert!(r.citation.contains("2A:18-61.6"));
        assert!(r.citation.contains("3× tenant damages"));
    }

    #[test]
    fn ny_citation_mentions_rsc_2524_4_and_3_year() {
        let r = check(&baseline("NY"));
        assert!(r.citation.contains("§ 2524.4"));
        assert!(r.citation.contains("3-year"));
    }

    // ── Coverage / single-state-uniqueness invariants ─────────────

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
    fn ca_only_sb567_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| matches!(r.regime, OwnerMoveInRegime::CaliforniaSb567Strict))
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn or_only_sb608_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| matches!(r.regime, OwnerMoveInRegime::OregonSb608Combined))
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn nj_only_triple_damages_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| matches!(r.regime, OwnerMoveInRegime::NewJerseyTripleDamagesGoodFaith))
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn ny_only_rent_stabilized_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    OwnerMoveInRegime::NewYorkRentStabilizedOnlyOneUnit
                )
            })
            .count();
        assert_eq!(count, 1);
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn ca_non_compliant_note_mentions_regime() {
        let mut i = baseline("CA");
        i.days_until_owner_moves_in = 200;
        let r = check(&i);
        assert!(r.note.contains("California SB 567 strict OMI"));
        assert!(r.note.contains("NON-COMPLIANT"));
    }

    #[test]
    fn nj_remedy_note_mentions_damages_award() {
        let mut i = baseline("NJ");
        i.months_owner_resides_after_moving_in = 0;
        i.tenant_actual_damages_dollars = 5_000;
        let r = check(&i);
        // 3 × $5k = $15k.
        assert!(r.note.contains("$15000"));
    }

    // ── State-code normalization ──────────────────────────────────

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&baseline("ca"));
        assert_eq!(r.regime, OwnerMoveInRegime::CaliforniaSb567Strict);
    }
}
