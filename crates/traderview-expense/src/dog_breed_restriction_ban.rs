//! Dog breed-specific restriction (BSL) ban in residential
//! rental housing — when may a landlord lawfully refuse to
//! rent to a tenant purely because of the breed of the
//! tenant's dog? Trader-landlord critical for any multifamily
//! rental property considering breed restrictions: Nevada SB
//! 166 (effective October 1, 2025) prohibits insurance
//! underwriting using breed as a factor for landlord
//! liability policies for multifamily residential
//! properties; combined with Nevada's 2013 statewide BSL
//! preemption, the practical effect is that landlords
//! generally cannot refuse to rent based on breed alone.
//!
//! Distinct from siblings `pet_fees` (pet deposit caps),
//! `emotional_support_animal_documentation` (ESA framework),
//! `service_animal` (ADA / FHA service-animal protection),
//! `tenant_organizing`, and `fair_chance_housing` (criminal-
//! background screening).
//!
//! **Two regimes**:
//!
//! **Nevada — SB 166 (eff. October 1, 2025) + SB 103 (2021)
//! + 2013 BSL preemption statute**:
//! - SB 166 — insurers MAY NOT use dog breed as a factor
//!   when underwriting landlord liability policies for
//!   multi-family residential properties.
//! - SB 103 — insurers MAY NOT deny homeowners or renters
//!   coverage based SOLELY on dog breed.
//! - 2013 statewide BSL preemption — local governments
//!   cannot pass laws banning specific breeds (e.g., pit
//!   bulls). Nevada was 14th state to prohibit BSL by local
//!   governments.
//! - Landlords may still require: (a) pet deposits within
//!   security-deposit cap; (b) liability coverage; (c)
//!   behavior screenings (history of bites, training, dog
//!   temperament evaluation). LANDLORDS MAY NOT refuse to
//!   rent purely because of breed.
//!
//! **Default — no statewide breed discrimination
//! protection**. ~22 states have passed BSL preemption laws
//! at local-government level (varies by state); many states
//! still allow breed-specific bans at city/county level;
//! insurance may still consider breed in most states. Some
//! municipalities have enacted breed discrimination bans
//! (Maryland counties + Massachusetts municipalities).
//!
//! Citations: NV SB 166 (eff. October 1, 2025); NV SB 103
//! (2021); NV Rev. Stat. § 202.500 (statewide BSL
//! preemption); local-government BSL preemption statutes in
//! ~22 states.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Nevada,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DenialBasis {
    /// Refusal purely because of breed (PROHIBITED in NV).
    BreedAlone,
    /// Refusal based on insurance underwriting using breed
    /// (PROHIBITED in NV under SB 166).
    InsuranceBreedUnderwriting,
    /// Refusal based on behavior screening (history of
    /// bites, aggression, training) — LAWFUL.
    BehaviorScreening,
    /// Refusal based on tenant declining liability coverage
    /// requirement — LAWFUL.
    DeclinedLiabilityCoverage,
    /// Refusal based on tenant declining pet deposit within
    /// statutory cap — LAWFUL.
    DeclinedPetDeposit,
    /// Local-government breed-specific legislation (BSL)
    /// banning the breed — PROHIBITED IN NV under 2013
    /// preemption.
    LocalGovernmentBsl,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DogBreedRestrictionBanInput {
    pub regime: Regime,
    pub denial_basis: DenialBasis,
    /// Whether the property is a multifamily residential
    /// property (SB 166 scope for insurance prohibition).
    pub multifamily_residential: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DogBreedRestrictionBanResult {
    pub denial_lawful: bool,
    pub statewide_protection_engaged: bool,
    pub insurance_breed_underwriting_prohibited: bool,
    pub local_bsl_preempted: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &DogBreedRestrictionBanInput) -> DogBreedRestrictionBanResult {
    match input.regime {
        Regime::Nevada => check_nevada(input),
        Regime::Default => check_default(input),
    }
}

fn check_nevada(input: &DogBreedRestrictionBanInput) -> DogBreedRestrictionBanResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "NV SB 166 (effective October 1, 2025) — insurers MAY NOT use dog breed as a factor when underwriting landlord liability policies for multi-family residential properties"
            .to_string(),
        "NV SB 103 (2021) — insurers MAY NOT deny homeowners or renters coverage based SOLELY on dog breed"
            .to_string(),
        "NV Rev. Stat. § 202.500 (2013 statewide BSL preemption; Nevada was 14th state) — local governments CANNOT pass laws banning specific breeds (e.g., pit bulls)"
            .to_string(),
        "NV landlords MAY require: (a) pet deposits within security-deposit cap; (b) liability coverage; (c) behavior screenings (history of bites, training, temperament evaluation); but MAY NOT refuse to rent purely because of breed"
            .to_string(),
    ];

    match input.denial_basis {
        DenialBasis::BreedAlone => {
            violations.push(
                "NV SB 166 + SB 103 + § 202.500 — landlord MAY NOT refuse to rent purely because of dog breed; breed alone is not a lawful basis for denial".to_string(),
            );
        }
        DenialBasis::InsuranceBreedUnderwriting => {
            if input.multifamily_residential {
                violations.push(
                    "NV SB 166 (eff. October 1, 2025) — insurers MAY NOT use dog breed as a factor when underwriting landlord liability policies for multi-family residential properties; denial based on this is unlawful".to_string(),
                );
            } else {
                violations.push(
                    "NV SB 103 (2021) — insurers MAY NOT deny homeowners or renters coverage based SOLELY on dog breed".to_string(),
                );
            }
        }
        DenialBasis::LocalGovernmentBsl => {
            violations.push(
                "NV Rev. Stat. § 202.500 (2013 statewide BSL preemption) — local-government breed-specific legislation (BSL) is preempted; landlord cannot rely on local BSL to refuse tenancy".to_string(),
            );
        }
        DenialBasis::BehaviorScreening
        | DenialBasis::DeclinedLiabilityCoverage
        | DenialBasis::DeclinedPetDeposit => {}
    }

    DogBreedRestrictionBanResult {
        denial_lawful: violations.is_empty(),
        statewide_protection_engaged: true,
        insurance_breed_underwriting_prohibited: true,
        local_bsl_preempted: true,
        violations,
        citation: "NV SB 166 (eff. October 1, 2025); NV SB 103 (2021); NV Rev. Stat. § 202.500 (2013 BSL preemption)",
        notes,
    }
}

fn check_default(_input: &DogBreedRestrictionBanInput) -> DogBreedRestrictionBanResult {
    let notes: Vec<String> = vec![
        "default rule — no statewide breed discrimination protection; ~22 states have passed BSL preemption laws at local-government level (varies by state); many states still allow breed-specific bans at city/county level; insurance may still consider breed in most states"
            .to_string(),
        "default rule — some municipalities have enacted breed discrimination bans (Maryland counties + Massachusetts municipalities); FHA does NOT cover breed unless tied to disability service-animal accommodation"
            .to_string(),
    ];

    DogBreedRestrictionBanResult {
        denial_lawful: true,
        statewide_protection_engaged: false,
        insurance_breed_underwriting_prohibited: false,
        local_bsl_preempted: false,
        violations: Vec::new(),
        citation: "no statewide statute; local-government BSL preemption varies by state; FHA does not cover breed",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nv_breed_alone() -> DogBreedRestrictionBanInput {
        DogBreedRestrictionBanInput {
            regime: Regime::Nevada,
            denial_basis: DenialBasis::BreedAlone,
            multifamily_residential: true,
        }
    }

    fn nv_behavior_screening() -> DogBreedRestrictionBanInput {
        DogBreedRestrictionBanInput {
            regime: Regime::Nevada,
            denial_basis: DenialBasis::BehaviorScreening,
            multifamily_residential: true,
        }
    }

    fn default_base() -> DogBreedRestrictionBanInput {
        let mut i = nv_breed_alone();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn nv_breed_alone_denial_unlawful() {
        let r = check(&nv_breed_alone());
        assert!(!r.denial_lawful);
        assert!(r.statewide_protection_engaged);
        assert!(r.insurance_breed_underwriting_prohibited);
        assert!(r.local_bsl_preempted);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("SB 166") && v.contains("breed alone")));
    }

    #[test]
    fn nv_behavior_screening_lawful() {
        let r = check(&nv_behavior_screening());
        assert!(r.denial_lawful);
        assert!(r.statewide_protection_engaged);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn nv_declined_liability_coverage_lawful() {
        let mut i = nv_breed_alone();
        i.denial_basis = DenialBasis::DeclinedLiabilityCoverage;
        let r = check(&i);
        assert!(r.denial_lawful);
    }

    #[test]
    fn nv_declined_pet_deposit_lawful() {
        let mut i = nv_breed_alone();
        i.denial_basis = DenialBasis::DeclinedPetDeposit;
        let r = check(&i);
        assert!(r.denial_lawful);
    }

    #[test]
    fn nv_insurance_breed_underwriting_multifamily_violates_sb166() {
        let mut i = nv_breed_alone();
        i.denial_basis = DenialBasis::InsuranceBreedUnderwriting;
        i.multifamily_residential = true;
        let r = check(&i);
        assert!(!r.denial_lawful);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("SB 166") && v.contains("multi-family residential")));
    }

    #[test]
    fn nv_insurance_breed_underwriting_single_family_violates_sb103() {
        let mut i = nv_breed_alone();
        i.denial_basis = DenialBasis::InsuranceBreedUnderwriting;
        i.multifamily_residential = false;
        let r = check(&i);
        assert!(!r.denial_lawful);
        assert!(r.violations.iter().any(|v| v.contains("SB 103")));
    }

    #[test]
    fn nv_local_government_bsl_preempted() {
        let mut i = nv_breed_alone();
        i.denial_basis = DenialBasis::LocalGovernmentBsl;
        let r = check(&i);
        assert!(!r.denial_lawful);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 202.500") && v.contains("preempted")));
    }

    #[test]
    fn nv_citation_pins_three_authorities() {
        let r = check(&nv_breed_alone());
        assert!(r.citation.contains("SB 166"));
        assert!(r.citation.contains("October 1, 2025"));
        assert!(r.citation.contains("SB 103"));
        assert!(r.citation.contains("§ 202.500"));
        assert!(r.citation.contains("2013 BSL preemption"));
    }

    #[test]
    fn nv_note_pins_sb166() {
        let r = check(&nv_breed_alone());
        assert!(r.notes.iter().any(|n| n.contains("SB 166")
            && n.contains("October 1, 2025")
            && n.contains("multi-family residential")));
    }

    #[test]
    fn nv_note_pins_sb103() {
        let r = check(&nv_breed_alone());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("SB 103") && n.contains("(2021)") && n.contains("SOLELY")));
    }

    #[test]
    fn nv_note_pins_2013_bsl_preemption() {
        let r = check(&nv_breed_alone());
        assert!(r.notes.iter().any(|n| n.contains("§ 202.500")
            && n.contains("14th state")
            && n.contains("pit bulls")));
    }

    #[test]
    fn nv_note_pins_lawful_requirements() {
        let r = check(&nv_breed_alone());
        assert!(r.notes.iter().any(|n| n.contains("pet deposits")
            && n.contains("liability coverage")
            && n.contains("behavior screenings")
            && n.contains("MAY NOT refuse to rent purely because of breed")));
    }

    #[test]
    fn default_no_protection_breed_alone_lawful() {
        let r = check(&default_base());
        assert!(r.denial_lawful);
        assert!(!r.statewide_protection_engaged);
        assert!(!r.insurance_breed_underwriting_prohibited);
        assert!(!r.local_bsl_preempted);
    }

    #[test]
    fn default_citation_pins_no_statewide_statute() {
        let r = check(&default_base());
        assert!(r.citation.contains("no statewide statute"));
        assert!(r.citation.contains("varies by state"));
    }

    #[test]
    fn default_note_pins_22_state_landscape() {
        let r = check(&default_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("22 states") && n.contains("BSL preemption")));
    }

    #[test]
    fn default_note_pins_fha_carveout() {
        let r = check(&default_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("FHA") && n.contains("disability service-animal accommodation")));
    }

    #[test]
    fn two_regimes_routed_correctly() {
        for regime in [Regime::Nevada, Regime::Default] {
            let mut i = nv_breed_alone();
            i.regime = regime;
            let r = check(&i);
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn denial_basis_truth_table_nv() {
        for (basis, exp_lawful) in [
            (DenialBasis::BreedAlone, false),
            (DenialBasis::InsuranceBreedUnderwriting, false),
            (DenialBasis::LocalGovernmentBsl, false),
            (DenialBasis::BehaviorScreening, true),
            (DenialBasis::DeclinedLiabilityCoverage, true),
            (DenialBasis::DeclinedPetDeposit, true),
        ] {
            let mut i = nv_breed_alone();
            i.denial_basis = basis;
            let r = check(&i);
            assert_eq!(r.denial_lawful, exp_lawful);
        }
    }

    #[test]
    fn denial_basis_truth_table_default() {
        for basis in [
            DenialBasis::BreedAlone,
            DenialBasis::InsuranceBreedUnderwriting,
            DenialBasis::LocalGovernmentBsl,
            DenialBasis::BehaviorScreening,
            DenialBasis::DeclinedLiabilityCoverage,
            DenialBasis::DeclinedPetDeposit,
        ] {
            let mut i = default_base();
            i.denial_basis = basis;
            let r = check(&i);
            assert!(r.denial_lawful);
        }
    }

    #[test]
    fn nv_uniquely_engages_statewide_protection_invariant() {
        let r_nv = check(&nv_breed_alone());
        assert!(r_nv.statewide_protection_engaged);

        let r_default = check(&default_base());
        assert!(!r_default.statewide_protection_engaged);
    }

    #[test]
    fn nv_sb166_multifamily_specific_vs_sb103_general_distinguished() {
        let mut i_multi = nv_breed_alone();
        i_multi.denial_basis = DenialBasis::InsuranceBreedUnderwriting;
        i_multi.multifamily_residential = true;
        let r_multi = check(&i_multi);
        assert!(r_multi
            .violations
            .iter()
            .any(|v| v.contains("SB 166") && v.contains("multi-family residential")));

        let mut i_single = nv_breed_alone();
        i_single.denial_basis = DenialBasis::InsuranceBreedUnderwriting;
        i_single.multifamily_residential = false;
        let r_single = check(&i_single);
        assert!(r_single.violations.iter().any(|v| v.contains("SB 103")));
    }

    #[test]
    fn nv_three_lawful_pathways_unaffected_by_multifamily_status() {
        for basis in [
            DenialBasis::BehaviorScreening,
            DenialBasis::DeclinedLiabilityCoverage,
            DenialBasis::DeclinedPetDeposit,
        ] {
            for multi in [true, false] {
                let mut i = nv_breed_alone();
                i.denial_basis = basis;
                i.multifamily_residential = multi;
                let r = check(&i);
                assert!(r.denial_lawful);
            }
        }
    }
}
