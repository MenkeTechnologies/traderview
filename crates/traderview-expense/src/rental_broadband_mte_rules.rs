//! FCC Multiple Tenant Environment (MTE) broadband-access
//! rules — when may a trader-landlord lawfully enter into a
//! contract with an internet service provider (ISP) granting
//! exclusive access + revenue sharing + bulk billing
//! arrangements for residential or commercial multifamily
//! buildings? Trader-landlord critical for any multifamily
//! property considering ISP partnership: an exclusive
//! contract OR an exclusive revenue-sharing agreement OR a
//! graduated revenue-sharing agreement for broadband-only is
//! categorically PROHIBITED under 47 CFR § 64.2500 et seq.
//! and the FCC's 2024 Open Internet Order.
//!
//! Distinct from siblings `tenant_data_privacy` (data
//! handling), `tenant_organizing` (tenant-association
//! formation), `landlord_identification_disclosure`, and
//! `lease_disclosures`.
//!
//! **Two regimes**:
//!
//! **Federal FCC (47 CFR § 64.2500 et seq. + Open Internet
//! Order FCC 24-52, July 2024)**:
//! - Exclusive access contracts categorically PROHIBITED
//!   for telecom + cable + broadband providers (47 CFR §
//!   64.2500-64.2503).
//! - Exclusive revenue-sharing agreements PROHIBITED (one
//!   provider exclusively shares revenue with landlord).
//! - Graduated revenue-sharing agreements for broadband-only
//!   providers PROHIBITED (Open Internet Order 2024).
//! - Flat licensing fees PERMITTED (not tied to revenue).
//! - Bulk billing arrangements PERMITTED (FCC 2010 + FCC
//!   withdrew 2024 proposed ban January 2025 per Chairman
//!   Carr).
//!
//! **Default state law** — common-law lease analysis governs
//! when FCC rules silent; some states have additional
//! protections (CA SB 1130 broadband disclosure, NY HSTPA).
//!
//! Citations: 47 CFR § 64.2500-64.2503 (FCC MTE rules); FCC
//! Open Internet Order (FCC 24-52, July 2024); 47 USC § 224
//! (pole attachments); FCC 22-12 (Multiple Tenant
//! Environment R&O); FCC 10-49 (2010 bulk billing
//! confirmation); FCC Chairman Carr January 2025 withdrawal
//! of 2024 bulk-billing-ban proposal.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    FederalFcc,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    /// Traditional telecommunications carrier (covered by 47
    /// CFR § 64.2500).
    Telecommunications,
    /// Cable provider (covered by 47 CFR § 64.2503).
    Cable,
    /// Broadband-only provider (covered by 2024 Open Internet
    /// Order).
    BroadbandOnly,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContractArrangement {
    /// Exclusive access contract — landlord grants ONE
    /// provider exclusive right to serve building.
    ExclusiveAccess,
    /// Exclusive revenue-sharing — landlord receives revenue
    /// share AND agrees not to enter similar arrangements
    /// with other providers.
    ExclusiveRevenueSharing,
    /// Graduated revenue-sharing — landlord revenue share
    /// scales with provider revenue (not flat fee).
    GraduatedRevenueSharing,
    /// Flat licensing fee — fixed-amount fee not tied to
    /// revenue.
    FlatLicensingFee,
    /// Bulk billing — provider serves every tenant; tenants
    /// billed prorated share.
    BulkBilling,
    /// Non-exclusive marketing access — landlord allows
    /// multiple providers.
    NonExclusiveMarketing,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalBroadbandMteRulesInput {
    pub regime: Regime,
    pub provider_type: ProviderType,
    pub arrangement: ContractArrangement,
    /// Whether the property is a multiple tenant environment
    /// (MTE) — residential or commercial multifamily.
    pub is_multi_tenant_environment: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalBroadbandMteRulesResult {
    pub arrangement_lawful: bool,
    pub mte_rules_apply: bool,
    pub fcc_categorical_prohibition_engaged: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RentalBroadbandMteRulesInput) -> RentalBroadbandMteRulesResult {
    match input.regime {
        Regime::FederalFcc => check_federal_fcc(input),
        Regime::Default => check_default(input),
    }
}

fn check_federal_fcc(input: &RentalBroadbandMteRulesInput) -> RentalBroadbandMteRulesResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "47 CFR § 64.2500-64.2503 — exclusive access contracts between landlords and telecom + cable + broadband providers categorically PROHIBITED in multiple tenant environments"
            .to_string(),
        "47 CFR § 64.2500 — exclusive revenue-sharing agreements PROHIBITED (provider pays landlord in exchange for access AND landlord agrees not to enter similar arrangements with other providers)"
            .to_string(),
        "FCC Open Internet Order (FCC 24-52, July 2024) — graduated revenue-sharing agreements for broadband-only providers PROHIBITED in MTEs; flat licensing fees not tied to revenue PERMITTED"
            .to_string(),
        "FCC 10-49 (2010 confirmation) + FCC Chairman Carr January 2025 withdrawal of 2024 proposed bulk-billing ban — bulk billing arrangements REMAIN PERMITTED in MTEs"
            .to_string(),
    ];

    if !input.is_multi_tenant_environment {
        return RentalBroadbandMteRulesResult {
            arrangement_lawful: true,
            mte_rules_apply: false,
            fcc_categorical_prohibition_engaged: false,
            violations,
            citation: "47 CFR § 64.2500-64.2503; FCC Open Internet Order (FCC 24-52, July 2024); FCC 22-12; FCC 10-49",
            notes,
        };
    }

    let categorical_prohibition = matches!(
        input.arrangement,
        ContractArrangement::ExclusiveAccess | ContractArrangement::ExclusiveRevenueSharing
    );

    let broadband_only_graduated_prohibited =
        matches!(input.provider_type, ProviderType::BroadbandOnly)
            && matches!(
                input.arrangement,
                ContractArrangement::GraduatedRevenueSharing
            );

    if categorical_prohibition {
        violations.push(format!(
            "47 CFR § 64.2500 — {:?} categorically PROHIBITED between landlord and provider in MTE",
            input.arrangement
        ));
    }

    if broadband_only_graduated_prohibited {
        violations.push(
            "FCC Open Internet Order (FCC 24-52, July 2024) — graduated revenue-sharing agreements between landlord and broadband-only provider PROHIBITED in MTEs".to_string(),
        );
    }

    RentalBroadbandMteRulesResult {
        arrangement_lawful: violations.is_empty(),
        mte_rules_apply: true,
        fcc_categorical_prohibition_engaged: categorical_prohibition
            || broadband_only_graduated_prohibited,
        violations,
        citation: "47 CFR § 64.2500-64.2503; FCC Open Internet Order (FCC 24-52, July 2024); FCC 22-12; FCC 10-49",
        notes,
    }
}

fn check_default(_input: &RentalBroadbandMteRulesInput) -> RentalBroadbandMteRulesResult {
    let notes: Vec<String> = vec![
        "default state law — common-law lease analysis governs when FCC MTE rules silent"
            .to_string(),
        "some states have additional broadband disclosure obligations (CA SB 1130 broadband disclosure; NY HSTPA 2019 broadband access provisions); verify state-specific rules before relying on default"
            .to_string(),
    ];

    RentalBroadbandMteRulesResult {
        arrangement_lawful: true,
        mte_rules_apply: false,
        fcc_categorical_prohibition_engaged: false,
        violations: Vec::new(),
        citation: "common-law lease analysis + state-specific broadband disclosure (CA SB 1130; NY HSTPA 2019)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fcc_lawful_base() -> RentalBroadbandMteRulesInput {
        RentalBroadbandMteRulesInput {
            regime: Regime::FederalFcc,
            provider_type: ProviderType::BroadbandOnly,
            arrangement: ContractArrangement::FlatLicensingFee,
            is_multi_tenant_environment: true,
        }
    }

    fn default_base() -> RentalBroadbandMteRulesInput {
        let mut i = fcc_lawful_base();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn fcc_flat_licensing_fee_lawful() {
        let r = check(&fcc_lawful_base());
        assert!(r.arrangement_lawful);
        assert!(r.mte_rules_apply);
        assert!(!r.fcc_categorical_prohibition_engaged);
    }

    #[test]
    fn fcc_bulk_billing_lawful() {
        let mut i = fcc_lawful_base();
        i.arrangement = ContractArrangement::BulkBilling;
        let r = check(&i);
        assert!(r.arrangement_lawful);
    }

    #[test]
    fn fcc_non_exclusive_marketing_lawful() {
        let mut i = fcc_lawful_base();
        i.arrangement = ContractArrangement::NonExclusiveMarketing;
        let r = check(&i);
        assert!(r.arrangement_lawful);
    }

    #[test]
    fn fcc_exclusive_access_prohibited() {
        let mut i = fcc_lawful_base();
        i.arrangement = ContractArrangement::ExclusiveAccess;
        let r = check(&i);
        assert!(!r.arrangement_lawful);
        assert!(r.fcc_categorical_prohibition_engaged);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 64.2500") && v.contains("ExclusiveAccess")));
    }

    #[test]
    fn fcc_exclusive_revenue_sharing_prohibited() {
        let mut i = fcc_lawful_base();
        i.arrangement = ContractArrangement::ExclusiveRevenueSharing;
        let r = check(&i);
        assert!(!r.arrangement_lawful);
        assert!(r.fcc_categorical_prohibition_engaged);
    }

    #[test]
    fn fcc_broadband_only_graduated_revenue_sharing_prohibited() {
        let mut i = fcc_lawful_base();
        i.provider_type = ProviderType::BroadbandOnly;
        i.arrangement = ContractArrangement::GraduatedRevenueSharing;
        let r = check(&i);
        assert!(!r.arrangement_lawful);
        assert!(r.violations.iter().any(|v| v.contains("FCC 24-52")
            && v.contains("graduated revenue-sharing")
            && v.contains("broadband-only")));
    }

    #[test]
    fn fcc_telecom_graduated_revenue_sharing_lawful() {
        let mut i = fcc_lawful_base();
        i.provider_type = ProviderType::Telecommunications;
        i.arrangement = ContractArrangement::GraduatedRevenueSharing;
        let r = check(&i);
        assert!(r.arrangement_lawful);
    }

    #[test]
    fn fcc_cable_graduated_revenue_sharing_lawful() {
        let mut i = fcc_lawful_base();
        i.provider_type = ProviderType::Cable;
        i.arrangement = ContractArrangement::GraduatedRevenueSharing;
        let r = check(&i);
        assert!(r.arrangement_lawful);
    }

    #[test]
    fn fcc_single_tenant_not_subject_to_mte_rules() {
        let mut i = fcc_lawful_base();
        i.is_multi_tenant_environment = false;
        i.arrangement = ContractArrangement::ExclusiveAccess;
        let r = check(&i);
        assert!(r.arrangement_lawful);
        assert!(!r.mte_rules_apply);
    }

    #[test]
    fn fcc_citation_pins_authorities() {
        let r = check(&fcc_lawful_base());
        assert!(r.citation.contains("§ 64.2500-64.2503"));
        assert!(r.citation.contains("FCC 24-52"));
        assert!(r.citation.contains("July 2024"));
        assert!(r.citation.contains("FCC 22-12"));
        assert!(r.citation.contains("FCC 10-49"));
    }

    #[test]
    fn fcc_note_pins_exclusive_access_prohibition() {
        let r = check(&fcc_lawful_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 64.2500-64.2503") && n.contains("categorically PROHIBITED")));
    }

    #[test]
    fn fcc_note_pins_exclusive_revenue_sharing_prohibition() {
        let r = check(&fcc_lawful_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 64.2500") && n.contains("exclusive revenue-sharing")));
    }

    #[test]
    fn fcc_note_pins_open_internet_order_broadband_only() {
        let r = check(&fcc_lawful_base());
        assert!(r.notes.iter().any(|n| n.contains("FCC 24-52")
            && n.contains("graduated revenue-sharing")
            && n.contains("broadband-only")
            && n.contains("flat licensing fees")));
    }

    #[test]
    fn fcc_note_pins_bulk_billing_carr_withdrawal() {
        let r = check(&fcc_lawful_base());
        assert!(r.notes.iter().any(|n| n.contains("FCC 10-49")
            && n.contains("Carr")
            && n.contains("January 2025")
            && n.contains("REMAIN PERMITTED")));
    }

    #[test]
    fn default_no_fcc_rules_apply() {
        let mut i = default_base();
        i.arrangement = ContractArrangement::ExclusiveAccess;
        let r = check(&i);
        assert!(r.arrangement_lawful);
        assert!(!r.mte_rules_apply);
    }

    #[test]
    fn default_citation_pins_state_disclosure() {
        let r = check(&default_base());
        assert!(r.citation.contains("CA SB 1130"));
        assert!(r.citation.contains("NY HSTPA 2019"));
    }

    #[test]
    fn two_regimes_routed_correctly() {
        for regime in [Regime::FederalFcc, Regime::Default] {
            let mut i = fcc_lawful_base();
            i.regime = regime;
            let r = check(&i);
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn arrangement_truth_table_fcc_mte_broadband() {
        for (arrangement, exp_lawful) in [
            (ContractArrangement::ExclusiveAccess, false),
            (ContractArrangement::ExclusiveRevenueSharing, false),
            (ContractArrangement::GraduatedRevenueSharing, false),
            (ContractArrangement::FlatLicensingFee, true),
            (ContractArrangement::BulkBilling, true),
            (ContractArrangement::NonExclusiveMarketing, true),
        ] {
            let mut i = fcc_lawful_base();
            i.arrangement = arrangement;
            i.provider_type = ProviderType::BroadbandOnly;
            let r = check(&i);
            assert_eq!(r.arrangement_lawful, exp_lawful);
        }
    }

    #[test]
    fn provider_type_distinguishes_graduated_revenue_sharing() {
        for (provider, exp_lawful) in [
            (ProviderType::Telecommunications, true),
            (ProviderType::Cable, true),
            (ProviderType::BroadbandOnly, false),
        ] {
            let mut i = fcc_lawful_base();
            i.provider_type = provider;
            i.arrangement = ContractArrangement::GraduatedRevenueSharing;
            let r = check(&i);
            assert_eq!(r.arrangement_lawful, exp_lawful);
        }
    }

    #[test]
    fn fcc_uniquely_engages_mte_rules_invariant() {
        let r_fcc = check(&fcc_lawful_base());
        assert!(r_fcc.mte_rules_apply);

        let r_default = check(&default_base());
        assert!(!r_default.mte_rules_apply);
    }

    #[test]
    fn fcc_mte_required_for_prohibition_invariant() {
        let mut i_mte = fcc_lawful_base();
        i_mte.is_multi_tenant_environment = true;
        i_mte.arrangement = ContractArrangement::ExclusiveAccess;
        let r_mte = check(&i_mte);
        assert!(!r_mte.arrangement_lawful);

        let mut i_no_mte = fcc_lawful_base();
        i_no_mte.is_multi_tenant_environment = false;
        i_no_mte.arrangement = ContractArrangement::ExclusiveAccess;
        let r_no_mte = check(&i_no_mte);
        assert!(r_no_mte.arrangement_lawful);
    }

    #[test]
    fn fcc_categorical_prohibition_invariant_for_exclusive_access_and_exclusive_revenue() {
        for arrangement in [
            ContractArrangement::ExclusiveAccess,
            ContractArrangement::ExclusiveRevenueSharing,
        ] {
            for provider in [
                ProviderType::Telecommunications,
                ProviderType::Cable,
                ProviderType::BroadbandOnly,
            ] {
                let mut i = fcc_lawful_base();
                i.arrangement = arrangement;
                i.provider_type = provider;
                let r = check(&i);
                assert!(!r.arrangement_lawful);
                assert!(r.fcc_categorical_prohibition_engaged);
            }
        }
    }

    #[test]
    fn fcc_flat_licensing_fee_lawful_for_all_provider_types() {
        for provider in [
            ProviderType::Telecommunications,
            ProviderType::Cable,
            ProviderType::BroadbandOnly,
        ] {
            let mut i = fcc_lawful_base();
            i.provider_type = provider;
            i.arrangement = ContractArrangement::FlatLicensingFee;
            let r = check(&i);
            assert!(r.arrangement_lawful);
        }
    }

    #[test]
    fn fcc_bulk_billing_lawful_for_all_provider_types() {
        for provider in [
            ProviderType::Telecommunications,
            ProviderType::Cable,
            ProviderType::BroadbandOnly,
        ] {
            let mut i = fcc_lawful_base();
            i.provider_type = provider;
            i.arrangement = ContractArrangement::BulkBilling;
            let r = check(&i);
            assert!(r.arrangement_lawful);
        }
    }
}
