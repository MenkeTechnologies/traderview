//! Texas HB 2127 of 2023 (Texas Regulatory Consistency Act —
//! "Death Star Bill") State Preemption Compliance Module.
//!
//! Pure-compute check for landlord compliance with Texas HB
//! 2127, the broad state-preemption regime that bars cities
//! and counties from passing ordinances in 8 broad chapters
//! of Texas state code (agriculture, business and commerce,
//! finance, insurance, labor, local government, natural
//! resources, occupations, property) unless prior legislative
//! approval. Effective September 1, 2023; authorizes a private
//! right of action — individuals and trade associations may
//! sue cities/counties to overturn preempted ordinances.
//! Court status: Travis County District Court declared HB 2127
//! unconstitutional on August 30, 2023, but did NOT enjoin
//! enforcement; State of Texas appealed and HB 2127 took
//! effect on schedule.
//!
//! Web research (verified 2026-06-03):
//! - **Texas HB 2127 of 2023** ("Texas Regulatory Consistency
//!   Act"; informally "**Death Star Bill**") — passed by
//!   Texas Legislature in May 2023; effective **September 1,
//!   2023** ([Texas Legislature 88(R) HB 2127](https://capitol.texas.gov/tlodocs/88R/billtext/pdf/HB02127I.pdf);
//!   [Texas Standard — Death Star Bill Could Affect Local
//!   Governments](https://www.texasstandard.org/stories/death-star-bill-texas-hb-2127-preemption-law-local-governments/);
//!   [Texas Tribune — Houston Sues Texas to Block Death Star
//!   Preemption Law](https://www.texastribune.org/2023/07/03/houston-texas-lawsuit-local-control/)).
//! - **Scope**: bars cities and counties from passing
//!   ordinances in **8 BROAD CHAPTERS** of Texas state code:
//!   (1) Agriculture, (2) Business and Commerce, (3) Finance,
//!   (4) Insurance, (5) Labor, (6) Local Government, (7)
//!   Natural Resources, (8) Occupations, plus Property Code.
//! - **Private Right of Action**: HB 2127 expressly authorizes
//!   **INDIVIDUALS AND TRADE ASSOCIATIONS** to sue cities or
//!   counties for violations of the preemption — including
//!   Texas Apartment Association (TAA), real estate developer
//!   groups, and individual property owners.
//! - **Landlord-Tenant Implications**: preempts local rules
//!   on rent notices, eviction notice provisions, source-of-
//!   income protections, tenants' bill of rights ordinances,
//!   proactive apartment inspections programs, late fee caps,
//!   security deposit caps. Particularly affected: **San
//!   Antonio Tenant Bill of Rights** and **Proactive Apartment
//!   Inspections Program**, Austin code enforcement rules,
//!   Dallas tenant protection ordinances, Houston rental rules.
//! - **Texas Property Code Chapter 92**: governs residential
//!   landlord-tenant relationships statewide; uniform state
//!   framework now applies preemptively over local ordinances
//!   addressing landlord-tenant matters.
//! - **Court Status**: Travis County District Court Judge
//!   Maya Guerra Gamble ruled HB 2127 **UNCONSTITUTIONAL** on
//!   **August 30, 2023** in *City of Houston v. State of Texas*
//!   (joined by San Antonio and El Paso) — declared "House Bill
//!   2127 in its entirety is unconstitutional — facially, as
//!   applied to Houston as a constitutional home rule city and
//!   to local laws that are not already preempted under
//!   Article XI, Section 5 of the Texas Constitution"
//!   ([Texas Tribune — Judge Declares HB 2127 Unconstitutional](https://www.texastribune.org/2023/08/30/texas-death-star-bill-unconstitutional/);
//!   [Texas Observer — Judge Strikes Down Lege's Power-Grab](https://www.texasobserver.org/judge-strikes-down-leges-power-grab-against-cities-hb-2127/)).
//! - **Appeal Status**: judge did NOT enjoin enforcement; State
//!   of Texas immediately appealed; HB 2127 took effect on
//!   schedule **September 1, 2023** pending appellate review.
//!   As of 2025-2026, law remains in effect with constitutional
//!   challenge pending Texas appellate courts.
//! - **Home Rule Cities**: HB 2127 directly conflicts with
//!   Article XI, Section 5 of the Texas Constitution which
//!   grants self-governance authority to home rule cities
//!   (Houston, Dallas, San Antonio, Austin, Fort Worth, El
//!   Paso, Arlington, etc. with population > 5,000).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const TX_HB_2127_EFFECTIVE_DATE_YEAR: u32 = 2023;
pub const TX_HB_2127_EFFECTIVE_DATE_MONTH: u32 = 9;
pub const TX_HB_2127_EFFECTIVE_DATE_DAY: u32 = 1;
pub const TX_HB_2127_DISTRICT_COURT_RULING_DATE_YEAR: u32 = 2023;
pub const TX_HB_2127_DISTRICT_COURT_RULING_DATE_MONTH: u32 = 8;
pub const TX_HB_2127_DISTRICT_COURT_RULING_DATE_DAY: u32 = 30;
pub const TX_HB_2127_PREEMPTED_CHAPTERS_COUNT: u32 = 8;
pub const TX_HOME_RULE_CITY_MIN_POPULATION: u32 = 5_000;
pub const TX_HB_2127_LEGISLATIVE_SESSION: u32 = 88;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyJurisdiction {
    TexasSubjectToHb2127Preemption,
    NotInTexas,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalOrdinanceCategory {
    NoLocalOrdinanceAtIssue,
    SanAntonioTenantBillOfRights,
    SanAntonioProactiveApartmentInspectionsProgram,
    LocalRentIncreaseNoticeExtension,
    LocalEvictionNoticeProvision,
    LocalSourceOfIncomeProtection,
    LocalLateFeeCap,
    LocalSecurityDepositCap,
    LocalCodeEnforcementProactiveInspection,
    OtherLocalLandlordTenantOrdinanceUnderTexasPropertyCodeCh92,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementActor {
    IndividualPropertyOwnerOrLandlord,
    TexasApartmentAssociationOrTradeAssociation,
    CityOrCountyEnforcingPreemptedOrdinance,
    NoEnforcementAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HomeRuleCityStatus {
    HomeRuleCityPopulationOver5000,
    GeneralLawCityPopulationUnder5000,
    NotInTexasCity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TexasHb2127Mode {
    NotApplicableNotInTexas,
    NotApplicableNoOrdinanceOrEnforcementAction,
    CompliantStatewidePreemptionRespectedTexasPropertyCodeCh92Only,
    CompliantPrivateRightOfActionToOverturnPreemptedOrdinance,
    ViolationLocalOrdinanceAppliedPreemptedUnderHb2127,
    ViolationCityOrCountyEnforcingPreemptedLandlordTenantOrdinance,
    ViolationProactiveApartmentInspectionProgramPreempted,
    ViolationTenantBillOfRightsOrdinancePreempted,
    NoteConstitutionalChallengePendingHomeRuleCityArticleXiSection5,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_jurisdiction: PropertyJurisdiction,
    pub local_ordinance_category: LocalOrdinanceCategory,
    pub enforcement_actor: EnforcementActor,
    pub home_rule_city_status: HomeRuleCityStatus,
    pub city_enforcing_against_landlord: bool,
    pub trade_association_or_individual_filed_suit_to_overturn: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: TexasHb2127Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalTexasHb2127StatePreemptionInput = Input;
pub type RentalTexasHb2127StatePreemptionOutput = Output;
pub type RentalTexasHb2127StatePreemptionResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Texas HB 2127 of 2023 (Texas Regulatory Consistency Act; informally 'Death Star Bill') — passed by Texas Legislature 88th Regular Session May 2023; effective September 1, 2023".to_string(),
        "Scope — bars cities and counties from passing ordinances in 8 broad chapters of Texas state code: (1) Agriculture, (2) Business and Commerce, (3) Finance, (4) Insurance, (5) Labor, (6) Local Government, (7) Natural Resources, (8) Occupations, plus Property Code".to_string(),
        "Private Right of Action — HB 2127 expressly authorizes individuals and trade associations to sue cities or counties for violations of the preemption — including Texas Apartment Association (TAA), real estate developer groups, and individual property owners".to_string(),
        "Landlord-Tenant Implications — preempts local rules on rent notices, eviction notice provisions, source-of-income protections, tenants' bill of rights ordinances, proactive apartment inspections programs, late fee caps, security deposit caps".to_string(),
        "Particularly affected ordinances — San Antonio Tenant Bill of Rights, San Antonio Proactive Apartment Inspections Program, Austin code enforcement rules, Dallas tenant protection ordinances, Houston rental rules".to_string(),
        "Texas Property Code Chapter 92 — governs residential landlord-tenant relationships statewide; uniform state framework now applies preemptively over local ordinances addressing landlord-tenant matters".to_string(),
        "Court Status — Travis County District Court Judge Maya Guerra Gamble ruled HB 2127 UNCONSTITUTIONAL on August 30, 2023 in City of Houston v. State of Texas (joined by San Antonio and El Paso); facially unconstitutional as applied to Houston as constitutional home rule city".to_string(),
        "Appeal Status — judge did NOT enjoin enforcement; State of Texas immediately appealed; HB 2127 took effect on schedule September 1, 2023 pending appellate review; as of 2025-2026 law remains in effect with constitutional challenge pending Texas appellate courts".to_string(),
        "Home Rule Cities — HB 2127 directly conflicts with Article XI, Section 5 of the Texas Constitution which grants self-governance authority to home rule cities (Houston, Dallas, San Antonio, Austin, Fort Worth, El Paso, Arlington, etc. with population > 5,000)".to_string(),
        "Texas Legislature 88(R) HB 2127 — primary bill text and legislative history".to_string(),
        "Texas Standard — Death Star Bill Could Affect Local Governments — analysis".to_string(),
        "Texas Tribune — Houston Sues Texas to Block Death Star Preemption Law".to_string(),
        "Texas Observer — Judge Strikes Down Lege's Power-Grab Against Cities".to_string(),
        "Apartment Association of Greater Dallas — State Preemption Bill Court Ruling".to_string(),
    ];

    if input.property_jurisdiction == PropertyJurisdiction::NotInTexas {
        return Output {
            mode: TexasHb2127Mode::NotApplicableNotInTexas,
            statutory_basis: "Property outside Texas; HB 2127 inapplicable".to_string(),
            notes: "NOT APPLICABLE: property outside Texas; Texas HB 2127 state preemption regime inapplicable.".to_string(),
            citations,
        };
    }

    if input.local_ordinance_category == LocalOrdinanceCategory::NoLocalOrdinanceAtIssue
        && input.enforcement_actor == EnforcementActor::NoEnforcementAction
    {
        return Output {
            mode: TexasHb2127Mode::NotApplicableNoOrdinanceOrEnforcementAction,
            statutory_basis: "Texas HB 2127 — no local ordinance or enforcement action triggered".to_string(),
            notes: "NOT APPLICABLE: no local landlord-tenant ordinance at issue and no enforcement action; HB 2127 preemption framework not invoked for this scenario.".to_string(),
            citations,
        };
    }

    if input.trade_association_or_individual_filed_suit_to_overturn {
        return Output {
            mode: TexasHb2127Mode::CompliantPrivateRightOfActionToOverturnPreemptedOrdinance,
            statutory_basis: "Texas HB 2127 — private right of action authorized to overturn preempted local ordinances".to_string(),
            notes: "COMPLIANT: trade association (Texas Apartment Association) or individual property owner has filed suit under HB 2127's private right of action to overturn preempted local landlord-tenant ordinance; statewide uniform Property Code Chapter 92 framework asserted.".to_string(),
            citations,
        };
    }

    if input.local_ordinance_category == LocalOrdinanceCategory::SanAntonioTenantBillOfRights
    {
        return Output {
            mode: TexasHb2127Mode::ViolationTenantBillOfRightsOrdinancePreempted,
            statutory_basis: "Texas HB 2127 — San Antonio Tenant Bill of Rights preempted".to_string(),
            notes: "VIOLATION: applying San Antonio Tenant Bill of Rights ordinance to residential tenancy; Texas HB 2127 preempts local tenants' bill of rights regulations in favor of uniform statewide Texas Property Code Chapter 92 framework.".to_string(),
            citations,
        };
    }

    if input.local_ordinance_category
        == LocalOrdinanceCategory::SanAntonioProactiveApartmentInspectionsProgram
        || input.local_ordinance_category
            == LocalOrdinanceCategory::LocalCodeEnforcementProactiveInspection
    {
        return Output {
            mode: TexasHb2127Mode::ViolationProactiveApartmentInspectionProgramPreempted,
            statutory_basis: "Texas HB 2127 — Proactive Apartment Inspections Program preempted".to_string(),
            notes: "VIOLATION: applying local Proactive Apartment Inspections Program (e.g., San Antonio) to residential property; Texas HB 2127 preempts local code-enforcement inspection regimes in favor of uniform statewide Texas Property Code Chapter 92 framework.".to_string(),
            citations,
        };
    }

    if matches!(
        input.local_ordinance_category,
        LocalOrdinanceCategory::LocalRentIncreaseNoticeExtension
            | LocalOrdinanceCategory::LocalEvictionNoticeProvision
            | LocalOrdinanceCategory::LocalSourceOfIncomeProtection
            | LocalOrdinanceCategory::LocalLateFeeCap
            | LocalOrdinanceCategory::LocalSecurityDepositCap
            | LocalOrdinanceCategory::OtherLocalLandlordTenantOrdinanceUnderTexasPropertyCodeCh92
    ) {
        return Output {
            mode: TexasHb2127Mode::ViolationLocalOrdinanceAppliedPreemptedUnderHb2127,
            statutory_basis: "Texas HB 2127 — local landlord-tenant ordinance preempted".to_string(),
            notes: format!(
                "VIOLATION: applying local landlord-tenant ordinance ({:?}) to residential tenancy; Texas HB 2127 preempts local rules in favor of uniform statewide Texas Property Code Chapter 92 framework.",
                input.local_ordinance_category
            ),
            citations,
        };
    }

    if input.enforcement_actor == EnforcementActor::CityOrCountyEnforcingPreemptedOrdinance
        && input.city_enforcing_against_landlord
    {
        return Output {
            mode: TexasHb2127Mode::ViolationCityOrCountyEnforcingPreemptedLandlordTenantOrdinance,
            statutory_basis: "Texas HB 2127 — city/county enforcement of preempted ordinance prohibited".to_string(),
            notes: "VIOLATION: city or county enforcing preempted local landlord-tenant ordinance against landlord; landlord may invoke HB 2127 private right of action to overturn enforcement; constitutional challenge under Article XI, Section 5 may apply for home rule cities.".to_string(),
            citations,
        };
    }

    if input.home_rule_city_status == HomeRuleCityStatus::HomeRuleCityPopulationOver5000 {
        return Output {
            mode: TexasHb2127Mode::NoteConstitutionalChallengePendingHomeRuleCityArticleXiSection5,
            statutory_basis: "Article XI, Section 5 of the Texas Constitution — home rule city self-governance authority".to_string(),
            notes: "NOTE: home rule city (population > 5,000) constitutional challenge to HB 2127 pending in Texas appellate courts; Travis County District Court ruled HB 2127 unconstitutional on August 30, 2023 but did not enjoin enforcement; status remains contested.".to_string(),
            citations,
        };
    }

    Output {
        mode: TexasHb2127Mode::CompliantStatewidePreemptionRespectedTexasPropertyCodeCh92Only,
        statutory_basis: "Texas HB 2127 — statewide preemption respected; Texas Property Code Chapter 92 only".to_string(),
        notes: "COMPLIANT: statewide preemption respected; only Texas Property Code Chapter 92 rules applied to residential landlord-tenant relationship; no local landlord-tenant ordinance imposed.".to_string(),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_compliant_texas() -> Input {
        Input {
            property_jurisdiction: PropertyJurisdiction::TexasSubjectToHb2127Preemption,
            local_ordinance_category: LocalOrdinanceCategory::NoLocalOrdinanceAtIssue,
            enforcement_actor: EnforcementActor::NoEnforcementAction,
            home_rule_city_status: HomeRuleCityStatus::GeneralLawCityPopulationUnder5000,
            city_enforcing_against_landlord: false,
            trade_association_or_individual_filed_suit_to_overturn: false,
        }
    }

    #[test]
    fn property_outside_texas_not_applicable() {
        let input = Input {
            property_jurisdiction: PropertyJurisdiction::NotInTexas,
            ..baseline_compliant_texas()
        };
        let result = check(&input);
        assert_eq!(result.mode, TexasHb2127Mode::NotApplicableNotInTexas);
    }

    #[test]
    fn no_ordinance_no_enforcement_not_applicable() {
        let result = check(&baseline_compliant_texas());
        assert_eq!(
            result.mode,
            TexasHb2127Mode::NotApplicableNoOrdinanceOrEnforcementAction
        );
    }

    #[test]
    fn san_antonio_tenant_bill_of_rights_preempted_violation() {
        let input = Input {
            local_ordinance_category: LocalOrdinanceCategory::SanAntonioTenantBillOfRights,
            ..baseline_compliant_texas()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TexasHb2127Mode::ViolationTenantBillOfRightsOrdinancePreempted
        );
    }

    #[test]
    fn san_antonio_proactive_apartment_inspections_preempted_violation() {
        let input = Input {
            local_ordinance_category:
                LocalOrdinanceCategory::SanAntonioProactiveApartmentInspectionsProgram,
            ..baseline_compliant_texas()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TexasHb2127Mode::ViolationProactiveApartmentInspectionProgramPreempted
        );
    }

    #[test]
    fn local_code_enforcement_inspection_preempted_violation() {
        let input = Input {
            local_ordinance_category:
                LocalOrdinanceCategory::LocalCodeEnforcementProactiveInspection,
            ..baseline_compliant_texas()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TexasHb2127Mode::ViolationProactiveApartmentInspectionProgramPreempted
        );
    }

    #[test]
    fn local_source_of_income_protection_preempted_violation() {
        let input = Input {
            local_ordinance_category: LocalOrdinanceCategory::LocalSourceOfIncomeProtection,
            ..baseline_compliant_texas()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TexasHb2127Mode::ViolationLocalOrdinanceAppliedPreemptedUnderHb2127
        );
    }

    #[test]
    fn local_late_fee_cap_preempted_violation() {
        let input = Input {
            local_ordinance_category: LocalOrdinanceCategory::LocalLateFeeCap,
            ..baseline_compliant_texas()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TexasHb2127Mode::ViolationLocalOrdinanceAppliedPreemptedUnderHb2127
        );
    }

    #[test]
    fn local_security_deposit_cap_preempted_violation() {
        let input = Input {
            local_ordinance_category: LocalOrdinanceCategory::LocalSecurityDepositCap,
            ..baseline_compliant_texas()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TexasHb2127Mode::ViolationLocalOrdinanceAppliedPreemptedUnderHb2127
        );
    }

    #[test]
    fn city_enforcing_preempted_ordinance_violation() {
        let input = Input {
            enforcement_actor: EnforcementActor::CityOrCountyEnforcingPreemptedOrdinance,
            city_enforcing_against_landlord: true,
            ..baseline_compliant_texas()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TexasHb2127Mode::ViolationCityOrCountyEnforcingPreemptedLandlordTenantOrdinance
        );
    }

    #[test]
    fn private_right_of_action_filed_compliant() {
        let input = Input {
            enforcement_actor: EnforcementActor::IndividualPropertyOwnerOrLandlord,
            trade_association_or_individual_filed_suit_to_overturn: true,
            ..baseline_compliant_texas()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TexasHb2127Mode::CompliantPrivateRightOfActionToOverturnPreemptedOrdinance
        );
    }

    #[test]
    fn trade_association_filed_suit_compliant() {
        let input = Input {
            enforcement_actor: EnforcementActor::TexasApartmentAssociationOrTradeAssociation,
            trade_association_or_individual_filed_suit_to_overturn: true,
            ..baseline_compliant_texas()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TexasHb2127Mode::CompliantPrivateRightOfActionToOverturnPreemptedOrdinance
        );
    }

    #[test]
    fn home_rule_city_constitutional_challenge_pending() {
        let input = Input {
            home_rule_city_status: HomeRuleCityStatus::HomeRuleCityPopulationOver5000,
            enforcement_actor: EnforcementActor::CityOrCountyEnforcingPreemptedOrdinance,
            ..baseline_compliant_texas()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TexasHb2127Mode::NoteConstitutionalChallengePendingHomeRuleCityArticleXiSection5
        );
    }

    #[test]
    fn citations_pin_hb_2127_court_status_and_property_code() {
        let result = check(&baseline_compliant_texas());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("Texas HB 2127 of 2023"));
        assert!(joined.contains("Texas Regulatory Consistency Act"));
        assert!(joined.contains("Death Star Bill"));
        assert!(joined.contains("September 1, 2023"));
        assert!(joined.contains("88th Regular Session"));
        assert!(joined.contains("8 broad chapters"));
        assert!(joined.contains("Texas Apartment Association"));
        assert!(joined.contains("Texas Property Code Chapter 92"));
        assert!(joined.contains("Travis County"));
        assert!(joined.contains("Judge Maya Guerra Gamble"));
        assert!(joined.contains("UNCONSTITUTIONAL"));
        assert!(joined.contains("August 30, 2023"));
        assert!(joined.contains("City of Houston v. State of Texas"));
        assert!(joined.contains("San Antonio"));
        assert!(joined.contains("El Paso"));
        assert!(joined.contains("Article XI, Section 5"));
        assert!(joined.contains("home rule"));
        assert!(joined.contains("Texas Tribune"));
        assert!(joined.contains("Texas Observer"));
        assert!(joined.contains("Texas Standard"));
    }

    #[test]
    fn constant_pin_dates_chapters_and_thresholds() {
        assert_eq!(TX_HB_2127_EFFECTIVE_DATE_YEAR, 2023);
        assert_eq!(TX_HB_2127_EFFECTIVE_DATE_MONTH, 9);
        assert_eq!(TX_HB_2127_EFFECTIVE_DATE_DAY, 1);
        assert_eq!(TX_HB_2127_DISTRICT_COURT_RULING_DATE_YEAR, 2023);
        assert_eq!(TX_HB_2127_DISTRICT_COURT_RULING_DATE_MONTH, 8);
        assert_eq!(TX_HB_2127_DISTRICT_COURT_RULING_DATE_DAY, 30);
        assert_eq!(TX_HB_2127_PREEMPTED_CHAPTERS_COUNT, 8);
        assert_eq!(TX_HOME_RULE_CITY_MIN_POPULATION, 5_000);
        assert_eq!(TX_HB_2127_LEGISLATIVE_SESSION, 88);
    }
}
