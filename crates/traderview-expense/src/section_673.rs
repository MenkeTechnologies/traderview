//! IRC § 673 — Reversionary Interests / Grantor Trust Module.
//!
//! Pure-compute check for grantor trust status under Internal
//! Revenue Code § 673 ("Reversionary interests"). § 673 is the
//! FIRST substantive grantor-trust trigger in the §§ 671-679
//! statutory progression (after § 672 definitions, before § 674
//! beneficial enjoyment, § 675 administrative powers — built
//! iter 644, § 676 power to revoke — built iter 646, § 677
//! income for benefit of grantor — built iter 642, § 678 person
//! other than grantor — built iter 640, § 679 foreign trusts).
//! Trader / family-office critical because § 673 is the
//! foundational substantive trigger for **GRAT (Grantor
//! Retained Annuity Trust)** and **GRUT (Grantor Retained
//! Unitrust)** grantor trust status — the grantor's annuity
//! interest under § 2702 valuation creates the reversionary
//! interest that triggers § 673 grantor trust treatment, while
//! the remainder beneficiary takes the residual asset at end
//! of term with no gift tax cost.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 673(a) General Rule**: grantor treated as owner of
//!   any portion of a trust in which he has a **reversionary
//!   interest** in either the corpus or the income therefrom,
//!   if, **as of the inception of that portion of the trust**,
//!   the value of such interest exceeds **5 PERCENT** of the
//!   value of such portion ([Cornell LII 26 USC § 673](https://www.law.cornell.edu/uscode/text/26/673);
//!   [Tax Notes Code Sec. 673](https://www.taxnotes.com/research/federal/usc26/673)).
//! - **Tax Reform Act of 1986** (Public Law 99-514): amended
//!   § 673 by substituting the **5 PERCENT PRESENT VALUE TEST**
//!   for the prior "10-year rule" (interest will or may
//!   reasonably be expected to take effect in possession or
//!   enjoyment within 10 years commencing with date of trust
//!   inception). The 10-year rule was the basis for the
//!   "Clifford trust" income-shifting structures used pre-1986;
//!   the 1986 amendment effectively eliminated Clifford-style
//!   short-term grantor trust planning.
//! - **IRC § 673(b) Minor Lineal Descendant Exception**: in the
//!   case of any beneficiary who is a **LINEAL DESCENDANT of
//!   the grantor** and holds all of the present interests in
//!   any portion of a trust, the grantor shall NOT be treated
//!   under subsection (a) as the owner of such portion solely
//!   by reason of a reversionary interest in such portion which
//!   takes effect upon the **DEATH of such beneficiary BEFORE
//!   such beneficiary attains age 21**.
//! - **IRC § 673(c) Special Rule for Determining Value**: for
//!   purposes of subsection (a), the value of the grantor's
//!   reversionary interest shall be determined by **assuming
//!   the MAXIMUM EXERCISE OF DISCRETION in favor of the
//!   grantor** (anti-avoidance valuation rule that prevents
//!   trustee discretion from reducing the actuarial reversion
//!   value below the 5 % threshold).
//! - **IRC § 673(d) Postponement Rule**: any postponement of
//!   the date specified for the reacquisition of possession or
//!   enjoyment of the reversionary interest shall be treated
//!   as a **NEW TRANSFER IN TRUST** commencing with the date
//!   on which the postponement is effective and terminating
//!   with the date prescribed by the postponement.
//! - **Valuation Method**: § 673(a) 5 % computation uses standard
//!   actuarial principles + Treasury § 7520 actuarial tables +
//!   monthly **applicable federal rate (AFR § 7520)** prevailing
//!   on date of transfer. Treas. Reg. § 1.673(a)-1 and § 1.673(c)-1
//!   implementing regulations.
//! - **GRAT (Grantor Retained Annuity Trust) Mechanics**: § 673
//!   is the substantive grantor-trust trigger when retained
//!   annuity interest value (computed under § 7520 tables)
//!   exceeds 5 % of trust corpus at inception; for a zeroed-out
//!   "Walton" GRAT (Walton v. Comm'r, 115 T.C. 589 (2000)),
//!   the annuity is engineered to equal trust corpus PV minus
//!   $0 remainder gift, ensuring § 673 trigger by definition.
//! - **GRUT (Grantor Retained Unitrust)**: parallel structure
//!   with unitrust interest (fixed percentage of annually
//!   revalued corpus); same § 673 5 % threshold analysis.
//! - **IRC § 7520 AFR**: monthly rate published by IRS used for
//!   actuarial valuation of split-interest gifts and reversions.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_673_REVERSIONARY_THRESHOLD_BASIS_POINTS: u64 = 500;
pub const IRC_673_BASIS_POINT_DENOMINATOR: u64 = 10_000;
pub const IRC_673_MINOR_BENEFICIARY_AGE_THRESHOLD: u32 = 21;
pub const TAX_REFORM_ACT_1986_YEAR: u32 = 1986;
pub const PRE_1986_CLIFFORD_RULE_YEARS: u32 = 10;
pub const WALTON_V_COMMR_YEAR: u32 = 2000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReversionaryInterestType {
    StandardReversionaryInterest,
    GratAnnuityInterestRetained,
    GrutUnitrustInterestRetained,
    NoReversionaryInterest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BeneficiaryCategory {
    LinealDescendantUnder21,
    LinealDescendant21OrOlder,
    NonLinealBeneficiary,
    NoBeneficiaryYet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PostponementStatus {
    NoPostponement,
    PostponementOfReacquisitionDate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section673Mode {
    NotApplicableNoTrustOrNoReversionaryInterest,
    NotApplicableValueAt5PctOrBelow,
    NotApplicableMinorLinealDescendantUnder21ExceptionUnderSection673B,
    CompliantSection673AReversionExceedsFivePctActiveGrantorTrust,
    CompliantGratAnnuityRetainedSection673AActiveGrantorTrust,
    CompliantGrutUnitrustRetainedSection673AActiveGrantorTrust,
    CompliantSection673CMaximumDiscretionValuationApplied,
    CompliantSection673DPostponementTreatedAsNewTransfer,
    ViolationSection673ActiveButGrantorTrustIncomeNotReportedOnForm1040,
    ViolationSection673BMinorExceptionClaimedButBeneficiaryNotLinealDescendant,
    ViolationSection673BMinorExceptionClaimedButBeneficiaryOver21,
    ViolationSection673CValuationManipulatedToReduceReversionBelow5Pct,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub trust_exists: bool,
    pub reversionary_interest_type: ReversionaryInterestType,
    pub reversion_value_basis_points: u64,
    pub beneficiary_category: BeneficiaryCategory,
    pub postponement_status: PostponementStatus,
    pub minor_lineal_descendant_exception_claimed: bool,
    pub maximum_discretion_valuation_applied_correctly: bool,
    pub grantor_trust_income_reported_on_form_1040: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section673Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section673Input = Input;
pub type Section673Output = Output;
pub type Section673Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 673(a) — grantor treated as owner of any portion of trust in which grantor has reversionary interest in corpus or income if value of interest exceeds 5 percent of trust portion value at inception".to_string(),
        "Tax Reform Act of 1986 (Public Law 99-514) — amended § 673 to substitute 5 % present-value test for prior 10-year Clifford-trust rule; effectively eliminated Clifford-style short-term grantor trust income-shifting".to_string(),
        "IRC § 673(b) — minor lineal descendant exception; grantor NOT treated as owner solely by reason of reversion taking effect on death of lineal-descendant beneficiary before beneficiary attains age 21".to_string(),
        "IRC § 673(c) — special rule for determining value of reversionary interest; value assumes MAXIMUM EXERCISE OF DISCRETION in favor of grantor (anti-avoidance valuation rule)".to_string(),
        "IRC § 673(d) — postponement of date for reacquisition of possession or enjoyment treated as NEW TRANSFER IN TRUST commencing with date postponement is effective".to_string(),
        "Pre-1986 § 673 10-year rule (Clifford trust era) — grantor treated as owner if reversion 'will or may reasonably be expected to take effect in possession or enjoyment within 10 years commencing with date of inception of trust'".to_string(),
        "IRC § 7520 — monthly applicable federal rate (AFR) used for actuarial valuation of split-interest gifts and reversions; § 673(a) computation uses standard actuarial principles + Treasury § 7520 tables".to_string(),
        "Treas. Reg. § 1.673(a)-1 — implementing regulation for § 673(a) reversionary interest valuation".to_string(),
        "Treas. Reg. § 1.673(c)-1 — implementing regulation for § 673(c) maximum discretion valuation rule".to_string(),
        "Walton v. Comm'r, 115 T.C. 589 (2000) — zeroed-out GRAT precedent confirming actuarial valuation of retained annuity equals trust corpus PV minus $0 remainder gift; ensures § 673 trigger by definition".to_string(),
        "GRAT (Grantor Retained Annuity Trust) — § 673 is substantive grantor-trust trigger when retained annuity interest value (§ 7520 tables) exceeds 5 % of trust corpus at inception".to_string(),
        "GRUT (Grantor Retained Unitrust) — parallel structure with unitrust interest (fixed percentage of annually revalued corpus); same § 673 5 % threshold analysis".to_string(),
        "IRC § 671 — Subpart E general attribution; § 673-triggered grantor trust status flows income, deductions, credits to grantor's Form 1040; trust files Form 1041 with grantor-trust statement".to_string(),
        "IRC § 676(b) postponement cross-reference — § 1.676(b)-1 postponement exception parallels § 673 5 % threshold analysis".to_string(),
    ];

    if !input.trust_exists {
        return Output {
            mode: Section673Mode::NotApplicableNoTrustOrNoReversionaryInterest,
            statutory_basis: "IRC § 673 inapplicable — no trust exists".to_string(),
            notes: "No trust exists; IRC § 673 inapplicable.".to_string(),
            citations,
        };
    }

    if input.reversionary_interest_type == ReversionaryInterestType::NoReversionaryInterest {
        return Output {
            mode: Section673Mode::NotApplicableNoTrustOrNoReversionaryInterest,
            statutory_basis:
                "IRC § 673 inapplicable — no reversionary interest retained by grantor".to_string(),
            notes: "No reversionary interest retained by grantor; § 673 not triggered.".to_string(),
            citations,
        };
    }

    if input.postponement_status == PostponementStatus::PostponementOfReacquisitionDate {
        return Output {
            mode: Section673Mode::CompliantSection673DPostponementTreatedAsNewTransfer,
            statutory_basis: "IRC § 673(d) — postponement treated as new transfer in trust".to_string(),
            notes: "COMPLIANT: § 673(d) postponement of reacquisition date treated as NEW TRANSFER IN TRUST commencing with postponement effective date and terminating with prescribed date.".to_string(),
            citations,
        };
    }

    if input.minor_lineal_descendant_exception_claimed {
        if input.beneficiary_category == BeneficiaryCategory::NonLinealBeneficiary {
            return Output {
                mode: Section673Mode::ViolationSection673BMinorExceptionClaimedButBeneficiaryNotLinealDescendant,
                statutory_basis: "IRC § 673(b) — minor exception limited to lineal descendant beneficiary".to_string(),
                notes: "VIOLATION: § 673(b) minor-beneficiary exception claimed but beneficiary is NOT a lineal descendant of grantor; exception inapplicable.".to_string(),
                citations,
            };
        }
        if input.beneficiary_category == BeneficiaryCategory::LinealDescendant21OrOlder {
            return Output {
                mode: Section673Mode::ViolationSection673BMinorExceptionClaimedButBeneficiaryOver21,
                statutory_basis: "IRC § 673(b) — minor exception limited to lineal-descendant beneficiary under age 21".to_string(),
                notes: "VIOLATION: § 673(b) minor-beneficiary exception claimed but lineal-descendant beneficiary is age 21 or older; exception inapplicable.".to_string(),
                citations,
            };
        }
        if input.beneficiary_category == BeneficiaryCategory::LinealDescendantUnder21 {
            return Output {
                mode: Section673Mode::NotApplicableMinorLinealDescendantUnder21ExceptionUnderSection673B,
                statutory_basis: "IRC § 673(b) — lineal descendant under 21 exception".to_string(),
                notes: "NOT APPLICABLE: § 673(b) exception applies; lineal descendant under age 21 holds all present interests; reversion taking effect only on death before age 21 does NOT trigger § 673(a) grantor trust status.".to_string(),
                citations,
            };
        }
    }

    if !input.maximum_discretion_valuation_applied_correctly {
        return Output {
            mode: Section673Mode::ViolationSection673CValuationManipulatedToReduceReversionBelow5Pct,
            statutory_basis: "IRC § 673(c) — maximum discretion valuation rule".to_string(),
            notes: "VIOLATION: § 673(c) valuation rule not applied correctly; § 673(c) requires reversion value computed assuming MAXIMUM EXERCISE OF DISCRETION in favor of grantor — manipulating trustee discretion downward to reduce reversion value below 5 % threshold is impermissible.".to_string(),
            citations,
        };
    }

    if input.reversion_value_basis_points <= IRC_673_REVERSIONARY_THRESHOLD_BASIS_POINTS {
        return Output {
            mode: Section673Mode::NotApplicableValueAt5PctOrBelow,
            statutory_basis: "IRC § 673(a) — reversion value 5 % or below threshold (statutory > 5 % reading)".to_string(),
            notes: format!(
                "NOT APPLICABLE: reversion value of {} basis points ≤ 500 bp (5 % threshold); § 673(a) requires value to EXCEED 5 % (statutory > reading); grantor trust status not triggered.",
                input.reversion_value_basis_points
            ),
            citations,
        };
    }

    if !input.grantor_trust_income_reported_on_form_1040 {
        return Output {
            mode: Section673Mode::ViolationSection673ActiveButGrantorTrustIncomeNotReportedOnForm1040,
            statutory_basis: "IRC § 673(a) — active grantor trust requires Form 1040 reporting via § 671 flow-through".to_string(),
            notes: format!(
                "VIOLATION: § 673(a) reversion value of {} basis points exceeds 5 % threshold; grantor trust status active but income not reported on Form 1040.",
                input.reversion_value_basis_points
            ),
            citations,
        };
    }

    match input.reversionary_interest_type {
        ReversionaryInterestType::GratAnnuityInterestRetained => Output {
            mode: Section673Mode::CompliantGratAnnuityRetainedSection673AActiveGrantorTrust,
            statutory_basis: "IRC § 673(a) — GRAT retained annuity interest exceeds 5 % threshold".to_string(),
            notes: format!(
                "COMPLIANT: GRAT retained annuity interest value of {} basis points exceeds 5 % threshold under § 673(a); grantor trust status active; Walton v. Comm'r zeroed-out GRAT structure available; income properly reported on Form 1040.",
                input.reversion_value_basis_points
            ),
            citations,
        },
        ReversionaryInterestType::GrutUnitrustInterestRetained => Output {
            mode: Section673Mode::CompliantGrutUnitrustRetainedSection673AActiveGrantorTrust,
            statutory_basis: "IRC § 673(a) — GRUT retained unitrust interest exceeds 5 % threshold".to_string(),
            notes: format!(
                "COMPLIANT: GRUT retained unitrust interest value of {} basis points exceeds 5 % threshold under § 673(a); grantor trust status active; income properly reported on Form 1040.",
                input.reversion_value_basis_points
            ),
            citations,
        },
        ReversionaryInterestType::StandardReversionaryInterest => Output {
            mode: Section673Mode::CompliantSection673AReversionExceedsFivePctActiveGrantorTrust,
            statutory_basis: "IRC § 673(a) — standard reversionary interest exceeds 5 % threshold".to_string(),
            notes: format!(
                "COMPLIANT: standard reversionary interest value of {} basis points exceeds 5 % threshold under § 673(a); grantor trust status active; income properly reported on Form 1040.",
                input.reversion_value_basis_points
            ),
            citations,
        },
        ReversionaryInterestType::NoReversionaryInterest => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_standard_reversion_compliant() -> Input {
        Input {
            trust_exists: true,
            reversionary_interest_type: ReversionaryInterestType::StandardReversionaryInterest,
            reversion_value_basis_points: 800,
            beneficiary_category: BeneficiaryCategory::NonLinealBeneficiary,
            postponement_status: PostponementStatus::NoPostponement,
            minor_lineal_descendant_exception_claimed: false,
            maximum_discretion_valuation_applied_correctly: true,
            grantor_trust_income_reported_on_form_1040: true,
        }
    }

    #[test]
    fn no_trust_not_applicable() {
        let input = Input {
            trust_exists: false,
            ..baseline_standard_reversion_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section673Mode::NotApplicableNoTrustOrNoReversionaryInterest
        );
    }

    #[test]
    fn no_reversionary_interest_not_applicable() {
        let input = Input {
            reversionary_interest_type: ReversionaryInterestType::NoReversionaryInterest,
            ..baseline_standard_reversion_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section673Mode::NotApplicableNoTrustOrNoReversionaryInterest
        );
    }

    #[test]
    fn standard_reversion_above_5_pct_compliant() {
        let result = check(&baseline_standard_reversion_compliant());
        assert_eq!(
            result.mode,
            Section673Mode::CompliantSection673AReversionExceedsFivePctActiveGrantorTrust
        );
    }

    #[test]
    fn reversion_at_exactly_5_pct_not_applicable() {
        let input = Input {
            reversion_value_basis_points: 500,
            ..baseline_standard_reversion_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, Section673Mode::NotApplicableValueAt5PctOrBelow);
    }

    #[test]
    fn reversion_at_501_bp_compliant() {
        let input = Input {
            reversion_value_basis_points: 501,
            ..baseline_standard_reversion_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section673Mode::CompliantSection673AReversionExceedsFivePctActiveGrantorTrust
        );
    }

    #[test]
    fn reversion_below_5_pct_not_applicable() {
        let input = Input {
            reversion_value_basis_points: 300,
            ..baseline_standard_reversion_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, Section673Mode::NotApplicableValueAt5PctOrBelow);
    }

    #[test]
    fn grat_annuity_retained_compliant() {
        let input = Input {
            reversionary_interest_type: ReversionaryInterestType::GratAnnuityInterestRetained,
            reversion_value_basis_points: 9_500,
            ..baseline_standard_reversion_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section673Mode::CompliantGratAnnuityRetainedSection673AActiveGrantorTrust
        );
    }

    #[test]
    fn grut_unitrust_retained_compliant() {
        let input = Input {
            reversionary_interest_type: ReversionaryInterestType::GrutUnitrustInterestRetained,
            reversion_value_basis_points: 6_000,
            ..baseline_standard_reversion_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section673Mode::CompliantGrutUnitrustRetainedSection673AActiveGrantorTrust
        );
    }

    #[test]
    fn section_673_b_minor_lineal_descendant_exception_applicable() {
        let input = Input {
            beneficiary_category: BeneficiaryCategory::LinealDescendantUnder21,
            minor_lineal_descendant_exception_claimed: true,
            ..baseline_standard_reversion_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section673Mode::NotApplicableMinorLinealDescendantUnder21ExceptionUnderSection673B
        );
    }

    #[test]
    fn section_673_b_minor_exception_beneficiary_not_lineal_violation() {
        let input = Input {
            beneficiary_category: BeneficiaryCategory::NonLinealBeneficiary,
            minor_lineal_descendant_exception_claimed: true,
            ..baseline_standard_reversion_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section673Mode::ViolationSection673BMinorExceptionClaimedButBeneficiaryNotLinealDescendant
        );
    }

    #[test]
    fn section_673_b_minor_exception_beneficiary_over_21_violation() {
        let input = Input {
            beneficiary_category: BeneficiaryCategory::LinealDescendant21OrOlder,
            minor_lineal_descendant_exception_claimed: true,
            ..baseline_standard_reversion_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section673Mode::ViolationSection673BMinorExceptionClaimedButBeneficiaryOver21
        );
    }

    #[test]
    fn section_673_c_valuation_manipulation_violation() {
        let input = Input {
            maximum_discretion_valuation_applied_correctly: false,
            ..baseline_standard_reversion_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section673Mode::ViolationSection673CValuationManipulatedToReduceReversionBelow5Pct
        );
    }

    #[test]
    fn section_673_d_postponement_treated_as_new_transfer() {
        let input = Input {
            postponement_status: PostponementStatus::PostponementOfReacquisitionDate,
            ..baseline_standard_reversion_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section673Mode::CompliantSection673DPostponementTreatedAsNewTransfer
        );
    }

    #[test]
    fn section_673_active_form_1040_omitted_violation() {
        let input = Input {
            grantor_trust_income_reported_on_form_1040: false,
            ..baseline_standard_reversion_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section673Mode::ViolationSection673ActiveButGrantorTrustIncomeNotReportedOnForm1040
        );
    }

    #[test]
    fn citations_pin_section_673_subsections_and_grat_grut() {
        let result = check(&baseline_standard_reversion_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("IRC § 673(a)"));
        assert!(joined.contains("IRC § 673(b)"));
        assert!(joined.contains("IRC § 673(c)"));
        assert!(joined.contains("IRC § 673(d)"));
        assert!(joined.contains("5 percent"));
        assert!(joined.contains("Tax Reform Act of 1986"));
        assert!(joined.contains("Public Law 99-514"));
        assert!(joined.contains("Clifford"));
        assert!(joined.contains("10-year rule"));
        assert!(joined.contains("age 21"));
        assert!(joined.contains("lineal descendant"));
        assert!(joined.contains("MAXIMUM EXERCISE OF DISCRETION"));
        assert!(joined.contains("NEW TRANSFER IN TRUST"));
        assert!(joined.contains("IRC § 7520"));
        assert!(joined.contains("Treas. Reg. § 1.673(a)-1"));
        assert!(joined.contains("Treas. Reg. § 1.673(c)-1"));
        assert!(joined.contains("Walton v. Comm'r"));
        assert!(joined.contains("GRAT"));
        assert!(joined.contains("GRUT"));
        assert!(joined.contains("IRC § 671"));
    }

    #[test]
    fn constant_pin_thresholds_and_dates() {
        assert_eq!(IRC_673_REVERSIONARY_THRESHOLD_BASIS_POINTS, 500);
        assert_eq!(IRC_673_BASIS_POINT_DENOMINATOR, 10_000);
        assert_eq!(IRC_673_MINOR_BENEFICIARY_AGE_THRESHOLD, 21);
        assert_eq!(TAX_REFORM_ACT_1986_YEAR, 1986);
        assert_eq!(PRE_1986_CLIFFORD_RULE_YEARS, 10);
        assert_eq!(WALTON_V_COMMR_YEAR, 2000);
    }
}
