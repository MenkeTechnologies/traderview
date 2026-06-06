//! IRC § 677 — Income for Benefit of Grantor
//! Grantor Trust Income Attribution Module.
//!
//! Pure-compute check for grantor trust status under Internal
//! Revenue Code § 677 ("Income for Benefit of Grantor"). § 677
//! is the third grantor-trust rule following § 671 (general),
//! § 673 (reversionary interest), § 674 (power to control
//! beneficial enjoyment), § 675 (administrative powers), § 676
//! (power to revoke), and is followed by § 678 (person other
//! than grantor as owner — built in iter 640). Trader critical
//! because grantor trust status pulls all trust income onto the
//! grantor's Form 1040 (income, deductions, credits) regardless
//! of whether the trust files Form 1041 — premature deactivation
//! or misapplication creates substantial IRS audit exposure +
//! § 6651/§ 6662 penalty exposure + estate inclusion issues
//! under § 2036 / § 2042.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 677(a)(1)**: grantor treated as owner of any
//!   portion of a trust whose income is, or in the discretion of
//!   the grantor or a nonadverse party (or both) without the
//!   approval or consent of any adverse party, may be
//!   **distributed to the grantor or the grantor's spouse**
//!   ([Cornell LII 26 USC § 677](https://www.law.cornell.edu/uscode/text/26/677);
//!   [26 CFR § 1.677(a)-1](https://www.ecfr.gov/current/title-26/chapter-I/subchapter-A/part-1/subject-group-ECFR245d884a8952b47/section-1.677(a)-1)).
//! - **IRC § 677(a)(2)**: same trigger for income **held or
//!   accumulated for future distribution to the grantor or
//!   the grantor's spouse**.
//! - **IRC § 677(a)(3)**: same trigger for income **applied to
//!   the payment of premiums on policies of insurance on the
//!   life of the grantor or the grantor's spouse** —
//!   "the ghost that haunts the divorced grantor" — except
//!   policies of insurance **irrevocably payable for charitable
//!   purposes** described in § 170(c).
//! - **IRC § 677(a) Spouse Rule (post-1969)**: applies only to
//!   property transferred in trust **after October 9, 1969**;
//!   only during the **period of marriage** of the grantor to
//!   the beneficiary spouse — § 677 deactivates when the
//!   marriage ends ("§ 677 — the ghost that haunts the divorced
//!   grantor"; Higgs Fletcher & Mack ABA 2021-09-23 presentation).
//! - **IRC § 677(b) Discharge of Legal Obligation**: grantor
//!   treated as owner of any portion of a trust whose income is,
//!   or in the discretion of the grantor or nonadverse party
//!   without approval or consent of adverse party may be,
//!   applied in **discharge of a legal obligation of the
//!   grantor or grantor's spouse** (for transfers after October
//!   9, 1969). Includes support obligations under state law for
//!   minor children of the grantor.
//! - **"Is" vs "May Be" Standards**: "is" standard applies where
//!   income is so applied for payment of premiums/distribution
//!   regardless of discretion (unless coupled with adverse-party
//!   approval); "may be" standard looks to discretionary use by
//!   grantor or nonadverse party — discretion ALONE creates
//!   grantor trust status without any actual exercise. ILIT
//!   premium-payment discretion = automatic § 677(a)(3) trigger.
//! - **Spouse Cannot Be Adverse**: under § 672(e) (added 1986),
//!   grantor treated as holding any power or interest held by
//!   any individual who was the spouse of the grantor at the
//!   time of the creation of such power or interest. Spouse
//!   is NEVER adverse for § 677 purposes — distinct from § 674
//!   where spouse adversity matters.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_677_POST_1969_EFFECTIVE_DATE_YEAR: u32 = 1969;
pub const IRC_677_POST_1969_EFFECTIVE_DATE_MONTH: u32 = 10;
pub const IRC_677_POST_1969_EFFECTIVE_DATE_DAY: u32 = 9;
pub const IRC_677_A_1_SUBSECTION_NUMBER: u32 = 1;
pub const IRC_677_A_2_SUBSECTION_NUMBER: u32 = 2;
pub const IRC_677_A_3_SUBSECTION_NUMBER: u32 = 3;
pub const IRC_672_E_SPOUSE_NON_ADVERSE_YEAR_ENACTED: u32 = 1986;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section677Subsection {
    Section677A1IncomeDistributableToGrantorOrSpouse,
    Section677A2IncomeHeldOrAccumulatedForGrantorOrSpouse,
    Section677A3LifeInsurancePremiumGrantorOrSpouse,
    Section677BDischargeOfLegalObligationGrantorOrSpouse,
    NoTriggerNoSubsection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdversePartyStatus {
    AdversePartyApprovalRequiredAndObtained,
    AdversePartyApprovalRequiredButNotObtained,
    NonAdversePartyDiscretionOnly,
    GrantorOrSpouseDirectControlNoAdverseParty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InsurancePolicyPurpose {
    LifeInsuranceGrantor,
    LifeInsuranceSpouse,
    LifeInsuranceCharitableIrrevocableSection170c,
    LifeInsuranceThirdParty,
    NoLifeInsurancePolicyInTrust,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MaritalStatus {
    MarriedAtCreationAndStillMarried,
    MarriedAtCreationDivorcedSince,
    NeverMarriedToBeneficiary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section677Mode {
    NotApplicableNoTrustExists,
    NotApplicableTrustIncomeNotPayableToGrantorOrSpouseOrLifeInsurance,
    CompliantSection677A1IncomeDistributedToGrantorActiveGrantorTrust,
    CompliantSection677A1IncomeDistributedToSpouseActiveGrantorTrust,
    CompliantSection677A2IncomeAccumulatedForGrantorOrSpouseActiveGrantorTrust,
    CompliantSection677A3LifeInsurancePremiumGrantorOrSpouseActiveGrantorTrust,
    CompliantSection677BDischargeOfLegalObligationGrantorOrSpouseActiveGrantorTrust,
    CompliantSection677ADivorceTerminatedSpousePortionDeactivated,
    CompliantAdversePartyApprovalDeactivatesSection677,
    CompliantCharitableIrrevocableInsurancePolicyExceptionUnderSection677A3,
    ViolationSection677ActiveButGrantorTrustIncomeNotReportedOnForm1040,
    ViolationSpouseTreatedAsAdversePartyInErrorUnderSection672E,
    ViolationPostDivorceSection677AppliedToFormerSpouseDespiteTermination,
    ViolationDischargeOfLegalObligationGrantorIncomeOmittedFromForm1040,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub trust_exists: bool,
    pub income_distributable_to_grantor: bool,
    pub income_distributable_to_spouse: bool,
    pub income_held_or_accumulated_for_grantor_or_spouse: bool,
    pub income_applied_to_life_insurance_premium: bool,
    pub insurance_policy_purpose: InsurancePolicyPurpose,
    pub income_applied_to_legal_obligation_of_grantor_or_spouse: bool,
    pub adverse_party_status: AdversePartyStatus,
    pub marital_status: MaritalStatus,
    pub spouse_treated_as_adverse: bool,
    pub trust_property_transferred_after_oct_9_1969: bool,
    pub grantor_trust_income_reported_on_form_1040: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section677Mode,
    pub triggered_subsection: Section677Subsection,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section677Input = Input;
pub type Section677Output = Output;
pub type Section677Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 677(a)(1) — grantor treated as owner if income is, or in discretion of grantor/nonadverse party may be, distributed to grantor or grantor's spouse (without approval/consent of adverse party)".to_string(),
        "IRC § 677(a)(2) — same trigger for income held or accumulated for future distribution to grantor or grantor's spouse".to_string(),
        "IRC § 677(a)(3) — same trigger for income applied to payment of premiums on policies of insurance on life of grantor or grantor's spouse (except policies irrevocably payable for charitable purposes described in § 170(c))".to_string(),
        "IRC § 677(a) Spouse Rule — applies only to property transferred in trust after October 9, 1969; only during period of marriage of grantor to beneficiary spouse".to_string(),
        "IRC § 677(b) — grantor treated as owner of portion of trust whose income may be applied in discharge of legal obligation of grantor or grantor's spouse (for transfers after October 9, 1969); includes state-law support obligations for minor children".to_string(),
        "IRC § 672(e) (enacted 1986) — grantor treated as holding any power or interest held by individual who was spouse at time of creation of power/interest; spouse cannot be adverse for § 677 purposes".to_string(),
        "26 CFR § 1.677(a)-1 General Rule — implementing regulation for § 677(a); 'is' standard vs 'may be' discretion standard; discretion ALONE creates grantor trust status without actual exercise".to_string(),
        "26 CFR § 1.677(b)-1 — implementing regulation for § 677(b) discharge of legal obligation".to_string(),
        "Higgs Fletcher & Mack 2021-09-23 ABA Presentation — '§ 677 The Ghost That Haunts the Divorced Grantor' — § 677(a)(3) ILIT premium continues to trigger grantor trust status post-divorce until policy modified".to_string(),
        "Asena Advisors — IRC § 677 commentary; ILIT (Irrevocable Life Insurance Trust) § 677(a)(3) automatic trigger when trustee has discretion to pay premium".to_string(),
        "Premier Trust IRC 671-679 Tax Review Checklist — § 677 third in grantor-trust statutory progression".to_string(),
        "IRC § 671 — Trust income, deductions, and credits attributable to grantors and others as substantial owners (general grantor trust statute)".to_string(),
        "IRC § 678 — Person Other Than Grantor as Owner (companion BDIT/Section-678 statute — built in iter 640 of this codebase)".to_string(),
        "Form 1041 trust income tax return + Form 1040 grantor reporting; grantor trust statement attaches to 1041 when grantor-trust treatment elected".to_string(),
    ];

    if !input.trust_exists {
        return Output {
            mode: Section677Mode::NotApplicableNoTrustExists,
            triggered_subsection: Section677Subsection::NoTriggerNoSubsection,
            statutory_basis: "IRC § 677 inapplicable — no trust exists".to_string(),
            notes: "No trust exists; IRC § 677 inapplicable.".to_string(),
            citations,
        };
    }

    if input.income_applied_to_life_insurance_premium
        && matches!(
            input.insurance_policy_purpose,
            InsurancePolicyPurpose::LifeInsuranceCharitableIrrevocableSection170c
        )
    {
        return Output {
            mode: Section677Mode::CompliantCharitableIrrevocableInsurancePolicyExceptionUnderSection677A3,
            triggered_subsection: Section677Subsection::Section677A3LifeInsurancePremiumGrantorOrSpouse,
            statutory_basis: "IRC § 677(a)(3) exception — policies irrevocably payable for charitable purposes under § 170(c)".to_string(),
            notes: "COMPLIANT: life insurance premium applied to policy irrevocably payable for charitable purposes under § 170(c); § 677(a)(3) statutory exception applies; no grantor trust status from premium trigger.".to_string(),
            citations,
        };
    }

    let triggered_by_subsection_a1_or_a2 = input.income_distributable_to_grantor
        || input.income_distributable_to_spouse
        || input.income_held_or_accumulated_for_grantor_or_spouse;
    let triggered_by_subsection_a3 = input.income_applied_to_life_insurance_premium
        && matches!(
            input.insurance_policy_purpose,
            InsurancePolicyPurpose::LifeInsuranceGrantor
                | InsurancePolicyPurpose::LifeInsuranceSpouse
        );
    let triggered_by_subsection_b = input.income_applied_to_legal_obligation_of_grantor_or_spouse;

    if !triggered_by_subsection_a1_or_a2
        && !triggered_by_subsection_a3
        && !triggered_by_subsection_b
    {
        return Output {
            mode: Section677Mode::NotApplicableTrustIncomeNotPayableToGrantorOrSpouseOrLifeInsurance,
            triggered_subsection: Section677Subsection::NoTriggerNoSubsection,
            statutory_basis: "IRC § 677 not triggered — no trust income flows to grantor, spouse, life insurance premium, or grantor's legal obligation".to_string(),
            notes: "No § 677 trigger present; trust income not distributable, accumulated, applied to life insurance premium, or applied to legal obligation of grantor or spouse.".to_string(),
            citations,
        };
    }

    if input.spouse_treated_as_adverse {
        return Output {
            mode: Section677Mode::ViolationSpouseTreatedAsAdversePartyInErrorUnderSection672E,
            triggered_subsection: Section677Subsection::Section677A1IncomeDistributableToGrantorOrSpouse,
            statutory_basis: "IRC § 672(e) — spouse cannot be adverse party; § 677 triggers regardless".to_string(),
            notes: "VIOLATION: spouse treated as adverse party; under IRC § 672(e) (enacted 1986), grantor's spouse is NEVER adverse for § 677 purposes — § 677 triggers regardless of spouse's purported objection.".to_string(),
            citations,
        };
    }

    if input.adverse_party_status == AdversePartyStatus::AdversePartyApprovalRequiredAndObtained {
        return Output {
            mode: Section677Mode::CompliantAdversePartyApprovalDeactivatesSection677,
            triggered_subsection: Section677Subsection::Section677A1IncomeDistributableToGrantorOrSpouse,
            statutory_basis: "IRC § 677(a) — adverse-party approval requirement satisfied".to_string(),
            notes: "COMPLIANT: distribution/accumulation/premium discretion conditioned on adverse-party approval that has been obtained; § 677 does NOT trigger when adverse-party approval required AND obtained (excluding spouse who is non-adverse under § 672(e)).".to_string(),
            citations,
        };
    }

    if input.income_distributable_to_spouse
        && input.marital_status == MaritalStatus::MarriedAtCreationDivorcedSince
        && !input.income_distributable_to_grantor
        && !input.income_held_or_accumulated_for_grantor_or_spouse
        && !triggered_by_subsection_a3
        && !triggered_by_subsection_b
    {
        return Output {
            mode: Section677Mode::CompliantSection677ADivorceTerminatedSpousePortionDeactivated,
            triggered_subsection: Section677Subsection::Section677A1IncomeDistributableToGrantorOrSpouse,
            statutory_basis: "IRC § 677(a) — applies only during period of marriage of grantor to beneficiary spouse".to_string(),
            notes: "COMPLIANT: § 677(a) deactivated post-divorce; spouse trigger applies ONLY during period of marriage; grantor no longer treated as owner of spouse-distribution portion after marriage terminates.".to_string(),
            citations,
        };
    }

    if input.income_distributable_to_spouse
        && input.marital_status == MaritalStatus::MarriedAtCreationDivorcedSince
        && !input.grantor_trust_income_reported_on_form_1040
    {
        return Output {
            mode: Section677Mode::CompliantSection677ADivorceTerminatedSpousePortionDeactivated,
            triggered_subsection: Section677Subsection::Section677A1IncomeDistributableToGrantorOrSpouse,
            statutory_basis: "IRC § 677(a) — applies only during period of marriage".to_string(),
            notes: "COMPLIANT: § 677(a) deactivated post-divorce; spouse-portion grantor trust status terminates upon dissolution of marriage.".to_string(),
            citations,
        };
    }

    if triggered_by_subsection_a3
        && input.marital_status == MaritalStatus::MarriedAtCreationDivorcedSince
        && matches!(
            input.insurance_policy_purpose,
            InsurancePolicyPurpose::LifeInsuranceSpouse
        )
        && input.grantor_trust_income_reported_on_form_1040
    {
        return Output {
            mode: Section677Mode::ViolationPostDivorceSection677AppliedToFormerSpouseDespiteTermination,
            triggered_subsection: Section677Subsection::Section677A3LifeInsurancePremiumGrantorOrSpouse,
            statutory_basis: "IRC § 677(a) — applies only during period of marriage; deactivates post-divorce".to_string(),
            notes: "VIOLATION: grantor reporting § 677(a)(3) ILIT premium-grantor-trust income on Form 1040 for former-spouse life insurance policy post-divorce; § 677(a) spouse trigger deactivates upon marriage termination — over-reporting trust income.".to_string(),
            citations,
        };
    }

    if triggered_by_subsection_b && !input.grantor_trust_income_reported_on_form_1040 {
        return Output {
            mode: Section677Mode::ViolationDischargeOfLegalObligationGrantorIncomeOmittedFromForm1040,
            triggered_subsection: Section677Subsection::Section677BDischargeOfLegalObligationGrantorOrSpouse,
            statutory_basis: "IRC § 677(b) — discharge of legal obligation triggers grantor trust status".to_string(),
            notes: "VIOLATION: trust income applied in discharge of grantor's (or spouse's) legal obligation activates § 677(b) grantor trust status; grantor failed to report income on Form 1040.".to_string(),
            citations,
        };
    }

    if (triggered_by_subsection_a1_or_a2 || triggered_by_subsection_a3)
        && !input.grantor_trust_income_reported_on_form_1040
    {
        return Output {
            mode: Section677Mode::ViolationSection677ActiveButGrantorTrustIncomeNotReportedOnForm1040,
            triggered_subsection: if triggered_by_subsection_a3 {
                Section677Subsection::Section677A3LifeInsurancePremiumGrantorOrSpouse
            } else if input.income_held_or_accumulated_for_grantor_or_spouse {
                Section677Subsection::Section677A2IncomeHeldOrAccumulatedForGrantorOrSpouse
            } else {
                Section677Subsection::Section677A1IncomeDistributableToGrantorOrSpouse
            },
            statutory_basis: "IRC § 677(a) — active grantor trust requires Form 1040 reporting".to_string(),
            notes: "VIOLATION: § 677(a) active grantor trust status triggered but grantor trust income not reported on Form 1040; § 671 income/deductions/credits flow-through omitted.".to_string(),
            citations,
        };
    }

    if triggered_by_subsection_b {
        return Output {
            mode: Section677Mode::CompliantSection677BDischargeOfLegalObligationGrantorOrSpouseActiveGrantorTrust,
            triggered_subsection: Section677Subsection::Section677BDischargeOfLegalObligationGrantorOrSpouse,
            statutory_basis: "IRC § 677(b) — discharge of legal obligation".to_string(),
            notes: "COMPLIANT: § 677(b) active grantor trust status; trust income applied in discharge of legal obligation of grantor or spouse; income properly reported on Form 1040.".to_string(),
            citations,
        };
    }

    if triggered_by_subsection_a3 {
        return Output {
            mode: Section677Mode::CompliantSection677A3LifeInsurancePremiumGrantorOrSpouseActiveGrantorTrust,
            triggered_subsection: Section677Subsection::Section677A3LifeInsurancePremiumGrantorOrSpouse,
            statutory_basis: "IRC § 677(a)(3) — life insurance premium on life of grantor or spouse".to_string(),
            notes: "COMPLIANT: § 677(a)(3) ILIT-style premium-payment trigger; grantor treated as owner; income properly reported on Form 1040.".to_string(),
            citations,
        };
    }

    if input.income_distributable_to_grantor {
        return Output {
            mode: Section677Mode::CompliantSection677A1IncomeDistributedToGrantorActiveGrantorTrust,
            triggered_subsection: Section677Subsection::Section677A1IncomeDistributableToGrantorOrSpouse,
            statutory_basis: "IRC § 677(a)(1) — income distributable to grantor".to_string(),
            notes: "COMPLIANT: § 677(a)(1) income distributable to grantor; grantor treated as owner; income properly reported on Form 1040.".to_string(),
            citations,
        };
    }

    if input.income_distributable_to_spouse {
        return Output {
            mode: Section677Mode::CompliantSection677A1IncomeDistributedToSpouseActiveGrantorTrust,
            triggered_subsection: Section677Subsection::Section677A1IncomeDistributableToGrantorOrSpouse,
            statutory_basis: "IRC § 677(a)(1) — income distributable to spouse".to_string(),
            notes: "COMPLIANT: § 677(a)(1) income distributable to spouse; grantor treated as owner during period of marriage; income properly reported on Form 1040.".to_string(),
            citations,
        };
    }

    Output {
        mode: Section677Mode::CompliantSection677A2IncomeAccumulatedForGrantorOrSpouseActiveGrantorTrust,
        triggered_subsection: Section677Subsection::Section677A2IncomeHeldOrAccumulatedForGrantorOrSpouse,
        statutory_basis: "IRC § 677(a)(2) — income held or accumulated for grantor or spouse".to_string(),
        notes: "COMPLIANT: § 677(a)(2) income held or accumulated for future distribution to grantor or spouse; grantor treated as owner; income properly reported on Form 1040.".to_string(),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_no_trigger() -> Input {
        Input {
            trust_exists: true,
            income_distributable_to_grantor: false,
            income_distributable_to_spouse: false,
            income_held_or_accumulated_for_grantor_or_spouse: false,
            income_applied_to_life_insurance_premium: false,
            insurance_policy_purpose: InsurancePolicyPurpose::NoLifeInsurancePolicyInTrust,
            income_applied_to_legal_obligation_of_grantor_or_spouse: false,
            adverse_party_status: AdversePartyStatus::GrantorOrSpouseDirectControlNoAdverseParty,
            marital_status: MaritalStatus::MarriedAtCreationAndStillMarried,
            spouse_treated_as_adverse: false,
            trust_property_transferred_after_oct_9_1969: true,
            grantor_trust_income_reported_on_form_1040: true,
        }
    }

    #[test]
    fn no_trust_not_applicable() {
        let input = Input {
            trust_exists: false,
            ..baseline_no_trigger()
        };
        let result = check(&input);
        assert_eq!(result.mode, Section677Mode::NotApplicableNoTrustExists);
    }

    #[test]
    fn no_trigger_not_applicable() {
        let result = check(&baseline_no_trigger());
        assert_eq!(
            result.mode,
            Section677Mode::NotApplicableTrustIncomeNotPayableToGrantorOrSpouseOrLifeInsurance
        );
    }

    #[test]
    fn section_677_a1_income_to_grantor_compliant() {
        let input = Input {
            income_distributable_to_grantor: true,
            ..baseline_no_trigger()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section677Mode::CompliantSection677A1IncomeDistributedToGrantorActiveGrantorTrust
        );
        assert_eq!(
            result.triggered_subsection,
            Section677Subsection::Section677A1IncomeDistributableToGrantorOrSpouse
        );
    }

    #[test]
    fn section_677_a1_income_to_spouse_compliant() {
        let input = Input {
            income_distributable_to_spouse: true,
            ..baseline_no_trigger()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section677Mode::CompliantSection677A1IncomeDistributedToSpouseActiveGrantorTrust
        );
    }

    #[test]
    fn section_677_a2_accumulated_for_grantor_compliant() {
        let input = Input {
            income_held_or_accumulated_for_grantor_or_spouse: true,
            ..baseline_no_trigger()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section677Mode::CompliantSection677A2IncomeAccumulatedForGrantorOrSpouseActiveGrantorTrust
        );
        assert_eq!(
            result.triggered_subsection,
            Section677Subsection::Section677A2IncomeHeldOrAccumulatedForGrantorOrSpouse
        );
    }

    #[test]
    fn section_677_a3_ilit_premium_grantor_life_compliant() {
        let input = Input {
            income_applied_to_life_insurance_premium: true,
            insurance_policy_purpose: InsurancePolicyPurpose::LifeInsuranceGrantor,
            ..baseline_no_trigger()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section677Mode::CompliantSection677A3LifeInsurancePremiumGrantorOrSpouseActiveGrantorTrust
        );
    }

    #[test]
    fn section_677_a3_ilit_premium_spouse_life_compliant() {
        let input = Input {
            income_applied_to_life_insurance_premium: true,
            insurance_policy_purpose: InsurancePolicyPurpose::LifeInsuranceSpouse,
            ..baseline_no_trigger()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section677Mode::CompliantSection677A3LifeInsurancePremiumGrantorOrSpouseActiveGrantorTrust
        );
    }

    #[test]
    fn section_677_a3_charitable_irrevocable_exception_compliant() {
        let input = Input {
            income_applied_to_life_insurance_premium: true,
            insurance_policy_purpose:
                InsurancePolicyPurpose::LifeInsuranceCharitableIrrevocableSection170c,
            ..baseline_no_trigger()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section677Mode::CompliantCharitableIrrevocableInsurancePolicyExceptionUnderSection677A3
        );
    }

    #[test]
    fn section_677_a3_third_party_life_insurance_not_triggered() {
        let input = Input {
            income_applied_to_life_insurance_premium: true,
            insurance_policy_purpose: InsurancePolicyPurpose::LifeInsuranceThirdParty,
            ..baseline_no_trigger()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section677Mode::NotApplicableTrustIncomeNotPayableToGrantorOrSpouseOrLifeInsurance
        );
    }

    #[test]
    fn section_677_b_discharge_of_legal_obligation_compliant() {
        let input = Input {
            income_applied_to_legal_obligation_of_grantor_or_spouse: true,
            ..baseline_no_trigger()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section677Mode::CompliantSection677BDischargeOfLegalObligationGrantorOrSpouseActiveGrantorTrust
        );
        assert_eq!(
            result.triggered_subsection,
            Section677Subsection::Section677BDischargeOfLegalObligationGrantorOrSpouse
        );
    }

    #[test]
    fn divorce_terminates_spouse_distribution_portion_compliant() {
        let input = Input {
            income_distributable_to_spouse: true,
            marital_status: MaritalStatus::MarriedAtCreationDivorcedSince,
            grantor_trust_income_reported_on_form_1040: false,
            ..baseline_no_trigger()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section677Mode::CompliantSection677ADivorceTerminatedSpousePortionDeactivated
        );
    }

    #[test]
    fn adverse_party_approval_obtained_deactivates_section_677() {
        let input = Input {
            income_distributable_to_grantor: true,
            adverse_party_status: AdversePartyStatus::AdversePartyApprovalRequiredAndObtained,
            ..baseline_no_trigger()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section677Mode::CompliantAdversePartyApprovalDeactivatesSection677
        );
    }

    #[test]
    fn spouse_treated_as_adverse_violation_under_section_672e() {
        let input = Input {
            income_distributable_to_spouse: true,
            spouse_treated_as_adverse: true,
            ..baseline_no_trigger()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section677Mode::ViolationSpouseTreatedAsAdversePartyInErrorUnderSection672E
        );
    }

    #[test]
    fn section_677_active_but_form_1040_omitted_violation() {
        let input = Input {
            income_distributable_to_grantor: true,
            grantor_trust_income_reported_on_form_1040: false,
            ..baseline_no_trigger()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section677Mode::ViolationSection677ActiveButGrantorTrustIncomeNotReportedOnForm1040
        );
    }

    #[test]
    fn section_677_a3_ilit_premium_form_1040_omitted_violation() {
        let input = Input {
            income_applied_to_life_insurance_premium: true,
            insurance_policy_purpose: InsurancePolicyPurpose::LifeInsuranceGrantor,
            grantor_trust_income_reported_on_form_1040: false,
            ..baseline_no_trigger()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section677Mode::ViolationSection677ActiveButGrantorTrustIncomeNotReportedOnForm1040
        );
        assert_eq!(
            result.triggered_subsection,
            Section677Subsection::Section677A3LifeInsurancePremiumGrantorOrSpouse
        );
    }

    #[test]
    fn post_divorce_ilit_premium_over_reporting_violation() {
        let input = Input {
            income_applied_to_life_insurance_premium: true,
            insurance_policy_purpose: InsurancePolicyPurpose::LifeInsuranceSpouse,
            marital_status: MaritalStatus::MarriedAtCreationDivorcedSince,
            ..baseline_no_trigger()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section677Mode::ViolationPostDivorceSection677AppliedToFormerSpouseDespiteTermination
        );
    }

    #[test]
    fn section_677_b_legal_obligation_form_1040_omitted_violation() {
        let input = Input {
            income_applied_to_legal_obligation_of_grantor_or_spouse: true,
            grantor_trust_income_reported_on_form_1040: false,
            ..baseline_no_trigger()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section677Mode::ViolationDischargeOfLegalObligationGrantorIncomeOmittedFromForm1040
        );
    }

    #[test]
    fn citations_pin_section_677_subsections_and_companion_statutes() {
        let result = check(&baseline_no_trigger());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("IRC § 677(a)(1)"));
        assert!(joined.contains("IRC § 677(a)(2)"));
        assert!(joined.contains("IRC § 677(a)(3)"));
        assert!(joined.contains("IRC § 677(b)"));
        assert!(joined.contains("October 9, 1969"));
        assert!(joined.contains("IRC § 672(e)"));
        assert!(joined.contains("1986"));
        assert!(joined.contains("§ 170(c)"));
        assert!(joined.contains("26 CFR § 1.677(a)-1"));
        assert!(joined.contains("Higgs Fletcher"));
        assert!(joined.contains("ILIT"));
        assert!(joined.contains("IRC § 671"));
        assert!(joined.contains("IRC § 678"));
        assert!(joined.contains("Form 1041"));
        assert!(joined.contains("Form 1040"));
    }

    #[test]
    fn constant_pin_dates_and_subsection_numbers() {
        assert_eq!(IRC_677_POST_1969_EFFECTIVE_DATE_YEAR, 1969);
        assert_eq!(IRC_677_POST_1969_EFFECTIVE_DATE_MONTH, 10);
        assert_eq!(IRC_677_POST_1969_EFFECTIVE_DATE_DAY, 9);
        assert_eq!(IRC_677_A_1_SUBSECTION_NUMBER, 1);
        assert_eq!(IRC_677_A_2_SUBSECTION_NUMBER, 2);
        assert_eq!(IRC_677_A_3_SUBSECTION_NUMBER, 3);
        assert_eq!(IRC_672_E_SPOUSE_NON_ADVERSE_YEAR_ENACTED, 1986);
    }
}
