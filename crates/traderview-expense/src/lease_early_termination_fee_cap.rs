//! Lease early-termination fee + liquidated damages cap
//! enforceability — when a residential tenant breaks the lease
//! early, what statutory cap or duty-to-mitigate framework limits
//! the landlord's recovery? Distinct from siblings
//! `duty_to_mitigate_damages` (general mitigation rules),
//! `rent_acceleration_enforceability` (full unpaid balance
//! acceleration), `lease_termination_catastrophic_damage`
//! (force-majeure / catastrophic event termination), and
//! `military_termination` (SCRA-covered military service member
//! termination).
//!
//! Trader-landlord operational concern in single-family +
//! multifamily rentals — aggressive lease drafting often imposes
//! liquidated damages or early-termination fee clauses that are
//! capped or unenforceable under state-specific rules.
//!
//! **Three regimes**:
//!
//! **Florida — Fla. Stat. § 83.595(4)**. Most explicit statutory
//! cap. Landlord may charge liquidated damages OR early-
//! termination fee IF (a) landlord and tenant agreed in advance,
//! (b) amount does NOT EXCEED 2 MONTHS' RENT, (c) tenant gives
//! at least 60 DAYS' notice, (d) clause is included as a SEPARATE
//! ADDENDUM containing specific statutory wording. If landlord
//! elects the liquidated-damages / early-termination-fee remedy,
//! landlord WAIVES additional rent beyond month of retaking
//! possession (still may recover accrued rent + property damage
//! charges).
//!
//! **California — Cal. Civ. Code § 1951.2 + § 1671 + duty to
//! mitigate**. No statutory liquidated damages cap. Landlord
//! recovers ACTUAL DAMAGES with strict DUTY TO MITIGATE (find
//! replacement tenant in reasonable time + reasonable effort).
//! Liquidated damages clauses are enforceable ONLY if they
//! represent a reasonable estimate of damages at lease execution
//! (§ 1671(d)) — NOT a penalty. Aggressive enforcement against
//! "penalty" clauses.
//!
//! **Default — common-law actual damages + duty to mitigate**.
//! Most states require landlord to attempt reletting and recover
//! only actual rent loss + reasonable releasing costs. Liquidated
//! damages clauses must represent a reasonable pre-estimate of
//! damages (Restatement (Second) of Contracts § 356).
//!
//! Citations: Fla. Stat. § 83.595 (FL landlord remedies + 2-month
//! liquidated damages cap + 60-day notice + separate addendum
//! requirement + waiver of additional rent); Cal. Civ. Code §
//! 1951.2 (CA actual damages framework + mitigation); Cal. Civ.
//! Code § 1671(d) (CA liquidated damages reasonable estimate
//! test); Restatement (Second) of Contracts § 356 (general
//! liquidated damages vs penalty doctrine).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Florida,
    California,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LeaseEarlyTerminationFeeInput {
    pub regime: Regime,
    /// The early-termination fee / liquidated damages amount the
    /// landlord seeks to charge, in cents.
    pub fee_amount_cents: i64,
    /// Monthly rent in cents — used for 2-month cap math under
    /// Fla. Stat. § 83.595(4).
    pub monthly_rent_cents: i64,
    /// Whether the lease contains a SEPARATE addendum with
    /// statutory wording (FL requirement).
    pub separate_addendum_signed: bool,
    /// Whether the tenant provided at least 60 days' notice prior
    /// to the proposed early termination date (FL requirement).
    pub tenant_provided_60_day_notice: bool,
    /// Whether the landlord actually attempted to mitigate
    /// damages by re-renting the unit (CA + default rule).
    pub landlord_attempted_mitigation: bool,
    /// Actual rent loss after mitigation, in cents (CA + default
    /// rule actual damages computation).
    pub actual_rent_loss_after_mitigation_cents: i64,
    /// Whether the liquidated damages clause represents a
    /// reasonable pre-estimate of damages at lease execution
    /// (CA § 1671(d) + Restatement § 356).
    pub clause_reasonable_pre_estimate_of_damages: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LeaseEarlyTerminationFeeResult {
    pub fee_enforceable: bool,
    /// Whether the fee exceeds the FL 2-month cap (only relevant
    /// to Florida regime).
    pub fl_two_month_cap_violated: bool,
    pub fl_addendum_compliance: bool,
    pub fl_notice_compliance: bool,
    /// Whether the landlord attempted mitigation (relevant to
    /// CA + default regimes).
    pub mitigation_compliance: bool,
    /// Maximum damages recoverable in cents under the regime.
    pub damages_recoverable_cents: i64,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &LeaseEarlyTerminationFeeInput) -> LeaseEarlyTerminationFeeResult {
    match input.regime {
        Regime::Florida => check_florida(input),
        Regime::California => check_california(input),
        Regime::Default => check_default(input),
    }
}

fn check_florida(input: &LeaseEarlyTerminationFeeInput) -> LeaseEarlyTerminationFeeResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    let two_month_cap_cents = input.monthly_rent_cents.saturating_mul(2);
    let cap_violated = input.fee_amount_cents > two_month_cap_cents;

    if cap_violated {
        violations.push(
            "Fla. Stat. § 83.595(4) — early-termination fee / liquidated damages may NOT EXCEED 2 MONTHS' RENT"
                .to_string(),
        );
    }

    if !input.separate_addendum_signed {
        violations.push(
            "Fla. Stat. § 83.595(4) — early-termination fee clause MUST be included as a SEPARATE ADDENDUM containing specific statutory wording"
                .to_string(),
        );
    }

    if !input.tenant_provided_60_day_notice {
        violations.push(
            "Fla. Stat. § 83.595(4) — tenant must provide at least 60 DAYS' notice prior to proposed early termination date"
                .to_string(),
        );
    }

    notes.push(
        "Fla. Stat. § 83.595(2) — landlord 'menu of remedies' upon early termination: (a) treat lease as terminated and retake possession, (b) re-rent on tenant's account and recover difference, (c) hold for full rent as it becomes due, OR (d) charge liquidated damages / early-termination fee under § 83.595(4)"
            .to_string(),
    );
    notes.push(
        "Fla. Stat. § 83.595(4) — election of liquidated damages / early-termination fee remedy WAIVES additional rent beyond month of retaking possession (still recovers accrued rent + property damage charges)"
            .to_string(),
    );

    let fee_enforceable = violations.is_empty();
    let damages_recoverable = if fee_enforceable {
        input.fee_amount_cents.min(two_month_cap_cents)
    } else {
        0
    };

    LeaseEarlyTerminationFeeResult {
        fee_enforceable,
        fl_two_month_cap_violated: cap_violated,
        fl_addendum_compliance: input.separate_addendum_signed,
        fl_notice_compliance: input.tenant_provided_60_day_notice,
        mitigation_compliance: true,
        damages_recoverable_cents: damages_recoverable,
        violations,
        citation: "Fla. Stat. § 83.595(2), § 83.595(4)",
        notes,
    }
}

fn check_california(input: &LeaseEarlyTerminationFeeInput) -> LeaseEarlyTerminationFeeResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if !input.landlord_attempted_mitigation {
        violations.push(
            "Cal. Civ. Code § 1951.2 — landlord MUST attempt to mitigate damages by re-renting in reasonable time + reasonable effort; failure to mitigate bars or reduces recovery"
                .to_string(),
        );
    }

    if !input.clause_reasonable_pre_estimate_of_damages {
        violations.push(
            "Cal. Civ. Code § 1671(d) — liquidated damages clause enforceable ONLY if reasonable estimate of damages at lease execution; penalty clauses VOID"
                .to_string(),
        );
    }

    notes.push(
        "Cal. Civ. Code § 1951.2 — California recovery framework is ACTUAL DAMAGES (not liquidated cap); landlord recovers actual rent loss after mitigation + reasonable releasing costs"
            .to_string(),
    );
    notes.push(
        "Cal. Civ. Code § 1671(d) — residential liquidated damages clauses are presumptively VOID unless landlord proves reasonable estimate at execution; § 1671(b) commercial-context test reversed for residential"
            .to_string(),
    );

    let fee_enforceable = violations.is_empty();
    let damages_recoverable = if input.landlord_attempted_mitigation {
        input.actual_rent_loss_after_mitigation_cents.max(0)
    } else {
        0
    };

    LeaseEarlyTerminationFeeResult {
        fee_enforceable,
        fl_two_month_cap_violated: false,
        fl_addendum_compliance: true,
        fl_notice_compliance: true,
        mitigation_compliance: input.landlord_attempted_mitigation,
        damages_recoverable_cents: damages_recoverable,
        violations,
        citation: "Cal. Civ. Code §§ 1951.2, 1671(b), 1671(d)",
        notes,
    }
}

fn check_default(input: &LeaseEarlyTerminationFeeInput) -> LeaseEarlyTerminationFeeResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if !input.landlord_attempted_mitigation {
        violations.push(
            "default common-law rule — landlord has duty to mitigate by attempting to re-rent in reasonable time; Restatement (Second) of Contracts § 350 mitigation principle; some states (e.g., AR, OK) historically rejected duty but trend is toward universal mitigation"
                .to_string(),
        );
    }

    if !input.clause_reasonable_pre_estimate_of_damages {
        violations.push(
            "Restatement (Second) of Contracts § 356 — liquidated damages clause enforceable ONLY if reasonable pre-estimate of damages at execution; penalty clauses VOID; uniform doctrine across most states"
                .to_string(),
        );
    }

    notes.push(
        "default rule — common-law actual damages + duty to mitigate; landlord recovers actual rent loss + reasonable releasing costs after mitigation"
            .to_string(),
    );

    let fee_enforceable = violations.is_empty();
    let damages_recoverable = if input.landlord_attempted_mitigation {
        input.actual_rent_loss_after_mitigation_cents.max(0)
    } else {
        0
    };

    LeaseEarlyTerminationFeeResult {
        fee_enforceable,
        fl_two_month_cap_violated: false,
        fl_addendum_compliance: true,
        fl_notice_compliance: true,
        mitigation_compliance: input.landlord_attempted_mitigation,
        damages_recoverable_cents: damages_recoverable,
        violations,
        citation: "common-law actual damages + Restatement (Second) of Contracts §§ 350, 356",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fl_compliant() -> LeaseEarlyTerminationFeeInput {
        LeaseEarlyTerminationFeeInput {
            regime: Regime::Florida,
            fee_amount_cents: 400_000,
            monthly_rent_cents: 200_000,
            separate_addendum_signed: true,
            tenant_provided_60_day_notice: true,
            landlord_attempted_mitigation: false,
            actual_rent_loss_after_mitigation_cents: 0,
            clause_reasonable_pre_estimate_of_damages: true,
        }
    }

    fn ca_compliant() -> LeaseEarlyTerminationFeeInput {
        LeaseEarlyTerminationFeeInput {
            regime: Regime::California,
            fee_amount_cents: 0,
            monthly_rent_cents: 300_000,
            separate_addendum_signed: false,
            tenant_provided_60_day_notice: false,
            landlord_attempted_mitigation: true,
            actual_rent_loss_after_mitigation_cents: 150_000,
            clause_reasonable_pre_estimate_of_damages: true,
        }
    }

    fn default_compliant() -> LeaseEarlyTerminationFeeInput {
        LeaseEarlyTerminationFeeInput {
            regime: Regime::Default,
            fee_amount_cents: 0,
            monthly_rent_cents: 250_000,
            separate_addendum_signed: false,
            tenant_provided_60_day_notice: false,
            landlord_attempted_mitigation: true,
            actual_rent_loss_after_mitigation_cents: 100_000,
            clause_reasonable_pre_estimate_of_damages: true,
        }
    }

    #[test]
    fn fl_clean_compliance_fee_enforceable() {
        let r = check(&fl_compliant());
        assert!(r.fee_enforceable);
        assert!(!r.fl_two_month_cap_violated);
        assert!(r.fl_addendum_compliance);
        assert!(r.fl_notice_compliance);
        assert_eq!(r.damages_recoverable_cents, 400_000);
    }

    #[test]
    fn fl_fee_at_exact_2_month_cap_enforceable() {
        let mut i = fl_compliant();
        i.fee_amount_cents = 400_000;
        i.monthly_rent_cents = 200_000;
        let r = check(&i);
        assert!(r.fee_enforceable);
        assert_eq!(r.damages_recoverable_cents, 400_000);
    }

    #[test]
    fn fl_fee_above_2_month_cap_unenforceable() {
        let mut i = fl_compliant();
        i.fee_amount_cents = 400_001;
        i.monthly_rent_cents = 200_000;
        let r = check(&i);
        assert!(!r.fee_enforceable);
        assert!(r.fl_two_month_cap_violated);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 83.595(4)") && v.contains("2 MONTHS' RENT")));
    }

    #[test]
    fn fl_missing_separate_addendum_unenforceable() {
        let mut i = fl_compliant();
        i.separate_addendum_signed = false;
        let r = check(&i);
        assert!(!r.fee_enforceable);
        assert!(!r.fl_addendum_compliance);
        assert!(r.violations.iter().any(|v| v.contains("SEPARATE ADDENDUM")));
    }

    #[test]
    fn fl_missing_60_day_notice_unenforceable() {
        let mut i = fl_compliant();
        i.tenant_provided_60_day_notice = false;
        let r = check(&i);
        assert!(!r.fee_enforceable);
        assert!(!r.fl_notice_compliance);
        assert!(r.violations.iter().any(|v| v.contains("60 DAYS' notice")));
    }

    #[test]
    fn fl_menu_of_remedies_note_present() {
        let r = check(&fl_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 83.595(2)") && n.contains("menu of remedies")));
    }

    #[test]
    fn fl_election_waiver_note_present() {
        let r = check(&fl_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 83.595(4)") && n.contains("WAIVES additional rent")));
    }

    #[test]
    fn fl_citation_pins_83_595_subsections() {
        let r = check(&fl_compliant());
        assert!(r.citation.contains("§ 83.595(2)"));
        assert!(r.citation.contains("§ 83.595(4)"));
    }

    #[test]
    fn fl_multiple_violations_simultaneous() {
        let mut i = fl_compliant();
        i.fee_amount_cents = 1_000_000;
        i.monthly_rent_cents = 200_000;
        i.separate_addendum_signed = false;
        i.tenant_provided_60_day_notice = false;
        let r = check(&i);
        assert!(!r.fee_enforceable);
        assert_eq!(r.violations.len(), 3);
    }

    #[test]
    fn fl_damages_recoverable_zero_when_unenforceable() {
        let mut i = fl_compliant();
        i.separate_addendum_signed = false;
        let r = check(&i);
        assert!(!r.fee_enforceable);
        assert_eq!(r.damages_recoverable_cents, 0);
    }

    #[test]
    fn ca_actual_damages_after_mitigation_recovered() {
        let r = check(&ca_compliant());
        assert!(r.fee_enforceable);
        assert!(r.mitigation_compliance);
        assert_eq!(r.damages_recoverable_cents, 150_000);
    }

    #[test]
    fn ca_no_mitigation_violation() {
        let mut i = ca_compliant();
        i.landlord_attempted_mitigation = false;
        let r = check(&i);
        assert!(!r.fee_enforceable);
        assert!(!r.mitigation_compliance);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1951.2") && v.contains("mitigate")));
    }

    #[test]
    fn ca_penalty_clause_violation() {
        let mut i = ca_compliant();
        i.clause_reasonable_pre_estimate_of_damages = false;
        let r = check(&i);
        assert!(!r.fee_enforceable);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1671(d)") && v.contains("penalty clauses VOID")));
    }

    #[test]
    fn ca_actual_damages_framework_note() {
        let r = check(&ca_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1951.2") && n.contains("ACTUAL DAMAGES (not liquidated cap)")));
    }

    #[test]
    fn ca_citation_pins_subsections() {
        let r = check(&ca_compliant());
        assert!(r.citation.contains("§§ 1951.2"));
        assert!(r.citation.contains("1671(b)"));
        assert!(r.citation.contains("1671(d)"));
    }

    #[test]
    fn ca_negative_actual_loss_clamped_to_zero() {
        let mut i = ca_compliant();
        i.actual_rent_loss_after_mitigation_cents = -50_000;
        let r = check(&i);
        assert_eq!(r.damages_recoverable_cents, 0);
    }

    #[test]
    fn default_actual_damages_with_mitigation() {
        let r = check(&default_compliant());
        assert!(r.fee_enforceable);
        assert_eq!(r.damages_recoverable_cents, 100_000);
    }

    #[test]
    fn default_no_mitigation_violation() {
        let mut i = default_compliant();
        i.landlord_attempted_mitigation = false;
        let r = check(&i);
        assert!(!r.fee_enforceable);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("default common-law rule") && v.contains("Restatement")));
    }

    #[test]
    fn default_penalty_clause_violation() {
        let mut i = default_compliant();
        i.clause_reasonable_pre_estimate_of_damages = false;
        let r = check(&i);
        assert!(!r.fee_enforceable);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 356") && v.contains("penalty clauses VOID")));
    }

    #[test]
    fn default_citation_references_restatement() {
        let r = check(&default_compliant());
        assert!(r.citation.contains("Restatement"));
        assert!(r.citation.contains("§§ 350, 356"));
    }

    #[test]
    fn florida_unique_2_month_cap_invariant() {
        let mut i_fl = fl_compliant();
        i_fl.fee_amount_cents = 1_000_000;
        i_fl.monthly_rent_cents = 200_000;
        let r_fl = check(&i_fl);
        assert!(r_fl.fl_two_month_cap_violated);

        for regime in [Regime::California, Regime::Default] {
            let mut i = fl_compliant();
            i.regime = regime;
            i.fee_amount_cents = 1_000_000;
            i.monthly_rent_cents = 200_000;
            i.landlord_attempted_mitigation = true;
            let r = check(&i);
            assert!(
                !r.fl_two_month_cap_violated,
                "regime {:?} should not engage FL 2-month cap",
                regime
            );
        }
    }

    #[test]
    fn florida_unique_addendum_requirement_invariant() {
        let mut i_fl = fl_compliant();
        i_fl.separate_addendum_signed = false;
        let r_fl = check(&i_fl);
        assert!(!r_fl.fee_enforceable);

        for regime in [Regime::California, Regime::Default] {
            let mut i = ca_compliant();
            i.regime = regime;
            i.separate_addendum_signed = false;
            i.landlord_attempted_mitigation = true;
            let r = check(&i);
            assert!(
                r.fee_enforceable,
                "regime {:?} should not require FL separate addendum",
                regime
            );
        }
    }

    #[test]
    fn three_regimes_routed_correctly() {
        for regime in [Regime::Florida, Regime::California, Regime::Default] {
            let mut i = fl_compliant();
            i.regime = regime;
            i.landlord_attempted_mitigation = true;
            let r = check(&i);
            let _ = r.fee_enforceable;
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn fl_fee_below_cap_recoverable_at_actual_amount() {
        let mut i = fl_compliant();
        i.fee_amount_cents = 300_000;
        i.monthly_rent_cents = 200_000;
        let r = check(&i);
        assert!(r.fee_enforceable);
        assert_eq!(r.damages_recoverable_cents, 300_000);
    }

    #[test]
    fn fl_saturating_mul_no_overflow_on_extreme_rent() {
        let mut i = fl_compliant();
        i.fee_amount_cents = 1_000_000_000;
        i.monthly_rent_cents = i64::MAX / 2;
        let r = check(&i);
        assert!(r.fee_enforceable);
    }

    #[test]
    fn ca_clause_reasonable_estimate_required_invariant() {
        let mut i_ca = ca_compliant();
        i_ca.clause_reasonable_pre_estimate_of_damages = false;
        let r_ca = check(&i_ca);
        assert!(!r_ca.fee_enforceable);

        let mut i_default = default_compliant();
        i_default.clause_reasonable_pre_estimate_of_damages = false;
        let r_default = check(&i_default);
        assert!(!r_default.fee_enforceable);

        let mut i_fl = fl_compliant();
        i_fl.clause_reasonable_pre_estimate_of_damages = false;
        let r_fl = check(&i_fl);
        assert!(r_fl.fee_enforceable);
    }

    #[test]
    fn default_no_mitigation_blocks_damages() {
        let mut i = default_compliant();
        i.landlord_attempted_mitigation = false;
        let r = check(&i);
        assert_eq!(r.damages_recoverable_cents, 0);
    }

    #[test]
    fn default_negative_actual_loss_clamped() {
        let mut i = default_compliant();
        i.actual_rent_loss_after_mitigation_cents = -1_000;
        let r = check(&i);
        assert_eq!(r.damages_recoverable_cents, 0);
    }

    #[test]
    fn fl_zero_rent_zero_cap() {
        let mut i = fl_compliant();
        i.monthly_rent_cents = 0;
        i.fee_amount_cents = 1;
        let r = check(&i);
        assert!(!r.fee_enforceable);
        assert!(r.fl_two_month_cap_violated);
    }
}
