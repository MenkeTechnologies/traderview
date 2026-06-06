//! Residential water heater earthquake strap / brace / anchor
//! requirement compliance — California Health & Safety Code §
//! 19211 (Article 8 Water Heater Strapping and Installation,
//! Chapter 2 Earthquake Protection). Trader-landlord critical
//! for CA rental property owners: § 19211 violation
//! categorically renders the building a NUISANCE under § 19211
//! and breaches the implied warranty of habitability.
//! Insurance carriers may deny earthquake / fire / flood
//! claims tied to non-compliant water heaters.
//!
//! Distinct from siblings `meth_contamination_disclosure`
//! (substance hazard), `mold_disclosure` (substance hazard),
//! `fire_sprinkler_disclosure` (fire-suppression system), and
//! `detector_requirements` (smoke + CO detectors).
//!
//! **Two regimes**:
//!
//! **California — Cal. Health & Safety Code § 19211**:
//! - § 19211(a) — all new + replacement + existing residential
//!   water heaters must be braced, anchored, or strapped to
//!   resist falling or horizontal displacement due to
//!   earthquake motion.
//! - § 19211(a) — at minimum, water heater shall be secured
//!   per California Plumbing Code (or city / county
//!   modifications).
//! - § 19211(b) — "water heater" means any standard water
//!   heater with capacity ≤ 120 gallons for which a
//!   pre-engineered strapping kit is readily available.
//! - § 19211(c) — building or dwelling unit in violation is
//!   deemed a NUISANCE.
//! - § 19211(d) — seller of real property containing a water
//!   heater shall certify in writing to prospective purchaser
//!   that § 19211 has been complied with.
//!
//! **Default — no statutory earthquake-strap requirement**.
//! Other seismic states (WA + OR + AK + HI) may have
//! analogous building-code provisions adopted via the IBC /
//! IPC but lack CA's habitability-nuisance + civil-liability
//! framework.
//!
//! Citations: Cal. Health & Safety Code §§ 19210-19217
//! (Article 8 Water Heater Strapping and Installation);
//! California Plumbing Code (Title 24, Part 5) Chapter 5
//! Water Heaters.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WaterHeaterEarthquakeStrapInput {
    pub regime: Regime,
    /// Water heater capacity in gallons (≤ 120 = covered).
    pub capacity_gallons: u32,
    /// Whether the water heater has been braced / anchored /
    /// strapped per California Plumbing Code minimum.
    pub strapping_satisfied: bool,
    /// Whether the seller certified § 19211 compliance in
    /// writing (applies on property sale).
    pub seller_certification_provided: bool,
    /// Whether this scenario involves a sale (triggers §
    /// 19211(d) certification gate) vs an existing tenancy.
    pub is_property_sale: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct WaterHeaterEarthquakeStrapResult {
    pub compliant: bool,
    pub within_120_gallon_scope: bool,
    pub strapping_required: bool,
    pub habitability_nuisance: bool,
    pub seller_certification_required: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &WaterHeaterEarthquakeStrapInput) -> WaterHeaterEarthquakeStrapResult {
    match input.regime {
        Regime::California => check_california(input),
        Regime::Default => check_default(input),
    }
}

fn check_california(input: &WaterHeaterEarthquakeStrapInput) -> WaterHeaterEarthquakeStrapResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Health & Safety Code § 19211(a) — all new + replacement + existing residential water heaters must be braced, anchored, or strapped to resist falling or horizontal displacement due to earthquake motion; minimum standard = California Plumbing Code (Title 24 Part 5) or local modifications"
            .to_string(),
        "Cal. Health & Safety Code § 19211(b) — 'water heater' means standard water heater with capacity ≤ 120 gallons for which a pre-engineered strapping kit is readily available"
            .to_string(),
        "Cal. Health & Safety Code § 19211(c) — building or dwelling unit in violation is deemed a NUISANCE; owner has right to correct; breaches implied warranty of habitability"
            .to_string(),
        "Cal. Health & Safety Code § 19211(d) — seller of real property containing water heater must certify § 19211 compliance IN WRITING to prospective purchaser (transfer of title transactional disclosure)"
            .to_string(),
    ];

    let within_scope = input.capacity_gallons <= 120;
    let strapping_required = within_scope;
    let mut nuisance = false;
    let seller_cert_required = input.is_property_sale;

    if within_scope && !input.strapping_satisfied {
        violations.push(
            "Cal. Health & Safety Code § 19211(a) — residential water heater not braced, anchored, or strapped to resist falling or horizontal displacement; California Plumbing Code minimum not satisfied".to_string(),
        );
        nuisance = true;
    }

    if seller_cert_required && !input.seller_certification_provided {
        violations.push(
            "Cal. Health & Safety Code § 19211(d) — seller did not provide written certification of § 19211 compliance to prospective purchaser".to_string(),
        );
    }

    WaterHeaterEarthquakeStrapResult {
        compliant: violations.is_empty(),
        within_120_gallon_scope: within_scope,
        strapping_required,
        habitability_nuisance: nuisance,
        seller_certification_required: seller_cert_required,
        violations,
        citation: "Cal. Health & Safety Code §§ 19210-19217 (Article 8); California Plumbing Code Title 24 Part 5",
        notes,
    }
}

fn check_default(_input: &WaterHeaterEarthquakeStrapInput) -> WaterHeaterEarthquakeStrapResult {
    let notes: Vec<String> = vec![
        "default rule — no statutory earthquake-strap requirement; other seismic states (WA + OR + AK + HI) may have analogous building-code provisions adopted via the International Plumbing Code (IPC) but lack CA's habitability-nuisance + civil-liability framework"
            .to_string(),
        "default rule — verify local jurisdiction building codes; common-law premises liability may apply if water-heater displacement during seismic event causes tenant injury"
            .to_string(),
    ];

    WaterHeaterEarthquakeStrapResult {
        compliant: true,
        within_120_gallon_scope: false,
        strapping_required: false,
        habitability_nuisance: false,
        seller_certification_required: false,
        violations: Vec::new(),
        citation: "International Plumbing Code (where adopted) + common-law premises liability",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_compliant() -> WaterHeaterEarthquakeStrapInput {
        WaterHeaterEarthquakeStrapInput {
            regime: Regime::California,
            capacity_gallons: 50,
            strapping_satisfied: true,
            seller_certification_provided: true,
            is_property_sale: false,
        }
    }

    fn default_base() -> WaterHeaterEarthquakeStrapInput {
        let mut i = ca_compliant();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn ca_50_gallon_strapped_compliant() {
        let r = check(&ca_compliant());
        assert!(r.compliant);
        assert!(r.within_120_gallon_scope);
        assert!(r.strapping_required);
        assert!(!r.habitability_nuisance);
    }

    #[test]
    fn ca_50_gallon_unstrapped_violates_engages_nuisance() {
        let mut i = ca_compliant();
        i.strapping_satisfied = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.habitability_nuisance);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 19211(a)") && v.contains("California Plumbing Code")));
    }

    #[test]
    fn ca_120_gallon_boundary_inside_scope() {
        let mut i = ca_compliant();
        i.capacity_gallons = 120;
        let r = check(&i);
        assert!(r.within_120_gallon_scope);
        assert!(r.strapping_required);
    }

    #[test]
    fn ca_121_gallon_outside_scope() {
        let mut i = ca_compliant();
        i.capacity_gallons = 121;
        i.strapping_satisfied = false;
        let r = check(&i);
        assert!(!r.within_120_gallon_scope);
        assert!(!r.strapping_required);
        assert!(r.compliant);
    }

    #[test]
    fn ca_property_sale_no_cert_violates() {
        let mut i = ca_compliant();
        i.is_property_sale = true;
        i.seller_certification_provided = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 19211(d)") && v.contains("written certification")));
    }

    #[test]
    fn ca_property_sale_with_cert_compliant() {
        let mut i = ca_compliant();
        i.is_property_sale = true;
        i.seller_certification_provided = true;
        let r = check(&i);
        assert!(r.compliant);
        assert!(r.seller_certification_required);
    }

    #[test]
    fn ca_tenancy_no_cert_obligation() {
        let mut i = ca_compliant();
        i.is_property_sale = false;
        i.seller_certification_provided = false;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.seller_certification_required);
    }

    #[test]
    fn ca_both_violations_stack() {
        let mut i = ca_compliant();
        i.strapping_satisfied = false;
        i.is_property_sale = true;
        i.seller_certification_provided = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert_eq!(r.violations.len(), 2);
        assert!(r.habitability_nuisance);
    }

    #[test]
    fn ca_oversize_water_heater_no_violations() {
        let mut i = ca_compliant();
        i.capacity_gallons = 200;
        i.strapping_satisfied = false;
        i.is_property_sale = true;
        i.seller_certification_provided = false;
        let r = check(&i);
        assert!(!r.within_120_gallon_scope);
        assert!(!r.strapping_required);
        assert!(!r.habitability_nuisance);
        assert!(r.violations.iter().any(|v| v.contains("§ 19211(d)")));
    }

    #[test]
    fn ca_citation_pins_article_8() {
        let r = check(&ca_compliant());
        assert!(r.citation.contains("§§ 19210-19217"));
        assert!(r.citation.contains("Article 8"));
        assert!(r.citation.contains("California Plumbing Code"));
        assert!(r.citation.contains("Title 24 Part 5"));
    }

    #[test]
    fn ca_note_pins_subsections() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 19211(a)")));
        assert!(r.notes.iter().any(|n| n.contains("§ 19211(b)")));
        assert!(r.notes.iter().any(|n| n.contains("§ 19211(c)")));
        assert!(r.notes.iter().any(|n| n.contains("§ 19211(d)")));
    }

    #[test]
    fn ca_note_pins_120_gallon_scope_definition() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 19211(b)")
            && n.contains("120 gallons")
            && n.contains("pre-engineered strapping kit")));
    }

    #[test]
    fn ca_note_pins_nuisance_and_habitability() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 19211(c)")
            && n.contains("NUISANCE")
            && n.contains("implied warranty of habitability")));
    }

    #[test]
    fn ca_note_pins_seller_certification_obligation() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 19211(d)")
            && n.contains("IN WRITING")
            && n.contains("transfer of title")));
    }

    #[test]
    fn default_no_strap_no_violation() {
        let mut i = default_base();
        i.strapping_satisfied = false;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.strapping_required);
        assert!(!r.habitability_nuisance);
    }

    #[test]
    fn default_no_seller_cert_obligation() {
        let mut i = default_base();
        i.is_property_sale = true;
        i.seller_certification_provided = false;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn default_citation_pins_ipc_and_premises() {
        let r = check(&default_base());
        assert!(r.citation.contains("International Plumbing Code"));
        assert!(r.citation.contains("common-law premises liability"));
    }

    #[test]
    fn default_note_pins_seismic_state_caveat() {
        let r = check(&default_base());
        assert!(r.notes.iter().any(|n| n.contains("WA")
            && n.contains("OR")
            && n.contains("AK")
            && n.contains("HI")
            && n.contains("habitability-nuisance")));
    }

    #[test]
    fn two_regimes_routed_correctly() {
        for regime in [Regime::California, Regime::Default] {
            let mut i = ca_compliant();
            i.regime = regime;
            let r = check(&i);
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn ca_uniquely_engages_strapping_requirement_invariant() {
        let r_ca = check(&ca_compliant());
        assert!(r_ca.strapping_required);

        let r_default = check(&default_base());
        assert!(!r_default.strapping_required);
    }

    #[test]
    fn ca_uniquely_engages_nuisance_invariant() {
        let mut i_ca = ca_compliant();
        i_ca.strapping_satisfied = false;
        let r_ca = check(&i_ca);
        assert!(r_ca.habitability_nuisance);

        let mut i_default = default_base();
        i_default.strapping_satisfied = false;
        let r_default = check(&i_default);
        assert!(!r_default.habitability_nuisance);
    }

    #[test]
    fn capacity_boundary_truth_table_california() {
        for (cap, exp_in_scope) in [
            (0u32, true),
            (40, true),
            (75, true),
            (119, true),
            (120, true),
            (121, false),
            (200, false),
            (500, false),
        ] {
            let mut i = ca_compliant();
            i.capacity_gallons = cap;
            let r = check(&i);
            assert_eq!(r.within_120_gallon_scope, exp_in_scope);
        }
    }

    #[test]
    fn ca_zero_gallon_within_scope_boundary() {
        let mut i = ca_compliant();
        i.capacity_gallons = 0;
        let r = check(&i);
        assert!(r.within_120_gallon_scope);
    }

    #[test]
    fn ca_119_gallon_within_scope_boundary() {
        let mut i = ca_compliant();
        i.capacity_gallons = 119;
        let r = check(&i);
        assert!(r.within_120_gallon_scope);
    }

    #[test]
    fn ca_compliant_with_no_property_sale_no_cert_compliant() {
        let r = check(&ca_compliant());
        assert!(r.compliant);
        assert!(!r.seller_certification_required);
    }

    #[test]
    fn ca_tenancy_strapping_satisfied_no_cert_required() {
        let mut i = ca_compliant();
        i.is_property_sale = false;
        i.seller_certification_provided = false;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn ca_unstrapped_outside_scope_no_violation() {
        let mut i = ca_compliant();
        i.capacity_gallons = 150;
        i.strapping_satisfied = false;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.habitability_nuisance);
    }

    #[test]
    fn ca_nuisance_only_engaged_when_within_scope_and_unstrapped() {
        let mut i = ca_compliant();
        i.capacity_gallons = 200;
        i.strapping_satisfied = false;
        let r = check(&i);
        assert!(!r.habitability_nuisance);
    }
}
