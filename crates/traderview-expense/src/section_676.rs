//! IRC § 676 — Power to Revoke / Grantor Trust Module.
//!
//! Pure-compute check for grantor trust status under Internal
//! Revenue Code § 676 ("Power to revoke"). § 676 is the fourth
//! grantor-trust trigger in the §§ 671-679 statutory progression
//! after § 673 reversionary interest, § 674 power to control
//! beneficial enjoyment, § 675 administrative powers (built iter
//! 644). Precedes § 677 income for benefit of grantor (iter 642),
//! § 678 person other than grantor (iter 640), and § 679 foreign
//! trusts with US beneficiaries. § 676 is the **broadest and most
//! commonly triggered grantor-trust rule** because the standard
//! revocable living trust used in routine estate planning falls
//! squarely within § 676(a); the entire revocable-trust industry
//! is built on § 676(a) grantor trust status.
//!
//! Web research (verified 2026-06-03):
//! - **IRC § 676(a) General Rule**: grantor treated as owner of
//!   any portion of a trust where, at any time, the **power to
//!   REVEST in the grantor title to such portion** is
//!   exercisable by the grantor OR a NONADVERSE party OR both
//!   ([Cornell LII 26 USC § 676](https://www.law.cornell.edu/uscode/text/26/676);
//!   [26 CFR § 1.676(a)-1](https://www.ecfr.gov/current/title-26/chapter-I/subchapter-A/part-1/subject-group-ECFR245d884a8952b47/section-1.676(a)-1)).
//! - **Power form irrelevant** (26 CFR § 1.676(a)-1): § 676(a)
//!   triggers regardless of whether the power is denominated a
//!   power to **REVOKE, TERMINATE, ALTER, AMEND, or APPOINT**;
//!   any power whose exercise will revest title in the grantor
//!   suffices.
//! - **Adverse Party Exception**: if § 676(a) power is
//!   exercisable only with the **CONSENT OF AN ADVERSE PARTY**
//!   (someone with a substantial beneficial interest adverse to
//!   exercise of the power per § 672(a)), § 676 does NOT trigger.
//!   Spouse cannot be adverse under § 672(e) — built in § 677
//!   module iter 642.
//! - **IRC § 676(b) Postponement Exception**: § 676(a) does NOT
//!   apply to a power the exercise of which can only affect
//!   beneficial enjoyment of income for a period **commencing
//!   AFTER the occurrence of an event** such that the grantor
//!   would NOT be treated as owner under § 673 (the 5 %
//!   reversionary interest rule) if the power were a reversionary
//!   interest ([26 CFR § 1.676(b)-1](https://www.ecfr.gov/current/title-26/chapter-I/subchapter-A/part-1/subject-group-ECFR245d884a8952b47/section-1.676(b)-1)).
//! - **Post-Event Owner Rule**: when the § 676(b) event occurs,
//!   the grantor MAY be treated as owner after the event UNLESS
//!   the power is relinquished. This is the "ticking clock" that
//!   converts a post-event § 676 trust back into grantor trust
//!   status absent affirmative relinquishment.
//! - **Postponement Period Rule (Treas. Reg. § 1.676(b)-1 +
//!   § 1.673(d)-1)**: if the beginning of the period during
//!   which the grantor may revest is **postponed**, the rules of
//!   § 1.673(d)-1 apply to determine whether grantor should be
//!   treated as owner during the period following postponement —
//!   treated as new transfer of present value at postponement
//!   for § 673 5 % threshold analysis.
//! - **Treas. Reg. § 1.671-1 + § 1.676(a)-1 General Rule**: §
//!   676 applies to powers exercisable at "any time" — power
//!   currently in effect OR power exercisable at any future
//!   time during grantor's life.
//! - **§ 672(c) attribution rules do NOT attribute adversity** —
//!   spouse, parent, descendant relationship cannot make a
//!   nonadverse party adverse for § 676 purposes.
//! - **Practical consequence**: standard revocable living trust
//!   (joint or single grantor) used for routine estate planning
//!   always falls within § 676(a); all trust income, deductions,
//!   credits flow through to grantor's Form 1040 via § 671;
//!   trust files Form 1041 with grantor-trust statement attached.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const IRC_676_SUBSECTION_A: u32 = 1;
pub const IRC_676_SUBSECTION_B: u32 = 2;
pub const IRC_673_REVERSIONARY_THRESHOLD_BASIS_POINTS: u64 = 500;
pub const IRC_673_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerForm {
    PowerToRevoke,
    PowerToTerminate,
    PowerToAlter,
    PowerToAmend,
    PowerToAppoint,
    NoPowerHeld,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PowerHolderCategory {
    GrantorAloneOrWithNonadverseParty,
    NonadversePartyAlone,
    GrantorWithAdversePartyConsentRequired,
    AdversePartyAlone,
    NoPowerHolder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PostponementStatus {
    NoPostponementPowerCurrentlyExercisable,
    PostponedPowerExercisableAfterEventLessThanFiveYearsFromInception,
    PostponedPowerExercisableAfterEventFiveYearsOrMoreFromInception,
    PostEventOccurredPowerNotRelinquished,
    PostEventOccurredPowerRelinquished,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Section676Mode {
    NotApplicableNoTrustOrNoPower,
    CompliantSection676ARevocableTrustClassicGrantorTrustFlowThrough,
    CompliantSection676ANonadversePartyPowerActiveGrantorTrust,
    CompliantSection676AAdversePartyConsentDeactivatesSection676,
    CompliantSection676BPostponementExceptionParallelTo673FiveYearTest,
    CompliantSection676BEventOccurredPowerRelinquishedDeactivates676,
    CompliantSection676BPostEventReversionToGrantorTrustNotRelinquished,
    CompliantSection676PowerFormCoversAllRevestingPowers,
    ViolationSection676ActiveButGrantorTrustIncomeNotReportedOnForm1040,
    ViolationSection676AdversePartyConsentClaimedButSpouseInappropriatelyTreatedAsAdverse,
    ViolationSection676BPostponementWithinFiveYearsClaimedButPresentValueExceedsThreshold,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub trust_exists: bool,
    pub power_form: PowerForm,
    pub power_holder_category: PowerHolderCategory,
    pub postponement_status: PostponementStatus,
    pub present_value_of_postponed_power_basis_points: u64,
    pub spouse_inappropriately_treated_as_adverse: bool,
    pub grantor_trust_income_reported_on_form_1040: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: Section676Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type Section676Input = Input;
pub type Section676Output = Output;
pub type Section676Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "IRC § 676(a) — grantor treated as owner of any portion of trust where, at any time, power to REVEST title in grantor is exercisable by grantor OR nonadverse party OR both".to_string(),
        "26 CFR § 1.676(a)-1 — power form irrelevant: power to revoke, terminate, alter, amend, or appoint all trigger § 676(a) if exercise revests title in grantor".to_string(),
        "Adverse Party Exception — if § 676(a) power exercisable only with consent of adverse party (substantial adverse beneficial interest per § 672(a)), § 676 does NOT trigger".to_string(),
        "IRC § 676(b) — § 676(a) does NOT apply to power whose exercise can only affect beneficial enjoyment for period commencing after occurrence of event such that grantor would not be treated as owner under § 673 if power were reversionary interest".to_string(),
        "26 CFR § 1.676(b)-1 — postponement exception parallels § 673 5 % reversionary threshold; if present value of postponed power < 5 % of trust value at inception, § 676 deactivated".to_string(),
        "Post-Event Owner Rule — when § 676(b) event occurs, grantor MAY be treated as owner after event UNLESS power is relinquished (the 'ticking clock' converts post-event trust back to grantor trust absent affirmative relinquishment)".to_string(),
        "Treas. Reg. § 1.673(d)-1 — postponement period rule; if beginning of period during which grantor may revest is postponed, treat as new transfer for § 673 threshold analysis".to_string(),
        "IRC § 672(a) ADVERSE PARTY — any person having substantial beneficial interest in trust adverse to exercise of power".to_string(),
        "IRC § 672(b) NONADVERSE PARTY — any person who is not adverse party".to_string(),
        "IRC § 672(e) SPOUSE NEVER ADVERSE (enacted 1986) — grantor treated as holding any power or interest held by spouse at time of creation; spouse cannot be adverse for § 671-679 grantor trust purposes".to_string(),
        "IRC § 671 — Subpart E general attribution; § 676-triggered grantor trust status flows income, deductions, credits to grantor's Form 1040; trust files Form 1041 with grantor-trust statement".to_string(),
        "Practical scope — standard revocable living trust (joint or single grantor) for routine estate planning always falls within § 676(a); the entire revocable-trust industry built on § 676(a) grantor trust status".to_string(),
        "Cornell LII 26 USC § 676 — primary statutory text".to_string(),
        "Knox Law Firm — Grantor Trusts Explained: Trusts You Can't Trust — § 676 analysis".to_string(),
        "Griffin Bridgers — Grantor Trusts and Powers to Revoke (Substack) — practitioner commentary".to_string(),
    ];

    if !input.trust_exists {
        return Output {
            mode: Section676Mode::NotApplicableNoTrustOrNoPower,
            statutory_basis: "IRC § 676 inapplicable — no trust exists".to_string(),
            notes: "No trust exists; IRC § 676 inapplicable.".to_string(),
            citations,
        };
    }

    if input.power_form == PowerForm::NoPowerHeld
        || input.power_holder_category == PowerHolderCategory::NoPowerHolder
    {
        return Output {
            mode: Section676Mode::NotApplicableNoTrustOrNoPower,
            statutory_basis: "IRC § 676 inapplicable — no power to revest title in grantor exists".to_string(),
            notes: "No power to revoke, terminate, alter, amend, or appoint title back to grantor exists; § 676 does not trigger.".to_string(),
            citations,
        };
    }

    if input.spouse_inappropriately_treated_as_adverse {
        return Output {
            mode: Section676Mode::ViolationSection676AdversePartyConsentClaimedButSpouseInappropriatelyTreatedAsAdverse,
            statutory_basis: "IRC § 672(e) — spouse cannot be adverse party; § 676 triggers regardless".to_string(),
            notes: "VIOLATION: adverse-party-consent exception claimed but the 'adverse party' is the grantor's spouse; under IRC § 672(e) (enacted 1986), spouse is NEVER adverse for § 671-679 grantor trust purposes — § 676 triggers regardless.".to_string(),
            citations,
        };
    }

    if input.power_holder_category
        == PowerHolderCategory::GrantorWithAdversePartyConsentRequired
        || input.power_holder_category == PowerHolderCategory::AdversePartyAlone
    {
        return Output {
            mode: Section676Mode::CompliantSection676AAdversePartyConsentDeactivatesSection676,
            statutory_basis: "IRC § 676(a) adverse party exception — power requires consent of adverse party".to_string(),
            notes: "COMPLIANT: § 676(a) power exercise conditioned on adverse-party consent (or held only by adverse party); § 676 does NOT trigger grantor trust status under standard § 672(a) adverse-party reading.".to_string(),
            citations,
        };
    }

    match input.postponement_status {
        PostponementStatus::PostponedPowerExercisableAfterEventLessThanFiveYearsFromInception => {
            if input.present_value_of_postponed_power_basis_points
                > IRC_673_REVERSIONARY_THRESHOLD_BASIS_POINTS
            {
                return Output {
                    mode: Section676Mode::ViolationSection676BPostponementWithinFiveYearsClaimedButPresentValueExceedsThreshold,
                    statutory_basis: "IRC § 676(b) postponement exception requires present-value parallel to § 673 5 % threshold".to_string(),
                    notes: format!(
                        "VIOLATION: § 676(b) postponement-exception claimed but present value of postponed power is {} basis points (> 500 bp = 5 % threshold from § 673 parallel); § 676(b) exception unavailable.",
                        input.present_value_of_postponed_power_basis_points
                    ),
                    citations,
                };
            }
            return Output {
                mode: Section676Mode::CompliantSection676BPostponementExceptionParallelTo673FiveYearTest,
                statutory_basis: "IRC § 676(b) — postponement exception with present value within § 673 5 % threshold".to_string(),
                notes: format!(
                    "COMPLIANT: § 676(b) postponement exception applies; present value of postponed power is {} basis points (≤ 500 bp = 5 % threshold parallel to § 673); § 676 not triggered.",
                    input.present_value_of_postponed_power_basis_points
                ),
                citations,
            };
        }
        PostponementStatus::PostponedPowerExercisableAfterEventFiveYearsOrMoreFromInception => {
            return Output {
                mode: Section676Mode::CompliantSection676BPostponementExceptionParallelTo673FiveYearTest,
                statutory_basis: "IRC § 676(b) postponement exception — period 5+ years from inception per § 673 parallel".to_string(),
                notes: "COMPLIANT: § 676(b) postponement exception applies; period of postponement parallels § 673 5-year test; § 676 not triggered during postponement period.".to_string(),
                citations,
            };
        }
        PostponementStatus::PostEventOccurredPowerRelinquished => {
            return Output {
                mode: Section676Mode::CompliantSection676BEventOccurredPowerRelinquishedDeactivates676,
                statutory_basis: "IRC § 676(b) post-event rule — power relinquished after event occurred".to_string(),
                notes: "COMPLIANT: § 676(b) event has occurred and power was affirmatively relinquished; § 676 deactivated post-event.".to_string(),
                citations,
            };
        }
        PostponementStatus::PostEventOccurredPowerNotRelinquished => {
            if !input.grantor_trust_income_reported_on_form_1040 {
                return Output {
                    mode: Section676Mode::ViolationSection676ActiveButGrantorTrustIncomeNotReportedOnForm1040,
                    statutory_basis: "IRC § 676(b) post-event rule — grantor MAY be treated as owner post-event unless power relinquished".to_string(),
                    notes: "VIOLATION: § 676(b) event occurred but power was not relinquished; grantor treated as owner post-event; income must be reported on Form 1040 but was omitted.".to_string(),
                    citations,
                };
            }
            return Output {
                mode: Section676Mode::CompliantSection676BPostEventReversionToGrantorTrustNotRelinquished,
                statutory_basis: "IRC § 676(b) post-event rule — power not relinquished triggers grantor trust status".to_string(),
                notes: "COMPLIANT: § 676(b) event occurred and power not relinquished; grantor treated as owner post-event; income properly reported on Form 1040.".to_string(),
                citations,
            };
        }
        PostponementStatus::NoPostponementPowerCurrentlyExercisable => {}
    }

    if !input.grantor_trust_income_reported_on_form_1040 {
        return Output {
            mode: Section676Mode::ViolationSection676ActiveButGrantorTrustIncomeNotReportedOnForm1040,
            statutory_basis: "IRC § 676(a) — active grantor trust requires Form 1040 reporting via § 671 flow-through".to_string(),
            notes: format!(
                "VIOLATION: § 676(a) active grantor trust status triggered ({:?} held by {:?}); § 671 income/deductions/credits flow-through omitted from grantor's Form 1040.",
                input.power_form, input.power_holder_category
            ),
            citations,
        };
    }

    if input.power_form == PowerForm::PowerToRevoke
        && input.power_holder_category
            == PowerHolderCategory::GrantorAloneOrWithNonadverseParty
    {
        return Output {
            mode: Section676Mode::CompliantSection676ARevocableTrustClassicGrantorTrustFlowThrough,
            statutory_basis: "IRC § 676(a) — classic revocable living trust; grantor holds power to revoke".to_string(),
            notes: "COMPLIANT: § 676(a) classic revocable living trust — grantor holds power to revoke; grantor trust status active; income properly reported on Form 1040 via § 671 flow-through.".to_string(),
            citations,
        };
    }

    if input.power_holder_category == PowerHolderCategory::NonadversePartyAlone {
        return Output {
            mode: Section676Mode::CompliantSection676ANonadversePartyPowerActiveGrantorTrust,
            statutory_basis: "IRC § 676(a) — nonadverse party holds power to revest title in grantor".to_string(),
            notes: format!(
                "COMPLIANT: § 676(a) nonadverse party holds {:?} which would revest title in grantor; § 676 triggers grantor trust status; income properly reported on Form 1040.",
                input.power_form
            ),
            citations,
        };
    }

    Output {
        mode: Section676Mode::CompliantSection676PowerFormCoversAllRevestingPowers,
        statutory_basis: format!("IRC § 676(a) — {:?} held by {:?} revests title in grantor", input.power_form, input.power_holder_category),
        notes: format!(
            "COMPLIANT: § 676(a) {:?} (any of revoke, terminate, alter, amend, or appoint) held by {:?} revests title in grantor; grantor trust status active; income properly reported on Form 1040.",
            input.power_form, input.power_holder_category
        ),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_classic_revocable_trust() -> Input {
        Input {
            trust_exists: true,
            power_form: PowerForm::PowerToRevoke,
            power_holder_category: PowerHolderCategory::GrantorAloneOrWithNonadverseParty,
            postponement_status: PostponementStatus::NoPostponementPowerCurrentlyExercisable,
            present_value_of_postponed_power_basis_points: 0,
            spouse_inappropriately_treated_as_adverse: false,
            grantor_trust_income_reported_on_form_1040: true,
        }
    }

    #[test]
    fn no_trust_not_applicable() {
        let input = Input {
            trust_exists: false,
            ..baseline_classic_revocable_trust()
        };
        let result = check(&input);
        assert_eq!(result.mode, Section676Mode::NotApplicableNoTrustOrNoPower);
    }

    #[test]
    fn no_power_held_not_applicable() {
        let input = Input {
            power_form: PowerForm::NoPowerHeld,
            power_holder_category: PowerHolderCategory::NoPowerHolder,
            ..baseline_classic_revocable_trust()
        };
        let result = check(&input);
        assert_eq!(result.mode, Section676Mode::NotApplicableNoTrustOrNoPower);
    }

    #[test]
    fn classic_revocable_trust_compliant() {
        let result = check(&baseline_classic_revocable_trust());
        assert_eq!(
            result.mode,
            Section676Mode::CompliantSection676ARevocableTrustClassicGrantorTrustFlowThrough
        );
    }

    #[test]
    fn nonadverse_party_alone_holds_power_compliant() {
        let input = Input {
            power_holder_category: PowerHolderCategory::NonadversePartyAlone,
            power_form: PowerForm::PowerToTerminate,
            ..baseline_classic_revocable_trust()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section676Mode::CompliantSection676ANonadversePartyPowerActiveGrantorTrust
        );
    }

    #[test]
    fn adverse_party_consent_required_deactivates() {
        let input = Input {
            power_holder_category: PowerHolderCategory::GrantorWithAdversePartyConsentRequired,
            ..baseline_classic_revocable_trust()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section676Mode::CompliantSection676AAdversePartyConsentDeactivatesSection676
        );
    }

    #[test]
    fn adverse_party_alone_holds_power_deactivates() {
        let input = Input {
            power_holder_category: PowerHolderCategory::AdversePartyAlone,
            ..baseline_classic_revocable_trust()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section676Mode::CompliantSection676AAdversePartyConsentDeactivatesSection676
        );
    }

    #[test]
    fn spouse_inappropriately_treated_as_adverse_violation() {
        let input = Input {
            power_holder_category: PowerHolderCategory::GrantorWithAdversePartyConsentRequired,
            spouse_inappropriately_treated_as_adverse: true,
            ..baseline_classic_revocable_trust()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section676Mode::ViolationSection676AdversePartyConsentClaimedButSpouseInappropriatelyTreatedAsAdverse
        );
    }

    #[test]
    fn power_to_alter_amend_appoint_terminate_all_covered() {
        for power_form in [
            PowerForm::PowerToTerminate,
            PowerForm::PowerToAlter,
            PowerForm::PowerToAmend,
            PowerForm::PowerToAppoint,
        ] {
            let input = Input {
                power_form,
                power_holder_category:
                    PowerHolderCategory::GrantorAloneOrWithNonadverseParty,
                ..baseline_classic_revocable_trust()
            };
            let result = check(&input);
            assert!(
                matches!(
                    result.mode,
                    Section676Mode::CompliantSection676PowerFormCoversAllRevestingPowers
                ),
                "power form {:?} failed",
                power_form
            );
        }
    }

    #[test]
    fn section_676_b_postponement_within_5_years_value_below_5_pct_compliant() {
        let input = Input {
            postponement_status:
                PostponementStatus::PostponedPowerExercisableAfterEventLessThanFiveYearsFromInception,
            present_value_of_postponed_power_basis_points: 499,
            ..baseline_classic_revocable_trust()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section676Mode::CompliantSection676BPostponementExceptionParallelTo673FiveYearTest
        );
    }

    #[test]
    fn section_676_b_postponement_at_exactly_5_pct_threshold_compliant() {
        let input = Input {
            postponement_status:
                PostponementStatus::PostponedPowerExercisableAfterEventLessThanFiveYearsFromInception,
            present_value_of_postponed_power_basis_points: 500,
            ..baseline_classic_revocable_trust()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section676Mode::CompliantSection676BPostponementExceptionParallelTo673FiveYearTest
        );
    }

    #[test]
    fn section_676_b_postponement_exceeds_5_pct_threshold_violation() {
        let input = Input {
            postponement_status:
                PostponementStatus::PostponedPowerExercisableAfterEventLessThanFiveYearsFromInception,
            present_value_of_postponed_power_basis_points: 501,
            ..baseline_classic_revocable_trust()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section676Mode::ViolationSection676BPostponementWithinFiveYearsClaimedButPresentValueExceedsThreshold
        );
    }

    #[test]
    fn section_676_b_postponement_5_years_or_more_compliant() {
        let input = Input {
            postponement_status:
                PostponementStatus::PostponedPowerExercisableAfterEventFiveYearsOrMoreFromInception,
            present_value_of_postponed_power_basis_points: 5_000,
            ..baseline_classic_revocable_trust()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section676Mode::CompliantSection676BPostponementExceptionParallelTo673FiveYearTest
        );
    }

    #[test]
    fn post_event_power_relinquished_deactivates() {
        let input = Input {
            postponement_status: PostponementStatus::PostEventOccurredPowerRelinquished,
            ..baseline_classic_revocable_trust()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section676Mode::CompliantSection676BEventOccurredPowerRelinquishedDeactivates676
        );
    }

    #[test]
    fn post_event_power_not_relinquished_compliant() {
        let input = Input {
            postponement_status: PostponementStatus::PostEventOccurredPowerNotRelinquished,
            ..baseline_classic_revocable_trust()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section676Mode::CompliantSection676BPostEventReversionToGrantorTrustNotRelinquished
        );
    }

    #[test]
    fn post_event_power_not_relinquished_form_1040_omitted_violation() {
        let input = Input {
            postponement_status: PostponementStatus::PostEventOccurredPowerNotRelinquished,
            grantor_trust_income_reported_on_form_1040: false,
            ..baseline_classic_revocable_trust()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section676Mode::ViolationSection676ActiveButGrantorTrustIncomeNotReportedOnForm1040
        );
    }

    #[test]
    fn section_676_active_form_1040_omitted_violation() {
        let input = Input {
            grantor_trust_income_reported_on_form_1040: false,
            ..baseline_classic_revocable_trust()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            Section676Mode::ViolationSection676ActiveButGrantorTrustIncomeNotReportedOnForm1040
        );
    }

    #[test]
    fn citations_pin_section_676_subsections_and_companion_statutes() {
        let result = check(&baseline_classic_revocable_trust());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("IRC § 676(a)"));
        assert!(joined.contains("IRC § 676(b)"));
        assert!(joined.contains("26 CFR § 1.676(a)-1"));
        assert!(joined.contains("26 CFR § 1.676(b)-1"));
        assert!(joined.contains("Treas. Reg. § 1.673(d)-1"));
        assert!(joined.contains("REVEST"));
        assert!(joined.contains("revoke, terminate, alter, amend, or appoint"));
        assert!(joined.contains("IRC § 672(a)"));
        assert!(joined.contains("IRC § 672(b)"));
        assert!(joined.contains("IRC § 672(e)"));
        assert!(joined.contains("IRC § 671"));
        assert!(joined.contains("§ 673"));
        assert!(joined.contains("Form 1041"));
        assert!(joined.contains("Form 1040"));
        assert!(joined.contains("Cornell LII"));
        assert!(joined.contains("Knox Law Firm"));
        assert!(joined.contains("Griffin Bridgers"));
    }

    #[test]
    fn constant_pin_subsection_numbers_and_thresholds() {
        assert_eq!(IRC_676_SUBSECTION_A, 1);
        assert_eq!(IRC_676_SUBSECTION_B, 2);
        assert_eq!(IRC_673_REVERSIONARY_THRESHOLD_BASIS_POINTS, 500);
        assert_eq!(IRC_673_BASIS_POINT_DENOMINATOR, 10_000);
    }
}
