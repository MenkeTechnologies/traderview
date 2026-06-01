//! State landlord / owner identification + agent-for-service disclosure
//! compliance.
//!
//! Every state has SOME rule (statutory, regulatory, or common-law)
//! requiring a landlord to provide tenants with the owner's name,
//! address, and identity of any agent authorized to receive notices
//! and legal service. The statutory frameworks fall into four
//! distinct regimes:
//!
//! `AffirmativePreLeaseDisclosure`: CA, FL. Landlord must
//! proactively disclose at or near lease commencement, in writing,
//! without any tenant demand:
//!   - CA Civ. Code § 1962: 15 days from written lease execution
//!     (or 15 days from oral agreement). Disclosure must include
//!     owner / authorized recipient / manager + street address for
//!     service of process. On noncompliance, tenant may serve at
//!     the rent-payment address.
//!   - FL Fla. Stat. § 83.50: in writing at or before commencement
//!     of tenancy. Authorized recipient continues until tenant is
//!     notified otherwise.
//!
//! `DisclosureUponWrittenDemand`: TX. Tex. Prop. Code § 92.201
//! requires landlord to disclose name + street/PO-box address of
//! record-deed-title holder, plus any off-site management company
//! address, within **7 days** of tenant's written demand. Alternative
//! compliance: continuously post in conspicuous place OR include in
//! lease / written rules. § 92.202 damages: one month's rent plus
//! $100 plus attorney's fees, with termination right available after
//! second failure to disclose within 7 days.
//!
//! `MultipleDwellingRegistration`: NY, NJ, MA. Multi-unit buildings
//! must be registered with state / city / local health board, AND
//! that registration must be available to tenants:
//!   - NY: MDL § 325 (state) + NYC HMC § 27-2098 (NYC HPD
//!     registration required for 3+ units; non-registration bars
//!     non-payment summary proceedings).
//!   - NJ: N.J.S.A. 55:13A (Hotel and Multiple Dwelling Law;
//!     N.J.A.C. 5:10-1.5 registration with DCA Bureau of Housing
//!     Inspection).
//!   - MA: G.L. c. 111 § 197A (local board of health registration
//!     for 3+ dwelling units); G.L. c. 186 § 15B(7) landlord
//!     identity rule for any rental.
//!
//! `LocalLawOrCommonLawOnly`: 44 other states + DC. No statewide
//! statutory pre-lease disclosure rule confirmed. URLTA-adopting
//! states may have analogous § 2-202 provisions; many cities have
//! local landlord registration ordinances (e.g., Chicago RLTO).
//! Common-law fraud / misrepresentation applies when landlord
//! conceals identity to evade service.
//!
//! Sources:
//! [Cal. Civ. Code § 1962 (Justia)](https://law.justia.com/codes/california/code-civ/division-3/part-4/title-5/chapter-4/section-1962/),
//! [Tex. Prop. Code § 92.201 (Justia)](https://law.justia.com/codes/texas/property-code/title-8/chapter-92/subchapter-e/section-92-201/),
//! [Tex. Prop. Code § 92.202 — landlord failure to disclose](https://texas.public.law/statutes/tex._prop._code_section_92.202),
//! [Fla. Stat. § 83.50 (Justia)](https://law.justia.com/codes/florida/title-vi/chapter-83/part-ii/section-83-50/).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerIdentificationRegime {
    AffirmativePreLeaseDisclosure,
    DisclosureUponWrittenDemand,
    MultipleDwellingRegistration,
    LocalLawOrCommonLawOnly,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: OwnerIdentificationRegime,
    /// Days within which the landlord must disclose. For
    /// AffirmativePreLeaseDisclosure this is measured from lease
    /// execution; for DisclosureUponWrittenDemand from the tenant's
    /// written request. Zero means at lease commencement / immediate.
    pub disclosure_window_days: u32,
    /// Minimum unit count that triggers MultipleDwellingRegistration
    /// regime. Zero for non-registration regimes.
    pub minimum_units_for_registration: u32,
    /// True if noncompliance gives the tenant a statutory damages
    /// remedy (e.g., TX § 92.202 one month's rent + $100).
    pub statutory_damages_available: bool,
    pub citation: &'static str,
}

const fn rule(
    regime: OwnerIdentificationRegime,
    disclosure_window_days: u32,
    minimum_units_for_registration: u32,
    statutory_damages_available: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        disclosure_window_days,
        minimum_units_for_registration,
        statutory_damages_available,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use OwnerIdentificationRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "CA",
        rule(
            AffirmativePreLeaseDisclosure,
            15,
            0,
            true,
            "Cal. Civ. Code § 1962 — landlord must disclose owner / authorized agent / manager name + telephone + street address for service of process within 15 days of written lease execution (or 15 days from oral agreement); on noncompliance tenant may serve at rent-payment address per § 1962(c)",
        ),
    );

    m.insert(
        "FL",
        rule(
            AffirmativePreLeaseDisclosure,
            0,
            0,
            false,
            "Fla. Stat. § 83.50 (Residential Landlord and Tenant Act) — landlord must disclose in writing at or before commencement of tenancy: name and address of landlord or person authorized to receive notices and demands; authorized recipient continues until tenant is notified otherwise",
        ),
    );

    m.insert(
        "TX",
        rule(
            DisclosureUponWrittenDemand,
            7,
            0,
            true,
            "Tex. Prop. Code § 92.201 — landlord must disclose name + street/PO-box address of record-deed-title holder + off-site management company address within 7 days of tenant's written request, OR by continuous posting in conspicuous place, OR by inclusion in lease / written rules; § 92.202 damages = one month's rent + $100 + attorney's fees + tenant termination right after second failure",
        ),
    );

    m.insert(
        "NY",
        rule(
            MultipleDwellingRegistration,
            0,
            3,
            true,
            "N.Y. MDL § 325 (state) + NYC HMC § 27-2098 — multi-unit buildings (3+ units in NYC) must be registered with HPD; non-registration bars landlord from maintaining non-payment summary proceedings",
        ),
    );

    m.insert(
        "NJ",
        rule(
            MultipleDwellingRegistration,
            0,
            3,
            true,
            "N.J.S.A. 55:13A-12.1 (Hotel and Multiple Dwelling Law) + N.J.A.C. 5:10-1.5 — multiple-dwelling registration required with NJ DCA Bureau of Housing Inspection; failure bars rent collection / eviction in many fact patterns",
        ),
    );

    m.insert(
        "MA",
        rule(
            MultipleDwellingRegistration,
            0,
            3,
            true,
            "Mass. G.L. c. 111 § 197A (local board of health registration for 3+ dwelling units) + G.L. c. 186 § 15B(7) (landlord identity rule for any rental) — failure exposes landlord to refusal of remedies and statutory penalties",
        ),
    );

    // LocalLawOrCommonLawOnly default — 44 other states + DC.
    let local_law_only = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE",
        "GA", "HI", "ID", "IL", "IN", "IA", "KS", "KY",
        "LA", "ME", "MD", "MI", "MN", "MS", "MO", "MT",
        "NE", "NV", "NH", "NM", "NC", "ND", "OH", "OK",
        "OR", "PA", "RI", "SC", "SD", "TN", "UT", "VT",
        "VA", "WA", "WV", "WI", "WY",
    ];
    for code in local_law_only {
        m.insert(
            code,
            rule(
                LocalLawOrCommonLawOnly,
                0,
                0,
                false,
                "No confirmed statewide statutory pre-lease owner identification rule; URLTA-adopting states may have analogous § 2-202 provisions; many cities maintain local landlord-registration ordinances (e.g., Chicago RLTO § 5-12-090); common-law fraud / misrepresentation applies when landlord conceals identity to evade service of process",
            ),
        );
    }

    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerIdentificationInput {
    pub state_code: String,
    /// Number of dwelling units in the building (1 for single-family
    /// rental; 4 for fourplex; etc.).
    pub building_unit_count: u32,
    /// True if the landlord has provided the statutorily required
    /// owner identification + agent-for-service disclosure to the
    /// tenant.
    pub landlord_provided_disclosure: bool,
    /// Days elapsed since lease execution (for AffirmativePreLease
    /// regime window analysis).
    pub days_since_lease_execution: u32,
    /// True if the tenant has made a written demand for owner
    /// identification (for DisclosureUponWrittenDemand regime).
    pub tenant_made_written_demand: bool,
    /// Days elapsed since the tenant's written demand was served on
    /// landlord (for TX § 92.201 7-day window).
    pub days_since_tenant_demand: u32,
    /// TX-specific: true if landlord posted the required information
    /// in a conspicuous place / on-site manager's office OR included
    /// it in lease / written rules — alternative compliance to
    /// 7-day-demand-response under § 92.201(b)(2)–(3).
    pub posted_or_in_lease: bool,
    /// TX-specific: true if landlord has previously failed to comply
    /// with a prior tenant demand within 7 days. Second failure
    /// triggers § 92.202(b) termination right.
    pub prior_failure_within_7_days: bool,
    /// For MultipleDwellingRegistration: true if the building has
    /// been registered with the appropriate state / city / local
    /// authority for the current registration period.
    pub building_registered_with_authority: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerIdentificationResult {
    pub regime: OwnerIdentificationRegime,
    pub disclosure_required: bool,
    pub landlord_compliant: bool,
    /// True if the tenant has access to a statutory damages remedy
    /// (e.g., TX § 92.202 one month's rent + $100; MA / NY / NJ
    /// registration-noncompliance penalties).
    pub statutory_damages_available: bool,
    /// True if the tenant has a statutory termination right under
    /// the noncompliance facts. Currently TX § 92.202(b) only —
    /// triggers on second failure within 7 days.
    pub tenant_termination_right: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &OwnerIdentificationInput) -> OwnerIdentificationResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: OwnerIdentificationRegime::LocalLawOrCommonLawOnly,
        disclosure_window_days: 0,
        minimum_units_for_registration: 0,
        statutory_damages_available: false,
        citation: "Unknown state code; local-law or common-law only assumed",
    });

    // Determine disclosure requirement + compliance by regime.
    let (required, compliant, termination_right) = match rule.regime {
        OwnerIdentificationRegime::AffirmativePreLeaseDisclosure => {
            // CA: 15-day window; FL: at commencement (window = 0).
            // Required from day 1; landlord compliant if disclosed.
            let within_window = input.days_since_lease_execution <= rule.disclosure_window_days
                || rule.disclosure_window_days == 0;
            let compliant_now = input.landlord_provided_disclosure
                || (within_window && rule.disclosure_window_days > 0);
            (true, compliant_now, false)
        }
        OwnerIdentificationRegime::DisclosureUponWrittenDemand => {
            // TX § 92.201 — landlord compliant if any of:
            //   (a) responded within 7 days of demand
            //   (b) continuously posted / in lease (no demand needed)
            //   (c) no demand has been made yet
            let posted_path = input.posted_or_in_lease;
            let no_demand_path = !input.tenant_made_written_demand;
            let demand_path = input.tenant_made_written_demand
                && input.landlord_provided_disclosure
                && input.days_since_tenant_demand <= rule.disclosure_window_days;
            let comp = posted_path || no_demand_path || demand_path;
            // § 92.202(b): tenant termination right only after second
            // failure to disclose within 7 days following a written
            // demand.
            let term = input.tenant_made_written_demand
                && !comp
                && input.prior_failure_within_7_days;
            (input.tenant_made_written_demand || !posted_path, comp, term)
        }
        OwnerIdentificationRegime::MultipleDwellingRegistration => {
            // Required only when building has ≥ minimum_units_for_registration.
            let req = input.building_unit_count >= rule.minimum_units_for_registration;
            // Compliant when either not required, OR registered, OR
            // landlord otherwise disclosed identity.
            let comp = !req
                || input.building_registered_with_authority
                || input.landlord_provided_disclosure;
            (req, comp, false)
        }
        OwnerIdentificationRegime::LocalLawOrCommonLawOnly => {
            // No statewide rule — landlord always "compliant" unless
            // facts show common-law fraud (not modeled here).
            (false, true, false)
        }
    };

    let statutory_damages = !compliant && rule.statutory_damages_available;

    let regime_label = match rule.regime {
        OwnerIdentificationRegime::AffirmativePreLeaseDisclosure => {
            "affirmative pre-lease disclosure"
        }
        OwnerIdentificationRegime::DisclosureUponWrittenDemand => "disclosure-on-written-demand",
        OwnerIdentificationRegime::MultipleDwellingRegistration => {
            "multiple-dwelling registration"
        }
        OwnerIdentificationRegime::LocalLawOrCommonLawOnly => "local-law / common-law only",
    };

    let note = if !required {
        format!(
            "State applies {} regime, but no disclosure required on these facts (no demand made / unit count below registration threshold / no statutory rule).",
            regime_label,
        )
    } else if compliant {
        format!(
            "State applies {} regime; landlord compliant on these facts.",
            regime_label,
        )
    } else {
        let mut parts = vec![format!(
            "State applies {} regime; landlord NON-COMPLIANT",
            regime_label,
        )];
        if statutory_damages {
            parts.push("statutory damages available".to_string());
        }
        if termination_right {
            parts.push("tenant termination right triggered (TX § 92.202(b) second-failure)".to_string());
        }
        format!("{}.", parts.join("; "))
    };

    OwnerIdentificationResult {
        regime: rule.regime,
        disclosure_required: required,
        landlord_compliant: compliant,
        statutory_damages_available: statutory_damages,
        tenant_termination_right: termination_right,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str) -> OwnerIdentificationInput {
        OwnerIdentificationInput {
            state_code: state.to_string(),
            building_unit_count: 4,
            landlord_provided_disclosure: false,
            days_since_lease_execution: 30,
            tenant_made_written_demand: false,
            days_since_tenant_demand: 0,
            posted_or_in_lease: false,
            prior_failure_within_7_days: false,
            building_registered_with_authority: false,
        }
    }

    // ── Regime classification ───────────────────────────────────────

    #[test]
    fn ca_affirmative_pre_lease_regime() {
        let r = check(&input("CA"));
        assert_eq!(
            r.regime,
            OwnerIdentificationRegime::AffirmativePreLeaseDisclosure
        );
    }

    #[test]
    fn fl_affirmative_pre_lease_regime() {
        let r = check(&input("FL"));
        assert_eq!(
            r.regime,
            OwnerIdentificationRegime::AffirmativePreLeaseDisclosure
        );
    }

    #[test]
    fn tx_disclosure_on_demand_regime() {
        let r = check(&input("TX"));
        assert_eq!(
            r.regime,
            OwnerIdentificationRegime::DisclosureUponWrittenDemand
        );
    }

    #[test]
    fn ny_nj_ma_multiple_dwelling_regime() {
        for s in ["NY", "NJ", "MA"] {
            let r = check(&input(s));
            assert_eq!(
                r.regime,
                OwnerIdentificationRegime::MultipleDwellingRegistration,
                "expected {s} multiple-dwelling regime"
            );
        }
    }

    #[test]
    fn default_state_local_or_common_law_regime() {
        for s in ["AL", "KS", "DC", "WY", "PA", "IL"] {
            let r = check(&input(s));
            assert_eq!(
                r.regime,
                OwnerIdentificationRegime::LocalLawOrCommonLawOnly,
                "expected {s} local-law regime"
            );
        }
    }

    // ── CA: 15-day window ──────────────────────────────────────────

    #[test]
    fn ca_within_15_day_window_treated_compliant() {
        let mut i = input("CA");
        i.days_since_lease_execution = 15;
        i.landlord_provided_disclosure = false;
        let r = check(&i);
        // Within window: compliant even without explicit disclosure
        // (still has time to comply).
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ca_day_16_post_lease_no_disclosure_noncompliant() {
        let mut i = input("CA");
        i.days_since_lease_execution = 16;
        i.landlord_provided_disclosure = false;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(r.statutory_damages_available);
    }

    #[test]
    fn ca_disclosed_post_window_compliant() {
        let mut i = input("CA");
        i.days_since_lease_execution = 90;
        i.landlord_provided_disclosure = true;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    // ── FL: at commencement (zero-day window) ──────────────────────

    #[test]
    fn fl_no_disclosure_noncompliant_immediately() {
        let r = check(&input("FL"));
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn fl_disclosed_compliant() {
        let mut i = input("FL");
        i.landlord_provided_disclosure = true;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn fl_no_statutory_damages_path_per_83_50() {
        // FL § 83.50 itself has no specific statutory damages
        // formula — that's why FL StateRule has
        // statutory_damages_available = false.
        let r = check(&input("FL"));
        assert!(!r.landlord_compliant);
        assert!(!r.statutory_damages_available);
    }

    // ── TX: 7-day demand window + alternative compliance paths ────

    #[test]
    fn tx_no_demand_no_posting_compliant() {
        // No demand made + no posting — still compliant because
        // demand has not yet been triggered.
        let r = check(&input("TX"));
        assert!(r.landlord_compliant);
    }

    #[test]
    fn tx_demand_disclosed_within_7_days_compliant() {
        let mut i = input("TX");
        i.tenant_made_written_demand = true;
        i.days_since_tenant_demand = 7;
        i.landlord_provided_disclosure = true;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn tx_demand_day_8_no_disclosure_noncompliant() {
        let mut i = input("TX");
        i.tenant_made_written_demand = true;
        i.days_since_tenant_demand = 8;
        i.landlord_provided_disclosure = false;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(r.statutory_damages_available);
    }

    #[test]
    fn tx_posted_satisfies_without_response() {
        let mut i = input("TX");
        i.tenant_made_written_demand = true;
        i.days_since_tenant_demand = 30;
        i.landlord_provided_disclosure = false;
        i.posted_or_in_lease = true;
        let r = check(&i);
        assert!(
            r.landlord_compliant,
            "continuous posting / in-lease inclusion is alternative compliance under § 92.201(b)"
        );
    }

    #[test]
    fn tx_in_lease_alternative_satisfies() {
        let mut i = input("TX");
        i.posted_or_in_lease = true;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn tx_second_failure_triggers_termination_right() {
        let mut i = input("TX");
        i.tenant_made_written_demand = true;
        i.days_since_tenant_demand = 8;
        i.landlord_provided_disclosure = false;
        i.prior_failure_within_7_days = true;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(
            r.tenant_termination_right,
            "§ 92.202(b) gives tenant termination right after second 7-day failure"
        );
        assert!(r.note.contains("§ 92.202(b)"));
    }

    #[test]
    fn tx_first_failure_no_termination_right_yet() {
        let mut i = input("TX");
        i.tenant_made_written_demand = true;
        i.days_since_tenant_demand = 8;
        i.landlord_provided_disclosure = false;
        i.prior_failure_within_7_days = false;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(!r.tenant_termination_right);
    }

    // ── Multiple-dwelling registration regime ──────────────────────

    #[test]
    fn ny_2_unit_below_threshold_not_required() {
        let mut i = input("NY");
        i.building_unit_count = 2;
        let r = check(&i);
        assert!(!r.disclosure_required);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ny_3_unit_at_threshold_required() {
        let mut i = input("NY");
        i.building_unit_count = 3;
        i.building_registered_with_authority = false;
        let r = check(&i);
        assert!(r.disclosure_required);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn nj_4_unit_registered_compliant() {
        let mut i = input("NJ");
        i.building_unit_count = 4;
        i.building_registered_with_authority = true;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ma_3_unit_unregistered_noncompliant() {
        let mut i = input("MA");
        i.building_unit_count = 3;
        i.building_registered_with_authority = false;
        i.landlord_provided_disclosure = false;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(r.statutory_damages_available);
    }

    #[test]
    fn ma_3_unit_landlord_disclosed_separately_compliant() {
        // c. 186 § 15B(7) landlord-identity rule is satisfied by
        // direct disclosure even without c. 111 § 197A registration.
        let mut i = input("MA");
        i.building_unit_count = 3;
        i.building_registered_with_authority = false;
        i.landlord_provided_disclosure = true;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    // ── Default state ──────────────────────────────────────────────

    #[test]
    fn default_state_no_statewide_rule_treated_compliant() {
        let r = check(&input("KS"));
        assert!(!r.disclosure_required);
        assert!(r.landlord_compliant);
    }

    // ── Citation contents ──────────────────────────────────────────

    #[test]
    fn ca_citation_mentions_1962_and_15_days() {
        let r = check(&input("CA"));
        assert!(r.citation.contains("§ 1962"));
        assert!(r.citation.contains("15 days"));
    }

    #[test]
    fn fl_citation_mentions_83_50() {
        let r = check(&input("FL"));
        assert!(r.citation.contains("§ 83.50"));
    }

    #[test]
    fn tx_citation_mentions_92_201_and_92_202_and_7_days() {
        let r = check(&input("TX"));
        assert!(r.citation.contains("§ 92.201"));
        assert!(r.citation.contains("§ 92.202"));
        assert!(r.citation.contains("7 days"));
    }

    #[test]
    fn ny_citation_mentions_mdl_325_and_27_2098() {
        let r = check(&input("NY"));
        assert!(r.citation.contains("MDL § 325"));
        assert!(r.citation.contains("27-2098"));
    }

    #[test]
    fn nj_citation_mentions_55_13a() {
        let r = check(&input("NJ"));
        assert!(r.citation.contains("55:13A"));
    }

    #[test]
    fn ma_citation_mentions_c_111_197a_and_c_186_15b() {
        let r = check(&input("MA"));
        assert!(r.citation.contains("c. 111 § 197A"));
        assert!(r.citation.contains("c. 186 § 15B(7)"));
    }

    // ── Coverage / invariants ──────────────────────────────────────

    #[test]
    fn coverage_is_all_50_states_plus_dc() {
        let count = RULES.len();
        assert_eq!(count, 51, "expected 50 states + DC, got {count}");
    }

    #[test]
    fn citations_never_empty() {
        for (code, rule) in RULES.iter() {
            assert!(!rule.citation.is_empty(), "{code} empty citation");
        }
    }

    #[test]
    fn affirmative_pre_lease_regime_has_exactly_2_states() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    OwnerIdentificationRegime::AffirmativePreLeaseDisclosure
                )
            })
            .count();
        assert_eq!(count, 2, "expected exactly CA + FL = 2 states");
    }

    #[test]
    fn disclosure_on_demand_regime_has_exactly_1_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    OwnerIdentificationRegime::DisclosureUponWrittenDemand
                )
            })
            .count();
        assert_eq!(count, 1, "expected exactly TX = 1 state");
    }

    #[test]
    fn multiple_dwelling_regime_has_exactly_3_states() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    OwnerIdentificationRegime::MultipleDwellingRegistration
                )
            })
            .count();
        assert_eq!(count, 3, "expected exactly NY + NJ + MA = 3 states");
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn ca_noncompliance_note_mentions_regime() {
        let mut i = input("CA");
        i.days_since_lease_execution = 30;
        i.landlord_provided_disclosure = false;
        let r = check(&i);
        assert!(r.note.contains("affirmative pre-lease disclosure"));
        assert!(r.note.contains("NON-COMPLIANT"));
    }

    #[test]
    fn tx_compliance_note_mentions_regime() {
        // TX with posted-in-lease + no demand: landlord-compliant
        // because the obligation is satisfied via posting; no demand
        // has triggered the 7-day clock.
        let mut i = input("TX");
        i.posted_or_in_lease = true;
        let r = check(&i);
        assert!(r.note.contains("disclosure-on-written-demand"));
        assert!(r.landlord_compliant);
    }

    #[test]
    fn tx_demand_compliance_note_says_compliant() {
        // TX with demand made + disclosed within 7 days: required
        // path active and landlord compliant — note should say so.
        let mut i = input("TX");
        i.tenant_made_written_demand = true;
        i.days_since_tenant_demand = 5;
        i.landlord_provided_disclosure = true;
        let r = check(&i);
        assert!(r.landlord_compliant);
        assert!(r.note.contains("compliant"));
    }

    // ── State-code normalization ──────────────────────────────────

    #[test]
    fn lowercase_state_code_normalizes() {
        let mut i = input("ca");
        i.landlord_provided_disclosure = true;
        let r = check(&i);
        assert_eq!(
            r.regime,
            OwnerIdentificationRegime::AffirmativePreLeaseDisclosure
        );
    }
}
