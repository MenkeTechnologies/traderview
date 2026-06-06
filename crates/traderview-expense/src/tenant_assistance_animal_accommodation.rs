//! Multi-jurisdictional tenant assistance animal
//! accommodation framework — among the highest-stakes
//! landlord exposure regimes in residential landlord-
//! tenant law. Misclassifying an emotional support animal
//! as a "pet" and applying pet fees, weight restrictions,
//! breed restrictions, or refusing the accommodation is
//! the single most common Fair Housing Act discrimination
//! complaint received by HUD and state fair-housing
//! agencies. Trader-landlord critical because complaints
//! route to HUD administrative penalties (2026: $25,597
//! first offense / $63,993 second offense / $127,985 third
//! offense per 24 CFR § 30.65, annually inflation-
//! adjusted) PLUS DOJ Pattern-or-Practice exposure PLUS
//! private § 3613 actions with actual + PUNITIVE damages
//! plus attorney fees plus costs.
//!
//! Companion modules: rental_pet_deposit_separate_security
//! and tenant_data_privacy and fair_chance_housing and
//! landlord_self_help_eviction_prohibition.
//!
//! **Federal Fair Housing Act § 3604(f)(3)(B) (42 USC
//! § 3604(f)(3)(B))** — unlawful housing discrimination
//! for landlord to refuse to make reasonable
//! accommodations in rules, policies, practices, or
//! services when such accommodations may be necessary to
//! afford a person with a DISABILITY equal opportunity to
//! enjoy and use a dwelling. Includes refusal to permit
//! assistance animal in a no-pets building or to waive
//! pet fees/deposits/weight/breed restrictions.
//!
//! **HUD FHEO Notice 2020-01 (January 28, 2020)** —
//! replaced HUD's 2013 guidance with new framework.
//! Defines two types of assistance animals:
//! 1. **Service animals** — under ADA Title III, dogs (and
//!    in some cases miniature horses) individually trained
//!    to do work or perform tasks for individuals with
//!    disabilities. Includes guide dogs, hearing dogs,
//!    psychiatric service dogs, mobility-assistance dogs.
//! 2. **Support animals (emotional support animals)** —
//!    any species providing emotional support, comfort,
//!    well-being, or therapeutic benefit; need NOT be
//!    individually trained but must be necessary to ameliorate
//!    a disability symptom.
//!
//! **Pet fee and deposit prohibition under HUD FHEO
//! 2020-01** — housing providers may NOT exclude OR
//! charge a fee or deposit for assistance animals
//! because these animals serve an important function for
//! individuals with disabilities. Landlords may NOT
//! require an extra fee or additional security deposit
//! as a condition of granting a reasonable accommodation.
//! Tenant remains responsible for actual damage caused
//! by the animal (general security deposit recovery).
//!
//! **Documentation standards under HUD FHEO 2020-01** —
//! housing providers may request reliable supporting
//! documentation when:
//! 1. The disability is NON-OBVIOUS (not known); AND/OR
//! 2. The disability-related need for the animal is NON-
//!    OBVIOUS.
//!
//! Housing providers may NOT:
//! 1. Require a health care professional to use a specific
//!    form;
//! 2. Require notarized statements;
//! 3. Require statements under penalty of perjury;
//! 4. Require an individual's diagnosis;
//! 5. Require detailed information about person's physical
//!    or mental impairments.
//!
//! **No breed, weight, or species restrictions** — HUD
//! FHEO 2020-01 prohibits applying generic pet restrictions
//! (breed bans, weight caps, species limitations) to
//! assistance animals. Assistance animals are NOT pets.
//!
//! **§ 3604(f)(9) Direct threat defense** — housing
//! provider may deny accommodation IF:
//! 1. Animal poses a DIRECT THREAT to health or safety of
//!    others that cannot be eliminated by another
//!    reasonable accommodation; OR
//! 2. Animal would cause SUBSTANTIAL PHYSICAL DAMAGE to
//!    property of others that cannot be reduced or
//!    eliminated by another reasonable accommodation.
//!
//! Direct-threat analysis must be INDIVIDUALIZED —
//! cannot be based on generic breed reputation,
//! stereotypes, or fears.
//!
//! **ADA Title III (42 USC § 12182)** — applies to PUBLIC
//! ACCOMMODATIONS (rental offices, leasing centers): only
//! dogs (and miniature horses) recognized as service
//! animals; emotional support animals NOT covered under
//! ADA Title III. FHA broader coverage applies in the
//! DWELLING ITSELF.
//!
//! **California Fair Employment and Housing Act
//! (FEHA)** — Cal. Gov. Code § 12955 + Cal. Civ. Code
//! § 54.1 — state-level overlay providing parallel
//! protection. **AB 468 (effective January 1, 2022)** —
//! tightened ESA documentation requirements: licensed
//! health professional must have an ESTABLISHED CLIENT
//! RELATIONSHIP of at least 30 days; must complete
//! clinical evaluation; must comply with California
//! professional standards.
//!
//! **§ 504 of Rehabilitation Act of 1973 (29 USC § 794)** —
//! federally-funded housing (HUD-assisted, Section 8,
//! LIHTC) parallel assistance-animal requirements.
//!
//! **FHA private enforcement remedies (42 USC § 3613)**:
//! - Actual damages (relocation, emotional distress, lost
//!   housing opportunity)
//! - **PUNITIVE damages**
//! - Reasonable attorney's fees + costs
//! - Injunctive relief
//!
//! **HUD administrative penalties (24 CFR § 30.65, annually
//! inflation-adjusted)** — 2026:
//! - **First offense**: $25,597
//! - **Second offense (within 5 years)**: $63,993
//! - **Third+ offense (within 7 years)**: $127,985
//!
//! Citations: 42 USC § 3604(f)(3)(B) and § 3604(f)(9);
//! HUD FHEO Notice 2020-01 (January 28, 2020); 42 USC
//! § 3613; 24 CFR § 30.65; ADA Title III (42 USC § 12182);
//! § 504 Rehabilitation Act of 1973 (29 USC § 794); Cal.
//! Gov. Code § 12955; Cal. Civ. Code § 54.1; Cal. AB 468
//! of 2021 (effective January 1, 2022).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnimalType {
    /// ADA service animal — dog (or sometimes miniature
    /// horse) individually trained to perform tasks for
    /// individual with disability.
    ServiceAnimal,
    /// HUD support animal — any species providing
    /// emotional support, comfort, well-being.
    SupportAnimal,
    /// Standard pet — no disability-related function.
    Pet,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OffenseHistory {
    /// First offense — $25,597 maximum 2026.
    FirstOffense,
    /// Second offense within 5 years — $63,993 maximum 2026.
    SecondOffense,
    /// Third+ offense within 7 years — $127,985 maximum 2026.
    ThirdOrSubsequent,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssistanceAnimalAccommodationInput {
    pub animal_type: AnimalType,
    /// Whether tenant has a disability (defined by ADA /
    /// FHA / Rehabilitation Act).
    pub tenant_has_disability: bool,
    /// Whether the disability is OBVIOUS (visible, known
    /// to landlord).
    pub disability_obvious: bool,
    /// Whether the disability-related need for the animal
    /// is OBVIOUS.
    pub disability_need_obvious: bool,
    /// Whether reliable supporting documentation provided
    /// when needed (licensed health professional verifies
    /// disability and/or need for animal).
    pub documentation_provided: bool,
    /// Whether landlord required excessive documentation
    /// (specific form, notarized statement, perjury
    /// statement, diagnosis, detailed impairment info).
    pub excessive_documentation_required: bool,
    /// Whether landlord charged a pet fee or deposit for
    /// the assistance animal (PROHIBITED).
    pub pet_fee_or_deposit_charged: bool,
    /// Whether landlord applied breed/weight/species
    /// restrictions to assistance animal (PROHIBITED).
    pub generic_pet_restrictions_applied: bool,
    /// Whether landlord refused accommodation request.
    pub accommodation_refused: bool,
    /// Whether refusal was based on individualized DIRECT
    /// THREAT determination (§ 3604(f)(9)).
    pub individualized_direct_threat_finding: bool,
    /// Whether animal causes substantial property damage
    /// beyond reasonable wear (§ 3604(f)(9) defense).
    pub substantial_property_damage_finding: bool,
    /// Whether jurisdiction is California (FEHA + AB 468
    /// overlay).
    pub california_feha_jurisdiction: bool,
    /// Whether California AB 468 30-day established-client-
    /// relationship requirement satisfied (only relevant if
    /// California jurisdiction + support animal).
    pub california_ab468_30_day_relationship: bool,
    /// HUD administrative penalty offense history.
    pub offense_history: OffenseHistory,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AssistanceAnimalAccommodationResult {
    pub animal_type: AnimalType,
    pub accommodation_required: bool,
    pub pet_fee_prohibition_applies: bool,
    pub generic_pet_restrictions_prohibited: bool,
    pub direct_threat_defense_engaged: bool,
    pub hud_administrative_penalty_max_cents: u64,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &AssistanceAnimalAccommodationInput) -> AssistanceAnimalAccommodationResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let is_assistance_animal = matches!(
        input.animal_type,
        AnimalType::ServiceAnimal | AnimalType::SupportAnimal
    );

    let accommodation_required = is_assistance_animal
        && input.tenant_has_disability
        && !input.individualized_direct_threat_finding
        && !input.substantial_property_damage_finding;

    let pet_fee_prohibition_applies = is_assistance_animal && input.tenant_has_disability;
    let generic_pet_restrictions_prohibited = is_assistance_animal && input.tenant_has_disability;

    let direct_threat_defense_engaged = is_assistance_animal
        && input.tenant_has_disability
        && (input.individualized_direct_threat_finding
            || input.substantial_property_damage_finding);

    let hud_administrative_penalty_max_cents: u64 = match input.offense_history {
        OffenseHistory::FirstOffense => 2_559_700,
        OffenseHistory::SecondOffense => 6_399_300,
        OffenseHistory::ThirdOrSubsequent => 12_798_500,
    };

    if accommodation_required && input.pet_fee_or_deposit_charged {
        failure_reasons.push(
            "42 USC § 3604(f)(3)(B) + HUD FHEO Notice 2020-01 — landlords may NOT charge a pet fee or deposit for assistance animals as a condition of granting a reasonable accommodation; assistance animals are NOT pets".to_string(),
        );
    }

    if accommodation_required && input.generic_pet_restrictions_applied {
        failure_reasons.push(
            "HUD FHEO Notice 2020-01 — landlords may NOT apply breed bans, weight caps, or species limitations to assistance animals; assistance animals are NOT pets".to_string(),
        );
    }

    if accommodation_required && input.accommodation_refused {
        failure_reasons.push(
            "42 USC § 3604(f)(3)(B) — refusal to make reasonable accommodation when necessary to afford person with disability equal opportunity to enjoy and use a dwelling is unlawful housing discrimination; refusal must be based on § 3604(f)(9) individualized direct-threat OR substantial-property-damage finding".to_string(),
        );
    }

    if input.excessive_documentation_required {
        failure_reasons.push(
            "HUD FHEO Notice 2020-01 — landlords may NOT require (1) specific form; (2) notarized statements; (3) statements under penalty of perjury; (4) individual's diagnosis; (5) detailed physical/mental impairment information from health care professional".to_string(),
        );
    }

    let documentation_needed =
        is_assistance_animal && (!input.disability_obvious || !input.disability_need_obvious);
    let documentation_satisfied = !documentation_needed || input.documentation_provided;

    if !documentation_satisfied {
        failure_reasons.push(
            "HUD FHEO Notice 2020-01 — when disability OR disability-related need for animal is non-obvious, landlord may request reliable supporting documentation; documentation must come from a knowledgeable licensed health care professional".to_string(),
        );
    }

    if input.california_feha_jurisdiction
        && matches!(input.animal_type, AnimalType::SupportAnimal)
        && !input.california_ab468_30_day_relationship
    {
        failure_reasons.push(
            "Cal. AB 468 of 2021 + Cal. Civ. Code § 54.1 (effective January 1, 2022) — California-specific ESA documentation requires licensed mental health professional to have ESTABLISHED CLIENT RELATIONSHIP of at least 30 DAYS before issuing ESA documentation; must complete clinical evaluation; must comply with California professional standards".to_string(),
        );
    }

    if input.individualized_direct_threat_finding {
        failure_reasons.push(
            "42 USC § 3604(f)(9) — direct-threat defense engaged; refusal valid IF (1) finding is INDIVIDUALIZED (not based on generic breed reputation or stereotypes) AND (2) threat to health or safety of others cannot be eliminated by another reasonable accommodation".to_string(),
        );
    }

    if input.substantial_property_damage_finding {
        failure_reasons.push(
            "42 USC § 3604(f)(9) — substantial-property-damage defense engaged; refusal valid IF (1) damage finding is INDIVIDUALIZED AND (2) damage cannot be reduced or eliminated by another reasonable accommodation".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "42 USC § 3604(f)(3)(B) — unlawful housing discrimination for landlord to refuse to make reasonable accommodations in rules, policies, practices, or services when such accommodations may be necessary to afford person with disability equal opportunity to enjoy and use a dwelling".to_string(),
        "HUD FHEO Notice 2020-01 (January 28, 2020) — replaced HUD 2013 guidance; defines two types of assistance animals: (1) SERVICE ANIMALS (ADA Title III dogs and miniature horses individually trained to do work or perform tasks) and (2) SUPPORT ANIMALS (any species providing emotional support, comfort, well-being, therapeutic benefit)".to_string(),
        "HUD FHEO Notice 2020-01 — housing providers may NOT exclude OR charge a fee or deposit for assistance animals; assistance animals are NOT pets; tenant remains responsible for actual damage caused by the animal (general security deposit recovery)".to_string(),
        "HUD FHEO Notice 2020-01 — landlords may NOT apply BREED BANS, WEIGHT CAPS, or SPECIES LIMITATIONS to assistance animals; assistance animals are NOT pets and are exempt from generic pet restrictions".to_string(),
        "HUD FHEO Notice 2020-01 documentation standards — landlords may NOT require (1) specific form; (2) notarized statements; (3) statements under penalty of perjury; (4) individual's diagnosis; (5) detailed physical/mental impairment information; reliable documentation may be requested only when disability OR disability-related need is NON-OBVIOUS".to_string(),
        "42 USC § 3604(f)(9) — direct-threat OR substantial-property-damage defense to accommodation: refusal valid IF (1) finding is INDIVIDUALIZED (not based on generic breed reputation, stereotypes, fears) AND (2) cannot be eliminated by another reasonable accommodation".to_string(),
        "ADA Title III (42 USC § 12182) — applies to public accommodations (rental offices, leasing centers): only dogs (and miniature horses) recognized as service animals; emotional support animals NOT covered under ADA Title III; FHA broader coverage applies in the DWELLING ITSELF".to_string(),
        "Cal. Gov. Code § 12955 + Cal. Civ. Code § 54.1 — California Fair Employment and Housing Act state overlay providing parallel protection".to_string(),
        "Cal. AB 468 of 2021 + Cal. Civ. Code § 54.1 (effective January 1, 2022) — California-specific ESA documentation requirements: licensed mental health professional must have ESTABLISHED CLIENT RELATIONSHIP of at least 30 DAYS before issuing ESA documentation; must complete clinical evaluation".to_string(),
        "§ 504 of Rehabilitation Act of 1973 (29 USC § 794) — federally-funded housing (HUD-assisted, Section 8, LIHTC) parallel assistance-animal requirements".to_string(),
        "42 USC § 3613 — FHA private enforcement remedies: ACTUAL damages (relocation, emotional distress, lost housing opportunity) + PUNITIVE damages + reasonable attorney's fees + costs + injunctive relief".to_string(),
        "24 CFR § 30.65 (annually inflation-adjusted) — HUD administrative penalties 2026: FIRST OFFENSE $25,597; SECOND OFFENSE (within 5 years) $63,993; THIRD+ OFFENSE (within 7 years) $127,985".to_string(),
    ];

    AssistanceAnimalAccommodationResult {
        animal_type: input.animal_type,
        accommodation_required,
        pet_fee_prohibition_applies,
        generic_pet_restrictions_prohibited,
        direct_threat_defense_engaged,
        hud_administrative_penalty_max_cents,
        failure_reasons,
        citation: "42 USC § 3604(f)(3)(B) and § 3604(f)(9); HUD FHEO Notice 2020-01 (January 28, 2020); 42 USC § 3613; 24 CFR § 30.65; ADA Title III (42 USC § 12182); § 504 Rehabilitation Act of 1973 (29 USC § 794); Cal. Gov. Code § 12955; Cal. Civ. Code § 54.1; Cal. AB 468 of 2021",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn esa_compliant() -> AssistanceAnimalAccommodationInput {
        AssistanceAnimalAccommodationInput {
            animal_type: AnimalType::SupportAnimal,
            tenant_has_disability: true,
            disability_obvious: false,
            disability_need_obvious: false,
            documentation_provided: true,
            excessive_documentation_required: false,
            pet_fee_or_deposit_charged: false,
            generic_pet_restrictions_applied: false,
            accommodation_refused: false,
            individualized_direct_threat_finding: false,
            substantial_property_damage_finding: false,
            california_feha_jurisdiction: false,
            california_ab468_30_day_relationship: true,
            offense_history: OffenseHistory::FirstOffense,
        }
    }

    #[test]
    fn baseline_esa_compliant() {
        let r = check(&esa_compliant());
        assert!(r.accommodation_required);
        assert!(r.pet_fee_prohibition_applies);
        assert!(r.generic_pet_restrictions_prohibited);
        assert!(!r.direct_threat_defense_engaged);
        assert!(r.failure_reasons.is_empty());
    }

    #[test]
    fn service_animal_engages_accommodation() {
        let mut i = esa_compliant();
        i.animal_type = AnimalType::ServiceAnimal;
        let r = check(&i);
        assert!(r.accommodation_required);
    }

    #[test]
    fn pet_no_accommodation_required() {
        let mut i = esa_compliant();
        i.animal_type = AnimalType::Pet;
        let r = check(&i);
        assert!(!r.accommodation_required);
        assert!(!r.pet_fee_prohibition_applies);
    }

    #[test]
    fn no_disability_no_accommodation() {
        let mut i = esa_compliant();
        i.tenant_has_disability = false;
        let r = check(&i);
        assert!(!r.accommodation_required);
    }

    #[test]
    fn pet_fee_charged_for_esa_violation() {
        let mut i = esa_compliant();
        i.pet_fee_or_deposit_charged = true;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 3604(f)(3)(B)")
                && f.contains("HUD FHEO Notice 2020-01")
                && f.contains("NOT charge a pet fee")));
    }

    #[test]
    fn breed_or_weight_restriction_for_esa_violation() {
        let mut i = esa_compliant();
        i.generic_pet_restrictions_applied = true;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("HUD FHEO Notice 2020-01")
                && f.contains("breed bans")
                && f.contains("weight caps")));
    }

    #[test]
    fn accommodation_refusal_without_direct_threat_violation() {
        let mut i = esa_compliant();
        i.accommodation_refused = true;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 3604(f)(3)(B)")
                && f.contains("refusal to make reasonable accommodation")));
    }

    #[test]
    fn excessive_documentation_requirement_violation() {
        let mut i = esa_compliant();
        i.excessive_documentation_required = true;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("HUD FHEO Notice 2020-01")
                && f.contains("notarized statements")
                && f.contains("penalty of perjury")
                && f.contains("diagnosis")));
    }

    #[test]
    fn non_obvious_disability_no_documentation_violation() {
        let mut i = esa_compliant();
        i.disability_obvious = false;
        i.disability_need_obvious = false;
        i.documentation_provided = false;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("HUD FHEO Notice 2020-01")
                && f.contains("non-obvious")
                && f.contains("licensed health care professional")));
    }

    #[test]
    fn obvious_disability_no_documentation_required() {
        let mut i = esa_compliant();
        i.disability_obvious = true;
        i.disability_need_obvious = true;
        i.documentation_provided = false;
        let r = check(&i);
        assert!(!r.failure_reasons.iter().any(|f| f.contains("non-obvious")));
    }

    #[test]
    fn direct_threat_defense_engages_with_individualized_finding() {
        let mut i = esa_compliant();
        i.individualized_direct_threat_finding = true;
        let r = check(&i);
        assert!(r.direct_threat_defense_engaged);
        assert!(!r.accommodation_required);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 3604(f)(9)")
            && f.contains("INDIVIDUALIZED")
            && f.contains("breed reputation or stereotypes")));
    }

    #[test]
    fn substantial_property_damage_defense_engages() {
        let mut i = esa_compliant();
        i.substantial_property_damage_finding = true;
        let r = check(&i);
        assert!(r.direct_threat_defense_engaged);
        assert!(!r.accommodation_required);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 3604(f)(9)") && f.contains("substantial-property-damage")));
    }

    #[test]
    fn california_ab468_30_day_relationship_required() {
        let mut i = esa_compliant();
        i.california_feha_jurisdiction = true;
        i.california_ab468_30_day_relationship = false;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f| f.contains("AB 468")
            && f.contains("Cal. Civ. Code § 54.1")
            && f.contains("ESTABLISHED CLIENT RELATIONSHIP")
            && f.contains("30 DAYS")));
    }

    #[test]
    fn california_ab468_only_applies_to_support_animals() {
        let mut i = esa_compliant();
        i.animal_type = AnimalType::ServiceAnimal;
        i.california_feha_jurisdiction = true;
        i.california_ab468_30_day_relationship = false;
        let r = check(&i);
        assert!(!r.failure_reasons.iter().any(|f| f.contains("AB 468")));
    }

    #[test]
    fn hud_first_offense_2026_penalty_25597() {
        let r = check(&esa_compliant());
        assert_eq!(r.hud_administrative_penalty_max_cents, 2_559_700);
    }

    #[test]
    fn hud_second_offense_2026_penalty_63993() {
        let mut i = esa_compliant();
        i.offense_history = OffenseHistory::SecondOffense;
        let r = check(&i);
        assert_eq!(r.hud_administrative_penalty_max_cents, 6_399_300);
    }

    #[test]
    fn hud_third_offense_2026_penalty_127985() {
        let mut i = esa_compliant();
        i.offense_history = OffenseHistory::ThirdOrSubsequent;
        let r = check(&i);
        assert_eq!(r.hud_administrative_penalty_max_cents, 12_798_500);
    }

    #[test]
    fn offense_history_progressive_increase_invariant() {
        let make = |hist| {
            let mut i = esa_compliant();
            i.offense_history = hist;
            check(&i)
        };
        let first = make(OffenseHistory::FirstOffense);
        let second = make(OffenseHistory::SecondOffense);
        let third = make(OffenseHistory::ThirdOrSubsequent);
        assert!(
            first.hud_administrative_penalty_max_cents
                < second.hud_administrative_penalty_max_cents
        );
        assert!(
            second.hud_administrative_penalty_max_cents
                < third.hud_administrative_penalty_max_cents
        );
    }

    #[test]
    fn animal_type_truth_table_three_cells() {
        for (animal, exp_required) in [
            (AnimalType::ServiceAnimal, true),
            (AnimalType::SupportAnimal, true),
            (AnimalType::Pet, false),
        ] {
            let mut i = esa_compliant();
            i.animal_type = animal;
            let r = check(&i);
            assert_eq!(
                r.accommodation_required, exp_required,
                "animal={:?}",
                animal
            );
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&esa_compliant());
        assert!(r.citation.contains("42 USC § 3604(f)(3)(B)"));
        assert!(r.citation.contains("§ 3604(f)(9)"));
        assert!(r.citation.contains("HUD FHEO Notice 2020-01"));
        assert!(r.citation.contains("January 28, 2020"));
        assert!(r.citation.contains("42 USC § 3613"));
        assert!(r.citation.contains("24 CFR § 30.65"));
        assert!(r.citation.contains("ADA Title III"));
        assert!(r.citation.contains("42 USC § 12182"));
        assert!(r.citation.contains("§ 504 Rehabilitation Act"));
        assert!(r.citation.contains("Cal. Gov. Code § 12955"));
        assert!(r.citation.contains("Cal. Civ. Code § 54.1"));
        assert!(r.citation.contains("Cal. AB 468 of 2021"));
    }

    #[test]
    fn note_pins_fha_3604f3b_reasonable_accommodation() {
        let r = check(&esa_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 3604(f)(3)(B)")
            && n.contains("unlawful housing discrimination")
            && n.contains("equal opportunity")));
    }

    #[test]
    fn note_pins_hud_2020_01_two_animal_types() {
        let r = check(&esa_compliant());
        assert!(r.notes.iter().any(|n| n.contains("HUD FHEO Notice 2020-01")
            && n.contains("January 28, 2020")
            && n.contains("SERVICE ANIMALS")
            && n.contains("SUPPORT ANIMALS")));
    }

    #[test]
    fn note_pins_hud_2020_01_pet_fee_prohibition() {
        let r = check(&esa_compliant());
        assert!(r.notes.iter().any(|n| n.contains("HUD FHEO Notice 2020-01")
            && n.contains("NOT exclude OR charge a fee or deposit")
            && n.contains("NOT pets")));
    }

    #[test]
    fn note_pins_hud_2020_01_no_breed_weight_species() {
        let r = check(&esa_compliant());
        assert!(r.notes.iter().any(|n| n.contains("HUD FHEO Notice 2020-01")
            && n.contains("BREED BANS")
            && n.contains("WEIGHT CAPS")
            && n.contains("SPECIES LIMITATIONS")));
    }

    #[test]
    fn note_pins_documentation_standards_five_prohibitions() {
        let r = check(&esa_compliant());
        assert!(r.notes.iter().any(|n| n
            .contains("HUD FHEO Notice 2020-01 documentation standards")
            && n.contains("specific form")
            && n.contains("notarized statements")
            && n.contains("penalty of perjury")
            && n.contains("diagnosis")
            && n.contains("NON-OBVIOUS")));
    }

    #[test]
    fn note_pins_3604f9_direct_threat_substantial_damage() {
        let r = check(&esa_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 3604(f)(9)")
            && n.contains("direct-threat OR substantial-property-damage")
            && n.contains("INDIVIDUALIZED")
            && n.contains("stereotypes")));
    }

    #[test]
    fn note_pins_ada_title_iii_dogs_horses() {
        let r = check(&esa_compliant());
        assert!(r.notes.iter().any(|n| n.contains("ADA Title III")
            && n.contains("dogs")
            && n.contains("miniature horses")
            && n.contains("NOT covered under ADA Title III")));
    }

    #[test]
    fn note_pins_california_feha() {
        let r = check(&esa_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Cal. Gov. Code § 12955")
            && n.contains("Cal. Civ. Code § 54.1")
            && n.contains("Fair Employment and Housing")));
    }

    #[test]
    fn note_pins_california_ab468_30_day_relationship() {
        let r = check(&esa_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Cal. AB 468 of 2021")
            && n.contains("January 1, 2022")
            && n.contains("ESTABLISHED CLIENT RELATIONSHIP")
            && n.contains("30 DAYS")));
    }

    #[test]
    fn note_pins_section_504_rehabilitation_act() {
        let r = check(&esa_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 504 of Rehabilitation Act of 1973")
                && n.contains("29 USC § 794")
                && n.contains("Section 8")
                && n.contains("LIHTC")));
    }

    #[test]
    fn note_pins_3613_private_enforcement_punitive() {
        let r = check(&esa_compliant());
        assert!(r.notes.iter().any(|n| n.contains("42 USC § 3613")
            && n.contains("ACTUAL damages")
            && n.contains("PUNITIVE damages")
            && n.contains("attorney's fees")));
    }

    #[test]
    fn note_pins_hud_2026_penalty_tiers() {
        let r = check(&esa_compliant());
        assert!(r.notes.iter().any(|n| n.contains("24 CFR § 30.65")
            && n.contains("FIRST OFFENSE $25,597")
            && n.contains("SECOND OFFENSE")
            && n.contains("$63,993")
            && n.contains("THIRD+ OFFENSE")
            && n.contains("$127,985")));
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = esa_compliant();
        i.pet_fee_or_deposit_charged = true;
        i.generic_pet_restrictions_applied = true;
        i.accommodation_refused = true;
        i.excessive_documentation_required = true;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 4);
    }

    #[test]
    fn direct_threat_with_finding_engages_defense_unique_invariant() {
        let mut threat = esa_compliant();
        threat.individualized_direct_threat_finding = true;
        let r_threat = check(&threat);
        assert!(r_threat.direct_threat_defense_engaged);

        let mut damage = esa_compliant();
        damage.substantial_property_damage_finding = true;
        let r_damage = check(&damage);
        assert!(r_damage.direct_threat_defense_engaged);

        let r_baseline = check(&esa_compliant());
        assert!(!r_baseline.direct_threat_defense_engaged);
    }
}
