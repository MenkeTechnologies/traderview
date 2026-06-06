//! Mandatory landlord-paid disclosure of RUBS (Ratio Utility
//! Billing System) allocation methodology. When a landlord
//! allocates utility costs to tenants in master-metered
//! buildings using a RUBS formula (square footage, occupant
//! count, etc.) WITHOUT individual sub-meters, what disclosure
//! and billing-cap requirements attach? Distinct from
//! `submetering_rules` (sub-meter setup), `tenant_utility_
//! account_designation` (direct utility account), and `utility_
//! shutoff`. Trader-landlord operational concern in master-
//! metered multifamily buildings.
//!
//! **Three regimes**:
//!
//! **Texas — Tex. Water Code § 13.502 + § 13.2502 + 16 TAC
//! 24.281**. Most explicit RUBS framework. Lease MUST state that
//! utilities will be allocated using RUBS AND specify the
//! EXACT CALCULATION METHOD. Statutorily approved allocation
//! methods: (1) number of occupants OR (2) square footage.
//! Landlord may NOT add SERVICE CHARGE OR ADMINISTRATIVE FEE
//! to RUBS bill. Landlord may NOT charge tenants in aggregate
//! MORE than what the utility provider bills the entire
//! property. Public Utility Commission of Texas enforcement +
//! private right of action under § 13.503 with civil damages.
//!
//! **District of Columbia — D.C. Code § 42-3502.06A + DC AG
//! Schwalb guidance**. Landlord must clearly identify
//! allocation method in lease + provide annual reconciliation
//! statement showing actual utility costs vs amounts collected.
//! No surcharges permitted. Consumer Protection Procedures Act
//! applies to deceptive RUBS billing.
//!
//! **Default — lease + state PUC tariff + state UDAP**. Most
//! states have no specific RUBS statute. Lease must identify
//! allocation method; common-law unconscionability + state UDAP
//! statutes available for abusive RUBS practices.
//!
//! Citations: Tex. Water Code §§ 13.502 (submetering + RUBS
//! framework), 13.2502 (RUBS lease disclosure + calculation
//! method + no service fees + aggregate cap), 13.503 (private
//! right of action with civil damages); 16 TAC § 24.281
//! (Public Utility Commission of Texas RUBS rules); D.C. Code
//! § 42-3502.06A (DC RUBS framework); DC Consumer Protection
//! Procedures Act; state-specific PUC tariff frameworks.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Texas,
    DistrictOfColumbia,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AllocationMethod {
    /// TX-approved: number of occupants.
    OccupantCount,
    /// TX-approved: unit square footage.
    SquareFootage,
    /// Other / unspecified methodology.
    Other,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RubsUtilityBillingInput {
    pub regime: Regime,
    pub allocation_method: AllocationMethod,
    /// Whether the lease specifies that utilities are allocated
    /// using RUBS (vs separately metered).
    pub lease_states_rubs_allocation: bool,
    /// Whether the lease specifies the EXACT calculation method.
    pub lease_specifies_exact_calculation_method: bool,
    /// Whether the landlord adds a service charge or
    /// administrative fee to the RUBS bill (TX prohibits).
    pub landlord_added_service_or_administrative_fee: bool,
    /// Aggregate amount charged to all tenants combined, in
    /// cents.
    pub aggregate_tenant_charges_cents: i64,
    /// Amount the utility provider billed the entire property,
    /// in cents.
    pub utility_provider_billed_property_cents: i64,
    /// Whether landlord provides annual reconciliation statement
    /// (DC requirement).
    pub annual_reconciliation_statement_provided: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RubsUtilityBillingResult {
    pub compliant: bool,
    pub aggregate_overcharge_engaged: bool,
    pub aggregate_overcharge_amount_cents: i64,
    pub service_fee_violation_engaged: bool,
    pub disclosure_violation_engaged: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RubsUtilityBillingInput) -> RubsUtilityBillingResult {
    match input.regime {
        Regime::Texas => check_texas(input),
        Regime::DistrictOfColumbia => check_district_of_columbia(input),
        Regime::Default => check_default(input),
    }
}

fn check_texas(input: &RubsUtilityBillingInput) -> RubsUtilityBillingResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = vec![
        "Tex. Water Code §§ 13.502 + 13.2502 + 16 TAC § 24.281 — RUBS allocation framework: lease MUST state RUBS allocation + specify exact calculation method (occupant count OR square footage); landlord may NOT add service or administrative fees; aggregate tenant charges cannot exceed utility provider's bill to entire property"
            .to_string(),
        "Tex. Water Code § 13.503 — private right of action with civil damages for RUBS billing violations; Public Utility Commission of Texas enforcement authority"
            .to_string(),
    ];

    if !input.lease_states_rubs_allocation {
        violations.push(
            "Tex. Water Code § 13.2502 — lease MUST state that utilities will be allocated using RUBS"
                .to_string(),
        );
    }

    if !input.lease_specifies_exact_calculation_method {
        violations.push(
            "Tex. Water Code § 13.2502 — lease MUST specify the EXACT calculation method used for RUBS allocation"
                .to_string(),
        );
    }

    if !matches!(
        input.allocation_method,
        AllocationMethod::OccupantCount | AllocationMethod::SquareFootage
    ) {
        violations.push(
            "Tex. Water Code § 13.2502 + 16 TAC § 24.281 — only statutorily-approved RUBS allocation methods are number of occupants OR unit square footage; other methodologies not permitted"
                .to_string(),
        );
    }

    if input.landlord_added_service_or_administrative_fee {
        violations.push(
            "Tex. Water Code § 13.2502 — landlord may NOT add service charge or administrative fee to RUBS bill"
                .to_string(),
        );
    }

    let aggregate_overcharge =
        input.aggregate_tenant_charges_cents - input.utility_provider_billed_property_cents;
    let aggregate_overcharge_engaged = aggregate_overcharge > 0;
    let aggregate_overcharge_amount = aggregate_overcharge.max(0);

    if aggregate_overcharge_engaged {
        violations.push(format!(
            "Tex. Water Code § 13.2502 — aggregate tenant charges (${}) EXCEED utility provider's bill to entire property (${}); overcharge ${}",
            input.aggregate_tenant_charges_cents / 100,
            input.utility_provider_billed_property_cents / 100,
            aggregate_overcharge_amount / 100
        ));
    }

    let lease_method_violation = !input.lease_states_rubs_allocation
        || !input.lease_specifies_exact_calculation_method
        || !matches!(
            input.allocation_method,
            AllocationMethod::OccupantCount | AllocationMethod::SquareFootage
        );

    if input.landlord_added_service_or_administrative_fee {
        notes.push(
            "TX RUBS framework uniquely prohibits service / administrative fees; landlord must absorb administrative cost of allocating + billing"
                .to_string(),
        );
    }

    let compliant = violations.is_empty();
    RubsUtilityBillingResult {
        compliant,
        aggregate_overcharge_engaged,
        aggregate_overcharge_amount_cents: aggregate_overcharge_amount,
        service_fee_violation_engaged: input.landlord_added_service_or_administrative_fee,
        disclosure_violation_engaged: lease_method_violation,
        violations,
        citation: "Tex. Water Code §§ 13.502, 13.2502, 13.503; 16 TAC § 24.281",
        notes,
    }
}

fn check_district_of_columbia(input: &RubsUtilityBillingInput) -> RubsUtilityBillingResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "D.C. Code § 42-3502.06A + DC AG Schwalb guidance — landlord must clearly identify RUBS allocation method in lease + provide ANNUAL RECONCILIATION STATEMENT showing actual utility costs vs amounts collected; no surcharges permitted"
            .to_string(),
        "DC Consumer Protection Procedures Act applies to deceptive RUBS billing practices; private right of action available"
            .to_string(),
    ];

    if !input.lease_states_rubs_allocation || !input.lease_specifies_exact_calculation_method {
        violations.push(
            "D.C. Code § 42-3502.06A — landlord must clearly identify RUBS allocation method in lease"
                .to_string(),
        );
    }

    if !input.annual_reconciliation_statement_provided {
        violations.push(
            "D.C. Code § 42-3502.06A + DC AG Schwalb guidance — landlord must provide ANNUAL RECONCILIATION STATEMENT showing actual utility costs vs amounts collected from tenants"
                .to_string(),
        );
    }

    let aggregate_overcharge =
        input.aggregate_tenant_charges_cents - input.utility_provider_billed_property_cents;
    let aggregate_overcharge_engaged = aggregate_overcharge > 0;
    let aggregate_overcharge_amount = aggregate_overcharge.max(0);

    if input.landlord_added_service_or_administrative_fee {
        violations.push(
            "D.C. Code § 42-3502.06A — no surcharges permitted on RUBS bill; landlord may not add service / administrative fee"
                .to_string(),
        );
    }

    let compliant = violations.is_empty();
    RubsUtilityBillingResult {
        compliant,
        aggregate_overcharge_engaged,
        aggregate_overcharge_amount_cents: aggregate_overcharge_amount,
        service_fee_violation_engaged: input.landlord_added_service_or_administrative_fee,
        disclosure_violation_engaged: !input.lease_states_rubs_allocation
            || !input.lease_specifies_exact_calculation_method,
        violations,
        citation: "D.C. Code § 42-3502.06A; DC Consumer Protection Procedures Act",
        notes,
    }
}

fn check_default(input: &RubsUtilityBillingInput) -> RubsUtilityBillingResult {
    let notes: Vec<String> = vec![
        "default rule — no specific RUBS statute; lease must identify allocation method; common-law unconscionability + state UDAP statutes available for abusive RUBS practices"
            .to_string(),
        "default rule — state PUC tariff framework may impose additional requirements on master-metered utility allocation"
            .to_string(),
    ];

    let aggregate_overcharge =
        input.aggregate_tenant_charges_cents - input.utility_provider_billed_property_cents;
    let aggregate_overcharge_engaged = aggregate_overcharge > 0;

    RubsUtilityBillingResult {
        compliant: true,
        aggregate_overcharge_engaged,
        aggregate_overcharge_amount_cents: aggregate_overcharge.max(0),
        service_fee_violation_engaged: false,
        disclosure_violation_engaged: false,
        violations: Vec::new(),
        citation: "state-specific landlord-tenant statute + PUC tariff + common-law unconscionability + state UDAP",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx_compliant() -> RubsUtilityBillingInput {
        RubsUtilityBillingInput {
            regime: Regime::Texas,
            allocation_method: AllocationMethod::SquareFootage,
            lease_states_rubs_allocation: true,
            lease_specifies_exact_calculation_method: true,
            landlord_added_service_or_administrative_fee: false,
            aggregate_tenant_charges_cents: 100_000,
            utility_provider_billed_property_cents: 100_000,
            annual_reconciliation_statement_provided: false,
        }
    }

    fn dc_compliant() -> RubsUtilityBillingInput {
        let mut i = tx_compliant();
        i.regime = Regime::DistrictOfColumbia;
        i.annual_reconciliation_statement_provided = true;
        i
    }

    fn default_base() -> RubsUtilityBillingInput {
        let mut i = tx_compliant();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn tx_clean_compliance_passes() {
        let r = check(&tx_compliant());
        assert!(r.compliant);
    }

    #[test]
    fn tx_missing_rubs_allocation_statement_violates() {
        let mut i = tx_compliant();
        i.lease_states_rubs_allocation = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.disclosure_violation_engaged);
    }

    #[test]
    fn tx_missing_calculation_method_violates() {
        let mut i = tx_compliant();
        i.lease_specifies_exact_calculation_method = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.disclosure_violation_engaged);
    }

    #[test]
    fn tx_other_allocation_method_violates() {
        let mut i = tx_compliant();
        i.allocation_method = AllocationMethod::Other;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("occupants OR unit square footage")));
    }

    #[test]
    fn tx_occupant_count_allocation_compliant() {
        let mut i = tx_compliant();
        i.allocation_method = AllocationMethod::OccupantCount;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn tx_service_fee_added_violates() {
        let mut i = tx_compliant();
        i.landlord_added_service_or_administrative_fee = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.service_fee_violation_engaged);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("service charge or administrative fee")));
    }

    #[test]
    fn tx_aggregate_overcharge_engaged() {
        let mut i = tx_compliant();
        i.aggregate_tenant_charges_cents = 150_000;
        i.utility_provider_billed_property_cents = 100_000;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.aggregate_overcharge_engaged);
        assert_eq!(r.aggregate_overcharge_amount_cents, 50_000);
    }

    #[test]
    fn tx_no_overcharge_when_equal() {
        let r = check(&tx_compliant());
        assert!(!r.aggregate_overcharge_engaged);
    }

    #[test]
    fn tx_under_billing_no_overcharge() {
        let mut i = tx_compliant();
        i.aggregate_tenant_charges_cents = 80_000;
        i.utility_provider_billed_property_cents = 100_000;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.aggregate_overcharge_engaged);
    }

    #[test]
    fn tx_service_fee_note_describes_administrative_burden() {
        let mut i = tx_compliant();
        i.landlord_added_service_or_administrative_fee = true;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("uniquely prohibits service")
                && n.contains("absorb administrative cost")));
    }

    #[test]
    fn tx_citation_pins_water_code_sections() {
        let r = check(&tx_compliant());
        assert!(r.citation.contains("§§ 13.502, 13.2502, 13.503"));
        assert!(r.citation.contains("16 TAC § 24.281"));
    }

    #[test]
    fn dc_clean_compliance_passes() {
        let r = check(&dc_compliant());
        assert!(r.compliant);
    }

    #[test]
    fn dc_missing_reconciliation_violates() {
        let mut i = dc_compliant();
        i.annual_reconciliation_statement_provided = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("ANNUAL RECONCILIATION STATEMENT")));
    }

    #[test]
    fn dc_service_fee_violates() {
        let mut i = dc_compliant();
        i.landlord_added_service_or_administrative_fee = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 42-3502.06A") && v.contains("no surcharges")));
    }

    #[test]
    fn dc_ag_schwalb_guidance_note_present() {
        let r = check(&dc_compliant());
        assert!(r.notes.iter().any(|n| n.contains("DC AG Schwalb guidance")));
    }

    #[test]
    fn dc_citation_pins_42_3502_06a() {
        let r = check(&dc_compliant());
        assert!(r.citation.contains("§ 42-3502.06A"));
        assert!(r.citation.contains("Consumer Protection Procedures Act"));
    }

    #[test]
    fn default_no_specific_rubs_statute_compliant() {
        let r = check(&default_base());
        assert!(r.compliant);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("no specific RUBS statute")));
    }

    #[test]
    fn default_no_violations_even_with_service_fee() {
        let mut i = default_base();
        i.landlord_added_service_or_administrative_fee = true;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn default_state_udap_note_present() {
        let r = check(&default_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("unconscionability") && n.contains("UDAP")));
    }

    #[test]
    fn tx_uniquely_prohibits_service_fees_invariant() {
        let mut i_tx = tx_compliant();
        i_tx.landlord_added_service_or_administrative_fee = true;
        let r_tx = check(&i_tx);
        assert!(!r_tx.compliant);

        let mut i_default = default_base();
        i_default.landlord_added_service_or_administrative_fee = true;
        let r_default = check(&i_default);
        assert!(r_default.compliant);
    }

    #[test]
    fn dc_uniquely_requires_annual_reconciliation_invariant() {
        let mut i_dc = dc_compliant();
        i_dc.annual_reconciliation_statement_provided = false;
        let r_dc = check(&i_dc);
        assert!(!r_dc.compliant);

        let mut i_tx = tx_compliant();
        i_tx.annual_reconciliation_statement_provided = false;
        let r_tx = check(&i_tx);
        assert!(r_tx.compliant);
    }

    #[test]
    fn three_regimes_routed_correctly() {
        for regime in [Regime::Texas, Regime::DistrictOfColumbia, Regime::Default] {
            let mut i = tx_compliant();
            i.regime = regime;
            i.annual_reconciliation_statement_provided = true;
            let r = check(&i);
            let _ = r.compliant;
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn tx_all_violations_simultaneous() {
        let mut i = tx_compliant();
        i.lease_states_rubs_allocation = false;
        i.lease_specifies_exact_calculation_method = false;
        i.allocation_method = AllocationMethod::Other;
        i.landlord_added_service_or_administrative_fee = true;
        i.aggregate_tenant_charges_cents = 200_000;
        i.utility_provider_billed_property_cents = 100_000;
        let r = check(&i);
        assert!(!r.compliant);
        assert_eq!(r.violations.len(), 5);
    }

    #[test]
    fn tx_clean_no_violations() {
        let r = check(&tx_compliant());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn dc_clean_no_violations() {
        let r = check(&dc_compliant());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn aggregate_overcharge_computed_in_all_regimes() {
        for regime in [Regime::Texas, Regime::DistrictOfColumbia, Regime::Default] {
            let mut i = tx_compliant();
            i.regime = regime;
            i.aggregate_tenant_charges_cents = 200_000;
            i.utility_provider_billed_property_cents = 100_000;
            i.annual_reconciliation_statement_provided = true;
            let r = check(&i);
            assert!(r.aggregate_overcharge_engaged);
            assert_eq!(r.aggregate_overcharge_amount_cents, 100_000);
        }
    }
}
