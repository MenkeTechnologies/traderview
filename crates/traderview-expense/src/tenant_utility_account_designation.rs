//! Tenant right to designate utility service account in tenant's
//! own name — when may the residential tenant become the direct
//! customer of record with the utility company (vs being forced
//! onto the landlord's master account)? Distinct from
//! `submetering_rules` (sub-metering setup), `utility_shutoff`
//! (procedural shutoff protections), and `non_refundable_
//! cleaning_fees` (move-out fees).
//!
//! Trader-landlord operational concern when landlord is delinquent
//! on utility bills and utility threatens to shutoff service. CA
//! Pub. Util. Code § 777 protects residential occupants from
//! losing service due to landlord's delinquency by giving them the
//! right to become the direct customer.
//!
//! **Three regimes**:
//!
//! **California — Cal. Pub. Util. Code § 777**. Most explicit
//! tenant-protection framework. Applies to landlord-tenant
//! arrangements where the landlord is the customer of record for
//! INDIVIDUALLY METERED residential service (electric, gas, heat,
//! water). Residential occupants have the right to BECOME
//! CUSTOMERS without being required to pay any delinquent amount
//! due on the landlord's account. Utility corporation MUST inform
//! occupants of this right. Occupant may verify landlord's
//! customer-of-record status via lease, rent receipts, or
//! government document confirming tenancy. **§ 777 does NOT apply
//! to master-metered apartment buildings** — RUBS / master-meter
//! pass-through scenarios remain outside § 777's protection.
//!
//! **New York — N.Y. Public Service Law §§ 32, 33, 33-a**.
//! Residential utility customer protections under PSC tariff
//! framework. Section 33-a Home Energy Fair Practices Act (HEFPA)
//! provides utility shutoff procedural protections + right to
//! direct customer relationship. Tenant in shared-meter
//! arrangement may petition PSC for separate account.
//!
//! **Default — lease + utility company tariff**. Most states have
//! no statewide tenant utility-designation right; rules are set
//! by utility company tariff (regulated by state PUC) and lease
//! provisions. Tenant generally may open direct account if landlord
//! consents and unit is individually metered.
//!
//! Citations: Cal. Pub. Util. Code § 777 (CA tenant right to
//! become customer when landlord delinquent on individually-
//! metered service); Cal. Pub. Util. Code § 777.1 (related
//! disclosure obligations); N.Y. Pub. Serv. Law §§ 32, 33, 33-a
//! (NY HEFPA residential utility protections); state PUC tariff
//! frameworks (Default regime).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UtilityMeteringType {
    /// Tenant's unit has its own dedicated utility meter; tenant
    /// can be direct customer.
    IndividuallyMetered,
    /// Building-level master meter; tenant share allocated via
    /// landlord (RUBS / sub-metering).
    MasterMetered,
    /// Sub-metered — building has master meter but landlord
    /// installs sub-meters per unit.
    SubMetered,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantUtilityAccountInput {
    pub regime: Regime,
    pub metering_type: UtilityMeteringType,
    /// Whether the landlord is the current customer of record
    /// with the utility company.
    pub landlord_is_customer_of_record: bool,
    /// Whether the landlord is DELINQUENT on the utility account
    /// (triggers § 777 right-to-become-customer).
    pub landlord_delinquent_on_utility_account: bool,
    /// Whether the tenant can verify landlord's customer-of-record
    /// status via lease, rent receipts, or government document.
    pub tenant_can_verify_landlord_status: bool,
    /// Whether the tenant wishes to assume the utility account
    /// in tenant's own name.
    pub tenant_requests_to_become_customer: bool,
    /// Whether the utility company has provided required notice
    /// of tenant's § 777 right.
    pub utility_provided_notice_of_tenant_right: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TenantUtilityAccountResult {
    pub tenant_right_to_become_customer_engaged: bool,
    /// Whether the tenant is protected from paying landlord's
    /// delinquent balance.
    pub tenant_protected_from_landlord_arrears: bool,
    /// Whether the regime applies to this metering type (CA § 777
    /// does NOT apply to master-metered buildings).
    pub regime_applies_to_metering_type: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &TenantUtilityAccountInput) -> TenantUtilityAccountResult {
    match input.regime {
        Regime::California => check_california(input),
        Regime::NewYork => check_new_york(input),
        Regime::Default => check_default(input),
    }
}

fn check_california(input: &TenantUtilityAccountInput) -> TenantUtilityAccountResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    let metering_applies = matches!(
        input.metering_type,
        UtilityMeteringType::IndividuallyMetered
    );

    notes.push(
        "Cal. Pub. Util. Code § 777 — tenant in INDIVIDUALLY METERED residential service has right to become direct customer when landlord is customer of record"
            .to_string(),
    );

    if !metering_applies {
        notes.push(
            "Cal. Pub. Util. Code § 777 does NOT apply to master-metered apartment buildings; RUBS / master-meter pass-through scenarios remain outside § 777 protection"
                .to_string(),
        );
        return TenantUtilityAccountResult {
            tenant_right_to_become_customer_engaged: false,
            tenant_protected_from_landlord_arrears: false,
            regime_applies_to_metering_type: false,
            violations,
            citation: "Cal. Pub. Util. Code §§ 777, 777.1",
            notes,
        };
    }

    let right_engaged = input.landlord_is_customer_of_record
        && input.tenant_requests_to_become_customer
        && input.tenant_can_verify_landlord_status;

    if right_engaged {
        notes.push(
            "Cal. Pub. Util. Code § 777 — tenant right to become customer ENGAGED; tenant NOT required to pay any delinquent amount due on landlord's account"
                .to_string(),
        );
    }

    if input.landlord_delinquent_on_utility_account
        && !input.utility_provided_notice_of_tenant_right
    {
        violations.push(
            "Cal. Pub. Util. Code § 777 — utility corporation MUST inform residential occupants of right to become customer when landlord delinquent on individually-metered service"
                .to_string(),
        );
    }

    notes.push(
        "Cal. Pub. Util. Code § 777 — occupant may verify landlord's customer-of-record status via lease, rent receipts, or government document confirming tenancy"
            .to_string(),
    );
    notes.push(
        "Cal. Pub. Util. Code § 777.1 — related utility-corporation disclosure obligations supplement § 777 tenant rights"
            .to_string(),
    );

    TenantUtilityAccountResult {
        tenant_right_to_become_customer_engaged: right_engaged,
        tenant_protected_from_landlord_arrears: right_engaged,
        regime_applies_to_metering_type: true,
        violations,
        citation: "Cal. Pub. Util. Code §§ 777, 777.1",
        notes,
    }
}

fn check_new_york(input: &TenantUtilityAccountInput) -> TenantUtilityAccountResult {
    let mut notes: Vec<String> = Vec::new();

    let metering_applies = !matches!(input.metering_type, UtilityMeteringType::MasterMetered);

    notes.push(
        "N.Y. Pub. Serv. Law §§ 32, 33, 33-a (Home Energy Fair Practices Act / HEFPA) — residential utility customer protections under PSC tariff framework"
            .to_string(),
    );
    notes.push(
        "NY HEFPA — tenant in shared-meter arrangement may petition NY PSC for separate account designation"
            .to_string(),
    );

    let right_engaged = metering_applies
        && input.landlord_is_customer_of_record
        && input.tenant_requests_to_become_customer;

    TenantUtilityAccountResult {
        tenant_right_to_become_customer_engaged: right_engaged,
        tenant_protected_from_landlord_arrears: right_engaged,
        regime_applies_to_metering_type: metering_applies,
        violations: Vec::new(),
        citation: "N.Y. Pub. Serv. Law §§ 32, 33, 33-a (HEFPA)",
        notes,
    }
}

fn check_default(input: &TenantUtilityAccountInput) -> TenantUtilityAccountResult {
    let notes: Vec<String> = vec![
        "default rule — no statewide tenant utility-designation right; rules set by state-PUC-regulated utility tariff + lease"
            .to_string(),
        "default rule — tenant generally may open direct account if landlord consents and unit is individually metered; master-metered buildings (RUBS pass-through) typically lack tenant-side designation right"
            .to_string(),
    ];

    let metering_applies = matches!(
        input.metering_type,
        UtilityMeteringType::IndividuallyMetered | UtilityMeteringType::SubMetered
    );

    TenantUtilityAccountResult {
        tenant_right_to_become_customer_engaged: false,
        tenant_protected_from_landlord_arrears: false,
        regime_applies_to_metering_type: metering_applies,
        violations: Vec::new(),
        citation: "state PUC tariff framework + lease provisions",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_individually_metered_base() -> TenantUtilityAccountInput {
        TenantUtilityAccountInput {
            regime: Regime::California,
            metering_type: UtilityMeteringType::IndividuallyMetered,
            landlord_is_customer_of_record: true,
            landlord_delinquent_on_utility_account: true,
            tenant_can_verify_landlord_status: true,
            tenant_requests_to_become_customer: true,
            utility_provided_notice_of_tenant_right: true,
        }
    }

    fn ca_master_metered() -> TenantUtilityAccountInput {
        let mut i = ca_individually_metered_base();
        i.metering_type = UtilityMeteringType::MasterMetered;
        i
    }

    fn ny_base() -> TenantUtilityAccountInput {
        let mut i = ca_individually_metered_base();
        i.regime = Regime::NewYork;
        i
    }

    fn default_base() -> TenantUtilityAccountInput {
        let mut i = ca_individually_metered_base();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn ca_individually_metered_full_request_right_engaged() {
        let r = check(&ca_individually_metered_base());
        assert!(r.tenant_right_to_become_customer_engaged);
        assert!(r.tenant_protected_from_landlord_arrears);
        assert!(r.regime_applies_to_metering_type);
    }

    #[test]
    fn ca_master_metered_not_applies_no_right() {
        let r = check(&ca_master_metered());
        assert!(!r.tenant_right_to_become_customer_engaged);
        assert!(!r.regime_applies_to_metering_type);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 777 does NOT apply to master-metered")));
    }

    #[test]
    fn ca_landlord_not_customer_of_record_no_right() {
        let mut i = ca_individually_metered_base();
        i.landlord_is_customer_of_record = false;
        let r = check(&i);
        assert!(!r.tenant_right_to_become_customer_engaged);
    }

    #[test]
    fn ca_tenant_cannot_verify_landlord_status_no_right() {
        let mut i = ca_individually_metered_base();
        i.tenant_can_verify_landlord_status = false;
        let r = check(&i);
        assert!(!r.tenant_right_to_become_customer_engaged);
    }

    #[test]
    fn ca_tenant_does_not_request_no_right() {
        let mut i = ca_individually_metered_base();
        i.tenant_requests_to_become_customer = false;
        let r = check(&i);
        assert!(!r.tenant_right_to_become_customer_engaged);
    }

    #[test]
    fn ca_utility_failed_to_notify_violation() {
        let mut i = ca_individually_metered_base();
        i.utility_provided_notice_of_tenant_right = false;
        let r = check(&i);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 777") && v.contains("MUST inform residential occupants")));
    }

    #[test]
    fn ca_no_delinquency_no_violation_for_missing_notice() {
        let mut i = ca_individually_metered_base();
        i.landlord_delinquent_on_utility_account = false;
        i.utility_provided_notice_of_tenant_right = false;
        let r = check(&i);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn ca_right_engaged_note_describes_delinquent_amount_protection() {
        let r = check(&ca_individually_metered_base());
        assert!(r.notes.iter().any(
            |n| n.contains("§ 777") && n.contains("NOT required to pay any delinquent amount")
        ));
    }

    #[test]
    fn ca_verification_methods_note_present() {
        let r = check(&ca_individually_metered_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 777") && n.contains("lease, rent receipts")));
    }

    #[test]
    fn ca_citation_pins_777_and_777_1() {
        let r = check(&ca_individually_metered_base());
        assert!(r.citation.contains("§§ 777, 777.1"));
    }

    #[test]
    fn ca_sub_metered_does_not_apply() {
        let mut i = ca_individually_metered_base();
        i.metering_type = UtilityMeteringType::SubMetered;
        let r = check(&i);
        assert!(!r.regime_applies_to_metering_type);
    }

    #[test]
    fn ny_hefpa_individually_metered_right_engaged() {
        let r = check(&ny_base());
        assert!(r.tenant_right_to_become_customer_engaged);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("HEFPA") && n.contains("PSC tariff framework")));
    }

    #[test]
    fn ny_master_metered_does_not_engage_right() {
        let mut i = ny_base();
        i.metering_type = UtilityMeteringType::MasterMetered;
        let r = check(&i);
        assert!(!r.tenant_right_to_become_customer_engaged);
        assert!(!r.regime_applies_to_metering_type);
    }

    #[test]
    fn ny_psc_petition_note_for_shared_meter() {
        let r = check(&ny_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("petition NY PSC for separate account")));
    }

    #[test]
    fn ny_citation_pins_hefpa_sections() {
        let r = check(&ny_base());
        assert!(r.citation.contains("§§ 32, 33, 33-a"));
        assert!(r.citation.contains("HEFPA"));
    }

    #[test]
    fn default_no_statewide_right_no_engagement() {
        let r = check(&default_base());
        assert!(!r.tenant_right_to_become_customer_engaged);
        assert!(!r.tenant_protected_from_landlord_arrears);
    }

    #[test]
    fn default_individually_metered_within_scope_note() {
        let r = check(&default_base());
        assert!(r.regime_applies_to_metering_type);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("individually metered") && n.contains("master-metered buildings")));
    }

    #[test]
    fn default_master_metered_outside_scope() {
        let mut i = default_base();
        i.metering_type = UtilityMeteringType::MasterMetered;
        let r = check(&i);
        assert!(!r.regime_applies_to_metering_type);
    }

    #[test]
    fn default_citation_references_puc_tariff_and_lease() {
        let r = check(&default_base());
        assert!(r.citation.contains("PUC tariff framework"));
        assert!(r.citation.contains("lease provisions"));
    }

    #[test]
    fn ca_unique_explicit_777_protection_invariant() {
        let r_ca = check(&ca_individually_metered_base());
        assert!(r_ca.tenant_protected_from_landlord_arrears);

        let r_default = check(&default_base());
        assert!(!r_default.tenant_protected_from_landlord_arrears);
    }

    #[test]
    fn three_regimes_routed_correctly() {
        for regime in [Regime::California, Regime::NewYork, Regime::Default] {
            let mut i = ca_individually_metered_base();
            i.regime = regime;
            let r = check(&i);
            let _ = r.tenant_right_to_become_customer_engaged;
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn metering_type_truth_table_california() {
        for metering in [
            UtilityMeteringType::IndividuallyMetered,
            UtilityMeteringType::MasterMetered,
            UtilityMeteringType::SubMetered,
        ] {
            let mut i = ca_individually_metered_base();
            i.metering_type = metering;
            let r = check(&i);
            let expected_applies = matches!(metering, UtilityMeteringType::IndividuallyMetered);
            assert_eq!(r.regime_applies_to_metering_type, expected_applies);
        }
    }

    #[test]
    fn ny_sub_metered_outside_master_meter_engages_right() {
        let mut i = ny_base();
        i.metering_type = UtilityMeteringType::SubMetered;
        let r = check(&i);
        assert!(r.regime_applies_to_metering_type);
        assert!(r.tenant_right_to_become_customer_engaged);
    }

    #[test]
    fn ca_master_meter_carveout_unique_invariant() {
        let mut i_ca = ca_master_metered();
        let r_ca = check(&i_ca);
        assert!(!r_ca.regime_applies_to_metering_type);

        i_ca.regime = Regime::NewYork;
        let r_ny = check(&i_ca);
        assert!(!r_ny.regime_applies_to_metering_type);

        i_ca.regime = Regime::Default;
        let r_default = check(&i_ca);
        assert!(!r_default.regime_applies_to_metering_type);
    }

    #[test]
    fn ca_three_conjunctive_elements_required_for_right() {
        let cases: [(bool, bool, bool, bool); 8] = [
            (false, false, false, false),
            (true, false, false, false),
            (false, true, false, false),
            (false, false, true, false),
            (true, true, false, false),
            (true, false, true, false),
            (false, true, true, false),
            (true, true, true, true),
        ];
        for (lord_record, verify, request, expected) in cases {
            let mut i = ca_individually_metered_base();
            i.landlord_is_customer_of_record = lord_record;
            i.tenant_can_verify_landlord_status = verify;
            i.tenant_requests_to_become_customer = request;
            let r = check(&i);
            assert_eq!(
                r.tenant_right_to_become_customer_engaged, expected,
                "landlord_record={} verify={} request={}",
                lord_record, verify, request
            );
        }
    }

    #[test]
    fn ca_777_1_disclosure_note_present() {
        let r = check(&ca_individually_metered_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 777.1") && n.contains("disclosure obligations")));
    }

    #[test]
    fn ca_master_meter_no_violation_even_when_landlord_delinquent() {
        let mut i = ca_master_metered();
        i.landlord_delinquent_on_utility_account = true;
        i.utility_provided_notice_of_tenant_right = false;
        let r = check(&i);
        assert!(r.violations.is_empty());
    }
}
