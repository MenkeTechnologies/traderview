//! Federal Protecting Tenants at Foreclosure Act (PTFA) compliance for
//! trader-landlords acquiring foreclosed rental property as "immediate
//! successor in interest".
//!
//! Statutory chain:
//!
//! - Originally enacted as Pub. L. 111-22 Title VII §§ 701-704 (May 20,
//!   2009) with December 31, 2014 sunset.
//! - Briefly expired January 1, 2015 through June 22, 2018.
//! - Reinstated PERMANENTLY by Pub. L. 115-174 § 304 (Economic Growth,
//!   Regulatory Relief, and Consumer Protection Act of 2018), effective
//!   June 23, 2018 — no further sunset.
//!
//! Core obligations on the immediate successor in interest (the
//! foreclosure-sale purchaser, typically the foreclosing bank or a
//! trader-landlord buying at auction):
//!
//! 1. **Bona fide lease**: takes title SUBJECT TO the tenant's right to
//!    occupy through the remaining lease term.
//! 2. **Bona fide month-to-month or non-written tenancy**: must serve
//!    minimum **90 days** written vacate notice before requiring tenant
//!    to vacate.
//! 3. **Owner-occupy exception** (the only carveout that can terminate
//!    a bona fide lease early): if the successor SELLS to a purchaser
//!    who will occupy as PRIMARY RESIDENCE, the lease terminates — but
//!    the tenant STILL receives 90 days written notice.
//!
//! Bona fide tenant definition (PTFA § 702(b), 12 U.S.C. § 5220 note —
//! three conjunctive prongs):
//!
//! 1. Tenant is NOT the mortgagor or the spouse, parent, or child of
//!    the mortgagor.
//! 2. Lease was the result of an ARM'S-LENGTH transaction.
//! 3. Rent is NOT substantially less than fair market rent (FMR), OR
//!    rent is reduced/subsidized due to a federal, state, or local
//!    subsidy (e.g., Section 8 Housing Choice Voucher).
//!
//! The "substantially less than FMR" test has no statutory bright
//! line; HUD guidance and most circuit-court interpretations treat
//! rent below 80% of FMR as substantially less.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const PTFA_MINIMUM_VACATE_NOTICE_DAYS: u32 = 90;
#[allow(dead_code)]
pub const PTFA_PERMANENT_REINSTATEMENT_YEAR: u32 = 2018;
#[allow(dead_code)]
pub const SUBSTANTIALLY_LESS_THAN_FMR_THRESHOLD_PERCENT: u64 = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    BonaFideLeaseAllowedFullTermRemaining,
    BonaFideMonthToMonthMustGive90DayVacateNotice,
    BonaFideLeaseEarlyTermByOwnerOccupyExceptionStill90DayNotice,
    NonBonaFideTenantNoProtectionMortgagorOrImmediateFamily,
    NonBonaFideTenantNoProtectionNonArmsLengthTransaction,
    NonBonaFideTenantNoProtectionBelowMarketRentNotSubsidized,
    ViolationLessThan90DayNoticeServed,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub successor_acquired_at_foreclosure: bool,
    pub tenant_is_mortgagor_or_immediate_family: bool,
    pub lease_was_arms_length: bool,
    pub monthly_rent_cents: u64,
    pub fair_market_rent_cents: u64,
    pub federally_state_or_locally_subsidized: bool,
    pub tenant_has_written_lease: bool,
    pub lease_remaining_months: u32,
    pub new_owner_will_occupy_as_primary_residence: bool,
    pub vacate_notice_days_given: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub bona_fide_tenant: bool,
    pub protected: bool,
    pub days_tenant_can_remain: u32,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type PtfaInput = Input;
pub type PtfaOutput = Output;
pub type PtfaResult = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "Pub. L. 111-22 Title VII §§ 701-704 (PTFA original enactment, May 20, 2009)".to_string(),
        "Pub. L. 115-174 § 304 (PTFA permanent reinstatement, June 23, 2018)".to_string(),
        "PTFA § 702(a) (90-day vacate notice + bona fide lease through remaining term)".to_string(),
        "PTFA § 702(b) (bona fide tenant three-prong definition)".to_string(),
        "12 U.S.C. § 5220 note (PTFA codification reference)".to_string(),
        "HUD guidance on 'substantially less than fair market rent' (80% threshold)".to_string(),
        "Bank of Am., N.A. v. Owens, 28 Misc. 3d 328 (N.Y. Sup. Ct. 2010)".to_string(),
    ];

    if !input.successor_acquired_at_foreclosure {
        notes.push("Property not acquired by successor at foreclosure sale — PTFA does not apply.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            bona_fide_tenant: false,
            protected: false,
            days_tenant_can_remain: 0,
            notes,
            citations,
        };
    }

    if input.tenant_is_mortgagor_or_immediate_family {
        notes.push("Tenant is the mortgagor or spouse/parent/child of mortgagor — fails PTFA § 702(b)(1) bona fide tenant prong; no PTFA protection.".to_string());
        return Output {
            severity: Severity::NonBonaFideTenantNoProtectionMortgagorOrImmediateFamily,
            bona_fide_tenant: false,
            protected: false,
            days_tenant_can_remain: 0,
            notes,
            citations,
        };
    }

    if !input.lease_was_arms_length {
        notes.push("Lease was not the result of an arm's-length transaction — fails PTFA § 702(b)(2); no PTFA protection.".to_string());
        return Output {
            severity: Severity::NonBonaFideTenantNoProtectionNonArmsLengthTransaction,
            bona_fide_tenant: false,
            protected: false,
            days_tenant_can_remain: 0,
            notes,
            citations,
        };
    }

    let rent_pct_of_fmr = input
        .monthly_rent_cents
        .saturating_mul(100)
        .checked_div(input.fair_market_rent_cents)
        .unwrap_or(100);
    let substantially_less_than_fmr =
        rent_pct_of_fmr < SUBSTANTIALLY_LESS_THAN_FMR_THRESHOLD_PERCENT;

    if substantially_less_than_fmr && !input.federally_state_or_locally_subsidized {
        notes.push(format!(
            "Rent {}% of FMR is below {}% threshold and not subsidized — fails PTFA § 702(b)(3); no PTFA protection.",
            rent_pct_of_fmr,
            SUBSTANTIALLY_LESS_THAN_FMR_THRESHOLD_PERCENT
        ));
        return Output {
            severity: Severity::NonBonaFideTenantNoProtectionBelowMarketRentNotSubsidized,
            bona_fide_tenant: false,
            protected: false,
            days_tenant_can_remain: 0,
            notes,
            citations,
        };
    }

    if input.vacate_notice_days_given > 0
        && input.vacate_notice_days_given < PTFA_MINIMUM_VACATE_NOTICE_DAYS
    {
        notes.push(format!(
            "Notice of {} days served — fails PTFA § 702(a)(2)(A) {}-day minimum. Violation: notice is void.",
            input.vacate_notice_days_given,
            PTFA_MINIMUM_VACATE_NOTICE_DAYS
        ));
        return Output {
            severity: Severity::ViolationLessThan90DayNoticeServed,
            bona_fide_tenant: true,
            protected: true,
            days_tenant_can_remain: PTFA_MINIMUM_VACATE_NOTICE_DAYS,
            notes,
            citations,
        };
    }

    if input.tenant_has_written_lease && input.lease_remaining_months > 0 {
        if input.new_owner_will_occupy_as_primary_residence {
            notes.push(format!(
                "Owner-occupy exception under PTFA § 702(a)(2)(A): sale to primary-residence purchaser terminates bona fide lease early, but tenant still entitled to {}-day vacate notice.",
                PTFA_MINIMUM_VACATE_NOTICE_DAYS
            ));
            return Output {
                severity: Severity::BonaFideLeaseEarlyTermByOwnerOccupyExceptionStill90DayNotice,
                bona_fide_tenant: true,
                protected: true,
                days_tenant_can_remain: PTFA_MINIMUM_VACATE_NOTICE_DAYS,
                notes,
                citations,
            };
        }
        let remaining_days = input.lease_remaining_months.saturating_mul(30);
        notes.push(format!(
            "Bona fide lease — successor takes title subject to lease; tenant may remain {} months ({} days) through remaining term.",
            input.lease_remaining_months,
            remaining_days
        ));
        return Output {
            severity: Severity::BonaFideLeaseAllowedFullTermRemaining,
            bona_fide_tenant: true,
            protected: true,
            days_tenant_can_remain: remaining_days.max(PTFA_MINIMUM_VACATE_NOTICE_DAYS),
            notes,
            citations,
        };
    }

    notes.push(format!(
        "Bona fide month-to-month tenancy (or no written lease) — must serve {}-day written vacate notice before requiring tenant to vacate.",
        PTFA_MINIMUM_VACATE_NOTICE_DAYS
    ));
    Output {
        severity: Severity::BonaFideMonthToMonthMustGive90DayVacateNotice,
        bona_fide_tenant: true,
        protected: true,
        days_tenant_can_remain: PTFA_MINIMUM_VACATE_NOTICE_DAYS,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_bona_fide_lease() -> Input {
        Input {
            successor_acquired_at_foreclosure: true,
            tenant_is_mortgagor_or_immediate_family: false,
            lease_was_arms_length: true,
            monthly_rent_cents: 200_000,
            fair_market_rent_cents: 200_000,
            federally_state_or_locally_subsidized: false,
            tenant_has_written_lease: true,
            lease_remaining_months: 6,
            new_owner_will_occupy_as_primary_residence: false,
            vacate_notice_days_given: 0,
        }
    }

    #[test]
    fn bona_fide_lease_protected_through_remaining_term() {
        let out = check(&base_bona_fide_lease());
        assert_eq!(
            out.severity,
            Severity::BonaFideLeaseAllowedFullTermRemaining
        );
        assert!(out.bona_fide_tenant);
        assert!(out.protected);
        assert_eq!(out.days_tenant_can_remain, 180);
    }

    #[test]
    fn bona_fide_month_to_month_gets_90_day_notice() {
        let mut i = base_bona_fide_lease();
        i.tenant_has_written_lease = false;
        i.lease_remaining_months = 0;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::BonaFideMonthToMonthMustGive90DayVacateNotice
        );
        assert_eq!(out.days_tenant_can_remain, 90);
    }

    #[test]
    fn owner_occupy_exception_terminates_lease_but_still_90_day_notice() {
        let mut i = base_bona_fide_lease();
        i.new_owner_will_occupy_as_primary_residence = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::BonaFideLeaseEarlyTermByOwnerOccupyExceptionStill90DayNotice
        );
        assert_eq!(out.days_tenant_can_remain, 90);
    }

    #[test]
    fn mortgagor_or_family_not_protected() {
        let mut i = base_bona_fide_lease();
        i.tenant_is_mortgagor_or_immediate_family = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::NonBonaFideTenantNoProtectionMortgagorOrImmediateFamily
        );
        assert!(!out.protected);
        assert_eq!(out.days_tenant_can_remain, 0);
    }

    #[test]
    fn non_arms_length_lease_not_protected() {
        let mut i = base_bona_fide_lease();
        i.lease_was_arms_length = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::NonBonaFideTenantNoProtectionNonArmsLengthTransaction
        );
    }

    #[test]
    fn below_80_percent_fmr_not_subsidized_not_protected() {
        let mut i = base_bona_fide_lease();
        i.monthly_rent_cents = 100_000;
        i.fair_market_rent_cents = 200_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::NonBonaFideTenantNoProtectionBelowMarketRentNotSubsidized
        );
    }

    #[test]
    fn below_80_percent_fmr_section_8_subsidized_still_protected() {
        let mut i = base_bona_fide_lease();
        i.monthly_rent_cents = 50_000;
        i.fair_market_rent_cents = 200_000;
        i.federally_state_or_locally_subsidized = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::BonaFideLeaseAllowedFullTermRemaining
        );
    }

    #[test]
    fn fmr_threshold_boundary_80_percent_is_protected() {
        let mut i = base_bona_fide_lease();
        i.monthly_rent_cents = 160_000;
        i.fair_market_rent_cents = 200_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::BonaFideLeaseAllowedFullTermRemaining
        );
    }

    #[test]
    fn fmr_threshold_boundary_79_percent_not_protected() {
        let mut i = base_bona_fide_lease();
        i.monthly_rent_cents = 158_000;
        i.fair_market_rent_cents = 200_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::NonBonaFideTenantNoProtectionBelowMarketRentNotSubsidized
        );
    }

    #[test]
    fn not_acquired_at_foreclosure_ptfa_not_applicable() {
        let mut i = base_bona_fide_lease();
        i.successor_acquired_at_foreclosure = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn less_than_90_day_notice_is_violation() {
        let mut i = base_bona_fide_lease();
        i.tenant_has_written_lease = false;
        i.lease_remaining_months = 0;
        i.vacate_notice_days_given = 30;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationLessThan90DayNoticeServed);
        assert_eq!(out.days_tenant_can_remain, 90);
        assert!(out.notes.iter().any(|n| n.contains("30 days served")));
    }

    #[test]
    fn exactly_90_day_notice_satisfies_minimum() {
        let mut i = base_bona_fide_lease();
        i.tenant_has_written_lease = false;
        i.lease_remaining_months = 0;
        i.vacate_notice_days_given = 90;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::BonaFideMonthToMonthMustGive90DayVacateNotice
        );
    }

    #[test]
    fn citations_pin_pub_l_111_22_and_pub_l_115_174_permanent_2018() {
        let out = check(&base_bona_fide_lease());
        assert!(out.citations.iter().any(|c| c.contains("Pub. L. 111-22")));
        assert!(out.citations.iter().any(|c| c.contains("Pub. L. 115-174")));
        assert!(out.citations.iter().any(|c| c.contains("June 23, 2018")));
    }

    #[test]
    fn citations_pin_section_702_subsections() {
        let out = check(&base_bona_fide_lease());
        assert!(out.citations.iter().any(|c| c.contains("§ 702(a)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 702(b)")));
    }

    #[test]
    fn citations_pin_bank_of_america_v_owens_case_law() {
        let out = check(&base_bona_fide_lease());
        assert!(out.citations.iter().any(|c| c.contains("Bank of Am., N.A. v. Owens")));
    }

    #[test]
    fn constant_pin_90_day_vacate_minimum() {
        assert_eq!(PTFA_MINIMUM_VACATE_NOTICE_DAYS, 90);
    }

    #[test]
    fn constant_pin_80_percent_fmr_threshold() {
        assert_eq!(SUBSTANTIALLY_LESS_THAN_FMR_THRESHOLD_PERCENT, 80);
    }

    #[test]
    fn constant_pin_2018_permanent_reinstatement_year() {
        assert_eq!(PTFA_PERMANENT_REINSTATEMENT_YEAR, 2018);
    }

    #[test]
    fn long_lease_remaining_term_capped_at_remaining_days() {
        let mut i = base_bona_fide_lease();
        i.lease_remaining_months = 36;
        let out = check(&i);
        assert_eq!(out.days_tenant_can_remain, 1080);
    }

    #[test]
    fn very_short_lease_remaining_falls_to_90_day_floor() {
        let mut i = base_bona_fide_lease();
        i.lease_remaining_months = 1;
        let out = check(&i);
        assert_eq!(out.days_tenant_can_remain, 90);
    }

    #[test]
    fn zero_fmr_assumed_at_market_protected() {
        let mut i = base_bona_fide_lease();
        i.fair_market_rent_cents = 0;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::BonaFideLeaseAllowedFullTermRemaining
        );
    }

    #[test]
    fn very_large_rent_saturating_no_overflow() {
        let mut i = base_bona_fide_lease();
        i.monthly_rent_cents = u64::MAX;
        i.fair_market_rent_cents = 1;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::BonaFideLeaseAllowedFullTermRemaining
        );
    }
}
