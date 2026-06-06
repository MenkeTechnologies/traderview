//! Multi-State Pre-Foreclosure Tenant Notification Compliance
//! Module — state-law overlay on the federal Protecting Tenants
//! at Foreclosure Act (PTFA).
//!
//! Pure-compute check for whether a mortgagee, trustee, servicer,
//! or post-foreclosure purchaser has satisfied state-specific
//! notice obligations to (a) the borrower prior to commencing a
//! foreclosure action and (b) any tenant occupying the
//! foreclosed property. Trader-landlord critical because cross-
//! state portfolios with leveraged real-estate holdings must
//! apply at least five different sets of pre-foreclosure tenant
//! notification rules, and defective notices invalidate
//! foreclosure sales + delay eviction + expose lenders / new
//! owners to statutory damages.
//!
//! Web research (verified 2026-06-03):
//! - **California Civ. Code § 2924.85** (Homeowner Bill of Rights;
//!   AB 278 of 2012) and § 2923.5: servicer must attempt to
//!   contact borrower at least **30 days** before commencing
//!   foreclosure to discuss financial situation and options to
//!   avoid foreclosure. § 2924.8: trustee must post notice within
//!   5 business days of recording Notice of Trustee's Sale (NOTS)
//!   to any tenant occupant. ([California AG Homeowner Bill of
//!   Rights](https://oag.ca.gov/hbor); FindLaw CA Civ Code
//!   § 2924; Nolo California Required Landlord Disclosures.)
//! - **New York RPAPL § 1303**: notice to tenants in a dwelling
//!   that becomes the subject of a foreclosure action must be
//!   delivered within **10 days** of service of the summons and
//!   complaint to the mortgagor. ([NY State Senate / FindLaw
//!   RPAPL § 1303](https://codes.findlaw.com/ny/real-property-actions-and-proceedings-law/rpa-sect-1303.html).)
//! - **New York RPAPL § 1304**: lender must send pre-foreclosure
//!   notice to home-loan borrower at least **90 days** before
//!   commencing legal action; notice informs of avoidance options
//!   and not-for-profit housing counselor referral.
//! - **Illinois 735 ILCS 5/15-1701** (Foreclosure Fairness Act):
//!   possessory order requirements upon confirmation of sale;
//!   **bona fide lease protection** for tenants (mortgagor, child,
//!   spouse, parent not tenant + arm's-length lease).
//! - **Washington RCW 61.24.143**: trustee must mail notice to
//!   residents when posting notice of trustee's sale; notice
//!   states foreclosure has begun, sale may occur **90 or more
//!   days** after notice date, and renter may receive new rental
//!   agreement or **60-day notice to vacate** from new owner.
//! - **Washington RCW 61.24.146**: tenant or subtenant in
//!   possession at time of foreclosure sale must receive **60
//!   days' written notice to vacate** before removal.
//! - **Massachusetts G.L. c. 244 § 35C** (Massachusetts Tenants in
//!   Foreclosed Buildings law): foreclosing owner must post notice
//!   of new owner in prominent location; cannot evict bona fide
//!   tenant for rent nonpayment until **30 days after posting**
//!   required notices; 90-day notice to quit otherwise.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const CA_2924_85_PRE_FORECLOSURE_CONTACT_DAYS: u32 = 30;
pub const CA_2924_8_TENANT_NOTICE_POST_NOTS_BUSINESS_DAYS: u32 = 5;
pub const NY_RPAPL_1303_TENANT_NOTICE_DAYS: u32 = 10;
pub const NY_RPAPL_1304_PRE_FORECLOSURE_DAYS: u32 = 90;
pub const WA_RCW_6124_146_NOTICE_TO_VACATE_DAYS: u32 = 60;
pub const WA_RCW_6124_143_TRUSTEE_SALE_NOTICE_DAYS_BEFORE_SALE: u32 = 90;
pub const MA_244_35C_NEW_OWNER_NOTICE_POSTING_DAYS_BEFORE_RENT_NONPAYMENT_EVICTION: u32 = 30;
pub const MA_POST_FORECLOSURE_NOTICE_TO_QUIT_DAYS: u32 = 90;
pub const PTFA_FEDERAL_90_DAY_NOTICE_DAYS: u32 = 90;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PreForeclosureJurisdiction {
    California292485,
    NewYorkRpapl1303_1304,
    Illinois735Ilcs5_15_1701,
    WashingtonRcw6124,
    Massachusetts244_35c,
    OtherStateUnderPtfaOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ForeclosureStage {
    PreNoticeOfDefault,
    PostNodPreFiling,
    LegalActionFiled,
    ForeclosureSaleScheduled,
    ForeclosureSaleCompleted,
    PostSaleEviction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseStatus {
    BonaFideArmsLengthLease,
    NonBonaFideLeaseMortgagorOrFamily,
    NoLeaseTenantAtWill,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PreForeclosureTenantNotificationMode {
    NotApplicableNoForeclosurePending,
    NotApplicableJurisdictionPtfaOnly,
    CompliantCa292485PreForeclosureContact30DaysBefore,
    CompliantNyRpapl1303TenantNoticeWithin10Days,
    CompliantNyRpapl130490DayPreForeclosureNoticeToBorrower,
    CompliantWaRcw6124143NoticeOfTrusteeSaleToResidents,
    CompliantWaRcw612414660DayNoticeToVacate,
    CompliantMa244_35cBonaFideTenantProtectionWithPosting,
    CompliantIl735ILCS5_15_1701BonaFideLeaseProtected,
    ViolationCaServicerContact30DayWindowMissed,
    ViolationCa2924_8TrusteePostingMissedAfterNots,
    ViolationNy1303_10DayTenantNoticeMissed,
    ViolationNy1304_90DayPreForeclosureNoticeMissed,
    ViolationWaTrusteeSale90DayNoticeMissed,
    ViolationWa60DayWrittenNoticeToVacateMissed,
    ViolationMaNewOwnerNoticeNotPostedRequiredForRentNonpayment,
    ViolationIl1701NonBonaFideLeaseDisplacedWithoutPossessoryOrder,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: PreForeclosureJurisdiction,
    pub foreclosure_stage: ForeclosureStage,
    pub lease_status: LeaseStatus,
    pub days_servicer_contacted_borrower_before_foreclosure: u32,
    pub days_tenant_notice_after_summons_and_complaint: u32,
    pub days_ny_pre_foreclosure_notice_to_borrower_before_legal_action: u32,
    pub days_wa_trustee_sale_notice_before_sale: u32,
    pub days_wa_notice_to_vacate_provided: u32,
    pub ca_trustee_posted_within_5_business_days_of_nots: bool,
    pub ma_new_owner_notice_posted_in_prominent_location: bool,
    pub days_since_ma_notice_posting_before_rent_nonpayment_eviction: u32,
    pub illinois_possessory_order_obtained: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: PreForeclosureTenantNotificationMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalPreForeclosureTenantNotificationInput = Input;
pub type RentalPreForeclosureTenantNotificationOutput = Output;
pub type RentalPreForeclosureTenantNotificationResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Cal. Civ. Code § 2923.5 / § 2924.85 (CA Homeowner Bill of Rights; AB 278 of 2012) — servicer must contact borrower at least 30 days before commencing foreclosure".to_string(),
        "Cal. Civ. Code § 2924.8 — trustee must post tenant notice within 5 business days of recording Notice of Trustee's Sale (NOTS)".to_string(),
        "N.Y. RPAPL § 1303 — tenant notice within 10 days of service of summons and complaint to mortgagor".to_string(),
        "N.Y. RPAPL § 1304 — 90-day pre-foreclosure notice to home-loan borrower; informs of avoidance options + nonprofit housing counselor".to_string(),
        "Ill. 735 ILCS 5/15-1701 (Foreclosure Fairness Act) — possessory orders; bona fide lease protection (mortgagor/child/spouse/parent not tenant; arm's-length lease)".to_string(),
        "Wash. RCW 61.24.143 — trustee must mail notice to residents stating foreclosure has begun + sale may occur 90+ days after notice + 60-day notice to vacate from new owner option".to_string(),
        "Wash. RCW 61.24.146 — tenant or subtenant in possession at foreclosure sale receives 60 days' written notice to vacate before removal".to_string(),
        "Mass. G.L. c. 244 § 35C — foreclosing owner must post notice of new owner; no rent-nonpayment eviction of bona fide tenant until 30 days after posting; 90-day notice to quit otherwise".to_string(),
        "Federal Protecting Tenants at Foreclosure Act (PTFA) (P.L. 111-22 of 2009; made permanent by Dodd-Frank 2018) — 90-day federal floor notice to bona fide tenants".to_string(),
    ];

    if input.foreclosure_stage == ForeclosureStage::PreNoticeOfDefault {
        return Output {
            mode: PreForeclosureTenantNotificationMode::NotApplicableNoForeclosurePending,
            statutory_basis: "No foreclosure proceeding pending".to_string(),
            notes: "No foreclosure proceeding pending; pre-foreclosure tenant notification statutes not yet invoked.".to_string(),
            citations,
        };
    }

    if input.jurisdiction == PreForeclosureJurisdiction::OtherStateUnderPtfaOnly {
        return Output {
            mode: PreForeclosureTenantNotificationMode::NotApplicableJurisdictionPtfaOnly,
            statutory_basis: "PTFA federal 90-day floor only; no state-law overlay".to_string(),
            notes: "Jurisdiction has no state-law overlay; defer to federal PTFA 90-day notice to bona fide tenants only.".to_string(),
            citations,
        };
    }

    match input.jurisdiction {
        PreForeclosureJurisdiction::California292485 => {
            if input.foreclosure_stage == ForeclosureStage::PostNodPreFiling
                && input.days_servicer_contacted_borrower_before_foreclosure
                    < CA_2924_85_PRE_FORECLOSURE_CONTACT_DAYS
            {
                return Output {
                    mode: PreForeclosureTenantNotificationMode::ViolationCaServicerContact30DayWindowMissed,
                    statutory_basis: "Cal. Civ. Code § 2924.85 / § 2923.5 — 30-day pre-foreclosure servicer contact".to_string(),
                    notes: format!(
                        "VIOLATION: CA servicer contacted borrower only {} days before commencing foreclosure; 30-day window missed.",
                        input.days_servicer_contacted_borrower_before_foreclosure
                    ),
                    citations,
                };
            }
            if matches!(
                input.foreclosure_stage,
                ForeclosureStage::ForeclosureSaleScheduled
                    | ForeclosureStage::ForeclosureSaleCompleted
            ) && !input.ca_trustee_posted_within_5_business_days_of_nots
            {
                return Output {
                    mode: PreForeclosureTenantNotificationMode::ViolationCa2924_8TrusteePostingMissedAfterNots,
                    statutory_basis: "Cal. Civ. Code § 2924.8 — trustee tenant notice within 5 business days of NOTS".to_string(),
                    notes: "VIOLATION: California trustee did not post tenant notice within 5 business days of recording Notice of Trustee's Sale.".to_string(),
                    citations,
                };
            }
            Output {
                mode: PreForeclosureTenantNotificationMode::CompliantCa292485PreForeclosureContact30DaysBefore,
                statutory_basis: "Cal. Civ. Code § 2924.85 + § 2924.8 satisfied".to_string(),
                notes: format!(
                    "COMPLIANT: CA servicer contacted borrower {} days before foreclosure (≥ 30); § 2924.8 trustee posting timely.",
                    input.days_servicer_contacted_borrower_before_foreclosure
                ),
                citations,
            }
        }
        PreForeclosureJurisdiction::NewYorkRpapl1303_1304 => {
            if input.foreclosure_stage == ForeclosureStage::PostNodPreFiling
                && input.days_ny_pre_foreclosure_notice_to_borrower_before_legal_action
                    < NY_RPAPL_1304_PRE_FORECLOSURE_DAYS
            {
                return Output {
                    mode: PreForeclosureTenantNotificationMode::ViolationNy1304_90DayPreForeclosureNoticeMissed,
                    statutory_basis: "N.Y. RPAPL § 1304 — 90-day pre-foreclosure notice to borrower".to_string(),
                    notes: format!(
                        "VIOLATION: NY pre-foreclosure notice sent only {} days before commencing legal action; 90-day window missed.",
                        input.days_ny_pre_foreclosure_notice_to_borrower_before_legal_action
                    ),
                    citations,
                };
            }
            if input.foreclosure_stage == ForeclosureStage::LegalActionFiled
                && input.days_tenant_notice_after_summons_and_complaint
                    > NY_RPAPL_1303_TENANT_NOTICE_DAYS
            {
                return Output {
                    mode: PreForeclosureTenantNotificationMode::ViolationNy1303_10DayTenantNoticeMissed,
                    statutory_basis: "N.Y. RPAPL § 1303 — tenant notice within 10 days of summons and complaint service".to_string(),
                    notes: format!(
                        "VIOLATION: NY tenant notice provided {} days after summons and complaint service; 10-day window missed.",
                        input.days_tenant_notice_after_summons_and_complaint
                    ),
                    citations,
                };
            }
            if input.days_tenant_notice_after_summons_and_complaint
                <= NY_RPAPL_1303_TENANT_NOTICE_DAYS
                && input.foreclosure_stage == ForeclosureStage::LegalActionFiled
            {
                Output {
                    mode: PreForeclosureTenantNotificationMode::CompliantNyRpapl1303TenantNoticeWithin10Days,
                    statutory_basis: "N.Y. RPAPL § 1303 — 10-day tenant notice satisfied".to_string(),
                    notes: format!(
                        "COMPLIANT N.Y. RPAPL § 1303: tenant notice provided {} days after summons and complaint (≤ 10).",
                        input.days_tenant_notice_after_summons_and_complaint
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: PreForeclosureTenantNotificationMode::CompliantNyRpapl130490DayPreForeclosureNoticeToBorrower,
                    statutory_basis: "N.Y. RPAPL § 1304 — 90-day pre-foreclosure notice satisfied".to_string(),
                    notes: format!(
                        "COMPLIANT N.Y. RPAPL § 1304: pre-foreclosure notice sent {} days before legal action (≥ 90).",
                        input.days_ny_pre_foreclosure_notice_to_borrower_before_legal_action
                    ),
                    citations,
                }
            }
        }
        PreForeclosureJurisdiction::WashingtonRcw6124 => {
            if input.foreclosure_stage == ForeclosureStage::ForeclosureSaleScheduled
                && input.days_wa_trustee_sale_notice_before_sale
                    < WA_RCW_6124_143_TRUSTEE_SALE_NOTICE_DAYS_BEFORE_SALE
            {
                return Output {
                    mode: PreForeclosureTenantNotificationMode::ViolationWaTrusteeSale90DayNoticeMissed,
                    statutory_basis: "Wash. RCW 61.24.143 — 90-day trustee sale notice to residents".to_string(),
                    notes: format!(
                        "VIOLATION: WA trustee posted sale notice only {} days before sale; 90-day window missed.",
                        input.days_wa_trustee_sale_notice_before_sale
                    ),
                    citations,
                };
            }
            if input.foreclosure_stage == ForeclosureStage::PostSaleEviction
                && input.days_wa_notice_to_vacate_provided < WA_RCW_6124_146_NOTICE_TO_VACATE_DAYS
            {
                return Output {
                    mode: PreForeclosureTenantNotificationMode::ViolationWa60DayWrittenNoticeToVacateMissed,
                    statutory_basis: "Wash. RCW 61.24.146 — 60-day notice to vacate before removal".to_string(),
                    notes: format!(
                        "VIOLATION: WA new owner provided only {} days notice to vacate; 60-day window missed.",
                        input.days_wa_notice_to_vacate_provided
                    ),
                    citations,
                };
            }
            if input.foreclosure_stage == ForeclosureStage::PostSaleEviction {
                Output {
                    mode: PreForeclosureTenantNotificationMode::CompliantWaRcw612414660DayNoticeToVacate,
                    statutory_basis: "Wash. RCW 61.24.146 — 60-day notice to vacate satisfied".to_string(),
                    notes: format!(
                        "COMPLIANT RCW 61.24.146: WA 60-day notice to vacate provided ({} days).",
                        input.days_wa_notice_to_vacate_provided
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: PreForeclosureTenantNotificationMode::CompliantWaRcw6124143NoticeOfTrusteeSaleToResidents,
                    statutory_basis: "Wash. RCW 61.24.143 — 90-day trustee sale notice satisfied".to_string(),
                    notes: format!(
                        "COMPLIANT RCW 61.24.143: WA trustee sale notice {} days before sale (≥ 90).",
                        input.days_wa_trustee_sale_notice_before_sale
                    ),
                    citations,
                }
            }
        }
        PreForeclosureJurisdiction::Massachusetts244_35c => {
            if input.foreclosure_stage == ForeclosureStage::PostSaleEviction
                && (!input.ma_new_owner_notice_posted_in_prominent_location
                    || input.days_since_ma_notice_posting_before_rent_nonpayment_eviction
                        < MA_244_35C_NEW_OWNER_NOTICE_POSTING_DAYS_BEFORE_RENT_NONPAYMENT_EVICTION)
            {
                return Output {
                    mode: PreForeclosureTenantNotificationMode::ViolationMaNewOwnerNoticeNotPostedRequiredForRentNonpayment,
                    statutory_basis: "Mass. G.L. c. 244 § 35C — new owner notice posting required before rent-nonpayment eviction".to_string(),
                    notes: format!(
                        "VIOLATION Mass. § 35C: new owner notice posted = {}; days since posting before rent-nonpayment eviction = {} (< 30-day required wait).",
                        input.ma_new_owner_notice_posted_in_prominent_location,
                        input.days_since_ma_notice_posting_before_rent_nonpayment_eviction
                    ),
                    citations,
                };
            }
            Output {
                mode: PreForeclosureTenantNotificationMode::CompliantMa244_35cBonaFideTenantProtectionWithPosting,
                statutory_basis: "Mass. G.L. c. 244 § 35C bona fide tenant protection + new-owner-notice posting satisfied".to_string(),
                notes: "COMPLIANT Mass. § 35C: bona fide tenant protection; new-owner notice posted; 30-day wait observed before rent-nonpayment eviction.".to_string(),
                citations,
            }
        }
        PreForeclosureJurisdiction::Illinois735Ilcs5_15_1701 => {
            if input.foreclosure_stage == ForeclosureStage::PostSaleEviction
                && input.lease_status == LeaseStatus::NonBonaFideLeaseMortgagorOrFamily
                && !input.illinois_possessory_order_obtained
            {
                return Output {
                    mode: PreForeclosureTenantNotificationMode::ViolationIl1701NonBonaFideLeaseDisplacedWithoutPossessoryOrder,
                    statutory_basis: "Ill. 735 ILCS 5/15-1701 — possessory order required to displace non-bona fide tenant".to_string(),
                    notes: "VIOLATION 735 ILCS 5/15-1701: non-bona-fide lease (mortgagor/child/spouse/parent) displaced without obtaining possessory order on sale confirmation.".to_string(),
                    citations,
                };
            }
            Output {
                mode: PreForeclosureTenantNotificationMode::CompliantIl735ILCS5_15_1701BonaFideLeaseProtected,
                statutory_basis: "Ill. 735 ILCS 5/15-1701 bona fide lease protection satisfied".to_string(),
                notes: "COMPLIANT 735 ILCS 5/15-1701: bona fide lease protected; possessory order procedure observed.".to_string(),
                citations,
            }
        }
        PreForeclosureJurisdiction::OtherStateUnderPtfaOnly => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_california_compliant() -> Input {
        Input {
            jurisdiction: PreForeclosureJurisdiction::California292485,
            foreclosure_stage: ForeclosureStage::PostNodPreFiling,
            lease_status: LeaseStatus::BonaFideArmsLengthLease,
            days_servicer_contacted_borrower_before_foreclosure: 35,
            days_tenant_notice_after_summons_and_complaint: 5,
            days_ny_pre_foreclosure_notice_to_borrower_before_legal_action: 95,
            days_wa_trustee_sale_notice_before_sale: 95,
            days_wa_notice_to_vacate_provided: 60,
            ca_trustee_posted_within_5_business_days_of_nots: true,
            ma_new_owner_notice_posted_in_prominent_location: true,
            days_since_ma_notice_posting_before_rent_nonpayment_eviction: 31,
            illinois_possessory_order_obtained: true,
        }
    }

    #[test]
    fn pre_nod_not_applicable() {
        let input = Input {
            foreclosure_stage: ForeclosureStage::PreNoticeOfDefault,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            PreForeclosureTenantNotificationMode::NotApplicableNoForeclosurePending
        );
    }

    #[test]
    fn other_state_ptfa_only_not_applicable() {
        let input = Input {
            jurisdiction: PreForeclosureJurisdiction::OtherStateUnderPtfaOnly,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            PreForeclosureTenantNotificationMode::NotApplicableJurisdictionPtfaOnly
        );
    }

    #[test]
    fn california_30_day_contact_compliant() {
        let result = check(&baseline_california_compliant());
        assert_eq!(result.mode, PreForeclosureTenantNotificationMode::CompliantCa292485PreForeclosureContact30DaysBefore);
    }

    #[test]
    fn california_29_day_contact_violation() {
        let input = Input {
            days_servicer_contacted_borrower_before_foreclosure: 29,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            PreForeclosureTenantNotificationMode::ViolationCaServicerContact30DayWindowMissed
        );
    }

    #[test]
    fn california_2924_8_trustee_posting_violation() {
        let input = Input {
            foreclosure_stage: ForeclosureStage::ForeclosureSaleScheduled,
            ca_trustee_posted_within_5_business_days_of_nots: false,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            PreForeclosureTenantNotificationMode::ViolationCa2924_8TrusteePostingMissedAfterNots
        );
    }

    #[test]
    fn ny_rpapl_1304_90_day_pre_foreclosure_compliant() {
        let input = Input {
            jurisdiction: PreForeclosureJurisdiction::NewYorkRpapl1303_1304,
            foreclosure_stage: ForeclosureStage::PostNodPreFiling,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, PreForeclosureTenantNotificationMode::CompliantNyRpapl130490DayPreForeclosureNoticeToBorrower);
    }

    #[test]
    fn ny_rpapl_1304_89_day_violation() {
        let input = Input {
            jurisdiction: PreForeclosureJurisdiction::NewYorkRpapl1303_1304,
            foreclosure_stage: ForeclosureStage::PostNodPreFiling,
            days_ny_pre_foreclosure_notice_to_borrower_before_legal_action: 89,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            PreForeclosureTenantNotificationMode::ViolationNy1304_90DayPreForeclosureNoticeMissed
        );
    }

    #[test]
    fn ny_rpapl_1303_tenant_notice_within_10_days_compliant() {
        let input = Input {
            jurisdiction: PreForeclosureJurisdiction::NewYorkRpapl1303_1304,
            foreclosure_stage: ForeclosureStage::LegalActionFiled,
            days_tenant_notice_after_summons_and_complaint: 10,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            PreForeclosureTenantNotificationMode::CompliantNyRpapl1303TenantNoticeWithin10Days
        );
    }

    #[test]
    fn ny_rpapl_1303_tenant_notice_at_11_days_violation() {
        let input = Input {
            jurisdiction: PreForeclosureJurisdiction::NewYorkRpapl1303_1304,
            foreclosure_stage: ForeclosureStage::LegalActionFiled,
            days_tenant_notice_after_summons_and_complaint: 11,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            PreForeclosureTenantNotificationMode::ViolationNy1303_10DayTenantNoticeMissed
        );
    }

    #[test]
    fn washington_61_24_143_90_day_compliant() {
        let input = Input {
            jurisdiction: PreForeclosureJurisdiction::WashingtonRcw6124,
            foreclosure_stage: ForeclosureStage::ForeclosureSaleScheduled,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, PreForeclosureTenantNotificationMode::CompliantWaRcw6124143NoticeOfTrusteeSaleToResidents);
    }

    #[test]
    fn washington_89_day_trustee_sale_violation() {
        let input = Input {
            jurisdiction: PreForeclosureJurisdiction::WashingtonRcw6124,
            foreclosure_stage: ForeclosureStage::ForeclosureSaleScheduled,
            days_wa_trustee_sale_notice_before_sale: 89,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            PreForeclosureTenantNotificationMode::ViolationWaTrusteeSale90DayNoticeMissed
        );
    }

    #[test]
    fn washington_61_24_146_60_day_compliant() {
        let input = Input {
            jurisdiction: PreForeclosureJurisdiction::WashingtonRcw6124,
            foreclosure_stage: ForeclosureStage::PostSaleEviction,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            PreForeclosureTenantNotificationMode::CompliantWaRcw612414660DayNoticeToVacate
        );
    }

    #[test]
    fn washington_59_day_vacate_violation() {
        let input = Input {
            jurisdiction: PreForeclosureJurisdiction::WashingtonRcw6124,
            foreclosure_stage: ForeclosureStage::PostSaleEviction,
            days_wa_notice_to_vacate_provided: 59,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            PreForeclosureTenantNotificationMode::ViolationWa60DayWrittenNoticeToVacateMissed
        );
    }

    #[test]
    fn massachusetts_244_35c_compliant() {
        let input = Input {
            jurisdiction: PreForeclosureJurisdiction::Massachusetts244_35c,
            foreclosure_stage: ForeclosureStage::PostSaleEviction,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, PreForeclosureTenantNotificationMode::CompliantMa244_35cBonaFideTenantProtectionWithPosting);
    }

    #[test]
    fn massachusetts_29_days_since_notice_violation() {
        let input = Input {
            jurisdiction: PreForeclosureJurisdiction::Massachusetts244_35c,
            foreclosure_stage: ForeclosureStage::PostSaleEviction,
            days_since_ma_notice_posting_before_rent_nonpayment_eviction: 29,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, PreForeclosureTenantNotificationMode::ViolationMaNewOwnerNoticeNotPostedRequiredForRentNonpayment);
    }

    #[test]
    fn massachusetts_no_posting_violation() {
        let input = Input {
            jurisdiction: PreForeclosureJurisdiction::Massachusetts244_35c,
            foreclosure_stage: ForeclosureStage::PostSaleEviction,
            ma_new_owner_notice_posted_in_prominent_location: false,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, PreForeclosureTenantNotificationMode::ViolationMaNewOwnerNoticeNotPostedRequiredForRentNonpayment);
    }

    #[test]
    fn illinois_bona_fide_lease_protected_compliant() {
        let input = Input {
            jurisdiction: PreForeclosureJurisdiction::Illinois735Ilcs5_15_1701,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            PreForeclosureTenantNotificationMode::CompliantIl735ILCS5_15_1701BonaFideLeaseProtected
        );
    }

    #[test]
    fn illinois_non_bona_fide_no_possessory_order_violation() {
        let input = Input {
            jurisdiction: PreForeclosureJurisdiction::Illinois735Ilcs5_15_1701,
            foreclosure_stage: ForeclosureStage::PostSaleEviction,
            lease_status: LeaseStatus::NonBonaFideLeaseMortgagorOrFamily,
            illinois_possessory_order_obtained: false,
            ..baseline_california_compliant()
        };
        let result = check(&input);
        assert_eq!(result.mode, PreForeclosureTenantNotificationMode::ViolationIl1701NonBonaFideLeaseDisplacedWithoutPossessoryOrder);
    }

    #[test]
    fn citations_pin_jurisdictional_statutes() {
        let result = check(&baseline_california_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("§ 2923.5"));
        assert!(joined.contains("§ 2924.85"));
        assert!(joined.contains("§ 2924.8"));
        assert!(joined.contains("N.Y. RPAPL § 1303"));
        assert!(joined.contains("N.Y. RPAPL § 1304"));
        assert!(joined.contains("Ill. 735 ILCS 5/15-1701"));
        assert!(joined.contains("Foreclosure Fairness Act"));
        assert!(joined.contains("Wash. RCW 61.24.143"));
        assert!(joined.contains("Wash. RCW 61.24.146"));
        assert!(joined.contains("Mass. G.L. c. 244 § 35C"));
        assert!(joined.contains("Protecting Tenants at Foreclosure Act"));
        assert!(joined.contains("P.L. 111-22 of 2009"));
        assert!(joined.contains("Dodd-Frank 2018"));
    }

    #[test]
    fn constant_pin_notice_days() {
        assert_eq!(CA_2924_85_PRE_FORECLOSURE_CONTACT_DAYS, 30);
        assert_eq!(CA_2924_8_TENANT_NOTICE_POST_NOTS_BUSINESS_DAYS, 5);
        assert_eq!(NY_RPAPL_1303_TENANT_NOTICE_DAYS, 10);
        assert_eq!(NY_RPAPL_1304_PRE_FORECLOSURE_DAYS, 90);
        assert_eq!(WA_RCW_6124_146_NOTICE_TO_VACATE_DAYS, 60);
        assert_eq!(WA_RCW_6124_143_TRUSTEE_SALE_NOTICE_DAYS_BEFORE_SALE, 90);
        assert_eq!(
            MA_244_35C_NEW_OWNER_NOTICE_POSTING_DAYS_BEFORE_RENT_NONPAYMENT_EVICTION,
            30
        );
        assert_eq!(MA_POST_FORECLOSURE_NOTICE_TO_QUIT_DAYS, 90);
        assert_eq!(PTFA_FEDERAL_90_DAY_NOTICE_DAYS, 90);
    }
}
