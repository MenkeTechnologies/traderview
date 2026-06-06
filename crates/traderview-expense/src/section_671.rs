//! IRC § 671 — Trust Income, Deductions, and Credits Attributable
//! to Grantors and Others as Substantial Owners (Grantor Trust
//! General Rule).
//!
//! Pure-compute attribution rule. When a grantor or another
//! person is treated as the owner of any portion of a trust under
//! the §§ 673-679 substantive trigger sections, that person
//! INCLUDES items of income, deductions, and credits attributable
//! to that portion in computing their own taxable income and
//! credits — the trust is TRANSPARENT for income tax purposes
//! while remaining separately respected for gift and estate tax
//! purposes (the "intentionally defective" feature).
//!
//! Trader-critical because grantor trust status is the foundation
//! of: **IDGT** (Intentionally Defective Grantor Trust) for
//! freeze-and-squeeze estate planning; **GRAT** (Grantor Retained
//! Annuity Trust) under § 673; **CRT** (Charitable Remainder
//! Trust) limited grantor trust status; **BDIT** (Beneficiary
//! Defective Inheritor Trust) under § 678; **CLAT** charitable
//! lead annuity trust grantor status; **§ 675(4)(C) substitution
//! power** — most common modern "intentional defect" because it
//! triggers § 671 attribution while not affecting estate
//! inclusion.
//!
//! Statute (verbatim mapping):
//! - § 671 — GENERAL RULE: where it is specified in this subpart
//!   that the grantor or another person shall be treated as the
//!   owner of any portion of a trust, there shall then be included
//!   in computing the TAXABLE INCOME AND CREDITS of the grantor or
//!   the other person those items of income, deductions, and
//!   credits against tax of the trust which are attributable to
//!   that portion of the trust to the extent that such items
//!   would be taken into account under this chapter in computing
//!   taxable income or credits against the tax of an individual.
//! - § 672 — DEFINITIONS: grantor, beneficiary, adverse party,
//!   nonadverse party, related or subordinate party.
//! - § 673 — REVERSIONARY INTERESTS: grantor treated as owner of
//!   any portion of a trust if grantor's reversionary interest >
//!   5 % of the value of such portion at trust inception
//!   (post-1986 standard; pre-1986: 10-year rule).
//! - § 674 — POWER TO CONTROL BENEFICIAL ENJOYMENT: grantor
//!   treated as owner if grantor (or nonadverse party without
//!   adverse party consent) has power to control beneficial
//!   enjoyment of corpus or income — BROAD trigger with extensive
//!   § 674(b) and § 674(c) exceptions.
//! - § 675 — ADMINISTRATIVE POWERS: grantor treated as owner if
//!   grantor has — (1) power to deal with trust at less than
//!   adequate consideration; (2) power to BORROW without adequate
//!   security; (3) actual borrowing without timely repayment;
//!   (4) general administrative powers exercisable in nonfiduciary
//!   capacity, including § 675(4)(C) POWER TO SUBSTITUTE TRUST
//!   ASSETS of equivalent value (most common modern grantor trust
//!   trigger).
//! - § 676 — POWER TO REVOKE: grantor treated as owner of any
//!   portion of a trust where the grantor or a nonadverse party
//!   has the power to revest title to that portion in the grantor.
//! - § 677 — INCOME FOR BENEFIT OF GRANTOR: grantor treated as
//!   owner of any portion of a trust whose income — (1) is or may
//!   be distributed to grantor or grantor's spouse; (2) may be
//!   accumulated for future distribution to grantor or spouse;
//!   (3) is applied to pay premiums on insurance policies on
//!   grantor's or spouse's life.
//! - § 678 — PERSON OTHER THAN GRANTOR TREATED AS OWNER: a person
//!   other than the grantor shall be treated as the owner of any
//!   portion of a trust with respect to which (1) such person has
//!   a power exercisable solely by self to vest the corpus or
//!   income therefrom in self, OR (2) such person has previously
//!   partially released or otherwise modified such a power and
//!   the lapse meets the § 678(b)(2) standard. Foundation of
//!   BDIT (Beneficiary Defective Inheritor Trust) planning.
//! - § 679 — FOREIGN TRUSTS WITH UNITED STATES BENEFICIARIES:
//!   U.S. transferor to foreign trust with U.S. beneficiary
//!   treated as owner — anti-deferral rule.
//!
//! Web research (verified 2026-06-03):
//! - Cornell LII § 671 confirms statutory text.
//! - IRS Rev. Rul. 2023-2 confirms grantor's death does NOT
//!   trigger basis step-up on IDGT assets (closes prior planning
//!   gap).
//! - Freeman Law Grantor Trusts guide confirms policy rationale
//!   (grantor's dominion and control = grantor's tax burden).
//! - 26 CFR Part 1 implementing regulations.
//! - Bradford Tax Institute IRC §§ 671-678 endnotes confirm
//!   modern post-1986 standards.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const SECTION_673_REVERSIONARY_INTEREST_THRESHOLD_BASIS_POINTS: u64 = 500;
pub const SECTION_678_B_2_LAPSE_5X5_RULE_GREATER_OF_DOLLARS: u64 = 5_000;
pub const SECTION_678_B_2_LAPSE_5X5_RULE_TRUST_CORPUS_PCT_BASIS_POINTS: u64 = 500;
pub const SECTION_671_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const SECTION_675_4C_SUBSTITUTION_POWER_AVAILABLE: bool = true;
pub const TAX_REFORM_ACT_1986_SECTION_673_EFFECTIVE_YEAR: u32 = 1986;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GrantorTriggerSection {
    Section673ReversionaryOver5Pct,
    Section674PowerControlBeneficialEnjoyment,
    Section675AdministrativePower,
    Section675_4cSubstitutionPower,
    Section676PowerToRevoke,
    Section677IncomeAccumulatedForGrantor,
    Section678ThirdPartyOwnerBdit,
    Section679ForeignTrustUsBeneficiary,
    NoTriggerStandardNonGrantorTrust,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowThroughItems {
    IncomeOnly,
    DeductionsOnly,
    CreditsOnly,
    IncomeAndDeductionsOnly,
    AllThreeIncomeDeductionsCredits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PortionOwned {
    EntireTrust,
    SubstantialPortion,
    SpecificAssetOrIncomeStream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section671Mode {
    NotApplicableNoGrantorTrustStatus,
    CompliantSection671AllItemsFlowToGrantor,
    CompliantSection673ReversionaryOver5PctTriggered,
    CompliantSection674PowerControlBeneficial,
    CompliantSection675AdministrativePower,
    CompliantSection675_4cSubstitutionPowerIdgtClassic,
    CompliantSection676PowerToRevoke,
    CompliantSection677IncomeForGrantorBenefit,
    CompliantSection678ThirdPartyTreatedAsOwnerBdit,
    CompliantSection679ForeignTrustUsBeneficiary,
    ViolationItemsFailedToFlowThroughToGrantor,
    ViolationSection674PowerNotExemptedFromTrigger,
    ViolationSection678LapseFailsFiveAndFiveSafeHarbor,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub grantor_trigger_section: GrantorTriggerSection,
    pub portion_owned: PortionOwned,
    pub flow_through_items_reported: FlowThroughItems,
    pub reversionary_interest_basis_points_section_673: u64,
    pub section_674_exemption_applies: bool,
    pub section_678_lapse_meets_5x5_safe_harbor: bool,
    pub grantor_reported_items_on_personal_return: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section671Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section671Input = Input;
pub type Section671Output = Output;
pub type Section671Result = Output;

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "26 U.S.C. § 671 — general rule: items of income, deductions, and credits attributable to grantor-owned portion INCLUDED in grantor's taxable income and credits".to_string(),
        "26 U.S.C. § 672 — definitions: grantor, beneficiary, adverse party, nonadverse party, related/subordinate party".to_string(),
        "26 U.S.C. § 673 — reversionary interest > 5 % of value at inception triggers ownership (post-1986 standard; Tax Reform Act of 1986)".to_string(),
        "26 U.S.C. § 674 — power to control beneficial enjoyment; broad trigger with § 674(b) and § 674(c) exceptions".to_string(),
        "26 U.S.C. § 675 — administrative powers: (1) less-than-adequate-consideration; (2) borrow without security; (3) actual borrowing without timely repayment; (4) general nonfiduciary administrative powers".to_string(),
        "26 U.S.C. § 675(4)(C) — power to substitute trust assets of equivalent value (most common modern IDGT 'intentional defect' trigger)".to_string(),
        "26 U.S.C. § 676 — power to revoke triggers ownership".to_string(),
        "26 U.S.C. § 677 — income accumulated or distributed for grantor's or spouse's benefit OR applied to insurance premiums on grantor's/spouse's life".to_string(),
        "26 U.S.C. § 678 — person other than grantor treated as owner: power exercisable solely by self to vest corpus or income in self; partial release/lapse subject to § 678(b)(2) 5x5 safe harbor".to_string(),
        "26 U.S.C. § 679 — foreign trusts with US beneficiaries: US transferor treated as owner (anti-deferral)".to_string(),
        "26 CFR Part 1 Subpart E — Grantors and Others Treated as Substantial Owners implementing regulations".to_string(),
        "IRS Rev. Rul. 2023-2 — grantor's death does NOT trigger basis step-up on IDGT assets (closes prior planning gap)".to_string(),
        "IDGT (Intentionally Defective Grantor Trust): grantor pays income tax on trust income → reduces grantor's estate further while trust assets grow tax-free for beneficiaries".to_string(),
        "BDIT (Beneficiary Defective Inheritor Trust) under § 678 — beneficiary's lapse of withdrawal power converts to grantor trust status with beneficiary as owner".to_string(),
        "§ 678(b)(2) 5x5 safe harbor — lapse of withdrawal power up to GREATER OF $5,000 OR 5 % of trust corpus NOT treated as release for § 678 purposes".to_string(),
    ];

    if input.grantor_trigger_section == GrantorTriggerSection::NoTriggerStandardNonGrantorTrust {
        return Output {
            mode: Section671Mode::NotApplicableNoGrantorTrustStatus,
            statutory_basis: "§§ 673-679 — no substantive trigger; standard non-grantor trust".to_string(),
            notes: "No §§ 673-679 grantor trust trigger satisfied; trust is non-grantor and pays its own income tax under § 641 (with § 651 / § 661 distribution deductions).".to_string(),
            citations,
        };
    }

    if input.grantor_trigger_section == GrantorTriggerSection::Section673ReversionaryOver5Pct {
        if input.reversionary_interest_basis_points_section_673
            <= SECTION_673_REVERSIONARY_INTEREST_THRESHOLD_BASIS_POINTS
        {
            return Output {
                mode: Section671Mode::NotApplicableNoGrantorTrustStatus,
                statutory_basis: "§ 673 — reversionary interest ≤ 5 % does NOT trigger ownership".to_string(),
                notes: format!(
                    "Reversionary interest of {} basis points (≤ 500 / 5 %) does not trigger § 673 ownership; standard non-grantor trust.",
                    input.reversionary_interest_basis_points_section_673
                ),
                citations,
            };
        }
        if !input.grantor_reported_items_on_personal_return {
            return Output {
                mode: Section671Mode::ViolationItemsFailedToFlowThroughToGrantor,
                statutory_basis: "§ 671 + § 673 — items must flow through to grantor".to_string(),
                notes: format!(
                    "VIOLATION § 671/§ 673: reversionary interest of {} basis points > 5 % triggers grantor ownership; grantor failed to report items on personal return.",
                    input.reversionary_interest_basis_points_section_673
                ),
                citations,
            };
        }
        return Output {
            mode: Section671Mode::CompliantSection673ReversionaryOver5PctTriggered,
            statutory_basis: "§ 673 — reversionary interest > 5 % triggers ownership; § 671 flow-through applied".to_string(),
            notes: format!(
                "COMPLIANT § 673: reversionary interest of {} basis points (> 500 / 5 %) triggers ownership; § 671 attributes items to grantor; grantor reported on personal return.",
                input.reversionary_interest_basis_points_section_673
            ),
            citations,
        };
    }

    if input.grantor_trigger_section
        == GrantorTriggerSection::Section674PowerControlBeneficialEnjoyment
        && !input.section_674_exemption_applies
    {
        if !input.grantor_reported_items_on_personal_return {
            return Output {
                mode: Section671Mode::ViolationSection674PowerNotExemptedFromTrigger,
                statutory_basis: "§ 671 + § 674 — power to control beneficial enjoyment triggers grantor ownership".to_string(),
                notes: "VIOLATION § 671/§ 674: power to control beneficial enjoyment triggers ownership; no § 674(b)/(c) exemption; grantor failed to report items.".to_string(),
                citations,
            };
        }
        return Output {
            mode: Section671Mode::CompliantSection674PowerControlBeneficial,
            statutory_basis: "§ 674 — power to control beneficial enjoyment triggers § 671 flow-through".to_string(),
            notes: "COMPLIANT § 674: power to control beneficial enjoyment triggers grantor ownership; § 671 flow-through applied to grantor return.".to_string(),
            citations,
        };
    }

    if input.grantor_trigger_section == GrantorTriggerSection::Section678ThirdPartyOwnerBdit
        && !input.section_678_lapse_meets_5x5_safe_harbor
    {
        return Output {
            mode: Section671Mode::ViolationSection678LapseFailsFiveAndFiveSafeHarbor,
            statutory_basis: "§ 678(b)(2) — 5x5 safe harbor not satisfied".to_string(),
            notes: "VIOLATION § 678(b)(2): beneficiary's lapse of withdrawal power exceeded GREATER OF $5,000 or 5 % of trust corpus; lapse treated as release converting beneficiary to grantor for that portion.".to_string(),
            citations,
        };
    }

    if !input.grantor_reported_items_on_personal_return {
        return Output {
            mode: Section671Mode::ViolationItemsFailedToFlowThroughToGrantor,
            statutory_basis: "§ 671 — items of income, deductions, and credits must flow to grantor".to_string(),
            notes: format!(
                "VIOLATION § 671: grantor trust trigger {:?} satisfied but grantor failed to report items on personal return; trust improperly treated as non-grantor.",
                input.grantor_trigger_section
            ),
            citations,
        };
    }

    let mode = match input.grantor_trigger_section {
        GrantorTriggerSection::Section673ReversionaryOver5Pct => {
            Section671Mode::CompliantSection673ReversionaryOver5PctTriggered
        }
        GrantorTriggerSection::Section674PowerControlBeneficialEnjoyment => {
            Section671Mode::CompliantSection674PowerControlBeneficial
        }
        GrantorTriggerSection::Section675AdministrativePower => {
            Section671Mode::CompliantSection675AdministrativePower
        }
        GrantorTriggerSection::Section675_4cSubstitutionPower => {
            Section671Mode::CompliantSection675_4cSubstitutionPowerIdgtClassic
        }
        GrantorTriggerSection::Section676PowerToRevoke => {
            Section671Mode::CompliantSection676PowerToRevoke
        }
        GrantorTriggerSection::Section677IncomeAccumulatedForGrantor => {
            Section671Mode::CompliantSection677IncomeForGrantorBenefit
        }
        GrantorTriggerSection::Section678ThirdPartyOwnerBdit => {
            Section671Mode::CompliantSection678ThirdPartyTreatedAsOwnerBdit
        }
        GrantorTriggerSection::Section679ForeignTrustUsBeneficiary => {
            Section671Mode::CompliantSection679ForeignTrustUsBeneficiary
        }
        GrantorTriggerSection::NoTriggerStandardNonGrantorTrust => {
            Section671Mode::NotApplicableNoGrantorTrustStatus
        }
    };

    Output {
        mode,
        statutory_basis: format!(
            "§ 671 + {:?} — grantor trust trigger satisfied; items flow to grantor",
            input.grantor_trigger_section
        ),
        notes: format!(
            "COMPLIANT: grantor trust trigger {:?} satisfied; portion owned = {:?}; items reported = {:?}; § 671 flow-through applied.",
            input.grantor_trigger_section, input.portion_owned, input.flow_through_items_reported
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_675_4c_substitution_compliant() -> Input {
        Input {
            grantor_trigger_section: GrantorTriggerSection::Section675_4cSubstitutionPower,
            portion_owned: PortionOwned::EntireTrust,
            flow_through_items_reported: FlowThroughItems::AllThreeIncomeDeductionsCredits,
            reversionary_interest_basis_points_section_673: 0,
            section_674_exemption_applies: false,
            section_678_lapse_meets_5x5_safe_harbor: true,
            grantor_reported_items_on_personal_return: true,
        }
    }

    #[test]
    fn no_trigger_standard_non_grantor_not_applicable() {
        let input = Input {
            grantor_trigger_section: GrantorTriggerSection::NoTriggerStandardNonGrantorTrust,
            ..baseline_675_4c_substitution_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section671Mode::NotApplicableNoGrantorTrustStatus
        );
    }

    #[test]
    fn section_675_4c_substitution_idgt_classic_compliant() {
        let result = compute(&baseline_675_4c_substitution_compliant());
        assert_eq!(
            result.mode,
            Section671Mode::CompliantSection675_4cSubstitutionPowerIdgtClassic
        );
    }

    #[test]
    fn section_673_reversionary_at_exactly_5_pct_not_applicable() {
        let input = Input {
            grantor_trigger_section: GrantorTriggerSection::Section673ReversionaryOver5Pct,
            reversionary_interest_basis_points_section_673: 500,
            ..baseline_675_4c_substitution_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section671Mode::NotApplicableNoGrantorTrustStatus
        );
    }

    #[test]
    fn section_673_reversionary_at_501_bp_triggered() {
        let input = Input {
            grantor_trigger_section: GrantorTriggerSection::Section673ReversionaryOver5Pct,
            reversionary_interest_basis_points_section_673: 501,
            ..baseline_675_4c_substitution_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section671Mode::CompliantSection673ReversionaryOver5PctTriggered
        );
    }

    #[test]
    fn section_673_triggered_but_items_not_flowed_violation() {
        let input = Input {
            grantor_trigger_section: GrantorTriggerSection::Section673ReversionaryOver5Pct,
            reversionary_interest_basis_points_section_673: 1_000,
            grantor_reported_items_on_personal_return: false,
            ..baseline_675_4c_substitution_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section671Mode::ViolationItemsFailedToFlowThroughToGrantor
        );
    }

    #[test]
    fn section_674_power_no_exemption_triggers_compliant() {
        let input = Input {
            grantor_trigger_section:
                GrantorTriggerSection::Section674PowerControlBeneficialEnjoyment,
            section_674_exemption_applies: false,
            ..baseline_675_4c_substitution_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section671Mode::CompliantSection674PowerControlBeneficial
        );
    }

    #[test]
    fn section_674_with_exemption_no_violation() {
        let input = Input {
            grantor_trigger_section:
                GrantorTriggerSection::Section674PowerControlBeneficialEnjoyment,
            section_674_exemption_applies: true,
            ..baseline_675_4c_substitution_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section671Mode::CompliantSection674PowerControlBeneficial
        );
    }

    #[test]
    fn section_674_no_exemption_items_not_reported_violation() {
        let input = Input {
            grantor_trigger_section:
                GrantorTriggerSection::Section674PowerControlBeneficialEnjoyment,
            section_674_exemption_applies: false,
            grantor_reported_items_on_personal_return: false,
            ..baseline_675_4c_substitution_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section671Mode::ViolationSection674PowerNotExemptedFromTrigger
        );
    }

    #[test]
    fn section_675_administrative_power_compliant() {
        let input = Input {
            grantor_trigger_section: GrantorTriggerSection::Section675AdministrativePower,
            ..baseline_675_4c_substitution_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section671Mode::CompliantSection675AdministrativePower
        );
    }

    #[test]
    fn section_676_power_to_revoke_compliant() {
        let input = Input {
            grantor_trigger_section: GrantorTriggerSection::Section676PowerToRevoke,
            ..baseline_675_4c_substitution_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section671Mode::CompliantSection676PowerToRevoke
        );
    }

    #[test]
    fn section_677_income_for_grantor_compliant() {
        let input = Input {
            grantor_trigger_section: GrantorTriggerSection::Section677IncomeAccumulatedForGrantor,
            ..baseline_675_4c_substitution_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section671Mode::CompliantSection677IncomeForGrantorBenefit
        );
    }

    #[test]
    fn section_678_bdit_5x5_safe_harbor_compliant() {
        let input = Input {
            grantor_trigger_section: GrantorTriggerSection::Section678ThirdPartyOwnerBdit,
            section_678_lapse_meets_5x5_safe_harbor: true,
            ..baseline_675_4c_substitution_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section671Mode::CompliantSection678ThirdPartyTreatedAsOwnerBdit
        );
    }

    #[test]
    fn section_678_bdit_lapse_fails_5x5_safe_harbor_violation() {
        let input = Input {
            grantor_trigger_section: GrantorTriggerSection::Section678ThirdPartyOwnerBdit,
            section_678_lapse_meets_5x5_safe_harbor: false,
            ..baseline_675_4c_substitution_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section671Mode::ViolationSection678LapseFailsFiveAndFiveSafeHarbor
        );
    }

    #[test]
    fn section_679_foreign_trust_us_beneficiary_compliant() {
        let input = Input {
            grantor_trigger_section: GrantorTriggerSection::Section679ForeignTrustUsBeneficiary,
            ..baseline_675_4c_substitution_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section671Mode::CompliantSection679ForeignTrustUsBeneficiary
        );
    }

    #[test]
    fn substitution_power_items_not_reported_violation() {
        let input = Input {
            grantor_reported_items_on_personal_return: false,
            ..baseline_675_4c_substitution_compliant()
        };
        let result = compute(&input);
        assert_eq!(
            result.mode,
            Section671Mode::ViolationItemsFailedToFlowThroughToGrantor
        );
    }

    #[test]
    fn citations_pin_section_671_through_679_subsections_and_idgt() {
        let result = compute(&baseline_675_4c_substitution_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("§ 671"));
        assert!(joined.contains("§ 672"));
        assert!(joined.contains("§ 673"));
        assert!(joined.contains("§ 674"));
        assert!(joined.contains("§ 675"));
        assert!(joined.contains("§ 675(4)(C)"));
        assert!(joined.contains("§ 676"));
        assert!(joined.contains("§ 677"));
        assert!(joined.contains("§ 678"));
        assert!(joined.contains("§ 678(b)(2)"));
        assert!(joined.contains("§ 679"));
        assert!(joined.contains("26 CFR Part 1 Subpart E"));
        assert!(joined.contains("IRS Rev. Rul. 2023-2"));
        assert!(joined.contains("IDGT"));
        assert!(joined.contains("BDIT"));
        assert!(joined.contains("5x5 safe harbor"));
    }

    #[test]
    fn constant_pin_5_pct_threshold_and_5x5_rule() {
        assert_eq!(
            SECTION_673_REVERSIONARY_INTEREST_THRESHOLD_BASIS_POINTS,
            500
        );
        assert_eq!(SECTION_678_B_2_LAPSE_5X5_RULE_GREATER_OF_DOLLARS, 5_000);
        assert_eq!(
            SECTION_678_B_2_LAPSE_5X5_RULE_TRUST_CORPUS_PCT_BASIS_POINTS,
            500
        );
        assert_eq!(SECTION_671_BASIS_POINT_DENOMINATOR, 10_000);
        assert!(SECTION_675_4C_SUBSTITUTION_POWER_AVAILABLE);
        assert_eq!(TAX_REFORM_ACT_1986_SECTION_673_EFFECTIVE_YEAR, 1986);
    }
}
