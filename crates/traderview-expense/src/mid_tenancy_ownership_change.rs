//! State landlord mid-tenancy ownership-change notice + security
//! deposit transfer compliance check.
//!
//! Operational concern for any landlord SELLING or transferring
//! ownership of a tenant-occupied property mid-tenancy. Distinct
//! from `landlord_identification_disclosure` (initial identity
//! disclosure at tenancy start) and `foreclosure_tenant_rights`
//! (foreclosure-driven ownership change with PTFA protections).
//! This module addresses the VOLUNTARY mid-tenancy sale path:
//! successor must notify tenant + transfer security deposit to
//! successor or refund to tenant.
//!
//! Six regimes:
//!
//!   - **California** — Cal. Civ. Code § 1962(c) + § 1950.5(g).
//!     Successor owner must comply with § 1962 identity-disclosure
//!     requirements within 15 DAYS of succession. Failure to
//!     comply BARS the successor from serving an eviction notice
//!     for nonpayment of rent that accrued during noncompliance.
//!     § 1950.5(g) requires the prior landlord to either (a)
//!     transfer remaining security deposit to successor + notify
//!     tenant by personal delivery or first-class mail, OR (b)
//!     refund to tenant. Joint and several liability of successor
//!     and prior landlord for deposit repayment if noncompliant.
//!
//!   - **Massachusetts** — Mass. G.L. c. 186 § 15B(2)(b). Transferor
//!     landlord must transfer security deposit (plus accrued
//!     interest) to transferee within 45 days of transfer. Failure
//!     creates personal liability of transferor.
//!
//!   - **Florida** — Fla. Stat. § 83.49(5). Upon sale/transfer of
//!     dwelling, landlord must EITHER transfer security deposit
//!     (plus accrued interest) to new owner with simultaneous
//!     written notice to tenant of transferee identity, OR refund
//!     to tenant within statutory window. Transferee is liable to
//!     tenant for deposit repayment.
//!
//!   - **Washington** — RCW 59.18.060(2) + RCW 59.18.270. New
//!     owner must furnish updated landlord identification within
//!     reasonable time. Security deposit must be transferred to
//!     successor or refunded to tenant per RCW 59.18.270.
//!
//!   - **NewYork** — N.Y. GOL § 7-105. Transferor must transfer
//!     security deposit to transferee within five days of sale +
//!     give written notice to tenant; failure creates personal
//!     liability of transferor.
//!
//!   - **Default** — most other states require some form of notice
//!     and deposit transfer via state landlord-tenant statute or
//!     common-law successor-liability principles, but timing and
//!     specific procedures vary. Landlord should verify state-
//!     specific deadlines.
//!
//! Citations: Cal. Civ. Code § 1962(c) (15-day successor disclosure);
//! § 1962.5 (successor compliance) + § 1950.5(g)(1)–(2) (deposit
//! transfer or refund); Mass. G.L. c. 186 § 15B(2)(b) (45-day
//! Massachusetts transfer + interest); Fla. Stat. § 83.49(5)
//! (Florida transfer-or-refund + simultaneous notice); RCW
//! 59.18.060(2) + RCW 59.18.270 (Washington); N.Y. GOL § 7-105
//! (5-day NY transfer + tenant notice).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Massachusetts,
    Florida,
    Washington,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    /// Days elapsed between ownership transfer (closing date) and
    /// successor's notice to tenant.
    pub days_since_ownership_transfer: u32,
    /// Whether successor owner has provided § 1962-equivalent
    /// identity disclosure to tenant.
    pub successor_provided_identity_disclosure: bool,
    /// Whether transferor transferred remaining security deposit
    /// to successor (alternative path: refund to tenant).
    pub deposit_transferred_to_successor: bool,
    /// Whether transferor refunded security deposit directly to
    /// tenant (alternative path to deposit transfer).
    pub deposit_refunded_to_tenant: bool,
    /// Whether tenant received written notice of the deposit
    /// transfer (when applicable).
    pub tenant_notified_of_deposit_transfer: bool,
    /// Whether successor or transferor attempts to serve a nonpayment
    /// eviction notice on the tenant for rent that accrued during
    /// the period of successor noncompliance (CA § 1962(c)).
    pub eviction_notice_for_nonpayment_during_noncompliance: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// Statutory deadline in days for successor identity disclosure
    /// (or deposit transfer, where the regime ties them together).
    pub statutory_deadline_days: u32,
    /// True if successor identity disclosure is overdue per the
    /// statutory deadline.
    pub disclosure_deadline_passed: bool,
    /// True if deposit was either transferred to successor OR
    /// refunded to tenant (alternative paths universally accepted).
    pub deposit_disposition_compliant: bool,
    /// True if successor or transferor is barred from serving a
    /// nonpayment eviction notice for rent that accrued during the
    /// noncompliance period (CA § 1962(c) specifically).
    pub nonpayment_eviction_barred: bool,
    /// True if successor + transferor are jointly and severally
    /// liable for deposit repayment under the applicable regime.
    pub joint_and_several_liability: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// California § 1962(c) — 15-day successor compliance deadline.
pub const CA_SUCCESSOR_DEADLINE_DAYS: u32 = 15;
/// Massachusetts G.L. c. 186 § 15B(2)(b) — 45-day deposit transfer
/// deadline.
pub const MA_DEPOSIT_TRANSFER_DEADLINE_DAYS: u32 = 45;
/// New York GOL § 7-105 — 5-day deposit transfer deadline.
pub const NY_DEPOSIT_TRANSFER_DEADLINE_DAYS: u32 = 5;

pub fn check(input: &Input) -> CheckResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    let (statutory_deadline_days, joint_and_several_liability, citation): (
        u32,
        bool,
        &'static str,
    ) = match input.regime {
        Regime::California => (
            CA_SUCCESSOR_DEADLINE_DAYS,
            true,
            "Cal. Civ. Code § 1962(c) (15-day successor identity-disclosure deadline); \
             § 1962.5 (successor compliance — pre-compliance failure bars eviction for \
             nonpayment that accrued during noncompliance); § 1950.5(g)(1)–(2) (security \
             deposit transfer or refund); joint and several liability of successor + prior \
             landlord for noncompliant deposit repayment",
        ),
        Regime::Massachusetts => (
            MA_DEPOSIT_TRANSFER_DEADLINE_DAYS,
            true,
            "Mass. G.L. c. 186 § 15B(2)(b) (45-day Massachusetts deposit transfer to transferee \
             plus accrued interest; transferor personal liability for failure)",
        ),
        Regime::Florida => (
            // FL § 83.49(5) does not specify a fixed day-count;
            // requires "upon sale/transfer" disposition with
            // simultaneous notice. We model as immediate (0 days).
            0,
            true,
            "Fla. Stat. § 83.49(5) (Florida — upon sale/transfer landlord must either transfer \
             deposit to new owner with simultaneous written notice to tenant OR refund to \
             tenant; transferee liable to tenant for deposit repayment)",
        ),
        Regime::Washington => (
            0,
            true,
            "RCW 59.18.060(2) (Washington — new owner must furnish updated landlord \
             identification within reasonable time); RCW 59.18.270 (deposit transfer to \
             successor or refund to tenant)",
        ),
        Regime::NewYork => (
            NY_DEPOSIT_TRANSFER_DEADLINE_DAYS,
            true,
            "N.Y. GOL § 7-105 (5-day NY deposit transfer + written tenant notice; transferor \
             personal liability for failure)",
        ),
        Regime::Default => (
            0,
            false,
            "Default — most other states require some form of notice + deposit transfer; \
             timing and specific procedures vary by state landlord-tenant statute or common-\
             law successor-liability principles",
        ),
    };

    // Disclosure deadline check (CA + MA + NY have explicit
    // day-counts; FL + WA + Default do not).
    let disclosure_deadline_passed = statutory_deadline_days > 0
        && input.days_since_ownership_transfer > statutory_deadline_days;
    if statutory_deadline_days > 0 && !input.successor_provided_identity_disclosure {
        if disclosure_deadline_passed {
            violations.push(format!(
                "{:?} — successor identity disclosure missed: {} days since transfer; \
                 statutory deadline is {} days.",
                input.regime, input.days_since_ownership_transfer, statutory_deadline_days,
            ));
        } else {
            // Within deadline but not yet disclosed — track without
            // violation (no statutory missed yet).
            notes.push(format!(
                "{:?} — successor identity disclosure pending; {} days elapsed of {}-day \
                 statutory window.",
                input.regime, input.days_since_ownership_transfer, statutory_deadline_days,
            ));
        }
    }

    // Deposit disposition compliance — either transfer or refund
    // satisfies. Both false = violation.
    let deposit_disposition_compliant =
        input.deposit_transferred_to_successor || input.deposit_refunded_to_tenant;
    if !deposit_disposition_compliant {
        violations.push(format!(
            "{:?} — security deposit must be either transferred to successor or refunded to \
             tenant; neither has occurred.",
            input.regime,
        ));
    }

    // Tenant notice of deposit transfer (required when transfer
    // path chosen).
    if input.deposit_transferred_to_successor && !input.tenant_notified_of_deposit_transfer {
        violations.push(format!(
            "{:?} — tenant must receive written notice of security-deposit transfer; notice \
             not provided.",
            input.regime,
        ));
    }

    // CA § 1962(c) — nonpayment eviction barred during
    // noncompliance.
    let nonpayment_eviction_barred = matches!(input.regime, Regime::California)
        && (!input.successor_provided_identity_disclosure || disclosure_deadline_passed);
    if matches!(input.regime, Regime::California)
        && input.eviction_notice_for_nonpayment_during_noncompliance
        && nonpayment_eviction_barred
    {
        violations.push(
            "Cal. Civ. Code § 1962(c) — successor served nonpayment eviction notice for rent \
             that accrued during period of noncompliance; eviction BARRED until successor \
             complies with § 1962 identity-disclosure requirements."
                .to_string(),
        );
    }

    notes.push(
        "Distinct from landlord_identification_disclosure (initial identity disclosure at \
         tenancy start) and foreclosure_tenant_rights (foreclosure-driven ownership change \
         with PTFA protections). This module addresses the VOLUNTARY mid-tenancy sale path."
            .to_string(),
    );

    if joint_and_several_liability {
        notes.push(
            "Successor and prior landlord are jointly and severally liable for deposit \
             repayment if successor fails to comply with statutory transfer/notice procedure."
                .to_string(),
        );
    }

    CheckResult {
        statutory_deadline_days,
        disclosure_deadline_passed,
        deposit_disposition_compliant,
        nonpayment_eviction_barred,
        joint_and_several_liability,
        compliant: violations.is_empty(),
        violations,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(regime: Regime) -> Input {
        Input {
            regime,
            days_since_ownership_transfer: 5,
            successor_provided_identity_disclosure: true,
            deposit_transferred_to_successor: true,
            deposit_refunded_to_tenant: false,
            tenant_notified_of_deposit_transfer: true,
            eviction_notice_for_nonpayment_during_noncompliance: false,
        }
    }

    // ── California § 1962(c) + § 1950.5(g) ─────────────────────

    #[test]
    fn california_within_15_days_with_disclosure_compliant() {
        let r = check(&base(Regime::California));
        assert!(r.compliant);
        assert_eq!(r.statutory_deadline_days, 15);
        assert!(!r.disclosure_deadline_passed);
        assert!(r.joint_and_several_liability);
        assert!(r.citation.contains("§ 1962(c)"));
        assert!(r.citation.contains("§ 1950.5(g)"));
    }

    #[test]
    fn california_past_15_day_deadline_no_disclosure_violation() {
        let mut i = base(Regime::California);
        i.days_since_ownership_transfer = 16;
        i.successor_provided_identity_disclosure = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.disclosure_deadline_passed);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("California") && v.contains("16 days")));
    }

    #[test]
    fn california_at_15_day_boundary_compliant() {
        let mut i = base(Regime::California);
        i.days_since_ownership_transfer = 15;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn california_nonpayment_eviction_during_noncompliance_barred() {
        let mut i = base(Regime::California);
        i.successor_provided_identity_disclosure = false;
        i.days_since_ownership_transfer = 20;
        i.eviction_notice_for_nonpayment_during_noncompliance = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.nonpayment_eviction_barred);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1962(c)") && v.contains("BARRED")));
    }

    #[test]
    fn california_refund_to_tenant_alternative_path_compliant() {
        let mut i = base(Regime::California);
        i.deposit_transferred_to_successor = false;
        i.deposit_refunded_to_tenant = true;
        i.tenant_notified_of_deposit_transfer = false; // not needed when refunding
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn california_neither_transfer_nor_refund_violation() {
        let mut i = base(Regime::California);
        i.deposit_transferred_to_successor = false;
        i.deposit_refunded_to_tenant = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("neither has occurred")));
    }

    #[test]
    fn california_transfer_without_tenant_notice_violation() {
        let mut i = base(Regime::California);
        i.deposit_transferred_to_successor = true;
        i.tenant_notified_of_deposit_transfer = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("written notice of security-deposit transfer")));
    }

    // ── Massachusetts G.L. c. 186 § 15B(2)(b) — 45-day ─────────

    #[test]
    fn massachusetts_within_45_days_compliant() {
        let mut i = base(Regime::Massachusetts);
        i.days_since_ownership_transfer = 30;
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.statutory_deadline_days, 45);
        assert!(r.citation.contains("c. 186 § 15B(2)(b)"));
    }

    #[test]
    fn massachusetts_at_45_day_boundary_compliant() {
        let mut i = base(Regime::Massachusetts);
        i.days_since_ownership_transfer = 45;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn massachusetts_past_45_day_deadline_violation() {
        let mut i = base(Regime::Massachusetts);
        i.days_since_ownership_transfer = 46;
        i.successor_provided_identity_disclosure = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.disclosure_deadline_passed);
    }

    // ── Florida Fla. Stat. § 83.49(5) — upon sale ──────────────

    #[test]
    fn florida_upon_sale_with_transfer_compliant() {
        let r = check(&base(Regime::Florida));
        assert!(r.compliant);
        assert_eq!(r.statutory_deadline_days, 0);
        assert!(r.citation.contains("§ 83.49(5)"));
    }

    #[test]
    fn florida_neither_transfer_nor_refund_violation() {
        let mut i = base(Regime::Florida);
        i.deposit_transferred_to_successor = false;
        i.deposit_refunded_to_tenant = false;
        let r = check(&i);
        assert!(!r.compliant);
    }

    // ── Washington RCW 59.18.060(2) + 59.18.270 ────────────────

    #[test]
    fn washington_reasonable_time_compliant() {
        let r = check(&base(Regime::Washington));
        assert!(r.compliant);
        assert!(r.citation.contains("RCW 59.18.060"));
        assert!(r.citation.contains("RCW 59.18.270"));
    }

    // ── New York GOL § 7-105 — 5-day ──────────────────────────

    #[test]
    fn new_york_within_5_days_compliant() {
        let mut i = base(Regime::NewYork);
        i.days_since_ownership_transfer = 3;
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.statutory_deadline_days, 5);
        assert!(r.citation.contains("§ 7-105"));
    }

    #[test]
    fn new_york_at_5_day_boundary_compliant() {
        let mut i = base(Regime::NewYork);
        i.days_since_ownership_transfer = 5;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn new_york_past_5_day_deadline_violation() {
        let mut i = base(Regime::NewYork);
        i.days_since_ownership_transfer = 6;
        i.successor_provided_identity_disclosure = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.disclosure_deadline_passed);
    }

    // ── Default ─────────────────────────────────────────────────

    #[test]
    fn default_no_specific_deadline_but_deposit_disposition_required() {
        let r = check(&base(Regime::Default));
        assert!(r.compliant);
        assert_eq!(r.statutory_deadline_days, 0);
        assert!(!r.joint_and_several_liability);
        assert!(r.citation.contains("Default"));
    }

    #[test]
    fn default_neither_transfer_nor_refund_violation() {
        let mut i = base(Regime::Default);
        i.deposit_transferred_to_successor = false;
        i.deposit_refunded_to_tenant = false;
        let r = check(&i);
        assert!(!r.compliant);
    }

    // ── Regression-critical invariants ──────────────────────────

    #[test]
    fn deposit_disposition_compliant_iff_transfer_or_refund_invariant() {
        // 4-cell truth table.
        for (transfer, refund, expected_compliant) in [
            (false, false, false), // neither = violation
            (true, false, true),   // transfer = ok
            (false, true, true),   // refund = ok
            (true, true, true),    // both = ok (transfer wins)
        ] {
            let mut i = base(Regime::Default);
            i.deposit_transferred_to_successor = transfer;
            i.deposit_refunded_to_tenant = refund;
            // Disable tenant notice requirement when no transfer.
            if !transfer {
                i.tenant_notified_of_deposit_transfer = true;
            }
            let r = check(&i);
            assert_eq!(
                r.deposit_disposition_compliant,
                transfer || refund,
                "transfer={} refund={} expected disposition compliant={}",
                transfer,
                refund,
                transfer || refund,
            );
            assert_eq!(
                r.compliant, expected_compliant,
                "transfer={} refund={} expected overall compliant={}",
                transfer, refund, expected_compliant,
            );
        }
    }

    #[test]
    fn statutory_deadline_per_regime_invariant() {
        assert_eq!(check(&base(Regime::California)).statutory_deadline_days, 15);
        assert_eq!(
            check(&base(Regime::Massachusetts)).statutory_deadline_days,
            45,
        );
        assert_eq!(check(&base(Regime::Florida)).statutory_deadline_days, 0);
        assert_eq!(check(&base(Regime::Washington)).statutory_deadline_days, 0);
        assert_eq!(check(&base(Regime::NewYork)).statutory_deadline_days, 5);
        assert_eq!(check(&base(Regime::Default)).statutory_deadline_days, 0);
    }

    #[test]
    fn joint_and_several_liability_only_in_5_regulated_regimes_invariant() {
        for &regime in &[
            Regime::California,
            Regime::Massachusetts,
            Regime::Florida,
            Regime::Washington,
            Regime::NewYork,
        ] {
            assert!(
                check(&base(regime)).joint_and_several_liability,
                "{:?}: must have joint and several liability",
                regime,
            );
        }
        assert!(!check(&base(Regime::Default)).joint_and_several_liability);
    }

    #[test]
    fn nonpayment_eviction_barred_only_in_california_invariant() {
        // CA § 1962(c) is the only regime that BARS nonpayment
        // eviction during successor noncompliance.
        let mut ca = base(Regime::California);
        ca.successor_provided_identity_disclosure = false;
        ca.days_since_ownership_transfer = 20;
        assert!(check(&ca).nonpayment_eviction_barred);

        for &regime in &[
            Regime::Massachusetts,
            Regime::Florida,
            Regime::Washington,
            Regime::NewYork,
            Regime::Default,
        ] {
            let mut i = base(regime);
            i.successor_provided_identity_disclosure = false;
            i.days_since_ownership_transfer = 100;
            assert!(
                !check(&i).nonpayment_eviction_barred,
                "{:?}: must NOT bar nonpayment eviction",
                regime,
            );
        }
    }

    #[test]
    fn citation_pins_authority_per_regime() {
        assert!(check(&base(Regime::California))
            .citation
            .contains("§ 1962(c)"));
        assert!(check(&base(Regime::Massachusetts))
            .citation
            .contains("c. 186 § 15B"));
        assert!(check(&base(Regime::Florida))
            .citation
            .contains("§ 83.49(5)"));
        assert!(check(&base(Regime::Washington))
            .citation
            .contains("RCW 59.18.060"));
        assert!(check(&base(Regime::NewYork)).citation.contains("§ 7-105"));
        assert!(check(&base(Regime::Default)).citation.contains("Default"));
    }

    #[test]
    fn sibling_module_note_present_across_all_regimes() {
        for &regime in &[
            Regime::California,
            Regime::Massachusetts,
            Regime::Florida,
            Regime::Washington,
            Regime::NewYork,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("landlord_identification_disclosure")
                        && n.contains("foreclosure_tenant_rights")
                        && n.contains("VOLUNTARY")),
                "{:?}: sibling-module note must be present",
                regime,
            );
        }
    }

    #[test]
    fn five_day_boundary_truth_table_new_york_invariant() {
        for (days, expected_passed) in [
            (0_u32, false),
            (4, false),
            (5, false),
            (6, true),
            (30, true),
        ] {
            let mut i = base(Regime::NewYork);
            i.days_since_ownership_transfer = days;
            i.successor_provided_identity_disclosure = false;
            let r = check(&i);
            assert_eq!(
                r.disclosure_deadline_passed, expected_passed,
                "day {} expected_passed={}",
                days, expected_passed,
            );
        }
    }
}
