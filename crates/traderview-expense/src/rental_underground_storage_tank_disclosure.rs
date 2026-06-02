//! Rental property underground storage tank (UST)
//! disclosure compliance — when a trader-landlord operating
//! a property with an active, inactive, or abandoned-in-place
//! underground storage tank (home heating oil tank,
//! industrial chemical tank, abandoned gas station tank)
//! must disclose UST presence to prospective and existing
//! tenants and prospective buyers. Trader-landlord
//! operational concern: undisclosed UST contamination
//! creates joint-and-several CERCLA Superfund liability +
//! state environmental cleanup orders + multi-million-dollar
//! remediation cost exposure + buyer rescission claim. UST
//! contamination is one of the largest single landlord-
//! exposure categories. Distinct from siblings
//! `rental_basement_water_intrusion_disclosure` (water/
//! mold), `radon_disclosure`, `asbestos_disclosure`,
//! `mold_disclosure`.
//!
//! **Four regimes**:
//!
//! **Federal — RCRA Subtitle I (42 USC § 6991 et seq.) +
//! 40 CFR Part 280 (EPA Underground Storage Tank
//! regulations)**:
//! - Federal regulatory program for petroleum and CERCLA
//!   hazardous substance USTs containing **more than 10%
//!   below ground surface**.
//! - **Heating oil USTs serving on-premises residential**
//!   buildings EXEMPT from federal regulation but subject
//!   to state regulation.
//! - 2015 EPA final rule expanded UST integrity testing
//!   + secondary containment requirements.
//! - Federal program enforced through delegated state UST
//!   programs.
//!
//! **California — Cal. Health & Safety Code Chapter 6.7
//! §§ 25280-25299.8 + § 25288 (annual inspection)**:
//! - California State Water Resources Control Board
//!   administers UST program through Certified Unified
//!   Program Agencies (CUPAs).
//! - **Annual inspection required** of all USTs, including
//!   abandoned USTs (§ 25288).
//! - Cal. Civ. Code § 1102 et seq. — Transfer Disclosure
//!   Statement (TDS) must disclose UST on property sale.
//! - California residential leases generally do NOT require
//!   specific UST disclosure (no statutory affirmative
//!   duty), but common-law fraud claim available if
//!   landlord conceals known UST contamination.
//!
//! **Florida — FL Statute § 376.30-376.317 + FL Statute §
//! 689.25 (residential seller disclosure)**:
//! - FL Storage Tank System Rules administered by FL
//!   Department of Environmental Protection (FDEP).
//! - FL § 376.317 — petroleum UST regulation supersedes
//!   conflicting state or local laws.
//! - FL § 689.25 + Johnson v. Davis common-law duty —
//!   home sellers must disclose all material facts not
//!   readily observable affecting property value,
//!   INCLUDING UST presence and contamination.
//! - FL Florida Petroleum Liability Insurance and
//!   Restoration Program — state-funded cleanup for
//!   eligible USTs.
//!
//! **New Jersey — NJDEP UST Program (N.J.A.C. 7:14B) + NJ
//! Seller's Property Condition Disclosure Statement**:
//! - **STRICTEST among comparators**.
//! - NJ Seller's Property Condition Disclosure Statement
//!   **explicitly requires disclosure of UST presence**,
//!   whether active, inactive, or abandoned in place,
//!   INCLUDING tanks landlord knows about but hasn't
//!   addressed.
//! - Buyer/tenant may request documentation (photos, soil
//!   analyses, NJDEP certifications for contractor) to
//!   verify proper removal or abandonment.
//! - **Unregulated Heating Oil Tank Program** (NJEDA grant
//!   funding) provides up to $250,000 grant for residential
//!   heating oil tank removal/cleanup.
//!
//! Citations: 42 USC § 6991 et seq. RCRA Subtitle I and 40
//! CFR Part 280 EPA UST regulations and Cal. Health & Safety
//! Code §§ 25280-25299.8 + § 25288 + Chapter 6.7 and Cal.
//! Civ. Code § 1102 et seq. TDS and FL Statute §§ 376.30-
//! 376.317 + § 689.25 and Johnson v. Davis 480 So. 2d 625
//! Fla. 1985 and N.J.A.C. 7:14B and NJDEP Property Condition
//! Disclosure Statement and NJEDA Petroleum Underground
//! Storage Tank Program PUSTP.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Federal,
    California,
    Florida,
    NewJersey,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TankStatus {
    /// Active UST in current operation.
    Active,
    /// Inactive UST (out of service but not removed).
    Inactive,
    /// Abandoned in place (never removed).
    AbandonedInPlace,
    /// Removed UST with documented cleanup.
    RemovedWithDocumentation,
    /// No UST on property.
    NoTank,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalUndergroundStorageTankDisclosureInput {
    pub regime: Regime,
    pub tank_status: TankStatus,
    /// Whether tank is heating oil tank serving on-premises
    /// residential (federal RCRA Subtitle I exemption).
    pub heating_oil_residential_on_premises: bool,
    /// Whether annual inspection was completed (CA § 25288).
    pub annual_inspection_completed: bool,
    /// Whether Transfer Disclosure Statement disclosed UST
    /// at sale (CA Civ Code § 1102 et seq.).
    pub ca_tds_disclosed_at_sale: bool,
    /// Whether NJ Seller's Property Condition Disclosure
    /// Statement disclosed UST (NJ STRICTEST).
    pub nj_property_disclosure_statement_disclosed: bool,
    /// Whether FL § 689.25 + Johnson v. Davis material fact
    /// disclosure to buyer (FL).
    pub fl_johnson_davis_material_disclosure: bool,
    /// Whether property is being sold (triggers TDS/seller
    /// disclosure obligations).
    pub property_being_sold: bool,
    /// Whether contamination is known to landlord but
    /// undisclosed (common-law fraud trigger).
    pub known_contamination_concealed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalUndergroundStorageTankDisclosureResult {
    pub disclosure_compliant: bool,
    pub federal_rcra_engaged: bool,
    pub state_program_engaged: bool,
    pub sale_disclosure_required: bool,
    pub annual_inspection_compliant: bool,
    pub fraud_concealment_violation: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &RentalUndergroundStorageTankDisclosureInput,
) -> RentalUndergroundStorageTankDisclosureResult {
    match input.regime {
        Regime::Federal => check_federal(input),
        Regime::California => check_ca(input),
        Regime::Florida => check_fl(input),
        Regime::NewJersey => check_nj(input),
    }
}

fn check_federal(
    input: &RentalUndergroundStorageTankDisclosureInput,
) -> RentalUndergroundStorageTankDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "42 USC § 6991 et seq. (RCRA Subtitle I) + 40 CFR Part 280 — federal regulatory program for petroleum and CERCLA hazardous substance USTs containing more than 10% below ground surface".to_string(),
        "RCRA Subtitle I — heating oil USTs serving on-premises residential buildings EXEMPT from federal regulation but subject to state regulation".to_string(),
        "2015 EPA final rule expanded UST integrity testing + secondary containment requirements".to_string(),
        "Federal program enforced through delegated state UST programs; states may impose stricter requirements".to_string(),
        "Federal CERCLA joint-and-several liability for contamination remains regardless of UST regulatory exemption".to_string(),
    ];

    let tank_exists = !matches!(input.tank_status, TankStatus::NoTank);
    let regulated = tank_exists && !input.heating_oil_residential_on_premises;

    if regulated && !input.annual_inspection_completed
        && !matches!(input.tank_status, TankStatus::RemovedWithDocumentation)
    {
        violations.push(
            "40 CFR Part 280 — federally regulated USTs require periodic integrity testing and inspection".to_string(),
        );
    }

    RentalUndergroundStorageTankDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        federal_rcra_engaged: regulated,
        state_program_engaged: false,
        sale_disclosure_required: false,
        annual_inspection_compliant: !regulated || input.annual_inspection_completed,
        fraud_concealment_violation: input.known_contamination_concealed,
        violations,
        citation: "42 USC § 6991 et seq. (RCRA Subtitle I); 40 CFR Part 280; 2015 EPA Final Rule",
        notes,
    }
}

fn check_ca(
    input: &RentalUndergroundStorageTankDisclosureInput,
) -> RentalUndergroundStorageTankDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Health & Safety Code §§ 25280-25299.8 + Chapter 6.7 — California State Water Resources Control Board administers UST program through Certified Unified Program Agencies (CUPAs)".to_string(),
        "Cal. Health & Safety Code § 25288 — annual inspection required of all USTs INCLUDING abandoned USTs".to_string(),
        "Cal. Civ. Code § 1102 et seq. — Transfer Disclosure Statement (TDS) must disclose UST on property sale".to_string(),
        "California residential leases do NOT require specific UST disclosure (no statutory affirmative duty); common-law fraud claim available if landlord conceals known UST contamination".to_string(),
        "California Underground Storage Tank Cleanup Fund — state-funded cleanup for eligible UST contamination".to_string(),
    ];

    let tank_exists = !matches!(input.tank_status, TankStatus::NoTank);

    if tank_exists && !input.annual_inspection_completed
        && !matches!(input.tank_status, TankStatus::RemovedWithDocumentation)
    {
        violations.push(
            "Cal. Health & Safety Code § 25288 — annual inspection required of all USTs including abandoned USTs".to_string(),
        );
    }

    if tank_exists && input.property_being_sold && !input.ca_tds_disclosed_at_sale {
        violations.push(
            "Cal. Civ. Code § 1102 et seq. — Transfer Disclosure Statement must disclose UST presence on property sale".to_string(),
        );
    }

    if input.known_contamination_concealed {
        violations.push(
            "California common-law fraud doctrine — landlord may not conceal known UST contamination from tenant or buyer".to_string(),
        );
    }

    RentalUndergroundStorageTankDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        federal_rcra_engaged: tank_exists && !input.heating_oil_residential_on_premises,
        state_program_engaged: tank_exists,
        sale_disclosure_required: tank_exists && input.property_being_sold,
        annual_inspection_compliant: !tank_exists || input.annual_inspection_completed
            || matches!(input.tank_status, TankStatus::RemovedWithDocumentation),
        fraud_concealment_violation: input.known_contamination_concealed,
        violations,
        citation: "Cal. Health & Safety Code §§ 25280-25299.8 + § 25288 + Chapter 6.7; Cal. Civ. Code § 1102 et seq.",
        notes,
    }
}

fn check_fl(
    input: &RentalUndergroundStorageTankDisclosureInput,
) -> RentalUndergroundStorageTankDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "FL Statute § 376.30-376.317 — FL Storage Tank System Rules administered by FL Department of Environmental Protection (FDEP)".to_string(),
        "FL Statute § 376.317 — petroleum UST regulation supersedes conflicting state or local laws".to_string(),
        "FL Statute § 689.25 + Johnson v. Davis, 480 So. 2d 625 (Fla. 1985) — home sellers must disclose all material facts not readily observable affecting property value INCLUDING UST presence and contamination".to_string(),
        "FL Petroleum Liability Insurance and Restoration Program — state-funded cleanup for eligible USTs".to_string(),
        "FL framework supersedes local UST regulation under § 376.317 preemption clause".to_string(),
    ];

    let tank_exists = !matches!(input.tank_status, TankStatus::NoTank);

    if tank_exists && input.property_being_sold && !input.fl_johnson_davis_material_disclosure {
        violations.push(
            "FL Statute § 689.25 + Johnson v. Davis, 480 So. 2d 625 (Fla. 1985) — sellers must disclose all material facts including UST presence and contamination".to_string(),
        );
    }

    if input.known_contamination_concealed {
        violations.push(
            "Johnson v. Davis common-law disclosure doctrine — landlord may not conceal known UST contamination affecting property value".to_string(),
        );
    }

    RentalUndergroundStorageTankDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        federal_rcra_engaged: tank_exists && !input.heating_oil_residential_on_premises,
        state_program_engaged: tank_exists,
        sale_disclosure_required: tank_exists && input.property_being_sold,
        annual_inspection_compliant: true,
        fraud_concealment_violation: input.known_contamination_concealed,
        violations,
        citation: "FL Statute §§ 376.30-376.317 + § 689.25; Johnson v. Davis, 480 So. 2d 625 (Fla. 1985); FL Petroleum Liability Insurance and Restoration Program",
        notes,
    }
}

fn check_nj(
    input: &RentalUndergroundStorageTankDisclosureInput,
) -> RentalUndergroundStorageTankDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "N.J.A.C. 7:14B — NJDEP UST Program; STRICTEST UST disclosure regime among comparators".to_string(),
        "NJ Seller's Property Condition Disclosure Statement explicitly requires disclosure of UST presence — whether ACTIVE + INACTIVE + ABANDONED IN PLACE — including tanks landlord knows about but hasn't addressed".to_string(),
        "NJ buyer/tenant may request documentation (photos + soil analyses + NJDEP certifications for contractor) to verify proper removal or abandonment".to_string(),
        "NJ Unregulated Heating Oil Tank Program (NJEDA grant funding) — up to $250,000 grant for residential heating oil tank removal/cleanup".to_string(),
        "NJ Spill Compensation and Control Act (N.J.S.A. 58:10-23.11) — strict joint-and-several liability for petroleum contamination".to_string(),
    ];

    let tank_exists = !matches!(input.tank_status, TankStatus::NoTank);

    if tank_exists && !input.nj_property_disclosure_statement_disclosed {
        violations.push(
            "NJ Seller's Property Condition Disclosure Statement — UST presence must be disclosed (active, inactive, OR abandoned in place; whether or not currently addressed)".to_string(),
        );
    }

    if input.known_contamination_concealed {
        violations.push(
            "NJ Spill Compensation and Control Act (N.J.S.A. 58:10-23.11) + common-law fraud — landlord may not conceal known UST contamination; strict joint-and-several liability".to_string(),
        );
    }

    RentalUndergroundStorageTankDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        federal_rcra_engaged: tank_exists && !input.heating_oil_residential_on_premises,
        state_program_engaged: tank_exists,
        sale_disclosure_required: tank_exists,
        annual_inspection_compliant: true,
        fraud_concealment_violation: input.known_contamination_concealed,
        violations,
        citation: "N.J.A.C. 7:14B; NJ Seller's Property Condition Disclosure Statement; NJ Spill Compensation and Control Act (N.J.S.A. 58:10-23.11); NJEDA Petroleum Underground Storage Tank Program (PUSTP)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fed_clean() -> RentalUndergroundStorageTankDisclosureInput {
        RentalUndergroundStorageTankDisclosureInput {
            regime: Regime::Federal,
            tank_status: TankStatus::Active,
            heating_oil_residential_on_premises: false,
            annual_inspection_completed: true,
            ca_tds_disclosed_at_sale: false,
            nj_property_disclosure_statement_disclosed: false,
            fl_johnson_davis_material_disclosure: false,
            property_being_sold: false,
            known_contamination_concealed: false,
        }
    }

    fn ca_clean() -> RentalUndergroundStorageTankDisclosureInput {
        let mut i = fed_clean();
        i.regime = Regime::California;
        i.ca_tds_disclosed_at_sale = true;
        i
    }

    fn fl_clean() -> RentalUndergroundStorageTankDisclosureInput {
        let mut i = fed_clean();
        i.regime = Regime::Florida;
        i.fl_johnson_davis_material_disclosure = true;
        i
    }

    fn nj_clean() -> RentalUndergroundStorageTankDisclosureInput {
        let mut i = fed_clean();
        i.regime = Regime::NewJersey;
        i.nj_property_disclosure_statement_disclosed = true;
        i
    }

    #[test]
    fn fed_clean_compliant() {
        let r = check(&fed_clean());
        assert!(r.disclosure_compliant);
        assert!(r.federal_rcra_engaged);
    }

    #[test]
    fn fed_no_inspection_violation() {
        let mut i = fed_clean();
        i.annual_inspection_completed = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("40 CFR Part 280")));
    }

    #[test]
    fn fed_heating_oil_residential_exempt() {
        let mut i = fed_clean();
        i.heating_oil_residential_on_premises = true;
        i.annual_inspection_completed = false;
        let r = check(&i);
        assert!(r.disclosure_compliant);
        assert!(!r.federal_rcra_engaged);
    }

    #[test]
    fn fed_removed_with_documentation_no_inspection_required() {
        let mut i = fed_clean();
        i.tank_status = TankStatus::RemovedWithDocumentation;
        i.annual_inspection_completed = false;
        let r = check(&i);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn ca_clean_compliant() {
        let r = check(&ca_clean());
        assert!(r.disclosure_compliant);
        assert!(r.state_program_engaged);
    }

    #[test]
    fn ca_no_annual_inspection_violation() {
        let mut i = ca_clean();
        i.annual_inspection_completed = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 25288") && v.contains("annual inspection")));
    }

    #[test]
    fn ca_property_sold_without_tds_violation() {
        let mut i = ca_clean();
        i.property_being_sold = true;
        i.ca_tds_disclosed_at_sale = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1102") && v.contains("Transfer Disclosure Statement")));
    }

    #[test]
    fn ca_property_sold_with_tds_compliant() {
        let mut i = ca_clean();
        i.property_being_sold = true;
        i.ca_tds_disclosed_at_sale = true;
        let r = check(&i);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn ca_known_concealment_fraud_violation() {
        let mut i = ca_clean();
        i.known_contamination_concealed = true;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.fraud_concealment_violation);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("common-law fraud")));
    }

    #[test]
    fn fl_clean_compliant() {
        let r = check(&fl_clean());
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn fl_property_sold_without_johnson_davis_violation() {
        let mut i = fl_clean();
        i.property_being_sold = true;
        i.fl_johnson_davis_material_disclosure = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 689.25") && v.contains("Johnson v. Davis")));
    }

    #[test]
    fn fl_known_concealment_violation() {
        let mut i = fl_clean();
        i.known_contamination_concealed = true;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Johnson v. Davis")));
    }

    #[test]
    fn nj_clean_compliant() {
        let r = check(&nj_clean());
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn nj_no_property_disclosure_violation() {
        let mut i = nj_clean();
        i.nj_property_disclosure_statement_disclosed = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Seller's Property Condition Disclosure Statement")));
    }

    #[test]
    fn nj_abandoned_in_place_must_be_disclosed() {
        let mut i = nj_clean();
        i.tank_status = TankStatus::AbandonedInPlace;
        i.nj_property_disclosure_statement_disclosed = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
    }

    #[test]
    fn nj_known_concealment_strict_liability() {
        let mut i = nj_clean();
        i.known_contamination_concealed = true;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Spill Compensation and Control Act")
                && v.contains("joint-and-several")));
    }

    #[test]
    fn citation_pins_federal_authority() {
        let r = check(&fed_clean());
        assert!(r.citation.contains("42 USC § 6991"));
        assert!(r.citation.contains("40 CFR Part 280"));
        assert!(r.citation.contains("2015 EPA"));
    }

    #[test]
    fn citation_pins_ca_authority() {
        let r = check(&ca_clean());
        assert!(r.citation.contains("§§ 25280-25299.8"));
        assert!(r.citation.contains("§ 25288"));
        assert!(r.citation.contains("§ 1102"));
    }

    #[test]
    fn citation_pins_fl_authority() {
        let r = check(&fl_clean());
        assert!(r.citation.contains("§§ 376.30-376.317"));
        assert!(r.citation.contains("§ 689.25"));
        assert!(r.citation.contains("Johnson v. Davis"));
        assert!(r.citation.contains("480 So. 2d 625"));
    }

    #[test]
    fn citation_pins_nj_authority() {
        let r = check(&nj_clean());
        assert!(r.citation.contains("7:14B"));
        assert!(r.citation.contains("58:10-23.11"));
        assert!(r.citation.contains("PUSTP"));
    }

    #[test]
    fn note_pins_federal_heating_oil_residential_exemption() {
        let r = check(&fed_clean());
        assert!(r.notes.iter().any(|n| n.contains("heating oil USTs")
            && n.contains("EXEMPT from federal")
            && n.contains("state regulation")));
    }

    #[test]
    fn note_pins_ca_section_25288_annual_inspection() {
        let r = check(&ca_clean());
        assert!(r.notes.iter().any(|n| n.contains("§ 25288")
            && n.contains("annual inspection")
            && n.contains("abandoned USTs")));
    }

    #[test]
    fn note_pins_fl_johnson_davis_doctrine() {
        let r = check(&fl_clean());
        assert!(r.notes.iter().any(|n| n.contains("Johnson v. Davis")
            && n.contains("480 So. 2d 625")));
    }

    #[test]
    fn note_pins_nj_strictest_three_tank_statuses() {
        let r = check(&nj_clean());
        assert!(r.notes.iter().any(|n| n.contains("STRICTEST")));
        assert!(r.notes.iter().any(|n| n.contains("ACTIVE")
            && n.contains("INACTIVE")
            && n.contains("ABANDONED IN PLACE")));
    }

    #[test]
    fn note_pins_nj_unregulated_heating_oil_grant() {
        let r = check(&nj_clean());
        assert!(r.notes.iter().any(|n| n.contains("$250,000 grant")
            && n.contains("NJEDA")));
    }

    #[test]
    fn no_tank_no_violations_regardless_of_inputs() {
        for regime in [
            Regime::Federal,
            Regime::California,
            Regime::Florida,
            Regime::NewJersey,
        ] {
            let mut i = fed_clean();
            i.regime = regime;
            i.tank_status = TankStatus::NoTank;
            i.annual_inspection_completed = false;
            let r = check(&i);
            assert!(r.disclosure_compliant);
            assert!(!r.state_program_engaged);
        }
    }

    #[test]
    fn nj_uniquely_requires_property_disclosure_regardless_of_sale_invariant() {
        let mut i_nj = nj_clean();
        i_nj.nj_property_disclosure_statement_disclosed = false;
        i_nj.property_being_sold = false;
        let r_nj = check(&i_nj);
        assert!(!r_nj.disclosure_compliant);

        let mut i_ca = ca_clean();
        i_ca.ca_tds_disclosed_at_sale = false;
        i_ca.property_being_sold = false;
        let r_ca = check(&i_ca);
        assert!(r_ca.disclosure_compliant);
    }

    #[test]
    fn ca_uniquely_requires_annual_inspection_invariant() {
        let mut i_ca = ca_clean();
        i_ca.annual_inspection_completed = false;
        let r_ca = check(&i_ca);
        assert!(!r_ca.disclosure_compliant);

        let mut i_fl = fl_clean();
        i_fl.annual_inspection_completed = false;
        let r_fl = check(&i_fl);
        assert!(r_fl.disclosure_compliant);
    }

    #[test]
    fn known_concealment_invariant_across_regimes() {
        for regime in [Regime::California, Regime::Florida, Regime::NewJersey] {
            let mut i = match regime {
                Regime::California => ca_clean(),
                Regime::Florida => fl_clean(),
                Regime::NewJersey => nj_clean(),
                _ => fed_clean(),
            };
            i.known_contamination_concealed = true;
            let r = check(&i);
            assert!(r.fraud_concealment_violation);
            assert!(!r.disclosure_compliant);
        }
    }

    #[test]
    fn tank_status_truth_table_five_cells() {
        for status in [
            TankStatus::Active,
            TankStatus::Inactive,
            TankStatus::AbandonedInPlace,
            TankStatus::RemovedWithDocumentation,
            TankStatus::NoTank,
        ] {
            let mut i = nj_clean();
            i.tank_status = status;
            if !matches!(status, TankStatus::NoTank) {
                i.nj_property_disclosure_statement_disclosed = true;
            }
            let r = check(&i);
            if matches!(status, TankStatus::NoTank) {
                assert!(!r.state_program_engaged);
            } else {
                assert!(r.state_program_engaged);
            }
        }
    }

    #[test]
    fn multiple_nj_violations_stack() {
        let mut i = nj_clean();
        i.nj_property_disclosure_statement_disclosed = false;
        i.known_contamination_concealed = true;
        let r = check(&i);
        assert_eq!(r.violations.len(), 2);
    }
}
