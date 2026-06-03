//! Rent control + rent stabilization compliance framework for residential rentals.
//!
//! Rent control and rent stabilization laws cap the annual rent increases landlords
//! may impose on existing tenants. The federal Fair Housing Act and most federal
//! statutes do NOT regulate rent levels; rent control is a state and municipal
//! framework. Two states (California + Oregon) enacted the first statewide rent caps
//! in 2019; New York, New Jersey, Washington DC, and several Massachusetts cities
//! operate strong rent-stabilization frameworks. Many additional states permit
//! municipalities to enact local rent control (e.g., MN, ME, NJ).
//!
//! Jurisdictional grid (effective 2026):
//!
//! - CA AB 1482 (Cal. Civ. Code § 1947.12 + § 1946.2) Tenant Protection Act of 2019
//!   (effective Jan 1, 2020 - Jan 1, 2030): annual rent cap = LESSER OF 5% + local
//!   CPI OR 10% per 12-month period. Applies to units 15+ years old (rolling
//!   exception). § 1946.2 just-cause eviction protection requires lawful basis
//!   after 12+ months of tenancy. Single-family homes not held by REIT/corporate
//!   landlord are exempt with notice. Local rent-control laws (LA, SF, Berkeley,
//!   Oakland, Santa Monica) take precedence where they are MORE protective.
//! - OR SB 608 (ORS 90.323 + ORS 90.427): first statewide US rent cap (2019);
//!   annual rent cap = 7% + CPI (capped at 10% under SB 611 effective Jul 6,
//!   2023). Just-cause eviction after first year. New construction exempt for
//!   15 years.
//! - NY State HSTPA 2019 (Emergency Tenant Protection Act expansion): rent-
//!   stabilization expanded statewide for buildings of 6+ units built before 1974
//!   in localities adopting ETPA. NYC RSL § 26-501 et seq. applies to NYC
//!   pre-1974 buildings; Rent Guidelines Board sets annual % cap each June.
//!   Vacancy decontrol eliminated; preferential-rent / IAI / MCI caps tightened.
//! - NJ Anti-Eviction Act N.J.S.A. 2A:18-61.1 + municipal rent-control ordinances
//!   in 100+ NJ municipalities (Newark, Jersey City, Hoboken, etc.). State Anti-
//!   Eviction Act requires just cause; local ordinances cap annual increases at
//!   varying percentages (typically 4-6%).
//! - DC Rental Housing Act of 1985 (D.C. Code § 42-3501 et seq.): rent-controlled
//!   buildings (pre-1976 + 5+ units) capped at CPI + 2% per year (10% maximum;
//!   5% for elderly/disabled tenants). DC RAD program registration required.
//! - MN St. Paul Charter Amendment (Nov 2021): 3% annual rent cap on all
//!   residential rentals; eligibility-based exemptions and adjustment procedures.
//! - DEFAULT: most states (TX, FL, TN, AZ, MI, IL outside Chicago) PREEMPT
//!   local rent control via state preemption statutes; only common-law unconscion-
//!   ability + state UDAP fraud-against-tenant frameworks restrict rent levels.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - leginfo.legislature.ca.gov/faces/billTextClient.xhtml?bill_id=201920200AB1482
//! - caanet.org/topics/ab-1482/
//! - nationaltenantauthority.com/rent-control-laws

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    CaliforniaAb1482Statewide,
    OregonSb608Statewide,
    NewYorkRentStabilizedNyc,
    NewJerseyAntiEvictionAct,
    DcRentalHousingAct,
    MinnesotaStPaulCharter,
    StatePreemptedNoRentControl,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyExemptionStatus {
    /// Subject to rent cap (not exempt).
    NotExemptSubjectToRentCap,
    /// CA AB 1482 single-family-home-by-non-corporate-owner exemption (notice
    /// required at lease signing).
    CaSingleFamilyNonCorporateExemptWithNotice,
    /// New construction exemption (15-year exemption for OR SB 608; 15-year
    /// rolling for CA AB 1482).
    NewConstructionWithinExemptionWindow,
    /// Building too small (4 or fewer units in NYC, etc.).
    BuildingTooSmallForCoverage,
    /// Owner-occupied small property (e.g., 1-3 family with owner residing).
    OwnerOccupiedSmallProperty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NoRentControlInJurisdiction,
    PropertyExemptFromRentCap,
    CompliantRentIncreaseWithinStatutoryCap,
    RentIncreaseExceedsStatutoryCapViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub property_exemption_status: PropertyExemptionStatus,
    pub current_monthly_rent_cents: u64,
    pub proposed_monthly_rent_cents: u64,
    pub local_cpi_bps: u32,
}

pub type RentalRentControlStabilizationInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub statutory_cap_bps: u32,
    pub max_allowed_monthly_rent_cents: u64,
    pub excess_over_cap_cents: u64,
    pub note: String,
}

pub type RentalRentControlStabilizationOutput = Output;
pub type RentalRentControlStabilizationResult = Output;

const CA_AB1482_FIXED_INCREMENT_BPS: u32 = 500;
const CA_AB1482_HARD_CAP_BPS: u32 = 1_000;
const OR_SB608_FIXED_INCREMENT_BPS: u32 = 700;
const OR_SB608_HARD_CAP_BPS: u32 = 1_000;
const MN_ST_PAUL_FIXED_CAP_BPS: u32 = 300;
const DC_FIXED_INCREMENT_BPS: u32 = 200;
const DC_HARD_CAP_BPS: u32 = 1_000;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.jurisdiction,
        Jurisdiction::StatePreemptedNoRentControl | Jurisdiction::Default
    ) {
        return Output {
            severity: Severity::NoRentControlInJurisdiction,
            statutory_cap_bps: 0,
            max_allowed_monthly_rent_cents: input.proposed_monthly_rent_cents,
            excess_over_cap_cents: 0,
            note: "Jurisdiction does NOT impose rent control or rent stabilization. \
                   Many states (TX, FL, TN, AZ, MI, IL outside Chicago) PREEMPT local rent \
                   control via state statute. Landlord may set market rent subject to lease \
                   terms + common-law unconscionability + state UDAP fraud-against-tenant \
                   frameworks. Confirm jurisdiction's preemption status with local counsel \
                   before relying."
                .to_string(),
        };
    }

    if !matches!(
        input.property_exemption_status,
        PropertyExemptionStatus::NotExemptSubjectToRentCap
    ) {
        return Output {
            severity: Severity::PropertyExemptFromRentCap,
            statutory_cap_bps: 0,
            max_allowed_monthly_rent_cents: input.proposed_monthly_rent_cents,
            excess_over_cap_cents: 0,
            note: format!(
                "Property EXEMPT from rent cap: {}. CA AB 1482 single-family-home exemption \
                 requires written notice at lease signing per Cal. Civ. Code § 1947.12(d)(5) \
                 in specified statutory language. New-construction exemption windows: 15 \
                 years rolling for CA AB 1482; 15 years from certificate-of-occupancy for OR \
                 SB 608.",
                exemption_label(input.property_exemption_status)
            ),
        };
    }

    let cap_bps = jurisdiction_cap_bps(input.jurisdiction, input.local_cpi_bps);
    let max_allowed_increment_cents = u64::try_from(
        u128::from(input.current_monthly_rent_cents)
            .saturating_mul(u128::from(cap_bps))
            .saturating_div(10_000),
    )
    .unwrap_or(u64::MAX);
    let max_allowed_rent = input
        .current_monthly_rent_cents
        .saturating_add(max_allowed_increment_cents);

    if input.proposed_monthly_rent_cents <= max_allowed_rent {
        return Output {
            severity: Severity::CompliantRentIncreaseWithinStatutoryCap,
            statutory_cap_bps: cap_bps,
            max_allowed_monthly_rent_cents: max_allowed_rent,
            excess_over_cap_cents: 0,
            note: format!(
                "Compliant: proposed rent (${}) within {}-bps statutory cap. {} Max allowed \
                 monthly rent ${} = current ${} + ${} increment ({}-bp cap on current rent).",
                input.proposed_monthly_rent_cents / 100,
                cap_bps,
                statute_citation(input.jurisdiction),
                max_allowed_rent / 100,
                input.current_monthly_rent_cents / 100,
                max_allowed_increment_cents / 100,
                cap_bps
            ),
        };
    }

    let excess = input
        .proposed_monthly_rent_cents
        .saturating_sub(max_allowed_rent);
    Output {
        severity: Severity::RentIncreaseExceedsStatutoryCapViolation,
        statutory_cap_bps: cap_bps,
        max_allowed_monthly_rent_cents: max_allowed_rent,
        excess_over_cap_cents: excess,
        note: format!(
            "Rent increase VIOLATION: proposed rent (${}) exceeds statutory cap. {} Max \
             allowed monthly rent ${} (current ${} + {}-bp cap). Excess (${}) creates: (a) \
             tenant defense to rent payment + rent-arrears reduction in summary-process \
             eviction proceeding; (b) tenant claim for excess rent paid + statutory damages \
             where applicable; (c) state Department of Housing or local Rent Board \
             complaint; (d) injunctive relief reducing rent to cap-compliant level.",
            input.proposed_monthly_rent_cents / 100,
            statute_citation(input.jurisdiction),
            max_allowed_rent / 100,
            input.current_monthly_rent_cents / 100,
            cap_bps,
            excess / 100
        ),
    }
}

fn jurisdiction_cap_bps(jurisdiction: Jurisdiction, cpi_bps: u32) -> u32 {
    match jurisdiction {
        Jurisdiction::CaliforniaAb1482Statewide => {
            CA_AB1482_FIXED_INCREMENT_BPS
                .saturating_add(cpi_bps)
                .min(CA_AB1482_HARD_CAP_BPS)
        }
        Jurisdiction::OregonSb608Statewide => OR_SB608_FIXED_INCREMENT_BPS
            .saturating_add(cpi_bps)
            .min(OR_SB608_HARD_CAP_BPS),
        Jurisdiction::NewYorkRentStabilizedNyc => cpi_bps,
        Jurisdiction::NewJerseyAntiEvictionAct => cpi_bps,
        Jurisdiction::DcRentalHousingAct => DC_FIXED_INCREMENT_BPS
            .saturating_add(cpi_bps)
            .min(DC_HARD_CAP_BPS),
        Jurisdiction::MinnesotaStPaulCharter => MN_ST_PAUL_FIXED_CAP_BPS,
        _ => 0,
    }
}

fn exemption_label(status: PropertyExemptionStatus) -> &'static str {
    match status {
        PropertyExemptionStatus::CaSingleFamilyNonCorporateExemptWithNotice => {
            "CA single-family home owned by non-corporate landlord with statutory notice"
        }
        PropertyExemptionStatus::NewConstructionWithinExemptionWindow => {
            "new construction within statutory exemption window"
        }
        PropertyExemptionStatus::BuildingTooSmallForCoverage => {
            "building below minimum-unit threshold (e.g., 4 or fewer units NYC RSL)"
        }
        PropertyExemptionStatus::OwnerOccupiedSmallProperty => {
            "owner-occupied small property"
        }
        PropertyExemptionStatus::NotExemptSubjectToRentCap => "not exempt",
    }
}

fn statute_citation(jurisdiction: Jurisdiction) -> &'static str {
    match jurisdiction {
        Jurisdiction::CaliforniaAb1482Statewide => {
            "CA AB 1482 (Cal. Civ. Code § 1947.12 + § 1946.2) Tenant Protection Act of 2019; \
             cap = LESSER OF 5% + CPI or 10%; effective Jan 1, 2020 - Jan 1, 2030."
        }
        Jurisdiction::OregonSb608Statewide => {
            "OR SB 608 (ORS 90.323 + ORS 90.427); 7% + CPI capped at 10% under SB 611 \
             effective Jul 6, 2023."
        }
        Jurisdiction::NewYorkRentStabilizedNyc => {
            "NY State HSTPA 2019 + NYC RSL § 26-501; Rent Guidelines Board annual % cap."
        }
        Jurisdiction::NewJerseyAntiEvictionAct => {
            "NJ Anti-Eviction Act N.J.S.A. 2A:18-61.1 + municipal rent-control ordinances \
             (Newark, Jersey City, Hoboken, etc., typically 4-6% cap)."
        }
        Jurisdiction::DcRentalHousingAct => {
            "DC Rental Housing Act of 1985 D.C. Code § 42-3501 et seq.; pre-1976 + 5+ \
             units; CPI + 2% capped at 10% (5% elderly/disabled)."
        }
        Jurisdiction::MinnesotaStPaulCharter => {
            "MN St. Paul Charter Amendment (Nov 2021); 3% annual cap on all residential \
             rentals subject to eligibility-based exemptions."
        }
        _ => "No rent control in jurisdiction.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_ca() -> Input {
        Input {
            jurisdiction: Jurisdiction::CaliforniaAb1482Statewide,
            property_exemption_status:
                PropertyExemptionStatus::NotExemptSubjectToRentCap,
            current_monthly_rent_cents: 2_000_00,
            proposed_monthly_rent_cents: 2_140_00,
            local_cpi_bps: 200,
        }
    }

    #[test]
    fn no_rent_control_jurisdiction_no_restriction() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::StatePreemptedNoRentControl;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NoRentControlInJurisdiction);
    }

    #[test]
    fn default_jurisdiction_no_rent_control() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Default;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NoRentControlInJurisdiction);
    }

    #[test]
    fn ca_single_family_exemption_no_cap() {
        let mut input = base_ca();
        input.property_exemption_status =
            PropertyExemptionStatus::CaSingleFamilyNonCorporateExemptWithNotice;
        let output = check(&input);
        assert_eq!(output.severity, Severity::PropertyExemptFromRentCap);
        assert!(output.note.contains("§ 1947.12(d)(5)"));
    }

    #[test]
    fn new_construction_exemption_no_cap() {
        let mut input = base_ca();
        input.property_exemption_status =
            PropertyExemptionStatus::NewConstructionWithinExemptionWindow;
        let output = check(&input);
        assert_eq!(output.severity, Severity::PropertyExemptFromRentCap);
        assert!(output.note.contains("15 years"));
    }

    #[test]
    fn ca_ab1482_compliant_5_pct_plus_2_pct_cpi() {
        let input = base_ca();
        let output = check(&input);
        // 5% + 2% = 7%; cap = min(7%, 10%) = 7%
        // $2000 + 7% = $2,140; proposed $2,140 → compliant
        assert_eq!(output.statutory_cap_bps, 700);
        assert_eq!(output.max_allowed_monthly_rent_cents, 2_140_00);
        assert_eq!(
            output.severity,
            Severity::CompliantRentIncreaseWithinStatutoryCap
        );
    }

    #[test]
    fn ca_ab1482_excess_over_cap_violation() {
        let mut input = base_ca();
        input.proposed_monthly_rent_cents = 2_300_00;
        let output = check(&input);
        // Max $2,140; proposed $2,300 → excess $160
        assert_eq!(
            output.severity,
            Severity::RentIncreaseExceedsStatutoryCapViolation
        );
        assert_eq!(output.excess_over_cap_cents, 160_00);
        assert!(output.note.contains("Rent Board"));
    }

    #[test]
    fn ca_ab1482_hard_cap_at_10_pct_even_with_high_cpi() {
        let mut input = base_ca();
        input.local_cpi_bps = 800; // 8% CPI
        // 5% + 8% = 13%; cap = min(13%, 10%) = 10%
        let output = check(&input);
        assert_eq!(output.statutory_cap_bps, 1_000);
    }

    #[test]
    fn oregon_sb608_7_pct_plus_cpi_capped_at_10() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::OregonSb608Statewide;
        input.local_cpi_bps = 500; // 5% CPI
        // 7% + 5% = 12%; cap = min(12%, 10%) = 10%
        let output = check(&input);
        assert_eq!(output.statutory_cap_bps, 1_000);
    }

    #[test]
    fn oregon_sb608_below_hard_cap() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::OregonSb608Statewide;
        input.local_cpi_bps = 100;
        // 7% + 1% = 8%; under 10% hard cap
        let output = check(&input);
        assert_eq!(output.statutory_cap_bps, 800);
    }

    #[test]
    fn nyc_rent_stabilized_uses_cpi_only() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewYorkRentStabilizedNyc;
        input.local_cpi_bps = 300;
        let output = check(&input);
        assert_eq!(output.statutory_cap_bps, 300);
    }

    #[test]
    fn dc_rental_housing_act_2_pct_plus_cpi_capped_at_10() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::DcRentalHousingAct;
        input.local_cpi_bps = 300;
        // 2% + 3% = 5%; under 10% hard cap
        let output = check(&input);
        assert_eq!(output.statutory_cap_bps, 500);
    }

    #[test]
    fn dc_hard_cap_at_10_pct_with_high_cpi() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::DcRentalHousingAct;
        input.local_cpi_bps = 900;
        // 2% + 9% = 11%; cap = 10%
        let output = check(&input);
        assert_eq!(output.statutory_cap_bps, 1_000);
    }

    #[test]
    fn mn_st_paul_fixed_3_pct_cap() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::MinnesotaStPaulCharter;
        input.local_cpi_bps = 500; // Irrelevant; cap is fixed
        let output = check(&input);
        assert_eq!(output.statutory_cap_bps, 300);
    }

    #[test]
    fn nj_anti_eviction_uses_cpi() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewJerseyAntiEvictionAct;
        input.local_cpi_bps = 400;
        let output = check(&input);
        assert_eq!(output.statutory_cap_bps, 400);
    }

    #[test]
    fn ca_ab1482_fixed_increment_constant_pins_5_pct() {
        assert_eq!(CA_AB1482_FIXED_INCREMENT_BPS, 500);
    }

    #[test]
    fn ca_ab1482_hard_cap_constant_pins_10_pct() {
        assert_eq!(CA_AB1482_HARD_CAP_BPS, 1_000);
    }

    #[test]
    fn or_sb608_fixed_increment_constant_pins_7_pct() {
        assert_eq!(OR_SB608_FIXED_INCREMENT_BPS, 700);
    }

    #[test]
    fn mn_st_paul_fixed_cap_constant_pins_3_pct() {
        assert_eq!(MN_ST_PAUL_FIXED_CAP_BPS, 300);
    }

    #[test]
    fn dc_fixed_increment_constant_pins_2_pct() {
        assert_eq!(DC_FIXED_INCREMENT_BPS, 200);
    }

    #[test]
    fn very_large_rent_no_overflow_in_percent_calc() {
        let mut input = base_ca();
        input.current_monthly_rent_cents = u64::MAX / 2;
        input.proposed_monthly_rent_cents = u64::MAX / 2;
        let output = check(&input);
        // saturating arithmetic prevents overflow
        assert!(output.max_allowed_monthly_rent_cents > 0);
    }

    #[test]
    fn zero_current_rent_zero_increment_floor() {
        let mut input = base_ca();
        input.current_monthly_rent_cents = 0;
        input.proposed_monthly_rent_cents = 100;
        let output = check(&input);
        // 0% × 7% = 0 increment; max = 0; proposed 1 → violation
        assert_eq!(
            output.severity,
            Severity::RentIncreaseExceedsStatutoryCapViolation
        );
    }

    #[test]
    fn note_pins_ca_ab1482_tenant_protection_act() {
        let input = base_ca();
        let output = check(&input);
        assert!(output.note.contains("AB 1482"));
        assert!(output.note.contains("§ 1947.12"));
        assert!(output.note.contains("Tenant Protection Act"));
    }

    #[test]
    fn note_pins_or_sb608_statewide_first() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::OregonSb608Statewide;
        let output = check(&input);
        assert!(output.note.contains("SB 608"));
        assert!(output.note.contains("SB 611"));
    }

    #[test]
    fn note_pins_nyc_rsl_26_501() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewYorkRentStabilizedNyc;
        let output = check(&input);
        assert!(output.note.contains("§ 26-501"));
    }

    #[test]
    fn note_pins_nj_anti_eviction_2a_18_61_1() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewJerseyAntiEvictionAct;
        let output = check(&input);
        assert!(output.note.contains("2A:18-61.1"));
    }
}
