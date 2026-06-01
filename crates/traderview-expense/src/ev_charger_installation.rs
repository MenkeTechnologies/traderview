//! State tenant right to install electric-vehicle charging stations.
//!
//! Modern "right-to-charge" laws addressing the growing demand for
//! at-home EV charging in multi-unit rental housing. Four states
//! have enacted tenant-specific protections; the rest leave the
//! decision to the lease (MD/VA/CO right-to-charge laws cover HOAs
//! and condos but not standalone rental tenants).
//!
//! Five regimes:
//!
//! `CaliforniaInsuranceRequired`: CA only. Cal. Civ. Code § 1947.6
//! (eff. 2015-07-01). For residential leases signed, renewed, or
//! extended on/after 2015-07-01, landlord MUST approve a tenant's
//! written request to install an EV charging station in the
//! tenant's dedicated parking space when the tenant pays all
//! charging-station / installation / utility costs, enters into a
//! written agreement covering installation/use/maintenance/removal,
//! and maintains a $1,000,000 general-liability insurance policy.
//! All three conditions are independently required. The $1M
//! insurance prong is the most-litigated gatekeeper.
//!
//! `HawaiiLeaseProvisionVoid`: HI only. HRS § 196-7.5. The
//! strongest tenant-protective regime: no covenant, declaration,
//! bylaw, restriction, deed, lease, or similar agreement can
//! prevent installation of an EV charging system on or near the
//! parking stall. Any contrary lease provision is statutorily
//! VOID and unenforceable. Protection extends to common-element
//! parking (first-come, first-served) — not just dedicated stalls.
//!
//! `IllinoisNewBuildingsOnly`: IL only. Electric Vehicle Charging
//! Act, 765 ILCS et seq. (eff. 2023). Renters and condo owners in
//! NEW homes and multi-unit dwellings have a right to install
//! chargers without unreasonable restriction. Requires 100% of
//! parking spaces at NEW construction to be EV-ready (conduit +
//! reserved capacity). Older buildings remain lease-governed.
//!
//! `NewJerseyMultiUnitRight`: NJ only. Right-to-charge protection
//! for tenants in multi-unit residential buildings. Lease
//! restrictions on EV charger installation are subject to
//! reasonable-terms review by the state.
//!
//! `DefaultLeaseGoverns`: 46 other states + DC. Lease terms
//! govern EV charger installation. Many states have HOA/condo
//! right-to-charge laws (MD, VA, CO, FL, etc.) but those do NOT
//! extend to standalone rental tenants. Tenant must obtain
//! landlord consent through lease negotiation.
//!
//! Sources:
//! [Cal. Civ. Code § 1947.6 — Kimball Tirey & St. John](https://www.kts-law.com/electric-vehicle-charging-stations-for-california-landlords/),
//! [HRS § 196-7.5 — Plug In America Right-to-Charge Policies](https://pluginamerica.org/policy/right-to-charge-policies/),
//! [Illinois Electric Vehicle Charging Act (765 ILCS) — VendorPM Chicago](https://www.vendorpm.com/blog-posts/installing-electric-car-charging-in-condos-and-apartments-in-chicago),
//! [GreenLancer — Right to Charge Laws in California & Beyond](https://www.greenlancer.com/post/right-to-charge-laws),
//! [The Conversation — Right-to-charge laws bring promise of EVs to apartments](https://theconversation.com/right-to-charge-laws-bring-the-promise-of-evs-to-apartments-condos-and-rentals-206721).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvChargerRegime {
    CaliforniaInsuranceRequired,
    HawaiiLeaseProvisionVoid,
    IllinoisNewBuildingsOnly,
    NewJerseyMultiUnitRight,
    DefaultLeaseGoverns,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: EvChargerRegime,
    /// True if the regime requires tenant-supplied liability
    /// insurance as a precondition.
    pub liability_insurance_required: bool,
    /// Minimum liability-insurance amount in dollars (CA $1M).
    pub minimum_liability_insurance_dollars: i64,
    /// True if the regime applies only to new construction (IL).
    pub new_construction_only: bool,
    /// True if the regime requires the building be multi-unit
    /// residential (NJ).
    pub multi_unit_residential_required: bool,
    /// True if any lease restriction on EV-charger installation is
    /// statutorily VOID and unenforceable (HI).
    pub lease_restrictions_void: bool,
    pub citation: &'static str,
}

const fn rule(
    regime: EvChargerRegime,
    liability_insurance_required: bool,
    minimum_liability_insurance_dollars: i64,
    new_construction_only: bool,
    multi_unit_residential_required: bool,
    lease_restrictions_void: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        liability_insurance_required,
        minimum_liability_insurance_dollars,
        new_construction_only,
        multi_unit_residential_required,
        lease_restrictions_void,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use EvChargerRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "CA",
        rule(
            CaliforniaInsuranceRequired,
            true,
            1_000_000,
            false,
            false,
            false,
            "Cal. Civ. Code § 1947.6 (eff. 2015-07-01) — landlord MUST approve tenant's written request to install EV charging station in dedicated parking space when tenant: (1) pays for charging station + installation + utility costs, (2) enters into written agreement covering installation/use/maintenance/removal, (3) maintains $1,000,000 general-liability insurance policy",
        ),
    );

    m.insert(
        "HI",
        rule(
            HawaiiLeaseProvisionVoid,
            false,
            0,
            false,
            false,
            true,
            "HRS § 196-7.5 — no covenant, declaration, bylaw, restriction, deed, lease, or similar agreement may prevent installation of EV charging system on or near parking stall of multi-family residential dwelling or townhouse; any contrary lease provision is statutorily VOID and unenforceable; protection extends to common-element parking",
        ),
    );

    m.insert(
        "IL",
        rule(
            IllinoisNewBuildingsOnly,
            false,
            0,
            true,
            true,
            false,
            "Illinois Electric Vehicle Charging Act, 765 ILCS et seq. (eff. 2023) — renters and condo owners in NEW homes and multi-unit dwellings have right to install chargers without unreasonable restriction; 100% of parking spaces at new construction must be EV-ready (conduit + reserved capacity); older buildings remain lease-governed",
        ),
    );

    m.insert(
        "NJ",
        rule(
            NewJerseyMultiUnitRight,
            false,
            0,
            false,
            true,
            false,
            "New Jersey right-to-charge protection for tenants in multi-unit residential buildings; lease restrictions on EV charger installation subject to reasonable-terms review",
        ),
    );

    // DefaultLeaseGoverns — 46 other states + DC.
    let default_states = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE",
        "FL", "GA", "ID", "IN", "IA", "KS", "KY", "LA",
        "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT",
        "NE", "NV", "NH", "NM", "NY", "NC", "ND", "OH",
        "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX",
        "UT", "VT", "VA", "WA", "WV", "WI", "WY",
    ];
    for code in default_states {
        m.insert(
            code,
            rule(
                DefaultLeaseGoverns,
                false,
                0,
                false,
                false,
                false,
                "No statewide tenant EV-charger installation right; lease terms govern; HOA / condo right-to-charge laws (MD, VA, CO, FL) do not extend to standalone rental tenants",
            ),
        );
    }

    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvChargerInput {
    pub state_code: String,
    /// True if the lease was signed / renewed / extended after the
    /// state's effective date.
    pub lease_signed_after_effective_date: bool,
    /// True if the building is new construction (post-effective-date).
    pub building_is_new_construction: bool,
    /// True if the building is multi-unit residential.
    pub building_is_multi_unit_residential: bool,
    /// True if the tenant is willing to pay all installation,
    /// equipment, and utility costs.
    pub tenant_willing_to_pay_all_costs: bool,
    /// True if the tenant carries a general-liability insurance
    /// policy. Module checks the amount below.
    pub tenant_carries_liability_insurance: bool,
    /// Tenant's liability-insurance coverage amount in dollars
    /// (CA $1M precondition).
    pub tenant_liability_insurance_amount_dollars: i64,
    /// True if tenant has submitted a written request to install
    /// the EV charger.
    pub written_request_submitted: bool,
    /// True if tenant has signed a written agreement covering
    /// installation, use, maintenance, and removal.
    pub written_agreement_signed: bool,
    /// True if the lease contains a restriction prohibiting EV
    /// charger installation.
    pub lease_contains_ev_restriction: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvChargerResult {
    pub regime: EvChargerRegime,
    pub statute_applies_on_facts: bool,
    pub tenant_eligible_to_install: bool,
    pub lease_restriction_enforceable: bool,
    /// True if the CA $1M insurance precondition is satisfied.
    pub liability_insurance_threshold_met: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &EvChargerInput) -> EvChargerResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: EvChargerRegime::DefaultLeaseGoverns,
        liability_insurance_required: false,
        minimum_liability_insurance_dollars: 0,
        new_construction_only: false,
        multi_unit_residential_required: false,
        lease_restrictions_void: false,
        citation: "Unknown state code; default lease-governs assumed",
    });

    // Insurance threshold check (CA).
    let insurance_met = !rule.liability_insurance_required
        || (input.tenant_carries_liability_insurance
            && input.tenant_liability_insurance_amount_dollars
                >= rule.minimum_liability_insurance_dollars);

    // Statute applicability gates.
    let new_construction_gate_ok =
        !rule.new_construction_only || input.building_is_new_construction;
    let multi_unit_gate_ok =
        !rule.multi_unit_residential_required || input.building_is_multi_unit_residential;
    let lease_date_gate_ok =
        matches!(rule.regime, EvChargerRegime::DefaultLeaseGoverns)
            || input.lease_signed_after_effective_date;

    let applies = !matches!(rule.regime, EvChargerRegime::DefaultLeaseGoverns)
        && new_construction_gate_ok
        && multi_unit_gate_ok
        && lease_date_gate_ok;

    // Eligibility depends on regime.
    let eligible = match rule.regime {
        EvChargerRegime::CaliforniaInsuranceRequired => {
            applies
                && input.tenant_willing_to_pay_all_costs
                && input.written_request_submitted
                && input.written_agreement_signed
                && insurance_met
        }
        EvChargerRegime::HawaiiLeaseProvisionVoid => applies,
        EvChargerRegime::IllinoisNewBuildingsOnly => {
            applies && input.tenant_willing_to_pay_all_costs
        }
        EvChargerRegime::NewJerseyMultiUnitRight => {
            applies && input.tenant_willing_to_pay_all_costs
        }
        EvChargerRegime::DefaultLeaseGoverns => !input.lease_contains_ev_restriction,
    };

    let lease_enforceable = !(rule.lease_restrictions_void && eligible);

    let regime_label = match rule.regime {
        EvChargerRegime::CaliforniaInsuranceRequired => {
            "California § 1947.6 + $1M insurance precondition"
        }
        EvChargerRegime::HawaiiLeaseProvisionVoid => "Hawaii lease-restriction-VOID rule",
        EvChargerRegime::IllinoisNewBuildingsOnly => {
            "Illinois Electric Vehicle Charging Act (new buildings only)"
        }
        EvChargerRegime::NewJerseyMultiUnitRight => "New Jersey multi-unit right-to-charge",
        EvChargerRegime::DefaultLeaseGoverns => "default lease-governs",
    };

    let note = if !applies && !matches!(rule.regime, EvChargerRegime::DefaultLeaseGoverns) {
        let mut reasons = vec![];
        if !lease_date_gate_ok {
            reasons.push("lease pre-dates effective date");
        }
        if !new_construction_gate_ok {
            reasons.push("not new construction");
        }
        if !multi_unit_gate_ok {
            reasons.push("not multi-unit residential");
        }
        format!(
            "State applies {} regime but statute gates not met: {}.",
            regime_label,
            reasons.join("; "),
        )
    } else if eligible {
        format!(
            "State applies {} regime; tenant eligible to install EV charger on these facts.",
            regime_label,
        )
    } else {
        let mut reasons = vec![];
        if rule.liability_insurance_required && !insurance_met {
            reasons.push(format!(
                "${} liability insurance threshold NOT met",
                rule.minimum_liability_insurance_dollars
            ));
        }
        if matches!(rule.regime, EvChargerRegime::CaliforniaInsuranceRequired) {
            if !input.tenant_willing_to_pay_all_costs {
                reasons.push("tenant not willing to pay costs".to_string());
            }
            if !input.written_request_submitted {
                reasons.push("written request not submitted".to_string());
            }
            if !input.written_agreement_signed {
                reasons.push("written agreement not signed".to_string());
            }
        }
        if reasons.is_empty() {
            format!(
                "State applies {} regime; tenant NOT eligible on these facts.",
                regime_label,
            )
        } else {
            format!(
                "State applies {} regime; tenant NOT eligible: {}.",
                regime_label,
                reasons.join("; "),
            )
        }
    };

    EvChargerResult {
        regime: rule.regime,
        statute_applies_on_facts: applies,
        tenant_eligible_to_install: eligible,
        lease_restriction_enforceable: lease_enforceable,
        liability_insurance_threshold_met: insurance_met,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline(state: &str) -> EvChargerInput {
        EvChargerInput {
            state_code: state.to_string(),
            lease_signed_after_effective_date: true,
            building_is_new_construction: true,
            building_is_multi_unit_residential: true,
            tenant_willing_to_pay_all_costs: true,
            tenant_carries_liability_insurance: true,
            tenant_liability_insurance_amount_dollars: 1_000_000,
            written_request_submitted: true,
            written_agreement_signed: true,
            lease_contains_ev_restriction: false,
        }
    }

    // ── Regime classification ───────────────────────────────────────

    #[test]
    fn ca_insurance_required_regime() {
        let r = check(&baseline("CA"));
        assert_eq!(r.regime, EvChargerRegime::CaliforniaInsuranceRequired);
    }

    #[test]
    fn hi_lease_provision_void_regime() {
        let r = check(&baseline("HI"));
        assert_eq!(r.regime, EvChargerRegime::HawaiiLeaseProvisionVoid);
    }

    #[test]
    fn il_new_buildings_only_regime() {
        let r = check(&baseline("IL"));
        assert_eq!(r.regime, EvChargerRegime::IllinoisNewBuildingsOnly);
    }

    #[test]
    fn nj_multi_unit_right_regime() {
        let r = check(&baseline("NJ"));
        assert_eq!(r.regime, EvChargerRegime::NewJerseyMultiUnitRight);
    }

    #[test]
    fn default_state_lease_governs_regime() {
        for s in ["AL", "FL", "TX", "WA", "DC", "WY", "NY", "MA"] {
            let r = check(&baseline(s));
            assert_eq!(
                r.regime,
                EvChargerRegime::DefaultLeaseGoverns,
                "expected {s} default regime"
            );
        }
    }

    // ── CA: 4-prong (date + insurance + pay + agreement) ───────────

    #[test]
    fn ca_all_4_prongs_met_eligible() {
        let r = check(&baseline("CA"));
        assert!(r.tenant_eligible_to_install);
        assert!(r.liability_insurance_threshold_met);
    }

    #[test]
    fn ca_insurance_below_1m_not_eligible() {
        let mut i = baseline("CA");
        i.tenant_liability_insurance_amount_dollars = 500_000;
        let r = check(&i);
        assert!(!r.liability_insurance_threshold_met);
        assert!(!r.tenant_eligible_to_install);
    }

    #[test]
    fn ca_insurance_exactly_1m_eligible() {
        let mut i = baseline("CA");
        i.tenant_liability_insurance_amount_dollars = 1_000_000;
        let r = check(&i);
        assert!(r.liability_insurance_threshold_met);
        assert!(r.tenant_eligible_to_install);
    }

    #[test]
    fn ca_no_insurance_carried_not_eligible() {
        let mut i = baseline("CA");
        i.tenant_carries_liability_insurance = false;
        let r = check(&i);
        assert!(!r.liability_insurance_threshold_met);
    }

    #[test]
    fn ca_tenant_unwilling_to_pay_not_eligible() {
        let mut i = baseline("CA");
        i.tenant_willing_to_pay_all_costs = false;
        let r = check(&i);
        assert!(!r.tenant_eligible_to_install);
    }

    #[test]
    fn ca_no_written_request_not_eligible() {
        let mut i = baseline("CA");
        i.written_request_submitted = false;
        let r = check(&i);
        assert!(!r.tenant_eligible_to_install);
    }

    #[test]
    fn ca_no_written_agreement_not_eligible() {
        let mut i = baseline("CA");
        i.written_agreement_signed = false;
        let r = check(&i);
        assert!(!r.tenant_eligible_to_install);
    }

    #[test]
    fn ca_lease_pre_effective_date_statute_inapplicable() {
        let mut i = baseline("CA");
        i.lease_signed_after_effective_date = false;
        let r = check(&i);
        assert!(!r.statute_applies_on_facts);
    }

    // ── HI: lease provision void; no other gates ───────────────────

    #[test]
    fn hi_baseline_eligible_no_insurance_required() {
        let mut i = baseline("HI");
        i.tenant_carries_liability_insurance = false;
        i.tenant_liability_insurance_amount_dollars = 0;
        let r = check(&i);
        assert!(r.tenant_eligible_to_install);
    }

    #[test]
    fn hi_lease_restriction_void_when_statute_permits() {
        let mut i = baseline("HI");
        i.lease_contains_ev_restriction = true;
        let r = check(&i);
        assert!(r.tenant_eligible_to_install);
        assert!(!r.lease_restriction_enforceable);
    }

    // ── IL: new buildings only ─────────────────────────────────────

    #[test]
    fn il_new_construction_eligible() {
        let r = check(&baseline("IL"));
        assert!(r.tenant_eligible_to_install);
    }

    #[test]
    fn il_older_construction_statute_inapplicable() {
        let mut i = baseline("IL");
        i.building_is_new_construction = false;
        let r = check(&i);
        assert!(!r.statute_applies_on_facts);
    }

    #[test]
    fn il_not_multi_unit_statute_inapplicable() {
        let mut i = baseline("IL");
        i.building_is_multi_unit_residential = false;
        let r = check(&i);
        assert!(!r.statute_applies_on_facts);
    }

    // ── NJ: multi-unit required ────────────────────────────────────

    #[test]
    fn nj_multi_unit_eligible() {
        let r = check(&baseline("NJ"));
        assert!(r.tenant_eligible_to_install);
    }

    #[test]
    fn nj_single_family_statute_inapplicable() {
        let mut i = baseline("NJ");
        i.building_is_multi_unit_residential = false;
        let r = check(&i);
        assert!(!r.statute_applies_on_facts);
    }

    // ── Default lease-governs ──────────────────────────────────────

    #[test]
    fn default_state_no_lease_restriction_eligible() {
        let r = check(&baseline("TX"));
        assert!(r.tenant_eligible_to_install);
    }

    #[test]
    fn default_state_lease_restriction_blocks_install() {
        let mut i = baseline("TX");
        i.lease_contains_ev_restriction = true;
        let r = check(&i);
        assert!(!r.tenant_eligible_to_install);
    }

    // ── Citations ──────────────────────────────────────────────────

    #[test]
    fn ca_citation_mentions_1947_6_and_1m() {
        let r = check(&baseline("CA"));
        assert!(r.citation.contains("§ 1947.6"));
        assert!(r.citation.contains("$1,000,000"));
    }

    #[test]
    fn hi_citation_mentions_196_7_5_and_void() {
        let r = check(&baseline("HI"));
        assert!(r.citation.contains("§ 196-7.5"));
        assert!(r.citation.contains("VOID"));
    }

    #[test]
    fn il_citation_mentions_765_ilcs_and_new_construction() {
        let r = check(&baseline("IL"));
        assert!(r.citation.contains("765 ILCS"));
        assert!(r.citation.contains("NEW"));
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
    fn ca_only_insurance_required_state() {
        let count = RULES.iter().filter(|(_, r)| r.liability_insurance_required).count();
        assert_eq!(count, 1, "only CA requires liability insurance");
    }

    #[test]
    fn hi_only_lease_void_state() {
        let count = RULES.iter().filter(|(_, r)| r.lease_restrictions_void).count();
        assert_eq!(count, 1, "only HI voids lease restrictions");
    }

    #[test]
    fn il_only_new_construction_state() {
        let count = RULES.iter().filter(|(_, r)| r.new_construction_only).count();
        assert_eq!(count, 1, "only IL restricts to new construction");
    }

    // ── Normalization ──────────────────────────────────────────────

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&baseline("ca"));
        assert_eq!(r.regime, EvChargerRegime::CaliforniaInsuranceRequired);
    }
}
