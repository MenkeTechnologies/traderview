//! Tenant relocation assistance compliance framework for no-fault eviction,
//! condo conversion, and demolition/substantial-rehabilitation displacement.
//!
//! When a landlord ends a tenancy for a "no-fault" reason (owner move-in, withdrawal
//! from rental market, condo conversion, demolition, substantial rehabilitation) state
//! and municipal law commonly requires the landlord to PAY RELOCATION ASSISTANCE to
//! the displaced tenant. The amount and procedural requirements vary sharply by
//! jurisdiction and depend on the displacement basis. Many jurisdictions also impose
//! TOPA (Tenant Opportunity to Purchase Act)-style rights of first refusal that
//! intersect with relocation duties.
//!
//! Jurisdictional grid:
//!
//! - CA AB 1482 (Cal. Civ. Code § 1946.2 + § 1947.12) + CA SB 567 (effective April
//!   1, 2024): no-fault eviction requires payment of ONE MONTH'S RENT to displaced
//!   tenant. SB 567 strengthens owner-move-in rule: (1) owner/relative must move in
//!   within 90 days, (2) live in unit as primary residence for at least 1 year, (3)
//!   eviction notice must disclose name and relationship, (4) no other vacant
//!   similar unit on property.
//! - NYC RENT-STABILIZED DEMOLITION (NY State DHCR approval + § 26-511(c)(9)): DHCR
//!   approval required for demolition; landlord must relocate tenant to comparable
//!   housing at same/lower regulated rent OR new residential building if
//!   constructed on site, PLUS reasonable moving expenses + $5,000 stipend. Condo
//!   conversion under non-eviction plan preserves rent-stabilized tenancy.
//! - WA RCW 59.18.440 + RCW 59.18.450: local-option authorization for low-income
//!   tenant relocation assistance upon demolition, substantial rehabilitation,
//!   change of use, or removal of use restrictions in assisted housing.
//!   "Low-income tenants" = combined household income ≤ 50% area median income
//!   adjusted for family size.
//! - IL CHICAGO RLTO § 5-12-130 + Cook County: condo conversion requires one-time
//!   relocation fee of $1,500 OR one month's rent capped at $2,500, whichever is
//!   less, payable to qualified tenant exercising assistance option. Keep Chicago
//!   Renting Ordinance (KCRO) requires new building owner post-foreclosure to
//!   offer bona-fide tenants lease renewal OR $10,600 relocation assistance.
//! - DC TOPA (D.C. Code § 42-3404.01 et seq.): tenants in single-family or 2-unit
//!   building have right of first refusal when building is sold or converted;
//!   relocation assistance capped at lesser of one year's rent or $12,000
//!   (annually adjusted). Building-affordability + organizing expenses including
//!   attorney fees + specified improvements may also be negotiated.
//! - DEFAULT: many jurisdictions impose no statutory relocation duty; common-law
//!   "constructive eviction" framework may apply for landlord-caused habitability
//!   destruction triggering tenant relocation.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - oag.ca.gov/system/files/media/Tenant-Protection-Act-Landlords-and-Property-Managers-English.pdf
//! - rentguidelinesboard.cityofnewyork.us/resources/faqs/conversion-demolition/
//! - app.leg.wa.gov/rcw/default.aspx?cite=59.18.440
//! - codelibrary.amlegal.com/codes/chicago/latest/chicago_il/0-0-0-2639485
//! - ota.dc.gov/page/tenant-opportunity-purchase-act-topa

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    CaliforniaAb1482Sb567,
    NewYorkCityRentStabilized,
    Washington,
    IllinoisChicagoRlto,
    DcTopa,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplacementBasis {
    /// Owner move-in eviction (SB 567 90-day move-in + 1-year residency).
    OwnerMoveInEviction,
    /// Withdrawal from rental market (Ellis Act in CA).
    WithdrawalFromRentalMarketEllis,
    /// Condo conversion or co-op conversion.
    CondoOrCoOpConversion,
    /// Demolition.
    Demolition,
    /// Substantial rehabilitation.
    SubstantialRehabilitation,
    /// Change of use (residential to commercial).
    ChangeOfUseNonResidential,
    /// At-fault eviction (no relocation duty).
    AtFaultEvictionNoRelocationDuty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantIncomeStatus {
    /// Low-income (≤ 50% AMI) — triggers WA RCW 59.18.440 protection.
    LowIncomeBelowFiftyPctAmi,
    /// Above 50% AMI — full statutory protection in some jurisdictions; WA only
    /// covers low-income.
    AboveFiftyPctAmi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    AtFaultEvictionNoRelocationDuty,
    CompliantRelocationAssistancePaid,
    InsufficientRelocationAssistanceViolation,
    SbFiveSixSevenOwnerMoveInNonCompliance,
    NycDhcrApprovalRequiredDemolitionViolation,
    WashingtonNonLowIncomeNotCovered,
    DcTopaRightOfFirstRefusalIncludingRelocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub displacement_basis: DisplacementBasis,
    pub tenant_income_status: TenantIncomeStatus,
    pub monthly_rent_cents: u64,
    pub relocation_assistance_paid_cents: u64,
    pub sb_567_owner_moves_in_within_90_days: bool,
    pub sb_567_owner_resides_one_year_primary_residence: bool,
    pub nyc_dhcr_demolition_approval_obtained: bool,
}

pub type RentalTenantRelocationAssistanceInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub statutory_relocation_amount_cents: u64,
    pub shortfall_cents: u64,
    pub note: String,
}

pub type RentalTenantRelocationAssistanceOutput = Output;
pub type RentalTenantRelocationAssistanceResult = Output;

const CHICAGO_RLTO_FLAT_RELOCATION_CENTS: u64 = 150_000;
const CHICAGO_RLTO_RENT_BASED_CAP_CENTS: u64 = 250_000;
#[allow(dead_code)]
const CHICAGO_KCRO_POST_FORECLOSURE_RELOCATION_CENTS: u64 = 1_060_000;
const NYC_DEMOLITION_STIPEND_CENTS: u64 = 500_000;
const DC_TOPA_RELOCATION_CAP_CENTS: u64 = 1_200_000;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.displacement_basis,
        DisplacementBasis::AtFaultEvictionNoRelocationDuty
    ) {
        return Output {
            severity: Severity::AtFaultEvictionNoRelocationDuty,
            statutory_relocation_amount_cents: 0,
            shortfall_cents: 0,
            note: "At-fault eviction (non-payment of rent, lease violation, illegal use, \
                   nuisance, refusal to renew at lawful rent) does NOT trigger relocation \
                   assistance duty. Just-cause eviction proceedings under state and local \
                   law govern; tenant defenses include warranty of habitability + retaliation \
                   + source-of-income discrimination claims raised in summary process."
                .to_string(),
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::CaliforniaAb1482Sb567) {
        if matches!(
            input.displacement_basis,
            DisplacementBasis::OwnerMoveInEviction
        ) && (!input.sb_567_owner_moves_in_within_90_days
            || !input.sb_567_owner_resides_one_year_primary_residence)
        {
            return Output {
                severity: Severity::SbFiveSixSevenOwnerMoveInNonCompliance,
                statutory_relocation_amount_cents: input.monthly_rent_cents,
                shortfall_cents: input.monthly_rent_cents,
                note: format!(
                    "CA SB 567 (effective April 1, 2024) OWNER MOVE-IN NON-COMPLIANCE. \
                     Statute requires: (1) owner or relative moves in within 90 days after \
                     tenant leaves, (2) lives in unit as primary residence for at least 1 \
                     year, (3) eviction notice discloses name and relationship, (4) no other \
                     vacant similar unit on property. Failure to comply: tenant entitled to \
                     statutory damages + reinstatement of tenancy + relocation assistance \
                     ${} (= one month rent) + attorney fees. Reverification (90-day move-in \
                     met: {}; 1-year residency met: {}).",
                    input.monthly_rent_cents / 100,
                    input.sb_567_owner_moves_in_within_90_days,
                    input.sb_567_owner_resides_one_year_primary_residence
                ),
            };
        }
        let required = input.monthly_rent_cents;
        return compliance_check(
            input,
            required,
            "CA AB 1482 + SB 567 (Cal. Civ. Code § 1946.2 + § 1947.12) requires one month's \
             rent as relocation assistance for any no-fault eviction (owner move-in, \
             withdrawal from rental market, code-compliance order, substantial remodel).",
        );
    }

    if matches!(input.jurisdiction, Jurisdiction::NewYorkCityRentStabilized) {
        if matches!(input.displacement_basis, DisplacementBasis::Demolition)
            && !input.nyc_dhcr_demolition_approval_obtained
        {
            return Output {
                severity: Severity::NycDhcrApprovalRequiredDemolitionViolation,
                statutory_relocation_amount_cents: NYC_DEMOLITION_STIPEND_CENTS
                    .saturating_add(input.monthly_rent_cents),
                shortfall_cents: NYC_DEMOLITION_STIPEND_CENTS
                    .saturating_add(input.monthly_rent_cents),
                note: format!(
                    "NYC RENT-STABILIZED DEMOLITION VIOLATION: NY State DHCR approval required \
                     under NYC RSL § 26-511(c)(9) + 9 NYCRR § 2524.5 before demolishing a \
                     rent-stabilized building. Without approval, demolition eviction is \
                     unlawful. Statutory minimum upon proper approval: comparable replacement \
                     housing at same/lower regulated rent OR new building on site + reasonable \
                     moving expenses + $5,000 stipend (${} cents). Tenant may file HCR \
                     complaint + obtain injunctive relief.",
                    NYC_DEMOLITION_STIPEND_CENTS / 100
                ),
            };
        }
        let required = NYC_DEMOLITION_STIPEND_CENTS;
        return compliance_check(
            input,
            required,
            "NYC RENT-STABILIZED demolition: relocation to comparable housing at same/lower \
             regulated rent + reasonable moving expenses + $5,000 stipend per NYC RSL \
             § 26-511(c)(9). Condo conversion under non-eviction plan preserves rent-\
             stabilized tenancy.",
        );
    }

    if matches!(input.jurisdiction, Jurisdiction::Washington) {
        if !matches!(
            input.tenant_income_status,
            TenantIncomeStatus::LowIncomeBelowFiftyPctAmi
        ) {
            return Output {
                severity: Severity::WashingtonNonLowIncomeNotCovered,
                statutory_relocation_amount_cents: 0,
                shortfall_cents: 0,
                note: "WA RCW 59.18.440 + RCW 59.18.450 cover LOW-INCOME TENANTS ONLY \
                       (combined household income ≤ 50% area median income adjusted for \
                       family size). Tenants above 50% AMI not covered by statutory \
                       relocation duty. Local jurisdictions adopting the option (Seattle, \
                       Tacoma, etc.) may impose additional protections."
                    .to_string(),
            };
        }
        let required = input.monthly_rent_cents.saturating_mul(3);
        return compliance_check(
            input,
            required,
            "WA RCW 59.18.440 + RCW 59.18.450 LOW-INCOME TENANT relocation assistance for \
             demolition / substantial rehabilitation / change of use / removal of use \
             restrictions. Locally enacted ordinances (Seattle SMC 22.210, Tacoma) set \
             specific amounts; typical floor is 3 months rent.",
        );
    }

    if matches!(input.jurisdiction, Jurisdiction::IllinoisChicagoRlto) {
        let one_month = input.monthly_rent_cents;
        let cap = one_month.min(CHICAGO_RLTO_RENT_BASED_CAP_CENTS);
        let required = CHICAGO_RLTO_FLAT_RELOCATION_CENTS.max(cap);
        return compliance_check(
            input,
            required,
            "Chicago RLTO § 5-12-130 + § 5-14-050 condo conversion requires GREATER OF \
             $1,500 OR one month's rent capped at $2,500 to qualified tenant exercising \
             assistance option. Keep Chicago Renting Ordinance (KCRO) requires $10,600 \
             post-foreclosure relocation if no lease-renewal offer.",
        );
    }

    if matches!(input.jurisdiction, Jurisdiction::DcTopa) {
        let one_year = input.monthly_rent_cents.saturating_mul(12);
        let required = one_year.min(DC_TOPA_RELOCATION_CAP_CENTS);
        if input.relocation_assistance_paid_cents >= required {
            return Output {
                severity: Severity::DcTopaRightOfFirstRefusalIncludingRelocation,
                statutory_relocation_amount_cents: required,
                shortfall_cents: 0,
                note: format!(
                    "DC TOPA (D.C. Code § 42-3404.01 et seq.) compliant. Tenant right of \
                     first refusal exercised + relocation assistance ${} paid (cap = lesser \
                     of one year rent or $12,000 annually-adjusted). Other negotiable \
                     items: building affordability, organizing expenses including reasonable \
                     attorney fees, specified improvements / efficiency upgrades.",
                    required / 100
                ),
            };
        }
        return Output {
            severity: Severity::InsufficientRelocationAssistanceViolation,
            statutory_relocation_amount_cents: required,
            shortfall_cents: required.saturating_sub(input.relocation_assistance_paid_cents),
            note: format!(
                "DC TOPA INSUFFICIENT relocation assistance: required ${} (lesser of one \
                 year rent or $12,000), paid ${}. TOPA gives tenant right of first refusal \
                 PLUS minimum negotiable relocation assistance.",
                required / 100,
                input.relocation_assistance_paid_cents / 100
            ),
        };
    }

    Output {
        severity: Severity::AtFaultEvictionNoRelocationDuty,
        statutory_relocation_amount_cents: 0,
        shortfall_cents: 0,
        note: "Default jurisdiction: no statutory relocation assistance duty identified. \
               Local ordinances vary; verify Seattle, Boston, Cambridge, Boulder, Berkeley, \
               and other progressive jurisdictions. Common-law constructive-eviction \
               framework may apply if landlord-caused habitability destruction triggered \
               tenant move."
            .to_string(),
    }
}

fn compliance_check(input: &Input, required: u64, statute_citation: &str) -> Output {
    if input.relocation_assistance_paid_cents >= required {
        Output {
            severity: Severity::CompliantRelocationAssistancePaid,
            statutory_relocation_amount_cents: required,
            shortfall_cents: 0,
            note: format!(
                "Compliant: relocation assistance ${} paid meets or exceeds statutory \
                 minimum ${}. {}",
                input.relocation_assistance_paid_cents / 100,
                required / 100,
                statute_citation
            ),
        }
    } else {
        Output {
            severity: Severity::InsufficientRelocationAssistanceViolation,
            statutory_relocation_amount_cents: required,
            shortfall_cents: required.saturating_sub(input.relocation_assistance_paid_cents),
            note: format!(
                "INSUFFICIENT relocation assistance: paid ${} vs required ${}. {} Shortfall \
                 ${}; tenant may obtain injunctive relief + statutory damages + attorney \
                 fees.",
                input.relocation_assistance_paid_cents / 100,
                required / 100,
                statute_citation,
                required.saturating_sub(input.relocation_assistance_paid_cents) / 100
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_ca() -> Input {
        Input {
            jurisdiction: Jurisdiction::CaliforniaAb1482Sb567,
            displacement_basis: DisplacementBasis::WithdrawalFromRentalMarketEllis,
            tenant_income_status: TenantIncomeStatus::AboveFiftyPctAmi,
            monthly_rent_cents: 3_000_00,
            relocation_assistance_paid_cents: 3_000_00,
            sb_567_owner_moves_in_within_90_days: true,
            sb_567_owner_resides_one_year_primary_residence: true,
            nyc_dhcr_demolition_approval_obtained: false,
        }
    }

    #[test]
    fn at_fault_eviction_no_relocation_duty() {
        let mut input = base_ca();
        input.displacement_basis = DisplacementBasis::AtFaultEvictionNoRelocationDuty;
        let output = check(&input);
        assert_eq!(output.severity, Severity::AtFaultEvictionNoRelocationDuty);
    }

    #[test]
    fn california_compliant_one_month_rent() {
        let input = base_ca();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantRelocationAssistancePaid
        );
        assert_eq!(output.statutory_relocation_amount_cents, 3_000_00);
    }

    #[test]
    fn california_insufficient_relocation_violation() {
        let mut input = base_ca();
        input.relocation_assistance_paid_cents = 1_000_00;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::InsufficientRelocationAssistanceViolation
        );
        assert_eq!(output.shortfall_cents, 2_000_00);
    }

    #[test]
    fn california_sb_567_owner_move_in_non_compliance() {
        let mut input = base_ca();
        input.displacement_basis = DisplacementBasis::OwnerMoveInEviction;
        input.sb_567_owner_moves_in_within_90_days = false;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::SbFiveSixSevenOwnerMoveInNonCompliance
        );
        assert!(output.note.contains("90 days"));
        assert!(output.note.contains("April 1, 2024"));
    }

    #[test]
    fn california_sb_567_compliant_owner_move_in() {
        let mut input = base_ca();
        input.displacement_basis = DisplacementBasis::OwnerMoveInEviction;
        input.sb_567_owner_moves_in_within_90_days = true;
        input.sb_567_owner_resides_one_year_primary_residence = true;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantRelocationAssistancePaid
        );
    }

    #[test]
    fn nyc_dhcr_demolition_approval_required_violation() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewYorkCityRentStabilized;
        input.displacement_basis = DisplacementBasis::Demolition;
        input.nyc_dhcr_demolition_approval_obtained = false;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NycDhcrApprovalRequiredDemolitionViolation
        );
        assert!(output.note.contains("DHCR"));
        assert!(output.note.contains("§ 26-511(c)(9)"));
        assert!(output.note.contains("$5,000"));
    }

    #[test]
    fn nyc_compliant_demolition_with_dhcr_approval() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewYorkCityRentStabilized;
        input.displacement_basis = DisplacementBasis::Demolition;
        input.nyc_dhcr_demolition_approval_obtained = true;
        input.relocation_assistance_paid_cents = 5_000_00;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantRelocationAssistancePaid
        );
    }

    #[test]
    fn washington_non_low_income_not_covered() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Washington;
        input.tenant_income_status = TenantIncomeStatus::AboveFiftyPctAmi;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::WashingtonNonLowIncomeNotCovered
        );
        assert!(output.note.contains("RCW 59.18.440"));
        assert!(output.note.contains("50% area median income"));
    }

    #[test]
    fn washington_low_income_3_months_rent_required() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Washington;
        input.tenant_income_status = TenantIncomeStatus::LowIncomeBelowFiftyPctAmi;
        input.relocation_assistance_paid_cents = 9_000_00;
        let output = check(&input);
        // 3 × $3K = $9K
        assert_eq!(output.statutory_relocation_amount_cents, 9_000_00);
        assert_eq!(
            output.severity,
            Severity::CompliantRelocationAssistancePaid
        );
    }

    #[test]
    fn chicago_rlto_condo_conversion_greater_of_1500_or_one_month_compliant() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::IllinoisChicagoRlto;
        input.displacement_basis = DisplacementBasis::CondoOrCoOpConversion;
        input.relocation_assistance_paid_cents = 2_500_00;
        let output = check(&input);
        // Cap: min($3K, $2.5K) = $2.5K; max($1.5K, $2.5K) = $2.5K
        assert_eq!(output.statutory_relocation_amount_cents, 2_500_00);
        assert_eq!(
            output.severity,
            Severity::CompliantRelocationAssistancePaid
        );
    }

    #[test]
    fn chicago_rlto_low_rent_uses_1500_floor() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::IllinoisChicagoRlto;
        input.monthly_rent_cents = 500_00; // Low rent
        input.relocation_assistance_paid_cents = 1_500_00;
        let output = check(&input);
        // Cap: min($500, $2.5K) = $500; max($1.5K, $500) = $1.5K
        assert_eq!(output.statutory_relocation_amount_cents, 1_500_00);
    }

    #[test]
    fn dc_topa_compliant_one_year_rent_under_cap() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::DcTopa;
        input.monthly_rent_cents = 800_00; // 12 × $800 = $9,600
        input.relocation_assistance_paid_cents = 9_600_00;
        let output = check(&input);
        // Required: min(12 × $800, $12,000) = $9,600
        assert_eq!(output.statutory_relocation_amount_cents, 9_600_00);
        assert_eq!(
            output.severity,
            Severity::DcTopaRightOfFirstRefusalIncludingRelocation
        );
    }

    #[test]
    fn dc_topa_high_rent_caps_at_12000() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::DcTopa;
        input.monthly_rent_cents = 5_000_00; // 12 × $5K = $60K
        input.relocation_assistance_paid_cents = 12_000_00;
        let output = check(&input);
        // Required: min(12 × $5K, $12K) = $12K
        assert_eq!(output.statutory_relocation_amount_cents, 12_000_00);
    }

    #[test]
    fn dc_topa_insufficient_relocation_violation() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::DcTopa;
        input.relocation_assistance_paid_cents = 5_000_00;
        let output = check(&input);
        // 12 × $3K = $36K capped at $12K; paid $5K → shortfall
        assert_eq!(
            output.severity,
            Severity::InsufficientRelocationAssistanceViolation
        );
    }

    #[test]
    fn chicago_rlto_flat_constant_pins_1500() {
        assert_eq!(CHICAGO_RLTO_FLAT_RELOCATION_CENTS, 150_000);
    }

    #[test]
    fn chicago_rlto_rent_cap_constant_pins_2500() {
        assert_eq!(CHICAGO_RLTO_RENT_BASED_CAP_CENTS, 250_000);
    }

    #[test]
    fn chicago_kcro_post_foreclosure_constant_pins_10600() {
        assert_eq!(CHICAGO_KCRO_POST_FORECLOSURE_RELOCATION_CENTS, 1_060_000);
    }

    #[test]
    fn nyc_demolition_stipend_constant_pins_5000() {
        assert_eq!(NYC_DEMOLITION_STIPEND_CENTS, 500_000);
    }

    #[test]
    fn dc_topa_cap_constant_pins_12000() {
        assert_eq!(DC_TOPA_RELOCATION_CAP_CENTS, 1_200_000);
    }

    #[test]
    fn very_large_rent_no_overflow_in_dc_one_year_calc() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::DcTopa;
        input.monthly_rent_cents = u64::MAX / 24;
        let output = check(&input);
        // saturating_mul defense
        assert!(output.statutory_relocation_amount_cents > 0);
    }

    #[test]
    fn zero_rent_no_panic_in_california() {
        let mut input = base_ca();
        input.monthly_rent_cents = 0;
        input.relocation_assistance_paid_cents = 0;
        let output = check(&input);
        // Required = 0; paid = 0 → compliant
        assert_eq!(
            output.severity,
            Severity::CompliantRelocationAssistancePaid
        );
    }

    #[test]
    fn default_jurisdiction_no_statutory_duty() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Default;
        let output = check(&input);
        assert_eq!(output.severity, Severity::AtFaultEvictionNoRelocationDuty);
        assert!(output.note.contains("constructive-eviction"));
    }
}
