//! Tenant lease termination right for catastrophic property damage
//! (fire, flood, hurricane, earthquake, explosion, similar casualty).
//! Trader-landlord operational concern after natural disaster or
//! catastrophic loss event — when must the landlord let the tenant
//! out of the lease, and what refund obligations attach?
//!
//! Distinct from `dv_termination` (DV survivor termination),
//! `military_termination` (SCRA-based PCS termination), `crime_
//! victim_termination` (state-broader crime-victim termination), and
//! `habitability_remedies` (ongoing repair / withholding / rent
//! escrow framework). This module addresses ONLY the CATASTROPHIC-
//! DAMAGE TERMINATION pathway after a casualty loss.
//!
//! Five regimes:
//!
//! **California — Cal. Civ. Code §§ 1932(2), 1933(4)**. § 1932(2)
//! permits tenant termination if the "GREATER PART OF THE THING
//! HIRED" is destroyed by inevitable casualty without the tenant's
//! fault. § 1933(4) — lease terminates automatically only if
//! ENTIRELY DESTROYED without fault of landlord OR tenant making it
//! entirely uninhabitable. "Entirely destroyed" is a question of
//! fact — typically requires complete leveling; partial damage that
//! can be repaired does NOT trigger automatic termination. Security
//! deposit return required within 21 days per Cal. Civ. Code §
//! 1950.5(g); no deduction permitted for disaster-caused damage.
//!
//! **Texas — Tex. Prop. Code § 92.054**. CASUALTY LOSS framework
//! covering fire, smoke, hail, explosion, or similar cause. If the
//! rental premises are "AS A PRACTICAL MATTER TOTALLY UNUSABLE for
//! residential purposes" AND the casualty is NOT caused by the
//! tenant's (or family/guest/invitee's) negligence or fault, EITHER
//! the landlord OR the tenant may terminate the lease by giving
//! WRITTEN NOTICE to the other at any time BEFORE REPAIRS ARE
//! COMPLETED. Tenant entitled to pro-rata rent refund + security
//! deposit refund. Repair period under § 92.052 does NOT begin
//! until landlord receives INSURANCE PROCEEDS — a uniquely Texas
//! procedural rule.
//!
//! **New York — N.Y. Real Prop. Law § 227**. PREMISES DESTROYED BY
//! FIRE OR OTHER CASUALTY without tenant's fault — tenant may
//! "SURRENDER POSSESSION" and lease terminates. Pro-rata rent only
//! through date of surrender. Applies to fire, explosion, flood, or
//! similar casualty. Tenant must elect to surrender — no automatic
//! termination.
//!
//! **New Jersey — N.J.S.A. 46:8-6 to 46:8-8**. § 46:8-6 covers
//! destruction-of-premises termination. § 46:8-7 — partial
//! destruction permits tenant to terminate OR continue at PROPORTIONALLY
//! REDUCED RENT for the unusable portion. § 46:8-8 — fault
//! attribution defeats termination right.
//!
//! **Default — common-law impossibility of performance**. Most
//! states follow Restatement (Second) of Contracts § 261 — total
//! destruction makes performance impossible, lease may terminate.
//! Partial destruction generally permits continued tenancy with
//! rent abatement.
//!
//! Citations: Cal. Civ. Code § 1932(2) (CA tenant termination right
//! on greater-part destruction); § 1933(4) (CA automatic termination
//! on entire destruction); § 1950.5(g) (CA 21-day deposit refund);
//! Tex. Prop. Code § 92.054(a) (TX casualty loss); § 92.054(b)
//! (totally-unusable standard); § 92.052(b) (insurance proceeds
//! trigger repair period); N.Y. Real Prop. Law § 227 (NY surrender-
//! possession right); N.J.S.A. 46:8-6 (NJ destruction); N.J.S.A.
//! 46:8-7 (NJ partial destruction proportional rent); N.J.S.A.
//! 46:8-8 (NJ fault attribution); Restatement (Second) of Contracts
//! § 261 (impossibility doctrine).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Texas,
    NewYork,
    NewJersey,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DamageExtent {
    /// Minor or repairable damage — lease generally continues with
    /// rent abatement for unusable portion.
    Partial,
    /// "Greater part" of premises destroyed (§ 1932(2)) or
    /// substantial portion making continued occupancy impractical
    /// — tenant termination right typically engages.
    GreaterPart,
    /// "As a practical matter totally unusable" (§ 92.054(b)) or
    /// entirely destroyed (§ 1933(4)) — automatic or both-party
    /// termination right.
    TotallyDestroyed,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CatastrophicDamageInput {
    pub regime: Regime,
    pub damage_extent: DamageExtent,
    /// Whether the tenant (or family member / guest / invitee)
    /// caused the damage. Tenant fault defeats the termination
    /// right in all regimes.
    pub tenant_fault: bool,
    /// Whether the landlord caused the damage. Some regimes
    /// (CA § 1933(4)) require absence of landlord fault for
    /// automatic termination; tenant may still terminate at-will.
    pub landlord_fault: bool,
    /// Whether the tenant gave written notice of termination (TX
    /// § 92.054(c) requires written notice).
    pub written_notice_given: bool,
    /// Whether repairs are already completed (TX § 92.054(c) bars
    /// termination after repairs complete).
    pub repairs_completed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CatastrophicDamageResult {
    pub termination_permitted: bool,
    pub pro_rata_rent_refund: bool,
    pub deposit_return_required: bool,
    pub partial_rent_abatement_available: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &CatastrophicDamageInput) -> CatastrophicDamageResult {
    let mut notes: Vec<String> = Vec::new();

    if input.tenant_fault {
        notes.push(
            "tenant fault defeats catastrophic-damage termination right in all regimes — tenant remains liable under lease"
                .to_string(),
        );
        return CatastrophicDamageResult {
            termination_permitted: false,
            pro_rata_rent_refund: false,
            deposit_return_required: false,
            partial_rent_abatement_available: false,
            citation: citation_for(input.regime),
            notes,
        };
    }

    match input.regime {
        Regime::California => check_california(input, &mut notes),
        Regime::Texas => check_texas(input, &mut notes),
        Regime::NewYork => check_new_york(input, &mut notes),
        Regime::NewJersey => check_new_jersey(input, &mut notes),
        Regime::Default => check_default(input, &mut notes),
    }
}

fn check_california(
    input: &CatastrophicDamageInput,
    notes: &mut Vec<String>,
) -> CatastrophicDamageResult {
    let termination = match input.damage_extent {
        DamageExtent::TotallyDestroyed => {
            if !input.landlord_fault {
                notes.push(
                    "§ 1933(4) — entirely destroyed without landlord or tenant fault; lease AUTOMATICALLY terminates"
                        .to_string(),
                );
            } else {
                notes.push(
                    "§ 1933(4) — entirely destroyed but landlord fault present; automatic termination unavailable but tenant may still elect § 1932(2) termination"
                        .to_string(),
                );
            }
            true
        }
        DamageExtent::GreaterPart => {
            notes.push(
                "§ 1932(2) — greater part destroyed by inevitable casualty without tenant fault; tenant may terminate at election"
                    .to_string(),
            );
            true
        }
        DamageExtent::Partial => {
            notes.push(
                "partial damage does NOT trigger § 1932(2) or § 1933(4); lease continues with potential rent abatement"
                    .to_string(),
            );
            false
        }
    };

    notes.push(
        "§ 1950.5(g) — security deposit refund required within 21 days; landlord cannot deduct for disaster-caused damage"
            .to_string(),
    );

    let partial_abatement = matches!(input.damage_extent, DamageExtent::Partial);

    CatastrophicDamageResult {
        termination_permitted: termination,
        pro_rata_rent_refund: termination,
        deposit_return_required: termination,
        partial_rent_abatement_available: partial_abatement,
        citation: citation_for(Regime::California),
        notes: notes.clone(),
    }
}

fn check_texas(
    input: &CatastrophicDamageInput,
    notes: &mut Vec<String>,
) -> CatastrophicDamageResult {
    if !matches!(input.damage_extent, DamageExtent::TotallyDestroyed) {
        notes.push(
            "Tex. Prop. Code § 92.054 — termination right requires \"AS A PRACTICAL MATTER TOTALLY UNUSABLE for residential purposes\"; partial damage does not satisfy the standard"
                .to_string(),
        );
        return CatastrophicDamageResult {
            termination_permitted: false,
            pro_rata_rent_refund: false,
            deposit_return_required: false,
            partial_rent_abatement_available: true,
            citation: citation_for(Regime::Texas),
            notes: notes.clone(),
        };
    }

    if input.repairs_completed {
        notes.push(
            "§ 92.054(c) — written notice of termination must be given BEFORE repairs are completed; right lost after completion"
                .to_string(),
        );
        return CatastrophicDamageResult {
            termination_permitted: false,
            pro_rata_rent_refund: false,
            deposit_return_required: false,
            partial_rent_abatement_available: false,
            citation: citation_for(Regime::Texas),
            notes: notes.clone(),
        };
    }

    if !input.written_notice_given {
        notes.push(
            "§ 92.054(c) — written notice required from terminating party (landlord OR tenant)"
                .to_string(),
        );
        return CatastrophicDamageResult {
            termination_permitted: false,
            pro_rata_rent_refund: false,
            deposit_return_required: false,
            partial_rent_abatement_available: false,
            citation: citation_for(Regime::Texas),
            notes: notes.clone(),
        };
    }

    notes.push(
        "§ 92.054(b) — premises totally unusable + not tenant fault + written notice before repairs complete = mutual termination right"
            .to_string(),
    );
    notes.push(
        "§ 92.052(b) — repair period does NOT begin until landlord receives insurance proceeds (uniquely Texas procedural rule)"
            .to_string(),
    );
    notes.push(
        "§ 92.054(c) — tenant entitled to pro-rata rent refund and security deposit refund; § 92.0561 repair-and-deduct unavailable after termination"
            .to_string(),
    );

    CatastrophicDamageResult {
        termination_permitted: true,
        pro_rata_rent_refund: true,
        deposit_return_required: true,
        partial_rent_abatement_available: false,
        citation: citation_for(Regime::Texas),
        notes: notes.clone(),
    }
}

fn check_new_york(
    input: &CatastrophicDamageInput,
    notes: &mut Vec<String>,
) -> CatastrophicDamageResult {
    let termination = match input.damage_extent {
        DamageExtent::TotallyDestroyed | DamageExtent::GreaterPart => {
            notes.push(
                "N.Y. Real Prop. Law § 227 — premises destroyed by fire or other casualty without tenant fault; tenant may SURRENDER POSSESSION and lease terminates"
                    .to_string(),
            );
            true
        }
        DamageExtent::Partial => {
            notes.push(
                "§ 227 surrender-possession right requires substantial destruction; partial damage permits continued tenancy"
                    .to_string(),
            );
            false
        }
    };

    notes.push(
        "§ 227 — tenant must affirmatively elect to surrender; no automatic termination"
            .to_string(),
    );

    CatastrophicDamageResult {
        termination_permitted: termination,
        pro_rata_rent_refund: termination,
        deposit_return_required: termination,
        partial_rent_abatement_available: matches!(input.damage_extent, DamageExtent::Partial),
        citation: citation_for(Regime::NewYork),
        notes: notes.clone(),
    }
}

fn check_new_jersey(
    input: &CatastrophicDamageInput,
    notes: &mut Vec<String>,
) -> CatastrophicDamageResult {
    let termination = match input.damage_extent {
        DamageExtent::TotallyDestroyed => {
            notes.push(
                "N.J.S.A. 46:8-6 — destruction-of-premises termination; lease ends".to_string(),
            );
            true
        }
        DamageExtent::GreaterPart => {
            notes.push(
                "N.J.S.A. 46:8-7 — partial destruction permits tenant to TERMINATE or CONTINUE at PROPORTIONALLY REDUCED RENT"
                    .to_string(),
            );
            true
        }
        DamageExtent::Partial => {
            notes.push(
                "§ 46:8-7 — minor partial damage permits continued tenancy with proportional rent reduction for unusable portion"
                    .to_string(),
            );
            false
        }
    };

    notes.push("§ 46:8-8 — fault attribution defeats termination right".to_string());

    CatastrophicDamageResult {
        termination_permitted: termination,
        pro_rata_rent_refund: termination,
        deposit_return_required: termination,
        partial_rent_abatement_available: !matches!(
            input.damage_extent,
            DamageExtent::TotallyDestroyed
        ),
        citation: citation_for(Regime::NewJersey),
        notes: notes.clone(),
    }
}

fn check_default(
    input: &CatastrophicDamageInput,
    notes: &mut Vec<String>,
) -> CatastrophicDamageResult {
    let termination = matches!(input.damage_extent, DamageExtent::TotallyDestroyed);
    notes.push(
        "default common-law rule — Restatement (Second) of Contracts § 261 impossibility of performance; total destruction terminates lease, partial destruction continues with rent abatement"
            .to_string(),
    );
    CatastrophicDamageResult {
        termination_permitted: termination,
        pro_rata_rent_refund: termination,
        deposit_return_required: termination,
        partial_rent_abatement_available: !matches!(
            input.damage_extent,
            DamageExtent::TotallyDestroyed
        ),
        citation: citation_for(Regime::Default),
        notes: notes.clone(),
    }
}

fn citation_for(regime: Regime) -> &'static str {
    match regime {
        Regime::California => "Cal. Civ. Code §§ 1932(2), 1933(4), 1950.5(g)",
        Regime::Texas => "Tex. Prop. Code §§ 92.054(a)/(b)/(c), 92.052(b)",
        Regime::NewYork => "N.Y. Real Prop. Law § 227",
        Regime::NewJersey => "N.J.S.A. 46:8-6, 46:8-7, 46:8-8",
        Regime::Default => "Restatement (Second) of Contracts § 261",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(regime: Regime, damage: DamageExtent) -> CatastrophicDamageInput {
        CatastrophicDamageInput {
            regime,
            damage_extent: damage,
            tenant_fault: false,
            landlord_fault: false,
            written_notice_given: true,
            repairs_completed: false,
        }
    }

    #[test]
    fn ca_totally_destroyed_automatic_termination() {
        let r = check(&base(Regime::California, DamageExtent::TotallyDestroyed));
        assert!(r.termination_permitted);
        assert!(r.pro_rata_rent_refund);
        assert!(r.deposit_return_required);
        assert!(r.notes.iter().any(|n| n.contains("§ 1933(4)")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("AUTOMATICALLY terminates")));
    }

    #[test]
    fn ca_greater_part_destroyed_tenant_election_termination() {
        let r = check(&base(Regime::California, DamageExtent::GreaterPart));
        assert!(r.termination_permitted);
        assert!(r.notes.iter().any(|n| n.contains("§ 1932(2)")));
    }

    #[test]
    fn ca_partial_damage_no_termination() {
        let r = check(&base(Regime::California, DamageExtent::Partial));
        assert!(!r.termination_permitted);
        assert!(r.partial_rent_abatement_available);
    }

    #[test]
    fn ca_landlord_fault_blocks_automatic_termination_under_1933() {
        let mut i = base(Regime::California, DamageExtent::TotallyDestroyed);
        i.landlord_fault = true;
        let r = check(&i);
        assert!(r.termination_permitted, "tenant may still elect § 1932(2)");
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("automatic termination unavailable")));
    }

    #[test]
    fn ca_21_day_deposit_refund_note_always_present() {
        let r = check(&base(Regime::California, DamageExtent::TotallyDestroyed));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1950.5(g)") && n.contains("21 days")));
    }

    #[test]
    fn tx_totally_unusable_with_written_notice_before_repairs_termination() {
        let r = check(&base(Regime::Texas, DamageExtent::TotallyDestroyed));
        assert!(r.termination_permitted);
        assert!(r.notes.iter().any(|n| n.contains("§ 92.054(b)")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 92.052(b)") && n.contains("insurance proceeds")));
    }

    #[test]
    fn tx_partial_damage_no_termination_right_under_92_054() {
        let r = check(&base(Regime::Texas, DamageExtent::Partial));
        assert!(!r.termination_permitted);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("AS A PRACTICAL MATTER TOTALLY UNUSABLE")));
    }

    #[test]
    fn tx_no_written_notice_termination_unavailable() {
        let mut i = base(Regime::Texas, DamageExtent::TotallyDestroyed);
        i.written_notice_given = false;
        let r = check(&i);
        assert!(!r.termination_permitted);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 92.054(c)") && n.contains("written notice")));
    }

    #[test]
    fn tx_repairs_completed_termination_unavailable() {
        let mut i = base(Regime::Texas, DamageExtent::TotallyDestroyed);
        i.repairs_completed = true;
        let r = check(&i);
        assert!(!r.termination_permitted);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("BEFORE repairs are completed")));
    }

    #[test]
    fn tx_insurance_proceeds_trigger_repair_period_note() {
        let r = check(&base(Regime::Texas, DamageExtent::TotallyDestroyed));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("uniquely Texas procedural rule")));
    }

    #[test]
    fn ny_totally_destroyed_surrender_termination() {
        let r = check(&base(Regime::NewYork, DamageExtent::TotallyDestroyed));
        assert!(r.termination_permitted);
        assert!(r.notes.iter().any(|n| n.contains("§ 227")));
        assert!(r.notes.iter().any(|n| n.contains("SURRENDER POSSESSION")));
    }

    #[test]
    fn ny_greater_part_also_surrender_termination() {
        let r = check(&base(Regime::NewYork, DamageExtent::GreaterPart));
        assert!(r.termination_permitted);
    }

    #[test]
    fn ny_partial_damage_no_termination_continued_tenancy() {
        let r = check(&base(Regime::NewYork, DamageExtent::Partial));
        assert!(!r.termination_permitted);
        assert!(r.partial_rent_abatement_available);
    }

    #[test]
    fn ny_tenant_must_affirmatively_elect_surrender_note() {
        let r = check(&base(Regime::NewYork, DamageExtent::TotallyDestroyed));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("affirmatively elect to surrender")));
    }

    #[test]
    fn nj_totally_destroyed_termination_under_46_8_6() {
        let r = check(&base(Regime::NewJersey, DamageExtent::TotallyDestroyed));
        assert!(r.termination_permitted);
        assert!(r.notes.iter().any(|n| n.contains("46:8-6")));
    }

    #[test]
    fn nj_partial_destruction_choice_terminate_or_proportional_rent() {
        let r = check(&base(Regime::NewJersey, DamageExtent::GreaterPart));
        assert!(r.termination_permitted);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("46:8-7") && n.contains("PROPORTIONALLY REDUCED")));
    }

    #[test]
    fn nj_minor_partial_damage_continued_tenancy_with_reduction() {
        let r = check(&base(Regime::NewJersey, DamageExtent::Partial));
        assert!(!r.termination_permitted);
        assert!(r.partial_rent_abatement_available);
    }

    #[test]
    fn default_total_destruction_terminates_under_impossibility() {
        let r = check(&base(Regime::Default, DamageExtent::TotallyDestroyed));
        assert!(r.termination_permitted);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Restatement (Second) of Contracts § 261")));
    }

    #[test]
    fn default_partial_no_termination_with_abatement() {
        let r = check(&base(Regime::Default, DamageExtent::Partial));
        assert!(!r.termination_permitted);
        assert!(r.partial_rent_abatement_available);
    }

    #[test]
    fn tenant_fault_blocks_termination_all_regimes() {
        for regime in [
            Regime::California,
            Regime::Texas,
            Regime::NewYork,
            Regime::NewJersey,
            Regime::Default,
        ] {
            let mut i = base(regime, DamageExtent::TotallyDestroyed);
            i.tenant_fault = true;
            let r = check(&i);
            assert!(
                !r.termination_permitted,
                "regime {:?} should block termination on tenant fault",
                regime
            );
            assert!(r.notes.iter().any(|n| n.contains("tenant fault defeats")));
        }
    }

    #[test]
    fn citation_california_pins_three_subsections() {
        let r = check(&base(Regime::California, DamageExtent::TotallyDestroyed));
        assert!(r.citation.contains("§§ 1932(2), 1933(4)"));
        assert!(r.citation.contains("1950.5(g)"));
    }

    #[test]
    fn citation_texas_pins_92_054_and_92_052() {
        let r = check(&base(Regime::Texas, DamageExtent::TotallyDestroyed));
        assert!(r.citation.contains("§§ 92.054(a)/(b)/(c)"));
        assert!(r.citation.contains("92.052(b)"));
    }

    #[test]
    fn citation_newyork_pins_section_227() {
        let r = check(&base(Regime::NewYork, DamageExtent::TotallyDestroyed));
        assert!(r.citation.contains("§ 227"));
    }

    #[test]
    fn citation_newjersey_pins_three_sections() {
        let r = check(&base(Regime::NewJersey, DamageExtent::TotallyDestroyed));
        assert!(r.citation.contains("46:8-6"));
        assert!(r.citation.contains("46:8-7"));
        assert!(r.citation.contains("46:8-8"));
    }

    #[test]
    fn citation_default_pins_restatement() {
        let r = check(&base(Regime::Default, DamageExtent::TotallyDestroyed));
        assert!(r
            .citation
            .contains("Restatement (Second) of Contracts § 261"));
    }

    #[test]
    fn texas_uniquely_requires_written_notice_invariant() {
        let mut i = base(Regime::Texas, DamageExtent::TotallyDestroyed);
        i.written_notice_given = false;
        let r_tx = check(&i);
        assert!(!r_tx.termination_permitted);
        for regime in [
            Regime::California,
            Regime::NewYork,
            Regime::NewJersey,
            Regime::Default,
        ] {
            let mut i2 = base(regime, DamageExtent::TotallyDestroyed);
            i2.written_notice_given = false;
            let r = check(&i2);
            assert!(
                r.termination_permitted,
                "regime {:?} does not require written notice formally",
                regime
            );
        }
    }

    #[test]
    fn nj_proportional_rent_reduction_distinct_from_ca() {
        let r_nj = check(&base(Regime::NewJersey, DamageExtent::Partial));
        let r_ca = check(&base(Regime::California, DamageExtent::Partial));
        assert!(r_nj.partial_rent_abatement_available);
        assert!(r_ca.partial_rent_abatement_available);
        let r_nj_greater = check(&base(Regime::NewJersey, DamageExtent::GreaterPart));
        let r_ca_greater = check(&base(Regime::California, DamageExtent::GreaterPart));
        assert!(r_nj_greater.partial_rent_abatement_available);
        assert!(!r_ca_greater.partial_rent_abatement_available);
    }
}
