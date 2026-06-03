//! IRC § 674 — Power to Control Beneficial Enjoyment / Grantor
//! Trust Module.
//!
//! Pure-compute check for grantor trust status under Internal
//! Revenue Code § 674 ("Power to control beneficial enjoyment").
//! § 674 is the SECOND substantive grantor-trust trigger in the
//! §§ 671-679 statutory progression (after § 673 reversionary —
//! built iter 648; before § 675 administrative powers — iter
//! 644, § 676 power to revoke — iter 646, § 677 income for
//! benefit of grantor — iter 642, § 678 person other than
//! grantor — iter 640, § 679 foreign trusts — iter 650). § 674
//! is the BROADEST single statute in Subpart E — its general
//! rule sweeps any power affecting beneficial enjoyment — but
//! is mitigated by **three layers of statutory exceptions**:
//! § 674(b) powers exercisable by ANY person (eight enumerated
//! exceptions), § 674(c) powers exercisable by INDEPENDENT
//! TRUSTEES (half-trustees-independent test), and § 674(d)
//! powers limited by ASCERTAINABLE STANDARD (HEMS).
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 674(a) General Rule**: grantor treated as owner of
//!   any portion of a trust where **beneficial enjoyment** of
//!   corpus or income is subject to a **POWER OF DISPOSITION**
//!   exercisable by grantor or nonadverse party without
//!   approval or consent of any adverse party ([Cornell LII 26
//!   USC § 674](https://www.law.cornell.edu/uscode/text/26/674);
//!   [26 CFR § 1.674(a)-1](https://www.ecfr.gov/current/title-26/chapter-I/subchapter-A/part-1/subject-group-ECFR245d884a8952b47/section-1.674(a)-1)).
//! - **IRC § 674(b) — Eight Enumerated Exceptions** (powers
//!   exercisable by ANY person without triggering § 674): (1)
//!   power to apply income to support of dependent; (2) power
//!   affecting beneficial enjoyment only after occurrence of
//!   event; (3) testamentary power; (4) power to allocate among
//!   charitable beneficiaries; (5) power to distribute corpus;
//!   (6) power to withhold income temporarily; (7) power to
//!   withhold income during legal disability of beneficiary;
//!   (8) power to allocate between corpus and income ([26 CFR
//!   § 1.674(b)-1](https://www.law.cornell.edu/cfr/text/26/1.674(b)-1)).
//! - **IRC § 674(c) — Independent Trustee Exception**: powers
//!   to distribute, apportion, or accumulate income (or
//!   distribute corpus) excepted if held by trustee or trustees
//!   other than the grantor, **at least HALF of whom are
//!   independent** (not related or subordinate party to
//!   grantor) AND grantor is **NOT a trustee**.
//! - **IRC § 674(d) — Ascertainable Standard Exception**: power
//!   to distribute, apportion, or accumulate income excepted if
//!   held by trustee other than grantor or grantor's spouse AND
//!   power is limited by an **ASCERTAINABLE STANDARD set forth
//!   in the trust instrument** (typically HEMS — Health,
//!   Education, Maintenance, Support — under Treas. Reg.
//!   § 1.674(d)-1).
//! - **IRC § 672(c) Related or Subordinate Party Definition**
//!   (key to § 674(c) independent trustee analysis): any
//!   nonadverse party who is (1) the grantor's spouse if living
//!   with the grantor; (2) the grantor's father, mother, issue,
//!   brother, or sister; (3) an employee of the grantor; (4) a
//!   corporation or any employee of a corporation in which
//!   the stock holdings of the grantor and the trust are
//!   significant from the viewpoint of voting control; or (5)
//!   a subordinate employee of a corporation in which the
//!   grantor is an executive ([26 CFR § 1.672(c)-1](https://www.ecfr.gov/current/title-26/chapter-I/subchapter-A/part-1/subject-group-ECFR245d884a8952b47/section-1.672(c)-1)).
//! - **§ 674(b) "Add Beneficiary" Limitation**: exceptions in
//!   § 674(b)(5), (6), (7) PLUS § 674(c) and § 674(d) are NOT
//!   applicable if any person has a power to **ADD BENEFICIARIES**
//!   (or to a class of beneficiaries designated to receive
//!   income or corpus), except where the action is to provide
//!   for **AFTER-BORN OR AFTER-ADOPTED CHILDREN**.
//! - Practitioner consensus (Asena Advisors, Griffin Bridgers,
//!   Florida Bar Trustee Selection Guide): § 674 is the most
//!   complex grantor-trust trigger to draft around — even
//!   modest discretionary distribution powers held by the
//!   grantor or related party trigger § 674(a) absent careful
//!   § 674(b)/(c)/(d) exception structuring.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_674_B_ENUMERATED_EXCEPTIONS_COUNT: u32 = 8;
pub const IRC_674_C_INDEPENDENT_TRUSTEE_MIN_PCT_BASIS_POINTS: u64 = 5_000;
pub const IRC_674_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const IRC_672_C_RELATED_OR_SUBORDINATE_PARTY_DEFINITION_COUNT: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerOfDispositionCategory {
    NoPowerOfDisposition,
    Section674B1ApplyIncomeToSupportOfDependent,
    Section674B2BeneficialEnjoymentOnlyAfterEvent,
    Section674B3TestamentaryPower,
    Section674B4AllocateAmongCharitableBeneficiaries,
    Section674B5PowerToDistributeCorpus,
    Section674B6PowerToWithholdIncomeTemporarily,
    Section674B7PowerToWithholdDuringLegalDisability,
    Section674B8PowerToAllocateBetweenCorpusAndIncome,
    Section674CIndependentTrusteeDistribution,
    Section674DAscertainableStandardHemsDistribution,
    StandardDiscretionaryPowerNotExcepted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerHolderCategory {
    GrantorAloneOrWithNonadverseParty,
    GrantorAsTrustee,
    GrantorSpouseAsTrustee,
    IndependentTrusteesAtLeastHalfAndGrantorNotTrustee,
    RelatedOrSubordinatePartyOnlyMoreThanHalfTrustees,
    OnlyAdversePartyHoldsPower,
    NoPowerHolder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddBeneficiaryPowerStatus {
    NoPowerToAddBeneficiaries,
    PowerToAddBeneficiariesExistsAfterBornAfterAdoptedChildrenOnly,
    PowerToAddBeneficiariesExistsBeyondAfterBornAfterAdoptedChildren,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section674Mode {
    NotApplicableNoTrustOrNoPowerOfDisposition,
    NotApplicableOnlyAdversePartyHoldsPower,
    NotApplicableSection674B1SupportOfDependentException,
    NotApplicableSection674B2BeneficialEnjoymentAfterEventException,
    NotApplicableSection674B3TestamentaryPowerException,
    NotApplicableSection674B4CharitableBeneficiariesException,
    NotApplicableSection674B5DistributeCorpusException,
    NotApplicableSection674B6WithholdIncomeTemporarilyException,
    NotApplicableSection674B7WithholdDuringLegalDisabilityException,
    NotApplicableSection674B8AllocateCorpusIncomeException,
    NotApplicableSection674CIndependentTrusteeException,
    NotApplicableSection674DAscertainableStandardHemsException,
    CompliantSection674AActiveGrantorTrustStandardDiscretionaryPower,
    CompliantSection674AGrantorAsTrusteeActiveGrantorTrust,
    ViolationSection674ActiveButGrantorTrustIncomeNotReportedOnForm1040,
    ViolationSection674CIndependentTrusteeExceptionClaimedButLessThanHalfIndependent,
    ViolationSection674CIndependentTrusteeExceptionClaimedButGrantorIsTrustee,
    ViolationSection674DAscertainableStandardClaimedButTrusteeIsGrantorOrSpouse,
    ViolationSection674AddBeneficiaryPowerDefeatsExceptionsB5B6B7CD,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub trust_exists: bool,
    pub power_of_disposition_category: PowerOfDispositionCategory,
    pub power_holder_category: PowerHolderCategory,
    pub add_beneficiary_power_status: AddBeneficiaryPowerStatus,
    pub trustees_independent_basis_points: u64,
    pub grantor_trust_income_reported_on_form_1040: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section674Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section674Input = Input;
pub type Section674Output = Output;
pub type Section674Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 674(a) — grantor treated as owner of any portion of trust where beneficial enjoyment of corpus or income is subject to power of disposition exercisable by grantor or nonadverse party without approval or consent of any adverse party".to_string(),
        "IRC § 674(b) — eight enumerated exceptions exercisable by ANY person: (1) support of dependent; (2) beneficial enjoyment only after event; (3) testamentary power; (4) charitable beneficiaries; (5) distribute corpus; (6) withhold income temporarily; (7) withhold during legal disability; (8) allocate between corpus and income".to_string(),
        "IRC § 674(c) — INDEPENDENT TRUSTEE exception; powers to distribute, apportion, or accumulate income (or distribute corpus) excepted if held by trustees other than grantor, at least HALF of whom are INDEPENDENT (not related or subordinate per § 672(c)) AND grantor is NOT a trustee".to_string(),
        "IRC § 674(d) — ASCERTAINABLE STANDARD exception; power to distribute, apportion, or accumulate income excepted if held by trustee other than grantor or grantor's spouse AND power limited by ascertainable standard set forth in trust instrument (typically HEMS — Health, Education, Maintenance, Support)".to_string(),
        "IRC § 672(c) RELATED OR SUBORDINATE PARTY — nonadverse party who is (1) spouse if living with grantor; (2) father, mother, issue, brother, sister; (3) employee of grantor; (4) corporation or employee where grantor + trust have significant voting control; (5) subordinate employee of corporation where grantor is executive".to_string(),
        "§ 674(b) 'Add Beneficiary' Limitation — exceptions § 674(b)(5), (6), (7) PLUS § 674(c) and § 674(d) NOT applicable if any person has power to ADD BENEFICIARIES (or class of beneficiaries) designated to receive income or corpus, except after-born or after-adopted children".to_string(),
        "26 CFR § 1.674(a)-1 — implementing regulation for § 674(a) general rule".to_string(),
        "26 CFR § 1.674(b)-1 — implementing regulation for § 674(b) excepted powers exercisable by any person".to_string(),
        "26 CFR § 1.674(c)-1 — implementing regulation for § 674(c) independent trustee exception".to_string(),
        "26 CFR § 1.674(d)-1 — implementing regulation for § 674(d) ascertainable standard exception".to_string(),
        "26 CFR § 1.672(c)-1 — implementing regulation for related or subordinate party definition under § 672(c)".to_string(),
        "Asena Advisors — IRC § 674 Regulations and Exceptions; practitioner guide".to_string(),
        "Griffin Bridgers — Grantor Trusts and Ascertainable Standards + Charitable Powers commentary".to_string(),
        "Florida Bar — A Practical Guide to Trustee Selection: A Review of the Most Common Tax (and Nontax) Traps".to_string(),
        "Cornell LII 26 USC § 674 — primary statutory text".to_string(),
        "IRC § 671 — Subpart E general attribution; § 674-triggered grantor trust status flows income, deductions, credits to grantor's Form 1040; trust files Form 1041 with grantor-trust statement".to_string(),
    ];

    if !input.trust_exists {
        return Output {
            mode: Section674Mode::NotApplicableNoTrustOrNoPowerOfDisposition,
            statutory_basis: "IRC § 674 inapplicable — no trust exists".to_string(),
            notes: "No trust exists; IRC § 674 inapplicable.".to_string(),
            citations,
        };
    }

    if input.power_of_disposition_category == PowerOfDispositionCategory::NoPowerOfDisposition
        || input.power_holder_category == PowerHolderCategory::NoPowerHolder
    {
        return Output {
            mode: Section674Mode::NotApplicableNoTrustOrNoPowerOfDisposition,
            statutory_basis: "IRC § 674 inapplicable — no power of disposition exists".to_string(),
            notes: "No power of disposition over beneficial enjoyment exists; § 674 does not trigger.".to_string(),
            citations,
        };
    }

    if input.power_holder_category == PowerHolderCategory::OnlyAdversePartyHoldsPower {
        return Output {
            mode: Section674Mode::NotApplicableOnlyAdversePartyHoldsPower,
            statutory_basis: "IRC § 674(a) — power exercisable only by adverse party".to_string(),
            notes: "NOT APPLICABLE: power of disposition exercisable only by adverse party (substantial beneficial interest adverse per § 672(a)); § 674(a) does not trigger.".to_string(),
            citations,
        };
    }

    let add_beneficiary_defeats_exceptions = matches!(
        input.add_beneficiary_power_status,
        AddBeneficiaryPowerStatus::PowerToAddBeneficiariesExistsBeyondAfterBornAfterAdoptedChildren
    );

    let category_subject_to_add_beneficiary_limitation = matches!(
        input.power_of_disposition_category,
        PowerOfDispositionCategory::Section674B5PowerToDistributeCorpus
            | PowerOfDispositionCategory::Section674B6PowerToWithholdIncomeTemporarily
            | PowerOfDispositionCategory::Section674B7PowerToWithholdDuringLegalDisability
            | PowerOfDispositionCategory::Section674CIndependentTrusteeDistribution
            | PowerOfDispositionCategory::Section674DAscertainableStandardHemsDistribution
    );

    if add_beneficiary_defeats_exceptions && category_subject_to_add_beneficiary_limitation {
        if !input.grantor_trust_income_reported_on_form_1040 {
            return Output {
                mode: Section674Mode::ViolationSection674ActiveButGrantorTrustIncomeNotReportedOnForm1040,
                statutory_basis: "IRC § 674 — add-beneficiary limitation defeats § 674(b)(5)/(6)/(7) + (c) + (d) exceptions".to_string(),
                notes: "VIOLATION: § 674(b)(5)/(6)/(7) + § 674(c) + § 674(d) exceptions defeated by existing power to add beneficiaries (beyond after-born/after-adopted children); § 674(a) triggers grantor trust status; Form 1040 reporting omitted.".to_string(),
                citations,
            };
        }
        return Output {
            mode: Section674Mode::ViolationSection674AddBeneficiaryPowerDefeatsExceptionsB5B6B7CD,
            statutory_basis: "IRC § 674 — § 674(b)(5)/(6)/(7) + (c) + (d) exceptions defeated".to_string(),
            notes: "Note: § 674 exceptions are defeated by existing power to add beneficiaries beyond after-born/after-adopted children; § 674(a) triggers grantor trust status; income properly reported on Form 1040.".to_string(),
            citations,
        };
    }

    match input.power_of_disposition_category {
        PowerOfDispositionCategory::Section674B1ApplyIncomeToSupportOfDependent => Output {
            mode: Section674Mode::NotApplicableSection674B1SupportOfDependentException,
            statutory_basis: "IRC § 674(b)(1) — power to apply income to support of dependent".to_string(),
            notes: "NOT APPLICABLE: § 674(b)(1) excepts power to apply income to support of grantor's dependent regardless of trustee identity.".to_string(),
            citations,
        },
        PowerOfDispositionCategory::Section674B2BeneficialEnjoymentOnlyAfterEvent => Output {
            mode: Section674Mode::NotApplicableSection674B2BeneficialEnjoymentAfterEventException,
            statutory_basis: "IRC § 674(b)(2) — power affecting beneficial enjoyment only after event".to_string(),
            notes: "NOT APPLICABLE: § 674(b)(2) excepts power affecting beneficial enjoyment only after occurrence of an event.".to_string(),
            citations,
        },
        PowerOfDispositionCategory::Section674B3TestamentaryPower => Output {
            mode: Section674Mode::NotApplicableSection674B3TestamentaryPowerException,
            statutory_basis: "IRC § 674(b)(3) — testamentary power".to_string(),
            notes: "NOT APPLICABLE: § 674(b)(3) excepts testamentary power exercisable by will (effective only at death).".to_string(),
            citations,
        },
        PowerOfDispositionCategory::Section674B4AllocateAmongCharitableBeneficiaries => Output {
            mode: Section674Mode::NotApplicableSection674B4CharitableBeneficiariesException,
            statutory_basis: "IRC § 674(b)(4) — power to allocate among charitable beneficiaries".to_string(),
            notes: "NOT APPLICABLE: § 674(b)(4) excepts power to allocate among specific charitable beneficiaries by identification or class.".to_string(),
            citations,
        },
        PowerOfDispositionCategory::Section674B5PowerToDistributeCorpus => Output {
            mode: Section674Mode::NotApplicableSection674B5DistributeCorpusException,
            statutory_basis: "IRC § 674(b)(5) — power to distribute corpus".to_string(),
            notes: "NOT APPLICABLE: § 674(b)(5) excepts power to distribute corpus (subject to add-beneficiary limitation).".to_string(),
            citations,
        },
        PowerOfDispositionCategory::Section674B6PowerToWithholdIncomeTemporarily => Output {
            mode: Section674Mode::NotApplicableSection674B6WithholdIncomeTemporarilyException,
            statutory_basis: "IRC § 674(b)(6) — power to withhold income temporarily".to_string(),
            notes: "NOT APPLICABLE: § 674(b)(6) excepts power to withhold income temporarily (subject to add-beneficiary limitation).".to_string(),
            citations,
        },
        PowerOfDispositionCategory::Section674B7PowerToWithholdDuringLegalDisability => Output {
            mode: Section674Mode::NotApplicableSection674B7WithholdDuringLegalDisabilityException,
            statutory_basis: "IRC § 674(b)(7) — power to withhold income during legal disability".to_string(),
            notes: "NOT APPLICABLE: § 674(b)(7) excepts power to withhold income during legal disability of beneficiary (subject to add-beneficiary limitation).".to_string(),
            citations,
        },
        PowerOfDispositionCategory::Section674B8PowerToAllocateBetweenCorpusAndIncome => Output {
            mode: Section674Mode::NotApplicableSection674B8AllocateCorpusIncomeException,
            statutory_basis: "IRC § 674(b)(8) — power to allocate between corpus and income".to_string(),
            notes: "NOT APPLICABLE: § 674(b)(8) excepts power to allocate between corpus and income.".to_string(),
            citations,
        },
        PowerOfDispositionCategory::Section674CIndependentTrusteeDistribution => {
            if input.power_holder_category == PowerHolderCategory::GrantorAsTrustee {
                return Output {
                    mode: Section674Mode::ViolationSection674CIndependentTrusteeExceptionClaimedButGrantorIsTrustee,
                    statutory_basis: "IRC § 674(c) — grantor cannot be trustee for independent trustee exception".to_string(),
                    notes: "VIOLATION: § 674(c) independent trustee exception claimed but grantor is a trustee; statutory requirement is grantor must NOT be a trustee.".to_string(),
                    citations,
                };
            }
            if input.trustees_independent_basis_points
                < IRC_674_C_INDEPENDENT_TRUSTEE_MIN_PCT_BASIS_POINTS
            {
                return Output {
                    mode: Section674Mode::ViolationSection674CIndependentTrusteeExceptionClaimedButLessThanHalfIndependent,
                    statutory_basis: "IRC § 674(c) — at least half of trustees must be independent".to_string(),
                    notes: format!(
                        "VIOLATION: § 674(c) independent trustee exception claimed but only {} basis points of trustees are independent (< 5000 bp = 50 % statutory minimum); § 672(c) related or subordinate party test fails.",
                        input.trustees_independent_basis_points
                    ),
                    citations,
                };
            }
            Output {
                mode: Section674Mode::NotApplicableSection674CIndependentTrusteeException,
                statutory_basis: "IRC § 674(c) — independent trustee exception satisfied".to_string(),
                notes: format!(
                    "NOT APPLICABLE: § 674(c) independent trustee exception applies; grantor is not a trustee; {} basis points of trustees are independent (≥ 5000 bp = 50 % statutory minimum); § 672(c) test satisfied (subject to add-beneficiary limitation).",
                    input.trustees_independent_basis_points
                ),
                citations,
            }
        }
        PowerOfDispositionCategory::Section674DAscertainableStandardHemsDistribution => {
            if matches!(
                input.power_holder_category,
                PowerHolderCategory::GrantorAsTrustee | PowerHolderCategory::GrantorSpouseAsTrustee
            ) {
                return Output {
                    mode: Section674Mode::ViolationSection674DAscertainableStandardClaimedButTrusteeIsGrantorOrSpouse,
                    statutory_basis: "IRC § 674(d) — trustee must not be grantor or grantor's spouse".to_string(),
                    notes: "VIOLATION: § 674(d) ascertainable standard exception claimed but trustee is the grantor or grantor's spouse; § 674(d) requires trustee to be someone other than grantor or grantor's spouse.".to_string(),
                    citations,
                };
            }
            Output {
                mode: Section674Mode::NotApplicableSection674DAscertainableStandardHemsException,
                statutory_basis: "IRC § 674(d) — ascertainable standard (HEMS) exception satisfied".to_string(),
                notes: "NOT APPLICABLE: § 674(d) ascertainable standard exception applies; trustee is someone other than grantor or grantor's spouse; power limited by ascertainable standard (e.g., HEMS — Health, Education, Maintenance, Support) set forth in trust instrument (subject to add-beneficiary limitation).".to_string(),
                citations,
            }
        }
        PowerOfDispositionCategory::StandardDiscretionaryPowerNotExcepted => {
            if !input.grantor_trust_income_reported_on_form_1040 {
                return Output {
                    mode: Section674Mode::ViolationSection674ActiveButGrantorTrustIncomeNotReportedOnForm1040,
                    statutory_basis: "IRC § 674(a) — active grantor trust requires Form 1040 reporting via § 671 flow-through".to_string(),
                    notes: "VIOLATION: § 674(a) standard discretionary power (not within any § 674(b)/(c)/(d) exception) triggers grantor trust status; income not reported on Form 1040.".to_string(),
                    citations,
                };
            }
            if input.power_holder_category == PowerHolderCategory::GrantorAsTrustee {
                return Output {
                    mode: Section674Mode::CompliantSection674AGrantorAsTrusteeActiveGrantorTrust,
                    statutory_basis: "IRC § 674(a) — grantor as trustee with power of disposition".to_string(),
                    notes: "COMPLIANT: § 674(a) grantor as trustee holds standard discretionary power not within any § 674(b)/(c)/(d) exception; grantor trust status active; income properly reported on Form 1040.".to_string(),
                    citations,
                };
            }
            Output {
                mode: Section674Mode::CompliantSection674AActiveGrantorTrustStandardDiscretionaryPower,
                statutory_basis: "IRC § 674(a) — standard discretionary power of disposition".to_string(),
                notes: format!(
                    "COMPLIANT: § 674(a) standard discretionary power of disposition held by {:?} not within any § 674(b)/(c)/(d) exception; grantor trust status active; income properly reported on Form 1040.",
                    input.power_holder_category
                ),
                citations,
            }
        }
        PowerOfDispositionCategory::NoPowerOfDisposition => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_standard_discretionary_power_compliant() -> Input {
        Input {
            trust_exists: true,
            power_of_disposition_category: PowerOfDispositionCategory::StandardDiscretionaryPowerNotExcepted,
            power_holder_category: PowerHolderCategory::GrantorAloneOrWithNonadverseParty,
            add_beneficiary_power_status: AddBeneficiaryPowerStatus::NoPowerToAddBeneficiaries,
            trustees_independent_basis_points: 0,
            grantor_trust_income_reported_on_form_1040: true,
        }
    }

    #[test]
    fn no_trust_not_applicable() {
        let input = Input {
            trust_exists: false,
            ..baseline_standard_discretionary_power_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, Section674Mode::NotApplicableNoTrustOrNoPowerOfDisposition);
    }

    #[test]
    fn no_power_of_disposition_not_applicable() {
        let input = Input {
            power_of_disposition_category: PowerOfDispositionCategory::NoPowerOfDisposition,
            power_holder_category: PowerHolderCategory::NoPowerHolder,
            ..baseline_standard_discretionary_power_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, Section674Mode::NotApplicableNoTrustOrNoPowerOfDisposition);
    }

    #[test]
    fn only_adverse_party_holds_power_not_applicable() {
        let input = Input {
            power_holder_category: PowerHolderCategory::OnlyAdversePartyHoldsPower,
            ..baseline_standard_discretionary_power_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, Section674Mode::NotApplicableOnlyAdversePartyHoldsPower);
    }

    #[test]
    fn standard_discretionary_power_compliant() {
        let result = check(&baseline_standard_discretionary_power_compliant());
        assert_eq!(
            result.mode,
            Section674Mode::CompliantSection674AActiveGrantorTrustStandardDiscretionaryPower
        );
    }

    #[test]
    fn grantor_as_trustee_active_grantor_trust_compliant() {
        let input = Input {
            power_holder_category: PowerHolderCategory::GrantorAsTrustee,
            ..baseline_standard_discretionary_power_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section674Mode::CompliantSection674AGrantorAsTrusteeActiveGrantorTrust
        );
    }

    #[test]
    fn section_674_b1_support_of_dependent_excepted() {
        let input = Input {
            power_of_disposition_category:
                PowerOfDispositionCategory::Section674B1ApplyIncomeToSupportOfDependent,
            ..baseline_standard_discretionary_power_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section674Mode::NotApplicableSection674B1SupportOfDependentException
        );
    }

    #[test]
    fn section_674_b2_beneficial_enjoyment_after_event_excepted() {
        let input = Input {
            power_of_disposition_category:
                PowerOfDispositionCategory::Section674B2BeneficialEnjoymentOnlyAfterEvent,
            ..baseline_standard_discretionary_power_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section674Mode::NotApplicableSection674B2BeneficialEnjoymentAfterEventException
        );
    }

    #[test]
    fn section_674_b3_testamentary_power_excepted() {
        let input = Input {
            power_of_disposition_category: PowerOfDispositionCategory::Section674B3TestamentaryPower,
            ..baseline_standard_discretionary_power_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section674Mode::NotApplicableSection674B3TestamentaryPowerException
        );
    }

    #[test]
    fn section_674_b4_charitable_beneficiaries_excepted() {
        let input = Input {
            power_of_disposition_category:
                PowerOfDispositionCategory::Section674B4AllocateAmongCharitableBeneficiaries,
            ..baseline_standard_discretionary_power_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section674Mode::NotApplicableSection674B4CharitableBeneficiariesException
        );
    }

    #[test]
    fn section_674_b5_distribute_corpus_excepted() {
        let input = Input {
            power_of_disposition_category: PowerOfDispositionCategory::Section674B5PowerToDistributeCorpus,
            ..baseline_standard_discretionary_power_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section674Mode::NotApplicableSection674B5DistributeCorpusException
        );
    }

    #[test]
    fn section_674_b8_allocate_corpus_income_excepted() {
        let input = Input {
            power_of_disposition_category:
                PowerOfDispositionCategory::Section674B8PowerToAllocateBetweenCorpusAndIncome,
            ..baseline_standard_discretionary_power_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section674Mode::NotApplicableSection674B8AllocateCorpusIncomeException
        );
    }

    #[test]
    fn section_674_c_independent_trustee_at_50_pct_compliant() {
        let input = Input {
            power_of_disposition_category:
                PowerOfDispositionCategory::Section674CIndependentTrusteeDistribution,
            power_holder_category:
                PowerHolderCategory::IndependentTrusteesAtLeastHalfAndGrantorNotTrustee,
            trustees_independent_basis_points: 5_000,
            ..baseline_standard_discretionary_power_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section674Mode::NotApplicableSection674CIndependentTrusteeException
        );
    }

    #[test]
    fn section_674_c_independent_trustee_below_50_pct_violation() {
        let input = Input {
            power_of_disposition_category:
                PowerOfDispositionCategory::Section674CIndependentTrusteeDistribution,
            power_holder_category:
                PowerHolderCategory::RelatedOrSubordinatePartyOnlyMoreThanHalfTrustees,
            trustees_independent_basis_points: 4_999,
            ..baseline_standard_discretionary_power_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section674Mode::ViolationSection674CIndependentTrusteeExceptionClaimedButLessThanHalfIndependent
        );
    }

    #[test]
    fn section_674_c_grantor_as_trustee_violation() {
        let input = Input {
            power_of_disposition_category:
                PowerOfDispositionCategory::Section674CIndependentTrusteeDistribution,
            power_holder_category: PowerHolderCategory::GrantorAsTrustee,
            trustees_independent_basis_points: 7_500,
            ..baseline_standard_discretionary_power_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section674Mode::ViolationSection674CIndependentTrusteeExceptionClaimedButGrantorIsTrustee
        );
    }

    #[test]
    fn section_674_d_ascertainable_standard_hems_compliant() {
        let input = Input {
            power_of_disposition_category:
                PowerOfDispositionCategory::Section674DAscertainableStandardHemsDistribution,
            power_holder_category:
                PowerHolderCategory::IndependentTrusteesAtLeastHalfAndGrantorNotTrustee,
            ..baseline_standard_discretionary_power_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section674Mode::NotApplicableSection674DAscertainableStandardHemsException
        );
    }

    #[test]
    fn section_674_d_grantor_as_trustee_violation() {
        let input = Input {
            power_of_disposition_category:
                PowerOfDispositionCategory::Section674DAscertainableStandardHemsDistribution,
            power_holder_category: PowerHolderCategory::GrantorAsTrustee,
            ..baseline_standard_discretionary_power_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section674Mode::ViolationSection674DAscertainableStandardClaimedButTrusteeIsGrantorOrSpouse
        );
    }

    #[test]
    fn section_674_d_grantor_spouse_as_trustee_violation() {
        let input = Input {
            power_of_disposition_category:
                PowerOfDispositionCategory::Section674DAscertainableStandardHemsDistribution,
            power_holder_category: PowerHolderCategory::GrantorSpouseAsTrustee,
            ..baseline_standard_discretionary_power_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section674Mode::ViolationSection674DAscertainableStandardClaimedButTrusteeIsGrantorOrSpouse
        );
    }

    #[test]
    fn add_beneficiary_power_defeats_section_674_b5_exception() {
        let input = Input {
            power_of_disposition_category: PowerOfDispositionCategory::Section674B5PowerToDistributeCorpus,
            add_beneficiary_power_status:
                AddBeneficiaryPowerStatus::PowerToAddBeneficiariesExistsBeyondAfterBornAfterAdoptedChildren,
            ..baseline_standard_discretionary_power_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section674Mode::ViolationSection674AddBeneficiaryPowerDefeatsExceptionsB5B6B7CD
        );
    }

    #[test]
    fn add_beneficiary_after_born_only_does_not_defeat_exception() {
        let input = Input {
            power_of_disposition_category: PowerOfDispositionCategory::Section674B5PowerToDistributeCorpus,
            add_beneficiary_power_status:
                AddBeneficiaryPowerStatus::PowerToAddBeneficiariesExistsAfterBornAfterAdoptedChildrenOnly,
            ..baseline_standard_discretionary_power_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section674Mode::NotApplicableSection674B5DistributeCorpusException
        );
    }

    #[test]
    fn add_beneficiary_does_not_defeat_section_674_b1_b2_b3_b4_b8() {
        for category in [
            PowerOfDispositionCategory::Section674B1ApplyIncomeToSupportOfDependent,
            PowerOfDispositionCategory::Section674B2BeneficialEnjoymentOnlyAfterEvent,
            PowerOfDispositionCategory::Section674B3TestamentaryPower,
            PowerOfDispositionCategory::Section674B4AllocateAmongCharitableBeneficiaries,
            PowerOfDispositionCategory::Section674B8PowerToAllocateBetweenCorpusAndIncome,
        ] {
            let input = Input {
                power_of_disposition_category: category,
                add_beneficiary_power_status:
                    AddBeneficiaryPowerStatus::PowerToAddBeneficiariesExistsBeyondAfterBornAfterAdoptedChildren,
                ..baseline_standard_discretionary_power_compliant()
            };
            let result = check(&input);
            assert!(
                matches!(
                    result.mode,
                    Section674Mode::NotApplicableSection674B1SupportOfDependentException
                        | Section674Mode::NotApplicableSection674B2BeneficialEnjoymentAfterEventException
                        | Section674Mode::NotApplicableSection674B3TestamentaryPowerException
                        | Section674Mode::NotApplicableSection674B4CharitableBeneficiariesException
                        | Section674Mode::NotApplicableSection674B8AllocateCorpusIncomeException
                ),
                "category {:?} should not be defeated by add-beneficiary power",
                category
            );
        }
    }

    #[test]
    fn section_674_active_form_1040_omitted_violation() {
        let input = Input {
            grantor_trust_income_reported_on_form_1040: false,
            ..baseline_standard_discretionary_power_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section674Mode::ViolationSection674ActiveButGrantorTrustIncomeNotReportedOnForm1040
        );
    }

    #[test]
    fn citations_pin_section_674_subsections_and_companion_statutes() {
        let result = check(&baseline_standard_discretionary_power_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("IRC § 674(a)"));
        assert!(joined.contains("IRC § 674(b)"));
        assert!(joined.contains("IRC § 674(c)"));
        assert!(joined.contains("IRC § 674(d)"));
        assert!(joined.contains("IRC § 672(c)"));
        assert!(joined.contains("eight enumerated exceptions"));
        assert!(joined.contains("INDEPENDENT TRUSTEE"));
        assert!(joined.contains("ASCERTAINABLE STANDARD"));
        assert!(joined.contains("RELATED OR SUBORDINATE PARTY"));
        assert!(joined.contains("HEMS"));
        assert!(joined.contains("'Add Beneficiary' Limitation"));
        assert!(joined.contains("after-born or after-adopted children"));
        assert!(joined.contains("26 CFR § 1.674(a)-1"));
        assert!(joined.contains("26 CFR § 1.674(b)-1"));
        assert!(joined.contains("26 CFR § 1.674(c)-1"));
        assert!(joined.contains("26 CFR § 1.674(d)-1"));
        assert!(joined.contains("26 CFR § 1.672(c)-1"));
        assert!(joined.contains("Asena Advisors"));
        assert!(joined.contains("Griffin Bridgers"));
        assert!(joined.contains("Florida Bar"));
        assert!(joined.contains("IRC § 671"));
    }

    #[test]
    fn constant_pin_subsection_count_and_independent_trustee_threshold() {
        assert_eq!(IRC_674_B_ENUMERATED_EXCEPTIONS_COUNT, 8);
        assert_eq!(IRC_674_C_INDEPENDENT_TRUSTEE_MIN_PCT_BASIS_POINTS, 5_000);
        assert_eq!(IRC_674_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(IRC_672_C_RELATED_OR_SUBORDINATE_PARTY_DEFINITION_COUNT, 5);
    }
}
