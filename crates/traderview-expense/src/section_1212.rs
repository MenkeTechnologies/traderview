//! IRC §1212(b) capital loss carryover.
//!
//! Active traders accumulate net losses that exceed the $3,000 annual
//! deduction against ordinary income ($1,500 MFS). The excess **carries
//! forward indefinitely** under §1212(b)(1), retaining its short-term
//! or long-term character. Per §1212(b)(2), when computing the next-
//! year carryover, the amount allowed against ordinary income is
//! **treated as absorbing short-term loss first**, then long-term.
//!
//! This module implements the IRS Capital Loss Carryover Worksheet
//! (Pub 550) precisely:
//!
//!   1. Combine prior-year ST / LT carryovers into the current year's
//!      ST / LT losses.
//!   2. Net within character (gain - loss). If there's both a net loss
//!      in one character and a net gain in the other, cross-absorb.
//!   3. Combined net = ST + LT.
//!   4. If combined net is a loss, deductible = min(|loss|, $3,000 /
//!      $1,500 MFS). Apply against ST first, then LT (§1212(b)(2)).
//!   5. ST carryover = net ST loss − ST absorbed.
//!      LT carryover = net LT loss − LT absorbed.
//!
//! Pure compute. The DB ledger persistence lives in 0034 + a small DAL.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    HeadOfHousehold,
    MarriedFilingJointly,
    MarriedFilingSeparately,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarryoverInput {
    /// This year's realized short-term gains (positive).
    pub st_gains: Decimal,
    /// This year's realized short-term losses (positive).
    pub st_losses: Decimal,
    pub lt_gains: Decimal,
    pub lt_losses: Decimal,
    /// Prior-year ST carryover (positive). Treated as additional ST
    /// loss in the current year per §1212(b)(1)(A).
    pub prior_st_carryover: Decimal,
    pub prior_lt_carryover: Decimal,
    pub filing_status: FilingStatus,
    pub tax_year: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CarryoverResult {
    pub tax_year: i32,
    /// Net ST gain after combining prior carryover. Zero if losses won.
    pub net_st_gain: Decimal,
    /// Net ST loss after combining prior carryover. Zero if gains won.
    pub net_st_loss: Decimal,
    pub net_lt_gain: Decimal,
    pub net_lt_loss: Decimal,
    /// Schedule D line 16 — combined ST + LT. Negative means loss.
    pub combined_net_gain_loss: Decimal,
    /// Schedule D line 21 — allowed against ordinary income this year.
    pub deductible_against_ordinary: Decimal,
    /// The $3,000 / $1,500-MFS cap that limited the deduction.
    pub deduction_cap: Decimal,
    /// ST loss absorbed by the ordinary-income deduction (§1212(b)(2)).
    pub st_absorbed_by_deduction: Decimal,
    pub lt_absorbed_by_deduction: Decimal,
    /// Carryovers to next tax year.
    pub st_carryover_next_year: Decimal,
    pub lt_carryover_next_year: Decimal,
    pub note: String,
}

fn deduction_cap(fs: FilingStatus) -> Decimal {
    match fs {
        FilingStatus::MarriedFilingSeparately => Decimal::from_str("1500").unwrap(),
        _ => Decimal::from_str("3000").unwrap(),
    }
}

pub fn compute(input: &CarryoverInput) -> CarryoverResult {
    let mut r = CarryoverResult {
        tax_year: input.tax_year,
        deduction_cap: deduction_cap(input.filing_status),
        ..CarryoverResult::default()
    };

    // Step 1: prior-year carryovers stack onto this year's losses.
    let st_losses_total = input.st_losses + input.prior_st_carryover;
    let lt_losses_total = input.lt_losses + input.prior_lt_carryover;

    // Step 2: net within character.
    let st_net = input.st_gains - st_losses_total;
    let lt_net = input.lt_gains - lt_losses_total;

    let mut net_st_gain = st_net.max(Decimal::ZERO);
    let mut net_st_loss = (-st_net).max(Decimal::ZERO);
    let mut net_lt_gain = lt_net.max(Decimal::ZERO);
    let mut net_lt_loss = (-lt_net).max(Decimal::ZERO);

    // Step 3: cross-absorption (ST loss vs LT gain, LT loss vs ST gain).
    if net_st_loss > Decimal::ZERO && net_lt_gain > Decimal::ZERO {
        let absorbed = net_st_loss.min(net_lt_gain);
        net_st_loss -= absorbed;
        net_lt_gain -= absorbed;
    }
    if net_lt_loss > Decimal::ZERO && net_st_gain > Decimal::ZERO {
        let absorbed = net_lt_loss.min(net_st_gain);
        net_lt_loss -= absorbed;
        net_st_gain -= absorbed;
    }
    r.net_st_gain = net_st_gain;
    r.net_st_loss = net_st_loss;
    r.net_lt_gain = net_lt_gain;
    r.net_lt_loss = net_lt_loss;

    // Step 4: combined.
    let total_gain = net_st_gain + net_lt_gain;
    let total_loss = net_st_loss + net_lt_loss;
    r.combined_net_gain_loss = total_gain - total_loss;

    if r.combined_net_gain_loss >= Decimal::ZERO {
        // Net gain — carryovers fully consumed by absorption above.
        r.note = if r.combined_net_gain_loss > Decimal::ZERO {
            format!("net gain ${} — all carryovers absorbed", r.combined_net_gain_loss)
        } else {
            "exact wash — no carryover, no deduction".into()
        };
        return r;
    }

    // Step 5: deductible against ordinary income, capped.
    let loss_magnitude = -r.combined_net_gain_loss;
    r.deductible_against_ordinary = loss_magnitude.min(r.deduction_cap);

    // §1212(b)(2): ST loss absorbed first by the deduction.
    r.st_absorbed_by_deduction = net_st_loss.min(r.deductible_against_ordinary);
    r.lt_absorbed_by_deduction = r.deductible_against_ordinary - r.st_absorbed_by_deduction;

    r.st_carryover_next_year = net_st_loss - r.st_absorbed_by_deduction;
    r.lt_carryover_next_year = net_lt_loss - r.lt_absorbed_by_deduction;
    r.note = format!(
        "§1212(b): deducted ${} against ordinary, carrying ${} ST + ${} LT to {}",
        r.deductible_against_ordinary,
        r.st_carryover_next_year,
        r.lt_carryover_next_year,
        input.tax_year + 1
    );
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> CarryoverInput {
        CarryoverInput {
            st_gains: Decimal::ZERO,
            st_losses: Decimal::ZERO,
            lt_gains: Decimal::ZERO,
            lt_losses: Decimal::ZERO,
            prior_st_carryover: Decimal::ZERO,
            prior_lt_carryover: Decimal::ZERO,
            filing_status: FilingStatus::Single,
            tax_year: 2024,
        }
    }

    #[test]
    fn pure_st_loss_deducts_3k_carries_rest_as_st() {
        let mut i = base();
        i.st_losses = dec!(10000);
        let r = compute(&i);
        assert_eq!(r.deductible_against_ordinary, dec!(3000));
        assert_eq!(r.st_carryover_next_year, dec!(7000));
        assert_eq!(r.lt_carryover_next_year, Decimal::ZERO);
        assert_eq!(r.st_absorbed_by_deduction, dec!(3000));
    }

    #[test]
    fn pure_lt_loss_deducts_3k_carries_rest_as_lt() {
        let mut i = base();
        i.lt_losses = dec!(10000);
        let r = compute(&i);
        assert_eq!(r.deductible_against_ordinary, dec!(3000));
        assert_eq!(r.st_carryover_next_year, Decimal::ZERO);
        assert_eq!(r.lt_carryover_next_year, dec!(7000));
        assert_eq!(r.lt_absorbed_by_deduction, dec!(3000));
    }

    #[test]
    fn st_loss_absorbed_first_when_both_st_and_lt_losses_present() {
        // ST 5k loss + LT 5k loss, single → $3k deduction.
        // §1212(b)(2): ST is absorbed first against the $3k.
        let mut i = base();
        i.st_losses = dec!(5000);
        i.lt_losses = dec!(5000);
        let r = compute(&i);
        assert_eq!(r.deductible_against_ordinary, dec!(3000));
        assert_eq!(r.st_absorbed_by_deduction, dec!(3000));
        assert_eq!(r.lt_absorbed_by_deduction, Decimal::ZERO);
        assert_eq!(r.st_carryover_next_year, dec!(2000));
        assert_eq!(r.lt_carryover_next_year, dec!(5000));
    }

    #[test]
    fn st_carryover_below_3k_lets_lt_absorb_remainder() {
        // ST 1k + LT 10k loss. $3k cap absorbs all $1k ST + $2k LT.
        let mut i = base();
        i.st_losses = dec!(1000);
        i.lt_losses = dec!(10000);
        let r = compute(&i);
        assert_eq!(r.deductible_against_ordinary, dec!(3000));
        assert_eq!(r.st_absorbed_by_deduction, dec!(1000));
        assert_eq!(r.lt_absorbed_by_deduction, dec!(2000));
        assert_eq!(r.st_carryover_next_year, Decimal::ZERO);
        assert_eq!(r.lt_carryover_next_year, dec!(8000));
    }

    #[test]
    fn mfs_caps_deduction_at_1500() {
        let mut i = base();
        i.filing_status = FilingStatus::MarriedFilingSeparately;
        i.st_losses = dec!(10000);
        let r = compute(&i);
        assert_eq!(r.deduction_cap, dec!(1500));
        assert_eq!(r.deductible_against_ordinary, dec!(1500));
        assert_eq!(r.st_carryover_next_year, dec!(8500));
    }

    #[test]
    fn prior_st_carryover_absorbs_current_st_gain_first() {
        // $5k prior ST carryover + $3k current ST gain = -$2k net ST.
        let mut i = base();
        i.prior_st_carryover = dec!(5000);
        i.st_gains = dec!(3000);
        let r = compute(&i);
        assert_eq!(r.net_st_loss, dec!(2000));
        assert_eq!(r.deductible_against_ordinary, dec!(2000));
        assert_eq!(r.st_carryover_next_year, Decimal::ZERO);
    }

    #[test]
    fn st_loss_cross_absorbs_lt_gain_before_deduction() {
        // ST loss $10k, LT gain $4k.
        // Cross-absorption: $4k of ST loss eaten by LT gain.
        // Net ST loss = $6k. Deduct $3k, carry $3k ST.
        let mut i = base();
        i.st_losses = dec!(10000);
        i.lt_gains = dec!(4000);
        let r = compute(&i);
        assert_eq!(r.net_st_loss, dec!(6000));
        assert_eq!(r.net_lt_gain, Decimal::ZERO);
        assert_eq!(r.deductible_against_ordinary, dec!(3000));
        assert_eq!(r.st_carryover_next_year, dec!(3000));
    }

    #[test]
    fn lt_loss_cross_absorbs_st_gain_before_deduction() {
        let mut i = base();
        i.lt_losses = dec!(10000);
        i.st_gains = dec!(4000);
        let r = compute(&i);
        assert_eq!(r.net_lt_loss, dec!(6000));
        assert_eq!(r.net_st_gain, Decimal::ZERO);
        assert_eq!(r.lt_carryover_next_year, dec!(3000));
    }

    #[test]
    fn loss_exactly_3k_no_carryover() {
        let mut i = base();
        i.st_losses = dec!(3000);
        let r = compute(&i);
        assert_eq!(r.deductible_against_ordinary, dec!(3000));
        assert_eq!(r.st_carryover_next_year, Decimal::ZERO);
        assert_eq!(r.lt_carryover_next_year, Decimal::ZERO);
    }

    #[test]
    fn net_gain_clears_carryovers_no_deduction() {
        let mut i = base();
        i.st_gains = dec!(20000);
        i.prior_lt_carryover = dec!(5000);
        let r = compute(&i);
        assert!(r.combined_net_gain_loss > Decimal::ZERO);
        assert_eq!(r.deductible_against_ordinary, Decimal::ZERO);
        assert_eq!(r.st_carryover_next_year, Decimal::ZERO);
        assert_eq!(r.lt_carryover_next_year, Decimal::ZERO);
    }

    #[test]
    fn exact_wash_returns_zero_no_panic() {
        let mut i = base();
        i.st_gains = dec!(5000);
        i.st_losses = dec!(5000);
        let r = compute(&i);
        assert_eq!(r.combined_net_gain_loss, Decimal::ZERO);
        assert_eq!(r.deductible_against_ordinary, Decimal::ZERO);
    }

    #[test]
    fn multi_year_chain_st_character_preserved() {
        // Year 1: $10k ST loss → deduct $3k, carry $7k ST.
        // Year 2: $0 trades → use $3k against ordinary, carry $4k ST.
        // Year 3: $0 trades → use $3k, carry $1k ST.
        // Year 4: $0 trades → use $1k, fully exhausted.
        let mut chain_st = dec!(10000);
        let mut year = 2024;
        let cap = dec!(3000);
        for expected_carry in [dec!(7000), dec!(4000), dec!(1000), Decimal::ZERO] {
            let r = compute(&CarryoverInput {
                st_gains: Decimal::ZERO,
                st_losses: if year == 2024 { dec!(10000) } else { Decimal::ZERO },
                lt_gains: Decimal::ZERO,
                lt_losses: Decimal::ZERO,
                prior_st_carryover: if year == 2024 { Decimal::ZERO } else { chain_st },
                prior_lt_carryover: Decimal::ZERO,
                filing_status: FilingStatus::Single,
                tax_year: year,
            });
            assert_eq!(
                r.st_carryover_next_year, expected_carry,
                "year {year}: expected ST carry {expected_carry}, got {}",
                r.st_carryover_next_year
            );
            chain_st = r.st_carryover_next_year;
            year += 1;
            let _ = cap; // silence unused
        }
    }

    #[test]
    fn carryover_to_next_year_indefinite_never_negative() {
        // Stress: huge prior carryover + tiny gain. Carryovers stay >= 0.
        let mut i = base();
        i.prior_st_carryover = dec!(1000000);
        i.prior_lt_carryover = dec!(500000);
        i.st_gains = dec!(1);
        let r = compute(&i);
        assert!(r.st_carryover_next_year >= Decimal::ZERO);
        assert!(r.lt_carryover_next_year >= Decimal::ZERO);
        // ST loss reduced by $1 gain + $3k deduction = $999,997 + (with LT picking up $0)
        assert_eq!(r.st_carryover_next_year, dec!(996999));
        assert_eq!(r.lt_carryover_next_year, dec!(500000));
    }
}
