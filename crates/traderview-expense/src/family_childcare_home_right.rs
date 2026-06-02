//! Tenant family child-care home (FCCH) operation right —
//! when may a landlord lawfully prohibit a tenant from
//! operating a licensed family day care home for children in
//! a residential rental unit? Trader-landlord critical: in
//! California (and several other states), a lease provision
//! prohibiting family child care home operation is VOID, and
//! a landlord cannot reject a tenant for intending to
//! operate one. Distinct from siblings `tenant_organizing`,
//! `tenant_data_privacy`, and `fair_chance_housing` (which
//! address different non-discrimination domains).
//!
//! **Three regimes**:
//!
//! **California — Cal. Health & Safety Code § 1597.40 (most
//! explicit)**:
//! - § 1597.40(a) — every restriction or prohibition on use
//!   of property for a family day care home for children is
//!   VOID, whether by covenant, condition upon use or
//!   occupancy, or upon transfer of title.
//! - § 1597.40(c) — prospective FCCH provider tenant must
//!   provide **30 days' written notice** to landlord prior
//!   to commencement of operation.
//! - § 1597.40(d) — landlord may require increased security
//!   deposit specifically for FCCH operation (notwithstanding
//!   lower amount required of non-FCCH tenants), but TOTAL
//!   security deposit may NOT exceed maximum allowable
//!   under existing law (CA AB 12 = 1 month's rent for most
//!   landlords post-July 2024).
//! - § 1597.40(e) state preemption — occupies the field to
//!   exclusion of municipal zoning, building, and fire
//!   codes, except as specifically provided in this chapter.
//!
//! **New York — N.Y. Social Services Law § 390 + N.Y. Real
//! Property Law § 235-b**: licensed family day care + group
//! family day care home operation protected; landlord may
//! not unreasonably withhold consent. Limited explicit
//! statutory protection compared to CA.
//!
//! **Default — federal Fair Housing Act, 42 USC § 3604
//! familial-status protection**: protects families with
//! children from refusal to rent, but does NOT extend to
//! childcare-business operation. Local protections may
//! apply.
//!
//! Citations: Cal. Health & Safety Code §§ 1597.40, 1597.41,
//! 1597.42, 1597.43, 1597.44, 1597.46; Cal. Civ. Code §
//! 1950.5 (AB 12 security deposit cap); N.Y. Social Services
//! Law § 390; N.Y. Real Property Law § 235-b; 42 USC § 3604
//! (federal FHA familial status).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FamilyChildcareHomeInput {
    pub regime: Regime,
    pub lease_prohibits_fcch: bool,
    pub days_advance_written_notice_to_landlord: u32,
    pub one_month_rent_cents: i64,
    pub baseline_security_deposit_cents: i64,
    pub increased_fcch_security_deposit_cents: i64,
    /// Whether the FCCH operation is properly licensed under
    /// state law (Title 22 in CA, OCFS in NY, etc.).
    pub fcch_properly_licensed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FamilyChildcareHomeResult {
    pub fcch_operation_permitted: bool,
    pub lease_prohibition_void: bool,
    pub required_notice_days: u32,
    pub notice_satisfied: bool,
    pub max_total_security_deposit_cents: i64,
    pub deposit_cap_violation: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &FamilyChildcareHomeInput) -> FamilyChildcareHomeResult {
    match input.regime {
        Regime::California => check_california(input),
        Regime::NewYork => check_new_york(input),
        Regime::Default => check_default(input),
    }
}

fn check_california(input: &FamilyChildcareHomeInput) -> FamilyChildcareHomeResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Health & Safety Code § 1597.40(a) — every restriction or prohibition on use of property for family day care home for children is VOID, whether by covenant, condition upon use or occupancy, or upon transfer of title"
            .to_string(),
        "Cal. Health & Safety Code § 1597.40(c) — prospective FCCH provider tenant must provide 30 days' written notice to landlord prior to commencement of operation"
            .to_string(),
        "Cal. Health & Safety Code § 1597.40(d) — landlord may require increased security deposit specifically for FCCH operation, capped by Cal. Civ. Code § 1950.5 (AB 12 = 1 month's rent post-July 2024)"
            .to_string(),
        "Cal. Health & Safety Code § 1597.40(e) — state preemption occupies the field to exclusion of municipal zoning, building, and fire codes"
            .to_string(),
    ];

    let rent = input.one_month_rent_cents.max(0);
    let total_deposit = input
        .baseline_security_deposit_cents
        .max(0)
        .saturating_add(input.increased_fcch_security_deposit_cents.max(0));

    let max_deposit = rent;
    let deposit_cap_violation = total_deposit > max_deposit;

    if input.lease_prohibits_fcch {
        violations.push(
            "Cal. Health & Safety Code § 1597.40(a) — lease provision prohibiting family day care home operation is VOID and unenforceable".to_string(),
        );
    }

    let notice_satisfied = input.days_advance_written_notice_to_landlord >= 30;
    if !notice_satisfied {
        violations.push(format!(
            "Cal. Health & Safety Code § 1597.40(c) — prospective FCCH provider tenant gave only {} days' written notice; 30 days required",
            input.days_advance_written_notice_to_landlord
        ));
    }

    if deposit_cap_violation {
        violations.push(format!(
            "Cal. Civ. Code § 1950.5 (AB 12) — total security deposit ${} cents exceeds 1-month rent cap (${} cents) — increased FCCH deposit not excepted from § 1950.5 ceiling",
            total_deposit, max_deposit
        ));
    }

    if !input.fcch_properly_licensed {
        violations.push(
            "Cal. Health & Safety Code § 1597.42 — FCCH operation requires Title 22 license from California Department of Social Services Community Care Licensing".to_string(),
        );
    }

    FamilyChildcareHomeResult {
        fcch_operation_permitted: !input.lease_prohibits_fcch
            && notice_satisfied
            && input.fcch_properly_licensed,
        lease_prohibition_void: input.lease_prohibits_fcch,
        required_notice_days: 30,
        notice_satisfied,
        max_total_security_deposit_cents: max_deposit,
        deposit_cap_violation,
        violations,
        citation: "Cal. Health & Safety Code §§ 1597.40, 1597.41, 1597.42, 1597.43, 1597.44, 1597.46; Cal. Civ. Code § 1950.5",
        notes,
    }
}

fn check_new_york(input: &FamilyChildcareHomeInput) -> FamilyChildcareHomeResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "N.Y. Social Services Law § 390 — licensed family day care and group family day care home operation protected; landlord may not unreasonably withhold consent under N.Y. Real Property Law § 235-b implied warranty of habitability framework"
            .to_string(),
        "NY differs from CA — NY lacks CA's explicit 'void as a matter of law' lease-prohibition provision; instead relies on reasonableness review of landlord consent"
            .to_string(),
    ];

    if input.lease_prohibits_fcch {
        violations.push(
            "N.Y. Social Services Law § 390 — landlord may not unreasonably withhold consent to licensed family day care operation".to_string(),
        );
    }

    if !input.fcch_properly_licensed {
        violations.push(
            "N.Y. Social Services Law § 390 — family day care home operation requires NY State Office of Children and Family Services (OCFS) license".to_string(),
        );
    }

    FamilyChildcareHomeResult {
        fcch_operation_permitted: !input.lease_prohibits_fcch
            && input.fcch_properly_licensed,
        lease_prohibition_void: false,
        required_notice_days: 0,
        notice_satisfied: true,
        max_total_security_deposit_cents: 0,
        deposit_cap_violation: false,
        violations,
        citation: "N.Y. Social Services Law § 390; N.Y. Real Property Law § 235-b",
        notes,
    }
}

fn check_default(input: &FamilyChildcareHomeInput) -> FamilyChildcareHomeResult {
    let notes: Vec<String> = vec![
        "default rule — federal Fair Housing Act 42 USC § 3604 familial-status protection covers families with children from refusal to rent, but does NOT extend to childcare-business operation"
            .to_string(),
        "default rule — local protections may apply (e.g., Cook County IL, Seattle WA, Minneapolis MN); verify jurisdiction-specific ordinances before relying on default rule"
            .to_string(),
    ];

    FamilyChildcareHomeResult {
        fcch_operation_permitted: !input.lease_prohibits_fcch,
        lease_prohibition_void: false,
        required_notice_days: 0,
        notice_satisfied: true,
        max_total_security_deposit_cents: 0,
        deposit_cap_violation: false,
        violations: Vec::new(),
        citation: "42 USC § 3604 (federal FHA familial status baseline)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_compliant() -> FamilyChildcareHomeInput {
        FamilyChildcareHomeInput {
            regime: Regime::California,
            lease_prohibits_fcch: false,
            days_advance_written_notice_to_landlord: 30,
            one_month_rent_cents: 300_000,
            baseline_security_deposit_cents: 200_000,
            increased_fcch_security_deposit_cents: 100_000,
            fcch_properly_licensed: true,
        }
    }

    fn ny_compliant() -> FamilyChildcareHomeInput {
        let mut i = ca_compliant();
        i.regime = Regime::NewYork;
        i
    }

    fn default_base() -> FamilyChildcareHomeInput {
        let mut i = ca_compliant();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn ca_compliant_passes() {
        let r = check(&ca_compliant());
        assert!(r.fcch_operation_permitted);
        assert!(!r.lease_prohibition_void);
        assert_eq!(r.required_notice_days, 30);
        assert!(r.notice_satisfied);
        assert!(!r.deposit_cap_violation);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn ca_lease_prohibition_void() {
        let mut i = ca_compliant();
        i.lease_prohibits_fcch = true;
        let r = check(&i);
        assert!(!r.fcch_operation_permitted);
        assert!(r.lease_prohibition_void);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1597.40(a)") && v.contains("VOID")));
    }

    #[test]
    fn ca_29_day_notice_violates() {
        let mut i = ca_compliant();
        i.days_advance_written_notice_to_landlord = 29;
        let r = check(&i);
        assert!(!r.fcch_operation_permitted);
        assert!(!r.notice_satisfied);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1597.40(c)") && v.contains("29 days")));
    }

    #[test]
    fn ca_30_day_notice_boundary_compliant() {
        let mut i = ca_compliant();
        i.days_advance_written_notice_to_landlord = 30;
        let r = check(&i);
        assert!(r.notice_satisfied);
    }

    #[test]
    fn ca_deposit_at_1_month_boundary_compliant() {
        let mut i = ca_compliant();
        i.baseline_security_deposit_cents = 200_000;
        i.increased_fcch_security_deposit_cents = 100_000;
        i.one_month_rent_cents = 300_000;
        let r = check(&i);
        assert!(!r.deposit_cap_violation);
    }

    #[test]
    fn ca_deposit_over_1_month_violates() {
        let mut i = ca_compliant();
        i.baseline_security_deposit_cents = 200_000;
        i.increased_fcch_security_deposit_cents = 200_000;
        i.one_month_rent_cents = 300_000;
        let r = check(&i);
        assert!(r.deposit_cap_violation);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("AB 12") && v.contains("1-month rent cap")));
    }

    #[test]
    fn ca_unlicensed_fcch_violates() {
        let mut i = ca_compliant();
        i.fcch_properly_licensed = false;
        let r = check(&i);
        assert!(!r.fcch_operation_permitted);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1597.42") && v.contains("Title 22")));
    }

    #[test]
    fn ca_citation_pins_six_health_safety_sections() {
        let r = check(&ca_compliant());
        assert!(r.citation.contains("§§ 1597.40"));
        assert!(r.citation.contains("1597.41"));
        assert!(r.citation.contains("1597.42"));
        assert!(r.citation.contains("1597.43"));
        assert!(r.citation.contains("1597.44"));
        assert!(r.citation.contains("1597.46"));
        assert!(r.citation.contains("§ 1950.5"));
    }

    #[test]
    fn ca_note_pins_state_preemption() {
        let r = check(&ca_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1597.40(e)") && n.contains("state preemption")));
    }

    #[test]
    fn ca_note_pins_30_day_notice() {
        let r = check(&ca_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1597.40(c)") && n.contains("30 days")));
    }

    #[test]
    fn ca_note_pins_ab_12_deposit_cap() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("AB 12") && n.contains("§ 1950.5")));
    }

    #[test]
    fn ny_compliant_passes() {
        let r = check(&ny_compliant());
        assert!(r.fcch_operation_permitted);
        assert_eq!(r.required_notice_days, 0);
    }

    #[test]
    fn ny_lease_prohibition_violates_reasonableness() {
        let mut i = ny_compliant();
        i.lease_prohibits_fcch = true;
        let r = check(&i);
        assert!(!r.fcch_operation_permitted);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 390") && v.contains("unreasonably withhold")));
    }

    #[test]
    fn ny_unlicensed_fcch_violates() {
        let mut i = ny_compliant();
        i.fcch_properly_licensed = false;
        let r = check(&i);
        assert!(!r.fcch_operation_permitted);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 390") && v.contains("OCFS")));
    }

    #[test]
    fn ny_citation_pins_state_law() {
        let r = check(&ny_compliant());
        assert!(r.citation.contains("§ 390"));
        assert!(r.citation.contains("§ 235-b"));
    }

    #[test]
    fn default_no_prohibition_permitted() {
        let r = check(&default_base());
        assert!(r.fcch_operation_permitted);
    }

    #[test]
    fn default_lease_prohibition_blocks_operation() {
        let mut i = default_base();
        i.lease_prohibits_fcch = true;
        let r = check(&i);
        assert!(!r.fcch_operation_permitted);
        assert!(!r.lease_prohibition_void);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn default_citation_pins_fha_baseline() {
        let r = check(&default_base());
        assert!(r.citation.contains("42 USC § 3604"));
    }

    #[test]
    fn three_regimes_routed_correctly() {
        for regime in [Regime::California, Regime::NewYork, Regime::Default] {
            let mut i = ca_compliant();
            i.regime = regime;
            let r = check(&i);
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn ca_uniquely_voids_lease_prohibition_invariant() {
        let mut i_ca = ca_compliant();
        i_ca.lease_prohibits_fcch = true;
        let r_ca = check(&i_ca);
        assert!(r_ca.lease_prohibition_void);

        let mut i_ny = ny_compliant();
        i_ny.lease_prohibits_fcch = true;
        let r_ny = check(&i_ny);
        assert!(!r_ny.lease_prohibition_void);

        let mut i_default = default_base();
        i_default.lease_prohibits_fcch = true;
        let r_default = check(&i_default);
        assert!(!r_default.lease_prohibition_void);
    }

    #[test]
    fn ca_uniquely_requires_30_day_notice_invariant() {
        let r_ca = check(&ca_compliant());
        assert_eq!(r_ca.required_notice_days, 30);

        let r_ny = check(&ny_compliant());
        assert_eq!(r_ny.required_notice_days, 0);

        let r_default = check(&default_base());
        assert_eq!(r_default.required_notice_days, 0);
    }

    #[test]
    fn ca_stacks_4_violations_when_all_4_failures() {
        let mut i = ca_compliant();
        i.lease_prohibits_fcch = true;
        i.days_advance_written_notice_to_landlord = 10;
        i.baseline_security_deposit_cents = 300_000;
        i.increased_fcch_security_deposit_cents = 200_000;
        i.fcch_properly_licensed = false;
        let r = check(&i);
        assert!(!r.fcch_operation_permitted);
        assert_eq!(r.violations.len(), 4);
    }

    #[test]
    fn defensive_negative_rent_clamped() {
        let mut i = ca_compliant();
        i.one_month_rent_cents = -100_000;
        let r = check(&i);
        assert!(r.deposit_cap_violation);
    }

    #[test]
    fn defensive_negative_deposit_clamped() {
        let mut i = ca_compliant();
        i.baseline_security_deposit_cents = -50_000;
        i.increased_fcch_security_deposit_cents = -50_000;
        let r = check(&i);
        assert!(!r.deposit_cap_violation);
    }

    #[test]
    fn ca_60_day_notice_compliant() {
        let mut i = ca_compliant();
        i.days_advance_written_notice_to_landlord = 60;
        let r = check(&i);
        assert!(r.notice_satisfied);
    }

    #[test]
    fn ca_zero_days_notice_violates() {
        let mut i = ca_compliant();
        i.days_advance_written_notice_to_landlord = 0;
        let r = check(&i);
        assert!(!r.notice_satisfied);
    }

    #[test]
    fn ca_increased_deposit_within_cap_does_not_violate() {
        let mut i = ca_compliant();
        i.baseline_security_deposit_cents = 100_000;
        i.increased_fcch_security_deposit_cents = 100_000;
        i.one_month_rent_cents = 300_000;
        let r = check(&i);
        assert!(!r.deposit_cap_violation);
    }

    #[test]
    fn ny_no_deposit_tracking() {
        let r = check(&ny_compliant());
        assert_eq!(r.max_total_security_deposit_cents, 0);
        assert!(!r.deposit_cap_violation);
    }

    #[test]
    fn default_no_deposit_tracking() {
        let r = check(&default_base());
        assert_eq!(r.max_total_security_deposit_cents, 0);
        assert!(!r.deposit_cap_violation);
    }

    #[test]
    fn ca_deposit_cap_uniquely_engaged_invariant() {
        let mut i_ca = ca_compliant();
        i_ca.baseline_security_deposit_cents = 500_000;
        i_ca.increased_fcch_security_deposit_cents = 300_000;
        i_ca.one_month_rent_cents = 300_000;
        let r_ca = check(&i_ca);
        assert!(r_ca.deposit_cap_violation);

        let mut i_ny = ny_compliant();
        i_ny.baseline_security_deposit_cents = 500_000;
        i_ny.increased_fcch_security_deposit_cents = 300_000;
        i_ny.one_month_rent_cents = 300_000;
        let r_ny = check(&i_ny);
        assert!(!r_ny.deposit_cap_violation);
    }

    #[test]
    fn default_note_pins_local_protections() {
        let r = check(&default_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Cook County") && n.contains("Seattle")));
    }

    #[test]
    fn ca_max_total_deposit_equals_one_month_rent() {
        let r = check(&ca_compliant());
        assert_eq!(r.max_total_security_deposit_cents, 300_000);
    }
}
