//! State landlord lock-change / rekey / security-device compliance
//! check for new-tenant turnover.
//!
//! Operational concern for every landlord between tenancies: must
//! locks be rekeyed or replaced before the new tenant moves in?
//! Tenant safety topic distinct from `lockout_penalties` (which
//! addresses unlawful self-help lockouts during tenancy) and
//! `dv_termination` (which addresses domestic-violence emergency
//! lock change during tenancy).
//!
//! Six regimes:
//!
//!   - **Texas** — Tex. Prop. Code § 92.156 — security device
//!     operated by a key, card, or combination shall be rekeyed
//!     by the landlord at the LANDLORD'S EXPENSE not later than
//!     the 7th day after each tenant turnover date. Tenant may
//!     request additional rekeying at the tenant's expense.
//!     Strongest mandatory rekey-between-tenancies statute in the
//!     matrix.
//!
//!   - **California** — Cal. Civ. Code § 1941.3 — landlord must
//!     install and maintain an OPERABLE DEAD BOLT LOCK on each
//!     main swinging entry door of a dwelling unit. Bolt must
//!     extend at least 13/16 of an inch beyond the strike edge of
//!     the door and protrude into the doorjamb. Does NOT mandate
//!     rekeying between tenancies — covers installation +
//!     maintenance of the deadbolt itself.
//!
//!   - **Illinois** — Chicago Residential Landlord and Tenant
//!     Ordinance + local statutes — landlord must change or
//!     rekey locks ON OR BEFORE the day the new tenant moves in.
//!     Strictest timing in matrix (same-day requirement).
//!
//!   - **Virginia** — Va. Code § 55.1-1221 — landlord must
//!     provide LOCKS AND PEEPHOLES on each rental dwelling unit.
//!     Installation + maintenance obligation; no automatic
//!     rekeying mandate between tenancies.
//!
//!   - **NewYork** — no statewide lock-change requirement; NYC
//!     Housing Maintenance Code may impose specific lock standards.
//!     State silent on rekeying between tenancies.
//!
//!   - **Default** — no statewide rekeying requirement; common-law
//!     best practice for new-tenant safety but no enforceable
//!     mandate.
//!
//! Citations: Tex. Prop. Code § 92.156(a) (7-day rekey requirement);
//! § 92.156(b) (tenant-requested additional rekeying at tenant
//! expense); Cal. Civ. Code § 1941.3(a)(1) (deadbolt installation
//! and maintenance); § 1941.3(a)(2) (bolt extension specification
//! of at least 13/16 inch beyond strike edge); 765 ILCS 5/12 plus
//! Chicago RLTO (Illinois same-day rekey mandate); Va. Code
//! § 55.1-1221 (Virginia locks and peepholes); NYC Housing
//! Maintenance Code § 27-2043 (NYC peephole requirement); common
//! law (Default best-practice without enforceable mandate).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Texas,
    California,
    Illinois,
    Virginia,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    /// Days elapsed between the previous tenant's move-out and the
    /// new tenant's move-in.
    pub days_since_prior_tenant_move_out: u32,
    /// Days elapsed since the new tenant's move-in date.
    pub days_since_new_tenant_move_in: u32,
    /// Whether the landlord rekeyed or replaced the security device
    /// (key-operated lock, electronic card, combination keypad)
    /// before or after the new-tenant move-in.
    pub landlord_rekeyed_security_device: bool,
    /// Days elapsed between the new-tenant move-in date and the
    /// landlord's rekeying action. Used for Texas + Illinois
    /// timing tests.
    pub days_from_move_in_to_rekey: u32,
    /// Whether the rekeying was done at the landlord's expense.
    pub rekeying_at_landlord_expense: bool,
    /// Whether the main entry door has an operable deadbolt
    /// (California § 1941.3 requirement).
    pub main_entry_door_has_operable_deadbolt: bool,
    /// Whether the deadbolt bolt extension is at least 13/16 of an
    /// inch beyond the strike edge (California § 1941.3
    /// specification).
    pub deadbolt_extension_meets_spec: bool,
    /// Whether the door has a peephole (Virginia § 55.1-1221).
    pub door_has_peephole: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    pub compliant: bool,
    /// Whether the regime imposes a mandatory rekey-between-tenancies
    /// duty on the landlord (TX + IL).
    pub mandatory_rekey_between_tenancies: bool,
    /// Statutory deadline for rekey from new-tenant move-in (days).
    /// None where no deadline applies.
    pub rekey_deadline_days: Option<u32>,
    /// Whether the regime requires the rekey to be done at
    /// LANDLORD'S expense (Texas — yes; Illinois — yes per Chicago
    /// RLTO).
    pub at_landlord_expense_required: bool,
    /// Whether the regime requires installation of a specific
    /// security device (CA deadbolt; VA peephole).
    pub specific_security_device_required: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// Texas § 92.156(a) — rekey within 7 days of tenant turnover.
pub const TEXAS_REKEY_DEADLINE_DAYS: u32 = 7;
/// Illinois — same-day rekey requirement (≤ 0 days from move-in).
pub const ILLINOIS_REKEY_DEADLINE_DAYS: u32 = 0;
/// California § 1941.3(a)(2) — deadbolt bolt extension threshold
/// (16ths of an inch).
pub const CA_DEADBOLT_EXTENSION_16THS: u32 = 13;

pub fn check(input: &Input) -> CheckResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    let (
        mandatory_rekey_between_tenancies,
        rekey_deadline_days,
        at_landlord_expense_required,
        specific_security_device_required,
        citation,
    ): (bool, Option<u32>, bool, bool, &'static str) = match input.regime {
        Regime::Texas => (
            true,
            Some(TEXAS_REKEY_DEADLINE_DAYS),
            true,
            false,
            "Tex. Prop. Code § 92.156(a) (security device operated by key/card/combination \
             shall be rekeyed by landlord at landlord's expense not later than 7th day after \
             each tenant turnover date); § 92.156(b) (tenant-requested additional rekeying at \
             tenant expense)",
        ),
        Regime::California => (
            false,
            None,
            false,
            true,
            "Cal. Civ. Code § 1941.3(a)(1) (operable deadbolt lock required on each main \
             swinging entry door); § 1941.3(a)(2) (bolt extension at least 13/16 inch beyond \
             strike edge into doorjamb)",
        ),
        Regime::Illinois => (
            true,
            Some(ILLINOIS_REKEY_DEADLINE_DAYS),
            true,
            false,
            "765 ILCS 5/12 + Chicago RLTO + local statutes — landlord shall change or rekey \
             locks on or before the day the new tenant moves in (same-day requirement)",
        ),
        Regime::Virginia => (
            false,
            None,
            false,
            true,
            "Va. Code § 55.1-1221 (landlord shall provide locks AND peepholes on each rental \
             dwelling unit; installation + maintenance obligation without automatic rekeying \
             mandate between tenancies)",
        ),
        Regime::NewYork => (
            false,
            None,
            false,
            false,
            "No statewide NY lock-change requirement; NYC Housing Maintenance Code § 27-2043 \
             may impose specific peephole standards in NYC dwellings",
        ),
        Regime::Default => (
            false,
            None,
            false,
            false,
            "No statewide rekeying requirement; common-law best practice for new-tenant safety \
             but no enforceable mandate",
        ),
    };

    // Texas + Illinois rekey timing.
    if let Some(deadline) = rekey_deadline_days {
        if !input.landlord_rekeyed_security_device {
            violations.push(format!(
                "{:?}: rekeying or change of security device required but not performed.",
                input.regime,
            ));
        } else if input.days_from_move_in_to_rekey > deadline {
            violations.push(format!(
                "{:?}: rekey performed {} days after new-tenant move-in; statutory deadline is \
                 {} days.",
                input.regime, input.days_from_move_in_to_rekey, deadline,
            ));
        }

        // Texas-specific landlord-expense requirement.
        if at_landlord_expense_required
            && input.landlord_rekeyed_security_device
            && !input.rekeying_at_landlord_expense
        {
            violations.push(format!(
                "{:?}: rekeying required at landlord's expense; charging tenant for the \
                 between-tenancy rekey violates the statute.",
                input.regime,
            ));
        }
    }

    // California deadbolt installation + bolt-extension spec.
    if matches!(input.regime, Regime::California) {
        if !input.main_entry_door_has_operable_deadbolt {
            violations.push(
                "Cal. Civ. Code § 1941.3(a)(1) — main swinging entry door lacks operable \
                 deadbolt lock."
                    .to_string(),
            );
        }
        if input.main_entry_door_has_operable_deadbolt && !input.deadbolt_extension_meets_spec {
            violations.push(format!(
                "Cal. Civ. Code § 1941.3(a)(2) — deadbolt bolt extension less than {}/16 of an \
                 inch beyond strike edge.",
                CA_DEADBOLT_EXTENSION_16THS,
            ));
        }
    }

    // Virginia locks + peepholes.
    if matches!(input.regime, Regime::Virginia) && !input.door_has_peephole {
        violations.push(
            "Va. Code § 55.1-1221 — rental dwelling unit lacks peephole on entry door."
                .to_string(),
        );
    }

    notes.push(
        "Distinct from lockout_penalties (unlawful self-help lockouts during tenancy) and \
         dv_termination (domestic-violence emergency lock change during tenancy). This module \
         addresses the BETWEEN-TENANCIES lock change / security device installation duty."
            .to_string(),
    );

    if matches!(input.regime, Regime::Texas) && input.landlord_rekeyed_security_device {
        notes.push(format!(
            "Texas § 92.156(a) — landlord rekeyed within {} days of move-in (statutory \
             threshold {} days).",
            input.days_from_move_in_to_rekey, TEXAS_REKEY_DEADLINE_DAYS,
        ));
    }

    CheckResult {
        compliant: violations.is_empty(),
        mandatory_rekey_between_tenancies,
        rekey_deadline_days,
        at_landlord_expense_required,
        specific_security_device_required,
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
            days_since_prior_tenant_move_out: 5,
            days_since_new_tenant_move_in: 3,
            landlord_rekeyed_security_device: true,
            days_from_move_in_to_rekey: 0,
            rekeying_at_landlord_expense: true,
            main_entry_door_has_operable_deadbolt: true,
            deadbolt_extension_meets_spec: true,
            door_has_peephole: true,
        }
    }

    // ── Texas § 92.156 ──────────────────────────────────────────

    #[test]
    fn texas_within_7_days_at_landlord_expense_compliant() {
        let mut i = base(Regime::Texas);
        i.days_from_move_in_to_rekey = 5;
        let r = check(&i);
        assert!(r.compliant);
        assert!(r.mandatory_rekey_between_tenancies);
        assert_eq!(r.rekey_deadline_days, Some(7));
        assert!(r.at_landlord_expense_required);
        assert!(r.citation.contains("§ 92.156(a)"));
    }

    #[test]
    fn texas_at_7_day_boundary_compliant() {
        let mut i = base(Regime::Texas);
        i.days_from_move_in_to_rekey = 7;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn texas_past_7_day_deadline_violation() {
        let mut i = base(Regime::Texas);
        i.days_from_move_in_to_rekey = 8;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("Texas") && v.contains("8 days")));
    }

    #[test]
    fn texas_landlord_did_not_rekey_violation() {
        let mut i = base(Regime::Texas);
        i.landlord_rekeyed_security_device = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("Texas") && v.contains("not performed"))
        );
    }

    #[test]
    fn texas_charged_tenant_expense_violation() {
        let mut i = base(Regime::Texas);
        i.rekeying_at_landlord_expense = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("landlord's expense") && v.contains("violates"))
        );
    }

    // ── California § 1941.3 ─────────────────────────────────────

    #[test]
    fn california_operable_deadbolt_with_spec_compliant() {
        let r = check(&base(Regime::California));
        assert!(r.compliant);
        assert!(!r.mandatory_rekey_between_tenancies);
        assert!(r.specific_security_device_required);
        assert!(r.citation.contains("§ 1941.3(a)(1)"));
        assert!(r.citation.contains("§ 1941.3(a)(2)"));
    }

    #[test]
    fn california_missing_deadbolt_violation() {
        let mut i = base(Regime::California);
        i.main_entry_door_has_operable_deadbolt = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("§ 1941.3(a)(1)") && v.contains("lacks operable deadbolt"))
        );
    }

    #[test]
    fn california_deadbolt_extension_below_spec_violation() {
        let mut i = base(Regime::California);
        i.deadbolt_extension_meets_spec = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("§ 1941.3(a)(2)") && v.contains("13/16"))
        );
    }

    // ── Illinois same-day rekey ─────────────────────────────────

    #[test]
    fn illinois_same_day_rekey_compliant() {
        let mut i = base(Regime::Illinois);
        i.days_from_move_in_to_rekey = 0;
        let r = check(&i);
        assert!(r.compliant);
        assert!(r.mandatory_rekey_between_tenancies);
        assert_eq!(r.rekey_deadline_days, Some(0));
    }

    #[test]
    fn illinois_one_day_after_move_in_violation() {
        let mut i = base(Regime::Illinois);
        i.days_from_move_in_to_rekey = 1;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("Illinois")));
    }

    #[test]
    fn illinois_did_not_rekey_violation() {
        let mut i = base(Regime::Illinois);
        i.landlord_rekeyed_security_device = false;
        let r = check(&i);
        assert!(!r.compliant);
    }

    // ── Virginia § 55.1-1221 ────────────────────────────────────

    #[test]
    fn virginia_locks_and_peephole_compliant() {
        let r = check(&base(Regime::Virginia));
        assert!(r.compliant);
        assert!(r.specific_security_device_required);
        assert!(!r.mandatory_rekey_between_tenancies);
        assert!(r.citation.contains("§ 55.1-1221"));
    }

    #[test]
    fn virginia_missing_peephole_violation() {
        let mut i = base(Regime::Virginia);
        i.door_has_peephole = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(
            r.violations
                .iter()
                .any(|v| v.contains("§ 55.1-1221") && v.contains("peephole"))
        );
    }

    // ── New York — no statewide rekeying mandate ────────────────

    #[test]
    fn new_york_no_statewide_mandate_compliant() {
        let mut i = base(Regime::NewYork);
        // Even without rekeying at all, NY state has no statewide
        // mandate; compliant.
        i.landlord_rekeyed_security_device = false;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.mandatory_rekey_between_tenancies);
        assert!(r.citation.contains("No statewide NY"));
    }

    // ── Default ─────────────────────────────────────────────────

    #[test]
    fn default_no_mandate_compliant_regardless() {
        let mut i = base(Regime::Default);
        i.landlord_rekeyed_security_device = false;
        i.main_entry_door_has_operable_deadbolt = false;
        i.door_has_peephole = false;
        let r = check(&i);
        // No statewide mandate — compliant.
        assert!(r.compliant);
        assert!(r.citation.contains("common-law best practice"));
    }

    // ── Regression-critical multi-regime invariants ─────────────

    #[test]
    fn only_tx_and_il_impose_mandatory_rekey_invariant() {
        for &regime in &[Regime::Texas, Regime::Illinois] {
            assert!(
                check(&base(regime)).mandatory_rekey_between_tenancies,
                "{:?}: must impose mandatory rekey",
                regime,
            );
        }
        for &regime in &[
            Regime::California,
            Regime::Virginia,
            Regime::NewYork,
            Regime::Default,
        ] {
            assert!(
                !check(&base(regime)).mandatory_rekey_between_tenancies,
                "{:?}: must NOT impose mandatory rekey",
                regime,
            );
        }
    }

    #[test]
    fn only_ca_and_va_require_specific_security_device_invariant() {
        for &regime in &[Regime::California, Regime::Virginia] {
            assert!(
                check(&base(regime)).specific_security_device_required,
                "{:?}: must require specific security device",
                regime,
            );
        }
        for &regime in &[
            Regime::Texas,
            Regime::Illinois,
            Regime::NewYork,
            Regime::Default,
        ] {
            assert!(
                !check(&base(regime)).specific_security_device_required,
                "{:?}: must NOT require specific security device",
                regime,
            );
        }
    }

    #[test]
    fn illinois_stricter_than_texas_timing_invariant() {
        // Both Texas and Illinois mandate rekeying — Illinois is
        // same-day (0 days), Texas is 7 days. Same-day-plus-1
        // should violate IL but not TX.
        let mut tx = base(Regime::Texas);
        tx.days_from_move_in_to_rekey = 1;
        let mut il = base(Regime::Illinois);
        il.days_from_move_in_to_rekey = 1;
        assert!(check(&tx).compliant);
        assert!(!check(&il).compliant);
    }

    #[test]
    fn only_tx_il_require_at_landlord_expense_invariant() {
        for &regime in &[Regime::Texas, Regime::Illinois] {
            assert!(
                check(&base(regime)).at_landlord_expense_required,
                "{:?}: must require at landlord expense",
                regime,
            );
        }
        for &regime in &[
            Regime::California,
            Regime::Virginia,
            Regime::NewYork,
            Regime::Default,
        ] {
            assert!(
                !check(&base(regime)).at_landlord_expense_required,
                "{:?}: must NOT require at landlord expense",
                regime,
            );
        }
    }

    #[test]
    fn citation_pins_authority_per_regime() {
        assert!(check(&base(Regime::Texas)).citation.contains("§ 92.156"));
        assert!(check(&base(Regime::California)).citation.contains("§ 1941.3"));
        assert!(check(&base(Regime::Illinois)).citation.contains("765 ILCS"));
        assert!(check(&base(Regime::Virginia)).citation.contains("§ 55.1-1221"));
        assert!(check(&base(Regime::NewYork)).citation.contains("No statewide NY"));
        assert!(check(&base(Regime::Default)).citation.contains("common-law"));
    }

    #[test]
    fn sibling_module_note_present_across_all_regimes() {
        for &regime in &[
            Regime::Texas,
            Regime::California,
            Regime::Illinois,
            Regime::Virginia,
            Regime::NewYork,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(
                r.notes.iter().any(|n| n.contains("lockout_penalties")
                    && n.contains("dv_termination")
                    && n.contains("BETWEEN-TENANCIES")),
                "{:?}: sibling-module note must be present",
                regime,
            );
        }
    }

    #[test]
    fn texas_boundary_7_versus_8_days_truth_table_invariant() {
        for (days, expected_compliant) in [(0_u32, true), (6, true), (7, true), (8, false)] {
            let mut i = base(Regime::Texas);
            i.days_from_move_in_to_rekey = days;
            let r = check(&i);
            assert_eq!(
                r.compliant, expected_compliant,
                "TX day {} expected_compliant={}",
                days, expected_compliant,
            );
        }
    }
}
