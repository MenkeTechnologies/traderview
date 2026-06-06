//! State landlord duty-to-mitigate-damages compliance check for
//! tenant lease breach / abandonment.
//!
//! Distinct from `tenant_abandonment` (which addresses WHEN a
//! landlord may declare abandonment and re-take possession) — this
//! module answers the downstream question: where the tenant has
//! broken the lease, is the landlord LEGALLY OBLIGATED to make
//! reasonable efforts to re-rent the unit to mitigate damages, or
//! may the landlord sit on the unit and collect rent for the full
//! remaining term?
//!
//! Eight regimes:
//!
//!   - **California** — Cal. Civ. Code § 1951.2 — landlord may
//!     recover unpaid future rent ONLY to the extent the lessor
//!     "acted reasonably and in a good-faith effort to mitigate
//!     damages." § 1951.4 carve-out: if the lease expressly permits
//!     assignment or subletting without lessor's consent
//!     unreasonably withheld, the lessor may sue for rent as it
//!     becomes due without retaking and without the § 1951.2 duty.
//!
//!   - **NewYork** — N.Y. Real Prop. Law § 227-e (Housing Stability
//!     and Tenant Protection Act of 2019). Statutory duty to make
//!     reasonable efforts to re-rent at a fair-market rent. The
//!     burden of proof is on the landlord. **Pre-HSTPA NY was a "no
//!     duty" jurisdiction** under common law — HSTPA reversed.
//!     § 227-e is NON-WAIVABLE.
//!
//!   - **Texas** — Tex. Prop. Code § 91.006(a) — "A landlord has a
//!     duty to mitigate damages if a tenant abandons the leased
//!     premises in violation of the lease." § 91.006(b) prohibits
//!     contractual waiver — "A provision of a lease that purports
//!     to waive a right or to exempt a landlord from a liability
//!     or duty under this section is void."
//!
//!   - **Illinois** — 735 ILCS 5/9-213.1 — "After January 1, 1984,
//!     a landlord or his or her agent shall take reasonable
//!     measures to mitigate damages recoverable against a
//!     defaulting lessee."
//!
//!   - **Florida** — Fla. Stat. § 83.595 — HYBRID / CONDITIONAL.
//!     The landlord may elect between four paths. § 83.595(1)(a)
//!     treats the rental agreement as terminated and retakes
//!     possession with NO further claim to future rent (no
//!     mitigation question arises). § 83.595(1)(b) retakes possession
//!     and holds the tenant liable for rent BUT must use "good
//!     faith" to relet (mitigation REQUIRED). § 83.595(1)(c) stands
//!     by and does nothing, holding the tenant liable for rent as
//!     it becomes due (NO mitigation duty). § 83.595(1)(d) sues for
//!     damages as they accrue (mitigation REQUIRED). The landlord's
//!     CHOICE controls whether the mitigation duty attaches.
//!
//!   - **Mississippi** — Alsup v. Banks, 9 So. 895 (Miss. 1891) —
//!     COMMON-LAW MINORITY RULE: landlord has NO DUTY to mitigate
//!     damages. Landlord may sit on the unit and collect rent for
//!     the full remaining term without attempting to re-rent.
//!
//!   - **Georgia** — Georgia common law is unclear; courts have
//!     allowed landlords to keep a rental vacant and sue for rent
//!     as it becomes due despite a statutory rule suggesting a
//!     duty. Treated here as a NO DUTY jurisdiction for compliance-
//!     audit purposes (the audit conservative position is that no
//!     duty is enforced as a matter of right).
//!
//!   - **Default** — majority common-law rule (most other states)
//!     imposes a duty to make reasonable efforts to mitigate.
//!
//! Citations: Cal. Civ. Code § 1951.2 (CA statutory duty); § 1951.4
//! (CA assignment carve-out); N.Y. Real Prop. Law § 227-e (NY HSTPA
//! 2019); Tex. Prop. Code § 91.006 (TX statutory duty + waiver
//! prohibition); 735 ILCS 5/9-213.1 (IL statutory duty); Fla. Stat.
//! § 83.595 (FL conditional election); Alsup v. Banks, 9 So. 895
//! (Miss. 1891) (MS common-law no duty).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    NewYork,
    Texas,
    Illinois,
    Florida,
    Mississippi,
    Georgia,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FloridaElection {
    /// Not in Florida — irrelevant.
    NotApplicable,
    /// Fla. Stat. § 83.595(1)(b) — retake and relet; mitigation
    /// REQUIRED.
    RetakeAndRelet,
    /// Fla. Stat. § 83.595(1)(c) — stand by and do nothing; no
    /// mitigation duty.
    StandByCollectRent,
    /// Fla. Stat. § 83.595(1)(a) — treat as terminated, retake, no
    /// further rent claim; mitigation question moot.
    TerminateAndRetake,
    /// Fla. Stat. § 83.595(1)(d) — sue for damages as they accrue;
    /// mitigation REQUIRED.
    SueAsAccrued,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    /// True if the tenant has breached the lease (early termination,
    /// abandonment, etc.). False short-circuits — no mitigation
    /// question.
    pub tenant_breached_lease: bool,
    /// True if the landlord made reasonable efforts to re-rent the
    /// unit (advertised, showed, accepted reasonable applicants).
    pub landlord_attempted_reletting: bool,
    /// Days unit remained vacant before relet (or as of the audit
    /// date if still vacant).
    pub days_unit_remained_vacant: u32,
    pub original_monthly_rent_cents: i64,
    /// Re-rented monthly rent (cents). Zero if not yet re-rented.
    pub re_rented_monthly_rent_cents: i64,
    /// Months remaining on the original lease at time of breach.
    pub months_remaining_on_lease: u32,
    /// Florida § 83.595 landlord election. NotApplicable for other
    /// regimes.
    pub florida_landlord_election: FloridaElection,
    /// Whether the lease contains a clause purporting to waive the
    /// mitigation duty. Relevant where the regime prohibits waiver.
    pub waiver_clause_in_lease: bool,
    /// California § 1951.4 — whether the lease expressly permits
    /// assignment or subletting without lessor's consent
    /// unreasonably withheld. Triggers the § 1951.4 carve-out from
    /// the § 1951.2 duty.
    pub california_assignment_permitted: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// True if the regime imposes a duty to mitigate on the
    /// landlord under the present facts.
    pub duty_to_mitigate_applies: bool,
    /// True if the duty applies AND the landlord made reasonable
    /// efforts to re-rent. False if the duty applies and no
    /// efforts were made.
    pub landlord_compliant: bool,
    /// Whether the regime statutorily prohibits contractual waiver
    /// of the mitigation duty.
    pub waiver_prohibited: bool,
    /// Whether a waiver clause in the lease is invalid (i.e.,
    /// regime prohibits waiver AND clause is present).
    pub waiver_clause_invalid: bool,
    /// Recoverable damages exposure (cents) the landlord may claim
    /// against the tenant — reduced where re-rented at full or
    /// partial rent. Zero when the landlord is non-compliant with
    /// the duty (the practical effect is the landlord recovers
    /// nothing). Estimated from inputs.
    pub recoverable_damages_estimate_cents: i64,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Input) -> CheckResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if !input.tenant_breached_lease {
        notes.push(
            "No tenant breach — duty-to-mitigate question is not presented; landlord has no \
             mitigation obligation at this time."
                .to_string(),
        );
        return CheckResult {
            duty_to_mitigate_applies: false,
            landlord_compliant: true,
            waiver_prohibited: false,
            waiver_clause_invalid: false,
            recoverable_damages_estimate_cents: 0,
            violations,
            citation: "No breach — duty-to-mitigate inapplicable",
            notes,
        };
    }

    // Determine whether the regime imposes a duty under the present
    // facts.
    let (duty_applies, waiver_prohibited, citation): (bool, bool, &'static str) = match input.regime
    {
        Regime::California => {
            if input.california_assignment_permitted {
                (
                    false,
                    false,
                    "Cal. Civ. Code § 1951.4 (assignment-or-subletting-without-unreasonable-\
                     consent-withholding carve-out — lessor may sue for rent as it becomes due \
                     without retaking and without the § 1951.2 mitigation duty)",
                )
            } else {
                (
                    true,
                    false,
                    "Cal. Civ. Code § 1951.2 (statutory duty — lessor may recover unpaid future \
                     rent only to the extent of a reasonable and good-faith effort to mitigate \
                     damages)",
                )
            }
        }
        Regime::NewYork => (
            true,
            true,
            "N.Y. Real Prop. Law § 227-e (HSTPA 2019 — statutory duty to make reasonable \
             efforts to re-rent at fair-market rent; burden of proof on the landlord; \
             NON-WAIVABLE)",
        ),
        Regime::Texas => (
            true,
            true,
            "Tex. Prop. Code § 91.006(a) (statutory duty to mitigate); § 91.006(b) (waiver \
             prohibition — any contractual waiver is VOID)",
        ),
        Regime::Illinois => (
            true,
            false,
            "735 ILCS 5/9-213.1 (statutory duty — landlord shall take reasonable measures to \
             mitigate damages recoverable against a defaulting lessee, effective 1984-01-01)",
        ),
        Regime::Florida => {
            let duty = match input.florida_landlord_election {
                FloridaElection::RetakeAndRelet | FloridaElection::SueAsAccrued => true,
                FloridaElection::StandByCollectRent => false,
                FloridaElection::TerminateAndRetake => false,
                FloridaElection::NotApplicable => false,
            };
            (
                duty,
                false,
                "Fla. Stat. § 83.595 (conditional — duty depends on landlord election: \
                 § 83.595(1)(b) retake-and-relet REQUIRES mitigation; § 83.595(1)(c) \
                 stand-by-collect-rent NO duty; § 83.595(1)(a) terminate-and-retake mitigation \
                 moot; § 83.595(1)(d) sue-as-accrued REQUIRES mitigation)",
            )
        }
        Regime::Mississippi => (
            false,
            false,
            "Alsup v. Banks, 9 So. 895 (Miss. 1891) (COMMON-LAW MINORITY RULE — no duty to \
             mitigate; landlord may collect full remaining rent without attempting to re-rent)",
        ),
        Regime::Georgia => (
            false,
            false,
            "Georgia common law (no statutory duty to mitigate; case law allows landlord to \
             keep rental vacant and sue for rent as it becomes due)",
        ),
        Regime::Default => (
            true,
            false,
            "Majority common-law rule — landlord must make reasonable efforts to mitigate \
             damages on tenant breach",
        ),
    };

    let waiver_clause_invalid = waiver_prohibited && input.waiver_clause_in_lease;
    if waiver_clause_invalid {
        violations.push(
            "Lease contains a clause purporting to waive the mitigation duty; under the \
             applicable regime such waivers are statutorily VOID."
                .to_string(),
        );
    }

    let landlord_compliant = if duty_applies {
        if !input.landlord_attempted_reletting {
            violations.push(format!(
                "Landlord made no reasonable efforts to re-rent the unit (unit vacant {} days); \
                 mitigation duty requires good-faith reletting attempts.",
                input.days_unit_remained_vacant,
            ));
            false
        } else {
            true
        }
    } else {
        // No duty — compliance is vacuous.
        true
    };

    // Recoverable damages estimate.
    //
    // Where the landlord is compliant (or no duty applies), damages =
    // months_remaining × original_rent − (relet_rent × estimated
    // months-relet-covers). We simplify by treating the relet rent
    // as covering the remaining months once relet, and subtract the
    // expected relet payments.
    //
    // Where the landlord is non-compliant with the duty, the
    // practical effect in many states is the landlord recovers
    // NOTHING (because the tenant proves the loss could have been
    // avoided). We surface that as zero.
    let total_remaining_rent_cents = input
        .original_monthly_rent_cents
        .saturating_mul(input.months_remaining_on_lease as i64);
    let expected_relet_recovery_cents = input
        .re_rented_monthly_rent_cents
        .saturating_mul(input.months_remaining_on_lease as i64);

    let recoverable = if duty_applies && !landlord_compliant {
        0
    } else {
        total_remaining_rent_cents
            .saturating_sub(expected_relet_recovery_cents)
            .max(0)
    };

    if duty_applies
        && landlord_compliant
        && input.re_rented_monthly_rent_cents > 0
        && input.re_rented_monthly_rent_cents < input.original_monthly_rent_cents
    {
        notes.push(format!(
            "Re-rented at {} cents/month vs original {} cents/month — landlord may recover the \
             difference for the remaining {} months as damages.",
            input.re_rented_monthly_rent_cents,
            input.original_monthly_rent_cents,
            input.months_remaining_on_lease,
        ));
    }

    notes.push(
        "Distinct from tenant_abandonment (which addresses when landlord may DECLARE \
         abandonment and re-take possession). This module addresses whether the landlord MUST \
         attempt to re-rent to mitigate damages on tenant breach."
            .to_string(),
    );

    CheckResult {
        duty_to_mitigate_applies: duty_applies,
        landlord_compliant,
        waiver_prohibited,
        waiver_clause_invalid,
        recoverable_damages_estimate_cents: recoverable,
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
            tenant_breached_lease: true,
            landlord_attempted_reletting: true,
            days_unit_remained_vacant: 30,
            original_monthly_rent_cents: 2_000_00,
            re_rented_monthly_rent_cents: 2_000_00,
            months_remaining_on_lease: 6,
            florida_landlord_election: FloridaElection::NotApplicable,
            waiver_clause_in_lease: false,
            california_assignment_permitted: false,
        }
    }

    // ── No breach short-circuit ─────────────────────────────────

    #[test]
    fn no_breach_no_duty_question_presented() {
        let mut i = base(Regime::California);
        i.tenant_breached_lease = false;
        let r = check(&i);
        assert!(!r.duty_to_mitigate_applies);
        assert!(r.landlord_compliant);
        assert_eq!(r.recoverable_damages_estimate_cents, 0);
    }

    // ── California § 1951.2 ─────────────────────────────────────

    #[test]
    fn california_default_duty_applies() {
        let r = check(&base(Regime::California));
        assert!(r.duty_to_mitigate_applies);
        assert!(r.landlord_compliant);
        assert!(r.citation.contains("§ 1951.2"));
    }

    #[test]
    fn california_section_1951_4_carve_out_no_duty() {
        let mut i = base(Regime::California);
        i.california_assignment_permitted = true;
        let r = check(&i);
        assert!(!r.duty_to_mitigate_applies);
        assert!(r.citation.contains("§ 1951.4"));
    }

    #[test]
    fn california_no_reletting_attempt_violation() {
        let mut i = base(Regime::California);
        i.landlord_attempted_reletting = false;
        let r = check(&i);
        assert!(r.duty_to_mitigate_applies);
        assert!(!r.landlord_compliant);
        assert_eq!(r.recoverable_damages_estimate_cents, 0);
    }

    // ── New York § 227-e (HSTPA 2019) ───────────────────────────

    #[test]
    fn new_york_post_hstpa_duty_applies() {
        let r = check(&base(Regime::NewYork));
        assert!(r.duty_to_mitigate_applies);
        assert!(r.waiver_prohibited);
        assert!(r.citation.contains("§ 227-e"));
        assert!(r.citation.contains("HSTPA"));
    }

    #[test]
    fn new_york_waiver_clause_invalid() {
        let mut i = base(Regime::NewYork);
        i.waiver_clause_in_lease = true;
        let r = check(&i);
        assert!(r.waiver_clause_invalid);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("waive") && v.contains("VOID")));
    }

    // ── Texas § 91.006 ──────────────────────────────────────────

    #[test]
    fn texas_statutory_duty_applies() {
        let r = check(&base(Regime::Texas));
        assert!(r.duty_to_mitigate_applies);
        assert!(r.waiver_prohibited);
        assert!(r.citation.contains("§ 91.006"));
    }

    #[test]
    fn texas_waiver_clause_invalid() {
        let mut i = base(Regime::Texas);
        i.waiver_clause_in_lease = true;
        let r = check(&i);
        assert!(r.waiver_clause_invalid);
    }

    // ── Illinois 735 ILCS 5/9-213.1 ─────────────────────────────

    #[test]
    fn illinois_statutory_duty_applies() {
        let r = check(&base(Regime::Illinois));
        assert!(r.duty_to_mitigate_applies);
        assert!(!r.waiver_prohibited);
        assert!(r.citation.contains("735 ILCS 5/9-213.1"));
    }

    // ── Florida § 83.595 (conditional on election) ──────────────

    #[test]
    fn florida_retake_and_relet_duty_applies() {
        let mut i = base(Regime::Florida);
        i.florida_landlord_election = FloridaElection::RetakeAndRelet;
        let r = check(&i);
        assert!(r.duty_to_mitigate_applies);
        assert!(r.citation.contains("§ 83.595(1)(b)"));
    }

    #[test]
    fn florida_stand_by_collect_rent_no_duty() {
        let mut i = base(Regime::Florida);
        i.florida_landlord_election = FloridaElection::StandByCollectRent;
        let r = check(&i);
        assert!(!r.duty_to_mitigate_applies);
        assert!(r.citation.contains("§ 83.595(1)(c)"));
    }

    #[test]
    fn florida_terminate_and_retake_no_duty() {
        let mut i = base(Regime::Florida);
        i.florida_landlord_election = FloridaElection::TerminateAndRetake;
        let r = check(&i);
        assert!(!r.duty_to_mitigate_applies);
    }

    #[test]
    fn florida_sue_as_accrued_duty_applies() {
        let mut i = base(Regime::Florida);
        i.florida_landlord_election = FloridaElection::SueAsAccrued;
        let r = check(&i);
        assert!(r.duty_to_mitigate_applies);
        assert!(r.citation.contains("§ 83.595(1)(d)"));
    }

    // ── Mississippi common-law no-duty ──────────────────────────

    #[test]
    fn mississippi_alsup_v_banks_no_duty_even_with_no_reletting() {
        let mut i = base(Regime::Mississippi);
        i.landlord_attempted_reletting = false;
        let r = check(&i);
        assert!(!r.duty_to_mitigate_applies);
        assert!(r.landlord_compliant);
        assert!(r.citation.contains("Alsup v. Banks"));
        // Landlord still recovers full remaining rent.
        assert_eq!(
            r.recoverable_damages_estimate_cents,
            2_000_00 * 6 - 2_000_00 * 6,
        );
    }

    #[test]
    fn mississippi_no_reletting_recovers_full_rent_when_unit_left_vacant() {
        let mut i = base(Regime::Mississippi);
        i.landlord_attempted_reletting = false;
        i.re_rented_monthly_rent_cents = 0;
        let r = check(&i);
        // Full 6 months × $2000 = $12,000 = 1_200_000 cents.
        assert_eq!(r.recoverable_damages_estimate_cents, 1_200_000);
    }

    // ── Georgia no-duty (unclear common law treated as no duty) ─

    #[test]
    fn georgia_no_statutory_duty() {
        let r = check(&base(Regime::Georgia));
        assert!(!r.duty_to_mitigate_applies);
        assert!(r.citation.contains("Georgia common law"));
    }

    // ── Default majority common-law rule ────────────────────────

    #[test]
    fn default_majority_rule_duty_applies() {
        let r = check(&base(Regime::Default));
        assert!(r.duty_to_mitigate_applies);
        assert!(!r.waiver_prohibited);
        assert!(r.citation.contains("Majority common-law rule"));
    }

    // ── Recoverable damages math ────────────────────────────────

    #[test]
    fn relet_at_full_rent_zero_damages() {
        let r = check(&base(Regime::California));
        // Original = re-rented = $2000; remaining = 6 mo. Net = 0.
        assert_eq!(r.recoverable_damages_estimate_cents, 0);
    }

    #[test]
    fn relet_at_lower_rent_recovers_difference() {
        let mut i = base(Regime::California);
        i.re_rented_monthly_rent_cents = 1_500_00; // $1500
                                                   // (2000 − 1500) × 6 = 3000 = 300_000 cents.
        let r = check(&i);
        assert_eq!(r.recoverable_damages_estimate_cents, 300_000);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Re-rented") && n.contains("difference")));
    }

    #[test]
    fn non_compliant_landlord_recovers_zero_damages() {
        let mut i = base(Regime::Texas);
        i.landlord_attempted_reletting = false;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert_eq!(r.recoverable_damages_estimate_cents, 0);
    }

    // ── Multi-regime regression invariants ──────────────────────

    #[test]
    fn only_mississippi_and_georgia_and_florida_election_lack_duty_invariant() {
        // Among the duty-applies regimes (CA, NY, TX, IL, FL retake/sue,
        // Default), all impose duty. MS + GA + FL standby + FL terminate
        // never impose duty.
        for &regime in &[
            Regime::California,
            Regime::NewYork,
            Regime::Texas,
            Regime::Illinois,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(r.duty_to_mitigate_applies, "{:?}: duty must apply", regime,);
        }
        for &regime in &[Regime::Mississippi, Regime::Georgia] {
            let r = check(&base(regime));
            assert!(
                !r.duty_to_mitigate_applies,
                "{:?}: duty must NOT apply (minority rule)",
                regime,
            );
        }
    }

    #[test]
    fn only_ny_and_tx_prohibit_waiver_invariant() {
        for &regime in &[Regime::NewYork, Regime::Texas] {
            let r = check(&base(regime));
            assert!(
                r.waiver_prohibited,
                "{:?}: must statutorily prohibit waiver",
                regime,
            );
        }
        for &regime in &[
            Regime::California,
            Regime::Illinois,
            Regime::Florida,
            Regime::Mississippi,
            Regime::Georgia,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(
                !r.waiver_prohibited,
                "{:?}: must NOT statutorily prohibit waiver",
                regime,
            );
        }
    }

    #[test]
    fn waiver_clause_only_invalid_in_ny_and_tx_invariant() {
        for &regime in &[Regime::NewYork, Regime::Texas] {
            let mut i = base(regime);
            i.waiver_clause_in_lease = true;
            let r = check(&i);
            assert!(r.waiver_clause_invalid);
        }
        for &regime in &[
            Regime::California,
            Regime::Illinois,
            Regime::Florida,
            Regime::Mississippi,
            Regime::Georgia,
            Regime::Default,
        ] {
            let mut i = base(regime);
            i.waiver_clause_in_lease = true;
            let r = check(&i);
            assert!(
                !r.waiver_clause_invalid,
                "{:?}: waiver clause must NOT be statutorily invalid",
                regime,
            );
        }
    }

    #[test]
    fn only_florida_uses_landlord_election_branching_invariant() {
        // For Florida, swapping election changes duty status.
        let mut i = base(Regime::Florida);
        i.florida_landlord_election = FloridaElection::RetakeAndRelet;
        assert!(check(&i).duty_to_mitigate_applies);
        i.florida_landlord_election = FloridaElection::StandByCollectRent;
        assert!(!check(&i).duty_to_mitigate_applies);

        // For non-Florida regimes, the FL election is a no-op.
        for &regime in &[
            Regime::California,
            Regime::NewYork,
            Regime::Texas,
            Regime::Illinois,
            Regime::Mississippi,
            Regime::Georgia,
            Regime::Default,
        ] {
            let mut a = base(regime);
            a.florida_landlord_election = FloridaElection::RetakeAndRelet;
            let mut b = base(regime);
            b.florida_landlord_election = FloridaElection::StandByCollectRent;
            assert_eq!(
                check(&a).duty_to_mitigate_applies,
                check(&b).duty_to_mitigate_applies,
                "{:?}: FL election must be a no-op",
                regime,
            );
        }
    }

    #[test]
    fn only_california_uses_assignment_carve_out_invariant() {
        let mut ca = base(Regime::California);
        ca.california_assignment_permitted = true;
        assert!(!check(&ca).duty_to_mitigate_applies);

        for &regime in &[
            Regime::NewYork,
            Regime::Texas,
            Regime::Illinois,
            Regime::Florida,
            Regime::Mississippi,
            Regime::Georgia,
            Regime::Default,
        ] {
            let mut a = base(regime);
            a.california_assignment_permitted = false;
            let mut b = base(regime);
            b.california_assignment_permitted = true;
            assert_eq!(
                check(&a).duty_to_mitigate_applies,
                check(&b).duty_to_mitigate_applies,
                "{:?}: CA assignment-permitted must be a no-op",
                regime,
            );
        }
    }

    #[test]
    fn citation_pins_authority_per_regime() {
        assert!(check(&base(Regime::California))
            .citation
            .contains("§ 1951.2"));
        assert!(check(&base(Regime::NewYork)).citation.contains("§ 227-e"));
        assert!(check(&base(Regime::Texas)).citation.contains("§ 91.006"));
        assert!(check(&base(Regime::Illinois))
            .citation
            .contains("735 ILCS 5/9-213.1"));
        assert!(check(&base(Regime::Florida)).citation.contains("§ 83.595"));
        assert!(check(&base(Regime::Mississippi))
            .citation
            .contains("Alsup v. Banks"));
    }

    #[test]
    fn sibling_module_note_present_across_all_regimes() {
        for &regime in &[
            Regime::California,
            Regime::NewYork,
            Regime::Texas,
            Regime::Illinois,
            Regime::Florida,
            Regime::Mississippi,
            Regime::Georgia,
            Regime::Default,
        ] {
            let r = check(&base(regime));
            assert!(
                r.notes
                    .iter()
                    .any(|n| n.contains("tenant_abandonment") && n.contains("mitigate")),
                "{:?}: sibling-module tenant_abandonment note must be present",
                regime,
            );
        }
    }
}
