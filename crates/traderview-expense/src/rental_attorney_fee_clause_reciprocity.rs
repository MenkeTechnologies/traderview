//! Residential Lease Attorney Fee Clause Reciprocity / Mutualization
//! Compliance Module.
//!
//! Bread-and-butter residential lease provision: when a landlord
//! drafts a lease awarding attorneys' fees and costs to landlord
//! upon tenant default, several states impose AUTOMATIC STATUTORY
//! MUTUALIZATION — the unilateral clause is read as bilateral by
//! operation of law, entitling the tenant to fees if the tenant
//! prevails. Some states (Florida residential under § 83.48) go
//! further and impose a NON-WAIVABLE prevailing-party fee right
//! regardless of whether the lease contains a fee clause.
//!
//! Web research (verified 2026-06-03):
//! - **California Civ. Code § 1717** — "in any action on a contract,
//!   where the contract specifically provides that attorney's fees
//!   and costs incurred to enforce that contract shall be awarded
//!   to one of the parties or to the prevailing party, then the
//!   party who is determined to be the party prevailing on the
//!   contract … shall be entitled to reasonable attorney's fees in
//!   addition to other costs." The Legislature's stated purpose is
//!   to protect the weaker contracting party by making contractual
//!   fee clauses bilateral; "any provision in a contract which
//!   provides for a waiver of attorney's fees is void". (Cornell
//!   LII; California Legislative Information; FindLaw Civ Code
//!   § 1717.)
//! - **Florida Stat. § 83.48 (Residential Landlord and Tenant Act)**
//!   — "In a civil action brought to enforce the provisions of the
//!   rental agreement or this part, the party in whose favor a
//!   judgment or decree has been rendered may recover reasonable
//!   attorney fees and court costs from the nonprevailing party.
//!   The right to attorney fees in this section may not be waived
//!   in a lease agreement. Attorney fees may not be awarded under
//!   this section in a claim for personal injury damages based on
//!   a breach of duty under s. 83.51." (Florida Senate official
//!   statute text; YouTube: §83.48 explainer; Arias Bosinger
//!   Lacquaniti — eviction statute strictly construed.)
//! - **Florida Stat. § 57.105(7)** — general statutory bilateral
//!   reading of unilateral attorney-fee clauses in any Florida
//!   contract. Seven states (FL, OR, WA, MT, OK, NC, HI) recognize
//!   bilateral reading regardless of contract type per the
//!   referenced commentary.
//! - **New York Real Property Law § 234** — RESIDENTIAL leases that
//!   include landlord-favor attorney-fee clause are automatically
//!   read to include a covenant by the landlord to pay the tenant's
//!   reasonable attorney fees and expenses incurred as a result of
//!   tenant's successful defense of any action. Non-waivable.
//! - **Washington RCW 4.84.330** — bilateral reading of unilateral
//!   contractual attorney-fee provisions; prevailing-party rule.
//! - **Oregon ORS 20.096** — reciprocal attorney-fee provisions;
//!   automatically mutualized.
//! - **Texas Prop. Code § 92.0563(c)** — tenant may recover
//!   reasonable attorney fees against landlord for violations;
//!   landlord-favor clauses enforceable as written without
//!   statutory mutualization for landlord side.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const ATTORNEY_FEE_CALIFORNIA_CIV_CODE_SECTION: u32 = 1717;
pub const ATTORNEY_FEE_FLORIDA_RESIDENTIAL_STATUTE_PART: u32 = 83;
pub const ATTORNEY_FEE_FLORIDA_RESIDENTIAL_STATUTE_SECTION: u32 = 48;
pub const ATTORNEY_FEE_FLORIDA_GENERAL_CONTRACT_STATUTE_SECTION: u32 = 57;
pub const ATTORNEY_FEE_NEW_YORK_RPL_SECTION: u32 = 234;
pub const ATTORNEY_FEE_WASHINGTON_RCW_TITLE: u32 = 4;
pub const ATTORNEY_FEE_WASHINGTON_RCW_CHAPTER: u32 = 84;
pub const ATTORNEY_FEE_WASHINGTON_RCW_SECTION: u32 = 330;
pub const ATTORNEY_FEE_OREGON_ORS_SECTION: u32 = 2096;
pub const ATTORNEY_FEE_TEXAS_PROP_CODE_SECTION: u32 = 920563;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    Florida,
    NewYork,
    Washington,
    Oregon,
    Texas,
    OtherStateWithoutMutualizationStatute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseAttorneyFeeClauseType {
    NoClause,
    UnilateralLandlordFavor,
    UnilateralTenantFavor,
    Bilateral,
    BilateralWithCap,
    LandlordFavorWithExplicitNonWaiverOverride,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PrevailingParty {
    Landlord,
    Tenant,
    NoJudgmentYet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttorneyFeeReciprocityMode {
    NotApplicable,
    CompliantBilateralClauseOriginalDrafting,
    CompliantUnilateralClauseStatutoryMutualization,
    CompliantFloridaPrevailingPartyByOperationOfStatuteNoClause,
    CompliantPersonalInjuryClaimExcludedPerFloridaStatute,
    ViolationUnilateralClauseInJurisdictionWithoutMutualizationStatute,
    ViolationLeaseAttemptedToWaiveStatutoryReciprocity,
    ViolationLeaseFeesCapBelowReasonablenessStandard,
    ViolationPersonalInjuryClaimAttemptedToInvokeFeeShifting,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub lease_clause_type: LeaseAttorneyFeeClauseType,
    pub claim_is_personal_injury_under_florida_section_83_51: bool,
    pub lease_caps_fees_at_amount_cents: Option<u64>,
    pub reasonable_fees_incurred_cents: u64,
    pub prevailing_party: PrevailingParty,
    pub lease_purports_to_waive_florida_section_83_48_reciprocity: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: AttorneyFeeReciprocityMode,
    pub recoverable_fees_cents: u64,
    pub statutory_basis_for_recovery: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalAttorneyFeeClauseReciprocityInput = Input;
pub type RentalAttorneyFeeClauseReciprocityOutput = Output;
pub type RentalAttorneyFeeClauseReciprocityResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

fn jurisdiction_has_automatic_mutualization_statute(j: Jurisdiction) -> bool {
    matches!(
        j,
        Jurisdiction::California
            | Jurisdiction::Florida
            | Jurisdiction::NewYork
            | Jurisdiction::Washington
            | Jurisdiction::Oregon
    )
}

fn florida_imposes_prevailing_party_fee_even_without_clause(j: Jurisdiction) -> bool {
    matches!(j, Jurisdiction::Florida)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Cal. Civ. Code § 1717 — unilateral attorney-fee clause read as bilateral; waiver void".to_string(),
        "Fla. Stat. § 83.48 — Florida Residential Landlord and Tenant Act prevailing-party attorney fees; right NOT WAIVABLE in lease; § 83.51 personal injury claims EXCLUDED".to_string(),
        "Fla. Stat. § 57.105(7) — general bilateral reading of unilateral attorney-fee provisions in Florida contracts".to_string(),
        "N.Y. Real Property Law § 234 — residential lease landlord-favor attorney-fee clause read as covenant by landlord to pay tenant's fees on tenant's successful defense; non-waivable".to_string(),
        "Wash. RCW 4.84.330 — bilateral reading of unilateral contractual attorney-fee provisions".to_string(),
        "Or. ORS 20.096 — reciprocal attorney-fee provisions automatically mutualized".to_string(),
        "Tex. Prop. Code § 92.0563(c) — tenant remedies; no automatic mutualization of landlord-favor clauses".to_string(),
    ];

    if input.lease_clause_type == LeaseAttorneyFeeClauseType::NoClause {
        if florida_imposes_prevailing_party_fee_even_without_clause(input.jurisdiction) {
            if input.claim_is_personal_injury_under_florida_section_83_51 {
                return Output {
                    mode: AttorneyFeeReciprocityMode::CompliantPersonalInjuryClaimExcludedPerFloridaStatute,
                    recoverable_fees_cents: 0,
                    statutory_basis_for_recovery: "Fla. Stat. § 83.48 excludes personal injury claims under § 83.51".to_string(),
                    notes: "No fee shifting under § 83.48 because claim is § 83.51 personal injury (statutory carve-out).".to_string(),
                    citations,
                };
            }
            let recoverable = if input.prevailing_party == PrevailingParty::NoJudgmentYet {
                0
            } else {
                input.reasonable_fees_incurred_cents
            };
            return Output {
                mode: AttorneyFeeReciprocityMode::CompliantFloridaPrevailingPartyByOperationOfStatuteNoClause,
                recoverable_fees_cents: recoverable,
                statutory_basis_for_recovery: "Fla. Stat. § 83.48 — prevailing-party fees by operation of statute; no lease clause required".to_string(),
                notes: format!(
                    "Florida § 83.48 imposes prevailing-party attorney-fee right regardless of lease silence. Prevailing party = {:?}; reasonable fees = {} cents recovered.",
                    input.prevailing_party, recoverable
                ),
                citations,
            };
        }
        return Output {
            mode: AttorneyFeeReciprocityMode::NotApplicable,
            recoverable_fees_cents: 0,
            statutory_basis_for_recovery: "American Rule — each party bears own fees absent contract or statute".to_string(),
            notes: "Lease contains no attorney-fee clause and jurisdiction has no statutory prevailing-party rule; American Rule applies.".to_string(),
            citations,
        };
    }

    if input.claim_is_personal_injury_under_florida_section_83_51
        && input.jurisdiction == Jurisdiction::Florida
    {
        return Output {
            mode: AttorneyFeeReciprocityMode::ViolationPersonalInjuryClaimAttemptedToInvokeFeeShifting,
            recoverable_fees_cents: 0,
            statutory_basis_for_recovery: "None — Fla. Stat. § 83.48 personal-injury exclusion".to_string(),
            notes: format!(
                "VIOLATION: claim is § 83.51 personal injury — Fla. Stat. § 83.48 expressly excludes fee shifting for personal injury claims based on § 83.51 duties (habitability). Lease attorney-fee clause of type {:?} cannot override statutory carve-out.",
                input.lease_clause_type
            ),
            citations,
        };
    }

    if input.lease_purports_to_waive_florida_section_83_48_reciprocity
        && input.jurisdiction == Jurisdiction::Florida
    {
        let recoverable = if input.prevailing_party == PrevailingParty::NoJudgmentYet {
            0
        } else {
            input.reasonable_fees_incurred_cents
        };
        return Output {
            mode: AttorneyFeeReciprocityMode::ViolationLeaseAttemptedToWaiveStatutoryReciprocity,
            recoverable_fees_cents: recoverable,
            statutory_basis_for_recovery: "Fla. Stat. § 83.48 — non-waivable right; lease waiver void".to_string(),
            notes: format!(
                "VIOLATION: lease purports to waive § 83.48 prevailing-party right. Statute explicitly states right may not be waived in a lease agreement. Waiver provision VOID. Prevailing party = {:?} recovers reasonable fees of {} cents.",
                input.prevailing_party, recoverable
            ),
            citations,
        };
    }

    if matches!(
        input.lease_clause_type,
        LeaseAttorneyFeeClauseType::Bilateral | LeaseAttorneyFeeClauseType::BilateralWithCap
    ) {
        let recoverable = match (input.prevailing_party, input.lease_clause_type) {
            (PrevailingParty::NoJudgmentYet, _) => 0,
            (_, LeaseAttorneyFeeClauseType::BilateralWithCap) => {
                let cap = input.lease_caps_fees_at_amount_cents.unwrap_or(0);
                let recoverable = input.reasonable_fees_incurred_cents.min(cap);
                if input.jurisdiction == Jurisdiction::California
                    && cap < input.reasonable_fees_incurred_cents
                {
                    return Output {
                        mode: AttorneyFeeReciprocityMode::ViolationLeaseFeesCapBelowReasonablenessStandard,
                        recoverable_fees_cents: input.reasonable_fees_incurred_cents,
                        statutory_basis_for_recovery: "Cal. Civ. Code § 1717 — reasonable fees standard; lease cap below reasonableness void".to_string(),
                        notes: format!(
                            "VIOLATION: California lease bilateral attorney-fee clause caps fees at {} cents but reasonable fees incurred = {} cents. Under § 1717, the prevailing party is entitled to REASONABLE attorney fees; contractual cap below the reasonable amount is void as inconsistent with statutory standard. Recoverable = {} cents (full reasonable amount).",
                            cap, input.reasonable_fees_incurred_cents, input.reasonable_fees_incurred_cents
                        ),
                        citations,
                    };
                }
                recoverable
            }
            (_, _) => input.reasonable_fees_incurred_cents,
        };
        return Output {
            mode: AttorneyFeeReciprocityMode::CompliantBilateralClauseOriginalDrafting,
            recoverable_fees_cents: recoverable,
            statutory_basis_for_recovery: "Lease bilateral attorney-fee clause as drafted".to_string(),
            notes: format!(
                "COMPLIANT: bilateral attorney-fee clause as originally drafted. Prevailing party = {:?}; recoverable fees = {} cents (reasonable fees incurred = {} cents).",
                input.prevailing_party, recoverable, input.reasonable_fees_incurred_cents
            ),
            citations,
        };
    }

    let is_unilateral = matches!(
        input.lease_clause_type,
        LeaseAttorneyFeeClauseType::UnilateralLandlordFavor
            | LeaseAttorneyFeeClauseType::UnilateralTenantFavor
            | LeaseAttorneyFeeClauseType::LandlordFavorWithExplicitNonWaiverOverride
    );

    if is_unilateral && !jurisdiction_has_automatic_mutualization_statute(input.jurisdiction) {
        let recoverable = match input.prevailing_party {
            PrevailingParty::NoJudgmentYet => 0,
            PrevailingParty::Landlord => {
                if input.lease_clause_type == LeaseAttorneyFeeClauseType::UnilateralLandlordFavor
                    || input.lease_clause_type
                        == LeaseAttorneyFeeClauseType::LandlordFavorWithExplicitNonWaiverOverride
                {
                    input.reasonable_fees_incurred_cents
                } else {
                    0
                }
            }
            PrevailingParty::Tenant => {
                if input.lease_clause_type == LeaseAttorneyFeeClauseType::UnilateralTenantFavor {
                    input.reasonable_fees_incurred_cents
                } else {
                    0
                }
            }
        };
        return Output {
            mode: AttorneyFeeReciprocityMode::ViolationUnilateralClauseInJurisdictionWithoutMutualizationStatute,
            recoverable_fees_cents: recoverable,
            statutory_basis_for_recovery: "None — jurisdiction recognizes unilateral attorney-fee clauses as written".to_string(),
            notes: format!(
                "VIOLATION (drafting): jurisdiction = {:?} has no automatic mutualization statute (Texas-like). Unilateral attorney-fee clause type {:?} is enforced AS WRITTEN. Asymmetric outcome: prevailing party = {:?}; recoverable fees = {} cents under one-way clause.",
                input.jurisdiction, input.lease_clause_type, input.prevailing_party, recoverable
            ),
            citations,
        };
    }

    if is_unilateral && jurisdiction_has_automatic_mutualization_statute(input.jurisdiction) {
        let recoverable = if input.prevailing_party == PrevailingParty::NoJudgmentYet {
            0
        } else {
            input.reasonable_fees_incurred_cents
        };
        return Output {
            mode: AttorneyFeeReciprocityMode::CompliantUnilateralClauseStatutoryMutualization,
            recoverable_fees_cents: recoverable,
            statutory_basis_for_recovery: format!(
                "Statutory mutualization applicable to jurisdiction {:?} (CA § 1717 / FL § 83.48 / NY RPL § 234 / WA RCW 4.84.330 / OR ORS 20.096)",
                input.jurisdiction
            ),
            notes: format!(
                "COMPLIANT (statutory mutualization): unilateral lease clause type {:?} is read as bilateral by operation of statute in jurisdiction {:?}. Prevailing party = {:?} recovers reasonable fees of {} cents regardless of clause asymmetry.",
                input.lease_clause_type, input.jurisdiction, input.prevailing_party, recoverable
            ),
            citations,
        };
    }

    Output {
        mode: AttorneyFeeReciprocityMode::NotApplicable,
        recoverable_fees_cents: 0,
        statutory_basis_for_recovery: "Default fall-through".to_string(),
        notes: "Unhandled combination; default to American Rule.".to_string(),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_ca_unilateral_ll_favor_tenant_prevails() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            lease_clause_type: LeaseAttorneyFeeClauseType::UnilateralLandlordFavor,
            claim_is_personal_injury_under_florida_section_83_51: false,
            lease_caps_fees_at_amount_cents: None,
            reasonable_fees_incurred_cents: 1_500_000,
            prevailing_party: PrevailingParty::Tenant,
            lease_purports_to_waive_florida_section_83_48_reciprocity: false,
        }
    }

    #[test]
    fn california_mutualizes_unilateral_landlord_favor_clause_for_tenant() {
        let result = compute(&baseline_ca_unilateral_ll_favor_tenant_prevails());
        assert_eq!(
            result.mode,
            AttorneyFeeReciprocityMode::CompliantUnilateralClauseStatutoryMutualization
        );
        assert_eq!(result.recoverable_fees_cents, 1_500_000);
        assert!(result.notes.contains("California"));
    }

    #[test]
    fn florida_prevailing_party_no_clause_landlord_recovers() {
        let input = Input {
            jurisdiction: Jurisdiction::Florida,
            lease_clause_type: LeaseAttorneyFeeClauseType::NoClause,
            claim_is_personal_injury_under_florida_section_83_51: false,
            lease_caps_fees_at_amount_cents: None,
            reasonable_fees_incurred_cents: 800_000,
            prevailing_party: PrevailingParty::Landlord,
            lease_purports_to_waive_florida_section_83_48_reciprocity: false,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            AttorneyFeeReciprocityMode::CompliantFloridaPrevailingPartyByOperationOfStatuteNoClause
        );
        assert_eq!(result.recoverable_fees_cents, 800_000);
    }

    #[test]
    fn florida_personal_injury_no_clause_excluded() {
        let input = Input {
            jurisdiction: Jurisdiction::Florida,
            lease_clause_type: LeaseAttorneyFeeClauseType::NoClause,
            claim_is_personal_injury_under_florida_section_83_51: true,
            lease_caps_fees_at_amount_cents: None,
            reasonable_fees_incurred_cents: 800_000,
            prevailing_party: PrevailingParty::Tenant,
            lease_purports_to_waive_florida_section_83_48_reciprocity: false,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            AttorneyFeeReciprocityMode::CompliantPersonalInjuryClaimExcludedPerFloridaStatute
        );
        assert_eq!(result.recoverable_fees_cents, 0);
    }

    #[test]
    fn florida_personal_injury_with_clause_violation() {
        let input = Input {
            jurisdiction: Jurisdiction::Florida,
            lease_clause_type: LeaseAttorneyFeeClauseType::UnilateralLandlordFavor,
            claim_is_personal_injury_under_florida_section_83_51: true,
            lease_caps_fees_at_amount_cents: None,
            reasonable_fees_incurred_cents: 800_000,
            prevailing_party: PrevailingParty::Landlord,
            lease_purports_to_waive_florida_section_83_48_reciprocity: false,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            AttorneyFeeReciprocityMode::ViolationPersonalInjuryClaimAttemptedToInvokeFeeShifting
        );
        assert_eq!(result.recoverable_fees_cents, 0);
        assert!(result.notes.contains("§ 83.51"));
    }

    #[test]
    fn florida_lease_attempt_to_waive_reciprocity_violation() {
        let input = Input {
            jurisdiction: Jurisdiction::Florida,
            lease_clause_type: LeaseAttorneyFeeClauseType::UnilateralLandlordFavor,
            claim_is_personal_injury_under_florida_section_83_51: false,
            lease_caps_fees_at_amount_cents: None,
            reasonable_fees_incurred_cents: 900_000,
            prevailing_party: PrevailingParty::Tenant,
            lease_purports_to_waive_florida_section_83_48_reciprocity: true,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            AttorneyFeeReciprocityMode::ViolationLeaseAttemptedToWaiveStatutoryReciprocity
        );
        assert_eq!(result.recoverable_fees_cents, 900_000);
        assert!(result.notes.contains("VOID"));
    }

    #[test]
    fn california_bilateral_with_cap_below_reasonable_violation() {
        let input = Input {
            jurisdiction: Jurisdiction::California,
            lease_clause_type: LeaseAttorneyFeeClauseType::BilateralWithCap,
            claim_is_personal_injury_under_florida_section_83_51: false,
            lease_caps_fees_at_amount_cents: Some(500_000),
            reasonable_fees_incurred_cents: 1_500_000,
            prevailing_party: PrevailingParty::Tenant,
            lease_purports_to_waive_florida_section_83_48_reciprocity: false,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            AttorneyFeeReciprocityMode::ViolationLeaseFeesCapBelowReasonablenessStandard
        );
        assert_eq!(result.recoverable_fees_cents, 1_500_000);
        assert!(result.notes.contains("§ 1717"));
    }

    #[test]
    fn texas_unilateral_landlord_favor_enforced_as_written_violation() {
        let input = Input {
            jurisdiction: Jurisdiction::Texas,
            lease_clause_type: LeaseAttorneyFeeClauseType::UnilateralLandlordFavor,
            claim_is_personal_injury_under_florida_section_83_51: false,
            lease_caps_fees_at_amount_cents: None,
            reasonable_fees_incurred_cents: 1_000_000,
            prevailing_party: PrevailingParty::Landlord,
            lease_purports_to_waive_florida_section_83_48_reciprocity: false,
        };
        let result = compute(&input);
        assert_eq!(result.mode, AttorneyFeeReciprocityMode::ViolationUnilateralClauseInJurisdictionWithoutMutualizationStatute);
        assert_eq!(result.recoverable_fees_cents, 1_000_000);
    }

    #[test]
    fn texas_unilateral_landlord_favor_tenant_prevails_no_recovery() {
        let input = Input {
            jurisdiction: Jurisdiction::Texas,
            lease_clause_type: LeaseAttorneyFeeClauseType::UnilateralLandlordFavor,
            claim_is_personal_injury_under_florida_section_83_51: false,
            lease_caps_fees_at_amount_cents: None,
            reasonable_fees_incurred_cents: 1_000_000,
            prevailing_party: PrevailingParty::Tenant,
            lease_purports_to_waive_florida_section_83_48_reciprocity: false,
        };
        let result = compute(&input);
        assert_eq!(result.mode, AttorneyFeeReciprocityMode::ViolationUnilateralClauseInJurisdictionWithoutMutualizationStatute);
        assert_eq!(result.recoverable_fees_cents, 0);
    }

    #[test]
    fn new_york_mutualizes_unilateral_landlord_favor_under_rpl_234() {
        let input = Input {
            jurisdiction: Jurisdiction::NewYork,
            lease_clause_type: LeaseAttorneyFeeClauseType::UnilateralLandlordFavor,
            claim_is_personal_injury_under_florida_section_83_51: false,
            lease_caps_fees_at_amount_cents: None,
            reasonable_fees_incurred_cents: 750_000,
            prevailing_party: PrevailingParty::Tenant,
            lease_purports_to_waive_florida_section_83_48_reciprocity: false,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            AttorneyFeeReciprocityMode::CompliantUnilateralClauseStatutoryMutualization
        );
        assert_eq!(result.recoverable_fees_cents, 750_000);
    }

    #[test]
    fn washington_mutualizes_under_rcw_4_84_330() {
        let input = Input {
            jurisdiction: Jurisdiction::Washington,
            lease_clause_type: LeaseAttorneyFeeClauseType::UnilateralLandlordFavor,
            claim_is_personal_injury_under_florida_section_83_51: false,
            lease_caps_fees_at_amount_cents: None,
            reasonable_fees_incurred_cents: 600_000,
            prevailing_party: PrevailingParty::Tenant,
            lease_purports_to_waive_florida_section_83_48_reciprocity: false,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            AttorneyFeeReciprocityMode::CompliantUnilateralClauseStatutoryMutualization
        );
        assert_eq!(result.recoverable_fees_cents, 600_000);
    }

    #[test]
    fn oregon_mutualizes_under_ors_20_096() {
        let input = Input {
            jurisdiction: Jurisdiction::Oregon,
            lease_clause_type: LeaseAttorneyFeeClauseType::UnilateralLandlordFavor,
            claim_is_personal_injury_under_florida_section_83_51: false,
            lease_caps_fees_at_amount_cents: None,
            reasonable_fees_incurred_cents: 400_000,
            prevailing_party: PrevailingParty::Landlord,
            lease_purports_to_waive_florida_section_83_48_reciprocity: false,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            AttorneyFeeReciprocityMode::CompliantUnilateralClauseStatutoryMutualization
        );
        assert_eq!(result.recoverable_fees_cents, 400_000);
    }

    #[test]
    fn bilateral_clause_as_drafted_compliant() {
        let input = Input {
            jurisdiction: Jurisdiction::California,
            lease_clause_type: LeaseAttorneyFeeClauseType::Bilateral,
            claim_is_personal_injury_under_florida_section_83_51: false,
            lease_caps_fees_at_amount_cents: None,
            reasonable_fees_incurred_cents: 1_200_000,
            prevailing_party: PrevailingParty::Landlord,
            lease_purports_to_waive_florida_section_83_48_reciprocity: false,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            AttorneyFeeReciprocityMode::CompliantBilateralClauseOriginalDrafting
        );
        assert_eq!(result.recoverable_fees_cents, 1_200_000);
    }

    #[test]
    fn no_judgment_yet_zero_recovery() {
        let mut input = baseline_ca_unilateral_ll_favor_tenant_prevails();
        input.prevailing_party = PrevailingParty::NoJudgmentYet;
        let result = compute(&input);
        assert_eq!(result.recoverable_fees_cents, 0);
    }

    #[test]
    fn american_rule_other_state_no_clause() {
        let input = Input {
            jurisdiction: Jurisdiction::OtherStateWithoutMutualizationStatute,
            lease_clause_type: LeaseAttorneyFeeClauseType::NoClause,
            claim_is_personal_injury_under_florida_section_83_51: false,
            lease_caps_fees_at_amount_cents: None,
            reasonable_fees_incurred_cents: 500_000,
            prevailing_party: PrevailingParty::Landlord,
            lease_purports_to_waive_florida_section_83_48_reciprocity: false,
        };
        let result = compute(&input);
        assert_eq!(result.mode, AttorneyFeeReciprocityMode::NotApplicable);
        assert_eq!(result.recoverable_fees_cents, 0);
        assert!(result
            .statutory_basis_for_recovery
            .contains("American Rule"));
    }

    #[test]
    fn citations_pin_jurisdictional_statutes() {
        let result = compute(&baseline_ca_unilateral_ll_favor_tenant_prevails());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("Cal. Civ. Code § 1717"));
        assert!(joined.contains("Fla. Stat. § 83.48"));
        assert!(joined.contains("Fla. Stat. § 57.105"));
        assert!(joined.contains("N.Y. Real Property Law § 234"));
        assert!(joined.contains("Wash. RCW 4.84.330"));
        assert!(joined.contains("Or. ORS 20.096"));
        assert!(joined.contains("Tex. Prop. Code § 92.0563"));
    }

    #[test]
    fn constant_pin_california_civ_code_1717() {
        assert_eq!(ATTORNEY_FEE_CALIFORNIA_CIV_CODE_SECTION, 1717);
        assert_eq!(ATTORNEY_FEE_FLORIDA_RESIDENTIAL_STATUTE_PART, 83);
        assert_eq!(ATTORNEY_FEE_FLORIDA_RESIDENTIAL_STATUTE_SECTION, 48);
        assert_eq!(ATTORNEY_FEE_NEW_YORK_RPL_SECTION, 234);
    }

    #[test]
    fn florida_landlord_favor_with_non_waiver_override_no_special_treatment() {
        let input = Input {
            jurisdiction: Jurisdiction::Florida,
            lease_clause_type:
                LeaseAttorneyFeeClauseType::LandlordFavorWithExplicitNonWaiverOverride,
            claim_is_personal_injury_under_florida_section_83_51: false,
            lease_caps_fees_at_amount_cents: None,
            reasonable_fees_incurred_cents: 700_000,
            prevailing_party: PrevailingParty::Tenant,
            lease_purports_to_waive_florida_section_83_48_reciprocity: false,
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            AttorneyFeeReciprocityMode::CompliantUnilateralClauseStatutoryMutualization
        );
        assert_eq!(result.recoverable_fees_cents, 700_000);
    }
}
