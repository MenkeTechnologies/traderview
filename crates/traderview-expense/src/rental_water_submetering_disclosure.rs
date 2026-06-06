//! Rental property water submetering disclosure compliance —
//! when a trader-landlord must register with state PUC,
//! disclose submetering arrangement at lease signing, and
//! follow billing-transparency rules before billing tenants
//! separately from rent for water/wastewater service via
//! submeters or RUBS (ratio utility billing system).
//! Trader-landlord operational concern: failure to register
//! and disclose creates PUC enforcement action, tenant
//! refund claims, and statutory damages. Distinct from
//! siblings `rental_hot_water_temperature` (habitability),
//! `rental_bed_bug_disclosure` (lease disclosure), `rental_
//! gas_appliance_ban` (electrification).
//!
//! **Four regimes**:
//!
//! **California — SB 7 of 2016 (Cal. Civ. Code § 1954.201
//! et seq.) + Cal. Public Utilities Code § 739.5**:
//! - **Mandatory** submetering for newly constructed
//!   multiunit and mixed-use structures submitting
//!   application for new water connection after **January
//!   1, 2018**.
//! - Pre-lease written disclosure required before execution
//!   of rental agreement if landlord intends to charge for
//!   water separately from rent.
//! - Disclosure must describe billing method (volumetric vs
//!   allocated), billing frequency, dispute resolution
//!   process.
//! - Service charges and administrative fees disclosed.
//! - Cal. Public Utilities Code § 739.5 governs master-
//!   meter customers serving submetered tenants — tariff
//!   discount passed through.
//!
//! **Texas — Texas Water Code § 13.503 + 16 TAC § 24.275 et
//! seq. (PUCT submetering rules)**:
//! - Property owner using submetered or allocated billing
//!   **must register with PUCT** (Public Utility Commission
//!   of Texas) BEFORE billing tenants.
//! - Tenant guide to submetered service must be provided.
//! - Tenant guide must include billing method, past usage,
//!   dispute rights.
//! - Charges may not exceed actual cost from supply utility
//!   plus reasonable administrative fee.
//! - 30 percent maximum administrative fee cap (16 TAC §
//!   24.275(b)) for water/wastewater services.
//! - Quarterly past-usage disclosure required.
//!
//! **Florida — Florida PSC voluntary framework**:
//! - Submetering encouraged for water conservation but NOT
//!   mandated statewide.
//! - FL PSC guidelines provide best practices.
//! - No statewide PUC registration required (distinct from
//!   Texas).
//! - Pre-lease disclosure recommended by FL PSC but not
//!   statutorily required.
//!
//! **Default — RUBS (Ratio Utility Billing System) general
//! framework**:
//! - 38+ states permit some form of submetering or RUBS.
//! - General requirement: pre-lease disclosure of method +
//!   billing frequency + dispute process.
//! - Generally no maximum administrative fee cap (unlike
//!   Texas 30% cap).
//! - Habitability + just-and-reasonable charges remain
//!   implied warranties.
//!
//! Citations: SB 7 of 2016; Cal. Civ. Code § 1954.201 et
//! seq.; Cal. Public Utilities Code § 739.5; Texas Water
//! Code § 13.503; 16 TAC § 24.275 et seq.; PUCT submetering
//! rules; Florida PSC submetering guidelines.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Texas,
    Florida,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BillingMethod {
    /// Submetering — each unit has a dedicated water meter.
    Submetering,
    /// RUBS — ratio utility billing system based on
    /// allocation formula (occupants, square footage, etc.).
    Rubs,
    /// Master metered — landlord pays utility; no tenant
    /// billing.
    MasterMeteredNoTenantBilling,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalWaterSubmeteringDisclosureInput {
    pub regime: Regime,
    pub billing_method: BillingMethod,
    /// Whether construction water connection application was
    /// submitted after January 1, 2018 (CA SB 7
    /// applicability).
    pub ca_new_construction_post_2018: bool,
    /// Whether building is multiunit or mixed-use (CA SB 7
    /// applicability).
    pub multiunit_or_mixed_use: bool,
    /// Whether pre-lease written disclosure was provided to
    /// tenant.
    pub pre_lease_disclosure_provided: bool,
    /// Whether disclosure describes billing method (CA + TX
    /// + general).
    pub billing_method_disclosed: bool,
    /// Whether disclosure includes billing frequency.
    pub billing_frequency_disclosed: bool,
    /// Whether disclosure includes dispute resolution
    /// process.
    pub dispute_process_disclosed: bool,
    /// Whether landlord registered with state PUC (Texas
    /// requirement).
    pub registered_with_puc: bool,
    /// Whether tenant guide to submetered service was
    /// provided (Texas requirement).
    pub tenant_guide_provided: bool,
    /// Administrative fee assessed as percent of utility
    /// cost (Texas 30% cap).
    pub administrative_fee_percent: u32,
    /// Whether past usage disclosure was provided quarterly
    /// (Texas requirement).
    pub quarterly_past_usage_disclosed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalWaterSubmeteringDisclosureResult {
    pub disclosure_compliant: bool,
    pub pre_lease_disclosure_required: bool,
    pub puc_registration_required: bool,
    pub administrative_fee_within_cap: bool,
    pub ca_submetering_mandatory: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &RentalWaterSubmeteringDisclosureInput,
) -> RentalWaterSubmeteringDisclosureResult {
    match input.regime {
        Regime::California => check_ca(input),
        Regime::Texas => check_tx(input),
        Regime::Florida => check_fl(input),
        Regime::Default => check_default(input),
    }
}

fn check_ca(
    input: &RentalWaterSubmeteringDisclosureInput,
) -> RentalWaterSubmeteringDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Civ. Code § 1954.201 et seq. (SB 7 of 2016) — newly constructed multiunit and mixed-use structures submitting application for new water connection after January 1, 2018 MUST have submeters".to_string(),
        "Cal. Civ. Code § 1954.202 — pre-lease written disclosure required before execution of rental agreement if landlord intends to charge for water separately from rent".to_string(),
        "Cal. Civ. Code § 1954.203 — disclosure must describe billing method (volumetric vs allocated), billing frequency, dispute resolution process, service charges, administrative fees".to_string(),
        "Cal. Public Utilities Code § 739.5 — master-meter customers serving submetered tenants entitled to tariff discount passed through to tenants".to_string(),
        "California regime mandatory submetering for new construction post-January 1, 2018; existing buildings voluntary".to_string(),
    ];

    let ca_mandatory = input.ca_new_construction_post_2018 && input.multiunit_or_mixed_use;

    if ca_mandatory
        && !matches!(input.billing_method, BillingMethod::Submetering)
        && !matches!(
            input.billing_method,
            BillingMethod::MasterMeteredNoTenantBilling
        )
    {
        violations.push(
            "Cal. Civ. Code § 1954.201 (SB 7 of 2016) — submetering MANDATORY for newly constructed multiunit/mixed-use structures with water connection applications after January 1, 2018".to_string(),
        );
    }

    let tenant_billing = !matches!(
        input.billing_method,
        BillingMethod::MasterMeteredNoTenantBilling
    );

    if tenant_billing && !input.pre_lease_disclosure_provided {
        violations.push(
            "Cal. Civ. Code § 1954.202 — pre-lease written disclosure required before execution of rental agreement when landlord intends to charge for water separately from rent".to_string(),
        );
    }

    if tenant_billing && input.pre_lease_disclosure_provided {
        if !input.billing_method_disclosed {
            violations.push(
                "Cal. Civ. Code § 1954.203 — disclosure must describe billing method (volumetric vs allocated)".to_string(),
            );
        }
        if !input.billing_frequency_disclosed {
            violations.push(
                "Cal. Civ. Code § 1954.203 — disclosure must describe billing frequency"
                    .to_string(),
            );
        }
        if !input.dispute_process_disclosed {
            violations.push(
                "Cal. Civ. Code § 1954.203 — disclosure must describe dispute resolution process"
                    .to_string(),
            );
        }
    }

    RentalWaterSubmeteringDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        pre_lease_disclosure_required: tenant_billing,
        puc_registration_required: false,
        administrative_fee_within_cap: true,
        ca_submetering_mandatory: ca_mandatory,
        violations,
        citation:
            "Cal. Civ. Code § 1954.201 et seq. (SB 7 of 2016); Cal. Public Utilities Code § 739.5",
        notes,
    }
}

fn check_tx(
    input: &RentalWaterSubmeteringDisclosureInput,
) -> RentalWaterSubmeteringDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Texas Water Code § 13.503 — property owner using submetered or allocated billing must register with PUCT (Public Utility Commission of Texas) BEFORE billing tenants".to_string(),
        "16 TAC § 24.275 et seq. — tenant guide to submetered service required; must include billing method, past usage, dispute rights".to_string(),
        "16 TAC § 24.275(b) — administrative fee for water/wastewater capped at 30 percent of utility cost".to_string(),
        "16 TAC § 24.275 — charges may not exceed actual cost from supply utility plus reasonable administrative fee".to_string(),
        "16 TAC § 24.275 — quarterly past-usage disclosure required".to_string(),
    ];

    let tenant_billing = !matches!(
        input.billing_method,
        BillingMethod::MasterMeteredNoTenantBilling
    );

    if tenant_billing && !input.registered_with_puc {
        violations.push(
            "Texas Water Code § 13.503 — property owner must register with PUCT BEFORE billing tenants for submetered or allocated water service".to_string(),
        );
    }

    if tenant_billing && !input.tenant_guide_provided {
        violations.push(
            "16 TAC § 24.275 — tenant guide to submetered service required; must include billing method, past usage, dispute rights".to_string(),
        );
    }

    let fee_within_cap = input.administrative_fee_percent <= 30;
    if tenant_billing && !fee_within_cap {
        violations.push(
            "16 TAC § 24.275(b) — administrative fee for water/wastewater capped at 30 percent of utility cost".to_string(),
        );
    }

    if tenant_billing && !input.quarterly_past_usage_disclosed {
        violations.push("16 TAC § 24.275 — quarterly past-usage disclosure required".to_string());
    }

    RentalWaterSubmeteringDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        pre_lease_disclosure_required: tenant_billing,
        puc_registration_required: tenant_billing,
        administrative_fee_within_cap: fee_within_cap,
        ca_submetering_mandatory: false,
        violations,
        citation: "Texas Water Code § 13.503; 16 TAC § 24.275 et seq.",
        notes,
    }
}

fn check_fl(
    input: &RentalWaterSubmeteringDisclosureInput,
) -> RentalWaterSubmeteringDisclosureResult {
    let violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Florida Public Service Commission — submetering encouraged for water conservation but NOT mandated statewide".to_string(),
        "Florida PSC guidelines provide best practices for billing transparency, billing frequency, dispute resolution".to_string(),
        "Florida regime distinct from Texas — no statewide PUC registration required".to_string(),
        "Florida regime distinct from California — no mandatory submetering even for new construction".to_string(),
        "Pre-lease disclosure recommended by FL PSC but not statutorily required".to_string(),
    ];

    let tenant_billing = !matches!(
        input.billing_method,
        BillingMethod::MasterMeteredNoTenantBilling
    );

    RentalWaterSubmeteringDisclosureResult {
        disclosure_compliant: true,
        pre_lease_disclosure_required: false,
        puc_registration_required: false,
        administrative_fee_within_cap: true,
        ca_submetering_mandatory: false,
        violations,
        citation: "Florida PSC submetering guidelines (voluntary framework); see § 627.7011 (water utility regulation)",
        notes: {
            let mut n = notes;
            if tenant_billing {
                n.push(
                    "Florida tenant billing — voluntary best practices apply; landlord-tenant common-law warranty of habitability + just-and-reasonable charges implied".to_string(),
                );
            }
            n
        },
    }
}

fn check_default(
    input: &RentalWaterSubmeteringDisclosureInput,
) -> RentalWaterSubmeteringDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Default RUBS (Ratio Utility Billing System) general framework — 38+ states permit some form of submetering or RUBS".to_string(),
        "Default — pre-lease disclosure of billing method + billing frequency + dispute process required by most state utility commissions or implied through landlord-tenant statutes".to_string(),
        "Default — generally no maximum administrative fee cap (unlike Texas 30% cap)".to_string(),
        "Default — habitability + just-and-reasonable charges remain implied warranties of state landlord-tenant law".to_string(),
        "Default — verify local jurisdiction PUC requirements + local landlord-tenant statutes for state-specific disclosure obligations".to_string(),
    ];

    let tenant_billing = !matches!(
        input.billing_method,
        BillingMethod::MasterMeteredNoTenantBilling
    );

    if tenant_billing && !input.pre_lease_disclosure_provided {
        violations.push(
            "Default — pre-lease disclosure of billing method + billing frequency + dispute process recommended; verify state-specific PUC rules and landlord-tenant statutes".to_string(),
        );
    }

    RentalWaterSubmeteringDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        pre_lease_disclosure_required: tenant_billing,
        puc_registration_required: false,
        administrative_fee_within_cap: true,
        ca_submetering_mandatory: false,
        violations,
        citation: "Default state PUC submetering / RUBS rules; verify local jurisdiction",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_clean() -> RentalWaterSubmeteringDisclosureInput {
        RentalWaterSubmeteringDisclosureInput {
            regime: Regime::California,
            billing_method: BillingMethod::Submetering,
            ca_new_construction_post_2018: true,
            multiunit_or_mixed_use: true,
            pre_lease_disclosure_provided: true,
            billing_method_disclosed: true,
            billing_frequency_disclosed: true,
            dispute_process_disclosed: true,
            registered_with_puc: false,
            tenant_guide_provided: false,
            administrative_fee_percent: 0,
            quarterly_past_usage_disclosed: false,
        }
    }

    fn tx_clean() -> RentalWaterSubmeteringDisclosureInput {
        let mut i = ca_clean();
        i.regime = Regime::Texas;
        i.registered_with_puc = true;
        i.tenant_guide_provided = true;
        i.administrative_fee_percent = 20;
        i.quarterly_past_usage_disclosed = true;
        i
    }

    fn fl_clean() -> RentalWaterSubmeteringDisclosureInput {
        let mut i = ca_clean();
        i.regime = Regime::Florida;
        i
    }

    fn default_clean() -> RentalWaterSubmeteringDisclosureInput {
        let mut i = ca_clean();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn ca_clean_compliant() {
        let r = check(&ca_clean());
        assert!(r.disclosure_compliant);
        assert!(r.ca_submetering_mandatory);
    }

    #[test]
    fn ca_new_construction_post_2018_must_have_submeters() {
        let mut i = ca_clean();
        i.billing_method = BillingMethod::Rubs;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1954.201") && v.contains("MANDATORY")));
    }

    #[test]
    fn ca_old_construction_rubs_no_mandatory_violation() {
        let mut i = ca_clean();
        i.ca_new_construction_post_2018 = false;
        i.billing_method = BillingMethod::Rubs;
        let r = check(&i);
        assert!(r.disclosure_compliant);
        assert!(!r.ca_submetering_mandatory);
    }

    #[test]
    fn ca_no_pre_lease_disclosure_violation() {
        let mut i = ca_clean();
        i.pre_lease_disclosure_provided = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1954.202") && v.contains("pre-lease written disclosure")));
    }

    #[test]
    fn ca_missing_billing_method_disclosure_violation() {
        let mut i = ca_clean();
        i.billing_method_disclosed = false;
        let r = check(&i);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1954.203") && v.contains("billing method")));
    }

    #[test]
    fn ca_missing_billing_frequency_disclosure_violation() {
        let mut i = ca_clean();
        i.billing_frequency_disclosed = false;
        let r = check(&i);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1954.203") && v.contains("billing frequency")));
    }

    #[test]
    fn ca_missing_dispute_process_disclosure_violation() {
        let mut i = ca_clean();
        i.dispute_process_disclosed = false;
        let r = check(&i);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1954.203") && v.contains("dispute resolution")));
    }

    #[test]
    fn ca_master_metered_no_billing_no_disclosure_required() {
        let mut i = ca_clean();
        i.billing_method = BillingMethod::MasterMeteredNoTenantBilling;
        i.pre_lease_disclosure_provided = false;
        let r = check(&i);
        assert!(r.disclosure_compliant);
        assert!(!r.pre_lease_disclosure_required);
    }

    #[test]
    fn tx_clean_compliant() {
        let r = check(&tx_clean());
        assert!(r.disclosure_compliant);
        assert!(r.puc_registration_required);
        assert!(r.administrative_fee_within_cap);
    }

    #[test]
    fn tx_no_puc_registration_violation() {
        let mut i = tx_clean();
        i.registered_with_puc = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 13.503") && v.contains("register with PUCT")));
    }

    #[test]
    fn tx_no_tenant_guide_violation() {
        let mut i = tx_clean();
        i.tenant_guide_provided = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 24.275") && v.contains("tenant guide")));
    }

    #[test]
    fn tx_30_percent_admin_fee_boundary_compliant() {
        let mut i = tx_clean();
        i.administrative_fee_percent = 30;
        let r = check(&i);
        assert!(r.administrative_fee_within_cap);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn tx_31_percent_admin_fee_violation() {
        let mut i = tx_clean();
        i.administrative_fee_percent = 31;
        let r = check(&i);
        assert!(!r.administrative_fee_within_cap);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 24.275(b)") && v.contains("30 percent")));
    }

    #[test]
    fn tx_no_quarterly_past_usage_violation() {
        let mut i = tx_clean();
        i.quarterly_past_usage_disclosed = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("quarterly past-usage")));
    }

    #[test]
    fn tx_master_metered_no_billing_no_puc_registration_required() {
        let mut i = tx_clean();
        i.billing_method = BillingMethod::MasterMeteredNoTenantBilling;
        i.registered_with_puc = false;
        i.tenant_guide_provided = false;
        let r = check(&i);
        assert!(r.disclosure_compliant);
        assert!(!r.puc_registration_required);
    }

    #[test]
    fn fl_voluntary_framework_no_violation() {
        let r = check(&fl_clean());
        assert!(r.disclosure_compliant);
        assert!(!r.pre_lease_disclosure_required);
        assert!(!r.puc_registration_required);
    }

    #[test]
    fn default_no_pre_lease_disclosure_violation() {
        let mut i = default_clean();
        i.pre_lease_disclosure_provided = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Default") && v.contains("pre-lease disclosure")));
    }

    #[test]
    fn default_clean_compliant() {
        let r = check(&default_clean());
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn citation_pins_ca_authority() {
        let r = check(&ca_clean());
        assert!(r.citation.contains("§ 1954.201"));
        assert!(r.citation.contains("SB 7 of 2016"));
        assert!(r.citation.contains("§ 739.5"));
    }

    #[test]
    fn citation_pins_tx_authority() {
        let r = check(&tx_clean());
        assert!(r.citation.contains("Texas Water Code § 13.503"));
        assert!(r.citation.contains("16 TAC § 24.275"));
    }

    #[test]
    fn citation_pins_fl_authority() {
        let r = check(&fl_clean());
        assert!(r.citation.contains("Florida PSC"));
        assert!(r.citation.contains("voluntary"));
    }

    #[test]
    fn citation_pins_default_authority() {
        let r = check(&default_clean());
        assert!(r.citation.contains("Default state PUC"));
        assert!(r.citation.contains("local jurisdiction"));
    }

    #[test]
    fn note_pins_ca_january_2018_effective_date() {
        let r = check(&ca_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("January 1, 2018") && n.contains("SB 7 of 2016")));
    }

    #[test]
    fn note_pins_tx_30_percent_cap() {
        let r = check(&tx_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("30 percent") && n.contains("§ 24.275(b)")));
    }

    #[test]
    fn note_pins_fl_voluntary_framework() {
        let r = check(&fl_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("voluntary") || n.contains("NOT mandated")));
    }

    #[test]
    fn note_pins_default_rubs_framework() {
        let r = check(&default_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("RUBS") && n.contains("38+")));
    }

    #[test]
    fn ca_uniquely_requires_mandatory_submetering_invariant() {
        let r_ca = check(&ca_clean());
        let r_tx = check(&tx_clean());
        let r_fl = check(&fl_clean());
        let r_def = check(&default_clean());
        assert!(r_ca.ca_submetering_mandatory);
        assert!(!r_tx.ca_submetering_mandatory);
        assert!(!r_fl.ca_submetering_mandatory);
        assert!(!r_def.ca_submetering_mandatory);
    }

    #[test]
    fn tx_uniquely_requires_puc_registration_invariant() {
        let r_ca = check(&ca_clean());
        let r_tx = check(&tx_clean());
        let r_fl = check(&fl_clean());
        let r_def = check(&default_clean());
        assert!(r_tx.puc_registration_required);
        assert!(!r_ca.puc_registration_required);
        assert!(!r_fl.puc_registration_required);
        assert!(!r_def.puc_registration_required);
    }

    #[test]
    fn fl_least_stringent_no_pre_lease_default_invariant() {
        let r_ca = check(&ca_clean());
        let r_tx = check(&tx_clean());
        let r_fl = check(&fl_clean());
        let r_def = check(&default_clean());
        assert!(r_ca.pre_lease_disclosure_required);
        assert!(r_tx.pre_lease_disclosure_required);
        assert!(!r_fl.pre_lease_disclosure_required);
        assert!(r_def.pre_lease_disclosure_required);
    }

    #[test]
    fn billing_method_truth_table_for_ca() {
        for (method, mandatory_violation_expected) in [
            (BillingMethod::Submetering, false),
            (BillingMethod::Rubs, true),
            (BillingMethod::MasterMeteredNoTenantBilling, false),
        ] {
            let mut i = ca_clean();
            i.billing_method = method;
            let r = check(&i);
            let has_mandatory_violation = r
                .violations
                .iter()
                .any(|v| v.contains("§ 1954.201") && v.contains("MANDATORY"));
            assert_eq!(
                has_mandatory_violation, mandatory_violation_expected,
                "method={:?} expected mandatory violation={}",
                method, mandatory_violation_expected
            );
        }
    }

    #[test]
    fn multiple_tx_violations_stack() {
        let mut i = tx_clean();
        i.registered_with_puc = false;
        i.tenant_guide_provided = false;
        i.administrative_fee_percent = 50;
        i.quarterly_past_usage_disclosed = false;
        let r = check(&i);
        assert_eq!(r.violations.len(), 4);
    }
}
