//! IRC §280B — Demolition of structures.
//!
//! Foundational rule for any real estate trader: when a property
//! owner demolishes a structure, the demolition costs AND any
//! loss sustained on the demolition (structure remaining basis −
//! salvage value) are NOT DEDUCTIBLE. Instead they are CAPITALIZED
//! into the basis of the LAND on which the demolished structure
//! was located ([Cornell LII 26 U.S.C. § 280B](https://www.law.cornell.edu/uscode/text/26/280B),
//! [Cornell LII 26 CFR § 1.280B-1](https://www.law.cornell.edu/cfr/text/26/1.280B-1)).
//!
//! **Why this matters**: land is NOT a depreciable asset, so the
//! "deduction" is deferred until the LAND is eventually sold —
//! potentially decades later. Compare: if the structure were simply
//! depreciated to zero and abandoned, the remaining basis would be
//! immediately deductible as an ordinary loss. §280B forces a
//! capital-account treatment that destroys timing.
//!
//! **§280B applies broadly with no exceptions for**:
//!
//! - Safety concerns necessitating demolition
//! - Unanticipated circumstances (fire damage, structural failure
//!   discovered post-purchase)
//! - Economic obsolescence
//! - Local code requirements forcing demolition
//!
//! **Two narrow EXCEPTIONS where the rule does not apply**:
//!
//! 1. **IRS Notice 90-21 casualty exception**: when a structure is
//!    damaged by a casualty (fire, flood, earthquake, storm), the
//!    taxpayer can compute a § 165 casualty loss separately on the
//!    pre-casualty basis before § 280B captures the demolition
//!    costs. The actual demolition expenses still capitalize to
//!    land, but the casualty loss on the structure is deductible.
//!
//! 2. **General Asset Account (GAA) election**: if the structure was
//!    placed in a § 168(i)(4) GAA at the time of acquisition or
//!    later, and the GAA election remains in effect, an
//!    abandonment loss on the demolished structure may be claimed
//!    by terminating the GAA. Treas. Reg. § 1.168(i)-1(e). The
//!    GAA election must have been made AT or BEFORE the demolition
//!    decision — it cannot be made retroactively.
//!
//! **Practical effect**: if no exception applies, the entire
//! demolition cost + structure remaining basis − salvage value
//! increases land basis, only realized as reduced gain (or
//! increased loss) when the land is sold.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section280BInput {
    pub demolition_costs_paid_dollars: i64,
    pub structure_remaining_adjusted_basis_dollars: i64,
    pub structure_salvage_value_dollars: i64,
    pub land_pre_demolition_basis_dollars: i64,
    /// True if the structure was damaged by a casualty (fire, flood,
    /// earthquake, storm) before being demolished — invokes IRS
    /// Notice 90-21 carveout for separate casualty loss.
    pub structure_damaged_by_casualty_before_demolition: bool,
    /// True if the structure was placed in a § 168(i)(4) General
    /// Asset Account (GAA) at or before the demolition decision.
    pub gaa_election_in_effect_at_demolition: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section280BResult {
    pub demolition_deduction_allowed_dollars: i64,
    pub demolition_loss_sustained_dollars: i64,
    pub amount_capitalized_to_land_dollars: i64,
    pub new_land_basis_dollars: i64,
    pub casualty_loss_separately_available_dollars: i64,
    pub gaa_abandonment_loss_available: bool,
    pub gaa_abandonment_loss_dollars: i64,
    pub citation: String,
    pub note: String,
}

pub fn compute(input: &Section280BInput) -> Section280BResult {
    // §280B(1): no deduction allowed for demolition costs OR loss.
    let demolition_deduction = 0;

    // Loss sustained = structure remaining basis − salvage value.
    let demolition_loss = (input.structure_remaining_adjusted_basis_dollars
        - input.structure_salvage_value_dollars)
        .max(0);

    // §280B(2): demolition costs + loss → capitalized to LAND.
    let capitalized = input.demolition_costs_paid_dollars + demolition_loss;
    let new_land_basis = input.land_pre_demolition_basis_dollars + capitalized;

    // IRS Notice 90-21 casualty exception: separately deductible
    // casualty loss on structure (compute on remaining basis).
    let casualty_loss = if input.structure_damaged_by_casualty_before_demolition {
        input
            .structure_remaining_adjusted_basis_dollars
            .saturating_sub(input.structure_salvage_value_dollars)
            .max(0)
    } else {
        0
    };

    // GAA abandonment loss available when GAA election in effect.
    let gaa_available = input.gaa_election_in_effect_at_demolition;
    let gaa_loss = if gaa_available { demolition_loss } else { 0 };

    let mut note_parts: Vec<String> = vec![format!(
        "§280B(1): NO deduction for ${} demolition costs or ${} loss; both ${} capitalized to land basis under §280B(2) → new land basis ${}.",
        input.demolition_costs_paid_dollars,
        demolition_loss,
        capitalized,
        new_land_basis,
    )];
    if input.structure_damaged_by_casualty_before_demolition {
        note_parts.push(format!(
            "IRS Notice 90-21 CASUALTY EXCEPTION: §165 casualty loss ${} separately deductible (computed on pre-casualty basis).",
            casualty_loss,
        ));
    }
    if gaa_available {
        note_parts.push(format!(
            "§ 168(i)(4) GAA election in effect: ${} abandonment loss available via GAA termination (Treas. Reg. § 1.168(i)-1(e)).",
            gaa_loss,
        ));
    }
    if !input.structure_damaged_by_casualty_before_demolition && !gaa_available {
        note_parts.push(
            "No casualty (Notice 90-21) or GAA exception available; entire demolition cost + loss defers until LAND is sold.".to_string(),
        );
    }

    Section280BResult {
        demolition_deduction_allowed_dollars: demolition_deduction,
        demolition_loss_sustained_dollars: demolition_loss,
        amount_capitalized_to_land_dollars: capitalized,
        new_land_basis_dollars: new_land_basis,
        casualty_loss_separately_available_dollars: casualty_loss,
        gaa_abandonment_loss_available: gaa_available,
        gaa_abandonment_loss_dollars: gaa_loss,
        citation:
            "IRC §280B(1) demolition cost / loss not deductible; §280B(2) capitalized to land basis; Treas. Reg. §1.280B-1; IRS Notice 90-21 casualty exception; §168(i)(4) + Treas. Reg. §1.168(i)-1(e) GAA abandonment loss election"
                .to_string(),
        note: note_parts.join(" "),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section280BInput {
        Section280BInput {
            demolition_costs_paid_dollars: 50_000,
            structure_remaining_adjusted_basis_dollars: 200_000,
            structure_salvage_value_dollars: 0,
            land_pre_demolition_basis_dollars: 100_000,
            structure_damaged_by_casualty_before_demolition: false,
            gaa_election_in_effect_at_demolition: false,
        }
    }

    // Standard rule.

    #[test]
    fn standard_demolition_no_deduction() {
        let r = compute(&base());
        assert_eq!(r.demolition_deduction_allowed_dollars, 0);
    }

    #[test]
    fn standard_demolition_loss_sustained() {
        let r = compute(&base());
        assert_eq!(r.demolition_loss_sustained_dollars, 200_000);
    }

    #[test]
    fn standard_demolition_capitalized_to_land() {
        // $50k demo costs + $200k loss = $250k capitalized.
        let r = compute(&base());
        assert_eq!(r.amount_capitalized_to_land_dollars, 250_000);
        assert_eq!(r.new_land_basis_dollars, 350_000);
    }

    #[test]
    fn salvage_value_reduces_loss() {
        // $200k basis − $30k salvage = $170k loss.
        let mut i = base();
        i.structure_salvage_value_dollars = 30_000;
        let r = compute(&i);
        assert_eq!(r.demolition_loss_sustained_dollars, 170_000);
        assert_eq!(r.amount_capitalized_to_land_dollars, 220_000);
    }

    #[test]
    fn salvage_greater_than_basis_loss_zero() {
        // Salvage > basis → no loss (positive recovery would be gain
        // — but module clamps loss at 0).
        let mut i = base();
        i.structure_remaining_adjusted_basis_dollars = 50_000;
        i.structure_salvage_value_dollars = 100_000;
        let r = compute(&i);
        assert_eq!(r.demolition_loss_sustained_dollars, 0);
    }

    #[test]
    fn zero_demolition_costs_only_loss_capitalized() {
        let mut i = base();
        i.demolition_costs_paid_dollars = 0;
        let r = compute(&i);
        assert_eq!(r.amount_capitalized_to_land_dollars, 200_000);
        assert_eq!(r.new_land_basis_dollars, 300_000);
    }

    #[test]
    fn fully_depreciated_structure_no_loss_only_demo_costs() {
        let mut i = base();
        i.structure_remaining_adjusted_basis_dollars = 0;
        let r = compute(&i);
        assert_eq!(r.demolition_loss_sustained_dollars, 0);
        assert_eq!(r.amount_capitalized_to_land_dollars, 50_000);
    }

    // Notice 90-21 casualty exception.

    #[test]
    fn casualty_exception_allows_separate_loss() {
        let mut i = base();
        i.structure_damaged_by_casualty_before_demolition = true;
        let r = compute(&i);
        assert_eq!(r.casualty_loss_separately_available_dollars, 200_000);
        assert!(r.note.contains("CASUALTY EXCEPTION"));
    }

    #[test]
    fn casualty_exception_does_not_change_280b_capitalization() {
        // Notice 90-21 still keeps the actual demolition costs
        // captured by §280B — only the casualty loss is separate.
        let mut i = base();
        i.structure_damaged_by_casualty_before_demolition = true;
        let r = compute(&i);
        assert_eq!(r.amount_capitalized_to_land_dollars, 250_000);
        assert_eq!(r.demolition_deduction_allowed_dollars, 0);
    }

    // GAA election exception.

    #[test]
    fn gaa_election_in_effect_allows_abandonment_loss() {
        let mut i = base();
        i.gaa_election_in_effect_at_demolition = true;
        let r = compute(&i);
        assert!(r.gaa_abandonment_loss_available);
        assert_eq!(r.gaa_abandonment_loss_dollars, 200_000);
        assert!(r.note.contains("GAA election"));
    }

    #[test]
    fn no_gaa_no_abandonment_loss() {
        let r = compute(&base());
        assert!(!r.gaa_abandonment_loss_available);
        assert_eq!(r.gaa_abandonment_loss_dollars, 0);
    }

    // Combined exceptions.

    #[test]
    fn both_casualty_and_gaa_exceptions_apply() {
        let mut i = base();
        i.structure_damaged_by_casualty_before_demolition = true;
        i.gaa_election_in_effect_at_demolition = true;
        let r = compute(&i);
        assert_eq!(r.casualty_loss_separately_available_dollars, 200_000);
        assert!(r.gaa_abandonment_loss_available);
    }

    #[test]
    fn no_exceptions_note_describes_indefinite_deferral() {
        let r = compute(&base());
        assert!(r.note.contains("defers until LAND is sold"));
    }

    // Large-scale / precision.

    #[test]
    fn very_large_demolition_costs_precision_path() {
        let mut i = base();
        i.demolition_costs_paid_dollars = 1_000_000_000;
        i.structure_remaining_adjusted_basis_dollars = 5_000_000_000;
        i.land_pre_demolition_basis_dollars = 10_000_000_000;
        let r = compute(&i);
        assert_eq!(r.amount_capitalized_to_land_dollars, 6_000_000_000);
        assert_eq!(r.new_land_basis_dollars, 16_000_000_000);
    }

    #[test]
    fn zero_all_inputs_no_op() {
        let i = Section280BInput {
            demolition_costs_paid_dollars: 0,
            structure_remaining_adjusted_basis_dollars: 0,
            structure_salvage_value_dollars: 0,
            land_pre_demolition_basis_dollars: 0,
            structure_damaged_by_casualty_before_demolition: false,
            gaa_election_in_effect_at_demolition: false,
        };
        let r = compute(&i);
        assert_eq!(r.amount_capitalized_to_land_dollars, 0);
        assert_eq!(r.new_land_basis_dollars, 0);
    }

    // Notes / citations.

    #[test]
    fn note_mentions_no_deduction_and_capitalization() {
        let r = compute(&base());
        assert!(r.note.contains("§280B(1)"));
        assert!(r.note.contains("§280B(2)"));
        assert!(r.note.contains("NO deduction"));
        assert!(r.note.contains("capitalized to land"));
    }

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&base());
        assert!(r.citation.contains("§280B(1)"));
        assert!(r.citation.contains("§280B(2)"));
        assert!(r.citation.contains("§1.280B-1"));
        assert!(r.citation.contains("Notice 90-21"));
        assert!(r.citation.contains("§168(i)(4)"));
    }

    #[test]
    fn note_mentions_gaa_when_election_in_effect() {
        let mut i = base();
        i.gaa_election_in_effect_at_demolition = true;
        let r = compute(&i);
        assert!(r.note.contains("GAA termination"));
    }

    #[test]
    fn note_does_not_mention_casualty_when_absent() {
        let r = compute(&base());
        assert!(!r.note.contains("CASUALTY EXCEPTION"));
    }
}
