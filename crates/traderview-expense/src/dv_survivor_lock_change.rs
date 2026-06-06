//! Domestic violence / sexual assault / stalking survivor —
//! mid-tenancy lock change rights. Landlord operational concern for
//! tenant safety while preserving the landlord's property-control
//! interest. Distinct from `dv_termination` (early termination of
//! lease without penalty for DV survivors), `lock_change_between_
//! tenancies` (mandatory rekeying when tenancy turns over), and
//! `landlord_harassment` (anti-retaliation framework). This module
//! addresses the SPECIFIC MID-TENANCY LOCK-CHANGE PATHWAY where the
//! tenant remains in possession.
//!
//! Four regimes:
//!
//! **California — Cal. Civ. Code §§ 1941.5, 1941.6**. STRONGEST
//! tenant-protection framework in the US. Two parallel provisions:
//! § 1941.5 (perpetrator NOT a co-tenant) — landlord must change
//! locks WITHIN 24 HOURS of receiving written documentation at the
//! LANDLORD'S OWN EXPENSE. § 1941.6 (perpetrator IS a co-tenant)
//! — landlord must still change at landlord's expense to exclude
//! the perpetrator-tenant. If the landlord fails to act within 24
//! hours, the tenant may self-help change locks and the landlord
//! must reimburse within 21 days. Accepted documentation: copy of
//! restraining order; copy of police report; letter from qualified
//! third party (DV counselor, healthcare provider, clergy); signed
//! statement by tenant (SB 1051 expansion, 2023). Applies to
//! domestic violence, sexual assault, stalking, human trafficking,
//! and elder abuse.
//!
//! **Texas — Tex. Prop. Code §§ 92.156, 92.016**. Standard
//! § 92.156(d) rekeying at LANDLORD'S expense applies on tenancy
//! turnover (within 7 days). Additional tenant-requested rekeying
//! (including DV survivor mid-tenancy) is at TENANT'S EXPENSE under
//! § 92.156(e) — sharp contrast with California's landlord-pays
//! rule. DV survivors may also terminate the lease early under
//! § 92.016 (out of scope; covered by `dv_termination`).
//!
//! **Washington — RCW 59.18.575**. TENANT-PAID lock change with
//! NOTICE-TO-LANDLORD requirement. The tenant may change or add
//! locks at the TENANT'S EXPENSE without prior landlord
//! authorization. Within 7 DAYS of changing/adding locks, the
//! tenant must deliver written notice to the landlord by mail,
//! fax, or personal delivery by a third party plus a copy of a
//! valid protection order OR a written record signed by a qualified
//! third party (law enforcement officer, state court employee,
//! healthcare professional, licensed mental health professional,
//! clergy, crime-victim or witness-program advocate). Applies to
//! domestic violence, sexual assault, stalking, and unlawful
//! harassment (including by the landlord).
//!
//! **Default — common-law lease + state-specific statutes**. Most
//! states have some form of DV lock-change framework, but
//! cost-allocation and notice procedures vary sharply. The
//! common-law lease does not authorize tenant self-help lock
//! changes absent state-specific DV statute.
//!
//! Citations: Cal. Civ. Code § 1941.5 (perpetrator-not-co-tenant
//! 24-hour landlord-pays rule); § 1941.6 (perpetrator-is-co-tenant
//! landlord-pays rule); SB 1051 (2023, signed-statement
//! documentation expansion); SB 782 (2010, DV lock-change
//! enactment); Tex. Prop. Code § 92.156(d) (turnover rekey at
//! landlord expense); § 92.156(e) (additional tenant-requested
//! rekey at tenant expense); § 92.016 (DV early termination —
//! separately covered); RCW 59.18.575 (WA 7-day notice rule
//! tenant-paid); RCW 59.18.570 (definitions of qualified third
//! party and DV).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Texas,
    Washington,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentationType {
    RestrainingOrder,
    PoliceReport,
    QualifiedThirdPartyLetter,
    /// CA SB 1051 (2023) added signed-statement-by-tenant
    /// documentation option. WA does not accept signed-statement
    /// alone — requires order OR third-party record.
    SignedStatementByTenant,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CostAllocation {
    Landlord,
    Tenant,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DvLockChangeInput {
    pub regime: Regime,
    pub documentation_type: DocumentationType,
    /// California § 1941.5 vs § 1941.6 distinction — drives statute
    /// citation but cost allocation is landlord-pays in both paths.
    pub perpetrator_is_co_tenant: bool,
    /// California: hours between tenant providing documentation and
    /// landlord changing locks. § 1941.5 requires within 24 hours.
    pub hours_to_landlord_action: Option<u32>,
    /// California § 1941.5 self-help — tenant changed locks because
    /// landlord did not act within 24 hours. Triggers 21-day
    /// landlord reimbursement deadline.
    pub tenant_self_help_after_landlord_inaction: bool,
    /// California § 1941.5(c) — days between tenant self-help and
    /// landlord reimbursement. Must be 21 days or less.
    pub days_to_landlord_reimbursement: Option<u32>,
    /// Washington RCW 59.18.575 — days between tenant changing
    /// locks and delivering written notice + documentation to
    /// landlord. Must be 7 days or less.
    pub days_to_landlord_notice: Option<u32>,
    /// Texas § 92.156(e) — tenant requested rekey. In TX additional
    /// tenant-requested rekeys are at tenant expense regardless of
    /// DV status.
    pub tenant_requested_rekey: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DvLockChangeResult {
    pub lock_change_valid: bool,
    pub cost_allocation: CostAllocation,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &DvLockChangeInput) -> DvLockChangeResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    let cost_allocation = match input.regime {
        Regime::California => {
            check_california(input, &mut violations, &mut notes);
            CostAllocation::Landlord
        }
        Regime::Texas => {
            check_texas(input, &mut violations, &mut notes);
            if input.tenant_requested_rekey {
                CostAllocation::Tenant
            } else {
                CostAllocation::Landlord
            }
        }
        Regime::Washington => {
            check_washington(input, &mut violations, &mut notes);
            CostAllocation::Tenant
        }
        Regime::Default => {
            check_default(input, &mut violations, &mut notes);
            CostAllocation::Tenant
        }
    };

    DvLockChangeResult {
        lock_change_valid: violations.is_empty(),
        cost_allocation,
        violations: violations.clone(),
        citation: citation_for(input.regime),
        notes: notes.clone(),
    }
}

fn check_california(
    input: &DvLockChangeInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) {
    if input.perpetrator_is_co_tenant {
        notes.push("§ 1941.6 — perpetrator is co-tenant; landlord-pays still applies".to_string());
    } else {
        notes.push(
            "§ 1941.5 — perpetrator NOT co-tenant; landlord must change locks within 24 hours of documentation at landlord's expense"
                .to_string(),
        );
    }

    match input.hours_to_landlord_action {
        Some(h) if h <= 24 => {
            notes.push(format!(
                "landlord acted within {} hours — within § 1941.5 24-hour deadline",
                h
            ));
        }
        Some(h) => {
            violations.push(format!(
                "landlord delayed {} hours — exceeds § 1941.5 24-hour deadline",
                h
            ));
        }
        None => {}
    }

    if input.tenant_self_help_after_landlord_inaction {
        notes.push(
            "§ 1941.5 — tenant self-help permitted after 24-hour landlord inaction; landlord must reimburse within 21 days"
                .to_string(),
        );
        if let Some(d) = input.days_to_landlord_reimbursement {
            if d > 21 {
                violations.push(format!(
                    "§ 1941.5(c) — landlord delayed reimbursement {} days — exceeds 21-day deadline",
                    d
                ));
            }
        }
    }

    if matches!(
        input.documentation_type,
        DocumentationType::SignedStatementByTenant
    ) {
        notes.push(
            "SB 1051 (2023) — signed statement by tenant accepted under CA framework".to_string(),
        );
    }
}

fn check_texas(input: &DvLockChangeInput, _violations: &mut Vec<String>, notes: &mut Vec<String>) {
    if input.tenant_requested_rekey {
        notes.push(
            "§ 92.156(e) — tenant-requested rekey at tenant expense; unlimited number of requests permitted"
                .to_string(),
        );
    } else {
        notes.push(
            "§ 92.156(d) — turnover rekey at landlord expense within 7 days; § 92.016 covers DV early termination separately"
                .to_string(),
        );
    }
}

fn check_washington(
    input: &DvLockChangeInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) {
    if matches!(
        input.documentation_type,
        DocumentationType::SignedStatementByTenant
    ) {
        violations.push(
            "RCW 59.18.575 — tenant signed statement alone NOT accepted; requires valid protection order OR written record signed by qualified third party"
                .to_string(),
        );
    }

    match input.days_to_landlord_notice {
        Some(d) if d <= 7 => {
            notes.push(format!(
                "tenant delivered notice + documentation within {} days — within RCW 59.18.575 7-day deadline",
                d
            ));
        }
        Some(d) => {
            violations.push(format!(
                "RCW 59.18.575 — tenant notice delivered {} days after lock change — exceeds 7-day deadline",
                d
            ));
        }
        None => {
            violations.push(
                "RCW 59.18.575 — tenant must deliver written notice + documentation to landlord within 7 days of lock change"
                    .to_string(),
            );
        }
    }

    notes.push(
        "RCW 59.18.575 — qualified third parties include law enforcement, state court employees, healthcare professionals, licensed mental health providers, clergy, and crime-victim advocates"
            .to_string(),
    );
}

fn check_default(
    _input: &DvLockChangeInput,
    _violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) {
    notes.push(
        "default common-law rule — most states have some form of DV lock-change statute but cost allocation and notice procedures vary sharply; common-law lease alone does not authorize tenant self-help"
            .to_string(),
    );
}

fn citation_for(regime: Regime) -> &'static str {
    match regime {
        Regime::California => "Cal. Civ. Code §§ 1941.5, 1941.6; SB 1051 (2023); SB 782 (2010)",
        Regime::Texas => "Tex. Prop. Code §§ 92.156(d)/(e), 92.016",
        Regime::Washington => "RCW 59.18.575; RCW 59.18.570",
        Regime::Default => "common-law lease + state-specific DV statute",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_base() -> DvLockChangeInput {
        DvLockChangeInput {
            regime: Regime::California,
            documentation_type: DocumentationType::RestrainingOrder,
            perpetrator_is_co_tenant: false,
            hours_to_landlord_action: Some(12),
            tenant_self_help_after_landlord_inaction: false,
            days_to_landlord_reimbursement: None,
            days_to_landlord_notice: None,
            tenant_requested_rekey: false,
        }
    }

    fn tx_base() -> DvLockChangeInput {
        DvLockChangeInput {
            regime: Regime::Texas,
            documentation_type: DocumentationType::PoliceReport,
            perpetrator_is_co_tenant: false,
            hours_to_landlord_action: None,
            tenant_self_help_after_landlord_inaction: false,
            days_to_landlord_reimbursement: None,
            days_to_landlord_notice: None,
            tenant_requested_rekey: true,
        }
    }

    fn wa_base() -> DvLockChangeInput {
        DvLockChangeInput {
            regime: Regime::Washington,
            documentation_type: DocumentationType::RestrainingOrder,
            perpetrator_is_co_tenant: false,
            hours_to_landlord_action: None,
            tenant_self_help_after_landlord_inaction: false,
            days_to_landlord_reimbursement: None,
            days_to_landlord_notice: Some(3),
            tenant_requested_rekey: false,
        }
    }

    fn default_base() -> DvLockChangeInput {
        DvLockChangeInput {
            regime: Regime::Default,
            documentation_type: DocumentationType::RestrainingOrder,
            perpetrator_is_co_tenant: false,
            hours_to_landlord_action: None,
            tenant_self_help_after_landlord_inaction: false,
            days_to_landlord_reimbursement: None,
            days_to_landlord_notice: None,
            tenant_requested_rekey: false,
        }
    }

    #[test]
    fn ca_landlord_acts_within_24_hours_valid_landlord_pays() {
        let r = check(&ca_base());
        assert!(r.lock_change_valid);
        assert_eq!(r.cost_allocation, CostAllocation::Landlord);
    }

    #[test]
    fn ca_landlord_delayed_25_hours_violation() {
        let mut i = ca_base();
        i.hours_to_landlord_action = Some(25);
        let r = check(&i);
        assert!(!r.lock_change_valid);
        assert!(r.violations.iter().any(|v| v.contains("24-hour deadline")));
    }

    #[test]
    fn ca_exactly_24_hours_valid() {
        let mut i = ca_base();
        i.hours_to_landlord_action = Some(24);
        let r = check(&i);
        assert!(r.lock_change_valid);
    }

    #[test]
    fn ca_perpetrator_co_tenant_uses_section_1941_6() {
        let mut i = ca_base();
        i.perpetrator_is_co_tenant = true;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 1941.6")));
        assert_eq!(r.cost_allocation, CostAllocation::Landlord);
    }

    #[test]
    fn ca_self_help_with_timely_reimbursement_valid() {
        let mut i = ca_base();
        i.hours_to_landlord_action = None;
        i.tenant_self_help_after_landlord_inaction = true;
        i.days_to_landlord_reimbursement = Some(15);
        let r = check(&i);
        assert!(r.lock_change_valid);
        assert!(r.notes.iter().any(|n| n.contains("21 days")));
    }

    #[test]
    fn ca_self_help_with_late_reimbursement_violation() {
        let mut i = ca_base();
        i.hours_to_landlord_action = None;
        i.tenant_self_help_after_landlord_inaction = true;
        i.days_to_landlord_reimbursement = Some(22);
        let r = check(&i);
        assert!(!r.lock_change_valid);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1941.5(c)") && v.contains("21-day deadline")));
    }

    #[test]
    fn ca_signed_statement_documentation_accepted() {
        let mut i = ca_base();
        i.documentation_type = DocumentationType::SignedStatementByTenant;
        let r = check(&i);
        assert!(r.lock_change_valid);
        assert!(r.notes.iter().any(|n| n.contains("SB 1051")));
    }

    #[test]
    fn ca_self_help_at_21_day_boundary_valid() {
        let mut i = ca_base();
        i.hours_to_landlord_action = None;
        i.tenant_self_help_after_landlord_inaction = true;
        i.days_to_landlord_reimbursement = Some(21);
        let r = check(&i);
        assert!(r.lock_change_valid);
    }

    #[test]
    fn tx_tenant_requested_rekey_at_tenant_expense() {
        let r = check(&tx_base());
        assert!(r.lock_change_valid);
        assert_eq!(r.cost_allocation, CostAllocation::Tenant);
        assert!(r.notes.iter().any(|n| n.contains("§ 92.156(e)")));
    }

    #[test]
    fn tx_turnover_rekey_at_landlord_expense() {
        let mut i = tx_base();
        i.tenant_requested_rekey = false;
        let r = check(&i);
        assert!(r.lock_change_valid);
        assert_eq!(r.cost_allocation, CostAllocation::Landlord);
        assert!(r.notes.iter().any(|n| n.contains("§ 92.156(d)")));
    }

    #[test]
    fn tx_notes_section_92_016_dv_termination_separately_covered() {
        let mut i = tx_base();
        i.tenant_requested_rekey = false;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 92.016")));
    }

    #[test]
    fn wa_tenant_paid_within_7_day_notice_valid() {
        let r = check(&wa_base());
        assert!(r.lock_change_valid);
        assert_eq!(r.cost_allocation, CostAllocation::Tenant);
    }

    #[test]
    fn wa_at_7_day_notice_boundary_valid() {
        let mut i = wa_base();
        i.days_to_landlord_notice = Some(7);
        let r = check(&i);
        assert!(r.lock_change_valid);
    }

    #[test]
    fn wa_8_day_notice_violation() {
        let mut i = wa_base();
        i.days_to_landlord_notice = Some(8);
        let r = check(&i);
        assert!(!r.lock_change_valid);
        assert!(r.violations.iter().any(|v| v.contains("7-day deadline")));
    }

    #[test]
    fn wa_no_notice_provided_violation() {
        let mut i = wa_base();
        i.days_to_landlord_notice = None;
        let r = check(&i);
        assert!(!r.lock_change_valid);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("must deliver written notice")));
    }

    #[test]
    fn wa_signed_statement_alone_not_accepted() {
        let mut i = wa_base();
        i.documentation_type = DocumentationType::SignedStatementByTenant;
        let r = check(&i);
        assert!(!r.lock_change_valid);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("signed statement alone NOT accepted")));
    }

    #[test]
    fn wa_qualified_third_party_letter_accepted() {
        let mut i = wa_base();
        i.documentation_type = DocumentationType::QualifiedThirdPartyLetter;
        let r = check(&i);
        assert!(r.lock_change_valid);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("qualified third parties")));
    }

    #[test]
    fn default_tenant_expense_with_common_law_caution_note() {
        let r = check(&default_base());
        assert_eq!(r.cost_allocation, CostAllocation::Tenant);
        assert!(r.notes.iter().any(|n| n.contains("vary sharply")));
    }

    #[test]
    fn citation_california_pins_both_sections_and_bills() {
        let r = check(&ca_base());
        assert!(r.citation.contains("§§ 1941.5, 1941.6"));
        assert!(r.citation.contains("SB 1051"));
        assert!(r.citation.contains("SB 782"));
    }

    #[test]
    fn citation_texas_pins_subsections_and_section_92_016() {
        let r = check(&tx_base());
        assert!(r.citation.contains("§§ 92.156(d)/(e), 92.016"));
    }

    #[test]
    fn citation_washington_pins_575_and_570() {
        let r = check(&wa_base());
        assert!(r.citation.contains("RCW 59.18.575"));
        assert!(r.citation.contains("RCW 59.18.570"));
    }

    #[test]
    fn california_uniquely_landlord_pays_for_dv_lock_change_invariant() {
        let r_ca = check(&ca_base());
        let mut tx_dv = tx_base();
        tx_dv.tenant_requested_rekey = true;
        let r_tx = check(&tx_dv);
        let r_wa = check(&wa_base());
        let r_default = check(&default_base());
        assert_eq!(r_ca.cost_allocation, CostAllocation::Landlord);
        assert_eq!(r_tx.cost_allocation, CostAllocation::Tenant);
        assert_eq!(r_wa.cost_allocation, CostAllocation::Tenant);
        assert_eq!(r_default.cost_allocation, CostAllocation::Tenant);
    }

    #[test]
    fn ca_landlord_inaction_violation_engages_24_hour_deadline_specifically() {
        let mut i = ca_base();
        i.hours_to_landlord_action = Some(48);
        let r = check(&i);
        assert!(r.violations.iter().any(|v| v.contains("48 hours")));
    }

    #[test]
    fn wa_tenant_signed_statement_with_protection_order_invalid_path_still_fails() {
        let mut i = wa_base();
        i.documentation_type = DocumentationType::SignedStatementByTenant;
        i.days_to_landlord_notice = Some(3);
        let r = check(&i);
        assert!(
            !r.lock_change_valid,
            "WA does not accept signed-statement alone regardless of notice timing"
        );
    }

    #[test]
    fn ca_self_help_at_21_days_exact_boundary_valid_22_days_invalid() {
        let mut i_21 = ca_base();
        i_21.hours_to_landlord_action = None;
        i_21.tenant_self_help_after_landlord_inaction = true;
        i_21.days_to_landlord_reimbursement = Some(21);
        let r_21 = check(&i_21);
        let mut i_22 = i_21.clone();
        i_22.days_to_landlord_reimbursement = Some(22);
        let r_22 = check(&i_22);
        assert!(r_21.lock_change_valid);
        assert!(!r_22.lock_change_valid);
    }
}
