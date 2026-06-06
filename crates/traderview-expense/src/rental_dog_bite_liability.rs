//! Landlord dog-bite liability compliance framework for residential rentals.
//!
//! Dog-bite liability for landlords (as distinct from dog-owner tenants) turns on a
//! NEGLIGENCE standard in nearly every U.S. jurisdiction: the landlord becomes liable
//! when (1) the landlord had ACTUAL OR CONSTRUCTIVE KNOWLEDGE of the dog's dangerous
//! propensities AND (2) the landlord had the LEGAL ABILITY TO ABATE the hazard
//! (refuse to renew, terminate lease, prohibit dog presence, restrict common-area
//! access) AND (3) failed to take reasonable steps. Strict-liability dog-bite statutes
//! (CA + WA + FL + IL + many others) apply to the DOG OWNER, not the landlord.
//!
//! Jurisdictional grid:
//!
//! - CA Cal. Civ. Code § 3342: STRICT LIABILITY for dog owner; landlord subject to
//!   NEGLIGENCE standard requiring (1) actual knowledge of dangerous dog, (2)
//!   reasonable opportunity to act, (3) failure to act. Uccello v. Laudenslayer
//!   (1975) 44 Cal. App. 3d 504 establishes landlord-knowledge-plus-ability-to-
//!   act test. Two-year statute of limitations under Code Civ. Proc. § 335.1.
//! - NY Agriculture & Markets Law § 121 + § 123: dog-owner strict liability for
//!   medical costs from dangerous-dog injuries; landlord liable on negligence if
//!   actual knowledge of dangerous propensities + ability to act. Bard v. Jahnke
//!   (2006) 6 N.Y.3d 592 establishes "vicious propensities" standard.
//! - IL 510 ILCS 5/16 (Animal Control Act): dog-owner strict liability for
//!   provoked-free injuries to lawful-presence victims. Landlord liability under
//!   premises-liability theory + dangerous-propensities knowledge.
//! - WA RCW 16.08.040: dog-owner strict liability for bites to lawful-presence
//!   victims REGARDLESS of prior viciousness or owner knowledge — strictest
//!   regime. Landlord subject to common-law negligence + premises liability.
//! - TX one-bite rule: dog owner liability requires knowledge of dog's aggressive
//!   tendencies. Marshall v. Ranne (1974) 511 S.W.2d 255 establishes vicious-
//!   propensity test. Landlord liability turns on knowledge + ability to act.
//! - FL Fla. Stat. § 767.04: dog-owner strict liability for bites to lawful-
//!   presence victims regardless of viciousness, with comparative-negligence
//!   reduction for victim fault. Landlord liability under premises-liability +
//!   actual-knowledge framework. § 767.13(2) dangerous-dog classification
//!   procedure.
//! - OH ORC § 955.28: dog-owner strict liability for property/personal injury
//!   subject to statutory exceptions. Landlord on negligence theory.
//! - DEFAULT: common-law negligence + premises-liability framework; "one-bite
//!   rule" controlling absent statutory modification.
//!
//! Citations (verified per WebSearch 2026-06-03):
//! - codes.findlaw.com/ca/civil-code/civ-sect-3342/
//! - codes.findlaw.com/ny/agriculture-and-markets-law/agm-sect-123/
//! - app.leg.wa.gov/rcw/default.aspx?cite=16.08.040
//! - flsenate.gov/Laws/Statutes/2021/767.04
//! - <https://recordinglaw.com/dog-laws/dog-bite-laws-summary-by-state/illinois-dog-bite-laws/>

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    NewYork,
    Illinois,
    Washington,
    Texas,
    Florida,
    Ohio,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlordKnowledgeStatus {
    /// Landlord had actual knowledge of dog's dangerous propensity (prior bite,
    /// growling at common-area users, prior animal-control complaint).
    ActualKnowledgeOfDangerousPropensity,
    /// Landlord had constructive knowledge (knew dog existed and breed/size
    /// presented foreseeable risk; reasonable inspection would have revealed).
    ConstructiveKnowledgePresumed,
    /// No knowledge of dangerous propensity reasonably attributable.
    NoKnowledgeReasonablyAttributable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlordControlOverPremises {
    /// Bite in COMMON AREA controlled by landlord (lobby, hallway, parking lot,
    /// pool deck).
    BiteInCommonAreaControlledByLandlord,
    /// Bite in tenant's leased premises — landlord control limited.
    BiteInTenantsLeasedPremises,
    /// Bite off-premises while dog being walked.
    BiteOffPremisesDuringWalk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlordAction {
    /// Landlord took reasonable abatement (notice to remove dog, lease
    /// termination, common-area restriction, leash/muzzle requirement).
    TookReasonableAbatementSteps,
    /// Landlord did not take reasonable abatement despite knowledge.
    FailedToAbateDespiteKnowledge,
    /// No abatement required (no knowledge or no control).
    NoAbatementRequiredAbsentKnowledgeOrControl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NoLandlordLiabilityNoKnowledgeOrNoControl,
    LandlordLiableKnowledgeFailedToAbateCommonArea,
    LandlordLiableKnowledgeFailedToAbateTenantPremises,
    LandlordTookReasonableAbatementNoLiability,
    DogOwnerStrictLiabilityNoLandlordExposure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub landlord_knowledge_status: LandlordKnowledgeStatus,
    pub landlord_control_over_premises: LandlordControlOverPremises,
    pub landlord_action: LandlordAction,
    pub plaintiff_actual_damages_cents: u64,
}

pub type RentalDogBiteLiabilityInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub estimated_landlord_exposure_cents: u64,
    pub note: String,
}

pub type RentalDogBiteLiabilityOutput = Output;
pub type RentalDogBiteLiabilityResult = Output;

const TYPICAL_DOG_BITE_TORT_SETTLEMENT_BASELINE_CENTS: u64 = 5_000_000;
const TYPICAL_EMOTIONAL_DISTRESS_AWARD_CENTS: u64 = 1_500_000;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.landlord_knowledge_status,
        LandlordKnowledgeStatus::NoKnowledgeReasonablyAttributable
    ) {
        return Output {
            severity: Severity::NoLandlordLiabilityNoKnowledgeOrNoControl,
            estimated_landlord_exposure_cents: 0,
            note: format!(
                "No landlord liability: no actual or constructive knowledge of dog's \
                 dangerous propensities. {} Landlord-dog-bite liability requires (1) \
                 knowledge of dangerous propensities (prior bite, growling, animal-control \
                 complaint), (2) ability to abate, and (3) failure to act. Without all \
                 three, NEGLIGENCE theory fails. Dog OWNER (tenant) remains exposed under \
                 jurisdiction-specific strict-liability statute.",
                statute_citation(input.jurisdiction)
            ),
        };
    }

    if matches!(
        input.landlord_action,
        LandlordAction::TookReasonableAbatementSteps
    ) {
        return Output {
            severity: Severity::LandlordTookReasonableAbatementNoLiability,
            estimated_landlord_exposure_cents: 0,
            note: format!(
                "Landlord took reasonable abatement steps despite knowledge — typically \
                 sufficient to defeat negligence claim. Reasonable abatement includes: \
                 notice to remove dog within statutory cure period + lease termination if \
                 not removed + common-area-restriction enforcement + leash/muzzle requirement \
                 + posted-warning signs. {} Document the abatement steps with dated records \
                 (notice service receipt + photographs + correspondence with tenant).",
                statute_citation(input.jurisdiction)
            ),
        };
    }

    if matches!(
        input.landlord_control_over_premises,
        LandlordControlOverPremises::BiteOffPremisesDuringWalk
    ) && !matches!(
        input.landlord_knowledge_status,
        LandlordKnowledgeStatus::ActualKnowledgeOfDangerousPropensity
    ) {
        return Output {
            severity: Severity::DogOwnerStrictLiabilityNoLandlordExposure,
            estimated_landlord_exposure_cents: 0,
            note: format!(
                "Off-premises bite during walk — landlord has minimal exposure absent actual \
                 knowledge of dangerous propensity AND specific failure to require leashing/ \
                 muzzling. Dog owner (tenant) faces strict-liability or one-bite-rule \
                 exposure under {}. Landlord may face only limited premises-liability claim \
                 if dog was known dangerous + landlord allowed off-leash departure.",
                statute_citation(input.jurisdiction)
            ),
        };
    }

    if matches!(
        input.landlord_action,
        LandlordAction::FailedToAbateDespiteKnowledge
    ) {
        let exposure = compute_landlord_exposure(input);
        let severity = match input.landlord_control_over_premises {
            LandlordControlOverPremises::BiteInCommonAreaControlledByLandlord => {
                Severity::LandlordLiableKnowledgeFailedToAbateCommonArea
            }
            LandlordControlOverPremises::BiteInTenantsLeasedPremises => {
                Severity::LandlordLiableKnowledgeFailedToAbateTenantPremises
            }
            LandlordControlOverPremises::BiteOffPremisesDuringWalk => {
                Severity::LandlordLiableKnowledgeFailedToAbateCommonArea
            }
        };
        return Output {
            severity,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "LANDLORD LIABILITY EXPOSURE: landlord had {} of dog's dangerous \
                 propensities + control over {} + failed to abate (no removal-notice, no \
                 lease termination, no common-area restriction). {} Estimated exposure ${} \
                 = plaintiff actual damages (${}) + typical emotional-distress baseline \
                 (${}) + tort settlement baseline (${}). Plaintiff also entitled to \
                 attorney fees where statute permits.",
                knowledge_label(input.landlord_knowledge_status),
                control_label(input.landlord_control_over_premises),
                statute_citation(input.jurisdiction),
                exposure / 100,
                input.plaintiff_actual_damages_cents / 100,
                TYPICAL_EMOTIONAL_DISTRESS_AWARD_CENTS / 100,
                TYPICAL_DOG_BITE_TORT_SETTLEMENT_BASELINE_CENTS / 100
            ),
        };
    }

    Output {
        severity: Severity::NoLandlordLiabilityNoKnowledgeOrNoControl,
        estimated_landlord_exposure_cents: 0,
        note: format!(
            "No landlord exposure under current facts. {} Dog-owner tenant remains exposed \
             under jurisdiction-specific strict-liability or one-bite framework.",
            statute_citation(input.jurisdiction)
        ),
    }
}

fn compute_landlord_exposure(input: &Input) -> u64 {
    input
        .plaintiff_actual_damages_cents
        .saturating_add(TYPICAL_EMOTIONAL_DISTRESS_AWARD_CENTS)
        .saturating_add(TYPICAL_DOG_BITE_TORT_SETTLEMENT_BASELINE_CENTS)
}

fn knowledge_label(status: LandlordKnowledgeStatus) -> &'static str {
    match status {
        LandlordKnowledgeStatus::ActualKnowledgeOfDangerousPropensity => "actual knowledge",
        LandlordKnowledgeStatus::ConstructiveKnowledgePresumed => "constructive knowledge",
        LandlordKnowledgeStatus::NoKnowledgeReasonablyAttributable => "no knowledge",
    }
}

fn control_label(control: LandlordControlOverPremises) -> &'static str {
    match control {
        LandlordControlOverPremises::BiteInCommonAreaControlledByLandlord => {
            "common area controlled by landlord"
        }
        LandlordControlOverPremises::BiteInTenantsLeasedPremises => "tenant's leased premises",
        LandlordControlOverPremises::BiteOffPremisesDuringWalk => "off-premises during walk",
    }
}

fn statute_citation(jurisdiction: Jurisdiction) -> &'static str {
    match jurisdiction {
        Jurisdiction::California => {
            "CA Cal. Civ. Code § 3342 (dog-owner strict liability) + Uccello v. Laudenslayer \
             (1975) 44 Cal. App. 3d 504 (landlord knowledge-plus-ability-to-act test) + \
             Code Civ. Proc. § 335.1 (2-year SOL)."
        }
        Jurisdiction::NewYork => {
            "NY Agriculture & Markets Law § 121 + § 123 (dangerous-dog statute) + Bard v. \
             Jahnke (2006) 6 N.Y.3d 592 (vicious-propensities standard)."
        }
        Jurisdiction::Illinois => {
            "IL 510 ILCS 5/16 (Animal Control Act dog-owner strict liability) + landlord \
             premises-liability framework."
        }
        Jurisdiction::Washington => {
            "WA RCW 16.08.040 (dog-owner strict liability regardless of viciousness or \
             knowledge) — strictest in the country; landlord on common-law negligence + \
             premises liability."
        }
        Jurisdiction::Texas => {
            "TX one-bite rule + Marshall v. Ranne (1974) 511 S.W.2d 255 (vicious-propensity \
             test) + landlord premises-liability framework."
        }
        Jurisdiction::Florida => {
            "FL Fla. Stat. § 767.04 (dog-owner strict liability with comparative-negligence \
             reduction) + § 767.13(2) dangerous-dog classification + landlord premises-\
             liability framework."
        }
        Jurisdiction::Ohio => {
            "OH ORC § 955.28 (dog-owner strict liability for property/personal injury) + \
             landlord on negligence."
        }
        Jurisdiction::Default => {
            "Common-law negligence + premises-liability framework with one-bite rule \
             controlling absent statutory modification."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_ca() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            landlord_knowledge_status:
                LandlordKnowledgeStatus::ActualKnowledgeOfDangerousPropensity,
            landlord_control_over_premises:
                LandlordControlOverPremises::BiteInCommonAreaControlledByLandlord,
            landlord_action: LandlordAction::FailedToAbateDespiteKnowledge,
            plaintiff_actual_damages_cents: 50_000_00,
        }
    }

    #[test]
    fn no_knowledge_no_landlord_liability() {
        let mut input = base_ca();
        input.landlord_knowledge_status =
            LandlordKnowledgeStatus::NoKnowledgeReasonablyAttributable;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoLandlordLiabilityNoKnowledgeOrNoControl
        );
        assert_eq!(output.estimated_landlord_exposure_cents, 0);
        assert!(output.note.contains("dangerous propensities"));
    }

    #[test]
    fn reasonable_abatement_no_liability() {
        let mut input = base_ca();
        input.landlord_action = LandlordAction::TookReasonableAbatementSteps;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::LandlordTookReasonableAbatementNoLiability
        );
        assert!(output.note.contains("abatement"));
        assert!(output.note.contains("lease termination"));
    }

    #[test]
    fn ca_knowledge_failed_to_abate_common_area_liability() {
        let input = base_ca();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::LandlordLiableKnowledgeFailedToAbateCommonArea
        );
        // $50K + $15K + $50K = $115K
        assert_eq!(output.estimated_landlord_exposure_cents, 115_000_00);
        assert!(output.note.contains("Uccello"));
        assert!(output.note.contains("§ 3342"));
    }

    #[test]
    fn ca_knowledge_failed_to_abate_tenant_premises_liability() {
        let mut input = base_ca();
        input.landlord_control_over_premises =
            LandlordControlOverPremises::BiteInTenantsLeasedPremises;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::LandlordLiableKnowledgeFailedToAbateTenantPremises
        );
    }

    #[test]
    fn off_premises_walk_without_knowledge_dog_owner_only() {
        let mut input = base_ca();
        input.landlord_knowledge_status = LandlordKnowledgeStatus::ConstructiveKnowledgePresumed;
        input.landlord_control_over_premises =
            LandlordControlOverPremises::BiteOffPremisesDuringWalk;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::DogOwnerStrictLiabilityNoLandlordExposure
        );
    }

    #[test]
    fn ny_negligence_standard_pins_bard_v_jahnke() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewYork;
        let output = check(&input);
        assert!(output.note.contains("Bard v. Jahnke"));
        assert!(output.note.contains("§ 121"));
        assert!(output.note.contains("§ 123"));
    }

    #[test]
    fn il_510_ilcs_5_16_animal_control_act() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Illinois;
        let output = check(&input);
        assert!(output.note.contains("510 ILCS 5/16"));
        assert!(output.note.contains("Animal Control Act"));
    }

    #[test]
    fn wa_rcw_16_08_040_strictest_in_country() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Washington;
        let output = check(&input);
        assert!(output.note.contains("RCW 16.08.040"));
        assert!(output.note.contains("strictest"));
    }

    #[test]
    fn tx_one_bite_rule_marshall_v_ranne() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Texas;
        let output = check(&input);
        assert!(output.note.contains("one-bite rule"));
        assert!(output.note.contains("Marshall v. Ranne"));
    }

    #[test]
    fn fl_767_04_comparative_negligence_reduction() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Florida;
        let output = check(&input);
        assert!(output.note.contains("§ 767.04"));
        assert!(output.note.contains("comparative-negligence"));
    }

    #[test]
    fn oh_orc_955_28_strict_liability() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Ohio;
        let output = check(&input);
        assert!(output.note.contains("§ 955.28"));
    }

    #[test]
    fn default_jurisdiction_one_bite_rule_common_law() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Default;
        let output = check(&input);
        assert!(output.note.contains("one-bite rule"));
    }

    #[test]
    fn typical_dog_bite_tort_baseline_constant_pins_50000() {
        assert_eq!(TYPICAL_DOG_BITE_TORT_SETTLEMENT_BASELINE_CENTS, 5_000_000);
    }

    #[test]
    fn typical_emotional_distress_constant_pins_15000() {
        assert_eq!(TYPICAL_EMOTIONAL_DISTRESS_AWARD_CENTS, 1_500_000);
    }

    #[test]
    fn very_large_damages_no_overflow() {
        let mut input = base_ca();
        input.plaintiff_actual_damages_cents = u64::MAX;
        let output = check(&input);
        assert_eq!(output.estimated_landlord_exposure_cents, u64::MAX);
    }

    #[test]
    fn zero_damages_uses_baseline_emotional_plus_tort() {
        let mut input = base_ca();
        input.plaintiff_actual_damages_cents = 0;
        let output = check(&input);
        // $15K + $50K = $65K
        assert_eq!(output.estimated_landlord_exposure_cents, 65_000_00);
    }

    #[test]
    fn constructive_knowledge_with_common_area_failure_liability() {
        let mut input = base_ca();
        input.landlord_knowledge_status = LandlordKnowledgeStatus::ConstructiveKnowledgePresumed;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::LandlordLiableKnowledgeFailedToAbateCommonArea
        );
    }

    #[test]
    fn note_pins_uccello_v_laudenslayer_california() {
        let input = base_ca();
        let output = check(&input);
        assert!(output.note.contains("Uccello v. Laudenslayer"));
    }
}
