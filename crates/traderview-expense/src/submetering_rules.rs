//! State utility submetering / RUBS (Ratio Utility Billing System)
//! compliance.
//!
//! When a landlord-trader operates multifamily housing and bills
//! tenants for water, sewer, electricity, or gas via SUBMETERS
//! (individual unit meters) or RUBS (mathematical-formula allocation
//! based on square footage / occupancy / bedrooms), several states
//! impose specific disclosure, fee-cap, registration, and testing
//! requirements.
//!
//! Three regimes:
//!
//! - `DisclosureAndTestingRequired` — CA (Cal. Civ. Code § 1954.201
//!   from SB 7 of 2016, eff. 2018-01-01), VA (Va. Code § 55.1-1212).
//!   Submetering or RUBS allowed only if clearly stated in the lease.
//!   Free tenant-requested meter testing required (max once per 24
//!   months in VA; CA tenants contact county sealer for testing).
//!
//! - `PSCRegisteredCappedFees` — TX (Tex. Water Code Ch. 13 + TCEQ
//!   16 TAC § 24.275 et seq.). Submetering operators must register
//!   with the Public Service Commission. Charges to tenants limited
//!   to: cost per gallon + applicable taxes + late fee max **5%** +
//!   service charge max **9%** of utility costs allocated to the
//!   submetered unit. Specific billing-format disclosure required.
//!
//! - `NoStateRegulation` — most other states. Common-law and
//!   contract terms govern. Federal Public Utility Regulatory
//!   Policies Act (PURPA) may apply to wholesale rates but doesn't
//!   regulate landlord-tenant submetering directly.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmeteringRegime {
    DisclosureAndTestingRequired,
    PSCRegisteredCappedFees,
    NoStateRegulation,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: SubmeteringRegime,
    pub written_lease_disclosure_required: bool,
    pub free_tenant_meter_testing_required: bool,
    pub psc_registration_required: bool,
    /// Maximum late fee as a percentage of the late bill, in basis
    /// points (500 = 5%). `None` if no statutory cap.
    pub late_fee_cap_pct_bp: Option<u32>,
    /// Maximum service charge as a percentage of allocated utility
    /// cost, in basis points (900 = 9%). `None` if no statutory cap.
    pub service_charge_cap_pct_bp: Option<u32>,
    pub citation: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    regime: SubmeteringRegime,
    written_lease_disclosure_required: bool,
    free_tenant_meter_testing_required: bool,
    psc_registration_required: bool,
    late_fee_cap_pct_bp: Option<u32>,
    service_charge_cap_pct_bp: Option<u32>,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        written_lease_disclosure_required,
        free_tenant_meter_testing_required,
        psc_registration_required,
        late_fee_cap_pct_bp,
        service_charge_cap_pct_bp,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use SubmeteringRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    // DisclosureAndTestingRequired regime.
    m.insert(
        "CA",
        rule(
            DisclosureAndTestingRequired,
            true, true, false, None, None,
            "Cal. Civ. Code § 1954.201 (SB 7 of 2016, eff. 2018-01-01) — written disclosure required before lease execution; county sealer contact info; tenant-requested testing",
        ),
    );
    m.insert(
        "VA",
        rule(
            DisclosureAndTestingRequired,
            true, true, false, None, None,
            "Va. Code § 55.1-1212 — submetering or RUBS allowed only if clearly stated in lease; periodic testing + free tenant-requested test max once per 24 months",
        ),
    );

    // PSCRegisteredCappedFees regime.
    m.insert(
        "TX",
        rule(
            PSCRegisteredCappedFees,
            true, false, true, Some(500), Some(900),
            "Tex. Water Code Ch. 13 + TCEQ 16 TAC § 24.275 — PSC registration required; late fee cap 5%; service charge cap 9%; specific billing-format disclosure",
        ),
    );

    // NoStateRegulation — all remaining states + DC.
    let no_rule = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA",
        "KS", "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ",
        "NM", "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "UT", "VT", "WA",
        "WV", "WI", "WY",
    ];
    for code in no_rule {
        m.insert(
            code,
            rule(
                NoStateRegulation,
                false, false, false, None, None,
                "No state-level submetering / RUBS regulation; common-law and contract terms govern",
            ),
        );
    }
    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmeteringInput {
    pub state_code: String,
    pub submetering_or_rubs_used: bool,
    pub disclosed_in_lease: bool,
    pub psc_registration_active: bool,
    /// Late fee charged as percentage of late bill in basis points.
    pub late_fee_pct_bp: u32,
    /// Service charge as percentage of allocated utility cost in
    /// basis points.
    pub service_charge_pct_bp: u32,
    pub tenant_requested_meter_test: bool,
    pub free_test_provided_within_24_months: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmeteringResult {
    pub regime: SubmeteringRegime,
    pub lease_disclosure_compliant: bool,
    pub psc_registration_compliant: bool,
    pub late_fee_within_cap: bool,
    pub service_charge_within_cap: bool,
    pub tenant_testing_compliant: bool,
    pub overall_compliant: bool,
    pub violations: Vec<String>,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &SubmeteringInput) -> SubmeteringResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: SubmeteringRegime::NoStateRegulation,
        written_lease_disclosure_required: false,
        free_tenant_meter_testing_required: false,
        psc_registration_required: false,
        late_fee_cap_pct_bp: None,
        service_charge_cap_pct_bp: None,
        citation: "Unknown state code; assuming no state-level submetering regulation",
    });

    let mut violations: Vec<String> = Vec::new();

    let disclosure_ok = if rule.written_lease_disclosure_required && input.submetering_or_rubs_used
    {
        if input.disclosed_in_lease {
            true
        } else {
            violations.push("submetering / RUBS not disclosed in lease".to_string());
            false
        }
    } else {
        true
    };

    let psc_ok = if rule.psc_registration_required && input.submetering_or_rubs_used {
        if input.psc_registration_active {
            true
        } else {
            violations.push("PSC registration not active".to_string());
            false
        }
    } else {
        true
    };

    let late_ok = match rule.late_fee_cap_pct_bp {
        Some(cap) if input.submetering_or_rubs_used && input.late_fee_pct_bp > cap => {
            violations.push(format!(
                "late fee {}.{}% exceeds {}.{}% statutory cap",
                input.late_fee_pct_bp / 100,
                input.late_fee_pct_bp % 100,
                cap / 100,
                cap % 100,
            ));
            false
        }
        _ => true,
    };

    let service_ok = match rule.service_charge_cap_pct_bp {
        Some(cap) if input.submetering_or_rubs_used && input.service_charge_pct_bp > cap => {
            violations.push(format!(
                "service charge {}.{}% exceeds {}.{}% statutory cap",
                input.service_charge_pct_bp / 100,
                input.service_charge_pct_bp % 100,
                cap / 100,
                cap % 100,
            ));
            false
        }
        _ => true,
    };

    let test_ok = if rule.free_tenant_meter_testing_required
        && input.tenant_requested_meter_test
        && !input.free_test_provided_within_24_months
    {
        violations.push(
            "tenant requested meter test but not provided free within 24-month window".to_string(),
        );
        false
    } else {
        true
    };

    let overall = disclosure_ok && psc_ok && late_ok && service_ok && test_ok;

    let note = if !input.submetering_or_rubs_used {
        "No submetering or RUBS in use; state regulation not triggered.".to_string()
    } else if overall {
        format!(
            "{:?}: all compliance checks satisfied for submetering / RUBS use.",
            rule.regime,
        )
    } else {
        format!(
            "{:?} VIOLATION: {} compliance issue(s) — {}.",
            rule.regime,
            violations.len(),
            violations.join("; "),
        )
    };

    SubmeteringResult {
        regime: rule.regime,
        lease_disclosure_compliant: disclosure_ok,
        psc_registration_compliant: psc_ok,
        late_fee_within_cap: late_ok,
        service_charge_within_cap: service_ok,
        tenant_testing_compliant: test_ok,
        overall_compliant: overall,
        violations,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str) -> SubmeteringInput {
        SubmeteringInput {
            state_code: state.to_string(),
            submetering_or_rubs_used: true,
            disclosed_in_lease: true,
            psc_registration_active: true,
            late_fee_pct_bp: 0,
            service_charge_pct_bp: 0,
            tenant_requested_meter_test: false,
            free_test_provided_within_24_months: false,
        }
    }

    // CA — disclosure + testing.

    #[test]
    fn ca_with_lease_disclosure_complies() {
        let r = check(&input("CA"));
        assert_eq!(r.regime, SubmeteringRegime::DisclosureAndTestingRequired);
        assert!(r.lease_disclosure_compliant);
        assert!(r.overall_compliant);
    }

    #[test]
    fn ca_without_lease_disclosure_violates() {
        let mut i = input("CA");
        i.disclosed_in_lease = false;
        let r = check(&i);
        assert!(!r.lease_disclosure_compliant);
        assert!(!r.overall_compliant);
        assert!(r.violations.iter().any(|v| v.contains("not disclosed")));
    }

    #[test]
    fn ca_tenant_test_request_must_be_free() {
        let mut i = input("CA");
        i.tenant_requested_meter_test = true;
        i.free_test_provided_within_24_months = false;
        let r = check(&i);
        assert!(!r.tenant_testing_compliant);
        assert!(!r.overall_compliant);
    }

    #[test]
    fn ca_tenant_test_provided_complies() {
        let mut i = input("CA");
        i.tenant_requested_meter_test = true;
        i.free_test_provided_within_24_months = true;
        let r = check(&i);
        assert!(r.tenant_testing_compliant);
        assert!(r.overall_compliant);
    }

    // VA — same regime as CA.

    #[test]
    fn va_without_lease_disclosure_violates() {
        let mut i = input("VA");
        i.disclosed_in_lease = false;
        let r = check(&i);
        assert_eq!(r.regime, SubmeteringRegime::DisclosureAndTestingRequired);
        assert!(!r.overall_compliant);
    }

    #[test]
    fn va_24_month_test_window_pinned_in_citation() {
        let r = check(&input("VA"));
        assert!(r.citation.contains("24 months"));
    }

    // TX — PSC registration + fee caps.

    #[test]
    fn tx_with_psc_registration_complies() {
        let r = check(&input("TX"));
        assert_eq!(r.regime, SubmeteringRegime::PSCRegisteredCappedFees);
        assert!(r.psc_registration_compliant);
    }

    #[test]
    fn tx_without_psc_registration_violates() {
        let mut i = input("TX");
        i.psc_registration_active = false;
        let r = check(&i);
        assert!(!r.psc_registration_compliant);
        assert!(!r.overall_compliant);
    }

    #[test]
    fn tx_late_fee_5_pct_exact_complies() {
        let mut i = input("TX");
        i.late_fee_pct_bp = 500;
        let r = check(&i);
        assert!(r.late_fee_within_cap);
    }

    #[test]
    fn tx_late_fee_6_pct_violates() {
        let mut i = input("TX");
        i.late_fee_pct_bp = 600;
        let r = check(&i);
        assert!(!r.late_fee_within_cap);
        assert!(!r.overall_compliant);
    }

    #[test]
    fn tx_service_charge_9_pct_exact_complies() {
        let mut i = input("TX");
        i.service_charge_pct_bp = 900;
        let r = check(&i);
        assert!(r.service_charge_within_cap);
    }

    #[test]
    fn tx_service_charge_10_pct_violates() {
        let mut i = input("TX");
        i.service_charge_pct_bp = 1000;
        let r = check(&i);
        assert!(!r.service_charge_within_cap);
        assert!(!r.overall_compliant);
    }

    #[test]
    fn tx_all_three_fee_caps_individually_pinned_at_5_and_9() {
        let tx = RULES.get("TX").unwrap();
        assert_eq!(tx.late_fee_cap_pct_bp, Some(500));
        assert_eq!(tx.service_charge_cap_pct_bp, Some(900));
    }

    // No-regulation states.

    #[test]
    fn no_regulation_state_no_compliance_issues() {
        let mut i = input("FL");
        i.disclosed_in_lease = false; // Irrelevant
        i.late_fee_pct_bp = 5000; // 50% — would violate elsewhere
        let r = check(&i);
        assert_eq!(r.regime, SubmeteringRegime::NoStateRegulation);
        assert!(r.overall_compliant);
    }

    // No submetering / RUBS in use.

    #[test]
    fn no_submetering_no_state_check_triggered() {
        let mut i = input("CA");
        i.submetering_or_rubs_used = false;
        i.disclosed_in_lease = false; // Doesn't matter
        let r = check(&i);
        assert!(r.overall_compliant);
        assert!(r.note.contains("not triggered"));
    }

    // Coverage / invariants.

    #[test]
    fn coverage_is_all_50_states_plus_dc() {
        let codes: Vec<&'static str> = RULES.keys().copied().collect();
        assert_eq!(
            codes.len(),
            51,
            "expected 50 states + DC, got {}",
            codes.len()
        );
    }

    #[test]
    fn citations_never_empty() {
        for (code, rule) in RULES.iter() {
            assert!(!rule.citation.is_empty(), "{code} missing citation");
        }
    }

    #[test]
    fn ca_va_only_disclosure_and_testing_regime() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == SubmeteringRegime::DisclosureAndTestingRequired {
                count += 1;
            }
        }
        assert_eq!(
            count, 2,
            "expected CA + VA only with DisclosureAndTestingRequired"
        );
    }

    #[test]
    fn tx_only_psc_registered_regime() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == SubmeteringRegime::PSCRegisteredCappedFees {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected TX only with PSCRegisteredCappedFees");
    }

    #[test]
    fn only_tx_has_fee_caps() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.late_fee_cap_pct_bp.is_some() || rule.service_charge_cap_pct_bp.is_some() {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected TX only with fee caps");
    }

    #[test]
    fn unknown_state_falls_back_to_no_regulation() {
        let r = check(&input("XX"));
        assert_eq!(r.regime, SubmeteringRegime::NoStateRegulation);
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let mut i = input("tx");
        i.late_fee_pct_bp = 600;
        let r = check(&i);
        assert!(!r.overall_compliant);
    }

    // Multiple violations stack.

    #[test]
    fn tx_multiple_violations_all_listed() {
        let mut i = input("TX");
        i.psc_registration_active = false;
        i.late_fee_pct_bp = 600;
        i.service_charge_pct_bp = 1000;
        let r = check(&i);
        assert_eq!(r.violations.len(), 3);
    }

    #[test]
    fn ca_violation_note_lists_issues() {
        let mut i = input("CA");
        i.disclosed_in_lease = false;
        let r = check(&i);
        assert!(r.note.contains("VIOLATION"));
        assert!(r.note.contains("not disclosed"));
    }
}
