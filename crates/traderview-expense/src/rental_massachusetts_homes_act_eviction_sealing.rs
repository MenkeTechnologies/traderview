//! Massachusetts Affordable Homes Act of 2024 (H.4138 /
//! Chapter 150 of the Acts of 2024) — HOMES Act Eviction
//! Record Sealing Compliance Module.
//!
//! Pure-compute check for landlord / tenant / housing-provider
//! compliance with Massachusetts's HOMES Act eviction record
//! sealing regime under Section 52 of Chapter 150 of the Acts
//! of 2024. Signed by Governor Maura Healey on August 6, 2024
//! as part of the $5.16 billion Affordable Homes Act; eviction
//! sealing provisions effective **May 5, 2025** (270 days
//! after signing). Tenants may petition the court to seal
//! eviction records depending on case outcome, with eligibility
//! ranging from immediate (dismissed / tenant-won / satisfied
//! judgment) to 4 years post-conclusion (non-payment cases not
//! paid + economic-hardship conditions).
//!
//! Web research (verified 2026-06-03):
//! - **Massachusetts Affordable Homes Act of 2024 (H.4138)** —
//!   signed by Governor **Maura Healey on August 6, 2024**;
//!   codified at **Chapter 150 of the Acts of 2024**;
//!   authorizes **$5.16 billion** in spending over 5 years +
//!   ~50 policy initiatives; eviction sealing provisions at
//!   **Section 52** ([Massachusetts Legislature H.4138](https://malegislature.gov/Bills/193/H4138);
//!   [Mass.gov — Affordable Homes Act](https://www.mass.gov/info-details/the-affordable-homes-act-smart-housing-livable-communities);
//!   [Nixon Peabody — Massachusetts Governor Signs Affordable
//!   Homes Act](https://www.nixonpeabody.com/insights/alerts/2024/08/12/massachusetts-governor-signs-affordable-homes-act)).
//! - **Eviction Sealing Effective Date**: **May 5, 2025** (270
//!   days after August 6, 2024 signing); codified permanently
//!   at **Mass. Gen. Laws Chapter 239** ([Mass.gov — Sealing
//!   Eviction Records Coming in May 2025](https://www.mass.gov/news/sealing-eviction-records-coming-in-may-2025);
//!   [MassLandlords — Eviction Sealing Effective May 2025](https://masslandlords.net/laws/eviction-sealing/);
//!   [MLRI Press Release — Massachusetts Eviction Record Sealing
//!   Law Takes Effect May 2025](https://mlri.org/2025/05/01/massachusetts-eviction-record-sealing-law-2025/)).
//! - **Eligibility Categories**:
//!   - **(a) Cases Dismissed**: tenant may petition to seal
//!     **IMMEDIATELY** (after any appeal period expires);
//!   - **(b) Cases Tenant Won**: tenant may petition to seal
//!     **IMMEDIATELY** (after any appeal period expires);
//!   - **(c) Satisfied Judgments**: where tenant has paid
//!     judgment or otherwise satisfied lessor's claim, tenant
//!     may petition to seal **IMMEDIATELY**;
//!   - **(d) Non-Payment Cases Not Paid**: tenant must wait
//!     **4 YEARS** after case conclusion AND satisfy certain
//!     conditions (economic hardship, no intervening lessor
//!     action) before petitioning.
//! - **Once Sealed**: eviction case is **no longer visible to
//!   the public or to tenant-screening and credit-reporting
//!   companies** — tenant-screening services (CoreLogic
//!   SafeRent, TransUnion SmartMove, Experian RentBureau) must
//!   not report sealed cases.
//! - **Landlord Inquiry Prohibition**: landlord may not ask
//!   prospective tenant about a sealed eviction record;
//!   tenant has no obligation to disclose; deceptive use of
//!   sealed records by landlord triggers statutory damages.
//! - **Other Affordable Homes Act Provisions** (not in this
//!   module but referenced): (1) ADUs (Accessory Dwelling
//!   Units) legalized as-of-right statewide; municipalities
//!   required to permit ADUs on same parcel as primary
//!   dwelling; (2) Public housing modernization; (3) First-
//!   time homebuyer programs; (4) Low/moderate-income housing
//!   investment.
//! - **Enforcement**: Massachusetts trial court system
//!   (Housing Court Department + District Court Department)
//!   processes sealing petitions; sealed records may still be
//!   accessed by parties for limited statutory purposes (court
//!   officials, certain law enforcement); per-violation
//!   damages for unauthorized disclosure.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const MA_AFFORDABLE_HOMES_ACT_SIGNED_DATE_YEAR: u32 = 2024;
pub const MA_AFFORDABLE_HOMES_ACT_SIGNED_DATE_MONTH: u32 = 8;
pub const MA_AFFORDABLE_HOMES_ACT_SIGNED_DATE_DAY: u32 = 6;
pub const MA_AFFORDABLE_HOMES_ACT_AUTHORIZATION_DOLLARS_BILLIONS: u64 = 5_160_000_000;
pub const MA_AFFORDABLE_HOMES_ACT_POLICY_INITIATIVES_COUNT: u32 = 50;
pub const MA_HOMES_ACT_EVICTION_SEALING_EFFECTIVE_DATE_YEAR: u32 = 2025;
pub const MA_HOMES_ACT_EVICTION_SEALING_EFFECTIVE_DATE_MONTH: u32 = 5;
pub const MA_HOMES_ACT_EVICTION_SEALING_EFFECTIVE_DATE_DAY: u32 = 5;
pub const MA_HOMES_ACT_EFFECTIVE_DATE_LAG_DAYS_AFTER_SIGNING: u32 = 270;
pub const MA_HOMES_ACT_CHAPTER_150_SECTION_52: u32 = 52;
pub const MA_HOMES_ACT_NON_PAYMENT_WAITING_PERIOD_YEARS: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyJurisdiction {
    MassachusettsSubjectToHomesAct,
    NotInMassachusetts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvictionCaseOutcome {
    CaseDismissed,
    TenantWonOnMerits,
    SatisfiedJudgmentTenantPaid,
    NonPaymentJudgmentNotPaid,
    PendingAppealPeriod,
    NoEvictionCaseFiled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum NonPaymentSealingPreconditions {
    EconomicHardshipDocumentedAndNoInterveningLessorAction,
    NoEconomicHardshipDocumented,
    NotApplicableNotNonPaymentCase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlordAction {
    InquiringAboutSealedEvictionRecord,
    DenyingHousingBasedOnSealedRecord,
    DisclosingSealedRecordToThirdParty,
    NoAdverseInquiryOrAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MassachusettsHomesActMode {
    NotApplicableNotInMassachusetts,
    NotApplicablePreMay5_2025EffectiveDate,
    NotApplicableNoEvictionCase,
    CompliantImmediateSealingEligibleForDismissedCase,
    CompliantImmediateSealingEligibleForTenantWonCase,
    CompliantImmediateSealingEligibleForSatisfiedJudgment,
    Compliant4YearPostConclusionSealingForNonPaymentWithConditions,
    CompliantLandlordRespectedSealedRecordNoInquiryOrAdverseAction,
    ViolationLandlordInquiredAboutSealedRecord,
    ViolationLandlordDeniedHousingBasedOnSealedRecord,
    ViolationLandlordDisclosedSealedRecordToThirdParty,
    ViolationPrematureNonPaymentSealingBefore4Years,
    ViolationNonPaymentSealingClaimedWithoutEconomicHardshipDocumented,
    ViolationAppealPeriodNotYetExpired,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_jurisdiction: PropertyJurisdiction,
    pub current_date_year: u32,
    pub current_date_month: u32,
    pub current_date_day: u32,
    pub eviction_case_outcome: EvictionCaseOutcome,
    pub non_payment_sealing_preconditions: NonPaymentSealingPreconditions,
    pub landlord_action: LandlordAction,
    pub years_since_case_conclusion: u32,
    pub appeal_period_has_expired: bool,
    pub tenant_petitioned_for_sealing: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: MassachusettsHomesActMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalMassachusettsHomesActEvictionSealingInput = Input;
pub type RentalMassachusettsHomesActEvictionSealingOutput = Output;
pub type RentalMassachusettsHomesActEvictionSealingResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Massachusetts Affordable Homes Act of 2024 (H.4138) — signed by Governor Maura Healey on August 6, 2024; codified at Chapter 150 of the Acts of 2024; authorizes $5.16 billion in spending over 5 years + ~50 policy initiatives; eviction sealing provisions at Section 52".to_string(),
        "Eviction Sealing Effective Date — May 5, 2025 (270 days after August 6, 2024 signing); codified permanently at Mass. Gen. Laws Chapter 239".to_string(),
        "Eligibility Category (a) — Cases Dismissed: tenant may petition to seal IMMEDIATELY (after any appeal period expires)".to_string(),
        "Eligibility Category (b) — Cases Tenant Won: tenant may petition to seal IMMEDIATELY (after any appeal period expires)".to_string(),
        "Eligibility Category (c) — Satisfied Judgments: where tenant has paid judgment or otherwise satisfied lessor's claim, tenant may petition to seal IMMEDIATELY".to_string(),
        "Eligibility Category (d) — Non-Payment Cases Not Paid: tenant must wait 4 YEARS after case conclusion AND satisfy certain conditions (economic hardship documented, no intervening lessor action) before petitioning".to_string(),
        "Once Sealed — eviction case is no longer visible to public or to tenant-screening and credit-reporting companies (CoreLogic SafeRent, TransUnion SmartMove, Experian RentBureau)".to_string(),
        "Landlord Inquiry Prohibition — landlord may not ask prospective tenant about a sealed eviction record; tenant has no obligation to disclose; deceptive use of sealed records by landlord triggers statutory damages".to_string(),
        "Other Affordable Homes Act Provisions (referenced) — (1) ADUs (Accessory Dwelling Units) legalized as-of-right statewide; municipalities required to permit ADUs on same parcel as primary dwelling; (2) Public housing modernization; (3) First-time homebuyer programs; (4) Low/moderate-income housing investment".to_string(),
        "Enforcement — Massachusetts trial court system (Housing Court Department + District Court Department) processes sealing petitions; sealed records may still be accessed by parties for limited statutory purposes (court officials, certain law enforcement); per-violation damages for unauthorized disclosure".to_string(),
        "Massachusetts Legislature H.4138 — primary bill text and history".to_string(),
        "Mass.gov — Affordable Homes Act program description".to_string(),
        "Mass.gov — Sealing Eviction Records Coming in May 2025".to_string(),
        "MassLandlords — Eviction Sealing Effective May 2025: What Landlords Need to Know".to_string(),
        "MLRI Press Release — Massachusetts Eviction Record Sealing Law Takes Effect May 2025".to_string(),
        "NLIHC — Massachusetts Eviction Sealing Law Strengthens Housing Access for Renters".to_string(),
        "Nixon Peabody — Massachusetts Governor Signs Affordable Homes Act".to_string(),
    ];

    if input.property_jurisdiction == PropertyJurisdiction::NotInMassachusetts {
        return Output {
            mode: MassachusettsHomesActMode::NotApplicableNotInMassachusetts,
            statutory_basis: "Property outside Massachusetts; HOMES Act eviction sealing inapplicable".to_string(),
            notes: "NOT APPLICABLE: property outside Massachusetts; Massachusetts Affordable Homes Act HOMES Act eviction sealing regime inapplicable.".to_string(),
            citations,
        };
    }

    let current_date_post_may_5_2025 = input.current_date_year > 2025
        || (input.current_date_year == 2025
            && (input.current_date_month > 5
                || (input.current_date_month == 5 && input.current_date_day >= 5)));

    if !current_date_post_may_5_2025 {
        return Output {
            mode: MassachusettsHomesActMode::NotApplicablePreMay5_2025EffectiveDate,
            statutory_basis: "Chapter 150 § 52 of the Acts of 2024 — eviction sealing effective May 5, 2025".to_string(),
            notes: format!(
                "NOT APPLICABLE: current date {}/{}/{} precedes the May 5, 2025 effective date for HOMES Act eviction sealing under Chapter 150 § 52 of the Acts of 2024.",
                input.current_date_year, input.current_date_month, input.current_date_day
            ),
            citations,
        };
    }

    if input.eviction_case_outcome == EvictionCaseOutcome::NoEvictionCaseFiled {
        return Output {
            mode: MassachusettsHomesActMode::NotApplicableNoEvictionCase,
            statutory_basis: "HOMES Act eviction sealing — no eviction case to seal".to_string(),
            notes: "NOT APPLICABLE: no eviction case has been filed against tenant; HOMES Act eviction sealing not triggered.".to_string(),
            citations,
        };
    }

    if input.eviction_case_outcome == EvictionCaseOutcome::PendingAppealPeriod
        && !input.appeal_period_has_expired
    {
        return Output {
            mode: MassachusettsHomesActMode::ViolationAppealPeriodNotYetExpired,
            statutory_basis: "Chapter 150 § 52 — sealing petitions require appeal period expiration".to_string(),
            notes: "VIOLATION: sealing petition filed before appeal period has expired; eligibility for sealing requires any appeal period to have expired first.".to_string(),
            citations,
        };
    }

    match input.landlord_action {
        LandlordAction::InquiringAboutSealedEvictionRecord => {
            return Output {
                mode: MassachusettsHomesActMode::ViolationLandlordInquiredAboutSealedRecord,
                statutory_basis: "Chapter 150 § 52 — landlord inquiry prohibition on sealed records".to_string(),
                notes: "VIOLATION: landlord inquired about a sealed eviction record; HOMES Act prohibits such inquiries; tenant has no obligation to disclose sealed records.".to_string(),
                citations,
            };
        }
        LandlordAction::DenyingHousingBasedOnSealedRecord => {
            return Output {
                mode: MassachusettsHomesActMode::ViolationLandlordDeniedHousingBasedOnSealedRecord,
                statutory_basis: "Chapter 150 § 52 — denial of housing based on sealed record prohibited".to_string(),
                notes: "VIOLATION: landlord denied housing application based on sealed eviction record; HOMES Act prohibits such denial; statutory damages may apply.".to_string(),
                citations,
            };
        }
        LandlordAction::DisclosingSealedRecordToThirdParty => {
            return Output {
                mode: MassachusettsHomesActMode::ViolationLandlordDisclosedSealedRecordToThirdParty,
                statutory_basis: "Chapter 150 § 52 — disclosure of sealed record to third party prohibited".to_string(),
                notes: "VIOLATION: landlord disclosed sealed eviction record to third party (tenant-screening company, credit bureau, prospective landlord); HOMES Act prohibits unauthorized disclosure of sealed records; per-violation statutory damages apply.".to_string(),
                citations,
            };
        }
        LandlordAction::NoAdverseInquiryOrAction => {}
    }

    match input.eviction_case_outcome {
        EvictionCaseOutcome::CaseDismissed => {
            if input.tenant_petitioned_for_sealing {
                return Output {
                    mode: MassachusettsHomesActMode::CompliantImmediateSealingEligibleForDismissedCase,
                    statutory_basis: "Chapter 150 § 52(a) — dismissed cases immediately sealable".to_string(),
                    notes: "COMPLIANT: dismissed eviction case is immediately eligible for sealing under § 52(a) after any appeal period expires; tenant petitioned for sealing.".to_string(),
                    citations,
                };
            }
            Output {
                mode: MassachusettsHomesActMode::CompliantLandlordRespectedSealedRecordNoInquiryOrAdverseAction,
                statutory_basis: "Chapter 150 § 52 — dismissed case; no landlord inquiry or adverse action".to_string(),
                notes: "COMPLIANT: dismissed eviction case is sealable under § 52(a); no landlord inquiry or adverse action.".to_string(),
                citations,
            }
        }
        EvictionCaseOutcome::TenantWonOnMerits => {
            if input.tenant_petitioned_for_sealing {
                return Output {
                    mode: MassachusettsHomesActMode::CompliantImmediateSealingEligibleForTenantWonCase,
                    statutory_basis: "Chapter 150 § 52(b) — tenant-won cases immediately sealable".to_string(),
                    notes: "COMPLIANT: case in which tenant prevailed on merits is immediately eligible for sealing under § 52(b) after any appeal period expires; tenant petitioned for sealing.".to_string(),
                    citations,
                };
            }
            Output {
                mode: MassachusettsHomesActMode::CompliantLandlordRespectedSealedRecordNoInquiryOrAdverseAction,
                statutory_basis: "Chapter 150 § 52 — tenant-won case; no landlord inquiry or adverse action".to_string(),
                notes: "COMPLIANT: case in which tenant prevailed on merits is sealable under § 52(b); no landlord inquiry or adverse action.".to_string(),
                citations,
            }
        }
        EvictionCaseOutcome::SatisfiedJudgmentTenantPaid => {
            if input.tenant_petitioned_for_sealing {
                return Output {
                    mode: MassachusettsHomesActMode::CompliantImmediateSealingEligibleForSatisfiedJudgment,
                    statutory_basis: "Chapter 150 § 52(c) — satisfied judgments immediately sealable".to_string(),
                    notes: "COMPLIANT: case with satisfied judgment (tenant paid lessor's claim) is immediately eligible for sealing under § 52(c) after any appeal period expires; tenant petitioned for sealing.".to_string(),
                    citations,
                };
            }
            Output {
                mode: MassachusettsHomesActMode::CompliantLandlordRespectedSealedRecordNoInquiryOrAdverseAction,
                statutory_basis: "Chapter 150 § 52 — satisfied judgment; no landlord inquiry or adverse action".to_string(),
                notes: "COMPLIANT: case with satisfied judgment is sealable under § 52(c); no landlord inquiry or adverse action.".to_string(),
                citations,
            }
        }
        EvictionCaseOutcome::NonPaymentJudgmentNotPaid => {
            if input.years_since_case_conclusion < MA_HOMES_ACT_NON_PAYMENT_WAITING_PERIOD_YEARS {
                return Output {
                    mode: MassachusettsHomesActMode::ViolationPrematureNonPaymentSealingBefore4Years,
                    statutory_basis: "Chapter 150 § 52(d) — non-payment cases require 4-year waiting period".to_string(),
                    notes: format!(
                        "VIOLATION: non-payment case sealing petition filed only {} years after case conclusion; HOMES Act § 52(d) requires 4-year waiting period.",
                        input.years_since_case_conclusion
                    ),
                    citations,
                };
            }
            if input.non_payment_sealing_preconditions
                == NonPaymentSealingPreconditions::NoEconomicHardshipDocumented
            {
                return Output {
                    mode: MassachusettsHomesActMode::ViolationNonPaymentSealingClaimedWithoutEconomicHardshipDocumented,
                    statutory_basis: "Chapter 150 § 52(d) — non-payment sealing requires economic hardship documentation".to_string(),
                    notes: "VIOLATION: non-payment case sealing petition filed without documented economic hardship and absence of intervening lessor action; § 52(d) requires both conditions.".to_string(),
                    citations,
                };
            }
            Output {
                mode: MassachusettsHomesActMode::Compliant4YearPostConclusionSealingForNonPaymentWithConditions,
                statutory_basis: "Chapter 150 § 52(d) — 4-year post-conclusion non-payment sealing satisfied".to_string(),
                notes: format!(
                    "COMPLIANT: non-payment case sealing eligible after {} years post-conclusion (≥ 4-year minimum) with documented economic hardship and no intervening lessor action under § 52(d).",
                    input.years_since_case_conclusion
                ),
                citations,
            }
        }
        EvictionCaseOutcome::PendingAppealPeriod => Output {
            mode: MassachusettsHomesActMode::CompliantLandlordRespectedSealedRecordNoInquiryOrAdverseAction,
            statutory_basis: "Chapter 150 § 52 — appeal period expired".to_string(),
            notes: "COMPLIANT: appeal period has expired; sealing petition eligibility determined by underlying case outcome.".to_string(),
            citations,
        },
        EvictionCaseOutcome::NoEvictionCaseFiled => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_compliant_post_effective_date() -> Input {
        Input {
            property_jurisdiction: PropertyJurisdiction::MassachusettsSubjectToHomesAct,
            current_date_year: 2026,
            current_date_month: 6,
            current_date_day: 3,
            eviction_case_outcome: EvictionCaseOutcome::CaseDismissed,
            non_payment_sealing_preconditions:
                NonPaymentSealingPreconditions::NotApplicableNotNonPaymentCase,
            landlord_action: LandlordAction::NoAdverseInquiryOrAction,
            years_since_case_conclusion: 5,
            appeal_period_has_expired: true,
            tenant_petitioned_for_sealing: true,
        }
    }

    #[test]
    fn property_outside_ma_not_applicable() {
        let input = Input {
            property_jurisdiction: PropertyJurisdiction::NotInMassachusetts,
            ..baseline_compliant_post_effective_date()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            MassachusettsHomesActMode::NotApplicableNotInMassachusetts
        );
    }

    #[test]
    fn pre_may_5_2025_effective_date_not_applicable() {
        let input = Input {
            current_date_year: 2025,
            current_date_month: 5,
            current_date_day: 4,
            ..baseline_compliant_post_effective_date()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            MassachusettsHomesActMode::NotApplicablePreMay5_2025EffectiveDate
        );
    }

    #[test]
    fn at_exactly_may_5_2025_effective_date_applicable() {
        let input = Input {
            current_date_year: 2025,
            current_date_month: 5,
            current_date_day: 5,
            ..baseline_compliant_post_effective_date()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            MassachusettsHomesActMode::CompliantImmediateSealingEligibleForDismissedCase
        );
    }

    #[test]
    fn no_eviction_case_not_applicable() {
        let input = Input {
            eviction_case_outcome: EvictionCaseOutcome::NoEvictionCaseFiled,
            ..baseline_compliant_post_effective_date()
        };
        let result = check(&input);
        assert_eq!(result.mode, MassachusettsHomesActMode::NotApplicableNoEvictionCase);
    }

    #[test]
    fn dismissed_case_immediate_sealing_compliant() {
        let result = check(&baseline_compliant_post_effective_date());
        assert_eq!(
            result.mode,
            MassachusettsHomesActMode::CompliantImmediateSealingEligibleForDismissedCase
        );
    }

    #[test]
    fn tenant_won_case_immediate_sealing_compliant() {
        let input = Input {
            eviction_case_outcome: EvictionCaseOutcome::TenantWonOnMerits,
            ..baseline_compliant_post_effective_date()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            MassachusettsHomesActMode::CompliantImmediateSealingEligibleForTenantWonCase
        );
    }

    #[test]
    fn satisfied_judgment_immediate_sealing_compliant() {
        let input = Input {
            eviction_case_outcome: EvictionCaseOutcome::SatisfiedJudgmentTenantPaid,
            ..baseline_compliant_post_effective_date()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            MassachusettsHomesActMode::CompliantImmediateSealingEligibleForSatisfiedJudgment
        );
    }

    #[test]
    fn non_payment_4_years_with_hardship_compliant() {
        let input = Input {
            eviction_case_outcome: EvictionCaseOutcome::NonPaymentJudgmentNotPaid,
            non_payment_sealing_preconditions:
                NonPaymentSealingPreconditions::EconomicHardshipDocumentedAndNoInterveningLessorAction,
            years_since_case_conclusion: 4,
            ..baseline_compliant_post_effective_date()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            MassachusettsHomesActMode::Compliant4YearPostConclusionSealingForNonPaymentWithConditions
        );
    }

    #[test]
    fn non_payment_under_4_years_violation() {
        let input = Input {
            eviction_case_outcome: EvictionCaseOutcome::NonPaymentJudgmentNotPaid,
            non_payment_sealing_preconditions:
                NonPaymentSealingPreconditions::EconomicHardshipDocumentedAndNoInterveningLessorAction,
            years_since_case_conclusion: 3,
            ..baseline_compliant_post_effective_date()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            MassachusettsHomesActMode::ViolationPrematureNonPaymentSealingBefore4Years
        );
    }

    #[test]
    fn non_payment_no_hardship_documented_violation() {
        let input = Input {
            eviction_case_outcome: EvictionCaseOutcome::NonPaymentJudgmentNotPaid,
            non_payment_sealing_preconditions:
                NonPaymentSealingPreconditions::NoEconomicHardshipDocumented,
            years_since_case_conclusion: 5,
            ..baseline_compliant_post_effective_date()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            MassachusettsHomesActMode::ViolationNonPaymentSealingClaimedWithoutEconomicHardshipDocumented
        );
    }

    #[test]
    fn appeal_period_not_yet_expired_violation() {
        let input = Input {
            eviction_case_outcome: EvictionCaseOutcome::PendingAppealPeriod,
            appeal_period_has_expired: false,
            ..baseline_compliant_post_effective_date()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            MassachusettsHomesActMode::ViolationAppealPeriodNotYetExpired
        );
    }

    #[test]
    fn landlord_inquiry_about_sealed_record_violation() {
        let input = Input {
            landlord_action: LandlordAction::InquiringAboutSealedEvictionRecord,
            ..baseline_compliant_post_effective_date()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            MassachusettsHomesActMode::ViolationLandlordInquiredAboutSealedRecord
        );
    }

    #[test]
    fn landlord_denial_based_on_sealed_record_violation() {
        let input = Input {
            landlord_action: LandlordAction::DenyingHousingBasedOnSealedRecord,
            ..baseline_compliant_post_effective_date()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            MassachusettsHomesActMode::ViolationLandlordDeniedHousingBasedOnSealedRecord
        );
    }

    #[test]
    fn landlord_disclosure_to_third_party_violation() {
        let input = Input {
            landlord_action: LandlordAction::DisclosingSealedRecordToThirdParty,
            ..baseline_compliant_post_effective_date()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            MassachusettsHomesActMode::ViolationLandlordDisclosedSealedRecordToThirdParty
        );
    }

    #[test]
    fn citations_pin_affordable_homes_act_homes_act_and_effective_date() {
        let result = check(&baseline_compliant_post_effective_date());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("Massachusetts Affordable Homes Act of 2024"));
        assert!(joined.contains("H.4138"));
        assert!(joined.contains("Governor Maura Healey"));
        assert!(joined.contains("August 6, 2024"));
        assert!(joined.contains("Chapter 150 of the Acts of 2024"));
        assert!(joined.contains("$5.16 billion"));
        assert!(joined.contains("Section 52"));
        assert!(joined.contains("May 5, 2025"));
        assert!(joined.contains("270 days"));
        assert!(joined.contains("Mass. Gen. Laws Chapter 239"));
        assert!(joined.contains("IMMEDIATELY"));
        assert!(joined.contains("4 YEARS"));
        assert!(joined.contains("economic hardship"));
        assert!(joined.contains("CoreLogic SafeRent"));
        assert!(joined.contains("TransUnion SmartMove"));
        assert!(joined.contains("Experian RentBureau"));
        assert!(joined.contains("ADUs"));
        assert!(joined.contains("Housing Court Department"));
        assert!(joined.contains("MassLandlords"));
        assert!(joined.contains("MLRI"));
        assert!(joined.contains("NLIHC"));
        assert!(joined.contains("Nixon Peabody"));
    }

    #[test]
    fn constant_pin_dates_and_thresholds() {
        assert_eq!(MA_AFFORDABLE_HOMES_ACT_SIGNED_DATE_YEAR, 2024);
        assert_eq!(MA_AFFORDABLE_HOMES_ACT_SIGNED_DATE_MONTH, 8);
        assert_eq!(MA_AFFORDABLE_HOMES_ACT_SIGNED_DATE_DAY, 6);
        assert_eq!(MA_AFFORDABLE_HOMES_ACT_AUTHORIZATION_DOLLARS_BILLIONS, 5_160_000_000);
        assert_eq!(MA_AFFORDABLE_HOMES_ACT_POLICY_INITIATIVES_COUNT, 50);
        assert_eq!(MA_HOMES_ACT_EVICTION_SEALING_EFFECTIVE_DATE_YEAR, 2025);
        assert_eq!(MA_HOMES_ACT_EVICTION_SEALING_EFFECTIVE_DATE_MONTH, 5);
        assert_eq!(MA_HOMES_ACT_EVICTION_SEALING_EFFECTIVE_DATE_DAY, 5);
        assert_eq!(MA_HOMES_ACT_EFFECTIVE_DATE_LAG_DAYS_AFTER_SIGNING, 270);
        assert_eq!(MA_HOMES_ACT_CHAPTER_150_SECTION_52, 52);
        assert_eq!(MA_HOMES_ACT_NON_PAYMENT_WAITING_PERIOD_YEARS, 4);
    }
}
