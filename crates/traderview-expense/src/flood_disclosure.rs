//! State flood-zone / flood-history disclosure compliance.
//!
//! As of 2025, **nine U.S. states** have enacted statutory pre-lease
//! flood disclosure requirements for residential rental property:
//! California, Florida, Georgia, Indiana, New Jersey, New York,
//! Oklahoma, Oregon, and Texas. Oklahoma (1986) and Georgia (1995)
//! were first; the rest came in waves between 2018 and 2025. The
//! remaining 41 states + DC have no statutory pre-lease flood
//! disclosure rule — though common-law fraud / misrepresentation
//! liability still applies when a landlord knowingly conceals a
//! known prior flood that materially affects habitability.
//!
//! Five regimes:
//!
//! `FloridaFloodHistoryClaimsFemaAid`: FL only. Fla. Stat. § 83.512
//! eff. 2025-10-01 (SB 948). Landlord must provide a separate flood
//! disclosure form before or at lease signing for any lease ≥ 1 year
//! covering: prior flood events during landlord's ownership,
//! insurance claims filed for flood damage, and any FEMA / federal
//! flood assistance received. Tenant remedy on noncompliance:
//! lease termination within 30 days of a "substantial loss" (defined
//! as 50% or more of the tenant's personal property market value),
//! plus recovery of prepaid rent for periods after termination.
//!
//! `NewJerseyFemaFloodZoneAndHistory`: NJ only. N.J.S.A. 46:8-50
//! (P.L. 2023, c. 93) eff. 2024-03-20 for new/renewed leases. Both
//! seller AND landlord disclosure required. Landlord must disclose
//! whether property is located in a FEMA Special Flood Hazard Area
//! (SFHA) or Moderate Risk Flood Hazard Area, AND any known prior
//! flooding. Tenant remedy on noncompliance + property in flood
//! zone: immediate termination without penalty + refund of all
//! prepaid rent and other amounts paid for periods after lease
//! effective date, refunded within 30 days of surrender. Exempt:
//! seasonal / transient rentals < 120 days.
//!
//! `CaliforniaNaturalHazardCombined`: CA only. Cal. Gov. Code
//! § 8589.45 (Special Flood Hazard Area + dam inundation zone
//! disclosure) eff. 2018-07-01, paired with Civ. Code § 1102.17 for
//! sales. Landlord must disclose in writing if property is in an
//! SFHA per FEMA flood maps or in a state-designated dam inundation
//! zone before lease execution. Cluster of natural-hazard disclosures
//! also covers wildfire / very high fire hazard severity zone.
//!
//! `PriorFloodKnowledgeDisclosure`: TX, NY, GA, IN, OK, OR. Six
//! states require landlord to disclose only ACTUAL KNOWLEDGE of
//! prior flood events:
//!   - TX Prop. Code § 92.0135 (eff. 2022-01-01)
//!   - NY RPL § 231-b (Flood History and Risk Notice)
//!   - GA O.C.G.A. § 44-7-20 (the first such law, eff. 1995)
//!   - IN Ind. Code § 32-31-1-21
//!   - OK Okla. Stat. tit. 41 § 113a (eff. 1986)
//!   - OR ORS 90.228
//!
//! `NoStateFloodDisclosure`: 41 other states + DC. No statutory
//! pre-lease flood disclosure rule; common-law fraud / negligent
//! misrepresentation remains available if landlord conceals known
//! material defect.
//!
//! Sources:
//! [FL Stat. § 83.512 (SB 948)](https://www.sjlawgroup.com/florida-flood-disclosure-form/),
//! [NJ N.J.S.A. 46:8-50 (P.L. 2023, c. 93)](https://dep.nj.gov/flooddisclosure/),
//! [Holland & Knight — NJ flood disclosure](https://www.hklaw.com/en/insights/publications/2024/04/nj-legislature-expands-real-property-owners),
//! [FEMA — State Flood Risk Disclosure Best Practices](https://www.fema.gov/sites/default/files/documents/fema_state-flood-risk-disclosure-best-practices_07142022.pdf),
//! [Tenant-Rights.com — CA flood/natural hazard disclosure](https://tenant-rights.com/california/flood-zone-and-natural-hazard-disclosures-for-renters),
//! [NY RPL § 231-b](https://law.justia.com/codes/new-york/rpp/article-7/231-b/).

use chrono::NaiveDate;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FloodDisclosureRegime {
    FloridaFloodHistoryClaimsFemaAid,
    NewJerseyFemaFloodZoneAndHistory,
    CaliforniaNaturalHazardCombined,
    PriorFloodKnowledgeDisclosure,
    NoStateFloodDisclosure,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: FloodDisclosureRegime,
    /// Lease-signing-date floor: disclosure law only applies to
    /// leases entered/renewed on or after this date. `None` for
    /// regimes that have no effective-date gate.
    pub effective_date: Option<NaiveDate>,
    /// Tenant remedy on noncompliance.
    pub tenant_termination_window_days: u32,
    /// True if the noncompliance remedy requires the tenant to
    /// suffer a "substantial loss" (FL only: 50%+ of personal
    /// property market value).
    pub substantial_loss_required: bool,
    /// True if tenant is entitled to refund of prepaid rent on
    /// noncompliance + flood-zone status.
    pub refund_of_prepaid_required: bool,
    pub citation: &'static str,
}

const fn rule(
    regime: FloodDisclosureRegime,
    effective_date: Option<NaiveDate>,
    tenant_termination_window_days: u32,
    substantial_loss_required: bool,
    refund_of_prepaid_required: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        effective_date,
        tenant_termination_window_days,
        substantial_loss_required,
        refund_of_prepaid_required,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use FloodDisclosureRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "FL",
        rule(
            FloridaFloodHistoryClaimsFemaAid,
            NaiveDate::from_ymd_opt(2025, 10, 1),
            30,
            true,
            true,
            "Fla. Stat. § 83.512 (SB 948, eff. 2025-10-01) — landlord must disclose prior flooding, insurance claims, and FEMA assistance via separate form at lease signing for leases ≥ 1 year; tenant may terminate within 30 days of substantial loss (50%+ market value of personal property) + recover prepaid rent",
        ),
    );

    m.insert(
        "NJ",
        rule(
            NewJerseyFemaFloodZoneAndHistory,
            NaiveDate::from_ymd_opt(2024, 3, 20),
            0,
            false,
            true,
            "N.J.S.A. 46:8-50 (P.L. 2023, c. 93, eff. 2024-03-20) — landlord must disclose FEMA SFHA / Moderate Risk Flood Hazard Area status AND known prior flooding for leases entered/renewed on/after 2024-03-20; tenant may terminate IMMEDIATELY without penalty + recover all prepaid amounts within 30 days of surrender if undisclosed property is in flood zone; seasonal/transient < 120 days exempt",
        ),
    );

    m.insert(
        "CA",
        rule(
            CaliforniaNaturalHazardCombined,
            NaiveDate::from_ymd_opt(2018, 7, 1),
            0,
            false,
            false,
            "Cal. Gov. Code § 8589.45 (eff. 2018-07-01) — landlord must disclose in writing before lease execution if property is in FEMA Special Flood Hazard Area or state-designated dam inundation zone; paired with Civ. Code § 1102.17 (sales) and natural hazard cluster (flood + wildfire / VHFHSZ + seismic)",
        ),
    );

    // PriorFloodKnowledgeDisclosure — 6 states.
    m.insert(
        "TX",
        rule(
            PriorFloodKnowledgeDisclosure,
            NaiveDate::from_ymd_opt(2022, 1, 1),
            0,
            false,
            false,
            "Tex. Prop. Code § 92.0135 (eff. 2022-01-01) — landlord must disclose actual knowledge of prior flooding in lease; no remedy beyond statutory non-disclosure damages",
        ),
    );
    m.insert(
        "NY",
        rule(
            PriorFloodKnowledgeDisclosure,
            None,
            0,
            false,
            false,
            "N.Y. RPL § 231-b (Flood History and Risk Notice in Residential Leases) — landlord must include flood history and risk notice in lease; covers prior flooding and known SFHA status",
        ),
    );
    m.insert(
        "GA",
        rule(
            PriorFloodKnowledgeDisclosure,
            None,
            0,
            false,
            false,
            "O.C.G.A. § 44-7-20 (eff. 1995, the first U.S. tenant flood disclosure statute) — landlord must disclose if property is in 100-year FEMA flood plain and has flooded at least 3 times in past 5 years",
        ),
    );
    m.insert(
        "IN",
        rule(
            PriorFloodKnowledgeDisclosure,
            None,
            0,
            false,
            false,
            "Ind. Code § 32-31-1-21 — landlord must disclose actual knowledge of flood hazard area status or prior flooding affecting the rental",
        ),
    );
    m.insert(
        "OK",
        rule(
            PriorFloodKnowledgeDisclosure,
            None,
            0,
            false,
            false,
            "Okla. Stat. tit. 41 § 113a (eff. 1986, earliest U.S. tenant flood disclosure statute) — landlord must disclose if rental has flooded in past 5 years",
        ),
    );
    m.insert(
        "OR",
        rule(
            PriorFloodKnowledgeDisclosure,
            None,
            0,
            false,
            false,
            "ORS 90.228 — landlord must disclose actual knowledge that rental is in a 100-year flood plain prior to lease execution",
        ),
    );

    // NoStateFloodDisclosure default — 41 states + DC.
    let no_state = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE",
        "HI", "ID", "IL", "IA", "KS", "KY", "LA", "ME",
        "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE",
        "NV", "NH", "NM", "NC", "ND", "OH", "PA", "RI",
        "SC", "SD", "TN", "UT", "VT", "VA", "WA", "WV",
        "WI", "WY",
    ];
    for code in no_state {
        m.insert(
            code,
            rule(
                NoStateFloodDisclosure,
                None,
                0,
                false,
                false,
                "No statutory pre-lease flood disclosure requirement; common-law fraud / negligent misrepresentation remains available if landlord knowingly conceals material defect",
            ),
        );
    }

    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloodDisclosureInput {
    pub state_code: String,
    pub lease_signing_date: NaiveDate,
    pub lease_term_months: u32,
    /// Whether the landlord provided the statutorily required
    /// flood disclosure at or before lease signing.
    pub landlord_provided_disclosure: bool,
    /// Whether the rental is located in a FEMA Special Flood Hazard
    /// Area or Moderate Risk Flood Hazard Area at lease signing.
    pub property_in_flood_zone: bool,
    /// Whether the rental had a prior flood event known to the
    /// landlord at lease signing.
    pub prior_flood_known_to_landlord: bool,
    /// FL-specific: whether the tenant has suffered a "substantial
    /// loss" defined as 50%+ of personal property market value.
    pub tenant_substantial_loss_suffered: bool,
    /// Tenant prepaid rent dollars at issue for refund computation.
    pub prepaid_rent_dollars: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloodDisclosureResult {
    pub regime: FloodDisclosureRegime,
    /// True if the state's flood disclosure law was in effect on the
    /// lease signing date.
    pub statute_in_effect_on_signing: bool,
    /// True if the landlord was required to provide disclosure.
    pub disclosure_required: bool,
    /// True if the landlord is compliant (either no requirement, or
    /// requirement met).
    pub landlord_compliant: bool,
    pub tenant_may_terminate: bool,
    pub tenant_termination_window_days: u32,
    pub tenant_refund_dollars: i64,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &FloodDisclosureInput) -> FloodDisclosureResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: FloodDisclosureRegime::NoStateFloodDisclosure,
        effective_date: None,
        tenant_termination_window_days: 0,
        substantial_loss_required: false,
        refund_of_prepaid_required: false,
        citation: "Unknown state code; no statutory pre-lease flood disclosure assumed",
    });

    // Effective-date gate.
    let in_effect = match rule.effective_date {
        Some(eff) => input.lease_signing_date >= eff,
        None => !matches!(rule.regime, FloodDisclosureRegime::NoStateFloodDisclosure),
    };

    // FL-specific: only applies to leases ≥ 12 months.
    let fl_term_ok =
        matches!(rule.regime, FloodDisclosureRegime::FloridaFloodHistoryClaimsFemaAid)
            && input.lease_term_months >= 12;
    let non_fl_term_ok = !matches!(
        rule.regime,
        FloodDisclosureRegime::FloridaFloodHistoryClaimsFemaAid
    );

    let disclosure_required = in_effect && (fl_term_ok || non_fl_term_ok);

    // Compliance: did landlord disclose when required?
    let landlord_compliant = !disclosure_required || input.landlord_provided_disclosure;

    // Tenant remedy availability by regime.
    let (may_terminate, refund) = if landlord_compliant {
        (false, 0i64)
    } else {
        match rule.regime {
            FloodDisclosureRegime::FloridaFloodHistoryClaimsFemaAid => {
                // FL: terminate only if substantial loss suffered.
                if input.tenant_substantial_loss_suffered {
                    (true, input.prepaid_rent_dollars.max(0))
                } else {
                    (false, 0)
                }
            }
            FloodDisclosureRegime::NewJerseyFemaFloodZoneAndHistory => {
                // NJ: immediate termination + refund only if property
                // was in fact in flood zone OR there was a prior flood.
                if input.property_in_flood_zone || input.prior_flood_known_to_landlord {
                    (true, input.prepaid_rent_dollars.max(0))
                } else {
                    (false, 0)
                }
            }
            FloodDisclosureRegime::CaliforniaNaturalHazardCombined => {
                // CA: civil damages but no statutory immediate-termination
                // remedy; tenant may rescind under common-law fraud only
                // if material concealment.
                (false, 0)
            }
            FloodDisclosureRegime::PriorFloodKnowledgeDisclosure => {
                // Prior-knowledge states: damages remedy, no statutory
                // termination right (GA has narrow exception not modeled).
                (false, 0)
            }
            FloodDisclosureRegime::NoStateFloodDisclosure => (false, 0),
        }
    };

    let regime_label = match rule.regime {
        FloodDisclosureRegime::FloridaFloodHistoryClaimsFemaAid => {
            "Florida flood-history + claims + FEMA aid disclosure"
        }
        FloodDisclosureRegime::NewJerseyFemaFloodZoneAndHistory => {
            "New Jersey FEMA flood-zone + history disclosure"
        }
        FloodDisclosureRegime::CaliforniaNaturalHazardCombined => {
            "California natural-hazard cluster disclosure"
        }
        FloodDisclosureRegime::PriorFloodKnowledgeDisclosure => {
            "prior-flood-knowledge disclosure"
        }
        FloodDisclosureRegime::NoStateFloodDisclosure => {
            "no statutory pre-lease flood disclosure"
        }
    };

    let note = if !in_effect
        && !matches!(rule.regime, FloodDisclosureRegime::NoStateFloodDisclosure)
    {
        format!(
            "State has {} regime, but statute not yet in effect on lease signing date {}; common-law fraud / misrepresentation remains.",
            regime_label, input.lease_signing_date,
        )
    } else if landlord_compliant {
        format!(
            "State applies {}. Landlord compliant — no tenant termination remedy triggered.",
            regime_label,
        )
    } else if may_terminate {
        format!(
            "State applies {}. Landlord NON-COMPLIANT: failed to provide required disclosure. Tenant may terminate{} and recover prepaid rent ${}.",
            regime_label,
            if rule.tenant_termination_window_days > 0 {
                format!(" within {} days", rule.tenant_termination_window_days)
            } else {
                " immediately without penalty".to_string()
            },
            refund,
        )
    } else {
        format!(
            "State applies {}. Landlord NON-COMPLIANT, but threshold for tenant termination remedy not met (no flood zone / no substantial loss).",
            regime_label,
        )
    };

    FloodDisclosureResult {
        regime: rule.regime,
        statute_in_effect_on_signing: in_effect,
        disclosure_required,
        landlord_compliant,
        tenant_may_terminate: may_terminate,
        tenant_termination_window_days: if may_terminate {
            rule.tenant_termination_window_days
        } else {
            0
        },
        tenant_refund_dollars: refund,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline(state: &str) -> FloodDisclosureInput {
        FloodDisclosureInput {
            state_code: state.to_string(),
            lease_signing_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            lease_term_months: 12,
            landlord_provided_disclosure: true,
            property_in_flood_zone: false,
            prior_flood_known_to_landlord: false,
            tenant_substantial_loss_suffered: false,
            prepaid_rent_dollars: 6_000,
        }
    }

    // ── Regime classification ───────────────────────────────────────

    #[test]
    fn fl_regime_classification() {
        let r = check(&baseline("FL"));
        assert_eq!(
            r.regime,
            FloodDisclosureRegime::FloridaFloodHistoryClaimsFemaAid
        );
    }

    #[test]
    fn nj_regime_classification() {
        let r = check(&baseline("NJ"));
        assert_eq!(
            r.regime,
            FloodDisclosureRegime::NewJerseyFemaFloodZoneAndHistory
        );
    }

    #[test]
    fn ca_regime_classification() {
        let r = check(&baseline("CA"));
        assert_eq!(
            r.regime,
            FloodDisclosureRegime::CaliforniaNaturalHazardCombined
        );
    }

    #[test]
    fn tx_prior_flood_knowledge_regime() {
        let r = check(&baseline("TX"));
        assert_eq!(
            r.regime,
            FloodDisclosureRegime::PriorFloodKnowledgeDisclosure
        );
    }

    #[test]
    fn ny_prior_flood_knowledge_regime() {
        let r = check(&baseline("NY"));
        assert_eq!(
            r.regime,
            FloodDisclosureRegime::PriorFloodKnowledgeDisclosure
        );
    }

    #[test]
    fn ga_in_ok_or_all_prior_flood_knowledge() {
        for s in ["GA", "IN", "OK", "OR"] {
            let r = check(&baseline(s));
            assert_eq!(
                r.regime,
                FloodDisclosureRegime::PriorFloodKnowledgeDisclosure,
                "expected {s} to be PriorFloodKnowledgeDisclosure"
            );
        }
    }

    #[test]
    fn default_state_no_disclosure_regime() {
        for s in ["AL", "KS", "DC", "WY"] {
            let r = check(&baseline(s));
            assert_eq!(
                r.regime,
                FloodDisclosureRegime::NoStateFloodDisclosure,
                "expected {s} to be NoStateFloodDisclosure"
            );
        }
    }

    // ── Effective-date gates ────────────────────────────────────────

    #[test]
    fn fl_pre_effective_date_not_yet_required() {
        // Lease signed 2025-09-30 — one day before FL eff. date.
        let mut i = baseline("FL");
        i.lease_signing_date = NaiveDate::from_ymd_opt(2025, 9, 30).unwrap();
        i.landlord_provided_disclosure = false;
        let r = check(&i);
        assert!(!r.statute_in_effect_on_signing);
        assert!(!r.disclosure_required);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn fl_on_effective_date_required() {
        let mut i = baseline("FL");
        i.lease_signing_date = NaiveDate::from_ymd_opt(2025, 10, 1).unwrap();
        i.landlord_provided_disclosure = false;
        let r = check(&i);
        assert!(r.statute_in_effect_on_signing);
        assert!(r.disclosure_required);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn nj_pre_effective_date_not_required() {
        let mut i = baseline("NJ");
        i.lease_signing_date = NaiveDate::from_ymd_opt(2024, 3, 19).unwrap();
        i.landlord_provided_disclosure = false;
        let r = check(&i);
        assert!(!r.statute_in_effect_on_signing);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn nj_on_effective_date_required() {
        let mut i = baseline("NJ");
        i.lease_signing_date = NaiveDate::from_ymd_opt(2024, 3, 20).unwrap();
        i.landlord_provided_disclosure = false;
        let r = check(&i);
        assert!(r.statute_in_effect_on_signing);
        assert!(r.disclosure_required);
    }

    // ── Lease-term gate (FL 12-month minimum) ──────────────────────

    #[test]
    fn fl_month_to_month_not_subject_to_disclosure() {
        let mut i = baseline("FL");
        i.lease_term_months = 1;
        i.landlord_provided_disclosure = false;
        let r = check(&i);
        assert!(r.statute_in_effect_on_signing);
        assert!(!r.disclosure_required, "FL only applies to leases ≥ 12 months");
        assert!(r.landlord_compliant);
    }

    #[test]
    fn fl_exactly_12_month_lease_subject_to_disclosure() {
        let mut i = baseline("FL");
        i.lease_term_months = 12;
        i.landlord_provided_disclosure = false;
        let r = check(&i);
        assert!(r.disclosure_required);
    }

    // ── FL tenant remedy ────────────────────────────────────────────

    #[test]
    fn fl_noncompliance_without_substantial_loss_no_termination() {
        let mut i = baseline("FL");
        i.landlord_provided_disclosure = false;
        i.tenant_substantial_loss_suffered = false;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(!r.tenant_may_terminate, "FL requires substantial loss for termination right");
    }

    #[test]
    fn fl_noncompliance_with_substantial_loss_30_day_termination() {
        let mut i = baseline("FL");
        i.landlord_provided_disclosure = false;
        i.tenant_substantial_loss_suffered = true;
        let r = check(&i);
        assert!(r.tenant_may_terminate);
        assert_eq!(r.tenant_termination_window_days, 30);
        assert_eq!(r.tenant_refund_dollars, 6_000);
    }

    // ── NJ tenant remedy ────────────────────────────────────────────

    #[test]
    fn nj_noncompliance_in_flood_zone_immediate_termination() {
        let mut i = baseline("NJ");
        i.landlord_provided_disclosure = false;
        i.property_in_flood_zone = true;
        let r = check(&i);
        assert!(r.tenant_may_terminate);
        assert_eq!(
            r.tenant_termination_window_days, 0,
            "NJ permits immediate (no-window) termination"
        );
        assert_eq!(r.tenant_refund_dollars, 6_000);
    }

    #[test]
    fn nj_noncompliance_not_in_flood_zone_no_termination() {
        let mut i = baseline("NJ");
        i.landlord_provided_disclosure = false;
        i.property_in_flood_zone = false;
        i.prior_flood_known_to_landlord = false;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(!r.tenant_may_terminate, "NJ termination requires flood zone OR prior flood");
    }

    #[test]
    fn nj_noncompliance_prior_flood_triggers_termination() {
        // Even outside FEMA-mapped flood zone, a known prior flood
        // triggers the NJ termination remedy.
        let mut i = baseline("NJ");
        i.landlord_provided_disclosure = false;
        i.property_in_flood_zone = false;
        i.prior_flood_known_to_landlord = true;
        let r = check(&i);
        assert!(r.tenant_may_terminate);
    }

    // ── CA / prior-knowledge: damages only, no termination ─────────

    #[test]
    fn ca_noncompliance_no_statutory_termination() {
        let mut i = baseline("CA");
        i.landlord_provided_disclosure = false;
        i.property_in_flood_zone = true;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(
            !r.tenant_may_terminate,
            "CA has no statutory termination remedy — damages / common-law only"
        );
    }

    #[test]
    fn tx_noncompliance_no_statutory_termination() {
        let mut i = baseline("TX");
        i.landlord_provided_disclosure = false;
        i.prior_flood_known_to_landlord = true;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(!r.tenant_may_terminate);
    }

    // ── Citation contents ──────────────────────────────────────────

    #[test]
    fn fl_citation_mentions_sb_948_and_eff_date() {
        let r = check(&baseline("FL"));
        assert!(r.citation.contains("§ 83.512"));
        assert!(r.citation.contains("SB 948"));
        assert!(r.citation.contains("2025-10-01"));
    }

    #[test]
    fn nj_citation_mentions_pl_2023_c93_and_eff_date() {
        let r = check(&baseline("NJ"));
        assert!(r.citation.contains("46:8-50"));
        assert!(r.citation.contains("P.L. 2023, c. 93"));
        assert!(r.citation.contains("2024-03-20"));
    }

    #[test]
    fn ca_citation_mentions_gov_code_8589_45() {
        let r = check(&baseline("CA"));
        assert!(r.citation.contains("Gov. Code § 8589.45"));
    }

    #[test]
    fn tx_citation_mentions_prop_code_92_0135() {
        let r = check(&baseline("TX"));
        assert!(r.citation.contains("§ 92.0135"));
    }

    #[test]
    fn ny_citation_mentions_rpl_231_b() {
        let r = check(&baseline("NY"));
        assert!(r.citation.contains("RPL § 231-b"));
    }

    #[test]
    fn ok_citation_mentions_1986_first() {
        // OK was the first U.S. state with a tenant flood disclosure
        // statute — pinning that historical claim.
        let r = check(&baseline("OK"));
        assert!(r.citation.contains("1986"));
        assert!(r.citation.contains("earliest"));
    }

    #[test]
    fn ga_citation_mentions_3_floods_5_years() {
        let r = check(&baseline("GA"));
        assert!(r.citation.contains("3 times"));
        assert!(r.citation.contains("5 years"));
    }

    // ── Uniqueness invariants ──────────────────────────────────────

    #[test]
    fn fl_is_only_florida_regime_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(r.regime, FloodDisclosureRegime::FloridaFloodHistoryClaimsFemaAid)
            })
            .count();
        assert_eq!(count, 1, "FloridaFloodHistoryClaimsFemaAid must be FL only");
    }

    #[test]
    fn nj_is_only_new_jersey_regime_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(r.regime, FloodDisclosureRegime::NewJerseyFemaFloodZoneAndHistory)
            })
            .count();
        assert_eq!(count, 1, "NewJerseyFemaFloodZoneAndHistory must be NJ only");
    }

    #[test]
    fn ca_is_only_california_regime_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(r.regime, FloodDisclosureRegime::CaliforniaNaturalHazardCombined)
            })
            .count();
        assert_eq!(count, 1, "CaliforniaNaturalHazardCombined must be CA only");
    }

    #[test]
    fn prior_flood_knowledge_regime_has_exactly_6_states() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(r.regime, FloodDisclosureRegime::PriorFloodKnowledgeDisclosure)
            })
            .count();
        assert_eq!(
            count, 6,
            "PriorFloodKnowledgeDisclosure must be exactly 6 states: TX/NY/GA/IN/OK/OR"
        );
    }

    // ── Coverage ────────────────────────────────────────────────────

    #[test]
    fn coverage_is_all_50_states_plus_dc() {
        let count = RULES.len();
        assert_eq!(count, 51, "expected 50 states + DC, got {count}");
    }

    #[test]
    fn citations_never_empty() {
        for (code, rule) in RULES.iter() {
            assert!(!rule.citation.is_empty(), "{code} has empty citation");
        }
    }

    // ── Compliance happy path ──────────────────────────────────────

    #[test]
    fn fl_compliant_disclosure_no_remedy() {
        let mut i = baseline("FL");
        i.landlord_provided_disclosure = true;
        i.tenant_substantial_loss_suffered = true;
        let r = check(&i);
        assert!(r.landlord_compliant);
        assert!(!r.tenant_may_terminate);
        assert_eq!(r.tenant_refund_dollars, 0);
    }

    #[test]
    fn nj_compliant_disclosure_no_remedy() {
        let mut i = baseline("NJ");
        i.landlord_provided_disclosure = true;
        i.property_in_flood_zone = true;
        let r = check(&i);
        assert!(r.landlord_compliant);
        assert!(!r.tenant_may_terminate);
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn fl_pre_effective_note_explains_not_yet_in_effect() {
        let mut i = baseline("FL");
        i.lease_signing_date = NaiveDate::from_ymd_opt(2025, 9, 1).unwrap();
        i.landlord_provided_disclosure = false;
        let r = check(&i);
        assert!(r.note.contains("not yet in effect"));
    }

    #[test]
    fn nj_termination_note_says_immediately() {
        let mut i = baseline("NJ");
        i.landlord_provided_disclosure = false;
        i.property_in_flood_zone = true;
        let r = check(&i);
        assert!(r.note.contains("immediately"));
    }

    #[test]
    fn no_state_default_note_is_no_disclosure_regime() {
        let r = check(&baseline("KS"));
        assert!(r.note.contains("no statutory pre-lease flood disclosure"));
    }
}
