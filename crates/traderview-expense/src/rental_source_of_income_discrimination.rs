//! Source-of-income (SOI) housing discrimination compliance framework.
//!
//! Source-of-income discrimination prohibitions make it unlawful for housing providers
//! to refuse to rent, set different terms, or otherwise discriminate against applicants
//! based on their lawful source of income — including Section 8 Housing Choice Vouchers
//! under 42 U.S.C. § 1437f, Veterans Affairs Supportive Housing (VASH) vouchers,
//! Supplemental Security Income (SSI), Temporary Assistance for Needy Families (TANF),
//! Social Security, public assistance, and similar federal/state/local rental subsidies.
//! 23 states + DC + 100+ municipalities now prohibit SOI discrimination by statute or
//! ordinance, but federal Fair Housing Act (42 U.S.C. § 3601 et seq.) does NOT explicitly
//! include SOI as a protected class — protection arises from state and local law.
//!
//! Major statutes:
//!
//! - CA SB 329 (Cal. Gov. Code §§ 12921 + 12955; eff. Jan 1, 2020): SOI explicit
//!   protected class for state-licensed and California-FEHA-covered housing.
//! - NY State Human Rights Law (NY Exec. Law § 296(2-a); eff. Apr 12, 2019) + NYC
//!   Human Rights Law (NYC Admin. Code § 8-107(5)): SOI explicit protected class
//!   statewide.
//! - NJ Law Against Discrimination (N.J.S.A. 10:5-12.5; amended by L. 2021, c. 197 +
//!   2026 amendments): SOI protected class; landlords prohibited from applying
//!   minimum-income requirements not based exclusively on the tenant's portion of rent.
//! - WA RCW 59.18.255 (eff. Sept 30, 2018, HB 2578): voucher program participation
//!   protected class; landlord must deduct voucher amount from rent when applying
//!   rent-to-income screening ratios.
//! - MA Gen. L. ch. 151B § 4(10) + 4(11): SOI protected class including Section 8.
//! - IL Statewide silent; many municipalities (Chicago, Cook County, Urbana) prohibit
//!   SOI discrimination by ordinance.
//! - FEDERAL FLOOR: 42 U.S.C. § 3601 et seq. Fair Housing Act does NOT prohibit SOI
//!   discrimination explicitly; HUD Section 8 program (24 C.F.R. Part 982) is
//!   voluntary for landlords absent state/local SOI law. However, racially-disparate
//!   application of voucher refusal still creates 42 U.S.C. § 3604 disparate-impact
//!   liability per Texas Dept. of Housing v. Inclusive Communities (2015).
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - leginfo.legislature.ca.gov/faces/billTextClient.xhtml?bill_id=201920200SB329
//! - dhr.ny.gov/nysdhr-source-income
//! - njoag.gov/about/divisions-and-offices/division-on-civil-rights-home/know-the-law/njlad/discrimination-in-housing/
//! - app.leg.wa.gov/rcw/default.aspx?cite=59.18.255
//! - mass.gov/doc/guidance-on-preventing-housing-discrimination-based-on-source-of-income/download

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    NewYork,
    NewJersey,
    Washington,
    Massachusetts,
    IllinoisChicagoOrdinance,
    IllinoisStatewideNoStatute,
    FederalFhaFloorOnly,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncomeSource {
    /// Section 8 Housing Choice Voucher under 42 U.S.C. § 1437f.
    Section8HousingChoiceVoucher,
    /// VA Supportive Housing voucher.
    VashVeteransAffairsSupportiveHousingVoucher,
    /// Supplemental Security Income — 42 U.S.C. § 1381 et seq.
    SsiSupplementalSecurityIncome,
    /// Temporary Assistance for Needy Families.
    TanfTemporaryAssistanceNeedyFamilies,
    /// Social Security retirement / disability benefits.
    SocialSecurityRetirementOrDisability,
    /// State or local rental assistance program.
    StateOrLocalRentalAssistanceProgram,
    /// Employment-only income — typical baseline, NOT subject to SOI analysis.
    EmploymentOnlyIncomeNoSoiTrigger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlordAction {
    /// Outright refusal to consider applicant due to voucher / SOI.
    OutrightRefusalToRentDueToVoucher,
    /// Stated "no Section 8" policy on application / listing.
    StatedNoSection8PolicyOnListing,
    /// Applied minimum-income test to FULL rent (not tenant's portion only).
    MinimumIncomeTestAppliedToFullRentNotTenantPortion,
    /// Higher security deposit demanded from voucher holder.
    DiscriminatoryHigherDepositDemand,
    /// Compliant: voucher accepted, minimum-income test applied only to tenant portion.
    CompliantAcceptedVoucherTenantPortionScreening,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    EmploymentOnlyIncomeNoSoiAnalysisTriggered,
    CompliantSoiAcceptanceWithTenantPortionScreening,
    IllinoisStatewideNoStateCoverageNoSoiClaim,
    FederalFhaDisparateImpactAvailable,
    StateSoiStatuteViolationActualAndPunitiveDamages,
    NjLadMinimumIncomeRuleViolationTenantPortionOnly,
    WashingtonRcw59_18_255VoucherDeductionFailureViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub income_source: IncomeSource,
    pub landlord_action: LandlordAction,
    pub monthly_rent_cents: u64,
    pub voucher_subsidy_amount_cents: u64,
    pub tenant_actual_damages_cents: u64,
}

pub type RentalSourceOfIncomeDiscriminationInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub estimated_landlord_exposure_cents: u64,
    pub note: String,
}

pub type RentalSourceOfIncomeDiscriminationOutput = Output;
pub type RentalSourceOfIncomeDiscriminationResult = Output;

const FHA_CIVIL_PENALTY_FIRST_VIOLATION_CENTS: u64 = 1_978_700;
const NJ_TYPICAL_CIVIL_PENALTY_CENTS: u64 = 1_000_000;
const NYS_HRL_TYPICAL_DAMAGES_CENTS: u64 = 500_000;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.income_source,
        IncomeSource::EmploymentOnlyIncomeNoSoiTrigger
    ) {
        return Output {
            severity: Severity::EmploymentOnlyIncomeNoSoiAnalysisTriggered,
            estimated_landlord_exposure_cents: 0,
            note: "Applicant relies on employment income only — SOI discrimination analysis \
                   not triggered. SOI protections kick in when applicant relies on government \
                   subsidies, vouchers, public-assistance income, or other lawful non-wage \
                   income sources. Standard income-verification screening permitted; \
                   apply uniform rent-to-income ratio."
                .to_string(),
        };
    }

    if matches!(
        input.landlord_action,
        LandlordAction::CompliantAcceptedVoucherTenantPortionScreening
    ) {
        return Output {
            severity: Severity::CompliantSoiAcceptanceWithTenantPortionScreening,
            estimated_landlord_exposure_cents: 0,
            note: format!(
                "Compliant: voucher accepted; minimum-income test applied only to tenant's \
                 portion of rent (${} - voucher subsidy ${} = ${} tenant portion). Document \
                 the screening methodology with the application file: apply the same rent-to- \
                 income ratio used for non-voucher applicants but to the tenant-portion-only \
                 rent figure.",
                input.monthly_rent_cents / 100,
                input.voucher_subsidy_amount_cents / 100,
                input
                    .monthly_rent_cents
                    .saturating_sub(input.voucher_subsidy_amount_cents)
                    / 100
            ),
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::IllinoisStatewideNoStatute) {
        return Output {
            severity: Severity::IllinoisStatewideNoStateCoverageNoSoiClaim,
            estimated_landlord_exposure_cents: 0,
            note: "Illinois statewide has NO SOI antidiscrimination statute. Chicago Fair \
                   Housing Ordinance + Cook County Human Rights Ordinance + Urbana Human \
                   Rights Ordinance prohibit SOI discrimination for properties within those \
                   municipal boundaries. Statewide refusal of Section 8 voucher is not \
                   actionable absent racial-disparate-impact theory under 42 U.S.C. § 3604 \
                   per Texas Dept. of Housing v. Inclusive Communities, 576 U.S. 519 (2015)."
                .to_string(),
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::FederalFhaFloorOnly) {
        return Output {
            severity: Severity::FederalFhaDisparateImpactAvailable,
            estimated_landlord_exposure_cents: input.tenant_actual_damages_cents,
            note: format!(
                "Federal Fair Housing Act 42 U.S.C. § 3601 et seq. does NOT explicitly include \
                 SOI as a protected class. HUD Section 8 program (24 C.F.R. Part 982) is \
                 voluntary for landlords absent state/local SOI law. HOWEVER, racially \
                 disparate application of voucher refusal creates 42 U.S.C. § 3604 disparate- \
                 impact liability per Texas Dept. of Housing v. Inclusive Communities, 576 \
                 U.S. 519 (2015) — tenant must show racially disparate impact of facially \
                 neutral voucher-refusal policy. Tenant actual damages (${}) available; \
                 civil penalty (${}) under 42 U.S.C. § 3612(g)(3) inflation-adjusted.",
                input.tenant_actual_damages_cents / 100,
                FHA_CIVIL_PENALTY_FIRST_VIOLATION_CENTS / 100
            ),
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::Washington)
        && matches!(
            input.landlord_action,
            LandlordAction::MinimumIncomeTestAppliedToFullRentNotTenantPortion
        )
    {
        let exposure = input
            .tenant_actual_damages_cents
            .saturating_add(NJ_TYPICAL_CIVIL_PENALTY_CENTS);
        return Output {
            severity: Severity::WashingtonRcw59_18_255VoucherDeductionFailureViolation,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "Washington RCW 59.18.255 SPECIFIC VIOLATION. When screening applicants with \
                 vouchers, the landlord MUST deduct the voucher subsidy from the monthly rent \
                 before applying any rent-to-income ratio. Applying the minimum-income test to \
                 the FULL rent (${}) instead of the tenant's portion (${} = ${} - ${} voucher) \
                 violates the statute. Estimated exposure ${} = actual damages + statutory \
                 civil penalty + attorney fees under § 59.18.085.",
                input.monthly_rent_cents / 100,
                input
                    .monthly_rent_cents
                    .saturating_sub(input.voucher_subsidy_amount_cents)
                    / 100,
                input.monthly_rent_cents / 100,
                input.voucher_subsidy_amount_cents / 100,
                exposure / 100
            ),
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::NewJersey)
        && matches!(
            input.landlord_action,
            LandlordAction::MinimumIncomeTestAppliedToFullRentNotTenantPortion
        )
    {
        let exposure = input
            .tenant_actual_damages_cents
            .saturating_add(NJ_TYPICAL_CIVIL_PENALTY_CENTS);
        return Output {
            severity: Severity::NjLadMinimumIncomeRuleViolationTenantPortionOnly,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "New Jersey LAD (N.J.S.A. 10:5-12.5) violation. 2026 amendments prohibit \
                 application of minimum-income requirements that are not based exclusively on \
                 the portion of rent to be paid by the tenant. Applying minimum-income test to \
                 FULL rent (${}) instead of tenant portion (${}) violates statute. NJ DCR has \
                 brought multiple enforcement actions under this rule. Estimated exposure ${} \
                 = actual damages (${}) + NJ DCR civil penalty (${} typical) + attorney fees.",
                input.monthly_rent_cents / 100,
                input
                    .monthly_rent_cents
                    .saturating_sub(input.voucher_subsidy_amount_cents)
                    / 100,
                exposure / 100,
                input.tenant_actual_damages_cents / 100,
                NJ_TYPICAL_CIVIL_PENALTY_CENTS / 100
            ),
        };
    }

    let exposure = input
        .tenant_actual_damages_cents
        .saturating_add(NYS_HRL_TYPICAL_DAMAGES_CENTS);
    Output {
        severity: Severity::StateSoiStatuteViolationActualAndPunitiveDamages,
        estimated_landlord_exposure_cents: exposure,
        note: format!(
            "State SOI statute violation in {}. {} Tenant may bring administrative complaint \
             with state Division of Human Rights / Fair Employment and Housing / Civil Rights \
             Division and/or private right of action. Estimated exposure ${} = actual damages \
             (${}) + typical mental-anguish/emotional-distress damages (${}) + attorney fees + \
             civil penalty. CA can impose $25,000 punitive damages under Gov. Code § 12987; \
             NY can award unlimited mental-anguish damages under Exec. Law § 297(9).",
            jurisdiction_label(input.jurisdiction),
            statute_label(input.jurisdiction),
            exposure / 100,
            input.tenant_actual_damages_cents / 100,
            NYS_HRL_TYPICAL_DAMAGES_CENTS / 100
        ),
    }
}

fn jurisdiction_label(jurisdiction: Jurisdiction) -> &'static str {
    match jurisdiction {
        Jurisdiction::California => "California",
        Jurisdiction::NewYork => "New York",
        Jurisdiction::NewJersey => "New Jersey",
        Jurisdiction::Washington => "Washington",
        Jurisdiction::Massachusetts => "Massachusetts",
        Jurisdiction::IllinoisChicagoOrdinance => "Chicago (Cook County) Illinois",
        Jurisdiction::IllinoisStatewideNoStatute => "Illinois statewide",
        Jurisdiction::FederalFhaFloorOnly => "Federal FHA floor only",
        Jurisdiction::Default => "Default",
    }
}

fn statute_label(jurisdiction: Jurisdiction) -> &'static str {
    match jurisdiction {
        Jurisdiction::California => {
            "CA SB 329 (Cal. Gov. Code §§ 12921 + 12955) prohibits refusal to rent based on \
             source of income including Section 8 vouchers; effective January 1, 2020."
        }
        Jurisdiction::NewYork => {
            "NY State Human Rights Law (NY Exec. Law § 296(2-a)) + NYC HRL (NYC Admin. Code \
             § 8-107(5)) prohibit SOI discrimination statewide; effective April 12, 2019."
        }
        Jurisdiction::NewJersey => {
            "NJ LAD (N.J.S.A. 10:5-12.5) prohibits SOI discrimination; 2026 amendments \
             extend the tenant-portion-only minimum-income rule."
        }
        Jurisdiction::Washington => {
            "WA RCW 59.18.255 (HB 2578) prohibits voucher-participation discrimination; \
             effective September 30, 2018."
        }
        Jurisdiction::Massachusetts => {
            "MA Gen. L. ch. 151B § 4(10) + 4(11) protect public-assistance recipients including \
             Section 8 voucher holders from housing discrimination."
        }
        Jurisdiction::IllinoisChicagoOrdinance => {
            "Chicago Fair Housing Ordinance + Cook County Human Rights Ordinance prohibit SOI \
             discrimination in those municipal boundaries."
        }
        _ => "State SOI statute prohibits refusal to rent based on source of lawful income.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            income_source: IncomeSource::Section8HousingChoiceVoucher,
            landlord_action: LandlordAction::OutrightRefusalToRentDueToVoucher,
            monthly_rent_cents: 3_000_00,
            voucher_subsidy_amount_cents: 2_000_00,
            tenant_actual_damages_cents: 5_000_00,
        }
    }

    #[test]
    fn employment_only_income_no_soi_analysis_triggered() {
        let mut input = base();
        input.income_source = IncomeSource::EmploymentOnlyIncomeNoSoiTrigger;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::EmploymentOnlyIncomeNoSoiAnalysisTriggered
        );
        assert_eq!(output.estimated_landlord_exposure_cents, 0);
    }

    #[test]
    fn compliant_voucher_acceptance_with_tenant_portion_screening() {
        let mut input = base();
        input.landlord_action =
            LandlordAction::CompliantAcceptedVoucherTenantPortionScreening;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantSoiAcceptanceWithTenantPortionScreening
        );
        assert_eq!(output.estimated_landlord_exposure_cents, 0);
        assert!(output.note.contains("$1000"));
    }

    #[test]
    fn california_outright_refusal_state_violation() {
        let input = base();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::StateSoiStatuteViolationActualAndPunitiveDamages
        );
        assert!(output.note.contains("SB 329"));
        assert!(output.note.contains("§§ 12921 + 12955"));
        assert!(output.note.contains("$25,000"));
    }

    #[test]
    fn new_york_outright_refusal_state_violation() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::NewYork;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::StateSoiStatuteViolationActualAndPunitiveDamages
        );
        assert!(output.note.contains("§ 296(2-a)"));
        assert!(output.note.contains("§ 8-107(5)"));
        assert!(output.note.contains("April 12, 2019"));
    }

    #[test]
    fn new_jersey_minimum_income_rule_violation() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::NewJersey;
        input.landlord_action =
            LandlordAction::MinimumIncomeTestAppliedToFullRentNotTenantPortion;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NjLadMinimumIncomeRuleViolationTenantPortionOnly
        );
        assert!(output.note.contains("N.J.S.A. 10:5-12.5"));
        assert!(output.note.contains("NJ DCR"));
    }

    #[test]
    fn washington_voucher_deduction_failure_violation() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Washington;
        input.landlord_action =
            LandlordAction::MinimumIncomeTestAppliedToFullRentNotTenantPortion;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::WashingtonRcw59_18_255VoucherDeductionFailureViolation
        );
        assert!(output.note.contains("RCW 59.18.255"));
        assert!(output.note.contains("§ 59.18.085"));
    }

    #[test]
    fn massachusetts_state_violation() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Massachusetts;
        let output = check(&input);
        assert!(output.note.contains("ch. 151B"));
        assert!(output.note.contains("§ 4(10)"));
    }

    #[test]
    fn illinois_statewide_no_statute_no_claim() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::IllinoisStatewideNoStatute;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::IllinoisStatewideNoStateCoverageNoSoiClaim
        );
        assert!(output.note.contains("Inclusive Communities"));
        assert!(output.note.contains("576 U.S. 519"));
    }

    #[test]
    fn illinois_chicago_ordinance_state_violation() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::IllinoisChicagoOrdinance;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::StateSoiStatuteViolationActualAndPunitiveDamages
        );
        assert!(output.note.contains("Chicago Fair Housing Ordinance"));
    }

    #[test]
    fn federal_fha_floor_only_disparate_impact_available() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::FederalFhaFloorOnly;
        let output = check(&input);
        assert_eq!(output.severity, Severity::FederalFhaDisparateImpactAvailable);
        assert!(output.note.contains("42 U.S.C. § 3601"));
        assert!(output.note.contains("Inclusive Communities"));
        assert!(output.note.contains("$19787"));
    }

    #[test]
    fn vash_voucher_triggers_soi_analysis() {
        let mut input = base();
        input.income_source =
            IncomeSource::VashVeteransAffairsSupportiveHousingVoucher;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::StateSoiStatuteViolationActualAndPunitiveDamages
        );
    }

    #[test]
    fn ssi_triggers_soi_analysis() {
        let mut input = base();
        input.income_source = IncomeSource::SsiSupplementalSecurityIncome;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::StateSoiStatuteViolationActualAndPunitiveDamages
        );
    }

    #[test]
    fn tanf_triggers_soi_analysis() {
        let mut input = base();
        input.income_source = IncomeSource::TanfTemporaryAssistanceNeedyFamilies;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::StateSoiStatuteViolationActualAndPunitiveDamages
        );
    }

    #[test]
    fn social_security_triggers_soi_analysis() {
        let mut input = base();
        input.income_source = IncomeSource::SocialSecurityRetirementOrDisability;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::StateSoiStatuteViolationActualAndPunitiveDamages
        );
    }

    #[test]
    fn state_or_local_rental_assistance_triggers_soi_analysis() {
        let mut input = base();
        input.income_source =
            IncomeSource::StateOrLocalRentalAssistanceProgram;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::StateSoiStatuteViolationActualAndPunitiveDamages
        );
    }

    #[test]
    fn stated_no_section_8_policy_triggers_violation() {
        let mut input = base();
        input.landlord_action = LandlordAction::StatedNoSection8PolicyOnListing;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::StateSoiStatuteViolationActualAndPunitiveDamages
        );
    }

    #[test]
    fn discriminatory_higher_deposit_triggers_violation() {
        let mut input = base();
        input.landlord_action = LandlordAction::DiscriminatoryHigherDepositDemand;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::StateSoiStatuteViolationActualAndPunitiveDamages
        );
    }

    #[test]
    fn fha_civil_penalty_constant_pins_19787_dollars() {
        assert_eq!(FHA_CIVIL_PENALTY_FIRST_VIOLATION_CENTS, 1_978_700);
    }

    #[test]
    fn nj_typical_civil_penalty_constant_pins_10000_dollars() {
        assert_eq!(NJ_TYPICAL_CIVIL_PENALTY_CENTS, 1_000_000);
    }

    #[test]
    fn nys_hrl_typical_damages_constant_pins_5000_dollars() {
        assert_eq!(NYS_HRL_TYPICAL_DAMAGES_CENTS, 500_000);
    }

    #[test]
    fn very_large_damages_no_overflow() {
        let mut input = base();
        input.tenant_actual_damages_cents = u64::MAX;
        let output = check(&input);
        assert_eq!(output.estimated_landlord_exposure_cents, u64::MAX);
    }

    #[test]
    fn zero_damages_uses_baseline_typical_damages() {
        let mut input = base();
        input.tenant_actual_damages_cents = 0;
        let output = check(&input);
        // Baseline = $5,000 typical damages
        assert_eq!(output.estimated_landlord_exposure_cents, 5_000_00);
    }
}
