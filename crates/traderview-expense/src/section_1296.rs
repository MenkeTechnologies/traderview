//! IRC §1296 — Mark-to-market election for marketable PFIC stock.
//!
//! Foreign corporations whose income is 75%+ passive OR whose assets
//! are 50%+ passive are **Passive Foreign Investment Companies**
//! (PFICs) under §1297. The default tax regime for U.S. shareholders
//! under §1291 is punitive — "excess distributions" are taxed at the
//! HIGHEST historical marginal rate plus a deferred-interest charge
//! computed back to the year of acquisition. Most retail traders
//! buying VWO, EWZ, EWJ-class international ETFs trip §1291 without
//! realizing it.
//!
//! §1296 offers an escape valve for **marketable PFIC stock**: elect
//! mark-to-market and report unrealized appreciation as **ordinary**
//! income each year. Gain is recognized at ordinary rates (no LTCG
//! preference), but the punitive interest charge goes away entirely.
//!
//! The non-obvious trap: MTM **losses** are deductible only up to the
//! taxpayer's **unreversed inclusions** — the running cumulative MTM
//! gain previously recognized. A first-year MTM loss with no prior
//! inclusions is **suspended** (not deductible, doesn't carry forward
//! as a future-year deduction, doesn't reduce basis). The suspended
//! amount simply vanishes from a tax perspective until the position
//! generates future gains that get clawed back.
//!
//! Basis adjustments per §1296(b):
//!   * Increased by MTM gain recognized.
//!   * Decreased by MTM loss recognized (the deductible portion).
//!   * Decreased by §1296(c)(2)(B) "unreversed inclusions" recapture
//!     when stock is sold below basis.
//!
//! Unreversed inclusions per §1296(d):
//!   * Increased by MTM gain recognized.
//!   * Decreased by MTM loss recognized.
//!   * Never goes negative.
//!
//! Pure compute. Caller supplies start-of-year basis + FMV at
//! year-end + prior cumulative unreversed inclusions. We compute the
//! ordinary inclusion / loss, suspended loss, new basis, new
//! unreversed inclusions for next year's input.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section1296Input {
    pub tax_year: i32,
    /// Adjusted basis at the START of the year. For year 1 of §1296
    /// election, this is the original cost basis.
    pub adjusted_basis_year_start: Decimal,
    /// Fair market value at the END of the year (the §1296 mark).
    pub fair_market_value_year_end: Decimal,
    /// Cumulative MTM gain recognized in all prior §1296 years on
    /// this same stock, net of any prior §1296 losses that absorbed
    /// it. Year 1 = 0.
    pub prior_unreversed_inclusions: Decimal,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section1296Result {
    pub tax_year: i32,
    /// FMV - basis. Positive = gain; negative = loss.
    pub mtm_unrealized_change: Decimal,
    /// Ordinary income reported on the return this year.
    pub ordinary_income_recognized: Decimal,
    /// Ordinary loss deduction allowed this year (capped at prior
    /// unreversed inclusions).
    pub ordinary_loss_recognized: Decimal,
    /// Loss SUSPENDED — no current deduction, doesn't carry forward,
    /// doesn't reduce basis. Just gone for tax purposes.
    pub suspended_loss: Decimal,
    /// New basis for next year's compute.
    pub adjusted_basis_year_end: Decimal,
    /// New unreversed inclusions total for next year's compute.
    pub unreversed_inclusions_year_end: Decimal,
    pub note: String,
}

pub fn compute(input: &Section1296Input) -> Section1296Result {
    let mut r = Section1296Result {
        tax_year: input.tax_year,
        adjusted_basis_year_end: input.adjusted_basis_year_start,
        unreversed_inclusions_year_end: input.prior_unreversed_inclusions,
        ..Section1296Result::default()
    };

    r.mtm_unrealized_change = input.fair_market_value_year_end - input.adjusted_basis_year_start;

    if r.mtm_unrealized_change > Decimal::ZERO {
        // Year of MTM gain.
        r.ordinary_income_recognized = r.mtm_unrealized_change;
        r.adjusted_basis_year_end = input.adjusted_basis_year_start + r.mtm_unrealized_change;
        r.unreversed_inclusions_year_end =
            input.prior_unreversed_inclusions + r.mtm_unrealized_change;
        r.note = format!(
            "§1296(a)(1) ordinary inclusion of ${} (FMV ${} - basis ${})",
            r.ordinary_income_recognized,
            input.fair_market_value_year_end,
            input.adjusted_basis_year_start,
        );
    } else if r.mtm_unrealized_change < Decimal::ZERO {
        // Year of MTM loss. Deductible only up to prior unreversed
        // inclusions per §1296(a)(2).
        let loss_magnitude = -r.mtm_unrealized_change;
        r.ordinary_loss_recognized = loss_magnitude.min(input.prior_unreversed_inclusions);
        r.suspended_loss = loss_magnitude - r.ordinary_loss_recognized;
        // Basis reduces only by the deductible loss; suspended portion
        // doesn't reduce basis (it's just gone).
        r.adjusted_basis_year_end = input.adjusted_basis_year_start - r.ordinary_loss_recognized;
        r.unreversed_inclusions_year_end =
            input.prior_unreversed_inclusions - r.ordinary_loss_recognized;
        r.note = if r.suspended_loss > Decimal::ZERO {
            format!(
                "§1296(a)(2) ordinary loss ${} (capped at unreversed inclusions ${}); ${} suspended (no deduction, doesn't carry forward)",
                r.ordinary_loss_recognized,
                input.prior_unreversed_inclusions,
                r.suspended_loss,
            )
        } else {
            format!(
                "§1296(a)(2) ordinary loss ${} fully absorbed by ${} of prior unreversed inclusions",
                r.ordinary_loss_recognized, input.prior_unreversed_inclusions,
            )
        };
    } else {
        r.note = "no MTM change — FMV equals basis".into();
    }

    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section1296Input {
        Section1296Input {
            tax_year: 2024,
            adjusted_basis_year_start: dec!(10000),
            fair_market_value_year_end: dec!(12000),
            prior_unreversed_inclusions: Decimal::ZERO,
        }
    }

    #[test]
    fn year_1_gain_recognized_ordinary_basis_steps_up() {
        let r = compute(&base());
        assert_eq!(r.ordinary_income_recognized, dec!(2000));
        assert_eq!(r.ordinary_loss_recognized, Decimal::ZERO);
        assert_eq!(r.adjusted_basis_year_end, dec!(12000));
        assert_eq!(r.unreversed_inclusions_year_end, dec!(2000));
    }

    #[test]
    fn year_1_loss_with_zero_prior_inclusions_fully_suspended() {
        // First-year MTM loss → no deduction, position basis stays the same
        // (because suspended losses don't reduce basis).
        let mut i = base();
        i.fair_market_value_year_end = dec!(8000);
        let r = compute(&i);
        assert_eq!(r.mtm_unrealized_change, dec!(-2000));
        assert_eq!(r.ordinary_loss_recognized, Decimal::ZERO);
        assert_eq!(r.suspended_loss, dec!(2000));
        // Basis stays at original year-start (no deductible loss to subtract).
        assert_eq!(r.adjusted_basis_year_end, dec!(10000));
        assert_eq!(r.unreversed_inclusions_year_end, Decimal::ZERO);
        assert!(r.note.contains("suspended"));
    }

    #[test]
    fn year_2_loss_absorbed_by_prior_inclusions() {
        // Year 2 setup: $2k of prior inclusions from year 1.
        // Year 2 MTM loss of $1k → fully deductible.
        let mut i = base();
        i.tax_year = 2025;
        i.adjusted_basis_year_start = dec!(12000); // year-end basis from year 1
        i.fair_market_value_year_end = dec!(11000);
        i.prior_unreversed_inclusions = dec!(2000);
        let r = compute(&i);
        assert_eq!(r.ordinary_loss_recognized, dec!(1000));
        assert_eq!(r.suspended_loss, Decimal::ZERO);
        assert_eq!(r.adjusted_basis_year_end, dec!(11000));
        assert_eq!(r.unreversed_inclusions_year_end, dec!(1000));
    }

    #[test]
    fn loss_exceeds_unreversed_inclusions_excess_suspended() {
        // $1k prior inclusions, $5k MTM loss → $1k deductible, $4k suspended.
        let mut i = base();
        i.adjusted_basis_year_start = dec!(12000);
        i.fair_market_value_year_end = dec!(7000);
        i.prior_unreversed_inclusions = dec!(1000);
        let r = compute(&i);
        assert_eq!(r.ordinary_loss_recognized, dec!(1000));
        assert_eq!(r.suspended_loss, dec!(4000));
        // Basis reduces ONLY by the deductible portion.
        assert_eq!(r.adjusted_basis_year_end, dec!(11000));
        assert_eq!(r.unreversed_inclusions_year_end, Decimal::ZERO);
    }

    #[test]
    fn no_mtm_change_no_op() {
        let mut i = base();
        i.fair_market_value_year_end = i.adjusted_basis_year_start;
        let r = compute(&i);
        assert_eq!(r.ordinary_income_recognized, Decimal::ZERO);
        assert_eq!(r.ordinary_loss_recognized, Decimal::ZERO);
        assert!(r.note.contains("no MTM change"));
    }

    #[test]
    fn multi_year_chain_gain_loss_gain_basis_evolves_correctly() {
        // Year 1: basis $10k → FMV $14k → +$4k gain, basis $14k, UI $4k.
        let y1 = compute(&Section1296Input {
            tax_year: 2024,
            adjusted_basis_year_start: dec!(10000),
            fair_market_value_year_end: dec!(14000),
            prior_unreversed_inclusions: Decimal::ZERO,
        });
        assert_eq!(y1.adjusted_basis_year_end, dec!(14000));
        assert_eq!(y1.unreversed_inclusions_year_end, dec!(4000));

        // Year 2: basis $14k → FMV $11k → -$3k loss. UI $4k absorbs it.
        let y2 = compute(&Section1296Input {
            tax_year: 2025,
            adjusted_basis_year_start: y1.adjusted_basis_year_end,
            fair_market_value_year_end: dec!(11000),
            prior_unreversed_inclusions: y1.unreversed_inclusions_year_end,
        });
        assert_eq!(y2.ordinary_loss_recognized, dec!(3000));
        assert_eq!(y2.suspended_loss, Decimal::ZERO);
        assert_eq!(y2.adjusted_basis_year_end, dec!(11000));
        assert_eq!(y2.unreversed_inclusions_year_end, dec!(1000));

        // Year 3: basis $11k → FMV $20k → +$9k gain.
        let y3 = compute(&Section1296Input {
            tax_year: 2026,
            adjusted_basis_year_start: y2.adjusted_basis_year_end,
            fair_market_value_year_end: dec!(20000),
            prior_unreversed_inclusions: y2.unreversed_inclusions_year_end,
        });
        assert_eq!(y3.ordinary_income_recognized, dec!(9000));
        assert_eq!(y3.adjusted_basis_year_end, dec!(20000));
        assert_eq!(y3.unreversed_inclusions_year_end, dec!(10000));
    }

    #[test]
    fn first_year_loss_then_gain_chain_preserves_suspension_in_economic_basis() {
        // Year 1: basis $10k → FMV $8k → $2k suspended (no deduction).
        //   Basis stays $10k, UI = 0.
        // Year 2: basis $10k → FMV $13k → +$3k inclusion.
        //   Note: economic gain since acquisition = $3k, NOT $5k. The
        //   $2k suspended in year 1 is lost forever — the only
        //   recovery channel is the basis staying flat, so the $3k
        //   gain reflects (FMV $13k - basis $10k), which captures
        //   only the gain ABOVE the year-1 mark.
        let y1 = compute(&Section1296Input {
            tax_year: 2024,
            adjusted_basis_year_start: dec!(10000),
            fair_market_value_year_end: dec!(8000),
            prior_unreversed_inclusions: Decimal::ZERO,
        });
        assert_eq!(y1.suspended_loss, dec!(2000));
        assert_eq!(y1.adjusted_basis_year_end, dec!(10000));

        let y2 = compute(&Section1296Input {
            tax_year: 2025,
            adjusted_basis_year_start: y1.adjusted_basis_year_end,
            fair_market_value_year_end: dec!(13000),
            prior_unreversed_inclusions: y1.unreversed_inclusions_year_end,
        });
        assert_eq!(y2.ordinary_income_recognized, dec!(3000));
        assert_eq!(y2.unreversed_inclusions_year_end, dec!(3000));
    }

    #[test]
    fn unreversed_inclusions_never_negative() {
        // Pathological: large loss in year 2 with small UI.
        let mut i = base();
        i.adjusted_basis_year_start = dec!(5000);
        i.fair_market_value_year_end = dec!(100);
        i.prior_unreversed_inclusions = dec!(500);
        let r = compute(&i);
        assert!(r.unreversed_inclusions_year_end >= Decimal::ZERO);
        assert_eq!(r.ordinary_loss_recognized, dec!(500));
        assert_eq!(r.suspended_loss, dec!(4400));
    }

    #[test]
    fn gain_exactly_zero_marks_no_inclusion() {
        let mut i = base();
        i.adjusted_basis_year_start = dec!(10000);
        i.fair_market_value_year_end = dec!(10000);
        let r = compute(&i);
        assert_eq!(r.ordinary_income_recognized, Decimal::ZERO);
        assert_eq!(r.ordinary_loss_recognized, Decimal::ZERO);
        assert_eq!(r.unreversed_inclusions_year_end, Decimal::ZERO);
    }

    #[test]
    fn loss_caps_at_full_basis_does_not_create_negative_basis() {
        // Loss bigger than basis — basis floors at zero conceptually,
        // but the deductible loss is still capped at UI. Basis would
        // be 10000 - 1000 = 9000 with $4000 suspended. Verify.
        let mut i = base();
        i.adjusted_basis_year_start = dec!(10000);
        i.fair_market_value_year_end = dec!(5000); // -$5k change
        i.prior_unreversed_inclusions = dec!(1000);
        let r = compute(&i);
        assert_eq!(r.ordinary_loss_recognized, dec!(1000));
        assert_eq!(r.suspended_loss, dec!(4000));
        assert_eq!(r.adjusted_basis_year_end, dec!(9000));
    }

    #[test]
    fn gain_note_describes_inclusion_with_amounts() {
        let r = compute(&base());
        assert!(r.note.contains("$2000"));
        assert!(r.note.contains("$12000")); // FMV
        assert!(r.note.contains("$10000")); // basis
    }

    #[test]
    fn loss_note_distinguishes_full_absorb_vs_partial_suspend() {
        // Full absorb
        let r_full = compute(&Section1296Input {
            tax_year: 2025,
            adjusted_basis_year_start: dec!(12000),
            fair_market_value_year_end: dec!(11000),
            prior_unreversed_inclusions: dec!(2000),
        });
        assert!(r_full.note.contains("fully absorbed"));

        // Partial suspend
        let r_part = compute(&Section1296Input {
            tax_year: 2025,
            adjusted_basis_year_start: dec!(12000),
            fair_market_value_year_end: dec!(7000),
            prior_unreversed_inclusions: dec!(1000),
        });
        assert!(r_part.note.contains("suspended"));
    }
}
