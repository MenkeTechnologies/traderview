//! State rent-increase notice-period landlord compliance check.
//!
//! Three statewide regimes plus a default. Distinct from
//! `late_payment_grace_period` (which addresses tenant late payments)
//! and `advance_rent_limit` (which caps the AMOUNT collectable in
//! advance) — this module addresses the NOTICE PERIOD a landlord must
//! provide BEFORE the new rent takes effect.
//!
//! **California (Cal. Civ. Code § 1947.12 / AB 1482)** — two-tier:
//!   - **≤ 10% increase**: 30 days notice (Cal. Civ. Code § 827(b)).
//!   - **> 10% increase**: 90 days notice (Cal. Civ. Code § 827(b)(3)).
//!
//! Note: AB 1482's own rent cap (5% + CPI or 10%, whichever is less)
//! makes the > 10% path uncommon for AB-1482-covered units.
//!
//! **Washington (RCW 59.18.140, as amended May 2025)** — uniform
//! **90 days** prior written notice for any rent increase. **Carve-out
//! for subsidized tenancies** (income-based): 30 days. Narrow May-2025
//! grandfather exception: leases with 60-90 days remaining on the term
//! as of 2025-05-07 may use 60-day notice. Local laws (Seattle, Spokane)
//! layer on top with up to 120-180 day requirements. Washington also
//! caps the increase at 7% + CPI or 10%, whichever is less (out of scope
//! of this notice module).
//!
//! **Oregon (ORS 90.323)** — uniform **90 days** written notice. Two
//! additional restrictions: (a) **no rent increase in the first 12 months**
//! of a non-week-to-week tenancy; (b) **only one rent increase per
//! 12-month period** during the tenancy. The notice must state the new
//! rent amount, the effective date, and be delivered according to law.
//!
//! **Default** — no statewide statute; lease terms control. Common-law
//! contract rules apply.
//!
//! Citations: Cal. Civ. Code § 827(b)(1)/(b)(3) (CA notice tiers); Cal.
//! Civ. Code § 1947.12 (AB 1482 rent cap); RCW 59.18.140 (WA 90-day
//! notice, as amended May 2025); RCW 59.18.720 (WA cap); ORS 90.323
//! (OR 90-day notice + first-year prohibition + once-per-12-months).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    CaliforniaAb1482,
    Washington,
    Oregon,
    Default,
}

impl Regime {
    pub fn for_state(state: &str) -> Self {
        match state.trim().to_ascii_uppercase().as_str() {
            "CA" => Self::CaliforniaAb1482,
            "WA" => Self::Washington,
            "OR" => Self::Oregon,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentIncreaseNoticeInput {
    pub regime: Regime,
    /// Percent increase × 100 (so 700 = 7.00%, 1000 = 10.00%). Drives
    /// the CA two-tier threshold.
    pub increase_basis_points: u32,
    pub notice_days_provided: u32,
    /// Months since the current tenancy began. Drives the OR first-year
    /// prohibition.
    pub months_since_tenancy_started: u32,
    /// WA-only carve-out — subsidized tenancies with income-based rent
    /// get 30-day notice instead of 90.
    pub subsidized_tenancy: bool,
    /// OR-only: count of prior rent increases in the preceding rolling
    /// 12-month window. > 0 violates ORS 90.323's once-per-year rule.
    pub prior_increases_in_12_months: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    None,
    InsufficientNotice,
    /// Oregon first-year prohibition (ORS 90.323(3)).
    FirstYearProhibited,
    /// Oregon one-increase-per-12-months rule (ORS 90.323(2)(b)).
    MultipleIncreasesIn12Months,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentIncreaseNoticeResult {
    pub regime: Regime,
    pub required_notice_days: u32,
    pub increase_above_10_percent: bool,
    pub in_first_year: bool,
    pub violation: ViolationType,
    pub landlord_compliant: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn check(input: &RentIncreaseNoticeInput) -> RentIncreaseNoticeResult {
    match input.regime {
        Regime::CaliforniaAb1482 => ca_check(input),
        Regime::Washington => wa_check(input),
        Regime::Oregon => or_check(input),
        Regime::Default => default_check(input),
    }
}

fn ca_check(input: &RentIncreaseNoticeInput) -> RentIncreaseNoticeResult {
    let above_10 = input.increase_basis_points > 1000;
    let required = if above_10 { 90 } else { 30 };
    let citation = if above_10 {
        "Cal. Civ. Code § 827(b)(3) — increase > 10% requires 90 days written notice"
    } else {
        "Cal. Civ. Code § 827(b)(1) — increase ≤ 10% requires 30 days written notice"
    };
    if input.notice_days_provided < required {
        return RentIncreaseNoticeResult {
            regime: Regime::CaliforniaAb1482,
            required_notice_days: required,
            increase_above_10_percent: above_10,
            in_first_year: false,
            violation: ViolationType::InsufficientNotice,
            landlord_compliant: false,
            citation,
            note: format!(
                "Required {} days; provided {} days. Insufficient notice.",
                required, input.notice_days_provided
            ),
        };
    }
    RentIncreaseNoticeResult {
        regime: Regime::CaliforniaAb1482,
        required_notice_days: required,
        increase_above_10_percent: above_10,
        in_first_year: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation,
        note: format!(
            "Notice {} days satisfies the {}-day requirement.",
            input.notice_days_provided, required
        ),
    }
}

fn wa_check(input: &RentIncreaseNoticeInput) -> RentIncreaseNoticeResult {
    let required = if input.subsidized_tenancy { 30 } else { 90 };
    let citation = if input.subsidized_tenancy {
        "RCW 59.18.140 — subsidized tenancy 30-day notice carve-out"
    } else {
        "RCW 59.18.140 — uniform 90-day written notice required (as amended May 2025)"
    };
    if input.notice_days_provided < required {
        return RentIncreaseNoticeResult {
            regime: Regime::Washington,
            required_notice_days: required,
            increase_above_10_percent: input.increase_basis_points > 1000,
            in_first_year: false,
            violation: ViolationType::InsufficientNotice,
            landlord_compliant: false,
            citation,
            note: format!(
                "Required {} days; provided {} days. WA RCW 59.18.140 (as amended May 2025) requires uniform 90-day notice except for subsidized tenancies.",
                required, input.notice_days_provided
            ),
        };
    }
    RentIncreaseNoticeResult {
        regime: Regime::Washington,
        required_notice_days: required,
        increase_above_10_percent: input.increase_basis_points > 1000,
        in_first_year: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation,
        note: format!(
            "Notice {} days satisfies the WA {}-day requirement.",
            input.notice_days_provided, required
        ),
    }
}

fn or_check(input: &RentIncreaseNoticeInput) -> RentIncreaseNoticeResult {
    let in_first_year = input.months_since_tenancy_started < 12;
    if in_first_year {
        return RentIncreaseNoticeResult {
            regime: Regime::Oregon,
            required_notice_days: 90,
            increase_above_10_percent: input.increase_basis_points > 1000,
            in_first_year: true,
            violation: ViolationType::FirstYearProhibited,
            landlord_compliant: false,
            citation:
                "ORS 90.323(3) — no rent increase during the first year of a non-week-to-week tenancy",
            note: format!(
                "Tenancy has been in effect {} months — within the 12-month first-year prohibition window.",
                input.months_since_tenancy_started
            ),
        };
    }
    if input.prior_increases_in_12_months > 0 {
        return RentIncreaseNoticeResult {
            regime: Regime::Oregon,
            required_notice_days: 90,
            increase_above_10_percent: input.increase_basis_points > 1000,
            in_first_year: false,
            violation: ViolationType::MultipleIncreasesIn12Months,
            landlord_compliant: false,
            citation:
                "ORS 90.323(2)(b) — only ONE rent increase per rolling 12-month period",
            note: format!(
                "{} prior increase(s) within the past 12 months — ORS 90.323 limits to one per 12-month period.",
                input.prior_increases_in_12_months
            ),
        };
    }
    if input.notice_days_provided < 90 {
        return RentIncreaseNoticeResult {
            regime: Regime::Oregon,
            required_notice_days: 90,
            increase_above_10_percent: input.increase_basis_points > 1000,
            in_first_year: false,
            violation: ViolationType::InsufficientNotice,
            landlord_compliant: false,
            citation: "ORS 90.323 — 90 days written notice required",
            note: format!(
                "Required 90 days; provided {} days. Insufficient notice.",
                input.notice_days_provided
            ),
        };
    }
    RentIncreaseNoticeResult {
        regime: Regime::Oregon,
        required_notice_days: 90,
        increase_above_10_percent: input.increase_basis_points > 1000,
        in_first_year: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: "ORS 90.323 — 90 days written notice; no increases in first 12 months; one per 12-month period",
        note: format!(
            "Notice {} days satisfies ORS 90.323 90-day requirement; outside first-year window; no prior increase in 12 months.",
            input.notice_days_provided
        ),
    }
}

fn default_check(_input: &RentIncreaseNoticeInput) -> RentIncreaseNoticeResult {
    RentIncreaseNoticeResult {
        regime: Regime::Default,
        required_notice_days: 0,
        increase_above_10_percent: false,
        in_first_year: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: "No statewide rent-increase notice statute identified — lease terms control",
        note: "Default regime: lease terms govern the required notice period for rent increases.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        bps: u32,
        notice_days: u32,
        months_in: u32,
        subsidized: bool,
        prior: u32,
    ) -> RentIncreaseNoticeInput {
        RentIncreaseNoticeInput {
            regime,
            increase_basis_points: bps,
            notice_days_provided: notice_days,
            months_since_tenancy_started: months_in,
            subsidized_tenancy: subsidized,
            prior_increases_in_12_months: prior,
        }
    }

    #[test]
    fn ca_under_10pct_30_day_notice_sufficient() {
        // 5% increase + 30-day notice.
        let r = check(&input(Regime::CaliforniaAb1482, 500, 30, 24, false, 0));
        assert_eq!(r.violation, ViolationType::None);
        assert_eq!(r.required_notice_days, 30);
        assert!(r.landlord_compliant);
        assert!(r.citation.contains("§ 827(b)(1)"));
    }

    #[test]
    fn ca_at_10pct_boundary_still_30_day_notice() {
        // Exactly 10% — strict > 10% required for 90-day tier.
        let r = check(&input(Regime::CaliforniaAb1482, 1000, 30, 24, false, 0));
        assert_eq!(r.required_notice_days, 30);
        assert!(!r.increase_above_10_percent);
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn ca_one_bp_above_10pct_jumps_to_90_day() {
        let r = check(&input(Regime::CaliforniaAb1482, 1001, 30, 24, false, 0));
        assert_eq!(r.required_notice_days, 90);
        assert!(r.increase_above_10_percent);
        assert_eq!(r.violation, ViolationType::InsufficientNotice);
        assert!(r.citation.contains("§ 827(b)(3)"));
    }

    #[test]
    fn ca_over_10pct_with_90_day_notice_compliant() {
        let r = check(&input(Regime::CaliforniaAb1482, 1500, 90, 24, false, 0));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ca_under_10pct_with_only_29_day_notice_insufficient() {
        let r = check(&input(Regime::CaliforniaAb1482, 500, 29, 24, false, 0));
        assert_eq!(r.violation, ViolationType::InsufficientNotice);
        assert!(r.note.contains("Required 30 days; provided 29 days"));
    }

    #[test]
    fn wa_uniform_90_day_notice() {
        let r = check(&input(Regime::Washington, 500, 90, 24, false, 0));
        assert_eq!(r.required_notice_days, 90);
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.citation.contains("amended May 2025"));
    }

    #[test]
    fn wa_subsidized_carve_out_30_days() {
        let r = check(&input(Regime::Washington, 500, 30, 24, true, 0));
        assert_eq!(r.required_notice_days, 30);
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.citation.contains("subsidized"));
    }

    #[test]
    fn wa_subsidized_29_days_insufficient() {
        let r = check(&input(Regime::Washington, 500, 29, 24, true, 0));
        assert_eq!(r.violation, ViolationType::InsufficientNotice);
    }

    #[test]
    fn wa_non_subsidized_60_days_insufficient_under_amended_law() {
        // Pre-amendment 60-day notice no longer suffices for non-subsidized
        // tenancies under the May 2025 amendment.
        let r = check(&input(Regime::Washington, 500, 60, 24, false, 0));
        assert_eq!(r.violation, ViolationType::InsufficientNotice);
        assert!(r.note.contains("uniform 90-day notice"));
    }

    #[test]
    fn or_first_year_prohibition_violation() {
        // 11 months in → first-year window applies, no increase permitted.
        let r = check(&input(Regime::Oregon, 500, 90, 11, false, 0));
        assert_eq!(r.violation, ViolationType::FirstYearProhibited);
        assert!(r.in_first_year);
        assert!(r.citation.contains("ORS 90.323(3)"));
    }

    #[test]
    fn or_at_12_month_boundary_no_longer_first_year() {
        let r = check(&input(Regime::Oregon, 500, 90, 12, false, 0));
        assert!(!r.in_first_year);
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn or_second_increase_in_12_months_violation() {
        let r = check(&input(Regime::Oregon, 500, 90, 24, false, 1));
        assert_eq!(r.violation, ViolationType::MultipleIncreasesIn12Months);
        assert!(r.citation.contains("ORS 90.323(2)(b)"));
    }

    #[test]
    fn or_90_day_notice_compliant() {
        let r = check(&input(Regime::Oregon, 500, 90, 24, false, 0));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn or_89_day_notice_insufficient() {
        let r = check(&input(Regime::Oregon, 500, 89, 24, false, 0));
        assert_eq!(r.violation, ViolationType::InsufficientNotice);
    }

    #[test]
    fn or_first_year_takes_precedence_over_notice_check() {
        // 11 months in but 90 days notice given — first-year prohibition
        // fires first.
        let r = check(&input(Regime::Oregon, 500, 90, 11, false, 0));
        assert_eq!(r.violation, ViolationType::FirstYearProhibited);
    }

    #[test]
    fn or_multiple_increase_check_after_first_year() {
        // After first year but prior increase exists.
        let r = check(&input(Regime::Oregon, 500, 90, 18, false, 2));
        assert_eq!(r.violation, ViolationType::MultipleIncreasesIn12Months);
    }

    #[test]
    fn default_no_violation_any_inputs() {
        let r = check(&input(Regime::Default, 1500, 0, 0, false, 5));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.landlord_compliant);
        assert!(r.citation.contains("lease terms control"));
    }

    #[test]
    fn state_routing_ca_wa_or_default() {
        assert_eq!(Regime::for_state("CA"), Regime::CaliforniaAb1482);
        assert_eq!(Regime::for_state("WA"), Regime::Washington);
        assert_eq!(Regime::for_state("OR"), Regime::Oregon);
        assert_eq!(Regime::for_state("TX"), Regime::Default);
        assert_eq!(Regime::for_state("NY"), Regime::Default);
    }

    #[test]
    fn state_routing_case_insensitive() {
        assert_eq!(Regime::for_state("ca"), Regime::CaliforniaAb1482);
        assert_eq!(Regime::for_state("Wa"), Regime::Washington);
        assert_eq!(Regime::for_state("or"), Regime::Oregon);
    }

    #[test]
    fn only_or_has_first_year_prohibition() {
        // Same 11-months-in input across all regimes — only OR triggers.
        let ca = check(&input(Regime::CaliforniaAb1482, 500, 30, 11, false, 0));
        let wa = check(&input(Regime::Washington, 500, 90, 11, false, 0));
        let or = check(&input(Regime::Oregon, 500, 90, 11, false, 0));
        assert_ne!(ca.violation, ViolationType::FirstYearProhibited);
        assert_ne!(wa.violation, ViolationType::FirstYearProhibited);
        assert_eq!(or.violation, ViolationType::FirstYearProhibited);
    }

    #[test]
    fn only_or_has_multiple_increase_rule() {
        let ca = check(&input(Regime::CaliforniaAb1482, 500, 30, 24, false, 3));
        let wa = check(&input(Regime::Washington, 500, 90, 24, false, 3));
        let or = check(&input(Regime::Oregon, 500, 90, 24, false, 3));
        assert_ne!(ca.violation, ViolationType::MultipleIncreasesIn12Months);
        assert_ne!(wa.violation, ViolationType::MultipleIncreasesIn12Months);
        assert_eq!(or.violation, ViolationType::MultipleIncreasesIn12Months);
    }

    #[test]
    fn only_ca_has_two_tier_notice() {
        // 15% increase: CA needs 90 days; WA needs 90 (uniform); OR needs 90.
        let ca_short = check(&input(Regime::CaliforniaAb1482, 1500, 30, 24, false, 0));
        let ca_full = check(&input(Regime::CaliforniaAb1482, 1500, 90, 24, false, 0));
        assert_eq!(ca_short.violation, ViolationType::InsufficientNotice);
        assert_eq!(ca_full.violation, ViolationType::None);
        // 5% increase: CA needs 30 days only.
        let ca_low_30 = check(&input(Regime::CaliforniaAb1482, 500, 30, 24, false, 0));
        assert_eq!(ca_low_30.required_notice_days, 30);
        // WA still needs 90 for 5% increase (no tiered rule).
        let wa_low_30 = check(&input(Regime::Washington, 500, 30, 24, false, 0));
        assert_eq!(wa_low_30.required_notice_days, 90);
        assert_eq!(wa_low_30.violation, ViolationType::InsufficientNotice);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let ca = check(&input(Regime::CaliforniaAb1482, 500, 30, 24, false, 0));
        assert!(ca.citation.contains("§ 827(b)(1)"));

        let ca_high = check(&input(Regime::CaliforniaAb1482, 1500, 90, 24, false, 0));
        assert!(ca_high.citation.contains("§ 827(b)(3)"));

        let wa = check(&input(Regime::Washington, 500, 90, 24, false, 0));
        assert!(wa.citation.contains("RCW 59.18.140"));

        let or = check(&input(Regime::Oregon, 500, 90, 24, false, 0));
        assert!(or.citation.contains("ORS 90.323"));
    }
}
