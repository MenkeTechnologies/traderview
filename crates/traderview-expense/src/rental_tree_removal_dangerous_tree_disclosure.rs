//! Multi-State Residential Property Tree Removal Liability +
//! Dangerous Tree Disclosure Compliance Module.
//!
//! Pure-compute check for whether a landlord has satisfied the
//! applicable state common-law or statutory regime governing
//! (a) liability for landlord-owned trees that injure third
//! parties or neighbors, (b) liability for failure to disclose
//! dangerous trees to tenants, and (c) self-help / nuisance
//! rights vis-à-vis encroaching neighbor trees. Trader-landlord
//! critical because tree-fall liability + dangerous-tree
//! premises-liability exposure compounds at scale: a single
//! diseased oak on a 50-unit property creates catastrophic
//! third-party injury exposure.
//!
//! Web research (verified 2026-06-03):
//! - **Hawaii Rule** (Whitesell v. Houlton, 1981; 63 Haw. 532;
//!   632 P.2d 1077): overhanging branches OR protruding roots
//!   constitute a nuisance when they actually cause, OR there is
//!   imminent danger of causing, "sensible" harm to property
//!   OTHER THAN plant life — excluding mere shade-casting or
//!   leaf/flower/fruit dropping. Damaged or imminently endangered
//!   neighbor may either (1) use self-help to cut back on the
//!   encroaching tree OR (2) require the owner of the offending
//!   tree to pay for damages AND cut back endangering branches /
//!   roots. ([Tree and Neighbor Law Blog — Hawaii Rule](https://treeandneighborlawblog.com/tag/hawaii-rule/);
//!   Florida Bar Journal — Nuisance Trees Massachusetts or Hawaii
//!   Rule.)
//! - **Massachusetts Rule** (libertarian view; Michalson v. Nutting
//!   1931; Ponte v. DaSilva 1985 reaffirmation): no liability on
//!   adjoining landowner for the natural processes and cycles of
//!   trees, plants, roots, and vines. Self-help to property line
//!   is the EXCLUSIVE remedy. ([Tree and Neighbor Law Blog —
//!   Massachusetts Rule](https://treeandneighborlawblog.com/tag/massachusetts-rule/).)
//! - **California Booska v. Patel** (1994; 24 Cal.App.4th 1786;
//!   30 Cal.Rptr.2d 241): a neighbor does NOT have the ABSOLUTE
//!   right to cut encroaching roots and branches to the property
//!   line. "Whatever rights Patel has in the management of his
//!   own land, those rights are TEMPERED by his duty to act
//!   reasonably." A property owner must exercise ORDINARY CARE
//!   when cutting encroaching branches or roots — cannot trim in
//!   a way that damages or kills the tree. ([Justia Booska v.
//!   Patel](https://law.justia.com/cases/california/court-of-appeal/4th/24/1786.html);
//!   FindLaw Booska v. Patel; Apartment Association of Greater
//!   Los Angeles — Legal Issues Related to Your Property's Trees
//!   and Vegetation.)
//! - **California Civ. Code § 833**: trees whose trunks stand
//!   WHOLLY upon the land of one owner belong exclusively to that
//!   owner, although their roots grow into the land of another.
//! - **California Civ. Code § 834**: boundary trees (trunk on
//!   boundary line) jointly owned.
//! - **California Civ. Code § 836**: nuisance abatement for tree
//!   intrusion.
//! - **Restatement (Second) of Torts § 363**: natural conditions —
//!   landowner not liable for natural conditions causing harm OFF
//!   premises EXCEPT in urban areas.
//! - **Restatement (Second) of Torts § 364**: artificial
//!   conditions — landowner liable.
//! - **Landlord premises liability**: when a tree creates a
//!   potential hazard to health and safety in an area the landlord
//!   controls, the landlord must address the risk AND provide
//!   reasonable warning for anyone who might be unaware. Common
//!   law negligence: duty of care + foreseeability + warranty of
//!   habitability includes safe outdoor conditions. ([iProperty
//!   Management Landlord Tree Responsibility](https://ipropertymanagement.com/laws/landlord-tree-responsibility);
//!   NC State Extension Tree Fall Liability.)

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const HAWAII_RULE_YEAR_WHITESELL_HOULTON: u32 = 1981;
pub const CALIFORNIA_BOOSKA_PATEL_YEAR: u32 = 1994;
pub const CALIFORNIA_BOOSKA_PATEL_CITATION_VOLUME: u32 = 24;
pub const CALIFORNIA_BOOSKA_PATEL_CITATION_PAGE: u32 = 1786;
pub const MASSACHUSETTS_RULE_MICHALSON_NUTTING_YEAR: u32 = 1931;
pub const MASSACHUSETTS_RULE_PONTE_DASILVA_REAFFIRMATION_YEAR: u32 = 1985;
pub const CALIFORNIA_CIV_CODE_833: u32 = 833;
pub const CALIFORNIA_CIV_CODE_834_BOUNDARY: u32 = 834;
pub const CALIFORNIA_CIV_CODE_836_NUISANCE: u32 = 836;
pub const RESTATEMENT_SECOND_TORTS_363_NATURAL: u32 = 363;
pub const RESTATEMENT_SECOND_TORTS_364_ARTIFICIAL: u32 = 364;
pub const RESTATEMENT_SECOND_TORTS_840_ENCROACHING: u32 = 840;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TreeJurisdiction {
    HawaiiRuleStates,
    MassachusettsRuleStates,
    CaliforniaBooskaPatelHybrid,
    OtherStateCommonLawNegligenceRestatement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TreeScenario {
    TreeOnLandlordPropertyOverhangingNeighbor,
    NeighborTreeEncroachingOnLandlordProperty,
    DangerousTreeOnLandlordPropertyPosingFalls,
    StormDamageFallenTreeOnTenantPremises,
    BoundaryTreeMutualOwnership,
    HealthySafeTreeNoIssue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlordAction {
    AddressedRiskAndWarnedTenant,
    AddressedRiskOnly,
    WarnedTenantOnly,
    NoActionTaken,
    SelfHelpTrimmedToPropertyLine,
    SelfHelpTrimmedWithOrdinaryCare,
    SelfHelpTrimmedDamagingTree,
    PaidDamagesAndCutBackBranches,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TreeRemovalDangerousDisclosureMode {
    NotApplicableNoTreeIssueOrLandlordNotPropertyOwner,
    CompliantHawaiiRuleSelfHelpOrPayDamages,
    CompliantMassachusettsRuleSelfHelpOnly,
    CompliantCaliforniaBooskaPatelOrdinaryCareExercised,
    CompliantDangerousTreeDisclosedToTenant,
    CompliantNeighborTreeRiskAddressedByLandlord,
    ViolationHawaiiRuleNuisanceTreeDamagesNotPaid,
    ViolationCaliforniaBooskaPatelOrdinaryCareNotExercised,
    ViolationLandlordFailedToAddressDangerousTreeOnPremises,
    ViolationLandlordFailedToWarnTenantOfTreeHazard,
    ViolationLandlordTrimmedNeighborTreeWithoutOrdinaryCare,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: TreeJurisdiction,
    pub tree_scenario: TreeScenario,
    pub landlord_action: LandlordAction,
    pub tree_creates_foreseeable_risk_of_harm: bool,
    pub tree_imminently_dangerous_to_neighboring_property: bool,
    pub tenant_warned_in_writing_of_dangerous_tree: bool,
    pub landlord_paid_neighbor_damages_for_tree_harm: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: TreeRemovalDangerousDisclosureMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalTreeRemovalDangerousTreeDisclosureInput = Input;
pub type RentalTreeRemovalDangerousTreeDisclosureOutput = Output;
pub type RentalTreeRemovalDangerousTreeDisclosureResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Hawaii Rule — Whitesell v. Houlton, 63 Haw. 532, 632 P.2d 1077 (1981): overhanging branches/protruding roots are nuisance when actually cause OR imminently endanger sensible harm to property (other than plant life, shade, leaves/flowers/fruit); neighbor may self-help cut OR require owner to pay damages + cut back".to_string(),
        "Massachusetts Rule — Michalson v. Nutting (1931); reaffirmed Ponte v. DaSilva (1985): libertarian view — no liability on adjoining landowner for natural processes; self-help to property line is exclusive remedy".to_string(),
        "California Booska v. Patel, 24 Cal.App.4th 1786, 30 Cal.Rptr.2d 241 (1994): self-help to cut encroaching branches/roots is NOT absolute; property owner must exercise ORDINARY CARE; cannot trim in way that damages or kills tree".to_string(),
        "Cal. Civ. Code § 833 — trees whose trunks stand wholly on one owner's land belong exclusively to that owner, even if roots extend onto neighbor's land".to_string(),
        "Cal. Civ. Code § 834 — boundary trees (trunk on boundary line) jointly owned".to_string(),
        "Cal. Civ. Code § 836 — nuisance abatement for tree intrusion".to_string(),
        "Restatement (Second) of Torts § 363 — natural conditions: landowner not liable for natural conditions causing harm OFF premises EXCEPT in urban areas".to_string(),
        "Restatement (Second) of Torts § 364 — artificial conditions: landowner liable".to_string(),
        "Restatement (Second) of Torts § 840 — encroaching trees: modern majority adopts Hawaii Rule of nuisance liability for imminent danger".to_string(),
        "Landlord premises liability — common law negligence: duty of care + foreseeability + warranty of habitability includes safe outdoor conditions; landlord must address risk AND warn unaware persons".to_string(),
    ];

    if input.tree_scenario == TreeScenario::HealthySafeTreeNoIssue {
        return Output {
            mode: TreeRemovalDangerousDisclosureMode::NotApplicableNoTreeIssueOrLandlordNotPropertyOwner,
            statutory_basis: "No tree issue identified".to_string(),
            notes: "No tree-removal or dangerous-tree issue identified; landlord has no actionable duty.".to_string(),
            citations,
        };
    }

    if input.tree_scenario == TreeScenario::DangerousTreeOnLandlordPropertyPosingFalls
        && input.tree_creates_foreseeable_risk_of_harm
    {
        match input.landlord_action {
            LandlordAction::AddressedRiskAndWarnedTenant => {
                return Output {
                    mode: TreeRemovalDangerousDisclosureMode::CompliantDangerousTreeDisclosedToTenant,
                    statutory_basis: "Landlord premises liability — addressed risk AND warned tenant".to_string(),
                    notes: "COMPLIANT: landlord addressed dangerous tree risk AND provided written warning to tenant; common law premises liability duty satisfied.".to_string(),
                    citations,
                };
            }
            LandlordAction::AddressedRiskOnly => {
                if !input.tenant_warned_in_writing_of_dangerous_tree {
                    return Output {
                        mode: TreeRemovalDangerousDisclosureMode::ViolationLandlordFailedToWarnTenantOfTreeHazard,
                        statutory_basis: "Landlord premises liability — failure to warn".to_string(),
                        notes: "VIOLATION: landlord addressed risk but failed to warn tenant in writing of dangerous tree hazard.".to_string(),
                        citations,
                    };
                }
                return Output {
                    mode: TreeRemovalDangerousDisclosureMode::CompliantDangerousTreeDisclosedToTenant,
                    statutory_basis: "Landlord premises liability — risk addressed + warning provided".to_string(),
                    notes: "COMPLIANT: landlord addressed dangerous tree risk and provided required warning.".to_string(),
                    citations,
                };
            }
            LandlordAction::WarnedTenantOnly => {
                return Output {
                    mode: TreeRemovalDangerousDisclosureMode::ViolationLandlordFailedToAddressDangerousTreeOnPremises,
                    statutory_basis: "Landlord premises liability — warning insufficient absent risk-addressing action".to_string(),
                    notes: "VIOLATION: landlord warned tenant but failed to address foreseeable risk of dangerous tree; warning alone is insufficient when remediation is possible.".to_string(),
                    citations,
                };
            }
            LandlordAction::NoActionTaken => {
                return Output {
                    mode: TreeRemovalDangerousDisclosureMode::ViolationLandlordFailedToAddressDangerousTreeOnPremises,
                    statutory_basis: "Landlord premises liability — failure to address known foreseeable risk".to_string(),
                    notes: "VIOLATION: landlord took no action despite foreseeable risk of harm from dangerous tree on premises; premises liability exposure.".to_string(),
                    citations,
                };
            }
            _ => {}
        }
    }

    if input.tree_scenario == TreeScenario::TreeOnLandlordPropertyOverhangingNeighbor
        && input.tree_imminently_dangerous_to_neighboring_property
    {
        match input.jurisdiction {
            TreeJurisdiction::HawaiiRuleStates
            | TreeJurisdiction::OtherStateCommonLawNegligenceRestatement => {
                if input.landlord_paid_neighbor_damages_for_tree_harm
                    || input.landlord_action == LandlordAction::PaidDamagesAndCutBackBranches
                {
                    return Output {
                        mode: TreeRemovalDangerousDisclosureMode::CompliantHawaiiRuleSelfHelpOrPayDamages,
                        statutory_basis: "Hawaii Rule (Whitesell v. Houlton, 1981) — pay damages + cut back".to_string(),
                        notes: "COMPLIANT Hawaii Rule: landlord paid neighbor damages and cut back endangering branches.".to_string(),
                        citations,
                    };
                }
                return Output {
                    mode: TreeRemovalDangerousDisclosureMode::ViolationHawaiiRuleNuisanceTreeDamagesNotPaid,
                    statutory_basis: "Hawaii Rule — nuisance tree damages owed".to_string(),
                    notes: "VIOLATION Hawaii Rule: tree imminently endangers neighbor's property but landlord did not pay damages or cut back branches.".to_string(),
                    citations,
                };
            }
            TreeJurisdiction::MassachusettsRuleStates => {
                return Output {
                    mode: TreeRemovalDangerousDisclosureMode::CompliantMassachusettsRuleSelfHelpOnly,
                    statutory_basis: "Massachusetts Rule (Michalson v. Nutting 1931) — no liability on landlord; neighbor's exclusive remedy is self-help".to_string(),
                    notes: "Massachusetts Rule jurisdiction: landlord has no liability for natural tree processes; neighbor's exclusive remedy is self-help to property line.".to_string(),
                    citations,
                };
            }
            _ => {}
        }
    }

    if input.tree_scenario == TreeScenario::NeighborTreeEncroachingOnLandlordProperty {
        match input.jurisdiction {
            TreeJurisdiction::CaliforniaBooskaPatelHybrid => {
                if input.landlord_action == LandlordAction::SelfHelpTrimmedDamagingTree {
                    return Output {
                        mode: TreeRemovalDangerousDisclosureMode::ViolationCaliforniaBooskaPatelOrdinaryCareNotExercised,
                        statutory_basis: "California Booska v. Patel (1994) — ordinary care required".to_string(),
                        notes: "VIOLATION Booska v. Patel: landlord trimmed encroaching tree in a manner that damaged or killed the tree; California requires ordinary care.".to_string(),
                        citations,
                    };
                }
                if input.landlord_action == LandlordAction::SelfHelpTrimmedWithOrdinaryCare {
                    return Output {
                        mode: TreeRemovalDangerousDisclosureMode::CompliantCaliforniaBooskaPatelOrdinaryCareExercised,
                        statutory_basis: "California Booska v. Patel — ordinary care exercised".to_string(),
                        notes: "COMPLIANT Booska v. Patel: landlord exercised ordinary care when trimming encroaching neighbor tree.".to_string(),
                        citations,
                    };
                }
                if input.landlord_action == LandlordAction::SelfHelpTrimmedToPropertyLine {
                    return Output {
                        mode: TreeRemovalDangerousDisclosureMode::ViolationLandlordTrimmedNeighborTreeWithoutOrdinaryCare,
                        statutory_basis: "California Booska v. Patel — pure property-line cut not absolute right".to_string(),
                        notes: "VIOLATION Booska v. Patel: California does NOT grant absolute right to trim to property line; trimming must be done with ordinary care.".to_string(),
                        citations,
                    };
                }
            }
            TreeJurisdiction::HawaiiRuleStates => {
                if input.landlord_paid_neighbor_damages_for_tree_harm
                    || matches!(
                        input.landlord_action,
                        LandlordAction::SelfHelpTrimmedToPropertyLine
                            | LandlordAction::PaidDamagesAndCutBackBranches
                    )
                {
                    return Output {
                        mode: TreeRemovalDangerousDisclosureMode::CompliantHawaiiRuleSelfHelpOrPayDamages,
                        statutory_basis: "Hawaii Rule — self-help permitted; pay-damages alternative also available".to_string(),
                        notes: "COMPLIANT Hawaii Rule: damaged neighbor (landlord) used self-help to cut back encroaching branches.".to_string(),
                        citations,
                    };
                }
            }
            TreeJurisdiction::MassachusettsRuleStates => {
                return Output {
                    mode: TreeRemovalDangerousDisclosureMode::CompliantMassachusettsRuleSelfHelpOnly,
                    statutory_basis: "Massachusetts Rule — self-help to property line is exclusive remedy".to_string(),
                    notes: "Massachusetts Rule: landlord may self-help trim to property line; no remedy against tree-owning neighbor.".to_string(),
                    citations,
                };
            }
            _ => {}
        }
    }

    if input.tree_scenario == TreeScenario::StormDamageFallenTreeOnTenantPremises {
        if input.tree_creates_foreseeable_risk_of_harm
            && input.landlord_action == LandlordAction::NoActionTaken
        {
            return Output {
                mode: TreeRemovalDangerousDisclosureMode::ViolationLandlordFailedToAddressDangerousTreeOnPremises,
                statutory_basis: "Common law negligence — foreseeable storm-damage tree fall".to_string(),
                notes: "VIOLATION: storm damage to tree was foreseeable; landlord failed to address and warn; tenant injuries support negligence claim.".to_string(),
                citations,
            };
        }
        return Output {
            mode: TreeRemovalDangerousDisclosureMode::CompliantNeighborTreeRiskAddressedByLandlord,
            statutory_basis: "Storm damage — landlord addressed risk where foreseeable".to_string(),
            notes:
                "COMPLIANT: storm-damaged tree addressed by landlord or not reasonably foreseeable."
                    .to_string(),
            citations,
        };
    }

    Output {
        mode: TreeRemovalDangerousDisclosureMode::CompliantNeighborTreeRiskAddressedByLandlord,
        statutory_basis: "No actionable tree liability triggered".to_string(),
        notes: format!(
            "No actionable tree liability under jurisdiction {:?} for scenario {:?} with landlord action {:?}.",
            input.jurisdiction, input.tree_scenario, input.landlord_action
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_california_compliant() -> Input {
        Input {
            jurisdiction: TreeJurisdiction::CaliforniaBooskaPatelHybrid,
            tree_scenario: TreeScenario::NeighborTreeEncroachingOnLandlordProperty,
            landlord_action: LandlordAction::SelfHelpTrimmedWithOrdinaryCare,
            tree_creates_foreseeable_risk_of_harm: false,
            tree_imminently_dangerous_to_neighboring_property: false,
            tenant_warned_in_writing_of_dangerous_tree: false,
            landlord_paid_neighbor_damages_for_tree_harm: false,
        }
    }

    #[test]
    fn no_tree_issue_not_applicable() {
        let input = Input {
            tree_scenario: TreeScenario::HealthySafeTreeNoIssue,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TreeRemovalDangerousDisclosureMode::NotApplicableNoTreeIssueOrLandlordNotPropertyOwner
        );
    }

    #[test]
    fn california_booska_patel_ordinary_care_compliant() {
        let result = check(&baseline_california_compliant());
        assert_eq!(
            result.mode,
            TreeRemovalDangerousDisclosureMode::CompliantCaliforniaBooskaPatelOrdinaryCareExercised
        );
    }

    #[test]
    fn california_self_help_damaging_tree_violation() {
        let input = Input {
            landlord_action: LandlordAction::SelfHelpTrimmedDamagingTree,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, TreeRemovalDangerousDisclosureMode::ViolationCaliforniaBooskaPatelOrdinaryCareNotExercised);
    }

    #[test]
    fn california_self_help_to_property_line_without_care_violation() {
        let input = Input {
            landlord_action: LandlordAction::SelfHelpTrimmedToPropertyLine,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, TreeRemovalDangerousDisclosureMode::ViolationLandlordTrimmedNeighborTreeWithoutOrdinaryCare);
    }

    #[test]
    fn hawaii_rule_self_help_compliant() {
        let input = Input {
            jurisdiction: TreeJurisdiction::HawaiiRuleStates,
            tree_scenario: TreeScenario::NeighborTreeEncroachingOnLandlordProperty,
            landlord_action: LandlordAction::SelfHelpTrimmedToPropertyLine,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TreeRemovalDangerousDisclosureMode::CompliantHawaiiRuleSelfHelpOrPayDamages
        );
    }

    #[test]
    fn massachusetts_rule_self_help_only_compliant() {
        let input = Input {
            jurisdiction: TreeJurisdiction::MassachusettsRuleStates,
            tree_scenario: TreeScenario::NeighborTreeEncroachingOnLandlordProperty,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TreeRemovalDangerousDisclosureMode::CompliantMassachusettsRuleSelfHelpOnly
        );
    }

    #[test]
    fn hawaii_rule_tree_endangers_neighbor_damages_paid_compliant() {
        let input = Input {
            jurisdiction: TreeJurisdiction::HawaiiRuleStates,
            tree_scenario: TreeScenario::TreeOnLandlordPropertyOverhangingNeighbor,
            tree_imminently_dangerous_to_neighboring_property: true,
            landlord_action: LandlordAction::PaidDamagesAndCutBackBranches,
            landlord_paid_neighbor_damages_for_tree_harm: true,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TreeRemovalDangerousDisclosureMode::CompliantHawaiiRuleSelfHelpOrPayDamages
        );
    }

    #[test]
    fn hawaii_rule_tree_endangers_neighbor_damages_not_paid_violation() {
        let input = Input {
            jurisdiction: TreeJurisdiction::HawaiiRuleStates,
            tree_scenario: TreeScenario::TreeOnLandlordPropertyOverhangingNeighbor,
            tree_imminently_dangerous_to_neighboring_property: true,
            landlord_action: LandlordAction::NoActionTaken,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TreeRemovalDangerousDisclosureMode::ViolationHawaiiRuleNuisanceTreeDamagesNotPaid
        );
    }

    #[test]
    fn massachusetts_rule_tree_endangers_neighbor_no_liability() {
        let input = Input {
            jurisdiction: TreeJurisdiction::MassachusettsRuleStates,
            tree_scenario: TreeScenario::TreeOnLandlordPropertyOverhangingNeighbor,
            tree_imminently_dangerous_to_neighboring_property: true,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TreeRemovalDangerousDisclosureMode::CompliantMassachusettsRuleSelfHelpOnly
        );
    }

    #[test]
    fn dangerous_tree_addressed_and_warned_compliant() {
        let input = Input {
            tree_scenario: TreeScenario::DangerousTreeOnLandlordPropertyPosingFalls,
            tree_creates_foreseeable_risk_of_harm: true,
            landlord_action: LandlordAction::AddressedRiskAndWarnedTenant,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TreeRemovalDangerousDisclosureMode::CompliantDangerousTreeDisclosedToTenant
        );
    }

    #[test]
    fn dangerous_tree_addressed_only_without_warning_violation() {
        let input = Input {
            tree_scenario: TreeScenario::DangerousTreeOnLandlordPropertyPosingFalls,
            tree_creates_foreseeable_risk_of_harm: true,
            landlord_action: LandlordAction::AddressedRiskOnly,
            tenant_warned_in_writing_of_dangerous_tree: false,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TreeRemovalDangerousDisclosureMode::ViolationLandlordFailedToWarnTenantOfTreeHazard
        );
    }

    #[test]
    fn dangerous_tree_warned_only_no_action_violation() {
        let input = Input {
            tree_scenario: TreeScenario::DangerousTreeOnLandlordPropertyPosingFalls,
            tree_creates_foreseeable_risk_of_harm: true,
            landlord_action: LandlordAction::WarnedTenantOnly,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, TreeRemovalDangerousDisclosureMode::ViolationLandlordFailedToAddressDangerousTreeOnPremises);
    }

    #[test]
    fn dangerous_tree_no_action_violation() {
        let input = Input {
            tree_scenario: TreeScenario::DangerousTreeOnLandlordPropertyPosingFalls,
            tree_creates_foreseeable_risk_of_harm: true,
            landlord_action: LandlordAction::NoActionTaken,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, TreeRemovalDangerousDisclosureMode::ViolationLandlordFailedToAddressDangerousTreeOnPremises);
    }

    #[test]
    fn storm_fallen_tree_foreseeable_no_action_violation() {
        let input = Input {
            tree_scenario: TreeScenario::StormDamageFallenTreeOnTenantPremises,
            tree_creates_foreseeable_risk_of_harm: true,
            landlord_action: LandlordAction::NoActionTaken,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, TreeRemovalDangerousDisclosureMode::ViolationLandlordFailedToAddressDangerousTreeOnPremises);
    }

    #[test]
    fn storm_fallen_tree_addressed_compliant() {
        let input = Input {
            tree_scenario: TreeScenario::StormDamageFallenTreeOnTenantPremises,
            tree_creates_foreseeable_risk_of_harm: false,
            landlord_action: LandlordAction::AddressedRiskAndWarnedTenant,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TreeRemovalDangerousDisclosureMode::CompliantNeighborTreeRiskAddressedByLandlord
        );
    }

    #[test]
    fn boundary_tree_mutual_ownership_default_compliant() {
        let input = Input {
            tree_scenario: TreeScenario::BoundaryTreeMutualOwnership,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TreeRemovalDangerousDisclosureMode::CompliantNeighborTreeRiskAddressedByLandlord
        );
    }

    #[test]
    fn citations_pin_hawaii_massachusetts_california_rules() {
        let result = check(&baseline_california_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("Whitesell v. Houlton"));
        assert!(joined.contains("1981"));
        assert!(joined.contains("63 Haw. 532"));
        assert!(joined.contains("Michalson v. Nutting"));
        assert!(joined.contains("Ponte v. DaSilva"));
        assert!(joined.contains("Booska v. Patel"));
        assert!(joined.contains("24 Cal.App.4th 1786"));
        assert!(joined.contains("ORDINARY CARE"));
        assert!(joined.contains("Cal. Civ. Code § 833"));
        assert!(joined.contains("Cal. Civ. Code § 834"));
        assert!(joined.contains("Cal. Civ. Code § 836"));
        assert!(joined.contains("Restatement (Second) of Torts § 363"));
        assert!(joined.contains("Restatement (Second) of Torts § 364"));
        assert!(joined.contains("Restatement (Second) of Torts § 840"));
        assert!(joined.contains("warranty of habitability"));
    }

    #[test]
    fn constant_pin_landmark_cases_and_codes() {
        assert_eq!(HAWAII_RULE_YEAR_WHITESELL_HOULTON, 1981);
        assert_eq!(CALIFORNIA_BOOSKA_PATEL_YEAR, 1994);
        assert_eq!(CALIFORNIA_BOOSKA_PATEL_CITATION_VOLUME, 24);
        assert_eq!(CALIFORNIA_BOOSKA_PATEL_CITATION_PAGE, 1786);
        assert_eq!(MASSACHUSETTS_RULE_MICHALSON_NUTTING_YEAR, 1931);
        assert_eq!(MASSACHUSETTS_RULE_PONTE_DASILVA_REAFFIRMATION_YEAR, 1985);
        assert_eq!(CALIFORNIA_CIV_CODE_833, 833);
        assert_eq!(CALIFORNIA_CIV_CODE_834_BOUNDARY, 834);
        assert_eq!(CALIFORNIA_CIV_CODE_836_NUISANCE, 836);
        assert_eq!(RESTATEMENT_SECOND_TORTS_363_NATURAL, 363);
        assert_eq!(RESTATEMENT_SECOND_TORTS_364_ARTIFICIAL, 364);
        assert_eq!(RESTATEMENT_SECOND_TORTS_840_ENCROACHING, 840);
    }
}
