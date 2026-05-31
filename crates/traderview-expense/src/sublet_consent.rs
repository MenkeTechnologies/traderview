//! State-by-state lease assignment + subletting consent rules.
//!
//! Common scenarios for trader-tenants: relocate for work mid-lease,
//! sublet vacation property during off-season, add a roommate. Most
//! states leave consent terms to the lease contract — landlord may
//! withhold for any reason if the lease says so. Two states impose
//! statutory or case-law "reasonable standard" tests:
//!
//! - **NY RPL § 226-b** (the canonical statute): residential tenants in
//!   buildings with 4+ units may sublet subject to landlord consent
//!   that "may not be unreasonably withheld." Landlord must respond
//!   within 30 days; failure to respond = deemed consent. **Lease
//!   assignment** is treated differently — landlord may UNCONDITIONALLY
//!   withhold consent, but if unreasonably withheld, the tenant may
//!   TERMINATE the lease with 30 days notice. The asymmetry (sublet
//!   reasonable standard vs assignment termination-right standard) is
//!   load-bearing in NY practice.
//!
//! - **CA Kendall v. Ernest Pestana, Inc. (1985)** — case law applies a
//!   reasonable-standard test to all commercial AND residential leases.
//!   Absolute-prohibition clauses are VOID where landlord unreasonably
//!   withholds consent. Cal. Civ. Code § 1995.260 codifies the
//!   commercial-lease default; residential is governed by case law.
//!
//! **NY RPL § 235-f roommate law**: separately protects a tenant's right
//! to share occupancy with another adult occupant (spouse, family,
//! roommate) regardless of any lease provision restricting occupancy.
//! No "unreasonable withholding" analysis — landlord cannot prevent at
//! all so long as occupancy count doesn't exceed legal limit.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferType {
    /// Subletting — tenant remains liable to landlord; sublessee pays
    /// tenant. Most common scenario.
    Sublet,
    /// Assignment — full transfer of leasehold; tenant typically removed
    /// from liability. Stricter consent regime.
    Assignment,
    /// Roommate addition — tenant retains lease and adds an occupant
    /// (spouse, family, friend) without transferring leasehold.
    RoommateAddition,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SubletConsentRegime {
    /// Statute requires landlord consent that "may not be unreasonably
    /// withheld" — NY RPL § 226-b model.
    StatuteReasonableStandard,
    /// Case law (e.g., Kendall v. Pestana) applies the reasonable
    /// standard.
    CaseLawReasonableStandard,
    /// Lease contract governs; landlord may withhold for any reason
    /// stated in the lease.
    ContractGoverns,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AssignmentConsentRegime {
    /// Owner may UNCONDITIONALLY withhold consent BUT if unreasonably
    /// withheld, tenant may terminate the lease with 30 days notice
    /// (NY § 226-b assignment treatment).
    UnconditionalDiscretionButTerminationRight,
    /// Same reasonable standard as subletting.
    ReasonableStandard,
    /// Contract governs.
    ContractGoverns,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSubletRule {
    pub state_code: &'static str,
    pub state_name: &'static str,
    pub sublet_regime: SubletConsentRegime,
    pub assignment_regime: AssignmentConsentRegime,
    /// True if state has a statute (like NY § 235-f) protecting the
    /// tenant's right to add a roommate / adult occupant.
    pub roommate_addition_protected_statute: bool,
    /// Days landlord has to respond before deemed consent fires.
    /// `None` if no statutory deemed-consent rule.
    pub deemed_consent_window_days: Option<u32>,
    /// Minimum building size for the statutory regime to apply (NY
    /// requires 4+ units for § 226-b sublet protection). `None` if
    /// applies to all rental units.
    pub building_unit_minimum: Option<u32>,
    pub citation: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubletConsentInput {
    pub state_code: String,
    pub transfer_type: TransferType,
    /// Number of residential units in the building. Required for the
    /// NY 4-unit threshold check.
    pub building_unit_count: u32,
    /// True if tenant has formally requested consent from landlord.
    pub consent_request_made: bool,
    pub days_since_consent_request: u32,
    /// True if landlord has refused consent.
    pub landlord_refused_consent: bool,
    /// True if landlord's stated refusal grounds are objectively
    /// reasonable (financial unreliability, criminal history, lease
    /// breach risk). Caller-side determination.
    pub landlord_basis_objectively_reasonable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubletConsentResult {
    /// True if the state statute / case law applies the reasonable-
    /// standard test to the transfer type.
    pub reasonable_standard_applies: bool,
    /// True if the statute's building-size minimum is satisfied (or no
    /// minimum applies).
    pub statute_coverage_satisfied: bool,
    /// True if the statutory deemed-consent window has expired without
    /// landlord response — tenant may proceed with transfer.
    pub deemed_consent_triggered: bool,
    /// True if landlord's refusal would be "unreasonable" under the
    /// applicable regime (only meaningful when reasonable_standard
    /// applies).
    pub refusal_unreasonable: bool,
    /// True if tenant may proceed with the transfer:
    ///   - landlord consented, OR
    ///   - deemed consent triggered, OR
    ///   - reasonable standard applies AND refusal unreasonable
    pub tenant_may_proceed: bool,
    /// For NY assignment: tenant has lease-termination right under
    /// §226-b assignment branch.
    pub tenant_termination_right_available: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn lookup(state_code: &str) -> Option<&'static StateSubletRule> {
    let up = state_code.to_ascii_uppercase();
    TABLE.get(up.as_str()).copied()
}

pub fn all_states() -> Vec<&'static StateSubletRule> {
    let mut v: Vec<&'static StateSubletRule> = TABLE.values().copied().collect();
    v.sort_by_key(|r| r.state_code);
    v
}

pub fn check(input: &SubletConsentInput) -> SubletConsentResult {
    let rule = match lookup(&input.state_code) {
        Some(r) => r,
        None => {
            return SubletConsentResult {
                reasonable_standard_applies: false,
                statute_coverage_satisfied: false,
                deemed_consent_triggered: false,
                refusal_unreasonable: false,
                tenant_may_proceed: false,
                tenant_termination_right_available: false,
                citation: "n/a",
                note: format!("unknown state code `{}`", input.state_code),
            };
        }
    };

    // Roommate addition: short-circuit for states with statutory
    // protection (NY § 235-f).
    if matches!(input.transfer_type, TransferType::RoommateAddition)
        && rule.roommate_addition_protected_statute
    {
        return SubletConsentResult {
            reasonable_standard_applies: true,
            statute_coverage_satisfied: true,
            deemed_consent_triggered: false,
            refusal_unreasonable: false,
            tenant_may_proceed: true,
            tenant_termination_right_available: false,
            citation: rule.citation,
            note: format!(
                "{}: roommate-addition statute (NY RPL § 235-f model) protects right to add adult occupant regardless of lease restrictions",
                rule.state_name
            ),
        };
    }

    // Building-size threshold check.
    let coverage_satisfied = rule
        .building_unit_minimum
        .map(|min| input.building_unit_count >= min)
        .unwrap_or(true);

    // Pick the regime based on transfer type.
    let (reasonable_standard, termination_right) = match input.transfer_type {
        TransferType::Sublet => match rule.sublet_regime {
            SubletConsentRegime::StatuteReasonableStandard
            | SubletConsentRegime::CaseLawReasonableStandard => (true, false),
            SubletConsentRegime::ContractGoverns => (false, false),
        },
        TransferType::Assignment => match rule.assignment_regime {
            AssignmentConsentRegime::ReasonableStandard => (true, false),
            AssignmentConsentRegime::UnconditionalDiscretionButTerminationRight => {
                (false, true) // termination right instead of reasonable standard
            }
            AssignmentConsentRegime::ContractGoverns => (false, false),
        },
        TransferType::RoommateAddition => (false, false),
    };

    let reasonable_standard_applies = reasonable_standard && coverage_satisfied;

    // Deemed consent check.
    let deemed_consent = match rule.deemed_consent_window_days {
        Some(window) => {
            input.consent_request_made
                && input.days_since_consent_request > window
                && !input.landlord_refused_consent
                && coverage_satisfied
        }
        None => false,
    };

    let refusal_unreasonable = reasonable_standard_applies
        && input.landlord_refused_consent
        && !input.landlord_basis_objectively_reasonable;

    let may_proceed = deemed_consent
        || refusal_unreasonable
        || (input.consent_request_made && !input.landlord_refused_consent);

    let termination_right_available = termination_right
        && coverage_satisfied
        && input.landlord_refused_consent
        && !input.landlord_basis_objectively_reasonable;

    let note = match input.transfer_type {
        TransferType::Sublet => {
            if !coverage_satisfied {
                format!(
                    "{}: building has {} units, below statutory {} minimum — statute does not cover; contract governs",
                    rule.state_name,
                    input.building_unit_count,
                    rule.building_unit_minimum.unwrap_or(0)
                )
            } else if deemed_consent {
                format!(
                    "{}: deemed consent triggered — {}d past landlord-response window of {}d; tenant may sublet",
                    rule.state_name,
                    input.days_since_consent_request,
                    rule.deemed_consent_window_days.unwrap_or(0)
                )
            } else if refusal_unreasonable {
                format!(
                    "{}: landlord refused without objectively reasonable basis under {} regime; tenant may proceed under §226-b model",
                    rule.state_name,
                    if matches!(rule.sublet_regime, SubletConsentRegime::CaseLawReasonableStandard) {
                        "case-law reasonable-standard"
                    } else {
                        "statutory reasonable-standard"
                    }
                )
            } else if reasonable_standard_applies && input.landlord_refused_consent {
                format!(
                    "{}: landlord refused with objectively reasonable basis; tenant cannot proceed (refusal is reasonable)",
                    rule.state_name
                )
            } else {
                format!(
                    "{}: sublet consent governed by {} — {}",
                    rule.state_name,
                    match rule.sublet_regime {
                        SubletConsentRegime::StatuteReasonableStandard => "statute reasonable-standard",
                        SubletConsentRegime::CaseLawReasonableStandard => "case-law reasonable-standard",
                        SubletConsentRegime::ContractGoverns => "lease contract",
                    },
                    if may_proceed { "consent granted; may proceed" } else { "consent denied" }
                )
            }
        }
        TransferType::Assignment => {
            if termination_right_available {
                format!(
                    "{}: NY § 226-b assignment branch — landlord may unconditionally withhold, but unreasonable withholding gives tenant LEASE-TERMINATION RIGHT with 30 days notice",
                    rule.state_name
                )
            } else {
                format!(
                    "{}: assignment consent governed by {}",
                    rule.state_name,
                    match rule.assignment_regime {
                        AssignmentConsentRegime::ReasonableStandard => "reasonable-standard",
                        AssignmentConsentRegime::UnconditionalDiscretionButTerminationRight =>
                            "unconditional discretion with termination right",
                        AssignmentConsentRegime::ContractGoverns => "lease contract",
                    }
                )
            }
        }
        TransferType::RoommateAddition => format!(
            "{}: no statewide roommate-addition statute — lease governs occupancy",
            rule.state_name
        ),
    };

    SubletConsentResult {
        reasonable_standard_applies,
        statute_coverage_satisfied: coverage_satisfied,
        deemed_consent_triggered: deemed_consent,
        refusal_unreasonable,
        tenant_may_proceed: may_proceed,
        tenant_termination_right_available: termination_right_available,
        citation: rule.citation,
        note,
    }
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    state_code: &'static str,
    state_name: &'static str,
    sublet_regime: SubletConsentRegime,
    assignment_regime: AssignmentConsentRegime,
    roommate_addition_protected_statute: bool,
    deemed_consent_window_days: Option<u32>,
    building_unit_minimum: Option<u32>,
    citation: &'static str,
) -> StateSubletRule {
    StateSubletRule {
        state_code,
        state_name,
        sublet_regime,
        assignment_regime,
        roommate_addition_protected_statute,
        deemed_consent_window_days,
        building_unit_minimum,
        citation,
    }
}

static TABLE: Lazy<HashMap<&'static str, &'static StateSubletRule>> = Lazy::new(|| {
    use AssignmentConsentRegime as A;
    use SubletConsentRegime as S;
    static RULES: &[StateSubletRule] = &[
        rule("AK", "Alaska", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("AL", "Alabama", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("AR", "Arkansas", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("AZ", "Arizona", S::ContractGoverns, A::ContractGoverns, false, None, None, "A.R.S. § 33-1378 (lease governs)"),
        rule(
            "CA",
            "California",
            S::CaseLawReasonableStandard,
            A::ReasonableStandard,
            false,
            None,
            None,
            "Kendall v. Ernest Pestana, Inc. (1985) + Cal. Civ. Code § 1995.260",
        ),
        rule("CO", "Colorado", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("CT", "Connecticut", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule(
            "DC",
            "District of Columbia",
            S::StatuteReasonableStandard,
            A::ReasonableStandard,
            false,
            Some(30),
            None,
            "D.C. Code § 42-3505.55",
        ),
        rule("DE", "Delaware", S::ContractGoverns, A::ContractGoverns, false, None, None, "25 Del. C. § 5511 (lease governs)"),
        rule("FL", "Florida", S::ContractGoverns, A::ContractGoverns, false, None, None, "Fla. Stat. § 83.49 (lease governs)"),
        rule("GA", "Georgia", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("HI", "Hawaii", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("IA", "Iowa", S::ContractGoverns, A::ContractGoverns, false, None, None, "Iowa Code § 562A.21 (lease governs)"),
        rule("ID", "Idaho", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("IL", "Illinois", S::ContractGoverns, A::ContractGoverns, false, None, None, "Chicago RLTO § 5-12-120 (local)"),
        rule("IN", "Indiana", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("KS", "Kansas", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("KY", "Kentucky", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("LA", "Louisiana", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("MA", "Massachusetts", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("MD", "Maryland", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("ME", "Maine", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("MI", "Michigan", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("MN", "Minnesota", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("MO", "Missouri", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("MS", "Mississippi", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("MT", "Montana", S::ContractGoverns, A::ContractGoverns, false, None, None, "Mont. Code § 70-24-305 (lease governs)"),
        rule("NC", "North Carolina", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("ND", "North Dakota", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("NE", "Nebraska", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("NH", "New Hampshire", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("NJ", "New Jersey", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("NM", "New Mexico", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("NV", "Nevada", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule(
            "NY",
            "New York",
            S::StatuteReasonableStandard,
            A::UnconditionalDiscretionButTerminationRight,
            true, // RPL § 235-f roommate law
            Some(30),
            Some(4),
            "RPL § 226-b (sublet/assign) + § 235-f (roommate law)",
        ),
        rule("OH", "Ohio", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("OK", "Oklahoma", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("OR", "Oregon", S::ContractGoverns, A::ContractGoverns, false, None, None, "ORS § 90.222 (lease governs)"),
        rule("PA", "Pennsylvania", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("RI", "Rhode Island", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("SC", "South Carolina", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("SD", "South Dakota", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("TN", "Tennessee", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("TX", "Texas", S::ContractGoverns, A::ContractGoverns, false, None, None, "Tex. Prop. Code § 91.005 (lease governs)"),
        rule("UT", "Utah", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule(
            "VA",
            "Virginia",
            S::StatuteReasonableStandard,
            A::ReasonableStandard,
            false,
            None,
            None,
            "Va. Code § 55.1-1224",
        ),
        rule("VT", "Vermont", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule(
            "WA",
            "Washington",
            S::StatuteReasonableStandard,
            A::ReasonableStandard,
            false,
            None,
            None,
            "RCW § 59.18.230",
        ),
        rule("WI", "Wisconsin", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("WV", "West Virginia", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
        rule("WY", "Wyoming", S::ContractGoverns, A::ContractGoverns, false, None, None, "no statewide statute"),
    ];
    RULES.iter().map(|r| (r.state_code, r)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        state: &str,
        transfer: TransferType,
        units: u32,
        days: u32,
        refused: bool,
        reasonable_basis: bool,
    ) -> SubletConsentInput {
        SubletConsentInput {
            state_code: state.to_string(),
            transfer_type: transfer,
            building_unit_count: units,
            consent_request_made: true,
            days_since_consent_request: days,
            landlord_refused_consent: refused,
            landlord_basis_objectively_reasonable: reasonable_basis,
        }
    }

    #[test]
    fn table_covers_all_50_states_plus_dc() {
        assert_eq!(TABLE.len(), 51);
    }

    #[test]
    fn ny_sublet_reasonable_standard_applies_at_4_units() {
        // 4-unit building meets NY § 226-b threshold.
        let r = check(&input("NY", TransferType::Sublet, 4, 0, false, false));
        assert!(r.reasonable_standard_applies);
        assert!(r.statute_coverage_satisfied);
    }

    #[test]
    fn ny_sublet_does_not_apply_below_4_units() {
        // 3-unit building below § 226-b threshold → contract governs.
        let r = check(&input("NY", TransferType::Sublet, 3, 0, false, false));
        assert!(!r.reasonable_standard_applies);
        assert!(!r.statute_coverage_satisfied);
        assert!(r.note.contains("below statutory 4 minimum"));
    }

    #[test]
    fn ny_deemed_consent_at_day_31_after_request() {
        // NY 30-day window — at day 31 without landlord response =
        // deemed consent.
        let r = check(&input("NY", TransferType::Sublet, 10, 31, false, false));
        assert!(r.deemed_consent_triggered);
        assert!(r.tenant_may_proceed);
        assert!(r.note.contains("deemed consent"));
    }

    #[test]
    fn ny_deemed_consent_not_at_day_30() {
        // Day 30 = within window; deemed consent does NOT fire yet.
        let r = check(&input("NY", TransferType::Sublet, 10, 30, false, false));
        assert!(!r.deemed_consent_triggered);
    }

    #[test]
    fn ny_unreasonable_refusal_tenant_may_proceed() {
        // Landlord refused but basis is NOT objectively reasonable →
        // refusal unreasonable, tenant may proceed.
        let r = check(&input("NY", TransferType::Sublet, 10, 5, true, false));
        assert!(r.refusal_unreasonable);
        assert!(r.tenant_may_proceed);
    }

    #[test]
    fn ny_reasonable_refusal_tenant_blocked() {
        // Landlord refused with objectively reasonable basis (financial
        // unreliability of subletee) → refusal not unreasonable; tenant
        // cannot proceed.
        let r = check(&input("NY", TransferType::Sublet, 10, 5, true, true));
        assert!(!r.refusal_unreasonable);
        assert!(!r.tenant_may_proceed);
    }

    #[test]
    fn ny_assignment_unreasonable_refusal_gives_termination_right() {
        // Assignment branch: unreasonable refusal does NOT let tenant
        // proceed with assignment, but DOES trigger 30-day lease
        // termination right per § 226-b assignment treatment.
        let r = check(&input("NY", TransferType::Assignment, 10, 5, true, false));
        assert!(r.tenant_termination_right_available);
        // For assignment, "tenant may proceed" means may terminate, not
        // may assign.
        assert!(r.note.contains("LEASE-TERMINATION RIGHT"));
    }

    #[test]
    fn ny_assignment_reasonable_refusal_no_termination_right() {
        let r = check(&input("NY", TransferType::Assignment, 10, 5, true, true));
        assert!(!r.tenant_termination_right_available);
    }

    #[test]
    fn ny_roommate_addition_protected_by_section_235f() {
        // NY § 235-f roommate law — short-circuit applies; tenant may
        // always add roommate regardless of lease.
        let r = check(&input("NY", TransferType::RoommateAddition, 10, 0, false, false));
        assert!(r.tenant_may_proceed);
        assert!(r.note.contains("§ 235-f"));
    }

    #[test]
    fn ca_case_law_reasonable_standard_applies() {
        // CA Kendall v. Pestana applies reasonable-standard via case law.
        let r = check(&input("CA", TransferType::Sublet, 10, 5, true, false));
        assert!(r.reasonable_standard_applies);
        assert!(r.refusal_unreasonable);
        assert!(r.tenant_may_proceed);
    }

    #[test]
    fn ca_no_building_size_threshold() {
        // CA case law applies regardless of building size. Even a
        // single-family rental gets the reasonable standard.
        let r = check(&input("CA", TransferType::Sublet, 1, 5, true, false));
        assert!(r.reasonable_standard_applies);
        assert!(r.statute_coverage_satisfied);
    }

    #[test]
    fn ca_assignment_uses_reasonable_standard_not_unconditional() {
        // CA treats assignment the same as sublet under Kendall —
        // reasonable standard for both. Distinct from NY which has
        // asymmetric regimes.
        let r = check(&input("CA", TransferType::Assignment, 1, 5, true, false));
        assert!(r.reasonable_standard_applies);
        assert!(r.tenant_may_proceed);
        // No termination-right path since CA uses ReasonableStandard not
        // the NY UnconditionalDiscretionButTerminationRight variant.
        assert!(!r.tenant_termination_right_available);
    }

    #[test]
    fn ca_roommate_no_statute_falls_through() {
        // CA does not have an equivalent to NY § 235-f.
        let r = check(&input("CA", TransferType::RoommateAddition, 10, 0, false, false));
        assert!(r.note.contains("no statewide roommate-addition statute"));
    }

    #[test]
    fn tx_contract_governs_no_reasonable_standard() {
        // TX contract governs. Landlord may refuse for any reason.
        let r = check(&input("TX", TransferType::Sublet, 100, 100, true, false));
        assert!(!r.reasonable_standard_applies);
        assert!(!r.refusal_unreasonable);
        assert!(!r.tenant_may_proceed);
    }

    #[test]
    fn dc_30_day_window_deemed_consent_at_31_days() {
        // DC § 42-3505.55 — 30-day deemed consent window.
        let r = check(&input("DC", TransferType::Sublet, 1, 31, false, false));
        assert!(r.deemed_consent_triggered);
    }

    #[test]
    fn va_statute_reasonable_standard_no_unit_threshold() {
        // VA Code § 55.1-1224 — reasonable standard, no building size
        // threshold.
        let r = check(&input("VA", TransferType::Sublet, 1, 5, true, false));
        assert!(r.reasonable_standard_applies);
        assert!(r.refusal_unreasonable);
    }

    #[test]
    fn wa_statute_reasonable_standard_applies() {
        let r = check(&input("WA", TransferType::Sublet, 1, 5, true, false));
        assert!(r.reasonable_standard_applies);
    }

    #[test]
    fn unknown_state_handled() {
        let r = check(&input("ZZ", TransferType::Sublet, 10, 5, true, false));
        assert!(r.note.contains("unknown state code"));
    }

    #[test]
    fn lookup_case_insensitive() {
        assert!(lookup("NY").is_some());
        assert!(lookup("ny").is_some());
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
    fn reasonable_standard_states_pinned() {
        // NY / DC / VA / WA have StatuteReasonableStandard for sublet.
        // CA has CaseLawReasonableStandard.
        for code in ["NY", "DC", "VA", "WA"] {
            let r = lookup(code).unwrap();
            assert!(
                matches!(r.sublet_regime, SubletConsentRegime::StatuteReasonableStandard),
                "{code} should be StatuteReasonableStandard"
            );
        }
        let ca = lookup("CA").unwrap();
        assert!(matches!(
            ca.sublet_regime,
            SubletConsentRegime::CaseLawReasonableStandard
        ));
    }

    #[test]
    fn ny_only_state_with_unit_threshold_for_sublet() {
        // NY uniquely has the 4-unit building threshold for § 226-b.
        let ny = lookup("NY").unwrap();
        assert_eq!(ny.building_unit_minimum, Some(4));
        for r in TABLE.values() {
            if r.state_code != "NY" {
                assert!(
                    r.building_unit_minimum.is_none(),
                    "{} should not have building_unit_minimum",
                    r.state_code
                );
            }
        }
    }

    #[test]
    fn ny_only_state_with_assignment_unconditional_plus_termination() {
        // NY uniquely has the asymmetric assignment regime.
        let ny = lookup("NY").unwrap();
        assert!(matches!(
            ny.assignment_regime,
            AssignmentConsentRegime::UnconditionalDiscretionButTerminationRight
        ));
        for r in TABLE.values() {
            if r.state_code != "NY" {
                assert!(
                    !matches!(
                        r.assignment_regime,
                        AssignmentConsentRegime::UnconditionalDiscretionButTerminationRight
                    ),
                    "{} should not have UnconditionalDiscretion assignment regime",
                    r.state_code
                );
            }
        }
    }

    #[test]
    fn ny_only_state_with_roommate_statute() {
        let ny = lookup("NY").unwrap();
        assert!(ny.roommate_addition_protected_statute);
        for r in TABLE.values() {
            if r.state_code != "NY" {
                assert!(
                    !r.roommate_addition_protected_statute,
                    "{} should not have roommate statute",
                    r.state_code
                );
            }
        }
    }
}
