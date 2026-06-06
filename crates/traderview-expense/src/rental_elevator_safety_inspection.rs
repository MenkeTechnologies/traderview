//! Rental property elevator safety inspection compliance —
//! when a trader-landlord operating a multifamily building
//! with elevators must comply with state-specific periodic
//! inspection, testing, and certification requirements
//! grounded in ASME A17.1 Safety Code for Elevators and
//! Escalators. Trader-landlord operational concern:
//! expired elevator certification creates strict-liability
//! exposure for elevator injury claims + per-day civil
//! penalty + emergency department shutdown authority + tort
//! liability multiplier. Distinct from siblings `rental_
//! swimming_pool_drain_safety` (federal VGB Act anti-
//! entrapment), `rental_carbon_monoxide_detector`, `rental_
//! bedroom_egress_window`, `soft_story_seismic_retrofit`.
//!
//! **Three regimes**:
//!
//! **California — Cal. Labor Code §§ 7300-7324.2 + Title 8
//! Subchapter 6 Elevator Safety Orders + Cal/OSHA Elevator
//! Unit**:
//! - Cal/OSHA Elevator Unit at California Department of
//!   Industrial Relations (DIR) issues elevator permits.
//! - **Annual inspection required**; permit valid 1 year.
//! - Inspectors must be Cal/OSHA-certified per Cal. Labor
//!   Code § 7317 (at least 4 years mechanical/electrical
//!   experience including 1 year elevator-specific).
//! - Adopts ASME A17.1 by reference under Cal. Labor Code
//!   § 7300.1.
//! - Permit-to-operate (Form 80) required and must be
//!   posted; $200/day civil penalty for operation without
//!   valid permit under Cal. Labor Code § 7320.
//!
//! **New York City — NYC Admin Code § 28-304 + NYC Building
//! Code Chapter 30 + Table N1 of ASME A17.1 as modified by
//! Chapter K1 of Appendix K**:
//! - **Two annual inspections required**: Category 1 test
//!   (PCT) by approved elevator agency + Department of
//!   Buildings (DOB) periodic inspection.
//! - **Category 5 test** required every 5 years; Category
//!   3 hydraulic test every 3 years.
//! - Inspections must be performed by **DOB-approved
//!   elevator agency** (private firms certified under
//!   § 28-304.6.1).
//! - **PVT-A Form** (Periodic Test and Inspection
//!   Affirmation) must be filed with DOB within 60 days of
//!   inspection.
//! - Civil penalty $3,000-$10,000 per violation under §
//!   28-304.6.5 + DOB stop-use order authority.
//!
//! **Default — ASME A17.1-2025 (Safety Code for Elevators
//! and Escalators) + state-specific adoption**:
//! - ASME A17.1-2025 published; **periodic inspection per
//!   Table N1 of ASME A17.1**.
//! - State adoption timelines vary; many states adopt with
//!   local amendments.
//! - QEI (Qualified Elevator Inspector) certification by
//!   ASME or state authority.
//! - **Category 1 test (annual)** + Category 3 (3-year
//!   hydraulic) + Category 5 (5-year full).
//! - Building owner responsible for engaging certified
//!   inspector + maintaining inspection records on premises.
//!
//! Citations: Cal. Labor Code §§ 7300-7324.2 + § 7317 +
//! § 7320; Cal. Title 8 Subchapter 6 Elevator Safety
//! Orders; NYC Admin Code § 28-304 + § 28-304.6.1 + § 28-
//! 304.6.5; NYC Building Code Chapter 30 + Appendix K
//! Chapter K1; ASME A17.1-2025 (Safety Code for Elevators
//! and Escalators); ANSI/ASME A17.1/CSA B44-2025; QEI
//! Qualified Elevator Inspector certification.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    NewYorkCity,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InspectorCertification {
    /// Cal/OSHA-certified elevator inspector (CA Labor Code
    /// § 7317).
    CalOshaCertified,
    /// NYC DOB-approved elevator agency (§ 28-304.6.1).
    NycDobApprovedAgency,
    /// QEI (Qualified Elevator Inspector) ASME-certified.
    QeiCertified,
    /// No certification.
    None,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalElevatorSafetyInspectionInput {
    pub regime: Regime,
    pub inspector_certification: InspectorCertification,
    /// Days since last Category 1 (annual) inspection.
    pub days_since_category_1_inspection: u32,
    /// Days since last Category 3 (hydraulic 3-year)
    /// inspection.
    pub days_since_category_3_inspection: u32,
    /// Days since last Category 5 (full 5-year) inspection.
    pub days_since_category_5_inspection: u32,
    /// Whether elevator is hydraulic (Category 3 test
    /// applies).
    pub elevator_is_hydraulic: bool,
    /// Whether Cal/OSHA Form 80 permit-to-operate is posted
    /// (CA requirement).
    pub ca_form_80_posted: bool,
    /// Whether NYC PVT-A Form was filed with DOB within 60
    /// days of inspection (NYC requirement).
    pub nyc_pvta_filed_within_60_days: bool,
    /// CA civil penalty assessed in cents ($200/day under §
    /// 7320).
    pub ca_per_day_penalty_cents: u64,
    /// NYC civil penalty assessed in cents ($3,000-$10,000
    /// per violation under § 28-304.6.5).
    pub nyc_per_violation_penalty_cents: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalElevatorSafetyInspectionResult {
    pub compliant: bool,
    pub category_1_annual_compliant: bool,
    pub category_3_hydraulic_compliant: bool,
    pub category_5_five_year_compliant: bool,
    pub inspector_qualified: bool,
    pub ca_permit_posted_compliant: bool,
    pub nyc_pvta_filed_compliant: bool,
    pub ca_penalty_in_statutory_range: bool,
    pub nyc_penalty_in_statutory_range: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RentalElevatorSafetyInspectionInput) -> RentalElevatorSafetyInspectionResult {
    match input.regime {
        Regime::California => check_ca(input),
        Regime::NewYorkCity => check_nyc(input),
        Regime::Default => check_default(input),
    }
}

fn check_ca(input: &RentalElevatorSafetyInspectionInput) -> RentalElevatorSafetyInspectionResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Labor Code §§ 7300-7324.2 + Title 8 Subchapter 6 Elevator Safety Orders — Cal/OSHA Elevator Unit issues elevator permits".to_string(),
        "Cal. Labor Code § 7300.1 — adopts ASME A17.1 Safety Code for Elevators and Escalators by reference".to_string(),
        "Cal. Labor Code § 7317 — inspectors must have at least 4 years mechanical/electrical experience including 1 year elevator-specific".to_string(),
        "Cal. Labor Code § 7320 — $200/day civil penalty for operation without valid permit; permit-to-operate (Form 80) required and must be posted".to_string(),
        "Cal/OSHA — annual inspection required; permit valid 1 year".to_string(),
    ];

    let cat1_compliant = input.days_since_category_1_inspection <= 365;
    if !cat1_compliant {
        violations.push(
            "Cal. Labor Code § 7320 + Cal/OSHA Elevator Safety Orders — annual elevator inspection required; permit-to-operate expires after 1 year".to_string(),
        );
    }

    let inspector_ok = matches!(
        input.inspector_certification,
        InspectorCertification::CalOshaCertified
    );
    if !inspector_ok {
        violations.push(
            "Cal. Labor Code § 7317 — inspections must be performed by Cal/OSHA-certified elevator inspector".to_string(),
        );
    }

    if !input.ca_form_80_posted {
        violations.push(
            "Cal. Labor Code § 7320 — permit-to-operate (Cal/OSHA Form 80) must be posted in elevator".to_string(),
        );
    }

    const CA_PENALTY_PER_DAY_CENTS: u64 = 20_000;
    let ca_penalty_in_range = input.ca_per_day_penalty_cents == 0
        || input.ca_per_day_penalty_cents == CA_PENALTY_PER_DAY_CENTS;
    if input.ca_per_day_penalty_cents > 0 && !ca_penalty_in_range {
        violations.push(
            "Cal. Labor Code § 7320 — civil penalty fixed at $200/day for operation without valid permit".to_string(),
        );
    }

    RentalElevatorSafetyInspectionResult {
        compliant: violations.is_empty(),
        category_1_annual_compliant: cat1_compliant,
        category_3_hydraulic_compliant: true,
        category_5_five_year_compliant: true,
        inspector_qualified: inspector_ok,
        ca_permit_posted_compliant: input.ca_form_80_posted,
        nyc_pvta_filed_compliant: true,
        ca_penalty_in_statutory_range: ca_penalty_in_range,
        nyc_penalty_in_statutory_range: true,
        violations,
        citation: "Cal. Labor Code §§ 7300-7324.2 + § 7317 + § 7320; Cal. Title 8 Subchapter 6 Elevator Safety Orders; ASME A17.1",
        notes,
    }
}

fn check_nyc(input: &RentalElevatorSafetyInspectionInput) -> RentalElevatorSafetyInspectionResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "NYC Admin Code § 28-304 + NYC Building Code Chapter 30 + Table N1 of ASME A17.1 as modified by Chapter K1 of Appendix K".to_string(),
        "Two annual inspections required: Category 1 test (PCT) by approved elevator agency + Department of Buildings (DOB) periodic inspection".to_string(),
        "Category 5 test required every 5 years; Category 3 hydraulic test every 3 years".to_string(),
        "NYC Admin Code § 28-304.6.1 — inspections must be performed by DOB-approved elevator agency".to_string(),
        "PVT-A Form (Periodic Test and Inspection Affirmation) must be filed with DOB within 60 days of inspection".to_string(),
        "NYC Admin Code § 28-304.6.5 — civil penalty $3,000-$10,000 per violation + DOB stop-use order authority".to_string(),
    ];

    let cat1_compliant = input.days_since_category_1_inspection <= 365;
    if !cat1_compliant {
        violations.push(
            "NYC Admin Code § 28-304 — Category 1 (PCT) annual elevator inspection required by approved elevator agency".to_string(),
        );
    }

    let cat3_compliant =
        !input.elevator_is_hydraulic || input.days_since_category_3_inspection <= 365 * 3;
    if input.elevator_is_hydraulic && !cat3_compliant {
        violations.push(
            "NYC Building Code Chapter 30 + ASME A17.1 Table N1 — Category 3 hydraulic test required every 3 years".to_string(),
        );
    }

    let cat5_compliant = input.days_since_category_5_inspection <= 365 * 5;
    if !cat5_compliant {
        violations.push(
            "NYC Building Code Chapter 30 + ASME A17.1 Table N1 — Category 5 full test required every 5 years".to_string(),
        );
    }

    let inspector_ok = matches!(
        input.inspector_certification,
        InspectorCertification::NycDobApprovedAgency
    );
    if !inspector_ok {
        violations.push(
            "NYC Admin Code § 28-304.6.1 — inspections must be performed by DOB-approved elevator agency".to_string(),
        );
    }

    if !input.nyc_pvta_filed_within_60_days {
        violations.push(
            "NYC Admin Code § 28-304 — PVT-A Form must be filed with DOB within 60 days of inspection".to_string(),
        );
    }

    const NYC_PENALTY_MIN_CENTS: u64 = 300_000;
    const NYC_PENALTY_MAX_CENTS: u64 = 1_000_000;
    let nyc_penalty_in_range = input.nyc_per_violation_penalty_cents == 0
        || (input.nyc_per_violation_penalty_cents >= NYC_PENALTY_MIN_CENTS
            && input.nyc_per_violation_penalty_cents <= NYC_PENALTY_MAX_CENTS);
    if !nyc_penalty_in_range {
        violations.push(
            "NYC Admin Code § 28-304.6.5 — civil penalty must be between $3,000 and $10,000 per violation".to_string(),
        );
    }

    RentalElevatorSafetyInspectionResult {
        compliant: violations.is_empty(),
        category_1_annual_compliant: cat1_compliant,
        category_3_hydraulic_compliant: cat3_compliant,
        category_5_five_year_compliant: cat5_compliant,
        inspector_qualified: inspector_ok,
        ca_permit_posted_compliant: true,
        nyc_pvta_filed_compliant: input.nyc_pvta_filed_within_60_days,
        ca_penalty_in_statutory_range: true,
        nyc_penalty_in_statutory_range: nyc_penalty_in_range,
        violations,
        citation: "NYC Admin Code § 28-304 + § 28-304.6.1 + § 28-304.6.5; NYC Building Code Chapter 30 + Appendix K Chapter K1; ASME A17.1 Table N1",
        notes,
    }
}

fn check_default(
    input: &RentalElevatorSafetyInspectionInput,
) -> RentalElevatorSafetyInspectionResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "ASME A17.1-2025 (Safety Code for Elevators and Escalators) — periodic inspection per Table N1; state adoption timelines vary".to_string(),
        "ANSI/ASME A17.1/CSA B44-2025 — joint US-Canada safety standard".to_string(),
        "QEI Qualified Elevator Inspector — ASME-certified inspector certification".to_string(),
        "Category 1 test (annual) + Category 3 (3-year hydraulic) + Category 5 (5-year full) inspection schedule".to_string(),
        "Building owner responsible for engaging certified inspector + maintaining inspection records on premises".to_string(),
    ];

    let cat1_compliant = input.days_since_category_1_inspection <= 365;
    if !cat1_compliant {
        violations.push(
            "ASME A17.1 Table N1 — Category 1 annual elevator inspection required".to_string(),
        );
    }

    let cat3_compliant =
        !input.elevator_is_hydraulic || input.days_since_category_3_inspection <= 365 * 3;
    if input.elevator_is_hydraulic && !cat3_compliant {
        violations.push(
            "ASME A17.1 Table N1 — Category 3 hydraulic test required every 3 years".to_string(),
        );
    }

    let cat5_compliant = input.days_since_category_5_inspection <= 365 * 5;
    if !cat5_compliant {
        violations
            .push("ASME A17.1 Table N1 — Category 5 full test required every 5 years".to_string());
    }

    let inspector_ok = matches!(
        input.inspector_certification,
        InspectorCertification::QeiCertified
            | InspectorCertification::CalOshaCertified
            | InspectorCertification::NycDobApprovedAgency
    );
    if !inspector_ok {
        violations.push(
            "ASME A17.1 — inspections must be performed by QEI Qualified Elevator Inspector or state-equivalent certified inspector".to_string(),
        );
    }

    RentalElevatorSafetyInspectionResult {
        compliant: violations.is_empty(),
        category_1_annual_compliant: cat1_compliant,
        category_3_hydraulic_compliant: cat3_compliant,
        category_5_five_year_compliant: cat5_compliant,
        inspector_qualified: inspector_ok,
        ca_permit_posted_compliant: true,
        nyc_pvta_filed_compliant: true,
        ca_penalty_in_statutory_range: true,
        nyc_penalty_in_statutory_range: true,
        violations,
        citation: "ASME A17.1-2025 (Safety Code for Elevators and Escalators); ANSI/ASME A17.1/CSA B44-2025; QEI Qualified Elevator Inspector certification",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_clean() -> RentalElevatorSafetyInspectionInput {
        RentalElevatorSafetyInspectionInput {
            regime: Regime::California,
            inspector_certification: InspectorCertification::CalOshaCertified,
            days_since_category_1_inspection: 200,
            days_since_category_3_inspection: 0,
            days_since_category_5_inspection: 0,
            elevator_is_hydraulic: false,
            ca_form_80_posted: true,
            nyc_pvta_filed_within_60_days: true,
            ca_per_day_penalty_cents: 0,
            nyc_per_violation_penalty_cents: 0,
        }
    }

    fn nyc_clean() -> RentalElevatorSafetyInspectionInput {
        let mut i = ca_clean();
        i.regime = Regime::NewYorkCity;
        i.inspector_certification = InspectorCertification::NycDobApprovedAgency;
        i.days_since_category_5_inspection = 200;
        i
    }

    fn default_clean() -> RentalElevatorSafetyInspectionInput {
        let mut i = ca_clean();
        i.regime = Regime::Default;
        i.inspector_certification = InspectorCertification::QeiCertified;
        i.days_since_category_5_inspection = 200;
        i
    }

    #[test]
    fn ca_clean_compliant() {
        let r = check(&ca_clean());
        assert!(r.compliant);
        assert!(r.category_1_annual_compliant);
    }

    #[test]
    fn ca_annual_366_days_violation() {
        let mut i = ca_clean();
        i.days_since_category_1_inspection = 366;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(!r.category_1_annual_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 7320") && v.contains("annual elevator inspection")));
    }

    #[test]
    fn ca_uncertified_inspector_violation() {
        let mut i = ca_clean();
        i.inspector_certification = InspectorCertification::None;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(!r.inspector_qualified);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 7317") && v.contains("Cal/OSHA-certified")));
    }

    #[test]
    fn ca_form_80_not_posted_violation() {
        let mut i = ca_clean();
        i.ca_form_80_posted = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("Form 80")));
    }

    #[test]
    fn ca_200_per_day_penalty_exact_compliant() {
        let mut i = ca_clean();
        i.ca_per_day_penalty_cents = 20_000;
        let r = check(&i);
        assert!(r.ca_penalty_in_statutory_range);
    }

    #[test]
    fn ca_wrong_penalty_amount_violation() {
        let mut i = ca_clean();
        i.ca_per_day_penalty_cents = 30_000;
        let r = check(&i);
        assert!(!r.ca_penalty_in_statutory_range);
    }

    #[test]
    fn nyc_clean_compliant() {
        let r = check(&nyc_clean());
        assert!(r.compliant);
        assert!(r.category_5_five_year_compliant);
    }

    #[test]
    fn nyc_annual_category_1_366_days_violation() {
        let mut i = nyc_clean();
        i.days_since_category_1_inspection = 366;
        let r = check(&i);
        assert!(!r.category_1_annual_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 28-304") && v.contains("Category 1 (PCT)")));
    }

    #[test]
    fn nyc_hydraulic_category_3_3_year_compliant_boundary() {
        let mut i = nyc_clean();
        i.elevator_is_hydraulic = true;
        i.days_since_category_3_inspection = 365 * 3;
        let r = check(&i);
        assert!(r.category_3_hydraulic_compliant);
    }

    #[test]
    fn nyc_hydraulic_category_3_over_3_year_violation() {
        let mut i = nyc_clean();
        i.elevator_is_hydraulic = true;
        i.days_since_category_3_inspection = 365 * 3 + 1;
        let r = check(&i);
        assert!(!r.category_3_hydraulic_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Category 3 hydraulic test") && v.contains("3 years")));
    }

    #[test]
    fn nyc_non_hydraulic_no_category_3_check() {
        let mut i = nyc_clean();
        i.elevator_is_hydraulic = false;
        i.days_since_category_3_inspection = 999_999;
        let r = check(&i);
        assert!(r.category_3_hydraulic_compliant);
    }

    #[test]
    fn nyc_category_5_5_year_compliant_boundary() {
        let mut i = nyc_clean();
        i.days_since_category_5_inspection = 365 * 5;
        let r = check(&i);
        assert!(r.category_5_five_year_compliant);
    }

    #[test]
    fn nyc_category_5_over_5_year_violation() {
        let mut i = nyc_clean();
        i.days_since_category_5_inspection = 365 * 5 + 1;
        let r = check(&i);
        assert!(!r.category_5_five_year_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Category 5 full test") && v.contains("5 years")));
    }

    #[test]
    fn nyc_unapproved_agency_violation() {
        let mut i = nyc_clean();
        i.inspector_certification = InspectorCertification::CalOshaCertified;
        let r = check(&i);
        assert!(!r.inspector_qualified);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 28-304.6.1") && v.contains("DOB-approved")));
    }

    #[test]
    fn nyc_pvta_not_filed_violation() {
        let mut i = nyc_clean();
        i.nyc_pvta_filed_within_60_days = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("PVT-A Form") && v.contains("60 days")));
    }

    #[test]
    fn nyc_penalty_at_minimum_in_range() {
        let mut i = nyc_clean();
        i.nyc_per_violation_penalty_cents = 300_000;
        let r = check(&i);
        assert!(r.nyc_penalty_in_statutory_range);
    }

    #[test]
    fn nyc_penalty_at_maximum_in_range() {
        let mut i = nyc_clean();
        i.nyc_per_violation_penalty_cents = 1_000_000;
        let r = check(&i);
        assert!(r.nyc_penalty_in_statutory_range);
    }

    #[test]
    fn nyc_penalty_below_minimum_violation() {
        let mut i = nyc_clean();
        i.nyc_per_violation_penalty_cents = 299_999;
        let r = check(&i);
        assert!(!r.nyc_penalty_in_statutory_range);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("$3,000") && v.contains("$10,000")));
    }

    #[test]
    fn nyc_penalty_above_maximum_violation() {
        let mut i = nyc_clean();
        i.nyc_per_violation_penalty_cents = 1_000_001;
        let r = check(&i);
        assert!(!r.nyc_penalty_in_statutory_range);
    }

    #[test]
    fn default_clean_compliant() {
        let r = check(&default_clean());
        assert!(r.compliant);
    }

    #[test]
    fn default_qei_certified_inspector_accepted() {
        let mut i = default_clean();
        i.inspector_certification = InspectorCertification::QeiCertified;
        let r = check(&i);
        assert!(r.inspector_qualified);
    }

    #[test]
    fn default_no_certification_violation() {
        let mut i = default_clean();
        i.inspector_certification = InspectorCertification::None;
        let r = check(&i);
        assert!(!r.inspector_qualified);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("QEI") || v.contains("ASME A17.1")));
    }

    #[test]
    fn citation_pins_ca_authority() {
        let r = check(&ca_clean());
        assert!(r.citation.contains("§§ 7300-7324.2"));
        assert!(r.citation.contains("§ 7317"));
        assert!(r.citation.contains("§ 7320"));
        assert!(r.citation.contains("Title 8 Subchapter 6"));
        assert!(r.citation.contains("ASME A17.1"));
    }

    #[test]
    fn citation_pins_nyc_authority() {
        let r = check(&nyc_clean());
        assert!(r.citation.contains("§ 28-304"));
        assert!(r.citation.contains("§ 28-304.6.1"));
        assert!(r.citation.contains("§ 28-304.6.5"));
        assert!(r.citation.contains("Chapter 30"));
        assert!(r.citation.contains("Table N1"));
    }

    #[test]
    fn citation_pins_default_authority() {
        let r = check(&default_clean());
        assert!(r.citation.contains("ASME A17.1-2025"));
        assert!(r.citation.contains("CSA B44-2025"));
        assert!(r.citation.contains("QEI"));
    }

    #[test]
    fn note_pins_ca_200_per_day_penalty() {
        let r = check(&ca_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("$200/day") && n.contains("§ 7320")));
    }

    #[test]
    fn note_pins_ca_4_years_inspector_experience() {
        let r = check(&ca_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("4 years") && n.contains("§ 7317")));
    }

    #[test]
    fn note_pins_nyc_three_test_categories() {
        let r = check(&nyc_clean());
        assert!(r.notes.iter().any(|n| n.contains("Category 1 test (PCT)")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Category 5 test") && n.contains("Category 3 hydraulic")));
    }

    #[test]
    fn note_pins_nyc_pvta_60_day_filing() {
        let r = check(&nyc_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("PVT-A Form") && n.contains("60 days")));
    }

    #[test]
    fn note_pins_nyc_3000_10000_penalty_range() {
        let r = check(&nyc_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("$3,000-$10,000") && n.contains("§ 28-304.6.5")));
    }

    #[test]
    fn note_pins_default_asme_a17_1_2025() {
        let r = check(&default_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("ASME A17.1-2025") && n.contains("Table N1")));
    }

    #[test]
    fn note_pins_default_three_category_schedule() {
        let r = check(&default_clean());
        assert!(r.notes.iter().any(|n| n.contains("Category 1")
            && n.contains("Category 3")
            && n.contains("Category 5")));
    }

    #[test]
    fn category_3_only_engages_for_hydraulic_invariant() {
        for hydraulic in [true, false] {
            let mut i = nyc_clean();
            i.elevator_is_hydraulic = hydraulic;
            i.days_since_category_3_inspection = 365 * 4;
            let r = check(&i);
            if hydraulic {
                assert!(!r.category_3_hydraulic_compliant);
            } else {
                assert!(r.category_3_hydraulic_compliant);
            }
        }
    }

    #[test]
    fn ca_uniquely_requires_form_80_invariant() {
        let mut i_ca = ca_clean();
        i_ca.ca_form_80_posted = false;
        let r_ca = check(&i_ca);
        assert!(!r_ca.compliant);

        let mut i_nyc = nyc_clean();
        i_nyc.ca_form_80_posted = false;
        let r_nyc = check(&i_nyc);
        assert!(r_nyc.compliant);
    }

    #[test]
    fn nyc_uniquely_requires_pvta_60_day_filing_invariant() {
        let mut i_nyc = nyc_clean();
        i_nyc.nyc_pvta_filed_within_60_days = false;
        let r_nyc = check(&i_nyc);
        assert!(!r_nyc.compliant);

        let mut i_default = default_clean();
        i_default.nyc_pvta_filed_within_60_days = false;
        let r_default = check(&i_default);
        assert!(r_default.compliant);
    }

    #[test]
    fn nyc_category_5_5_year_unique_invariant() {
        let mut i_nyc = nyc_clean();
        i_nyc.days_since_category_5_inspection = 365 * 6;
        let r_nyc = check(&i_nyc);
        assert!(!r_nyc.category_5_five_year_compliant);

        let mut i_default = default_clean();
        i_default.days_since_category_5_inspection = 365 * 6;
        let r_default = check(&i_default);
        assert!(!r_default.category_5_five_year_compliant);
    }

    #[test]
    fn inspector_certification_truth_table_for_default() {
        for (cert, exp_qualified) in [
            (InspectorCertification::CalOshaCertified, true),
            (InspectorCertification::NycDobApprovedAgency, true),
            (InspectorCertification::QeiCertified, true),
            (InspectorCertification::None, false),
        ] {
            let mut i = default_clean();
            i.inspector_certification = cert;
            let r = check(&i);
            assert_eq!(
                r.inspector_qualified, exp_qualified,
                "cert={:?} expected qualified={}",
                cert, exp_qualified
            );
        }
    }

    #[test]
    fn multiple_nyc_violations_stack() {
        let mut i = nyc_clean();
        i.days_since_category_1_inspection = 400;
        i.inspector_certification = InspectorCertification::None;
        i.nyc_pvta_filed_within_60_days = false;
        let r = check(&i);
        assert!(r.violations.len() >= 3);
    }
}
