//! IRC § 675 — Administrative Powers / Grantor Trust Module.
//!
//! Pure-compute check for grantor trust status under Internal
//! Revenue Code § 675 ("Administrative Powers"). § 675 is the
//! third grantor-trust trigger in the § 671-679 statutory
//! progression (after § 673 reversionary interest and § 674
//! power to control beneficial enjoyment; before § 676 power to
//! revoke, § 677 income for benefit of grantor — built in iter
//! 642 of this codebase, § 678 person other than grantor —
//! built in iter 640). Trader / family-office critical because
//! **§ 675(4)(C) substitution power** is the single most-used
//! modern IDGT (Intentionally Defective Grantor Trust) trigger:
//! its mere inclusion in trust instrument creates full grantor
//! trust status without any actual exercise, while not causing
//! § 2036 / § 2042 estate inclusion (per Rev. Rul. 2008-22).
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 675(1)** ("Power to deal for less than adequate
//!   consideration"): grantor treated as owner of any portion
//!   of a trust if power exercisable by grantor or nonadverse
//!   party (without consent of adverse party) enables grantor
//!   or any person to **purchase, exchange, or otherwise deal
//!   with or dispose of corpus or income therefrom for LESS
//!   THAN ADEQUATE CONSIDERATION** in money or money's worth
//!   ([Cornell LII 26 USC § 675](https://www.law.cornell.edu/uscode/text/26/675);
//!   [26 CFR § 1.675-1](https://www.law.cornell.edu/cfr/text/26/1.675-1)).
//! - **IRC § 675(2)** ("Power to borrow without adequate
//!   interest or security"): grantor treated as owner if power
//!   exercisable by grantor or nonadverse party enables grantor
//!   to **borrow corpus or income, directly or indirectly,
//!   without adequate interest OR without adequate security** —
//!   except where trustee (OTHER than grantor) is authorized
//!   under a general lending power to make loans to any person
//!   without regard to interest or security.
//! - **IRC § 675(3)** ("Borrowing of trust funds"): grantor
//!   treated as owner if the grantor has DIRECTLY OR INDIRECTLY
//!   **borrowed corpus or income and has NOT COMPLETELY REPAID
//!   the loan, including interest, before beginning of taxable
//!   year**. Exception: loan with adequate interest AND adequate
//!   security made by trustee OTHER than grantor AND other than
//!   related or subordinate party subservient to grantor.
//! - **IRC § 675(4)** ("General powers of administration"):
//!   power exercisable in a **NONFIDUCIARY CAPACITY** by any
//!   person without approval/consent of any person in a
//!   fiduciary capacity. Three enumerated powers:
//!   - **§ 675(4)(A)**: power to **VOTE OR DIRECT THE VOTING**
//!     of stock or other securities of a corporation in which
//!     the holdings of the grantor and the trust are
//!     **significant from the viewpoint of voting control**.
//!   - **§ 675(4)(B)**: power to **CONTROL THE INVESTMENT** of
//!     trust funds either by directing investments or vetoing
//!     proposed investments, where holdings consist of stocks
//!     or securities of corporations in which holdings of
//!     grantor and trust are significant from voting-control
//!     viewpoint.
//!   - **§ 675(4)(C) — THE SUBSTITUTION POWER**: power to
//!     **REACQUIRE TRUST CORPUS by SUBSTITUTING OTHER PROPERTY
//!     of an EQUIVALENT VALUE** ("swap power"). Mere inclusion
//!     extends grantor trust status to entire trust without
//!     further action by grantor ([Paul Hood — The Power of
//!     Substitution Under IRC Sec 675(4)(C)](https://paulhoodservices.com/wp-content/uploads/2021/09/Swap-Power-Monograph-LISI.pdf);
//!     [Kitces — Utilizing Swap Powers in Irrevocable Trusts](https://www.kitces.com/blog/swap-powers-irrevocable-trusts-tax-efficiency-idgt-estate-planning/)).
//! - **Rev. Rul. 2008-22** (2008-1 C.B. 796): IRS confirmed that
//!   power to substitute trust property of equivalent value
//!   under § 675(4)(C) does NOT result in § 2036 or § 2038
//!   estate inclusion when exercised in a FIDUCIARY CAPACITY
//!   (substitution may be challenged by trustee + cannot shift
//!   value among beneficiaries).
//! - **Rev. Rul. 2011-28**: same conclusion applied to power
//!   exercisable in nonfiduciary capacity — power "by itself"
//!   does not trigger § 2042 inclusion in life insurance trust.
//! - **§ 672(a) "ADVERSE PARTY"**: any person having a
//!   substantial beneficial interest in trust adverse to
//!   exercise of power; § 672(b) "NONADVERSE PARTY" = any
//!   person who is not an adverse party.
//! - **§ 672(c) "RELATED OR SUBORDINATE PARTY"**: any
//!   nonadverse party who is grantor's spouse, parent,
//!   descendant, sibling, employee, subordinate employee,
//!   corporation in which grantor + trust hold significant
//!   voting control, or subordinate employee thereof.
//! - **Treas. Reg. § 1.675-1**: implementing regulation
//!   defining "fiduciary capacity" + "nonfiduciary capacity" +
//!   "significant from voting control viewpoint" (general
//!   guidance — 10 %+ is typically presumed significant but
//!   facts-and-circumstances test applies).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_675_SUBSECTION_1: u32 = 1;
pub const IRC_675_SUBSECTION_2: u32 = 2;
pub const IRC_675_SUBSECTION_3: u32 = 3;
pub const IRC_675_SUBSECTION_4: u32 = 4;
pub const IRC_675_4_A_VOTING_POWER_SIGNIFICANT_BASIS_POINTS: u64 = 1_000;
pub const IRC_675_4_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const REV_RUL_2008_22_YEAR: u32 = 2008;
pub const REV_RUL_2011_28_YEAR: u32 = 2011;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section675Subsection {
    Section675_1PowerToDealForLessThanAdequateConsideration,
    Section675_2PowerToBorrowWithoutAdequateInterestOrSecurity,
    Section675_3BorrowingOfTrustFundsNotRepaid,
    Section675_4APowerToVoteSignificantVotingControl,
    Section675_4BPowerToControlInvestmentSignificantVotingControl,
    Section675_4CSubstitutionPowerClassicIdgt,
    NoTriggerNoSubsection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FiduciaryCapacityStatus {
    NonfiduciaryCapacityNoFiduciaryApprovalRequired,
    FiduciaryCapacityTrusteeOrFiduciaryControl,
    NotApplicableNoAdministrativePower,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BorrowingExceptionStatus {
    NoBorrowingExceptionApplicable,
    AdequateInterestAndSecurityTrusteeIsNotGrantorNorSubservient,
    InadequateInterestOrSecurity,
    BorrowingNotRepaidBeforeTaxableYear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section675Mode {
    NotApplicableNoTrustOrNoAdministrativePower,
    CompliantSection675_1LessThanAdequateConsiderationActiveGrantorTrust,
    CompliantSection675_2PowerToBorrowWithoutAdequateTermsActiveGrantorTrust,
    CompliantSection675_2GeneralLendingPowerExceptionDeactivates675_2,
    CompliantSection675_3BorrowedAndNotRepaidActiveGrantorTrust,
    CompliantSection675_3AdequateInterestAndSecurityTrusteeExceptionDeactivates675_3,
    CompliantSection675_4AVotingPowerNonfiduciaryActiveGrantorTrust,
    CompliantSection675_4BInvestmentControlNonfiduciaryActiveGrantorTrust,
    CompliantSection675_4CSubstitutionPowerClassicIdgtActiveGrantorTrustNoEstateInclusion,
    CompliantSection675_4CSubstitutionPowerFiduciaryCapacityRevRul2008_22ProtectsEstate,
    ViolationSection675ActiveButGrantorTrustIncomeNotReportedOnForm1040,
    ViolationSubstitutionPowerExercisedToShiftValueAmongBeneficiariesEstateInclusionRisk,
    ViolationSection675_4FiduciaryCapacityClaimedButPowerActuallyNonfiduciary,
    ViolationSection675_2BorrowingExceptionInappropriatelyClaimedWithSubservientTrustee,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub trust_exists: bool,
    pub power_to_deal_for_less_than_adequate_consideration: bool,
    pub power_to_borrow_without_adequate_interest_or_security: bool,
    pub grantor_actually_borrowed_and_not_repaid_before_taxable_year: bool,
    pub power_to_vote_significant_corporate_holdings: bool,
    pub voting_holdings_basis_points: u64,
    pub power_to_control_investment_significant_corporate_holdings: bool,
    pub power_to_substitute_corpus_for_equivalent_value: bool,
    pub fiduciary_capacity_status: FiduciaryCapacityStatus,
    pub borrowing_exception_status: BorrowingExceptionStatus,
    pub substitution_power_exercised_to_shift_value_among_beneficiaries: bool,
    pub grantor_trust_income_reported_on_form_1040: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section675Mode,
    pub triggered_subsection: Section675Subsection,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section675Input = Input;
pub type Section675Output = Output;
pub type Section675Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 675(1) — power to deal with corpus or income for LESS THAN ADEQUATE CONSIDERATION in money or money's worth triggers grantor trust status".to_string(),
        "IRC § 675(2) — power to borrow corpus or income without adequate interest OR without adequate security triggers grantor trust status; exception when trustee (other than grantor) acts under general lending power".to_string(),
        "IRC § 675(3) — actual borrowing by grantor (directly or indirectly) of corpus/income NOT completely repaid (with interest) before beginning of taxable year; exception for loans with adequate interest AND security by independent trustee (not grantor + not related or subordinate)".to_string(),
        "IRC § 675(4) — general powers of administration exercisable in NONFIDUCIARY CAPACITY without approval/consent of fiduciary".to_string(),
        "IRC § 675(4)(A) — power to VOTE or direct voting of stock/securities of corporation where holdings of grantor + trust are SIGNIFICANT from voting-control viewpoint".to_string(),
        "IRC § 675(4)(B) — power to CONTROL INVESTMENT of trust funds where holdings consist of stocks/securities in which grantor + trust holdings are significant from voting-control viewpoint".to_string(),
        "IRC § 675(4)(C) — POWER TO SUBSTITUTE TRUST CORPUS for property of equivalent value ('swap power'); THE classic modern IDGT trigger — mere inclusion creates grantor trust status without further action".to_string(),
        "Rev. Rul. 2008-22 (2008-1 C.B. 796) — § 675(4)(C) substitution power does NOT cause § 2036 or § 2038 estate inclusion when exercised in FIDUCIARY CAPACITY (trustee challenge + no value-shifting among beneficiaries)".to_string(),
        "Rev. Rul. 2011-28 — § 675(4)(C) substitution power 'by itself' does not trigger § 2042 inclusion in life insurance trust".to_string(),
        "IRC § 672(a) ADVERSE PARTY — any person having substantial beneficial interest in trust adverse to exercise of power".to_string(),
        "IRC § 672(b) NONADVERSE PARTY — any person who is not an adverse party".to_string(),
        "IRC § 672(c) RELATED OR SUBORDINATE PARTY — nonadverse party who is spouse, parent, descendant, sibling, employee, subordinate employee, etc.".to_string(),
        "Treas. Reg. § 1.675-1 — implementing regulation defining 'fiduciary capacity' vs 'nonfiduciary capacity' + 'significant from voting control viewpoint' (10 %+ presumed significant; facts-and-circumstances test)".to_string(),
        "IRC § 671 — Subpart E general attribution; § 675-triggered grantor trust status causes flow-through of income, deductions, credits to grantor's Form 1040".to_string(),
        "Paul Hood — The Power of Substitution Under IRC Sec 675(4)(C) (Swap Power Monograph LISI) — foundational § 675(4)(C) drafting analysis".to_string(),
        "Kitces — Utilizing Swap Powers in Irrevocable Trusts — modern IDGT practitioner guide on § 675(4)(C) swap power".to_string(),
    ];

    if !input.trust_exists {
        return Output {
            mode: Section675Mode::NotApplicableNoTrustOrNoAdministrativePower,
            triggered_subsection: Section675Subsection::NoTriggerNoSubsection,
            statutory_basis: "IRC § 675 inapplicable — no trust exists".to_string(),
            notes: "No trust exists; IRC § 675 inapplicable.".to_string(),
            citations,
        };
    }

    let any_administrative_power_held = input.power_to_deal_for_less_than_adequate_consideration
        || input.power_to_borrow_without_adequate_interest_or_security
        || input.grantor_actually_borrowed_and_not_repaid_before_taxable_year
        || input.power_to_vote_significant_corporate_holdings
        || input.power_to_control_investment_significant_corporate_holdings
        || input.power_to_substitute_corpus_for_equivalent_value;

    if !any_administrative_power_held {
        return Output {
            mode: Section675Mode::NotApplicableNoTrustOrNoAdministrativePower,
            triggered_subsection: Section675Subsection::NoTriggerNoSubsection,
            statutory_basis: "IRC § 675 not triggered — no enumerated administrative power held".to_string(),
            notes: "No § 675 administrative power held by grantor or nonadverse party; grantor trust status not triggered by § 675.".to_string(),
            citations,
        };
    }

    if input.power_to_substitute_corpus_for_equivalent_value
        && input.substitution_power_exercised_to_shift_value_among_beneficiaries
    {
        return Output {
            mode: Section675Mode::ViolationSubstitutionPowerExercisedToShiftValueAmongBeneficiariesEstateInclusionRisk,
            triggered_subsection: Section675Subsection::Section675_4CSubstitutionPowerClassicIdgt,
            statutory_basis: "Rev. Rul. 2008-22 — substitution power that shifts value among beneficiaries loses Rev. Rul. 2008-22 estate-inclusion protection".to_string(),
            notes: "VIOLATION: § 675(4)(C) substitution power exercised in manner that shifted value among beneficiaries; loses Rev. Rul. 2008-22 estate-inclusion protection; § 2036/§ 2038 estate inclusion risk activated.".to_string(),
            citations,
        };
    }

    if matches!(
        input.fiduciary_capacity_status,
        FiduciaryCapacityStatus::FiduciaryCapacityTrusteeOrFiduciaryControl
    ) && (input.power_to_vote_significant_corporate_holdings
        || input.power_to_control_investment_significant_corporate_holdings)
        && !input.power_to_substitute_corpus_for_equivalent_value
        && !input.power_to_deal_for_less_than_adequate_consideration
        && !input.power_to_borrow_without_adequate_interest_or_security
        && !input.grantor_actually_borrowed_and_not_repaid_before_taxable_year
    {
        return Output {
            mode: Section675Mode::ViolationSection675_4FiduciaryCapacityClaimedButPowerActuallyNonfiduciary,
            triggered_subsection: Section675Subsection::Section675_4APowerToVoteSignificantVotingControl,
            statutory_basis: "IRC § 675(4) — fiduciary-capacity carve-out only if power EXERCISABLE in fiduciary capacity with fiduciary approval".to_string(),
            notes: "Note: § 675(4) requires the power be exercisable in NONFIDUCIARY CAPACITY; if power is genuinely held in fiduciary capacity with fiduciary approval requirement, § 675(4) is not triggered. Verify capacity classification is accurate.".to_string(),
            citations,
        };
    }

    if input.power_to_substitute_corpus_for_equivalent_value {
        let triggered_subsection = Section675Subsection::Section675_4CSubstitutionPowerClassicIdgt;
        if !input.grantor_trust_income_reported_on_form_1040 {
            return Output {
                mode: Section675Mode::ViolationSection675ActiveButGrantorTrustIncomeNotReportedOnForm1040,
                triggered_subsection,
                statutory_basis: "IRC § 675(4)(C) — substitution power triggers § 671 flow-through; income must be reported on grantor Form 1040".to_string(),
                notes: "VIOLATION: § 675(4)(C) substitution power triggers grantor trust status (mere inclusion suffices); grantor trust income not reported on Form 1040.".to_string(),
                citations,
            };
        }
        if matches!(
            input.fiduciary_capacity_status,
            FiduciaryCapacityStatus::FiduciaryCapacityTrusteeOrFiduciaryControl
        ) {
            return Output {
                mode: Section675Mode::CompliantSection675_4CSubstitutionPowerFiduciaryCapacityRevRul2008_22ProtectsEstate,
                triggered_subsection,
                statutory_basis: "IRC § 675(4)(C) + Rev. Rul. 2008-22 — fiduciary-capacity substitution power; grantor trust + estate exclusion".to_string(),
                notes: "COMPLIANT: § 675(4)(C) substitution power exercised in FIDUCIARY CAPACITY; Rev. Rul. 2008-22 protects from § 2036/§ 2038 estate inclusion; full grantor trust status with estate exclusion (the classic IDGT outcome).".to_string(),
                citations,
            };
        }
        return Output {
            mode: Section675Mode::CompliantSection675_4CSubstitutionPowerClassicIdgtActiveGrantorTrustNoEstateInclusion,
            triggered_subsection,
            statutory_basis: "IRC § 675(4)(C) — substitution power triggers grantor trust status with no § 2036/§ 2038 estate inclusion".to_string(),
            notes: "COMPLIANT: § 675(4)(C) substitution power active; grantor trust status engaged; Rev. Rul. 2008-22 protects from estate inclusion when not exercised to shift value among beneficiaries — classic IDGT outcome.".to_string(),
            citations,
        };
    }

    if input.grantor_actually_borrowed_and_not_repaid_before_taxable_year {
        if matches!(
            input.borrowing_exception_status,
            BorrowingExceptionStatus::AdequateInterestAndSecurityTrusteeIsNotGrantorNorSubservient
        ) {
            return Output {
                mode: Section675Mode::CompliantSection675_3AdequateInterestAndSecurityTrusteeExceptionDeactivates675_3,
                triggered_subsection: Section675Subsection::Section675_3BorrowingOfTrustFundsNotRepaid,
                statutory_basis: "IRC § 675(3) exception — loan with adequate interest AND security by independent trustee".to_string(),
                notes: "COMPLIANT: § 675(3) exception applies; loan with adequate interest AND security made by independent trustee (not grantor, not related or subordinate); § 675(3) does not trigger.".to_string(),
                citations,
            };
        }
        if !input.grantor_trust_income_reported_on_form_1040 {
            return Output {
                mode: Section675Mode::ViolationSection675ActiveButGrantorTrustIncomeNotReportedOnForm1040,
                triggered_subsection: Section675Subsection::Section675_3BorrowingOfTrustFundsNotRepaid,
                statutory_basis: "IRC § 675(3) — borrowing not repaid; grantor trust status active".to_string(),
                notes: "VIOLATION: grantor borrowed corpus/income and failed to fully repay (with interest) before beginning of taxable year; § 675(3) triggers grantor trust status; income not reported on Form 1040.".to_string(),
                citations,
            };
        }
        return Output {
            mode: Section675Mode::CompliantSection675_3BorrowedAndNotRepaidActiveGrantorTrust,
            triggered_subsection: Section675Subsection::Section675_3BorrowingOfTrustFundsNotRepaid,
            statutory_basis: "IRC § 675(3) — borrowing not repaid".to_string(),
            notes: "COMPLIANT: § 675(3) triggers grantor trust status due to uncured borrowing; income properly reported on Form 1040.".to_string(),
            citations,
        };
    }

    if input.power_to_borrow_without_adequate_interest_or_security {
        if matches!(
            input.borrowing_exception_status,
            BorrowingExceptionStatus::AdequateInterestAndSecurityTrusteeIsNotGrantorNorSubservient
        ) {
            return Output {
                mode: Section675Mode::CompliantSection675_2GeneralLendingPowerExceptionDeactivates675_2,
                triggered_subsection: Section675Subsection::Section675_2PowerToBorrowWithoutAdequateInterestOrSecurity,
                statutory_basis: "IRC § 675(2) exception — general lending power exercised by independent trustee".to_string(),
                notes: "COMPLIANT: § 675(2) exception applies; trustee (not grantor) authorized under general lending power to make loans to any person without regard to interest or security; § 675(2) does not trigger.".to_string(),
                citations,
            };
        }
        if !input.grantor_trust_income_reported_on_form_1040 {
            return Output {
                mode: Section675Mode::ViolationSection675ActiveButGrantorTrustIncomeNotReportedOnForm1040,
                triggered_subsection: Section675Subsection::Section675_2PowerToBorrowWithoutAdequateInterestOrSecurity,
                statutory_basis: "IRC § 675(2) — power to borrow without adequate terms; grantor trust active".to_string(),
                notes: "VIOLATION: § 675(2) power to borrow without adequate interest or security held by grantor or nonadverse party; grantor trust income not reported on Form 1040.".to_string(),
                citations,
            };
        }
        return Output {
            mode: Section675Mode::CompliantSection675_2PowerToBorrowWithoutAdequateTermsActiveGrantorTrust,
            triggered_subsection: Section675Subsection::Section675_2PowerToBorrowWithoutAdequateInterestOrSecurity,
            statutory_basis: "IRC § 675(2) — power to borrow without adequate terms".to_string(),
            notes: "COMPLIANT: § 675(2) triggers grantor trust status; income properly reported on Form 1040.".to_string(),
            citations,
        };
    }

    if input.power_to_deal_for_less_than_adequate_consideration {
        if !input.grantor_trust_income_reported_on_form_1040 {
            return Output {
                mode: Section675Mode::ViolationSection675ActiveButGrantorTrustIncomeNotReportedOnForm1040,
                triggered_subsection: Section675Subsection::Section675_1PowerToDealForLessThanAdequateConsideration,
                statutory_basis: "IRC § 675(1) — power to deal for less than adequate consideration".to_string(),
                notes: "VIOLATION: § 675(1) power to deal for less than adequate consideration triggers grantor trust status; income not reported on Form 1040.".to_string(),
                citations,
            };
        }
        return Output {
            mode: Section675Mode::CompliantSection675_1LessThanAdequateConsiderationActiveGrantorTrust,
            triggered_subsection: Section675Subsection::Section675_1PowerToDealForLessThanAdequateConsideration,
            statutory_basis: "IRC § 675(1) — power to deal for less than adequate consideration".to_string(),
            notes: "COMPLIANT: § 675(1) triggers grantor trust status; income properly reported on Form 1040.".to_string(),
            citations,
        };
    }

    let voting_holdings_significant =
        input.voting_holdings_basis_points >= IRC_675_4_A_VOTING_POWER_SIGNIFICANT_BASIS_POINTS;

    if input.power_to_vote_significant_corporate_holdings
        && voting_holdings_significant
        && matches!(
            input.fiduciary_capacity_status,
            FiduciaryCapacityStatus::NonfiduciaryCapacityNoFiduciaryApprovalRequired
        )
    {
        if !input.grantor_trust_income_reported_on_form_1040 {
            return Output {
                mode: Section675Mode::ViolationSection675ActiveButGrantorTrustIncomeNotReportedOnForm1040,
                triggered_subsection: Section675Subsection::Section675_4APowerToVoteSignificantVotingControl,
                statutory_basis: "IRC § 675(4)(A) — power to vote significant corporate holdings in nonfiduciary capacity".to_string(),
                notes: "VIOLATION: § 675(4)(A) nonfiduciary voting power over significant corporate holdings triggers grantor trust status; income not reported on Form 1040.".to_string(),
                citations,
            };
        }
        return Output {
            mode: Section675Mode::CompliantSection675_4AVotingPowerNonfiduciaryActiveGrantorTrust,
            triggered_subsection: Section675Subsection::Section675_4APowerToVoteSignificantVotingControl,
            statutory_basis: "IRC § 675(4)(A) — nonfiduciary voting power over significant corporate holdings".to_string(),
            notes: format!(
                "COMPLIANT: § 675(4)(A) triggers grantor trust status; voting holdings {} basis points (≥ 1000 bp = 10 % presumed significant); income properly reported on Form 1040.",
                input.voting_holdings_basis_points
            ),
            citations,
        };
    }

    if input.power_to_control_investment_significant_corporate_holdings
        && voting_holdings_significant
        && matches!(
            input.fiduciary_capacity_status,
            FiduciaryCapacityStatus::NonfiduciaryCapacityNoFiduciaryApprovalRequired
        )
    {
        if !input.grantor_trust_income_reported_on_form_1040 {
            return Output {
                mode: Section675Mode::ViolationSection675ActiveButGrantorTrustIncomeNotReportedOnForm1040,
                triggered_subsection: Section675Subsection::Section675_4BPowerToControlInvestmentSignificantVotingControl,
                statutory_basis: "IRC § 675(4)(B) — power to control investment of significant corporate holdings in nonfiduciary capacity".to_string(),
                notes: "VIOLATION: § 675(4)(B) nonfiduciary investment control over significant corporate holdings triggers grantor trust status; income not reported on Form 1040.".to_string(),
                citations,
            };
        }
        return Output {
            mode: Section675Mode::CompliantSection675_4BInvestmentControlNonfiduciaryActiveGrantorTrust,
            triggered_subsection: Section675Subsection::Section675_4BPowerToControlInvestmentSignificantVotingControl,
            statutory_basis: "IRC § 675(4)(B) — nonfiduciary investment control over significant corporate holdings".to_string(),
            notes: "COMPLIANT: § 675(4)(B) triggers grantor trust status; income properly reported on Form 1040.".to_string(),
            citations,
        };
    }

    Output {
        mode: Section675Mode::NotApplicableNoTrustOrNoAdministrativePower,
        triggered_subsection: Section675Subsection::NoTriggerNoSubsection,
        statutory_basis: "IRC § 675 not triggered — administrative power held but not satisfying any subsection's trigger criteria".to_string(),
        notes: "No § 675 subsection trigger satisfied; voting holdings may be insignificant (< 10 % basis points), or power exercised in genuine fiduciary capacity.".to_string(),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_no_administrative_power() -> Input {
        Input {
            trust_exists: true,
            power_to_deal_for_less_than_adequate_consideration: false,
            power_to_borrow_without_adequate_interest_or_security: false,
            grantor_actually_borrowed_and_not_repaid_before_taxable_year: false,
            power_to_vote_significant_corporate_holdings: false,
            voting_holdings_basis_points: 0,
            power_to_control_investment_significant_corporate_holdings: false,
            power_to_substitute_corpus_for_equivalent_value: false,
            fiduciary_capacity_status: FiduciaryCapacityStatus::NotApplicableNoAdministrativePower,
            borrowing_exception_status: BorrowingExceptionStatus::NoBorrowingExceptionApplicable,
            substitution_power_exercised_to_shift_value_among_beneficiaries: false,
            grantor_trust_income_reported_on_form_1040: true,
        }
    }

    #[test]
    fn no_trust_not_applicable() {
        let input = Input {
            trust_exists: false,
            ..baseline_no_administrative_power()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section675Mode::NotApplicableNoTrustOrNoAdministrativePower
        );
    }

    #[test]
    fn no_administrative_power_not_applicable() {
        let result = check(&baseline_no_administrative_power());
        assert_eq!(
            result.mode,
            Section675Mode::NotApplicableNoTrustOrNoAdministrativePower
        );
    }

    #[test]
    fn section_675_1_less_than_adequate_consideration_compliant() {
        let input = Input {
            power_to_deal_for_less_than_adequate_consideration: true,
            fiduciary_capacity_status:
                FiduciaryCapacityStatus::NonfiduciaryCapacityNoFiduciaryApprovalRequired,
            ..baseline_no_administrative_power()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section675Mode::CompliantSection675_1LessThanAdequateConsiderationActiveGrantorTrust
        );
        assert_eq!(
            result.triggered_subsection,
            Section675Subsection::Section675_1PowerToDealForLessThanAdequateConsideration
        );
    }

    #[test]
    fn section_675_2_power_to_borrow_without_adequate_terms_compliant() {
        let input = Input {
            power_to_borrow_without_adequate_interest_or_security: true,
            fiduciary_capacity_status:
                FiduciaryCapacityStatus::NonfiduciaryCapacityNoFiduciaryApprovalRequired,
            borrowing_exception_status: BorrowingExceptionStatus::InadequateInterestOrSecurity,
            ..baseline_no_administrative_power()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section675Mode::CompliantSection675_2PowerToBorrowWithoutAdequateTermsActiveGrantorTrust
        );
    }

    #[test]
    fn section_675_2_general_lending_power_exception_deactivates() {
        let input = Input {
            power_to_borrow_without_adequate_interest_or_security: true,
            borrowing_exception_status:
                BorrowingExceptionStatus::AdequateInterestAndSecurityTrusteeIsNotGrantorNorSubservient,
            ..baseline_no_administrative_power()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section675Mode::CompliantSection675_2GeneralLendingPowerExceptionDeactivates675_2
        );
    }

    #[test]
    fn section_675_3_borrowing_not_repaid_compliant() {
        let input = Input {
            grantor_actually_borrowed_and_not_repaid_before_taxable_year: true,
            borrowing_exception_status:
                BorrowingExceptionStatus::BorrowingNotRepaidBeforeTaxableYear,
            ..baseline_no_administrative_power()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section675Mode::CompliantSection675_3BorrowedAndNotRepaidActiveGrantorTrust
        );
    }

    #[test]
    fn section_675_3_independent_trustee_exception_deactivates() {
        let input = Input {
            grantor_actually_borrowed_and_not_repaid_before_taxable_year: true,
            borrowing_exception_status:
                BorrowingExceptionStatus::AdequateInterestAndSecurityTrusteeIsNotGrantorNorSubservient,
            ..baseline_no_administrative_power()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section675Mode::CompliantSection675_3AdequateInterestAndSecurityTrusteeExceptionDeactivates675_3
        );
    }

    #[test]
    fn section_675_4a_voting_power_compliant() {
        let input = Input {
            power_to_vote_significant_corporate_holdings: true,
            voting_holdings_basis_points: 2_000,
            fiduciary_capacity_status:
                FiduciaryCapacityStatus::NonfiduciaryCapacityNoFiduciaryApprovalRequired,
            ..baseline_no_administrative_power()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section675Mode::CompliantSection675_4AVotingPowerNonfiduciaryActiveGrantorTrust
        );
    }

    #[test]
    fn section_675_4a_voting_power_below_10_pct_not_significant() {
        let input = Input {
            power_to_vote_significant_corporate_holdings: true,
            voting_holdings_basis_points: 999,
            fiduciary_capacity_status:
                FiduciaryCapacityStatus::NonfiduciaryCapacityNoFiduciaryApprovalRequired,
            ..baseline_no_administrative_power()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section675Mode::NotApplicableNoTrustOrNoAdministrativePower
        );
    }

    #[test]
    fn section_675_4a_voting_power_at_exactly_10_pct_significant() {
        let input = Input {
            power_to_vote_significant_corporate_holdings: true,
            voting_holdings_basis_points: 1_000,
            fiduciary_capacity_status:
                FiduciaryCapacityStatus::NonfiduciaryCapacityNoFiduciaryApprovalRequired,
            ..baseline_no_administrative_power()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section675Mode::CompliantSection675_4AVotingPowerNonfiduciaryActiveGrantorTrust
        );
    }

    #[test]
    fn section_675_4b_investment_control_compliant() {
        let input = Input {
            power_to_control_investment_significant_corporate_holdings: true,
            voting_holdings_basis_points: 1_500,
            fiduciary_capacity_status:
                FiduciaryCapacityStatus::NonfiduciaryCapacityNoFiduciaryApprovalRequired,
            ..baseline_no_administrative_power()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section675Mode::CompliantSection675_4BInvestmentControlNonfiduciaryActiveGrantorTrust
        );
    }

    #[test]
    fn section_675_4c_substitution_power_classic_idgt_compliant() {
        let input = Input {
            power_to_substitute_corpus_for_equivalent_value: true,
            fiduciary_capacity_status:
                FiduciaryCapacityStatus::NonfiduciaryCapacityNoFiduciaryApprovalRequired,
            ..baseline_no_administrative_power()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section675Mode::CompliantSection675_4CSubstitutionPowerClassicIdgtActiveGrantorTrustNoEstateInclusion
        );
        assert_eq!(
            result.triggered_subsection,
            Section675Subsection::Section675_4CSubstitutionPowerClassicIdgt
        );
    }

    #[test]
    fn section_675_4c_substitution_fiduciary_capacity_rev_rul_2008_22_protects_estate() {
        let input = Input {
            power_to_substitute_corpus_for_equivalent_value: true,
            fiduciary_capacity_status:
                FiduciaryCapacityStatus::FiduciaryCapacityTrusteeOrFiduciaryControl,
            ..baseline_no_administrative_power()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section675Mode::CompliantSection675_4CSubstitutionPowerFiduciaryCapacityRevRul2008_22ProtectsEstate
        );
    }

    #[test]
    fn section_675_4c_substitution_shifts_value_violation() {
        let input = Input {
            power_to_substitute_corpus_for_equivalent_value: true,
            substitution_power_exercised_to_shift_value_among_beneficiaries: true,
            fiduciary_capacity_status:
                FiduciaryCapacityStatus::NonfiduciaryCapacityNoFiduciaryApprovalRequired,
            ..baseline_no_administrative_power()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section675Mode::ViolationSubstitutionPowerExercisedToShiftValueAmongBeneficiariesEstateInclusionRisk
        );
    }

    #[test]
    fn section_675_active_but_form_1040_omitted_violation() {
        let input = Input {
            power_to_substitute_corpus_for_equivalent_value: true,
            grantor_trust_income_reported_on_form_1040: false,
            fiduciary_capacity_status:
                FiduciaryCapacityStatus::NonfiduciaryCapacityNoFiduciaryApprovalRequired,
            ..baseline_no_administrative_power()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section675Mode::ViolationSection675ActiveButGrantorTrustIncomeNotReportedOnForm1040
        );
    }

    #[test]
    fn section_675_2_borrowing_exception_inappropriately_claimed() {
        let input = Input {
            grantor_actually_borrowed_and_not_repaid_before_taxable_year: true,
            borrowing_exception_status: BorrowingExceptionStatus::InadequateInterestOrSecurity,
            grantor_trust_income_reported_on_form_1040: false,
            ..baseline_no_administrative_power()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section675Mode::ViolationSection675ActiveButGrantorTrustIncomeNotReportedOnForm1040
        );
        assert_eq!(
            result.triggered_subsection,
            Section675Subsection::Section675_3BorrowingOfTrustFundsNotRepaid
        );
    }

    #[test]
    fn citations_pin_section_675_subsections_and_rev_ruls() {
        let result = check(&baseline_no_administrative_power());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("IRC § 675(1)"));
        assert!(joined.contains("IRC § 675(2)"));
        assert!(joined.contains("IRC § 675(3)"));
        assert!(joined.contains("IRC § 675(4)"));
        assert!(joined.contains("IRC § 675(4)(A)"));
        assert!(joined.contains("IRC § 675(4)(B)"));
        assert!(joined.contains("IRC § 675(4)(C)"));
        assert!(joined.contains("LESS THAN ADEQUATE CONSIDERATION"));
        assert!(joined.contains("SUBSTITUTE TRUST CORPUS"));
        assert!(joined.contains("Rev. Rul. 2008-22"));
        assert!(joined.contains("Rev. Rul. 2011-28"));
        assert!(joined.contains("§ 2036"));
        assert!(joined.contains("§ 2038"));
        assert!(joined.contains("§ 2042"));
        assert!(joined.contains("Treas. Reg. § 1.675-1"));
        assert!(joined.contains("Paul Hood"));
        assert!(joined.contains("Kitces"));
        assert!(joined.contains("IDGT"));
        assert!(joined.contains("swap power"));
    }

    #[test]
    fn constant_pin_subsection_numbers_and_voting_threshold() {
        assert_eq!(IRC_675_SUBSECTION_1, 1);
        assert_eq!(IRC_675_SUBSECTION_2, 2);
        assert_eq!(IRC_675_SUBSECTION_3, 3);
        assert_eq!(IRC_675_SUBSECTION_4, 4);
        assert_eq!(IRC_675_4_A_VOTING_POWER_SIGNIFICANT_BASIS_POINTS, 1_000);
        assert_eq!(IRC_675_4_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(REV_RUL_2008_22_YEAR, 2008);
        assert_eq!(REV_RUL_2011_28_YEAR, 2011);
    }
}
