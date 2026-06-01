//! Tenant Rights Statement disclosure — when must a residential
//! landlord distribute the official state-prepared statement of
//! tenants' rights and responsibilities? Distinct from
//! `lease_disclosures` (mandated lease content), `landlord_
//! identification_disclosure` (party-identification), and
//! `plain_language_lease` (lease readability).
//!
//! Trader-landlord operational concern in just-cause-style
//! jurisdictions — failure to distribute the official statement
//! exposes landlord to statutory penalties + private right of
//! action under state consumer-protection / landlord-tenant
//! frameworks.
//!
//! **Four regimes:**
//!
//! **New Jersey — Truth in Renting Act, N.J.S.A. 46:8-43 to -50
//! (most aggressive)**. NJ Department of Community Affairs (DCA)
//! must publish a Statement of Legal Rights and Responsibilities
//! of Tenants and Landlords of Rental Dwelling Units in BOTH
//! English and Spanish, posted on DCA website, UPDATED ANNUALLY.
//! Landlord MUST: (1) distribute one copy to each tenant within
//! 30 DAYS after DCA publication; (2) provide copy to each new
//! tenant at or prior to occupancy; (3) post current statement in
//! prominent accessible location. Applies to buildings with
//! MORE THAN 2 UNITS (or non-owner-occupied buildings with more
//! than 3 units). **No tenant waiver or refusal alters landlord
//! obligations**.
//!
//! **Maryland — Md. Code, Real Property § 8-208 (Tenant Bill of
//! Rights notice)**. Limited disclosure requirement; landlord
//! must provide certain tenant-rights notices but not the
//! comprehensive Statement-of-Rights document NJ requires.
//!
//! **New York — DHCR Residential Tenants' Rights Guide + AG
//! Residential Tenants' Rights Guide**. No statutory annual-
//! distribution mandate equivalent to NJ. Landlord may
//! voluntarily distribute AG guide. HSTPA of 2019 imposes
//! discrete-disclosure mandates (rent stabilization status,
//! security deposit handling, etc.) but no comprehensive
//! Statement-of-Rights distribution requirement.
//!
//! **Default — no obligation**. Most states have no statewide
//! Tenant-Rights-Statement distribution mandate; municipal
//! ordinances may impose distribution requirements (e.g.,
//! Chicago Tenants in Foreclosure Notification Ordinance,
//! San Francisco Rent Ordinance § 37).
//!
//! Citations: N.J.S.A. 46:8-43 (NJ Truth in Renting Act
//! definitions); § 46:8-44 (DCA Statement preparation +
//! bilingual); § 46:8-45 (landlord distribution + 30-day rule +
//! new-tenant rule); § 46:8-46 (posting requirement); § 46:8-47
//! (applicability — 2+ unit buildings); § 46:8-48 (tenant cannot
//! waive); § 46:8-49 (penalties); § 46:8-50 (severability);
//! Md. Code, Real Property § 8-208; HSTPA of 2019 (NY discrete
//! disclosures).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    NewJersey,
    Maryland,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantRightsStatementInput {
    pub regime: Regime,
    /// Number of dwelling units in the building (NJ scoping
    /// rule: > 2 units, or non-owner-occupied > 3 units).
    pub building_unit_count: u32,
    /// Whether the building is owner-occupied (NJ scoping rule).
    pub owner_occupied_building: bool,
    /// Whether the landlord distributed the Statement of Tenants'
    /// Rights to existing tenants within 30 days of DCA
    /// publication (NJ Truth in Renting Act).
    pub statement_distributed_within_30_days_of_publication: bool,
    /// Whether the landlord delivered the statement to each new
    /// tenant at or prior to occupancy.
    pub statement_delivered_to_new_tenant_at_occupancy: bool,
    /// Whether the landlord posted the current statement in a
    /// prominent accessible location.
    pub statement_posted_in_prominent_location: bool,
    /// Whether the landlord is relying on a tenant's purported
    /// waiver or refusal to receive the statement.
    pub relying_on_tenant_waiver: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TenantRightsStatementResult {
    pub compliant: bool,
    /// Whether the building is within the scope of the regime's
    /// distribution requirement.
    pub regime_applies_to_building: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &TenantRightsStatementInput) -> TenantRightsStatementResult {
    match input.regime {
        Regime::NewJersey => check_new_jersey(input),
        Regime::Maryland => check_maryland(input),
        Regime::NewYork => check_new_york(input),
        Regime::Default => check_default(input),
    }
}

fn check_new_jersey(input: &TenantRightsStatementInput) -> TenantRightsStatementResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    // NJ scoping: > 2 units, OR non-owner-occupied > 3 units (the
    // > 3 rule is the owner-occupied exemption pattern: an
    // owner-occupied building with 3 or fewer units is OUT, but
    // a non-owner-occupied building with 3+ units is IN since the
    // statute applies broadly to multi-unit buildings).
    let applies = if input.owner_occupied_building {
        input.building_unit_count > 3
    } else {
        input.building_unit_count > 2
    };

    notes.push(
        "N.J.S.A. 46:8-44 — NJ Department of Community Affairs (DCA) publishes Statement of Legal Rights and Responsibilities of Tenants and Landlords; bilingual English + Spanish; updated ANNUALLY"
            .to_string(),
    );

    if !applies {
        notes.push(
            "N.J.S.A. 46:8-47 — building outside scope of Truth in Renting Act (must have > 2 units, or non-owner-occupied > 3 units)"
                .to_string(),
        );
        return TenantRightsStatementResult {
            compliant: true,
            regime_applies_to_building: false,
            violations,
            citation: "N.J.S.A. §§ 46:8-43 to 46:8-50",
            notes,
        };
    }

    if !input.statement_distributed_within_30_days_of_publication {
        violations.push(
            "N.J.S.A. 46:8-45 — landlord MUST distribute Statement to each existing tenant within 30 DAYS after DCA publication"
                .to_string(),
        );
    }

    if !input.statement_delivered_to_new_tenant_at_occupancy {
        violations.push(
            "N.J.S.A. 46:8-45 — landlord MUST provide Statement to each new tenant at or prior to occupancy"
                .to_string(),
        );
    }

    if !input.statement_posted_in_prominent_location {
        violations.push(
            "N.J.S.A. 46:8-46 — landlord MUST post current Statement in one or more prominent and accessible locations"
                .to_string(),
        );
    }

    if input.relying_on_tenant_waiver {
        violations.push(
            "N.J.S.A. 46:8-48 — no waiver or refusal by tenant alters landlord's obligations; landlord cannot rely on tenant waiver"
                .to_string(),
        );
    }

    notes.push(
        "N.J.S.A. 46:8-49 — penalties for noncompliance; private right of action under N.J.S.A. 56:8-1 et seq. (Consumer Fraud Act) treble damages framework"
            .to_string(),
    );

    let compliant = violations.is_empty();
    TenantRightsStatementResult {
        compliant,
        regime_applies_to_building: true,
        violations,
        citation: "N.J.S.A. §§ 46:8-43 to 46:8-50",
        notes,
    }
}

fn check_maryland(_input: &TenantRightsStatementInput) -> TenantRightsStatementResult {
    let notes: Vec<String> = vec![
        "Md. Code, Real Property § 8-208 — limited disclosure requirements; landlord must provide tenant-rights notices but not the comprehensive Statement-of-Rights document NJ requires"
            .to_string(),
        "Maryland landlord-tenant framework relies on county / municipal supplements (Montgomery County, Prince George's County) for broader tenant-rights notice"
            .to_string(),
    ];

    TenantRightsStatementResult {
        compliant: true,
        regime_applies_to_building: true,
        violations: Vec::new(),
        citation: "Md. Code, Real Property § 8-208",
        notes,
    }
}

fn check_new_york(_input: &TenantRightsStatementInput) -> TenantRightsStatementResult {
    let notes: Vec<String> = vec![
        "HSTPA of 2019 — NY imposes discrete-disclosure mandates (rent stabilization status, security deposit handling) but NO statutory annual-distribution mandate equivalent to NJ Truth in Renting Act"
            .to_string(),
        "NY DHCR Residential Tenants' Rights Guide + NY AG Residential Tenants' Rights Guide — voluntary distribution recommended but not statutorily mandated"
            .to_string(),
    ];

    TenantRightsStatementResult {
        compliant: true,
        regime_applies_to_building: true,
        violations: Vec::new(),
        citation: "N.Y. Real Prop. Law §§ 220 et seq.; HSTPA of 2019",
        notes,
    }
}

fn check_default(_input: &TenantRightsStatementInput) -> TenantRightsStatementResult {
    let notes: Vec<String> = vec![
        "default rule — most states have NO statewide Tenant-Rights-Statement distribution mandate; municipal ordinances may impose distribution requirements (Chicago Tenants in Foreclosure Notification, San Francisco Rent Ordinance § 37)"
            .to_string(),
    ];

    TenantRightsStatementResult {
        compliant: true,
        regime_applies_to_building: true,
        violations: Vec::new(),
        citation: "state-specific landlord-tenant statute + municipal ordinances",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nj_base() -> TenantRightsStatementInput {
        TenantRightsStatementInput {
            regime: Regime::NewJersey,
            building_unit_count: 6,
            owner_occupied_building: false,
            statement_distributed_within_30_days_of_publication: true,
            statement_delivered_to_new_tenant_at_occupancy: true,
            statement_posted_in_prominent_location: true,
            relying_on_tenant_waiver: false,
        }
    }

    fn md_base() -> TenantRightsStatementInput {
        TenantRightsStatementInput {
            regime: Regime::Maryland,
            building_unit_count: 6,
            owner_occupied_building: false,
            statement_distributed_within_30_days_of_publication: false,
            statement_delivered_to_new_tenant_at_occupancy: false,
            statement_posted_in_prominent_location: false,
            relying_on_tenant_waiver: false,
        }
    }

    fn ny_base() -> TenantRightsStatementInput {
        TenantRightsStatementInput {
            regime: Regime::NewYork,
            building_unit_count: 10,
            owner_occupied_building: false,
            statement_distributed_within_30_days_of_publication: false,
            statement_delivered_to_new_tenant_at_occupancy: false,
            statement_posted_in_prominent_location: false,
            relying_on_tenant_waiver: false,
        }
    }

    fn default_base() -> TenantRightsStatementInput {
        TenantRightsStatementInput {
            regime: Regime::Default,
            building_unit_count: 10,
            owner_occupied_building: false,
            statement_distributed_within_30_days_of_publication: false,
            statement_delivered_to_new_tenant_at_occupancy: false,
            statement_posted_in_prominent_location: false,
            relying_on_tenant_waiver: false,
        }
    }

    #[test]
    fn nj_full_compliance_passes() {
        let r = check(&nj_base());
        assert!(r.compliant);
        assert!(r.regime_applies_to_building);
    }

    #[test]
    fn nj_missing_30_day_distribution_violates() {
        let mut i = nj_base();
        i.statement_distributed_within_30_days_of_publication = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("46:8-45") && v.contains("30 DAYS")));
    }

    #[test]
    fn nj_missing_new_tenant_delivery_violates() {
        let mut i = nj_base();
        i.statement_delivered_to_new_tenant_at_occupancy = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("46:8-45") && v.contains("new tenant")));
    }

    #[test]
    fn nj_missing_posting_violates() {
        let mut i = nj_base();
        i.statement_posted_in_prominent_location = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("46:8-46") && v.contains("prominent")));
    }

    #[test]
    fn nj_relying_on_tenant_waiver_violates() {
        let mut i = nj_base();
        i.relying_on_tenant_waiver = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("46:8-48") && v.contains("no waiver")));
    }

    #[test]
    fn nj_two_unit_non_owner_occupied_out_of_scope() {
        let mut i = nj_base();
        i.building_unit_count = 2;
        i.owner_occupied_building = false;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.regime_applies_to_building);
        assert!(r.notes.iter().any(|n| n.contains("46:8-47") && n.contains("outside scope")));
    }

    #[test]
    fn nj_three_unit_non_owner_occupied_in_scope() {
        let mut i = nj_base();
        i.building_unit_count = 3;
        i.owner_occupied_building = false;
        let r = check(&i);
        assert!(r.regime_applies_to_building);
    }

    #[test]
    fn nj_three_unit_owner_occupied_out_of_scope() {
        let mut i = nj_base();
        i.building_unit_count = 3;
        i.owner_occupied_building = true;
        let r = check(&i);
        assert!(!r.regime_applies_to_building);
    }

    #[test]
    fn nj_four_unit_owner_occupied_in_scope() {
        let mut i = nj_base();
        i.building_unit_count = 4;
        i.owner_occupied_building = true;
        let r = check(&i);
        assert!(r.regime_applies_to_building);
    }

    #[test]
    fn nj_dca_annual_publication_note_present() {
        let r = check(&nj_base());
        assert!(r.notes.iter().any(|n| n.contains("46:8-44") && n.contains("DCA") && n.contains("ANNUALLY")));
    }

    #[test]
    fn nj_bilingual_english_spanish_note() {
        let r = check(&nj_base());
        assert!(r.notes.iter().any(|n| n.contains("English + Spanish")));
    }

    #[test]
    fn nj_penalties_note_with_consumer_fraud_act() {
        let r = check(&nj_base());
        assert!(r.notes.iter().any(|n| n.contains("46:8-49") && n.contains("56:8-1") && n.contains("treble damages")));
    }

    #[test]
    fn nj_all_four_violations_simultaneous() {
        let mut i = nj_base();
        i.statement_distributed_within_30_days_of_publication = false;
        i.statement_delivered_to_new_tenant_at_occupancy = false;
        i.statement_posted_in_prominent_location = false;
        i.relying_on_tenant_waiver = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert_eq!(r.violations.len(), 4);
    }

    #[test]
    fn nj_citation_pins_46_8_43_to_46_8_50() {
        let r = check(&nj_base());
        assert!(r.citation.contains("46:8-43"));
        assert!(r.citation.contains("46:8-50"));
    }

    #[test]
    fn md_no_violations_even_when_no_distribution() {
        let r = check(&md_base());
        assert!(r.compliant);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn md_county_municipal_supplement_note() {
        let r = check(&md_base());
        assert!(r.notes.iter().any(|n| n.contains("Montgomery County") && n.contains("Prince George's County")));
    }

    #[test]
    fn md_citation_pins_8_208() {
        let r = check(&md_base());
        assert!(r.citation.contains("§ 8-208"));
    }

    #[test]
    fn ny_no_violations_no_mandate_note() {
        let r = check(&ny_base());
        assert!(r.compliant);
        assert!(r.notes.iter().any(|n| n.contains("HSTPA of 2019") && n.contains("NO statutory annual-distribution mandate")));
    }

    #[test]
    fn ny_dhcr_and_ag_guide_note() {
        let r = check(&ny_base());
        assert!(r.notes.iter().any(|n| n.contains("DHCR") && n.contains("AG Residential Tenants' Rights Guide")));
    }

    #[test]
    fn default_no_obligation_compliant() {
        let r = check(&default_base());
        assert!(r.compliant);
        assert!(r.notes.iter().any(|n| n.contains("NO statewide Tenant-Rights-Statement distribution mandate")));
    }

    #[test]
    fn default_municipal_examples_note() {
        let r = check(&default_base());
        assert!(r.notes.iter().any(|n| n.contains("Chicago") && n.contains("San Francisco Rent Ordinance § 37")));
    }

    #[test]
    fn nj_unique_distribution_mandate_invariant() {
        let mut i_nj = nj_base();
        i_nj.statement_distributed_within_30_days_of_publication = false;
        let r_nj = check(&i_nj);
        assert!(!r_nj.compliant);

        for regime in [Regime::Maryland, Regime::NewYork, Regime::Default] {
            let mut i = nj_base();
            i.regime = regime;
            i.statement_distributed_within_30_days_of_publication = false;
            i.statement_delivered_to_new_tenant_at_occupancy = false;
            i.statement_posted_in_prominent_location = false;
            let r = check(&i);
            assert!(r.compliant, "regime {:?} should not impose annual distribution mandate", regime);
        }
    }

    #[test]
    fn four_regimes_routed_correctly() {
        for regime in [Regime::NewJersey, Regime::Maryland, Regime::NewYork, Regime::Default] {
            let mut i = nj_base();
            i.regime = regime;
            let r = check(&i);
            let _ = r.compliant;
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn nj_2_unit_owner_occupied_out_of_scope() {
        let mut i = nj_base();
        i.building_unit_count = 2;
        i.owner_occupied_building = true;
        let r = check(&i);
        assert!(!r.regime_applies_to_building);
    }

    #[test]
    fn nj_out_of_scope_compliant_even_with_no_distribution() {
        let mut i = nj_base();
        i.building_unit_count = 2;
        i.owner_occupied_building = false;
        i.statement_distributed_within_30_days_of_publication = false;
        i.statement_delivered_to_new_tenant_at_occupancy = false;
        i.statement_posted_in_prominent_location = false;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn scoping_truth_table_owner_occupied_vs_unit_count() {
        let cases: [(u32, bool, bool); 6] = [
            (1, true, false),
            (2, true, false),
            (3, true, false),
            (4, true, true),
            (2, false, false),
            (3, false, true),
        ];
        for (units, owner_occ, expected) in cases {
            let mut i = nj_base();
            i.building_unit_count = units;
            i.owner_occupied_building = owner_occ;
            let r = check(&i);
            assert_eq!(
                r.regime_applies_to_building, expected,
                "unit_count={} owner_occupied={} expected_applies={}",
                units, owner_occ, expected
            );
        }
    }
}
