//! Rental property unpermitted / illegal dwelling unit
//! disclosure compliance — when a trader-landlord rents a
//! unit lacking a Certificate of Occupancy (in-law
//! apartment, garage conversion, basement apartment, attic
//! unit, ADU without permits), what statutory disclosure
//! and rent-collection rules apply? Trader-landlord
//! operational concern: an unpermitted-unit lease creates
//! ASYMMETRIC enforceability — tenant may enforce lease and
//! recover damages, but landlord may NOT collect rent or
//! enforce eviction without first legalizing the unit.
//! Distinct from siblings `rental_bedroom_egress_window`
//! (structural), `rental_carbon_monoxide_detector`
//! (safety), `rental_hot_water_temperature` (habitability),
//! `rental_property_registration` (general registration).
//!
//! **Four regimes**:
//!
//! **California — common law + Cal. Civ. Code § 1942.4 +
//! § 1102 et seq. + Espinoza v. Calva, 169 Cal.App.4th 1393
//! (2008)**:
//! - Landlord **CANNOT legally collect rent** for unit
//!   lacking Certificate of Occupancy.
//! - Tenant **CAN enforce lease and sue for damages**
//!   despite illegality (asymmetric enforceability
//!   doctrine).
//! - Tenant may recover past rent paid for unpermitted
//!   unit.
//! - Tenant may pursue relocation assistance under local
//!   ordinances.
//! - On property sale, **Transfer Disclosure Statement
//!   (TDS) required** under § 1102 et seq. disclosing
//!   unpermitted unit as material defect.
//! - Local rent control overlay: SF, Berkeley, Oakland
//!   prohibit no-cause eviction.
//!
//! **Oakland — Oakland Municipal Code § 8.22 (Tenant
//! Protection Ordinance + Just Cause for Eviction
//! Ordinance)**:
//! - Strictest among California comparators.
//! - Treats unpermitted-unit eviction as **no just cause**;
//!   eviction prohibited except for substantial rehab.
//! - **Relocation payments REQUIRED** when displacing
//!   tenant from unpermitted unit (currently $7,931 base +
//!   $5,287 per senior/disabled/minor; OMC § 8.22.450, 2024
//!   amounts).
//! - Rent-control overlay prohibits rent increase above
//!   annual CPI adjustment even in unpermitted units.
//!
//! **New York City — NYC Multiple Dwelling Law § 325 + NYC
//! Admin Code § 27-2107 + NYC Building Code § 28-118**:
//! - Buildings of 3+ units MUST have Certificate of
//!   Occupancy.
//! - Unauthorized cellar/basement apartments illegal under
//!   MDL.
//! - Landlord **CANNOT collect rent** for illegal unit
//!   until legalized (DOB approval).
//! - Tenant may pursue back-rent refund.
//! - Tenant remedies under NYC Loft Law (MDL Article 7-C)
//!   for legalized loft conversions.
//! - Civil penalty up to $1,000 plus $50 per day for
//!   continuing violation under NYC Admin Code § 27-2115.
//!
//! **Default — common-law warranty of habitability + state
//! building code**:
//! - No specific statewide unpermitted-unit disclosure
//!   mandate.
//! - General implied warranty of habitability + landlord-
//!   tenant statutes provide constructive eviction +
//!   damages remedies.
//! - HUD Section 8 housing quality standards apply for
//!   federally-subsidized units.
//!
//! Citations: Cal. Civ. Code § 1942.4; Cal. Civ. Code §
//! 1102 et seq. (TDS); Espinoza v. Calva, 169 Cal.App.4th
//! 1393 (Cal. Ct. App. 2008); Oakland Municipal Code § 8.22
//! (TPO + Just Cause + § 8.22.450 relocation); NYC Multiple
//! Dwelling Law § 325; NYC Admin Code § 27-2107 + § 27-
//! 2115; NYC Building Code § 28-118; NYC Loft Law (MDL
//! Article 7-C).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Oakland,
    NewYorkCity,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalUnpermittedUnitDisclosureInput {
    pub regime: Regime,
    /// Whether unit has Certificate of Occupancy.
    pub has_certificate_of_occupancy: bool,
    /// Whether landlord collects or seeks to collect rent.
    pub landlord_collects_rent: bool,
    /// Whether landlord disclosed unpermitted status to
    /// tenant before lease execution.
    pub pre_lease_unpermitted_status_disclosed: bool,
    /// Whether unit is in a 3+ unit multiple dwelling (NYC
    /// MDL § 325 scope).
    pub three_plus_unit_building: bool,
    /// Whether landlord seeks no-cause eviction of tenant
    /// from unpermitted unit (Oakland TPO violation).
    pub landlord_seeks_no_cause_eviction: bool,
    /// Whether substantial-rehabilitation eviction exception
    /// applies (Oakland TPO carve-out).
    pub substantial_rehab_eviction_basis: bool,
    /// Number of senior/disabled/minor household members
    /// for Oakland relocation calculation.
    pub senior_disabled_or_minor_count: u32,
    /// Whether Oakland relocation payment was made.
    pub oakland_relocation_payment_made: bool,
    /// Whether TDS Transfer Disclosure Statement disclosed
    /// unpermitted unit at sale (CA § 1102 et seq.).
    pub tds_disclosed_at_sale: bool,
    /// Whether property is being sold (triggers CA TDS
    /// requirement).
    pub property_being_sold: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalUnpermittedUnitDisclosureResult {
    pub disclosure_compliant: bool,
    pub rent_collection_prohibited: bool,
    pub eviction_prohibited: bool,
    pub tenant_may_enforce_lease: bool,
    pub tds_required: bool,
    pub oakland_relocation_amount_cents: u64,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &RentalUnpermittedUnitDisclosureInput,
) -> RentalUnpermittedUnitDisclosureResult {
    match input.regime {
        Regime::California => check_ca(input),
        Regime::Oakland => check_oakland(input),
        Regime::NewYorkCity => check_nyc(input),
        Regime::Default => check_default(input),
    }
}

fn check_ca(
    input: &RentalUnpermittedUnitDisclosureInput,
) -> RentalUnpermittedUnitDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "California asymmetric enforceability doctrine (Espinoza v. Calva, 169 Cal.App.4th 1393 (Cal. Ct. App. 2008)) — tenant may enforce lease and sue for damages despite illegality; landlord may NOT collect or enforce rent for unpermitted unit".to_string(),
        "Cal. Civ. Code § 1942.4 — landlord cannot collect rent for unit substantially in violation of housing standards making it unhabitable; supplements common-law warranty of habitability".to_string(),
        "Cal. Civ. Code § 1102 et seq. — Transfer Disclosure Statement (TDS) required at sale disclosing unpermitted unit as material defect to buyer".to_string(),
        "California tenant remedies: (1) constructive eviction + damages, (2) past rent refund, (3) relocation assistance under local ordinances (SF/Berkeley/Oakland), (4) attorney fees under § 1942.4(b)(2)".to_string(),
        "Local rent-control overlay — SF, Berkeley, Oakland prohibit no-cause eviction even for unpermitted units".to_string(),
    ];

    let unpermitted = !input.has_certificate_of_occupancy;

    if unpermitted && input.landlord_collects_rent {
        violations.push(
            "California — landlord may NOT collect rent for unit lacking Certificate of Occupancy; Cal. Civ. Code § 1942.4 + Espinoza v. Calva, 169 Cal.App.4th 1393 (2008)".to_string(),
        );
    }

    if unpermitted && input.property_being_sold && !input.tds_disclosed_at_sale {
        violations.push(
            "Cal. Civ. Code § 1102 et seq. — Transfer Disclosure Statement must disclose unpermitted unit as material defect when property is sold".to_string(),
        );
    }

    RentalUnpermittedUnitDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        rent_collection_prohibited: unpermitted,
        eviction_prohibited: false,
        tenant_may_enforce_lease: unpermitted,
        tds_required: unpermitted && input.property_being_sold,
        oakland_relocation_amount_cents: 0,
        violations,
        citation: "Cal. Civ. Code § 1942.4 + § 1102 et seq.; Espinoza v. Calva, 169 Cal.App.4th 1393 (Cal. Ct. App. 2008)",
        notes,
    }
}

fn check_oakland(
    input: &RentalUnpermittedUnitDisclosureInput,
) -> RentalUnpermittedUnitDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Oakland Municipal Code § 8.22 (Tenant Protection Ordinance) — strictest among CA comparators; treats unpermitted-unit eviction as no just cause; eviction prohibited except substantial rehab".to_string(),
        "Oakland Municipal Code § 8.22.450 — relocation payments REQUIRED when displacing tenant from unpermitted unit; base amount $7,931 + $5,287 per senior/disabled/minor household member (2024 amounts)".to_string(),
        "Oakland Just Cause for Eviction Ordinance — rent-control overlay prohibits rent increase above annual CPI even in unpermitted units".to_string(),
        "Oakland framework parallels California asymmetric enforceability doctrine + adds local relocation payment + just-cause eviction protection".to_string(),
        "Trader-landlord critical: Oakland routinely upholds tenant claims for past rent recovery + relocation + attorney fees under TPO § 8.22.640".to_string(),
    ];

    let unpermitted = !input.has_certificate_of_occupancy;

    if unpermitted && input.landlord_collects_rent {
        violations.push(
            "California + Oakland Municipal Code § 8.22 — landlord may NOT collect rent for unpermitted unit; rent-control overlay further prohibits rent increase".to_string(),
        );
    }

    if unpermitted
        && input.landlord_seeks_no_cause_eviction
        && !input.substantial_rehab_eviction_basis
    {
        violations.push(
            "Oakland Municipal Code § 8.22 (Just Cause for Eviction Ordinance) — unpermitted-unit eviction treated as no just cause; eviction prohibited except for substantial rehab".to_string(),
        );
    }

    let base_relocation: u64 = 793_100;
    let per_qualifying: u64 = 528_700;
    let oakland_relocation: u64 = if unpermitted && input.landlord_seeks_no_cause_eviction {
        base_relocation
            .saturating_add(per_qualifying.saturating_mul(input.senior_disabled_or_minor_count as u64))
    } else {
        0
    };

    if unpermitted
        && input.landlord_seeks_no_cause_eviction
        && !input.oakland_relocation_payment_made
    {
        violations.push(
            "Oakland Municipal Code § 8.22.450 — relocation payments REQUIRED when displacing tenant from unpermitted unit; failure to pay creates per-incident OMC § 8.22.640 enforcement liability".to_string(),
        );
    }

    RentalUnpermittedUnitDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        rent_collection_prohibited: unpermitted,
        eviction_prohibited: unpermitted && !input.substantial_rehab_eviction_basis,
        tenant_may_enforce_lease: unpermitted,
        tds_required: false,
        oakland_relocation_amount_cents: oakland_relocation,
        violations,
        citation: "Oakland Municipal Code § 8.22 (TPO + Just Cause + § 8.22.450 relocation); Cal. Civ. Code § 1942.4; Espinoza v. Calva",
        notes,
    }
}

fn check_nyc(
    input: &RentalUnpermittedUnitDisclosureInput,
) -> RentalUnpermittedUnitDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "NYC Multiple Dwelling Law § 325 — buildings of 3+ units MUST have Certificate of Occupancy; unauthorized cellar/basement apartments illegal".to_string(),
        "NYC Admin Code § 27-2107 — landlord may NOT collect rent for illegal unit until legalized (DOB approval); tenant may pursue back-rent refund".to_string(),
        "NYC Admin Code § 27-2115 — civil penalty up to $1,000 plus $50 per day for continuing violation".to_string(),
        "NYC Building Code § 28-118 — Certificate of Occupancy required for change of use or new construction".to_string(),
        "NYC Loft Law (MDL Article 7-C) — provides legalization pathway for converted loft tenants and rent-stabilization protections post-legalization".to_string(),
    ];

    let unpermitted = !input.has_certificate_of_occupancy;
    let mdl_scope = input.three_plus_unit_building;

    if unpermitted && mdl_scope {
        if input.landlord_collects_rent {
            violations.push(
                "NYC Multiple Dwelling Law § 325 + NYC Admin Code § 27-2107 — landlord may NOT collect rent for illegal unit in 3+ unit building until legalized (DOB approval)".to_string(),
            );
        }
        if !input.pre_lease_unpermitted_status_disclosed {
            violations.push(
                "NYC Multiple Dwelling Law § 325 — illegal cellar/basement apartments in 3+ unit buildings prohibited; landlord cannot rent unauthorized space".to_string(),
            );
        }
    }

    RentalUnpermittedUnitDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        rent_collection_prohibited: unpermitted && mdl_scope,
        eviction_prohibited: false,
        tenant_may_enforce_lease: unpermitted && mdl_scope,
        tds_required: false,
        oakland_relocation_amount_cents: 0,
        violations,
        citation: "NYC Multiple Dwelling Law § 325; NYC Admin Code § 27-2107 + § 27-2115; NYC Building Code § 28-118; NYC Loft Law (MDL Article 7-C)",
        notes,
    }
}

fn check_default(
    input: &RentalUnpermittedUnitDisclosureInput,
) -> RentalUnpermittedUnitDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Default — no specific statewide unpermitted-unit disclosure mandate".to_string(),
        "Default — general implied warranty of habitability + state landlord-tenant statutes provide constructive eviction + damages remedies".to_string(),
        "Default — HUD Section 8 housing quality standards apply for federally-subsidized units".to_string(),
        "Default — local building codes typically require Certificate of Occupancy; verify municipal ordinance for specific disclosure obligation".to_string(),
        "Default — tenant common-law remedies typically include constructive eviction + return of past rent + relocation".to_string(),
    ];

    let unpermitted = !input.has_certificate_of_occupancy;

    if unpermitted && input.landlord_collects_rent && !input.pre_lease_unpermitted_status_disclosed
    {
        violations.push(
            "Default — common-law warranty of habitability typically implied by state law; verify local jurisdiction for specific Certificate of Occupancy requirement and rent-collection restriction".to_string(),
        );
    }

    RentalUnpermittedUnitDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        rent_collection_prohibited: false,
        eviction_prohibited: false,
        tenant_may_enforce_lease: unpermitted,
        tds_required: false,
        oakland_relocation_amount_cents: 0,
        violations,
        citation: "Default state landlord-tenant law + local Certificate of Occupancy requirement",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_unpermitted_renting() -> RentalUnpermittedUnitDisclosureInput {
        RentalUnpermittedUnitDisclosureInput {
            regime: Regime::California,
            has_certificate_of_occupancy: false,
            landlord_collects_rent: true,
            pre_lease_unpermitted_status_disclosed: false,
            three_plus_unit_building: false,
            landlord_seeks_no_cause_eviction: false,
            substantial_rehab_eviction_basis: false,
            senior_disabled_or_minor_count: 0,
            oakland_relocation_payment_made: false,
            tds_disclosed_at_sale: false,
            property_being_sold: false,
        }
    }

    fn ca_clean() -> RentalUnpermittedUnitDisclosureInput {
        let mut i = ca_unpermitted_renting();
        i.has_certificate_of_occupancy = true;
        i.landlord_collects_rent = true;
        i
    }

    fn oakland_unpermitted_evicting() -> RentalUnpermittedUnitDisclosureInput {
        let mut i = ca_unpermitted_renting();
        i.regime = Regime::Oakland;
        i.landlord_seeks_no_cause_eviction = true;
        i.oakland_relocation_payment_made = true;
        i
    }

    fn nyc_unpermitted_renting() -> RentalUnpermittedUnitDisclosureInput {
        let mut i = ca_unpermitted_renting();
        i.regime = Regime::NewYorkCity;
        i.three_plus_unit_building = true;
        i
    }

    fn default_clean() -> RentalUnpermittedUnitDisclosureInput {
        let mut i = ca_clean();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn ca_clean_compliant() {
        let r = check(&ca_clean());
        assert!(r.disclosure_compliant);
        assert!(!r.rent_collection_prohibited);
    }

    #[test]
    fn ca_unpermitted_renting_violation() {
        let r = check(&ca_unpermitted_renting());
        assert!(!r.disclosure_compliant);
        assert!(r.rent_collection_prohibited);
        assert!(r.tenant_may_enforce_lease);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1942.4") && v.contains("Espinoza")));
    }

    #[test]
    fn ca_unpermitted_no_rent_collection_no_violation() {
        let mut i = ca_unpermitted_renting();
        i.landlord_collects_rent = false;
        let r = check(&i);
        assert!(r.disclosure_compliant);
        assert!(r.rent_collection_prohibited);
    }

    #[test]
    fn ca_unpermitted_sale_without_tds_violation() {
        let mut i = ca_unpermitted_renting();
        i.landlord_collects_rent = false;
        i.property_being_sold = true;
        i.tds_disclosed_at_sale = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.tds_required);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1102") && v.contains("Transfer Disclosure Statement")));
    }

    #[test]
    fn ca_unpermitted_sale_with_tds_compliant() {
        let mut i = ca_unpermitted_renting();
        i.landlord_collects_rent = false;
        i.property_being_sold = true;
        i.tds_disclosed_at_sale = true;
        let r = check(&i);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn oakland_unpermitted_with_relocation_compliant() {
        let r = check(&oakland_unpermitted_evicting());
        assert!(!r.disclosure_compliant);
        assert!(r.rent_collection_prohibited);
        assert!(r.eviction_prohibited);
    }

    #[test]
    fn oakland_unpermitted_no_relocation_payment_violation() {
        let mut i = oakland_unpermitted_evicting();
        i.oakland_relocation_payment_made = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 8.22.450") && v.contains("relocation")));
    }

    #[test]
    fn oakland_substantial_rehab_eviction_basis_carveout() {
        let mut i = oakland_unpermitted_evicting();
        i.substantial_rehab_eviction_basis = true;
        let r = check(&i);
        assert!(!r.eviction_prohibited);
    }

    #[test]
    fn oakland_relocation_amount_base_only() {
        let mut i = oakland_unpermitted_evicting();
        i.senior_disabled_or_minor_count = 0;
        let r = check(&i);
        assert_eq!(r.oakland_relocation_amount_cents, 793_100);
    }

    #[test]
    fn oakland_relocation_amount_one_qualifying_member() {
        let mut i = oakland_unpermitted_evicting();
        i.senior_disabled_or_minor_count = 1;
        let r = check(&i);
        assert_eq!(r.oakland_relocation_amount_cents, 793_100 + 528_700);
    }

    #[test]
    fn oakland_relocation_amount_three_qualifying_members() {
        let mut i = oakland_unpermitted_evicting();
        i.senior_disabled_or_minor_count = 3;
        let r = check(&i);
        assert_eq!(r.oakland_relocation_amount_cents, 793_100 + 528_700 * 3);
    }

    #[test]
    fn oakland_defensive_relocation_overflow_saturates_without_panic() {
        let mut i = oakland_unpermitted_evicting();
        i.senior_disabled_or_minor_count = u32::MAX;
        let r = check(&i);
        let expected = 793_100_u64
            .saturating_add(528_700_u64.saturating_mul(u32::MAX as u64));
        assert_eq!(r.oakland_relocation_amount_cents, expected);
        assert!(r.oakland_relocation_amount_cents > 793_100);
    }

    #[test]
    fn nyc_unpermitted_3_unit_building_violation() {
        let r = check(&nyc_unpermitted_renting());
        assert!(!r.disclosure_compliant);
        assert!(r.rent_collection_prohibited);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 325") && v.contains("§ 27-2107")));
    }

    #[test]
    fn nyc_unpermitted_2_unit_building_out_of_mdl_scope() {
        let mut i = nyc_unpermitted_renting();
        i.three_plus_unit_building = false;
        let r = check(&i);
        assert!(!r.rent_collection_prohibited);
    }

    #[test]
    fn nyc_certificate_of_occupancy_compliant() {
        let mut i = nyc_unpermitted_renting();
        i.has_certificate_of_occupancy = true;
        let r = check(&i);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn default_clean_compliant() {
        let r = check(&default_clean());
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn default_unpermitted_renting_without_disclosure_violation() {
        let mut i = default_clean();
        i.has_certificate_of_occupancy = false;
        i.pre_lease_unpermitted_status_disclosed = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
    }

    #[test]
    fn default_unpermitted_renting_with_disclosure_compliant() {
        let mut i = default_clean();
        i.has_certificate_of_occupancy = false;
        i.pre_lease_unpermitted_status_disclosed = true;
        let r = check(&i);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn citation_pins_ca_authority() {
        let r = check(&ca_clean());
        assert!(r.citation.contains("§ 1942.4"));
        assert!(r.citation.contains("§ 1102"));
        assert!(r.citation.contains("Espinoza v. Calva"));
    }

    #[test]
    fn citation_pins_oakland_authority() {
        let r = check(&oakland_unpermitted_evicting());
        assert!(r.citation.contains("§ 8.22"));
        assert!(r.citation.contains("§ 8.22.450"));
    }

    #[test]
    fn citation_pins_nyc_authority() {
        let r = check(&nyc_unpermitted_renting());
        assert!(r.citation.contains("§ 325"));
        assert!(r.citation.contains("§ 27-2107"));
        assert!(r.citation.contains("Loft Law"));
    }

    #[test]
    fn citation_pins_default_authority() {
        let r = check(&default_clean());
        assert!(r.citation.contains("Default state landlord-tenant"));
        assert!(r.citation.contains("Certificate of Occupancy"));
    }

    #[test]
    fn note_pins_ca_asymmetric_enforceability_doctrine() {
        let r = check(&ca_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("asymmetric enforceability") && n.contains("Espinoza")));
    }

    #[test]
    fn note_pins_oakland_relocation_amounts() {
        let r = check(&oakland_unpermitted_evicting());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("$7,931") && n.contains("$5,287")));
    }

    #[test]
    fn note_pins_nyc_3_unit_mdl_threshold() {
        let r = check(&nyc_unpermitted_renting());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("3+ unit") && n.contains("Certificate of Occupancy")));
    }

    #[test]
    fn note_pins_nyc_loft_law() {
        let r = check(&nyc_unpermitted_renting());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Loft Law") && n.contains("Article 7-C")));
    }

    #[test]
    fn note_pins_default_hud_section_8() {
        let r = check(&default_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("HUD Section 8")));
    }

    #[test]
    fn oakland_uniquely_requires_relocation_payment_invariant() {
        let mut i_ca = ca_unpermitted_renting();
        i_ca.landlord_seeks_no_cause_eviction = true;
        let r_ca = check(&i_ca);
        assert_eq!(r_ca.oakland_relocation_amount_cents, 0);

        let r_oakland = check(&oakland_unpermitted_evicting());
        assert!(r_oakland.oakland_relocation_amount_cents > 0);
    }

    #[test]
    fn nyc_uniquely_requires_mdl_3_plus_unit_scope_invariant() {
        let mut i_nyc = nyc_unpermitted_renting();
        i_nyc.three_plus_unit_building = false;
        let r_nyc_below = check(&i_nyc);
        assert!(!r_nyc_below.rent_collection_prohibited);

        let r_nyc_in_scope = check(&nyc_unpermitted_renting());
        assert!(r_nyc_in_scope.rent_collection_prohibited);
    }

    #[test]
    fn ca_uniquely_triggers_tds_at_sale_invariant() {
        let mut i_ca = ca_unpermitted_renting();
        i_ca.landlord_collects_rent = false;
        i_ca.property_being_sold = true;
        let r_ca = check(&i_ca);
        assert!(r_ca.tds_required);

        let mut i_oakland = oakland_unpermitted_evicting();
        i_oakland.landlord_collects_rent = false;
        i_oakland.property_being_sold = true;
        let r_oakland = check(&i_oakland);
        assert!(!r_oakland.tds_required);

        let mut i_nyc = nyc_unpermitted_renting();
        i_nyc.landlord_collects_rent = false;
        i_nyc.property_being_sold = true;
        let r_nyc = check(&i_nyc);
        assert!(!r_nyc.tds_required);
    }

    #[test]
    fn tenant_enforceability_asymmetry_across_regimes() {
        for regime in [Regime::California, Regime::Oakland, Regime::NewYorkCity] {
            let mut i = match regime {
                Regime::California => ca_unpermitted_renting(),
                Regime::Oakland => oakland_unpermitted_evicting(),
                Regime::NewYorkCity => nyc_unpermitted_renting(),
                _ => ca_unpermitted_renting(),
            };
            i.has_certificate_of_occupancy = false;
            if matches!(regime, Regime::NewYorkCity) {
                i.three_plus_unit_building = true;
            }
            let r = check(&i);
            assert!(r.tenant_may_enforce_lease);
        }
    }

    #[test]
    fn multiple_oakland_violations_stack() {
        let mut i = oakland_unpermitted_evicting();
        i.landlord_collects_rent = true;
        i.oakland_relocation_payment_made = false;
        let r = check(&i);
        assert_eq!(r.violations.len(), 3);
    }
}
