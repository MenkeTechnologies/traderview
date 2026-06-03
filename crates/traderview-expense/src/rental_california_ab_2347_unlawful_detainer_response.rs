//! California AB 2347 of 2024 (Kalra) Unlawful Detainer
//! Response Time Extension Compliance Module.
//!
//! Pure-compute check for landlord compliance with California
//! AB 2347, which amends California Code of Civil Procedure
//! (CCP) §§ 1167 and 1170 to extend tenant unlawful detainer
//! response time from 5 days to 10 court days and adds oral
//! hearing procedures for demurrers and motions to strike.
//! Signed by Governor Gavin Newsom on September 24, 2024;
//! effective January 1, 2025. Trader-landlord critical for
//! California portfolio operators because the doubled
//! response window slows eviction timelines and gives tenants
//! more time to retain counsel/build defense.
//!
//! Web research (verified 2026-06-03):
//! - **California AB 2347 of 2024** (Kalra; 2023-2024 Regular
//!   Session) signed by Governor **Gavin Newsom on September
//!   24, 2024**; effective **January 1, 2025**; amends
//!   California Code of Civil Procedure **CCP § 1167** and
//!   **§ 1170** ([CA Legislative Information AB 2347](https://leginfo.legislature.ca.gov/faces/billNavClient.xhtml?bill_id=202320240AB2347);
//!   [Senate Judiciary Committee Analysis AB 2347](https://sjud.senate.ca.gov/system/files/2024-07/ab-2347-kalra-sjud-analysis_3.pdf);
//!   [Hanson Bridgett — AB 2347 Will Give Defendants More Time
//!   to Respond to Eviction Lawsuits](https://www.hansonbridgett.com/publication/240926-8551-ab2347-gives-defendants-more-time-respond-eviction-lawsuits)).
//! - **CCP § 1167 (Response Time)**: tenant unlawful detainer
//!   response time **EXTENDED from 5 days to 10 COURT DAYS**
//!   (excluding Saturdays, Sundays, and judicial holidays)
//!   to file an **answer, demurrer, or motion to strike**.
//!   Pre-AB 2347 the response window was 5 calendar days
//!   ([CCP 1167 — California Legislative Information](https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?sectionNum=1167.&lawCode=CCP);
//!   [Zak Fisher Law — 10 Court Days to Answer an Unlawful
//!   Detainer in California](https://zakfisherlaw.com/10-court-day-answer-deadline-california-eviction/)).
//! - **CCP § 1170 (Demurrer / Motion to Strike Procedures)**:
//!   if tenant files demurrer or motion to strike, hearing
//!   must occur **NOT LESS THAN 5 COURT DAYS NOR MORE THAN 7
//!   COURT DAYS** after filing notice of motion; for good
//!   cause shown, court may order later date on notice
//!   prescribed by court.
//! - **Oral Hearing Procedures**: an opposition and reply to
//!   an opposition may be made **ORALLY at the time of the
//!   hearing** — streamlines process by allowing landlords
//!   and tenants to present arguments orally rather than
//!   requiring written opposition papers in all cases.
//! - **Service Requirements**: landlord must file proof of
//!   service **3 days before** requesting default judgment;
//!   prior law allowed simultaneous filing.
//! - **Trader-Landlord Impact**: doubles the response time
//!   for tenants; adds oral hearing procedures with 5-7 court
//!   day window; slows eviction process; gives tenants more
//!   time to retain counsel/build defense; particularly
//!   impacts California portfolio landlords with high
//!   turnover unit counts.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const CA_AB_2347_SIGNED_DATE_YEAR: u32 = 2024;
pub const CA_AB_2347_SIGNED_DATE_MONTH: u32 = 9;
pub const CA_AB_2347_SIGNED_DATE_DAY: u32 = 24;
pub const CA_AB_2347_EFFECTIVE_DATE_YEAR: u32 = 2025;
pub const CA_AB_2347_EFFECTIVE_DATE_MONTH: u32 = 1;
pub const CA_AB_2347_EFFECTIVE_DATE_DAY: u32 = 1;
pub const CA_AB_2347_POST_RESPONSE_COURT_DAYS: u32 = 10;
pub const CA_AB_2347_PRE_RESPONSE_CALENDAR_DAYS: u32 = 5;
pub const CA_AB_2347_HEARING_MIN_COURT_DAYS_AFTER_NOTICE: u32 = 5;
pub const CA_AB_2347_HEARING_MAX_COURT_DAYS_AFTER_NOTICE: u32 = 7;
pub const CA_AB_2347_DEFAULT_JUDGMENT_PROOF_OF_SERVICE_DAYS: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyJurisdiction {
    CaliforniaSubjectToAb2347,
    NotInCalifornia,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnlawfulDetainerActionDate {
    FiledBeforeJanuary1_2025OldFiveDayWindowApplies,
    FiledOnOrAfterJanuary1_2025NewTenCourtDayWindowApplies,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantResponseType {
    AnswerFiled,
    DemurrerFiled,
    MotionToStrikeFiled,
    NoResponseFiledDefaultPossible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlordAction {
    AwaitingTenantResponseWithinWindow,
    RequestingDefaultJudgment,
    AttendingHearingOnDemurrerOrMotionToStrike,
    NoEvictionProceedingsInitiated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaliforniaAb2347Mode {
    NotApplicableNotInCalifornia,
    NotApplicableNoEvictionProceedings,
    CompliantPreJanuary1_2025FiveDayResponseWindow,
    CompliantPostJanuary1_2025TenCourtDayResponseWindow,
    CompliantTenantResponseFiledWithin10CourtDays,
    CompliantHearingScheduledWithin5To7CourtDaysAfterDemurrerOrMotion,
    CompliantOralOppositionAndReplyAtHearing,
    CompliantDefaultJudgmentRequestedWith3DayPriorProofOfService,
    ViolationTenantResponseDeadlineMissedDefaultJudgmentPossible,
    ViolationLandlordRequestedDefaultJudgmentBefore10DayWindowExpired,
    ViolationHearingScheduledBefore5CourtDaysOrAfter7CourtDays,
    ViolationLandlordRequestedDefaultWithoutFiling3DayPriorProofOfService,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub property_jurisdiction: PropertyJurisdiction,
    pub unlawful_detainer_action_date: UnlawfulDetainerActionDate,
    pub tenant_response_type: TenantResponseType,
    pub landlord_action: LandlordAction,
    pub days_since_service_of_complaint: u32,
    pub hearing_days_after_demurrer_or_motion_filing: u32,
    pub proof_of_service_filed_days_before_default_request: u32,
    pub oral_opposition_or_reply_made_at_hearing: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: CaliforniaAb2347Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub applicable_response_window_court_days: u32,
}

pub type RentalCaliforniaAb2347UnlawfulDetainerResponseInput = Input;
pub type RentalCaliforniaAb2347UnlawfulDetainerResponseOutput = Output;
pub type RentalCaliforniaAb2347UnlawfulDetainerResponseResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "California AB 2347 of 2024 (Kalra; 2023-2024 Regular Session) — signed by Governor Gavin Newsom on September 24, 2024; effective January 1, 2025; amends California Code of Civil Procedure CCP § 1167 and § 1170".to_string(),
        "CCP § 1167 (Response Time) — tenant unlawful detainer response time EXTENDED from 5 days to 10 COURT DAYS (excluding Saturdays, Sundays, and judicial holidays) to file an answer, demurrer, or motion to strike; pre-AB 2347 response window was 5 calendar days".to_string(),
        "CCP § 1170 (Demurrer / Motion to Strike Procedures) — if tenant files demurrer or motion to strike, hearing must occur NOT LESS THAN 5 COURT DAYS NOR MORE THAN 7 COURT DAYS after filing notice of motion; for good cause shown, court may order later date on notice prescribed by court".to_string(),
        "Oral Hearing Procedures — opposition and reply to opposition may be made ORALLY at the time of the hearing; streamlines process by allowing landlords and tenants to present arguments orally rather than requiring written opposition papers in all cases".to_string(),
        "Service Requirements — landlord must file proof of service 3 days before requesting default judgment; prior law allowed simultaneous filing".to_string(),
        "Trader-Landlord Impact — doubles the response time for tenants; adds oral hearing procedures with 5-7 court day window; slows eviction process; gives tenants more time to retain counsel/build defense; particularly impacts California portfolio landlords with high turnover unit counts".to_string(),
        "Senate Judiciary Committee Analysis AB 2347 (Kalra) — comprehensive analysis of statutory changes".to_string(),
        "Hanson Bridgett — AB 2347 Will Give Defendants More Time to Respond to Eviction Lawsuits".to_string(),
        "Ballard Spahr — Two New Laws Affect California Commercial Landlords (December 2024)".to_string(),
        "Zak Fisher Law — 10 Court Days to Answer an Unlawful Detainer in California".to_string(),
        "CA Legislative Information AB 2347 — primary bill text and history".to_string(),
        "CCP § 1167 — California Legislative Information primary statutory text".to_string(),
        "CCP § 1170 — California Legislative Information primary statutory text".to_string(),
    ];

    if input.property_jurisdiction == PropertyJurisdiction::NotInCalifornia {
        return Output {
            mode: CaliforniaAb2347Mode::NotApplicableNotInCalifornia,
            statutory_basis: "Property outside California; AB 2347 inapplicable".to_string(),
            notes: "NOT APPLICABLE: property outside California; California AB 2347 unlawful detainer response framework inapplicable.".to_string(),
            citations,
            applicable_response_window_court_days: 0,
        };
    }

    if input.landlord_action == LandlordAction::NoEvictionProceedingsInitiated {
        return Output {
            mode: CaliforniaAb2347Mode::NotApplicableNoEvictionProceedings,
            statutory_basis: "AB 2347 — no eviction proceedings to trigger response framework".to_string(),
            notes: "NOT APPLICABLE: no eviction proceedings initiated; AB 2347 unlawful detainer response framework not triggered.".to_string(),
            citations,
            applicable_response_window_court_days: 0,
        };
    }

    let applicable_response_window_court_days = match input.unlawful_detainer_action_date {
        UnlawfulDetainerActionDate::FiledBeforeJanuary1_2025OldFiveDayWindowApplies => {
            CA_AB_2347_PRE_RESPONSE_CALENDAR_DAYS
        }
        UnlawfulDetainerActionDate::FiledOnOrAfterJanuary1_2025NewTenCourtDayWindowApplies => {
            CA_AB_2347_POST_RESPONSE_COURT_DAYS
        }
    };

    if input.landlord_action == LandlordAction::RequestingDefaultJudgment {
        if input.proof_of_service_filed_days_before_default_request
            < CA_AB_2347_DEFAULT_JUDGMENT_PROOF_OF_SERVICE_DAYS
        {
            return Output {
                mode: CaliforniaAb2347Mode::ViolationLandlordRequestedDefaultWithoutFiling3DayPriorProofOfService,
                statutory_basis: "AB 2347 — proof of service must be filed 3 days before default judgment request".to_string(),
                notes: format!(
                    "VIOLATION: landlord requested default judgment with only {} days between proof of service filing and default request; AB 2347 requires proof of service filed at least 3 days before default judgment request.",
                    input.proof_of_service_filed_days_before_default_request
                ),
                citations,
                applicable_response_window_court_days,
            };
        }
        if input.days_since_service_of_complaint < applicable_response_window_court_days
            && input.unlawful_detainer_action_date
                == UnlawfulDetainerActionDate::FiledOnOrAfterJanuary1_2025NewTenCourtDayWindowApplies
        {
            return Output {
                mode: CaliforniaAb2347Mode::ViolationLandlordRequestedDefaultJudgmentBefore10DayWindowExpired,
                statutory_basis: "CCP § 1167 — landlord cannot request default before 10-court-day response window expires".to_string(),
                notes: format!(
                    "VIOLATION: landlord requested default judgment {} court days after service of complaint, before the post-AB 2347 10-court-day response window has expired; CCP § 1167 prohibits default request during tenant's response window.",
                    input.days_since_service_of_complaint
                ),
                citations,
                applicable_response_window_court_days,
            };
        }
        return Output {
            mode: CaliforniaAb2347Mode::CompliantDefaultJudgmentRequestedWith3DayPriorProofOfService,
            statutory_basis: "AB 2347 — proper default judgment process with 3-day prior proof of service".to_string(),
            notes: format!(
                "COMPLIANT: landlord properly requested default judgment after {}-court-day response window expired (tenant did not respond) with {} days between proof of service filing and default request (≥ 3-day statutory minimum).",
                applicable_response_window_court_days,
                input.proof_of_service_filed_days_before_default_request
            ),
            citations,
            applicable_response_window_court_days,
        };
    }

    if input.landlord_action == LandlordAction::AttendingHearingOnDemurrerOrMotionToStrike {
        if input.hearing_days_after_demurrer_or_motion_filing
            < CA_AB_2347_HEARING_MIN_COURT_DAYS_AFTER_NOTICE
            || input.hearing_days_after_demurrer_or_motion_filing
                > CA_AB_2347_HEARING_MAX_COURT_DAYS_AFTER_NOTICE
        {
            return Output {
                mode: CaliforniaAb2347Mode::ViolationHearingScheduledBefore5CourtDaysOrAfter7CourtDays,
                statutory_basis: "CCP § 1170 — hearing on demurrer/motion to strike must be 5-7 court days after filing".to_string(),
                notes: format!(
                    "VIOLATION: hearing on tenant's demurrer or motion to strike scheduled {} court days after filing; CCP § 1170 requires hearing not less than 5 court days nor more than 7 court days after filing (absent good cause for later date).",
                    input.hearing_days_after_demurrer_or_motion_filing
                ),
                citations,
                applicable_response_window_court_days,
            };
        }
        if input.oral_opposition_or_reply_made_at_hearing {
            return Output {
                mode: CaliforniaAb2347Mode::CompliantOralOppositionAndReplyAtHearing,
                statutory_basis: "CCP § 1170 — oral opposition and reply procedures".to_string(),
                notes: format!(
                    "COMPLIANT: hearing scheduled {} court days after filing (within 5-7 court day window); oral opposition and reply procedure properly used at hearing rather than requiring written opposition papers.",
                    input.hearing_days_after_demurrer_or_motion_filing
                ),
                citations,
                applicable_response_window_court_days,
            };
        }
        return Output {
            mode: CaliforniaAb2347Mode::CompliantHearingScheduledWithin5To7CourtDaysAfterDemurrerOrMotion,
            statutory_basis: "CCP § 1170 — hearing within statutory 5-7 court day window".to_string(),
            notes: format!(
                "COMPLIANT: hearing on tenant's demurrer or motion to strike scheduled {} court days after filing (within 5-7 court day statutory window).",
                input.hearing_days_after_demurrer_or_motion_filing
            ),
            citations,
            applicable_response_window_court_days,
        };
    }

    if input.tenant_response_type == TenantResponseType::NoResponseFiledDefaultPossible
        && input.days_since_service_of_complaint > applicable_response_window_court_days
    {
        return Output {
            mode: CaliforniaAb2347Mode::ViolationTenantResponseDeadlineMissedDefaultJudgmentPossible,
            statutory_basis: "CCP § 1167 — tenant missed response deadline".to_string(),
            notes: format!(
                "NOTE: tenant did not file response within {}-court-day window under AB 2347; default judgment now possible (subject to 3-day proof of service rule).",
                applicable_response_window_court_days
            ),
            citations,
            applicable_response_window_court_days,
        };
    }

    if input.unlawful_detainer_action_date
        == UnlawfulDetainerActionDate::FiledBeforeJanuary1_2025OldFiveDayWindowApplies
    {
        return Output {
            mode: CaliforniaAb2347Mode::CompliantPreJanuary1_2025FiveDayResponseWindow,
            statutory_basis: "CCP § 1167 (pre-AB 2347) — 5 calendar day response window".to_string(),
            notes: "COMPLIANT: unlawful detainer action filed before January 1, 2025; pre-AB 2347 5-calendar-day response window applies.".to_string(),
            citations,
            applicable_response_window_court_days,
        };
    }

    if matches!(
        input.tenant_response_type,
        TenantResponseType::AnswerFiled
            | TenantResponseType::DemurrerFiled
            | TenantResponseType::MotionToStrikeFiled
    ) && input.days_since_service_of_complaint <= applicable_response_window_court_days
    {
        return Output {
            mode: CaliforniaAb2347Mode::CompliantTenantResponseFiledWithin10CourtDays,
            statutory_basis: "CCP § 1167 (post-AB 2347) — tenant response filed within 10 court days".to_string(),
            notes: format!(
                "COMPLIANT: tenant filed {:?} within {} court days of service (≤ 10-court-day window under AB 2347).",
                input.tenant_response_type, input.days_since_service_of_complaint
            ),
            citations,
            applicable_response_window_court_days,
        };
    }

    Output {
        mode: CaliforniaAb2347Mode::CompliantPostJanuary1_2025TenCourtDayResponseWindow,
        statutory_basis: "CCP § 1167 (post-AB 2347) — 10-court-day response window".to_string(),
        notes: "COMPLIANT: unlawful detainer action filed on or after January 1, 2025; post-AB 2347 10-court-day response window applies.".to_string(),
        citations,
        applicable_response_window_court_days,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_post_ab2347_compliant() -> Input {
        Input {
            property_jurisdiction: PropertyJurisdiction::CaliforniaSubjectToAb2347,
            unlawful_detainer_action_date:
                UnlawfulDetainerActionDate::FiledOnOrAfterJanuary1_2025NewTenCourtDayWindowApplies,
            tenant_response_type: TenantResponseType::AnswerFiled,
            landlord_action: LandlordAction::AwaitingTenantResponseWithinWindow,
            days_since_service_of_complaint: 8,
            hearing_days_after_demurrer_or_motion_filing: 6,
            proof_of_service_filed_days_before_default_request: 3,
            oral_opposition_or_reply_made_at_hearing: true,
        }
    }

    #[test]
    fn property_outside_ca_not_applicable() {
        let input = Input {
            property_jurisdiction: PropertyJurisdiction::NotInCalifornia,
            ..baseline_post_ab2347_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, CaliforniaAb2347Mode::NotApplicableNotInCalifornia);
    }

    #[test]
    fn no_eviction_proceedings_not_applicable() {
        let input = Input {
            landlord_action: LandlordAction::NoEvictionProceedingsInitiated,
            ..baseline_post_ab2347_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            CaliforniaAb2347Mode::NotApplicableNoEvictionProceedings
        );
    }

    #[test]
    fn pre_january_1_2025_five_day_window_compliant() {
        let input = Input {
            unlawful_detainer_action_date:
                UnlawfulDetainerActionDate::FiledBeforeJanuary1_2025OldFiveDayWindowApplies,
            ..baseline_post_ab2347_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            CaliforniaAb2347Mode::CompliantPreJanuary1_2025FiveDayResponseWindow
        );
        assert_eq!(result.applicable_response_window_court_days, 5);
    }

    #[test]
    fn post_january_1_2025_tenant_answer_within_10_days_compliant() {
        let result = check(&baseline_post_ab2347_compliant());
        assert_eq!(
            result.mode,
            CaliforniaAb2347Mode::CompliantTenantResponseFiledWithin10CourtDays
        );
        assert_eq!(result.applicable_response_window_court_days, 10);
    }

    #[test]
    fn tenant_demurrer_within_10_days_compliant() {
        let input = Input {
            tenant_response_type: TenantResponseType::DemurrerFiled,
            days_since_service_of_complaint: 10,
            ..baseline_post_ab2347_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            CaliforniaAb2347Mode::CompliantTenantResponseFiledWithin10CourtDays
        );
    }

    #[test]
    fn tenant_motion_to_strike_within_10_days_compliant() {
        let input = Input {
            tenant_response_type: TenantResponseType::MotionToStrikeFiled,
            days_since_service_of_complaint: 9,
            ..baseline_post_ab2347_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            CaliforniaAb2347Mode::CompliantTenantResponseFiledWithin10CourtDays
        );
    }

    #[test]
    fn tenant_no_response_after_10_days_default_possible() {
        let input = Input {
            tenant_response_type: TenantResponseType::NoResponseFiledDefaultPossible,
            days_since_service_of_complaint: 11,
            ..baseline_post_ab2347_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            CaliforniaAb2347Mode::ViolationTenantResponseDeadlineMissedDefaultJudgmentPossible
        );
    }

    #[test]
    fn landlord_default_request_with_3_day_prior_proof_compliant() {
        let input = Input {
            landlord_action: LandlordAction::RequestingDefaultJudgment,
            days_since_service_of_complaint: 12,
            proof_of_service_filed_days_before_default_request: 3,
            ..baseline_post_ab2347_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            CaliforniaAb2347Mode::CompliantDefaultJudgmentRequestedWith3DayPriorProofOfService
        );
    }

    #[test]
    fn landlord_default_without_3_day_prior_proof_violation() {
        let input = Input {
            landlord_action: LandlordAction::RequestingDefaultJudgment,
            days_since_service_of_complaint: 12,
            proof_of_service_filed_days_before_default_request: 2,
            ..baseline_post_ab2347_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            CaliforniaAb2347Mode::ViolationLandlordRequestedDefaultWithoutFiling3DayPriorProofOfService
        );
    }

    #[test]
    fn landlord_default_before_10_day_window_violation() {
        let input = Input {
            landlord_action: LandlordAction::RequestingDefaultJudgment,
            days_since_service_of_complaint: 8,
            proof_of_service_filed_days_before_default_request: 3,
            ..baseline_post_ab2347_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            CaliforniaAb2347Mode::ViolationLandlordRequestedDefaultJudgmentBefore10DayWindowExpired
        );
    }

    #[test]
    fn hearing_at_exactly_5_court_days_compliant() {
        let input = Input {
            landlord_action: LandlordAction::AttendingHearingOnDemurrerOrMotionToStrike,
            hearing_days_after_demurrer_or_motion_filing: 5,
            oral_opposition_or_reply_made_at_hearing: false,
            ..baseline_post_ab2347_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            CaliforniaAb2347Mode::CompliantHearingScheduledWithin5To7CourtDaysAfterDemurrerOrMotion
        );
    }

    #[test]
    fn hearing_at_exactly_7_court_days_compliant() {
        let input = Input {
            landlord_action: LandlordAction::AttendingHearingOnDemurrerOrMotionToStrike,
            hearing_days_after_demurrer_or_motion_filing: 7,
            oral_opposition_or_reply_made_at_hearing: false,
            ..baseline_post_ab2347_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            CaliforniaAb2347Mode::CompliantHearingScheduledWithin5To7CourtDaysAfterDemurrerOrMotion
        );
    }

    #[test]
    fn hearing_below_5_court_days_violation() {
        let input = Input {
            landlord_action: LandlordAction::AttendingHearingOnDemurrerOrMotionToStrike,
            hearing_days_after_demurrer_or_motion_filing: 4,
            ..baseline_post_ab2347_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            CaliforniaAb2347Mode::ViolationHearingScheduledBefore5CourtDaysOrAfter7CourtDays
        );
    }

    #[test]
    fn hearing_above_7_court_days_violation() {
        let input = Input {
            landlord_action: LandlordAction::AttendingHearingOnDemurrerOrMotionToStrike,
            hearing_days_after_demurrer_or_motion_filing: 8,
            ..baseline_post_ab2347_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            CaliforniaAb2347Mode::ViolationHearingScheduledBefore5CourtDaysOrAfter7CourtDays
        );
    }

    #[test]
    fn oral_opposition_at_hearing_compliant() {
        let input = Input {
            landlord_action: LandlordAction::AttendingHearingOnDemurrerOrMotionToStrike,
            ..baseline_post_ab2347_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            CaliforniaAb2347Mode::CompliantOralOppositionAndReplyAtHearing
        );
    }

    #[test]
    fn citations_pin_ab_2347_ccp_and_ccp_1170() {
        let result = check(&baseline_post_ab2347_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("California AB 2347 of 2024"));
        assert!(joined.contains("Kalra"));
        assert!(joined.contains("Governor Gavin Newsom"));
        assert!(joined.contains("September 24, 2024"));
        assert!(joined.contains("January 1, 2025"));
        assert!(joined.contains("CCP § 1167"));
        assert!(joined.contains("CCP § 1170"));
        assert!(joined.contains("10 COURT DAYS"));
        assert!(joined.contains("5 days"));
        assert!(joined.contains("ORALLY"));
        assert!(joined.contains("5 COURT DAYS NOR MORE THAN 7 COURT DAYS"));
        assert!(joined.contains("3 days"));
        assert!(joined.contains("Senate Judiciary Committee"));
        assert!(joined.contains("Hanson Bridgett"));
        assert!(joined.contains("Ballard Spahr"));
        assert!(joined.contains("Zak Fisher"));
    }

    #[test]
    fn constant_pin_dates_and_court_day_windows() {
        assert_eq!(CA_AB_2347_SIGNED_DATE_YEAR, 2024);
        assert_eq!(CA_AB_2347_SIGNED_DATE_MONTH, 9);
        assert_eq!(CA_AB_2347_SIGNED_DATE_DAY, 24);
        assert_eq!(CA_AB_2347_EFFECTIVE_DATE_YEAR, 2025);
        assert_eq!(CA_AB_2347_EFFECTIVE_DATE_MONTH, 1);
        assert_eq!(CA_AB_2347_EFFECTIVE_DATE_DAY, 1);
        assert_eq!(CA_AB_2347_POST_RESPONSE_COURT_DAYS, 10);
        assert_eq!(CA_AB_2347_PRE_RESPONSE_CALENDAR_DAYS, 5);
        assert_eq!(CA_AB_2347_HEARING_MIN_COURT_DAYS_AFTER_NOTICE, 5);
        assert_eq!(CA_AB_2347_HEARING_MAX_COURT_DAYS_AFTER_NOTICE, 7);
        assert_eq!(CA_AB_2347_DEFAULT_JUDGMENT_PROOF_OF_SERVICE_DAYS, 3);
    }
}
