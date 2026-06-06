//! Rental property sinkhole disclosure compliance — when a
//! trader-landlord operating Florida (or other karst-prone)
//! rental property must disclose past sinkhole claims, paid
//! insurance proceeds, and known sinkhole conditions to
//! prospective tenants and buyers. Trader-landlord
//! operational concern: undisclosed sinkhole history
//! creates breach of duty under FL Statute § 627.7073 +
//! Johnson v. Davis common-law material-fact disclosure
//! doctrine + buyer rescission claim + multi-tenant
//! constructive eviction exposure. Distinct from siblings
//! `rental_underground_storage_tank_disclosure` (UST),
//! `rental_basement_water_intrusion_disclosure` (water/
//! mold), `flood_disclosure`, `radon_disclosure`.
//!
//! **Two regimes**:
//!
//! **Florida — FL Statute § 627.7073 + § 627.707 + § 627.706
//! + § 689.25 + Johnson v. Davis 480 So. 2d 625 Fla. 1985**:
//! - § 627.7073(1)(c) — seller of real property upon which
//!   a sinkhole claim has been made by the seller and
//!   **paid by the insurer** must disclose to the buyer
//!   **before closing** that (1) a claim has been paid AND
//!   (2) whether or not the full amount of proceeds were
//!   used to repair the damage.
//! - § 627.707 — sinkhole loss claim investigation
//!   framework; insurer must complete testing and issue
//!   professional engineer or geologist report and
//!   certification.
//! - § 627.706 — "sinkhole loss" defined as structural
//!   damage to building consistent with sinkhole activity.
//! - § 689.25 — narrow disclosure exemptions (homicide,
//!   suicide, deaths, HIV/AIDS infection NOT material
//!   facts requiring disclosure).
//! - Johnson v. Davis, 480 So. 2d 625 (Fla. 1985) —
//!   common-law duty to disclose all material facts not
//!   readily observable affecting property value, including
//!   sinkhole conditions, applies REGARDLESS of whether
//!   insurance claim made.
//! - Florida Property Tax Disclosure Summary (§ 689.261)
//!   required at or before contract execution.
//!
//! **Default — Johnson v. Davis common-law material fact
//! doctrine + state-specific karst geology disclosure
//! requirements (other states)**:
//! - No federal mandate.
//! - Other karst-prone states (TX hill country, PA
//!   limestone regions, MO, KY, TN) typically rely on
//!   common-law material fact disclosure doctrine.
//! - Pennsylvania Real Estate Seller Disclosure Law (68
//!   Pa.C.S. § 7301 et seq.) requires Seller Disclosure
//!   Statement listing geological conditions including
//!   sinkholes.
//! - Most states do NOT impose specific landlord disclosure
//!   mandate for sinkholes (rental leases governed by
//!   common-law warranty of habitability).
//!
//! Citations: FL Statute § 627.7073(1)(c); FL Statute §
//! 627.707; FL Statute § 627.706; FL Statute § 689.25; FL
//! Statute § 689.261; Johnson v. Davis, 480 So. 2d 625
//! (Fla. 1985); 68 Pa.C.S. § 7301 et seq. (Pennsylvania
//! Real Estate Seller Disclosure Law); FL Department of
//! Environmental Protection FL Geological Survey Sinkhole
//! FAQ.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Florida,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalSinkholeDisclosureInput {
    pub regime: Regime,
    /// Whether a sinkhole insurance claim was made by
    /// seller (FL § 627.7073 trigger).
    pub sinkhole_claim_made_by_seller: bool,
    /// Whether insurer paid the sinkhole claim.
    pub insurer_paid_claim: bool,
    /// Whether full amount of insurance proceeds was used
    /// to repair the damage.
    pub full_proceeds_used_for_repair: bool,
    /// Whether property is being sold (triggers § 627.7073
    /// disclosure).
    pub property_being_sold: bool,
    /// Whether disclosure was made BEFORE closing (§
    /// 627.7073(1)(c)).
    pub disclosure_before_closing: bool,
    /// Whether landlord/seller knows of sinkhole condition
    /// (common-law material fact trigger).
    pub known_sinkhole_condition: bool,
    /// Whether known sinkhole condition was disclosed
    /// (Johnson v. Davis duty).
    pub known_condition_disclosed: bool,
    /// Whether professional engineer or geologist report
    /// completed under § 627.707.
    pub engineer_or_geologist_report_completed: bool,
    /// Whether FL Property Tax Disclosure Summary (§
    /// 689.261) was provided at or before contract.
    pub fl_property_tax_disclosure_summary_provided: bool,
    /// Whether property is rental (not sale) — disclosure
    /// requirements differ.
    pub rental_not_sale: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalSinkholeDisclosureResult {
    pub disclosure_compliant: bool,
    pub fl_section_7073_disclosure_required: bool,
    pub fl_property_tax_summary_required: bool,
    pub common_law_disclosure_required: bool,
    pub engineer_report_compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RentalSinkholeDisclosureInput) -> RentalSinkholeDisclosureResult {
    match input.regime {
        Regime::Florida => check_fl(input),
        Regime::Default => check_default(input),
    }
}

fn check_fl(input: &RentalSinkholeDisclosureInput) -> RentalSinkholeDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "FL Statute § 627.7073(1)(c) — seller of real property upon which a sinkhole claim has been made by the seller and PAID by the insurer must disclose to the buyer BEFORE CLOSING (1) that a claim has been paid AND (2) whether or not the full amount of proceeds were used to repair the damage".to_string(),
        "FL Statute § 627.707 — sinkhole loss claim investigation framework; insurer must complete testing and issue professional engineer or geologist report and certification".to_string(),
        "FL Statute § 627.706 — sinkhole loss defined as structural damage to building consistent with sinkhole activity".to_string(),
        "FL Statute § 689.25 — homicide, suicide, deaths, HIV/AIDS infection NOT material facts requiring disclosure (narrow disclosure exemptions; sinkhole conditions remain material)".to_string(),
        "FL Statute § 689.261 — Florida Property Tax Disclosure Summary required at or before contract execution".to_string(),
        "Johnson v. Davis, 480 So. 2d 625 (Fla. 1985) — common-law duty to disclose all material facts not readily observable affecting property value, including sinkhole conditions REGARDLESS of whether insurance claim made".to_string(),
    ];

    let s627_required = input.sinkhole_claim_made_by_seller
        && input.insurer_paid_claim
        && input.property_being_sold;

    if s627_required && !input.disclosure_before_closing {
        violations.push(
            "FL Statute § 627.7073(1)(c) — sinkhole-claim disclosure required BEFORE closing when seller made and insurer paid a sinkhole claim".to_string(),
        );
    }

    if input.known_sinkhole_condition && !input.known_condition_disclosed {
        violations.push(
            "Johnson v. Davis, 480 So. 2d 625 (Fla. 1985) — common-law duty requires disclosure of known sinkhole conditions affecting property value to buyer or tenant".to_string(),
        );
    }

    if input.property_being_sold && !input.fl_property_tax_disclosure_summary_provided {
        violations.push(
            "FL Statute § 689.261 — Florida Property Tax Disclosure Summary required at or before contract execution".to_string(),
        );
    }

    let engineer_ok =
        !input.sinkhole_claim_made_by_seller || input.engineer_or_geologist_report_completed;

    if input.sinkhole_claim_made_by_seller && !engineer_ok {
        violations.push(
            "FL Statute § 627.707 — sinkhole claim investigation requires completion of professional engineer or geologist report and certification".to_string(),
        );
    }

    RentalSinkholeDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        fl_section_7073_disclosure_required: s627_required,
        fl_property_tax_summary_required: input.property_being_sold,
        common_law_disclosure_required: input.known_sinkhole_condition,
        engineer_report_compliant: engineer_ok,
        violations,
        citation: "FL Statute § 627.7073(1)(c) + § 627.707 + § 627.706 + § 689.25 + § 689.261; Johnson v. Davis, 480 So. 2d 625 (Fla. 1985)",
        notes,
    }
}

fn check_default(input: &RentalSinkholeDisclosureInput) -> RentalSinkholeDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Default — no federal sinkhole disclosure mandate".to_string(),
        "Other karst-prone states (TX hill country, PA limestone regions, MO, KY, TN) typically rely on common-law material fact disclosure doctrine".to_string(),
        "Pennsylvania Real Estate Seller Disclosure Law (68 Pa.C.S. § 7301 et seq.) requires Seller Disclosure Statement listing geological conditions including sinkholes".to_string(),
        "Most states do NOT impose specific landlord disclosure mandate for sinkholes (rental leases governed by common-law warranty of habitability)".to_string(),
        "Common-law material fact doctrine — when landlord/seller knows of sinkhole condition affecting property value, duty to disclose to buyer or tenant arises under state common-law fraud + warranty of habitability framework".to_string(),
    ];

    if input.known_sinkhole_condition && !input.known_condition_disclosed {
        violations.push(
            "Common-law material fact doctrine — landlord/seller knowing of sinkhole condition has duty to disclose to buyer or tenant; concealment creates fraud claim".to_string(),
        );
    }

    RentalSinkholeDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        fl_section_7073_disclosure_required: false,
        fl_property_tax_summary_required: false,
        common_law_disclosure_required: input.known_sinkhole_condition,
        engineer_report_compliant: true,
        violations,
        citation: "Common-law material fact disclosure doctrine; 68 Pa.C.S. § 7301 et seq. (Pennsylvania Real Estate Seller Disclosure Law); state-specific karst geology disclosure requirements",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fl_clean() -> RentalSinkholeDisclosureInput {
        RentalSinkholeDisclosureInput {
            regime: Regime::Florida,
            sinkhole_claim_made_by_seller: false,
            insurer_paid_claim: false,
            full_proceeds_used_for_repair: true,
            property_being_sold: false,
            disclosure_before_closing: false,
            known_sinkhole_condition: false,
            known_condition_disclosed: false,
            engineer_or_geologist_report_completed: false,
            fl_property_tax_disclosure_summary_provided: false,
            rental_not_sale: true,
        }
    }

    fn default_clean() -> RentalSinkholeDisclosureInput {
        let mut i = fl_clean();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn fl_no_claim_no_sale_compliant() {
        let r = check(&fl_clean());
        assert!(r.disclosure_compliant);
        assert!(!r.fl_section_7073_disclosure_required);
    }

    #[test]
    fn fl_paid_claim_sale_without_disclosure_violation() {
        let mut i = fl_clean();
        i.sinkhole_claim_made_by_seller = true;
        i.insurer_paid_claim = true;
        i.property_being_sold = true;
        i.disclosure_before_closing = false;
        i.engineer_or_geologist_report_completed = true;
        i.fl_property_tax_disclosure_summary_provided = true;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.fl_section_7073_disclosure_required);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 627.7073(1)(c)") && v.contains("BEFORE closing")));
    }

    #[test]
    fn fl_paid_claim_sale_with_disclosure_compliant() {
        let mut i = fl_clean();
        i.sinkhole_claim_made_by_seller = true;
        i.insurer_paid_claim = true;
        i.property_being_sold = true;
        i.disclosure_before_closing = true;
        i.engineer_or_geologist_report_completed = true;
        i.fl_property_tax_disclosure_summary_provided = true;
        let r = check(&i);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn fl_known_sinkhole_concealment_violation() {
        let mut i = fl_clean();
        i.known_sinkhole_condition = true;
        i.known_condition_disclosed = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.common_law_disclosure_required);
        assert!(r.violations.iter().any(|v| v.contains("Johnson v. Davis")));
    }

    #[test]
    fn fl_known_sinkhole_disclosed_compliant() {
        let mut i = fl_clean();
        i.known_sinkhole_condition = true;
        i.known_condition_disclosed = true;
        let r = check(&i);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn fl_property_sold_without_tax_summary_violation() {
        let mut i = fl_clean();
        i.property_being_sold = true;
        i.fl_property_tax_disclosure_summary_provided = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 689.261") && v.contains("Property Tax Disclosure Summary")));
    }

    #[test]
    fn fl_claim_without_engineer_report_violation() {
        let mut i = fl_clean();
        i.sinkhole_claim_made_by_seller = true;
        i.insurer_paid_claim = true;
        i.property_being_sold = true;
        i.disclosure_before_closing = true;
        i.fl_property_tax_disclosure_summary_provided = true;
        i.engineer_or_geologist_report_completed = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(!r.engineer_report_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 627.707") && v.contains("professional engineer")));
    }

    #[test]
    fn fl_claim_made_but_not_paid_no_627_7073_required() {
        let mut i = fl_clean();
        i.sinkhole_claim_made_by_seller = true;
        i.insurer_paid_claim = false;
        i.property_being_sold = true;
        i.fl_property_tax_disclosure_summary_provided = true;
        i.engineer_or_geologist_report_completed = true;
        let r = check(&i);
        assert!(!r.fl_section_7073_disclosure_required);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn fl_paid_claim_no_sale_no_627_7073_required() {
        let mut i = fl_clean();
        i.sinkhole_claim_made_by_seller = true;
        i.insurer_paid_claim = true;
        i.property_being_sold = false;
        i.engineer_or_geologist_report_completed = true;
        let r = check(&i);
        assert!(!r.fl_section_7073_disclosure_required);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn default_clean_compliant() {
        let r = check(&default_clean());
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn default_known_sinkhole_concealment_violation() {
        let mut i = default_clean();
        i.known_sinkhole_condition = true;
        i.known_condition_disclosed = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.common_law_disclosure_required);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Common-law material fact doctrine")));
    }

    #[test]
    fn default_known_sinkhole_disclosed_compliant() {
        let mut i = default_clean();
        i.known_sinkhole_condition = true;
        i.known_condition_disclosed = true;
        let r = check(&i);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn citation_pins_fl_authority() {
        let r = check(&fl_clean());
        assert!(r.citation.contains("§ 627.7073(1)(c)"));
        assert!(r.citation.contains("§ 627.707"));
        assert!(r.citation.contains("§ 627.706"));
        assert!(r.citation.contains("§ 689.25"));
        assert!(r.citation.contains("§ 689.261"));
        assert!(r.citation.contains("Johnson v. Davis"));
        assert!(r.citation.contains("480 So. 2d 625"));
    }

    #[test]
    fn citation_pins_default_authority() {
        let r = check(&default_clean());
        assert!(r.citation.contains("Common-law material fact"));
        assert!(r.citation.contains("68 Pa.C.S. § 7301"));
    }

    #[test]
    fn note_pins_fl_627_7073_before_closing() {
        let r = check(&fl_clean());
        assert!(r.notes.iter().any(|n| n.contains("§ 627.7073(1)(c)")
            && n.contains("BEFORE CLOSING")
            && n.contains("full amount of proceeds")));
    }

    #[test]
    fn note_pins_fl_627_707_engineer_geologist() {
        let r = check(&fl_clean());
        assert!(r.notes.iter().any(|n| n.contains("§ 627.707")
            && n.contains("professional engineer")
            && n.contains("geologist")));
    }

    #[test]
    fn note_pins_fl_689_25_narrow_exemptions() {
        let r = check(&fl_clean());
        assert!(r.notes.iter().any(|n| n.contains("§ 689.25")
            && n.contains("homicide")
            && n.contains("HIV/AIDS")
            && n.contains("sinkhole conditions remain material")));
    }

    #[test]
    fn note_pins_fl_johnson_v_davis_common_law() {
        let r = check(&fl_clean());
        assert!(r.notes.iter().any(|n| n.contains("Johnson v. Davis")
            && n.contains("480 So. 2d 625")
            && n.contains("REGARDLESS of whether insurance claim")));
    }

    #[test]
    fn note_pins_default_pa_seller_disclosure_law() {
        let r = check(&default_clean());
        assert!(r.notes.iter().any(|n| n.contains("Pennsylvania")
            && n.contains("68 Pa.C.S. § 7301")
            && n.contains("Seller Disclosure Statement")));
    }

    #[test]
    fn note_pins_default_karst_states() {
        let r = check(&default_clean());
        assert!(r.notes.iter().any(|n| n.contains("TX hill country")
            && n.contains("PA limestone regions")
            && n.contains("MO")
            && n.contains("KY")
            && n.contains("TN")));
    }

    #[test]
    fn fl_uniquely_requires_paid_claim_disclosure_invariant() {
        let mut i_fl = fl_clean();
        i_fl.sinkhole_claim_made_by_seller = true;
        i_fl.insurer_paid_claim = true;
        i_fl.property_being_sold = true;
        i_fl.disclosure_before_closing = false;
        i_fl.engineer_or_geologist_report_completed = true;
        i_fl.fl_property_tax_disclosure_summary_provided = true;
        let r_fl = check(&i_fl);
        assert!(!r_fl.disclosure_compliant);

        let mut i_default = default_clean();
        i_default.sinkhole_claim_made_by_seller = true;
        i_default.insurer_paid_claim = true;
        i_default.property_being_sold = true;
        i_default.disclosure_before_closing = false;
        let r_default = check(&i_default);
        assert!(r_default.disclosure_compliant);
    }

    #[test]
    fn known_concealment_invariant_across_regimes() {
        for regime in [Regime::Florida, Regime::Default] {
            let mut i = match regime {
                Regime::Florida => fl_clean(),
                Regime::Default => default_clean(),
            };
            i.known_sinkhole_condition = true;
            i.known_condition_disclosed = false;
            let r = check(&i);
            assert!(!r.disclosure_compliant);
            assert!(r.common_law_disclosure_required);
        }
    }

    #[test]
    fn fl_uniquely_requires_property_tax_summary_at_sale_invariant() {
        let mut i_fl = fl_clean();
        i_fl.property_being_sold = true;
        i_fl.fl_property_tax_disclosure_summary_provided = false;
        let r_fl = check(&i_fl);
        assert!(!r_fl.disclosure_compliant);

        let mut i_default = default_clean();
        i_default.property_being_sold = true;
        let r_default = check(&i_default);
        assert!(r_default.disclosure_compliant);
    }

    #[test]
    fn multiple_fl_violations_stack() {
        let mut i = fl_clean();
        i.sinkhole_claim_made_by_seller = true;
        i.insurer_paid_claim = true;
        i.property_being_sold = true;
        i.disclosure_before_closing = false;
        i.engineer_or_geologist_report_completed = false;
        i.fl_property_tax_disclosure_summary_provided = false;
        i.known_sinkhole_condition = true;
        i.known_condition_disclosed = false;
        let r = check(&i);
        assert_eq!(r.violations.len(), 4);
    }

    #[test]
    fn defensive_all_zero_no_violations() {
        let r = check(&fl_clean());
        assert!(r.disclosure_compliant);
        assert!(!r.fl_section_7073_disclosure_required);
        assert!(!r.fl_property_tax_summary_required);
    }
}
