//! Landlord's duty to deliver possession at lease commencement —
//! when must a landlord deliver ACTUAL possession (not just legal
//! right) of the leased premises to the tenant, and what are the
//! tenant's remedies if a prior tenant is holding over or
//! squatters remain?
//!
//! Two-rule structural divergence with URLTA codification:
//!
//! ENGLISH RULE (modern majority) — landlord must deliver
//! ACTUAL possession on the lease commencement date. If a prior
//! tenant is holding over or squatters remain in occupancy, the
//! landlord has breached and the tenant may (a) cancel the lease,
//! (b) recover damages including rent abatement for the delay
//! period, and (c) compel landlord to take action against the
//! holdover party. Adopted by most U.S. jurisdictions and
//! Restatement (Second) of Property § 6.2.
//!
//! AMERICAN RULE (minority) — landlord delivers only LEGAL
//! POSSESSION (the right to possess). If a prior tenant or
//! squatter is in occupancy, the NEW TENANT has standing to
//! evict and recover damages directly from the holdover party —
//! NOT the landlord. The tenant cannot cancel the lease and
//! must continue to pay rent. Followed by a minority of states
//! including (historically) New York and a few others.
//!
//! URLTA — Uniform Residential Landlord and Tenant Act § 2.103
//! codifies the ENGLISH RULE: "At the commencement of the term
//! a landlord shall ... deliver possession of the premises to
//! the tenant in compliance with the rental agreement." URLTA
//! § 4.102 provides tenant remedies for failure to deliver:
//!   (a) Tenant may upon written notice to the landlord
//!       terminate the rental agreement and recover all prepaid
//!       rent and security; OR
//!   (b) Tenant may demand performance and recover damages,
//!       including punitive damages in form of NOT MORE THAN
//!       3 MONTHS' PERIODIC RENT OR THREEFOLD THE ACTUAL DAMAGES
//!       SUSTAINED, whichever is GREATER, PLUS reasonable
//!       attorney's fees.
//!
//! Approximately 20 U.S. states have adopted URLTA in some form
//! (with state-specific modifications). Notable adopters
//! include AK, AZ, FL, IA, KS, KY, MS, MT, NE, NM, OR, RI, SC,
//! TN, VA, and WA (residential landlord-tenant acts modeled on
//! URLTA).
//!
//! Citations: Restatement (Second) of Property: Landlord and
//! Tenant § 6.2 (delivery of possession — adopts English rule
//! as default); Uniform Residential Landlord and Tenant Act
//! § 2.103 (1972, amended 1974) — landlord obligation to
//! deliver possession; URLTA § 4.102 — tenant remedies for
//! failure to deliver (greater of 3 months' rent or threefold
//! actual damages + attorney's fees + injunctive relief);
//! Hannan v. Dusch, 153 S.E. 824 (Va. 1930) — leading
//! American-rule decision; American Rule (property) common-law
//! doctrine.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    /// URLTA-adopting states — § 2.103 + § 4.102 statutory
    /// remedies (greater of 3 months' rent or 3× actual damages).
    UrltaStates,
    /// English Rule (modern majority) — landlord must deliver
    /// actual possession; common-law damages.
    EnglishRule,
    /// American Rule (minority) — landlord delivers only legal
    /// right; tenant must sue holdover party.
    AmericanRule,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DamagesCalculationMethod {
    /// URLTA § 4.102 — greater of 3 months' rent OR 3× actual.
    UrltaTripleActualOrThreeMonths,
    /// English rule — common-law actual damages with rent
    /// abatement for the delay period.
    EnglishRuleActualWithAbatement,
    /// American rule — no statutory or contractual damages
    /// from landlord; tenant must pursue holdover party.
    NoLandlordDamages,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    /// True if landlord failed to deliver actual possession at
    /// the commencement of the lease term.
    pub landlord_failed_to_deliver_actual_possession: bool,
    /// True if a prior tenant is holding over OR squatters are
    /// in occupancy.
    pub prior_tenant_holdover_or_squatter_present: bool,
    /// Monthly rent under the lease (cents).
    pub monthly_rent_cents: i64,
    /// Tenant's actual damages from the delayed possession
    /// (cents) — typically temporary lodging + storage + lost
    /// wages.
    pub tenant_actual_damages_cents: i64,
    /// Number of days possession was delayed beyond the lease
    /// commencement date.
    pub days_delayed_possession: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    pub landlord_duty_to_deliver_actual_possession: bool,
    pub tenant_may_cancel_lease: bool,
    pub tenant_must_evict_prior_tenant_party: bool,
    /// Statutory or common-law damages tenant may recover from
    /// landlord (cents).
    pub damages_recoverable_from_landlord_cents: i64,
    pub damages_calculation_method: DamagesCalculationMethod,
    /// True if URLTA § 4.102 statutory remedies are available
    /// (3 months' rent or 3× damages, whichever greater).
    pub urlta_statutory_remedies_available: bool,
    /// True if URLTA § 4.102 attorney-fee shifting applies.
    pub attorney_fees_recoverable: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// URLTA § 4.102 — multiplier for actual damages alternative.
pub const URLTA_DAMAGES_MULTIPLIER: i64 = 3;
/// URLTA § 4.102 — months of rent alternative.
pub const URLTA_RENT_MONTHS_FLOOR: i64 = 3;

pub fn check(input: &Input) -> CheckResult {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let monthly_rent = input.monthly_rent_cents.max(0);
    let actual_damages = input.tenant_actual_damages_cents.max(0);

    // English-rule and URLTA regimes impose the duty; American rule does not.
    let landlord_duty_to_deliver_actual_possession =
        matches!(input.regime, Regime::UrltaStates | Regime::EnglishRule);

    let breach_engaged = input.landlord_failed_to_deliver_actual_possession
        && input.prior_tenant_holdover_or_squatter_present;

    let tenant_may_cancel_lease = landlord_duty_to_deliver_actual_possession && breach_engaged;
    let tenant_must_evict_prior_tenant_party = matches!(input.regime, Regime::AmericanRule)
        && input.prior_tenant_holdover_or_squatter_present;

    let urlta_statutory_remedies_available =
        matches!(input.regime, Regime::UrltaStates) && breach_engaged;
    let attorney_fees_recoverable = urlta_statutory_remedies_available;

    let (damages_recoverable_from_landlord_cents, damages_calculation_method) = match input.regime {
        Regime::UrltaStates if breach_engaged => {
            let three_months_rent = monthly_rent.saturating_mul(URLTA_RENT_MONTHS_FLOOR);
            let triple_actual = actual_damages.saturating_mul(URLTA_DAMAGES_MULTIPLIER);
            let damages = three_months_rent.max(triple_actual);
            (
                damages,
                DamagesCalculationMethod::UrltaTripleActualOrThreeMonths,
            )
        }
        Regime::EnglishRule if breach_engaged => {
            // Common-law actual damages with rent abatement for delay period.
            // Multiply-before-divide preserves precision (avoids 30-cent
            // truncation loss per month from naive daily-rent calculation).
            let days = input.days_delayed_possession.max(0);
            let rent_abatement = monthly_rent.saturating_mul(days) / 30;
            let damages = actual_damages.saturating_add(rent_abatement);
            (
                damages,
                DamagesCalculationMethod::EnglishRuleActualWithAbatement,
            )
        }
        Regime::AmericanRule => {
            // No damages from landlord; tenant pursues holdover party.
            (0, DamagesCalculationMethod::NoLandlordDamages)
        }
        _ => (0, DamagesCalculationMethod::NoLandlordDamages),
    };

    // Violations + regime-specific notes.
    match input.regime {
        Regime::UrltaStates => {
            notes.push(
                "URLTA § 2.103 — landlord MUST deliver actual possession at commencement \
                 of the term. URLTA § 4.102 tenant remedies for breach: (a) terminate \
                 lease + recover prepaid rent and security; OR (b) demand performance + \
                 recover damages (greater of 3 months' periodic rent OR threefold \
                 actual damages) + attorney's fees + injunctive relief."
                    .to_string(),
            );
            if breach_engaged {
                let three_months = monthly_rent * URLTA_RENT_MONTHS_FLOOR;
                let triple_actual = actual_damages * URLTA_DAMAGES_MULTIPLIER;
                violations.push(format!(
                    "URLTA § 2.103 — landlord failed to deliver actual possession; prior \
                     tenant/squatter in occupancy. URLTA § 4.102 statutory remedies: \
                     greater of 3 months' rent ({} cents) OR 3× actual damages ({} cents) \
                     = {} cents recoverable from landlord, plus attorney's fees.",
                    three_months, triple_actual, damages_recoverable_from_landlord_cents,
                ));
            }
        }
        Regime::EnglishRule => {
            notes.push(
                "English Rule (modern majority + Restatement (Second) of Property § 6.2) \
                 — landlord must deliver actual possession on lease commencement. Tenant \
                 may cancel lease and recover actual damages including rent abatement \
                 for the delay period. No statutory multiplier or attorney-fee shifting \
                 without URLTA adoption."
                    .to_string(),
            );
            if breach_engaged {
                violations.push(format!(
                    "English Rule — landlord breached delivery duty; prior tenant/squatter \
                     in occupancy. Tenant may cancel lease + recover actual damages of \
                     {} cents (including rent abatement for {} days at {} cents/day).",
                    damages_recoverable_from_landlord_cents,
                    input.days_delayed_possession,
                    monthly_rent / 30,
                ));
            }
        }
        Regime::AmericanRule => {
            notes.push(
                "American Rule (minority) — landlord delivers only LEGAL POSSESSION (the \
                 right to possess). Landlord NOT liable when prior tenant or squatter \
                 remains in occupancy. NEW TENANT has standing to evict the holdover \
                 party directly and recover damages from them — not from landlord. \
                 Tenant CANNOT cancel the lease and must continue to pay rent. \
                 Hannan v. Dusch, 153 S.E. 824 (Va. 1930) is the leading decision."
                    .to_string(),
            );
            if input.prior_tenant_holdover_or_squatter_present {
                notes.push(
                    "Tenant's remedy under American Rule: bring eviction proceeding \
                     against the holdover prior tenant / squatter directly; recover \
                     actual damages from THAT party (not the landlord). Lease remains \
                     in force; rent continues to accrue."
                        .to_string(),
                );
            }
        }
    }

    notes.push(
        "Sibling distinction: this module covers the LEASE-COMMENCEMENT DELIVERY DUTY. \
         Related modules: `holdover_tenant_damages` (the prior tenant's liability for \
         remaining past lease end), `quiet_enjoyment` (post-delivery covenant against \
         landlord interference), `habitability_remedies` (post-delivery uninhabitable-\
         condition remedies), `eviction_notices` (procedures the landlord must follow to \
         evict). URLTA adopters (~20 states including AK, AZ, FL, IA, KS, KY, MS, MT, \
         NE, NM, OR, RI, SC, TN, VA, WA) get the statutory 3-months-rent or 3× damages \
         remedy + attorney-fee shifting; non-URLTA English-rule states get common-law \
         actual damages only; American-rule minority shifts the burden of evicting the \
         holdover party entirely to the new tenant."
            .to_string(),
    );

    let compliant = violations.is_empty();

    CheckResult {
        landlord_duty_to_deliver_actual_possession,
        tenant_may_cancel_lease,
        tenant_must_evict_prior_tenant_party,
        damages_recoverable_from_landlord_cents,
        damages_calculation_method,
        urlta_statutory_remedies_available,
        attorney_fees_recoverable,
        compliant,
        violations,
        citation: "Restatement (Second) of Property: Landlord and Tenant § 6.2 (delivery \
                   of possession — adopts English rule as default); Uniform Residential \
                   Landlord and Tenant Act § 2.103 (1972, amended 1974) — landlord \
                   obligation to deliver possession; URLTA § 4.102 — tenant remedies \
                   (greater of 3 months' periodic rent OR threefold actual damages + \
                   reasonable attorney's fees + injunctive relief); Hannan v. Dusch, \
                   153 S.E. 824 (Va. 1930) — leading American-rule decision",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(regime: Regime) -> Input {
        Input {
            regime,
            landlord_failed_to_deliver_actual_possession: true,
            prior_tenant_holdover_or_squatter_present: true,
            monthly_rent_cents: 200_000,          // $2,000/month
            tenant_actual_damages_cents: 500_000, // $5,000 actual damages
            days_delayed_possession: 30,
        }
    }

    // ── URLTA regime ─────────────────────────────────────────

    #[test]
    fn urlta_landlord_failure_triggers_statutory_remedies() {
        let r = check(&input(Regime::UrltaStates));
        assert!(r.landlord_duty_to_deliver_actual_possession);
        assert!(r.tenant_may_cancel_lease);
        assert!(r.urlta_statutory_remedies_available);
        assert!(r.attorney_fees_recoverable);
        assert!(!r.compliant);
    }

    #[test]
    fn urlta_3_months_rent_greater_than_triple_damages() {
        // Rent $2K/mo × 3 = $6K; actual damages $1K × 3 = $3K → 3 months wins.
        let mut b = input(Regime::UrltaStates);
        b.tenant_actual_damages_cents = 100_000; // $1K
        let r = check(&b);
        // 3 × $2K = $6K (600,000 cents)
        assert_eq!(r.damages_recoverable_from_landlord_cents, 600_000);
    }

    #[test]
    fn urlta_triple_damages_greater_than_3_months() {
        // Rent $2K/mo × 3 = $6K; actual damages $5K × 3 = $15K → triple wins.
        let r = check(&input(Regime::UrltaStates));
        // max($6K, $15K) = $15K
        assert_eq!(r.damages_recoverable_from_landlord_cents, 1_500_000);
    }

    #[test]
    fn urlta_no_actual_damages_3_months_floor() {
        let mut b = input(Regime::UrltaStates);
        b.tenant_actual_damages_cents = 0;
        let r = check(&b);
        // No actual damages; 3 months × $2K = $6K floor.
        assert_eq!(r.damages_recoverable_from_landlord_cents, 600_000);
    }

    #[test]
    fn urlta_no_breach_no_damages() {
        let mut b = input(Regime::UrltaStates);
        b.landlord_failed_to_deliver_actual_possession = false;
        b.prior_tenant_holdover_or_squatter_present = false;
        let r = check(&b);
        assert!(r.compliant);
        assert!(!r.tenant_may_cancel_lease);
        assert!(!r.urlta_statutory_remedies_available);
        assert_eq!(r.damages_recoverable_from_landlord_cents, 0);
    }

    // ── English rule regime ──────────────────────────────────

    #[test]
    fn english_rule_failure_actual_damages_plus_abatement() {
        let r = check(&input(Regime::EnglishRule));
        assert!(r.landlord_duty_to_deliver_actual_possession);
        assert!(r.tenant_may_cancel_lease);
        // Actual $5K + 30 days × ($2K/30 = $66.67) = $5K + $2K = $7K.
        // $66.67 × 30 = exact $2,000.
        assert_eq!(r.damages_recoverable_from_landlord_cents, 500_000 + 200_000);
        assert!(!r.urlta_statutory_remedies_available);
        assert!(!r.attorney_fees_recoverable);
    }

    #[test]
    fn english_rule_no_actual_damages_abatement_only() {
        let mut b = input(Regime::EnglishRule);
        b.tenant_actual_damages_cents = 0;
        let r = check(&b);
        // 30 days × ($2K/30 = $66.67/day) = $2K abatement only.
        assert_eq!(r.damages_recoverable_from_landlord_cents, 200_000);
    }

    #[test]
    fn english_rule_no_breach_no_damages() {
        let mut b = input(Regime::EnglishRule);
        b.landlord_failed_to_deliver_actual_possession = false;
        b.prior_tenant_holdover_or_squatter_present = false;
        let r = check(&b);
        assert!(!r.tenant_may_cancel_lease);
        assert_eq!(r.damages_recoverable_from_landlord_cents, 0);
    }

    // ── American rule regime ─────────────────────────────────

    #[test]
    fn american_rule_no_landlord_duty() {
        let r = check(&input(Regime::AmericanRule));
        // Landlord has no duty to deliver actual possession.
        assert!(!r.landlord_duty_to_deliver_actual_possession);
        assert!(!r.tenant_may_cancel_lease);
        // Tenant has burden to evict holdover.
        assert!(r.tenant_must_evict_prior_tenant_party);
        // No landlord damages even with breach scenario.
        assert_eq!(r.damages_recoverable_from_landlord_cents, 0);
    }

    #[test]
    fn american_rule_no_statutory_remedies() {
        let r = check(&input(Regime::AmericanRule));
        assert!(!r.urlta_statutory_remedies_available);
        assert!(!r.attorney_fees_recoverable);
    }

    #[test]
    fn american_rule_holdover_party_remedy_note() {
        let r = check(&input(Regime::AmericanRule));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("bring eviction proceeding against the holdover")));
    }

    // ── Damages calculation method enum coverage ──────────────

    #[test]
    fn urlta_uses_urlta_triple_actual_or_three_months() {
        let r = check(&input(Regime::UrltaStates));
        assert_eq!(
            r.damages_calculation_method,
            DamagesCalculationMethod::UrltaTripleActualOrThreeMonths
        );
    }

    #[test]
    fn english_rule_uses_actual_with_abatement() {
        let r = check(&input(Regime::EnglishRule));
        assert_eq!(
            r.damages_calculation_method,
            DamagesCalculationMethod::EnglishRuleActualWithAbatement
        );
    }

    #[test]
    fn american_rule_uses_no_landlord_damages() {
        let r = check(&input(Regime::AmericanRule));
        assert_eq!(
            r.damages_calculation_method,
            DamagesCalculationMethod::NoLandlordDamages
        );
    }

    // ── Multi-regime invariants ───────────────────────────────

    #[test]
    fn only_urlta_has_statutory_remedies_invariant() {
        for regime in [
            Regime::UrltaStates,
            Regime::EnglishRule,
            Regime::AmericanRule,
        ] {
            let r = check(&input(regime));
            let expected = matches!(regime, Regime::UrltaStates);
            assert_eq!(
                r.urlta_statutory_remedies_available, expected,
                "{:?}",
                regime
            );
            assert_eq!(r.attorney_fees_recoverable, expected, "{:?}", regime);
        }
    }

    #[test]
    fn only_american_rule_shifts_burden_to_tenant_invariant() {
        for regime in [
            Regime::UrltaStates,
            Regime::EnglishRule,
            Regime::AmericanRule,
        ] {
            let r = check(&input(regime));
            let expected = matches!(regime, Regime::AmericanRule);
            assert_eq!(
                r.tenant_must_evict_prior_tenant_party, expected,
                "{:?}",
                regime
            );
        }
    }

    #[test]
    fn american_rule_only_regime_without_duty_invariant() {
        for regime in [
            Regime::UrltaStates,
            Regime::EnglishRule,
            Regime::AmericanRule,
        ] {
            let r = check(&input(regime));
            let expected = !matches!(regime, Regime::AmericanRule);
            assert_eq!(
                r.landlord_duty_to_deliver_actual_possession, expected,
                "{:?}",
                regime
            );
        }
    }

    #[test]
    fn urlta_dollar_for_dollar_test_with_3_months_rent_versus_actual_damages_truth_table() {
        // Test the URLTA max(3 months rent, 3× actual) calculation.
        let cells = [
            // (monthly_rent, actual_damages, expected_recoverable)
            (200_000, 100_000, 600_000),   // 3mo $6K vs 3× $3K → $6K
            (200_000, 500_000, 1_500_000), // 3mo $6K vs 3× $15K → $15K
            (200_000, 0, 600_000),         // 3mo floor $6K
            (0, 500_000, 1_500_000),       // No rent → triple actual
            (500_000, 0, 1_500_000),       // High rent → $15K floor
        ];
        for (rent, damages, expected) in cells.iter() {
            let mut b = input(Regime::UrltaStates);
            b.monthly_rent_cents = *rent;
            b.tenant_actual_damages_cents = *damages;
            let r = check(&b);
            assert_eq!(
                r.damages_recoverable_from_landlord_cents, *expected,
                "rent={} damages={}",
                rent, damages
            );
        }
    }

    #[test]
    fn urlta_constants_invariant() {
        assert_eq!(URLTA_DAMAGES_MULTIPLIER, 3);
        assert_eq!(URLTA_RENT_MONTHS_FLOOR, 3);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&input(Regime::UrltaStates));
        assert!(r.citation.contains("Restatement (Second) of Property"));
        assert!(r.citation.contains("§ 6.2"));
        assert!(r.citation.contains("URLTA"));
        assert!(r.citation.contains("§ 2.103"));
        assert!(r.citation.contains("§ 4.102"));
        assert!(r.citation.contains("Hannan v. Dusch"));
        assert!(r.citation.contains("153 S.E. 824"));
        assert!(r.citation.contains("Va. 1930"));
    }

    #[test]
    fn sibling_distinction_note_present() {
        let r = check(&input(Regime::UrltaStates));
        assert!(
            r.notes.iter().any(|n| n.contains("holdover_tenant_damages")
                && n.contains("quiet_enjoyment")
                && n.contains("habitability_remedies")
                && n.contains("eviction_notices")
                && n.contains("LEASE-COMMENCEMENT DELIVERY DUTY")),
            "sibling distinction note must reference related modules + delivery-duty focus"
        );
    }

    // ── Defensive input clamping ──────────────────────────────

    #[test]
    fn defensive_negative_rent_clamped() {
        let mut b = input(Regime::UrltaStates);
        b.monthly_rent_cents = -100_000;
        b.tenant_actual_damages_cents = 0;
        let r = check(&b);
        // Negative rent clamps to 0; triple-actual = 0 → recover 0.
        assert_eq!(r.damages_recoverable_from_landlord_cents, 0);
    }

    #[test]
    fn defensive_negative_damages_clamped() {
        let mut b = input(Regime::UrltaStates);
        b.tenant_actual_damages_cents = -50_000;
        let r = check(&b);
        // Negative clamps to 0; 3 months rent = $6K wins.
        assert_eq!(r.damages_recoverable_from_landlord_cents, 600_000);
    }

    #[test]
    fn defensive_negative_days_no_abatement() {
        let mut b = input(Regime::EnglishRule);
        b.days_delayed_possession = -5;
        let r = check(&b);
        // Negative days → 0 abatement; only actual damages.
        assert_eq!(r.damages_recoverable_from_landlord_cents, 500_000);
    }

    #[test]
    fn english_rule_long_delay_large_abatement() {
        let mut b = input(Regime::EnglishRule);
        b.days_delayed_possession = 365;
        b.tenant_actual_damages_cents = 0;
        let r = check(&b);
        // 200_000 × 365 / 30 = 73,000,000 / 30 = 2,433,333.
        assert_eq!(r.damages_recoverable_from_landlord_cents, 2_433_333);
    }
}
