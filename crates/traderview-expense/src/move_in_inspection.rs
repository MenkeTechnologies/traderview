//! State move-in / move-out inspection requirements.
//!
//! Direct trader-landlord deposit-return liability. Several states
//! statutorily mandate a written move-in condition checklist signed
//! by both parties. Failure to provide the checklist exposes the
//! landlord to forfeit of the entire security deposit, regardless of
//! whether the deductions were otherwise justified.
//!
//! Four regimes:
//!
//! 1. **`MandatoryMoveInChecklist`** — WA (RCW 59.18.260), AZ (ARS
//!    § 33-1321), MI (MCL 554.608), KY (KRS 383.580(2)). Written
//!    checklist describing condition / damages must be provided to
//!    the tenant at the commencement of tenancy. WA is the strictest:
//!    failure to provide the checklist makes the landlord LIABLE
//!    FOR THE FULL DEPOSIT plus attorney's fees and court costs.
//!
//! 2. **`TenantRequestedMoveInChecklist`** — MD (Real Property
//!    § 8-203.1). Landlord must provide a written list of pre-existing
//!    damages ONLY IF the tenant requests it within 15 days of
//!    occupancy. No automatic obligation absent request.
//!
//! 3. **`PreMoveOutInspectionOffer`** — CA (Civ. Code § 1950.5(f)).
//!    Distinct from move-in regimes: landlord must OFFER the tenant
//!    a pre-move-out walk-through inspection upon proper request,
//!    giving the tenant the opportunity to cure curable deficiencies
//!    before the final deposit deduction. No move-in checklist
//!    mandated.
//!
//! 4. **`NoStateRequirement`** — most other states. Federal Fair
//!    Housing Act / state common-law landlord-tenant principles
//!    still allow either party to demand a walk-through by mutual
//!    agreement, but there's no statutory mandate or specific
//!    penalty for omission.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectionRegime {
    MandatoryMoveInChecklist,
    TenantRequestedMoveInChecklist,
    PreMoveOutInspectionOffer,
    NoStateRequirement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DepositForfeitPenalty {
    /// Failure to provide the checklist forfeits the entire deposit
    /// PLUS attorney's fees + court costs (WA RCW 59.18.260).
    FullDepositPlusAttorneysFees,
    /// Failure forfeits deposit deductions for any undocumented
    /// pre-existing damages (most states).
    PreExistingDamageDeductionsBarred,
    NoSpecificPenalty,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: InspectionRegime,
    /// Statutory window (days from commencement) within which the
    /// landlord must provide the checklist. 0 = "at commencement"
    /// (i.e., before deposit collected); `None` for non-mandatory
    /// regimes.
    pub provide_within_days_of_commencement: Option<u32>,
    /// For tenant-requested regimes: days the tenant has to request.
    pub tenant_request_window_days: Option<u32>,
    pub forfeit_penalty: DepositForfeitPenalty,
    pub citation: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    regime: InspectionRegime,
    provide_within_days_of_commencement: Option<u32>,
    tenant_request_window_days: Option<u32>,
    forfeit_penalty: DepositForfeitPenalty,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        provide_within_days_of_commencement,
        tenant_request_window_days,
        forfeit_penalty,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use DepositForfeitPenalty::*;
    use InspectionRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    // MandatoryMoveInChecklist regime.
    m.insert(
        "WA",
        rule(
            MandatoryMoveInChecklist,
            Some(0), // At commencement before deposit collected
            None,
            FullDepositPlusAttorneysFees,
            "Wash. RCW 59.18.260 — written checklist mandatory; full-deposit forfeit + attorney fees on failure",
        ),
    );
    m.insert(
        "AZ",
        rule(
            MandatoryMoveInChecklist,
            Some(0),
            None,
            PreExistingDamageDeductionsBarred,
            "Ariz. ARS § 33-1321 — move-in inspection form required",
        ),
    );
    m.insert(
        "MI",
        rule(
            MandatoryMoveInChecklist,
            Some(7), // 7-day commencement inventory window
            None,
            PreExistingDamageDeductionsBarred,
            "Mich. MCL 554.608 — commencement inventory checklist; tenant has 7 days to return signed",
        ),
    );
    m.insert(
        "KY",
        rule(
            MandatoryMoveInChecklist,
            Some(0),
            None,
            PreExistingDamageDeductionsBarred,
            "Ky. KRS 383.580(2) — move-in inspection and listing required",
        ),
    );

    // TenantRequestedMoveInChecklist regime.
    m.insert(
        "MD",
        rule(
            TenantRequestedMoveInChecklist,
            None,
            Some(15), // Tenant must request within 15 days
            PreExistingDamageDeductionsBarred,
            "Md. Real Prop. § 8-203.1 — written damage list required upon tenant request within 15 days",
        ),
    );

    // PreMoveOutInspectionOffer regime.
    m.insert(
        "CA",
        rule(
            PreMoveOutInspectionOffer,
            None,
            None,
            PreExistingDamageDeductionsBarred,
            "Cal. Civ. Code § 1950.5(f) — pre-move-out inspection offer required; no move-in checklist mandated",
        ),
    );

    // NoStateRequirement — all remaining states + DC.
    let no_rule_states = [
        "AL", "AK", "AR", "CO", "CT", "DC", "DE", "FL", "GA", "HI",
        "ID", "IL", "IN", "IA", "KS", "LA", "ME", "MA", "MN", "MS",
        "MO", "MT", "NE", "NV", "NH", "NJ", "NM", "NY", "NC", "ND",
        "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT",
        "VT", "VA", "WV", "WI", "WY",
    ];
    for code in no_rule_states {
        m.insert(
            code,
            rule(
                NoStateRequirement,
                None,
                None,
                NoSpecificPenalty,
                "No statewide move-in inspection requirement; common-law walk-through by agreement",
            ),
        );
    }
    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionInput {
    pub state_code: String,
    /// Did the landlord provide a written move-in checklist signed by
    /// both parties?
    pub landlord_provided_move_in_checklist: bool,
    /// Day on which the landlord delivered the checklist relative to
    /// tenancy commencement (0 = same day; negative = before
    /// commencement; large positive = late).
    pub day_landlord_delivered_relative_to_commencement: i64,
    /// For tenant-requested regimes: did the tenant request the
    /// written damage list, and how many days after occupancy?
    pub tenant_requested_damage_list: bool,
    pub tenant_request_day_after_commencement: u32,
    /// For PreMoveOutInspectionOffer regime: did the landlord offer
    /// the pre-move-out walk-through?
    pub landlord_offered_pre_move_out_inspection: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionResult {
    pub regime: InspectionRegime,
    pub statutorily_required: bool,
    pub landlord_compliant: bool,
    pub forfeit_penalty_triggered: DepositForfeitPenalty,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &InspectionInput) -> InspectionResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: InspectionRegime::NoStateRequirement,
        provide_within_days_of_commencement: None,
        tenant_request_window_days: None,
        forfeit_penalty: DepositForfeitPenalty::NoSpecificPenalty,
        citation: "Unknown state code; assuming no statewide requirement",
    });

    let (required, compliant) = match rule.regime {
        InspectionRegime::MandatoryMoveInChecklist => {
            let within_window = match rule.provide_within_days_of_commencement {
                Some(w) => {
                    // Provided within window if delivered on or after
                    // commencement (day >= 0) AND within the statutory
                    // window. Window of 0 means at-or-before commencement.
                    if w == 0 {
                        input.day_landlord_delivered_relative_to_commencement <= 0
                    } else {
                        input.day_landlord_delivered_relative_to_commencement >= 0
                            && input.day_landlord_delivered_relative_to_commencement
                                <= w as i64
                    }
                }
                None => true,
            };
            (
                true,
                input.landlord_provided_move_in_checklist && within_window,
            )
        }
        InspectionRegime::TenantRequestedMoveInChecklist => {
            let request_timely = input.tenant_requested_damage_list
                && rule.tenant_request_window_days.is_some_and(|w| {
                    input.tenant_request_day_after_commencement <= w
                });
            // Only "required" if tenant timely requested.
            if request_timely {
                (true, input.landlord_provided_move_in_checklist)
            } else {
                // No tenant request → not required → compliant by default.
                (false, true)
            }
        }
        InspectionRegime::PreMoveOutInspectionOffer => (
            true,
            input.landlord_offered_pre_move_out_inspection,
        ),
        InspectionRegime::NoStateRequirement => (false, true),
    };

    let forfeit_penalty = if required && !compliant {
        rule.forfeit_penalty
    } else {
        DepositForfeitPenalty::NoSpecificPenalty
    };

    let note = match (rule.regime, required, compliant) {
        (InspectionRegime::MandatoryMoveInChecklist, _, true) => {
            "MandatoryMoveInChecklist: landlord provided checklist within statutory window — compliant.".to_string()
        }
        (InspectionRegime::MandatoryMoveInChecklist, _, false) => {
            format!(
                "MandatoryMoveInChecklist VIOLATION: landlord failed to provide checklist or missed window. Penalty: {:?}.",
                forfeit_penalty,
            )
        }
        (InspectionRegime::TenantRequestedMoveInChecklist, true, true) =>
            "TenantRequestedMoveInChecklist: tenant requested timely; landlord provided checklist — compliant.".to_string(),
        (InspectionRegime::TenantRequestedMoveInChecklist, true, false) =>
            "TenantRequestedMoveInChecklist VIOLATION: tenant requested timely but landlord failed to provide written damage list.".to_string(),
        (InspectionRegime::TenantRequestedMoveInChecklist, false, _) =>
            "TenantRequestedMoveInChecklist: tenant did not request within statutory window; no landlord duty.".to_string(),
        (InspectionRegime::PreMoveOutInspectionOffer, _, true) =>
            "PreMoveOutInspectionOffer: landlord offered pre-move-out walk-through — compliant.".to_string(),
        (InspectionRegime::PreMoveOutInspectionOffer, _, false) =>
            "PreMoveOutInspectionOffer VIOLATION: landlord failed to offer pre-move-out walk-through; tenant lost opportunity to cure curable deficiencies.".to_string(),
        (InspectionRegime::NoStateRequirement, _, _) =>
            "NoStateRequirement: no statewide move-in inspection requirement applies.".to_string(),
    };

    InspectionResult {
        regime: rule.regime,
        statutorily_required: required,
        landlord_compliant: compliant,
        forfeit_penalty_triggered: forfeit_penalty,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str) -> InspectionInput {
        InspectionInput {
            state_code: state.to_string(),
            landlord_provided_move_in_checklist: false,
            day_landlord_delivered_relative_to_commencement: 0,
            tenant_requested_damage_list: false,
            tenant_request_day_after_commencement: 0,
            landlord_offered_pre_move_out_inspection: false,
        }
    }

    // WA — strictest regime with full-deposit penalty.

    #[test]
    fn wa_checklist_at_commencement_complies() {
        let mut i = input("WA");
        i.landlord_provided_move_in_checklist = true;
        i.day_landlord_delivered_relative_to_commencement = 0;
        let r = check(&i);
        assert_eq!(r.regime, InspectionRegime::MandatoryMoveInChecklist);
        assert!(r.statutorily_required);
        assert!(r.landlord_compliant);
        assert_eq!(
            r.forfeit_penalty_triggered,
            DepositForfeitPenalty::NoSpecificPenalty
        );
    }

    #[test]
    fn wa_no_checklist_full_deposit_forfeit_penalty() {
        let i = input("WA");
        let r = check(&i);
        assert!(r.statutorily_required);
        assert!(!r.landlord_compliant);
        assert_eq!(
            r.forfeit_penalty_triggered,
            DepositForfeitPenalty::FullDepositPlusAttorneysFees
        );
        assert!(r.note.contains("VIOLATION"));
    }

    #[test]
    fn wa_late_checklist_violates() {
        // WA window is at commencement (≤ 0); day 1 is late.
        let mut i = input("WA");
        i.landlord_provided_move_in_checklist = true;
        i.day_landlord_delivered_relative_to_commencement = 1;
        let r = check(&i);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn wa_pre_commencement_checklist_complies() {
        // Delivered before commencement is fine.
        let mut i = input("WA");
        i.landlord_provided_move_in_checklist = true;
        i.day_landlord_delivered_relative_to_commencement = -2;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    // MI — MCL 554.608 7-day inventory window.

    #[test]
    fn mi_7_day_window_day_7_complies() {
        let mut i = input("MI");
        i.landlord_provided_move_in_checklist = true;
        i.day_landlord_delivered_relative_to_commencement = 7;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn mi_7_day_window_day_8_violates() {
        let mut i = input("MI");
        i.landlord_provided_move_in_checklist = true;
        i.day_landlord_delivered_relative_to_commencement = 8;
        let r = check(&i);
        assert!(!r.landlord_compliant);
    }

    // AZ, KY.

    #[test]
    fn az_checklist_required_at_commencement() {
        let mut i = input("AZ");
        i.landlord_provided_move_in_checklist = true;
        let r = check(&i);
        assert_eq!(r.regime, InspectionRegime::MandatoryMoveInChecklist);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ky_no_checklist_pre_existing_damage_penalty() {
        let i = input("KY");
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert_eq!(
            r.forfeit_penalty_triggered,
            DepositForfeitPenalty::PreExistingDamageDeductionsBarred
        );
    }

    // MD — TenantRequestedMoveInChecklist.

    #[test]
    fn md_tenant_did_not_request_no_duty() {
        let i = input("MD");
        let r = check(&i);
        assert_eq!(r.regime, InspectionRegime::TenantRequestedMoveInChecklist);
        assert!(!r.statutorily_required);
        assert!(r.landlord_compliant); // No duty without request
    }

    #[test]
    fn md_tenant_requested_within_15_days_landlord_must_provide() {
        let mut i = input("MD");
        i.tenant_requested_damage_list = true;
        i.tenant_request_day_after_commencement = 10;
        let r = check(&i);
        assert!(r.statutorily_required);
        assert!(!r.landlord_compliant); // Did not provide
    }

    #[test]
    fn md_tenant_request_day_15_exact_boundary_complies() {
        let mut i = input("MD");
        i.tenant_requested_damage_list = true;
        i.tenant_request_day_after_commencement = 15;
        i.landlord_provided_move_in_checklist = true;
        let r = check(&i);
        assert!(r.statutorily_required);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn md_tenant_request_day_16_late_no_duty() {
        let mut i = input("MD");
        i.tenant_requested_damage_list = true;
        i.tenant_request_day_after_commencement = 16;
        let r = check(&i);
        assert!(!r.statutorily_required);
        assert!(r.landlord_compliant); // No duty (request was untimely)
    }

    // CA — PreMoveOutInspectionOffer.

    #[test]
    fn ca_pre_move_out_offer_complies() {
        let mut i = input("CA");
        i.landlord_offered_pre_move_out_inspection = true;
        let r = check(&i);
        assert_eq!(r.regime, InspectionRegime::PreMoveOutInspectionOffer);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ca_failure_to_offer_pre_move_out_violates() {
        let i = input("CA");
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(r.note.contains("PreMoveOutInspectionOffer VIOLATION"));
    }

    #[test]
    fn ca_no_move_in_checklist_required_regardless() {
        // CA has no move-in checklist mandate — only pre-move-out.
        // Setting move-in checklist true should not change CA path.
        let mut i = input("CA");
        i.landlord_provided_move_in_checklist = true;
        i.landlord_offered_pre_move_out_inspection = false;
        let r = check(&i);
        // Still violation because pre-move-out wasn't offered.
        assert!(!r.landlord_compliant);
    }

    // NoStateRequirement.

    #[test]
    fn no_state_rule_compliant_by_default() {
        for st in &["TX", "FL", "NY", "PA", "OH", "MA", "OR"] {
            let r = check(&input(st));
            assert_eq!(r.regime, InspectionRegime::NoStateRequirement, "{st}");
            assert!(!r.statutorily_required, "{st}");
            assert!(r.landlord_compliant, "{st}");
        }
    }

    // Coverage / structural pins.

    #[test]
    fn coverage_is_all_50_states_plus_dc() {
        let codes: Vec<&'static str> = RULES.keys().copied().collect();
        assert_eq!(codes.len(), 51, "expected 50 states + DC, got {}", codes.len());
    }

    #[test]
    fn citations_never_empty() {
        for (code, rule) in RULES.iter() {
            assert!(!rule.citation.is_empty(), "{code} missing citation");
        }
    }

    #[test]
    fn unknown_state_falls_back_to_no_requirement() {
        let r = check(&input("XX"));
        assert_eq!(r.regime, InspectionRegime::NoStateRequirement);
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let mut i = input("wa");
        i.landlord_provided_move_in_checklist = true;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn wa_unique_full_deposit_penalty_invariant() {
        // Invariant: WA is the only state with FullDepositPlusAttorneysFees.
        let mut count = 0;
        for rule in RULES.values() {
            if rule.forfeit_penalty == DepositForfeitPenalty::FullDepositPlusAttorneysFees
            {
                count += 1;
            }
        }
        assert_eq!(
            count, 1,
            "expected WA only with FullDepositPlusAttorneysFees penalty"
        );
    }

    #[test]
    fn mandatory_checklist_regime_states_count() {
        // Invariant: WA, AZ, MI, KY are the four MandatoryMoveInChecklist states.
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == InspectionRegime::MandatoryMoveInChecklist {
                count += 1;
            }
        }
        assert_eq!(count, 4, "expected 4 MandatoryMoveInChecklist states");
    }

    #[test]
    fn note_describes_wa_full_penalty_path() {
        let r = check(&input("WA"));
        assert!(r.note.contains("VIOLATION"));
        assert!(r.note.contains("FullDepositPlusAttorneysFees"));
    }

    #[test]
    fn note_describes_md_no_request_path() {
        let r = check(&input("MD"));
        assert!(r.note.contains("did not request"));
    }
}
