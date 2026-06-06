//! IRC §704(d) — Partner basis limitation on partnership loss deduction.
//!
//! Completes the partner loss-limitation framework with `section_465`
//! (at-risk) and `section_469` (passive activity losses). Sequential
//! application order:
//!
//! 1. **§704(d)** — outside basis limit (THIS module)
//! 2. **§465** — at-risk amount limit
//! 3. **§469** — passive activity loss limit
//! 4. **§461(l)** — excess business loss limit (TCJA addition)
//!
//! Each limit applies to the loss surviving the prior limit. A loss may
//! be allowed under §704(d) but suspended under §465 (e.g., partner has
//! basis from nonrecourse liabilities but no economic at-risk amount).
//!
//! **§704(d)(1) general rule**: a partner's distributive share of
//! partnership loss is allowed only to the extent of the partner's
//! adjusted basis in the partnership interest (outside basis) at the
//! end of the partnership year in which the loss occurred. Excess
//! losses carry forward indefinitely until the partner has sufficient
//! basis in a subsequent year.
//!
//! **Outside basis computation** (for the partnership year):
//!
//! ```text
//!   Beginning basis
//!   + Capital contributions during the year
//!   + Allocated share of partnership income (and tax-exempt income)
//!   + Increases in share of partnership liabilities under §752
//!     (recourse + nonrecourse)
//!   - Decreases in share of partnership liabilities
//!   - Distributions received (money + adjusted basis of property)
//!   - Allocated share of partnership losses (limited by basis itself)
//! ```
//!
//! **§752 liability allocation** (caller pre-computes):
//!
//!   - **Recourse**: partner bears economic risk of loss (EROL) — full
//!     amount of liability increases basis to the partner who bears
//!     the EROL.
//!   - **Nonrecourse**: no partner bears EROL — allocated by share of
//!     equity in the securing property, then minimum-gain shares.
//!
//! Caller provides the §752 increase/decrease amounts already
//! computed; this module just applies them.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section704dInput {
    pub beginning_outside_basis: Decimal,
    pub capital_contributions_this_year: Decimal,
    pub share_of_partnership_income: Decimal,
    pub share_of_recourse_liabilities_increase: Decimal,
    pub share_of_nonrecourse_liabilities_increase: Decimal,
    pub share_of_recourse_liabilities_decrease: Decimal,
    pub share_of_nonrecourse_liabilities_decrease: Decimal,
    pub distributions_received: Decimal,
    /// Allocated partnership loss for the year (positive number — the
    /// magnitude of the loss before §704(d) limitation).
    pub allocated_partnership_loss: Decimal,
    /// Prior-year losses suspended under §704(d) and carrying forward.
    pub prior_year_suspended_loss: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section704dResult {
    /// Outside basis after additions and distributions but BEFORE
    /// applying the loss limitation.
    pub outside_basis_before_loss: Decimal,
    /// Loss deduction allowed under §704(d) this year.
    pub allowed_loss_deduction: Decimal,
    /// Loss suspended and carried forward to next year.
    pub suspended_loss_carryforward: Decimal,
    /// Outside basis after the allowed loss reduces it. Never negative.
    pub outside_basis_after_loss: Decimal,
    /// True if §704(d) limit binds (loss exceeded basis).
    pub basis_limit_binding: bool,
    pub note: String,
}

pub fn compute(input: &Section704dInput) -> Section704dResult {
    // Step 1: Compute outside basis at year-end before loss.
    let outside_basis_before_loss = input.beginning_outside_basis
        + input.capital_contributions_this_year
        + input.share_of_partnership_income
        + input.share_of_recourse_liabilities_increase
        + input.share_of_nonrecourse_liabilities_increase
        - input.share_of_recourse_liabilities_decrease
        - input.share_of_nonrecourse_liabilities_decrease
        - input.distributions_received;

    // Defensive: basis cannot be negative even before loss (distributions
    // in excess of basis are treated as gain under §731(a)(1); module
    // clamps at zero and flags via note).
    let basis_for_loss_absorption = outside_basis_before_loss.max(Decimal::ZERO);

    // Step 2: Total potential loss = current + prior carryforward.
    let total_potential_loss = input.allocated_partnership_loss + input.prior_year_suspended_loss;

    // Step 3: Allowed loss = min(total potential, basis).
    let allowed = total_potential_loss.min(basis_for_loss_absorption);
    let suspended = (total_potential_loss - allowed).max(Decimal::ZERO);
    let basis_after = (basis_for_loss_absorption - allowed).max(Decimal::ZERO);
    let limit_binds = total_potential_loss > basis_for_loss_absorption;

    let note = if outside_basis_before_loss < Decimal::ZERO {
        format!(
            "§731(a)(1): distributions exceeded basis by ${} — gain recognized; §704(d) allows ${} loss; ${} suspended",
            (-outside_basis_before_loss).round_dp(2),
            allowed.round_dp(2),
            suspended.round_dp(2),
        )
    } else if limit_binds {
        format!(
            "§704(d) limit BINDS: outside basis ${} < total potential loss ${}; ${} allowed; ${} suspended for future basis",
            outside_basis_before_loss.round_dp(2),
            total_potential_loss.round_dp(2),
            allowed.round_dp(2),
            suspended.round_dp(2),
        )
    } else {
        format!(
            "§704(d) limit satisfied: outside basis ${} ≥ loss ${}; full loss allowed; ${} basis remaining (subject to §465 + §469 + §461(l) downstream)",
            outside_basis_before_loss.round_dp(2),
            total_potential_loss.round_dp(2),
            basis_after.round_dp(2),
        )
    };

    Section704dResult {
        outside_basis_before_loss,
        allowed_loss_deduction: allowed,
        suspended_loss_carryforward: suspended,
        outside_basis_after_loss: basis_after,
        basis_limit_binding: limit_binds,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section704dInput {
        Section704dInput {
            beginning_outside_basis: dec!(50_000),
            capital_contributions_this_year: Decimal::ZERO,
            share_of_partnership_income: Decimal::ZERO,
            share_of_recourse_liabilities_increase: Decimal::ZERO,
            share_of_nonrecourse_liabilities_increase: Decimal::ZERO,
            share_of_recourse_liabilities_decrease: Decimal::ZERO,
            share_of_nonrecourse_liabilities_decrease: Decimal::ZERO,
            distributions_received: Decimal::ZERO,
            allocated_partnership_loss: dec!(30_000),
            prior_year_suspended_loss: Decimal::ZERO,
        }
    }

    #[test]
    fn loss_within_basis_full_allowance() {
        // $50k basis - $30k loss → full $30k allowed, $20k basis remains.
        let r = compute(&base());
        assert_eq!(r.outside_basis_before_loss, dec!(50_000));
        assert_eq!(r.allowed_loss_deduction, dec!(30_000));
        assert_eq!(r.suspended_loss_carryforward, Decimal::ZERO);
        assert_eq!(r.outside_basis_after_loss, dec!(20_000));
        assert!(!r.basis_limit_binding);
    }

    #[test]
    fn loss_exceeds_basis_partial_allowance() {
        // $50k basis, $80k loss → $50k allowed, $30k suspended.
        let mut i = base();
        i.allocated_partnership_loss = dec!(80_000);
        let r = compute(&i);
        assert_eq!(r.allowed_loss_deduction, dec!(50_000));
        assert_eq!(r.suspended_loss_carryforward, dec!(30_000));
        assert_eq!(r.outside_basis_after_loss, Decimal::ZERO);
        assert!(r.basis_limit_binding);
    }

    #[test]
    fn prior_carryforward_combined_with_current_loss() {
        // $50k basis, $20k current + $20k prior = $40k total → $40k
        // allowed, $0 suspended, $10k basis remains.
        let mut i = base();
        i.allocated_partnership_loss = dec!(20_000);
        i.prior_year_suspended_loss = dec!(20_000);
        let r = compute(&i);
        assert_eq!(r.allowed_loss_deduction, dec!(40_000));
        assert_eq!(r.outside_basis_after_loss, dec!(10_000));
    }

    #[test]
    fn capital_contributions_increase_basis() {
        // $50k beginning + $30k contributions = $80k basis. Loss $30k →
        // full allowance, $50k remains.
        let mut i = base();
        i.capital_contributions_this_year = dec!(30_000);
        let r = compute(&i);
        assert_eq!(r.outside_basis_before_loss, dec!(80_000));
        assert_eq!(r.outside_basis_after_loss, dec!(50_000));
    }

    #[test]
    fn share_of_partnership_income_increases_basis() {
        // $50k + $20k allocated income = $70k. Loss $30k → $40k remains.
        let mut i = base();
        i.share_of_partnership_income = dec!(20_000);
        let r = compute(&i);
        assert_eq!(r.outside_basis_before_loss, dec!(70_000));
        assert_eq!(r.outside_basis_after_loss, dec!(40_000));
    }

    #[test]
    fn recourse_liability_increase_under_752() {
        // $50k + $100k recourse liability allocation = $150k basis.
        // Loss $30k → full allowance, $120k remains.
        let mut i = base();
        i.share_of_recourse_liabilities_increase = dec!(100_000);
        let r = compute(&i);
        assert_eq!(r.outside_basis_before_loss, dec!(150_000));
    }

    #[test]
    fn nonrecourse_liability_increase_also_in_basis() {
        // Nonrecourse liabilities also increase outside basis (distinct
        // from at-risk for §465 — that's why §704(d) basis can exceed
        // at-risk amount).
        let mut i = base();
        i.share_of_nonrecourse_liabilities_increase = dec!(100_000);
        let r = compute(&i);
        assert_eq!(r.outside_basis_before_loss, dec!(150_000));
    }

    #[test]
    fn liability_decrease_reduces_basis() {
        // Decrease in share of partnership liabilities (e.g., partner
        // exits, liability paid down) reduces outside basis under §752.
        let mut i = base();
        i.share_of_recourse_liabilities_decrease = dec!(20_000);
        let r = compute(&i);
        assert_eq!(r.outside_basis_before_loss, dec!(30_000));
    }

    #[test]
    fn distributions_reduce_basis() {
        // Distributions reduce outside basis dollar-for-dollar.
        let mut i = base();
        i.distributions_received = dec!(30_000);
        let r = compute(&i);
        assert_eq!(r.outside_basis_before_loss, dec!(20_000));
        // Loss $30k > $20k basis → only $20k allowed, $10k suspended.
        assert_eq!(r.allowed_loss_deduction, dec!(20_000));
        assert_eq!(r.suspended_loss_carryforward, dec!(10_000));
    }

    #[test]
    fn distributions_exceeding_basis_trigger_731_gain_note() {
        // $50k basis - $80k distributions = -$30k. Defensive clamp to 0
        // for loss absorption; note flags §731(a)(1) gain recognition
        // (caller responsible for actual gain reporting).
        let mut i = base();
        i.distributions_received = dec!(80_000);
        let r = compute(&i);
        assert_eq!(r.outside_basis_before_loss, dec!(-30_000));
        // Basis for loss absorption clamped to 0 → no loss allowed.
        assert_eq!(r.allowed_loss_deduction, Decimal::ZERO);
        assert_eq!(r.suspended_loss_carryforward, dec!(30_000));
        assert!(r.note.contains("§731(a)(1)"));
    }

    #[test]
    fn zero_basis_no_loss_allowed_all_suspended() {
        // Beginning basis $0 → no loss can be absorbed.
        let mut i = base();
        i.beginning_outside_basis = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.allowed_loss_deduction, Decimal::ZERO);
        assert_eq!(r.suspended_loss_carryforward, dec!(30_000));
    }

    #[test]
    fn zero_loss_no_op_basis_unchanged() {
        let mut i = base();
        i.allocated_partnership_loss = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.allowed_loss_deduction, Decimal::ZERO);
        assert_eq!(r.outside_basis_after_loss, dec!(50_000));
    }

    #[test]
    fn complex_combination_all_basis_components() {
        // Demonstrates the full outside-basis formula:
        // $50k beginning + $30k contributions + $10k income +
        // $40k recourse liability + $20k nonrecourse - $5k liability
        // decrease - $25k distributions = $120k basis.
        // Loss $80k → full allowance, $40k remains.
        let mut i = base();
        i.capital_contributions_this_year = dec!(30_000);
        i.share_of_partnership_income = dec!(10_000);
        i.share_of_recourse_liabilities_increase = dec!(40_000);
        i.share_of_nonrecourse_liabilities_increase = dec!(20_000);
        i.share_of_recourse_liabilities_decrease = dec!(5_000);
        i.distributions_received = dec!(25_000);
        i.allocated_partnership_loss = dec!(80_000);
        let r = compute(&i);
        assert_eq!(r.outside_basis_before_loss, dec!(120_000));
        assert_eq!(r.allowed_loss_deduction, dec!(80_000));
        assert_eq!(r.outside_basis_after_loss, dec!(40_000));
    }

    #[test]
    fn basis_exact_match_no_remaining_no_suspension() {
        // Basis = loss exactly. Full allowance, $0 remaining, $0 suspended.
        let mut i = base();
        i.allocated_partnership_loss = dec!(50_000);
        let r = compute(&i);
        assert_eq!(r.allowed_loss_deduction, dec!(50_000));
        assert_eq!(r.outside_basis_after_loss, Decimal::ZERO);
        assert!(!r.basis_limit_binding); // = not >
    }

    #[test]
    fn nonrecourse_basis_exceeds_at_risk_amount() {
        // Distinction from §465: nonrecourse liabilities INCREASE outside
        // basis (§704(d)) but NOT at-risk (§465). Loss may pass §704(d)
        // but fail §465. Pinned as conceptual regression target.
        let mut i = base();
        i.beginning_outside_basis = dec!(10_000);
        i.share_of_nonrecourse_liabilities_increase = dec!(100_000);
        i.allocated_partnership_loss = dec!(50_000);
        let r = compute(&i);
        // §704(d): basis = $110k, full $50k allowed.
        assert_eq!(r.outside_basis_before_loss, dec!(110_000));
        assert_eq!(r.allowed_loss_deduction, dec!(50_000));
        // Downstream §465 would NOT include the $100k nonrecourse →
        // §465 might disallow $40k of this. Caller applies sequentially.
    }

    #[test]
    fn very_large_basis_no_precision_loss() {
        // $1B basis with $5B liability allocation. Loss $3B → full
        // allowance.
        let mut i = base();
        i.beginning_outside_basis = dec!(1_000_000_000);
        i.share_of_recourse_liabilities_increase = dec!(5_000_000_000);
        i.allocated_partnership_loss = dec!(3_000_000_000);
        let r = compute(&i);
        assert_eq!(r.outside_basis_before_loss, dec!(6_000_000_000));
        assert_eq!(r.allowed_loss_deduction, dec!(3_000_000_000));
    }

    #[test]
    fn note_describes_binding_path() {
        let mut i = base();
        i.allocated_partnership_loss = dec!(80_000);
        let r = compute(&i);
        assert!(r.note.contains("§704(d) limit BINDS"));
    }

    #[test]
    fn note_describes_satisfied_path_mentions_downstream() {
        let r = compute(&base());
        assert!(r.note.contains("§704(d) limit satisfied"));
        assert!(r.note.contains("§465") && r.note.contains("§469"));
    }

    #[test]
    fn outside_basis_after_loss_never_negative() {
        // Defensive: any input combination should never produce
        // negative outside basis after loss reporting.
        let mut i = base();
        i.beginning_outside_basis = dec!(1_000);
        i.allocated_partnership_loss = dec!(1_000_000);
        let r = compute(&i);
        assert!(r.outside_basis_after_loss >= Decimal::ZERO);
    }

    #[test]
    fn prior_carryforward_alone_absorbed_into_current_basis() {
        // No current loss but $30k prior carryforward. $50k basis →
        // full $30k absorbed.
        let mut i = base();
        i.allocated_partnership_loss = Decimal::ZERO;
        i.prior_year_suspended_loss = dec!(30_000);
        let r = compute(&i);
        assert_eq!(r.allowed_loss_deduction, dec!(30_000));
        assert_eq!(r.outside_basis_after_loss, dec!(20_000));
    }

    #[test]
    fn prior_carryforward_partial_absorption_with_current() {
        // Current $30k + prior $50k = $80k total loss. Basis $50k →
        // $50k allowed, $30k suspended.
        let mut i = base();
        i.prior_year_suspended_loss = dec!(50_000);
        let r = compute(&i);
        assert_eq!(r.allowed_loss_deduction, dec!(50_000));
        assert_eq!(r.suspended_loss_carryforward, dec!(30_000));
    }
}
