//! Federal Protecting Tenants at Foreclosure Act (PTFA) compliance +
//! state additions for tenants in foreclosed rental properties.
//!
//! **Federal floor (universal)** — PTFA was enacted in 2009 as Title VII
//! of the Helping Families Save Their Homes Act, sunset in 2014, then
//! **permanently reinstated** in 2018 by § 304 of the Economic Growth,
//! Regulatory Relief, and Consumer Protection Act (EGRRCPA). Two core
//! protections:
//!
//! 1. **90-day notice minimum** — all bona fide tenants must receive at
//!    least 90 days written notice before being required to vacate
//!    after a foreclosure.
//! 2. **Lease honor through expiration** — if the tenant has more than
//!    90 days remaining on a bona fide lease entered into before the
//!    notice of foreclosure, the tenant may stay through lease end.
//!    Exception: if the foreclosure purchaser will occupy the unit as
//!    a primary residence, the 90-day notice (not lease term) applies.
//!
//! **Bona fide tenancy test** has three prongs (ALL three required):
//!
//!  (a) Tenant is NOT the mortgagor or the spouse, parent, or child of
//!      the mortgagor.
//!  (b) The lease/tenancy was the result of an arm's length transaction.
//!  (c) The lease/tenancy requires rent that is not substantially below
//!      fair market rent (or rent is reduced/subsidized by a federal,
//!      state, or local subsidy).
//!
//! Failing any prong → PTFA protections do NOT apply; tenant is
//! evictable per normal state foreclosure law.
//!
//! **State additions** layered atop the federal floor (federal is a
//! floor, not a ceiling — states may exceed):
//!
//! - **CA** SB 1079 (2020), expanded by AB 1837 (2022) — 90-day notice
//!   floor PLUS tenant's **right of first refusal** to purchase the
//!   property at the trustee's sale, up to 45 days post-auction.
//! - **MA** c. 186A — Tenant Protections in Foreclosed Properties Act,
//!   one of the strongest state regimes; tenant may continue past
//!   federal 90 days at landlord's election.
//! - **NJ** § 2A:50-69 — long-standing tenant protection during
//!   foreclosure with extended notice.
//! - **DC** § 42-3505.01a — 120-day notice extension (above federal 90).
//! - **NY**, **MD**, **IL** — comprehensive state regimes with various
//!   extensions.

use chrono::NaiveDate;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ForeclosureTenantRegime {
    /// Federal PTFA floor only — 90-day notice, no state additions.
    FederalFloorOnly,
    /// State extends federal notice period beyond 90 days.
    ExtendedNoticePeriod,
    /// State adds right-of-first-refusal layer on top of federal floor.
    RightOfFirstRefusal,
    /// Comprehensive state regime combining extended notice + ROFR +
    /// other protections (CA, MA, NJ comprehensive).
    ComprehensiveStateProtections,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateForeclosureTenantRule {
    pub state_code: &'static str,
    pub state_name: &'static str,
    pub regime: ForeclosureTenantRegime,
    /// Notice period required to vacate post-foreclosure (days). Always
    /// ≥ 90 (federal floor).
    pub notice_period_days: u32,
    /// True if state law gives tenant a right of first refusal to
    /// purchase the property (CA SB 1079).
    pub right_of_first_refusal: bool,
    /// True if state honors the federal exception allowing purchaser
    /// owner-occupants to give 90-day notice instead of completing lease
    /// term. (Most states honor this; some pro-tenant states limit it.)
    pub purchaser_owner_occupant_exemption_honored: bool,
    pub citation: &'static str,
}

/// Federal PTFA notice-period floor in days.
const FEDERAL_NOTICE_FLOOR_DAYS: u32 = 90;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeclosureTenantInput {
    pub state_code: String,
    pub notice_received_date: NaiveDate,
    /// Deadline by which the tenant is required to vacate (per the
    /// notice).
    pub vacate_deadline: NaiveDate,
    pub lease_expiration_date: NaiveDate,
    /// True if tenant is the mortgagor OR the spouse, parent, or child
    /// of the mortgagor. Disqualifies bona fide tenancy under PTFA.
    pub tenant_is_mortgagor_or_family: bool,
    pub lease_arm_length_transaction: bool,
    /// True if rent is at or above fair market rent OR is reduced/
    /// subsidized by a federal, state, or local program (HCV, LIHTC,
    /// public housing).
    pub rent_at_or_above_fmr_or_subsidized: bool,
    /// True if the foreclosure purchaser will occupy the property as a
    /// primary residence. Triggers the federal owner-occupant exception
    /// (90-day notice replaces lease completion).
    pub purchaser_will_occupy_as_primary_residence: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeclosureTenantResult {
    /// True if all three bona-fide-tenancy prongs are satisfied.
    pub bona_fide_tenant: bool,
    /// Bona-fide test prongs broken out so callers can show exactly
    /// which prong failed.
    pub bona_fide_prong_not_mortgagor_family: bool,
    pub bona_fide_prong_arm_length: bool,
    pub bona_fide_prong_fair_market_rent: bool,
    pub notice_period_required_days: u32,
    pub actual_notice_days: i64,
    pub complies_with_notice: bool,
    /// True if the tenant may stay through the original lease expiration
    /// instead of being limited to the notice-period vacate date.
    pub tenant_may_complete_lease: bool,
    pub right_of_first_refusal_available: bool,
    pub federal_floor_only: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn lookup(state_code: &str) -> Option<&'static StateForeclosureTenantRule> {
    let up = state_code.to_ascii_uppercase();
    TABLE.get(up.as_str()).copied()
}

pub fn all_states() -> Vec<&'static StateForeclosureTenantRule> {
    let mut v: Vec<&'static StateForeclosureTenantRule> = TABLE.values().copied().collect();
    v.sort_by_key(|r| r.state_code);
    v
}

pub fn check(input: &ForeclosureTenantInput) -> ForeclosureTenantResult {
    let rule = match lookup(&input.state_code) {
        Some(r) => r,
        None => {
            return ForeclosureTenantResult {
                bona_fide_tenant: false,
                bona_fide_prong_not_mortgagor_family: !input.tenant_is_mortgagor_or_family,
                bona_fide_prong_arm_length: input.lease_arm_length_transaction,
                bona_fide_prong_fair_market_rent: input.rent_at_or_above_fmr_or_subsidized,
                notice_period_required_days: FEDERAL_NOTICE_FLOOR_DAYS,
                actual_notice_days: 0,
                complies_with_notice: false,
                tenant_may_complete_lease: false,
                right_of_first_refusal_available: false,
                federal_floor_only: true,
                citation: "n/a",
                note: format!("unknown state code `{}`", input.state_code),
            };
        }
    };

    // Bona fide tenancy three-prong test.
    let prong_a = !input.tenant_is_mortgagor_or_family;
    let prong_b = input.lease_arm_length_transaction;
    let prong_c = input.rent_at_or_above_fmr_or_subsidized;
    let bona_fide = prong_a && prong_b && prong_c;

    let required_days = rule.notice_period_days;
    let actual_notice_days = (input.vacate_deadline - input.notice_received_date).num_days();

    if !bona_fide {
        return ForeclosureTenantResult {
            bona_fide_tenant: false,
            bona_fide_prong_not_mortgagor_family: prong_a,
            bona_fide_prong_arm_length: prong_b,
            bona_fide_prong_fair_market_rent: prong_c,
            notice_period_required_days: required_days,
            actual_notice_days,
            complies_with_notice: false,
            tenant_may_complete_lease: false,
            right_of_first_refusal_available: false,
            federal_floor_only: matches!(rule.regime, ForeclosureTenantRegime::FederalFloorOnly),
            citation: rule.citation,
            note: format!(
                "PTFA protections do NOT apply — tenant fails bona fide test (mortgagor-family: {}, arm's-length: {}, fair-market-rent-or-subsidized: {}); tenant evictable per normal {} foreclosure law",
                if prong_a { "OK" } else { "FAIL" },
                if prong_b { "OK" } else { "FAIL" },
                if prong_c { "OK" } else { "FAIL" },
                rule.state_name
            ),
        };
    }

    let complies = actual_notice_days >= required_days as i64;

    // Tenant may complete lease IF the lease extends beyond the
    // required-notice vacate date AND the purchaser exception doesn't
    // override.
    let lease_extends_beyond_notice = input.lease_expiration_date > input.vacate_deadline;
    let owner_occupant_overrides = input.purchaser_will_occupy_as_primary_residence
        && rule.purchaser_owner_occupant_exemption_honored;
    let may_complete = lease_extends_beyond_notice && !owner_occupant_overrides;

    let note = if complies {
        format!(
            "{}: bona fide tenant; notice {}d ≥ {}d required; tenant {} complete lease",
            rule.state_name,
            actual_notice_days,
            required_days,
            if may_complete { "MAY" } else { "may NOT" }
        )
    } else {
        format!(
            "{}: bona fide tenant; notice {}d < {}d required; landlord {}d short of statutory floor",
            rule.state_name,
            actual_notice_days,
            required_days,
            required_days as i64 - actual_notice_days
        )
    };

    ForeclosureTenantResult {
        bona_fide_tenant: true,
        bona_fide_prong_not_mortgagor_family: prong_a,
        bona_fide_prong_arm_length: prong_b,
        bona_fide_prong_fair_market_rent: prong_c,
        notice_period_required_days: required_days,
        actual_notice_days,
        complies_with_notice: complies,
        tenant_may_complete_lease: may_complete,
        right_of_first_refusal_available: rule.right_of_first_refusal,
        federal_floor_only: matches!(rule.regime, ForeclosureTenantRegime::FederalFloorOnly),
        citation: rule.citation,
        note,
    }
}

const fn rule(
    state_code: &'static str,
    state_name: &'static str,
    regime: ForeclosureTenantRegime,
    notice_period_days: u32,
    right_of_first_refusal: bool,
    purchaser_owner_occupant_exemption_honored: bool,
    citation: &'static str,
) -> StateForeclosureTenantRule {
    StateForeclosureTenantRule {
        state_code,
        state_name,
        regime,
        notice_period_days,
        right_of_first_refusal,
        purchaser_owner_occupant_exemption_honored,
        citation,
    }
}

static TABLE: Lazy<HashMap<&'static str, &'static StateForeclosureTenantRule>> = Lazy::new(|| {
    use ForeclosureTenantRegime::*;
    static RULES: &[StateForeclosureTenantRule] = &[
        rule(
            "AK",
            "Alaska",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "AL",
            "Alabama",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "AR",
            "Arkansas",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "AZ",
            "Arizona",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "CA",
            "California",
            ComprehensiveStateProtections,
            90,
            true,
            true,
            "Cal. Civ. Code §§ 2924.8, 2924m (SB 1079 + AB 1837)",
        ),
        rule(
            "CO",
            "Colorado",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "CT",
            "Connecticut",
            ExtendedNoticePeriod,
            90,
            false,
            true,
            "Conn. Gen. Stat. § 47a-23",
        ),
        rule(
            "DC",
            "District of Columbia",
            ExtendedNoticePeriod,
            120,
            false,
            true,
            "D.C. Code § 42-3505.01a",
        ),
        rule(
            "DE",
            "Delaware",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "FL",
            "Florida",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "GA",
            "Georgia",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "HI",
            "Hawaii",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "IA",
            "Iowa",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "ID",
            "Idaho",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "IL",
            "Illinois",
            ExtendedNoticePeriod,
            90,
            false,
            true,
            "765 ILCS 5/31.5 (Tenant Foreclosure Notice Act)",
        ),
        rule(
            "IN",
            "Indiana",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "KS",
            "Kansas",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "KY",
            "Kentucky",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "LA",
            "Louisiana",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "MA",
            "Massachusetts",
            ComprehensiveStateProtections,
            90,
            false,
            false,
            "M.G.L. c. 186A (Tenant Protections in Foreclosed Properties Act)",
        ),
        rule(
            "MD",
            "Maryland",
            ExtendedNoticePeriod,
            90,
            false,
            true,
            "Md. Code Real Prop. § 7-105.6",
        ),
        rule(
            "ME",
            "Maine",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "MI",
            "Michigan",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "MN",
            "Minnesota",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "MO",
            "Missouri",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "MS",
            "Mississippi",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "MT",
            "Montana",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "NC",
            "North Carolina",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "ND",
            "North Dakota",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "NE",
            "Nebraska",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "NH",
            "New Hampshire",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "NJ",
            "New Jersey",
            ComprehensiveStateProtections,
            90,
            false,
            false,
            "N.J.S.A. § 2A:50-69 (Foreclosure Fairness Act + Anti-Eviction Act)",
        ),
        rule(
            "NM",
            "New Mexico",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "NV",
            "Nevada",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "NY",
            "New York",
            ExtendedNoticePeriod,
            90,
            false,
            true,
            "RPL § 235-f + NY RPAPL § 1305",
        ),
        rule(
            "OH",
            "Ohio",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "OK",
            "Oklahoma",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "OR",
            "Oregon",
            ExtendedNoticePeriod,
            90,
            false,
            true,
            "ORS § 86.782 (foreclosure tenant notice)",
        ),
        rule(
            "PA",
            "Pennsylvania",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "RI",
            "Rhode Island",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "SC",
            "South Carolina",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "SD",
            "South Dakota",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "TN",
            "Tennessee",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "TX",
            "Texas",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "UT",
            "Utah",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "VA",
            "Virginia",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "VT",
            "Vermont",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "WA",
            "Washington",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "WI",
            "Wisconsin",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "WV",
            "West Virginia",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
        rule(
            "WY",
            "Wyoming",
            FederalFloorOnly,
            90,
            false,
            true,
            "PTFA federal floor",
        ),
    ];
    RULES.iter().map(|r| (r.state_code, r)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn bona_fide_input(
        state: &str,
        notice: NaiveDate,
        vacate: NaiveDate,
    ) -> ForeclosureTenantInput {
        ForeclosureTenantInput {
            state_code: state.to_string(),
            notice_received_date: notice,
            vacate_deadline: vacate,
            lease_expiration_date: vacate + chrono::Duration::days(180),
            tenant_is_mortgagor_or_family: false,
            lease_arm_length_transaction: true,
            rent_at_or_above_fmr_or_subsidized: true,
            purchaser_will_occupy_as_primary_residence: false,
        }
    }

    #[test]
    fn table_covers_all_50_states_plus_dc() {
        assert_eq!(TABLE.len(), 51);
    }

    #[test]
    fn federal_floor_90_day_notice_complies_at_exact_boundary() {
        // 90 days exact notice → complies (≥ floor).
        let i = bona_fide_input("TX", d(2026, 1, 1), d(2026, 4, 1));
        let days = (d(2026, 4, 1) - d(2026, 1, 1)).num_days();
        assert_eq!(days, 90);
        let r = check(&i);
        assert!(r.bona_fide_tenant);
        assert!(r.complies_with_notice);
        assert_eq!(r.notice_period_required_days, 90);
        assert!(r.federal_floor_only);
    }

    #[test]
    fn federal_floor_89_day_notice_fails() {
        // 89 days = 1 day short of federal floor.
        let i = bona_fide_input("TX", d(2026, 1, 1), d(2026, 3, 31));
        let r = check(&i);
        assert!(r.bona_fide_tenant);
        assert!(!r.complies_with_notice);
    }

    #[test]
    fn dc_extended_120_day_notice_required() {
        // DC § 42-3505.01a: 120 days minimum. 90-day federal notice
        // doesn't satisfy DC.
        let mut i = bona_fide_input("DC", d(2026, 1, 1), d(2026, 4, 1));
        i.lease_expiration_date = d(2027, 1, 1);
        let r = check(&i);
        assert_eq!(r.notice_period_required_days, 120);
        assert_eq!(r.actual_notice_days, 90);
        assert!(!r.complies_with_notice);
    }

    #[test]
    fn dc_120_day_notice_complies_at_exact_boundary() {
        // 120 days exact → DC complies.
        let mut i = bona_fide_input("DC", d(2026, 1, 1), d(2026, 5, 1));
        i.lease_expiration_date = d(2027, 1, 1);
        let days = (d(2026, 5, 1) - d(2026, 1, 1)).num_days();
        assert_eq!(days, 120);
        let r = check(&i);
        assert!(r.complies_with_notice);
    }

    #[test]
    fn california_right_of_first_refusal_available() {
        // CA SB 1079: bona fide tenant gets ROFR flag set true.
        let i = bona_fide_input("CA", d(2026, 1, 1), d(2026, 4, 1));
        let r = check(&i);
        assert!(r.bona_fide_tenant);
        assert!(r.right_of_first_refusal_available);
        assert!(!r.federal_floor_only);
    }

    #[test]
    fn texas_no_right_of_first_refusal() {
        // TX federal floor only → no ROFR.
        let i = bona_fide_input("TX", d(2026, 1, 1), d(2026, 4, 1));
        let r = check(&i);
        assert!(!r.right_of_first_refusal_available);
    }

    #[test]
    fn bona_fide_test_fails_when_tenant_is_mortgagor_family() {
        // Prong A fail: tenant is mortgagor's child. PTFA doesn't apply.
        let mut i = bona_fide_input("CA", d(2026, 1, 1), d(2026, 4, 1));
        i.tenant_is_mortgagor_or_family = true;
        let r = check(&i);
        assert!(!r.bona_fide_tenant);
        assert!(!r.bona_fide_prong_not_mortgagor_family);
        assert!(r.bona_fide_prong_arm_length);
        assert!(r.bona_fide_prong_fair_market_rent);
        assert!(r.note.contains("mortgagor-family: FAIL"));
    }

    #[test]
    fn bona_fide_test_fails_when_not_arms_length() {
        // Prong B fail: not arm's length (e.g., friend rented from
        // mortgagor at below-market rate as favor).
        let mut i = bona_fide_input("CA", d(2026, 1, 1), d(2026, 4, 1));
        i.lease_arm_length_transaction = false;
        let r = check(&i);
        assert!(!r.bona_fide_tenant);
        assert!(!r.bona_fide_prong_arm_length);
        assert!(r.note.contains("arm's-length: FAIL"));
    }

    #[test]
    fn bona_fide_test_fails_when_rent_below_fmr_unsubsidized() {
        // Prong C fail: rent substantially below FMR with no subsidy.
        let mut i = bona_fide_input("CA", d(2026, 1, 1), d(2026, 4, 1));
        i.rent_at_or_above_fmr_or_subsidized = false;
        let r = check(&i);
        assert!(!r.bona_fide_tenant);
        assert!(!r.bona_fide_prong_fair_market_rent);
        assert!(r.note.contains("fair-market-rent-or-subsidized: FAIL"));
    }

    #[test]
    fn subsidized_rent_satisfies_prong_c() {
        // Section 8 voucher tenant with below-market rent still
        // satisfies prong C because subsidized rent qualifies.
        let i = bona_fide_input("CA", d(2026, 1, 1), d(2026, 4, 1));
        let r = check(&i);
        assert!(r.bona_fide_prong_fair_market_rent);
    }

    #[test]
    fn tenant_may_complete_lease_when_extends_beyond_notice() {
        // Lease ends 6 months after the notice vacate date → tenant may
        // complete lease instead of vacating at notice deadline.
        let i = bona_fide_input("TX", d(2026, 1, 1), d(2026, 4, 1));
        let r = check(&i);
        assert!(r.tenant_may_complete_lease);
    }

    #[test]
    fn purchaser_owner_occupant_exception_overrides_lease_completion() {
        // Federal exception: if purchaser will occupy as primary, tenant
        // is limited to 90 days notice even if lease extends.
        let mut i = bona_fide_input("TX", d(2026, 1, 1), d(2026, 4, 1));
        i.purchaser_will_occupy_as_primary_residence = true;
        let r = check(&i);
        assert!(!r.tenant_may_complete_lease);
    }

    #[test]
    fn massachusetts_does_not_honor_owner_occupant_exception() {
        // MA c. 186A is stronger pro-tenant — does NOT honor the
        // purchaser-owner-occupant exception. Even if purchaser will
        // occupy, tenant may complete lease.
        let mut i = bona_fide_input("MA", d(2026, 1, 1), d(2026, 4, 1));
        i.purchaser_will_occupy_as_primary_residence = true;
        let r = check(&i);
        assert!(r.tenant_may_complete_lease);
    }

    #[test]
    fn new_jersey_does_not_honor_owner_occupant_exception() {
        // NJ Anti-Eviction Act + Foreclosure Fairness Act: also strong
        // pro-tenant, doesn't honor purchaser exception.
        let mut i = bona_fide_input("NJ", d(2026, 1, 1), d(2026, 4, 1));
        i.purchaser_will_occupy_as_primary_residence = true;
        let r = check(&i);
        assert!(r.tenant_may_complete_lease);
    }

    #[test]
    fn lease_already_expired_no_completion_path() {
        // If lease already expired before notice received, tenant has
        // no completion path — only the notice period applies.
        let mut i = bona_fide_input("TX", d(2026, 1, 1), d(2026, 4, 1));
        i.lease_expiration_date = d(2025, 12, 1); // before notice
        let r = check(&i);
        assert!(!r.tenant_may_complete_lease);
    }

    #[test]
    fn unknown_state_handled_with_federal_floor() {
        let i = bona_fide_input("ZZ", d(2026, 1, 1), d(2026, 4, 1));
        let r = check(&i);
        assert!(!r.bona_fide_tenant);
        assert!(r.note.contains("unknown state code"));
        assert_eq!(r.notice_period_required_days, 90);
    }

    #[test]
    fn lookup_case_insensitive() {
        assert!(lookup("CA").is_some());
        assert!(lookup("ca").is_some());
    }

    #[test]
    fn all_states_sorted_by_code() {
        let states = all_states();
        assert_eq!(states.len(), 51);
        assert_eq!(states.first().unwrap().state_code, "AK");
        assert_eq!(states.last().unwrap().state_code, "WY");
    }

    #[test]
    fn citation_present_for_every_row() {
        for r in TABLE.values() {
            assert!(!r.citation.is_empty(), "{} citation empty", r.state_code);
        }
    }

    #[test]
    fn comprehensive_states_pinned() {
        // CA / MA / NJ have ComprehensiveStateProtections.
        for code in ["CA", "MA", "NJ"] {
            let r = lookup(code).unwrap();
            assert!(matches!(
                r.regime,
                ForeclosureTenantRegime::ComprehensiveStateProtections
            ));
        }
    }

    #[test]
    fn dc_120_day_notice_is_the_only_extended_period_state() {
        // DC § 42-3505.01a: 120 days. Everyone else is 90.
        let dc = lookup("DC").unwrap();
        assert_eq!(dc.notice_period_days, 120);
        for r in TABLE.values() {
            if r.state_code != "DC" {
                assert_eq!(
                    r.notice_period_days, 90,
                    "{} should be 90 days",
                    r.state_code
                );
            }
        }
    }

    #[test]
    fn all_three_bona_fide_prongs_required() {
        // If ALL three prongs pass, bona_fide_tenant = true. If ANY
        // single prong fails, bona_fide_tenant = false. Combinatorial
        // sweep across {true, false} for each prong.
        for prong_a in [true, false] {
            for prong_b in [true, false] {
                for prong_c in [true, false] {
                    let mut i = bona_fide_input("CA", d(2026, 1, 1), d(2026, 4, 1));
                    i.tenant_is_mortgagor_or_family = !prong_a;
                    i.lease_arm_length_transaction = prong_b;
                    i.rent_at_or_above_fmr_or_subsidized = prong_c;
                    let r = check(&i);
                    let expected = prong_a && prong_b && prong_c;
                    assert_eq!(
                        r.bona_fide_tenant, expected,
                        "prongs A={prong_a} B={prong_b} C={prong_c} → expected {expected}"
                    );
                }
            }
        }
    }

    #[test]
    fn note_for_compliant_describes_lease_completion_option() {
        let i = bona_fide_input("TX", d(2026, 1, 1), d(2026, 4, 1));
        let r = check(&i);
        assert!(r.note.contains("MAY complete lease"));
    }

    #[test]
    fn note_for_noncompliant_states_shortfall() {
        let i = bona_fide_input("TX", d(2026, 1, 1), d(2026, 3, 1)); // 59 days
        let r = check(&i);
        assert!(r.note.contains("short of statutory floor"));
    }
}
