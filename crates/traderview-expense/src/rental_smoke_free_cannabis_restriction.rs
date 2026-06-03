//! Smoke-free housing + cannabis-use restriction compliance framework.
//!
//! Landlords face overlapping tobacco and cannabis smoking regulation at federal,
//! state, and local levels. HUD's 24 C.F.R. parts 200 + 982 + 5 final rule
//! (effective February 3, 2017) mandates smoke-free policies in HUD-subsidized
//! public housing. State and municipal laws authorize private landlords to prohibit
//! smoking and require advance disclosure to incoming tenants. Cannabis use carries
//! a parallel landlord-control framework: most state cannabis legalization statutes
//! expressly preserve landlord authority to prohibit cannabis consumption on
//! premises.
//!
//! Federal floor + state grid:
//!
//! - HUD 24 C.F.R. Parts 200 + 982 + 5 SMOKE-FREE PUBLIC HOUSING RULE (effective
//!   February 3, 2017): bans smoking of lit tobacco in HUD-subsidized public-housing
//!   apartments, common areas, administrative offices, and grounds within 25 feet.
//!   Public Housing Agencies must implement and enforce. Section 8 voucher
//!   properties NOT subject to the federal mandate (private landlord discretion).
//! - CA SB 332 (Cal. Civ. Code § 1947.5; effective Jan 1, 2012): authorizes private
//!   landlords to prohibit smoking on property; lease must contain smoke-free
//!   provision; pre-existing-tenant grandfathering may apply.
//! - CA AB-2103 / Cal. Civ. Code § 1947.5(b): landlord must disclose
//!   designated-smoking-areas to incoming tenants in writing.
//! - NY MTA + NYC Local Law 147 of 2017 (NYC Admin. Code § 17-505 et seq.):
//!   smoking-policy disclosure mandate for NYC multi-unit residential buildings.
//! - WA Smoking In Public Places Act (RCW 70.160): bans smoking in workplaces +
//!   public accommodations + within 25 feet of entrances; private rental units
//!   subject to landlord discretion under WA RLTA.
//! - IL Smoke Free Illinois Act (410 ILCS 82): bans smoking in public-accommodation
//!   indoor areas + within 15 feet of building entrances.
//! - OR State Smoke-Free Policy (ORS 433.835-990): bans in indoor workplaces +
//!   public accommodations.
//!
//! Cannabis-specific landlord rights:
//!
//! - CA AUMA / Prop 64 (Cal. Health & Safety Code § 11362.45(h)): private property
//!   owners may prohibit cannabis use, possession, sale, or cultivation on premises.
//! - NY MRTA (NY Cannabis Law § 222 + Penal Law § 222.05): landlord can prohibit
//!   cannabis combustion + cultivation; some local rent-stabilized protections.
//! - CO Amendment 64 + § 12-43.4 (Cannabis Code) + HB 1287: landlord may prohibit.
//! - WA RCW 69.50.4014: landlord may prohibit cannabis on rental property.
//! - All states with legal cannabis: federal Controlled Substances Act
//!   classification of marijuana as Schedule I retains landlord ability to
//!   prohibit due to Section 8 + federal-subsidized-housing federal preemption.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - leginfo.legislature.ca.gov/faces/billNavClient.xhtml?bill_id=201120120SB332
//! - hud.gov/program_offices/public_indian_housing/programs/ph/phecc/smokefree
//! - tenantstogether.org/updates/calif-governor-signs-senator-padilla-smoke-free-rental-housing-bill
//! - changelabsolutions.org/tobacco-control/question/landlords-smokefree

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    HudSubsidizedPublicHousing,
    California,
    NewYorkCity,
    Washington,
    Illinois,
    Oregon,
    Colorado,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubstanceCategory {
    /// Lit tobacco (cigarettes, cigars, pipes, hookah).
    LitTobaccoCigarettesOrCigarOrPipe,
    /// Electronic cigarettes (vaping); regulation varies — some states treat as
    /// tobacco, others separately.
    ElectronicCigaretteVaping,
    /// Cannabis combustion (smoking marijuana).
    CannabisCombustion,
    /// Cannabis vaporization (vaping marijuana).
    CannabisVaporization,
    /// Cannabis edibles or non-smoked forms (typically not restricted by smoke-free
    /// policy).
    CannabisEdiblesOrNonSmoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseDisclosureStatus {
    /// Smoke-free policy and designated-smoking-areas disclosed in lease.
    DisclosedInLeaseWithDesignatedAreas,
    /// Smoke-free policy disclosed but designated-smoking-areas missing.
    DisclosedButDesignatedAreasMissing,
    /// No disclosure provided to tenant.
    NoDisclosureProvided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyStatus {
    /// Lease signed before policy implementation; grandfathered.
    PreExistingTenantGrandfathered,
    /// New tenancy signed after policy implementation.
    NewTenancyPostPolicyImplementation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantSmokeFreePolicyEnforceable,
    HudFederalMandatePreemptsLocalLawSubsidizedHousing,
    PreExistingTenantGrandfatheredPolicyUnenforceable,
    LeaseDisclosureDefectivePolicyUnenforceable,
    NoDisclosureSmokeFreePolicyUnenforceable,
    CannabisEdiblesNotSubjectToSmokeFreePolicy,
    LandlordMayProhibitCannabisOnPremises,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub substance_category: SubstanceCategory,
    pub lease_disclosure_status: LeaseDisclosureStatus,
    pub tenancy_status: TenancyStatus,
}

pub type RentalSmokeFreeCannabisRestrictionInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub policy_enforceable: bool,
    pub note: String,
}

pub type RentalSmokeFreeCannabisRestrictionOutput = Output;
pub type RentalSmokeFreeCannabisRestrictionResult = Output;

const HUD_SMOKE_FREE_RULE_EFFECTIVE_YEAR: u32 = 2017;
const CA_SB_332_EFFECTIVE_YEAR: u32 = 2012;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.substance_category,
        SubstanceCategory::CannabisEdiblesOrNonSmoked
    ) {
        return Output {
            severity: Severity::CannabisEdiblesNotSubjectToSmokeFreePolicy,
            policy_enforceable: false,
            note: "Cannabis edibles, tinctures, and other non-smoked forms are NOT subject \
                   to a smoke-free policy by definition. Landlord may still prohibit cannabis \
                   USE generally on premises under state cannabis legalization statutes that \
                   preserve property-owner authority (CA H&S § 11362.45(h), NY Cannabis Law \
                   § 222 + Penal Law § 222.05, CO § 12-43.4, WA RCW 69.50.4014). Lease should \
                   contain a general no-cannabis-use clause if the landlord intends to \
                   prohibit non-smoked cannabis."
                .to_string(),
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::HudSubsidizedPublicHousing) {
        return Output {
            severity: Severity::HudFederalMandatePreemptsLocalLawSubsidizedHousing,
            policy_enforceable: true,
            note: format!(
                "HUD 24 C.F.R. Parts 200 + 982 + 5 final rule (effective Feb 3, \
                 {HUD_SMOKE_FREE_RULE_EFFECTIVE_YEAR}) MANDATES smoke-free policy in \
                 HUD-subsidized public housing. Lit tobacco smoking banned in apartments, \
                 common areas, administrative offices, and grounds within 25 feet. PHAs \
                 (Public Housing Agencies) must implement and enforce. Federal preemption \
                 overrides state law. Section 8 voucher properties NOT subject to federal \
                 mandate (private landlord discretion). Cannabis remains FEDERALLY ILLEGAL \
                 (Controlled Substances Act Schedule I) and is prohibited in HUD-subsidized \
                 housing regardless of state legalization."
            ),
        };
    }

    let cannabis_prohibitable = matches!(
        input.substance_category,
        SubstanceCategory::CannabisCombustion | SubstanceCategory::CannabisVaporization
    );

    if cannabis_prohibitable && policy_enforceable_in_jurisdiction(input) {
        return Output {
            severity: Severity::LandlordMayProhibitCannabisOnPremises,
            policy_enforceable: true,
            note: format!(
                "Cannabis use prohibitable on rental premises per state cannabis-\
                 legalization statute preserving landlord authority. {} Disclosure in lease \
                 mandatory in jurisdictions with codified smoke-free housing law. Federal \
                 Controlled Substances Act Schedule I classification independently supports \
                 prohibition. Coordinate with FHA reasonable-accommodation analysis for \
                 medical-cannabis users — federal courts have generally held that FHA does \
                 NOT require accommodation of cannabis use due to federal illegality \
                 (Forest City Residential Mgmt. v. Beasley, 71 F. Supp. 3d 715 (E.D. Mich. \
                 2014); James v. City of Costa Mesa, 700 F.3d 394 (9th Cir. 2012)).",
                state_cannabis_citation(input.jurisdiction)
            ),
        };
    }

    if matches!(
        input.tenancy_status,
        TenancyStatus::PreExistingTenantGrandfathered
    ) {
        return Output {
            severity: Severity::PreExistingTenantGrandfatheredPolicyUnenforceable,
            policy_enforceable: false,
            note: format!(
                "Pre-existing tenant grandfathered: lease signed before policy implementation. \
                 New smoke-free policy applies only to NEW tenants and lease renewals. \
                 Landlord may not retroactively impose smoke-free policy on existing \
                 tenants without lease-amendment consent. CA SB 332 (Cal. Civ. Code § 1947.5, \
                 effective Jan 1 {CA_SB_332_EFFECTIVE_YEAR}) preserves this grandfathering \
                 norm; similar protection in most state law. Wait for natural lease \
                 expiration / renewal to enforce."
            ),
        };
    }

    match input.lease_disclosure_status {
        LeaseDisclosureStatus::DisclosedInLeaseWithDesignatedAreas => Output {
            severity: Severity::CompliantSmokeFreePolicyEnforceable,
            policy_enforceable: true,
            note: format!(
                "Compliant smoke-free policy enforceable: disclosed in lease with \
                 designated-smoking-areas description. {} Landlord may evict for material \
                 lease violation following customary notice + cure procedures. Document \
                 specific lease-violation evidence (photos, witness statements, smell \
                 reports) for any summary-process proceeding.",
                state_disclosure_citation(input.jurisdiction)
            ),
        },
        LeaseDisclosureStatus::DisclosedButDesignatedAreasMissing => Output {
            severity: Severity::LeaseDisclosureDefectivePolicyUnenforceable,
            policy_enforceable: false,
            note: format!(
                "Lease disclosure DEFECTIVE: smoke-free policy disclosed but \
                 designated-smoking-areas description MISSING. CA Civ. Code § 1947.5(b) \
                 + similar state statutes require lease to identify portions of property \
                 that are smoke-free + portions that are not. Cure by amending lease with \
                 designated-areas description before enforcement. {}",
                state_disclosure_citation(input.jurisdiction)
            ),
        },
        LeaseDisclosureStatus::NoDisclosureProvided => Output {
            severity: Severity::NoDisclosureSmokeFreePolicyUnenforceable,
            policy_enforceable: false,
            note: format!(
                "No disclosure provided to tenant. Smoke-free policy is NOT enforceable \
                 against tenant absent contemporaneous lease-disclosure. CA Civ. Code \
                 § 1947.5(a) + similar state laws require advance written disclosure as \
                 condition of enforcement. Cure by issuing lease amendment with smoke-free \
                 policy + designated-areas description + tenant acknowledgment. {}",
                state_disclosure_citation(input.jurisdiction)
            ),
        },
    }
}

fn policy_enforceable_in_jurisdiction(_input: &Input) -> bool {
    // All surveyed states permit cannabis prohibition on rental premises.
    true
}

fn state_cannabis_citation(jurisdiction: Jurisdiction) -> &'static str {
    match jurisdiction {
        Jurisdiction::California => {
            "CA Health & Safety Code § 11362.45(h) preserves property-owner authority to \
             prohibit cannabis use, possession, sale, or cultivation on premises."
        }
        Jurisdiction::NewYorkCity => {
            "NY Cannabis Law § 222 + Penal Law § 222.05 authorize landlord prohibition of \
             cannabis combustion + cultivation."
        }
        Jurisdiction::Colorado => {
            "CO Amendment 64 + § 12-43.4 Cannabis Code + HB 1287 authorize landlord \
             prohibition."
        }
        Jurisdiction::Washington => {
            "WA RCW 69.50.4014 authorizes landlord prohibition of cannabis on rental \
             property."
        }
        _ => {
            "State cannabis legalization statutes commonly preserve landlord authority to \
             prohibit cannabis use on premises."
        }
    }
}

fn state_disclosure_citation(jurisdiction: Jurisdiction) -> &'static str {
    match jurisdiction {
        Jurisdiction::California => {
            "CA SB 332 (Cal. Civ. Code § 1947.5) requires lease disclosure of smoke-free \
             policy + designated-smoking-areas; effective Jan 1, 2012."
        }
        Jurisdiction::NewYorkCity => {
            "NYC Local Law 147 of 2017 + NYC Admin. Code § 17-505 et seq. require \
             smoking-policy disclosure for multi-unit residential buildings."
        }
        Jurisdiction::Washington => {
            "WA RCW 70.160 Clean Indoor Air Act + WA RLTA disclosure norms."
        }
        Jurisdiction::Illinois => {
            "IL Smoke Free Illinois Act 410 ILCS 82 + IL landlord-tenant common-law \
             disclosure norms."
        }
        Jurisdiction::Oregon => {
            "OR Smoke-Free Workplace Law ORS 433.835-990 + OR landlord-tenant disclosure \
             norms."
        }
        _ => {
            "State smoke-free policy + lease-disclosure framework varies by jurisdiction; \
             verify with local counsel."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_ca() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            substance_category: SubstanceCategory::LitTobaccoCigarettesOrCigarOrPipe,
            lease_disclosure_status:
                LeaseDisclosureStatus::DisclosedInLeaseWithDesignatedAreas,
            tenancy_status: TenancyStatus::NewTenancyPostPolicyImplementation,
        }
    }

    #[test]
    fn ca_compliant_smoke_free_policy_enforceable() {
        let input = base_ca();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantSmokeFreePolicyEnforceable
        );
        assert!(output.policy_enforceable);
        assert!(output.note.contains("SB 332"));
        assert!(output.note.contains("§ 1947.5"));
    }

    #[test]
    fn ca_pre_existing_tenant_grandfathered_unenforceable() {
        let mut input = base_ca();
        input.tenancy_status = TenancyStatus::PreExistingTenantGrandfathered;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::PreExistingTenantGrandfatheredPolicyUnenforceable
        );
        assert!(!output.policy_enforceable);
    }

    #[test]
    fn ca_designated_areas_missing_defective() {
        let mut input = base_ca();
        input.lease_disclosure_status =
            LeaseDisclosureStatus::DisclosedButDesignatedAreasMissing;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::LeaseDisclosureDefectivePolicyUnenforceable
        );
        assert!(output.note.contains("§ 1947.5(b)"));
    }

    #[test]
    fn ca_no_disclosure_unenforceable() {
        let mut input = base_ca();
        input.lease_disclosure_status = LeaseDisclosureStatus::NoDisclosureProvided;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoDisclosureSmokeFreePolicyUnenforceable
        );
        assert!(output.note.contains("§ 1947.5(a)"));
    }

    #[test]
    fn hud_public_housing_federal_mandate() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::HudSubsidizedPublicHousing;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::HudFederalMandatePreemptsLocalLawSubsidizedHousing
        );
        assert!(output.policy_enforceable);
        assert!(output.note.contains("24 C.F.R. Parts 200 + 982 + 5"));
        assert!(output.note.contains("25 feet"));
        assert!(output.note.contains("Controlled Substances Act"));
    }

    #[test]
    fn cannabis_edibles_not_subject_to_smoke_free_policy() {
        let mut input = base_ca();
        input.substance_category = SubstanceCategory::CannabisEdiblesOrNonSmoked;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CannabisEdiblesNotSubjectToSmokeFreePolicy
        );
        assert!(!output.policy_enforceable);
    }

    #[test]
    fn cannabis_combustion_prohibitable_by_landlord() {
        let mut input = base_ca();
        input.substance_category = SubstanceCategory::CannabisCombustion;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::LandlordMayProhibitCannabisOnPremises
        );
        assert!(output.policy_enforceable);
        assert!(output.note.contains("§ 11362.45(h)"));
        assert!(output.note.contains("Forest City"));
        assert!(output.note.contains("James v. City of Costa Mesa"));
    }

    #[test]
    fn cannabis_vaporization_prohibitable_by_landlord() {
        let mut input = base_ca();
        input.substance_category = SubstanceCategory::CannabisVaporization;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::LandlordMayProhibitCannabisOnPremises
        );
    }

    #[test]
    fn ny_cannabis_prohibition_pins_mrta() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewYorkCity;
        input.substance_category = SubstanceCategory::CannabisCombustion;
        let output = check(&input);
        assert!(output.note.contains("§ 222.05"));
    }

    #[test]
    fn co_cannabis_prohibition_pins_amendment_64() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Colorado;
        input.substance_category = SubstanceCategory::CannabisCombustion;
        let output = check(&input);
        assert!(output.note.contains("Amendment 64"));
    }

    #[test]
    fn wa_cannabis_prohibition_pins_69_50_4014() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Washington;
        input.substance_category = SubstanceCategory::CannabisCombustion;
        let output = check(&input);
        assert!(output.note.contains("RCW 69.50.4014"));
    }

    #[test]
    fn nyc_disclosure_pins_local_law_147_2017() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewYorkCity;
        let output = check(&input);
        assert!(output.note.contains("Local Law 147"));
        assert!(output.note.contains("§ 17-505"));
    }

    #[test]
    fn wa_disclosure_pins_clean_indoor_air_act() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Washington;
        let output = check(&input);
        assert!(output.note.contains("Clean Indoor Air Act"));
        assert!(output.note.contains("RCW 70.160"));
    }

    #[test]
    fn il_disclosure_pins_smoke_free_illinois_act() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Illinois;
        let output = check(&input);
        assert!(output.note.contains("410 ILCS 82"));
    }

    #[test]
    fn or_disclosure_pins_ors_433() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Oregon;
        let output = check(&input);
        assert!(output.note.contains("ORS 433.835-990"));
    }

    #[test]
    fn electronic_cigarette_vaping_treated_as_smoke_free_subject() {
        let mut input = base_ca();
        input.substance_category = SubstanceCategory::ElectronicCigaretteVaping;
        let output = check(&input);
        // Vaping subject to smoke-free policy under modern interpretation
        assert_eq!(
            output.severity,
            Severity::CompliantSmokeFreePolicyEnforceable
        );
    }

    #[test]
    fn hud_smoke_free_rule_effective_year_constant_pins_2017() {
        assert_eq!(HUD_SMOKE_FREE_RULE_EFFECTIVE_YEAR, 2017);
    }

    #[test]
    fn ca_sb_332_effective_year_constant_pins_2012() {
        assert_eq!(CA_SB_332_EFFECTIVE_YEAR, 2012);
    }

    #[test]
    fn hud_takes_priority_over_all_other_branches() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::HudSubsidizedPublicHousing;
        input.lease_disclosure_status = LeaseDisclosureStatus::NoDisclosureProvided;
        let output = check(&input);
        // HUD federal mandate dispositive
        assert_eq!(
            output.severity,
            Severity::HudFederalMandatePreemptsLocalLawSubsidizedHousing
        );
    }

    #[test]
    fn cannabis_edibles_overrides_all_other_branches() {
        let mut input = base_ca();
        input.substance_category = SubstanceCategory::CannabisEdiblesOrNonSmoked;
        input.lease_disclosure_status = LeaseDisclosureStatus::NoDisclosureProvided;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CannabisEdiblesNotSubjectToSmokeFreePolicy
        );
    }
}
