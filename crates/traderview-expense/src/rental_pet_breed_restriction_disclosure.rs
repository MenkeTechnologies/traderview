//! Rental pet breed restriction disclosure framework — covers landlord
//! enforcement of pet-breed restrictions in residential leases, FHA
//! reasonable-accommodation exception for assistance animals (service +
//! emotional support), state preemption of breed-specific landlord
//! restrictions, insurance breed-ban interaction, and disclosure
//! requirements.
//!
//! Distinct from sibling `rental_pet_deposit_separate_security` (pet
//! deposit framework — distinct from breed restriction), [[fair_housing_
//! reasonable_modification]] (broader FHA modification framework), [[tenant_
//! emotional_distress_damages]] (IIED claim for wrongful denial of
//! assistance animal).
//!
//! Trader-landlord critical because (1) **Fair Housing Act 42 U.S.C. §
//! 3604(f)(3)(B)** + 24 C.F.R. § 100.204 require landlord reasonable
//! accommodation of assistance animals (service animals + emotional support
//! animals) and PREEMPT lease pet-breed restrictions; HUD Notice FHEO-2020-
//! 01 (January 28, 2020) provides assistance-animal assessment framework;
//! (2) **HUD interpretation**: "housing providers may not limit the breed
//! or size of a dog used as a service animal or support animal just because
//! of the size or breed" — must look at actual animal behavior not breed
//! stereotypes; (3) **insurance carrier breed bans** (Liberty Mutual +
//! State Farm + Allstate + Farmers + Nationwide exclude pit bulls +
//! Rottweilers + Akitas + Dobermans + Chow Chows + German Shepherds +
//! Wolf hybrids) CANNOT justify denying assistance animal per FHA — federal
//! disability accommodation preempts insurance coverage exclusion; (4)
//! **Nevada 2025 law**: SB 245 prohibits landlord liability insurance
//! breed discrimination; (5) **Maryland 2025 Pet Policy Transparency Act**:
//! requires landlords to post pet policies including breeds + weight limits
//! plus vaccination requirements plus pet deposits on rental websites plus
//! rental applications; (6) state preemption status varies (29 states
//! preempt local breed-specific legislation; 21 states permit local BSL
//! such as MD, MI, OH, KS); (7) CDC reports approximately 4.5M dog bites
//! annually in US with about 800K requiring medical care; landlord premises
//! liability for tenant or guest bite injuries routinely $50K to $500K
//! settlement exposure.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    /// Maryland — Pet Policy Transparency Act 2025 + no state preemption.
    Maryland,
    /// Michigan — no state preemption + local BSL permitted.
    Michigan,
    /// Nevada — SB 245 (2025) prohibits insurance breed discrimination.
    Nevada,
    /// California — Cal. Civ. Code § 1942.7 limits dog restrictions for
    /// service/assistance animals.
    California,
    /// New York — NY Civ. Rights Law § 47-a service animal protection.
    NewYork,
    /// Default — federal FHA + state-specific.
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimalType {
    /// Service animal under ADA Title III — task-trained for disability.
    AdaServiceAnimal,
    /// Emotional support animal under FHA — alleviates disability symptoms.
    FhaEmotionalSupportAnimal,
    /// Regular pet — no FHA / ADA protection.
    RegularPetNoFhaProtection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BreedRestrictionType {
    /// No breed restriction in lease.
    NoBreedRestriction,
    /// Specific breeds prohibited (pit bull + Rottweiler + Akita + etc.).
    SpecificBreedsProhibited,
    /// Weight or size restriction.
    WeightOrSizeRestriction,
    /// Total pet prohibition.
    TotalPetProhibition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    BreedRestrictionEnforceableNonAssistance,
    FhaPreemptsBreedRestrictionAssistanceAnimal,
    InsuranceBreedBanIneffectiveVsFha,
    MdPetPolicyTransparencyActDisclosureRequired,
    NvSb245InsuranceBreedDiscriminationProhibited,
    DocumentationRequestedFromTenantPermissible,
    DiscriminatoryDenialFhaViolationFineExposure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub animal_type: AnimalType,
    pub breed_restriction_type: BreedRestrictionType,
    pub tenant_provided_qualifying_documentation: bool,
    pub landlord_denied_accommodation_based_on_breed: bool,
    pub insurance_breed_ban_cited_as_justification: bool,
    pub pet_policy_posted_on_rental_website: bool,
    pub annual_rent_cents: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub recommended_actions: Vec<String>,
    pub annual_rent_at_risk_cents: u64,
    pub hud_civil_penalty_first_offense_cents: u64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const HUD_FIRST_OFFENSE_CIVIL_PENALTY_CENTS: u64 = 2_564_500;
pub const HUD_SUBSEQUENT_OFFENSE_PENALTY_CENTS: u64 = 12_822_500;
pub const STATES_WITH_BSL_PREEMPTION: u32 = 29;
pub const CDC_ANNUAL_DOG_BITES_MILLIONS: u32 = 4_500_000;
pub const MD_PET_POLICY_TRANSPARENCY_ACT_YEAR: i32 = 2025;
pub const NV_SB_245_EFFECTIVE_YEAR: i32 = 2025;
pub const HUD_NOTICE_FHEO_2020_01_DATE: &str = "2020-01-28";

pub fn check(input: &Input) -> Output {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let severity: Severity;

    if matches!(
        input.breed_restriction_type,
        BreedRestrictionType::NoBreedRestriction
    ) && !input.landlord_denied_accommodation_based_on_breed
    {
        notes.push(
            "No breed restriction enforced — framework inapplicable for current rental \
             interaction. Recommend disclosure of pet policy on rental website if Maryland \
             jurisdiction (Pet Policy Transparency Act 2025); Nevada landlords should review \
             liability insurance breed-exclusion clauses for SB 245 (2025) compliance."
                .to_string(),
        );
        return Output {
            severity: Severity::NotApplicable,
            recommended_actions: actions,
            annual_rent_at_risk_cents: 0,
            hud_civil_penalty_first_offense_cents: 0,
            citation: "n/a (no breed restriction)",
            notes,
        };
    }

    if input.landlord_denied_accommodation_based_on_breed
        && !matches!(input.animal_type, AnimalType::RegularPetNoFhaProtection)
        && input.tenant_provided_qualifying_documentation
    {
        severity = Severity::DiscriminatoryDenialFhaViolationFineExposure;
        actions.push(format!(
            "FHA discriminatory denial: landlord denied accommodation of qualifying \
             assistance animal ({:?}) based on breed despite tenant-provided qualifying \
             documentation. Federal civil penalty up to ${} first offense + ${} subsequent \
             offense per 24 C.F.R. § 180.671 + 28 C.F.R. § 85.5. HUD complaint may be filed \
             within 1 year per 42 U.S.C. § 3610. Compensatory damages + statutory damages + \
             attorney's fees plus injunctive relief available. IMMEDIATELY grant \
             accommodation; document denial-reversal; consult fair-housing counsel.",
            input.animal_type,
            HUD_FIRST_OFFENSE_CIVIL_PENALTY_CENTS / 100,
            HUD_SUBSEQUENT_OFFENSE_PENALTY_CENTS / 100
        ));
    } else if input.insurance_breed_ban_cited_as_justification
        && matches!(
            input.animal_type,
            AnimalType::AdaServiceAnimal | AnimalType::FhaEmotionalSupportAnimal
        )
    {
        severity = Severity::InsuranceBreedBanIneffectiveVsFha;
        actions.push(
            "Insurance breed-exclusion ban (Liberty Mutual + State Farm + Allstate + \
             Farmers + Nationwide commonly exclude pit bull + Rottweiler + Akita + Doberman \
             + Chow Chow + German Shepherd + Wolf hybrids) CANNOT justify denying \
             qualifying assistance animal — federal disability accommodation preempts \
             insurance coverage exclusion per multiple federal court decisions. Switch \
             insurance carrier to non-breed-exclusion policy (USAA + many specialty pet-\
             friendly carriers offer broader coverage); document carrier change in landlord \
             file; preserve fair-housing compliance."
                .to_string(),
        );
    } else if matches!(
        input.animal_type,
        AnimalType::AdaServiceAnimal | AnimalType::FhaEmotionalSupportAnimal
    ) && input.tenant_provided_qualifying_documentation
    {
        severity = Severity::FhaPreemptsBreedRestrictionAssistanceAnimal;
        actions.push(format!(
            "FHA 42 U.S.C. § 3604(f)(3)(B) + 24 C.F.R. § 100.204 + HUD Notice FHEO-2020-01 \
             ({}) PREEMPT lease breed restriction for qualifying {:?}. HUD guidance: \
             'housing providers may not limit the breed or size of a dog used as a service \
             animal or support animal just because of the size or breed.' Grant accommodation \
             plus document tenant-supplied qualifying documentation plus accommodation \
             approval in landlord file plus tenant-acknowledgment receipt.",
            HUD_NOTICE_FHEO_2020_01_DATE, input.animal_type
        ));
    } else if matches!(
        input.animal_type,
        AnimalType::AdaServiceAnimal | AnimalType::FhaEmotionalSupportAnimal
    ) && !input.tenant_provided_qualifying_documentation
    {
        severity = Severity::DocumentationRequestedFromTenantPermissible;
        actions.push(
            "Tenant claims assistance-animal status without supplying qualifying \
             documentation. Landlord MAY request: (1) for service animals — only two \
             questions per ADA Title III: 'Is the dog required because of a disability?' \
             plus 'What work or task has the dog been trained to perform?' — NO documentation \
             of training or certification may be required; (2) for emotional support animals \
             — written letter from licensed mental-health provider establishing tenant \
             disability and animal-disability nexus per HUD guidance. Document request plus \
             tenant response in writing."
                .to_string(),
        );
    } else if matches!(input.jurisdiction, Jurisdiction::Maryland)
        && !input.pet_policy_posted_on_rental_website
    {
        severity = Severity::MdPetPolicyTransparencyActDisclosureRequired;
        actions.push(format!(
            "Maryland Pet Policy Transparency Act of {} requires landlords to post pet \
             policies on rental websites and rental applications including: (1) number of \
             pets permitted, (2) weight limits, (3) restricted breeds, (4) vaccination \
             requirements, (5) pet deposits, (6) 'pet rent' charges. Post complete policy \
             plus update rental advertisement; document compliance via screenshot retention.",
            MD_PET_POLICY_TRANSPARENCY_ACT_YEAR
        ));
    } else if matches!(input.jurisdiction, Jurisdiction::Nevada)
        && input.insurance_breed_ban_cited_as_justification
    {
        severity = Severity::NvSb245InsuranceBreedDiscriminationProhibited;
        actions.push(format!(
            "Nevada SB 245 effective {} PROHIBITS landlord liability insurance breed \
             discrimination — insurance carrier cannot exclude coverage based on dog breed; \
             landlord cannot cite insurance breed ban as justification for breed restriction. \
             Switch carrier to SB 245-compliant policy; preserve documentation of \
             enforcement complaint to Nevada Division of Insurance.",
            NV_SB_245_EFFECTIVE_YEAR
        ));
    } else if matches!(input.animal_type, AnimalType::RegularPetNoFhaProtection)
        && matches!(
            input.breed_restriction_type,
            BreedRestrictionType::SpecificBreedsProhibited
        )
    {
        severity = Severity::BreedRestrictionEnforceableNonAssistance;
        actions.push(
            "Lease breed restriction enforceable for non-assistance animal under common-law \
             contract freedom + state premises-liability law. CDC reports approximately 4.5M \
             dog bites annually in US with 800K requiring medical care; landlord premises-\
             liability exposure for tenant or guest bite injuries routinely $50K-$500K \
             settlement exposure. Verify enforcement consistent across tenants to avoid \
             pretext discrimination claim."
                .to_string(),
        );
    } else {
        severity = Severity::NotApplicable;
        actions.push(
            "No actionable framework outcome under current facts; document interaction in \
             landlord file."
                .to_string(),
        );
    }

    match input.jurisdiction {
        Jurisdiction::Maryland => {
            notes.push(format!(
                "Maryland Pet Policy Transparency Act of {} requires landlords to post pet \
                 policies on rental websites and rental applications. Maryland has NO state \
                 preemption of local breed-specific legislation per DogsBite.org state \
                 preemption map — 21 states permit local BSL including Maryland. Federal FHA \
                 reasonable-accommodation supersedes state BSL for assistance animals.",
                MD_PET_POLICY_TRANSPARENCY_ACT_YEAR
            ));
        }
        Jurisdiction::Michigan => {
            notes.push(
                "Michigan has NO state preemption of local breed-specific legislation — \
                 local governments may impose BSL. Michigan disability accommodation \
                 protected by state law plus federal FHA. Michigan Civil Rights Commission \
                 enforces."
                    .to_string(),
            );
        }
        Jurisdiction::Nevada => {
            notes.push(format!(
                "Nevada SB 245 effective {} PROHIBITS landlord liability insurance carriers \
                 from breed discrimination — insurance carrier cannot exclude coverage based \
                 on dog breed; landlord cannot cite insurance breed ban as justification for \
                 lease breed restriction. Nevada Division of Insurance enforces.",
                NV_SB_245_EFFECTIVE_YEAR
            ));
        }
        Jurisdiction::California => {
            notes.push(
                "California Cal. Civ. Code § 1942.7 limits dog restrictions for service / \
                 assistance animals; Cal. Civ. Code § 54.2 service-animal access; California \
                 Department of Fair Employment and Housing (DFEH) enforces. AB 468 effective \
                 2022 emotional support animal documentation requirements (30-day \
                 client/provider relationship)."
                    .to_string(),
            );
        }
        Jurisdiction::NewYork => {
            notes.push(
                "NY Civ. Rights Law § 47-a service animal protection plus NY Real Property \
                 Law § 235-b implied warranty of habitability extends to FHA accommodation. \
                 NYC Admin Code § 8-107(15) plus NYS Human Rights Law § 296(2-a) prohibit \
                 housing discrimination based on disability including assistance-animal \
                 denial. NYC HPD plus NY Division of Human Rights enforce."
                    .to_string(),
            );
        }
        Jurisdiction::Default => {
            notes.push(format!(
                "Federal Fair Housing Act 42 U.S.C. § 3604(f)(3)(B) + 24 C.F.R. § 100.204 + \
                 HUD Notice FHEO-2020-01 ({}) govern. State preemption of local breed-\
                 specific legislation varies: {} states preempt local BSL; 21 states permit \
                 local BSL (including MD + MI + OH + KS). CDC reports approximately {} dog \
                 bites annually in US with 800K requiring medical care.",
                HUD_NOTICE_FHEO_2020_01_DATE,
                STATES_WITH_BSL_PREEMPTION,
                CDC_ANNUAL_DOG_BITES_MILLIONS
            ));
        }
    }

    notes.push(
        "Coordination with [[rental_pet_deposit_separate_security]] (pet-deposit framework — \
         distinct from breed restriction; FHA bars pet-deposit requirement for assistance \
         animals), [[fair_housing_reasonable_modification]] (broader FHA modification \
         framework), [[tenant_emotional_distress_damages]] (IIED claim for wrongful denial \
         of assistance-animal request), [[rental_attached_garage_carbon_monoxide_disclosure]] \
         (iter 523 — parallel landlord-disclosure framework), [[rental_storage_unit_lease_\
         disclosure]] (iter 509 — separate disclosure framework analog)."
            .to_string(),
    );

    let annual_rent_at_risk: u64 = match severity {
        Severity::DiscriminatoryDenialFhaViolationFineExposure => input.annual_rent_cents,
        Severity::InsuranceBreedBanIneffectiveVsFha
        | Severity::FhaPreemptsBreedRestrictionAssistanceAnimal
        | Severity::MdPetPolicyTransparencyActDisclosureRequired
        | Severity::NvSb245InsuranceBreedDiscriminationProhibited => {
            input.annual_rent_cents.saturating_div(2)
        }
        _ => 0,
    };

    Output {
        severity,
        recommended_actions: actions,
        annual_rent_at_risk_cents: annual_rent_at_risk,
        hud_civil_penalty_first_offense_cents: HUD_FIRST_OFFENSE_CIVIL_PENALTY_CENTS,
        citation: match input.jurisdiction {
            Jurisdiction::Maryland => {
                "Md. Pet Policy Transparency Act 2025 + FHA 42 U.S.C. § 3604(f)(3)(B)"
            }
            Jurisdiction::Michigan => "Mich. Civ. Rights Commission + FHA + state-law disability",
            Jurisdiction::Nevada => "Nev. SB 245 (2025) + FHA + Nev. Div. of Insurance",
            Jurisdiction::California => "Cal. Civ. Code § 1942.7 + § 54.2 + DFEH + AB 468",
            Jurisdiction::NewYork => {
                "NY Civ. Rights Law § 47-a + NY RPL § 235-b + NYC Admin § 8-107(15)"
            }
            Jurisdiction::Default => {
                "42 U.S.C. § 3604(f)(3)(B) FHA + 24 C.F.R. § 100.204 + HUD FHEO-2020-01"
            }
        },
        notes,
    }
}

pub type RentalPetBreedRestrictionDisclosureInput = Input;
pub type RentalPetBreedRestrictionDisclosureResult = Output;

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            animal_type: AnimalType::RegularPetNoFhaProtection,
            breed_restriction_type: BreedRestrictionType::SpecificBreedsProhibited,
            tenant_provided_qualifying_documentation: false,
            landlord_denied_accommodation_based_on_breed: false,
            insurance_breed_ban_cited_as_justification: false,
            pet_policy_posted_on_rental_website: true,
            annual_rent_cents: 36_000_00,
        }
    }

    #[test]
    fn no_breed_restriction_not_applicable() {
        let mut i = baseline();
        i.breed_restriction_type = BreedRestrictionType::NoBreedRestriction;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotApplicable));
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }

    #[test]
    fn breed_restriction_enforceable_for_regular_pet() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::BreedRestrictionEnforceableNonAssistance
        ));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("4.5M dog bites")));
    }

    #[test]
    fn fha_preempts_breed_restriction_for_assistance_animal() {
        let mut i = baseline();
        i.animal_type = AnimalType::FhaEmotionalSupportAnimal;
        i.tenant_provided_qualifying_documentation = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::FhaPreemptsBreedRestrictionAssistanceAnimal
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents / 2);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("§ 3604(f)(3)(B)")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("FHEO-2020-01")));
    }

    #[test]
    fn discriminatory_denial_fha_violation_full_rent_at_risk() {
        let mut i = baseline();
        i.animal_type = AnimalType::AdaServiceAnimal;
        i.tenant_provided_qualifying_documentation = true;
        i.landlord_denied_accommodation_based_on_breed = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::DiscriminatoryDenialFhaViolationFineExposure
        ));
        assert_eq!(r.annual_rent_at_risk_cents, i.annual_rent_cents);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("24 C.F.R. § 180.671")));
        assert!(r.recommended_actions.iter().any(|a| a.contains("§ 3610")));
    }

    #[test]
    fn documentation_requested_permissible_without_qualifying_docs() {
        let mut i = baseline();
        i.animal_type = AnimalType::AdaServiceAnimal;
        i.tenant_provided_qualifying_documentation = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::DocumentationRequestedFromTenantPermissible
        ));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("two questions")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("ADA Title III")));
    }

    #[test]
    fn insurance_breed_ban_ineffective_vs_fha() {
        let mut i = baseline();
        i.animal_type = AnimalType::FhaEmotionalSupportAnimal;
        i.insurance_breed_ban_cited_as_justification = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::InsuranceBreedBanIneffectiveVsFha
        ));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Liberty Mutual")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("State Farm")));
    }

    #[test]
    fn maryland_pet_policy_transparency_act_disclosure_required() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Maryland;
        i.pet_policy_posted_on_rental_website = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::MdPetPolicyTransparencyActDisclosureRequired
        ));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("vaccination requirements")));
    }

    #[test]
    fn nevada_sb_245_insurance_breed_discrimination_prohibited() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Nevada;
        i.insurance_breed_ban_cited_as_justification = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::NvSb245InsuranceBreedDiscriminationProhibited
        ));
        assert!(r.recommended_actions.iter().any(|a| a.contains("SB 245")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Nevada Division of Insurance")));
    }

    #[test]
    fn ma_jurisdiction_pins_pet_policy_transparency_act() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Maryland;
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Pet Policy Transparency Act")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("21 states permit local BSL")));
    }

    #[test]
    fn mi_jurisdiction_pins_no_state_preemption() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Michigan;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("NO state preemption")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Michigan Civil Rights Commission")));
    }

    #[test]
    fn nv_jurisdiction_pins_sb_245_and_division_of_insurance() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Nevada;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("Nevada SB 245")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Nevada Division of Insurance")));
    }

    #[test]
    fn ca_jurisdiction_pins_civ_code_1942_7_and_dfeh() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 1942.7")));
        assert!(r.notes.iter().any(|n| n.contains("DFEH")));
        assert!(r.notes.iter().any(|n| n.contains("AB 468")));
    }

    #[test]
    fn ny_jurisdiction_pins_civ_rights_law_47_a() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::NewYork;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 47-a")));
        assert!(r.notes.iter().any(|n| n.contains("§ 8-107(15)")));
        assert!(r.notes.iter().any(|n| n.contains("§ 296(2-a)")));
    }

    #[test]
    fn default_jurisdiction_pins_fha_and_cdc_dog_bites() {
        let mut i = baseline();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 3604(f)(3)(B)")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("HUD Notice FHEO-2020-01")));
        assert!(r.notes.iter().any(|n| n.contains("4500000")));
    }

    #[test]
    fn coordination_note_references_siblings() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_pet_deposit_separate_security")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rental_attached_garage_carbon_monoxide_disclosure")));
    }

    #[test]
    fn hud_first_offense_penalty_pins_25645() {
        assert_eq!(HUD_FIRST_OFFENSE_CIVIL_PENALTY_CENTS, 2_564_500);
    }

    #[test]
    fn hud_subsequent_offense_penalty_pins_128225() {
        assert_eq!(HUD_SUBSEQUENT_OFFENSE_PENALTY_CENTS, 12_822_500);
    }

    #[test]
    fn states_with_bsl_preemption_pins_29() {
        assert_eq!(STATES_WITH_BSL_PREEMPTION, 29);
    }

    #[test]
    fn cdc_annual_dog_bites_pins_4_5_million() {
        assert_eq!(CDC_ANNUAL_DOG_BITES_MILLIONS, 4_500_000);
    }

    #[test]
    fn md_pet_policy_transparency_act_year_pins_2025() {
        assert_eq!(MD_PET_POLICY_TRANSPARENCY_ACT_YEAR, 2025);
    }

    #[test]
    fn nv_sb_245_effective_year_pins_2025() {
        assert_eq!(NV_SB_245_EFFECTIVE_YEAR, 2025);
    }

    #[test]
    fn hud_notice_fheo_2020_01_date_pins_2020_01_28() {
        assert_eq!(HUD_NOTICE_FHEO_2020_01_DATE, "2020-01-28");
    }

    #[test]
    fn citation_branch_for_each_jurisdiction() {
        let md = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::Maryland;
            i
        });
        let mi = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::Michigan;
            i
        });
        let nv = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::Nevada;
            i
        });
        let ca = check(&baseline());
        let ny = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::NewYork;
            i
        });
        let de = check(&{
            let mut i = baseline();
            i.jurisdiction = Jurisdiction::Default;
            i
        });
        assert!(md.citation.contains("Pet Policy Transparency Act"));
        assert!(mi.citation.contains("Mich. Civ. Rights Commission"));
        assert!(nv.citation.contains("Nev. SB 245"));
        assert!(ca.citation.contains("Cal. Civ. Code"));
        assert!(ny.citation.contains("NY Civ. Rights Law"));
        assert!(de.citation.contains("FHA"));
    }

    #[test]
    fn severity_priority_discriminatory_denial_overrides_fha_preemption() {
        let mut i = baseline();
        i.animal_type = AnimalType::FhaEmotionalSupportAnimal;
        i.tenant_provided_qualifying_documentation = true;
        i.landlord_denied_accommodation_based_on_breed = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::DiscriminatoryDenialFhaViolationFineExposure
        ));
    }

    #[test]
    fn zero_rent_zero_risk_does_not_panic() {
        let mut i = baseline();
        i.annual_rent_cents = 0;
        i.animal_type = AnimalType::AdaServiceAnimal;
        i.tenant_provided_qualifying_documentation = true;
        i.landlord_denied_accommodation_based_on_breed = true;
        let r = check(&i);
        assert_eq!(r.annual_rent_at_risk_cents, 0);
    }
}
