//! State pre-move-out inspection landlord compliance check. California
//! is currently the only state with a statutory pre-move-out inspection
//! regime (Cal. Civ. Code § 1950.5(f)), but it imposes substantial
//! procedural obligations that materially affect security-deposit
//! disputes. Distinct from `move_in_inspection` (which addresses
//! START-of-tenancy condition documentation) and from
//! `abandoned_property_handling` (which addresses belongings left
//! behind).
//!
//! California Civ. Code § 1950.5(f) — Initial inspection before tenant
//! moves out. Within a reasonable time after notification of either
//! party's intention to terminate the tenancy or before the end of the
//! lease term, the landlord MUST notify the tenant IN WRITING of the
//! tenant's option to request an initial inspection AND of the tenant's
//! right to be present at the inspection. Upon the tenant's request,
//! the landlord must conduct the inspection at a reasonable time but
//! NO EARLIER THAN 2 WEEKS BEFORE the termination or end-of-lease date.
//! Based on the inspection, the landlord must provide an ITEMIZED
//! STATEMENT specifying proposed repairs/cleanings that would be the
//! basis for deductions from the security deposit. The tenant has a
//! CURE PERIOD until termination to remedy identified deficiencies.
//!
//! KEY LIMITATION: if an initial inspection is conducted and the
//! premises do not contain tenant possessions preventing landlord from
//! identifying repairs/cleanings, the landlord SHALL NOT use the
//! security deposit for deductions NOT IDENTIFIED in the itemized
//! statement. This converts pre-move-out inspection into a
//! waiver-of-undisclosed-damages mechanism for landlords who skip the
//! process.
//!
//! Default — no statutory pre-move-out inspection requirement. Landlord
//! conducts post-move-out inspection only; state-specific
//! security-deposit-itemization rules (TX 30-day + itemized list, MA
//! 30-day, MD 45-day) apply on the back end.
//!
//! Citations: Cal. Civ. Code § 1950.5(f)(1) (notice of right to request
//! inspection); § 1950.5(f)(2) (2-week earliest timing); § 1950.5(f)(3)
//! (itemized statement requirement); § 1950.5(f)(4) (waiver of
//! undisclosed-damages deductions).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    CaliforniaCivCode19505F,
    Default,
}

impl Regime {
    pub fn for_state(state: &str) -> Self {
        match state.trim().to_ascii_uppercase().as_str() {
            "CA" => Self::CaliforniaCivCode19505F,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PreMoveOutInspectionInput {
    pub regime: Regime,
    /// Whether landlord notified tenant in writing of the tenant's right
    /// to request an initial inspection and to be present.
    pub written_notice_of_inspection_right_provided: bool,
    /// Whether the tenant requested an initial inspection.
    pub tenant_requested_inspection: bool,
    /// Whether the landlord conducted the inspection at the tenant's
    /// request.
    pub inspection_conducted: bool,
    /// Days between the inspection and the lease termination date. Must
    /// be ≤ 14 (2 weeks) under § 1950.5(f)(2).
    pub days_between_inspection_and_termination: u32,
    /// Whether the landlord provided the itemized statement of proposed
    /// deductions after the inspection.
    pub itemized_statement_provided: bool,
    /// Whether the tenant had possessions in the premises that prevented
    /// the landlord from identifying repairs/cleanings during the
    /// inspection. Drives the § 1950.5(f)(4) waiver-of-undisclosed-
    /// damages mechanic.
    pub premises_had_blocking_tenant_possessions: bool,
    /// Whether the landlord, at move-out, attempted to deduct items NOT
    /// listed on the pre-move-out itemized statement. Triggers the
    /// § 1950.5(f)(4) waiver violation when premises were clear.
    pub deducted_for_items_not_on_itemized_statement: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    None,
    MissingWrittenNoticeOfInspectionRight,
    InspectionTooEarly,
    MissingItemizedStatement,
    /// § 1950.5(f)(4) waiver: landlord cannot deduct for items not on
    /// the itemized statement when premises were clear of tenant
    /// possessions during inspection.
    DeductedForUndisclosedDamagesWaiver,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PreMoveOutInspectionResult {
    pub regime: Regime,
    pub statute_applies: bool,
    pub maximum_days_before_termination: u32,
    pub waiver_of_undisclosed_damages_applies: bool,
    pub violation: ViolationType,
    pub landlord_compliant: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn check(input: &PreMoveOutInspectionInput) -> PreMoveOutInspectionResult {
    match input.regime {
        Regime::CaliforniaCivCode19505F => ca_check(input),
        Regime::Default => default_check(input),
    }
}

fn ca_check(input: &PreMoveOutInspectionInput) -> PreMoveOutInspectionResult {
    // Step 1: notice of inspection right must be provided by the
    // landlord regardless of whether the tenant requests an inspection.
    if !input.written_notice_of_inspection_right_provided {
        return PreMoveOutInspectionResult {
            regime: Regime::CaliforniaCivCode19505F,
            statute_applies: true,
            maximum_days_before_termination: 14,
            waiver_of_undisclosed_damages_applies: false,
            violation: ViolationType::MissingWrittenNoticeOfInspectionRight,
            landlord_compliant: false,
            citation: "Cal. Civ. Code § 1950.5(f)(1) — landlord must notify tenant in writing of right to request initial inspection and to be present",
            note: "Landlord did not provide the required § 1950.5(f)(1) written notice of tenant's right to request initial inspection.".to_string(),
        };
    }

    // Step 2: if tenant requested + landlord conducted inspection, check
    // timing.
    if input.tenant_requested_inspection && input.inspection_conducted {
        if input.days_between_inspection_and_termination > 14 {
            return PreMoveOutInspectionResult {
                regime: Regime::CaliforniaCivCode19505F,
                statute_applies: true,
                maximum_days_before_termination: 14,
                waiver_of_undisclosed_damages_applies: false,
                violation: ViolationType::InspectionTooEarly,
                landlord_compliant: false,
                citation: "Cal. Civ. Code § 1950.5(f)(2) — inspection must be conducted no earlier than 2 weeks (14 days) before termination",
                note: format!(
                    "Inspection conducted {} days before termination; § 1950.5(f)(2) caps at 14 days.",
                    input.days_between_inspection_and_termination
                ),
            };
        }

        if !input.itemized_statement_provided {
            return PreMoveOutInspectionResult {
                regime: Regime::CaliforniaCivCode19505F,
                statute_applies: true,
                maximum_days_before_termination: 14,
                waiver_of_undisclosed_damages_applies: false,
                violation: ViolationType::MissingItemizedStatement,
                landlord_compliant: false,
                citation: "Cal. Civ. Code § 1950.5(f)(3) — landlord must provide itemized statement specifying proposed repairs and cleanings",
                note: "Landlord did not provide the required itemized statement after the inspection.".to_string(),
            };
        }

        // § 1950.5(f)(4) waiver: if premises were CLEAR of tenant
        // possessions AND landlord did the inspection AND the landlord
        // tries to deduct undisclosed items → waiver violation.
        let waiver_applies = !input.premises_had_blocking_tenant_possessions
            && input.itemized_statement_provided;
        if waiver_applies && input.deducted_for_items_not_on_itemized_statement {
            return PreMoveOutInspectionResult {
                regime: Regime::CaliforniaCivCode19505F,
                statute_applies: true,
                maximum_days_before_termination: 14,
                waiver_of_undisclosed_damages_applies: true,
                violation: ViolationType::DeductedForUndisclosedDamagesWaiver,
                landlord_compliant: false,
                citation: "Cal. Civ. Code § 1950.5(f)(4) — if premises were clear of tenant possessions at inspection, landlord shall NOT use security deposit for deductions not identified in itemized statement",
                note: "Premises were clear of tenant possessions at inspection; landlord attempted to deduct for items NOT identified in the itemized statement — § 1950.5(f)(4) waiver violation.".to_string(),
            };
        }

        return PreMoveOutInspectionResult {
            regime: Regime::CaliforniaCivCode19505F,
            statute_applies: true,
            maximum_days_before_termination: 14,
            waiver_of_undisclosed_damages_applies: waiver_applies,
            violation: ViolationType::None,
            landlord_compliant: true,
            citation: "Cal. Civ. Code § 1950.5(f) — pre-move-out inspection compliance OK",
            note: format!(
                "Pre-move-out inspection conducted within 14-day window with itemized statement provided. Waiver of undisclosed damages {}.",
                if waiver_applies { "APPLIES (premises were clear)" } else { "does NOT apply (premises had blocking possessions)" }
            ),
        };
    }

    // Notice provided but tenant did not request inspection — landlord
    // is compliant with the statute's notice requirement; § 1950.5(f)(4)
    // waiver doesn't apply because no inspection occurred.
    PreMoveOutInspectionResult {
        regime: Regime::CaliforniaCivCode19505F,
        statute_applies: true,
        maximum_days_before_termination: 14,
        waiver_of_undisclosed_damages_applies: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: "Cal. Civ. Code § 1950.5(f)(1) — notice of inspection right provided; tenant did not exercise the option (or no inspection occurred)",
        note: "Required § 1950.5(f)(1) written notice provided. Tenant did not request inspection or no inspection occurred. Landlord retains full deduction rights under standard § 1950.5(b) at move-out.".to_string(),
    }
}

fn default_check(_input: &PreMoveOutInspectionInput) -> PreMoveOutInspectionResult {
    PreMoveOutInspectionResult {
        regime: Regime::Default,
        statute_applies: false,
        maximum_days_before_termination: 0,
        waiver_of_undisclosed_damages_applies: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation:
            "No statewide pre-move-out inspection statute identified — landlord conducts post-move-out inspection only; standard state security-deposit itemization timelines apply on back-end",
        note: "Default regime: no pre-move-out inspection obligation. Landlord conducts post-move-out inspection. State-specific security-deposit itemization rules apply (e.g., TX 30-day + itemized list, MA 30-day, MD 45-day).".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        notice: bool,
        requested: bool,
        conducted: bool,
        days_early: u32,
        itemized: bool,
        blocked: bool,
        deducted_undisclosed: bool,
    ) -> PreMoveOutInspectionInput {
        PreMoveOutInspectionInput {
            regime,
            written_notice_of_inspection_right_provided: notice,
            tenant_requested_inspection: requested,
            inspection_conducted: conducted,
            days_between_inspection_and_termination: days_early,
            itemized_statement_provided: itemized,
            premises_had_blocking_tenant_possessions: blocked,
            deducted_for_items_not_on_itemized_statement: deducted_undisclosed,
        }
    }

    #[test]
    fn ca_missing_notice_of_right_violation() {
        let r = check(&input(
            Regime::CaliforniaCivCode19505F,
            false,
            false,
            false,
            0,
            false,
            false,
            false,
        ));
        assert_eq!(
            r.violation,
            ViolationType::MissingWrittenNoticeOfInspectionRight
        );
        assert!(r.citation.contains("§ 1950.5(f)(1)"));
    }

    #[test]
    fn ca_notice_provided_no_request_compliant() {
        // Notice given; tenant did not request inspection → compliant.
        let r = check(&input(
            Regime::CaliforniaCivCode19505F,
            true,
            false,
            false,
            0,
            false,
            false,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.landlord_compliant);
        assert!(!r.waiver_of_undisclosed_damages_applies);
    }

    #[test]
    fn ca_inspection_15_days_before_termination_too_early() {
        let r = check(&input(
            Regime::CaliforniaCivCode19505F,
            true,
            true,
            true,
            15,
            true,
            false,
            false,
        ));
        assert_eq!(r.violation, ViolationType::InspectionTooEarly);
        assert!(r.citation.contains("§ 1950.5(f)(2)"));
        assert!(r.note.contains("15 days"));
    }

    #[test]
    fn ca_inspection_at_14_day_boundary_compliant() {
        let r = check(&input(
            Regime::CaliforniaCivCode19505F,
            true,
            true,
            true,
            14,
            true,
            false,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn ca_inspection_5_days_compliant() {
        let r = check(&input(
            Regime::CaliforniaCivCode19505F,
            true,
            true,
            true,
            5,
            true,
            false,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn ca_missing_itemized_statement_violation() {
        let r = check(&input(
            Regime::CaliforniaCivCode19505F,
            true,
            true,
            true,
            7,
            false,
            false,
            false,
        ));
        assert_eq!(r.violation, ViolationType::MissingItemizedStatement);
        assert!(r.citation.contains("§ 1950.5(f)(3)"));
    }

    #[test]
    fn ca_premises_clear_with_undisclosed_deduction_waiver_violation() {
        let r = check(&input(
            Regime::CaliforniaCivCode19505F,
            true,
            true,
            true,
            7,
            true,
            false,
            true,
        ));
        assert_eq!(
            r.violation,
            ViolationType::DeductedForUndisclosedDamagesWaiver
        );
        assert!(r.waiver_of_undisclosed_damages_applies);
        assert!(r.citation.contains("§ 1950.5(f)(4)"));
    }

    #[test]
    fn ca_premises_blocked_no_waiver_undisclosed_deduction_allowed() {
        // Premises had tenant possessions blocking inspection — § 1950.5(f)(4)
        // waiver does NOT apply, so landlord CAN deduct for items not on
        // the itemized statement.
        let r = check(&input(
            Regime::CaliforniaCivCode19505F,
            true,
            true,
            true,
            7,
            true,
            true,
            true,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(!r.waiver_of_undisclosed_damages_applies);
    }

    #[test]
    fn ca_full_compliance_premises_clear_no_undisclosed_deduction() {
        let r = check(&input(
            Regime::CaliforniaCivCode19505F,
            true,
            true,
            true,
            7,
            true,
            false,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.waiver_of_undisclosed_damages_applies);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn default_no_obligation() {
        let r = check(&input(
            Regime::Default,
            false,
            false,
            false,
            0,
            false,
            false,
            false,
        ));
        assert!(!r.statute_applies);
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.landlord_compliant);
        assert!(r.citation.contains("No statewide pre-move-out inspection statute"));
    }

    #[test]
    fn default_landlord_can_deduct_undisclosed_no_waiver() {
        // Default regime — even if landlord didn't inspect, no waiver
        // applies, and undisclosed deductions are allowed (subject to
        // back-end itemization rules).
        let r = check(&input(
            Regime::Default,
            false,
            false,
            false,
            0,
            false,
            false,
            true,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(!r.waiver_of_undisclosed_damages_applies);
    }

    #[test]
    fn state_routing_ca_default() {
        assert_eq!(
            Regime::for_state("CA"),
            Regime::CaliforniaCivCode19505F
        );
        assert_eq!(Regime::for_state("TX"), Regime::Default);
        assert_eq!(Regime::for_state("NY"), Regime::Default);
        assert_eq!(Regime::for_state("FL"), Regime::Default);
    }

    #[test]
    fn state_routing_case_insensitive() {
        assert_eq!(
            Regime::for_state("ca"),
            Regime::CaliforniaCivCode19505F
        );
    }

    #[test]
    fn ca_notice_requirement_does_not_depend_on_tenant_request() {
        // Landlord must give notice REGARDLESS of whether tenant requests.
        let r_no_request = check(&input(
            Regime::CaliforniaCivCode19505F,
            false,
            false,
            false,
            0,
            false,
            false,
            false,
        ));
        assert_eq!(
            r_no_request.violation,
            ViolationType::MissingWrittenNoticeOfInspectionRight
        );
    }

    #[test]
    fn ca_one_day_above_14_too_early() {
        let r = check(&input(
            Regime::CaliforniaCivCode19505F,
            true,
            true,
            true,
            15,
            true,
            false,
            false,
        ));
        assert_eq!(r.violation, ViolationType::InspectionTooEarly);
    }

    #[test]
    fn ca_zero_days_inspection_same_day_compliant() {
        // Same-day inspection (0 days before termination) is permitted —
        // statute only caps the EARLIEST date, not the latest.
        let r = check(&input(
            Regime::CaliforniaCivCode19505F,
            true,
            true,
            true,
            0,
            true,
            false,
            false,
        ));
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn ca_waiver_only_when_premises_clear_and_itemized_provided() {
        // Three conditions for § 1950.5(f)(4) waiver: notice + inspection
        // + clear premises. Missing premises-clear → no waiver.
        let r = check(&input(
            Regime::CaliforniaCivCode19505F,
            true,
            true,
            true,
            7,
            true,
            true, // blocked
            true,
        ));
        // Blocked premises → no waiver applies → no violation despite
        // undisclosed deduction.
        assert_eq!(r.violation, ViolationType::None);
        assert!(!r.waiver_of_undisclosed_damages_applies);
    }

    #[test]
    fn only_ca_has_pre_move_out_inspection_obligation() {
        // Same no-notice scenario across regimes.
        let ca = check(&input(
            Regime::CaliforniaCivCode19505F,
            false,
            false,
            false,
            0,
            false,
            false,
            false,
        ));
        let d = check(&input(
            Regime::Default,
            false,
            false,
            false,
            0,
            false,
            false,
            false,
        ));
        assert_eq!(
            ca.violation,
            ViolationType::MissingWrittenNoticeOfInspectionRight
        );
        assert_eq!(d.violation, ViolationType::None);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let r_notice = check(&input(
            Regime::CaliforniaCivCode19505F,
            false,
            false,
            false,
            0,
            false,
            false,
            false,
        ));
        assert!(r_notice.citation.contains("§ 1950.5(f)(1)"));

        let r_timing = check(&input(
            Regime::CaliforniaCivCode19505F,
            true,
            true,
            true,
            20,
            true,
            false,
            false,
        ));
        assert!(r_timing.citation.contains("§ 1950.5(f)(2)"));
        assert!(r_timing.citation.contains("14 days"));

        let r_itemized = check(&input(
            Regime::CaliforniaCivCode19505F,
            true,
            true,
            true,
            7,
            false,
            false,
            false,
        ));
        assert!(r_itemized.citation.contains("§ 1950.5(f)(3)"));

        let r_waiver = check(&input(
            Regime::CaliforniaCivCode19505F,
            true,
            true,
            true,
            7,
            true,
            false,
            true,
        ));
        assert!(r_waiver.citation.contains("§ 1950.5(f)(4)"));
    }

    #[test]
    fn ca_no_inspection_no_waiver_even_premises_clear() {
        // If tenant never requested inspection (so none was conducted),
        // the § 1950.5(f)(4) waiver does NOT apply — landlord retains
        // full back-end deduction rights.
        let r = check(&input(
            Regime::CaliforniaCivCode19505F,
            true,
            false, // tenant didn't request
            false,
            0,
            false,
            false,
            true, // tried to deduct undisclosed
        ));
        // No waiver, no violation.
        assert_eq!(r.violation, ViolationType::None);
        assert!(!r.waiver_of_undisclosed_damages_applies);
    }
}
