//! Tenant in foreclosure protections — what notice and timing
//! must a successor in interest provide to existing tenants
//! when a residential property is foreclosed?
//!
//! Trader-critical for landlord-investors purchasing foreclosed
//! properties — buying property with existing bona fide tenants
//! triggers a federal floor plus state-specific overlays that
//! govern when (and how) those tenants may be evicted. Distinct
//! from sibling `foreclosure_tenant_rights` if that module
//! covers a different aspect of foreclosure-tenant interaction.
//!
//! Federal PTFA — Protecting Tenants at Foreclosure Act
//! (Pub. L. 111-22 § 702, restored permanently by Pub. L. 115-174,
//! May 24, 2018): Bona fide tenants receive (a) 90 days' notice
//! before eviction OR (b) the remainder of the lease term,
//! whichever is greater. Owner-occupant primary-residence
//! exception terminates a lease early but still requires 90-day
//! notice. State law preserved when more protective. "Bona fide
//! tenant" requires three conditions: (1) tenant is NOT the
//! mortgagor or mortgagor's spouse, parent, or child; (2) lease
//! is an arm's-length transaction; (3) rent is NOT substantially
//! less than fair market rent (unless reduced/subsidized by
//! federal, state, or local subsidy program).
//!
//! California — Cal. Civ. Code § 2924.8 + § 1161b: § 2924.8
//! requires pre-sale POSTING of foreclosure notice on the
//! property AND first-class mailing addressed to "Resident of
//! property subject to foreclosure sale." 90-day post-foreclosure
//! eviction notice applies. § 2924.8(d) makes it an INFRACTION
//! to tear down the posted notice within 72 hours of posting
//! ($100 fine). § 1161b mirrors federal PTFA for unlawful
//! detainer proceedings after foreclosure.
//!
//! New York — N.Y. RPAPL § 1305: Bona fide tenants entitled to
//! remain for GREATER of 90 days OR lease remainder. Owner-
//! occupant single-unit primary-residence exception. Crucially,
//! § 1305(4) PRESERVES additional rights for (a) tenants
//! subsidized by federal government, (b) tenants subject to rent
//! control or rent stabilization, (c) tenants NOT NAMED in the
//! foreclosure action — these tenants retain ALL pre-foreclosure
//! protections plus § 1305 floor.
//!
//! Default — Federal PTFA only. No additional state-law overlay.
//! Most states follow this baseline.
//!
//! Citations: Pub. L. 111-22 § 702 (Protecting Tenants at
//! Foreclosure Act of 2009); Pub. L. 115-174 § 304 (S.2155 —
//! permanent restoration May 24, 2018 effective June 23, 2018);
//! Cal. Civ. Code § 2924.8 (California pre-sale notice posting +
//! 90-day eviction); Cal. Civ. Code § 2924.8(d) (72-hour tear-
//! down infraction); Cal. Civ. Code § 1161b (parallel unlawful
//! detainer); N.Y. RPAPL § 1305 (greater-of-90-or-lease-end +
//! owner-occupant exception); N.Y. RPAPL § 1305(2) (90-day
//! notice mechanics); N.Y. RPAPL § 1305(4) (additional rights
//! preservation for subsidized + rent-stabilized + non-named
//! tenants). Sibling modules: `foreclosure_tenant_rights`
//! (related but distinct), `mid_tenancy_ownership_change`
//! (general ownership-change rules), `eviction_notices` (post-
//! eviction-notice procedural windows).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    /// Cal. Civ. Code § 2924.8 + § 1161b — adds posting + tear-
    /// down infraction over federal PTFA floor.
    California,
    /// N.Y. RPAPL § 1305 — adds non-named-tenant + rent-control/
    /// stabilization preservation over federal PTFA floor.
    NewYork,
    /// Federal PTFA only — most states.
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    /// True if tenant is the mortgagor or mortgagor's spouse,
    /// parent, or child — disqualifies bona fide tenant status
    /// under PTFA.
    pub tenant_is_mortgagor_or_family: bool,
    /// True if the lease was an arm's-length transaction.
    pub lease_is_arms_length: bool,
    /// True if the lease rent is substantially less than fair
    /// market rent.
    pub rent_is_substantially_less_than_market: bool,
    /// True if the tenancy is subsidized by federal, state, or
    /// local subsidy (rescues the bona fide status even if rent
    /// is substantially below market).
    pub has_government_subsidy: bool,
    /// True if the tenant holds a written lease at the time of
    /// the foreclosure notice.
    pub written_lease_exists: bool,
    /// Remaining days on the lease term at the time of the
    /// foreclosure notice.
    pub lease_remaining_days: i64,
    /// True if foreclosure notice has been issued/served.
    pub foreclosure_notice_provided: bool,
    /// Days elapsed since foreclosure notice was provided.
    pub days_since_foreclosure_notice: i64,
    /// True if the foreclosure buyer intends to occupy the unit
    /// as their primary residence — engages owner-occupant
    /// exception.
    pub buyer_intends_owner_occupancy: bool,
    /// California-specific — true if pre-sale notice was posted
    /// on the property AND mailed to "Resident of property
    /// subject to foreclosure sale" per § 2924.8.
    pub ca_pre_sale_notice_posted_and_mailed: bool,
    /// California-specific — true if posted notice was torn
    /// down within 72 hours of posting (§ 2924.8(d) infraction).
    pub ca_notice_torn_down_within_72_hours: bool,
    /// New York-specific — true if tenant is subject to rent
    /// control / rent stabilization or has federal subsidy
    /// (preserves additional rights under § 1305(4)).
    pub ny_tenant_subject_to_rent_control_or_subsidy: bool,
    /// New York-specific — true if tenant was NOT named in the
    /// foreclosure action (preserves additional rights under
    /// § 1305(4)).
    pub ny_tenant_not_named_in_foreclosure: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// True if tenant qualifies as bona fide under PTFA.
    pub bona_fide_tenant_status: bool,
    /// Minimum notice period in days required before tenant may
    /// be required to vacate.
    pub minimum_notice_period_days: i64,
    /// True if tenant may remain through the end of the lease
    /// term (rather than just the 90-day notice period).
    pub tenant_may_remain_through_lease_end: bool,
    /// True if the buyer's owner-occupant exception engages
    /// (terminates lease early but 90-day notice still required).
    pub owner_occupant_exception_engaged: bool,
    /// True if New York § 1305(4) additional-rights preservation
    /// engages (rent-controlled/stabilized/subsidized tenant OR
    /// non-named tenant).
    pub ny_additional_rights_preserved: bool,
    /// True if California § 2924.8(d) tear-down infraction
    /// engaged (current $100 fine).
    pub ca_72_hour_tear_down_infraction: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// Federal PTFA — 90-day minimum notice period.
pub const PTFA_MINIMUM_NOTICE_DAYS: i64 = 90;
/// California § 2924.8(d) — 72-hour tear-down protection window.
pub const CA_TEAR_DOWN_PROTECTION_HOURS: i64 = 72;
/// California § 2924.8(d) — $100 fine.
pub const CA_TEAR_DOWN_FINE_CENTS: i64 = 10_000;

pub fn check(input: &Input) -> CheckResult {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    // PTFA bona fide determination.
    let arms_length_ok = input.lease_is_arms_length;
    let no_family_relationship = !input.tenant_is_mortgagor_or_family;
    let rent_ok = !input.rent_is_substantially_less_than_market || input.has_government_subsidy;
    let bona_fide_tenant_status = arms_length_ok && no_family_relationship && rent_ok;

    // PTFA notice period — 90 days or lease end, whichever greater.
    let lease_remaining = input.lease_remaining_days.max(0);
    let minimum_notice_period_days = if bona_fide_tenant_status && input.written_lease_exists {
        PTFA_MINIMUM_NOTICE_DAYS.max(lease_remaining)
    } else if bona_fide_tenant_status {
        PTFA_MINIMUM_NOTICE_DAYS
    } else {
        0
    };

    let tenant_may_remain_through_lease_end = bona_fide_tenant_status
        && input.written_lease_exists
        && lease_remaining > PTFA_MINIMUM_NOTICE_DAYS
        && !input.buyer_intends_owner_occupancy;

    let owner_occupant_exception_engaged =
        input.buyer_intends_owner_occupancy && bona_fide_tenant_status;

    // New York § 1305(4) additional rights preservation.
    let ny_additional_rights_preserved = matches!(input.regime, Regime::NewYork)
        && (input.ny_tenant_subject_to_rent_control_or_subsidy
            || input.ny_tenant_not_named_in_foreclosure);

    // California § 2924.8(d) tear-down infraction.
    let ca_72_hour_tear_down_infraction =
        matches!(input.regime, Regime::California) && input.ca_notice_torn_down_within_72_hours;

    // Violation: California missing pre-sale notice posting.
    if matches!(input.regime, Regime::California)
        && input.foreclosure_notice_provided
        && !input.ca_pre_sale_notice_posted_and_mailed
    {
        violations.push(
            "Cal. Civ. Code § 2924.8 — pre-sale notice MUST be posted on the property AND \
             mailed to 'Resident of property subject to foreclosure sale' by first-class \
             mail. Trustee or authorized agent's failure to post + mail the notice \
             violates § 2924.8(a) procedural prerequisites."
                .to_string(),
        );
    }

    // Violation: eviction attempted before 90-day notice period.
    if bona_fide_tenant_status
        && input.foreclosure_notice_provided
        && input.days_since_foreclosure_notice < PTFA_MINIMUM_NOTICE_DAYS
        && input.days_since_foreclosure_notice >= 0
    {
        violations.push(format!(
            "Federal PTFA (Pub. L. 111-22 § 702) — eviction may not proceed within 90 \
             days of foreclosure notice for bona fide tenants. {} days since notice; {} \
             days remaining in PTFA minimum window.",
            input.days_since_foreclosure_notice,
            PTFA_MINIMUM_NOTICE_DAYS - input.days_since_foreclosure_notice,
        ));
    }

    // Bona fide status notes.
    if bona_fide_tenant_status {
        notes.push(format!(
            "PTFA BONA FIDE TENANT — minimum {} days notice ({}). Lease honored through \
             remainder of term: {}. Owner-occupant exception engaged: {}.",
            minimum_notice_period_days,
            if input.written_lease_exists && lease_remaining > PTFA_MINIMUM_NOTICE_DAYS {
                "greater of 90 days OR remainder of lease term"
            } else {
                "90 days minimum"
            },
            tenant_may_remain_through_lease_end,
            owner_occupant_exception_engaged,
        ));
    } else {
        let missing: Vec<&str> = [
            (no_family_relationship, "NOT mortgagor/family member"),
            (arms_length_ok, "lease is arm's-length transaction"),
            (rent_ok, "rent at fair market OR subsidized"),
        ]
        .iter()
        .filter_map(|(present, label)| if *present { None } else { Some(*label) })
        .collect();
        notes.push(format!(
            "NOT bona fide tenant — failing PTFA prong(s): {}. Federal PTFA protections \
             do not apply; state-law residential tenant protections (eviction notice, \
             cure periods) still apply.",
            missing.join("; "),
        ));
    }

    // Regime-specific notes.
    match input.regime {
        Regime::California => {
            notes.push(
                "Cal. Civ. Code § 2924.8 — pre-sale posting + first-class mailing \
                 requirement layered on PTFA. § 2924.8(d) — INFRACTION ($100 fine) to \
                 tear down posted notice within 72 hours of posting. § 1161b parallels \
                 PTFA for unlawful detainer proceedings."
                    .to_string(),
            );
            if ca_72_hour_tear_down_infraction {
                notes.push(
                    "§ 2924.8(d) infraction TRIGGERED — posted notice torn down within \
                     72 hours of posting. $100 fine applies to the person who tore it \
                     down (not the landlord)."
                        .to_string(),
                );
            }
        }
        Regime::NewYork => {
            notes.push(
                "N.Y. RPAPL § 1305 — greater of 90-day notice OR remainder of lease term. \
                 Owner-occupant single-unit primary-residence exception terminates lease \
                 early. § 1305(4) PRESERVES additional rights for (a) federally subsidized \
                 tenants, (b) rent-control/rent-stabilization tenants, (c) tenants NOT \
                 NAMED in the foreclosure action."
                    .to_string(),
            );
            if ny_additional_rights_preserved {
                notes.push(format!(
                    "§ 1305(4) additional-rights preservation ENGAGED — tenant qualifies \
                     based on: {}. PTFA + § 1305 protections layer ON TOP of pre-existing \
                     rent-control / rent-stabilization / subsidy / non-named-tenant \
                     rights.",
                    if input.ny_tenant_subject_to_rent_control_or_subsidy
                        && input.ny_tenant_not_named_in_foreclosure
                    {
                        "rent-control/subsidy + not-named-in-foreclosure (both prongs)"
                    } else if input.ny_tenant_subject_to_rent_control_or_subsidy {
                        "rent-control/stabilization/subsidy"
                    } else {
                        "not named in foreclosure action"
                    },
                ));
            }
        }
        Regime::Default => {
            notes.push(
                "Federal PTFA only — Pub. L. 111-22 § 702 (restored permanently by Pub. L. \
                 115-174 § 304 effective June 23, 2018). No state-law overlay in this \
                 regime; PTFA federal floor applies. State law would apply if more \
                 protective."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Sibling modules: `foreclosure_tenant_rights` (related but distinct), \
         `mid_tenancy_ownership_change` (general ownership-change rules), \
         `eviction_notices` (post-eviction-notice procedural windows). Federal PTFA \
         creates a national FLOOR (90-day notice + lease-end honoring for bona fide \
         tenants); state law adds CEILING when more protective. California adds pre-sale \
         posting + 72-hour tear-down infraction; New York adds non-named-tenant + rent-\
         control preservation under § 1305(4)."
            .to_string(),
    );

    let compliant = violations.is_empty();

    CheckResult {
        bona_fide_tenant_status,
        minimum_notice_period_days,
        tenant_may_remain_through_lease_end,
        owner_occupant_exception_engaged,
        ny_additional_rights_preserved,
        ca_72_hour_tear_down_infraction,
        compliant,
        violations,
        citation: "Pub. L. 111-22 § 702 (Protecting Tenants at Foreclosure Act of 2009 — \
                   PTFA); Pub. L. 115-174 § 304 (S.2155 — permanent restoration May 24, \
                   2018 effective June 23, 2018); Cal. Civ. Code § 2924.8 (California \
                   pre-sale notice posting + 90-day eviction); Cal. Civ. Code § 2924.8(a) \
                   (posting + mailing requirements); Cal. Civ. Code § 2924.8(d) (72-hour \
                   tear-down infraction); Cal. Civ. Code § 1161b (parallel unlawful \
                   detainer); N.Y. RPAPL § 1305 (greater-of-90-or-lease-end + owner-\
                   occupant exception); N.Y. RPAPL § 1305(2) (90-day notice mechanics); \
                   N.Y. RPAPL § 1305(4) (additional rights preservation for subsidized + \
                   rent-stabilized + non-named tenants)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(regime: Regime) -> Input {
        Input {
            regime,
            tenant_is_mortgagor_or_family: false,
            lease_is_arms_length: true,
            rent_is_substantially_less_than_market: false,
            has_government_subsidy: false,
            written_lease_exists: true,
            lease_remaining_days: 180,
            foreclosure_notice_provided: true,
            days_since_foreclosure_notice: 30,
            buyer_intends_owner_occupancy: false,
            ca_pre_sale_notice_posted_and_mailed: true,
            ca_notice_torn_down_within_72_hours: false,
            ny_tenant_subject_to_rent_control_or_subsidy: false,
            ny_tenant_not_named_in_foreclosure: false,
        }
    }

    // ── PTFA bona fide tenant qualification ───────────────────

    #[test]
    fn bona_fide_baseline_all_prongs_pass() {
        let r = check(&input(Regime::Default));
        assert!(r.bona_fide_tenant_status);
    }

    #[test]
    fn mortgagor_family_disqualifies() {
        let mut b = input(Regime::Default);
        b.tenant_is_mortgagor_or_family = true;
        let r = check(&b);
        assert!(!r.bona_fide_tenant_status);
    }

    #[test]
    fn non_arms_length_disqualifies() {
        let mut b = input(Regime::Default);
        b.lease_is_arms_length = false;
        let r = check(&b);
        assert!(!r.bona_fide_tenant_status);
    }

    #[test]
    fn rent_substantially_below_market_disqualifies() {
        let mut b = input(Regime::Default);
        b.rent_is_substantially_less_than_market = true;
        b.has_government_subsidy = false;
        let r = check(&b);
        assert!(!r.bona_fide_tenant_status);
    }

    #[test]
    fn subsidy_rescues_below_market_rent() {
        let mut b = input(Regime::Default);
        b.rent_is_substantially_less_than_market = true;
        b.has_government_subsidy = true;
        let r = check(&b);
        assert!(r.bona_fide_tenant_status);
    }

    // ── PTFA 90-day-or-lease-end math ─────────────────────────

    #[test]
    fn bona_fide_with_lease_gets_lease_end() {
        let r = check(&input(Regime::Default));
        // Lease has 180 remaining; > 90; so notice period = 180.
        assert_eq!(r.minimum_notice_period_days, 180);
        assert!(r.tenant_may_remain_through_lease_end);
    }

    #[test]
    fn bona_fide_lease_below_90_days_gets_90() {
        let mut b = input(Regime::Default);
        b.lease_remaining_days = 30;
        let r = check(&b);
        assert_eq!(r.minimum_notice_period_days, 90);
        assert!(!r.tenant_may_remain_through_lease_end);
    }

    #[test]
    fn bona_fide_no_lease_gets_90_days_only() {
        let mut b = input(Regime::Default);
        b.written_lease_exists = false;
        b.lease_remaining_days = 0;
        let r = check(&b);
        assert_eq!(r.minimum_notice_period_days, 90);
    }

    #[test]
    fn non_bona_fide_no_notice_period() {
        let mut b = input(Regime::Default);
        b.tenant_is_mortgagor_or_family = true;
        let r = check(&b);
        assert_eq!(r.minimum_notice_period_days, 0);
    }

    #[test]
    fn owner_occupant_exception_engages() {
        let mut b = input(Regime::Default);
        b.buyer_intends_owner_occupancy = true;
        let r = check(&b);
        assert!(r.owner_occupant_exception_engaged);
        assert!(!r.tenant_may_remain_through_lease_end);
    }

    #[test]
    fn early_eviction_violation_before_90_day_window() {
        let mut b = input(Regime::Default);
        b.days_since_foreclosure_notice = 30;
        let r = check(&b);
        // Still in 90-day window; eviction not yet permitted.
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("PTFA")));
    }

    #[test]
    fn day_90_passed_no_ptfa_window_violation() {
        let mut b = input(Regime::Default);
        b.days_since_foreclosure_notice = 90;
        let r = check(&b);
        // Day 90+: outside PTFA window (the test is < 90).
        assert!(r.compliant);
    }

    // ── California § 2924.8 ───────────────────────────────────

    #[test]
    fn california_pre_sale_notice_posted_compliant() {
        let r = check(&input(Regime::California));
        // PTFA window still active (day 30 < 90), so still violation, but pre-sale notice OK.
        // To get compliant, advance past 90 days.
        let mut b = input(Regime::California);
        b.days_since_foreclosure_notice = 90;
        let r2 = check(&b);
        assert!(r2.compliant);
        // First check: PTFA violation but no pre-sale violation.
        assert!(r.violations.iter().any(|v| v.contains("PTFA")));
        assert!(!r.violations.iter().any(|v| v.contains("§ 2924.8")));
    }

    #[test]
    fn california_pre_sale_notice_missing_violation() {
        let mut b = input(Regime::California);
        b.ca_pre_sale_notice_posted_and_mailed = false;
        b.days_since_foreclosure_notice = 90; // outside PTFA window
        let r = check(&b);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 2924.8")));
    }

    #[test]
    fn california_tear_down_infraction_engages() {
        let mut b = input(Regime::California);
        b.ca_notice_torn_down_within_72_hours = true;
        let r = check(&b);
        assert!(r.ca_72_hour_tear_down_infraction);
    }

    #[test]
    fn california_tear_down_only_applies_in_california() {
        let mut b = input(Regime::NewYork);
        b.ca_notice_torn_down_within_72_hours = true; // ignored outside CA
        let r = check(&b);
        assert!(!r.ca_72_hour_tear_down_infraction);
    }

    // ── New York § 1305(4) ────────────────────────────────────

    #[test]
    fn new_york_rent_control_preserves_additional_rights() {
        let mut b = input(Regime::NewYork);
        b.ny_tenant_subject_to_rent_control_or_subsidy = true;
        let r = check(&b);
        assert!(r.ny_additional_rights_preserved);
    }

    #[test]
    fn new_york_not_named_in_foreclosure_preserves_rights() {
        let mut b = input(Regime::NewYork);
        b.ny_tenant_not_named_in_foreclosure = true;
        let r = check(&b);
        assert!(r.ny_additional_rights_preserved);
    }

    #[test]
    fn new_york_preservation_only_in_ny() {
        let mut b = input(Regime::California);
        b.ny_tenant_subject_to_rent_control_or_subsidy = true; // ignored outside NY
        let r = check(&b);
        assert!(!r.ny_additional_rights_preserved);
    }

    #[test]
    fn new_york_no_preservation_when_neither_condition_met() {
        let r = check(&input(Regime::NewYork));
        assert!(!r.ny_additional_rights_preserved);
    }

    // ── Multi-regime invariants ───────────────────────────────

    #[test]
    fn ptfa_90_day_floor_all_regimes_invariant() {
        // All regimes provide at least 90 days for bona fide tenants.
        for regime in [Regime::California, Regime::NewYork, Regime::Default] {
            let mut b = input(regime);
            b.written_lease_exists = false;
            b.lease_remaining_days = 0;
            let r = check(&b);
            assert_eq!(r.minimum_notice_period_days, 90, "{:?}", regime);
        }
    }

    #[test]
    fn bona_fide_prongs_truth_table() {
        // 8-cell truth table: family/arms-length/rent ok.
        let cells = [
            (false, true, true, true),    // pass all → bona fide
            (true, true, true, false),    // family fails
            (false, false, true, false),  // not arms-length
            (false, true, false, false),  // rent not ok
            (true, true, false, false),   // family + rent fail
            (true, false, true, false),   // family + not arms-length
            (false, false, false, false), // all three fail
            (true, false, false, false),  // all bad
        ];
        for (family, arms_length, rent_ok_flag, expected_bona_fide) in cells.iter() {
            let mut b = input(Regime::Default);
            b.tenant_is_mortgagor_or_family = *family;
            b.lease_is_arms_length = *arms_length;
            b.rent_is_substantially_less_than_market = !*rent_ok_flag;
            let r = check(&b);
            assert_eq!(
                r.bona_fide_tenant_status, *expected_bona_fide,
                "family={} arms={} rent_ok={}",
                family, arms_length, rent_ok_flag,
            );
        }
    }

    #[test]
    fn only_california_has_tear_down_infraction_invariant() {
        for regime in [Regime::California, Regime::NewYork, Regime::Default] {
            let mut b = input(regime);
            b.ca_notice_torn_down_within_72_hours = true;
            let r = check(&b);
            let expected = matches!(regime, Regime::California);
            assert_eq!(r.ca_72_hour_tear_down_infraction, expected, "{:?}", regime);
        }
    }

    #[test]
    fn only_new_york_preserves_additional_rights_invariant() {
        for regime in [Regime::California, Regime::NewYork, Regime::Default] {
            let mut b = input(regime);
            b.ny_tenant_subject_to_rent_control_or_subsidy = true;
            b.ny_tenant_not_named_in_foreclosure = true;
            let r = check(&b);
            let expected = matches!(regime, Regime::NewYork);
            assert_eq!(r.ny_additional_rights_preserved, expected, "{:?}", regime);
        }
    }

    #[test]
    fn citation_pins_all_regime_authorities() {
        let r = check(&input(Regime::Default));
        assert!(r.citation.contains("Pub. L. 111-22 § 702"));
        assert!(r.citation.contains("Pub. L. 115-174 § 304"));
        assert!(r.citation.contains("June 23, 2018"));
        assert!(r.citation.contains("§ 2924.8"));
        assert!(r.citation.contains("§ 2924.8(a)"));
        assert!(r.citation.contains("§ 2924.8(d)"));
        assert!(r.citation.contains("§ 1161b"));
        assert!(r.citation.contains("RPAPL § 1305"));
        assert!(r.citation.contains("§ 1305(2)"));
        assert!(r.citation.contains("§ 1305(4)"));
    }

    #[test]
    fn sibling_distinction_note_present() {
        let r = check(&input(Regime::Default));
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("foreclosure_tenant_rights")
                    && n.contains("mid_tenancy_ownership_change")
                    && n.contains("eviction_notices")
                    && n.contains("national FLOOR")),
            "sibling distinction note must reference related modules + federal floor concept"
        );
    }

    // ── Defensive input clamping ───────────────────────────────

    #[test]
    fn defensive_negative_lease_remaining_clamped() {
        let mut b = input(Regime::Default);
        b.lease_remaining_days = -30;
        let r = check(&b);
        // Negative clamped to 0; falls back to 90-day minimum.
        assert_eq!(r.minimum_notice_period_days, 90);
    }

    #[test]
    fn defensive_negative_days_since_notice_no_violation() {
        let mut b = input(Regime::Default);
        b.days_since_foreclosure_notice = -10;
        let r = check(&b);
        // Negative days = notice not yet served; no PTFA window violation triggered.
        assert!(r.compliant);
    }

    #[test]
    fn ca_tear_down_fine_constant() {
        // $100 fine = 10,000 cents.
        assert_eq!(CA_TEAR_DOWN_FINE_CENTS, 10_000);
        assert_eq!(CA_TEAR_DOWN_PROTECTION_HOURS, 72);
        assert_eq!(PTFA_MINIMUM_NOTICE_DAYS, 90);
    }
}
