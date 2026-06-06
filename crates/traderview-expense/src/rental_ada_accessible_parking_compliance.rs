//! ADA and FHA accessible-parking compliance framework for multifamily rentals.
//!
//! Multifamily rental properties are subject to two distinct accessible-parking
//! regimes: (1) the federal Americans with Disabilities Act (ADA) Title III for
//! covered places of public accommodation (rental office, leasing center, common
//! amenities) and (2) the Fair Housing Act (FHA) reasonable-modification provision
//! (42 U.S.C. § 3604(f)(3)(A)) for individualized tenant accommodation requests.
//! Many state codes additionally adopt the ANSI A117.1 + IBC 1106 accessible-parking
//! standards through state building codes.
//!
//! Regime grid:
//!
//! - ADA TITLE III + 2010 ADA STANDARDS FOR ACCESSIBLE DESIGN § 502:
//!   - Public accommodation (rental office, leasing center, common areas) MUST
//!     provide accessible parking per § 208.2 minimum-table (1 accessible per 25
//!     spaces for first 100; sliding scale thereafter).
//!   - § 208.2.4 VAN-ACCESSIBLE ratio: at least 1 of every 6 accessible spaces
//!     (rounded up) must be van-accessible.
//!   - § 502.3 access-aisle width: 60 inches minimum for car spaces, 96 inches for
//!     van-accessible spaces (or 60-inch aisle + 132-inch space).
//!   - § 502.7 vertical clearance: 98 inches for van-accessible spaces.
//!   - § 502.6 signage with International Symbol of Accessibility.
//! - FHA REASONABLE ACCOMMODATION (42 U.S.C. § 3604(f)(3)(A) + (B)):
//!   - Even non-ADA-covered multifamily must grant reasonable parking
//!     accommodation requests at landlord expense unless undue burden.
//!   - HUD/DOJ Joint Statement on Reasonable Modifications (March 5, 2008).
//!   - Tenant request, disability nexus, individualized reasonableness analysis.
//! - FHA § 3604(f)(3)(C) ACCESSIBILITY DESIGN+CONSTRUCTION for "covered multifamily
//!   dwellings" first-occupancy after March 13, 1991: 4+ units with elevator (all
//!   units covered) or 4+ units without elevator (all ground-floor units covered) —
//!   2% of parking spaces (minimum 1) accessible.
//! - ANSI A117.1-2009 + IBC § 1106 (state building code adoption): parallel
//!   minimum-table requirements.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - huduser.gov/portal/publications/pdf/fairhousing/fairch2.pdf
//! - northeastada.org/blog/the-fair-housing-act-fha-and-accessible-parking
//! - adatile.com/ada-requirements-for-apartment-buildings/
//! - multifamily.loans/apartment-finance-blog/what-are-the-ada-requirements-for-multifamily-properties/

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegimeCategory {
    /// ADA Title III public-accommodation parking (rental office, leasing center,
    /// common areas).
    AdaTitleIiiPublicAccommodation,
    /// FHA covered-multifamily dwellings designed/constructed after March 13 1991
    /// (4+ units with elevator OR ground-floor units in 4+ without).
    FhaCoveredMultifamily,
    /// FHA reasonable-modification accommodation request by individual tenant.
    FhaReasonableAccommodationRequest,
    /// Non-covered (1-3-unit detached single-family / pre-March-13-1991
    /// construction); only common-law / state-law may apply.
    NotCoveredByAdaOrFha,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    AccessibleSpaceCountMeetsRequirement,
    AccessibleSpaceCountBelowRequirement,
    VanAccessibleRatioMet,
    VanAccessibleRatioNotMet,
    AccessAisleWidthNonCompliant,
    SignageMissingOrIncorrect,
    FhaTenantAccommodationGranted,
    FhaTenantAccommodationDenied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NotCoveredByAdaOrFhaNoComplianceRequirement,
    Compliant2010AdaStandardsSection502,
    InsufficientAccessibleSpaceCountAdaViolation,
    VanAccessibleRatioViolationOnePerSixRule,
    AccessAisleWidthOrSignageViolation,
    FhaCoveredMultifamilyTwoPercentRequirementViolation,
    FhaReasonableAccommodationGranted,
    FhaReasonableAccommodationDeniedSection3604F3aViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub regime_category: RegimeCategory,
    pub compliance_status: ComplianceStatus,
    pub total_parking_spaces: u32,
    pub accessible_spaces_provided: u32,
    pub van_accessible_spaces_provided: u32,
}

pub type RentalAdaAccessibleParkingComplianceInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub minimum_accessible_spaces_required: u32,
    pub minimum_van_accessible_spaces_required: u32,
    pub estimated_landlord_exposure_cents: u64,
    pub note: String,
}

pub type RentalAdaAccessibleParkingComplianceOutput = Output;
pub type RentalAdaAccessibleParkingComplianceResult = Output;

const ADA_FHA_CIVIL_PENALTY_FIRST_VIOLATION_CENTS: u64 = 1_978_700;
const ADA_FHA_TYPICAL_EMOTIONAL_DISTRESS_CENTS: u64 = 1_500_000;
const VAN_ACCESSIBLE_RATIO_DIVISOR: u32 = 6;
const FHA_COVERED_MULTIFAMILY_PERCENT_BPS: u32 = 200;
const FHA_COVERED_MULTIFAMILY_DESIGN_DATE_YEAR: u32 = 1991;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(input.regime_category, RegimeCategory::NotCoveredByAdaOrFha) {
        return Output {
            severity: Severity::NotCoveredByAdaOrFhaNoComplianceRequirement,
            minimum_accessible_spaces_required: 0,
            minimum_van_accessible_spaces_required: 0,
            estimated_landlord_exposure_cents: 0,
            note: format!(
                "Property NOT covered by federal accessible-parking requirements: 1-3-unit \
                 detached single-family OR pre-March-13-{FHA_COVERED_MULTIFAMILY_DESIGN_DATE_YEAR} construction without subsequent \
                 substantial renovation triggering ADA new-construction standards. State and \
                 local building codes (e.g., CA Health & Safety Code § 19955 + § 19956, IL \
                 410 ILCS 25/3 Environmental Barriers Act, NJ Barrier Free Subcode N.J.A.C. \
                 5:23-7) may still impose accessibility requirements. ADA Title III public-\
                 accommodation requirements still apply to rental-office / leasing-center \
                 common areas regardless of dwelling-unit coverage."
            ),
        };
    }

    if matches!(
        input.regime_category,
        RegimeCategory::FhaReasonableAccommodationRequest
    ) {
        match input.compliance_status {
            ComplianceStatus::FhaTenantAccommodationGranted => {
                return Output {
                    severity: Severity::FhaReasonableAccommodationGranted,
                    minimum_accessible_spaces_required: 0,
                    minimum_van_accessible_spaces_required: 0,
                    estimated_landlord_exposure_cents: 0,
                    note: "Compliant: FHA reasonable-accommodation request for accessible \
                           parking granted. 42 U.S.C. § 3604(f)(3)(A) reasonable-modification \
                           and § 3604(f)(3)(B) reasonable-accommodation duties satisfied. \
                           Document the disability-nexus analysis (HUD/DOJ Joint Statement on \
                           Reasonable Accommodations May 17, 2004 + HUD/DOJ Joint Statement on \
                           Reasonable Modifications March 5, 2008) and the reserved-parking \
                           designation (signage, lot configuration, lease addendum)."
                        .to_string(),
                };
            }
            ComplianceStatus::FhaTenantAccommodationDenied => {
                let exposure = ADA_FHA_TYPICAL_EMOTIONAL_DISTRESS_CENTS
                    .saturating_add(ADA_FHA_CIVIL_PENALTY_FIRST_VIOLATION_CENTS);
                return Output {
                    severity: Severity::FhaReasonableAccommodationDeniedSection3604F3aViolation,
                    minimum_accessible_spaces_required: 0,
                    minimum_van_accessible_spaces_required: 0,
                    estimated_landlord_exposure_cents: exposure,
                    note: format!(
                        "FHA § 3604(f)(3)(A) + (B) VIOLATION: outright denial of reasonable \
                         accessible-parking accommodation request. Landlord must engage in \
                         cooperative interactive dialogue, evaluate disability nexus, and \
                         consider less-burdensome alternatives before denial. Estimated \
                         exposure ${} = typical emotional-distress baseline (${}) + 42 U.S.C. \
                         § 3612(g)(3) civil penalty (${}) + attorney fees + injunctive relief \
                         + administrative complaint with HUD or state fair-housing agency.",
                        exposure / 100,
                        ADA_FHA_TYPICAL_EMOTIONAL_DISTRESS_CENTS / 100,
                        ADA_FHA_CIVIL_PENALTY_FIRST_VIOLATION_CENTS / 100
                    ),
                };
            }
            _ => {}
        }
    }

    let required_accessible =
        if matches!(input.regime_category, RegimeCategory::FhaCoveredMultifamily) {
            u32::try_from(
                u128::from(input.total_parking_spaces)
                    .saturating_mul(u128::from(FHA_COVERED_MULTIFAMILY_PERCENT_BPS))
                    .saturating_div(10_000),
            )
            .unwrap_or(u32::MAX)
            .max(1)
        } else {
            ada_title_iii_minimum_table(input.total_parking_spaces)
        };

    let required_van = required_accessible
        .saturating_add(VAN_ACCESSIBLE_RATIO_DIVISOR - 1)
        .saturating_div(VAN_ACCESSIBLE_RATIO_DIVISOR);

    if input.accessible_spaces_provided < required_accessible {
        let exposure = ADA_FHA_TYPICAL_EMOTIONAL_DISTRESS_CENTS
            .saturating_add(ADA_FHA_CIVIL_PENALTY_FIRST_VIOLATION_CENTS);
        let severity = if matches!(input.regime_category, RegimeCategory::FhaCoveredMultifamily) {
            Severity::FhaCoveredMultifamilyTwoPercentRequirementViolation
        } else {
            Severity::InsufficientAccessibleSpaceCountAdaViolation
        };
        return Output {
            severity,
            minimum_accessible_spaces_required: required_accessible,
            minimum_van_accessible_spaces_required: required_van,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "Accessible-space-count VIOLATION. Total parking spaces ({}) require {} \
                 accessible spaces per {} standard but only {} provided. Estimated exposure \
                 ${} = typical emotional-distress baseline + § 3612(g)(3) civil penalty. \
                 Tenant + DOJ remedies under 28 C.F.R. § 36.501 (private action) and \
                 § 36.502 (administrative referral); landlord may also face injunctive order \
                 to retrofit + barrier-removal cost.",
                input.total_parking_spaces,
                required_accessible,
                if matches!(input.regime_category, RegimeCategory::FhaCoveredMultifamily) {
                    "FHA 2% minimum"
                } else {
                    "2010 ADA Standards § 208.2 minimum table"
                },
                input.accessible_spaces_provided,
                exposure / 100
            ),
        };
    }

    if input.van_accessible_spaces_provided < required_van {
        let exposure = ADA_FHA_TYPICAL_EMOTIONAL_DISTRESS_CENTS;
        return Output {
            severity: Severity::VanAccessibleRatioViolationOnePerSixRule,
            minimum_accessible_spaces_required: required_accessible,
            minimum_van_accessible_spaces_required: required_van,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "2010 ADA Standards § 208.2.4 VAN-ACCESSIBLE RATIO VIOLATION. At least 1 of \
                 every 6 accessible spaces (rounded up) must be van-accessible. Required: \
                 {} van-accessible spaces; provided: {}. § 502.3 access-aisle width: 60 \
                 inches minimum for car spaces, 96 inches for van-accessible (or 60-inch \
                 aisle + 132-inch space). § 502.7 vertical clearance: 98 inches for van-\
                 accessible. Estimated exposure ${} excludes retrofit cost.",
                required_van,
                input.van_accessible_spaces_provided,
                exposure / 100
            ),
        };
    }

    if matches!(
        input.compliance_status,
        ComplianceStatus::AccessAisleWidthNonCompliant
            | ComplianceStatus::SignageMissingOrIncorrect
    ) {
        return Output {
            severity: Severity::AccessAisleWidthOrSignageViolation,
            minimum_accessible_spaces_required: required_accessible,
            minimum_van_accessible_spaces_required: required_van,
            estimated_landlord_exposure_cents: ADA_FHA_TYPICAL_EMOTIONAL_DISTRESS_CENTS,
            note: "2010 ADA Standards § 502.3 (access aisle width) or § 502.6 (signage with \
                   International Symbol of Accessibility) violation. Curable by repainting \
                   aisle stripes to required width + installing proper signage. Document \
                   remediation completion with photographs."
                .to_string(),
        };
    }

    Output {
        severity: Severity::Compliant2010AdaStandardsSection502,
        minimum_accessible_spaces_required: required_accessible,
        minimum_van_accessible_spaces_required: required_van,
        estimated_landlord_exposure_cents: 0,
        note: format!(
            "Compliant: 2010 ADA Standards § 502 requirements satisfied. Total parking ({}) \
             + accessible spaces ({}) ≥ required ({}) + van-accessible ({}) ≥ required ({}). \
             § 502.3 access aisle width + § 502.6 signage with International Symbol of \
             Accessibility + § 502.7 98-inch vertical clearance for van spaces also \
             confirmed. Retain construction drawings, post-construction survey, and any \
             ADA-compliance certification.",
            input.total_parking_spaces,
            input.accessible_spaces_provided,
            required_accessible,
            input.van_accessible_spaces_provided,
            required_van
        ),
    }
}

fn ada_title_iii_minimum_table(total_spaces: u32) -> u32 {
    match total_spaces {
        0 => 0,
        1..=25 => 1,
        26..=50 => 2,
        51..=75 => 3,
        76..=100 => 4,
        101..=150 => 5,
        151..=200 => 6,
        201..=300 => 7,
        301..=400 => 8,
        401..=500 => 9,
        501..=1000 => total_spaces.saturating_div(50),
        _ => 20u32.saturating_add(total_spaces.saturating_sub(1000).saturating_div(100)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_ada() -> Input {
        Input {
            regime_category: RegimeCategory::AdaTitleIiiPublicAccommodation,
            compliance_status: ComplianceStatus::AccessibleSpaceCountMeetsRequirement,
            total_parking_spaces: 100,
            accessible_spaces_provided: 4,
            van_accessible_spaces_provided: 1,
        }
    }

    #[test]
    fn not_covered_no_compliance_required() {
        let mut input = base_ada();
        input.regime_category = RegimeCategory::NotCoveredByAdaOrFha;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NotCoveredByAdaOrFhaNoComplianceRequirement
        );
        assert!(output.note.contains("March-13-1991"));
        assert!(output.note.contains("§ 19955"));
        assert!(output.note.contains("Environmental Barriers"));
    }

    #[test]
    fn ada_100_space_lot_4_accessible_compliant() {
        let input = base_ada();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Compliant2010AdaStandardsSection502
        );
        assert_eq!(output.minimum_accessible_spaces_required, 4);
        // 4 / 6 ceiling = 1 van required
        assert_eq!(output.minimum_van_accessible_spaces_required, 1);
    }

    #[test]
    fn ada_25_space_lot_1_accessible_required() {
        let mut input = base_ada();
        input.total_parking_spaces = 25;
        input.accessible_spaces_provided = 1;
        let output = check(&input);
        assert_eq!(output.minimum_accessible_spaces_required, 1);
        assert_eq!(
            output.severity,
            Severity::Compliant2010AdaStandardsSection502
        );
    }

    #[test]
    fn ada_50_space_lot_2_accessible_required() {
        let mut input = base_ada();
        input.total_parking_spaces = 50;
        input.accessible_spaces_provided = 2;
        let output = check(&input);
        assert_eq!(output.minimum_accessible_spaces_required, 2);
        assert_eq!(
            output.severity,
            Severity::Compliant2010AdaStandardsSection502
        );
    }

    #[test]
    fn ada_200_space_lot_6_accessible_required() {
        let mut input = base_ada();
        input.total_parking_spaces = 200;
        input.accessible_spaces_provided = 6;
        let output = check(&input);
        assert_eq!(output.minimum_accessible_spaces_required, 6);
    }

    #[test]
    fn ada_500_space_lot_9_accessible_required() {
        let mut input = base_ada();
        input.total_parking_spaces = 500;
        input.accessible_spaces_provided = 9;
        let output = check(&input);
        assert_eq!(output.minimum_accessible_spaces_required, 9);
    }

    #[test]
    fn ada_insufficient_accessible_space_count_violation() {
        let mut input = base_ada();
        input.accessible_spaces_provided = 2;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::InsufficientAccessibleSpaceCountAdaViolation
        );
        assert!(output.note.contains("28 C.F.R. § 36.501"));
    }

    #[test]
    fn ada_van_accessible_ratio_violation() {
        let mut input = base_ada();
        input.van_accessible_spaces_provided = 0;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::VanAccessibleRatioViolationOnePerSixRule
        );
        assert!(output.note.contains("§ 208.2.4"));
        assert!(output.note.contains("§ 502.7"));
    }

    #[test]
    fn ada_aisle_width_violation_signage_violation() {
        let mut input = base_ada();
        input.compliance_status = ComplianceStatus::AccessAisleWidthNonCompliant;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::AccessAisleWidthOrSignageViolation
        );
        assert!(output.note.contains("§ 502.3"));
        assert!(output.note.contains("§ 502.6"));
    }

    #[test]
    fn fha_covered_multifamily_100_spaces_requires_2_accessible() {
        let mut input = base_ada();
        input.regime_category = RegimeCategory::FhaCoveredMultifamily;
        input.total_parking_spaces = 100;
        input.accessible_spaces_provided = 2;
        let output = check(&input);
        // 2% × 100 = 2 required → compliant
        assert_eq!(output.minimum_accessible_spaces_required, 2);
        // Status set to AccessibleSpaceCountMeetsRequirement → compliant
        assert_eq!(
            output.severity,
            Severity::Compliant2010AdaStandardsSection502
        );
    }

    #[test]
    fn fha_covered_multifamily_below_2_pct_violation() {
        let mut input = base_ada();
        input.regime_category = RegimeCategory::FhaCoveredMultifamily;
        input.total_parking_spaces = 100;
        input.accessible_spaces_provided = 1;
        let output = check(&input);
        // 2% × 100 = 2 required; provided 1 → violation
        assert_eq!(
            output.severity,
            Severity::FhaCoveredMultifamilyTwoPercentRequirementViolation
        );
        assert!(output.note.contains("FHA 2% minimum"));
    }

    #[test]
    fn fha_covered_multifamily_minimum_one_accessible_floor() {
        let mut input = base_ada();
        input.regime_category = RegimeCategory::FhaCoveredMultifamily;
        input.total_parking_spaces = 10;
        input.accessible_spaces_provided = 1;
        let output = check(&input);
        // 2% × 10 = 0 → floor to 1
        assert_eq!(output.minimum_accessible_spaces_required, 1);
    }

    #[test]
    fn fha_reasonable_accommodation_granted_compliant() {
        let mut input = base_ada();
        input.regime_category = RegimeCategory::FhaReasonableAccommodationRequest;
        input.compliance_status = ComplianceStatus::FhaTenantAccommodationGranted;
        let output = check(&input);
        assert_eq!(output.severity, Severity::FhaReasonableAccommodationGranted);
        assert_eq!(output.estimated_landlord_exposure_cents, 0);
        assert!(output.note.contains("§ 3604(f)(3)(A)"));
        assert!(output.note.contains("§ 3604(f)(3)(B)"));
    }

    #[test]
    fn fha_reasonable_accommodation_denied_violation() {
        let mut input = base_ada();
        input.regime_category = RegimeCategory::FhaReasonableAccommodationRequest;
        input.compliance_status = ComplianceStatus::FhaTenantAccommodationDenied;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::FhaReasonableAccommodationDeniedSection3604F3aViolation
        );
        // $15,000 + $19,787 = $34,787
        assert_eq!(output.estimated_landlord_exposure_cents, 3_478_700);
        assert!(output.note.contains("§ 3612(g)(3)"));
    }

    #[test]
    fn van_accessible_ratio_constant_pins_one_in_six() {
        assert_eq!(VAN_ACCESSIBLE_RATIO_DIVISOR, 6);
    }

    #[test]
    fn fha_covered_multifamily_percent_constant_pins_2_pct() {
        assert_eq!(FHA_COVERED_MULTIFAMILY_PERCENT_BPS, 200);
    }

    #[test]
    fn fha_covered_multifamily_design_date_constant_pins_1991() {
        assert_eq!(FHA_COVERED_MULTIFAMILY_DESIGN_DATE_YEAR, 1991);
    }

    #[test]
    fn ada_civil_penalty_constant_pins_19787() {
        assert_eq!(ADA_FHA_CIVIL_PENALTY_FIRST_VIOLATION_CENTS, 1_978_700);
    }

    #[test]
    fn ada_emotional_distress_constant_pins_15000() {
        assert_eq!(ADA_FHA_TYPICAL_EMOTIONAL_DISTRESS_CENTS, 1_500_000);
    }

    #[test]
    fn very_large_lot_size_no_overflow() {
        let mut input = base_ada();
        input.total_parking_spaces = 10_000;
        input.accessible_spaces_provided = 200;
        let output = check(&input);
        // 20 + (10000 - 1000) / 100 = 20 + 90 = 110
        assert_eq!(output.minimum_accessible_spaces_required, 110);
    }

    #[test]
    fn zero_spaces_zero_required() {
        let mut input = base_ada();
        input.total_parking_spaces = 0;
        let output = check(&input);
        assert_eq!(output.minimum_accessible_spaces_required, 0);
    }
}
